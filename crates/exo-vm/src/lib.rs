#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod exobyte_format {
    pub use exo_emit::{
        header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8,
        read_utf8, supported_headers, ExobyteFormatError, ExobyteHeaderSpec, Opcode,
    };
}

#[cfg(feature = "std")]
mod frontend {
    pub use exo_frontend::QuadVal;
    #[cfg(test)]
    pub use exo_emit::compile_program_to_exobyte;
}

#[cfg(feature = "std")]
mod exobyte_vm;

#[cfg(feature = "std")]
pub use exobyte_vm::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use exo_emit::compile_program_to_exobyte;

    #[test]
    fn exo_vm_smoke_run() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_exobyte(src).expect("compile");
        run_exobyte(&bytes).expect("run");
        let dis = disasm_exobyte(&bytes).expect("disasm");
        assert!(dis.contains("main"));
    }
}
