# Closure — Y003 LeaseSet Session-Option Surface

Disposition: **closed as blocked**

Plan: `plans/implementation/003-leaseset-session-option-surface.md`

Review baseline: `472b6684a3f7f967e4023661e479f55b850080bd` (Yosemite Y002 closure head)

Read-only consumer evidence: Emissary head `70360a0325181a1e9e2e01b8cbb6ffbe099ec03a`
(`plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`
and its active registry).

## 1. Disposition

Y003 cannot be implemented truthfully at this baseline. Y001 is closed, but its required
interface dependency is not: the current Emissary M113 plan remains `proposed / blocked` and
requires an accepted exact primitive before Yosemite work begins. Its current authoritative
records still classify the server-side `EncryptLeaseSet`, `OptionalLookup`, and
`LeaseSetClientAuths` cells as `blocked_primitive` because no supported Yosemite/SAM LeaseSet
serializer, lookup-policy serializer, or client-authorization key store/session handoff exists.

The Y003 plan's hard blockers and stop conditions therefore apply. No LeaseSet option names,
auth cardinality, key representation, or wire mapping was guessed, and no production code was
changed. This is a planning closure, not a claim of encrypted LeaseSet support.

## 2. Implementation commits

- None. Execution stopped before production implementation because the mandatory M113 semantic
  and primitive-availability gate is unsatisfied.
- The closure documentation commit is the commit containing this record.

## 3. Requirement-to-evidence mapping

| Requirement | Evidence and disposition |
| --- | --- |
| Y001 baseline is available | Y001 closure `plans/closure/001-bounded-session-create-option-surface.md`; Y001 implementation commit `beafafa33e563760a0484df1b5fcaec4e0f8c5e4`. |
| Exact consumer contract is frozen | **Not satisfied.** Emissary M113 remains `proposed / blocked`; its current plan requires M110/M111 ownership evidence and an accepted exact LeaseSet/client-auth primitive. |
| Accepted LeaseSet mappings | **Not available.** Emissary's authoritative M095 matrix records `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` as `blocked_primitive` for all applicable server roles. |
| Typed client-auth representation | **Not added.** Auth cardinality, encoding, key handoff, persistence owner, and numbered/repeated SAM keys are not frozen; adding a type would invent consumer policy. |
| Actual `SESSION CREATE` coverage | **Not applicable.** No accepted Y003 field may be serialized, and no focused Y003 wire tests can truthfully assert unsupported semantics. |
| Secret handling and no downgrade | No new secret-bearing path was introduced. The required fail-closed behavior cannot be implemented without the missing accepted primitive, so no weaker fallback was added. |
| Default compatibility and consumer isolation | No production changes were made; the existing default behavior and generic Y001/Y002 surface are unchanged. |

## 4. Verification outcomes

Run against the unchanged review baseline:

| Command | Outcome |
| --- | --- |
| `cargo test --features tokio` | passed; 31 passed, 1 ignored |
| `cargo test --lib --no-default-features --features smol` | passed; 24 passed |
| `cargo test --lib --no-default-features --features sync` | passed; 24 passed |
| `cargo check --features tokio` | passed |
| `cargo check --no-default-features --features smol` | passed |
| `cargo check --no-default-features --features sync` | passed |
| `cargo clippy --all-targets --features tokio -- -D warnings` | failed on pre-existing parser/router/session/style lint diagnostics; no Y003 code was present |
| `cargo clippy --all-targets --no-default-features --features smol -- -D warnings` | failed on the same pre-existing diagnostics |
| `cargo clippy --all-targets --no-default-features --features sync -- -D warnings` | failed on pre-existing parser/router/session/style lint diagnostics |
| `cargo fmt --all -- --check` | failed because stable rustfmt requests unrelated existing match-arm formatting; no formatter churn retained |
| `git diff --check` | passed before closure-document edits |

No live encrypted-LeaseSet router test was run: the plan assigns that behavior to the consuming
router repository, and the Yosemite-side semantic gate is not satisfied.

## 5. Compatibility and security review

There are no production or public API changes. No key or secret material appears in this
closure, examples, logs, or test fixtures. The missing mapping was left explicit rather than
approximated with an existing field, generic option, or weaker/default session command. In
particular, no private raw SAM/I2CP serializer or Proposal-specific API was introduced.

## 6. Unresolved findings

The following blockers remain intentionally unresolved and are owned outside Yosemite:

- Emissary M113 must freeze the exact portable semantics and option/key representation for
  `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths`.
- An accepted neutral owner must provide the encrypted/authenticated LeaseSet session primitive,
  including bounded client-auth key handoff and fail-closed activation semantics.
- M113 must complete its own secret persistence/ownership and server-session integration gates.

These are blockers, not Y003 defects. A future implementation requires a newly promoted or
replacement plan after those gates close; this historical closure must not be rewritten.

## 7. Future-plan disposition

No future Yosemite plan can be unblocked by this closure. Y003 is now recorded as **closed as
blocked**, while the active roadmap and registry retain the precise M113 dependency. Emissary
M117 remains independently unblocked by the already-closed Y001/Y002 milestones, but that does
not unblock M113 or authorize Y003. No Yosemite successor, router, crypto, or adoption plan is
ready at this time.

No writes were made to the consuming Emissary repository; its current M113 status was read-only
dependency evidence.
