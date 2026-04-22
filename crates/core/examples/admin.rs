use quicknode_sdk::{admin::GetEndpointsRequest, QuicknodeSdk, SdkFullConfig};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    let params = GetEndpointsRequest::builder()
        .limit(20)
        .sort_by("created_at".to_string())
        .sort_direction("desc".to_string())
        .build();

    match qn.admin.get_endpoints(&params).await {
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
                    "{} | {} | {} | {} | dedicated={} flat={}",
                    ep.id, ep.name, ep.status, ep.chain, ep.is_dedicated, ep.is_flat_rate
                );
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
