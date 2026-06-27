//! Go binding facade (UniFFI), compiled only under the `go` feature.
//!
//! This module is the synchronous, FFI-friendly surface that
//! `uniffi-bindgen-go` turns into the Go package. It holds no HTTP or business
//! logic: it wraps the async core API with a shared Tokio runtime + `block_on`
//! (the same approach as the Ruby binding) and maps [`SdkError`] to a
//! UniFFI-representable error enum at the boundary.
//!
//! It lives inside the core crate (rather than a separate facade crate) because
//! uniffi 0.31 requires the crate that derives `uniffi::Record`/`Error` on the
//! data types to be the same crate that calls `setup_scaffolding!`. Everything
//! here is gated on the `go` feature, so default and python/node/ruby builds
//! are unaffected.

// `#[uniffi::export]` methods must take owned arguments — references cannot
// cross the FFI boundary — so `needless_pass_by_value` is a false positive
// here. Runtime initialization failure is unrecoverable at this boundary, so
// `expect` is the honest choice (the Ruby binding allows the same crate-wide).
#![allow(clippy::needless_pass_by_value, clippy::expect_used)]

use std::sync::OnceLock;

use crate::admin::{GetEndpointsRequest, GetEndpointsResponse};
use crate::config::{AdminConfig, SdkFullConfig};
use crate::errors::{HttpKind, SdkError};
use crate::{ClientInfo, QuicknodeSdk};

// A single shared runtime for all blocking HTTP calls, mirroring the Ruby
// binding. `uniffi-bindgen-go` surfaces calls as blocking on the Go side
// regardless, so exposing a synchronous API here costs no ergonomics; Go
// callers do concurrency with their own goroutines.
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME
        .get_or_init(|| tokio::runtime::Runtime::new().expect("failed to initialize tokio runtime"))
}

/// Errors surfaced across the Go FFI boundary. Mirrors the typed hierarchy used
/// by the other bindings (see CLAUDE.md §Error Handling): `Config` covers
/// configuration/URL-parse failures; `Http`/`Timeout`/`Connection` cover
/// transport-level failures; `Api` carries the HTTP status and raw body;
/// `Decode` carries the raw body that failed to parse.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum QuicknodeError {
    #[error("configuration error: {message}")]
    Config { message: String },
    #[error("HTTP error: {message}")]
    Http { message: String },
    #[error("request timed out: {message}")]
    Timeout { message: String },
    #[error("connection error: {message}")]
    Connection { message: String },
    #[error("API error (status {status}): {body}")]
    Api {
        message: String,
        status: u16,
        body: String,
    },
    #[error("failed to decode response: {message}")]
    Decode { message: String, body: String },
}

impl From<SdkError> for QuicknodeError {
    fn from(e: SdkError) -> Self {
        let message = e.to_string();
        match &e {
            SdkError::Config(_) | SdkError::UrlParse(_) => QuicknodeError::Config { message },
            SdkError::Api { status, body } => QuicknodeError::Api {
                message,
                status: status.as_u16(),
                body: body.clone(),
            },
            SdkError::Decode { body, .. } => QuicknodeError::Decode {
                message,
                body: body.clone(),
            },
            SdkError::Http(_) => match e.http_kind() {
                Some(HttpKind::Timeout) => QuicknodeError::Timeout { message },
                Some(HttpKind::Connect) => QuicknodeError::Connection { message },
                _ => QuicknodeError::Http { message },
            },
        }
    }
}

/// Go-facing handle to the SDK. Wraps the core [`QuicknodeSdk`] and exposes its
/// methods synchronously.
#[derive(uniffi::Object)]
pub struct QuicknodeSdkClient {
    inner: QuicknodeSdk,
}

#[uniffi::export]
impl QuicknodeSdkClient {
    /// Construct an SDK client from an API key. The `User-Agent` is attributed
    /// to the Go binding.
    #[uniffi::constructor]
    pub fn new(api_key: String) -> Result<Self, QuicknodeError> {
        Self::build(api_key, None)
    }

    /// Construct an SDK client overriding the admin API base URL. Primarily for
    /// testing against a mock server; production callers use [`Self::new`].
    #[uniffi::constructor]
    pub fn new_with_admin_base_url(
        api_key: String,
        admin_base_url: String,
    ) -> Result<Self, QuicknodeError> {
        Self::build(api_key, Some(admin_base_url))
    }

    /// List endpoints on the account. See [`GetEndpointsRequest`] for filters.
    pub fn get_endpoints(
        &self,
        params: GetEndpointsRequest,
    ) -> Result<GetEndpointsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.admin.get_endpoints(&params))
            .map_err(QuicknodeError::from)
    }
}

impl QuicknodeSdkClient {
    fn build(api_key: String, admin_base_url: Option<String>) -> Result<Self, QuicknodeError> {
        let config = SdkFullConfig {
            api_key,
            http: None,
            admin: admin_base_url.map(|base_url| AdminConfig {
                base_url: Some(base_url),
            }),
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
        };
        let client_info = ClientInfo {
            language: "go".to_string(),
            language_version: "unknown".to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let inner = QuicknodeSdk::new_with_client_info(&config, Some(client_info))?;
        Ok(Self { inner })
    }
}
