use std::net::IpAddr;

/// Validates webhook URL syntax and host policy before DNS resolution.
pub fn validate_webhook_https_url(address: &str) -> Result<&str, &'static str> {
    let trimmed = address.trim();
    if trimmed.is_empty() || trimmed.len() > 2048 {
        return Err("address must be an https webhook URL");
    }
    if !trimmed.starts_with("https://") {
        return Err("address must be an https webhook URL");
    }

    let authority = trimmed
        .get(8..)
        .and_then(|rest| rest.split(&['/', '?', '#'][..]).next())
        .ok_or("address must be an https webhook URL")?;
    if authority.is_empty() {
        return Err("address must be an https webhook URL");
    }

    let host = extract_webhook_host(authority).ok_or("address must be an https webhook URL")?;
    validate_webhook_host(host)?;
    Ok(trimmed)
}

/// Validates webhook URL syntax, host policy, and resolved IP addresses (SSRF hardening).
pub async fn validate_webhook_https_url_for_dispatch(address: &str) -> Result<String, String> {
    let trimmed = validate_webhook_https_url(address)
        .map_err(str::to_string)?
        .to_string();
    let authority = trimmed
        .get(8..)
        .and_then(|rest| rest.split(&['/', '?', '#'][..]).next())
        .ok_or_else(|| "address must be an https webhook URL".to_string())?;
    let host = extract_webhook_host(authority)
        .ok_or_else(|| "address must be an https webhook URL".to_string())?;
    resolve_and_validate_webhook_host(host).await?;
    Ok(trimmed)
}

async fn resolve_and_validate_webhook_host(host: &str) -> Result<(), String> {
    if host.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    let mut resolved_any = false;
    let lookup = tokio::net::lookup_host((host, 443))
        .await
        .map_err(|error| format!("webhook host DNS resolution failed: {error}"))?;
    for socket_addr in lookup {
        resolved_any = true;
        if is_blocked_webhook_ip(socket_addr.ip()) {
            return Err("address host resolves to a blocked network".to_string());
        }
    }
    if !resolved_any {
        return Err("webhook host did not resolve to any address".to_string());
    }
    Ok(())
}

fn extract_webhook_host(authority: &str) -> Option<&str> {
    if authority.starts_with('[') {
        let end = authority.find(']')?;
        return Some(&authority[1..end]);
    }

    if authority.matches(':').count() == 1 {
        let (host, port) = authority.rsplit_once(':')?;
        if port.chars().all(|ch| ch.is_ascii_digit()) && !port.is_empty() {
            return Some(host);
        }
    }

    Some(authority)
}

fn validate_webhook_host(host: &str) -> Result<(), &'static str> {
    let normalized = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("address must be an https webhook URL");
    }

    if is_blocked_webhook_hostname(&normalized) {
        return Err("address host is not allowed for webhooks");
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_webhook_ip(ip) {
            return Err("address host is not allowed for webhooks");
        }
    }

    Ok(())
}

fn is_blocked_webhook_hostname(host: &str) -> bool {
    matches!(
        host,
        "localhost"
            | "localhost.localdomain"
            | "metadata.google.internal"
            | "metadata"
            | "instance-data"
    ) || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".localdomain")
}

fn is_blocked_webhook_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || is_cgnat_v4(v4)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || is_unique_local_v6(v6)
                || is_link_local_v6(v6)
        }
    }
}

fn is_cgnat_v4(v4: std::net::Ipv4Addr) -> bool {
    let octets = v4.octets();
    octets[0] == 100 && (octets[1] & 0xc0) == 64
}

fn is_unique_local_v6(v6: std::net::Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(v6: std::net::Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_public_https_webhook_urls() {
        assert_eq!(
            validate_webhook_https_url("https://hooks.example.com/drive/events"),
            Ok("https://hooks.example.com/drive/events")
        );
    }

    #[test]
    fn rejects_localhost_webhooks() {
        assert!(validate_webhook_https_url("https://localhost/hook").is_err());
        assert!(validate_webhook_https_url("https://127.0.0.1/hook").is_err());
    }

    #[test]
    fn rejects_private_network_targets() {
        assert!(validate_webhook_https_url("https://10.0.0.5/hook").is_err());
        assert!(validate_webhook_https_url("https://192.168.1.20/hook").is_err());
        assert!(validate_webhook_https_url("https://169.254.169.254/latest/meta-data").is_err());
    }

    #[test]
    fn rejects_non_https_webhooks() {
        assert!(validate_webhook_https_url("http://hooks.example.com/hook").is_err());
    }
}
