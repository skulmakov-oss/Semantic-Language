use std::fs;
use std::path::{Path, PathBuf};
use std::collections::BTreeSet;

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(root) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs_files(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_path_adapter_back_to_root_src() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("crates"), &mut files);
    for f in files {
        let txt = fs::read_to_string(&f).expect("read rs");
        assert!(
            !txt.contains("#[path = \"../../../src/"),
            "forbidden legacy path adapter in {}",
            f.display()
        );
    }
}

#[test]
fn root_src_is_shim_and_bins_only_allowlist() {
    let mut rs_files = Vec::new();
    collect_rs_files(Path::new("src"), &mut rs_files);
    let mut bad = Vec::new();
    for f in rs_files {
        let rel = f.strip_prefix(".").unwrap_or(&f);
        let rel = rel.to_string_lossy().replace('\\', "/");
        let ok = rel == "src/lib.rs" || rel.starts_with("src/bin/");
        if !ok {
            bad.push(rel);
        }
    }
    assert!(
        bad.is_empty(),
        "root/src contains forbidden rust sources outside allowlist: {:?}",
        bad
    );
}

#[test]
fn root_src_contains_only_lib_and_bin_dir() {
    let mut top = BTreeSet::new();
    for e in fs::read_dir("src").expect("read src").flatten() {
        top.insert(e.file_name().to_string_lossy().to_string());
    }
    let expected: BTreeSet<String> = ["lib.rs", "bin"].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        top, expected,
        "root/src must contain only lib.rs and bin/"
    );
}

#[test]
fn root_src_bans_legacy_patterns() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files);
    for f in files {
        let txt = fs::read_to_string(&f).expect("read rs");
        let rel = f.to_string_lossy().replace('\\', "/");
        assert!(
            !txt.contains("legacy_"),
            "forbidden legacy marker in {}",
            rel
        );
        assert!(
            !txt.contains("#[path ="),
            "forbidden #[path = ...] in {}",
            rel
        );
        assert!(
            !txt.contains("include!("),
            "forbidden include!(...) in {}",
            rel
        );
        assert!(
            !txt.contains("mod legacy"),
            "forbidden legacy module declaration in {}",
            rel
        );
    }
}

#[test]
fn root_smc_is_thin_wrapper_over_smc_cli() {
    let root = fs::read_to_string("src/bin/smc.rs").expect("read root smc");
    let owner = fs::read_to_string("crates/smc-cli/src/app.rs").expect("read smc-cli app");

    assert!(
        root.contains("smc_cli::main_entry()"),
        "root smc must delegate to the canonical smc-cli owner"
    );
    assert!(
        !root.contains("match args[0].as_str()"),
        "root smc must not carry its own command dispatch"
    );
    assert!(
        owner.contains("pub fn main_entry()"),
        "smc-cli must own the exported CLI process entry"
    );
    assert!(
        owner.contains("pub fn run(args: Vec<String>)"),
        "smc-cli must own the command runner"
    );
}

#[test]
fn legacy_compatibility_perimeter_is_explicit_and_narrow() {
    let explicit_shims = [
        "src/bin/ton618_core.rs",
        "src/bin/support/mod.rs",
        "src/bin/support/language.rs",
        "src/bin/support/parser.rs",
        "crates/ton618-core/src/lib.rs",
    ];

    for path in explicit_shims {
        let txt = fs::read_to_string(path).expect("read compatibility shim");
        assert!(
            txt.contains("compatibility"),
            "legacy shim {} must declare explicit compatibility status",
            path
        );
    }

    let mut ton618_named = Vec::new();
    collect_rs_files(Path::new("src"), &mut ton618_named);
    collect_rs_files(Path::new("crates/ton618-core/src"), &mut ton618_named);
    let mut matches = Vec::new();
    for file in ton618_named {
        let rel = file.to_string_lossy().replace('\\', "/");
        if rel.contains("ton618") {
            matches.push(rel);
        }
    }
    matches.sort();
    matches.dedup();

    assert!(
        matches.iter().all(|rel| {
            rel == "src/bin/ton618_core.rs" || rel.starts_with("crates/ton618-core/src/")
        }),
        "legacy ton618 naming must remain inside the explicit compatibility perimeter only: {:?}",
        matches
    );
    assert!(
        matches.iter().any(|rel| rel == "src/bin/ton618_core.rs"),
        "legacy compatibility bin must remain explicit"
    );
}
