use sdk_core::{
    admin::{
        BulkAddTagRequest, BulkRemoveTagRequest, BulkUpdateEndpointStatusRequest,
        CreateDomainMaskRequest, CreateEndpointRequest, CreateIpRequest, CreateJwtRequest,
        CreateMethodRateLimitRequest, CreateOrUpdateIpCustomHeaderRequest, CreateReferrerRequest,
        CreateRequestFilterRequest, CreateTagRequest, CreateTeamRequest, GetAccountMetricsRequest,
        GetEndpointLogsRequest, GetEndpointMetricsRequest, GetEndpointsRequest, GetUsageRequest,
        InviteTeamMemberRequest, RateLimitSettings, RenameTagRequest, SecurityOptionsUpdate,
        UpdateEndpointRequest, UpdateEndpointStatusRequest, UpdateMethodRateLimitRequest,
        UpdateRateLimitsRequest, UpdateRequestFilterRequest, UpdateSecurityOptionsRequest,
        UpdateTeamEndpointsRequest,
    },
    errors::SdkError,
    AdminConfig, HttpConfig, QuickNodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(&config).expect("sdk failed to initialize");

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
        Ok(resp) => println!("get_usage_by_endpoint: {:?}", resp.data),
        Err(e) => eprintln!("get_usage_by_endpoint error: {e}"),
    }

    match qn
        .admin
        .get_usage_by_method(&GetUsageRequest::default())
        .await
    {
        Ok(resp) => println!("get_usage_by_method: {:?}", resp.data),
        Err(e) => eprintln!("get_usage_by_method error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    match qn
        .admin
        .get_usage_by_chain(&GetUsageRequest::default())
        .await
    {
        Ok(resp) => println!("get_usage_by_chain: {:?}", resp.data),
        Err(e) => eprintln!("get_usage_by_chain error: {e}"),
    }

    match qn.admin.get_usage_by_tag(&GetUsageRequest::default()).await {
        Ok(resp) => println!("get_usage_by_tag: {:?}", resp.data),
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
        Ok(resp) => println!("get_account_metrics: {} series", resp.data.len()),
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
            eprintln!("create_endpoint error: {e}");
            return;
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

    match qn
        .admin
        .create_tag(
            &endpoint_id,
            &CreateTagRequest {
                label: Some("example-tag".to_string()),
            },
        )
        .await
    {
        Ok(()) => println!("create_tag: ok"),
        Err(e) => eprintln!("create_tag error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let tag_id = match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => resp
            .data
            .and_then(|ep| ep.tags.into_iter().next())
            .map(|t| t.tag_id.to_string()),
        Err(e) => {
            eprintln!("show_endpoint (for tag) error: {e}");
            None
        }
    };

    if let Some(tag_id) = tag_id {
        match qn.admin.delete_tag(&endpoint_id, &tag_id).await {
            Ok(()) => println!("delete_tag: ok"),
            Err(e) => eprintln!("delete_tag error: {e}"),
        }
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
                metric: "credits_over_time".to_string(),
            },
        )
        .await
    {
        Ok(resp) => println!("get_endpoint_metrics: {} series", resp.data.len()),
        Err(e) => eprintln!("get_endpoint_metrics error: {e}"),
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

    match qn.admin.create_token(&endpoint_id).await {
        Ok(()) => println!("create_token: ok"),
        Err(e) => eprintln!("create_token error: {e}"),
    }

    let token_id = match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => resp
            .data
            .and_then(|ep| ep.security)
            .and_then(|s| s.tokens)
            .and_then(|tokens| tokens.into_iter().next())
            .map(|t| t.id),
        Err(e) => {
            eprintln!("show_endpoint (for token) error: {e}");
            None
        }
    };

    if let Some(token_id) = token_id {
        match qn.admin.delete_token(&endpoint_id, &token_id).await {
            Ok(resp) => println!("delete_token: {:?}", resp.data),
            Err(e) => eprintln!("delete_token error: {e}"),
        }
    }

    // --- Referrer ---

    match qn
        .admin
        .create_referrer(
            &endpoint_id,
            &CreateReferrerRequest {
                referrer: Some("https://example.com".to_string()),
            },
        )
        .await
    {
        Ok(()) => println!("create_referrer: ok"),
        Err(e) => eprintln!("create_referrer error: {e}"),
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let referrer_id = match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => resp
            .data
            .and_then(|ep| ep.security)
            .and_then(|s| s.referrers)
            .and_then(|rs| rs.into_iter().next())
            .map(|r| r.id),
        Err(e) => {
            eprintln!("show_endpoint (for referrer) error: {e}");
            None
        }
    };

    if let Some(referrer_id) = referrer_id {
        match qn.admin.delete_referrer(&endpoint_id, &referrer_id).await {
            Ok(resp) => println!("delete_referrer: {:?}", resp.data),
            Err(e) => eprintln!("delete_referrer error: {e}"),
        }
    }

    // --- IP allowlist ---

    match qn
        .admin
        .create_ip(
            &endpoint_id,
            &CreateIpRequest {
                ip: Some("192.0.2.1".to_string()),
            },
        )
        .await
    {
        Ok(()) => println!("create_ip: ok"),
        Err(e) => eprintln!("create_ip error: {e}"),
    }

    let ip_id = match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => resp
            .data
            .and_then(|ep| ep.security)
            .and_then(|s| s.ips)
            .and_then(|ips| ips.into_iter().next())
            .map(|i| i.id),
        Err(e) => {
            eprintln!("show_endpoint (for ip) error: {e}");
            None
        }
    };

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    if let Some(ip_id) = ip_id {
        match qn.admin.delete_ip(&endpoint_id, &ip_id).await {
            Ok(resp) => println!("delete_ip: {:?}", resp.data),
            Err(e) => eprintln!("delete_ip error: {e}"),
        }
    }

    // --- Domain mask ---

    match qn
        .admin
        .create_domain_mask(
            &endpoint_id,
            &CreateDomainMaskRequest {
                domain_mask: Some("example.com".to_string()),
            },
        )
        .await
    {
        Ok(()) => println!("create_domain_mask: ok"),
        Err(e) => eprintln!("create_domain_mask error: {e}"),
    }

    let mask_id = match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => resp
            .data
            .and_then(|ep| ep.security)
            .and_then(|s| s.domain_masks)
            .and_then(|masks| masks.into_iter().next())
            .map(|m| m.id),
        Err(e) => {
            eprintln!("show_endpoint (for domain_mask) error: {e}");
            None
        }
    };

    if let Some(mask_id) = mask_id {
        match qn.admin.delete_domain_mask(&endpoint_id, &mask_id).await {
            Ok(resp) => println!("delete_domain_mask: {:?}", resp.data),
            Err(e) => eprintln!("delete_domain_mask error: {e}"),
        }
    }

    // --- JWT (placeholder public key will fail at runtime) ---

    match qn
        .admin
        .create_jwt(
            &endpoint_id,
            &CreateJwtRequest {
                public_key: Some(
                    "-----BEGIN PUBLIC KEY-----\nPLACEHOLDER\n-----END PUBLIC KEY-----".to_string(),
                ),
                kid: Some("kid1".to_string()),
                name: Some("example-jwt".to_string()),
            },
        )
        .await
    {
        Ok(()) => println!("create_jwt: ok"),
        Err(e) => eprintln!("create_jwt error (expected with placeholder key): {e}"),
    }

    let jwt_id = match qn.admin.show_endpoint(&endpoint_id).await {
        Ok(resp) => resp
            .data
            .and_then(|ep| ep.security)
            .and_then(|s| s.jwts)
            .and_then(|jwts| jwts.into_iter().next())
            .map(|j| j.id),
        Err(e) => {
            eprintln!("show_endpoint (for jwt) error: {e}");
            None
        }
    };

    if let Some(jwt_id) = jwt_id {
        match qn.admin.delete_jwt(&endpoint_id, &jwt_id).await {
            Ok(()) => println!("delete_jwt: ok"),
            Err(e) => eprintln!("delete_jwt error: {e}"),
        }
    }

    // --- Request filter ---

    let rf_id = match qn
        .admin
        .create_request_filter(
            &endpoint_id,
            &CreateRequestFilterRequest {
                method: Some(vec!["eth_getBalance".to_string()]),
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

    match qn
        .admin
        .update_rate_limits(
            &endpoint_id,
            &UpdateRateLimitsRequest {
                rate_limits: RateLimitSettings {
                    rps: Some(3),
                    ..Default::default()
                },
            },
        )
        .await
    {
        Ok(()) => println!("update_rate_limits: ok"),
        Err(e) => eprintln!("update_rate_limits error: {e}"),
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
            },
        )
        .await
    {
        Ok(resp) => {
            println!("create_method_rate_limit: {:?}", resp.data);
            resp.data.map(|d| d.id)
        }
        Err(e) => {
            eprintln!("create_method_rate_limit error: {e}");
            None
        }
    };

    if let Some(mrl_id) = mrl_id {
        match qn
            .admin
            .update_method_rate_limit(
                &endpoint_id,
                &mrl_id,
                &UpdateMethodRateLimitRequest {
                    rate: Some(10),
                    ..Default::default()
                },
            )
            .await
        {
            Ok(resp) => println!("update_method_rate_limit: {:?}", resp.data),
            Err(e) => eprintln!("update_method_rate_limit error: {e}"),
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

    // --- Bulk endpoint ops (single-endpoint batch) ---

    match qn
        .admin
        .bulk_update_endpoint_status(&BulkUpdateEndpointStatusRequest {
            ids: vec![endpoint_id.clone()],
            status: "paused".to_string(),
        })
        .await
    {
        Ok(resp) => println!("bulk_update_endpoint_status paused: {:?}", resp.data),
        Err(e) => eprintln!("bulk_update_endpoint_status paused error: {e}"),
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

    // --- Cleanup endpoint ---

    match qn.admin.archive_endpoint(&endpoint_id).await {
        Ok(()) => println!("archive_endpoint: ok"),
        Err(e) => eprintln!("archive_endpoint error: {e}"),
    }

    // --- Error handling ---

    // 1) API error path — 404 on a bogus endpoint id.
    match qn.admin.show_endpoint("does-not-exist").await {
        Err(SdkError::Api { status, body }) => {
            println!("api error {status}: {}", &body[..body.len().min(80)]);
            assert_eq!(status.as_u16(), 404);
        }
        other => eprintln!("expected Api 404, got {other:?}"),
    }

    // 2) Timeout path — unreachable base URL + 1s timeout forces a timeout
    // from reqwest, which maps to SdkError::Http with http_kind() == Timeout.
    let blackhole = SdkFullConfig {
        api_key: config.api_key.clone(),
        http: Some(HttpConfig {
            timeout_secs: Some(1),
            pool_max_idle_per_host: None,
        }),
        admin: Some(AdminConfig {
            base_url: Some("http://10.255.255.1/".to_string()),
        }),
        streams: None,
        webhooks: None,
        kvstore: None,
    };
    let tiny = QuickNodeSdk::new(&blackhole).expect("build tiny sdk");
    match tiny
        .admin
        .get_endpoints(&GetEndpointsRequest::default())
        .await
    {
        Err(e) if matches!(e.http_kind(), Some(sdk_core::errors::HttpKind::Timeout)) => {
            println!("timed out as expected");
        }
        other => eprintln!("expected timeout, got {other:?}"),
    }
}
