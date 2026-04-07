# Rich Diagnostics Scope

Status: proposed tooling track

## Goal

All parser, typecheck, and lowering errors include line:col, a rendered source
snippet with caret pointer, and a structured error code — matching the
rustc-style diagnostic contract described in `docs/ROADMAP_V0_3.md` Track A.

This is a forward-only tooling track. It does not reinterpret the published
`v1.1.1` error output contract as if rich diagnostics already shipped there.

## Why This Track Exists

`FrontendError` carries `pos: usize` (byte offset) and `message: String`.
Many errors show no source context; some have line numbers but no caret
rendering. Without rich diagnostics:

- users must manually count characters to find the error location
- error messages are not scannable at a glance
- tooling cannot identify errors by structured code

This track delivers the minimum rustc-style rendering without introducing a
full diagnostic framework, warning levels, or IDE integration.

## Decision Check

- [ ] This is a new explicit tooling track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

- `FrontendError { pos: usize, message: String }` — byte offset only
- `smc check` emits plain text errors with no source snippet
- some errors include approximate line numbers; none include carets
- published `v1.1.1` does not claim rustc-style diagnostic rendering

## Included In This Track

- `pos` → `(line, col)` conversion utility in `sm-front`
- caret rendering in `smc-cli` error output: two context lines + error line +
  caret (`^`) pointing at the error column
- source snippet display for all `FrontendError` emitted by `smc check`
- `error[E0NNN]:` prefix on error output (coordinated with `smc-explain` track)
- `--no-color` flag to suppress ANSI escape codes

## Explicit Non-Goals

- multi-span diagnostics (secondary labels, notes on separate locations)
- IDE hover integration or LSP server
- warning levels or lint framework
- runtime VM error rendering (separate track)
- i18n / localised messages
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance
- scope checkpoint and backlog/milestone linkage

### Wave 1 — pos → line:col in FrontendError
- `pos_to_line_col(source: &str, pos: usize) -> (u32, u32)` utility
- `FrontendError` extended with optional `(line, col)` fields populated at
  error construction sites in the parser and typechecker

### Wave 2 — Caret rendering in smc-cli
- `smc check` error output uses caret renderer
- two lines of source context + error line + caret under the column
- `--no-color` flag

### Wave 3 — Error code prefixes
- `error[E0NNN]:` prefix on all structured errors
- coordinated with `smc-explain` track error code enum

### Wave 4 — Freeze
- docs/spec/tests/golden freeze for the widened diagnostic contract

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint (this PR)
2. PR 2: `pos_to_line_col` utility + FrontendError line/col fields
3. PR 3: caret renderer in `smc-cli`
4. PR 4: error code prefix output
5. PR 5: freeze and close-out

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
