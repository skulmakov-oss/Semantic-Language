# Gate 1 Release Qualification Protocol

Status: active internal qualification checkpoint

## Goal

Define the evidence-based internal gate that must be passed before Semantic is
described as ready for a broader release contour beyond the currently published
stable line.

This is a release qualification program, not a language-maturity feature
roadmap.

## Current Default Qualification Contour

The first Gate 1 cycle qualifies:

- source/frontend behavior
- IR, SemCode, verifier, and VM integrity
- representative real-program coverage
- benchmark and reproducibility baselines
- release-scope honesty

The first Gate 1 cycle does **not** treat UI as blocking by default.

UI enters the qualification contour only if it is explicitly admitted into the
release scope for the cycle being judged.

## Gate 1 Blocks

### G1-A Surface Expressiveness

Question: can real programs be written without the language constantly fighting
the author?

Required output:

- `reports/g1_surface_expressiveness.md`

### G1-B Frontend Trust

Question: can the source entry surface be trusted across positive, negative,
and edge-case inputs?

Required output:

- `reports/g1_frontend_trust.md`

### G1-C Execution Integrity

Question: does
`source -> sema -> IR -> SemCode -> verifier -> VM`
preserve the promised meaning of representative programs?

Required output:

- `reports/g1_execution_integrity.md`

### G1-D Real Program Trial

Question: is the current language surface sufficient for writing actual small
programs rather than feature demonstrations?

Required output:

- `reports/g1_real_program_trial.md`

Required first-cycle program families:

1. CLI utility
2. rule/state-oriented program
3. module-based program
4. data-heavy small program

Optional conditional family:

5. minimal UI program, only if UI is explicitly admitted into the release
   contour for the cycle

### G1-E Benchmark Baseline

Question: is there a reproducible performance and stability baseline rather
than intuition?

Required output:

- `reports/g1_benchmark_baseline.md`

### G1-F Honest Release Scope

Question: what is actually qualified for release, and what is still merely
landed on `main`?

Required output:

- `reports/g1_release_scope_statement.md`

## Decision States

Gate 1 may end in exactly one of these states:

- `not ready`
- `limited release`
- `public release`

### Decision Rule

- `not ready` if any blocking block remains red, if the release scope cannot be
  stated honestly, or if evidence from Q1-Q5 contradicts release confidence
- `limited release` if all blocking blocks are green for a narrow admitted
  contour and the remaining non-admitted surfaces stay explicit
- `public release` if all blocking blocks are green, practical program coverage
  is strong enough for a broader promise, and the release statement remains
  honest

UI may become blocking only if UI is explicitly admitted into the release
contour for the cycle.

## Hard Rules

### Rule 1 - No Release By Intuition

Release decisions must be made from evidence collected in Q1-Q5, not from the
feeling that the platform now looks mature enough.

### Rule 2 - No Scope Inflation During Qualification

The active release contour must not widen during the qualification cycle unless
the widening is itself treated as a new explicit scope decision.

### Rule 3 - Landed Is Not Release-Promised

Behavior present on `main` is not automatically stable, qualified, or promised
in the release scope.

## Execution Order

The qualification cycle runs in this order:

1. `Q0` - freeze this Gate 1 protocol
2. `Q1` - `G1-D Real Program Trial`
3. `Q2` - `G1-B Frontend Trust`
4. `Q3` - `G1-C Execution Integrity`
5. `Q4` - `G1-E Benchmark Baseline`
6. `Q5` - `G1-A` and `G1-F` synthesis

This order is intentional:

- real programs expose the truth fastest
- frontend trust and execution integrity should be measured against those real
  scenarios
- benchmarks come after representative scenarios exist
- final release judgment comes only after evidence is collected

## Done Boundary For Q0

`Q0` is complete when:

1. this document is the canonical internal Gate 1 protocol,
2. the UI conditional rule is explicit,
3. the `not ready` / `limited release` / `public release` states are defined
   without ambiguity,
4. backlog and readiness docs point to this protocol as the release
   qualification authority.
