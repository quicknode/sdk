#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{
    BulkAddTagRequest, BulkRemoveTagRequest, BulkUpdateEndpointStatusRequest,
    CreateBlockedIpRequest, CreateClientCertRequest, CreateDomainMaskRequest, CreateIpRequest,
    CreateJwtRequest, CreateMethodRateLimitRequest, CreateReferrerRequest, CreateTagRequest,
    GetEndpointMetricsRequest, GetUsageRequest, RateLimitSettings, UpdateClientCertsRequest,
    UpdateMethodBlacklistRequest, UpdateMethodRateLimitRequest, UpdateRangeLimitsRequest,
    UpdateRateLimitsRequest,
};
use crate::{errors::SdkError, AdminConfig, QuicknodeSdk, SdkFullConfig};
use serde_json::{json, Value};
use std::collections::HashMap;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_sdk(server: &MockServer) -> QuicknodeSdk {
    QuicknodeSdk::new(&SdkFullConfig {
        api_key: Some("fixture-key".into()),
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

async fn mount_json(
    server: &MockServer,
    verb: &str,
    route: &str,
    request: Option<Value>,
    data: Value,
) {
    let mut mock = Mock::given(method(verb)).and(path(route));
    if let Some(request) = request {
        mock = mock.and(body_json(request));
    }
    mock.respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": data})))
        .expect(1)
        .mount(server)
        .await;
}

fn assert_api_error(result: Result<(), SdkError>, expected_status: u16, expected_body: &str) {
    assert!(matches!(result, Err(SdkError::Api { status, body })
        if status.as_u16() == expected_status && body == expected_body));
}

#[tokio::test]
async fn legacy_usage_preserves_large_integers_and_nullable_metadata() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let large = 4_294_967_297_i64;
    let method_row = json!({"method_name":"eth_chainId", "credits_used":large,
        "bytes":large, "archive":null, "network":null, "chain":null});
    for (route, data) in [
        (
            "by-endpoint",
            json!({"endpoints":[{"name":"fixture-endpoint", "credits_used":large,
            "rpc_credits":large, "data_credits":large, "bytes":large,
            "methods_breakdown":[method_row.clone()]}]}),
        ),
        ("by-method", json!({"methods":[method_row]})),
        (
            "by-chain",
            json!({"chains":[{"name":"ethereum", "credits_used":large,
            "rpc_credits":large, "data_credits":large, "bytes":large}]}),
        ),
        (
            "by-tag",
            json!({"tags":[{"tag_id":null, "label":"untagged", "credits_used":large,
            "requests":large, "rpc_credits":large, "data_credits":large, "bytes":large}]}),
        ),
    ] {
        mount_json(&server, "GET", &format!("/usage/rpc/{route}"), None, data).await;
    }
    let params = GetUsageRequest::default();
    let endpoints = sdk
        .admin
        .get_usage_by_endpoint(&params)
        .await
        .unwrap()
        .data
        .unwrap();
    let endpoint = &endpoints.endpoints[0];
    assert_eq!(
        (endpoint.rpc_credits, endpoint.data_credits, endpoint.bytes),
        (Some(large), Some(large), Some(large))
    );
    assert_eq!(endpoint.methods_breakdown[0].bytes, Some(large));
    let methods = sdk
        .admin
        .get_usage_by_method(&params)
        .await
        .unwrap()
        .data
        .unwrap();
    let row = &methods.methods[0];
    assert_eq!(row.bytes, Some(large));
    assert_eq!(
        (&row.archive, &row.network, &row.chain),
        (&None, &None, &None)
    );
    let chains = sdk
        .admin
        .get_usage_by_chain(&params)
        .await
        .unwrap()
        .data
        .unwrap();
    let chain = &chains.chains[0];
    assert_eq!(
        (chain.rpc_credits, chain.data_credits, chain.bytes),
        (Some(large), Some(large), Some(large))
    );
    let tags = sdk
        .admin
        .get_usage_by_tag(&params)
        .await
        .unwrap()
        .data
        .unwrap();
    let tag = &tags.tags[0];
    assert_eq!(tag.tag_id, None);
    assert_eq!(
        (tag.rpc_credits, tag.data_credits, tag.bytes),
        (Some(large), Some(large), Some(large))
    );
}

#[tokio::test]
async fn bulk_http_success_retains_each_item_failure() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let summary = json!({"total":2, "updated_count":1, "failed_count":1,
        "results":[{"id":"12", "success":true}, {"id":"missing", "success":false, "error":"Endpoint not found"}]});
    mount_json(
        &server,
        "POST",
        "/endpoints/bulk/status",
        Some(json!({"ids":["12","missing"], "status":"paused"})),
        summary.clone(),
    )
    .await;
    let mut tag_summary = summary.clone();
    tag_summary["tag"] = json!({"tag_id":7,"label":"fixture"});
    mount_json(
        &server,
        "POST",
        "/endpoints/bulk/tags",
        Some(json!({"ids":["12","missing"], "label":"fixture"})),
        tag_summary,
    )
    .await;
    mount_json(
        &server,
        "DELETE",
        "/endpoints/bulk/tags",
        Some(json!({"ids":["12","missing"], "tag_id":7})),
        summary,
    )
    .await;
    let ids = vec!["12".into(), "missing".into()];
    let status = sdk
        .admin
        .bulk_update_endpoint_status(&BulkUpdateEndpointStatusRequest {
            ids: ids.clone(),
            status: "paused".into(),
        })
        .await
        .unwrap()
        .data
        .unwrap();
    let added = sdk
        .admin
        .bulk_add_tag(&BulkAddTagRequest {
            ids: ids.clone(),
            label: "fixture".into(),
        })
        .await
        .unwrap()
        .data
        .unwrap();
    let removed = sdk
        .admin
        .bulk_remove_tag(&BulkRemoveTagRequest { ids, tag_id: 7 })
        .await
        .unwrap()
        .data
        .unwrap();
    for results in [status.results, added.results, removed.results] {
        assert!(results[0].success);
        assert_eq!(results[0].error, None);
        assert!(!results[1].success);
        assert_eq!(results[1].error.as_deref(), Some("Endpoint not found"));
    }
}

#[tokio::test]
async fn method_rate_limits_preserve_explicit_false_and_server_validation() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let limiter = json!({"id":"limit", "interval":"second", "methods":["eth_chainId"],
        "rate":5, "status":"enabled", "created":"2026-01-01T00:00:00Z", "limit_by_ip":true});
    mount_json(
        &server,
        "POST",
        "/endpoints/12/method-rate-limits",
        Some(json!({"interval":"second", "methods":["eth_chainId"], "rate":5, "limit_by_ip":true})),
        limiter.clone(),
    )
    .await;
    let created = sdk
        .admin
        .create_method_rate_limit(
            "12",
            &CreateMethodRateLimitRequest {
                interval: "second".into(),
                methods: vec!["eth_chainId".into()],
                rate: 5,
                limit_by_ip: Some(true),
            },
        )
        .await
        .unwrap()
        .data
        .unwrap();
    assert_eq!(created.limit_by_ip, Some(true));
    let mut updated = limiter.clone();
    updated["limit_by_ip"] = json!(false);
    mount_json(
        &server,
        "PATCH",
        "/endpoints/12/method-rate-limits/limit",
        Some(json!({"limit_by_ip":false})),
        updated.clone(),
    )
    .await;
    assert_eq!(
        sdk.admin
            .update_method_rate_limit(
                "12",
                "limit",
                &UpdateMethodRateLimitRequest {
                    limit_by_ip: Some(false),
                    ..Default::default()
                }
            )
            .await
            .unwrap()
            .data
            .unwrap()
            .limit_by_ip,
        Some(false)
    );
    mount_json(
        &server,
        "GET",
        "/endpoints/12/method-rate-limits",
        None,
        json!({"rate_limiters":[updated]}),
    )
    .await;
    assert_eq!(
        sdk.admin
            .get_method_rate_limits("12")
            .await
            .unwrap()
            .data
            .unwrap()
            .rate_limiters[0]
            .limit_by_ip,
        Some(false)
    );
    Mock::given(method("POST"))
        .and(path("/endpoints/12/method-rate-limits"))
        .and(body_json(
            json!({"interval":"second", "methods":["eth_chainId"], "rate":0}),
        ))
        .respond_with(ResponseTemplate::new(400).set_body_string("rate must be positive"))
        .expect(1)
        .mount(&server)
        .await;
    assert_api_error(
        sdk.admin
            .create_method_rate_limit(
                "12",
                &CreateMethodRateLimitRequest {
                    interval: "second".into(),
                    methods: vec!["eth_chainId".into()],
                    rate: 0,
                    limit_by_ip: None,
                },
            )
            .await
            .map(|_| ()),
        400,
        "rate must be positive",
    );
    Mock::given(method("PATCH"))
        .and(path("/endpoints/12/method-rate-limits/limit"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(400).set_body_string("at least one field is required"))
        .expect(1)
        .mount(&server)
        .await;
    assert_api_error(
        sdk.admin
            .update_method_rate_limit("12", "limit", &UpdateMethodRateLimitRequest::default())
            .await
            .map(|_| ()),
        400,
        "at least one field is required",
    );
}

#[tokio::test]
async fn endpoint_metrics_send_percentile_and_preserve_missing_parameter_errors() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    Mock::given(method("GET"))
        .and(path("/endpoints/12/metrics"))
        .and(|request: &wiremock::Request| {
            request.url.query() == Some("period=day&metric=response_time_over_time&percentile=95")
        })
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":[{"tag":"p95","data":[[1,20]]}]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let params = GetEndpointMetricsRequest {
        period: "day".into(),
        metric: "response_time_over_time".into(),
        percentile: Some(95),
    };
    assert_eq!(
        sdk.admin
            .get_endpoint_metrics("12", &params)
            .await
            .unwrap()
            .data[0]
            .tag,
        vec!["p95"]
    );
    Mock::given(method("GET"))
        .and(path("/endpoints/12/metrics"))
        .and(|request: &wiremock::Request| {
            request.url.query() == Some("period=day&metric=response_time_over_time")
        })
        .respond_with(ResponseTemplate::new(400).set_body_string("percentile required"))
        .expect(1)
        .mount(&server)
        .await;
    assert_api_error(
        sdk.admin
            .get_endpoint_metrics(
                "12",
                &GetEndpointMetricsRequest {
                    percentile: None,
                    ..params
                },
            )
            .await
            .map(|_| ()),
        400,
        "percentile required",
    );
}

