use std::collections::HashMap;

use sdk_core::{
    kvstore::{AddListItemParams, BulkSetsParams, CreateListParams, CreateSetParams, UpdateListParams},
    QuickNodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(&config).expect("sdk failed to initialize");

    // ── Sets ────────────────────────────────────────────────────────────────

    qn.kvstore
        .create_set(&CreateSetParams { key: "e2e-set-key".to_string(), value: "e2e-value".to_string() })
        .await
        .expect("create_set failed");
    println!("created set: e2e-set-key");

    let set = qn.kvstore.get_set("e2e-set-key").await.expect("get_set failed");
    println!("get set: {}", set.value);

    let sets = qn.kvstore.get_sets(&Default::default()).await.expect("get_sets failed");
    println!("all sets: {:?}", sets.data.iter().map(|e| &e.key).collect::<Vec<_>>());

    let mut add_sets = HashMap::new();
    add_sets.insert("e2e-bulk-key-1".to_string(), "bulk-value-1".to_string());
    add_sets.insert("e2e-bulk-key-2".to_string(), "bulk-value-2".to_string());
    qn.kvstore
        .bulk_sets(&BulkSetsParams {
            add_sets: Some(add_sets),
            delete_sets: Some(vec!["e2e-set-key".to_string()]),
        })
        .await
        .expect("bulk_sets failed");
    println!("bulk sets: added 2, deleted e2e-set-key");

    qn.kvstore.delete_set("e2e-bulk-key-1").await.expect("delete_set failed");
    qn.kvstore.delete_set("e2e-bulk-key-2").await.expect("delete_set failed");
    println!("deleted bulk sets");

    // ── Lists ───────────────────────────────────────────────────────────────

    qn.kvstore
        .create_list(&CreateListParams {
            key: "e2e-list-key".to_string(),
            items: vec!["0xabc".to_string(), "0xdef".to_string()],
        })
        .await
        .expect("create_list failed");
    println!("created list: e2e-list-key");

    let list = qn.kvstore.get_list("e2e-list-key", &Default::default()).await.expect("get_list failed");
    println!("get list items: {:?}", list.data.items);

    let lists = qn.kvstore.get_lists(&Default::default()).await.expect("get_lists failed");
    println!("all list keys: {:?}", lists.data.keys);

    qn.kvstore
        .add_list_item("e2e-list-key", &AddListItemParams { item: "0x123".to_string() })
        .await
        .expect("add_list_item failed");
    println!("added list item: 0x123");

    let contains = qn.kvstore.list_contains_item("e2e-list-key", "0x123").await.expect("list_contains_item failed");
    println!("list contains 0x123: {}", contains.exists);

    qn.kvstore
        .update_list(
            "e2e-list-key",
            &UpdateListParams {
                add_items: Some(vec!["0x456".to_string()]),
                remove_items: Some(vec!["0xabc".to_string()]),
            },
        )
        .await
        .expect("update_list failed");
    println!("updated list: added 0x456, removed 0xabc");

    qn.kvstore.delete_list_item("e2e-list-key", "0x123").await.expect("delete_list_item failed");
    println!("deleted list item: 0x123");

    qn.kvstore.delete_list("e2e-list-key").await.expect("delete_list failed");
    println!("deleted list: e2e-list-key");
}
