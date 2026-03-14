#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod semcode_format;
#[cfg(feature = "std")]
pub use semcode_format::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8,
    read_utf8, supported_headers, write_f64_le, write_i32_le, write_u16_le, write_u32_le,
    CAP_DEBUG_SYMBOLS, CAP_F64_MATH, CAP_FX_VALUES, CAP_GATE_SURFACE, SemcodeFormatError,
    SemcodeHeaderSpec, Opcode, HEADER_V0, HEADER_V1, HEADER_V2, MAGIC0, MAGIC1, MAGIC2,
};
#[cfg(feature = "std")]
pub use sm_ir::{
    CompileProfile, OptLevel,
    compile_program_to_semcode, compile_program_to_semcode_with_options,
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode,
};

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn sm_emit_smoke_compile_to_semcode() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("emit");
        assert_eq!(&bytes[0..8], &MAGIC0);
    }
}
