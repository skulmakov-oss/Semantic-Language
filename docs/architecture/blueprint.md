# Semantic Architecture Blueprint

Semantic is split into two architectural products:

- `Semantic Core`: source, frontend, semantic analysis, IR, optimization, SemCode, verifier, VM.
- `PROMETHEUS Integration`: ABI, capabilities, semantic runtime, state/rule execution, audit.

The core execution rule is:

`source -> AST -> sema -> IR -> opt -> SemCode -> verify -> execute`

Current repository state:

- active core crates: `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`, `sm-profile`
- active integration crates: `prom-abi`, `prom-cap`, `prom-gates`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-audit`
- `prom-*` crates remain separate from compiler and VM internals

Current repository gaps that remain before an honest `v1` claim:

- `fx` is still incomplete in the canonical Rust-like execution path
- canonical ownership is still pending decision for:
  - SemCode format surface: `sm-emit` vs `sm-ir/local_format`
  - public CLI surface: `smc-cli` vs root `smc`
- optimizer surface is fixed to `sm-ir` for `v1`; no separate `sm-opt` owner exists in the current repository baseline
- PROMETHEUS `v1` scope is still pending a narrow-vs-full boundary decision
- CI enforcement is still weaker than the planned `M6` boundary and release gates

Non-negotiable architecture rules:

- compiler semantics and runtime semantics must stay separate;
- VM mechanics and semantic state/rule logic must stay separate;
- all host effects must cross a formal ABI boundary;
- verifier is a public admission layer, not an internal VM detail.
