use http::HeaderMap;
use sdkwork_drive_security::TRACE_ID_HEADER;
use sdkwork_web_core::trace::resolve_trace_context;

pub fn resolve_trace_id(headers: &HeaderMap, request_id: &str) -> String {
    if let Some(value) = optional_header(headers, TRACE_ID_HEADER) {
        return value;
    }

    let trace_context = resolve_trace_context(headers, request_id);
    trace_context
        .traceparent
        .split('-')
        .nth(1)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .unwrap_or(trace_context.traceparent)
}

fn optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn prefers_explicit_trace_header() {
        let mut headers = HeaderMap::new();
        headers.insert(TRACE_ID_HEADER, HeaderValue::from_static("trace-explicit"));
        assert_eq!(resolve_trace_id(&headers, "request-001"), "trace-explicit");
    }

    #[test]
    fn derives_trace_id_from_traceparent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "traceparent",
            HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
        );
        assert_eq!(
            resolve_trace_id(&headers, "request-001"),
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
    }
}
