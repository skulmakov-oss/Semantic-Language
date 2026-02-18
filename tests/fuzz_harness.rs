use exocode_core::frontend::{lex, parse_logos_program, parse_program};

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

fn random_source(seed: &mut u64, len: usize) -> String {
    const TOKS: &[&str] = &[
        "Law", "Entity", "When", "System", "state", "prop", "fn", "let", "if", "else", "{", "}",
        "(", ")", "[", "]", "->", "=>", "==", "!=", ":", ":=", ",", ".", "true", "false", "T",
        "S", "N", "F", "quad", "bool", "i32", "f64", "\"x\"", "123", "1.25", "\n", "    ", "#c\n",
        "//c\n",
    ];
    let mut out = String::new();
    for _ in 0..len {
        let idx = (lcg(seed) as usize) % TOKS.len();
        out.push_str(TOKS[idx]);
        if (lcg(seed) & 3) == 0 {
            out.push(' ');
        }
    }
    out
}

#[test]
fn fuzz_harness_no_panic_on_random_streams() {
    let mut seed = 0xC0DEC0DEu64;
    for _ in 0..400usize {
        let src = random_source(&mut seed, 120);
        let r = std::panic::catch_unwind(|| {
            let _ = lex(&src);
            let _ = parse_program(&src);
            let _ = parse_logos_program(&src);
        });
        assert!(r.is_ok(), "panic on fuzz input: {}", src);
    }
}
