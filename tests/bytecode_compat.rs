use exocode_core::exobyte_format::{
    header_spec_from_magic, CAP_F64_MATH, CAP_GATE_SURFACE, MAGIC0, MAGIC1,
};
use exocode_core::exobyte_vm::{run_exobyte, RuntimeError};
use exocode_core::frontend::{
    compile_program_to_exobyte, compile_program_to_exobyte_with_options_debug, CompileProfile,
    OptLevel,
};

fn first_function_code_offset(bytes: &[u8]) -> usize {
    let name_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    8 + 2 + name_len + 4
}

#[test]
fn compat_v0_header_and_run() {
    let src = "fn main() { return; }";
    let bytes = compile_program_to_exobyte(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC0);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 1);
    assert_eq!(spec.capabilities & CAP_F64_MATH, 0);
    assert_ne!(spec.capabilities & CAP_GATE_SURFACE, 0);
    run_exobyte(&bytes).expect("run");
}

#[test]
fn compat_v1_header_and_run() {
    let src = r#"
        fn main() {
            let x: f64 = 1.0 + 2.0;
            return;
        }
    "#;
    let bytes = compile_program_to_exobyte(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC1);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 2);
    assert_ne!(spec.capabilities & CAP_F64_MATH, 0);
    run_exobyte(&bytes).expect("run");
}

#[test]
fn compat_unsupported_version_has_migration_hint() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_exobyte(src).expect("compile");
    bytes[7] = b'9';
    let err = run_exobyte(&bytes).expect_err("must fail");
    match err {
        RuntimeError::UnsupportedBytecodeVersion { found, supported } => {
            assert!(found.starts_with("EXOBYTE"));
            assert!(supported.contains("EXOBYTE0"));
            assert!(supported.contains("EXOBYTE1"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compat_rejects_too_many_strings_in_function_section() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_exobyte(src).expect("compile");
    let code_off = first_function_code_offset(&bytes);
    bytes[code_off] = 0x01;
    bytes[code_off + 1] = 0x20; // 8193 strings
    let err = run_exobyte(&bytes).expect_err("must fail");
    match err {
        RuntimeError::BadFormat(msg) => assert!(msg.contains("too many strings")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compat_rejects_too_many_debug_symbols_in_function_section() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_exobyte_with_options_debug(
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
    let err = run_exobyte(&bytes).expect_err("must fail");
    match err {
        RuntimeError::BadFormat(msg) => assert!(msg.contains("too many debug symbols")),
        other => panic!("unexpected error: {other:?}"),
    }
}
