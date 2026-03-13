use sm_vm::{disasm_semcode, run_verified_semcode};
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage());
    }

    match args[0].as_str() {
        "run" => cmd_run(&args[1..]),
        "disasm" => cmd_disasm(&args[1..]),
        "help" | "--help" | "-h" => Err(usage()),
        other => Err(format!("unknown command '{}'\n\n{}", other, usage())),
    }
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: svm run <input.smc>".to_string());
    }
    let input = &args[0];
    let bytes = fs::read(input).map_err(|e| format!("failed to read '{}': {}", input, e))?;
    run_verified_semcode(&bytes).map_err(|e| e.to_string())
}

fn cmd_disasm(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: svm disasm <input.smc>".to_string());
    }
    let input = &args[0];
    let bytes = fs::read(input).map_err(|e| format!("failed to read '{}': {}", input, e))?;
    let text = disasm_semcode(&bytes).map_err(|e| e.to_string())?;
    print!("{text}");
    Ok(())
}

fn usage() -> String {
    [
        "Semantic Language VM",
        "  svm run <input.smc>",
        "  svm disasm <input.smc>",
    ]
    .join("\n")
}
