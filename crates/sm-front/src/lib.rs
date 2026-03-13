#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::format;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec::Vec;

#[cfg(any(feature = "alloc", feature = "std"))]
pub mod types;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use types::{
    AstArena, BinaryOp, Expr, ExprId, FrontendError, FrontendErrorKind, Function, LogosEntity,
    LogosEntityField, LogosEntityFieldKind, LogosLaw, LogosProgram, LogosSystem, LogosWhen,
    MatchArm, Program, QuadVal, Stmt, StmtId, SymbolId, Token, TokenKind, Type, UnaryOp,
};
#[cfg(any(feature = "alloc", feature = "std"))]
pub use sm_profile::{CompatibilityMode, ParserProfile};

#[cfg(any(feature = "alloc", feature = "std"))]
pub mod lexer;
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod parser;
#[cfg(any(feature = "alloc", feature = "std"))]
mod typecheck;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use typecheck::{type_check_function, type_check_function_with_table, type_check_program};

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSig {
    pub params: Vec<Type>,
    pub ret: Type,
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub type FnTable = BTreeMap<SymbolId, FnSig>;

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEnv {
    scopes: Vec<BTreeMap<SymbolId, Type>>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl ScopeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![BTreeMap::new()],
        }
    }

    pub fn with_params(params: &[(SymbolId, Type)]) -> Self {
        let mut env = Self::new();
        for (name, ty) in params {
            env.insert(*name, *ty);
        }
        env
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let _ = self.scopes.pop();
        }
    }

    pub fn insert(&mut self, name: SymbolId, ty: Type) {
        if let Some(last) = self.scopes.last_mut() {
            last.insert(name, ty);
        }
    }

    pub fn get(&self, name: SymbolId) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(&name) {
                return Some(*t);
            }
        }
        None
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl Default for ScopeEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn build_fn_table(program: &Program) -> Result<FnTable, FrontendError> {
    let mut out = BTreeMap::new();
    for f in &program.functions {
        if out.contains_key(&f.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "duplicate function '{}'",
                    resolve_symbol_name(&program.arena, f.name)?
                ),
            });
        }
        out.insert(
            f.name,
            FnSig {
                params: f.params.iter().map(|(_, t)| *t).collect(),
                ret: f.ret,
            },
        );
    }
    Ok(out)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn builtin_sig(name: &str) -> Option<FnSig> {
    match name {
        "sin" | "cos" | "tan" | "sqrt" | "abs" => Some(FnSig {
            params: vec![Type::F64],
            ret: Type::F64,
        }),
        "pow" => Some(FnSig {
            params: vec![Type::F64, Type::F64],
            ret: Type::F64,
        }),
        _ => None,
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn resolve_symbol_name<'a>(arena: &'a AstArena, id: SymbolId) -> Result<&'a str, FrontendError> {
    arena.try_symbol_name(id).ok_or(FrontendError {
        pos: 0,
        message: format!("invalid symbol id {}", id.0),
    })
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq)]
pub enum AstBundle {
    RustLike(Program),
    Logos(LogosProgram),
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy)]
pub struct CompilePolicyView<'a> {
    pub profile: &'a ParserProfile,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<'a> CompilePolicyView<'a> {
    pub const fn new(profile: &'a ParserProfile) -> Self {
        Self { profile }
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_rustlike(input: &str) -> Result<AstBundle, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_rustlike_with_profile(input, &profile).map(AstBundle::RustLike)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_rustlike_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<AstBundle, FrontendError> {
    parser::parse_rustlike_with_profile(input, profile).map(AstBundle::RustLike)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos(input: &str) -> Result<AstBundle, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_logos_with_profile(input, &profile).map(AstBundle::Logos)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<AstBundle, FrontendError> {
    parser::parse_logos_with_profile(input, profile).map(AstBundle::Logos)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_program(input: &str) -> Result<Program, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_rustlike_with_profile(input, &profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_program_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<Program, FrontendError> {
    parser::parse_rustlike_with_profile(input, profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos_program(input: &str) -> Result<LogosProgram, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_logos_with_profile(input, &profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos_program_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<LogosProgram, FrontendError> {
    parser::parse_logos_with_profile(input, profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn lex(input: &str) -> Result<Vec<Token>, FrontendError> {
    lexer::lex_tokens(input)
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileProfile {
    Auto,
    RustLike,
    Logos,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptLevel {
    O0,
    O1,
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn parse_rustlike_bundle() {
        let src = "fn main() { return; }";
        let ast = parse_rustlike(src).expect("parse");
        match ast {
            AstBundle::RustLike(p) => assert_eq!(p.functions.len(), 1),
            AstBundle::Logos(_) => panic!("expected rustlike bundle"),
        }
    }

    #[test]
    fn parse_logos_bundle() {
        let src = r#"
Law "L" [priority 1]:
    When true -> System.recovery()
"#;
        let ast = parse_logos(src).expect("parse");
        match ast {
            AstBundle::Logos(p) => assert_eq!(p.laws.len(), 1),
            AstBundle::RustLike(_) => panic!("expected logos bundle"),
        }
    }

    #[test]
    fn lex_via_frontend_crate() {
        let toks = lexer::lex_tokens("fn main() { return; }").expect("lex");
        assert!(!toks.is_empty());
    }
}
