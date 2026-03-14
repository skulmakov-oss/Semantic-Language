# Module Ownership Map

One concept must have one owner module.

Core ownership:

- lexical model: `sm-lexer` / current frontend lexer layer
- AST and syntax model: `sm-ast` / current frontend AST layer
- parser profiles: `sm-profile`
- compiler semantics: `sm-sema`
- IR and lowering: `sm-ir`
- optimization passes: current implementation lives in `sm-ir`; canonical owner is pending decision between `sm-ir` and `sm-opt`
- SemCode binary contract: current implementation is split between `sm-ir/local_format` and `sm-emit`; canonical owner is pending decision
- bytecode admission contract: `sm-verify`
- runtime primitives: `sm-runtime-core`
- VM execution mechanics: `sm-vm`
- CLI orchestration: current implementation is split between root `smc` and `smc-cli`; canonical owner is pending decision

Integration ownership:

- host ABI: `prom-abi`
- capability policy: `prom-cap`
- semantic state: `prom-state`
- rule runtime: `prom-rules`
- orchestration: `prom-runtime`
- audit and replay metadata: `prom-audit`

Ownership rules:

- no public contract may keep two long-term owners;
- pending ownership decisions must be marked explicitly rather than implied as settled;
- `sm-vm` consumes SemCode but does not own its format contract;
- `sm-verify` owns admission, `sm-vm` executes only admitted code;
- `prom-state` owns semantic state, not `sm-vm`;
- `prom-rules` owns agenda/conflict logic, not `sm-vm`;
- `prom-runtime` orchestrates but does not own all subdomains.
