# data_audit_record_iterable

- purpose: data-heavy audit pass over direct-record `Iterable` dispatch
- demonstrates:
  - direct-record `Iterable` impls
  - `for value in samples`
  - immutable record update with `with { ... }`
  - boolean accumulation over record data
- commands:
  - `cargo run --bin smc -- check examples/canonical/data_audit_record_iterable/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/data_audit_record_iterable/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/data_audit_record_iterable/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
