#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

use super::AdminApiClient;
use crate::errors::SdkError;

/// Time window, filters, and cursor for `list_transactions`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListTransactionsRequest {
    /// Positive duration ending now, such as `1h` or `7d`; units are s, m, h, or d.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// ISO 8601 start timestamp with timezone; supply with `to_time` instead of `last`.
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: Option<String>,
    /// ISO 8601 end timestamp with timezone.
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: Option<String>,
    /// Network to filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// RPC method to filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Transaction state: `submitted`, `succeeded`, or `failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Transaction hash to filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
    /// Opaque continuation cursor; can be supplied without a time window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Numeric endpoint IDs, encoded as repeated `endpoints[]` keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<i64>>,
    /// Page size from 1 through 1000; the API defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

/// Options for `get_transaction`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetTransactionRequest {
    /// Include retained request/response text. The API defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_payloads: Option<bool>,
}

/// Response from `list_transactions`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTransactionsResponse {
    /// Resolved time window and transaction page.
    pub data: TransactionsData,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// A page of transactions within the resolved time window.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsData {
    /// Resolved start timestamp.
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: String,
    /// Resolved end timestamp.
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: String,
    /// Transactions, newest first.
    pub transactions: Vec<TransactionSummary>,
    /// Cursor for the next page; omitted when there are no more rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// A retained transaction and its aggregate submission metadata.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    /// Network on which the transaction was submitted.
    pub network: String,
    /// Transaction hash.
    pub transaction_hash: String,
    /// Timestamp of the first submission.
    pub first_submitted_at: String,
    /// Timestamp of the latest submission.
    pub last_submitted_at: String,
    /// Number of submission attempts.
    pub attempt_count: i64,
    /// Numeric IDs of submitting endpoints; null entries remain in place.
    pub endpoint_ids: Vec<Option<i64>>,
    /// RPC methods used to submit this transaction.
    pub methods: Vec<String>,
    /// Whether archive data was used.
    pub archive: bool,
    /// Decimal block-height string, preserving values above JavaScript safe integers.
    pub height: Option<String>,
    /// Transaction state: `submitted`, `succeeded`, or `failed`.
    pub state: String,
}

/// Response from `get_transaction`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionResponse {
    /// Lookup result, including whether retained details exist.
    pub data: TransactionLookup,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Result of a retained transaction lookup.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLookup {
    /// Whether the transaction has retained details.
    pub exists: bool,
    /// Details when retained; null or absent when `exists` is false.
    pub data: Option<TransactionDetail>,
}

/// Retained transaction details and ordered submission attempts.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    /// Network on which the transaction was submitted.
    pub network: String,
    /// Transaction hash.
    pub transaction_hash: String,
    /// Timestamp of the first submission.
    pub first_submitted_at: String,
    /// Timestamp of the latest submission.
    pub last_submitted_at: String,
    /// Current transaction state.
    pub state: String,
    /// Number of attempts, when available.
    pub attempt_count: Option<i64>,
    /// Numeric endpoint IDs, when available; individual entries may be null.
    pub endpoint_ids: Option<Vec<Option<i64>>>,
    /// Submission methods, when available.
    pub methods: Option<Vec<String>>,
    /// Whether archive data was used, when available.
    pub archive: Option<bool>,
    /// Decimal block-height string, when available.
    pub height: Option<String>,
    /// Retained attempts in the order returned by the API.
    pub attempts: Vec<TransactionAttempt>,
}

/// One retained submission attempt.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAttempt {
    /// Submission timestamp.
    pub submitted_at: String,
    /// Numeric ID of the submitting endpoint.
    pub endpoint_id: i64,
    /// RPC method used.
    pub method: String,
    /// Request UUID.
    pub request_id: String,
    /// Payload text, only when requested and available.
    pub payloads: Option<TransactionPayloads>,
}

/// Original request and response text for an attempt.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayloads {
    /// Original request text; preserved without parsing or reformatting.
    pub request: Option<String>,
    /// Original response text; may contain non-JSON content.
    pub response: Option<String>,
}

