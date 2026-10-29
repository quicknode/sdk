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

/// Filters and pagination for `list_activity_logs`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListActivityLogsRequest {
    /// Search descriptions, endpoint subdomains, and team names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    /// User or API-key UUID returned by `list_activity_log_filters`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    /// Numeric endpoint ID represented as a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>,
    /// Target type, such as `endpoint`, `team`, or `api_key`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    /// ISO 8601 start date or timestamp; a bare date starts at midnight UTC.
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: Option<String>,
    /// ISO 8601 end date or timestamp; a bare date includes that entire UTC day.
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: Option<String>,
    /// Page size from 0 through 100; the API defaults to 20.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Entries to skip. Pin `to_time` when exporting to avoid shifting pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

/// Response from `list_activity_logs`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActivityLogsResponse {
    /// Activity entries, newest first.
    pub data: Vec<ActivityLogEntry>,
    /// Offset pagination metadata.
    pub pagination: ActivityLogPagination,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Offset pagination for activity logs.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogPagination {
    /// Total entries matching the filters.
    pub total: i64,
    /// Page size used for this response.
    pub limit: i32,
    /// Number of entries skipped.
    pub offset: i64,
}

/// Response from `get_activity_log`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetActivityLogResponse {
    /// The matching activity entry, when present.
    pub data: Option<ActivityLogEntry>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// An account change and the actor who made it.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    /// Activity entry UUID.
    pub id: String,
    /// ISO 8601 timestamp of the change.
    pub occurred_at: String,
    /// Action performed, such as `created`.
    pub action: String,
    /// Human-readable description, when available.
    pub description: Option<String>,
    /// Resource affected by the change.
    pub target: ActivityLogTarget,
    /// Actor responsible for the change.
    pub actor: ActivityLogActor,
}

/// Resource affected by an activity entry.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogTarget {
    /// Resource identifier; endpoint targets use numeric IDs as strings.
    pub id: String,
    /// Resource kind, when available.
    pub kind: Option<String>,
    /// Resource label.
    pub label: String,
    /// Resource name, when available.
    pub name: Option<String>,
}

/// Actor attached to an activity entry, including deleted users.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogActor {
    /// User or API-key UUID; absent for Quicknode and deleted users.
    pub id: Option<String>,
    /// Actor kind: `user`, `api_key`, `quicknode`, or `deleted_user`.
    #[serde(rename = "type")]
    pub actor_type: String,
    /// Actor name, when available.
    pub name: Option<String>,
    /// Actor email, when available.
    pub email: Option<String>,
}

/// Response from `list_activity_log_filters`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActivityLogFiltersResponse {
    /// Available actor and target-type filters.
    pub data: ActivityLogFiltersData,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Values accepted by the activity-log filters.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogFiltersData {
    /// Users and API keys available for filtering.
    pub actors: Vec<ActivityLogFilterActor>,
    /// Target types available for filtering.
    pub types: Vec<ActivityLogFilterType>,
}

/// An actor available for activity-log filtering.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogFilterActor {
    /// User or API-key UUID to pass as `actor`.
    pub id: String,
    /// Actor name, when available.
    pub name: Option<String>,
    /// Actor kind, such as `user` or `api_key`.
    #[serde(rename = "type")]
    pub actor_type: String,
}

/// A target type available for activity-log filtering.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogFilterType {
    /// Value to pass as `target_type`.
    pub value: String,
    /// Human-readable target type.
    pub label: String,
}

