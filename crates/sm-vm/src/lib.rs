#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod semcode_format {
    pub use sm_emit::{
        header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8,
        read_utf8, supported_headers, Opcode, SemcodeFormatError, SemcodeHeaderSpec,
        OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE,
        OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX,
        OWNERSHIP_SECTION_TAG,
    };
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadVal {
    N,
    F,
    T,
    S,
}

#[cfg(feature = "std")]
mod semcode_vm;

#[cfg(feature = "std")]
pub use semcode_vm::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_emit::compile_program_to_semcode;

    #[test]
    fn sm_vm_smoke_run() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_verified_semcode(&bytes).expect("run");
        let dis = disasm_semcode(&bytes).expect("disasm");
        assert!(dis.contains("main"));
    }
}
