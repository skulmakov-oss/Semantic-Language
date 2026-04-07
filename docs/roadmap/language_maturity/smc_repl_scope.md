# smc repl Interactive Mode Scope

Status: proposed tooling track

## Goal

Introduce `smc repl` — a read-eval-print loop that accepts Semantic expressions
and statements one at a time, evaluates them through the existing pipeline, and
prints results. A persistent environment is maintained across lines within a
session.

This is a forward-only tooling track. It does not reinterpret the published
`v1.1.1` CLI baseline as if an interactive mode already shipped there.

## Why This Track Exists

`smc check` and `smc run` operate on complete program files. Without a REPL:

- exploring language behaviour requires writing, saving, and running whole files
- incremental experimentation with types and expressions is not possible
- onboarding new users is friction-heavy

This track adds the minimum interactive surface without adding a new language
dialect, hidden evaluation semantics, or persistent session storage.

## Decision Check

- [ ] This is a new explicit tooling track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

- `smc check <file>` and `smc run <file>` work for file input
- parser operates on complete program strings
- no incremental/single-input parse mode exists
- published `v1.1.1` does not claim a REPL mode

## Included In This Track

- incremental parse mode in `sm-front` accepting a single expression or
  statement without requiring a full program wrapper
- REPL loop in `smc-cli`: read line → parse → typecheck → execute → print result
- persistent type environment across lines within a session
- `quit` and `exit` commands to end the session
- expression value printing (debug representation in first wave)

## Explicit Non-Goals

- persistent session save/load to disk
- notebook format or literate programming mode
- history search or readline-style editing in first wave
- syntax highlighting or tab completion in first wave
- multi-line expression continuation (deferred to a later wave)
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance
- scope checkpoint and backlog/milestone linkage

### Wave 1 — Incremental parser mode
- single-expression/statement parse mode in `sm-front`
- `parse_expr` and `parse_stmt` entry points without full program wrapper

### Wave 2 — REPL loop in smc-cli
- `smc repl` subcommand with read/print loop
- line dispatch through typecheck and VM

### Wave 3 — Environment persistence and value printing
- persistent scope env across REPL lines
- value display for primitive types and records

### Wave 4 — Freeze
- docs/spec/tests/golden freeze for the widened contract

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint (this PR)
2. PR 2: incremental parser entry points in `sm-front`
3. PR 3: `smc repl` loop in `smc-cli`
4. PR 4: environment persistence + value printing
5. PR 5: freeze and close-out

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
