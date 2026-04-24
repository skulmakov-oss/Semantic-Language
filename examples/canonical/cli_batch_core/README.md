# cli_batch_core

- purpose: small CLI-style computation core over `Sequence(i32)` and `text`
- demonstrates:
  - `Sequence(i32)` literals and iteration
  - `text` production and equality
  - single-file executable authoring
- commands:
  - `cargo run --bin smc -- check examples/canonical/cli_batch_core/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/cli_batch_core/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/cli_batch_core/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
