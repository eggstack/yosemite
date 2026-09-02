# Yosemite Planning and Agent-Handoff Process

Status: normative planning governance for the `eggstack/yosemite` internal fork

This document defines how bounded internal Yosemite work is planned, handed off, verified, and closed. The keywords MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Scope and authority

The `eggstack/yosemite` fork exists to carry narrowly scoped, generic Yosemite/SAM capabilities needed by authorized internal consumers, currently `eggstack/emissary` and its I2P Proposal 170 workstream.

Planning authority order is:

1. applicable I2P/SAM specifications and established Yosemite public contracts;
2. accepted architecture decisions in the consuming internal repository when they define a dependency boundary;
3. this planning process;
4. an active subsystem roadmap;
5. a registered implementation plan;
6. current repository evidence.

Consumer requirements do not authorize Proposal-specific APIs in Yosemite. Library changes MUST remain generic SAM/client capabilities that are coherent for Yosemite independently of Emissary.

## 2. Document classes

### 2.1 Planning governance

This document is normative. It may be amended only to resolve a material omission/contradiction or when a maintainer intentionally changes the internal fork workflow.

### 2.2 Subsystem roadmaps

A roadmap MUST define purpose, ownership boundary, current evidence, target architecture, invariants, explicit non-goals, ordered milestones, dependency classes, exit conditions, risks, and milestone status.

### 2.3 Implementation plans

An implementation plan is the coding-agent handoff. It MUST be tied to an exact repository baseline and include:

- source roadmap;
- one bounded objective;
- readiness and dependencies;
- current evidence;
- invariants and non-goals;
- exact or tightly bounded production paths;
- ordered work packages;
- failure/cancellation/restart/contention semantics where applicable;
- public API and compatibility effects;
- focused and broad tests;
- verification commands;
- acceptance and stop conditions;
- required closure evidence.

A plan MUST NOT silently weaken a public/SAM contract merely because a consumer could tolerate weaker behavior.

### 2.4 Closure records

Closure records live under `plans/closure/` and MUST include implementation commits, requirement-to-evidence mapping, verification outcomes, compatibility/security review, unresolved findings, and a disposition of closed, conditionally closed, corrective pass required, or blocked.

Compilation or an implementation-agent assertion is not sufficient closure evidence.

## 3. Work classification

Every milestone receives one primary class:

- **invariant** — a property that must remain true across implementations;
- **capability** — externally visible Yosemite/SAM behavior;
- **infrastructure** — internal machinery consumed by capabilities;
- **polish** — diagnostics, cleanup, ergonomics, or documentation.

Infrastructure is not represented as completed capability until a real public/runtime path consumes it.

## 4. Dependency model

Dependencies are classified as hard, interface, soft, or operational.

A plan is dependency-ready only when all hard dependencies are closed and all interface dependencies have a stable written contract. `plans/registry.md` MUST name blockers precisely.

Only the next dependency-ready plan should normally be registered as `ready`. Later milestones remain roadmap-defined/proposed or blocked.

## 5. Baselines and fork discipline

Every implementation plan MUST freeze the exact `eggstack/yosemite` commit used as its baseline.

The initial internal-fork baseline for the Emissary Proposal 170 adapter line is upstream Yosemite 0.7.0 commit:

`d0fe71da214b212790773be12a93162ae71f3e03`

Changes MUST remain easy to audit against that baseline. Avoid unrelated formatting, dependency churn, refactors, feature changes, CI changes, or release automation.

The fork MUST preserve existing Yosemite behavior by default unless a plan explicitly changes a public contract and supplies compatibility evidence.

## 6. Consumer isolation

The Emissary workstream consumes this fork as an internal dependency, but Yosemite MUST NOT contain:

- `Proposal170`, `I2PControl`, TunnelManager, Emissary backend, or Emissary persistence concepts;
- consumer-specific feature flags solely to select Proposal fields;
- raw command APIs that bypass SAM framing/validation merely for one consumer;
- behavior whose only purpose is making a consumer support matrix green.

Preferred changes extend existing generic Yosemite concepts such as `SessionOptions`, `RouterApi`, and SAM command serialization.

## 7. Security and protocol rules

SAM command construction MUST be injection-safe. Any generic additional-option surface MUST bound count/key/value length, reject control characters/newlines and malformed keys, prevent command-token injection, and define conflict behavior with typed/canonical options.

Secret-bearing options MUST NOT be exposed by ordinary `Debug`, logs, errors, fixtures, or closure artifacts.

No option may be reported as serialized if it is silently dropped. Unsupported behavior must remain explicit.

## 8. Corrective passes

A corrective pass is a new numbered implementation plan. It MUST reference the original plan/closure, enumerate defects, explain why prior evidence missed them, add regressions, and avoid reopening unrelated scope.

Historical closure records are not rewritten to conceal later defects.

## 9. Registry requirements

`plans/registry.md` is the active control surface. It SHOULD contain active roadmaps, the next ready/active/closing plan, named blockers, recently closed milestones, and deferred roadmap-only work.

The registry links requirements; it does not duplicate full plans.

## 10. Verification policy

Plans SHOULD use the existing crate checks rather than introduce orchestration infrastructure. Typical gates are:

```text
cargo check --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Feature combinations that Yosemite already treats as mutually exclusive MUST be tested in the combinations relevant to the changed code rather than forcing an invalid `--all-features` combination. Plans must spell out the actual commands used.

No new hosted CI, fuzzing campaign, release workflow, or benchmark harness is required unless a separate plan explicitly justifies it.

## 11. Internal-only external-interaction boundary

All writes under this workstream are internal to `eggstack/yosemite` and, when a consuming plan explicitly authorizes it, `eggstack/emissary`.

Upstream Yosemite, I2P repositories, specifications, issues, pull requests, and discussions are read-only evidence. Agents MUST NOT open, draft, update, comment on, or request review/merge/adoption in upstream or third-party repositories; contact upstream maintainers; push branches/tags/releases upstream; or prepare an upstream contribution package unless a later explicit maintainer directive names the target and authorized action.

The existence of a public fork does not grant upstream-submission authority.

## 12. Handoff review

Before registration, verify:

1. baseline is exact;
2. objective is bounded;
3. API/protocol ownership is generic;
4. dependencies are satisfied;
5. compatibility and secret handling are explicit;
6. failure/cancellation/contention semantics are defined where relevant;
7. tests reach the actual command/runtime path;
8. stop conditions prevent scope expansion;
9. no upstream action is implied.
