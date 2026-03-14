use std::fs;

const TARGETS: &[(&str, &str)] = &[
    ("crates/sm-ir/src/lib.rs", "tests/golden_snapshots/public_api/sm_ir_lib.txt"),
    (
        "crates/sm-profile/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_profile_lib.txt",
    ),
    (
        "crates/sm-runtime-core/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_runtime_core_lib.txt",
    ),
    (
        "crates/sm-verify/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_verify_lib.txt",
    ),
    ("crates/sm-vm/src/lib.rs", "tests/golden_snapshots/public_api/sm_vm_lib.txt"),
    (
        "crates/sm-vm/src/semcode_vm.rs",
        "tests/golden_snapshots/public_api/sm_vm_semcode_vm.txt",
    ),
    ("crates/prom-abi/src/lib.rs", "tests/golden_snapshots/public_api/prom_abi_lib.txt"),
    ("crates/prom-cap/src/lib.rs", "tests/golden_snapshots/public_api/prom_cap_lib.txt"),
    (
        "crates/prom-runtime/src/lib.rs",
        "tests/golden_snapshots/public_api/prom_runtime_lib.txt",
    ),
    ("crates/smc-cli/src/lib.rs", "tests/golden_snapshots/public_api/smc_cli_lib.txt"),
];

fn normalize_ws(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_public_item(line: &str) -> bool {
    line.starts_with("pub ") || line.starts_with("pub(")
}

fn normalized_public_surface(path: &str) -> String {
    let src = fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut lines = Vec::new();
    let mut pending_attrs = Vec::new();

    for raw in src.lines() {
        let line = raw.trim();
        if line.starts_with("#[") {
            pending_attrs.push(normalize_ws(line));
            continue;
        }
        if is_public_item(line) {
            lines.append(&mut pending_attrs);
            lines.push(normalize_ws(line));
            continue;
        }
        pending_attrs.clear();
    }

    format!("source: {}\n{}\n", path.replace('\\', "/"), lines.join("\n"))
}

fn normalize_snapshot_text(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end().to_string()
}

#[test]
fn public_api_inventory_matches_checked_in_contract_snapshots() {
    for (source, snapshot) in TARGETS {
        let actual = normalized_public_surface(source).trim_end().to_string();
        let expected =
            fs::read_to_string(snapshot).unwrap_or_else(|err| panic!("read {snapshot}: {err}"));
        assert_eq!(
            actual,
            normalize_snapshot_text(&expected),
            "public API inventory drifted for {source}; update snapshot only for intentional contract changes"
        );
    }
}
