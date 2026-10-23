#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{
    CreateRequestFilterPatternRequest, CreateRequestFilterRequest, SecurityOptionsUpdate,
    UpdateRequestFilterPatternRequest, UpdateRequestFilterRequest, UpdateSecurityOptionsRequest,
};
use crate::{errors::SdkError, AdminConfig, QuicknodeSdk, SdkFullConfig};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_sdk(server: &MockServer) -> QuicknodeSdk {
    QuicknodeSdk::new(&SdkFullConfig {
        api_key: Some("fixture-key".to_string()),
        admin: Some(AdminConfig {
            base_url: Some(format!("{}/", server.uri())),
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

#[tokio::test]
async fn filters_preserve_shapes_and_empty_success_bodies() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let pattern = json!({"method": [{"prefix": "eth_"}], "jsonrpc": ["2.0"], "anything-but": [null, {"nested": [true, 7.5]}]});
    Mock::given(method("POST"))
        .and(path("/endpoints/12/security/request_filters"))
        .and(body_json(json!({"pattern": pattern})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"data": {"id": "custom", "pattern": pattern}, "error": null}),
            ),
        )
        .expect(1)
        .mount(&server)
        .await;
    let created = sdk
        .admin
        .create_request_filter_pattern(
            "12",
            &CreateRequestFilterPatternRequest {
                pattern: serde_json::from_value(pattern.clone()).unwrap(),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(created.data.unwrap().pattern).unwrap(),
        pattern
    );
    Mock::given(method("POST"))
        .and(path("/endpoints/12/security/request_filters"))
        .and(body_json(json!({"method": ["eth_chainId"]})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"id": "whitelist"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    assert!(sdk
        .admin
        .create_request_filter(
            "12",
            &CreateRequestFilterRequest {
                method: vec!["eth_chainId".into()]
            }
        )
        .await
        .unwrap()
        .data
        .unwrap()
        .pattern
        .is_none());
    Mock::given(method("GET"))
        .and(path("/endpoints/12/security/request_filters"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"data": [{"id": "custom", "pattern": pattern, "method_whitelist": false}]}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let filters = sdk.admin.get_request_filters("12").await.unwrap();
    assert!(!filters.data[0].method_whitelist);
    assert_eq!(
        serde_json::to_value(&filters.data[0].pattern).unwrap(),
        pattern
    );
    Mock::given(method("PUT"))
        .and(path("/endpoints/12/security/request_filters/custom"))
        .and(body_json(json!({"pattern": pattern})))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    sdk.admin
        .update_request_filter_pattern(
            "12",
            "custom",
            &UpdateRequestFilterPatternRequest {
                pattern: serde_json::from_value(pattern).unwrap(),
            },
        )
        .await
        .unwrap();
    Mock::given(method("PUT"))
        .and(path("/endpoints/12/security/request_filters/whitelist"))
        .and(body_json(json!({"method": ["eth_chainId"]})))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    sdk.admin
        .update_request_filter(
            "12",
            "whitelist",
            &UpdateRequestFilterRequest {
                method: Some(vec!["eth_chainId".into()]),
            },
        )
        .await
        .unwrap();
    Mock::given(method("DELETE"))
        .and(path("/endpoints/12/security/request_filters/custom"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    sdk.admin
        .delete_request_filter("12", "custom")
        .await
        .unwrap();
}

#[tokio::test]
async fn filter_errors_keep_api_status_and_body() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    Mock::given(path("/endpoints/12/security/request_filters"))
        .respond_with(ResponseTemplate::new(422).set_body_string("invalid filter"))
        .expect(2)
        .mount(&server)
        .await;
    assert!(
        matches!(sdk.admin.get_request_filters("12").await, Err(SdkError::Api { status, body }) if status.as_u16() == 422 && body == "invalid filter")
    );
    assert!(
        matches!(sdk.admin.create_request_filter_pattern("12", &CreateRequestFilterPatternRequest { pattern: Default::default() }).await, Err(SdkError::Api { status, .. }) if status.as_u16() == 422)
    );
    Mock::given(method("PUT"))
        .and(path("/endpoints/12/security/request_filters/filter"))
        .respond_with(ResponseTemplate::new(422))
        .expect(2)
        .mount(&server)
        .await;
    assert!(
        matches!(sdk.admin.update_request_filter_pattern("12", "filter", &UpdateRequestFilterPatternRequest { pattern: Default::default() }).await, Err(SdkError::Api { status, .. }) if status.as_u16() == 422)
    );
    assert!(
        matches!(sdk.admin.update_request_filter("12", "filter", &UpdateRequestFilterRequest::default()).await, Err(SdkError::Api { status, .. }) if status.as_u16() == 422)
    );
}

#[tokio::test]
async fn security_snapshots_preserve_filters_and_options() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let filters = json!([
        {"id":"structured", "method":[{"prefix":"eth_"}], "params":{"anything-but":[null, {"jsonrpc":"2.0"}]}},
        {"id":"empty", "method":[], "params":{"jsonrpc":["2.0"]}},
        {"id":"whitelist", "method":["eth_chainId"], "params":{}}
    ]);
    let security = json!({"request_filters": filters, "blocked_ips":[{"id":"ip", "ip":"192.0.2.0/24"}], "options":{"hsts":true, "cors":false, "responseLogging":true}, "tokens":[{"id":"token", "token":"fixture-secret"}], "jwts":[{"id":"jwt","public_key":"fixture-public-key","kid":"kid","name":"test"}]});
    Mock::given(method("GET"))
        .and(path("/endpoints/12/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":security})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/endpoints/12")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":{"id":"12","chain":"ethereum","network":"mainnet","http_url":"https://example.invalid","security":security}}))).expect(1).mount(&server).await;
    let response = sdk.admin.get_endpoint_security("12").await.unwrap();
    let debug = format!("{response:?}");
    assert!(!debug.contains("fixture-secret"));
    assert!(!debug.contains("fixture-public-key"));
    let data = serde_json::to_value(response.data).unwrap();
    assert_eq!(data["request_filters"], filters);
    assert_eq!(data["blocked_ips"], security["blocked_ips"]);
    assert_eq!(data["options"]["responseLogging"], true);
    let endpoint = sdk.admin.show_endpoint("12").await.unwrap().data.unwrap();
    assert_eq!(serde_json::to_value(endpoint.security).unwrap(), data);
    Mock::given(method("PATCH"))
        .and(path("/endpoints/12/security_options"))
        .and(body_json(json!({"options":{"responseLogging":"enabled"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":[]})))
        .expect(1)
        .mount(&server)
        .await;
    sdk.admin
        .update_security_options(
            "12",
            &UpdateSecurityOptionsRequest {
                options: SecurityOptionsUpdate {
                    response_logging: Some("enabled".into()),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();
}
