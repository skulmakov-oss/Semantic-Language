# Dependency And Boundary Rules

Project zones:

- `Construction`: `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-profile`
- `Execution`: `sm-verify`, `sm-runtime-core`, `sm-vm`
- `Integration`: `prom-abi`, `prom-cap`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-gates`, `prom-audit`

Current pending ownership notes:

- optimizer surface currently lives in `sm-ir`; a dedicated `sm-opt` owner is still a decision item
- SemCode format contract currently spans `sm-ir/local_format` and `sm-emit`; canonical ownership is still a decision item
- public CLI surface is currently split between root `smc` and `smc-cli`

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
- unresolved optimizer / SemCode / CLI ownership is architectural debt.
