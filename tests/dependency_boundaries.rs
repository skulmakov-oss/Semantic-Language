use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn assert_no_path_dep(path: &str, forbidden: &[&str]) {
    let src = read(path);
    for dep in forbidden {
        let needle = format!("path = \"../{dep}\"");
        assert!(
            !src.contains(&needle),
            "{path} must not depend on forbidden crate {dep}"
        );
    }
}

#[test]
fn construction_crates_do_not_depend_on_vm_or_prometheus_layers() {
    let construction = [
        "crates/sm-front/Cargo.toml",
        "crates/sm-sema/Cargo.toml",
        "crates/sm-ir/Cargo.toml",
        "crates/sm-emit/Cargo.toml",
        "crates/sm-profile/Cargo.toml",
    ];
    let forbidden = [
        "sm-vm",
        "prom-abi",
        "prom-cap",
        "prom-gates",
        "prom-runtime",
        "prom-state",
        "prom-rules",
        "prom-audit",
    ];

    for path in construction {
        assert_no_path_dep(path, &forbidden);
    }
}

#[test]
fn execution_crates_do_not_depend_on_frontend_or_sema_layers() {
    let execution = [
        "crates/sm-verify/Cargo.toml",
        "crates/sm-runtime-core/Cargo.toml",
        "crates/sm-vm/Cargo.toml",
    ];
    let forbidden = ["sm-front", "sm-sema"];

    for path in execution {
        assert_no_path_dep(path, &forbidden);
    }
}

#[test]
fn integration_crates_do_not_depend_on_compiler_layers() {
    let integration = [
        "crates/prom-abi/Cargo.toml",
        "crates/prom-cap/Cargo.toml",
        "crates/prom-gates/Cargo.toml",
        "crates/prom-runtime/Cargo.toml",
        "crates/prom-state/Cargo.toml",
        "crates/prom-rules/Cargo.toml",
        "crates/prom-audit/Cargo.toml",
    ];
    let forbidden = ["sm-front", "sm-sema", "sm-ir", "sm-emit"];

    for path in integration {
        assert_no_path_dep(path, &forbidden);
    }
}
