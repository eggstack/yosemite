# Y002 — Signature-Aware Destination Generation

Status: **closed**

Closure record: `plans/closure/002-signature-aware-destination-generation.md`

Class: capability / public API / protocol serialization

Baseline: `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`

Source roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`

## 1. Objective

Add a generic Yosemite Router API path that requests a caller-selected SAM `SIGNATURE_TYPE` for `DEST GENERATE`, while preserving the existing parameterless `generate_destination()` behavior as the compatibility/default Ed25519 (`7`) path.

Both asynchronous and synchronous Router APIs must expose equivalent semantics and share the same protocol-controller implementation.

## 2. Readiness

Y001 is closed at the baseline above, so signature-type representation/validation is stable and duplicated validation policy can be avoided.

## 3. Current evidence

At Yosemite 0.7.0 baseline, `RouterApiController::generate_destination()` emits `DEST GENERATE SIGNATURE_TYPE=7` unconditionally. The async and sync public `RouterApi::generate_destination()` methods invoke that controller path without a signature parameter.

## 4. Required production changes

Expected paths:

- `src/proto/router.rs`;
- `src/asynchronous/router.rs`;
- `src/synchronous/router.rs`;
- focused tests around `RouterApiController` and public API where practical;
- `src/lib.rs` only if Y001 introduced a public signature type that must be re-exported.

No session lifecycle, destination key parsing, cryptographic algorithm implementation, dependency, CI, or release change is authorized.

## 5. Public compatibility

Keep:

```text
generate_destination() -> Result<(String, String)>
```

with its existing effective signature type 7.

Add a clearly named parameterized variant, for example:

```text
generate_destination_with_signature_type(signature_type)
```

The exact Rust type should reuse Y001's accepted signature representation. Do not overload behavior based on hidden global state.

## 6. Invariants

- default API continues to request type 7;
- caller-selected type is serialized exactly once;
- unsupported types are not silently converted to 7;
- the router remains authoritative for whether a protocol-valid type is implemented unless Yosemite has an existing normative validation owner;
- destination/private-key response handling is unchanged;
- no secret response data enters logs/debug;
- async/sync semantics stay equivalent;
- no Emissary/Proposal-specific naming;
- no upstream interaction.

## 7. Work packages

### WP1 — Controller API

Parameterize the protocol command constructor and keep a compatibility/default wrapper if that minimizes churn.

### WP2 — Async public API

Add the selected-signature method while preserving existing connection/handshake/error semantics.

### WP3 — Sync public API

Mirror the async API and controller use without duplicating command formatting.

### WP4 — Regression tests

Test default and non-default command bytes, unsupported/router-error propagation, and compatibility of the existing method.

## 8. Failure/cancellation semantics

The change does not alter transport cancellation. A router rejection propagates through the existing error path; Yosemite MUST NOT retry with signature type 7 after a selected type fails.

## 9. Focused tests

- controller default wrapper emits `SIGNATURE_TYPE=7`;
- selected values emit exact decimal type once;
- existing public method remains callable/compatible;
- async and sync variants route through the same selected value;
- router error response is propagated with no fallback;
- destination/private key are not logged.

## 10. Broad verification

Run tokio, smol and sync test/check combinations plus clippy/fmt/diff checks as defined by planning governance.

## 11. Acceptance criteria

Y002 closes only when selected signature type reaches actual `DEST GENERATE` bytes in both public API families, default behavior remains type 7, no fallback/downgrade exists, and closure records the exact commit for Emissary M117.

## 12. Stop conditions

Stop if implementation would require adding signing algorithms to Yosemite, changing the returned destination format, breaking the existing method, or adding consumer-specific validation/policy.

## 13. Closure evidence

Record API diff, exact controller wire tests, async/sync verification, compatibility review, unresolved findings, and resulting commit SHA.
