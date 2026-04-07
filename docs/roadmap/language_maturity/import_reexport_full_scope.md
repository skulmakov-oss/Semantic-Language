# Import Re-export Full Scope

Status: active post-stable closure track

## Goal

Define the narrow closure boundary for bringing the current `v0.2`
import/re-export surface from "working baseline" to "FULL" without reopening
language/runtime scope.

## Why This Is A Post-Stable Track

The repository already shipped stable `v1.1.1` with a working import/export
surface.

The remaining work here is not a release blocker for the shipped stable line.
It is a correctness/documentation closure track intended to:

- remove edge-case ambiguity
- lock deterministic lookup behavior
- complete diagnostics coverage
- strengthen fixtures and snapshots

## In Scope

The `NEXT-1` closure track may include only:

- lookup/export order contract clarification
- select-import missing symbol behavior
- kind-mismatch validation across import/re-export paths
- alias collision and re-export collision matrix completion
- symbol-level cycle reporting and deterministic chain traces
- wildcard ambiguity policy finalization
- fixture/snapshot completion for those cases
- `docs/imports.md`, `docs/exports.md`, and `docs/errors/E0242..E0245.md` sync

## Out Of Scope

This closure track must not silently expand into:

- new module syntax families
- new namespace systems
- package registry/import resolution redesign
- host/runtime/`prom-*` widening
- package publishing semantics
- broad dependency-management work

## Intended Slice Order

1. docs/governance checkpoint
2. lookup/export order contract clarification
3. collision and missing-symbol matrix completion
4. symbol-cycle and wildcard-policy completion
5. error-page/docs freeze

## Acceptance Reading

`NEXT-1` is done only when:

- import/export behavior is deterministic across the documented edge-case matrix
- fixtures and snapshots cover the matrix explicitly
- `docs/imports.md` and `docs/exports.md` match actual repository behavior
- `E0242..E0245` pages stop being placeholders in practice

## Decision Check

- [ ] This is a closure pass with its own explicit scope decision
- [ ] This does not silently widen published `v1.1.1`
- [ ] This is one stream, not a mixture of multiple tracks
- [ ] This can be closed with a clear done-boundary

## Merge Gate

Before closing this track:

- [ ] code/tests are green
- [ ] spec/docs are synced
- [ ] fixtures and snapshots cover the edge-case matrix
- [ ] compatibility/release-facing wording is honest

## Non-Goal Reminder

This track is a closure pass, not a new feature wave.
