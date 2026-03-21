use crate::lexer::lex_tokens;
use crate::types::{
    AstArena, BinaryOp, BlockExpr, Expr, ExprId, FrontendError, Function, IfExpr, LogosEntity,
    LogosEntityField, LogosEntityFieldKind, LogosLaw, LogosProgram, LogosSystem, LogosWhen,
    MatchArm, MatchExpr, MatchExprArm, Program, QuadVal, Stmt, StmtId, SymbolId, Token, TokenKind,
    Type, UnaryOp,
};
use crate::CompilePolicyView;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use sm_profile::{CompatibilityMode, ParserProfile};
use ton618_core::diagnostics::{
    format_multiple_parser_errors, format_parser_error_at_input, suggest_closest_case_insensitive,
};
use ton618_core::SourceMap;

pub fn parse_rustlike_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<Program, FrontendError> {
    let tokens = lex_tokens(input)?;
    let mut p = Parser {
        tokens,
        idx: 0,
        source: input.to_string(),
        arena: AstArena::default(),
        policy: CompilePolicyView::new(profile),
    };
    p.parse_program()
}

pub fn parse_logos_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<LogosProgram, FrontendError> {
    let tokens = lex_tokens(input)?;
    let mut p = Parser {
        tokens,
        idx: 0,
        source: input.to_string(),
        arena: AstArena::default(),
        policy: CompilePolicyView::new(profile),
    };
    p.parse_logos_program()
}

struct Parser<'a> {
    tokens: Vec<Token>,
    idx: usize,
    source: String,
    arena: AstArena,
    policy: CompilePolicyView<'a>,
}

impl<'a> Parser<'a> {
    fn parse_program(&mut self) -> Result<Program, FrontendError> {
        let mut functions = Vec::new();
        loop {
            let i = self.next_non_layout_idx();
            if i >= self.tokens.len() {
                break;
            }
            self.idx = i;
            functions.push(self.parse_function()?);
        }
        Ok(Program {
            arena: ::core::mem::take(&mut self.arena),
            functions,
        })
    }

