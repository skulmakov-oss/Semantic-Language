# Executable Module Entry Scope

Status: active post-qualification blocker-removal checkpoint

## Goal

Open a narrow follow-up track after the first `Gate 1` cycle to remove the
largest remaining practical-programming blocker on current `main`:

- ordinary module-based executable authoring is still blocked because top-level
  `Import` is not admitted on the executable source path

This track is intentionally narrow. It is not a reboot of the whole package or
module ecosystem story.

## Why This Track Exists

The first `Gate 1` qualification cycle ended in:

- `limited release`

The main blocker preventing a broader practical-programming claim was not VM
integrity or verifier trust. It was ordinary module-based executable authoring.

Evidence is frozen in:

- `reports/g1_real_program_trial.md`
- `reports/g1_frontend_trust.md`
- `reports/g1_release_scope_statement.md`

Those reports showed:

- single-file executable programs are admitted and runnable
- the frontend and execution path are trusted on the admitted contour
- module-based executable entry with top-level `Import` is still blocked at the
  parser/source-contract boundary

## Decision Check

- [x] This is a new explicit post-qualification track with its own scope decision
- [x] This addresses a proven release blocker rather than speculative feature work
- [x] This remains one stream, not a mixture of package, registry, and stdlib expansion
- [x] This can close with a clear done-boundary

## Narrow First-Wave Decision

The first wave will target only:

- direct local-path executable module imports
- one root executable entry module containing `fn main()`
- imported executable declarations needed for ordinary helper-module programs

The goal is to admit ordinary module-based executable authoring without
silently widening into a general package or registry system.

## Included In First Wave

- top-level `Import` admission on the executable source path
- direct local-path import resolution for executable module graphs
- imported executable declarations for current source items such as:
  - `fn`
  - `record`
  - `enum`
  - `schema`
- deterministic executable module graph loading before semantic checking
- executable-path diagnostics for missing modules, missing selected symbols, and
  blocked out-of-scope import forms
- docs/tests/readiness updates that distinguish the admitted first wave from
  the already-frozen package ecosystem baseline

## Explicit Non-Goals

- external registries
- package manifests as part of this implementation wave
- lockfiles
- semantic version resolution
- generalized module loader redesign
- dynamic imports
- module-level executable statements
- wildcard or public re-export promises for the executable path unless they are
  explicitly admitted in a later scope decision

## Honest First-Wave Rules

- this track widens executable source admission, not the host/runtime boundary
- the root executable entrypoint remains `fn main()`
- imports remain deterministic and source-level only
- this wave is about helper-module authoring, not a full ecosystem
- landed package/dependency work on `main` does not automatically mean broader
  executable-module promises are qualified

## Suggested Wave Order

### Wave 0 — Governance

- scope checkpoint
- backlog/milestone/readiness sync

### Wave 1 — Parser And Source Admission

- admit top-level `Import` in executable source files
- keep rejected forms explicit where the executable path is still narrower than
  the broader module spec

### Wave 2 — Executable Module Resolution

- build the executable module graph before semantic checking
- make imported declarations available to the executable semantic path

### Wave 3 — Lowering / CLI / End-To-End

- run the admitted executable module graph through:
  - sema
  - IR
  - SemCode
  - verifier
  - VM
- keep diagnostics deterministic for missing or out-of-scope module cases

### Wave 4 — Freeze And Qualification Sync

- docs/spec/tests agree on the admitted executable module contour
- rerun qualification evidence if the practical-programming contour widens

## Acceptance Reading

This track is done only when:

1. ordinary helper-module executable programs are admitted on current `main`,
2. the executable import path is deterministic and tested end to end,
3. docs/spec/readiness language matches the actual admitted contour,
4. the release scope can be widened honestly only after the updated evidence is
   collected.
