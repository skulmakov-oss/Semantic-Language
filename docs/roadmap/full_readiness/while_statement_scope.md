# FR-2.3 While Statement Scope

Status: proposed implementation scope  
Parent: [FR-2 Everyday Expressiveness Scope](./everyday_expressiveness_scope.md)

## Goal

Admit the narrow `while` statement needed for ordinary stateful programs without silently opening the broader control-flow family.

This scope is implementation-facing. It defines one exact widening step and the tests/docs required to merge it honestly.

## In scope

- statement form `while condition { ... }`
- boolean condition requirement
- deterministic lowering through source -> sema -> IR -> SemCode -> verify -> VM
- runtime quota interaction for non-terminating loops
- nested use inside existing function/block forms
- diagnostics for unsupported or malformed `while` forms

## Required semantics

- `condition` must typecheck as `bool`
- the condition is re-evaluated before each iteration
- loop body executes in source order
- local reassignment inside the body continues to use the existing assignment path
- non-terminating `while` execution remains bounded by runtime quotas rather than implicit special runtime escape behavior

## Required diagnostics

- non-`bool` loop condition rejects deterministically
- malformed `while` syntax rejects deterministically
- unsupported control exits inside `while` continue to reject according to the currently admitted control surface

## Explicitly out of scope

- statement `loop`
- `continue`
- bare `break;`
- `break expr;` for `while`
- value-producing `while`
- labeled loops
- iterator/collection loop redesign
- quota model redesign

## Required tests

Positive:

- simple counting `while`
- `while` with zero iterations
- nested `if` inside `while`
- `while` with mutable local state updates
- end-to-end `smc check` / `run` / `compile` / `verify` path for at least one canonical fixture

Negative:

- non-`bool` condition
- unsupported `continue`
- unsupported bare `break;`
- unsupported value-carrying `while` expectations
- malformed body / syntax boundary cases

## Files expected to change

- frontend parser/typecheck as required
- lowering/runtime/verifier only if the current loop machinery is insufficient
- spec docs:
  - `docs/spec/syntax.md`
  - `docs/spec/source_semantics.md`
  - `docs/spec/diagnostics.md`
- targeted fixtures/tests

## Merge gate

- `cargo test -q`
- `cargo test -q --test public_api_contracts`
- targeted `while` tests green
- CI green

## Done condition

This scope is complete when `while condition { ... }` is admitted as a narrow statement form with deterministic runtime behavior, while the broader deferred control-flow family remains explicitly out of scope.
