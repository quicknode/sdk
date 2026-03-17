use sdk_core::{
    admin::{CreateEndpointRequest, GetEndpointsRequest},
    QuickNodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(config).expect("sdk failed to initialize");

    let params = GetEndpointsRequest::builder().limit(20).build();

    match qn.admin.get_endpoints(&params).await {
        Ok(resp) => {
            for ep in &resp.data {
                println!("{} | {:?} | {}", ep.id, ep.label, ep.chain);
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    let create_params = CreateEndpointRequest::builder()
        .chain("ethereum".to_string())
        .network("mainnet".to_string())
        .build();
    match qn.admin.create_endpoint(&create_params).await {
        Ok(resp) => {
            println!("{} created", resp.data.http_url);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
