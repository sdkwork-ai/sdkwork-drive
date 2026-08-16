use jsonwebtoken::DecodingKey;
use std::collections::BTreeMap;

pub fn load_jwt_jwks_from_env() -> BTreeMap<String, DecodingKey> {
    if let Ok(raw) = std::env::var("SDKWORK_DRIVE_JWT_JWKS_JSON") {
        let raw = raw.trim();
        if !raw.is_empty() {
            match parse_jwt_jwks_json(raw) {
                Ok(keys) => return keys,
                Err(error) => tracing::warn!(
                    "SDKWORK_DRIVE_JWT_JWKS_JSON is set but could not be parsed: {error}"
                ),
            }
        }
    }

    if let Ok(url) = std::env::var("SDKWORK_DRIVE_JWT_JWKS_URL") {
        let url = url.trim();
        if !url.is_empty() {
            match fetch_jwks_keys_from_url_blocking(url) {
                Ok(keys) => return keys,
                Err(error) => tracing::warn!("failed to fetch JWKS from {url}: {error}"),
            }
        }
    }

    BTreeMap::new()
}

pub async fn fetch_jwks_keys_from_url(url: &str) -> Result<BTreeMap<String, DecodingKey>, String> {
    let response = reqwest::get(url.trim())
        .await
        .map_err(|error| format!("JWKS HTTP request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "JWKS HTTP request returned status {}",
            response.status()
        ));
    }
    let body = response
        .text()
        .await
        .map_err(|error| format!("JWKS HTTP response body could not be read: {error}"))?;
    parse_jwt_jwks_json(&body)
}

pub fn fetch_jwks_keys_from_url_blocking(
    url: &str,
) -> Result<BTreeMap<String, DecodingKey>, String> {
    let response = reqwest::blocking::get(url.trim())
        .map_err(|error| format!("JWKS HTTP request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "JWKS HTTP request returned status {}",
            response.status()
        ));
    }
    let body = response
        .text()
        .map_err(|error| format!("JWKS HTTP response body could not be read: {error}"))?;
    parse_jwt_jwks_json(&body)
}

pub fn parse_jwt_jwks_json(raw: &str) -> Result<BTreeMap<String, DecodingKey>, String> {
    let document = serde_json::from_str::<serde_json::Value>(raw.trim())
        .map_err(|error| format!("invalid JWKS JSON: {error}"))?;
    let keys = document
        .get("keys")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "JWKS document must include a keys array".to_string())?;
    let mut decoded = BTreeMap::new();
    for key in keys {
        let Some(kty) = key.get("kty").and_then(|value| value.as_str()) else {
            continue;
        };
        if kty != "RSA" {
            continue;
        }
        let n = key
            .get("n")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "RSA JWK must include n".to_string())?;
        let e = key
            .get("e")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "RSA JWK must include e".to_string())?;
        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|error| format!("invalid RSA JWK components: {error}"))?;
        let kid = key
            .get("kid")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("default")
            .to_string();
        decoded.insert(kid, decoding_key);
    }
    if decoded.is_empty() {
        return Err("JWKS document did not include any RSA keys".to_string());
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_jwks_document() {
        assert!(matches!(
            parse_jwt_jwks_json(r#"{"keys":[]}"#),
            Err(message) if message.contains("did not include any RSA keys")
        ));
    }
}
