# Closure — Y002 Signature-Aware Destination Generation

Disposition: **closed**

Plan: `plans/implementation/002-signature-aware-destination-generation.md`

## Implementation commit

- `8026f5b424fc178d683e63555335f8b33e0aba04` — add signature-aware destination generation and protocol-error propagation.

This is the exact production commit that Emissary M117 may pin. This closure and the registry/roadmap status updates are documentation-only follow-up work.

## Requirement-to-evidence mapping

| Requirement | Evidence |
| --- | --- |
| Parameterized controller API | `RouterApiController::generate_destination_with_signature_type(u16)` formats `DEST GENERATE SIGNATURE_TYPE={value}`. |
| Default compatibility | `generate_destination()` remains available and delegates to type `7`; controller, async public, and sync public tests assert `DEST GENERATE SIGNATURE_TYPE=7\n`. |
| Exact selected wire value | Controller tests assert the exact type-11 bytes and count one `SIGNATURE_TYPE=` occurrence. |
| Async and sync public paths | Local mock-router tests in `src/asynchronous/router.rs` and `src/synchronous/router.rs` observe selected type `11` on the TCP wire and verify the unchanged `(destination, private_key)` response tuple. Tokio and smol variants are covered. |
| Shared protocol implementation | Both public API families use the parameterized `RouterApiController` method through their shared command/handshake/response flow; no command formatting is duplicated in the runtime modules. |
| Router-error propagation/no fallback | `DEST REPLY RESULT=INVALID_KEY` is parsed into the generic destination-generation error path and returned as `ProtocolError::Router(I2pError::InvalidKey)`. The controller becomes terminally poisoned after the error, with no type-7 retry. |
| Response and secret handling | Successful `PUB`/`PRIV` parsing and `generated_destination()` handling are unchanged. The new trace records only the numeric signature type and does not log destination/private-key response data. |
| Scope and consumer isolation | Changed production paths are `src/asynchronous/router.rs`, `src/proto/parser.rs`, `src/proto/router.rs`, and `src/synchronous/router.rs`; no consumer-specific names, signing algorithms, dependencies, TLS, lifecycle, LeaseSet crypto, or upstream changes were added. |

## Verification outcomes

Passed:

- `cargo test --features tokio` — 31 passed, 1 ignored.
- `cargo test --lib --no-default-features --features smol` — 24 passed.
- `cargo test --lib --no-default-features --features sync` — 24 passed.
- `cargo test --no-default-features --features sync` — 30 passed, 1 ignored.
- `cargo check --features tokio`.
- `cargo check --no-default-features --features smol`.
- `cargo check --no-default-features --features sync`.
- `git diff --check`.

Not clean due to repository-baseline/toolchain issues, with no Y002-specific finding:

- `cargo clippy --all-targets --features tokio -- -D warnings` reports existing parser/router/session lint violations; Y002-specific additions were checked and the new test-module ordering allowance is limited to the new test module.
- The corresponding smol and sync clippy commands report the same existing parser/session/style violations.
- `cargo fmt --all -- --check` reports unchanged stable-rustfmt differences in existing `src/proto/router.rs` and `src/proto/session.rs` match arms. Y002 changes are formatted without accepting those unrelated baseline rewrites.

## Compatibility and security review

The public parameterless methods retain their signatures and type-7 behavior. The new `u16` parameter reuses Y001’s stable signature representation and is serialized exactly once without a client-side whitelist or downgrade. Router support remains authoritative; router rejection is propagated and not retried. Existing destination/private-key response handling is preserved, and no response secrets are included in the new logging fields or test artifacts.

No dependency, feature, runtime, cryptography, TLS, session lifecycle, release, CI, or upstream interaction changes were made.

## Unresolved findings

None within Y002 scope. Router support for a selected signature type and the decision of which types Emissary supports remain outside Yosemite.

## Future-plan disposition

The Emissary M117 adoption dependency is unblocked and may pin `8026f5b424fc178d683e63555335f8b33e0aba04`.

Y003 remains **proposed / semantically blocked**. Its Emissary M113 dependency has not frozen the exact reference-backed LeaseSet/client-auth semantics, so no Yosemite future implementation plan can be promoted to `ready`.
