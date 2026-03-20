use sdk_core::{
    streams::{
        CreateStreamParams, StreamDataset, StreamDestination, StreamMetadataLocation, StreamRegion,
        StreamStatus, WebhookAttributes,
    },
    QuickNodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(&config).expect("sdk failed to initialize");

    let params = CreateStreamParams::builder()
        .name("My Stream".to_string())
        .region(StreamRegion::UsaEast)
        .network("ethereum-mainnet".to_string())
        .dataset(StreamDataset::Block)
        .start_range(24691804)
        .end_range(24691904)
        .dataset_batch_size(1)
        .include_stream_metadata(StreamMetadataLocation::Body)
        .destination(StreamDestination::Webhook)
        .webhook_attributes(WebhookAttributes {
            url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef".to_string(),
            compression: Some("none".to_string()),
            max_retry: 3,
            retry_interval_sec: 1,
            post_timeout_sec: 10,
            security_token: None,
        })
        .fix_block_reorgs(0)
        .keep_distance_from_tip(0)
        .elastic_batch_enabled(true)
        .status(StreamStatus::Active)
        .plan("growth_plan".to_string())
        .threshold_fetch_buffer(1000)
        .build();

    match qn.streams.create_stream(&params).await {
        Ok(stream) => println!("{} | {} | {}", stream.id, stream.name, stream.status),
        Err(e) => eprintln!("Error: {e}"),
    }
}
