//! Tooling Access control plane.
//!
//! Tooling Access provisions a single multichain, read-only endpoint per
//! account and mints short-lived ES256 session JWTs. Those JWTs are consumed by
//! the [`crate::rpc::RpcApiClient`] to authenticate RPC calls directly against
//! the provisioned endpoint; the private signing key never leaves the server.
//!
//! These routes live on the same Admin API base URL as the rest of this client.
//! Note this is distinct from [`super::AdminApiClient::create_jwt`], which
//! registers a public key on an endpoint's security config — here we mint the
//! session tokens themselves.

use serde::Deserialize;

use crate::{config::CachedToken, errors::SdkError};

use super::AdminApiClient;

/// Current Tooling Access status for the account. `enabled` is the source of
/// truth — a previously-provisioned-but-disabled account may still report a
/// non-null `endpoint_url`.
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all))]
#[cfg_attr(feature = "node", napi_derive::napi(object))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolingAccessStatus {
    pub enabled: bool,
    pub endpoint_url: Option<String>,
    pub enabled_at: Option<String>,
    /// The provisioned endpoint's id. Used to fetch the per-network URL map
    /// (`get_endpoint_urls`) for multichain routing. `None` on control planes
    /// that don't yet return it.
    pub endpoint_id: Option<String>,
}

// Control-plane responses use the `{ data, error }` envelope. `data` is null on
// error and `error` carries the message; success carries the payload in `data`.
#[derive(Deserialize)]
struct Envelope<T> {
    data: Option<T>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct StatusData {
    enabled: bool,
    #[serde(default)]
    endpoint_url: Option<String>,
    #[serde(default)]
    enabled_at: Option<String>,
    // The endpoint id. Serde may receive it as a string or a number depending
    // on the control plane; deserialize_optional_id normalizes both to String.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    endpoint_id: Option<String>,
}

#[derive(Deserialize)]
struct TokenData {
    endpoint_url: String,
    token: String,
    expires_at: String,
}

impl AdminApiClient {
    /// Returns the current Tooling Access status. Always succeeds (when
    /// authorized); inspect `enabled` to decide whether to enable.
    pub async fn tooling_access_status(&self) -> Result<ToolingAccessStatus, SdkError> {
        let url = self.config.admin().base_url.join("tooling-access")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        Self::parse_status(resp).await
    }

    /// Enables (provisions) Tooling Access. Idempotent — safe to call when
    /// already enabled. Requires an admin role and an eligible plan; ineligible
    /// callers receive an [`SdkError::Api`] carrying the reason.
    pub async fn enable_tooling_access(&self) -> Result<ToolingAccessStatus, SdkError> {
        self.set_tooling_access_enabled(true).await
    }

    /// Disables Tooling Access, pausing the endpoint. Idempotent.
    pub async fn disable_tooling_access(&self) -> Result<ToolingAccessStatus, SdkError> {
        self.set_tooling_access_enabled(false).await
    }

    async fn set_tooling_access_enabled(
        &self,
        enabled: bool,
    ) -> Result<ToolingAccessStatus, SdkError> {
        let url = self.config.admin().base_url.join("tooling-access")?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(&serde_json::json!({ "enabled": enabled }))
            .send()
            .await
            .map_err(SdkError::Http)?;
        Self::parse_status(resp).await
    }

    /// Mints a short-lived session JWT for the provisioned endpoint. Returns the
    /// endpoint URL, the JWT, and its expiry as a [`CachedToken`]. Requires
    /// Tooling Access to be enabled first; otherwise returns an
    /// [`SdkError::Api`] with status 400.
    pub async fn mint_tooling_token(&self) -> Result<CachedToken, SdkError> {
        let url = self.config.admin().base_url.join("tooling-access/token")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;
        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        let env: Envelope<TokenData> =
            serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
        let data = env.data.ok_or_else(|| SdkError::Api {
            status,
            body: env
                .error
                .unwrap_or_else(|| "missing token data".to_string()),
        })?;
        let exp_unix = parse_rfc3339_to_unix(&data.expires_at)?;
        Ok(CachedToken {
            endpoint_url: data.endpoint_url,
            token: data.token,
            exp_unix,
        })
    }

    async fn parse_status(resp: reqwest::Response) -> Result<ToolingAccessStatus, SdkError> {
        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;
        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        let env: Envelope<StatusData> =
            serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
        let data = env.data.ok_or_else(|| SdkError::Api {
            status,
            body: env
                .error
                .unwrap_or_else(|| "missing status data".to_string()),
        })?;
        Ok(ToolingAccessStatus {
            enabled: data.enabled,
            endpoint_url: data.endpoint_url,
            enabled_at: data.enabled_at,
            endpoint_id: data.endpoint_id,
        })
    }
}

// The endpoint id arrives as either a JSON string or a number. Accept both and
// normalize to an owned String so the field is uniform regardless of source.
fn deserialize_optional_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        Str(String),
        Num(i64),
    }
    let opt = Option::<StringOrNum>::deserialize(deserializer)?;
    Ok(opt.map(|v| match v {
        StringOrNum::Str(s) => s,
        StringOrNum::Num(n) => n.to_string(),
    }))
}

