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
- **closed** — closure accepted.

## Initial fork source evidence

At the baseline:

- `SessionOptions` already models `signature_type`, inbound/outbound length variance, inbound/outbound backup quantity, multiple LeaseSet fields, reduce/close fields, and `ssl`;
- `SessionController::create_session()` serializes publication, LeaseSet encryption type, tunnel length and tunnel quantity, but hardcodes `SIGNATURE_TYPE=7` and does not serialize variance/backup fields;
- `RouterApiController::generate_destination()` hardcodes `SIGNATURE_TYPE=7`;
- there is no accepted bounded generic additional-session-option surface suitable for Emissary `CustomOptions`/future numbered I2CP options;
- `SessionOptions.ssl` describes SAM-router transport SSL and MUST NOT be assumed to implement I2PTunnel/Proposal `UseSSL` semantics.

## Dependency graph

```text
Y001 bounded SESSION CREATE option surface          [CLOSED]
  |
  v
Y002 signature-aware DEST GENERATE API              [CLOSED]
  |
  +--------------------------+
  |                          |
  v                          v
Emissary M117 adoption       Y003 LeaseSet option surface
[UNBLOCKED; pin Y002]        [CLOSED AS BLOCKED; M113 contract absent]
```

Y003 additionally depends on the consuming Emissary M113 semantic/client-auth contract being frozen so Yosemite does not invent consumer policy.

## Recently closed — Y001

Plan:

- `plans/implementation/001-bounded-session-create-option-surface.md`

Status: **closed** at commit `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`.

Closure:

- `plans/closure/001-bounded-session-create-option-surface.md`

## Recently closed — Y002

Plan:

- `plans/implementation/002-signature-aware-destination-generation.md`

Status: **closed** at implementation commit `8026f5b424fc178d683e63555335f8b33e0aba04`.

Closure:

- `plans/closure/002-signature-aware-destination-generation.md`

Objective: add a generic signature-type-aware `DEST GENERATE` path for both async and sync Router APIs while preserving the default method.

The Emissary M117 adoption boundary is unblocked and may pin the implementation commit above. No Emissary/I2PControl concepts, dependency additions, TLS behavior, LeaseSet crypto implementation, upstream interaction, or release work were added.

## Roadmap-defined future plans

### Y003

`plans/implementation/003-leaseset-session-option-surface.md`

Closed as blocked in `plans/closure/003-leaseset-session-option-surface.md`. No production
implementation was authorized or landed because Emissary M113 remains proposed/blocked and
still records the required LeaseSet serializer, lookup-policy serializer, and client-auth
key-handoff primitives as unavailable. A later replacement or successor Y003 implementation
may not be promoted until that contract is frozen and the required neutral owner is accepted.

## Registry rules

1. A replacement or successor Y003 implementation MUST NOT execute from this registry until the consuming contract is frozen and it is explicitly promoted.
2. Default Yosemite behavior must remain compatible for callers that do not set new options.
3. All work is internal-only; no upstream PR/issue/review/release/contact is authorized.
