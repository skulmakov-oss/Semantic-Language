# Arena-first + CrystalFold Full Scope

Status: proposed post-stable closure track
Related track: `NEXT-4`

## Goal

Close the remaining gap between the current working arena/optimization baseline
and a fully frozen, production-disciplined ownership/pass contract.

This track is about formalizing and tightening what already exists. It is not a
new feature wave.

## Why This Is A Post-Stable Track

The stable line already ships with:

- arena-backed frontend ownership through `AstArena`
- deterministic lowering based on arena-owned frontend structures
- `CrystalFold` as an IR pass documented in `docs/opts.md`

The remaining work is about making the internal stage contracts complete and
explicit:

- remove remaining non-arena ownership tails where they still leak through
  internal boundaries
- strengthen arena invariants as test-backed guarantees instead of informal
  assumptions
- freeze `CrystalFold` placement, scope, and determinism expectations as a pass
  contract, not just an implementation detail

## Included In NEXT-4

- inventory and removal of remaining non-arena ownership tails inside frontend /
  semantics / lowering boundaries
- explicit arena invariants tests for append-only ID stability, cross-stage
  ownership, and stable diagnostics inputs
- frozen `CrystalFold` pass contract:
  - stage placement
  - pass interface
  - deterministic rewrite order
  - warning/correctness boundaries
- deterministic correctness test pack for optimizer behavior and warning/span
  stability
- docs sync in `docs/opts.md` and related architecture/roadmap pages

## Explicit Non-Goals

- new optimization families beyond `CrystalFold`
- semantic/runtime widening
- changing language surface or source syntax
- introducing new public CLI flags for optimizer internals
- hidden behavior changes masked as "cleanup"
- `prom-*` or host boundary expansion

## Honest Current Baseline

- `AstArena` already exists and has direct append-only ID stability tests.
- `CrystalFold` already exists as an IR pass and is already documented as
  deterministic and idempotent.
- raw frontend `Expr` / `Stmt` storage is already confined to `AstArena`; this
  track should freeze and guard that perimeter before attempting deeper
  ownership cleanup.
- `NEXT-4` is therefore not about inventing arena-first ownership or introducing
  CrystalFold from scratch.

It is about moving from "implemented and partially tested" to "complete and
frozen as an internal stage contract".

## Intended Slice Order

1. docs/governance checkpoint
2. ownership guard freeze for the current arena perimeter
3. remaining non-arena ownership inventory and narrow cleanup slices
4. explicit arena invariants test freeze
5. `CrystalFold` stage-placement/interface freeze and determinism pack
6. docs-only close-out for the full stage contract

## Acceptance Reading

`NEXT-4` is done only when:

- remaining internal ownership tails are either removed or explicitly justified
- arena invariants are test-backed and stable
- `CrystalFold` has a frozen pass contract in `docs/opts.md`
- correctness, warning behavior, and deterministic output are covered by a
  dedicated test pack
- no part of the track widens runtime, CLI, or language surface boundaries

## Slice History

1. scope checkpoint and done-boundary freeze
2. frontend arena-perimeter guards for raw `Expr` / `Stmt` ownership
3. move-only cleanup-pass ownership for the current structural IR cleanup stage
