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

#[test]
fn executable_module_entry_wave2_local_helper_import_checks_and_runs() {
    let rel = "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}
