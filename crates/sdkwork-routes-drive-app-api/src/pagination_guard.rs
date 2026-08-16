use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::{problem, SdkWorkResultCode};

const FORBIDDEN_PAGINATION_QUERY_KEYS: &[&str] = &[
    "pageSize",
    "pageToken",
    "page_token",
    "limit",
    "page_no",
    "pageNo",
    "per_page",
    "size",
];

pub(crate) async fn reject_legacy_pagination_query(request: Request<Body>, next: Next) -> Response {
    if let Some(forbidden_key) = request.uri().query().and_then(find_forbidden_query_key) {
        return problem(
            StatusCode::BAD_REQUEST,
            "invalid pagination query",
            format!(
                "{forbidden_key} is not a supported pagination query parameter; use page_size and cursor"
            ),
            SdkWorkResultCode::InvalidParameter,
        )
        .into_response();
    }

    next.run(request).await
}

fn find_forbidden_query_key(query: &str) -> Option<&'static str> {
    for pair in query.split('&') {
        let key = pair.split_once('=').map(|(key, _)| key).unwrap_or(pair);
        let key = percent_decode_query_component(key);
        if let Some(forbidden) = FORBIDDEN_PAGINATION_QUERY_KEYS
            .iter()
            .copied()
            .find(|forbidden| key == *forbidden)
        {
            return Some(forbidden);
        }
    }

    None
}

fn percent_decode_query_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hi = hex_value(bytes[index + 1]);
                let lo = hex_value(bytes[index + 2]);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    decoded.push((hi << 4) | lo);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(decoded).unwrap_or_else(|_| value.to_string())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::find_forbidden_query_key;

    #[test]
    fn detects_legacy_pagination_aliases() {
        assert_eq!(find_forbidden_query_key("pageSize=1"), Some("pageSize"));
        assert_eq!(
            find_forbidden_query_key("q=x&pageToken=2"),
            Some("pageToken")
        );
        assert_eq!(
            find_forbidden_query_key("page%5Ftoken=2"),
            Some("page_token")
        );
        assert_eq!(find_forbidden_query_key("page_size=1&cursor=2"), None);
    }
}
