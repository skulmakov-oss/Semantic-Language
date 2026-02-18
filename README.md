<img width="1536" height="1024" alt="ChatGPT Image 19 февр  2026 г , 03_13_05" src="https://github.com/user-attachments/assets/b7653b92-1dd9-4257-881f-4f24417b6ef4" />

# EXOcode

**EXOcode** is a deterministic compiler toolchain and virtual machine runtime for a quad-logic language (N/F/T/S), supporting both a Rust-like syntax and a LogosIndent DSL, targeting a versioned and verifiable EXObyte execution contract.

The project is designed around strict reproducibility, architectural boundaries, and system-level determinism.

It is not just a language frontend — it is a complete:

```
Source → Semantics → IR → Optimization → EXObyte → VM
```

pipeline.

---

# Philosophy

EXOcode is built around several engineering principles:

* Deterministic compilation: identical source + profile + feature set + caps must produce identical IR and EXObyte.
* Strict diagnostics: rustc-style errors with codes, spans, caret markers and help text.
* Layered architecture: frontend, semantics, IR, emit and VM are separate crates with enforced boundaries.
* Reproducible pipelines: incremental builds, dependency-aware cache invalidation, and traceable reasons for rebuilds.
* Readiness for `no_std` / `alloc` contexts: the core compiler layers are usable outside full `std` environments.

This makes EXOcode suitable for deterministic and system-level execution scenarios (e.g., VectorOS/Transjector-style environments).

---

# What EXOcode Solves

EXOcode addresses two core engineering problems:

### 1. Deterministic Compilation

Compile multiple source profiles (Rust-like and LogosIndent) into a unified IR and finally into a versioned EXObyte format, with:

* stable lowering,
* deterministic law scheduling,
* controlled optimization passes.

### 2. Verifiable Execution

Provide a bytecode contract (EXObyte) with:

* explicit header/version/caps,
* structural validation,
* opcode/section sanity checks,
* VM runtime behavior that matches compilation guarantees.

In short: EXOcode defines both the compiler and the execution contract.

---

# Current State (Beta Line)

Current release: `v1.1.0-beta.1`

Status of core subsystems:

* Workspace fully split into crates with explicit API boundaries.
* Import/Re-export model v0.2 implemented with collision and symbol-cycle diagnostics.
* Incremental cache pipeline supports dependency-aware invalidation and trace reasons.
* CrystalFold is formalized as a dedicated IR optimization pass.
* Root crate reduced to shim + binaries only (enforced by tests and CI guards).

The beta line focuses on architectural stability and reproducibility rather than rapid feature expansion.

---

# Core Capabilities

## Frontend

Two syntax profiles compile into the same IR:

* Rust-like
* LogosIndent (INDENT/DEDENT + continuation depth policy)

Both frontends:

* produce arena-first ASTs,
* attach SourceMark spans to tokens and nodes,
* emit rustc-style diagnostics.

---

## Semantics

The semantic layer provides:

* type checking (Int, Fx, QVec, Mask, Str, Bool, Quad, Unit),
* symbol scopes and duplicate/shadow detection,
* import/export validation,
* collision detection,
* symbol-level cycle detection,
* scheduling validation for laws,
* W0241 hint-only constant-fold detection.

Semantics guarantees structural correctness before lowering.

---

## IR

The IR layer provides:

* deterministic law ordering:

  * priority descending,
  * declaration-order tie-break,
* explicit gate surface:

  * `GateRead`
  * `GateWrite`
  * `PulseEmit`
* structural canonicalization before optimization.

IR is the canonical optimization boundary.

---

## Optimization (CrystalFold)

CrystalFold is a formal IR pass:

* implemented as `exo-ir::passes::crystalfold`,
* deterministic (linear instruction traversal),
* idempotent (folding twice yields identical IR),
* isolated from lowering (enforced by boundary tests).

Semantics-level `W0241` is a hint-only diagnostic.
Materialized rewrites occur exclusively at IR stage.

---

## EXObyte

EXObyte is a versioned bytecode format with:

* explicit header and schema version,
* caps table (epoch/rev/caps),
* section and opcode validation,
* reproducible layout guarantees.

The format is designed to support deterministic and system-level execution.

---

## VM

The VM runtime:

* executes EXObyte deterministically,
* validates sections before execution,
* provides disassembly tools,
* is separated from the compiler pipeline.

---

# Workspace Architecture

```text
crates/
  exo-core      # ids, arena, source/diag, interner, base types
  exo-frontend  # lexer/parser (Rust-like + LogosIndent), AST build
  exo-semantics # scopes/type/import checks, warnings, scheduling
  exo-ir        # IR model, lowering, opt passes (CrystalFold)
  exo-emit      # EXObyte emit/validation helpers
  exo-vm        # VM runtime + disasm
  exo-cli       # orchestration (std-only)
```

The root crate is intentionally limited to:

* shim/re-export API
* binaries (`exoc`, `exocode_core`)

Legacy backend code in root is forbidden and enforced by CI guards.

---

# Incremental Compilation & Cache Model

The incremental pipeline supports:

* AST packs
* Semantics packs
* IR packs
* EXObyte packs

Cache invalidation is dependency-aware and produces trace reasons such as:

* SOURCE_CHANGED
* DEP_CHANGED
* GRAPH_CHANGED
* TOOLCHAIN_CHANGED
* FEATURES_CHANGED
* SCHEMA_CHANGED
* CAPS_CHANGED
* CORRUPT_PACK

The system is designed to minimize rebuild scope while preserving correctness.

---

# Quick Start

```powershell
# Build CLI
cargo build --bin exoc

# Create minimal source
@'
fn main() {
    return;
}
'@ | Set-Content program.exo

# Compile to EXObyte
cargo run --bin exoc -- compile program.exo -o program.exb

# Semantic check only
cargo run --bin exoc -- check program.exo

# Inspect IR
cargo run --bin exoc -- dump-ir program.exo --opt-level O1

# Execute bytecode
cargo run --bin exoc -- runb program.exb

# Disassemble
cargo run --bin exoc -- disasm program.exb
```

---

# CLI Commands

Primary commands:

* `compile`
* `check`
* `lint`
* `watch`
* `dump-ast`
* `dump-ir`
* `hash-ast`
* `hash-ir`
* `hash-exb`
* `explain`
* `repl`
* `run`
* `runb`
* `disasm`
* `features`

Each command integrates with incremental cache and deterministic pipeline guarantees.

---

# no_std / alloc

* `exo-core`: no_std-oriented.
* `exo-frontend` and `exo-semantics`: alloc-first mode supported.
* CLI and filesystem orchestration are std-only.

Validation commands:

```powershell
cargo check --no-default-features --quiet
cargo check -p exo-frontend --no-default-features --features alloc --quiet
cargo check -p exo-semantics --no-default-features --features alloc --quiet
```

See `docs/NO_STD.md` for details.

---

# Testing & Quality Gates

Recommended gates:

```powershell
cargo test -q
cargo test -q --test golden_snapshots
cargo test -q --test imports_matrix
cargo test -q --test cache_trace_reason_matrix
cargo test -q --test cache_trace_dep_changed
cargo test -q --test legacy_guards
```

Architectural boundaries are protected by dedicated guard tests.

---

# Beta Constraints

* Internal APIs may evolve within beta iterations.
* IR and EXObyte contracts are beta-stable within the v1.1.x line.
* Optimization and incremental internals may evolve without external API guarantees.
* `W0241` remains hint-only; actual rewriting is IR-pass based.
