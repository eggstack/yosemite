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

## Current source evidence

At the baseline:

- `SessionOptions` already models `signature_type`, inbound/outbound length variance, inbound/outbound backup quantity, multiple LeaseSet fields, reduce/close fields, and `ssl`;
- `SessionController::create_session()` serializes publication, LeaseSet encryption type, tunnel length and tunnel quantity, but hardcodes `SIGNATURE_TYPE=7` and does not serialize variance/backup fields;
- `RouterApiController::generate_destination()` hardcodes `SIGNATURE_TYPE=7`;
- there is no accepted bounded generic additional-session-option surface suitable for Emissary `CustomOptions`/future numbered I2CP options;
- `SessionOptions.ssl` describes SAM-router transport SSL and MUST NOT be assumed to implement I2PTunnel/Proposal `UseSSL` semantics.

## Dependency graph

```text
Y001 bounded SESSION CREATE option surface          [READY]
  |
  v
Y002 signature-aware DEST GENERATE API              [PROPOSED / BLOCKED ON Y001]
  |
  +--------------------------+
  |                          |
  v                          v
Emissary M117 adoption       Y003 LeaseSet option surface
[external/internal blocker]  [ROADMAP ONLY / SEMANTICALLY BLOCKED]
```

Y003 additionally depends on the consuming Emissary M113 semantic/client-auth contract being frozen so Yosemite does not invent consumer policy.

## Current handoff — Y001

Plan:

- `plans/implementation/001-bounded-session-create-option-surface.md`

Status: **ready**.

Objective: extend Yosemite's existing `SESSION CREATE` serializer generically so the current typed `signature_type`, tunnel variance and backup-quantity fields are honored, and add one bounded injection-safe additional-option surface with deterministic conflict behavior.

No Emissary/I2PControl concepts, dependency additions, TLS behavior, LeaseSet crypto implementation, upstream interaction, or release work are authorized.

## Roadmap-defined future plans

### Y002

`plans/implementation/002-signature-aware-destination-generation.md`

Proposed/blocked on Y001 closure. Preserve `generate_destination()` as the Ed25519/default compatibility path and add a generic signature-type-aware path for both async and sync Router APIs.

### Y003

`plans/implementation/003-leaseset-session-option-surface.md`

Proposed/blocked. Serialize only reference-proven generic LeaseSet/I2CP session settings and bounded client-auth entries. This milestone implements no LeaseSet cryptography and does not become ready until Emissary M113 freezes the exact required semantic/interface surface.

## Registry rules

1. Y001 is the sole dependency-ready Yosemite handoff.
2. Y002 MUST NOT execute until Y001 closes and its option/validation types are stable.
3. Y003 MUST NOT execute from this registry until the consuming contract is frozen and it is explicitly promoted.
4. Default Yosemite behavior must remain compatible for callers that do not set new options.
5. All work is internal-only; no upstream PR/issue/review/release/contact is authorized.
