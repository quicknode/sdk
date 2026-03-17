use sdk_core::{admin::GetEndpointsRequest, QuickNodeSdk, SdkFullConfig};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let api_key = std::env::var("QN_API_KEY").expect("set QN_API_KEY env var");
    let config = SdkFullConfig::builder().api_key(api_key).build();
    let qn = QuickNodeSdk::new(&config);

    let params = GetEndpointsRequest::builder().limit(20).build();

    match qn.admin.get_endpoints(&params).await {
        Ok(resp) => {
            for ep in &resp.data {
                println!("{} | {:?} | {}", ep.id, ep.label, ep.chain);
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
