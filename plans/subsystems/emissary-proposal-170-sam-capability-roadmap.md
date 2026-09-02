# Emissary Proposal 170 — Yosemite SAM Capability Roadmap

Status: active; Y001 ready; Y002/Y003 roadmap-defined and blocked by dependency/semantic gates

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

`SessionOptions` already contains several fields needed by the consumer, including `signature_type`, `inbound_len_variance`, `outbound_len_variance`, `inbound_backup_quantity`, and `outbound_backup_quantity`.

However, `SessionController::create_session()` currently emits only publication, LeaseSet encryption type, base tunnel lengths/quantities, then hardcodes `SIGNATURE_TYPE=7`. Therefore changing the existing typed fields does not currently alter the `SESSION CREATE` wire for variance/backups/signature type.

`RouterApiController::generate_destination()` separately hardcodes `SIGNATURE_TYPE=7`, so a non-default destination signature type cannot be requested through the public Router API.

The controller already serializes a style-owned `SessionParameters.options` collection, but that is not a stable public generic consumer surface for arbitrary validated session options and currently uses direct formatting. The Proposal consumer needs a bounded public mechanism without bypassing typed policy.

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
Y001 SESSION CREATE option surface        [READY]
  |
  v
Y002 signature-aware DEST GENERATE        [BLOCKED ON Y001]

Y003 LeaseSet option surface              [BLOCKED ON Y001 + EMISSARY M113 INTERFACE]
```

Emissary adoption is an external/internal consumer dependency and occurs only after exact Yosemite commits close the relevant milestones.

## 7. Y001 — bounded SESSION CREATE option surface

Plan: `plans/implementation/001-bounded-session-create-option-surface.md`

Target capabilities:

- serialize `SessionOptions.signature_type` rather than hardcoding 7;
- serialize inbound/outbound length variance;
- serialize inbound/outbound backup quantity;
- expose one generic bounded additional-session-option collection suitable for valid I2CP/session options not represented by typed fields;
- reject reserved/typed conflicts and malformed/injection-capable tokens;
- make serialization deterministic enough for direct protocol regression tests;
- redact secret-bearing/additional option values from ordinary debug output.

Y001 does not claim that any router honors these values; it guarantees only Yosemite API-to-wire behavior.

## 8. Y002 — signature-aware destination generation

Plan: `plans/implementation/002-signature-aware-destination-generation.md`

Add an explicitly typed/parameterized public destination-generation path that serializes the requested SAM `SIGNATURE_TYPE`, while preserving the current parameterless API as the compatibility/default Ed25519 path. Async and sync APIs must match.

Y002 does not add signing algorithms to routers and does not decide which signature types Emissary supports.

## 9. Y003 — LeaseSet session-option surface

Plan: `plans/implementation/003-leaseset-session-option-surface.md`

Once the Emissary M113 interface is frozen, serialize only the exact generic SAM/I2CP LeaseSet settings required by that contract, using existing typed `SessionOptions` fields where correct and a bounded typed client-auth representation where repeated/numbered options are necessary.

Y003 is transport/configuration plumbing only. Router-side encrypted/authenticated LeaseSet semantics remain outside Yosemite.

## 10. Compatibility and security

The public default for signature type remains 7. Existing callers that construct `SessionOptions::default()` must observe the same session wire except for ordering changes that are semantically neutral and covered by tests.

Generic options must have strict grammar/count/size limits, deterministic ordering, and reserved-key conflict rejection. They may not override `STYLE`, `ID`, `DESTINATION`, `SIGNATURE_TYPE`, datagram framing fields, or any typed option emitted by Yosemite.

If a future required value cannot be represented safely by the generic token grammar, the consumer must remain blocked until a typed Yosemite API is planned; relaxing framing validation is not an escape hatch.

## 11. Verification strategy

Prefer controller-level byte-for-byte command tests plus existing sync/async session tests. Each wire field must be exercised through the actual controller path, including negative injection/conflict cases.

No new hosted test system is required.

## 12. Exit condition

This roadmap is complete when every Yosemite capability required by the accepted Emissary Proposal 170 dependency boundary has a closed milestone and Emissary is pinned to exact internal fork revisions without any Proposal-specific code in Yosemite.
