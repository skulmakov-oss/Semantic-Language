# Persistent Incremental Packs Full Scope

Status: completed post-stable closure track
Related track: `NEXT-2`

## Goal

Freeze the persistent incremental cache contract so reuse and invalidation
behavior is explicit, deterministic, and test-backed for the currently
supported CLI pack flows.

## Why This Is A Post-Stable Track

The repository already shipped stable `v1.1.1` with a usable incremental/cache
path and `--trace-cache` diagnostics.

The remaining work here is not a release blocker for the shipped stable line.
It is a tooling-discipline closure track intended to:

- make cache ownership and dependency edges explicit
- lock rebuild reasons into deterministic named categories
- prevent stale downstream reuse after dependency changes
- add inspectable smoke coverage for reuse and invalidation behavior
## Included In NEXT-2

- stable on-disk cache layout and pack header validation
- explicit `--trace-cache` reason codes for hits, misses, and invalidation
- dependency-aware semantic-pack invalidation via module-graph fingerprinting
- root-scoped `AstPack`, `IrPack`, and `SmcPack` key rules for current
  single-root CLI commands
- reuse smoke scenarios that prove:
  - unchanged reruns reuse cache entries
  - dependency edits invalidate semantic analysis and then settle back to reuse
  - EXB hashing reuses generated packs on unchanged reruns

## Explicit Non-Goals

- remote or distributed cache layers
- package-manager or workspace cache federation
- runtime-side cache integration
- new CLI flags or alternate cache stores
- widening `prom-*` or host/runtime boundaries

## Honest Current Contract

- `AstPack` is root-scoped and keyed by canonical input path plus source hash.
- `SemPack` is dependency-aware and keyed by module-graph fingerprint.
- `IrPack` is root-scoped for current `hash-ir` single-root compilation.
- `SmcPack` is root-scoped for current `hash-smc` single-root compilation.
- `DEP_CHANGED` is emitted when the semantic module graph changes between runs.
- `REUSED` is emitted only for real cache hits on the current pack family.

This means "leaf change rebuilds only downstream" is satisfied for the current
multi-module semantic path, while AST/IR/EXB remain intentionally root-scoped
until their command surfaces become graph-driven.

## Acceptance Freeze

`NEXT-2` is considered done when all of the following stay green and aligned
with `docs/cache.md`:

- `cargo test --test cache_trace_reason_matrix`
- `cargo test --test cache_trace_dep_changed`
- `cargo test --test cache_reuse_smoke`

## Slice History

1. cache layout and stage-key contract documented in `docs/cache.md`
2. deterministic `--trace-cache` reason-code matrix added
3. semantic downstream invalidation via dependency graph covered
4. reuse smoke scenarios frozen for unchanged reruns and post-rebuild steady
   state
