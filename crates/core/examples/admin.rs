use quicknode_sdk::{admin::GetEndpointsRequest, QuicknodeSdk, SdkFullConfig};

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
}
