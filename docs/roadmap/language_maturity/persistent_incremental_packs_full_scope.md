# Persistent Incremental Packs To FULL

Status: proposed post-stable closure track

## Goal

Define the narrow closure boundary for bringing the current incremental pack
pipeline from "working baseline" to "FULL" without reopening language, runtime,
or packaging scope.

## Why This Is A Post-Stable Track

The repository already shipped stable `v1.1.1` with a usable incremental/cache
path and `--trace-cache` diagnostics.

The remaining work here is not a release blocker for the shipped stable line.
It is a tooling-discipline closure track intended to:

- make cache ownership and dependency edges explicit
- lock rebuild reasons into deterministic named categories
- prevent stale downstream reuse after dependency changes
- add inspectable smoke coverage for reuse and invalidation behavior

## In Scope

The `NEXT-2` closure track may include only:

- dependency-aware `CacheIndex` ownership and graph metadata
- deterministic rebuild reason codes for `--trace-cache`
- strict downstream invalidation rules across Ast/Sem/Ir/Exb pack families
- stable reuse/invalidation smoke scenarios and fixtures
- documentation sync for incremental ownership and trace semantics

## Out Of Scope

This closure track must not silently expand into:

- package manager or lockfile redesign
- distributed/shared cache systems
- build farm or remote execution infrastructure
- runtime/`prom-*` widening
- unrelated CLI surface redesign
- new language features disguised as cache invalidation work

## Intended Slice Order

1. docs/governance checkpoint
2. dependency-aware cache graph ownership
3. formal trace reason codes
4. strict downstream invalidation completion
5. reuse smoke/perf freeze

## Acceptance Reading

`NEXT-2` is done only when:

- cache reuse and invalidation reasons are explicit and deterministic
- dependency changes invalidate all downstream packs and only downstream packs
- `--trace-cache` output uses stable named reasons rather than ad hoc wording
- smoke coverage demonstrates predictable reuse across representative pipelines

## Non-Goal Reminder

This track is a tooling closure pass, not a new product surface wave.
