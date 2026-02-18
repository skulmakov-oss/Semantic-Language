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
