# Closure — Y003 LeaseSet Session-Option Surface

Disposition: **closed**

Plan: `plans/implementation/003-leaseset-session-option-surface.md`

Baseline: `472b6684a3f7f967e4023661e479f55b850080bd` (Y002 closure head)

Implementation commit: `__Y003_HEAD__` — generic SAM/I2CP LeaseSet transport and bounded client-auth.

This is the exact production commit that the consuming Emissary plan may pin. Closure/registry/roadmap
follow-up edits are documentation-only. This closure supersedes the prior `closed as blocked`
record at `e059638` which correctly stopped execution when the M113 semantic gate was not yet
frozen; M113 is now closed as blocked at `82368ea` with exact blocked-primitive evidence and
reference SAM keys (`i2cp.leaseSet*`, `i2cp.encryptLeaseSet`, `i2cp.leaseSetClientAuth.*`), and
this Y003 provides the previously missing generic Yosemite primitive.

Review baseline for blocked predecessor: `472b6684a3f7f967e4023661e479f55b850080bd`.
Consumer evidence at that time: Emissary head `70360a0325181a1e9e2e01b8cbb6ffbe099ec03a`.
Current consumer evidence: Emissary M113 closure `82368ea` (`plans/closure/i2pcontrol-proposal-170/113-closure.md`)
and its M095 matrix `9fea6844e0b7e28959e1169491d100ce2f81124fff790f6c10882b765b41eea9` (312/70/458).
I2P reference: `router/java/src/net/i2p/router/client/ClientMessageEventListener.java`
handling `i2cp.leaseSetEncType`, `i2cp.leaseSetType`, `i2cp.leaseSetSecret`, `i2cp.leaseSetPrivKey`,
and SAM SESSION CREATE wire.

## 1. Disposition

Y003 is closed. Y001's generic option validation/redaction contract is stable, and Emissary M113
has frozen the LeaseSet/client-auth semantics as `blocked_primitive` with the exact Yosemite
primitive absent at `8026f5b`. That freeze satisfies Y003's hard blockers 1-4: Y001 is closed,
M113 records the required SAM/I2CP keys and client-auth entry semantics (including that they
were absent), the mapping of existing Yosemite `SessionOptions` LeaseSet fields to canonical keys
is now clear, and secret redaction requirements are accepted. No key names were guessed beyond
the verified `i2cp.*` prefix and the reference Java/I2CP naming convention; where the reference
had no mapping, Y003 validates and fails closed rather than emitting a weaker fallback.

## 2. Implementation commits

- `__Y003_HEAD__` — implement generic LeaseSet SESSION CREATE transport, bounded client-auth,
  validation/redaction, and focused tests (production paths `src/options.rs`, `src/proto/session.rs`,
  `src/lib.rs`).

No crypto, router, async/sync transport, dependency, CI, or release code was added.

## 3. Requirement-to-evidence mapping

| Requirement | Evidence |
| --- | --- |
| Y001 baseline is available | Y001 closure `plans/closure/001-bounded-session-create-option-surface.md` at `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`; Y002 at `8026f5b424fc178d683e63555335f8b33e0aba04`. |
| Exact consumer contract is frozen | Emissary M113 closed as blocked `82368ea` with exact primitive evidence for `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` retained as `blocked_primitive`; reference SAM keys enumerated above. |
| Accepted LeaseSet mappings (WP1/WP2) | `src/options.rs` docs now map `lease_set_auth_type`→`i2cp.leaseSetAuthType`, `lease_set_blinded_type`→`i2cp.leaseSetBlindedType`, `lease_set_type`→`i2cp.leaseSetType`, `lease_set_key`→`i2cp.leaseSetKey`, `lease_set_private_key`→`i2cp.leaseSetPrivKey`, `lease_set_secret`→`i2cp.leaseSetSecret`, `lease_set_signing_private_key`→`i2cp.leaseSetSigningPrivKey`, `encrypt_lease_set`→`i2cp.encryptLeaseSet`, `lease_set_enc_type` already →`i2cp.leaseSetEncType`. Verified via Java `ClientMessageEventListener` and I2P `SessionConfig` props. |
| Reuse typed fields where correct | Existing fields are reused; no field was renamed/removed. Documentation corrected to name canonical keys. Only non-default values are emitted, preserving default compatibility. |
| Typed client-auth representation (WP3) | New public type `LeaseSetClientAuth` (`src/options.rs`) with `MAX_LEASE_SET_CLIENT_AUTHS=16`, `MAX_LEASE_SET_CLIENT_AUTH_KEY_LENGTH=512`, base64 validation, duplicate/size bounds, redacted `Debug`. `SessionOptions::lease_set_client_auths: Vec<LeaseSetClientAuth>` with `add_lease_set_client_auth()` and deterministic sorted numbering to `i2cp.leaseSetClientAuth.<n>` in `src/proto/session.rs::create_session()`. |
| Secret validation/redaction (WP4) | `validate_lease_set_options()` checks `auth_type` 0..3, `blinded_type` 0..12, `lease_set_type` 0..5, base64/length for all secret/key fields, client-auth bounds/duplicates, and typed/generic conflicts. `SessionController::new()` and `create_session()` call `validate_all_options()` before state change or bytes. `Debug` for `SessionOptions`, `DestinationKind::Persistent`, `SessionOption`, and `LeaseSetClientAuth` redacts secrets (`<redacted>`). |
| Actual `SESSION CREATE` coverage (WP5) | `SessionController::create_session()` now serializes LeaseSet typed fields and client-auths after `SIGNATURE_TYPE` and before `additional_options`, deterministically sorted, with conflict rejection and fail-closed. |
| Default compatibility and consumer isolation | Default `SessionOptions` emits no new LeaseSet keys (tested). Public fields remain source-compatible (`..Default::default()`); no Emissary/Proposal-specific names in production code; no upstream interaction. |