    fn parse_function(&mut self) -> Result<Function, FrontendError> {
        self.expect(TokenKind::KwFn, "expected 'fn'")?;
        let name = self.expect_symbol()?;
        self.expect(TokenKind::LParen, "expected '('")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let pname = self.expect_symbol()?;
                self.expect(TokenKind::Colon, "expected ':'")?;
                let pty = self.parse_type()?;
                params.push((pname, pty));
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen, "expected ')'")?;
        let ret = if self.eat(TokenKind::Implies) {
            self.parse_type()?
        } else {
            Type::Unit
        };
        let body = if self.eat(TokenKind::Assign) {
            let expr = self.parse_expr()?;
            self.expect(
                TokenKind::Semi,
                "expected ';' after expression-bodied function",
            )?;
            vec![self.arena.alloc_stmt(Stmt::Return(Some(expr)))]
        } else {
            self.parse_block()?
        };
        Ok(Function {
            name,
            params,
            ret,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<StmtId>, FrontendError> {
        self.expect(TokenKind::LBrace, "expected '{'")?;
        let mut out = Vec::new();
        while !self.check(TokenKind::RBrace) {
            out.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace, "expected '}'")?;
        Ok(out)
    }

    fn parse_stmt(&mut self) -> Result<StmtId, FrontendError> {
        if self.eat(TokenKind::KwLet) {
            if self.eat(TokenKind::Underscore) {
                let ty = if self.eat(TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(TokenKind::Assign, "expected '='")?;
                let value = self.parse_expr()?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                return Ok(self.arena.alloc_stmt(Stmt::Discard { ty, value }));
            }
            let name = self.expect_symbol()?;
            let ty = if self.eat(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(TokenKind::Assign, "expected '='")?;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Let { name, ty, value }));
        }
        if self.check(TokenKind::Ident) {
            if let Some(op) = self.peek_compound_assign_op() {
                let name = self.expect_symbol()?;
                let _ = self.advance();
                let rhs = self.parse_expr()?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                let lhs = self.arena.alloc_expr(Expr::Var(name));
                let value = self.arena.alloc_expr(Expr::Binary(lhs, op, rhs));
                return Ok(self.arena.alloc_stmt(Stmt::Assign { name, value }));
            }
        }
        if self.eat(TokenKind::KwGuard) {
            let condition = self.parse_expr()?;
            if !self.eat(TokenKind::KwElse) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "guard clause requires else return branch".to_string(),
                });
            }
            if !self.eat(TokenKind::KwReturn) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "guard clause currently supports only else return".to_string(),
                });
            }
            let else_return = if self.eat(TokenKind::Semi) {
                None
            } else {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                Some(expr)
            };
            return Ok(self.arena.alloc_stmt(Stmt::Guard {
                condition,
                else_return,
            }));
        }
        if self.eat(TokenKind::KwIf) {
            let condition = self.parse_expr()?;
            let then_block = self.parse_block()?;
            let else_block = if self.eat(TokenKind::KwElse) {
                if self.eat(TokenKind::KwIf) {
                    let nested = self.parse_if_after_kw_if()?;
                    vec![self.arena.alloc_stmt(nested)]
                } else {
                    self.parse_block()?
                }
            } else {
                Vec::new()
            };
            return Ok(self.arena.alloc_stmt(Stmt::If {
                condition,
                then_block,
                else_block,
            }));
        }
        if self.eat(TokenKind::KwMatch) {
            return self.parse_match_stmt_after_kw_match();
        }
        if self.eat(TokenKind::KwReturn) {
            if self.eat(TokenKind::Semi) {
                return Ok(self.arena.alloc_stmt(Stmt::Return(None)));
            }
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Return(Some(expr))));
        }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semi, "expected ';'")?;
        Ok(self.arena.alloc_stmt(Stmt::Expr(expr)))
    }

    fn peek_compound_assign_op(&self) -> Option<BinaryOp> {
        let current = self.next_non_layout_idx();
        let mut next_idx = current + 1;
        while next_idx < self.tokens.len() && Self::is_layout(self.tokens[next_idx].kind) {
            next_idx += 1;
        }
        let next = self.tokens.get(next_idx)?;
        match next.kind {
            TokenKind::PlusAssign => Some(BinaryOp::Add),
            TokenKind::MinusAssign => Some(BinaryOp::Sub),
            TokenKind::StarAssign => Some(BinaryOp::Mul),
            TokenKind::SlashAssign => Some(BinaryOp::Div),
            TokenKind::AndAndAssign => Some(BinaryOp::AndAnd),
            TokenKind::OrOrAssign => Some(BinaryOp::OrOr),
            _ => None,
        }
    }

    fn parse_if_after_kw_if(&mut self) -> Result<Stmt, FrontendError> {
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.eat(TokenKind::KwElse) {
            if self.eat(TokenKind::KwIf) {
                let nested = self.parse_if_after_kw_if()?;
                vec![self.arena.alloc_stmt(nested)]
            } else {
                self.parse_block()?
            }
        } else {
            Vec::new()
        };
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_expr(&mut self) -> Result<ExprId, FrontendError> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_impl()?;
        while self.eat(TokenKind::PipeForward) {
            left = self.parse_pipeline_stage(left)?;
        }
        Ok(left)
    }

    fn parse_impl(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_or()?;
        while self.eat(TokenKind::Implies) {
            let right = self.parse_or()?;
            left = self
                .arena
                .alloc_expr(Expr::Binary(left, BinaryOp::Implies, right));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_and()?;
        while self.eat(TokenKind::OrOr) {
            let right = self.parse_and()?;
            left = self
                .arena
                .alloc_expr(Expr::Binary(left, BinaryOp::OrOr, right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_eq()?;
        while self.eat(TokenKind::AndAnd) {
            let right = self.parse_eq()?;
            left = self
                .arena
                .alloc_expr(Expr::Binary(left, BinaryOp::AndAnd, right));
        }
        Ok(left)
    }

    fn parse_eq(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_add()?;
        loop {
            if self.eat(TokenKind::EqEq) {
                let right = self.parse_add()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Eq, right));
                continue;
            }
            if self.eat(TokenKind::Ne) {
                let right = self.parse_add()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Ne, right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_mul()?;
        loop {
            if self.eat(TokenKind::Plus) {
                let right = self.parse_mul()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Add, right));
                continue;
            }
            if self.eat(TokenKind::Minus) {
                let right = self.parse_mul()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Sub, right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_unary()?;
        loop {
            if self.eat(TokenKind::Star) {
                let right = self.parse_unary()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Mul, right));
                continue;
            }
            if self.eat(TokenKind::Slash) {
                let right = self.parse_unary()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Div, right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<ExprId, FrontendError> {
        if self.eat(TokenKind::Bang) {
            let inner = self.parse_unary()?;
            return Ok(self.arena.alloc_expr(Expr::Unary(UnaryOp::Not, inner)));
        }
        if self.eat(TokenKind::Plus) {
            let inner = self.parse_unary()?;
            return Ok(self.arena.alloc_expr(Expr::Unary(UnaryOp::Pos, inner)));
        }
        if self.eat(TokenKind::Minus) {
            let inner = self.parse_unary()?;
            return Ok(self.arena.alloc_expr(Expr::Unary(UnaryOp::Neg, inner)));
        }
        self.parse_primary()
    }

    fn parse_pipeline_stage(&mut self, input: ExprId) -> Result<ExprId, FrontendError> {
        if self.eat(TokenKind::LParen) {
            return self.parse_short_lambda_apply_after_lparen(Some(input), true);
        }
        if !self.check(TokenKind::Ident) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "pipeline stage must start with function name or call".to_string(),
            });
        }

        let name = self.expect_symbol()?;
        let mut args = vec![input];
        if self.eat(TokenKind::LParen) {
            if !self.check(TokenKind::RParen) {
                loop {
                    args.push(self.parse_expr()?);
                    if self.eat(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RParen, "expected ')'")?;
        }
        Ok(self.arena.alloc_expr(Expr::Call(name, args)))
    }

    fn parse_primary(&mut self) -> Result<ExprId, FrontendError> {
        if self.eat(TokenKind::KwIf) {
            return self.parse_if_expr_after_kw_if();
        }
        if self.eat(TokenKind::KwMatch) {
            return self.parse_match_expr_after_kw_match();
        }
        if self.check(TokenKind::LBrace) {
            return self.parse_block_expr();
        }
        if self.eat(TokenKind::LParen) {
            if self.starts_short_lambda_head() {
                return self.parse_short_lambda_apply_after_lparen(None, false);
            }
            let e = self.parse_expr()?;
            self.expect(TokenKind::RParen, "expected ')'")?;
            return Ok(e);
        }
        if self.eat(TokenKind::QuadN) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::N)));
        }
        if self.eat(TokenKind::QuadF) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::F)));
        }
        if self.eat(TokenKind::QuadT) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::T)));
        }
        if self.eat(TokenKind::QuadS) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::S)));
        }
        if self.eat(TokenKind::KwTrue) {
            return Ok(self.arena.alloc_expr(Expr::BoolLiteral(true)));
        }
        if self.eat(TokenKind::KwFalse) {
            return Ok(self.arena.alloc_expr(Expr::BoolLiteral(false)));
        }
        if self.check(TokenKind::Num) {
            let text = self.advance().text;
            if text.contains('.') {
                self.require_f64_feature("f64 literals are disabled by profile policy")?;
                let n = text.parse::<f64>().map_err(|_| FrontendError {
                    pos: 0,
                    message: "invalid float number".to_string(),
                })?;
                return Ok(self.arena.alloc_expr(Expr::Float(n)));
            }
            let n = text.parse::<i64>().map_err(|_| FrontendError {
                pos: 0,
                message: "invalid number".to_string(),
            })?;
            return Ok(self.arena.alloc_expr(Expr::Num(n)));
        }
        if self.check(TokenKind::Ident) {
            let name = self.expect_symbol()?;
            if self.eat(TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.eat(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokenKind::RParen, "expected ')'")?;
                return Ok(self.arena.alloc_expr(Expr::Call(name, args)));
            }
            return Ok(self.arena.alloc_expr(Expr::Var(name)));
        }
        Err(FrontendError {
            pos: self.pos(),
            message: "expected primary expression".to_string(),
        })
    }

    fn starts_short_lambda_head(&self) -> bool {
        self.check(TokenKind::Ident) && self.peek_next_kind() == Some(TokenKind::FatArrow)
    }

    fn parse_short_lambda_apply_after_lparen(
        &mut self,
        pipeline_input: Option<ExprId>,
        from_pipeline: bool,
    ) -> Result<ExprId, FrontendError> {
        if !self.starts_short_lambda_head() {
            return Err(FrontendError {
                pos: self.pos(),
                message: if from_pipeline {
                    "pipeline short lambda must use form '(x => expr)'".to_string()
                } else {
                    "expected parenthesized expression or short lambda".to_string()
                },
            });
        }

        let param = self.expect_symbol()?;
        self.expect(TokenKind::FatArrow, "expected '=>' after short lambda parameter")?;
        let body = self.parse_expr()?;
        self.expect(TokenKind::RParen, "expected ')' after short lambda body")?;
        self.ensure_short_lambda_capture_free(body, param)?;

        let arg = if from_pipeline {
            pipeline_input.expect("pipeline input must be provided for pipeline short lambda")
        } else {
            self.parse_short_lambda_immediate_arg()?
        };
        self.build_short_lambda_apply(param, body, arg)
    }

    fn parse_short_lambda_immediate_arg(&mut self) -> Result<ExprId, FrontendError> {
        if !self.eat(TokenKind::LParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message:
                    "short lambda is v0 call-site sugar only; use immediate invocation '(x => expr)(arg)' or pipeline stage 'value |> (x => expr)'"
                        .to_string(),
            });
        }
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "short lambda immediate call requires exactly one argument".to_string(),
            });
        }
        let arg = self.parse_expr()?;
        if self.eat(TokenKind::Comma) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "short lambda v0 currently supports exactly one argument".to_string(),
            });
        }
        self.expect(TokenKind::RParen, "expected ')' after short lambda argument")?;
        Ok(arg)
    }

    fn build_short_lambda_apply(
        &mut self,
        param: SymbolId,
        body: ExprId,
        arg: ExprId,
    ) -> Result<ExprId, FrontendError> {
        let binding = self.arena.alloc_stmt(Stmt::Let {
            name: param,
            ty: None,
            value: arg,
        });
        Ok(self.arena.alloc_expr(Expr::Block(BlockExpr {
            statements: vec![binding],
            tail: body,
        })))
    }

    fn ensure_short_lambda_capture_free(
        &self,
        body: ExprId,
        param: SymbolId,
    ) -> Result<(), FrontendError> {
        let mut scopes = vec![vec![param]];
        self.ensure_short_lambda_expr_capture_free(body, &mut scopes)
    }

    fn ensure_short_lambda_expr_capture_free(
        &self,
        expr_id: ExprId,
        scopes: &mut Vec<Vec<SymbolId>>,
    ) -> Result<(), FrontendError> {
        match self.arena.expr(expr_id) {
            Expr::QuadLiteral(_) | Expr::BoolLiteral(_) | Expr::Num(_) | Expr::Float(_) => Ok(()),
            Expr::Var(name) => {
                if scopes.iter().rev().any(|scope| scope.contains(name)) {
                    Ok(())
                } else {
                    Err(FrontendError {
                        pos: self.pos(),
                        message: format!(
                            "short lambda v0 is capture-free only; body may not reference non-local '{}'",
                            self.arena.symbol_name(*name)
                        ),
                    })
                }
            }
            Expr::Call(_, args) => {
                for arg in args {
                    self.ensure_short_lambda_expr_capture_free(*arg, scopes)?;
                }
                Ok(())
            }
            Expr::Unary(_, inner) => self.ensure_short_lambda_expr_capture_free(*inner, scopes),
            Expr::Binary(lhs, _, rhs) => {
                self.ensure_short_lambda_expr_capture_free(*lhs, scopes)?;
                self.ensure_short_lambda_expr_capture_free(*rhs, scopes)
            }
            Expr::Block(block) => {
                self.ensure_short_lambda_block_capture_free(block, scopes)
            }
            Expr::If(if_expr) => {
                self.ensure_short_lambda_expr_capture_free(if_expr.condition, scopes)?;
                self.ensure_short_lambda_block_capture_free(&if_expr.then_block, scopes)?;
                self.ensure_short_lambda_block_capture_free(&if_expr.else_block, scopes)
            }
            Expr::Match(match_expr) => {
                self.ensure_short_lambda_expr_capture_free(match_expr.scrutinee, scopes)?;
                for arm in &match_expr.arms {
                    if let Some(guard) = arm.guard {
                        self.ensure_short_lambda_expr_capture_free(guard, scopes)?;
                    }
                    self.ensure_short_lambda_block_capture_free(&arm.block, scopes)?;
                }
                if let Some(default) = &match_expr.default {
                    self.ensure_short_lambda_block_capture_free(default, scopes)?;
                }
                Ok(())
            }
        }
    }

    fn ensure_short_lambda_block_capture_free(
        &self,
        block: &BlockExpr,
        scopes: &mut Vec<Vec<SymbolId>>,
    ) -> Result<(), FrontendError> {
        scopes.push(Vec::new());
        for stmt_id in &block.statements {
            self.ensure_short_lambda_stmt_capture_free(*stmt_id, scopes)?;
        }
        self.ensure_short_lambda_expr_capture_free(block.tail, scopes)?;
        let _ = scopes.pop();
        Ok(())
    }

    fn ensure_short_lambda_stmt_capture_free(
        &self,
        stmt_id: StmtId,
        scopes: &mut Vec<Vec<SymbolId>>,
    ) -> Result<(), FrontendError> {
        match self.arena.stmt(stmt_id) {
            Stmt::Let { name, value, .. } => {
                self.ensure_short_lambda_expr_capture_free(*value, scopes)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.push(*name);
                }
                Ok(())
            }
            Stmt::Discard { value, .. } => self.ensure_short_lambda_expr_capture_free(*value, scopes),
            Stmt::Expr(expr_id) => self.ensure_short_lambda_expr_capture_free(*expr_id, scopes),
            _ => Err(FrontendError {
                pos: self.pos(),
                message: "short lambda body currently supports only expression-compatible block forms"
                    .to_string(),
            }),
        }
    }

    fn peek_next_kind(&self) -> Option<TokenKind> {
        let mut i = self.next_non_layout_idx();
        i += 1;
        while i < self.tokens.len() && Self::is_layout(self.tokens[i].kind) {
            i += 1;
        }
        self.tokens.get(i).map(|t| t.kind)
    }

    fn parse_block_expr(&mut self) -> Result<ExprId, FrontendError> {
        let block = self.parse_value_block()?;
        Ok(self.arena.alloc_expr(Expr::Block(block)))
    }

    fn starts_stmt_only_in_block_expr(&self) -> bool {
        self.check(TokenKind::KwLet)
    }

    fn parse_if_expr_after_kw_if(&mut self) -> Result<ExprId, FrontendError> {
        let condition = self.parse_expr()?;
        let then_block = self.parse_value_block()?;
        if !self.eat(TokenKind::KwElse) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "if expression requires explicit else branch".to_string(),
            });
        }
        if self.check(TokenKind::KwIf) {
            return Err(FrontendError {
                pos: self.pos(),
                message:
                    "else-if sugar is not supported in if expressions yet; use else { if ... }"
                        .to_string(),
            });
        }
        let else_block = self.parse_value_block()?;
        Ok(self.arena.alloc_expr(Expr::If(IfExpr {
            condition,
            then_block,
            else_block,
        })))
    }

    fn parse_match_stmt_after_kw_match(&mut self) -> Result<StmtId, FrontendError> {
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace, "expected '{' after match expr")?;
        let mut arms = Vec::new();
        let mut default: Option<Vec<StmtId>> = None;
        while !self.check(TokenKind::RBrace) {
            if self.eat(TokenKind::Underscore) {
                if self.check(TokenKind::KwIf) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "default '_' arm in match currently cannot have guard".to_string(),
                    });
                }
                self.expect(TokenKind::FatArrow, "expected '=>' after '_'")?;
                let block = self.parse_block()?;
                default = Some(block);
                continue;
            }
            let pat = self.parse_quad_match_pattern()?;
            let guard = self.parse_match_guard_opt()?;
            self.expect(TokenKind::FatArrow, "expected '=>' after match pattern")?;
            let block = self.parse_block()?;
            arms.push(MatchArm { pat, guard, block });
        }
        self.expect(TokenKind::RBrace, "expected '}' after match arms")?;
        Ok(self.arena.alloc_stmt(Stmt::Match {
            scrutinee,
            arms,
            default: default.unwrap_or_default(),
        }))
    }

    fn parse_match_expr_after_kw_match(&mut self) -> Result<ExprId, FrontendError> {
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace, "expected '{' after match expr")?;
        let mut arms = Vec::new();
        let mut default: Option<BlockExpr> = None;
        while !self.check(TokenKind::RBrace) {
            if self.eat(TokenKind::Underscore) {
                if self.check(TokenKind::KwIf) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "default '_' arm in match currently cannot have guard".to_string(),
                    });
                }
                self.expect(TokenKind::FatArrow, "expected '=>' after '_'")?;
                let block = self.parse_value_block()?;
                default = Some(block);
                continue;
            }
            let pat = self.parse_quad_match_pattern()?;
            let guard = self.parse_match_guard_opt()?;
            self.expect(TokenKind::FatArrow, "expected '=>' after match pattern")?;
            let block = self.parse_value_block()?;
            arms.push(MatchExprArm { pat, guard, block });
        }
        self.expect(TokenKind::RBrace, "expected '}' after match arms")?;
        Ok(self.arena.alloc_expr(Expr::Match(MatchExpr {
            scrutinee,
            arms,
            default,
        })))
    }

    fn parse_quad_match_pattern(&mut self) -> Result<QuadVal, FrontendError> {
        if self.eat(TokenKind::QuadN) {
            Ok(QuadVal::N)
        } else if self.eat(TokenKind::QuadF) {
            Ok(QuadVal::F)
        } else if self.eat(TokenKind::QuadT) {
            Ok(QuadVal::T)
        } else if self.eat(TokenKind::QuadS) {
            Ok(QuadVal::S)
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: "expected match pattern N|F|T|S|_".to_string(),
            })
        }
    }

    fn parse_match_guard_opt(&mut self) -> Result<Option<ExprId>, FrontendError> {
        if self.eat(TokenKind::KwIf) {
            return Ok(Some(self.parse_expr()?));
        }
        Ok(None)
    }

    fn parse_value_block(&mut self) -> Result<BlockExpr, FrontendError> {
        self.expect(TokenKind::LBrace, "expected '{'")?;
        let mut statements = Vec::new();

        loop {
            if self.check(TokenKind::RBrace) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "value-producing block requires trailing value expression before '}'"
                        .to_string(),
                });
            }

            if self.starts_stmt_only_in_block_expr() {
                statements.push(self.parse_stmt()?);
                continue;
            }

            if self.check(TokenKind::KwGuard) || self.check(TokenKind::KwReturn) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message:
                        "value-producing block currently supports only let-bindings and expression statements before the tail value"
                            .to_string(),
                });
            }

            let expr = self.parse_expr()?;
            if self.eat(TokenKind::Semi) {
                statements.push(self.arena.alloc_stmt(Stmt::Expr(expr)));
                continue;
            }

            self.expect(
                TokenKind::RBrace,
                "expected '}' after value-producing block",
            )?;
            return Ok(BlockExpr {
                statements,
                tail: expr,
            });
        }
    }

    fn parse_type(&mut self) -> Result<Type, FrontendError> {
        if self.check(TokenKind::Ident) {
            let t = self.tokens[self.next_non_layout_idx()].text.clone();
            if t == "qvec" {
                let _ = self.advance();
                if self.eat(TokenKind::LBracket) || self.eat(TokenKind::LParen) {
                    let n = if self.check(TokenKind::Num) {
                        self.advance().text.parse::<usize>().unwrap_or(32)
                    } else {
                        32
                    };
                    let _ = self.eat(TokenKind::RBracket) || self.eat(TokenKind::RParen);
                    return Ok(Type::QVec(n));
                }
                return Ok(Type::QVec(32));
            }
        }
        if self.eat(TokenKind::TyQuad) {
            return Ok(Type::Quad);
        }
        if self.eat(TokenKind::TyBool) {
            return Ok(Type::Bool);
        }
        if self.eat(TokenKind::TyI32) {
            return Ok(Type::I32);
        }
        if self.eat(TokenKind::TyU32) {
            return Ok(Type::U32);
        }
        if self.eat(TokenKind::TyFx) {
            return Ok(Type::Fx);
        }
        if self.eat(TokenKind::TyF64) {
            self.require_f64_feature("type 'f64' is disabled by profile policy")?;
            return Ok(Type::F64);
        }
        Err(FrontendError {
            pos: self.pos(),
            message: "expected type".to_string(),
        })
    }

    fn parse_logos_program(&mut self) -> Result<LogosProgram, FrontendError> {
        self.require_logos_surface("Logos surface is disabled by profile policy")?;
        let mut out = LogosProgram::default();
        let mut errors: Vec<FrontendError> = Vec::new();
        while self.idx < self.tokens.len() {
            self.skip_newlines_raw();
            if self.idx >= self.tokens.len() {
                break;
            }
            if self.check_raw(TokenKind::KwSystem) {
                match self.parse_logos_system() {
                    Ok(system) => out.system = Some(system),
                    Err(e) => {
                        errors.push(e);
                        self.recover_logos_anchor();
                    }
                }
                continue;
            }
            if self.check_raw(TokenKind::KwEntity) {
                match self.parse_logos_entity() {
                    Ok(entity) => out.entities.push(entity),
                    Err(e) => {
                        errors.push(e);
                        self.recover_logos_anchor();
                    }
                }
                continue;
            }
            if self.check_raw(TokenKind::KwLaw) {
                match self.parse_logos_law() {
                    Ok(law) => out.laws.push(law),
                    Err(e) => {
                        errors.push(e);
                        self.recover_logos_anchor();
                    }
                }
                continue;
            }
            if self.check_raw(TokenKind::KwImport)
                || self.check_raw(TokenKind::KwPulse)
                || self.check_raw(TokenKind::KwProfile)
            {
                self.require_legacy_compatibility(
                    "legacy Logos directives require legacy compatibility mode",
                )?;
                while !self.check_raw(TokenKind::Newline) && self.idx < self.tokens.len() {
                    self.idx += 1;
                }
                self.eat_raw(TokenKind::Newline);
                continue;
            }
            let mut msg = "expected Logos declaration".to_string();
            if let Some(tok) = self.tokens.get(self.idx) {
                if tok.kind == TokenKind::Ident {
                    if let Some(s) = suggest_closest_case_insensitive(
                        &tok.text,
                        &["System", "Entity", "Law", "Import", "Pulse", "Profile"],
                        3,
                    ) {
                        msg.push_str(&format!("\nhelp: did you mean '{}'?", s));
                    }
                }
            }
            errors.push(self.error_at_current(&msg, "E0200"));
            self.recover_logos_anchor();
        }
        if !errors.is_empty() {
            return Err(self.merge_logos_errors(errors));
        }
        out.laws.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(out)
    }

    fn recover_logos_anchor(&mut self) {
        while self.idx < self.tokens.len() {
            let k = self.tokens[self.idx].kind;
            if matches!(
                k,
                TokenKind::Newline
                    | TokenKind::Dedent
                    | TokenKind::KwSystem
                    | TokenKind::KwEntity
                    | TokenKind::KwLaw
            ) {
                if matches!(k, TokenKind::Newline | TokenKind::Dedent) {
                    self.idx += 1;
                }
                break;
            }
            self.idx += 1;
        }
    }

    fn merge_logos_errors(&self, errors: Vec<FrontendError>) -> FrontendError {
        let pos = errors.first().map(|e| e.pos).unwrap_or(0);
        let msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
        FrontendError {
            pos,
            message: format_multiple_parser_errors("E0200", &msgs),
        }
    }

    fn parse_logos_system(&mut self) -> Result<LogosSystem, FrontendError> {
        let kw = self.expect_raw(TokenKind::KwSystem, "expected 'System'", "E0201")?;
        let name = self.expect_raw_ident()?;
        let mut params = Vec::new();
        if self.eat_raw(TokenKind::LParen) {
            if !self.check_raw(TokenKind::RParen) {
                loop {
                    let pname = self.expect_raw_ident()?;
                    self.expect_raw(TokenKind::Assign, "expected '='", "E0202")?;
                    let pval = self.expect_raw_ident_or_num()?;
                    params.push((pname, pval));
                    if self.eat_raw(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_raw(TokenKind::RParen, "expected ')'", "E0203")?;
        }
        self.eat_raw(TokenKind::Colon);
        self.eat_raw(TokenKind::Newline);
        Ok(LogosSystem {
            name,
            params,
            mark: kw.mark,
        })
    }

    fn parse_logos_entity(&mut self) -> Result<LogosEntity, FrontendError> {
        let kw = self.expect_raw(TokenKind::KwEntity, "expected 'Entity'", "E0210")?;
        let name = self.expect_raw_ident()?;
        self.expect_raw(TokenKind::Colon, "expected ':'", "E0211")?;
        self.expect_raw(TokenKind::Newline, "expected newline", "E0212")?;
        self.expect_raw(TokenKind::Indent, "expected INDENT", "E0213")?;

        let mut fields = Vec::new();
        loop {
            self.skip_newlines_raw();
            if self.eat_raw(TokenKind::Dedent) {
                break;
            }
            let kind_tok =
                self.expect_raw(TokenKind::Ident, "expected 'state' or 'prop'", "E0214")?;
            let field_kind = match kind_tok.text.as_str() {
                "state" => LogosEntityFieldKind::State,
                "prop" => LogosEntityFieldKind::Prop,
                _ => {
                    let mut msg = "expected 'state' or 'prop'".to_string();
                    if let Some(s) =
                        suggest_closest_case_insensitive(&kind_tok.text, &["state", "prop"], 3)
                    {
                        msg.push_str(&format!("\nhelp: did you mean '{}'?", s));
                    }
                    return Err(self.error_at_token(&kind_tok, &msg, "E0215"));
                }
            };
            let field_name = self.expect_raw_ident()?;
            self.expect_raw(TokenKind::Colon, "expected ':'", "E0216")?;
            let ty = self.parse_type_raw()?;
            fields.push(LogosEntityField {
                kind: field_kind,
                name: field_name,
                ty,
                mark: kind_tok.mark,
            });
            self.eat_raw(TokenKind::Newline);
        }

        Ok(LogosEntity {
            name,
            fields,
            mark: kw.mark,
        })
    }

    fn parse_logos_law(&mut self) -> Result<LogosLaw, FrontendError> {
        let kw = self.expect_raw(TokenKind::KwLaw, "expected 'Law'", "E0220")?;
        let name_tok = self.expect_raw(TokenKind::String, "expected law name", "E0221")?;
        let name = name_tok.text.trim_matches('"').to_string();
        let mut priority = 0u32;
        if self.eat_raw(TokenKind::LBracket) {
            let p_kw = self.expect_raw(TokenKind::Ident, "expected 'priority'", "E0222")?;
            if p_kw.text != "priority" {
                return Err(self.error_at_token(&p_kw, "expected 'priority'", "E0223"));
            }
            let num = self.expect_raw(TokenKind::Num, "expected priority number", "E0224")?;
            priority = num
                .text
                .parse::<u32>()
                .map_err(|_| self.error_at_token(&num, "invalid priority value", "E0225"))?;
            self.expect_raw(TokenKind::RBracket, "expected ']'", "E0226")?;
        }
        self.expect_raw(TokenKind::Colon, "expected ':'", "E0227")?;
        self.expect_raw(TokenKind::Newline, "expected newline", "E0228")?;
        self.expect_raw(TokenKind::Indent, "expected INDENT", "E0229")?;

        let mut whens = Vec::new();
        loop {
            self.skip_newlines_raw();
            if self.eat_raw(TokenKind::Dedent) {
                break;
            }
            let when_tok = self.expect_raw(TokenKind::KwWhen, "expected 'When'", "E0230")?;
            let condition_tokens = self.collect_until_raw(TokenKind::Implies)?;
            if condition_tokens.is_empty() {
                return Err(self.error_at_token(&when_tok, "empty When condition", "E0231"));
            }
            self.expect_raw(TokenKind::Implies, "expected '->'", "E0232")?;
            let effect_tokens = if self.eat_raw(TokenKind::Newline) {
                self.skip_newlines_raw();
                self.collect_until_newline_or_dedent()
            } else {
                self.collect_until_newline_or_dedent()
            };
            if effect_tokens.is_empty() {
                return Err(self.error_at_token(&when_tok, "empty When effect", "E0233"));
            }
            let condition = self.join_token_text(&condition_tokens);
            let effect = self.join_token_text(&effect_tokens);
            whens.push(LogosWhen {
                condition,
                effect,
                mark: when_tok.mark,
            });
            self.eat_raw(TokenKind::Newline);
        }

        Ok(LogosLaw {
            name,
            priority,
            whens,
            mark: kw.mark,
        })
    }

    fn parse_type_raw(&mut self) -> Result<Type, FrontendError> {
        if self.check_raw(TokenKind::Ident) {
            let t = self.tokens[self.idx].text.clone();
            if t == "qvec" {
                self.idx += 1;
                return Ok(Type::QVec(32));
            }
        }
        if self.eat_raw(TokenKind::TyQuad) {
            return Ok(Type::Quad);
        }
        if self.eat_raw(TokenKind::TyBool) {
            return Ok(Type::Bool);
        }
        if self.eat_raw(TokenKind::TyI32) {
            return Ok(Type::I32);
        }
        if self.eat_raw(TokenKind::TyU32) {
            return Ok(Type::U32);
        }
        if self.eat_raw(TokenKind::TyFx) {
            return Ok(Type::Fx);
        }
        if self.eat_raw(TokenKind::TyF64) {
            self.require_f64_feature("type 'f64' is disabled by profile policy")?;
            return Ok(Type::F64);
        }
        Err(self.error_at_current("expected type", "E0234"))
    }

    fn collect_until_raw(&mut self, stop: TokenKind) -> Result<Vec<Token>, FrontendError> {
        let mut out = Vec::new();
        let mut paren = 0usize;
        while self.idx < self.tokens.len() {
            let t = self.tokens[self.idx].clone();
            if t.kind == TokenKind::LParen {
                paren += 1;
            } else if t.kind == TokenKind::RParen {
                paren = paren.saturating_sub(1);
            }
            if paren == 0 && t.kind == stop {
                break;
            }
            if t.kind == TokenKind::Newline || t.kind == TokenKind::Dedent {
                return Err(self.error_at_token(
                    &t,
                    "unexpected line break in expression",
                    "E0235",
                ));
            }
            out.push(t);
            self.idx += 1;
        }
        Ok(out)
    }

    fn collect_until_newline_or_dedent(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while self.idx < self.tokens.len() {
            let t = self.tokens[self.idx].clone();
            if matches!(t.kind, TokenKind::Newline | TokenKind::Dedent) {
                break;
            }
            out.push(t);
            self.idx += 1;
        }
        out
    }

    fn join_token_text(&self, toks: &[Token]) -> String {
        toks.iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn check_raw(&self, kind: TokenKind) -> bool {
        self.tokens
            .get(self.idx)
            .map(|t| t.kind == kind)
            .unwrap_or(false)
    }

    fn eat_raw(&mut self, kind: TokenKind) -> bool {
        if self.check_raw(kind) {
            self.idx += 1;
            true
        } else {
            false
        }
    }

    fn expect_raw(
        &mut self,
        kind: TokenKind,
        msg: &str,
        code: &str,
    ) -> Result<Token, FrontendError> {
        if self.check_raw(kind) {
            let t = self.tokens[self.idx].clone();
            self.idx += 1;
            Ok(t)
        } else {
            Err(self.error_at_current(msg, code))
        }
    }

    fn expect_raw_ident(&mut self) -> Result<String, FrontendError> {
        let tok = self.expect_raw(TokenKind::Ident, "expected identifier", "E0236")?;
        Ok(tok.text)
    }

    fn expect_raw_ident_or_num(&mut self) -> Result<String, FrontendError> {
        if self.check_raw(TokenKind::Ident)
            || self.check_raw(TokenKind::Num)
            || self.check_raw(TokenKind::String)
        {
            let t = self.tokens[self.idx].clone();
            self.idx += 1;
            return Ok(t.text);
        }
        Err(self.error_at_current("expected identifier/number", "E0237"))
    }

    fn skip_newlines_raw(&mut self) {
        while self.eat_raw(TokenKind::Newline) {}
    }

    fn require_f64_feature(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.features.allow_f64_math {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn require_logos_surface(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.features.allow_logos_surface {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn require_legacy_compatibility(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.compatibility == CompatibilityMode::LegacySupport {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn error_at_current(&self, msg: &str, code: &str) -> FrontendError {
        if let Some(tok) = self.tokens.get(self.idx) {
            self.error_at_token(tok, msg, code)
        } else {
            FrontendError {
                pos: self.pos(),
                message: format!("error[{code}]: {msg}"),
            }
        }
    }

    fn error_at_token(&self, tok: &Token, msg: &str, code: &str) -> FrontendError {
        let line = tok.mark.line.max(1);
        let col = tok.mark.col.max(1);
        let mut sm = SourceMap::new();
        let fid = sm.add_file("<input>", &self.source);
        let src_line = sm.line(fid, line).unwrap_or_default();
        FrontendError {
            pos: tok.pos,
            message: format_parser_error_at_input(code, msg, line, col, src_line),
        }
    }

    fn pos(&self) -> usize {
        self.tokens.get(self.idx).map(|t| t.pos).unwrap_or(0)
    }

    fn is_layout(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
        )
    }

    fn next_non_layout_idx(&self) -> usize {
        let mut i = self.idx;
        while i < self.tokens.len() && Self::is_layout(self.tokens[i].kind) {
            i += 1;
        }
        i
    }

    fn check(&self, kind: TokenKind) -> bool {
        let i = self.next_non_layout_idx();
        self.tokens.get(i).map(|t| t.kind == kind).unwrap_or(false)
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        let i = self.next_non_layout_idx();
        if self.tokens.get(i).map(|t| t.kind == kind).unwrap_or(false) {
            self.idx = i + 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<(), FrontendError> {
        if self.eat(kind) {
            Ok(())
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: msg.to_string(),
            })
        }
    }

    fn expect_symbol(&mut self) -> Result<SymbolId, FrontendError> {
        if self.check(TokenKind::Ident) {
            let name = self.advance().text;
            Ok(self.arena.intern_symbol(&name))
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: "expected identifier".to_string(),
            })
        }
    }

    fn advance(&mut self) -> Token {
        let i = self.next_non_layout_idx();
        let t = self.tokens[i].clone();
        self.idx = i + 1;
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrontendErrorKind;
    use sm_profile::ParserProfile;

    #[test]
    fn rustlike_parser_smoke() {
        let src = r#"
fn idq(q: quad) -> quad { return q; }
fn main() { let x: quad = idq(T); return; }
"#;
        let a = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("frontend rustlike");
        assert_eq!(a.functions.len(), 2);
    }

    #[test]
    fn rustlike_parser_accepts_expression_bodied_function() {
        let src = r#"
fn idq(q: quad) -> quad = q;
fn main() { return; }
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("expression-bodied function should parse");
        let func = &program.functions[0];
        let Stmt::Return(Some(value)) = program.arena.stmt(func.body[0]) else {
            panic!("expected desugared return statement");
        };
        assert!(matches!(program.arena.expr(*value), Expr::Var(_)));
    }

    #[test]
    fn rustlike_parser_rejects_expression_bodied_function_without_semi() {
        let src = r#"
fn idq(q: quad) -> quad = q
fn main() { return; }
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("missing semicolon must reject");
        assert!(err
            .message
            .contains("expected ';' after expression-bodied function"));
    }

    #[test]
    fn rustlike_parser_accepts_compound_assignment() {
        let src = r#"
fn main() {
    let total: f64 = 1.0;
    total += 2.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("compound assignment should parse");
        let func = &program.functions[0];
        let Stmt::Assign { name, value } = program.arena.stmt(func.body[1]) else {
            panic!("expected compound assignment statement");
        };
        assert_eq!(program.arena.symbol_name(*name), "total");
        let Expr::Binary(lhs, BinaryOp::Add, rhs) = program.arena.expr(*value) else {
            panic!("expected desugared additive assignment");
        };
        assert!(matches!(program.arena.expr(*lhs), Expr::Var(_)));
        assert!(matches!(program.arena.expr(*rhs), Expr::Float(_)));
    }

    #[test]
    fn rustlike_parser_accepts_pipeline_chain() {
        let src = r#"
fn inc(x: f64) -> f64 = x + 1.0;
fn scale(x: f64, factor: f64) -> f64 = x * factor;
fn main() {
    let value: f64 = 1.0 |> inc() |> scale(3.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("pipeline chain should parse");
        let func = &program.functions[2];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Call(scale_name, scale_args) = program.arena.expr(*value) else {
            panic!("expected outer desugared call");
        };
        assert_eq!(program.arena.symbol_name(*scale_name), "scale");
        assert_eq!(scale_args.len(), 2);
        let Expr::Call(inc_name, inc_args) = program.arena.expr(scale_args[0]) else {
            panic!("expected nested pipeline call");
        };
        assert_eq!(program.arena.symbol_name(*inc_name), "inc");
        assert_eq!(inc_args.len(), 1);
    }

    #[test]
    fn rustlike_parser_accepts_immediate_short_lambda() {
        let src = r#"
fn main() {
    let value: f64 = (x => x + 1.0)(2.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("immediate short lambda should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected desugared block expression");
        };
        assert_eq!(block.statements.len(), 1);
        let Stmt::Let { name, value, .. } = program.arena.stmt(block.statements[0]) else {
            panic!("expected lambda parameter binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "x");
        assert!(matches!(program.arena.expr(*value), Expr::Float(_)));
        assert!(matches!(program.arena.expr(block.tail), Expr::Binary(_, BinaryOp::Add, _)));
    }

    #[test]
    fn rustlike_parser_accepts_pipeline_short_lambda_stage() {
        let src = r#"
fn main() {
    let value: f64 = 2.0 |> (x => x + 1.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("pipeline short lambda should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected desugared block expression");
        };
        let Stmt::Let { name, .. } = program.arena.stmt(block.statements[0]) else {
            panic!("expected lambda parameter binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "x");
    }

    #[test]
    fn rustlike_parser_rejects_standalone_short_lambda_value() {
        let src = r#"
fn main() {
    let value: f64 = (x => x + 1.0);
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("standalone short lambda must reject");
        assert!(err
            .message
            .contains("short lambda is v0 call-site sugar only"));
    }

    #[test]
    fn rustlike_parser_rejects_captureful_short_lambda() {
        let src = r#"
fn main() {
    let offset: f64 = 1.0;
    let value: f64 = (x => x + offset)(2.0);
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("captureful short lambda must reject");
        assert!(err.message.contains("capture-free only"));
    }

    #[test]
    fn rustlike_parser_rejects_pipeline_without_call_target() {
        let src = r#"
fn main() {
    let value: f64 = 1.0 |> true;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("pipeline stage without function target must reject");
        assert!(err
            .message
            .contains("pipeline stage must start with function name or call"));
    }

    #[test]
    fn rustlike_parser_accepts_block_expression_tail() {
        let src = r#"
fn main() {
    let value: f64 = {
        let base: f64 = 1.0;
        base + 2.0
    };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("block expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected block expression");
        };
        assert_eq!(block.statements.len(), 1);
        match program.arena.expr(block.tail) {
            Expr::Binary(_, BinaryOp::Add, _) => {}
            other => panic!("expected additive tail expression, got {:?}", other),
        }
    }

    #[test]
    fn rustlike_parser_rejects_block_expression_without_tail() {
        let src = r#"
fn main() {
    let value: i32 = {
        let base: i32 = 1;
    };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("block expression without tail must reject");
        assert!(err
            .message
            .contains("value-producing block requires trailing value expression"));
    }

    #[test]
    fn rustlike_parser_accepts_if_expression() {
        let src = r#"
fn main() {
    let value: f64 = if true { 1.0 } else { 2.0 };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("if expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::If(if_expr) = program.arena.expr(*value) else {
            panic!("expected if expression");
        };
        assert!(matches!(
            program.arena.expr(if_expr.condition),
            Expr::BoolLiteral(true)
        ));
    }

    #[test]
    fn rustlike_parser_rejects_if_expression_without_else() {
        let src = r#"
fn main() {
    let value: f64 = if true { 1.0 };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("if expression without else must reject");
        assert!(err
            .message
            .contains("if expression requires explicit else branch"));
    }

    #[test]
    fn rustlike_parser_rejects_else_if_sugar_in_if_expression() {
        let src = r#"
fn main() {
    let value: f64 = if true { 1.0 } else if false { 2.0 } else { 3.0 };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("else-if sugar must reject in value position");
        assert!(err
            .message
            .contains("else-if sugar is not supported in if expressions yet"));
    }

    #[test]
    fn rustlike_parser_accepts_match_expression() {
        let src = r#"
fn main() {
    let value: f64 = match T {
        T if true => { 1.0 }
        _ => { 2.0 }
    };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("match expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Match(match_expr) = program.arena.expr(*value) else {
            panic!("expected match expression");
        };
        assert!(matches!(
            program.arena.expr(match_expr.scrutinee),
            Expr::QuadLiteral(QuadVal::T)
        ));
        assert_eq!(match_expr.arms.len(), 1);
        assert!(match_expr.arms[0].guard.is_some());
        assert!(match_expr.default.is_some());
    }

    #[test]
    fn rustlike_parser_rejects_guarded_default_match_arm() {
        let src = r#"
fn main() {
    let value: f64 = match T {
        _ if true => { 2.0 }
    };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("guarded default arm must reject");
        assert!(err
            .message
            .contains("default '_' arm in match currently cannot have guard"));
    }

    #[test]
    fn rustlike_parser_accepts_guard_clause() {
        let src = r#"
fn main() {
    guard true else return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("guard clause should parse");
        let func = &program.functions[0];
        let Stmt::Guard {
            condition,
            else_return,
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected guard statement");
        };
        assert!(matches!(
            program.arena.expr(*condition),
            Expr::BoolLiteral(true)
        ));
        assert!(else_return.is_none());
    }

    #[test]
    fn rustlike_parser_rejects_guard_without_else_return() {
        let src = r#"
fn main() {
    guard true else { return; }
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("guard clause without else return should reject");
        assert!(err
            .message
            .contains("guard clause currently supports only else return"));
    }

    #[test]
    fn rustlike_parser_accepts_discard_bind() {
        let src = r#"
fn main() {
    let _ = 1.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("discard bind should parse");
        let func = &program.functions[0];
        let Stmt::Discard { ty, value } = program.arena.stmt(func.body[0]) else {
            panic!("expected discard statement");
        };
        assert!(ty.is_none());
        assert!(matches!(program.arena.expr(*value), Expr::Float(_)));
    }

    #[test]
    fn rustlike_parser_accepts_typed_discard_bind() {
        let src = r#"
fn main() {
    let _: f64 = 1.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("typed discard bind should parse");
        let func = &program.functions[0];
        let Stmt::Discard { ty, value } = program.arena.stmt(func.body[0]) else {
            panic!("expected discard statement");
        };
        assert_eq!(*ty, Some(Type::F64));
        assert!(matches!(program.arena.expr(*value), Expr::Float(_)));
    }

    #[test]
    fn rustlike_parser_accepts_assert_statement_surface() {
        let src = r#"
fn main() {
    assert(true);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("assert statement should parse");
        let func = &program.functions[0];
        let Stmt::Expr(expr_id) = program.arena.stmt(func.body[0]) else {
            panic!("expected expression statement");
        };
        let Expr::Call(name, args) = program.arena.expr(*expr_id) else {
            panic!("expected call-shaped assert surface");
        };
        assert_eq!(program.arena.symbol_name(*name), "assert");
        assert_eq!(args.len(), 1);
        assert!(matches!(
            program.arena.expr(args[0]),
            Expr::BoolLiteral(true)
        ));
    }

    #[test]
    fn logos_parser_smoke() {
        let src = r#"
Entity Sensor:
    state val: quad

Law "CheckSignal" [priority 10]:
    When Sensor.val == T ->
        Log.emit("Signal OK")
"#;
        let a = parse_logos_with_profile(src, &ParserProfile::foundation_default())
            .expect("frontend logos");
        assert_eq!(a.entities.len(), 1);
        assert_eq!(a.laws.len(), 1);
    }

    #[test]
    fn strict_profile_rejects_f64_surface() {
        let profile = ParserProfile::default();
        let err = parse_rustlike_with_profile("fn main() -> f64 { return 1.5; }", &profile)
            .expect_err("strict profile must reject f64");

        assert_eq!(err.kind(), FrontendErrorKind::PolicyViolation);
        assert!(err.message.contains("f64"));
    }

    #[test]
    fn strict_profile_rejects_logos_surface() {
        let profile = ParserProfile::default();
        let src = r#"
Law "L" [priority 1]:
    When true -> System.recovery()
"#;
        let err =
            parse_logos_with_profile(src, &profile).expect_err("strict profile must reject logos");

        assert_eq!(err.kind(), FrontendErrorKind::PolicyViolation);
        assert!(err.message.contains("Logos surface"));
    }
}
