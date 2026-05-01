use std::process::Command;

fn reserved_labels() -> [String; 8] {
    [
        ["pri", "vate"].concat(),
        ["tes", "ser", "act"].concat(),
        ["andr", "omeda"].concat(),
        ["ax", "iom"].concat(),
        ["un", "lock"].concat(),
        ["hid", "den"].concat(),
        ["se", "lf"].concat(),
        ["resi", "dency"].concat(),
    ]
}

fn deny_output_labels(text: &str) {
    for word in reserved_labels() {
        assert!(
            !text.to_ascii_lowercase().contains(&word),
            "unexpected word '{word}' in output: {text}"
        );
    }
}

#[test]
fn help_hygiene_public_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_core-lab"))
        .arg("--help")
        .output()
        .expect("help output");
    let text = String::from_utf8_lossy(&output.stdout);
    deny_output_labels(&text);
}

#[test]
fn error_hygiene_public_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_core-lab"))
        .arg("nope")
        .output()
        .expect("error output");
    let text = String::from_utf8_lossy(&output.stderr);
    deny_output_labels(&text);
}

#[test]
fn completion_hygiene_public_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_core-lab"))
        .arg("completions")
        .output()
        .expect("completion output");
    let text = String::from_utf8_lossy(&output.stdout);
    deny_output_labels(&text);
}
