# x402 / MPP payment lane for `rpc.call`

Add a crypto-micropayment payment lane to `rpc.call` so a caller can pay per RPC request
with a stablecoin instead of a provisioned account + API key, against Quicknode's
`x402.quicknode.com` and `mpp.quicknode.com` gateways.

**Design (Option C):** the crypto lives in `quicknode-sdk` core (polyglot-reusable),
feature-gated. One concrete `pay_and_call` driver runs the shared 402 loop; an inline
`enum PaymentScheme` matches the per-protocol differences (no `PaymentScheme` *trait* until
a third scheme lands). Signing is an **`enum Signer`** (not a trait) — see Decisions.

**Public repo.** Code, comments, commits, PRs are world-readable. Fixtures use fake data
only (`0xabc…`, `ep-1`, `hook.example.com`), regenerated from a throwaway key that never
touched mainnet. Brand is always Quicknode. Describe observable behavior, never internal
triggers.

---

## Decisions locked

- **Scope — protocols:** x402 pay-per-request + MPP charge. Deferred: x402 credit-drawdown,
  x402 nanopayment (Circle Gateway), MPP session/voucher channels.
- **Scope — pay-chains:** x402/EVM, x402/Solana, MPP/Tempo (the live MPP challenge only ever
  offers Tempo chain IDs). **MPP/Solana out of v1** (no client-side signer). MPP uses the
  **native Tempo tx** construction (0.4 — `@quicknode/mpp` is unusable against the gateway).
- **Reference to mirror, per protocol:**
  - **x402** (EVM + Solana): mirror Quicknode's own `@quicknode/x402` (0.1.3) +
    `@quicknode/x402-solana` (0.2.0). Confirmed working against the gateway.
  - **MPP**: mirror the **wire format** produced by generic `mppx` (native Tempo tx).
    `@quicknode/mpp@0.2.0` is **unusable** against the live gateway (0.4: only registers
    `evm.charge`; gateway emits `tempo.charge`) — do NOT use it as the reference.
- **Signer is an `enum`, not a trait** (resolves the fat-trait + config-surface problem):
  ```rust
  enum Signer { Evm(SecretString), Svm(SecretString), Tempo(SecretString) }  // three (0.4)
  ```
  (`Tempo` restored by 0.4 — MPP is a native Tempo tx, not EIP-712.)
  A trait would force `Box<dyn Signer>` into `RpcConfig`, breaking its derived `Clone` /
  `Serialize` / `Deserialize` / `Default` and its `pyclass(get_all)` / `napi(object)` — and
  `get_all` would expose the key, breaking redaction. The enum holds `SecretString`, has a
  manual `Debug`, `#[serde(skip)]` on the key, and dispatches at runtime. A trait would only
  earn its keep for external KMS/hardware signers, which aren't a goal and don't cross three
  FFI boundaries anyway.
- **THREE signing constructions (0.4 restored the third):** (1) EIP-712
  `TransferWithAuthorization` — x402/EVM only; (2) SPL transfer tx — x402/Solana; (3)
  **native Tempo tx (type `0x76`)** — MPP. MPP is NOT EIP-712 (that was a source read of an
  unusable package). `payments-tempo` feature is required.
