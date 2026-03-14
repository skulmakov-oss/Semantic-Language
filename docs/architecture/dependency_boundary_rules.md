# Dependency And Boundary Rules

Project zones:

- `Construction`: `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-profile`
- `Execution`: `sm-verify`, `sm-runtime-core`, `sm-vm`
- `Integration`: `prom-abi`, `prom-cap`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-gates`, `prom-audit`

Current pending ownership notes:

- optimizer surface is owned by `sm-ir` in the current `v1` baseline; a future `sm-opt` split would require an explicit follow-up decision and code move
- SemCode format contract is owned by `sm-ir` in the current `v1` baseline; `sm-emit` remains a producer-facing facade and compatibility layer
- public CLI contract is owned by `smc-cli` in the current `v1` baseline; root `smc` remains an entrypoint shell pending cleanup

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
- full CI enforcement for dependency graph and forbidden imports is still pending `M6`

Immediate debt markers:

- `ParserProfile` outside `sm-profile` is architectural debt;
- incomplete `fx` canonical execution path is architectural debt;
- root-bin CLI implementation split is architectural debt until code layout catches up with the owner decision.