impl AdminApiClient {
    /// Lists retained transactions. Supply `last`, or both `from_time` and `to_time`, for
    /// the first page. A cursor alone continues the same resolved window.
    pub async fn list_transactions(
        &self,
        params: &ListTransactionsRequest,
    ) -> Result<ListTransactionsResponse, SdkError> {
        let url = self.config.admin().base_url.join("transactions")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&transactions_query(params))
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;
        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Looks up a transaction by network and hash. An absent or expired
    /// transaction returns `data.exists = false`; payloads are opt-in.
    pub async fn get_transaction(
        &self,
        network: &str,
        hash: &str,
        params: &GetTransactionRequest,
    ) -> Result<GetTransactionResponse, SdkError> {
        let mut url = self.config.admin().base_url.join("transactions/")?;
        url.path_segments_mut()
            .map_err(|()| SdkError::Config("Admin base URL does not support path segments".into()))?
            .pop_if_empty()
            .extend([network, hash]);
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;
        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }
}

fn transactions_query(params: &ListTransactionsRequest) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    for (key, value) in [
        ("last", &params.last),
        ("from", &params.from_time),
        ("to", &params.to_time),
        ("network", &params.network),
        ("method", &params.method),
        ("state", &params.state),
        ("transaction_hash", &params.transaction_hash),
        ("cursor", &params.cursor),
    ] {
        if let Some(value) = value {
            query.push((key, value.clone()));
        }
    }
    if let Some(endpoints) = &params.endpoints {
        for endpoint in endpoints {
            query.push(("endpoints[]", endpoint.to_string()));
        }
    }
    if let Some(limit) = params.limit {
        query.push(("limit", limit.to_string()));
    }
    query
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{GetTransactionRequest, ListTransactionsRequest};
    use crate::{errors::SdkError, AdminConfig, QuicknodeSdk, SdkFullConfig};
    use serde_json::{json, Value};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuicknodeSdk {
        QuicknodeSdk::new(&SdkFullConfig {
            api_key: Some("test-key".into()),
            admin: Some(AdminConfig {
                base_url: Some(base_url),
            }),
            http: None,
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
            rpc: None,
        })
        .unwrap()
    }

    fn page() -> Value {
        json!({"data": {
            "from": "2026-01-01T00:00:00Z", "to": "2026-01-02T00:00:00Z",
            "transactions": [{"network": "ethereum-mainnet", "transaction_hash": "0xfixture",
                "first_submitted_at": "2026-01-01T01:00:00Z", "last_submitted_at": "2026-01-01T02:00:00Z",
                "attempt_count": 2, "endpoint_ids": [4123, null, 4124],
                "methods": ["eth_sendRawTransaction"], "archive": false,
                "height": "9007199254740993", "state": "succeeded"}], "next_cursor": "fixture+/cursor="
        }, "error": null})
    }

    #[tokio::test]
    async fn list_preserves_height_and_bracketed_endpoint_filters() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v0/transactions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page()))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/v0/", server.uri()));
        let result = sdk
            .admin
            .list_transactions(&ListTransactionsRequest {
                last: Some("1d".into()),
                network: Some("ethereum-mainnet".into()),
                method: Some("eth_sendRawTransaction".into()),
                state: Some("succeeded".into()),
                transaction_hash: Some("0xfixture&state=failed".into()),
                endpoints: Some(vec![4123, 4124]),
                limit: Some(2),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(
            result.data.transactions[0].height.as_deref(),
            Some("9007199254740993")
        );
        assert_eq!(result.data.transactions[0].attempt_count, 2);
        assert_eq!(
            result.data.transactions[0].endpoint_ids,
            vec![Some(4123), None, Some(4124)]
        );
        assert_eq!(
            serde_json::to_value(&result).unwrap()["data"]["transactions"][0]["endpoint_ids"],
            json!([4123, null, 4124])
        );
        assert!(!result.data.transactions[0].archive);
        assert_eq!(result.data.next_cursor.as_deref(), Some("fixture+/cursor="));
        assert_eq!(
            serde_json::to_value(&result).unwrap()["data"]["transactions"][0]["height"],
            "9007199254740993"
        );
        let requests = server.received_requests().await.unwrap();
        let query: Vec<_> = requests[0]
            .url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert_eq!(
            query,
            vec![
                ("last".into(), "1d".into()),
                ("network".into(), "ethereum-mainnet".into()),
                ("method".into(), "eth_sendRawTransaction".into()),
                ("state".into(), "succeeded".into()),
                ("transaction_hash".into(), "0xfixture&state=failed".into()),
                ("endpoints[]".into(), "4123".into()),
                ("endpoints[]".into(), "4124".into()),
                ("limit".into(), "2".into()),
            ]
        );
    }

    #[tokio::test]
    async fn list_supports_explicit_window_and_cursor_only_continuation() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/transactions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {
                "from": "2026-01-01T00:00:00Z", "to": "2026-01-02T00:00:00Z", "transactions": []
            }, "error": null})))
            .expect(2)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let first = sdk
            .admin
            .list_transactions(&ListTransactionsRequest {
                from_time: Some("2026-01-01T00:00:00Z".into()),
                to_time: Some("2026-01-02T00:00:00Z".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(first.data.transactions.is_empty());
        assert!(first.data.next_cursor.is_none());
        let second = sdk
            .admin
            .list_transactions(&ListTransactionsRequest {
                cursor: Some("fixture+/cursor=".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(second.data.from_time, first.data.from_time);
        assert_eq!(second.data.to_time, first.data.to_time);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query_pairs().collect::<Vec<_>>(),
            vec![
                ("from".into(), "2026-01-01T00:00:00Z".into()),
                ("to".into(), "2026-01-02T00:00:00Z".into())
            ]
        );
        assert_eq!(
            requests[1].url.query_pairs().collect::<Vec<_>>(),
            vec![("cursor".into(), "fixture+/cursor=".into())]
        );
    }

    #[tokio::test]
    async fn absent_detail_keeps_nested_data_and_does_not_request_payloads() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/transactions/ethereum-mainnet/missing"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {"exists": false, "data": null}, "error": null
            })))
            .expect(2)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let result = sdk
            .admin
            .get_transaction(
                "ethereum-mainnet",
                "missing",
                &GetTransactionRequest::default(),
            )
            .await
            .unwrap();
        assert!(!result.data.exists);
        assert!(result.data.data.is_none());
        assert_eq!(
            serde_json::to_value(&result).unwrap()["data"],
            json!({"exists": false, "data": null})
        );
        sdk.admin
            .get_transaction(
                "ethereum-mainnet",
                "missing",
                &GetTransactionRequest {
                    include_payloads: Some(false),
                },
            )
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        assert!(requests[0].url.query().is_none());
        assert_eq!(requests[1].url.query().unwrap(), "include_payloads=false");
    }

    #[tokio::test]
    async fn detail_preserves_payload_text_and_attempt_order() {
        let server = MockServer::start().await;
        let request_text = "{ \"method\": \"eth_sendRawTransaction\",\n \"params\": [] }\n";
        let response_text = "upstream response is not JSON\n";
        Mock::given(method("GET"))
            .and(path("/v0/transactions/network%2Fpart%3Fx%23y/hash%2Fpart%3Finclude_payloads=false"))
            .and(query_param("include_payloads", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"exists": true, "data": {
                "network": "network/part?x#y", "transaction_hash": "hash/part?include_payloads=false",
                "first_submitted_at": "2026-01-01T01:00:00Z", "last_submitted_at": "2026-01-01T02:00:00Z",
                "state": "succeeded", "height": "9007199254740993", "archive": false,
                "attempt_count": null, "endpoint_ids": [4123, null, 4124], "methods": null,
                "attempts": [
                    {"submitted_at": "2026-01-01T01:00:00Z", "endpoint_id": 4123,
                     "method": "eth_sendRawTransaction", "request_id": "fixture-request-a",
                     "payloads": {"request": request_text, "response": response_text}},
                    {"submitted_at": "2026-01-01T02:00:00Z", "endpoint_id": 4124,
                     "method": "eth_sendRawTransaction", "request_id": "fixture-request-b",
                     "payloads": {"request": null, "response": null}}
                ]}}, "error": null}))).expect(1).mount(&server).await;
        let sdk = make_sdk(format!("{}/v0/", server.uri()));
        let result = sdk
            .admin
            .get_transaction(
                "network/part?x#y",
                "hash/part?include_payloads=false",
                &GetTransactionRequest {
                    include_payloads: Some(true),
                },
            )
            .await
            .unwrap();
        assert!(result.data.exists);
        let detail = result.data.data.unwrap();
        assert_eq!(detail.height.as_deref(), Some("9007199254740993"));
        assert_eq!(detail.archive, Some(false));
        assert!(detail.attempt_count.is_none());
        assert_eq!(
            detail.endpoint_ids,
            Some(vec![Some(4123), None, Some(4124)])
        );
        assert!(detail.methods.is_none());
        assert_eq!(detail.attempts[0].endpoint_id, 4123);
        assert_eq!(detail.attempts[1].endpoint_id, 4124);
        assert_eq!(detail.attempts[0].request_id, "fixture-request-a");
        assert_eq!(detail.attempts[1].request_id, "fixture-request-b");
        let payloads = detail.attempts[0].payloads.as_ref().unwrap();
        assert_eq!(payloads.request.as_deref(), Some(request_text));
        assert_eq!(payloads.response.as_deref(), Some(response_text));
        assert!(detail.attempts[1]
            .payloads
            .as_ref()
            .unwrap()
            .request
            .is_none());
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[0].url.query().unwrap(), "include_payloads=true");
        assert!(requests[0].url.fragment().is_none());
    }

    #[tokio::test]
    async fn detail_accepts_missing_optional_metadata_and_payloads() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/transactions/ethereum-mainnet/0xfixture"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"exists": true, "data": {
                "network": "ethereum-mainnet", "transaction_hash": "0xfixture",
                "first_submitted_at": "2026-01-01T01:00:00Z", "last_submitted_at": "2026-01-01T01:00:00Z",
                "state": "submitted", "attempts": [{"submitted_at": "2026-01-01T01:00:00Z",
                    "endpoint_id": 4123, "method": "eth_sendRawTransaction", "request_id": "fixture-request-a"}]
            }}, "error": null}))).expect(1).mount(&server).await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let detail = sdk
            .admin
            .get_transaction(
                "ethereum-mainnet",
                "0xfixture",
                &GetTransactionRequest::default(),
            )
            .await
            .unwrap()
            .data
            .data
            .unwrap();
        assert!(detail.height.is_none());
        assert!(detail.archive.is_none());
        assert!(detail.attempt_count.is_none());
        assert!(detail.endpoint_ids.is_none());
        assert!(detail.methods.is_none());
        assert!(detail.attempts[0].payloads.is_none());
    }

    #[tokio::test]
    async fn transaction_routes_preserve_api_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/transactions"))
            .respond_with(ResponseTemplate::new(403).set_body_string("feature unavailable"))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/transactions/unknown/0xfixture"))
            .respond_with(ResponseTemplate::new(404).set_body_string("network not found"))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let list = sdk
            .admin
            .list_transactions(&ListTransactionsRequest {
                last: Some("1h".into()),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(
            matches!(list, SdkError::Api { status, body } if status.as_u16() == 403 && body == "feature unavailable")
        );
        let detail = sdk
            .admin
            .get_transaction("unknown", "0xfixture", &GetTransactionRequest::default())
            .await
            .unwrap_err();
        assert!(
            matches!(detail, SdkError::Api { status, body } if status.as_u16() == 404 && body == "network not found")
        );
    }
}