- **Pay-chain RPC (revised by the 1a live probes):**
  - x402/EVM: sync, no chain I/O.
  - x402/Solana: async; the signer's payment-build reads (`getAccountInfo` +
    `getLatestBlockhash`) go to a plain Solana RPC, NOT the gateway — the gateway 402s
    keyless sub-reads. Per-call cost = one payment. **RPC source precedence (decided):**
    (1) explicit `PaymentConfig` RPC-URL override if set; (2) the SDK's tooling lane if
    enabled (API key present and the tooling network map resolves the pay-chain's
    Solana network) — reads then go to the caller's own Quicknode endpoint; (3) public
    Solana RPC default matching the pay cluster (`api.mainnet-beta.solana.com` /
    `api.devnet.solana.com`, mirroring the reference client). (Stage-0's "gateway URL"
    claim described the wrapper's query transport, which pays per sub-call — not the
    signer.) **The public default rate-limits aggressively** — fine as a fallback, but
    keyless production users (the payment lane's core audience) land on it by default, so
    the READMEs must push the explicit RPC override hard for x402/Solana at any volume
    (Stage 5).
  - MPP/Tempo: **RESOLVED (1a): sync, ZERO chain reads.** The expiring nonce is derived
    locally and sponsorship drops fee-token commitment; mppx's third-party
    `eth_fillTransaction` call existed only to populate gas + fee caps, which we set
    ourselves (values pinned in the pre-Stage-2 probe). So "pay-chain RPC = gateway URL"
    is x402-scoped, and MPP needs no pay-chain RPC at all.
- **Key intake:** host passes the raw private key (hex/bytes) as a plain field on the
  binding-facing `PaymentConfig` — the ethers.js (`.privateKey` is readable) / web3.py
  convention. GC-residency is *not* a deciding factor (inherent to every managed runtime).
  - **Redaction promise (scoped, decision (b)):** the SDK **never itself prints or logs the
    key** — the internal resolved config holds it in `secrecy::SecretString` with a manual
    `Debug` that prints `[redacted]`, and it never appears in an error. But the SDK does
    **NOT** guarantee the caller's own `PaymentConfig` object is redacted: it's a plain
    `napi(object)`/`pyclass(get_all)`/Ruby-hash, so `console.log(config)` / `repr(config)` /
    `config.inspect` **will** show the raw key, exactly like ethers' readable `privateKey`.
    That is an accepted, documented limitation — chosen over the heavier opaque-handle
    design. Document it in the READMEs so callers don't log their own config.
  - No opaque-handle machinery; no native Signer-constructor requirement in the bindings.
- **Pay-network: explicit selector required.** A single-network 402 returns a *menu* (21
  entries in the live capture), so the caller declares what they fund; derivation can't
  pick. Selector = `{pay_network (CAIP-2), asset}` (scheme is implied by the protocol, so
  it's not a separate field — see the redundancy note below).
- **Spend ceiling is required (v1 core):** `PaymentConfig.max_amount` is **required**. The
  selector skips any `accepts` entry above it, and the driver refuses to sign one. Guards
  against a buggy/hostile gateway (or anything via `base_url_override`) presenting an
  arbitrary amount to a key we custody. The funded-wallet balance is NOT a guard (wallets
  get topped up). **Units: base units of the selector's `asset`** (integer), compared
  against the menu's decimal-string `amount` parsed as an integer — no float math.
- **Double-spend guard, both cases (review #4):** the 402→retry is exactly one resend; a
  second 402 is terminal. AND: if the retry request is sent but the response is lost
  (timeout/reset), that surfaces as a distinct `PaymentIndeterminate` error (not a generic
  `Http`), so a caller cannot blindly retry into a double-charge.
- **Receipt exposure (decided): `call_with_receipt`, `call` unchanged.** The MPP
  `Payment-Receipt` (settlement tx hash = the caller's proof of payment) needs a public
  channel. Add `call_with_receipt` returning `RpcCallResponse { result: Value,
  payment_receipt: Option<PaymentReceipt> }` alongside `call`; `call` keeps returning the
  bare `result` and discards the receipt. No breaking change; receipt is `None` for x402
  and for non-payment lanes. Rejected: changing `call`'s return type (breaks every caller
  in four languages), a `last_payment_receipt()` accessor (shared mutable state, clobbered
  by concurrent calls).

---

## Research findings (Stage 0 — complete; all three v1 paths live-confirmed)

Full detail + probes in `scratch/STAGE0-FINDINGS.md` and `scratch/{x402,mpp,solana}-sign.mjs`.
Every v1 path was reproduced end-to-end against the live gateways with a green 200 and a
real settled payment. Key outcomes the implementation must honor:

### v1 payment matrix

| Protocol / pay-chain | Status | Signing construction |
|---|---|---|
| x402 / EVM    | ✅ confirmed (known-good vector) | EIP-712 `TransferWithAuthorization` |
| x402 / Solana | ✅ confirmed (real mainnet USDC settled) | partial-signed SPL transfer tx |
| MPP / Tempo   | ✅ confirmed (real mainnet PathUSD settled) | **native Tempo tx** (type `0x76`, via `mppx`/`viem/tempo`) |
| MPP / Solana  | ❌ dropped from v1 | no client signer exists |

> **MPP = native Tempo tx (0.4 resolved this).** `@quicknode/mpp@0.2.0` is unusable against
> the live gateway — it only registers `evm.charge` while the gateway emits
> `tempo.charge`/`solana.charge`, so it fails at method routing before any credential is
> built. The ONLY confirmed MPP path is `mppx`'s native Tempo transaction (green 200
> `0x2a0747b`). **Decision: match the WIRE FORMAT (native Tempo tx), not any npm package.**
> This restores a **third construction + the `payments-tempo` feature**, and makes the
> Tempo tx-encoding spike (Stage 1a) the top build risk — there is no written Tempo MPP
> spec and no `viem/tempo` equivalent for Rust; we match reverse-engineered `viem/tempo`
> output + the one validated capture.

### Protocol version, transport, and the gateway-as-RPC finding

- **x402 is v2** (`x402Version: 2`): CAIP-2 `network`, `amount`, top-level `resource`. The
  correct reference is the `@quicknode/*` packages (→ `@x402/*@^2`), NOT `x402-fetch@1.x`
  (v1 schema, won't parse).
- x402: `POST /:network` (query chain). 402 body carries `accepts[]`; also mirrored in a
  base64 `payment-required:` header. Retry `PAYMENT-SIGNATURE: <base64>`.
- MPP: `POST /:network`. 402 carries multiple `WWW-Authenticate: Payment` challenges in one
  header. Retry `Authorization: Payment <credential>`; success `Payment-Receipt: <base64url>`.
- **Pay-chain RPC is the gateway URL — for x402/Solana ONLY.** `@quicknode/x402-solana`
  hardcodes `rpcUrl:"https://x402.quicknode.com/solana-mainnet"` and builds its Solana RPC
  through `client.fetch` on it — blockhash fetch and payment share one keyless URL. **Does
  NOT generalize to MPP/Tempo:** that path filled its tx against a third-party Tempo RPC
  (`rpc.moderato.tempo.xyz`), or may need no read at all — resolved in Stage 1a.

### x402 `accepts` — a menu with three `extra` shapes

21 entries in a single 402 (7 EVM chains + 2 Solana clusters). Each:
`{scheme:"exact", network, amount, payTo, maxTimeoutSeconds, asset, extra}`. The selector
must distinguish three `extra` shapes:
1. `{name, version}` — standard USDC, EIP-3009. **v1 target.** `verifyingContract` = `asset`.
2. `{name:"GatewayWalletBatched", version, verifyingContract}` — Circle Gateway nanopayment,
   **deferred → SKIP these** (`verifyingContract` is a separate field, not the asset).
3. Solana `{feePayer}` — SPL partial-sign target; gateway feePayer sponsors gas.

### EIP-712 construction (x402/EVM ONLY — NOT MPP; see 0.4)

- Domain: `{name, version, chainId, verifyingContract:<asset>}` (verified against USDC's own
  `name()`/`version()` = `"USDC"`/`"2"`).
- Types: `TransferWithAuthorization: [from:address, to:address, value:uint256,
  validAfter:uint256, validBefore:uint256, nonce:bytes32]`.
- x402 envelope: `base64(JSON({x402Version:2, accepted:<entry>, payload:{signature,
  authorization}}))` → `PAYMENT-SIGNATURE`.
- **Known-good vector for the Stage 1 unit test — REGENERATE from a fresh throwaway key.**
  The captured vector in `scratch/` is from a mainnet-funded address; do not commit it
  (it ties a funded wallet to the repo and publishes a briefly-valid EIP-3009 auth). Mint a
  new key that never touches mainnet, sign the same message offline, commit that.
- **MPP does NOT share this construction.** An earlier draft claimed MPP reused EIP-3009
  typed-data (from an `@quicknode/mpp` source read). 0.4 proved that package unusable against
  the gateway; MPP is the native Tempo tx below.

### MPP credential — native Tempo tx (the confirmed construction)

From `mppx`/`viem/tempo`, validated by the green-200 capture (`0x2a0747b`):
- Build a Tempo type-`0x76` tx: TIP20 transfer (selector `0x95777d59`) to `recipient` for
  `amount`, via `prepareTransactionRequest(nonceKey:"expiring", validBefore, calls)`; with
  `feePayer:true` set and fee fields dropped (gateway sponsors gas). `signTransaction`.
- Credential = `base64url(JSON({ challenge, payload:{signature:<serialized tx>,
  type:"transaction"}, source:"did:pkh:eip155:<chainId>:<addr>" }))` → `Authorization: Payment`.
- Receipt (`Payment-Receipt`, base64url): `{method:"tempo", status:"success", timestamp,
  reference:<settlement tx hash>}`.
- **Concurrency-safe with `nonce=0` — LIVE-CONFIRMED (probe 4, 2026-07-13).** Two fully
  concurrent pay flows with identical `(nonceKey=expiring, nonce=0, validBefore)` both
  settled with distinct references (`scratch/probe-4-mpp-concurrent.mjs`). Uniqueness comes
  from the per-challenge memo (each 402 mints a fresh challenge id), which the driver
  guarantees by never reusing a challenge. No per-call nonce entropy needed. Corollary: do
  NOT sign two credentials against the SAME challenge — that's the one shape this result
  does not cover.
- Whether building this needs a live Tempo RPC read is an open Stage-1a question (see below):
  `nonceKey:"expiring"` may derive the nonce from challenge expiry, and `feePayer:true` drops
  fee estimation, so the Tempo signer might need **zero** chain reads. Confirm in 1a.

### 0.4 — MPP construction (RESOLVED)
Ran `scratch/mpp-qn-sign.mjs` (`@quicknode/mpp`) against the live gateway → threw
`No method found for challenges: tempo.charge … solana.charge. Available: evm.charge`.
The package registers only `evm.charge`; the gateway emits `tempo.charge`/`solana.charge`,
so it fails at routing before any credential. **`@quicknode/mpp@0.2.0` is unusable here.**
⇒ **MPP = native Tempo tx** (the only confirmed path, via `mppx`/`viem/tempo`). We match the
wire format, not the package. Three constructions; `payments-tempo` restored; Tempo tx
encoding is the top build risk (Stage 1a). See that section for the escalation path.
**Still open — is the Solana `getLatestBlockhash` (through the gateway) itself charged/402'd?**
Our solana probe used the client's internal fetch, so our wrapper never saw the sub-request.
If it 402s, the driver needs a nested-payment story. Probe before Stage 2 (wrap the transport
to log every sub-request + status). Same question applies to MPP's Tempo `prepareTransactionRequest`.
**Status**: MPP construction RESOLVED; blockhash-charging sub-question OPEN.

---

## Stage 1: `enum Signer` (three constructions), feature-gated
**Goal**: the three signing constructions as an enum, each verified against its Stage-0
gateway-accepted payload.
**Step 1a — DONE (2026-07-13). Top build risk RETIRED; MPP stays in v1.** Full detail in
`scratch/STAGE1A-FINDINGS.md`; artifacts `scratch/tempo-vector.mjs` + `scratch/tempo-spike/`
(Rust spike, **6/6 byte-for-byte PASS** vs an offline ox/tempo reference vector).
1. **Encode/sign in Rust: YES, via a FIRST-PARTY CRATE — no hand-port.** The "no Rust
   reference, no spec" premise was wrong: **`tempo-primitives` v1.8.1 on crates.io**
   (tempoxyz/tempo node repo, alloy-team-maintained, MIT/Apache-2.0) provides
   `TempoTransaction`, `signature_hash()`, `encode_for_signing()`, expiring-nonce constant,
   0x76/0x78 handling; a written spec exists (tempo.xyz spec-tempo-transaction). The
   credential's `payload.signature` is the **0x78 fee-payer handoff envelope** (sender-signed,
   sender address in the fee-payer slot) — no public serializer for that exact form, ~25
   lines of alloy-rlp (validated in the spike). Constraints: **`default-features = false`**
   (default pulls `revm` + `aws-lc-rs` C/cmake — cross+zig hazard; both gone without it);
   one-line `base64/alloc` feature-unification workaround (upstream no_std bug); **MSRV
   floor becomes Rust 1.93** (CI `@stable` OK today; verify cross images before Stage 5).
2. **Chain reads: ZERO required — `sign_tempo_tx` is SYNC, no RPC param.** Traced in viem
   source: `nonceKey:'expiring'` resolves locally (`nonceKey=U256::MAX, nonce=0,
   validBefore=min(now+25s, challenge expiry)`); `feePayer:true` drops feeToken from the
   sender payload; mppx called `eth_fillTransaction` ONLY to populate `gas` + fee caps, and
   viem skips the fill entirely when those are preset.
3. **Values sliver RESOLVED by the live probes (2026-07-13): the zero-RPC recipe is
   LIVE-CONFIRMED** — a hand-built credential with fixed guessed caps got a green 200 +
   real settlement in exactly 2 gateway requests (`scratch/probe-2-mpp-zerorpc.mjs`).
   Ship generous fixed defaults + config overrides; no fee/gas RPC.

**Deliverables**:
- `crates/core/Cargo.toml` — three feature axes:
  ```
  payments       = ["dep:k256", "dep:alloy-sol-types", …]          # EIP-712 (x402/EVM)
  payments-svm   = ["payments", "dep:ed25519-dalek", "dep:bs58", …] # + x402/Solana (SPL); bs58 for address()
  payments-tempo = ["payments", "dep:tempo-primitives", …]         # + MPP; default-features=false
                                                                    #   (+ base64/alloc unification — 1a)
  ```
- `crates/core/src/rpc/payment/signer.rs`:
  ```rust
  enum Signer { Evm(SecretString), Svm(SecretString), Tempo(SecretString) }
  impl Signer {
      fn kind(&self) -> ChainKind; fn address(&self) -> String;
      fn sign_eip712(&self, domain, message) -> Result<[u8;65], SdkError>;             // sync, x402/EVM
      async fn sign_svm_transfer(&self, req, solana_rpc) -> Result<Vec<u8>, SdkError>; // async, x402/Solana
                                                 // solana_rpc = resolved read source (override → tooling → public), NOT the gateway (1a)
      fn sign_tempo_tx(&self, req) -> Result<Vec<u8>, SdkError>; // sync, no RPC (1a); returns 0x78 envelope
  }
  ```
  Manual `Debug` (`[redacted]`), `#[serde(skip)]` on the key. Constructors take raw
  hex/bytes; never cached.
  - EVM: `k256` + hand-rolled EIP-712 (domain is simple).
  - SVM: `ed25519-dalek`. **Hand-roll the SPL `TransferChecked` instruction rather than
    pulling `spl-token`** (`spl-token`→`solana-program` drags curve25519/MSRV conflicts under
    cross+zig at glibc-2.17 + musl).
  - Tempo: `tempo-primitives` (default-features=false) + `k256`; 0x78 handoff envelope
    hand-assembled with alloy-rlp; memo + credential builders per the 1a wire recipe
    (`scratch/STAGE1A-FINDINGS.md`).
**Success criteria**: `sign_eip712` reproduces the (regenerated, throwaway-key) vector; SVM
signer reproduces its captured green-200 payload byte-for-byte; Tempo signer reproduces the
1a reference vector (already proven in the spike — port the vector as the unit test).
**Status**: **Complete (Rust)**. Signer enum + three constructions implemented; EIP-712
reproduces the throwaway viem vector byte-for-byte, Tempo reproduces the 1a spike vector
6/6, SVM builds a partial-signed TransferChecked tx (live smoke is the Stage 5 gate). All
feature combos build; clippy clean.

## Stage 2: 402 driver + `PaymentScheme` enum + payment error variants
**Goal**: `pay_and_call` — the shared 402 loop; per-scheme parse/select/authorize inline.
(Payment error variants are defined here, not Stage 4 — the driver needs them; the *binding
fan-out* stays in Stage 4.)
**Deliverables** (`crates/core/src/rpc/payment/mod.rs`, `errors.rs`):
- `enum PaymentScheme { X402, MppCharge }`:
  - **parse + select:** parse `accepts[]` (x402 body/header) or split the multi-challenge
    MPP header; select the entry matching the selector, **skip `GatewayWalletBatched` and
    any entry over `max_amount`**. No match ⇒ `PaymentUnsupported` listing what was offered.
    **Amounts are `u128` base units** (EVM amounts are uint256-shaped; u64 overflows for
    18-decimal assets): parse the menu's `amount` string as integer-only — an entry whose
    amount has a decimal point or doesn't parse is skipped like `GatewayWalletBatched`
    (and named in `PaymentUnsupported` if nothing matches). `max_amount` parse failure ⇒
    `Config` error at construction, not at call time.
  - **authorize:** EIP-712 for x402/EVM (sync); SVM tx for x402/Solana (async, reads from the
    resolved Solana RPC source — override → tooling → public, per 1a);
    native Tempo tx for MPP (sync, zero chain reads — 1a). Build header/credential + envelope
    (shapes in research; MPP credential = `{challenge, payload:{signature, type:"transaction"},
    source:"did:pkh:eip155:<chainId>:<addr>"}` → base64url → `Authorization: Payment`).
  - **receipt:** MPP `Payment-Receipt` → typed `PaymentReceipt {method, status, timestamp,
    reference}`; x402 none.
- New `SdkError` variants (definition only here): `PaymentUnsupported`, `PaymentRejected
  {status, body}` (terminal second 402), `PaymentIndeterminate` (retry sent, response lost —
  do not blind-retry). Signing/parse failures reuse `Config`.
  - **`PaymentIndeterminate` classification (decided):** on the *paid resend only*, map
    transport errors by `HttpKind`: `Connect` ⇒ plain `Http` (TCP never established, nothing
    was sent — provably safe to retry); `Timeout` and `Other` ⇒ `PaymentIndeterminate`
    (bytes may have reached the gateway). Errors on the *first, unpaid* request stay plain
    `Http` — no payment exists yet. Future option (not v1): both EIP-3009 and Tempo
    credentials are nonce-idempotent, so resending the *same* credential on a lost response
    may be safe; deferred until gateway dedupe behavior is confirmed.
  - **Clock-skew hint (Tempo):** `validBefore = now+25s` from the local clock, so a skewed
    clock (>~25s behind) signs already-expired credentials and every call ends in
    `PaymentRejected`. When building the `PaymentRejected` error for an MPP credential whose
    `validBefore` is already past at response time, append a "check system clock" hint to
    the message. (x402/EVM windows are wider but get the same check for free if cheap.)
- Driver: build → send on keyless `rpc_http_client()` → on 402 parse→select→authorize→
  **resend exactly once** → 200 capture receipt. Second 402 ⇒ `PaymentRejected`. Lost
  response after the paid resend ⇒ `PaymentIndeterminate`. Driver returns
  `(Value, Option<PaymentReceipt>)` so Stage 3 can surface the receipt via
  `call_with_receipt` while `call` discards it.
**Success criteria**: wiremock tests — happy path per scheme, second-402-terminal,
**lost-response-after-payment ⇒ `PaymentIndeterminate`** (timeout on the paid resend)
while connect-refused on the paid resend ⇒ plain `Http`, multi-challenge parse,
`GatewayWalletBatched` skipped, over-`max_amount` skipped, non-integer amount skipped,
huge (>u64) amount compared correctly, MPP receipt captured.
**Status**: **Complete**. `pay_and_call` driver + `PaymentScheme` + the three error
variants implemented; 25 payment unit/wiremock tests green (x402 happy path, over-max,
GatewayWalletBatched, non-integer, huge>u64 amount, second-402 terminal, lost-response
indeterminate, MPP happy-path+receipt, multi-challenge split).

## Stage 3: Wire into `RpcApiClient::call` + config + lane precedence
**Goal**: a payment lane as a fourth mode, with a defined precedence table (review #7).
**Deliverables**:
- **FFI-safe config shape (review #3 — decided):** the internal `enum Signer { Evm, Svm,
  Tempo }(SecretString)` is enum-with-data, so it CANNOT be `napi(object)`/`pyclass`, so it
  cannot live inside `RpcConfig` (which derives those + `Serialize`/`Clone`/`Default` and
  ships with payments ON in bindings). Resolution: the **binding-facing `PaymentConfig` is
  plain data** — `{ scheme: String, key: String, pay_network: String, asset: String,
  max_amount: String, base_url_override: Option<String> }` — converted to the internal
  `enum Signer` + typed config at the Rust boundary. Keeps `RpcConfig`'s derives intact,
  matches the kwargs-in / typed-struct-out pattern.
  - **`signer_kind` dropped:** the signer variant is derivable from `pay_network` (CAIP-2:
    `eip155:` → Evm, `solana:` → Svm; MPP scheme → Tempo). One fewer field that can only
    agree-with or contradict the others, one fewer validation error.
  - **Manual redacting `Debug` on the boundary `PaymentConfig` (fixes the SDK-side Debug
    trap).** The struct derives everything EXCEPT `Debug`; it gets a hand-written `Debug`
    that prints `key` as `[redacted]`. The field stays readable to the caller (decision (b));
    only the SDK's own `{:?}` rendering redacts — so an SDK log line / error context / panic
    can't leak it. **Copy the in-repo pattern at `crates/core/src/config.rs:165`
    (`CachedToken`).** (The internal resolved config keeps `SecretString`.)
  - **`from_env` must NOT configure payments — enforced by `#[serde(skip)]` on
    `RpcConfig.payment` itself, not on the internal signer.** `from_env` deserializes
    `RpcConfig`, and `PaymentConfig` is all-`String`, so serde would happily populate
    `QN_SDK__RPC__PAYMENT__KEY` unless the whole `payment` field is skipped (`Option` defaults
    to `None`). The caller must pass `PaymentConfig` programmatically. (An env-derived private
    key is exactly what we don't want.)
  - `scheme` is top-level; the selector is just `{pay_network, asset}`.
- **Lane precedence table** in `RpcApiClient::call` (`crates/core/src/rpc/mod.rs:133`),
  matching the existing mutual-exclusion style at `rpc/mod.rs:144`:
  - per-call `endpoint_url` + `payment` ⇒ `Config` error.
  - **client-wide `endpoint_url` + `payment` ⇒ `Config` error** (decided: consistent with
    the per-call rule; a custom self-auth URL and a payment lane are mutually exclusive).
  - `payment` present ⇒ `network` (query chain) is required, routed to the gateway path
    slug; NOT looked up in the seeded tooling network map.
  - no `payment` ⇒ today's behavior unchanged.
  Write the full table + `Config` errors before coding.
- Payment host base is scheme-derived (`x402`/`mpp .quicknode.com`), `base_url_override`
  for tests.
- **`call_with_receipt` (receipt decision):** public method alongside `call`, returning
  `RpcCallResponse { result: Value, payment_receipt: Option<PaymentReceipt> }`. `call`
  delegates and drops the receipt, so both share one driver path. `payment_receipt` is
  `None` for x402 and non-payment lanes. Note for Stage 5: `serde_json::Value` cannot sit
  in a `napi(object)`/`pyclass` field, so `RpcCallResponse` likely needs per-binding
  construction at the FFI boundary (same caveat as the discriminated unions), while
  `PaymentReceipt` itself is plain strings and annotates normally.
- **Keyless construction (decided 2026-07-13): the API key must NOT be required to use the
  payment lane.** Today `SdkFullConfig.api_key` is a required `String`
  (`crates/core/src/config.rs:265`) stamped into a default header at construction — the
  SDK cannot be built keyless. Make it `Option<String>` (constructor kwarg optional in all
  four bindings): absent key ⇒ no auth header installed; admin/streams/webhooks/kvstore/sql
  calls and tooling-JWT `rpc.call` fail with a clear `Config` error ("api_key required");
  payment-lane `rpc.call` works. The SVM signer's chain-read precedence (explicit override
  → tooling endpoint → public RPC) treats the tooling step as best-effort: no API key ⇒
  skip to the public default, never an error. Pre-1.0, so the breaking constructor change
  is acceptable; note it in the changelog and READMEs.
  - **`from_env` stays strict (decided):** `from_env` keeps requiring the API key and fails
    at construction if it's absent — it can't configure payments anyway (`RpcConfig.payment`
    is serde-skipped), so a `from_env` caller by definition wants the keyed lanes, and
    keyless-by-typo'd-env-var must not surface later as a confusing per-call `Config` error.
    Only programmatic construction can omit the key.
**Success criteria**: full handshake against wiremock returns the unwrapped `result`;
`call_with_receipt` returns the parsed MPP receipt on the MPP happy path and `None` for
x402; precedence table covered by Config-error tests; a keyless SDK instance completes a
payment-lane call and gets the clear `Config` error on every other surface.
**Status**: **Complete (Rust)**. `PaymentConfig` (plain data, redacting Debug,
serde-skipped on `RpcConfig`), `api_key` now `Option` (keyless), `from_env` stays strict,
lane precedence + `call_with_receipt`/`RpcCallResponse` wired; integration tests green
(keyless payment call returns unwrapped result, x402 receipt=None, network-required,
endpoint_url+payment Config error, bad max_amount Config error).
**Follow-up (not blocking):** keyed surfaces currently get a server 401 when keyless rather
than a pre-flight `Config` error — the header is simply absent. A clear client-side
"api_key required" guard per keyed client is a nice-to-have.

## Stage 4: Error binding fan-out
**Goal**: surface the Stage-2 payment variants through every binding's typed hierarchy +
the CLI exit buckets. (Variants already exist from Stage 2; this is the fan-out.)
**Deliverables**:
- Map each new variant in every binding: `PaymentRejected` → `ApiError`-family (**this is a
  per-binding + CLI mapping change, not automatic**), `PaymentUnsupported`/signing → a
  Config/QuicknodeError-family class, `PaymentIndeterminate` → its own class so callers can
  catch "do not retry" distinctly. Compiler-enforced arms in Python/Ruby; add to Node match
  + `npm/errors.js`, `__init__.py`, `sdk.d.ts`/`sdk.mjs` if any new class.
- CLI exit-code mapping updated for the new classes.
**Success criteria**: each variant has a mapping arm in every binding; exception-raising
tests in each language example assert the class + `status`/`body`.
**Status**: **Complete**. `PaymentError` family added to Python (`create_exception!` +
`add_to_module`), Ruby (`define_error` + ivar readers + RBS), Node (tagged-message kinds +
`npm/errors.js` classes + `sdk.js`/`.mjs`/`.d.ts`), plus `__init__.py`/`.pyi`. Compiler-
enforced arms in Python/Ruby; all binding crates + clippy green. (No CLI in this repo — the
plan's CLI exit-bucket item is out of scope here.)

## Stage 5: Polyglot bindings + docs (all four SDKs)
**Goal**: expose the payment lane + `enum Signer` construction in Python, Node, Ruby.
**Deliverables**:
- Plain-data `PaymentConfig` per binding (the key is a readable field — decision (b)). The
  binding-facing config is converted to the internal `enum Signer` at the Rust boundary.
  **Redaction test is scoped to SDK-printed surfaces:** a per-binding test asserts the key
  does not appear in the SDK's own error messages / any `Debug` the SDK emits (the internal
  `SecretString` config prints `[redacted]`). It does NOT assert the caller's `PaymentConfig`
  is redacted — that's the accepted, documented exposure. READMEs warn against logging the
  config object.
- **Binding feature-cost reality (review #5):** wheels/npm/gems ship **precompiled with a
  fixed feature set** (presumably all payments features on), so those consumers pay the full
  dep/audit/binary-size cost regardless — "zero cost when off" is true ONLY for crates.io.
  State this in the plan and READMEs. Also: `#[cfg]`'d fields on `pyclass`/`napi(object)`
  change generated TS/stubs per feature combo → a **CI feature-matrix** is required; budget
  it (build each feature combo + a features-off build).
- **Release-matrix build risk (review #6):** add a **branch run of `release.yml`** before
  merge — the SVM surface must cross-compile under cross+zig at glibc-2.17 + musl on both
  arches; Stage 5's local macOS build is not sufficient proof. Hand-rolled SPL (Stage 1)
  reduces but does not eliminate this.
- Public-type exports (CLAUDE.md checklist): `PaymentConfig`, `PaymentScheme`, selector,
  `Signer` via `lib.rs`, plus `RpcCallResponse` + `PaymentReceipt` and the
  `call_with_receipt` method on every binding's rpc client, `__init__.py`+
  `init_manual_override.pyi`+`__all__`, `sdk.d.ts` (+`sdk.mjs`), Ruby binding +
  `quicknode_sdk.rbs`. Watch the discriminated-union caveat for any flattened tagged enum
  (per-binding wrapper, cf. `DestinationAttributes`) — `RpcCallResponse` holds a
  `serde_json::Value` so it takes the per-binding-construction route (Stage 3 note); Ruby
  returns it as an `IndifferentHash` like every other response.
- Four per-language READMEs (config field, env vars, new error classes — Configuration +
  Error tables byte-identical). Examples in all four languages. Payment-lane doc must
  cover: don't log your own `PaymentConfig` (readable key), x402/Solana per-call cost =
  one payment, the public-Solana-RPC default rate-limits (set the explicit override at any
  volume), `PaymentIndeterminate` means "may have been charged — do not blind-retry", and
  `max_amount` is integer base units of the selected asset.
**Success criteria**: `just python-build && node-build && ruby-build && test` green; the CI
feature-matrix green; a branch `release.yml` run green; the SDK does not print the key in its
own error/`Debug` output (boundary `PaymentConfig` has a redacting `Debug`).
- **Live end-to-end smoke (review #3 — the design rests on byte-level wire compatibility with
  reverse-engineered formats, so wiremock alone is insufficient):** through the **Rust SDK**
  (and at least one binding), settle **one real payment per confirmed path** — x402/EVM
  (Base Sepolia testnet), x402/Solana (mainnet, tiny), MPP/Tempo (mainnet, tiny) — spending
  from the throwaway wallets, mirroring the 0.3 checklist. This is the acceptance gate that
  the Rust implementation matches the gateways, not just the mocks. (Keep it a manual/gated
  run, not CI — it moves real funds.)
**Status**: **Complete (mock-verified; live smoke still pending).** `PaymentConfig`
exposed as a `pyclass`/`napi(object)`/Ruby-hash; `call_with_receipt` + receipt on all
three rpc clients (per-binding JSON construction). Public-type exports done across
`lib.rs`, `__init__.py`/`.pyi`, `sdk.d.ts`/`.mjs`/`.js`, Ruby binding + `.rbs`. Four
READMEs get a payment-lane section + byte-identical error rows. Examples in all four
languages (Python/Ruby with no-funds selfchecks that pass; Node `test.js` asserts the
payment surface). CI feature-matrix job added (5 combos, all green locally). Also fixed a
pre-existing branch bug: `ApiCredit`/`GetApiCreditsResponse` were imported in Python
`__init__.py` but never registered as pyclasses, which had made the module unimportable.
**Still pending: the live end-to-end smoke** (one real settled payment per path) — the
acceptance gate that the Rust impl matches the gateways byte-for-byte. Wiremock + the
Stage-0/1a captured vectors cover the construction; the live run moves real funds and is a
gated manual step.

---

## Open questions (live)
1. ~~Tempo tx encoding (Stage 1a)~~ **RESOLVED 2026-07-13** — first-party `tempo-primitives`
   crate + spec exist; spike reproduced the ox/tempo vector 6/6 byte-for-byte; signer is
   sync with zero chain reads. MPP stays in v1. See `scratch/STAGE1A-FINDINGS.md`.
2. ~~Live probe session~~ **RESOLVED 2026-07-13** (probes 1–3, outputs in
   `scratch/STAGE1A-FINDINGS.md` §Live probe results):
   - (a) **Tempo values: fixed defaults work.** A hand-built zero-RPC credential with
     guessed caps (gas 125k, maxFee 1 gwei, maxPrio 0.001 gwei) got a green 200 + real
     settlement, 2 gateway requests total. Ship generous fixed defaults + config
     overrides (sponsor pays the fee, so caps cost the payer nothing); no fee/gas RPC.
   - (b) **Solana sub-reads ARE charged at the gateway** (keyless `getLatestBlockhash`
     402s), and the reference client sources its payment-build reads (`getAccountInfo`,
     `getLatestBlockhash`) from the PUBLIC `api.mainnet-beta.solana.com` instead.
     **Per-call cost = ONE payment** (document in READMEs). New decision for Stage 1/3:
     the SVM signer's RPC source — mirror the reference (public Solana RPC default +
     config override) vs explicit-only. The Stage-0 "pay-chain RPC = gateway URL" claim
     described the wrapper's query transport, not the signer's reads.
3. ~~Concurrent MPP nonce collision~~ **RESOLVED 2026-07-13** (probe 4): two concurrent
   pay flows with identical `nonce=0`/`nonceKey=expiring`/`validBefore` both settled,
   distinct settlement references. v1 recipe is concurrency-safe as designed; uniqueness
   comes from the per-challenge memo. See the MPP credential section.
4. **CLI confirm-gating for per-request spend** — `max_amount` covers the core guard; the
   CLI can likely gate once per session. CLI follow-up. Decide with requester.

*(Resolved: MPP = native Tempo tx (0.4 — `@quicknode/mpp` unusable, gateway emits
`tempo.charge`); THREE constructions + `payments-tempo`; signer = enum-with-data (NOT trait);
FFI-facing `PaymentConfig` is plain data converted at the Rust boundary; `from_env` does not
configure payments; `max_amount` required, base-units integer compare; `PaymentIndeterminate`
for lost-response; lane precedence = Config error for per-call AND client-wide
`endpoint_url`+`payment`; binding feature-cost + release-matrix scoped; test vector
regenerated from a throwaway key; scope label = MPP/Tempo; gitignore `scratch/`;
receipt exposure = `call_with_receipt` returning `RpcCallResponse`, `call` unchanged;
concurrent MPP safe with `nonce=0` (probe 4); `PaymentIndeterminate` = Timeout/Other on the
paid resend only, Connect stays `Http`; Tempo clock-skew hint on `PaymentRejected`;
amounts `u128` integer-only, non-integer entries skipped; `from_env` still requires the
API key; public-Solana-RPC rate-limit warning in READMEs.)*

## Verification per stage
- Rust: `cargo check && just lint`, `cargo test -p quicknode-sdk --lib`, each feature combo
  (`payments`, `payments-svm`, `payments-tempo`) + a features-off build.
- Stage 5: `just python-build && node-build && ruby-build && test`; CI feature-matrix;
  branch `release.yml`.

## scratch/ hygiene
`scratch/` is a real directory **inside the repo working tree** holding funded-wallet
captures + probe scripts referencing a mainnet-funded address. **DONE: `scratch/` is now in
`.gitignore`** (verified out of `git status`). The regenerated throwaway test vector is the
ONLY payment artifact that should enter the repo, and it goes under the crate's test dir,
not `scratch/`.
