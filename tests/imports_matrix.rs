use sm_sema::{check_file_with_provider, ModuleProvider};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

struct FsProvider;

impl ModuleProvider for FsProvider {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
        fs::read(module_id).map_err(|e| e.to_string())
    }
}

fn fixture_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in fs::read_dir(root).expect("read fixture root").flatten() {
        let p = e.path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn imports_policy_matrix() {
    let root = Path::new("tests/fixtures/imports");
    let provider = FsProvider;
    let mut seen = BTreeSet::<String>::new();

    for case in fixture_dirs(root) {
        let expect = fs::read_to_string(case.join("EXPECT"))
            .expect("EXPECT")
            .trim()
            .to_string();
        let entry = case.join("main.sm").canonicalize().expect("main.sm");
        let res = check_file_with_provider(&entry, &provider);

        if expect == "OK" {
            seen.insert("OK".to_string());
            if let Err(e) = res {
                panic!("case '{}' expected OK, got: {}", case.display(), e);
            }
            continue;
        }

        let code = expect.strip_prefix("ERR ").expect("ERR format");
        seen.insert(code.to_string());
        match res {
            Ok(_) => panic!("case '{}' expected {}, got OK", case.display(), code),
            Err(e) => {
                let text = e.to_string();
                assert!(
                    text.contains(code),
                    "case '{}': expected code {}, got:\n{}",
                    case.display(),
                    code,
                    text
                );
                let expect_substr = case.join("EXPECT_SUBSTR");
                if expect_substr.exists() {
                    let content = fs::read_to_string(&expect_substr).expect("EXPECT_SUBSTR");
                    for needle in content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                        assert!(
                            text.contains(needle),
                            "case '{}': expected diagnostic to contain '{}', got:\n{}",
                            case.display(),
                            needle,
                            text
                        );
                    }
                }
            }
        }
    }

    for required in ["OK", "E0241", "E0242", "E0243", "E0244", "E0245"] {
        assert!(
            seen.contains(required),
            "imports matrix missing coverage for {}",
            required
        );
    }
}
