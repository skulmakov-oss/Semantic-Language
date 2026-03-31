use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub use ton618_core::{ExprId, StmtId, SymbolId};
use ton618_core::{SigTable, SourceMark};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Quad,
    QVec(usize),
    Bool,
    I32,
    U32,
    Fx,
    F64,
    Measured(Box<Type>, SymbolId),
    RangeI32,
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Record(SymbolId),
    Adt(SymbolId),
    Unit,
}

impl Type {
    pub fn erase_units(&self) -> Type {
        match self {
            Type::Measured(base, _) => base.erase_units(),
            Type::Tuple(items) => Type::Tuple(items.iter().map(Type::erase_units).collect()),
            Type::Option(item) => Type::Option(Box::new(item.erase_units())),
            Type::Result(ok_ty, err_ty) => Type::Result(
                Box::new(ok_ty.erase_units()),
                Box::new(err_ty.erase_units()),
            ),
            _ => self.clone(),
        }
    }

    pub fn measured_parts(&self) -> Option<(&Type, SymbolId)> {
        match self {
            Type::Measured(base, unit) => Some((base.as_ref(), *unit)),
            _ => None,
        }
    }

    pub fn is_core_numeric_scalar(&self) -> bool {
        matches!(self, Type::I32 | Type::U32 | Type::Fx | Type::F64)
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericLiteral {
    I32(i32),
    U32(u32),
    F64(f64),
    Fx(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<SymbolId>,
    pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordInitField {
    pub name: SymbolId,
    pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordLiteralExpr {
    pub name: SymbolId,
    pub fields: Vec<RecordInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordFieldExpr {
    pub base: ExprId,
    pub field: SymbolId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordUpdateExpr {
    pub base: ExprId,
    pub fields: Vec<RecordInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdtCtorExpr {
    pub adt_name: SymbolId,
    pub variant_name: SymbolId,
    pub payload: Vec<ExprId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdtPatternItem {
    Bind(SymbolId),
    Discard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdtMatchPattern {
    pub adt_name: SymbolId,
    pub variant_name: SymbolId,
    pub items: Vec<AdtPatternItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordPatternItem {
    pub field: SymbolId,
    pub target: RecordPatternTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordPatternTarget {
    Bind(SymbolId),
    Discard,
    QuadLiteral(QuadVal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    QuadLiteral(QuadVal),
    BoolLiteral(bool),
    NumericLiteral(NumericLiteral),
    Range(RangeExpr),
    Tuple(Vec<ExprId>),
    RecordLiteral(RecordLiteralExpr),
    RecordField(RecordFieldExpr),
    RecordUpdate(RecordUpdateExpr),
    AdtCtor(AdtCtorExpr),
    Var(SymbolId),
    Call(SymbolId, Vec<CallArg>),
    Unary(UnaryOp, ExprId),
    Binary(ExprId, BinaryOp, ExprId),
    Block(BlockExpr),
    If(IfExpr),
    Match(MatchExpr),
    Loop(LoopExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Const {
        name: SymbolId,
        ty: Option<Type>,
        value: ExprId,
    },
    Let {
        name: SymbolId,
        ty: Option<Type>,
        value: ExprId,
    },
    LetTuple {
        items: Vec<Option<SymbolId>>,
        ty: Option<Type>,
        value: ExprId,
    },
    LetRecord {
        record_name: SymbolId,
        items: Vec<RecordPatternItem>,
        value: ExprId,
    },
    LetElseRecord {
        record_name: SymbolId,
        items: Vec<RecordPatternItem>,
        value: ExprId,
        else_return: Option<ExprId>,
    },
    LetElseTuple {
        items: Vec<TuplePatternItem>,
        ty: Option<Type>,
        value: ExprId,
        else_return: Option<ExprId>,
    },
    Discard {
        ty: Option<Type>,
        value: ExprId,
    },
    Assign {
        name: SymbolId,
        value: ExprId,
    },
    AssignTuple {
        items: Vec<Option<SymbolId>>,
        value: ExprId,
    },
    ForRange {
        name: SymbolId,
        range: ExprId,
        body: Vec<StmtId>,
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
    Break(ExprId),
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
    pub pat: MatchPattern,
    pub guard: Option<ExprId>,
    pub block: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpr {
    pub body: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub start: ExprId,
    pub end: ExprId,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuplePatternItem {
    Bind(SymbolId),
    Discard,
    QuadLiteral(QuadVal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Quad(QuadVal),
    Adt(AdtMatchPattern),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pat: MatchPattern,
    pub guard: Option<ExprId>,
    pub block: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: SymbolId,
    pub params: Vec<(SymbolId, Type)>,
    pub param_defaults: Vec<Option<ExprId>>,
    pub requires: Vec<ExprId>,
    pub ensures: Vec<ExprId>,
    pub invariants: Vec<ExprId>,
    pub ret: Type,
    pub body: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    pub name: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDecl {
    pub name: SymbolId,
    pub fields: Vec<RecordField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtVariant {
    pub name: SymbolId,
    pub payload: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtDecl {
    pub name: SymbolId,
    pub variants: Vec<AdtVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaField {
    pub name: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pub name: SymbolId,
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaRole {
    Config,
    Api,
    Wire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion {
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaShape {
    Record(Vec<SchemaField>),
    TaggedUnion(Vec<SchemaVariant>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDecl {
    pub name: SymbolId,
    pub role: Option<SchemaRole>,
    pub version: Option<SchemaVersion>,
    pub shape: SchemaShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationFieldPlan {
    pub name: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationVariantPlan {
    pub name: SymbolId,
    pub fields: Vec<ValidationFieldPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationCheck {
    RequiredField { field: SymbolId },
    FieldType { field: SymbolId, ty: Type },
    TaggedUnionBranch { variant: SymbolId },
    TaggedUnionBranchRequiredField { variant: SymbolId, field: SymbolId },
    TaggedUnionBranchFieldType { variant: SymbolId, field: SymbolId, ty: Type },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationShapePlan {
    Record(Vec<ValidationFieldPlan>),
    TaggedUnion(Vec<ValidationVariantPlan>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationPlan {
    pub schema_name: SymbolId,
    pub role: Option<SchemaRole>,
    pub shape: ValidationShapePlan,
    pub checks: Vec<ValidationCheck>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub arena: AstArena,
    pub adts: Vec<AdtDecl>,
    pub records: Vec<RecordDecl>,
    pub schemas: Vec<SchemaDecl>,
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
    KwRequires,
    KwEnsures,
    KwInvariant,
    KwRecord,
    KwSchema,
    KwEnum,
    KwConst,
    KwLet,
    KwFor,
    KwIn,
    KwGuard,
    KwIf,
    KwElse,
    KwLoop,
    KwBreak,
    KwWhere,
    KwWith,
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
    AndAndAssign,
    OrOrAssign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
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
    DotDot,
    DotDotEq,
    Assign,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semi,
    Comma,
    Colon,
    PathSep,
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
        let _e2 = arena.alloc_expr(Expr::NumericLiteral(NumericLiteral::I32(42)));
        let _s2 = arena.alloc_stmt(Stmt::Return(None));
        assert_eq!(arena.expr(e0), &Expr::QuadLiteral(QuadVal::T));
        assert_eq!(arena.stmt(s0), &Stmt::Expr(e0));
    }
}
