# Dependency And Boundary Rules

Project zones:

- `Construction`: `sm-front`, `sm-sema`, `sm-ir`, `sm-opt`, `sm-emit`, `sm-profile`
- `Execution`: `sm-verify`, `sm-runtime-core`, `sm-vm`
- `Integration`: `prom-abi`, `prom-cap`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-gates`, `prom-audit`

Allowed flow:

`Construction -> Execution -> Integration`

Boundary rules:

- construction crates must not depend on VM/runtime state or PROMETHEUS internals;
- execution crates must not reach back into parser/sema internals;
- integration crates must not rewrite compiler or VM semantics;
- all host effects must cross ABI and capability checks;
- public contracts require versioning, tests, and spec updates.

Immediate debt markers:

- `ParserProfile` outside `sm-profile` is architectural debt;
- string-keyed runtime locals are architectural debt;
- missing verifier/admission split is architectural debt.
