# Y001 — Bounded SESSION CREATE Option Surface

Status: **closed**

Closure record: `plans/closure/001-bounded-session-create-option-surface.md`

Class: capability / protocol serialization / security containment

Baseline: `d0fe71da214b212790773be12a93162ae71f3e03`

Source roadmap:

- `plans/subsystems/emissary-proposal-170-sam-capability-roadmap.md`

## 1. Objective

Make Yosemite's existing generic session configuration truthful on the SAM `SESSION CREATE` wire for the bounded option family needed by the internal consumer:

- `SessionOptions.signature_type`;
- `SessionOptions.inbound_len_variance`;
- `SessionOptions.outbound_len_variance`;
- `SessionOptions.inbound_backup_quantity`;
- `SessionOptions.outbound_backup_quantity`.

Also add one generic, bounded, injection-safe additional-session-option surface for valid SAM/I2CP session options that do not yet have a typed Yosemite field.

This plan establishes API-to-wire serialization only. It does not claim that the connected router implements the requested option.

## 2. Current evidence

At the baseline, all five typed fields already exist in `src/options.rs`, but `src/proto/session.rs` emits only base tunnel lengths/quantities and hardcodes `SIGNATURE_TYPE=7`.

`SessionParameters.options` is appended with direct `key=value` formatting, but it is style-owned/internal machinery rather than an accepted public bounded session-option API and does not establish conflict policy against `SessionOptions` fields.

## 3. Required production changes

Expected paths:

- `src/options.rs`;
- `src/proto/session.rs`;
- focused tests in those modules or existing session-controller test locations;
- `src/lib.rs` only if a new public option wrapper type must be re-exported.

No dependency, Cargo feature, async runtime, sync runtime, parser-response, README/example, CI, or release change is required unless closure records an independently necessary documentation/test adjustment.

## 4. Invariants

Y001 MUST preserve:

- default `signature_type = 7` and current default tunnel settings;
- current session styles and destination handling;
- a single trailing newline and valid SAM token framing;
- no newline/control/whitespace injection from generic option keys/values;
- no duplicate/conflicting canonical key emission;
- typed Yosemite fields remain authoritative over generic additional options;
- deterministic additional-option ordering;
- no secret/additional values in ordinary `Debug`, logs, protocol errors, or assertion failure messages;
- no Emissary/I2PControl/Proposal types or names in production code;
- no upstream interaction.

## 5. Generic additional-option contract

Implement one public generic representation owned by `SessionOptions` or a closely related public Yosemite type.

The exact Rust shape may be chosen during implementation, but it MUST enforce these semantics before wire construction:

- finite maximum entry count;
- finite maximum key and value length;
- non-empty keys;
- keys contain only a conservative SAM option grammar such as ASCII alphanumeric plus `.`, `_`, and `-`;
- keys contain no `=`, quotes, backslash, whitespace, or controls;
- values contain no CR/LF/NUL/control or unescaped whitespace/token delimiters;
- duplicate keys reject;
- reserved command fields reject (`STYLE`, `ID`, `DESTINATION`, `SIGNATURE_TYPE`, framing/port fields emitted structurally);
- keys already emitted from typed Yosemite fields reject rather than override or duplicate;
- serialization order is deterministic.

Do not expose an arbitrary preformatted command fragment.

If valid required I2CP values need quoting/escaping beyond the conservative grammar, stop and add a typed encoder rather than weakening injection protection.

## 6. Canonical typed serialization

Serialize the existing fields using the SAM/I2CP canonical keys established by the reference specifications:

- `SIGNATURE_TYPE=<signature_type>`;
- `inbound.lengthVariance=<...>`;
- `outbound.lengthVariance=<...>`;
- `inbound.backupQuantity=<...>`;
- `outbound.backupQuantity=<...>`.

Do not emit hardcoded `SIGNATURE_TYPE=7` when the field differs.

Numeric formatting must be canonical decimal with no alternate representation.

## 7. Secret/debug hardening

`SessionOptions` currently contains username/password and LeaseSet secret/key fields in addition to any new generic option collection. If derived `Debug` would expose these values, replace it with a manual redacted implementation or an equivalent representation that does not print secret-bearing fields/additional values.

The implementation need not redesign secret ownership; it must prevent this plan's new option path from creating a straightforward debug exfiltration channel and should correct the existing directly adjacent derived-debug hazard while in the owner.

## 8. Work packages

### WP1 — Freeze wire keys and reserved set

Record the exact SAM/I2CP names for the five typed fields and enumerate every structural/typed key that generic options may not replace.

### WP2 — Add bounded public additional options

Implement validated insertion/construction. Prefer validation at mutation/construction time and revalidate before serialization defensively if the type remains publicly mutable.

### WP3 — Serialize typed fields

Replace the signature hardcode and emit variance/backup fields from `SessionOptions`.

### WP4 — Merge options deterministically

Build the command from structural fields, typed session options, then validated additional options in deterministic order with explicit collision checks before bytes are returned.

### WP5 — Redaction

Ensure ordinary debug/log/error paths do not reveal secret-bearing option values.

## 9. Failure and contention semantics

This path is synchronous command construction with no shared mutable state or network I/O. Invalid generic options return a protocol/configuration error before a command is produced.

No partial command should escape on validation failure.

## 10. Compatibility

`SessionOptions::default()` must continue to produce signature type 7 and the same semantic tunnel configuration. Existing callers that never use new fields/options require no migration.

Do not rename/remove existing public fields.

If introducing a new error variant is necessary, keep it generic and preserve existing error conversions where possible.

## 11. Focused tests

At minimum:

- default command emits `SIGNATURE_TYPE=7`;
- non-default `signature_type` is emitted exactly once;
- positive/negative inbound/outbound variance formatting is exact;
- both backup quantities are emitted exactly once;
- generic valid `i2cp.*`/session options appear deterministically;
- duplicate typed/generic keys reject;
- structural-key override attempts reject;
- CR/LF/NUL/space/`=` key injection rejects;
- value token injection rejects;
- maximum count/length boundaries are tested;
- persistent destination/private data and added option values are absent from `Debug` output.

Tests must exercise `SessionController::create_session()`, not only an isolated validator.

## 12. Broad verification

Run the feature combinations that cover the changed shared protocol code, for example:

```text
cargo test --features tokio
cargo test --features smol
cargo test --features sync
cargo check --features tokio
cargo check --features smol
cargo check --features sync
cargo clippy --all-targets --features tokio -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Do not combine mutually exclusive runtime features merely to satisfy an `--all-features` convention.

## 13. Acceptance criteria

Y001 closes only when:

1. all five typed fields affect actual `SESSION CREATE` bytes;
2. the additional-option surface is bounded, injection-safe and collision-safe;
3. default callers remain semantically compatible;
4. secret-bearing debug paths are redacted;
5. no consumer-specific API appears;
6. focused and broad verification pass or failures are explicitly dispositioned;
7. closure records the exact commit that Emissary may later pin.

## 14. Stop conditions

Stop rather than broaden scope if:

- a required value needs raw unvalidated command fragments;
- completing the fields requires router-side behavior;
- a new dependency is proposed solely for option formatting/validation;
- an API change would break default/current callers rather than extend them compatibly;
- the work starts implementing TLS, lifecycle timers, LeaseSet cryptography, or consumer policy.

## 15. Closure evidence

Record changed paths, exact serialized examples from tests, validation/redaction regressions, all verification outcomes, unresolved findings, compatibility review, the resulting commit SHA, and Y002 readiness.
