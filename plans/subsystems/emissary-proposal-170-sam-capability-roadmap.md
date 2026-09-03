# Emissary Proposal 170 — Yosemite SAM Capability Roadmap

Status: active; Y001/Y002/Y004/Y005 closed, Y003 historical; future Emissary adoption unblocked for separate consumer review

Initial baseline: `d0fe71da214b212790773be12a93162ae71f3e03` (Yosemite 0.7.0)

Consumer: internal `eggstack/emissary` Proposal 170 workstream.

## 1. Purpose

Provide the smallest generic Yosemite/SAM-client capability surface required for the internal Emissary fork to express Proposal 170 settings without vendoring Yosemite into Emissary, constructing SAM commands in I2PControl, or replacing the ordinary Yosemite dependency used by non-I2PControl Emissary paths.

This is not a Yosemite parity project, general SAM expansion, router implementation, release program, or upstream contribution program.

## 2. Ownership boundary

Yosemite owns:

- public generic SAM client option types;
- injection-safe command construction and serialization;
- Router API command construction;
- response parsing and public compatibility of Yosemite APIs.

Emissary owns:

- Proposal 170 JSON-RPC names/types/applicability;
- TunnelManager policy, persistence, lifecycle and support matrix;
- deciding which generic Yosemite settings a Proposal option maps to;
- router-side SAM option consumption and actual tunnel/LeaseSet behavior;
- cryptographic/runtime security policy beyond SAM framing validation.

Yosemite MUST NOT import Emissary, Proposal170, I2PControl or TunnelManager concepts.

## 3. Invariants

All milestones preserve:

- one Yosemite SAM stack;
- existing default behavior when new settings are unused;
- no command injection through typed or generic options;
- deterministic serialization and conflict handling;
- no silent typed-option weakening/drop;
- no secret leakage through ordinary `Debug`, logs or errors;
- async/sync behavior aligned where both surfaces exist;
- no dependency additions unless separately justified;
- no consumer-specific feature flags;
- no upstream writes/review/contact/release activity.

## 4. Current state

Y001 and Y002 provide the generic session-wire and destination-generation primitives used by Emissary.

Y003 attempted LeaseSet session-option transport but emitted non-canonical/ambiguous fields. Y004 corrected that vocabulary and is closed at `c2db73dba35dd9392947af5c74df29b0b556775f`. Emissary M122 now exact-pins Y004 through its I2PControl-only dependency alias.

Post-Y004 review found one remaining protocol-correctness defect: Yosemite validates LeaseSet auth type and DH/PSK client entries individually but does not validate them as one coherent authentication configuration. The serializer can emit both namespaces regardless of `lease_set_auth_type`, while the Java reference consumes DH only for auth type 1, PSK only for auth type 2, and neither in the no-auth branch.

Y005 owns this cross-field corrective. Current Emissary does not map Proposal LeaseSet client-auth settings, so Y005 is a prerequisite for future capability work rather than a currently active runtime downgrade.

## 5. Explicit non-goals

This roadmap does not:

- implement Proposal `UseSSL`;
- implement router tunnel variance/backup behavior;
- implement close/reduce lifecycle policy merely because similarly named fields exist;
- implement encrypted LeaseSet cryptography, blinding, NetDb publication or client authorization in Yosemite;
- add raw SAM-string escape hatches;
- change SAM version negotiation;
- add release/CI/upstreaming workflow.

## 6. Dependency graph

```text
Y001 SESSION CREATE option surface                  [CLOSED]
  |
  v
Y002 signature-aware DEST GENERATE                  [CLOSED]
  |
  v
Y003 LeaseSet option attempt                        [HISTORICAL]
  |
  v
Y004 canonical LeaseSet vocabulary/representation  [CLOSED / CONSUMED]
  |
  v
Y005 LeaseSet auth-mode/type consistency            [CLOSED]
  |
  v
Emissary corrected exact-pin adoption               [EXTERNAL / UNBLOCKED; SEPARATE CONSUMER REVIEW]
```

