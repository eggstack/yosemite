# Y003 — LeaseSet Session-Option Surface

Status: **closed**

Closure record: `plans/closure/003-leaseset-session-option-surface.md`

Class: capability / protocol serialization / secret-handling boundary

Baseline: `472b6684a3f7f967e4023661e479f55b850080bd` (Y002 closure head)

Source roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`

Consumer interface dependency:

- Emissary Proposal 170 M113 must first freeze the exact portable semantics and option/key representation required for `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths`.

## 1. Objective

Once the consumer contract is frozen, complete only the Yosemite-side generic SAM/I2CP serialization needed to convey encrypted/authenticated LeaseSet session settings to a router.

Use existing `SessionOptions` LeaseSet fields where they match canonical SAM/I2CP semantics and add a typed bounded client-authorization representation only where repeated/numbered options cannot be safely represented otherwise.

Y003 implements configuration transport only. It does not implement LeaseSet cryptography.

## 2. Hard blockers

Do not register Y003 as ready until:

1. Y001 is closed and its generic option validation/redaction contract is stable;
2. Emissary M113 records exact reference-backed SAM/I2CP keys and client-auth entry semantics;
3. it is clear which existing Yosemite LeaseSet fields correspond to those semantics;
4. secret redaction requirements are accepted.

Do not guess key names or auth cardinality from field names alone.

## 3. Expected production paths

Primarily:

- `src/options.rs`;
- `src/proto/session.rs`;
- focused session-controller tests;
- `src/lib.rs` only for a new generic public client-auth type.

No crypto, router, async/sync transport, dependency, CI, or release code is pre-authorized.

## 4. Invariants

- no LeaseSet secret/key/client-auth material in `Debug`, logs, errors, examples, or plan evidence;
- no malformed/base64-unvalidated value is emitted when the canonical contract requires structural validation;
- bounded client-auth entry count and size;
- duplicate client authorization identities reject;
- deterministic numbered/repeated serialization;
- no weaker/default fallback if an explicitly requested setting cannot be serialized;
- typed fields and generic Y001 options cannot conflict silently;
- default `SessionOptions` wire remains unchanged when encrypted/auth settings are unused;
- no Emissary/Proposal-specific production names;
- no upstream interaction.

## 5. Work packages

### WP1 — Freeze reference mapping

From the accepted consumer interface, map every required field to its canonical SAM/I2CP key, value format, units, and multiplicity.

### WP2 — Reuse typed fields

Wire existing Yosemite fields only where their documented meaning exactly matches the accepted mapping. Correct misleading documentation if necessary without expanding behavior.

### WP3 — Typed client-auth entries

If client authorization requires multiple numbered keys, add a bounded generic client-auth collection rather than asking Emissary to preformat raw option names. Derive numbering internally and deterministically.

### WP4 — Secret validation/redaction

Validate required encodings/lengths to the degree appropriate for a client library before command creation and guarantee redaction.

### WP5 — Wire tests

Exercise the actual `SessionController::create_session()` output for each supported mode and negative case.

## 6. Failure semantics

Invalid or conflicting security configuration fails before command bytes are returned. Yosemite must never omit the invalid field and continue with a weaker/default session command.

## 7. Compatibility

No behavior change for default/non-encrypted sessions. Existing public fields remain source-compatible unless a separately justified correctness fix requires documentation/validation tightening.

## 8. Focused tests

At minimum after readiness:

- exact encryption/auth/blinding/client-entry command bytes;
- deterministic multi-client numbering;
- duplicate/oversized/malformed client entries reject;
- typed/generic conflicts reject;
- explicit security request cannot silently disappear;
- secret values absent from debug/error output.

## 9. Broad verification

Run tokio, smol and sync test/check combinations plus clippy/fmt/diff checks. No live encrypted-LeaseSet router implementation test is required for Yosemite closure; that belongs to the consuming/router repository.

## 10. Acceptance criteria

Y003 closes only when every accepted field reaches actual `SESSION CREATE` bytes, secret handling is fail-closed/redacted, default sessions are compatible, and closure records an exact commit for the consuming Emissary plan.

## 11. Stop conditions

Stop if the work requires implementing LeaseSet encryption/blinding/authentication cryptography in Yosemite, a raw command fragment API, consumer-specific option names, or guessed semantics not frozen by the consumer/reference contract.

## 12. Closure evidence

Record the accepted mapping, changed public types, exact wire tests, secret/redaction tests, compatibility review, full verification outcomes, unresolved findings, and resulting commit SHA.
