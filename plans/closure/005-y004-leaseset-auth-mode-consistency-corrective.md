# Closure — Y005 Y004 LeaseSet Auth-Mode Consistency Corrective

Status: **closed**

Plan: `plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Corrective targets:

- `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`
- `plans/closure/004-y003-leaseset-wire-semantics-corrective.md`

Baseline: `022b2ea192c5ad893531e344890728da0eb563a8`

Implementation commit: `59140a2277bf296928d2e8ce39a148182eeff044`

## Disposition

Y005 is complete. Yosemite now validates LeaseSet client-auth mode, numbered entries, and
LeaseSet type as one cross-field contract. It rejects any typed client-auth material that the
Java reference would ignore, and emits only the namespace selected by the validated auth mode.
No LeaseSet cryptography, router behavior, Proposal 170 policy, Emissary source, dependency,
feature, CI, release, or upstream activity was added.

## Independent reference freeze

The read-only freeze used the current [I2CP option documentation](https://i2p.net/en/docs/specs/i2cp-overview/),
[Proposal 123](https://i2p.net/en/proposals/123-new-netdb-entries/), the Java
[RequestLeaseSetMessageHandler](https://github.com/i2p/i2p.i2p/blob/master/core/java/src/net/i2p/client/impl/RequestLeaseSetMessageHandler.java),
and Java [TunnelConfig](https://github.com/i2p/i2p.i2p/blob/master/apps/i2ptunnel/java/src/net/i2p/i2ptunnel/ui/TunnelConfig.java).

The option documentation defines auth type `0` as no per-client auth, `1` as DH, and `2` as
PSK; it defines DH and PSK numbered values and restricts LeaseSet type representation to
`1..255`, with Encrypted LS2 represented by type `5`. The handler selects the auth branch only
inside its Encrypted LS2 path: type `1` walks `i2cp.leaseSetClient.dh.<n>`, type `2` walks
`i2cp.leaseSetClient.psk.<n>`, and all other auth values sign without per-client entries.
The handler's numbered loops naturally accept zero entries, and its separate
`i2cp.leaseSetPrivKey` is optional router-local key material; therefore a nonzero auth type with
an empty numbered external-client set is reference-permitted. `i2cp.encryptLeaseSet` is read as
a separate legacy/client-side encryption flag and does not select Encrypted LS2.

| lease_set_type | auth_type | DH entries | PSK entries | Reference behavior | Yosemite disposition |
|---:|---:|---:|---:|---|---|
| 5 | 0 | none | none | Encrypted LS2 with no per-client auth | Accept |
| 5 | 0 | present | none | DH namespace is not read by the no-auth branch | Reject |
| 5 | 0 | none | present | PSK namespace is not read by the no-auth branch | Reject |
| 5 | 1 | none | none | DH auth branch with an empty numbered set is allowed | Accept |
| 5 | 1 | DH | none | DH namespace is consumed | Accept |
| 5 | 1 | none | PSK | PSK namespace is ignored | Reject |
| 5 | 1 | DH | PSK | PSK namespace is ignored | Reject |
| 5 | 2 | none | none | PSK auth branch with an empty numbered set is allowed | Accept |
| 5 | 2 | none | PSK | PSK namespace is consumed | Accept |
| 5 | 2 | DH | none | DH namespace is ignored | Reject |
| 5 | 2 | DH | PSK | DH namespace is ignored | Reject |
| 1, 3, or 7 | 0 | none | none | Ordinary LS1, LS2, or Meta LS2 without client auth | Accept |
| 1, 3, or 7 | 1 or 2 | none | none | Auth branch is not entered because type is not Encrypted LS2 | Reject |
| 1, 3, or 7 | any | present | present or absent | Per-client namespaces are not consumed | Reject |

The legacy `i2cp.encryptLeaseSet` flag remains independent: Y005 does not require it for type 5
auth configurations and does not treat it as a substitute for `lease_set_type == 5`.

## Requirement-to-evidence mapping

| Requirement | Evidence |
|---|---|
| WP1 reference contract | The table above records the I2CP/Java freeze, including auth `0`, DH/PSK branch selection, empty selected-mode entries, type-5 applicability, and the distinct legacy encrypt flag. |
| WP2 single authoritative validator | `SessionOptions::validate_lease_set_options()` in `src/options.rs:915-1008` performs individual validation first, then rejects non-type-5 auth settings, empty-mode mismatches, and wrong-mode entries. `SessionController::new()` and `create_session()` both use `validate_all_options()`. |
| WP3 serializer alignment | `src/proto/session.rs:306-329` derives one selected namespace from auth type and emits only that collection with deterministic contiguous numbering. |
| WP4 regressions | `lease_set_auth_mode_and_type_consistency_is_fail_closed` at `src/proto/session.rs:1544-1617` covers no-auth entries, DH/PSK-only and mixed mismatches, non-applicable types, direct post-construction mutation, unchanged Handshaked state, no command bytes, generic errors, and redaction. Coherent deterministic DH/PSK fixtures are in `session_create_serializes_leaseset_client_auths_deterministically`. |
| WP5 compatibility/security | Existing canonical Y004 tests were repaired to use type 5 where auth is selected; the default wire fixture, key/value construction, generic collision tests, redaction tests, and Y001/Y002 tests remain green. |

## API compatibility and security review

- Default `SessionOptions` callers retain the prior wire: auth type `0` and an empty auth
  collection emit no LeaseSet auth settings.
- Valid DH and PSK configurations retain the canonical Y004 namespaces, values, ordering, and
  strict key validation. The only changed behavior is rejection of configurations whose typed
  auth material would be ignored by the reference.
- Direct public-field mutation is revalidated before `SESSION CREATE`; invalid options leave the
  controller in `Handshaked` and return no command bytes. No invalid combination is normalized by
  dropping entries or changing auth type.
- `LeaseSetClientAuth`, `SessionOptions`, and invalid-option errors remain redacted/generic. The
  regression helper checks that client key material is absent from both debug output and errors.
- Only `src/options.rs` and `src/proto/session.rs` changed in the implementation commit. No
  public re-export, dependency, feature, transport, router, cryptography, CI, or release path
  changed.

## Verification outcomes

| Command | Outcome |
|---|---|
| `cargo test --features tokio` | Passed: 40 passed, 1 ignored across 2 suites. |
| `cargo test --lib --no-default-features --features smol` | Passed: 33 passed. |
| `cargo test --lib --no-default-features --features sync` | Passed: 33 passed. |
| `cargo test --no-default-features --features sync` | Passed: 39 passed, 1 ignored across 2 suites. |
| `cargo check --features tokio` | Passed. |
| `cargo check --no-default-features --features smol` | Passed. |
| `cargo check --no-default-features --features sync` | Passed. |
| `cargo clippy --all-targets --features tokio -- -D warnings` | Baseline-only failure: 52 existing errors and 1 warning in parser/router/session and async style code; no diagnostic points to changed Y005 implementation lines. |
| `cargo fmt --all -- --check` | Baseline-only failure: existing state-machine formatting diffs in `src/proto/router.rs` and `src/proto/session.rs` at pre-existing lines 614 and 653; no Y005 formatting diff. |
| `git diff --check` | Passed before the implementation commit; rerun after closure edits is required before the documentation commit. |

The Clippy and format results are explicitly dispositioned as repository baseline drift. No new
Y005 diagnostic or whitespace error remains.

## Changed-path audit

Implementation commit `59140a2277bf296928d2e8ce39a148182eeff044` changes exactly:

- `src/options.rs`
- `src/proto/session.rs`

The closure, plan status, registry, and active roadmap are documentation-only follow-up changes.
No path outside the authorized production scope was modified.

## Findings and future disposition

No high- or medium-severity in-scope finding remains open. The Y004 cross-field auth-mode
consistency defect is closed by the implementation above. The distinct router-local
`i2cp.leaseSetPrivKey` remains intentionally outside Yosemite's typed API, as established by
Y004, and is not reopened by Y005.

Y005 unblocks a future Emissary exact-revision dependency-pin corrective. Emissary M122 remains
unchanged and currently pins Y004; a separate consumer plan and independent review must select
`59140a2277bf296928d2e8ce39a148182eeff044` before implementing LeaseSet client-auth mappings.
No future Yosemite implementation plan is currently ready behind Y005.

## External-interaction attestation

All I2P/Java sources used for the reference freeze were read-only. No upstream issue, PR, review,
release, submission, contact, merge, adoption activity, or Emissary source/dependency change was
performed from this repository.
