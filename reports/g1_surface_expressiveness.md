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
- narrow helper-module executable authoring through direct local-path bare
  imports

Evidence:

- `reports/g1_real_program_trial.md` marked the rule/state program as
  `natural`
- `reports/g1_frontend_trust.md` marked sequence loops, direct-record iterable
  admission, where-clause source sugar, and direct local-path bare helper-module
  imports as `trusted`
- `reports/g1_execution_integrity.md` confirmed end-to-end semantic
  preservation on the representative admitted programs

### Tolerable but narrow zone

The current surface is usable, but still narrow, for:

- small single-file executable cores
- narrow module-based helper-module executable programs
- data-heavy programs over `Sequence(T)` and direct-record iterable impls

Evidence:

- `reports/g1_real_program_trial.md` marked both the CLI-style core and the
  data-heavy iterable program as `tolerable`
- `reports/g1_real_program_trial.md` also marked the admitted helper-module
  executable program as `tolerable`
- `reports/g1_benchmark_baseline.md` showed these programs stay reproducible
  and fast on the admitted current pipeline

Why this is only `tolerable`:

- admitted practical programs still skew toward single-file or bare helper-module
  shapes
- full CLI-style authoring is not yet qualified because argv/stdout/file IO is
  not part of the admitted contour
- iterable reuse remains narrow rather than general-purpose

### Blocked zone

The current surface is still blocked for broader executable-module authoring
beyond the admitted bare local-path helper-module slice.

Evidence:

- `reports/g1_real_program_trial.md` preserved the selected-import helper-module
  probe as an explicit blocked boundary
- `reports/g1_frontend_trust.md` confirms that selected/alias/wildcard/re-export
  executable import forms remain out of scope with deterministic diagnostics

This matters because a language can be technically executable while still
impose a much narrower practical-authoring contour than a broader public-release
claim would imply.

## Friction Inventory

Current trust-reducing friction that does not break the admitted contour, but
still matters for practical readiness:

- broader executable-module entry remains outside the admitted contour beyond
  the direct local-path bare-import slice
- CLI-style programs remain core-only rather than fully user-facing
- integer arithmetic ergonomics still show rough edges, as seen in the `i32`
  `+=` probe noted in `Q1`

## G1-A Verdict

`G1-A Surface Expressiveness` is green only for a **limited** admitted contour.

Operational meaning:

- the language is expressive enough for a narrow release contour centered on
  single-file executable programs, narrow helper-module executable programs,
  rule/state logic, sequence loops, and direct-record iterable dispatch
- the language is **not** yet expressive enough to justify a broader practical
  programming claim while executable-module authoring remains intentionally
  narrow and CLI-style authoring remains incomplete

This is sufficient for a `limited release` decision.

It is not sufficient for a `public release` decision.
