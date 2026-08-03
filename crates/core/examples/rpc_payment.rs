//! Crypto-micropayment lane for `rpc.call`: pay per RPC request with a
//! stablecoin instead of an account API key, against Quicknode's x402/MPP
//! gateways.
//!
//! ⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded
//! wallet. Reads the private key from `QN_PAYMENT_KEY` — never hard-code it.
//!
//! Run (x402/EVM on Base Sepolia testnet):
//!   QN_PAYMENT_KEY=0x<throwaway-key> \
//!     cargo run --example rpc_payment -p quicknode-sdk \
//!     --features rust,payments,payments-svm,payments-tempo

use quicknode_sdk::{PaymentConfig, QuicknodeSdk, RpcConfig, SdkFullConfig};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let key = std::env::var("QN_PAYMENT_KEY").expect("set QN_PAYMENT_KEY to a throwaway key");

    // Keyless SDK: this example only uses the payment lane.
    let mut config = SdkFullConfig::keyless();
    config.rpc = Some(RpcConfig {
        // Do not log this config; it contains the private key.
        payment: Some(PaymentConfig {
            scheme: "x402".into(),
            key,
            // Base Sepolia testnet USDC (x402/EVM).
            pay_network: "eip155:84532".into(),
            asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".into(),
            // Spend ceiling in asset base units.
            max_amount: "10000".into(),
            svm_rpc_url: None,
            base_url_override: None,
        }),
        ..Default::default()
    });

    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    // Query network is independent of the payment network.
    match qn
        .rpc
        .call(
            "eth_blockNumber",
            None,
            Some("base-sepolia".to_string()),
            None,
        )
        .await
    {
        Ok(result) => println!("paid eth_blockNumber => {result}"),
        Err(e) => eprintln!("payment call error: {e}"),
    }

    // This call also returns an MPP settlement receipt. Do not retry an
    // indeterminate payment.
    match qn
        .rpc
        .call_with_receipt(
            "eth_blockNumber",
            None,
            Some("base-sepolia".to_string()),
            None,
        )
        .await
    {
        Ok(resp) => {
            println!("result => {}", resp.result);
            match resp.payment_receipt {
                Some(r) => println!("settlement reference: {}", r.reference),
                None => println!("(no receipt — x402 lane)"),
            }
        }
        Err(e) => eprintln!("payment call error: {e}"),
    }
}
