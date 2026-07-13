"""Crypto-micropayment lane for rpc.call: pay per RPC request with a stablecoin
instead of an account API key, against Quicknode's x402/MPP gateways.

⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded wallet.
Reads the key from QN_PAYMENT_KEY — never hard-code it.

Run (x402/EVM on Base Sepolia testnet):
    QN_PAYMENT_KEY=0x<throwaway-key> python examples/rpc_payment.py
"""

import asyncio
import os

from quicknode_sdk import (
    QuicknodeSdk,
    SdkFullConfig,
    RpcConfig,
    PaymentConfig,
    ConfigError,
    PaymentError,
    PaymentIndeterminateError,
    PaymentRejectedError,
)


async def selfcheck() -> None:
    """No-funds checks that always run: error hierarchy + the network-required
    ConfigError. Asserts the payment surface is wired without moving money."""
    assert issubclass(PaymentIndeterminateError, PaymentError)
    assert issubclass(PaymentRejectedError, PaymentError)
    qn = QuicknodeSdk(
        SdkFullConfig(
            api_key=None,
            rpc=RpcConfig(
                payment=PaymentConfig(
                    scheme="x402",
                    key="0xabc",
                    pay_network="eip155:84532",
                    asset="0xUSDC",
                    max_amount="10000",
                )
            ),
        )
    )
    try:
        await qn.rpc.call("eth_blockNumber")
        raise SystemExit("expected a ConfigError (payment lane requires network)")
    except ConfigError as e:
        assert "requires" in str(e), str(e)
    print("selfcheck OK: payment error classes + network-required ConfigError")


async def main() -> None:
    await selfcheck()

    key = os.environ.get("QN_PAYMENT_KEY")
    if not key:
        print("set QN_PAYMENT_KEY to a throwaway key to run the live payment call")
        return

    # A keyless SDK: the payment lane needs no account API key. Do NOT log the
    # config object — the `key` field is readable (like ethers' .privateKey).
    config = SdkFullConfig(
        api_key=None,
        rpc=RpcConfig(
            payment=PaymentConfig(
                scheme="x402",
                key=key,
                # Base Sepolia testnet USDC (x402/EVM).
                pay_network="eip155:84532",
                asset="0x036CbD53842c5426634e7929541eC2318f3dCF7e",
                # Spend ceiling in base units of the asset (required).
                max_amount="10000",
                # For x402/Solana at any volume, set svm_rpc_url to your own
                # Solana RPC — the public default rate-limits aggressively.
            )
        ),
    )
    qn = QuicknodeSdk(config)

    try:
        # `network` is the QUERY chain (gateway path slug), independent of the
        # pay network. The SDK runs the 402 -> sign -> resend handshake.
        resp = await qn.rpc.call_with_receipt(
            "eth_blockNumber", [], "base-sepolia"
        )
        print("paid eth_blockNumber =>", resp["result"])
        # payment_receipt is set on the MPP lane (reference = settlement tx
        # hash), None for x402.
        if resp["payment_receipt"]:
            print("settlement reference:", resp["payment_receipt"]["reference"])
    except PaymentIndeterminateError as e:
        # The paid request was sent but the response was lost — you may already
        # have been charged. Do NOT blindly retry.
        print("payment indeterminate — do not retry:", e)
    except PaymentRejectedError as e:
        print(f"payment rejected ({e.status}):", e.body)


if __name__ == "__main__":
    asyncio.run(main())
