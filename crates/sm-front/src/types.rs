use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ton618_core::{SigTable, SourceMark};
pub use ton618_core::{ExprId, StmtId, SymbolId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Quad,
    QVec(usize),
    Bool,
    I32,
    U32,
    Fx,
    F64,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadVal {
    N,
    F,
    T,
    S,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Pos,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    AndAnd,
    OrOr,
    Implies,
    Eq,
    Ne,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    QuadLiteral(QuadVal),
    BoolLiteral(bool),
    Num(i64),
    Float(f64),
    Var(SymbolId),
    Call(SymbolId, Vec<ExprId>),
    Unary(UnaryOp, ExprId),
    Binary(ExprId, BinaryOp, ExprId),
    Block(BlockExpr),
    If(IfExpr),
    Match(MatchExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: SymbolId,
        ty: Option<Type>,
        value: ExprId,
    },
    Guard {
        condition: ExprId,
        else_return: Option<ExprId>,
    },
    If {
        condition: ExprId,
        then_block: Vec<StmtId>,
        else_block: Vec<StmtId>,
    },
    Match {
        scrutinee: ExprId,
        arms: Vec<MatchArm>,
        default: Vec<StmtId>,
    },
    Return(Option<ExprId>),
    Expr(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpr {
    pub statements: Vec<StmtId>,
    pub tail: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: ExprId,
    pub then_block: BlockExpr,
    pub else_block: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub scrutinee: ExprId,
    pub arms: Vec<MatchExprArm>,
    pub default: Option<BlockExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExprArm {
    pub pat: QuadVal,
    pub guard: Option<ExprId>,
    pub block: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pat: QuadVal,
    pub guard: Option<ExprId>,
    pub block: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: SymbolId,
    pub params: Vec<(SymbolId, Type)>,
    pub ret: Type,
    pub body: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub arena: AstArena,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendErrorKind {
    Syntax,
    PolicyViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    pub pos: usize,
    pub message: String,
}

impl FrontendError {
    pub fn syntax(pos: usize, message: impl Into<String>) -> Self {
        Self {
            pos,
            message: message.into(),
        }
    }

    pub fn policy_violation(pos: usize, message: impl Into<String>) -> Self {
        Self {
            pos,
            message: format!("policy violation: {}", message.into()),
        }
    }

    pub fn kind(&self) -> FrontendErrorKind {
        if self.message.starts_with("policy violation:") {
            FrontendErrorKind::PolicyViolation
        } else {
            FrontendErrorKind::Syntax
        }
    }
}

impl ::core::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "at {}: {}", self.pos, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FrontendError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub pos: usize,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    KwFn,
    KwLet,
    KwGuard,
    KwIf,
    KwElse,
    KwReturn,
    KwMatch,
    KwTrue,
    KwFalse,
    KwSystem,
    KwEntity,
    KwLaw,
    KwWhen,
    KwPulse,
    KwProfile,
    KwImport,
    TyQuad,
    TyBool,
    TyI32,
    TyU32,
    TyFx,
    TyF64,
    QuadN,
    QuadF,
    QuadT,
    QuadS,
    Ident,
    Num,
    String,
    AndAnd,
    OrOr,
    PipeForward,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    Arrow,
    Implies,
    FatArrow,
    EqEq,
    Ne,
    Assign,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semi,
    Comma,
    Colon,
    Dot,
    LBracket,
    RBracket,
    Underscore,
    Indent,
    Dedent,
    Newline,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosSystem {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosEntityField {
    pub kind: LogosEntityFieldKind,
    pub name: String,
    pub ty: Type,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogosEntityFieldKind {
    State,
    Prop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosEntity {
    pub name: String,
    pub fields: Vec<LogosEntityField>,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosWhen {
    pub condition: String,
    pub effect: String,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosLaw {
    pub name: String,
    pub priority: u32,
    pub whens: Vec<LogosWhen>,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogosProgram {
    pub system: Option<LogosSystem>,
    pub entities: Vec<LogosEntity>,
    pub laws: Vec<LogosLaw>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AstArena {
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    sigtable: SigTable,
    pub(crate) symbol_to_id: BTreeMap<String, SymbolId>,
}

impl AstArena {
    pub fn alloc_expr(&mut self, expr: Expr) -> ExprId {
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(expr);
        id
    }

    pub fn alloc_stmt(&mut self, stmt: Stmt) -> StmtId {
        let id = StmtId(self.stmts.len() as u32);
        self.stmts.push(stmt);
        id
    }

    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }

    pub fn stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.0 as usize]
    }

    pub fn intern_symbol(&mut self, name: &str) -> SymbolId {
        if let Some(id) = self.symbol_to_id.get(name) {
            return *id;
        }
        let id = self.sigtable.intern(name);
        self.symbol_to_id.insert(name.to_string(), id);
        id
    }

    pub fn expr_count(&self) -> usize {
        self.exprs.len()
    }

    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    pub fn symbol_count(&self) -> usize {
        self.sigtable.len()
    }

    pub fn symbol_name(&self, id: SymbolId) -> &str {
        self.try_symbol_name(id).unwrap_or("<invalid-symbol>")
    }

    pub fn try_symbol_name(&self, id: SymbolId) -> Option<&str> {
        self.sigtable.resolve(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_invariants_append_only_and_stable_ids() {
        let mut arena = AstArena::default();

        let e0 = arena.alloc_expr(Expr::QuadLiteral(QuadVal::T));
        let e1 = arena.alloc_expr(Expr::BoolLiteral(true));
        assert_eq!(e0.0, 0);
        assert_eq!(e1.0, 1);
        assert_eq!(arena.expr_count(), 2);
        assert_eq!(arena.expr(e0), &Expr::QuadLiteral(QuadVal::T));

        let s0 = arena.alloc_stmt(Stmt::Expr(e0));
        let s1 = arena.alloc_stmt(Stmt::Expr(e1));
        assert_eq!(s0.0, 0);
        assert_eq!(s1.0, 1);
        assert_eq!(arena.stmt_count(), 2);
        assert_eq!(arena.stmt(s0), &Stmt::Expr(e0));

        // Additional allocations must not change earlier IDs or referenced nodes.
        let _e2 = arena.alloc_expr(Expr::Num(42));
        let _s2 = arena.alloc_stmt(Stmt::Return(None));
        assert_eq!(arena.expr(e0), &Expr::QuadLiteral(QuadVal::T));
        assert_eq!(arena.stmt(s0), &Stmt::Expr(e0));
    }
}
