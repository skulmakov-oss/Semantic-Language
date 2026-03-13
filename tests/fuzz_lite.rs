use semantic_language::frontend::{lex, parse_logos_program, parse_program};

fn noisy(src: &str, seed: usize) -> String {
    let mut out = String::new();
    for (i, line) in src.lines().enumerate() {
        if (i + seed).is_multiple_of(3) {
            out.push_str("    ");
        }
        out.push_str(line);
        if (i + seed).is_multiple_of(2) {
            out.push_str("   ");
        }
        out.push('\n');
        if (i + seed).is_multiple_of(4) {
            out.push_str("# fuzz comment\n");
        }
        if (i + seed).is_multiple_of(5) {
            out.push('\n');
        }
    }
    out
}

#[test]
fn fuzz_lite_logos_no_panic() {
    let base = r#"
Entity Sensor:
    state val: quad
    prop active: bool

Law "L" [priority 1]:
    When Sensor.val == T -> System.recovery()
"#;
    for seed in 0..64usize {
        let src = noisy(base, seed);
        let _ = lex(&src);
        let _ = parse_logos_program(&src);
    }
}

#[test]
fn fuzz_lite_rust_like_no_panic() {
    let base = r#"
fn main() {
    let x: bool = true;
    if x { return; }
    return;
}
"#;
    for seed in 0..64usize {
        let src = noisy(base, seed);
        let _ = lex(&src);
        let _ = parse_program(&src);
    }
}
