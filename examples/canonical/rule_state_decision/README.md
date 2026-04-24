# rule_state_decision

- purpose: record-oriented rule/state decision logic with explicit
  `Result(T, E)` handling
- demonstrates:
  - nominal records
  - `quad`
  - contextual `Result::Ok` / `Result::Err`
  - explicit `match` settlement
- commands:
  - `cargo run --bin smc -- check examples/canonical/rule_state_decision/src/main.sm`
  - `cargo run --bin smc -- run examples/canonical/rule_state_decision/src/main.sm`
  - `cargo run --bin smc -- compile examples/canonical/rule_state_decision/src/main.sm -o out.smc`
  - `cargo run --bin smc -- verify out.smc`
- expected output:
  - `check` succeeds
  - `run` exits successfully
  - `verify` accepts the compiled `.smc`
