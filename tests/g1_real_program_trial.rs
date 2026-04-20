use std::path::PathBuf;

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(command: &str, rel: &str) {
    let path = repo_path(rel);
    smc_cli::run(vec![command.to_string(), path.clone()])
        .unwrap_or_else(|err| panic!("smc {command} failed for {path}: {err}"));
}

fn cli_err(command: &str, rel: &str) -> String {
    let path = repo_path(rel);
    smc_cli::run(vec![command.to_string(), path.clone()])
        .expect_err(&format!("smc {command} unexpectedly passed for {path}"))
}

#[test]
fn g1_cli_batch_core_checks_and_runs() {
    let rel = "examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_rule_state_decision_checks_and_runs() {
    let rel = "examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_data_audit_record_iterable_checks_and_runs() {
    let rel =
        "examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_module_helpers_program_is_blocked() {
    let rel = "examples/qualification/g1_real_program_trial/module_helpers_blocked/src/main.sm";
    let check_err = cli_err("check", rel);
    assert!(check_err.contains("expected top-level"));
    let run_err = cli_err("run", rel);
    assert!(run_err.contains("expected top-level"));
}
