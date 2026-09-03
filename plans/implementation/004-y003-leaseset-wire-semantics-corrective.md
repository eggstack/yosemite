# Y004 — Y003 LeaseSet Wire Semantics Corrective

Status: **ready**

Class: corrective / protocol serialization / security containment

Baseline: `94d7455c9f78ebb74b7a68823e921db0d76c85c1`

Corrects:

- `plans/implementation/003-leaseset-session-option-surface.md`;
- `plans/closure/003-leaseset-session-option-surface.md`.

Source roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`.

Consumer context:

- internal `eggstack/emissary` Proposal 170 workstream only;
- current Emissary pin remains Y002 implementation commit `8026f5b424fc178d683e63555335f8b33e0aba04` and therefore does **not** consume the defective Y003 surface.

## 1. Objective

Correct Y003 so Yosemite's generic LeaseSet/session option API emits the canonical I2CP property names, value domains, and per-client authorization representation required by the pinned I2P reference contract.

Y004 is API-to-SAM-wire plumbing only. It does not implement LeaseSet cryptography, router-side blinding/encryption, Proposal 170 policy, Emissary persistence, or client authorization decisions.

## 2. Defects being corrected

Y003 closed with tests that verified the implementation's own chosen keys but did not independently compare every emitted key and value domain against the authoritative Java/I2CP vocabulary. That allowed protocol-shaped but non-canonical names to pass local byte tests.

The current Y003 implementation has the following corrective findings:

1. `SessionOptions::lease_set_private_key` is emitted as `i2cp.leaseSetPrivKey`. The canonical persistent LeaseSet encryption-private-key property is `i2cp.leaseSetPrivateKey`; `i2cp.leaseSetPrivKey` is a distinct property and MUST NOT be used as an alias.
2. `SessionOptions::lease_set_signing_private_key` is emitted as `i2cp.leaseSetSigningPrivKey`; the canonical property is `i2cp.leaseSetSigningPrivateKey`.
3. Per-client authorization entries are emitted as `i2cp.leaseSetClientAuth.<n>`. The reference I2PTunnel/I2CP vocabulary distinguishes authorization mode with `i2cp.leaseSetClient.dh.<n>` and `i2cp.leaseSetClient.psk.<n>`.
4. `LeaseSetClientAuth` currently stores one opaque base64 token. Canonical DH/PSK entries carry a bounded client name and key payload on the wire; the API must model enough structure to serialize the exact accepted form without exposing a raw command fragment.
5. `lease_set_auth_type` validation accepts a wider domain than the reference contract used by the consumer. Y004 must freeze and enforce the exact defined values rather than accepting speculative forward-compatible values.
6. `lease_set_blinded_type` is artificially limited to a small range even though the protocol field is a wider signature-type identifier. Y004 must validate against the actual wire type/domain.
7. `lease_set_type` is artificially limited to `0..=5`, excluding valid LeaseSet type identifiers. Y004 must freeze the actual protocol domain and cover at least the values required by the pinned reference.
8. Y003 documentation and closure repeat the incorrect client-auth and private-key names, so a new closure must supersede those claims without rewriting the historical Y003 record.

## 3. Canonical/reference freeze

Before production edits, record exact read-only evidence from the pinned/current I2P reference sources for:

- `i2cp.encryptLeaseSet`;
- `i2cp.leaseSetAuthType` and its exact value domain;
- `i2cp.leaseSetBlindedType` and its numeric wire domain;
- `i2cp.leaseSetType` and its numeric wire domain;
- `i2cp.leaseSetKey`;
- `i2cp.leaseSetPrivateKey`;
- `i2cp.leaseSetPrivKey`, explicitly documenting why it is a different semantic property and is not an alias for `leaseSetPrivateKey`;
- `i2cp.leaseSetSecret`;
- `i2cp.leaseSetSigningPrivateKey`;
- `i2cp.leaseSetClient.dh.<n>`;
- `i2cp.leaseSetClient.psk.<n>`;
- exact value grammar for DH/PSK entries, including how client name and key are represented.

Use Java I2PTunnel/I2P client code and the I2CP specification as authorities. If two authorities materially disagree, stop and record the conflict rather than choosing whichever makes the existing API easiest to preserve.

No key may be retained merely because Y003 already emitted it.

## 4. Required production changes

Expected production paths are limited to generic Yosemite owners:

- `src/options.rs`;
- `src/proto/session.rs`;
- `src/lib.rs` only for public type re-exports;
- `src/error.rs` only if a generic validation error distinction is independently necessary.

No dependency, Cargo feature, runtime, router, transport, crypto, TLS, CI, release, or consumer-specific source change is authorized.

### 4.1 Correct typed LeaseSet keys

Map existing fields only when their semantics exactly match the reference:

- `lease_set_key` → `i2cp.leaseSetKey`;
- persistent LeaseSet encryption private key → `i2cp.leaseSetPrivateKey`;
- secret → `i2cp.leaseSetSecret`;
- persistent LeaseSet signing private key → `i2cp.leaseSetSigningPrivateKey`.

Do not silently map any existing field to `i2cp.leaseSetPrivKey` unless the field is explicitly renamed/redefined or a new typed field is added with the exact local-decryption-key semantics and there is a current consumer requirement for it.

If correcting a public field's wire mapping is source-compatible, preserve the field name and fix its documentation/serializer. If the existing public field name is semantically ambiguous, prefer additive typed APIs and deprecation documentation over breaking removal.

### 4.2 Replace opaque client-auth representation

Replace or supersede `LeaseSetClientAuth { key }` with a generic typed representation that encodes at least:

- authorization mode: DH or PSK;
- bounded client name/identifier in the reference-supported grammar;
- bounded key material in the exact expected encoding.

The public API MUST NOT accept a preformatted `i2cp.*=...` fragment.

Serialization must produce deterministic contiguous numbering per mode using canonical keys:

- `i2cp.leaseSetClient.dh.0`, `.1`, ...;
- `i2cp.leaseSetClient.psk.0`, `.1`, ... .

If the reference requires a combined value such as `<encoded-name>:<encoded-key>`, construct it from validated typed components. Do not relax Y001's generic token injection rules to accommodate it.

Ordering must be deterministic and independent of insertion order. Duplicate logical clients within the same mode must reject. Define and test whether the same client name may appear once in each distinct mode according to the reference behavior.

### 4.3 Correct value domains

Freeze exact protocol value domains and reject outside them before controller state changes or command construction.

At minimum, the corrective must remove the speculative Y003 ranges and test:

- every accepted `leaseSetAuthType` value used by the reference plus one below/above the domain;
- `leaseSetBlindedType` at zero/default, representative supported values, and the numeric wire boundary;
- `leaseSetType` at the default and representative non-default valid values including values above 5 when allowed by the reference;
- malformed, overflowing, signed-negative, whitespace, and non-decimal numeric values through the public construction/serialization path where applicable.

Do not validate cryptographic support that belongs to a router. Yosemite validates representation/domain, not whether the connected router can actually construct a given LeaseSet.

## 5. Invariants

Y004 MUST preserve:

- Y001 bounded generic option grammar and collision protection;
- Y001/Y003 secret redaction in `DestinationKind`, `SessionOption`, `SessionOptions`, and client-auth types;
- validation before `SessionController` state transition or emitted bytes;
- deterministic wire ordering;
- no duplicate typed/generic canonical key emission;
- default `SessionOptions` wire unchanged when LeaseSet features are unused;
- no weaker fallback when a requested LeaseSet option is malformed or inconsistent;
- no Emissary, I2PControl, Proposal 170, TunnelManager, matrix, persistence, or router concepts in production code;
- no upstream write/review/submission/release activity.

## 6. Explicit non-goals

Y004 does not:

- make Emissary consume Y003/Y004;
- implement encrypted LeaseSet construction or authentication in a router;
- implement Proposal `OptionalLookup` without a positively verified generic I2CP/SAM mapping;
- add TLS or local presentation behavior;
- change signature-aware destination generation from Y002;
- change tunnel variance/backup serialization from Y001;
- add a raw SAM command escape hatch;
- broaden to general I2CP parity.

## 7. Work packages

### WP1 — Independent protocol fixture

Create a small table/fixture in tests or closure evidence whose expected canonical keys are written from the reference freeze, not generated from Yosemite constants. This fixture must include the private-key/signing-key distinction and both DH/PSK client-auth prefixes.

### WP2 — Correct typed fields and reserved-key set

Fix serializer keys, field documentation, typed/generic conflict detection, and reserved namespaces. The generic option path must reject canonical LeaseSet typed keys case-insensitively, including all numbered DH/PSK client-auth prefixes owned by the typed API.

### WP3 — Typed client authorization

Introduce the bounded mode/name/key representation, deterministic per-mode numbering, duplicate rules, exact value construction, redacted `Debug`, and defensive revalidation before wire creation.

### WP4 — Correct numeric domains

Replace Y003's guessed `auth_type`, `blinded_type`, and `lease_set_type` bounds with the independently frozen protocol domains.

### WP5 — Regression and compatibility review

Prove default wire compatibility, existing Y001/Y002 behavior, no secret disclosure, no injection, no typed/generic collision, and exact command bytes for representative encrypted/authenticated LeaseSet configurations.

## 8. Failure and contention semantics

This is synchronous option validation and command construction. Invalid configuration must return a generic protocol/configuration error before the controller leaves the handshaked state and before command bytes are returned.

There is no shared mutable runtime owner in scope. No partial client-auth sequence may be emitted if any entry fails validation.

## 9. Compatibility

Existing callers that do not configure LeaseSet security must remain behaviorally unchanged.

Because Y003 has not been consumed by the current Emissary pin, correctness takes precedence over preserving Y003's defective wire spelling. Source compatibility should still be preserved where it does not perpetuate a wrong semantic mapping.

If a Y003 public type cannot be corrected without ambiguity, retain it only as a deprecated compatibility wrapper that converts into the new exact typed representation when that conversion is unambiguous; otherwise document a source-breaking correction explicitly in closure rather than emitting non-canonical protocol bytes.

## 10. Focused tests

Required tests include:

- default session emits none of the new LeaseSet-security keys;
- persistent encryption key emits `i2cp.leaseSetPrivateKey` exactly once and never emits it as `i2cp.leaseSetPrivKey`;
- signing key emits `i2cp.leaseSetSigningPrivateKey` exactly once;
- DH auth entries serialize under `i2cp.leaseSetClient.dh.<n>`;
- PSK auth entries serialize under `i2cp.leaseSetClient.psk.<n>`;
- mixed DH/PSK insertion order produces deterministic per-mode numbering;
- canonical client-auth value grammar is exact;
- duplicate/oversized/malformed client name/key material rejects before bytes;
- typed/generic attempts to occupy LeaseSet private/signing/client prefixes reject;
- exact auth/blinded/LeaseSet-type numeric boundary tests;
- invalid configuration leaves controller state unchanged and emits no weaker subset;
- `Debug`/errors contain no key, secret, PSK, DH material, or additional-option value;
- Y001 variance/backup/signature/custom-option tests and Y002 destination-generation tests remain green.

Tests must exercise `SessionController::create_session()`; isolated validator tests alone are insufficient.

## 11. Broad verification

Run the valid feature/runtime combinations used by prior Yosemite closures:

```text
cargo test --features tokio
cargo test --lib --no-default-features --features smol
cargo test --lib --no-default-features --features sync
cargo test --no-default-features --features sync
cargo check --features tokio
cargo check --no-default-features --features smol
cargo check --no-default-features --features sync
cargo clippy --all-targets --features tokio -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Pre-existing lint/format drift may be dispositioned exactly as in Y001-Y003, but new Y004 diagnostics are not exempt.

