# Dependency And Boundary Rules

Project zones:

- `Construction`: `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-profile`
- `Execution`: `sm-verify`, `sm-runtime-core`, `sm-vm`
- `Integration`: `prom-abi`, `prom-cap`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-gates`, `prom-audit`

Current pending ownership notes:

- optimizer surface is owned by `sm-ir` in the current `v1` baseline; a future `sm-opt` split would require an explicit follow-up decision and code move
- SemCode format contract is owned by `sm-ir` in the current `v1` baseline; `sm-emit` remains a producer-facing facade and compatibility layer
- public CLI contract is owned by `smc-cli` in the current `v1` baseline; root `smc` is a thin entrypoint wrapper over that owner
- `ton618_core`, `src/bin/support/**`, and `ton618-core` remain allowlisted compatibility shims only and must not grow into second owners

Allowed flow:

`Construction -> Execution -> Integration`

Boundary rules:

- construction crates must not depend on VM/runtime state or PROMETHEUS internals;
- execution crates must not reach back into parser/sema internals;
- integration crates must not rewrite compiler or VM semantics;
- all host effects must cross ABI and capability checks;
- public contracts require versioning, tests, and spec updates.

Current enforcement note:

- these boundary rules are repository policy now
- CI now enforces baseline ownership and dependency guards for root shims, SemCode owner alignment, optimizer owner alignment, and crate-level forbidden dependency checks
- broader graph visualization, API diff, and release gating remain pending `M6`

Immediate debt markers:

- `ParserProfile` outside `sm-profile` is architectural debt;
- richer `fx` arithmetic beyond the current literal/value transport path is architectural debt;
