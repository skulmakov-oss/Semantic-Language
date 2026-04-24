# positive_selected_import

- purpose: admitted helper-module executable authoring with direct local-path
  selected import
- demonstrates:
  - `Import "helper.sm" { score }`
  - function-only helper-module selected import
  - deterministic selected-binding synthesis before executable semantic checking
- commands:
  - `cargo run --bin smc -- check examples/canonical/positive_selected_import/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/positive_selected_import/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/positive_selected_import/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
