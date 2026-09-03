# Emissary Proposal 170 — Yosemite SAM Capability Roadmap

Status: active; Y001/Y002/Y004 closed, Y003 historical closure superseded for LeaseSet wire semantics

Baseline: `d0fe71da214b212790773be12a93162ae71f3e03` (Yosemite 0.7.0)

Consumer: internal `eggstack/emissary` Proposal 170 workstream.

## 1. Purpose

Provide the smallest generic Yosemite/SAM-client capability surface required for the internal Emissary fork to express Proposal 170 settings without vendoring Yosemite into Emissary, constructing SAM commands in I2PControl, or replacing the ordinary Yosemite dependency used by non-I2PControl Emissary paths.

This is not a Yosemite parity project, general SAM feature expansion, release program, or upstream contribution program.

## 2. Ownership boundary

Yosemite owns:

- public generic SAM client option types;
- injection-safe SAM command serialization;
- Router API command construction;
- response parsing and public compatibility of Yosemite APIs.

Emissary owns:

- Proposal 170 JSON-RPC names/types/applicability;
- TunnelManager policy, persistence, lifecycle, rollback and support matrix;
- deciding which Yosemite generic options a Proposal field maps to;
- router-side SAM option consumption and real tunnel/LeaseSet behavior;
- security policy beyond generic SAM framing validation.

Yosemite MUST NOT import Emissary concepts or contain Proposal-specific switches.

## 3. Current state

Y001 and Y002 established a bounded generic session-option surface, truthful variance/backup/signature serialization, and signature-aware destination generation. Current Emissary pins Y002 implementation `8026f5b424fc178d683e63555335f8b33e0aba04` through its I2PControl-only package alias.

Y003 subsequently attempted to add generic LeaseSet session-option transport. A later independent audit found that the Y003 closure verified its own command strings but did not independently freeze every emitted property against the I2CP/Java reference vocabulary. Material issues include:

- persistent LeaseSet encryption/signing private-key fields emitted with shortened property names that have different or non-canonical semantics;
- per-client authorization emitted under `i2cp.leaseSetClientAuth.<n>` rather than the reference mode-specific DH/PSK namespaces;
- a client-auth value type that cannot represent the reference client-name/key pair without a raw token;
- speculative numeric bounds for authentication, blinded signature type, and LeaseSet type.

Because Emissary remains pinned to Y002, this defective Y003 surface is not currently in the Emissary dependency graph. Y004 was the corrective owner and is now closed.

## 4. Invariants

All milestones preserve:

- one Yosemite SAM stack;
- existing default behavior when new fields/options are unused;
- no command injection through key/value material;
- deterministic option conflict resolution;
- no silent typed-option override by generic options;
- no secret leakage through ordinary `Debug`/logs/errors;
- async and sync public behavior remain equivalent where both surfaces exist;
- no dependency additions unless independently justified;
- no Emissary/Proposal-specific public types;
- no upstream interaction.

## 5. Explicit non-goals

This roadmap does not:

- implement Proposal `UseSSL`; Yosemite's `SessionOptions.ssl` concerns the client-to-SAM-router transport and is not assumed equivalent;
- implement tunnel variance/backup behavior inside an I2P router;
- implement close/reduce lifecycle policy merely because `SessionOptions` contains similarly named fields;
- implement encrypted LeaseSet cryptography, blinding or client authorization in Yosemite;
- add raw string SAM command escape hatches;
- change the SAM version negotiation policy;
- add a release or upstreaming workflow.

## 6. Dependency graph

```text
Y001 SESSION CREATE option surface        [CLOSED]
  |
  v
Y002 signature-aware DEST GENERATE        [CLOSED]
  |
  v
Y003 LeaseSet option surface              [HISTORICAL CLOSED; CORRECTIVE REQUIRED]
  |
  v
Y004 LeaseSet wire semantics corrective   [CLOSED]
  |
  v
Emissary corrected pin / LeaseSet retry   [EXTERNAL; UNBLOCKED BY Y004 CLOSURE / CONSUMER REVIEW REQUIRED]
```

Emissary adoption is an external/internal consumer dependency and occurs only after exact Yosemite implementation commits close the relevant milestones. Y004 closes the Yosemite-side blocker; current Emissary remains on Y002 until a separate consumer plan reviews and selects the exact Y004 revision.

