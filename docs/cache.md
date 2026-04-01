# Cache & Incremental Pipeline

This document defines the current persistent cache behavior for Semantic.

## Layout

Cache root:

```text
.smantic-cache/
  schema.json
  index.bin
  graph.bin
  packs/
    sem/*.smpack
    ast/*.astpack
    ir/*.irpack
    exb/*.smcpack
```

## Pack Header Contract

Each pack file uses a binary header (`EXOP`) with:

- `kind` (`SEMP`, `ASTP`, `IRPK`, `SMCP`)
- `schema_version`
- `toolchain_hash`
- `feature_hash` (for `SMCP`: includes caps hash salt)
- `payload_len`
- `payload_checksum`

Validation fails on any mismatch and is surfaced via trace reasons.

## Stage Keys (Current)

- `AstPack`: file canonical path + source hash + frontend marker
- `SemPack`: module graph fingerprint (`module_graph_fingerprint`)
- `IrPack`: file canonical path + source hash + profile + opt-level
- `ExbPack`: file canonical path + source hash + profile + opt-level +
  debug-symbol flag

## Trace Reasons (`--trace-cache`)

Structured events:

- `cache_hit`
- `cache_miss`
- `invalidate`

Reasons currently emitted:

- `REUSED`
- `CACHE_DISABLED`
- `NOT_FOUND`
- `HEADER_INVALID`
- `KIND_MISMATCH`
- `SCHEMA_CHANGED`
- `TOOLCHAIN_CHANGED`
- `FEATURES_CHANGED`
- `CAPS_CHANGED` (EXB path)
- `CORRUPT_PACK`
- `SOURCE_CHANGED`
- `DEP_CHANGED`
- `DENY_POLICY`

## Invalidation Scope (Current)

- `SOURCE_CHANGED`: current root-scoped stage invalidated and rebuilt.
- `DEP_CHANGED`: semantic dependency graph/fingerprint changed, downstream
  semantic analysis rebuilt.
- `TOOLCHAIN_CHANGED`: all packs for current target become misses.
- `FEATURES_CHANGED`: pack misses for changed feature hash (all affected stages).
- `SCHEMA_CHANGED`: pack misses due to schema mismatch.
- `CAPS_CHANGED`: EXB pack miss (caps salt differs), rebuild emit stage.
- `CORRUPT_PACK`: pack payload/header integrity failure; rebuild from previous stage.

## Test Overrides (for deterministic tests)

Supported env vars:

- `SM_TOOLCHAIN_HASH`
- `SM_FEATURE_HASH`
- `SM_CACHE_SCHEMA`
- `SM_CAPS_HASH`

These are used in integration tests to force specific cache-reason paths.

## Notes

- `hash-smc` supports `--trace-cache` for EXB cache diagnostics.
- Dependency changes are reported as `DEP_CHANGED`.
- Corrupted pack payloads are reported as `CORRUPT_PACK`.

## Reuse Smoke Matrix

The current freeze relies on three focused integration tests:

- `tests/cache_trace_reason_matrix.rs` freezes reason-code names and
  miss/invalidate categories.
- `tests/cache_trace_dep_changed.rs` proves dependency edits surface
  `DEP_CHANGED` on the semantic path.
- `tests/cache_reuse_smoke.rs` proves unchanged reruns reuse semantic and EXB
  packs, and that a dependency-triggered rebuild settles back to `REUSED` on
  the next unchanged run.

