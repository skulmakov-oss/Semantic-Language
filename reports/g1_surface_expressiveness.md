# G1 Surface Expressiveness

Status: completed synthesis report for `Q5`

## Goal

Decide whether the currently admitted Semantic surface is expressive enough for
real programming, based on the evidence collected in:

- `reports/g1_real_program_trial.md`
- `reports/g1_frontend_trust.md`
- `reports/g1_execution_integrity.md`
- `reports/g1_benchmark_baseline.md`

This report follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

UI remains outside this qualification contour.

## Evidence Summary

### Natural zone

The current language surface reads naturally for:

- rule/state-oriented programs over records and nominal data
- explicit `Option` / `Result` handling through contextual constructors and
  exhaustive `match`
- narrow direct-record iterable dispatch with `for x in collection`

Evidence:

- `reports/g1_real_program_trial.md` marked the rule/state program as
  `natural`
- `reports/g1_frontend_trust.md` marked sequence loops, direct-record iterable
  admission, and where-clause source sugar as `trusted`
- `reports/g1_execution_integrity.md` confirmed end-to-end semantic
  preservation on the representative admitted programs

### Tolerable but narrow zone

The current surface is usable, but still narrow, for:

- small single-file executable cores
- data-heavy programs over `Sequence(T)` and direct-record iterable impls

Evidence:

- `reports/g1_real_program_trial.md` marked both the CLI-style core and the
  data-heavy iterable program as `tolerable`
- `reports/g1_benchmark_baseline.md` showed these programs stay reproducible
  and fast on the admitted current pipeline

Why this is only `tolerable`:

- admitted practical programs still skew toward single-file shapes
- full CLI-style authoring is not yet qualified because argv/stdout/file IO is
  not part of the admitted contour
- iterable reuse remains narrow rather than general-purpose

### Blocked zone

The current surface is still blocked for ordinary module-based executable
authoring.

Evidence:

- `reports/g1_real_program_trial.md` marked the module-based helper split as
  `blocked`
- `reports/g1_frontend_trust.md` confirmed the current wave1 executable import
  contract still rejects the selected-import module helper path with
  deterministic but trust-reducing diagnostics

This matters because a language can be technically executable while still
failing a practical-authoring gate. That is exactly the current situation for
multi-file executable programs on `main`.

## Friction Inventory

Current trust-reducing friction that does not break the admitted contour, but
still matters for practical readiness:

- module-based executable entry is blocked at the current executable module-entry
  boundary
- CLI-style programs remain core-only rather than fully user-facing
- integer arithmetic ergonomics still show rough edges, as seen in the `i32`
  `+=` probe noted in `Q1`

## G1-A Verdict

`G1-A Surface Expressiveness` is green only for a **limited** admitted contour.

Operational meaning:

- the language is expressive enough for a narrow release contour centered on
  single-file executable programs, rule/state logic, sequence loops, and
  direct-record iterable dispatch
- the language is **not** yet expressive enough to justify a broader practical
  programming claim while ordinary module-based executable authoring remains
  blocked

This is sufficient for a `limited release` decision.

It is not sufficient for a `public release` decision.
