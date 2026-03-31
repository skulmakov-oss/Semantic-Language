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

Current repository limits that remain within the published stable `v1` line:

- richer `fx` arithmetic remains intentionally narrower than the `f64` surface in the canonical Rust-like execution path
- optimizer surface is fixed to `sm-ir` for the current `v1`; no separate `sm-opt` owner is planned inside the current baseline
- SemCode format surface is fixed to `sm-ir` for the current `v1`; `sm-emit` remains a producer facade over that contract
- public CLI surface is fixed to `smc-cli` for the current `v1`; root `smc` remains a thin process entrypoint over the canonical CLI owner
- PROMETHEUS `v1` scope is fixed to the current narrow ABI/capability/gate boundary; wider planned calls remain post-`v1`
- stable release packaging policy remains narrower than the long-term planned distribution story

Current release-line state:

- `main` carries the active narrow `v1` stable line
- release validation runs through boundary guards, public API inventory, runtime matrix/goldens, and the release-bundle verifier
- published stable releases are expected to ship `smc.exe`, `svm.exe`, and a bundled Windows archive

Non-negotiable architecture rules:

- compiler semantics and runtime semantics must stay separate;
- VM mechanics and semantic state/rule logic must stay separate;
- all host effects must cross a formal ABI boundary;
- verifier is a public admission layer, not an internal VM detail.
