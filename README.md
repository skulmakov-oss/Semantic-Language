<p align="center">
  <img src="assets/brand/semantic-logo.png" alt="Semantic" width="860">
</p>

# Semantic
Rust-like deterministic language toolchain with SemCode emission, verifier admission, and VM execution.

Semantic is built for reasoning rules, semantic state transitions, declarative Logos surfaces, and executable logic inside the broader PROMETHEUS system model.

The public contract is centered in `docs/spec/*`. Historical roadmap notes and legacy compatibility shims remain in the repository, but they are not the primary source of truth for the current toolchain surface.

## Current State
- Active draft toolchain on `main`; this repository is not frozen on `release/v0`.
- Standard execution route: `source -> AST -> sema -> IR -> SemCode -> verify -> execute`.
- SemCode is versioned and verifier-gated before standard VM execution.
- Tuple + direct record-field runtime ownership is implemented end-to-end for borrowed-path write rejection:
  - frontend preserves borrow capture
  - lowering emits canonical ownership path events
  - SemCode transports ownership metadata
  - verifier admits ownership payload structurally
  - VM rejects overlapping tuple and direct record-field writes at runtime
- CLI ownership is centered in `crates/smc-cli`; root `smc` and `svm` binaries remain process entrypoints.

## Primary References
- `docs/spec/index.md` - canonical spec bundle entrypoint
- `docs/spec/syntax.md` - source syntax contract
- `docs/spec/types.md` - source type contract
- `docs/spec/source_semantics.md` - source execution and binding semantics
- `docs/spec/semcode.md` - SemCode contract and version policy
- `docs/spec/verifier.md` - admission verifier contract
- `docs/spec/vm.md` - VM execution contract
- `docs/spec/runtime_ownership.md` - frozen tuple + direct record-field runtime ownership contract
- `docs/spec/cli.md` - public CLI surface
- `docs/LANGUAGE.md` - language overview and design intent
- `docs/NAMING.md` - naming rules and short forms

## What Is In The Repository
- Source frontend: lexer, parser, typing, and source-surface ownership work in `crates/sm-front`
- Semantic analysis and diagnostics in `crates/sm-sema`
- Lowering, IR, optimization passes, and canonical SemCode contract in `crates/sm-ir`
- Producer-facing SemCode facade in `crates/sm-emit`
- Structural SemCode admission verifier in `crates/sm-verify`
- Shared runtime vocabulary and quotas in `crates/sm-runtime-core`
- Verified-only VM execution in `crates/sm-vm`
- Canonical public CLI owner in `crates/smc-cli`
- PROMETHEUS-facing boundary crates:
  - `crates/prom-abi`
  - `crates/prom-cap`
  - `crates/prom-gates`
  - `crates/prom-runtime`
  - `crates/prom-state`
  - `crates/prom-rules`
  - `crates/prom-audit`
  - `crates/prom-ui`
  - `crates/prom-ui-runtime`
  - `crates/prom-ui-demo`
- Compatibility perimeter:
  - `src/bin/ton618_core.rs`
  - `crates/ton618-core`

## Quickstart
Use these commands from repository root.

```powershell
# 1) Build the public entrypoints
cargo build --bin smc --bin svm

# 2) Create a minimal program
@'
fn main() {
    return;
}
'@ | Set-Content program.sm

# 3) Check source
cargo run --bin smc -- check program.sm

# 4) Compile source -> SemCode
cargo run --bin smc -- compile program.sm -o program.smc

# 5) Verify compiled SemCode
cargo run --bin smc -- verify program.smc

# 6) Run source directly
cargo run --bin smc -- run program.sm

# 7) Run precompiled SemCode through the standard CLI route
cargo run --bin smc -- run-smc program.smc

# 8) Disassemble SemCode
cargo run --bin svm -- disasm program.smc
```

## Current CLI Surface
Current command families exposed by `smc`:
- `compile`
- `check`
- `lint`
- `watch`
- `fmt`
- `dump-ast`
- `dump-ir`
- `dump-bytecode`
- `hash-ast`
- `hash-ir`
- `hash-smc`
- `snapshots`
- `features`
- `explain`
- `repl`
- `verify`
- `run`
- `run-smc`
- `disasm`

Low-level VM entrypoint:
- `svm run <input.smc>`
- `svm disasm <input.smc>`

## Current SemCode And Runtime Notes
- The SemCode contract is owned by `sm-ir` and surfaced through `sm-emit`.
- The current spec documents a versioned SemCode family and capability-gated emission.
- Standard `.smc` execution is verifier-first; verified admission is not optional on the public route.
- The current runtime ownership slice is intentionally narrow:
  - tuple paths
  - direct record-field paths
  - frame-local borrow lifetime
  - exact overlap rejection
  - parent-child rejection
  - child-parent rejection
  - sibling writes allowed
  - unsupported: ADT payload paths, schema paths, partial release, aliasing graphs, inter-frame persistence, and indirect projections

## Testing
```powershell
cargo fmt --check
cargo test -q
cargo test -q --test public_api_contracts
cargo test -q --test runtime_ownership_e2e
```

## no_std Smoke Check
Core library supports `no_std` mode.

```powershell
cargo check --no-default-features
```

Reference:
- `docs/NO_STD.md`

## License
Apache License 2.0

Copyright (c) 2026 Said Kulmakov

See `LICENSE` for the repository license text.