No further Yosemite implementation plan is currently registered as ready. Consumer adoption remains a separate Emissary review under ADR-0005.

## 7. Closed milestones

### Y001 — bounded `SESSION CREATE` option surface

Plan: `plans/implementation/001-bounded-session-create-option-surface.md`

Status: closed at `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`.

Provides truthful signature/variance/backup serialization plus a bounded generic session-option collection with reserved-key collision protection and redaction.

### Y002 — signature-aware destination generation

Plan: `plans/implementation/002-signature-aware-destination-generation.md`

Status: closed at `8026f5b424fc178d683e63555335f8b33e0aba04`.

Adds a typed signature-aware destination-generation path while preserving the default type-7 API.

### Y003 — historical LeaseSet session-option attempt

Plan: `plans/implementation/003-leaseset-session-option-surface.md`

Historical implementation: `9ac7d9a0ac2a8d526e363f150466b579b017e116`.

Its LeaseSet wire-semantic claims are superseded; do not offer Y003 as a consumer pin.

### Y004 — canonical LeaseSet wire corrective

Plan: `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`

Closure: `plans/closure/004-y003-leaseset-wire-semantics-corrective.md`

Status: closed at `c2db73dba35dd9392947af5c74df29b0b556775f`.

Corrected:

- `leaseSetPrivateKey` / `leaseSetSigningPrivateKey` semantics;
- mode-aware DH/PSK client-auth key/value representation;
- reference-backed numeric domains;
- canonical reserved namespaces and deterministic numbering.

Y004 remains valid for those claims. Y005 supersedes only its later-discovered cross-field auth-consistency assumption.

## 8. Y005 — Y004 auth-mode consistency corrective

Plan:

- `plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Status: **closed at `59140a2277bf296928d2e8ce39a148182eeff044`**.

Closure: `plans/closure/005-y004-leaseset-auth-mode-consistency-corrective.md`

Baseline: `022b2ea192c5ad893531e344890728da0eb563a8`.

Y005 independently froze and enforces the relationship among:

- LeaseSet type/applicability;
- `i2cp.leaseSetAuthType`;
- `i2cp.leaseSetClient.dh.<n>`;
- `i2cp.leaseSetClient.psk.<n>`.

A typed configuration whose client-auth entries would be ignored under the selected reference branch now rejects before `SESSION CREATE` bytes rather than serializing inert security material.

The reference freeze records that nonzero auth may have an empty numbered external-client set, while auth settings and entries are applicable only to `leaseSetType=5`.

Production authority is limited to Yosemite-generic option/serializer paths named by the Y005 plan.

## 9. Compatibility and security

Correct protocol semantics take precedence over preserving Y003/Y004 source shapes that permit ambiguous or inert security configuration.

Default callers that do not configure LeaseSet auth must retain the same semantic wire. Generic additional options remain bounded, token-safe, deterministic and unable to override typed/reserved LeaseSet namespaces.

Secret-bearing fields and client keys remain redacted. Validation errors must not echo material.

## 10. Verification strategy

Use controller-level byte-for-byte command tests and existing tokio/smol/sync feature combinations. Y005 adds cross-field truth-table regressions and direct-field-mutation defensive validation.

No new hosted test infrastructure is required.

## 11. Exit condition

This roadmap is complete for the current dependency slice when:

1. Y005 closes with independently frozen cross-field LeaseSet auth semantics;
2. no high/medium Yosemite protocol/security corrective remains for the surface needed by Emissary;
3. Emissary independently exact-pins the reviewed Y005 implementation before beginning LeaseSet client-auth capability work;
4. no Proposal-specific policy enters Yosemite.

All external/upstream sources remain read-only. No upstream issue, PR, review, release, submission, merge/adoption request or maintainer contact is part of this roadmap.