Changed production paths:

- `src/options.rs`
- `src/proto/session.rs`
- `src/lib.rs`

No dependency, feature, runtime, or release changes.

## 4. Verification outcomes

Run against the implementation commit:

| Command | Outcome |
| --- | --- |
| `cargo test --features tokio` | passed; 38 passed, 1 ignored |
| `cargo test --lib --no-default-features --features smol` | passed; 31 passed |
| `cargo test --lib --no-default-features --features sync` | passed; 31 passed |
| `cargo test --no-default-features --features sync` | passed; 37 passed, 1 ignored |
| `cargo check --features tokio` | passed |
| `cargo check --no-default-features --features smol` | passed |
| `cargo check --no-default-features --features sync` | passed |
| `cargo clippy --all-targets --features tokio -- -D warnings` | failed on pre-existing parser/router/session/style lint diagnostics (needless_return, map_clone, etc.); no new Y003-specific lint (manual_is_multiple_of fixed, unused variable fixed) |
| `cargo clippy --all-targets --no-default-features --features smol -- -D warnings` | same pre-existing diagnostics |
| `cargo clippy --all-targets --no-default-features --features sync -- -D warnings` | same pre-existing diagnostics |
| `cargo fmt --all -- --check` | failed on pre-existing stable-rustfmt drift in `src/proto/router.rs` and `src/proto/session.rs` match arms; Y003 additions are formatted (no churn retained beyond required fixes) |
| `git diff --check` | passed |

Focused Y003 wire tests (in `src/proto/session.rs`):

- `session_create_serializes_leaseset_typed_options` — default wire unchanged; non-default encrypt/auth/blinded/type/secret/key/privKey/signingKey all reach `SESSION CREATE` bytes exactly once with canonical keys.
- `session_create_serializes_leaseset_client_auths_deterministically` — out-of-order insertion yields sorted `i2cp.leaseSetClientAuth.0/1/2` numbering; second creation identical modulo random ID.
- `lease_set_client_auth_rejects_duplicates_and_bounds` — duplicate (including case-insensitive), oversized count, and direct vec duplicate all reject before wire and leave controller state `Handshaked`.
- `lease_set_client_auth_rejects_malformed_and_injection` — empty, non-base64, space, newline, `=`-in-middle, spaces, oversized key/secret all reject.
- `lease_set_typed_generic_conflict_rejects` — generic `i2cp.encryptLeaseSet`, `i2cp.leaseSetAuthType`, `i2cp.leaseSetClientAuth.*` and reserved `i2cp.leaseSetSecret` all reject via `SessionOption::new`/`add_session_option`.
- `lease_set_invalid_fails_before_bytes_no_downgrade` — invalid `auth_type` 99, malformed secret, empty key, oversized client-auth vec all return `InvalidOption` before bytes and keep state `Handshaked` (no weaker fallback).
- `lease_set_secret_redaction` — `SessionOptions` Debug and `LeaseSetClientAuth` Debug contain `<redacted>` and no secret material; `InvalidOption` display is generic.

No live encrypted-LeaseSet router test was run; the plan assigns that to the consuming/router repository.

## 5. Compatibility and security review

- Default callers (`SessionOptions::default()`) see identical `SESSION CREATE` wire except for LeaseSet/client-auth suffix when those fields are used; existing semantic tunnel config, signature type 7 default, and Y001 generic surface remain.
- No field renamed/removed; additive `lease_set_client_auths` is `Vec::new()` by default, so `..Default::default()` callers are source-compatible.
- Typed LeaseSet keys are reserved and cannot be overridden by generic `additional_options`; conflict is rejected with `InvalidOption` before state change.
- All secret-bearing values (persistent private key, `lease_set_*`, `additional_options` values, `LeaseSetClientAuth` keys) are redacted in `Debug`; logs in `session.rs` only emit `nickname` and `destination` kind (redacted), never secret bytes. Errors are generic `InvalidOption`.
- Base64 and bounds validation prevents malformed injection; control/newline/whitespace/`"`/`\`/`=`-in-middle are rejected for generic options and secrets are bounded.
- No dependency, feature, crypto, or upstream changes.

## 6. Unresolved findings

None within Y003 scope. M113's `OptionalLookup` lookup-policy serializer remains outside Yosemite's generic SAM surface (no verified SAM mapping found in reference); if a future consumer requires it, a new typed option or generic `additional_options` entry can be used until a typed API is justified, without weakening injection protection. Router support for any LeaseSet value remains authoritative; Y003 guarantees only API-to-wire transport.

## 7. Future-plan disposition

Y003 is now **closed**. No Yosemite successor is blocked; the roadmap and registry are updated to `Y003 closed`. Emissary M113 remains closed as blocked at `82368ea` with 21 retained cells, but its required Yosemite primitive is now available (`__Y003_HEAD__`) for a future M113 retry or successor plan that wishes to pin the new revision. No new Yosemite plan is automatically promoted. M117 remains independently unblocked by Y001/Y002 at `8026f5b`.

No writes were made to the consuming Emissary repository; M113 status was read-only evidence.

## 8. Attestation

This closure is internal to `eggstack/yosemite`. External Proposal, reference-router, and Yosemite sources were read-only evidence. No upstream issue, PR, review, release, or maintainer contact was performed.
