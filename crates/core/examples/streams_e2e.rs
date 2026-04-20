use std::{thread, time::Duration};

use sdk_core::{
    streams::{
        CreateStreamParams, DestinationAttributes, ListStreamsParams, StreamDataset,
        StreamMetadataLocation, StreamRegion, StreamStatus, TestFilterParams, UpdateStreamParams,
        WebhookAttributes,
    },
    QuickNodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(&config).expect("sdk failed to initialize");

    let before = qn
        .streams
        .list_streams(&ListStreamsParams::default())
        .await
        .expect("list_streams failed");
    println!("streams before: {}", before.page_info.total);

    let count = qn
        .streams
        .get_enabled_count(None)
        .await
        .expect("get_enabled_count failed");
    println!("enabled count: {}", count.total);

    let filter_params = TestFilterParams {
        network: "ethereum-mainnet".to_string(),
        dataset: StreamDataset::Block,
        block: "17811625".to_string(),
        filter_function: Some("ZnVuY3Rpb24gbWFpbihkYXRhKSB7IHJldHVybiBkYXRhOyB9".to_string()),
        filter_language: None,
        address_book_config: None,
    };
    let filter_result = qn
        .streams
        .test_filter(&filter_params)
        .await
        .expect("test_filter failed");
    println!("filter logs: {:?}", filter_result.logs);
    thread::sleep(Duration::from_secs(1));

    let create_params = CreateStreamParams::builder()
        .name("E2E Test Stream".to_string())
        .region(StreamRegion::UsaEast)
        .network("ethereum-mainnet".to_string())
        .dataset(StreamDataset::Block)
        .start_range(24691804)
        .end_range(24691904)
        .dataset_batch_size(1)
        .include_stream_metadata(StreamMetadataLocation::Body)
        .destination_attributes(DestinationAttributes::Webhook(WebhookAttributes {
            url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef".to_string(),
            compression: "none".to_string(),
            max_retry: 3,
            retry_interval_sec: 1,
            post_timeout_sec: 10,
            security_token: None,
        }))
        .extra_destinations(vec![DestinationAttributes::Webhook(WebhookAttributes {
            url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef".to_string(),
            compression: "none".to_string(),
            max_retry: 3,
            retry_interval_sec: 1,
            post_timeout_sec: 10,
            security_token: None,
        })])
        .fix_block_reorgs(0)
        .keep_distance_from_tip(0)
        .elastic_batch_enabled(true)
        .status(StreamStatus::Active)
        .plan("growth_plan".to_string())
        .threshold_fetch_buffer(1000)
        .build();

    let stream = qn
        .streams
        .create_stream(&create_params)
        .await
        .expect("create_stream failed");
    let id = stream.id.clone();
    println!("created: {} | {}", id, stream.status);

    let fetched = qn.streams.get_stream(&id).await.expect("get_stream failed");
    println!("fetched: {} | {}", fetched.id, fetched.name);

    let update_params = UpdateStreamParams {
        name: Some("E2E Test Stream Updated".to_string()),
        ..Default::default()
    };
    let updated = qn
        .streams
        .update_stream(&id, &update_params)
        .await
        .expect("update_stream failed");
    println!("updated name: {}", updated.name);
    thread::sleep(Duration::from_secs(1));

    qn.streams
        .pause_stream(&id)
        .await
        .expect("pause_stream failed");
    println!("paused");

    qn.streams
        .activate_stream(&id)
        .await
        .expect("activate_stream failed");
    println!("activated");

    qn.streams
        .delete_stream(&id)
        .await
        .expect("delete_stream failed");
    println!("deleted: {id}");
    thread::sleep(Duration::from_secs(1));

    let after = qn
        .streams
        .list_streams(&ListStreamsParams::default())
        .await
        .expect("list_streams failed");
    println!("streams after: {}", after.page_info.total);
}
