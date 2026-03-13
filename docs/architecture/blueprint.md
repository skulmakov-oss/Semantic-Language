# Semantic Architecture Blueprint

Semantic is split into two architectural products:

- `Semantic Core`: source, frontend, semantic analysis, IR, optimization, SemCode, verifier, VM.
- `PROMETHEUS Integration`: ABI, capabilities, semantic runtime, state/rule execution, audit.

The core execution rule is:

`source -> AST -> sema -> IR -> opt -> SemCode -> verify -> execute`

Current repository direction:

- `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-vm` are the active core layers.
- `sm-verify`, `sm-runtime-core`, `sm-profile`, `sm-opt` are canonical next-step crates.
- `prom-*` crates are reserved for the integration boundary and must stay separate from compiler/VM internals.

Non-negotiable architecture rules:

- compiler semantics and runtime semantics must stay separate;
- VM mechanics and semantic state/rule logic must stay separate;
- all host effects must cross a formal ABI boundary;
- verifier is a public admission layer, not an internal VM detail.