// Parse an RFC3339 / ISO8601 timestamp (e.g. "2026-06-23T20:40:00.000Z") to
// unix seconds without pulling in a date crate. Handles the common `Z` (UTC)
// and `±HH:MM` offset forms. The control plane emits UTC `Z` timestamps.
pub(crate) fn parse_rfc3339_to_unix(s: &str) -> Result<i64, SdkError> {
    let bad = || SdkError::Decode {
        // Reuse Decode for malformed control-plane payloads; build a synthetic
        // serde error so the variant carries a useful message and the raw body.
        source: serde::de::Error::custom("invalid expires_at timestamp"),
        body: s.to_string(),
    };

    let (date, rest) = s.split_once('T').ok_or_else(bad)?;
    let mut date_parts = date.split('-');
    let y: i64 = date_parts
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(bad)?;
    let mo: i64 = date_parts
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(bad)?;
    let d: i64 = date_parts
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(bad)?;

    // Strip the timezone designator, capturing the offset in seconds.
    let (time_part, offset_secs) = if let Some(t) = rest.strip_suffix('Z') {
        (t, 0i64)
    } else if let Some(idx) = rest.rfind(['+', '-']) {
        let (t, tz) = rest.split_at(idx);
        let sign = if tz.starts_with('-') { -1 } else { 1 };
        let tz = &tz[1..];
        let (oh, om) = tz.split_once(':').ok_or_else(bad)?;
        let oh: i64 = oh.parse().map_err(|_| bad())?;
        let om: i64 = om.parse().map_err(|_| bad())?;
        (t, sign * (oh * 3600 + om * 60))
    } else {
        (rest, 0i64)
    };

    // Time may carry fractional seconds; drop them.
    let time_main = time_part.split('.').next().unwrap_or(time_part);
    let mut tparts = time_main.split(':');
    let hh: i64 = tparts.next().and_then(|v| v.parse().ok()).ok_or_else(bad)?;
    let mm: i64 = tparts.next().and_then(|v| v.parse().ok()).ok_or_else(bad)?;
    let ss: i64 = tparts.next().and_then(|v| v.parse().ok()).unwrap_or(0);

    let days = days_from_civil(y, mo, d);
    let secs = days * 86_400 + hh * 3600 + mm * 60 + ss - offset_secs;
    Ok(secs)
}

// Days from 1970-01-01 for a proleptic Gregorian calendar date (Howard
// Hinnant's civil-date algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::config::{AdminConfig, SdkFullConfig};
    use crate::QuicknodeSdk;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sdk_for(base: &str) -> QuicknodeSdk {
        let mut cfg = SdkFullConfig::from_api_key("test-key".to_string());
        cfg.admin = Some(AdminConfig {
            base_url: Some(format!("{base}/")),
        });
        QuicknodeSdk::new(&cfg).unwrap()
    }

    #[test]
    fn parses_utc_z_timestamp() {
        // 2026-06-23T20:40:00Z is 1782247200 unix seconds.
        let got = parse_rfc3339_to_unix("2026-06-23T20:40:00.000Z").unwrap();
        assert_eq!(got, 1_782_247_200, "got {got}");
    }

    #[test]
    fn epoch_round_trips() {
        assert_eq!(parse_rfc3339_to_unix("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(parse_rfc3339_to_unix("1970-01-01T00:00:01Z").unwrap(), 1);
    }

    #[test]
    fn applies_positive_offset() {
        // 00:00 at +01:00 is 23:00 the previous day UTC == -3600.
        assert_eq!(
            parse_rfc3339_to_unix("1970-01-01T00:00:00+01:00").unwrap(),
            -3600
        );
    }

    #[test]
    fn rejects_garbage_timestamp() {
        assert!(matches!(
            parse_rfc3339_to_unix("not-a-date"),
            Err(SdkError::Decode { .. })
        ));
    }

    #[tokio::test]
    async fn status_happy_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tooling-access"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "enabled": true,
                    // endpoint_id is a JSON number on the wire; it must
                    // deserialize into the String endpoint_id field.
                    "endpoint_id": 3,
                    "endpoint_url": "https://tooling-access-abc123.quiknode.pro",
                    "enabled_at": "2026-06-23T20:30:00.000Z"
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = sdk_for(&server.uri());
        let status = sdk.admin.tooling_access_status().await.unwrap();
        assert!(status.enabled);
        assert_eq!(
            status.endpoint_url.as_deref(),
            Some("https://tooling-access-abc123.quiknode.pro")
        );
        assert_eq!(status.endpoint_id.as_deref(), Some("3"));
    }

    #[tokio::test]
    async fn enable_returns_status() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/tooling-access"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "enabled": true, "endpoint_url": "https://x.quiknode.pro" },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = sdk_for(&server.uri());
        let status = sdk.admin.enable_tooling_access().await.unwrap();
        assert!(status.enabled);
    }

    #[tokio::test]
    async fn mint_token_happy_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/tooling-access/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "endpoint_url": "https://tooling-access-abc123.quiknode.pro",
                    "token": "header.payload.sig",
                    "expires_at": "2026-06-23T20:40:00.000Z"
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = sdk_for(&server.uri());
        let tok = sdk.admin.mint_tooling_token().await.unwrap();
        assert_eq!(tok.token, "header.payload.sig");
        assert_eq!(tok.exp_unix, 1_782_247_200);
    }

    #[tokio::test]
    async fn mint_token_not_enabled_is_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/tooling-access/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "data": null,
                "error": "Tooling access is not enabled. Enable it first."
            })))
            .mount(&server)
            .await;

        let sdk = sdk_for(&server.uri());
        let err = sdk.admin.mint_tooling_token().await.unwrap_err();
        match err {
            SdkError::Api { status, body } => {
                assert_eq!(status.as_u16(), 400);
                assert!(body.contains("not enabled"), "body: {body}");
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }
}