## 12. Documentation/static evidence

Update:

- this plan's status/evidence;
- `plans/registry.md`;
- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`;
- a new Y004 closure record.

Do not edit Y003 closure history to pretend the defect was never present. Y004 closure must state that it supersedes Y003's LeaseSet wire-semantic claims.

## 13. Acceptance criteria

Y004 closes only when:

1. every emitted LeaseSet typed key in scope matches independently frozen reference vocabulary;
2. DH/PSK client authorization is mode-aware and serializes canonical numbered keys/values;
3. numeric domains are reference-backed rather than guessed;
4. invalid values fail before controller state changes/wire output with no downgrade;
5. secret and client-auth material remains redacted;
6. default callers and Y001/Y002 capabilities remain compatible;
7. focused/broad verification passes or baseline-only failures are explicitly dispositioned;
8. closure records the exact implementation commit suitable for a future Emissary exact-revision pin.

## 14. Stop conditions

Stop rather than broaden scope if:

- the authoritative reference contract for a required key/value remains ambiguous;
- correct client-auth serialization requires cryptographic derivation rather than transport of already-provided material;
- a router implementation change is proposed;
- a new dependency is proposed solely for parsing/formatting;
- completing `OptionalLookup` requires guessing an unverified option;
- preserving Y003 source compatibility would require continuing to emit non-canonical wire data.

## 15. External-interaction boundary

All external I2P/Yosemite sources are read-only evidence. Writes are authorized only to `eggstack/yosemite` for this plan. No upstream issue, PR, review, discussion, merge request, release, maintainer contact, contribution package, or submission activity is authorized.

## 16. Closure evidence required

Record:

- exact reference files/spec sections and canonical key/value table;
- changed production paths;
- public API compatibility decisions;
- byte-for-byte command examples from tests;
- redaction/injection/conflict tests;
- all verification commands and outcomes;
- unresolved findings with severity;
- exact Y004 implementation SHA;
- whether a future Emissary dependency-pin corrective may proceed.