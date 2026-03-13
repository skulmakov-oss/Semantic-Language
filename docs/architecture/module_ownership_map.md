# Module Ownership Map

One concept must have one owner module.

Core ownership:

- lexical model: `sm-lexer` / current frontend lexer layer
- AST and syntax model: `sm-ast` / current frontend AST layer
- parser profiles: `sm-profile`
- compiler semantics: `sm-sema`
- IR and lowering: `sm-ir`
- optimization passes: `sm-opt`
- SemCode binary contract: `sm-emit`
- bytecode admission contract: `sm-verify`
- runtime primitives: `sm-runtime-core`
- VM execution mechanics: `sm-vm`
- CLI orchestration: `sm-cli`

Integration ownership:

- host ABI: `prom-abi`
- capability policy: `prom-cap`
- semantic state: `prom-state`
- rule runtime: `prom-rules`
- orchestration: `prom-runtime`
- audit and replay metadata: `prom-audit`

Ownership rules:

- `sm-emit` owns the format, `sm-vm` consumes it;
- `sm-verify` owns admission, `sm-vm` executes only admitted code;
- `prom-state` owns semantic state, not `sm-vm`;
- `prom-rules` owns agenda/conflict logic, not `sm-vm`;
- `prom-runtime` orchestrates but does not own all subdomains.
