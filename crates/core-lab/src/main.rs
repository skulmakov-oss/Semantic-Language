use semantic_core_backend::{detect_backend_caps, BackendKind};
use semantic_core_bench::{format_caps_report, run_benchmark};
use semantic_core_capsule::{CoreCapsule, CoreConfig, CoreStatus};
use semantic_core_exec::{CoreProgram, CoreResultDigest};

const CORE_PROGRAM_FILE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CoreProgramFile {
    format_version: u32,
    config: Option<CoreConfig>,
    program: CoreProgram,
}

fn main() {
    match run_cli(std::env::args().skip(1).collect()) {
        Ok(text) => {
            if !text.is_empty() {
                println!("{text}");
            }
        }
        Err(text) => {
            eprintln!("{text}");
            std::process::exit(2);
        }
    }
}

fn run_cli(args: Vec<String>) -> Result<String, String> {
    if args.is_empty() {
        return Ok(help_text());
    }
    match args[0].as_str() {
        "--help" | "help" => Ok(help_text()),
        "run" => {
            let path = args.get(1).ok_or_else(help_text)?;
            let file = load_program_file(path)?;
            let capsule = CoreCapsule::new(file.config.unwrap_or_else(CoreConfig::default));
            let result = capsule.run(&file.program).map_err(format_core_error)?;
            let digest = CoreResultDigest::from_result(&result);
            let status = match result.status {
                CoreStatus::Returned => "returned".to_string(),
                CoreStatus::Trapped(trap) => format!("trapped({trap:?})"),
            };
            Ok(format!(
                "status: {status}\nvalue: {:?}\nfuel_used: {}\ndigest: {:016x}",
                result.return_value, result.fuel_used, digest.0
            ))
        }
        "validate" => {
            let path = args.get(1).ok_or_else(help_text)?;
            let file = load_program_file(path)?;
            let capsule = CoreCapsule::new(file.config.unwrap_or_else(CoreConfig::default));
            capsule.validate(&file.program).map_err(format_core_error)?;
            Ok("validation: ok".to_string())
        }
        "caps" => Ok(format_caps_report(BackendKind::Auto, detect_backend_caps())),
        "bench" => {
            let kind = args.get(1).ok_or_else(help_text)?;
            run_benchmark(kind)
        }
        "completions" => Ok("run\nvalidate\ncaps\nbench\nhelp\ncompletions".to_string()),
        other => Err(format!("unknown command '{other}'\n\n{}", help_text())),
    }
}

fn help_text() -> String {
    "core-lab commands:\n  run <file>\n  validate <file>\n  caps\n  bench <quad-reg|tile|exec|all|caps>\n  help\n  completions".to_string()
}

fn format_core_error(err: impl std::fmt::Debug) -> String {
    format!("{err:?}")
}

fn load_program_file(path: &str) -> Result<CoreProgramFile, String> {
    let text = std::fs::read_to_string(path).map_err(format_core_error)?;
    let file: CoreProgramFile = serde_json::from_str(&text).map_err(format_core_error)?;
    if file.format_version != CORE_PROGRAM_FILE_FORMAT_VERSION {
        return Err(format!(
            "unsupported core program file format version {}; expected {}",
            file.format_version, CORE_PROGRAM_FILE_FORMAT_VERSION
        ));
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_file(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("core-lab-{name}-{}.json", std::process::id()));
        path
    }

    #[test]
    fn rejects_unsupported_program_file_format_version() {
        let path = temp_file("format-version");
        std::fs::write(
            &path,
            r#"{
  "format_version": 2,
  "program": {
    "functions": [
      {
        "name_id": 0,
        "regs": 1,
        "instrs": [{ "op": "ret", "src": 0 }]
      }
    ],
    "entry": 0
  }
}"#,
        )
        .unwrap();

        let err = load_program_file(path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("unsupported core program file format version"));

        let _ = std::fs::remove_file(path);
    }
}
