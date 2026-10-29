use quicknode_sdk::{
    admin::{
        GetEndpointMetricsRequest, GetEndpointsRequest, GetTransactionRequest,
        GetUsageRpcSummaryRequest, ListActivityLogsRequest, ListTransactionsRequest,
    },
    errors::SdkError,
    QuicknodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    match qn.admin.account_info().await {
        Ok(resp) => {
            if let Some(a) = &resp.data {
                let plan = a
                    .subscription
                    .as_ref()
                    .and_then(|s| s.plan_name.clone())
                    .unwrap_or_else(|| "<none>".to_string());
                println!(
                    "account {} | {} | billing={:?} | plan={}",
                    a.id, a.name, a.billing_version, plan
                );
            }
        }
        Err(e) => eprintln!("account_info error: {e}"),
    }

    match qn.admin.get_api_credits("ethereum").await {
        Ok(resp) => {
            if let Some(credits) = &resp.data {
                println!("ethereum api credits: {} methods", credits.len());
                for c in credits.iter().take(3) {
                    println!("  {} = {}", c.method, c.credits);
                }
            }
        }
        Err(e) => eprintln!("get_api_credits error: {e}"),
    }

    let params = GetEndpointsRequest::builder()
        .limit(20)
        .sort_by("created_at".to_string())
        .sort_direction("desc".to_string())
        .build();

    let first_endpoint_id = match qn.admin.get_endpoints(&params).await {
        Ok(resp) => {
            if let Some(p) = &resp.pagination {
                println!(
                    "{} of {} (offset {}, limit {})",
                    resp.data.len(),
                    p.total,
                    p.offset,
                    p.limit
                );
            }
            for ep in &resp.data {
                println!(
                    "{} | {} | {} | {} | dedicated={} flat={} multichain={}",
                    ep.id,
                    ep.name,
                    ep.status,
                    ep.chain,
                    ep.is_dedicated,
                    ep.is_flat_rate,
                    ep.is_multichain
                );
            }
            resp.data.first().map(|ep| ep.id.clone())
        }
        Err(e) => {
            eprintln!("get_endpoints error: {e}");
            None
        }
    };

    match show_observability(&qn).await {
        Ok(()) => {}
        Err(SdkError::Api { status, .. }) if status.as_u16() == 403 => {
            eprintln!("Observability access is unavailable for this account.");
        }
        Err(error) => eprintln!("Observability error: {error}"),
    }

    let Some(endpoint_id) = first_endpoint_id else {
        return;
    };

    match qn.admin.get_rate_limits(&endpoint_id).await {
        Ok(resp) => println!("get_rate_limits: {:?}", resp.data),
        Err(e) => eprintln!("get_rate_limits error: {e}"),
    }

    match qn.admin.get_endpoint_urls(&endpoint_id).await {
        Ok(resp) => println!("get_endpoint_urls: {:?}", resp.data),
        Err(e) => eprintln!("get_endpoint_urls error: {e}"),
    }

    match qn.admin.get_request_filters(&endpoint_id).await {
        Ok(response) => {
            for filter in response.data {
                println!(
                    "filter {}: whitelist={}, pattern keys={}",
                    filter.id,
                    filter.method_whitelist,
                    filter.pattern.len()
                );
            }
        }
        Err(error) => eprintln!("get_request_filters error: {error}"),
    }
    match qn.admin.get_client_certs(&endpoint_id).await {
        Ok(response) => println!(
            "client certificates: {}, enabled={}",
            response.data.certs.len(),
            response.data.enabled
        ),
        Err(error) => eprintln!("get_client_certs error: {error}"),
    }
    match qn.admin.get_method_blacklist(&endpoint_id).await {
        Ok(response) => println!("blocked methods: {}", response.data.methods.len()),
        Err(error) => eprintln!("get_method_blacklist error: {error}"),
    }
    match qn.admin.get_range_limits(&endpoint_id).await {
        Ok(response) => println!(
            "range limits: {}, supported methods={}",
            response.data.limits.len(),
            response.data.supported_methods.len()
        ),
        Err(error) => eprintln!("get_range_limits error: {error}"),
    }
    match qn.admin.get_blocked_ips(&endpoint_id).await {
        Ok(response) => println!("blocked IP records: {}", response.data.len()),
        Err(error) => eprintln!("get_blocked_ips error: {error}"),
    }
    let latency = GetEndpointMetricsRequest {
        period: "day".to_string(),
        metric: "response_time_over_time".to_string(),
        percentile: Some(95),
    };
    match qn.admin.get_endpoint_metrics(&endpoint_id, &latency).await {
        Ok(response) => println!("p95 latency series: {}", response.data.len()),
        Err(error) => eprintln!("get_endpoint_metrics error: {error}"),
    }
}

async fn show_observability(qn: &QuicknodeSdk) -> Result<(), SdkError> {
    let summary = qn
        .admin
        .get_usage_rpc_summary(
            &GetUsageRpcSummaryRequest::builder()
                .last("1h".to_string())
                .group_by(vec!["method".to_string()])
                .limit(20)
                .build(),
        )
        .await?;
    println!("RPC credits in window: {}", summary.data.totals.credits);
    if let Some(cursor) = summary.data.next_cursor {
        let next = qn
            .admin
            .get_usage_rpc_summary(&GetUsageRpcSummaryRequest::builder().cursor(cursor).build())
            .await?;
        println!(
            "next summary page: {} groups",
            next.data.breakdown.map_or(0, |rows| rows.len())
        );
    }
    let filters = qn.admin.list_activity_log_filters().await?;
    println!("activity filter actors: {}", filters.data.actors.len());
    let activity = qn
        .admin
        .list_activity_logs(&ListActivityLogsRequest::builder().limit(20).build())
        .await?;
    println!("activity entries: {}", activity.data.len());
    if let Some(entry) = activity.data.first() {
        let detail = qn.admin.get_activity_log(&entry.id).await?;
        println!("activity detail present: {}", detail.data.is_some());
    }
    let transactions = qn
        .admin
        .list_transactions(
            &ListTransactionsRequest::builder()
                .last("1h".to_string())
                .limit(20)
                .build(),
        )
        .await?;
    for transaction in &transactions.data.transactions {
        println!(
            "transaction state={}, height={:?}",
            transaction.state, transaction.height
        );
    }
    if let Some(transaction) = transactions.data.transactions.first() {
        let detail = qn
            .admin
            .get_transaction(
                &transaction.network,
                &transaction.transaction_hash,
                &GetTransactionRequest::builder()
                    .include_payloads(false)
                    .build(),
            )
            .await?;
        println!("retained transaction exists: {}", detail.data.exists);
    }
    Ok(())
}