impl AdminApiClient {
    /// Lists account changes. Feature access can return HTTP 403.
    pub async fn list_activity_logs(
        &self,
        params: &ListActivityLogsRequest,
    ) -> Result<ListActivityLogsResponse, SdkError> {
        let url = self.config.admin().base_url.join("activity_logs")?;
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

    /// Returns available actor and target-type filters for activity logs.
    pub async fn list_activity_log_filters(
        &self,
    ) -> Result<ListActivityLogFiltersResponse, SdkError> {
        let url = self.config.admin().base_url.join("activity_logs/filters")?;
        let resp = self
            .config
            .http_client()
            .get(url)
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

    /// Returns one activity entry by its UUID. Missing entries return HTTP 404.
    pub async fn get_activity_log(&self, id: &str) -> Result<GetActivityLogResponse, SdkError> {
        let mut url = self.config.admin().base_url.join("activity_logs/")?;
        url.path_segments_mut()
            .map_err(|()| SdkError::Config("Admin base URL does not support path segments".into()))?
            .pop_if_empty()
            .push(id);
        let resp = self
            .config
            .http_client()
            .get(url)
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{ListActivityLogsRequest, ListActivityLogsResponse};
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

    fn entry(actor_type: &str, actor_id: Option<&str>) -> Value {
        json!({
            "id": format!("fixture-{actor_type}"),
            "occurred_at": "2026-01-02T03:04:05Z",
            "action": "updated",
            "description": null,
            "target": {"id": "42", "kind": null, "label": "Test endpoint", "name": null},
            "actor": {"id": actor_id, "type": actor_type, "name": null, "email": null}
        })
    }

    #[tokio::test]
    async fn list_preserves_nullable_actors_and_exact_filters() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v0/activity_logs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [entry("user", Some("fixture-user")), entry("api_key", Some("fixture-key")),
                         entry("quicknode", None), entry("deleted_user", None)],
                "pagination": {"total": 4, "limit": 100, "offset": 4000000000_i64}, "error": null
            })))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/v0/", server.uri()));
        let result = sdk
            .admin
            .list_activity_logs(&ListActivityLogsRequest {
                q: Some("test & actor=other".into()),
                actor: Some("fixture-user".into()),
                endpoint_id: Some("42".into()),
                target_type: Some("endpoint".into()),
                from_time: Some("2026-01-01".into()),
                to_time: Some("2026-01-02".into()),
                limit: Some(100),
                offset: Some(4000000000),
            })
            .await
            .unwrap();
        assert_eq!(result.pagination.offset, 4000000000);
        assert_eq!(result.data.len(), 4);
        assert_eq!(result.data[0].actor.id.as_deref(), Some("fixture-user"));
        assert_eq!(result.data[1].actor.actor_type, "api_key");
        assert_eq!(result.data[2].actor.actor_type, "quicknode");
        assert_eq!(result.data[3].actor.actor_type, "deleted_user");
        assert!(result.data[3].actor.id.is_none());
        assert!(result.data[3].actor.name.is_none());
        assert!(result.data[3].actor.email.is_none());
        assert!(result.data[0].target.kind.is_none());
        assert!(result.data[0].target.name.is_none());
        let serialized = serde_json::to_value(&result).unwrap();
        assert_eq!(serialized["data"][3]["actor"]["type"], "deleted_user");
        assert!(serialized["data"][3]["actor"]["id"].is_null());
        assert!(serialized["data"][3]["actor"].get("actor_type").is_none());
        let requests = server.received_requests().await.unwrap();
        let mut query: Vec<_> = requests[0]
            .url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        query.sort();
        let mut expected = vec![
            ("q".to_string(), "test & actor=other".to_string()),
            ("actor".into(), "fixture-user".into()),
            ("endpoint_id".into(), "42".into()),
            ("type".into(), "endpoint".into()),
            ("from".into(), "2026-01-01".into()),
            ("to".into(), "2026-01-02".into()),
            ("limit".into(), "100".into()),
            ("offset".into(), "4000000000".into()),
        ];
        expected.sort();
        assert_eq!(query, expected);
    }

    #[tokio::test]
    async fn list_accepts_zero_limit_and_empty_data() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/activity_logs"))
            .and(query_param("limit", "0"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [], "pagination": {"total": 0, "limit": 0, "offset": 0}, "error": null
            })))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let result = sdk
            .admin
            .list_activity_logs(&ListActivityLogsRequest {
                limit: Some(0),
                offset: Some(0),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(result.data.is_empty());
        assert_eq!(result.pagination.limit, 0);
        assert_eq!(
            server.received_requests().await.unwrap()[0]
                .url
                .query_pairs()
                .count(),
            2
        );
    }

    #[tokio::test]
    async fn filters_use_distinct_actor_and_type_metadata() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/activity_logs/filters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {"actors": [{"id": "fixture-key", "name": null, "type": "api_key"}],
                         "types": [{"value": "endpoint", "label": "Endpoints"}]}, "error": null
            })))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let result = sdk.admin.list_activity_log_filters().await.unwrap();
        assert_eq!(result.data.actors[0].id, "fixture-key");
        assert!(result.data.actors[0].name.is_none());
        assert_eq!(result.data.actors[0].actor_type, "api_key");
        assert_eq!(result.data.types[0].value, "endpoint");
        assert_eq!(result.data.types[0].label, "Endpoints");
        assert!(server.received_requests().await.unwrap()[0]
            .url
            .query()
            .is_none());
    }

    #[tokio::test]
    async fn detail_encodes_entry_id_as_one_segment() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/activity_logs/entry%2Fpart%3Fx%23y"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": entry("user", Some("fixture-user")), "error": null
            })))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let result = sdk.admin.get_activity_log("entry/part?x#y").await.unwrap();
        assert_eq!(result.data.unwrap().target.id, "42");
        assert!(server.received_requests().await.unwrap()[0]
            .url
            .query()
            .is_none());
    }

    #[tokio::test]
    async fn all_activity_routes_preserve_api_errors() {
        let server = MockServer::start().await;
        for route in ["/activity_logs", "/activity_logs/filters"] {
            Mock::given(method("GET"))
                .and(path(route))
                .respond_with(ResponseTemplate::new(403).set_body_string("feature unavailable"))
                .expect(1)
                .mount(&server)
                .await;
        }
        Mock::given(method("GET"))
            .and(path("/activity_logs/missing"))
            .respond_with(ResponseTemplate::new(404).set_body_string("entry not found"))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let list = sdk
            .admin
            .list_activity_logs(&ListActivityLogsRequest::default())
            .await
            .unwrap_err();
        let filters = sdk.admin.list_activity_log_filters().await.unwrap_err();
        for error in [list, filters] {
            assert!(
                matches!(error, SdkError::Api { status, body } if status.as_u16() == 403 && body == "feature unavailable")
            );
        }
        let detail = sdk.admin.get_activity_log("missing").await.unwrap_err();
        assert!(
            matches!(detail, SdkError::Api { status, body } if status.as_u16() == 404 && body == "entry not found")
        );
    }

    #[test]
    fn required_pagination_is_not_fabricated() {
        assert!(serde_json::from_value::<ListActivityLogsResponse>(
            json!({"data": [], "error": null})
        )
        .is_err());
    }
}
