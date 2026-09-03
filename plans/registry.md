# Yosemite Active Planning Registry

This file is the compact control surface for the `eggstack/yosemite` internal fork.

Planning governance:

- `plans/000-planning-process.md`

Active roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`

Initial fork baseline:

- Yosemite 0.7.0 / `d0fe71da214b212790773be12a93162ae71f3e03`.

## Status vocabulary

- **proposed** — documented but not approved for execution;
- **ready** — dependency-ready and may be handed off;
- **active** — implementation or closure is in progress;
- **blocked** — named dependency/evidence gate prevents execution;
- **closing** — implementation landed and closure evidence is being gathered;
- **closed** — closure accepted;
- **corrective pass required** — a later audit invalidated a material claim from an earlier closure.

## Current state

Y001 and Y002 remain closed and valid:

- Y001 implementation `beafafa33e563760a0484df1b5fcaec4e0f8c5e4` provides bounded generic `SESSION CREATE` options plus truthful signature/variance/backup serialization;
- Y002 implementation `8026f5b424fc178d683e63555335f8b33e0aba04` provides signature-aware destination generation and remains the exact revision consumed by current Emissary.

Y003 implementation `9ac7d9a0ac2a8d526e363f150466b579b017e116` is historical but its LeaseSet wire-semantic claims require correction. A post-closure audit found non-canonical private/signing-key spellings, a non-canonical client-auth namespace/representation, and guessed numeric domains. Current Emissary is not exposed because it remains pinned to Y002.

## Dependency graph

```text
Y001 bounded SESSION CREATE option surface          [CLOSED]
  |
  v
Y002 signature-aware DEST GENERATE API              [CLOSED]
  |
  v
Y003 LeaseSet option surface                         [HISTORICAL CLOSED; CORRECTIVE REQUIRED]
  |
  v
Y004 canonical LeaseSet wire corrective              [READY]
  |
  v
future Emissary exact-revision adoption              [EXTERNAL / BLOCKED ON Y004 CLOSURE]
```

## Current handoff — Y004

Plan:

- `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`

Status: **ready**.

Baseline:

- `94d7455c9f78ebb74b7a68823e921db0d76c85c1`.

Objective:

- independently re-freeze the canonical I2CP LeaseSet property vocabulary;
- correct `leaseSetPrivateKey` / `leaseSetSigningPrivateKey` semantics;
- replace the generic `leaseSetClientAuth.<n>` representation with exact mode-aware DH/PSK client authorization;
- replace Y003's guessed auth/blinded/LeaseSet-type domains with reference-backed domains;
- retain bounded validation, deterministic serialization, fail-before-wire behavior, redaction, and default compatibility.

Authorized production paths are Yosemite-generic owners only:

- `src/options.rs`;
- `src/proto/session.rs`;
- `src/lib.rs` if public type re-exports are required;
- `src/error.rs` only if a generic validation error distinction is necessary.

Y004 implements no router cryptography, Emissary/I2PControl policy, Proposal matrix state, TLS behavior, dependency/release work, or upstream activity.

## Recently closed / historical milestones

### Y001

Plan: `plans/implementation/001-bounded-session-create-option-surface.md`

Closure: `plans/closure/001-bounded-session-create-option-surface.md`

Status: **closed** at implementation `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`.

### Y002

Plan: `plans/implementation/002-signature-aware-destination-generation.md`

Closure: `plans/closure/002-signature-aware-destination-generation.md`

Status: **closed** at implementation `8026f5b424fc178d683e63555335f8b33e0aba04`.

### Y003

Plan: `plans/implementation/003-leaseset-session-option-surface.md`

Closure: `plans/closure/003-leaseset-session-option-surface.md`

Historical implementation: `9ac7d9a0ac2a8d526e363f150466b579b017e116`.

Disposition: historical closure preserved, but its LeaseSet wire-semantic correctness is superseded by Y004 corrective authority. Do not offer Y003 as an Emissary pin candidate.

## Registry rules

1. Y004 is the sole dependency-ready Yosemite handoff.
2. Do not rewrite Y003 closure history; Y004 closure records the corrective disposition.
3. Current Emissary must remain pinned to Y002 until a reviewed Y004 implementation commit closes.
4. Default Yosemite behavior must remain compatible for callers that do not configure the corrected LeaseSet features.
5. No Yosemite router/crypto implementation is authorized by this workstream.
6. All work is internal-only; no upstream PR/issue/review/release/contact/submission/adoption activity is authorized.