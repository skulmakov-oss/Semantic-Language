# wave2_local_helper_import

- purpose: admitted helper-module executable authoring with direct local-path
  bare import
- demonstrates:
  - `Import "helper.sm"`
  - deterministic helper-module bundling
  - end-to-end `check -> compile -> verify -> run` on a multi-file executable
- commands:
  - `cargo run --bin smc -- check examples/canonical/wave2_local_helper_import/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/wave2_local_helper_import/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/wave2_local_helper_import/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
