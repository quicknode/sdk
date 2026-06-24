use quicknode_sdk::{errors::SdkError, QuicknodeSdk, SdkFullConfig};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    // Ensure Tooling Access is provisioned (idempotent; requires admin role).
    let status = qn
        .admin
        .tooling_access_status()
        .await
        .expect("tooling_access_status failed");
    println!("tooling access enabled: {}", status.enabled);
    if !status.enabled {
        match qn.admin.enable_tooling_access().await {
            Ok(s) => println!("enabled tooling access: {}", s.enabled),
            Err(e) => {
                eprintln!("could not enable tooling access: {e}");
                return;
            }
        }
    }

    // Make a JSON-RPC call. The SDK mints and refreshes the session JWT.
    match qn.rpc.call("eth_blockNumber", None, None).await {
        Ok(result) => println!("eth_blockNumber => {result}"),
        Err(e) => eprintln!("rpc call error: {e}"),
    }

    // Multichain: seed the per-network URL map from the endpoint id (returned by
    // status), then route a call to a specific network by its key.
    if let Some(id) = &status.endpoint_id {
        if let Ok(urls) = qn.admin.get_endpoint_urls(id).await {
            if let Some(mc) = urls.data.and_then(|d| d.multichain_urls) {
                qn.rpc
                    .set_networks(mc.into_iter().map(|(k, v)| (k, v.http_url)).collect());
                match qn
                    .rpc
                    .call("getSlot", None, Some("solana-mainnet".to_string()))
                    .await
                {
                    Ok(result) => println!("solana getSlot => {result}"),
                    Err(e) => eprintln!("solana rpc error: {e}"),
                }
            }
        }
    }

    // Demonstrate the typed JSON-RPC error path.
    match qn
        .rpc
        .call(
            "eth_getBalance",
            Some(serde_json::json!(["not-an-address"])),
            None,
        )
        .await
    {
        Ok(result) => println!("unexpected ok: {result}"),
        Err(SdkError::Rpc { code, message }) => {
            println!("got expected RpcError: code={code} message={message}");
        }
        Err(e) => eprintln!("other error: {e}"),
    }
}
