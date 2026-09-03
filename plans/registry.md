# Yosemite Active Planning Registry

This file is the compact control surface for the `eggstack/yosemite` internal fork.

Planning governance:

- `plans/000-planning-process.md`

Active roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`

Initial fork baseline:

- Yosemite 0.7.0 / `d0fe71da214b212790773be12a93162ae71f3e03`.

## Current state

- Y001 is closed at `beafafa33e563760a0484df1b5fcaec4e0f8c5e4` (bounded `SESSION CREATE` options, signature/variance/backup serialization).
- Y002 is closed at `8026f5b424fc178d683e63555335f8b33e0aba04` (signature-aware destination generation).
- Y003 is historical; its LeaseSet wire-semantic claims were superseded by Y004.
- Y004 is closed at `c2db73dba35dd9392947af5c74df29b0b556775f` and is the exact revision currently consumed by Emissary M122.
- Post-Y004 review found a remaining cross-field correctness defect: typed `lease_set_auth_type` and DH/PSK client-auth entries can be serialized in combinations the Java reference does not consume under the selected auth branch.
- Y005 is the corrective owner.

## Dependency graph

```text
Y001 bounded SESSION CREATE option surface           [CLOSED]
  |
  v
Y002 signature-aware DEST GENERATE                   [CLOSED]
  |
  v
Y003 LeaseSet option attempt                         [HISTORICAL]
  |
  v
Y004 canonical LeaseSet wire corrective              [CLOSED / CONSUMED BY EMISSARY]
  |
  v
Y005 auth-mode/type cross-field consistency          [READY]
  |
  v
future Emissary exact-revision adoption              [EXTERNAL / BLOCKED ON Y005 CLOSURE]
```

## Current handoff — Y005

Plan:

- `plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Status: **ready**.

Baseline:

- `022b2ea192c5ad893531e344890728da0eb563a8`.

Objective:

- independently freeze the relationship among LeaseSet type, `i2cp.leaseSetAuthType`, and numbered DH/PSK client-auth entries;
- reject typed combinations whose security-relevant entries would be ignored by the reference branch;
- preserve canonical Y004 property spelling, numeric domains, deterministic numbering, bounded validation, redaction and default wire behavior;
- remain generic Yosemite API-to-SAM behavior only.

Authorized production scope is limited to:

- `src/options.rs`;
- `src/proto/session.rs`;
- `src/lib.rs` only if a public generic API correction genuinely requires a re-export.

Y005 implements no router cryptography, Proposal/I2PControl policy, dependency/release work or upstream activity.

## Historical milestones

| Milestone | Disposition |
|---|---|
| Y001 | closed |
| Y002 | closed |
| Y003 | historical closure; LeaseSet wire semantics superseded |
| Y004 | closed; canonical wire vocabulary, but Y005 corrects later-discovered cross-field auth consistency |
| Y005 | ready |

Historical closure records are not rewritten. Y005 supersedes only the affected Y004 consistency claim.

## Consumer state

`eggstack/emissary` M122 currently exact-pins Y004 `c2db73dba35dd9392947af5c74df29b0b556775f` through its optional I2PControl-only package alias. Y004 is not used by ordinary/non-I2PControl Emissary paths.

Current Emissary has no active Proposal mapping for LeaseSet client authorization, so the Y005 defect is not an active runtime security downgrade there. However, no future M113/LeaseSet implementation should build against Y004 after Y005 is known.

A future Emissary plan may advance the exact pin only after Y005 closes and the consumer independently reviews the exact implementation revision.

## Registry rules

1. Y005 is the sole dependency-ready Yosemite handoff.
2. Do not rewrite Y003/Y004 closure history; Y005 closure records the new corrective disposition.
3. Default Yosemite behavior must remain compatible for callers that do not configure corrected LeaseSet features.
4. No Yosemite router/crypto implementation is authorized by this workstream.
5. No consumer dependency pin is changed from this repository.
6. All external/upstream sources remain read-only; no upstream PR/issue/review/release/contact/submission/adoption activity is authorized.
