# G1 Release Scope Statement

Status: completed synthesis report for `Q5`

## Decision State

`limited release`

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

- ordinary module-based executable authoring is still blocked at the parser
  boundary
- practical CLI-style authoring remains incomplete
- the admitted contour is still narrow enough that broader ecosystem/program
  authoring would be overstated

## Release-Promised Contour

The currently qualified release contour is:

- single-file executable programs on the admitted current source surface
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

- module-based executable entry with top-level `Import`
- broader practical-programming claims beyond the admitted single-file contour
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
