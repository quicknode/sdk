"""Crypto-micropayment lane for rpc.call: pay per RPC request with a stablecoin
instead of an account API key, against Quicknode's x402/MPP gateways.

⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded wallet.
Reads the key from QN_PAYMENT_KEY — never hard-code it.

Run (x402/EVM on Base Sepolia testnet):
    QN_PAYMENT_KEY=0x<throwaway-key> python examples/rpc_payment.py

Run the x402 drawdown lane (authenticate once, then 1 credit per call):
    QN_PAYMENT_KEY=0x<key> QN_PAYMENT_LANE=drawdown python examples/rpc_payment.py

With no QN_PAYMENT_KEY set, only the no-funds selfcheck runs.
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
    PaymentUnsupportedError,
    generate_payment_wallet,
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

    # Wallet generation is offline; persist the returned key.
    wallet = generate_payment_wallet("evm")
    assert wallet["address"].startswith("0x") and len(wallet["address"]) == 42
    assert wallet["chain"] == "evm"
    assert isinstance(wallet["key"], str)
    try:
        generate_payment_wallet("dogecoin")
        raise SystemExit("expected a ConfigError for an unknown chain")
    except ConfigError:
        pass

    # Amounts are decimal strings; reject non-integers.
    try:
        await qn.rpc.mpp_open("12.5")
        raise SystemExit("expected a ConfigError for a non-integer deposit")
    except ConfigError as e:
        assert "decimal base-unit" in str(e), str(e)

    # A full channel rejects the status probe before network I/O.
    full_channel = {
        "channel_id": "0x" + "11" * 32,
        "token": "0x20c0000000000000000000000000000000000000",
        "payee": "0xfd24114c3981aba78ae2441991b1bdb89329c556",
        "salt": "0x" + "22" * 32,
        "authorized_signer": wallet["address"],
        "escrow_contract": "0x33b901018174DDabE4841042ab76ba85D4e24f25",
        "deposit": "1000",
        "cumulative_spent": "1000",
        "per_call": "500",
        "chain_id": 42431,
    }
    try:
        await qn.rpc.mpp_status(full_channel)
        raise SystemExit("expected a PaymentUnsupportedError (no room to probe)")
    except PaymentUnsupportedError as e:
        assert "no room" in str(e), str(e)

    print("selfcheck OK: error classes, wallet generation, u128 string amounts")


async def drawdown_demo(key: str) -> None:
    """The x402 drawdown lane: authenticate once, then draw 1 credit per call.

    Cheaper per call than the per-request lane (one signature buys a block of
    credits), and the session JWT is free to mint — so a host can re-auth
    transparently. Persist the session dict between runs.
    """
    qn = QuicknodeSdk(
        SdkFullConfig(
            api_key=None,
            rpc=RpcConfig(
                payment=PaymentConfig(
                    scheme="x402",
                    key=key,
                    pay_network="eip155:84532",
                    asset="0x036CbD53842c5426634e7929541eC2318f3dCF7e",
                    max_amount="10000",
                )
            ),
        )
    )

    # Derived locally; use it to key a session cache.
    print("payment wallet:", qn.rpc.payment_address())

    session = await qn.rpc.gateway_authenticate()
    print("session account:", session["account_id"], "expires:", session["exp_unix"])

    balance = await qn.rpc.gateway_credits(session)
    print("credits:", balance["credits"])

    if balance["credits"] == 0:
        # The faucet returns a funding transaction, not a balance.
        try:
            drip = await qn.rpc.gateway_drip(session)
            print("faucet tx:", drip["transaction_hash"])
        except PaymentRejectedError as e:
            print(f"faucet refused ({e.status}):", e.body)

    result = await qn.rpc.gateway_drawdown_call(
        "eth_blockNumber", session, "base-sepolia"
    )
    print("drawdown eth_blockNumber =>", result)


async def main() -> None:
    await selfcheck()

    key = os.environ.get("QN_PAYMENT_KEY")
    if not key:
        print("set QN_PAYMENT_KEY to a throwaway key to run the live payment call")
        return

    # QN_PAYMENT_LANE=drawdown runs the credit lane instead of per-request.
    if os.environ.get("QN_PAYMENT_LANE") == "drawdown":
        await drawdown_demo(key)
        return

    # Keyless SDK. Do not log this config; it contains the private key.
    config = SdkFullConfig(
        api_key=None,
        rpc=RpcConfig(
            payment=PaymentConfig(
                scheme="x402",
                key=key,
                # Base Sepolia testnet USDC (x402/EVM).
                pay_network="eip155:84532",
                asset="0x036CbD53842c5426634e7929541eC2318f3dCF7e",
                # Spend ceiling in asset base units.
                max_amount="10000",
                # Set svm_rpc_url for x402/Solana at volume.
            )
        ),
    )
    qn = QuicknodeSdk(config)

    try:
        # Query network is independent of the payment network.
        resp = await qn.rpc.call_with_receipt(
            "eth_blockNumber", [], "base-sepolia"
        )
        print("paid eth_blockNumber =>", resp["result"])
        # x402 does not return a settlement receipt.
        if resp["payment_receipt"]:
            print("settlement reference:", resp["payment_receipt"]["reference"])
    except PaymentIndeterminateError as e:
        # The request may have settled. Do not retry blindly.
        print("payment indeterminate — do not retry:", e)
    except PaymentRejectedError as e:
        print(f"payment rejected ({e.status}):", e.body)


if __name__ == "__main__":
    asyncio.run(main())
