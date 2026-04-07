# smc explain Error-Code Scope

Status: proposed tooling track

## Goal

Introduce `smc explain <E0NNN>` — a CLI subcommand that prints a human-readable
explanation of any error code, including an example source snippet, root cause
description, and fix suggestion. `smc explain --list` lists all known codes.

This is a forward-only tooling track for current `main`. It does not reinterpret
the published `v1.1.1` line as if structured error codes already shipped there.

## Why This Track Exists

`FrontendError` currently carries a free-form `message: String` and a byte
`pos: usize`. Errors are human-readable but not machine-addressable. Without
structured error codes:

- users cannot look up what an error means independently of the source location
- tooling (editors, CI scripts) cannot pattern-match on error identity
- the error surface has no stable identity contract across versions

This track adds the minimum structured code surface without redesigning the
diagnostic system.

## Decision Check

- [ ] This is a new explicit tooling track with its own scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

- `docs/ERROR_CODES.md` exists listing known error families
- `FrontendError { pos: usize, message: String }` is the only error type
- `smc check` emits errors as plain text with no structured code prefix
- no `smc explain` subcommand exists
- published `v1.1.1` does not claim structured error codes

## Included In This Track

- `ErrorCode` enum in `sm-front` mapping to existing `FrontendError` categories
- structured `error[E0NNN]:` prefix on all `smc check` output lines
- `smc explain <E0NNN>` subcommand printing cause + example + fix suggestion
- `smc explain --list` subcommand listing all known codes with one-line summaries
- `docs/errors/E0NNN.md` pages for each admitted error code family

## Explicit Non-Goals

- IDE hover integration or LSP server
- warning levels or lint framework
- runtime error explanations (VM errors are separate)
- i18n / localised error messages
- interactive diagnostic explorer
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance
- scope checkpoint and backlog/milestone linkage

### Wave 1 — ErrorCode enum
- `ErrorCode` enum in `sm-front` with one variant per current error family
- `FrontendError` extended with optional `ErrorCode` field
- all existing error sites tagged with appropriate code

### Wave 2 — CLI prefix and explain subcommand
- `smc check` output prefixed with `error[E0NNN]:` when code is present
- `smc explain <code>` subcommand wired in `smc-cli`
- `smc explain --list` subcommand

### Wave 3 — docs/errors/ pages
- `docs/errors/E0NNN.md` for each admitted error family
- each page: description, example source, cause, suggested fix

### Wave 4 — Freeze
- docs/spec/tests/golden freeze for the widened contract

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint (this PR)
2. PR 2: `ErrorCode` enum + tagging in `sm-front`
3. PR 3: `smc explain` CLI subcommand wiring
4. PR 4: `docs/errors/` pages
5. PR 5: freeze and close-out

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] public API or golden snapshots are updated if needed
- [ ] compatibility/release-facing wording is honest
