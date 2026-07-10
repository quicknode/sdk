#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (status {status}): {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("Failed to decode response: {source}\nBody: {body}")]
    Decode {
        #[source]
        source: serde_json::Error,
        body: String,
    },

    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("JSON-RPC error (code {code}): {message}")]
    Rpc { code: i64, message: String },
}

// Classifies a transport-level HTTP failure. Bindings use this to pick a
// typed exception subclass (TimeoutError / ConnectionError / HttpError) so the
// reqwest predicate logic lives in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpKind {
    Timeout,
    Connect,
    Other,
}

impl SdkError {
    pub fn http_kind(&self) -> Option<HttpKind> {
        match self {
            SdkError::Http(e) if e.is_timeout() => Some(HttpKind::Timeout),
            SdkError::Http(e) if e.is_connect() => Some(HttpKind::Connect),
            SdkError::Http(_) => Some(HttpKind::Other),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_display_includes_status_and_body() {
        let err = SdkError::Api {
            status: reqwest::StatusCode::NOT_FOUND,
            body: "not found".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("404"), "expected 404 in {s}");
        assert!(s.contains("not found"), "expected body in {s}");
    }

    #[test]
    fn config_error_display() {
        let err = SdkError::Config("missing api key".to_string());
        assert!(err.to_string().contains("missing api key"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn http_kind_none_for_non_http_variants() {
        assert!(SdkError::Config("x".to_string()).http_kind().is_none());
        let decode_err = SdkError::Decode {
            source: serde_json::from_str::<i32>("bad").unwrap_err(),
            body: "bad".to_string(),
        };
        assert!(decode_err.http_kind().is_none());
    }
}
