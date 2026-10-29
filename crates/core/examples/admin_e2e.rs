use serde_json::json;
use std::collections::HashMap;

use quicknode_sdk::{
    admin::{
        BulkAddTagRequest, BulkRemoveTagRequest, BulkUpdateEndpointStatusRequest,
        CreateBlockedIpRequest, CreateClientCertRequest, CreateDomainMaskRequest,
        CreateEndpointRequest, CreateIpRequest, CreateJwtRequest, CreateMethodRateLimitRequest,
        CreateOrUpdateIpCustomHeaderRequest, CreateReferrerRequest,
        CreateRequestFilterPatternRequest, CreateRequestFilterRequest, CreateTagRequest,
        CreateTeamRequest, GetAccountMetricsRequest, GetEndpointErrorsSummaryRequest,
        GetEndpointLogsRequest, GetEndpointMetricsRequest, GetEndpointsRequest,
        GetErrorsSummaryRequest, GetTrafficSummaryRequest, GetTransactionRequest,
        GetUsageCreditsSummaryRequest, GetUsageRequest, GetUsageRpcSummaryRequest,
        GetUsageStreamsSummaryRequest, GetUsageWebhooksSummaryRequest, InviteTeamMemberRequest,
        ListActivityLogsRequest, ListTransactionsRequest, RateLimitSettings, RenameTagRequest,
        SecurityOptionsUpdate, UpdateEndpointRequest, UpdateEndpointStatusRequest,
        UpdateMethodBlacklistRequest, UpdateMethodRateLimitRequest, UpdateRangeLimitsRequest,
        UpdateRateLimitsRequest, UpdateRequestFilterPatternRequest, UpdateRequestFilterRequest,
        UpdateSecurityOptionsRequest, UpdateTeamEndpointsRequest,
    },
    errors::SdkError,
    AdminConfig, HttpConfig, QuicknodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() -> Result<(), SdkError> {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    let run_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() % 1_000_000)
        .unwrap_or(0);

    // --- Read-only globals ---

    match qn.admin.list_chains().await {
        Ok(resp) => println!("list_chains: {} chains", resp.data.len()),
        Err(e) => eprintln!("list_chains error: {e}"),
    }

    match qn
        .admin
        .get_endpoints(&GetEndpointsRequest::builder().limit(5).build())
        .await
    {
        Ok(resp) => println!("get_endpoints: {} endpoints", resp.data.len()),
        Err(e) => eprintln!("get_endpoints error: {e}"),
    }

    match qn.admin.get_usage(&GetUsageRequest::default()).await {
        Ok(resp) => println!("get_usage: {:?}", resp.data),
        Err(e) => eprintln!("get_usage error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    match qn
        .admin
        .get_usage_by_endpoint(&GetUsageRequest::default())
        .await
    {
        Ok(resp) => {
            for row in resp.data.expect("usage payload").endpoints {
                if let Some(value) = row.rpc_credits {
                    assert!(value >= 0);
                }
                if let Some(value) = row.data_credits {
                    assert!(value >= 0);
                }
                if let Some(value) = row.bytes {
                    assert!(value >= 0);
                }
                for method in row.methods_breakdown {
                    if let Some(value) = method.bytes {
                        assert!(value >= 0);
                    }
                }
            }
            println!("get_usage_by_endpoint: fields retained");
        }
        Err(e) => eprintln!("get_usage_by_endpoint error: {e}"),
    }

    match qn
        .admin
        .get_usage_by_method(&GetUsageRequest::default())
        .await
    {
        Ok(resp) => {
            for row in resp.data.expect("usage payload").methods {
                if let Some(value) = row.bytes {
                    assert!(value >= 0);
                }
            }
            println!("get_usage_by_method: fields retained");
        }
        Err(e) => eprintln!("get_usage_by_method error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    match qn
        .admin
        .get_usage_by_chain(&GetUsageRequest::default())
        .await
    {
        Ok(resp) => {
            for row in resp.data.expect("usage payload").chains {
                if let Some(value) = row.rpc_credits {
                    assert!(value >= 0);
                }
                if let Some(value) = row.data_credits {
                    assert!(value >= 0);
                }
                if let Some(value) = row.bytes {
                    assert!(value >= 0);
                }
            }
            println!("get_usage_by_chain: fields retained");
        }
        Err(e) => eprintln!("get_usage_by_chain error: {e}"),
    }

    match qn.admin.get_usage_by_tag(&GetUsageRequest::default()).await {
        Ok(resp) => {
            for row in resp.data.expect("usage payload").tags {
                if let Some(value) = row.rpc_credits {
                    assert!(value >= 0);
                }
                if let Some(value) = row.data_credits {
                    assert!(value >= 0);
                }
                if let Some(value) = row.bytes {
                    assert!(value >= 0);
                }
            }
            println!("get_usage_by_tag: fields retained");
        }
        Err(e) => eprintln!("get_usage_by_tag error: {e}"),
    }

    match qn.admin.list_tags().await {
        Ok(resp) => println!(
            "list_tags: {} tags",
            resp.data.map(|d| d.tags.len()).unwrap_or(0)
        ),
        Err(e) => eprintln!("list_tags error: {e}"),
    }

    match qn
        .admin
        .get_account_metrics(&GetAccountMetricsRequest {
            period: "day".to_string(),
            metric: "requests".to_string(),
            percentile: None,
        })
        .await
    {
        Ok(resp) => {
            let first = resp
                .data
                .first()
                .map(|m| m.tag.join(":"))
                .unwrap_or_else(|| "<none>".to_string());
            println!(
                "get_account_metrics: {} series, first tag: {first}",
                resp.data.len()
            );
        }
        Err(e) => eprintln!("get_account_metrics error: {e}"),
    }

    match qn.admin.list_invoices().await {
        Ok(resp) => println!("list_invoices: {:?}", resp.data),
        Err(e) => eprintln!("list_invoices error: {e}"),
    }

    match qn.admin.list_payments().await {
        Ok(resp) => println!("list_payments: {:?}", resp.data),
        Err(e) => eprintln!("list_payments error: {e}"),
    }

    match qn.admin.list_teams().await {
        Ok(resp) => println!("list_teams: {} teams", resp.data.len()),
        Err(e) => eprintln!("list_teams error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // --- Create endpoint ---

    let endpoint_id = match qn
        .admin
        .create_endpoint(
            &CreateEndpointRequest::builder()
                .chain("ethereum".to_string())
                .network("mainnet".to_string())
                .build(),
        )
        .await
    {
        Ok(resp) => {
            println!("create_endpoint: {} ({})", resp.data.id, resp.data.http_url);
            resp.data.id
        }
        Err(e) => {
            return Err(e);
        }
    };

    // --- Endpoint CRUD ---

    match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => println!("show_endpoint: {:?}", resp.data.map(|e| e.id)),
        Err(e) => eprintln!("show_endpoint error: {e}"),
    }

    match qn
        .admin
        .update_endpoint(
            &endpoint_id,
            &UpdateEndpointRequest {
                label: Some("sdk-example".to_string()),
            },
        )
        .await
    {
        Ok(()) => println!("update_endpoint: ok"),
        Err(e) => eprintln!("update_endpoint error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    match qn
        .admin
        .update_endpoint_status(
            &endpoint_id,
            &UpdateEndpointStatusRequest {
                status: "paused".to_string(),
            },
        )
        .await
    {
        Ok(resp) => println!("update_endpoint_status paused: {:?}", resp.data),
        Err(e) => eprintln!("update_endpoint_status paused error: {e}"),
    }

    match qn
        .admin
        .update_endpoint_status(
            &endpoint_id,
            &UpdateEndpointStatusRequest {
                status: "active".to_string(),
            },
        )
        .await
    {
        Ok(resp) => println!("update_endpoint_status active: {:?}", resp.data),
        Err(e) => eprintln!("update_endpoint_status active error: {e}"),
    }

    // --- Tags ---

    let created_tag_label = format!("sdk-tag-{run_suffix}");
    match qn
        .admin
        .create_tag_with_response(
            &endpoint_id,
            &CreateTagRequest {
                label: Some(created_tag_label.clone()),
            },
        )
        .await
    {
        Ok(resp) => {
            let created_tags: Vec<_> = resp
                .data
                .tags
                .into_iter()
                .filter(|tag| tag.label == created_tag_label)
                .collect();
            assert!(
                !created_tags.is_empty(),
                "created tag missing from response"
            );
            assert!(
                created_tags
                    .iter()
                    .all(|tag| tag.endpoint_id.to_string() == endpoint_id),
                "created tag belongs to another endpoint"
            );
            for tag in created_tags {
                qn.admin
                    .delete_tag(&endpoint_id, &tag.tag_id.to_string())
                    .await
                    .expect("detach created tag");
                qn.admin
                    .delete_account_tag(i32::try_from(tag.tag_id).expect("tag ID fits API input"))
                    .await
                    .expect("delete created account tag");
            }
            println!("create_tag_with_response and cleanup: ok");
        }
        Err(e) => return Err(e),
    }

    // --- Logs & metrics ---

    match qn
        .admin
        .get_endpoint_logs(
            &endpoint_id,
            &GetEndpointLogsRequest {
                from: "2025-01-01T00:00:00Z".to_string(),
                to: "2025-01-02T00:00:00Z".to_string(),
                ..Default::default()
            },
        )
        .await
    {
        Ok(resp) => println!("get_endpoint_logs: {} entries", resp.data.len()),
        Err(e) => eprintln!("get_endpoint_logs error: {e}"),
    }

    match qn
        .admin
        .get_endpoint_metrics(
            &endpoint_id,
            &GetEndpointMetricsRequest {
                period: "day".to_string(),
                metric: "response_time_over_time".to_string(),
                percentile: Some(95),
            },
        )
        .await
    {
        Ok(resp) => {
            let first = resp
                .data
                .first()
                .map(|m| m.tag.join(":"))
                .unwrap_or_else(|| "<none>".to_string());
            println!(
                "get_endpoint_metrics: {} series, first tag: {first}",
                resp.data.len()
            );
        }
        Err(e) => return Err(e),
    }

    // --- Security options ---

    match qn.admin.get_security_options(&endpoint_id).await {
        Ok(resp) => println!("get_security_options: {} options", resp.data.len()),
        Err(e) => eprintln!("get_security_options error: {e}"),
    }

    match qn.admin.get_endpoint_security(&endpoint_id).await {
        Ok(resp) => println!("get_endpoint_security: {:?}", resp.data.is_some()),
        Err(e) => eprintln!("get_endpoint_security error: {e}"),
    }

    match qn
        .admin
        .update_security_options(
            &endpoint_id,
            &UpdateSecurityOptionsRequest {
                options: SecurityOptionsUpdate {
                    tokens: Some("enabled".to_string()),
                    ..Default::default()
                },
            },
        )
        .await
    {
        Ok(resp) => println!("update_security_options: {} options", resp.data.len()),
        Err(e) => eprintln!("update_security_options error: {e}"),
    }

    // --- Token ---

    match qn.admin.create_token_with_response(&endpoint_id).await {
        Ok(resp) => {
            let deletion = qn.admin.delete_token(&endpoint_id, &resp.data.id).await;
            assert!(!resp.data.token.is_empty());
            assert!(!format!("{resp:?}").contains(&resp.data.token));
            deletion.expect("delete created token");
            println!("create_token_with_response and cleanup: ok");
        }
        Err(e) => return Err(e),
    }

    // --- Referrer ---

    match qn
        .admin
        .create_referrer_with_response(
            &endpoint_id,
            &CreateReferrerRequest {
                referrer: "https://example.com".to_string(),
            },
        )
        .await
    {
        Ok(resp) => {
            let deletion = qn.admin.delete_referrer(&endpoint_id, &resp.data.id).await;
            assert_eq!(resp.data.url, "https://example.com");
            deletion.expect("delete created referrer");
            println!("create_referrer_with_response and cleanup: ok");
        }
        Err(e) => return Err(e),
    }

    // --- IP allowlist ---

    match qn
        .admin
        .create_ip_with_response(
            &endpoint_id,
            &CreateIpRequest {
                ip: "192.0.2.1".to_string(),
            },
        )
        .await
    {
        Ok(resp) => {
            let deletion = qn.admin.delete_ip(&endpoint_id, &resp.data.id).await;
            assert_eq!(resp.data.ip, "192.0.2.1");
            deletion.expect("delete created ip");
            println!("create_ip_with_response and cleanup: ok");
        }
        Err(e) => return Err(e),
    }

    // --- Domain mask ---

    match qn
        .admin
        .create_domain_mask_with_response(
            &endpoint_id,
            &CreateDomainMaskRequest {
                domain_mask: Some("example.com".to_string()),
            },
        )
        .await
    {
        Ok(resp) => {
            let deletion = qn
                .admin
                .delete_domain_mask(&endpoint_id, &resp.data.id)
                .await;
            assert_eq!(resp.data.domain_mask, "example.com");
            deletion.expect("delete created domain_mask");
            println!("create_domain_mask_with_response and cleanup: ok");
        }
        Err(e) => return Err(e),
    }

    // --- JWT validation ---

    match qn
        .admin
        .create_jwt_with_response(
            &endpoint_id,
            &CreateJwtRequest {
                public_key: "-----BEGIN PUBLIC KEY-----\nPLACEHOLDER\n-----END PUBLIC KEY-----"
                    .to_string(),
                kid: "kid1".to_string(),
                name: "example-jwt".to_string(),
            },
        )
        .await
    {
        Err(SdkError::Api { status, body }) => {
            assert!([400, 422].contains(&status.as_u16()));
            assert!(!body.is_empty());
            println!("create_jwt_with_response: invalid key rejected");
        }
        Ok(resp) => {
            qn.admin
                .delete_jwt(&endpoint_id, &resp.data.id)
                .await
                .expect("delete unexpected JWT");
            expect_api_error(Ok::<_, SdkError>(resp), 422);
        }
        Err(error) => assert!(
            matches!(error, SdkError::Api { .. }),
            "expected JWT API validation error: {error}"
        ),
    }

    // --- Request filter ---

    let rf_id = match qn
        .admin
        .create_request_filter(
            &endpoint_id,
            &CreateRequestFilterRequest {
                method: vec!["eth_getBalance".to_string()],
            },
        )
        .await
    {
        Ok(resp) => {
            println!("create_request_filter: {:?}", resp.data);
            resp.data.map(|d| d.id)
        }
        Err(e) => {
            eprintln!("create_request_filter error: {e}");
            None
        }
    };

    if let Some(rf_id) = rf_id {
        match qn
            .admin
            .update_request_filter(
                &endpoint_id,
                &rf_id,
                &UpdateRequestFilterRequest {
                    method: Some(vec!["eth_call".to_string()]),
                },
            )
            .await
        {
            Ok(()) => println!("update_request_filter: ok"),
            Err(e) => eprintln!("update_request_filter error: {e}"),
        }

        match qn.admin.delete_request_filter(&endpoint_id, &rf_id).await {
            Ok(()) => println!("delete_request_filter: ok"),
            Err(e) => eprintln!("delete_request_filter error: {e}"),
        }
    }

    // --- IP custom header ---

    match qn
        .admin
        .create_or_update_ip_custom_header(
            &endpoint_id,
            &CreateOrUpdateIpCustomHeaderRequest {
                header_name: "X-Custom-Header".to_string(),
            },
        )
        .await
    {
        Ok(resp) => println!("create_or_update_ip_custom_header: {:?}", resp.data),
        Err(e) => eprintln!("create_or_update_ip_custom_header error: {e}"),
    }

    match qn.admin.delete_ip_custom_header(&endpoint_id).await {
        Ok(resp) => println!("delete_ip_custom_header: {:?}", resp.data),
        Err(e) => eprintln!("delete_ip_custom_header error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // --- Rate limits ---

    match qn.admin.get_rate_limits(&endpoint_id).await {
        Ok(resp) => println!("get_rate_limits before PATCH: {:?}", resp.data),
        Err(e) => eprintln!("get_rate_limits error: {e}"),
    }

    match qn
        .admin
        .update_rate_limits_with_response(
            &endpoint_id,
            &UpdateRateLimitsRequest {
                rate_limits: Some(RateLimitSettings {
                    rps: Some(3),
                    ..Default::default()
                }),
                limit_by_ip: Some(true),
            },
        )
        .await
    {
        Ok(resp) => {
            assert_eq!(resp.data.limits.rps, Some(3));
            println!("update_rate_limits_with_response: effective override retained");
        }
        Err(e) => return Err(e),
    }

    qn.admin
        .update_rate_limits(
            &endpoint_id,
            &UpdateRateLimitsRequest {
                limit_by_ip: Some(false),
                ..Default::default()
            },
        )
        .await?;

    match qn.admin.get_rate_limits(&endpoint_id).await {
        Ok(resp) => println!("get_rate_limits after PATCH: {:?}", resp.data),
        Err(e) => eprintln!("get_rate_limits error: {e}"),
    }

    match qn.admin.get_endpoint_urls(&endpoint_id).await {
        Ok(resp) => println!("get_endpoint_urls: {:?}", resp.data),
        Err(e) => eprintln!("get_endpoint_urls error: {e}"),
    }

    match qn.admin.get_method_rate_limits(&endpoint_id).await {
        Ok(resp) => println!("get_method_rate_limits: {:?}", resp.data),
        Err(e) => eprintln!("get_method_rate_limits error: {e}"),
    }

    let mrl_id = match qn
        .admin
        .create_method_rate_limit(
            &endpoint_id,
            &CreateMethodRateLimitRequest {
                interval: "second".to_string(),
                methods: vec!["eth_call".to_string()],
                rate: 5,
                limit_by_ip: Some(true),
            },
        )
        .await
    {
        Ok(resp) => resp.data.map(|d| {
            assert_eq!(d.limit_by_ip, Some(true));
            d.id
        }),
        Err(e) => return Err(e),
    };

    if let Some(mrl_id) = mrl_id {
        match qn
            .admin
            .update_method_rate_limit(
                &endpoint_id,
                &mrl_id,
                &UpdateMethodRateLimitRequest {
                    rate: Some(10),
                    limit_by_ip: Some(false),
                    ..Default::default()
                },
            )
            .await
        {
            Ok(resp) => assert_eq!(
                resp.data.expect("updated method limiter").limit_by_ip,
                Some(false)
            ),
            Err(e) => return Err(e),
        }

        match qn
            .admin
            .delete_method_rate_limit(&endpoint_id, &mrl_id)
            .await
        {
            Ok(()) => println!("delete_method_rate_limit: ok"),
            Err(e) => eprintln!("delete_method_rate_limit error: {e}"),
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // --- Multichain ---

    match qn.admin.enable_multichain(&endpoint_id).await {
        Ok(()) => println!("enable_multichain: ok"),
        Err(e) => eprintln!("enable_multichain error: {e}"),
    }

    match qn.admin.disable_multichain(&endpoint_id).await {
        Ok(()) => println!("disable_multichain: ok"),
        Err(e) => eprintln!("disable_multichain error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // --- Bulk endpoint ops ---

    match qn
        .admin
        .bulk_update_endpoint_status(&BulkUpdateEndpointStatusRequest {
            ids: vec![endpoint_id.clone(), "0".to_string()],
            status: "paused".to_string(),
        })
        .await
    {
        Ok(resp) => {
            let results = resp.data.expect("bulk results").results;
            let failed = results
                .iter()
                .find(|row| row.id == "0")
                .expect("missing endpoint result");
            assert!(!failed.success);
            assert!(failed.error.as_ref().is_some_and(|error| !error.is_empty()));
            let succeeded = results
                .iter()
                .find(|row| row.id == endpoint_id)
                .expect("created endpoint result");
            assert!(succeeded.success);
            assert!(succeeded.error.is_none());
        }
        Err(e) => return Err(e),
    }

    match qn
        .admin
        .bulk_update_endpoint_status(&BulkUpdateEndpointStatusRequest {
            ids: vec![endpoint_id.clone()],
            status: "active".to_string(),
        })
        .await
    {
        Ok(resp) => println!("bulk_update_endpoint_status active: {:?}", resp.data),
        Err(e) => eprintln!("bulk_update_endpoint_status active error: {e}"),
    }

    // --- Account-level tags (via bulk_add/remove + rename/delete) ---

    let bulk_tag_id = match qn
        .admin
        .bulk_add_tag(&BulkAddTagRequest {
            ids: vec![endpoint_id.clone()],
            label: format!("sdk-bulk-tag-{run_suffix}"),
        })
        .await
    {
        Ok(resp) => {
            println!("bulk_add_tag: {:?}", resp.data);
            resp.data.map(|d| d.tag.tag_id)
        }
        Err(e) => {
            eprintln!("bulk_add_tag error: {e}");
            None
        }
    };

    if let Some(tag_id) = bulk_tag_id {
        match qn
            .admin
            .rename_tag(
                tag_id,
                &RenameTagRequest {
                    label: format!("sdk-renamed-{run_suffix}"),
                },
            )
            .await
        {
            Ok(resp) => println!("rename_tag: {:?}", resp.data),
            Err(e) => eprintln!("rename_tag error: {e}"),
        }

        match qn
            .admin
            .bulk_remove_tag(&BulkRemoveTagRequest {
                ids: vec![endpoint_id.clone()],
                tag_id,
            })
            .await
        {
            Ok(resp) => println!("bulk_remove_tag: {:?}", resp.data),
            Err(e) => eprintln!("bulk_remove_tag error: {e}"),
        }

        match qn.admin.delete_account_tag(tag_id).await {
            Ok(resp) => println!("delete_account_tag: {:?}", resp.data),
            Err(e) => eprintln!("delete_account_tag error: {e}"),
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // --- Teams ---

    let team_id = match qn
        .admin
        .create_team(&CreateTeamRequest {
            name: "sdk-example-team".to_string(),
        })
        .await
    {
        Ok(resp) => {
            println!("create_team: {:?}", resp.data);
            resp.data.map(|d| d.id)
        }
        Err(e) => {
            eprintln!("create_team error: {e}");
            None
        }
    };

    if let Some(team_id) = team_id {
        match qn.admin.get_team(team_id).await {
            Ok(resp) => println!("get_team: {:?}", resp.data.map(|d| d.name)),
            Err(e) => eprintln!("get_team error: {e}"),
        }

        match qn.admin.list_team_endpoints(team_id).await {
            Ok(resp) => println!("list_team_endpoints: {} endpoints", resp.data.len()),
            Err(e) => eprintln!("list_team_endpoints error: {e}"),
        }

        match qn
            .admin
            .update_team_endpoints(
                team_id,
                &UpdateTeamEndpointsRequest {
                    endpoint_ids: vec![endpoint_id.clone()],
                },
            )
            .await
        {
            Ok(resp) => println!("update_team_endpoints: {:?}", resp.data),
            Err(e) => eprintln!("update_team_endpoints error: {e}"),
        }

        match qn
            .admin
            .invite_team_member(
                team_id,
                &InviteTeamMemberRequest {
                    email: "placeholder@example.com".to_string(),
                    full_name: Some("Placeholder User".to_string()),
                    role: Some("viewer".to_string()),
                },
            )
            .await
        {
            Ok(resp) => println!("invite_team_member: {:?}", resp.data),
            Err(e) => eprintln!("invite_team_member error: {e}"),
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Detach endpoints before deleting the team so the endpoint remains
        // account-owned and archive_endpoint works below.
        match qn
            .admin
            .update_team_endpoints(
                team_id,
                &UpdateTeamEndpointsRequest {
                    endpoint_ids: vec![],
                },
            )
            .await
        {
            Ok(resp) => println!("update_team_endpoints detach: {:?}", resp.data),
            Err(e) => eprintln!("update_team_endpoints detach error: {e}"),
        }

        match qn.admin.delete_team(team_id).await {
            Ok(resp) => println!("delete_team: {:?}", resp.data),
            Err(e) => eprintln!("delete_team error: {e}"),
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let checks_sdk = QuicknodeSdk::new(&config)?;
    let checks_endpoint_id = endpoint_id.clone();
    let checks = tokio::spawn(async move {
        let security = aligned_security(&checks_sdk, &checks_endpoint_id).await;
        let reads = aligned_reads(&checks_sdk, &checks_endpoint_id).await;
        (security, reads)
    })
    .await;

    // --- Cleanup endpoint ---

    match qn.admin.archive_endpoint(&endpoint_id).await {
        Ok(()) => println!("archive_endpoint: ok"),
        Err(e) => eprintln!("archive_endpoint error: {e}"),
    }

    let (security_result, reads_result) =
        checks.expect("Admin checks failed after endpoint cleanup");
    security_result?;
    reads_result?;

    // --- Error handling ---

    expect_api_error(qn.admin.show_endpoint("does-not-exist").await, 404);
    expect_api_error(
        qn.admin
            .delete_rate_limit_override("does-not-exist", "00000000-0000-0000-0000-000000000000")
            .await,
        404,
    );

    // Custom headers smoke test — construct an SDK that overrides User-Agent
    // and adds a correlation header. We don't verify what reaches the wire
    // here; we just confirm the config is accepted and an API call succeeds.
    let mut custom_headers = std::collections::HashMap::new();
    custom_headers.insert("User-Agent".to_string(), "qn-e2e-rust/1.0".to_string());
    custom_headers.insert("X-E2E-Correlation".to_string(), "rust-smoke".to_string());
    let with_headers = SdkFullConfig {
        api_key: config.api_key.clone(),
        http: Some(HttpConfig {
            timeout_secs: None,
            pool_max_idle_per_host: None,
            headers: Some(custom_headers),
        }),
        admin: config.admin.clone(),
        streams: None,
        webhooks: None,
        kvstore: None,
        sql: None,
        rpc: None,
    };
    let headered = QuicknodeSdk::new(&with_headers).expect("build sdk with custom headers");
    match headered
        .admin
        .get_endpoints(&GetEndpointsRequest::builder().limit(1).build())
        .await
    {
        Ok(_) => println!("custom-headers smoke: ok"),
        Err(e) => eprintln!("custom-headers smoke error: {e}"),
    }

    // 2) Timeout path — unreachable base URL + 1s timeout forces a timeout
    // from reqwest, which maps to SdkError::Http with http_kind() == Timeout.
    let blackhole = SdkFullConfig {
        api_key: config.api_key.clone(),
        http: Some(HttpConfig {
            timeout_secs: Some(1),
            pool_max_idle_per_host: None,
            headers: None,
        }),
        admin: Some(AdminConfig {
            base_url: Some("http://10.255.255.1/".to_string()),
        }),
        streams: None,
        webhooks: None,
        kvstore: None,
        sql: None,
        rpc: None,
    };
    let tiny = QuicknodeSdk::new(&blackhole).expect("build tiny sdk");
    let timeout_error = tiny
        .admin
        .get_endpoints(&GetEndpointsRequest::default())
        .await
        .expect_err("expected timeout, got success");
    assert!(
        matches!(
            timeout_error.http_kind(),
            Some(quicknode_sdk::errors::HttpKind::Timeout)
        ),
        "expected timeout, got {timeout_error}"
    );
    println!("timed out as expected");
    Ok(())
}

fn feature_access<T>(operation: &str, result: Result<T, SdkError>) -> Result<Option<T>, SdkError> {
    match result {
        Ok(response) => Ok(Some(response)),
        Err(SdkError::Api { status, body }) if status.as_u16() == 403 => {
            assert!(!body.is_empty(), "{operation}: empty 403 response");
            println!("{operation}: skipped because feature access returned 403");
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn expect_api_error<T>(result: Result<T, SdkError>, expected_status: u16) {
    match result {
        Err(SdkError::Api { status, body }) => {
            assert_eq!(status.as_u16(), expected_status);
            assert!(!body.is_empty());
        }
        unexpected => assert!(
            matches!(unexpected, Err(SdkError::Api { .. })),
            "expected API {expected_status}"
        ),
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn aligned_security(qn: &QuicknodeSdk, endpoint_id: &str) -> Result<(), SdkError> {
    if let Some(certificates) = feature_access(
        "client certificates",
        qn.admin.get_client_certs(endpoint_id).await,
    )? {
        println!(
            "get_client_certs: {} certificates",
            certificates.data.certs.len()
        );
        expect_api_error(
            qn.admin
                .create_client_cert(
                    endpoint_id,
                    &CreateClientCertRequest {
                        certificate: "invalid certificate".into(),
                    },
                )
                .await,
            422,
        );
        expect_api_error(
            qn.admin
                .delete_client_cert(endpoint_id, "missing-fingerprint")
                .await,
            422,
        );
    }

    let original_blacklist = qn
        .admin
        .get_method_blacklist(endpoint_id)
        .await?
        .data
        .methods;
    let replaced = qn
        .admin
        .update_method_blacklist(
            endpoint_id,
            &UpdateMethodBlacklistRequest {
                methods: vec!["eth_sendRawTransaction".into()],
            },
        )
        .await;
    let cleared = qn
        .admin
        .update_method_blacklist(
            endpoint_id,
            &UpdateMethodBlacklistRequest { methods: vec![] },
        )
        .await;
    qn.admin
        .update_method_blacklist(
            endpoint_id,
            &UpdateMethodBlacklistRequest {
                methods: original_blacklist,
            },
        )
        .await?;
    assert_eq!(replaced?.data.methods, vec!["eth_sendRawTransaction"]);
    assert!(cleared?.data.methods.is_empty());

    let original_ranges = qn.admin.get_range_limits(endpoint_id).await?.data;
    if let Some(method) = original_ranges.supported_methods.first() {
        let limits = HashMap::from([(method.clone(), 100)]);
        let replaced = qn
            .admin
            .update_range_limits(
                endpoint_id,
                &UpdateRangeLimitsRequest {
                    limits: limits.clone(),
                },
            )
            .await;
        let cleared = qn
            .admin
            .update_range_limits(
                endpoint_id,
                &UpdateRangeLimitsRequest {
                    limits: HashMap::new(),
                },
            )
            .await;
        qn.admin
            .update_range_limits(
                endpoint_id,
                &UpdateRangeLimitsRequest {
                    limits: original_ranges.limits,
                },
            )
            .await?;
        assert_eq!(replaced?.data.limits, limits);
        assert!(cleared?.data.limits.is_empty());
    }

    let before = qn
        .admin
        .get_endpoint_security(endpoint_id)
        .await?
        .data
        .expect("security snapshot");
    let ips_enabled = before.options.as_ref().and_then(|options| options.ips) == Some(true);
    let blocked = qn.admin.get_blocked_ips(endpoint_id).await?;
    println!("get_blocked_ips: {} visible records", blocked.data.len());
    if ips_enabled {
        let created = qn
            .admin
            .create_blocked_ip(
                endpoint_id,
                &CreateBlockedIpRequest {
                    ip: "192.0.2.0/24".into(),
                },
            )
            .await?;
        let listed = qn.admin.get_blocked_ips(endpoint_id).await;
        let deleted = qn
            .admin
            .delete_blocked_ip(endpoint_id, &created.data.id)
            .await;
        assert_eq!(created.data.ip, "192.0.2.0/24");
        assert!(listed?
            .data
            .iter()
            .any(|record| record.id == created.data.id));
        assert_eq!(deleted?.data, Some(true));
    } else {
        println!("blocked-IP create/delete skipped: IP security is disabled; leave its setting unchanged");
    }
    expect_api_error(
        qn.admin
            .delete_blocked_ip(endpoint_id, "missing-blocked-ip")
            .await,
        404,
    );

    let pattern = HashMap::from([
        ("method".to_string(), json!([{"prefix":"eth_"}])),
        ("jsonrpc".to_string(), json!(["2.0"])),
    ]);
    let created = qn
        .admin
        .create_request_filter_pattern(
            endpoint_id,
            &CreateRequestFilterPatternRequest {
                pattern: pattern.clone(),
            },
        )
        .await?
        .data
        .expect("created filter");
    let replacement = HashMap::from([
        ("method".to_string(), json!([{"prefix":"eth_get"}])),
        ("jsonrpc".to_string(), json!(["2.0"])),
    ]);
    let updated = qn
        .admin
        .update_request_filter_pattern(
            endpoint_id,
            &created.id,
            &UpdateRequestFilterPatternRequest {
                pattern: replacement.clone(),
            },
        )
        .await;
    let listed = qn.admin.get_request_filters(endpoint_id).await;
    let security = qn.admin.get_endpoint_security(endpoint_id).await;
    let endpoint = qn.admin.show_endpoint(endpoint_id).await;
    let deleted = qn
        .admin
        .delete_request_filter(endpoint_id, &created.id)
        .await;
    assert_eq!(created.pattern, Some(pattern));
    updated?;
    deleted?;
    let listed = listed?;
    let listed_filter = listed
        .data
        .iter()
        .find(|filter| filter.id == created.id)
        .expect("listed custom filter");
    assert!(!listed_filter.method_whitelist);
    assert_eq!(listed_filter.pattern, replacement);
    for snapshot in [
        security?.data,
        endpoint?.data.and_then(|endpoint| endpoint.security),
    ] {
        let snapshot = snapshot.expect("security snapshot");
        let filters = snapshot.request_filters.expect("snapshot filters");
        let filter = filters
            .iter()
            .find(|filter| filter.id == created.id)
            .expect("snapshot custom filter");
        assert_eq!(json!(filter.method), replacement["method"]);
        assert_eq!(filter.params["jsonrpc"], replacement["jsonrpc"]);
        assert_eq!(
            snapshot.options.and_then(|options| options.ips),
            before.options.as_ref().and_then(|options| options.ips)
        );
    }
    expect_api_error(qn.admin.get_request_filters("0").await, 404);
    expect_api_error(
        qn.admin
            .get_endpoint_metrics(
                endpoint_id,
                &GetEndpointMetricsRequest {
                    period: "day".into(),
                    metric: "response_time_over_time".into(),
                    percentile: None,
                },
            )
            .await,
        400,
    );
    expect_api_error(
        qn.admin
            .create_method_rate_limit(
                endpoint_id,
                &CreateMethodRateLimitRequest {
                    interval: "second".into(),
                    methods: vec!["eth_call".into()],
                    rate: 0,
                    limit_by_ip: None,
                },
            )
            .await,
        400,
    );
    Ok(())
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn aligned_reads(qn: &QuicknodeSdk, endpoint_id: &str) -> Result<(), SdkError> {
    if let Some(response) = feature_access(
        "credits summary",
        qn.admin
            .get_usage_credits_summary(&GetUsageCreditsSummaryRequest {
                last: Some("1h".into()),
                ..Default::default()
            })
            .await,
    )? {
        assert!(response.data.totals.credits.is_finite());
    }
    if let Some(response) = feature_access(
        "RPC summary",
        qn.admin
            .get_usage_rpc_summary(&GetUsageRpcSummaryRequest {
                last: Some("1h".into()),
                group_by: Some(vec!["method".into(), "endpoint".into()]),
                limit: Some(1),
                ..Default::default()
            })
            .await,
    )? {
        assert!(response.data.totals.requests.is_finite());
        for row in response.data.breakdown.as_deref().unwrap_or_default() {
            assert!(
                row.endpoint.is_some(),
                "selected endpoint dimension omitted"
            );
        }
        if let Some(cursor) = response.data.next_cursor {
            let next = qn
                .admin
                .get_usage_rpc_summary(&GetUsageRpcSummaryRequest {
                    cursor: Some(cursor),
                    ..Default::default()
                })
                .await?;
            assert_eq!(next.data.from_time, response.data.from_time);
            assert_eq!(next.data.to_time, response.data.to_time);
        }
    }
    if let Some(response) = feature_access(
        "Streams summary",
        qn.admin
            .get_usage_streams_summary(&GetUsageStreamsSummaryRequest {
                last: Some("1h".into()),
                group_by: Some(vec!["stream".into()]),
                ..Default::default()
            })
            .await,
    )? {
        assert!(response.data.totals.block_credits.is_finite());
    }
    if let Some(response) = feature_access(
        "Webhooks summary",
        qn.admin
            .get_usage_webhooks_summary(&GetUsageWebhooksSummaryRequest {
                last: Some("1h".into()),
                group_by: Some(vec!["webhook".into()]),
                ..Default::default()
            })
            .await,
    )? {
        assert!(response.data.totals.deliveries.is_finite());
    }
    if let Some(response) = feature_access(
        "traffic summary",
        qn.admin
            .get_traffic_summary(&GetTrafficSummaryRequest {
                last: Some("1h".into()),
                group_by: Some(vec!["method".into(), "endpoint".into()]),
                ..Default::default()
            })
            .await,
    )? {
        assert!(response.data.totals.bytes.is_finite());
        for row in response.data.breakdown.as_deref().unwrap_or_default() {
            assert!(
                row.endpoint.is_some(),
                "selected endpoint dimension omitted"
            );
        }
    }
    for kind in ["rpc", "http"] {
        if let Some(response) = feature_access(
            "account errors summary",
            qn.admin
                .get_errors_summary(&GetErrorsSummaryRequest {
                    last: Some("1h".into()),
                    kind: Some(kind.into()),
                    group_by: Some(vec!["code".into(), "endpoint".into()]),
                    ..Default::default()
                })
                .await,
        )? {
            assert!(response.data.totals.rpc_errors.is_finite());
            for row in response.data.breakdown.unwrap_or_default() {
                assert_eq!(row.kind, kind);
                assert!(row.count.is_finite());
                assert!(
                    row.endpoint.is_some(),
                    "selected endpoint dimension omitted"
                );
            }
        }
    }
    if let Some(response) = feature_access(
        "endpoint errors summary",
        qn.admin
            .get_endpoint_errors_summary(
                endpoint_id,
                &GetEndpointErrorsSummaryRequest {
                    last: Some("1h".into()),
                    ..Default::default()
                },
            )
            .await,
    )? {
        assert!(response.data.totals.http_errors.is_finite());
    }

    if let Some(filters) = feature_access(
        "activity filters",
        qn.admin.list_activity_log_filters().await,
    )? {
        println!(
            "list_activity_log_filters: {} target types",
            filters.data.types.len()
        );
    }
    if let Some(page) = feature_access(
        "activity list",
        qn.admin
            .list_activity_logs(&ListActivityLogsRequest {
                endpoint_id: Some(endpoint_id.into()),
                limit: Some(1),
                offset: Some(0),
                ..Default::default()
            })
            .await,
    )? {
        assert_eq!(page.pagination.limit, 1);
        assert_eq!(page.pagination.offset, 0);
        if let Some(entry) = page.data.first() {
            let detail = qn
                .admin
                .get_activity_log(&entry.id)
                .await?
                .data
                .expect("activity detail");
            assert_eq!(detail.id, entry.id);
            assert_eq!(detail.actor.actor_type, entry.actor.actor_type);
        } else {
            println!("activity detail skipped: no entries for the new endpoint");
        }
        expect_api_error(
            qn.admin
                .get_activity_log("00000000-0000-0000-0000-000000000000")
                .await,
            404,
        );
    }

    if let Some(page) = feature_access(
        "transaction list",
        qn.admin
            .list_transactions(&ListTransactionsRequest {
                last: Some("1h".into()),
                limit: Some(1),
                ..Default::default()
            })
            .await,
    )? {
        if let Some(cursor) = page.data.next_cursor {
            let next = qn
                .admin
                .list_transactions(&ListTransactionsRequest {
                    cursor: Some(cursor),
                    ..Default::default()
                })
                .await?;
            assert_eq!(next.data.from_time, page.data.from_time);
            assert_eq!(next.data.to_time, page.data.to_time);
        }
        if let Some(transaction) = page.data.transactions.first() {
            if let Some(height) = &transaction.height {
                assert!(!height.is_empty() && height.bytes().all(|byte| byte.is_ascii_digit()));
                assert_eq!(json!(transaction)["height"].as_str(), Some(height.as_str()));
            }
            for include_payloads in [false, true] {
                let lookup = qn
                    .admin
                    .get_transaction(
                        &transaction.network,
                        &transaction.transaction_hash,
                        &GetTransactionRequest {
                            include_payloads: Some(include_payloads),
                        },
                    )
                    .await?
                    .data;
                if lookup.exists {
                    let detail = lookup.data.expect("retained transaction details");
                    assert_eq!(detail.transaction_hash, transaction.transaction_hash);
                    if let Some(height) = &detail.height {
                        assert_eq!(json!(detail)["height"].as_str(), Some(height.as_str()));
                    }
                    for attempt in &detail.attempts {
                        if !include_payloads {
                            assert!(attempt.payloads.is_none());
                        }
                        if let Some(payloads) = &attempt.payloads {
                            let serialized = json!(payloads);
                            assert_eq!(serialized["request"].as_str(), payloads.request.as_deref());
                            assert_eq!(
                                serialized["response"].as_str(),
                                payloads.response.as_deref()
                            );
                        }
                    }
                } else {
                    assert!(lookup.data.is_none());
                }
            }
        } else {
            println!("populated transaction checks skipped: no retained transactions");
        }
        let absent = qn
            .admin
            .get_transaction(
                "ethereum-mainnet",
                &format!("0x{}", "0".repeat(64)),
                &GetTransactionRequest {
                    include_payloads: Some(false),
                },
            )
            .await?
            .data;
        assert!(!absent.exists);
        assert!(absent.data.is_none());
    }
    Ok(())
}
