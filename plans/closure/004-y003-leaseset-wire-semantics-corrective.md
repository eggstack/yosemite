# Closure — Y004 Y003 LeaseSet Wire Semantics Corrective

Status: **closed**

Plan: `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`

Corrective target: `plans/closure/003-leaseset-session-option-surface.md`

Baseline: `94d7455c9f78ebb74b7a68823e921db0d76c85c1`

Implementation commit: `c2db73dba35dd9392947af5c74df29b0b556775f`

## Disposition

Y004 is complete. It supersedes Y003's LeaseSet wire-semantic claims while preserving Y003 as an unmodified historical record. Yosemite now owns a canonical, bounded, mode-aware LeaseSet option surface for API-to-SAM serialization only. No LeaseSet cryptography, router behavior, Proposal 170 policy, Emissary source, dependency pin, or upstream activity was added.

## Independent reference freeze

The read-only reference freeze used the [I2CP specification](https://i2p.net/en/docs/specs/i2cp/), the Java [RequestLeaseSetMessageHandler property and parsing code](https://github.com/i2p/i2p.i2p/blob/master/core/java/src/net/i2p/client/impl/RequestLeaseSetMessageHandler.java), the Java [I2PTunnel client-auth option construction](https://github.com/i2p/i2p.i2p/blob/master/apps/i2ptunnel/java/src/net/i2p/i2ptunnel/ui/TunnelConfig.java), and the Java [I2P Base64 implementation](https://github.com/i2p/i2p.i2p/blob/master/core/java/src/net/i2p/data/Base64.java).

| Wire property | Frozen semantics |
| --- | --- |
| `i2cp.encryptLeaseSet` | Boolean enable flag; Yosemite emits `true` only when enabled. |
| `i2cp.leaseSetAuthType` | `0` none, `1` DH, `2` PSK; values outside `0..=2` reject. Zero remains omitted by default. |
| `i2cp.leaseSetBlindedType` | Two-byte numeric signature-type identifier, represented by `0..=65535`; zero selects the default and is omitted. |
| `i2cp.leaseSetType` | One-byte LeaseSet type identifier, represented by `1..=255`; type `1` is the default and is omitted. Reference examples include `1`, `3`, `5`, and `7`. |
| `i2cp.leaseSetKey` | Base64 session key material, retained as a typed bounded value. |
| `i2cp.leaseSetPrivateKey` | Persistent LeaseSet encryption private key. This is the corrected mapping for `SessionOptions::lease_set_private_key`. |
| `i2cp.leaseSetPrivKey` | Distinct local X25519 private key used by the router for local encrypted-LeaseSet decryption; it is not an alias for `leaseSetPrivateKey` and is deliberately not modeled by Y004. |
| `i2cp.leaseSetSecret` | Base64-encoded UTF-8 blinding secret. |
| `i2cp.leaseSetSigningPrivateKey` | Persistent LeaseSet signing private key. This is the corrected mapping for `SessionOptions::lease_set_signing_private_key`. |
| `i2cp.leaseSetClient.dh.<n>` | Per-client DH entry, numbered contiguously from zero; value is `b64name:b64pubkey`. |
| `i2cp.leaseSetClient.psk.<n>` | Per-client PSK entry, numbered contiguously from zero; value is `b64name:b64privkey`. |

For typed DH/PSK entries, Yosemite accepts a raw UTF-8 client name and a strict I2P-Base64 32-byte key, then constructs the combined value. The client name is I2P-Base64 encoded from UTF-8; the key uses the I2P alphabet and valid padding/unused-bit rules. A caller cannot supply a raw `i2cp.*=...` fragment.

## Requirement-to-evidence mapping

| Requirement | Evidence |
| --- | --- |
| WP1 independent protocol fixture | `session_create_serializes_leaseset_typed_options` in `src/proto/session.rs` uses literal canonical keys, including both private-key spellings and both client-auth namespaces. |
| WP2 canonical typed fields and conflicts | `src/proto/session.rs` emits `leaseSetPrivateKey` and `leaseSetSigningPrivateKey`; `src/options.rs` reserves canonical keys, the distinct `leaseSetPrivKey`, historical aliases, and all numbered DH/PSK namespaces. |
| WP3 typed client authorization | `LeaseSetClientAuthMode` and `LeaseSetClientAuth` in `src/options.rs` enforce mode/name/key structure, bounded inputs, I2P-Base64 grammar, redacted `Debug`, per-mode duplicate rules, and constructed `b64name:b64key` values. `src/proto/session.rs` assigns deterministic contiguous per-mode numbers. |
| WP4 exact numeric domains | `SessionOptions::validate_lease_set_options` in `src/options.rs` enforces auth `0..=2`, blinded type `0..=u16::MAX`, and LeaseSet type `1..=u8::MAX`; controller-level boundary and malformed-value tests are in `src/proto/session.rs`. |
| WP5 compatibility/security review | Controller-level tests cover default wire stability, Y001/Y002 regression behavior, redaction, injection rejection, typed/generic collisions, invalid-before-wire behavior, duplicate auths, and deterministic serialization. |

## Representative command evidence

The controller-level fixture verifies these representative values byte-for-byte in the generated `SESSION CREATE` command:

```text
i2cp.leaseSetPrivateKey=BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=
i2cp.leaseSetSigningPrivateKey=CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=
i2cp.leaseSetClient.dh.0=YWxpY2U=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
i2cp.leaseSetClient.psk.0=YWxpY2U=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
```

The default `SessionOptions` fixture asserts that none of the canonical LeaseSet-security keys are emitted. Mixed insertion-order fixtures assert deterministic per-mode numbering, and malformed names/keys, duplicate logical names, generic namespace collisions, invalid numeric values, and direct invalid controller options all fail before command bytes are returned.

## API compatibility and security review

- Callers that leave LeaseSet features unused retain the default wire behavior. Y001 generic option grammar, count/size limits, collision protection, variance/backup serialization, and Y002 signature-aware destination generation remain covered by the existing tests.
- The old one-argument opaque `LeaseSetClientAuth::new(key)` shape was source-level corrected to require `mode`, `client_name`, and `key`. There is no unambiguous conversion from that token to the reference DH/PSK namespace and client identity, so a compatibility wrapper would preserve the defective semantics. The new `new(mode, name, key)`, `dh(name, key)`, `psk(name, key)`, and `SessionOptions` convenience methods are the supported API.
- Generic options cannot occupy any canonical typed LeaseSet key or DH/PSK numbered namespace. The historical incorrect aliases remain reserved and are never emitted.
- Client-auth names and key material are bounded and validated before serialization. Key, secret, password, and option values remain redacted in ordinary `Debug` output; validation errors remain generic and do not echo secret material.
- Direct mutation of the public options fields is defensively revalidated by `SessionController::create_session`; invalid options leave the controller in `Handshaked` and return no weaker command.
- Only `src/options.rs`, `src/proto/session.rs`, and `src/lib.rs` changed in the implementation commit. No dependency, feature, router, crypto, TLS, CI, release, consumer, or upstream source changed.

## Verification outcomes

| Command | Outcome |
| --- | --- |
| `cargo test --features tokio` | Passed: 39 passed, 1 ignored across 2 suites. |
| `cargo test --lib --no-default-features --features smol` | Passed: 32 passed. |
| `cargo test --lib --no-default-features --features sync` | Passed: 32 passed. |
| `cargo test --no-default-features --features sync` | Passed: 38 passed, 1 ignored across 2 suites. |
| `cargo check --features tokio` | Passed. |
| `cargo check --no-default-features --features smol` | Passed. |
| `cargo check --no-default-features --features sync` | Passed. |
| `cargo clippy --all-targets --features tokio -- -D warnings` | Baseline-only failure. The repository still reports existing parser/router/session style lints and async-function suggestions (for example `src/proto/parser.rs`, `src/proto/router.rs`, `src/proto/session.rs:767`, and `src/asynchronous/session/style/*`); no Y004-added diagnostic remained after the test cleanup. |
| `cargo fmt --all -- --check` | Baseline-only failure. The remaining diffs are pre-existing state-machine formatting in `src/proto/router.rs` and `src/proto/session.rs`; the Y004 changes themselves are formatted. |
| `git diff --check` | Passed before the implementation commit. |

The Clippy and format failures are explicitly dispositioned rather than waived for new Y004 code. No new Y004 lint or whitespace diagnostic remains.

## Unresolved findings and future disposition

No in-scope Y004 finding remains open. The distinct `i2cp.leaseSetPrivKey` property is intentionally reserved but not exposed because Y004 has no current consumer requirement for router-local decryption-key semantics; a future request for it needs a separate typed plan and reference review.

Y004 removes the Yosemite-side blocker for a future Emissary exact-revision dependency-pin/LeaseSet retry plan. Current Emissary remains pinned to Y002, and no consumer-side source or dependency change is implied by this closure. Adoption may proceed only through a separately reviewed consumer plan selecting the exact Y004 implementation commit above.
