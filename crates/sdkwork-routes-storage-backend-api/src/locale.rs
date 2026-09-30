//! Request-boundary locale resolution for the admin storage plane
//! (`I18N_SPEC.md` §2, §3, §4).
//!
//! Locale resolution is a framework request-boundary responsibility: handlers
//! receive an already-resolved [`DriveLocale`] extractor and never read locale
//! headers themselves. Only the standard `Accept-Language` header negotiates
//! the locale — there is no custom locale header — tags are normalized before
//! comparison, and a missing header or an unsupported tag resolves to the
//! module default locale. Responses built from the resolved locale are stamped
//! with `Content-Language` and `Vary: Accept-Language` by
//! [`stamp_locale_headers`].

use axum::extract::FromRequestParts;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::request::Parts;
use axum::http::{header::CONTENT_LANGUAGE, header::VARY, HeaderValue, Response};

/// Locales the storage plane can serve. Mirrors the active seed locales in
/// `database/database.manifest.json` and the host console's language set.
pub const SUPPORTED_LOCALES: [&str; 2] = ["zh-CN", "en-US"];

/// Locale served when the request carries no usable locale preference
/// (`I18N_SPEC.md` §2 fallback order ends at the application default locale).
pub const DEFAULT_LOCALE: &str = "zh-CN";

/// Normalized BCP 47 locale tag of the effective response language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriveLocale(&'static str);

impl DriveLocale {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl Default for DriveLocale {
    fn default() -> Self {
        Self(DEFAULT_LOCALE)
    }
}

/// Normalize a client locale tag onto the supported set. Prefix matching keeps
/// regional variants (`zh-Hans`, `en-GB`, …) on their language's entry.
fn normalize_locale_tag(raw: &str) -> Option<&'static str> {
    let lowered = raw.trim().to_ascii_lowercase();
    let tag = if lowered == "zh" || lowered.starts_with("zh-") {
        "zh-CN"
    } else if lowered == "en" || lowered.starts_with("en-") {
        "en-US"
    } else {
        return None;
    };
    SUPPORTED_LOCALES.contains(&tag).then_some(tag)
}

/// Resolve an `Accept-Language` header value to the highest-weight supported
/// locale, defaulting when the header is missing, unparsable, or names no
/// supported locale.
#[must_use]
pub fn resolve_accept_language(raw: Option<&str>) -> DriveLocale {
    let Some(raw) = raw else {
        return DriveLocale(DEFAULT_LOCALE);
    };
    let mut best: Option<(f32, &'static str)> = None;
    for part in raw.split(',') {
        let mut segment = part.split(';');
        let tag = normalize_locale_tag(segment.next().unwrap_or(""));
        let quality = segment
            .next()
            .and_then(|param| param.trim().strip_prefix("q="))
            .and_then(|value| value.trim().parse::<f32>().ok())
            .unwrap_or(1.0);
        if let Some(tag) = tag {
            let wins = match best {
                Some((best_quality, _)) => quality > best_quality,
                None => quality > 0.0,
            };
            if wins {
                best = Some((quality, tag));
            }
        }
    }
    DriveLocale(best.map_or(DEFAULT_LOCALE, |(_, tag)| tag))
}

/// Stamp the locale-sensitive response headers required by `I18N_SPEC.md` §4
/// onto a response whose representation varies by language.
pub fn stamp_locale_headers<T>(response: &mut Response<T>, locale: DriveLocale) {
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(locale.as_str()) {
        headers.insert(CONTENT_LANGUAGE, value);
    }
    headers.insert(VARY, HeaderValue::from_static("Accept-Language"));
}

impl<S> FromRequestParts<S> for DriveLocale
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let raw = parts
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok());
        Ok(resolve_accept_language(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_header_resolves_to_default_locale() {
        assert_eq!(resolve_accept_language(None).as_str(), "zh-CN");
        assert_eq!(resolve_accept_language(Some("")).as_str(), "zh-CN");
    }

    #[test]
    fn unsupported_tags_fall_back_to_default_locale() {
        assert_eq!(resolve_accept_language(Some("fr-FR,de-DE")).as_str(), "zh-CN");
        assert_eq!(resolve_accept_language(Some("*")).as_str(), "zh-CN");
    }

    #[test]
    fn language_prefixes_normalize_onto_supported_locales() {
        assert_eq!(resolve_accept_language(Some("zh")).as_str(), "zh-CN");
        assert_eq!(resolve_accept_language(Some("zh-Hans-CN")).as_str(), "zh-CN");
        assert_eq!(resolve_accept_language(Some("en-GB,en;q=0.9")).as_str(), "en-US");
    }

    #[test]
    fn highest_quality_supported_tag_wins() {
        assert_eq!(
            resolve_accept_language(Some("fr-FR;q=1.0,zh-CN;q=0.8,en-US;q=0.9")).as_str(),
            "en-US"
        );
        assert_eq!(
            resolve_accept_language(Some("en-US;q=0.3,zh-CN;q=0.9")).as_str(),
            "zh-CN"
        );
    }

    #[test]
    fn zero_quality_tags_are_ignored() {
        assert_eq!(resolve_accept_language(Some("en-US;q=0")).as_str(), "zh-CN");
    }
}
