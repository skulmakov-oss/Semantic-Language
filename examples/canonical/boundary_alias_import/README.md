# boundary_alias_import

- purpose: honest boundary example for the still-blocked top-level alias import
  form on the executable path
- demonstrates:
  - `Import "helper.sm" as Helper`
  - current executable-path narrowing
- commands:
  - `cargo run --bin smc -- check examples/canonical/boundary_alias_import/src/main.sm`
- expected output:
  - `check` fails
  - the diagnostic contains:
    `top-level executable Import currently admits direct local-path helper-module imports plus selected imports in wave2`
