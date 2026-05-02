# M-Tail T5 — Legacy/Perimeter Truth Check

Status: closed  
Date: 2026-05-02

## Scope

Verify that no architecture drift has occurred into legacy areas:

- `legacy_lowering.rs` — is this a dead dual-path or the live pipeline?
- `ton618-core` — is the stated compatibility perimeter still respected?
- All other `legacy`-named identifiers, contracts, and markers

## Findings

### 1. `crates/sm-ir/src/legacy_lowering.rs`

**Status: active production pipeline — no drift.**

`legacy_lowering.rs` is the sole IR lowering and SemCode emit module.  
It is the live production pipeline:

```rust
// sm-ir/src/lib.rs
mod legacy_lowering;
pub use legacy_lowering::*;  // entire public API
```

The `legacy_` prefix is a historical naming artifact from a period when a
replacement IR was planned. No replacement was built. There is no dual-path
situation; this module IS the pipeline. The name is benign and documented
in context — no action needed.

The IR passes (`cleanup.rs`, `crystalfold.rs`, `mod.rs`) correctly import
from `legacy_lowering` as the source of `IrInstr` and `IrFunction` types.

### 2. `crates/ton618-core`

**Status: correctly scoped — perimeter is intact.**

`ton618-core/src/lib.rs` self-documents its constraint:

> "Canonical ownership of public platform contracts lives in the `sm-*` crates.
> This crate keeps the historical `ton618-core` name only as part of the retained
> compatibility perimeter and must not become a second public owner."

Current dependents:

| Crate | Items used |
|---|---|
| `sm-front` | `SymbolId`, `SourceMark`, `SourceMap`, diagnostics |
| `sm-sema` | `SourceMark`, `SourceMap`, `Arena`, diagnostics |
| `smc-cli` | `diagnostic_catalog` |

No `prom-*`, `semantic-core-*`, or `sm-ir` crate depends on `ton618-core`.  
The crate has not expanded its surface. The perimeter constraint is respected.

### 3. `sm-profile` — `"semantic.legacy"` identity string

**Status: test fixture only — no drift.**

`crates/sm-profile/src/lib.rs:361` assigns `profile.identity = "semantic.legacy"`
inside a `#[test]` roundtrip test. It is a test value for the `identity` string
field, not a production contract identifier.

### 4. `sm-front/src/parser.rs` — `require_legacy_compatibility`

**Status: intentional compatibility gate — no drift.**

`require_legacy_compatibility` is a parser-level method that gates old Logos
directive syntax behind a `legacy_compat` mode flag. It does not represent
architecture drift; it is an explicit compatibility boundary already present in
the parser design.

### 5. `prom-cap/src/lib.rs` — `"prom.cap.legacy"` contract name

**Status: capability contract name string — no drift.**

`CapabilityManifest::with_contract("prom.cap.legacy", ...)` is a named contract
identifier (a string). The `prom-cap` crate maintains its own versioned contract
registry; `"prom.cap.legacy"` is one entry in that registry, not evidence of
architectural bleed.

### 6. `sm-front/src/typecheck.rs:2065` — `TODO(M9.5)`

**Status: documented known limitation — not drift.**

```rust
// TODO(M9.5): disambiguate expr parsing for scrutinee to avoid record-literal conflict
```

This is a scoped, labeled outstanding issue for a future milestone. It does not
indicate unplanned architectural drift.

## Conclusion

No uncontrolled drift into legacy areas is present.

| Area | Verdict |
|---|---|
| `legacy_lowering.rs` | Active pipeline — name is historical, no dual-path |
| `ton618-core` perimeter | Intact — 3 dependents, foundational types only |
| `"semantic.legacy"` string | Test fixture only |
| `require_legacy_compatibility` | Intentional parser compat gate |
| `"prom.cap.legacy"` | Capability registry entry name |
| `TODO(M9.5)` | Labeled known limitation for future milestone |

T5 is closed. No further action required.
