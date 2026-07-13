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

    // A keyless SDK: no account API key is needed for the payment lane. Every
    // other surface (admin/streams/…) would error without a key — that's fine,
    // this SDK only pays per request.
    let mut config = SdkFullConfig::keyless();
    config.rpc = Some(RpcConfig {
        // The payment config is plain data; the private key stays in `key`.
        // WARNING: do not log this object — the `key` field is readable. The
        // SDK never prints it in its own errors/Debug.
        payment: Some(PaymentConfig {
            scheme: "x402".into(),
            key,
            // Base Sepolia testnet USDC (x402/EVM).
            pay_network: "eip155:84532".into(),
            asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".into(),
            // Spend ceiling in base units of the asset (required). The SDK
            // refuses to sign any offered amount above this.
            max_amount: "10000".into(),
            svm_rpc_url: None,
            base_url_override: None,
        }),
        ..Default::default()
    });

    let qn = QuicknodeSdk::new(&config).expect("sdk failed to initialize");

    // `network` is the QUERY chain (path slug on the gateway), independent of
    // the pay network. The SDK does the 402 → sign → resend handshake.
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

    // `call_with_receipt` also returns the settlement receipt. It is `Some` on
    // the MPP lane (the reference is the settlement tx hash) and `None` for
    // x402. On a lost response after paying, the error is `PaymentIndeterminate`
    // — do NOT blindly retry (you may have already been charged).
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