## 7. Y001 — bounded SESSION CREATE option surface

Plan: `plans/implementation/001-bounded-session-create-option-surface.md`

Status: **closed** at implementation commit `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`.

Capabilities:

- serialize `SessionOptions.signature_type` rather than hardcoding 7;
- serialize inbound/outbound length variance;
- serialize inbound/outbound backup quantity;
- expose one generic bounded additional-session-option collection suitable for valid I2CP/session options not represented by typed fields;
- reject reserved/typed conflicts and malformed/injection-capable tokens;
- deterministic serialization and secret redaction.

Y001 guarantees API-to-wire behavior only; router support remains the consumer/router owner's responsibility.

## 8. Y002 — signature-aware destination generation

Plan: `plans/implementation/002-signature-aware-destination-generation.md`

Status: **closed** at implementation commit `8026f5b424fc178d683e63555335f8b33e0aba04`.

Y002 adds an explicitly signature-aware public destination-generation path while preserving the parameterless/default Ed25519 path. Async and sync APIs remain aligned.

This is the current Emissary I2PControl pin. It remains unchanged until a later Emissary plan explicitly advances the revision to an exact reviewed Y004 implementation commit.

## 9. Y003 — historical LeaseSet session-option surface

Plan: `plans/implementation/003-leaseset-session-option-surface.md`

Closure: `plans/closure/003-leaseset-session-option-surface.md`

Historical implementation: `9ac7d9a0ac2a8d526e363f150466b579b017e116`.

Y003 remains a historical record of the first LeaseSet transport attempt. Its default-wire, redaction, boundedness, and fail-before-command intent remain useful evidence, but its canonical LeaseSet wire-semantic claims are not accepted for consumer adoption after the post-closure audit.

Do not rewrite Y003 history and do not pin Emissary to Y003.

## 10. Y004 — Y003 LeaseSet wire semantics corrective

Plan: `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`

Status: **closed** at implementation commit `c2db73dba35dd9392947af5c74df29b0b556775f`.

Closure: `plans/closure/004-y003-leaseset-wire-semantics-corrective.md`.

Y004 independently re-froze the exact Java/I2CP vocabulary and corrected:

- persistent LeaseSet encryption/signing private-key property names and the semantic distinction from `i2cp.leaseSetPrivKey`;
- mode-aware DH/PSK client-auth keys and values;
- auth/blinded-type/LeaseSet-type numeric domains;
- typed/generic reserved namespaces and conflict behavior;
- tests so expected protocol fixtures are independent from the implementation's own constants.

Y004 preserves Y001/Y002 behavior, bounded validation, redaction, deterministic ordering, and default compatibility. It still performs no LeaseSet cryptography and makes no claim that a connected router honors the supplied configuration.

## 11. Compatibility and security

The public default for signature type remains 7. Existing callers that do not configure new LeaseSet security options must observe the same semantic session wire.

Generic options retain strict grammar/count/size limits, deterministic ordering, and reserved-key conflict rejection. They may not override structural fields or a canonical typed option emitted by Yosemite.

Correct protocol spelling takes precedence over preserving Y003's defective wire representation. Source compatibility may be retained through additive/deprecated typed wrappers only where that does not preserve non-canonical semantics.

If a future required value cannot be represented safely by the generic token grammar, the consumer remains blocked until a typed Yosemite API is planned; relaxing framing validation is not an escape hatch.

## 12. Verification strategy

Prefer controller-level byte-for-byte command tests plus existing sync/async session tests. Y004 must add independent protocol fixtures whose expected keys are frozen from external read-only reference evidence, specifically covering DH/PSK client-auth and the private-key property distinctions.

No new hosted test system is required.

## 13. Exit condition

This roadmap is complete when:

1. Y004 closes with canonical, independently verified LeaseSet transport;
2. no high/medium Yosemite protocol/security corrective remains open for the capabilities required by Emissary;
3. Emissary consumes only exact reviewed internal fork revisions through its accepted I2PControl-only dependency boundary;
4. no Proposal-specific policy enters Yosemite.

All external/upstream sources remain read-only. No upstream PR, issue, review, release, submission, or maintainer contact is part of this roadmap; consumer adoption remains a separate reviewed internal-consumer activity.
