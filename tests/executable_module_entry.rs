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
fn executable_module_entry_wave2_local_helper_import_checks_and_runs() {
    let rel = "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn executable_module_entry_negative_out_of_scope_import_forms_report_wave2_boundary() {
    let wave2_boundary =
        "top-level executable Import currently admits only direct local-path helper-module imports in wave2";
    let cases = [
        "examples/qualification/executable_module_entry/negative_alias_import/src/main.sm",
        "examples/qualification/executable_module_entry/negative_selected_import/src/main.sm",
        "examples/qualification/executable_module_entry/negative_wildcard_import/src/main.sm",
        "examples/qualification/executable_module_entry/negative_reexport_import/src/main.sm",
        "examples/qualification/executable_module_entry/negative_package_qualified_import/src/main.sm",
    ];

    for rel in cases {
        let err = cli_err("check", rel);
        assert!(
            err.contains(wave2_boundary),
            "expected wave2 executable import boundary diagnostic for {rel}, got: {err}"
        );
    }
}

#[test]
fn executable_module_entry_negative_graph_and_namespace_cases_report_explicit_failures() {
    let cases = [
        (
            "examples/qualification/executable_module_entry/negative_cycle_bare_import/src/main.sm",
            "cyclic executable helper import detected:",
        ),
        (
            "examples/qualification/executable_module_entry/negative_duplicate_symbol_collision/src/main.sm",
            "duplicate function 'score'",
        ),
        (
            "examples/qualification/executable_module_entry/negative_namespace_collision/src/main.sm",
            "duplicate function 'score'",
        ),
    ];

    for (rel, needle) in cases {
        let err = cli_err("check", rel);
        assert!(
            err.contains(needle),
            "expected diagnostic '{needle}' for {rel}, got: {err}"
        );
    }
}
