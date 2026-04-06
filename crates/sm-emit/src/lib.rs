#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub use sm_ir::semcode_format::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8,
    supported_headers, write_f64_le, write_i32_le, write_u16_le, write_u32_le, Opcode,
    SemcodeFormatError, SemcodeHeaderSpec, CAP_CLOCK_READ, CAP_DEBUG_SYMBOLS, CAP_EVENT_POST,
    CAP_F64_MATH, CAP_FX_MATH, CAP_FX_VALUES, CAP_GATE_SURFACE, CAP_SEQUENCE_VALUES,
    CAP_STATE_QUERY, CAP_STATE_UPDATE, CAP_TEXT_VALUES, CAP_CLOSURE_VALUES, HEADER_V0, HEADER_V1,
    HEADER_V2, HEADER_V3, HEADER_V4, HEADER_V5, HEADER_V6, HEADER_V7, HEADER_V8, HEADER_V9,
    HEADER_V10, MAGIC0, MAGIC1, MAGIC2, MAGIC3, MAGIC4, MAGIC5, MAGIC6, MAGIC7, MAGIC8, MAGIC9,
    MAGIC10,
};
#[cfg(feature = "std")]
pub use sm_ir::{
    compile_program_to_semcode, compile_program_to_semcode_with_options,
    compile_program_to_semcode_with_options_debug, emit_ir_to_semcode, CompileProfile, OptLevel,
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
