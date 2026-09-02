# Closure — Y001 Bounded SESSION CREATE Option Surface

Disposition: **closed**

Plan: `plans/implementation/001-bounded-session-create-option-surface.md`

## Implementation commits

- `beafafa33e563760a0484df1b5fcaec4e0f8c5e4` — implement and test the bounded option surface.

This is the exact production commit that a consuming plan may pin. Closure and registry records are
documentation-only follow-up work.

## Requirement-to-evidence mapping

| Requirement | Evidence |
| --- | --- |
| All five typed fields reach `SESSION CREATE` | `src/proto/session.rs` serializes `SIGNATURE_TYPE`, both `lengthVariance`, and both `backupQuantity` fields; controller tests cover defaults, a non-default signature type, positive/negative variance, and non-zero backups. |
| Default compatibility | `SessionOptions::default()` retains signature type `7`, existing tunnel lengths/quantities, styles, and destination handling; default command test asserts the canonical typed suffix. |
| Bounded generic surface | `SessionOption` validates key/value grammar and byte limits; `SessionOptions::add_session_option` enforces duplicate/count limits; controller construction and serialization defensively revalidate the public collection. |
| Collision safety and deterministic output | Reserved structural/typed keys are rejected, duplicate keys are rejected case-insensitively, and additional options are sorted by key before serialization. Tests exercise structural, typed, duplicate, count, and boundary cases. |
| Injection-safe framing | Empty/invalid keys, key delimiters, whitespace/control characters, CR/LF/NUL, quotes, backslashes, equals, and value token injection are rejected before command construction. The command test verifies one newline and no trailing token delimiter. |
| Secret/debug handling | Manual `Debug` implementations redact session secrets and additional option values; controller tests verify persistent destination/private data, LeaseSet material, and added option values do not appear. |
| Consumer isolation | Changed production paths are limited to generic `SessionOption`/`SessionOptions`, `ProtocolError`, and the generic session controller. No consumer-specific names or APIs were added. |

Changed production paths:

- `src/options.rs`
- `src/proto/session.rs`
- `src/error.rs`
- `src/lib.rs`

## Verification outcomes

Passed:

- `cargo test --features tokio` — 25 passed, 1 ignored.
- `cargo check --features tokio`.
- `cargo test --lib --no-default-features --features smol` — 18 passed.
- `cargo check --no-default-features --features smol`.
- `cargo test --lib --no-default-features --features sync` — 18 passed.
- `cargo check --no-default-features --features sync`.
- `git diff --check`.

Not clean due to repository-baseline/toolchain issues, with no Y001-specific finding:

- `cargo clippy --all-targets --features tokio -- -D warnings` reports existing parser/router/style lint violations; the only session-controller location reported is the unchanged pre-existing `destination()` borrow at `src/proto/session.rs:700`.
- `cargo fmt --all -- --check` reports only pre-existing stable-rustfmt differences in `src/proto/router.rs` and the unchanged response-match arm in `src/proto/session.rs`; the Y001 additions were formatted.
- `cargo test --no-default-features --features smol` reaches pre-existing examples without a runtime feature and fails because those examples have no `main`; the valid library-only smol gate passes above. The literal `cargo test --features smol` form was not used as a gate because it combines default Tokio with smol, which the crate explicitly rejects.

## Compatibility and security review

Existing fields were not renamed or removed, and callers that do not set additional options retain the same default semantic configuration. The new public option wrapper cannot represent raw command fragments; validation bounds count/key/value sizes, rejects controls/token delimiters, reserves structural and typed keys, and rejects duplicates. Invalid public collection mutations fail before controller state changes or command bytes escape. Debug output no longer exposes existing secret-bearing fields or new option values.

No dependency, feature, runtime, router behavior, cryptography, TLS, upstream, CI, release, or consumer-policy changes were made.

## Unresolved findings

None within Y001 scope. Router support for requested options remains explicitly outside this plan.

## Future-plan disposition

Y002 is unblocked and promoted from proposed/blocked to **ready**, with its baseline frozen to
`beafafa33e563760a0484df1b5fcaec4e0f8c5e4`. Y003 remains **proposed / semantically blocked** until
Emissary M113 freezes the exact reference-backed LeaseSet/client-auth contract and the plan is
explicitly promoted.
