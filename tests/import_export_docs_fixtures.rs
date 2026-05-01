use sm_sema::{check_file_with_provider, ModuleProvider};
use std::fs;
use std::path::{Path, PathBuf};

struct FsProvider;

impl ModuleProvider for FsProvider {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
        fs::read(module_id).map_err(|e| e.to_string())
    }

    fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String> {
        Ok(resolve_fixture_import(importer_module_id, spec))
    }
}

fn resolve_fixture_import(importer_module_id: &str, spec: &str) -> String {
    let importer = Path::new(importer_module_id);
    let base = importer.parent().unwrap_or_else(|| Path::new("."));
    let mut spec_path = PathBuf::from(spec);
    if spec_path.extension().is_none() {
        spec_path.set_extension("exo");
    }
    let joined = if spec_path.is_absolute() {
        spec_path
    } else {
        base.join(spec_path)
    };
    joined.to_string_lossy().replace('\\', "/")
}

fn fixture_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let rd = fs::read_dir(root).expect("read fixture root");
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn imports_docs_fixtures() {
    let root = Path::new("tests/fixtures/imports");
    let provider = FsProvider;
    for case in fixture_dirs(root) {
        let expect = fs::read_to_string(case.join("EXPECT"))
            .expect("EXPECT")
            .trim()
            .to_string();
        let entry = case.join("main.sm").canonicalize().expect("main.sm");
        let res = check_file_with_provider(&entry, &provider);
        if expect == "OK" {
            if let Err(e) = res {
                panic!("case '{}' expected OK, got: {}", case.display(), e);
            }
            continue;
        }
        let code = expect.strip_prefix("ERR ").expect("ERR format");
        match res {
            Ok(_) => panic!("case '{}' expected {}, got OK", case.display(), code),
            Err(e) => {
                let text = e.to_string();
                assert!(
                    text.contains(&format!("Error [{}]", code)),
                    "case '{}': expected code {}, got:\n{}",
                    case.display(),
                    code,
                    text
                );
            }
        }
    }
}