#[tokio::test]
async fn useful_mutation_responses_preserve_records_and_wire_shapes() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let fixtures = [
        (
            "POST",
            "security/tokens",
            None,
            json!({"id":"token-id", "token":"fixture-token"}),
        ),
        (
            "POST",
            "security/referrers",
            Some(json!({"referrer":"https://example.invalid"})),
            json!({"id":"referrer-id", "url":"https://example.invalid"}),
        ),
        (
            "POST",
            "security/ips",
            Some(json!({"ip":"192.0.2.1"})),
            json!({"id":"ip-id", "ip":"192.0.2.1"}),
        ),
        (
            "POST",
            "security/domain_masks",
            Some(json!({"domain_mask":"rpc.example.invalid"})),
            json!({"id":"mask-id", "domain_mask":"rpc.example.invalid"}),
        ),
        (
            "POST",
            "security/jwts",
            Some(json!({"public_key":"fixture-public-key", "kid":"key-id", "name":"fixture"})),
            json!({"id":"jwt-id", "public_key":"fixture-public-key", "kid":"key-id", "name":"fixture"}),
        ),
        (
            "POST",
            "tags",
            Some(json!({"label":"fixture"})),
            json!({"tags":[{"id":91, "tag_id":7, "account_id":8, "endpoint_id":12, "label":"fixture", "created_at":"2026-01-01", "updated_at":"2026-01-02"}]}),
        ),
        (
            "PATCH",
            "rate-limits",
            Some(json!({"rate_limits":{"rps":10}})),
            json!({"limits":{"rps":10, "rpm":null, "rpd":null}}),
        ),
    ];
    for (verb, route, request, response) in &fixtures {
        mount_json(
            &server,
            verb,
            &format!("/endpoints/12/{route}"),
            request.clone(),
            response.clone(),
        )
        .await;
    }
    let token = sdk.admin.create_token_with_response("12").await.unwrap();
    assert!(!format!("{token:?}").contains("fixture-token"));
    let referrer = sdk
        .admin
        .create_referrer_with_response(
            "12",
            &CreateReferrerRequest {
                referrer: "https://example.invalid".into(),
            },
        )
        .await
        .unwrap();
    let ip = sdk
        .admin
        .create_ip_with_response(
            "12",
            &CreateIpRequest {
                ip: "192.0.2.1".into(),
            },
        )
        .await
        .unwrap();
    let domain = sdk
        .admin
        .create_domain_mask_with_response(
            "12",
            &CreateDomainMaskRequest {
                domain_mask: Some("rpc.example.invalid".into()),
            },
        )
        .await
        .unwrap();
    let jwt = sdk
        .admin
        .create_jwt_with_response(
            "12",
            &CreateJwtRequest {
                public_key: "fixture-public-key".into(),
                kid: "key-id".into(),
                name: "fixture".into(),
            },
        )
        .await
        .unwrap();
    assert!(!format!("{jwt:?}").contains("fixture-public-key"));
    let tag = sdk
        .admin
        .create_tag_with_response(
            "12",
            &CreateTagRequest {
                label: Some("fixture".into()),
            },
        )
        .await
        .unwrap();
    assert_ne!(tag.data.tags[0].id, tag.data.tags[0].tag_id);
    let limits = sdk
        .admin
        .update_rate_limits_with_response(
            "12",
            &UpdateRateLimitsRequest {
                rate_limits: Some(RateLimitSettings {
                    rps: Some(10),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let responses = [
        serde_json::to_value(token.data).unwrap(),
        serde_json::to_value(referrer.data).unwrap(),
        serde_json::to_value(ip.data).unwrap(),
        serde_json::to_value(domain.data).unwrap(),
        serde_json::to_value(jwt.data).unwrap(),
        serde_json::to_value(tag.data).unwrap(),
        serde_json::to_value(limits.data).unwrap(),
    ];
    for (response, (_, _, _, expected)) in responses.into_iter().zip(fixtures) {
        assert_eq!(response, expected);
    }
}

#[tokio::test]
async fn legacy_mutations_ignore_success_body_but_keep_http_failures() {
    for (status, body) in [
        (204, ""),
        (200, ""),
        (200, "not JSON"),
        (422, "invalid input"),
    ] {
        let server = MockServer::start().await;
        let sdk = make_sdk(&server);
        Mock::given(|_: &wiremock::Request| true)
            .respond_with(ResponseTemplate::new(status).set_body_string(body))
            .expect(7)
            .mount(&server)
            .await;
        let results = [
            sdk.admin.create_token("12").await,
            sdk.admin
                .create_referrer(
                    "12",
                    &CreateReferrerRequest {
                        referrer: "https://example.invalid".into(),
                    },
                )
                .await,
            sdk.admin
                .create_ip(
                    "12",
                    &CreateIpRequest {
                        ip: "192.0.2.1".into(),
                    },
                )
                .await,
            sdk.admin
                .create_domain_mask(
                    "12",
                    &CreateDomainMaskRequest {
                        domain_mask: Some("rpc.example.invalid".into()),
                    },
                )
                .await,
            sdk.admin
                .create_jwt(
                    "12",
                    &CreateJwtRequest {
                        public_key: "fixture".into(),
                        kid: "key-id".into(),
                        name: "fixture".into(),
                    },
                )
                .await,
            sdk.admin
                .create_tag(
                    "12",
                    &CreateTagRequest {
                        label: Some("fixture".into()),
                    },
                )
                .await,
            sdk.admin
                .update_rate_limits("12", &UpdateRateLimitsRequest::default())
                .await,
        ];
        for result in results {
            if status < 300 {
                result.unwrap();
            } else {
                assert_api_error(result, status, body);
            }
        }
    }
}

#[tokio::test]
async fn response_returning_mutations_keep_http_failures() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    Mock::given(|_: &wiremock::Request| true)
        .respond_with(ResponseTemplate::new(422).set_body_string("invalid input"))
        .expect(7)
        .mount(&server)
        .await;
    let results = [
        sdk.admin.create_token_with_response("12").await.map(|_| ()),
        sdk.admin
            .create_referrer_with_response("12", &CreateReferrerRequest::default())
            .await
            .map(|_| ()),
        sdk.admin
            .create_ip_with_response("12", &CreateIpRequest::default())
            .await
            .map(|_| ()),
        sdk.admin
            .create_domain_mask_with_response("12", &CreateDomainMaskRequest::default())
            .await
            .map(|_| ()),
        sdk.admin
            .create_jwt_with_response("12", &CreateJwtRequest::default())
            .await
            .map(|_| ()),
        sdk.admin
            .create_tag_with_response("12", &CreateTagRequest::default())
            .await
            .map(|_| ()),
        sdk.admin
            .update_rate_limits_with_response("12", &UpdateRateLimitsRequest::default())
            .await
            .map(|_| ()),
    ];
    for result in results {
        assert_api_error(result, 422, "invalid input");
    }
}

#[tokio::test]
async fn certificates_return_full_state_and_encode_fingerprints() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let pem = "-----BEGIN CERTIFICATE-----\nfixture-certificate\n-----END CERTIFICATE-----\n";
    let state = json!({"enabled":true, "certs":[{"fingerprint":"aa:bb", "subject":"CN=fixture", "issuer":"CN=fixture-ca", "expires_at":"2030-01-01T00:00:00Z"}]});
    mount_json(
        &server,
        "GET",
        "/endpoints/12/security/client_certs",
        None,
        state.clone(),
    )
    .await;
    mount_json(
        &server,
        "POST",
        "/endpoints/12/security/client_certs",
        Some(json!({"certificate":pem})),
        state.clone(),
    )
    .await;
    let mut disabled = state.clone();
    disabled["enabled"] = json!(false);
    mount_json(
        &server,
        "PATCH",
        "/endpoints/12/security/client_certs",
        Some(json!({"enabled":false})),
        disabled.clone(),
    )
    .await;
    let empty = json!({"enabled":false, "certs":[]});
    Mock::given(method("DELETE"))
        .and(|request: &wiremock::Request| {
            request.url.path() == "/endpoints/12/security/client_certs/aa%2Fbb%3F%23%25"
                && request.url.query().is_none()
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":empty})))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        serde_json::to_value(sdk.admin.get_client_certs("12").await.unwrap().data).unwrap(),
        state
    );
    assert_eq!(
        serde_json::to_value(
            sdk.admin
                .create_client_cert(
                    "12",
                    &CreateClientCertRequest {
                        certificate: pem.into()
                    }
                )
                .await
                .unwrap()
                .data
        )
        .unwrap(),
        state
    );
    assert_eq!(
        serde_json::to_value(
            sdk.admin
                .update_client_certs("12", &UpdateClientCertsRequest { enabled: false })
                .await
                .unwrap()
                .data
        )
        .unwrap(),
        disabled
    );
    assert_eq!(
        serde_json::to_value(
            sdk.admin
                .delete_client_cert("12", "aa/bb?#%")
                .await
                .unwrap()
                .data
        )
        .unwrap(),
        empty
    );
}

#[tokio::test]
async fn blacklist_and_range_limits_support_replacement_and_clear() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    mount_json(
        &server,
        "GET",
        "/endpoints/12/security/method_blacklist",
        None,
        json!({"methods":["eth_sendRawTransaction"]}),
    )
    .await;
    assert_eq!(
        sdk.admin
            .get_method_blacklist("12")
            .await
            .unwrap()
            .data
            .methods,
        vec!["eth_sendRawTransaction"]
    );
    for methods in [vec!["eth_chainId".to_string()], vec![]] {
        mount_json(
            &server,
            "PUT",
            "/endpoints/12/security/method_blacklist",
            Some(json!({"methods":methods})),
            json!({"methods":methods}),
        )
        .await;
        assert_eq!(
            sdk.admin
                .update_method_blacklist(
                    "12",
                    &UpdateMethodBlacklistRequest {
                        methods: methods.clone()
                    }
                )
                .await
                .unwrap()
                .data
                .methods,
            methods
        );
    }
    let supported = vec!["eth_getLogs".to_string()];
    mount_json(
        &server,
        "GET",
        "/endpoints/12/security/range_limits",
        None,
        json!({"limits":{"eth_getLogs":100}, "supported_methods":supported}),
    )
    .await;
    let initial = sdk.admin.get_range_limits("12").await.unwrap().data;
    assert_eq!(initial.limits["eth_getLogs"], 100);
    assert_eq!(initial.supported_methods, supported);
    for limits in [
        HashMap::from([("eth_getLogs".to_string(), 200)]),
        HashMap::new(),
    ] {
        mount_json(
            &server,
            "PUT",
            "/endpoints/12/security/range_limits",
            Some(json!({"limits":limits})),
            json!({"limits":limits, "supported_methods":supported}),
        )
        .await;
        let updated = sdk
            .admin
            .update_range_limits(
                "12",
                &UpdateRangeLimitsRequest {
                    limits: limits.clone(),
                },
            )
            .await
            .unwrap()
            .data;
        assert_eq!(updated.limits, limits);
        assert_eq!(updated.supported_methods, supported);
    }
}

#[tokio::test]
async fn blocked_ip_operations_preserve_cidr_and_do_not_toggle_security() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    let record = json!({"id":"blocked-id", "ip":"192.0.2.0/24"});
    mount_json(
        &server,
        "GET",
        "/endpoints/12/security/blocked_ips",
        None,
        json!([record.clone()]),
    )
    .await;
    mount_json(
        &server,
        "POST",
        "/endpoints/12/security/blocked_ips",
        Some(json!({"ip":"192.0.2.0/24"})),
        record.clone(),
    )
    .await;
    mount_json(
        &server,
        "DELETE",
        "/endpoints/12/security/blocked_ips/blocked-id",
        None,
        json!(true),
    )
    .await;
    assert_eq!(
        serde_json::to_value(&sdk.admin.get_blocked_ips("12").await.unwrap().data[0]).unwrap(),
        record
    );
    assert_eq!(
        serde_json::to_value(
            sdk.admin
                .create_blocked_ip(
                    "12",
                    &CreateBlockedIpRequest {
                        ip: "192.0.2.0/24".into()
                    }
                )
                .await
                .unwrap()
                .data
        )
        .unwrap(),
        record
    );
    assert_eq!(
        sdk.admin
            .delete_blocked_ip("12", "blocked-id")
            .await
            .unwrap()
            .data,
        Some(true)
    );
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests
        .iter()
        .all(|request| request.url.path().contains("/security/blocked_ips")));
}

