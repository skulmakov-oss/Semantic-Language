# FR-2.4 Statement Loop And Control Exit Scope

Status: proposed implementation scope  
Parent: [FR-2 Everyday Expressiveness Scope](./everyday_expressiveness_scope.md)

## Goal

Admit the next narrow control-flow slice after landed `while`: statement `loop`
with bare `break;` and `continue`, without widening into value-carrying loops,
labeled loops, or a broader control redesign.

This scope is implementation-facing. It defines one exact widening step and the
tests/docs required to merge it honestly.

## In scope

- statement form `loop { ... }`
- bare `break;` inside admitted statement-loop bodies
- `continue;` inside admitted loop bodies
- deterministic nested-loop behavior for `while` + statement `loop`
- diagnostics for unsupported loop/control forms that remain deferred

## Required semantics

- statement `loop` does not produce a value
- bare `break;` exits the innermost admitted loop
- `continue;` resumes the next iteration of the innermost admitted loop
- nested loop behavior is deterministic and source-ordered
- lowering reuses the existing label/jump path; no new runtime carrier is
  introduced for this slice
- non-terminating execution remains bounded by runtime quotas

## Required diagnostics

- `break;` outside admitted loop context rejects deterministically
- `continue;` outside admitted loop context rejects deterministically
- `break expr;` remains restricted to loop-expression bodies
- value-carrying statement-loop forms reject deterministically

## Explicitly out of scope

- labeled loops
- value-producing statement `loop`
- `break expr;` for `while` or statement `loop`
- generalized loop-control redesign
- quota model redesign

## Required tests

Positive:

- simple `loop { ... break; }`
- `continue;` skipping intermediate work
- nested admitted loops with deterministic innermost control exit behavior
- end-to-end `smc check` / `run` / `compile` / `verify` path for at least one
  canonical fixture

Negative:

- bare `break;` outside loop
- `continue;` outside loop
- `break expr;` inside statement-loop body
- malformed statement-loop syntax

## Files expected to change

- frontend parser/typecheck as required
- lowering/runtime/verifier only if the current control-flow machinery is
  insufficient
- spec docs:
  - `docs/spec/syntax.md`
  - `docs/spec/source_semantics.md`
  - `docs/spec/diagnostics.md`
- targeted fixtures/tests

## Merge gate

- `cargo test -q`
- `cargo test -q --test public_api_contracts`
- targeted loop/control tests green
- CI green

## Done condition

This scope is complete when statement `loop`, bare `break;`, and `continue;`
are admitted as a narrow deterministic control-flow slice, while value-carrying
and labeled loop families remain explicitly deferred.
