use exo_cli::CliPipeline;
use exo_ir::{CompileProfile, OptLevel};
use exo_vm::{disasm_exobyte, run_exobyte};
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage());
    }
    match args[0].as_str() {
        "check" => cmd_check(&args[1..]),
        "compile" => cmd_compile(&args[1..]),
        "run" => cmd_run(&args[1..]),
        "disasm" => cmd_disasm(&args[1..]),
        "explain" => cmd_explain(&args[1..]),
        "help" | "--help" | "-h" => Err(usage()),
        other => Err(format!("unknown command '{}'\n\n{}", other, usage())),
    }
}

fn cmd_check(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: exo-cli check <input.exo>".to_string());
    }
    let src = fs::read_to_string(&args[0]).map_err(|e| format!("failed to read '{}': {}", args[0], e))?;
    let report = CliPipeline::semantic_check_source(&src)?;
    for w in &report.warnings {
        eprintln!("{}", w.rendered.trim_end());
    }
    println!(
        "semantic check passed: {} warning(s), {} scheduled law(s)",
        report.warnings.len(),
        report.scheduled_laws.len()
    );
    Ok(())
}

fn cmd_compile(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("usage: exo-cli compile <input.exo> -o <out.exb>".to_string());
    }
    let input = args[0].as_str();
    let mut out: Option<&str> = None;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--out" => {
                i += 1;
                out = args.get(i).map(|s| s.as_str());
            }
            other => return Err(format!("unknown flag '{}'", other)),
        }
        i += 1;
    }
    let out = out.ok_or_else(|| "missing -o <out.exb>".to_string())?;
    let src = fs::read_to_string(input).map_err(|e| format!("failed to read '{}': {}", input, e))?;
    let bytes = CliPipeline::compile_source(&src, CompileProfile::Auto, OptLevel::O0, false)?;
    fs::write(out, &bytes).map_err(|e| format!("failed to write '{}': {}", out, e))?;
    println!("compiled '{}' -> '{}' ({} bytes)", input, out, bytes.len());
    Ok(())
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: exo-cli run <input.exo>".to_string());
    }
    let src = fs::read_to_string(&args[0]).map_err(|e| format!("failed to read '{}': {}", args[0], e))?;
    let bytes = CliPipeline::compile_source(&src, CompileProfile::Auto, OptLevel::O0, false)?;
    run_exobyte(&bytes).map_err(|e| e.to_string())
}

fn cmd_disasm(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: exo-cli disasm <input.exb>".to_string());
    }
    let bytes = fs::read(&args[0]).map_err(|e| format!("failed to read '{}': {}", args[0], e))?;
    let text = disasm_exobyte(&bytes).map_err(|e| e.to_string())?;
    print!("{text}");
    Ok(())
}

fn cmd_explain(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: exo-cli explain <code>".to_string());
    }
    match CliPipeline::explain(&args[0]) {
        Some(text) => {
            println!("{}: {}", args[0].to_ascii_uppercase(), text);
            Ok(())
        }
        None => Err(format!("unknown diagnostic code '{}'", args[0])),
    }
}

fn usage() -> String {
    [
        "exo-cli",
        "  exo-cli check <input.exo>",
        "  exo-cli compile <input.exo> -o <out.exb>",
        "  exo-cli run <input.exo>",
        "  exo-cli disasm <input.exb>",
        "  exo-cli explain <code>",
    ]
    .join("\n")
}
