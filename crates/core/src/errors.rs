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
}
