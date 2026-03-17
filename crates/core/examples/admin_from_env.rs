use sdk_core::{admin::GetEndpointsRequest, QuickNodeSdk, SdkFullConfig};

#[tokio::main]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(config);

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
