#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "std")]
mod frontend {
    pub use exo_frontend::{
        build_fn_table, builtin_sig, parse_logos_program, parse_program, resolve_symbol_name,
        type_check_function_with_table, type_check_program, AstArena, BinaryOp, CompileProfile,
        Expr, ExprId, FnTable, FrontendError, Function, LogosProgram, OptLevel,
        QuadVal, ScopeEnv, Stmt, StmtId, SymbolId, Type, UnaryOp,
    };
}

#[cfg(feature = "std")]
#[allow(dead_code)]
mod local_format;

#[cfg(feature = "std")]
mod exobyte_format {
    pub use crate::local_format::{
        write_f64_le, write_i32_le, write_u16_le, write_u32_le, Opcode, MAGIC0, MAGIC1,
    };
}

#[cfg(feature = "std")]
use frontend::*;

#[cfg(feature = "std")]
mod legacy_lowering;
#[cfg(feature = "std")]
pub mod passes;

#[cfg(feature = "std")]
pub use frontend::{CompileProfile, OptLevel};
#[cfg(feature = "std")]
pub use legacy_lowering::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn exo_ir_smoke_compile_program_to_ir() {
        let src = "fn main() { return; }";
        let ir = compile_program_to_ir(src).expect("ir compile");
        assert_eq!(ir.len(), 1);
        assert_eq!(ir[0].name, "main");
    }
}