#[tokio::test]
async fn new_security_operations_preserve_api_errors() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    Mock::given(|request: &wiremock::Request| {
        request.url.path() != "/endpoints/12/security/blocked_ips/missing"
    })
    .respond_with(ResponseTemplate::new(422).set_body_string("invalid security input"))
    .expect(10)
    .mount(&server)
    .await;
    Mock::given(method("DELETE"))
        .and(path("/endpoints/12/security/blocked_ips/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("blocked IP not found"))
        .expect(1)
        .mount(&server)
        .await;
    let results = [
        sdk.admin.get_client_certs("12").await.map(|_| ()),
        sdk.admin
            .create_client_cert(
                "12",
                &CreateClientCertRequest {
                    certificate: "invalid PEM".into(),
                },
            )
            .await
            .map(|_| ()),
        sdk.admin
            .update_client_certs("12", &UpdateClientCertsRequest { enabled: true })
            .await
            .map(|_| ()),
        sdk.admin
            .delete_client_cert("12", "missing")
            .await
            .map(|_| ()),
        sdk.admin.get_method_blacklist("12").await.map(|_| ()),
        sdk.admin
            .update_method_blacklist(
                "12",
                &UpdateMethodBlacklistRequest {
                    methods: vec!["invalid".into()],
                },
            )
            .await
            .map(|_| ()),
        sdk.admin.get_range_limits("12").await.map(|_| ()),
        sdk.admin
            .update_range_limits(
                "12",
                &UpdateRangeLimitsRequest {
                    limits: HashMap::from([("eth_getLogs".into(), 0)]),
                },
            )
            .await
            .map(|_| ()),
        sdk.admin.get_blocked_ips("12").await.map(|_| ()),
        sdk.admin
            .create_blocked_ip(
                "12",
                &CreateBlockedIpRequest {
                    ip: "invalid IP".into(),
                },
            )
            .await
            .map(|_| ()),
    ];
    for result in results {
        assert_api_error(result, 422, "invalid security input");
    }
    assert_api_error(
        sdk.admin
            .delete_blocked_ip("12", "missing")
            .await
            .map(|_| ()),
        404,
        "blocked IP not found",
    );
}

#[tokio::test]
async fn endpoint_rate_limits_support_independent_per_ip_updates() {
    let server = MockServer::start().await;
    let sdk = make_sdk(&server);
    for (rate_limits, limit_by_ip, body) in [
        (None, Some(true), json!({"limit_by_ip": true})),
        (None, Some(false), json!({"limit_by_ip": false})),
        (
            Some(RateLimitSettings {
                rps: Some(10),
                ..Default::default()
            }),
            None,
            json!({"rate_limits": {"rps": 10}}),
        ),
        (
            Some(RateLimitSettings {
                rps: Some(10),
                ..Default::default()
            }),
            Some(false),
            json!({"rate_limits": {"rps": 10}, "limit_by_ip": false}),
        ),
    ] {
        let params = UpdateRateLimitsRequest {
            rate_limits,
            limit_by_ip,
        };
        Mock::given(method("PATCH"))
            .and(path("/endpoints/12/rate-limits"))
            .and(body_json(body))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {"limits": {"rps": 10, "rpm": null, "rpd": null}}
            })))
            .expect(2)
            .mount(&server)
            .await;
        sdk.admin.update_rate_limits("12", &params).await.unwrap();
        let response = sdk
            .admin
            .update_rate_limits_with_response("12", &params)
            .await
            .unwrap();
        assert_eq!(response.data.limits.rps, Some(10));
    }
    Mock::given(method("PATCH"))
        .and(path("/endpoints/12/rate-limits"))
        .and(body_json(json!({})))
        .respond_with(
            ResponseTemplate::new(400).set_body_string("at least one property is required"),
        )
        .expect(2)
        .mount(&server)
        .await;
    assert_api_error(
        sdk.admin
            .update_rate_limits("12", &UpdateRateLimitsRequest::default())
            .await,
        400,
        "at least one property is required",
    );
    assert_api_error(
        sdk.admin
            .update_rate_limits_with_response("12", &UpdateRateLimitsRequest::default())
            .await
            .map(|_| ()),
        400,
        "at least one property is required",
    );
}
