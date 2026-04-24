# G1 Release Scope Statement

Status: completed synthesis report for `Q5`

## Decision State

`qualified limited release`

## Status Reading

This report uses the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Per the authority order defined there, this report is the current
practical-programming qualification verdict authority.

It does not by itself:

- promote behavior into the `published stable` line
- override stable publication rules in `docs/roadmap/stable_release_policy.md`
- reinterpret all landed current-`main` behavior as release-promised

## Decision Basis

This decision is based on the evidence collected in:

- `reports/g1_real_program_trial.md`
- `reports/g1_frontend_trust.md`
- `reports/g1_execution_integrity.md`
- `reports/g1_benchmark_baseline.md`
- `reports/g1_surface_expressiveness.md`

and follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

## Why The Decision Is Not `not ready`

The first Gate 1 cycle did prove a coherent admitted contour:

- real small programs can be written and run on current `main`
- the admitted frontend slices are stable and diagnostically understandable
- the execution path
  `source -> sema -> IR -> SemCode -> verifier -> VM`
  is trusted on representative programs
- a reproducible benchmark baseline now exists

This is enough to support a narrow release promise.

## Why The Decision Is Not `public release`

The evidence does **not** support a broader practical-programming claim because:

- practical CLI-style authoring remains incomplete
- executable-module authoring is now admitted only for the narrow direct
  local-path bare/selected-import helper-module slice
- the admitted contour is still narrow enough that broader ecosystem/program
  authoring would be overstated

## Release-Promised Contour

The currently qualified release contour is:

- single-file executable programs on the admitted current source surface
- narrow helper-module executable programs using direct local-path bare imports
  and direct local-path selected imports over function-only helper modules
- rule/state-oriented programs using records, `quad`, and explicit
  `Option` / `Result` handling
- built-in `Sequence(T)` iteration
- direct-record user-defined `Iterable` impl dispatch
- verified execution through the admitted IR, SemCode, verifier, and VM
  pipeline
- deterministic behavior and reproducible benchmark baselines for the
  representative current program pack

## Explicitly Not Release-Promised

The following are **not** qualified by this Gate 1 cycle and must remain
outside the release promise:

- alias, wildcard, public re-export, package-qualified, and
  namespace-qualified executable import forms
- broader practical-programming claims beyond the admitted single-file plus
  bare/selected helper-module contour
- full CLI application authoring with admitted argv/stdout/file IO
- UI, which remains outside the current qualification contour
- ADT iterable dispatch
- schema-owned iterable dispatch
- indirect or generalized iterable dispatch beyond the admitted direct-record
  slice

## Landed Is Not Automatically Release-Promised

This report does not widen the release promise merely because behavior exists on
`main`.

Any landed surface outside the admitted contour must stay unpromoted until a
later qualification cycle explicitly qualifies it.

## Operational Release Statement

Semantic is currently qualified for a **limited release** as a deterministic,
verified language/runtime system with a narrow admitted practical-programming
contour.

Semantic is **not yet qualified** for a broader public-release claim as a
general practical programming language.
