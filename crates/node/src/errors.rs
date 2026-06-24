use napi::{bindgen_prelude::Error, Status};
use quicknode_sdk::errors::{HttpKind, SdkError};

// napi-rs can only throw plain napi::Error with a status + message. To give
// callers a typed class hierarchy in JS, we encode the variant as a structured
// prefix in the message; the JS-side wrapper (npm/sdk.js) parses the prefix
// and rethrows as the matching typed class (ApiError / TimeoutError / ...).
//
// Wire format: "[<kind>|<status>|<body_len>]<original_message>"
//   - kind: one of Config | Http | Timeout | Connect | Api | Decode | Rpc
//   - status: u16 for Api, the JSON-RPC code (may be negative) for Rpc, "-" otherwise
//   - body_len: byte length of body blob appended after message, "-" otherwise.
//     For Api/Decode the body is the raw response body; for Rpc it is the
//     JSON-RPC error `message` so JS can expose `.message` distinctly.
// The body bytes are appended after a "\x1f" (unit separator) so JS can split cleanly.
#[allow(clippy::needless_pass_by_value)]
pub fn map_sdk_err(e: SdkError) -> Error {
    let msg = e.to_string();
    // status_str carries a u16 for Api but the JSON-RPC code (i64, possibly
    // negative) for Rpc, so it is a free-form string rather than Option<u16>.
    let (kind, status_str, body) = match &e {
        SdkError::Config(_) | SdkError::UrlParse(_) => ("Config", None, None),
        SdkError::Api { status, body } => {
            ("Api", Some(status.as_u16().to_string()), Some(body.clone()))
        }
        SdkError::Decode { body, .. } => ("Decode", None, Some(body.clone())),
        SdkError::Rpc { code, message } => ("Rpc", Some(code.to_string()), Some(message.clone())),
        SdkError::Http(_) => match e.http_kind() {
            Some(HttpKind::Timeout) => ("Timeout", None, None),
            Some(HttpKind::Connect) => ("Connect", None, None),
            _ => ("Http", None, None),
        },
    };
    let status_s = status_str.unwrap_or_else(|| "-".to_string());
    let body_s = body.as_deref().unwrap_or("");
    let body_len = if body.is_some() {
        body_s.len().to_string()
    } else {
        "-".to_string()
    };
    let tagged = format!("[{kind}|{status_s}|{body_len}]{msg}\u{001f}{body_s}");
    Error::new(Status::GenericFailure, tagged)
}
