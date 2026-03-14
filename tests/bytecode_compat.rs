use semantic_language::semcode_format::{
    header_spec_from_magic, CAP_F64_MATH, CAP_FX_VALUES, CAP_GATE_SURFACE, MAGIC0, MAGIC1, MAGIC2,
};
use semantic_language::semcode_vm::{run_semcode, RuntimeError};
use semantic_language::frontend::{
    compile_program_to_semcode, compile_program_to_semcode_with_options_debug, CompileProfile,
    OptLevel,
};
use sm_vm::run_verified_semcode;

fn first_function_code_offset(bytes: &[u8]) -> usize {
    let name_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    8 + 2 + name_len + 4
}

fn compile_cli_default_semcode(src: &str) -> Vec<u8> {
    compile_program_to_semcode_with_options_debug(src, CompileProfile::Auto, OptLevel::O0, false)
        .expect("compile")
}

#[test]
fn compat_v0_header_and_run() {
    let src = "fn main() { return; }";
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC0);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 1);
    assert_eq!(spec.capabilities & CAP_F64_MATH, 0);
    assert_ne!(spec.capabilities & CAP_GATE_SURFACE, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v1_header_and_run() {
    let src = r#"
        fn main() {
            let x: f64 = 1.0 + 2.0;
            return;
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC1);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 2);
    assert_ne!(spec.capabilities & CAP_F64_MATH, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_i32_value_path_runs_under_v0_header() {
    let src = r#"
        fn id(x: i32) -> i32 {
            return x;
        }

        fn main() {
            let x: i32 = 1;
            let y: i32 = id(2);
            if x != y { return; } else { return; }
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC0);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 1);
    assert_eq!(spec.capabilities & CAP_F64_MATH, 0);
    assert_eq!(spec.capabilities & CAP_FX_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v2_header_and_run() {
    let src = r#"
        fn id(x: fx) -> fx {
            return x;
        }

        fn main() {
            let x: fx = 1.25;
            let y: fx = id(-2.0);
            if x == x { return; } else { return; }
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC2);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 3);
    assert_ne!(spec.capabilities & CAP_FX_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_cli_o0_v1_f64_arithmetic_runs_on_verified_path() {
    let src = r#"
        fn main() {
            let y: f64 = 1.0 + 2.0;
            return;
        }
    "#;
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_cli_o0_v1_builtin_call_runs_on_verified_path() {
    let src = r#"
        fn main() {
            let y: f64 = sqrt(16.0);
            return;
        }
    "#;
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_cli_o0_complex_semantic_stress_runs_on_verified_path() {
    let src = r#"
        fn sensor_a() -> f64 {
            return 1.0 + 2.0;
        }

        fn sensor_b() -> f64 {
            return sqrt(16.0);
        }

        fn normalize(v: f64) -> f64 {
            let delta: f64 = v - 3.0;
            return abs(delta);
        }

        fn fused_signal(a: f64, b: f64) -> f64 {
            let sum: f64 = a + b;
            let energy: f64 = pow(sum, 2.0);
            return sqrt(energy);
        }

        fn risk_score(signal: f64, bias: f64) -> f64 {
            let weighted: f64 = signal / bias;
            return normalize(weighted);
        }

        fn final_decision(score: f64) -> quad {
            if score == 0.0 {
                return T;
            } else {
                return N;
            }
            return N;
        }

        fn main() {
            let a: f64 = sensor_a();
            let b: f64 = sensor_b();
            let fused: f64 = fused_signal(a, b);
            let score: f64 = risk_score(fused, 7.0);
            let decision: quad = final_decision(score);
            if decision == T { return; } else { return; }
        }
    "#;
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_unsupported_version_has_migration_hint() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_semcode(src).expect("compile");
    bytes[7] = b'9';
    let err = run_semcode(&bytes).expect_err("must fail");
    match err {
        RuntimeError::UnsupportedBytecodeVersion { found, supported } => {
            assert!(found.starts_with("SEMCODE"));
            assert!(supported.contains("SEMCODE0"));
            assert!(supported.contains("SEMCODE1"));
            assert!(supported.contains("SEMCODE2"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compat_rejects_too_many_strings_in_function_section() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_semcode(src).expect("compile");
    let code_off = first_function_code_offset(&bytes);
    bytes[code_off] = 0x01;
    bytes[code_off + 1] = 0x20; // 8193 strings
    let err = run_semcode(&bytes).expect_err("must fail");
    match err {
        RuntimeError::BadFormat(msg) => assert!(msg.contains("too many strings")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compat_rejects_too_many_debug_symbols_in_function_section() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_semcode_with_options_debug(
        src,
        CompileProfile::RustLike,
        OptLevel::O0,
        true,
    )
    .expect("compile");
    let code_off = first_function_code_offset(&bytes);
    let rel = bytes[code_off..]
        .windows(4)
        .position(|w| w == b"DBG0")
        .expect("DBG0 marker");
    let dbg_count_off = code_off + rel + 4;
    bytes[dbg_count_off] = 0x01;
    bytes[dbg_count_off + 1] = 0x20; // 8193 debug entries
    let err = run_semcode(&bytes).expect_err("must fail");
    match err {
        RuntimeError::BadFormat(msg) => assert!(msg.contains("too many debug symbols")),
        other => panic!("unexpected error: {other:?}"),
    }
}
