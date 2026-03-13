use sdk_core::{admin::GetEndpointsRequest, QuickNodeSdk};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("QN_API_KEY").expect("set QN_API_KEY env var");
    let qn = QuickNodeSdk::new(api_key);

    let params = GetEndpointsRequest {
        limit: Some(5),
        ..Default::default()
    };

    match qn.admin.get_endpoints(&params).await {
        Ok(resp) => {
            for ep in &resp.data {
                println!("{} | {:?} | {}", ep.id, ep.label, ep.chain);
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
