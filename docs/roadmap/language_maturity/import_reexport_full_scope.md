# Import Re-export Full Scope

Status: completed NEXT-1 post-stable closure track

## Goal

Close the narrow policy/documentation/test gap that remained after the working
`v0.2` import/re-export surface had already landed, without reopening
language/runtime scope.

This was a post-stable closure pass on current `main`. It did not reinterpret
the published `v1.1.1` line as if every post-stable import-path widening had
already shipped there.

## Stable Baseline Before This Track

The published stable line already froze these facts:

- the repository had a working source-level import/export baseline
- module linkage was deterministic and checked before execution
- import/export support was not a release blocker for `v1.1.1`
- broader executable-module authoring and broader package ecosystem work were
  still separate concerns

That stable reading remained the source of truth until this closure track
finished its docs/tests hardening on current `main`.

## Included In This Track

- lookup/export order contract clarification
- select-import missing-symbol behavior
- kind-mismatch validation across import/re-export paths
- alias collision and re-export collision matrix completion
- symbol-level cycle reporting and deterministic chain traces
- wildcard overlap policy clarification
- fixture/snapshot completion for those cases
- sync of `docs/imports.md`, `docs/exports.md`, and `docs/errors/E0242..E0245.md`

## Explicit Non-Goals

- new module syntax families
- new namespace systems
- package registry/import-resolution redesign
- host/runtime/`prom-*` widening
- package publishing semantics
- broad dependency-management work
- silent widening of published `v1.1.1`

## Historical Slice Reading

1. docs/governance checkpoint
2. lookup/export order contract clarification
3. collision and missing-symbol matrix completion
4. symbol-cycle and wildcard-policy completion
5. error-page/docs freeze

## Close-Out Reading

`NEXT-1` is now completed on current `main` as the import/re-export closure
track.

Completed closure reading:

- the canonical module/import contract now lives in
  `docs/spec/modules.md`
- deterministic module/linkage diagnostics are aligned in
  `docs/spec/diagnostics.md`
- `docs/imports.md` and `docs/exports.md` now read as compact companion guides
  rather than stale placeholders
- `docs/errors/E0242.md` through `docs/errors/E0245.md` now describe the
  admitted collision/cycle/select-import families in practice
- fixture coverage exists in `tests/fixtures/imports/`
- repository-level checks exist in:
  - `tests/import_export_docs_fixtures.rs`
  - `tests/imports_matrix.rs`

Still intentionally not included after close-out:

- registry/package-manager redesign
- new namespace families beyond the admitted source contract
- broader executable-module import families beyond the separately tracked
  executable-module-entry contour
- any release claim that landed-on-`main` behavior is automatically part of the
  published stable line

## Acceptance Reading

This closure track is done because:

- import/export behavior is documented as deterministic across the edge-case
  matrix
- fixtures and repository tests explicitly cover the admitted matrix
- `docs/imports.md` and `docs/exports.md` match current repository behavior
- `E0242..E0245` pages are no longer placeholders in practice

## Non-Commitments After Close-Out

Even after this closure pass, the repository still does not claim:

- broader package registry/import semantics as part of this track
- new import syntax families
- automatic promotion of post-stable `main` behavior into stable
- broader executable-module authoring than the separately admitted narrow
  helper-module contour
