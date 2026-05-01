use std::process::Command;

fn forbidden(text: &str) {
    for word in [
        "private",
        "tesseract",
        "andromeda",
        "axiom",
        "unlock",
        "hidden",
        "self",
        "residency",
    ] {
        assert!(
            !text.to_ascii_lowercase().contains(word),
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
    forbidden(&text);
}

#[test]
fn error_hygiene_public_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_core-lab"))
        .arg("nope")
        .output()
        .expect("error output");
    let text = String::from_utf8_lossy(&output.stderr);
    forbidden(&text);
}

#[test]
fn completion_hygiene_public_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_core-lab"))
        .arg("completions")
        .output()
        .expect("completion output");
    let text = String::from_utf8_lossy(&output.stdout);
    forbidden(&text);
}
