# Y005 — Y004 LeaseSet Auth-Mode Consistency Corrective

Status: **ready**

Class: invariant / protocol-correctness corrective

Repository: `eggstack/yosemite`

Baseline: `022b2ea192c5ad893531e344890728da0eb563a8`

Source roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`

Corrective targets:

- `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`
- `plans/closure/004-y003-leaseset-wire-semantics-corrective.md`

Consumer context:

- `eggstack/emissary` currently exact-pins Y004 implementation `c2db73dba35dd9392947af5c74df29b0b556775f` only through its optional `yosemite-i2pcontrol` alias.
- Emissary M122 proved Y004 transport reachability but intentionally maps no Proposal 170 LeaseSet client-auth option yet.

External/reference sources are read-only. No upstream issue, PR, review, release, submission, contact, merge or adoption activity is authorized.

## 1. Objective

Make Yosemite's typed encrypted-LeaseSet client-authorization surface internally coherent so it cannot serialize a combination of `i2cp.leaseSetAuthType` and `i2cp.leaseSetClient.{dh,psk}.<n>` entries that the Java/I2CP reference path would silently ignore or interpret under a different authentication mode.

This plan corrects **API-to-SAM validation/serialization only**. It does not implement LeaseSet cryptography, router behavior, Proposal 170 policy, Emissary mappings, or a new I2CP stack.

## 2. Why Y004 needs a corrective

Y004 correctly repaired Y003's property names, numeric domains and DH/PSK wire representation, but its tests validated each field mostly independently.

Current Y004 permits and serializes combinations such as:

- `lease_set_auth_type = 0` with DH and/or PSK client entries;
- `lease_set_auth_type = 1` with PSK entries;
- `lease_set_auth_type = 2` with DH entries;
- both DH and PSK namespaces in one `SessionOptions` value.

The serializer walks DH and PSK collections independently from `lease_set_auth_type`, so these combinations reach `SESSION CREATE` even though the reference client-side LeaseSet builder chooses exactly one branch:

- auth type `1` consumes only `i2cp.leaseSetClient.dh.<n>`;
- auth type `2` consumes only `i2cp.leaseSetClient.psk.<n>`;
- other/no-auth values sign without either per-client namespace.

That means a typed Yosemite caller can currently supply apparently security-relevant material that is inert at the reference consumer. This violates the fork invariant that a typed option must not be silently weakened or dropped by the effective protocol semantics.

Y004's own mixed-mode fixture and Emissary M122's fake-SAM fixture did not catch this because they asserted wire reachability, not cross-field semantic consistency.

## 3. Reference freeze required before editing

Before production changes, re-read and record the exact behavior from:

- current I2CP option documentation for `i2cp.leaseSetAuthType`, `i2cp.leaseSetClient.dh.nnn`, `i2cp.leaseSetClient.psk.nnn`, and `i2cp.leaseSetType`;
- Java `RequestLeaseSetMessageHandler` auth selection and numbered-key parsing;
- Java I2PTunnel configuration construction for DH/PSK modes.

The implementation must freeze at least these questions in tests/comments or the closure record:

1. whether auth type `0` may coexist with any per-client auth entry;
2. whether auth type `1` may contain anything except DH entries;
3. whether auth type `2` may contain anything except PSK entries;
4. whether a nonzero auth type requires at least one numbered external client entry, or whether the reference permits an empty numbered set because it has other local-key semantics;
5. whether per-client auth settings are meaningful only for Encrypted LS2 (`leaseSetType=5`) and therefore must reject other LeaseSet types at the Yosemite typed boundary;
6. whether `i2cp.encryptLeaseSet` is a distinct legacy/client-side setting and MUST NOT be used as a substitute for the Encrypted-LS2 type relationship.

Do not guess. If the reference permits a combination, preserve it. If the reference ignores a typed field in that combination, Yosemite must reject it before command bytes rather than serialize an inert security setting.

## 4. Invariants

Y005 MUST preserve:

- all canonical Y004 property names and numeric domains;
- Y001 bounded generic-option grammar and reserved-key collision rules;
- Y002 signature-aware destination generation;
- deterministic serialization;
- strict I2P-base64 validation for client-auth key material;
- secret/key redaction in `Debug`, logs and errors;
- unchanged default wire for callers that do not configure LeaseSet security options;
- no raw SAM fragments supplied by callers;
- no Proposal/Emissary concepts in Yosemite;
- no dependency, feature, CI or release changes.

A rejected cross-field combination must fail before any `SESSION CREATE` bytes are returned. No invalid combination may be normalized by silently dropping entries or changing `lease_set_auth_type`.

## 5. Authorized production scope

Production changes are limited to:

- `src/options.rs`;
- `src/proto/session.rs` only where serializer/tests must reflect the validated invariant;
- `src/lib.rs` only if a public helper/type re-export is genuinely required by an API correction.

No other production path is authorized without stopping and recording why the plan is insufficient.

Specifically forbidden:

- Cargo/dependency changes;
- async/sync transport rewrites;
- router or LeaseSet cryptography;
- SAM version changes;
- raw-command escape hatches;
- Emissary-specific code;
- upstream writes or contribution preparation.

## 6. Work packages

### WP1 — freeze cross-field contract

Produce a small reference table with columns:

`lease_set_type | auth_type | DH entries | PSK entries | reference behavior | Yosemite disposition`

At minimum cover:

- no-auth/no entries;
- no-auth + DH;
- no-auth + PSK;
- DH + DH;
- DH + PSK;
- DH + mixed;
- PSK + PSK;
- PSK + DH;
- PSK + mixed;
- applicable/non-applicable LeaseSet types.

The table belongs in closure evidence or focused test comments, not in a new runtime subsystem.

### WP2 — centralize validation

Make `SessionOptions::validate_lease_set_options()` the single authoritative cross-field validator.

Requirements:

- mode/type consistency is checked after individual values are validated;
- direct public-field mutation receives the same validation as convenience methods;
- convenience methods may reject impossible combinations early, but controller validation remains authoritative;
- errors remain `ProtocolError::InvalidOption` or another existing generic, non-secret-bearing error unless a narrowly justified generic validation distinction is required;
- do not silently filter a mismatched `LeaseSetClientAuth` entry.

### WP3 — align serializer assumptions

`SessionController::create_session()` may assume Y005 validation succeeded, but its control flow must make it impossible for both auth namespaces to be emitted when the frozen contract permits only one.

Prefer deriving the emitted auth collection from the selected mode rather than independently walking every mode and hoping validation kept them coherent.

Do not change canonical numbering rules: numbered entries remain deterministic and contiguous within the selected reference namespace.

### WP4 — repair misleading Y004 tests

Replace the mixed-DH/PSK positive fixture with mode-coherent fixtures.

Add negative regressions for every cross-field combination that the reference would ignore or weaken. Tests must assert:

- `SessionController::new()` rejects where possible;
- defensive validation in `create_session()` also rejects after direct public-field mutation;
- controller state remains `Handshaked` on post-handshake validation failure;
- no command bytes are returned;
- no secret material is present in errors/debug output.

If the reference permits a nonzero auth type with zero numbered entries, add a positive test explicitly documenting why. If it does not, reject and test it.

### WP5 — compatibility/security review

Confirm:

- default `SessionOptions` wire is byte-equivalent to Y004 for LeaseSet-unconfigured callers;
- Y001/Y002 tests remain unchanged/green;
- Y004 canonical names and DH/PSK value construction remain unchanged;
- source compatibility is not preserved by an adapter that reintroduces ambiguous mixed-mode semantics;
- no typed secret-bearing material enters `Debug`, errors or logs.

## 7. Failure, cancellation, restart and contention semantics

Y005 is synchronous option validation/serialization logic. It creates no task, timer, lock, persistent state or network owner.

Therefore:

- validation failure is atomic and side-effect free;
- invalid direct field mutation must fail before the first session-create command is produced;
- there is no rollback or migration state;
- concurrent independent `SessionOptions` values share no mutable global state.

Any implementation requiring a new shared mutable registry, background task or persistent state is outside scope and must stop.

## 8. Focused tests

Required tests include:

1. auth type 0 with disallowed client entries rejects;
2. auth type 1 rejects PSK-only and mixed entries when reference evidence says those entries are ignored;
3. auth type 2 rejects DH-only and mixed entries when reference evidence says those entries are ignored;
4. valid DH configuration emits only `i2cp.leaseSetClient.dh.<n>`;
5. valid PSK configuration emits only `i2cp.leaseSetClient.psk.<n>`;
6. exact rule for empty selected-mode entries is covered positively or negatively from WP1 evidence;
7. exact LeaseSet-type applicability rule is covered if reference freeze proves it is required;
8. direct post-construction mutation cannot bypass validation;
9. default wire stability;
10. secret redaction and generic error behavior;
11. existing Y001/Y002/Y004 canonical-value tests remain green after correction.

## 9. Verification

Run the feature combinations already used by Y004 rather than adding CI infrastructure:

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

Pre-existing lint/format drift may be dispositioned only if the changed Y005 lines introduce no new diagnostic.

## 10. Acceptance criteria

Y005 may close only when:

1. the cross-field contract is independently frozen from read-only reference evidence;
2. no typed client-auth setting can be serialized under a mode/type in which the reference would silently ignore it;
3. valid DH and PSK configurations serialize exact canonical namespaces/values;
4. invalid combinations fail before command bytes with no downgrade or secret echo;
5. default and unrelated Y001/Y002 behavior remains compatible;
6. production diff stays within the authorized Yosemite-generic files;
7. no high/medium in-scope protocol/security finding remains open;
8. closure records the exact implementation commit suitable for an Emissary exact-revision pin review.

## 11. Stop conditions

Stop and return the plan for revision if:

- correct behavior requires implementing encrypted LeaseSet cryptography in Yosemite;
- reference behavior cannot distinguish a safe invariant from consumer-specific policy;
- the fix would require a raw SAM command API;
- the change requires a dependency/feature redesign;
- Emissary/Proposal types would enter Yosemite;
- an upstream write/review/contact is proposed.

## 12. Closure record required

Create `plans/closure/005-y004-leaseset-auth-mode-consistency-corrective.md` containing:

- implementation commit(s);
- reference cross-field truth table;
- requirement-to-evidence mapping;
- exact verification commands/outcomes;
- default/API compatibility review;
- redaction/security review;
- changed-path audit;
- unresolved findings with severity;
- exact consumer-pin suitability decision;
- internal-only external-interaction attestation.
