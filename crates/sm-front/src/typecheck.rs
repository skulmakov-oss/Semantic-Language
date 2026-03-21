use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::ToString;
use crate::*;

fn fx_coercion_gap_message() -> &'static str {
    "fx coercion from non-literal numeric expressions is not implemented in the canonical Rust-like path yet"
}

fn fx_arithmetic_gap_message() -> &'static str {
    "fx arithmetic is not implemented in the canonical Rust-like path yet"
}

fn fx_unary_gap_message() -> &'static str {
    "fx unary +/- is not implemented in the canonical Rust-like path yet"
}

fn is_numeric_for_fx_gap(ty: Type) -> bool {
    matches!(ty, Type::I32 | Type::U32 | Type::F64)
}

fn is_fx_literal_expr(expr_id: ExprId, arena: &AstArena) -> bool {
    match arena.expr(expr_id) {
        Expr::Num(_) | Expr::Float(_) => true,
        Expr::Unary(UnaryOp::Pos | UnaryOp::Neg, inner) => is_fx_literal_expr(*inner, arena),
        _ => false,
    }
}

pub fn type_check_function(program: &Program) -> Result<(), FrontendError> {
    if program.functions.len() != 1 {
        return Err(FrontendError {
            pos: 0,
            message: "type_check_function expects exactly one function in program".to_string(),
        });
    }
    let mut table = BTreeMap::new();
    let func = &program.functions[0];
    table.insert(
        func.name,
        FnSig {
            params: func.params.iter().map(|(_, t)| *t).collect(),
            ret: func.ret,
        },
    );
    type_check_function_with_table(func, &program.arena, &table)
}

pub fn type_check_program(p: &Program) -> Result<(), FrontendError> {
    let table = build_fn_table(p)?;
    let main_id = p
        .arena
        .symbol_to_id
        .get("main")
        .copied()
        .ok_or(FrontendError {
            pos: 0,
            message: "program must define fn main()".to_string(),
        })?;
    let main_sig = table.get(&main_id).ok_or(FrontendError {
        pos: 0,
        message: "program must define fn main()".to_string(),
    })?;
    if !main_sig.params.is_empty() || main_sig.ret != Type::Unit {
        return Err(FrontendError {
            pos: 0,
            message: "main must have signature fn main()".to_string(),
        });
    }
    for f in &p.functions {
        type_check_function_with_table(f, &p.arena, &table)?;
    }
    Ok(())
}

pub fn type_check_function_with_table(
    func: &Function,
    arena: &AstArena,
    table: &FnTable,
) -> Result<(), FrontendError> {
    let mut env = ScopeEnv::with_params(&func.params);
    for stmt in &func.body {
        check_stmt(*stmt, arena, &mut env, func.ret, table)?;
    }
    Ok(())
}

fn check_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
) -> Result<(), FrontendError> {
    let stmt = arena.stmt(stmt_id);
    match stmt {
        Stmt::Let { name, ty, value } => {
            let vt = infer_expr_type(*value, arena, env, table, ret_ty)?;
            let final_ty = if let Some(ann) = ty {
                if *ann != vt {
                    if *ann == Type::Fx && is_numeric_for_fx_gap(vt) {
                        if is_fx_literal_expr(*value, arena) {
                            Type::Fx
                        } else {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "{}; let '{}' currently accepts only fx literals or existing fx-typed values",
                                    fx_coercion_gap_message(),
                                    resolve_symbol_name(arena, *name)?,
                                ),
                            });
                        }
                    } else {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "type mismatch in let '{}': {:?} vs {:?}",
                                resolve_symbol_name(arena, *name)?,
                                ann,
                                vt
                            ),
                        });
                    }
                } else {
                    *ann
                }
            } else {
                vt
            };
            env.insert(*name, final_ty);
            Ok(())
        }
        Stmt::Guard {
            condition,
            else_return,
        } => {
            let condition_ty = infer_expr_type(*condition, arena, env, table, ret_ty)?;
            if condition_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "guard clause condition must be bool; explicit compare is required for quad"
                            .to_string(),
                });
            }
            check_return_payload(*else_return, arena, env, table, ret_ty)
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            let ct = infer_expr_type(*condition, arena, env, table, ret_ty)?;
            if ct != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool; explicit compare is required for quad"
                        .to_string(),
                });
            }

            let mut then_env = env.clone();
            then_env.push_scope();
            for s in then_block {
                check_stmt(*s, arena, &mut then_env, ret_ty, table)?;
            }
            then_env.pop_scope();

            let mut else_env = env.clone();
            else_env.push_scope();
            for s in else_block {
                check_stmt(*s, arena, &mut else_env, ret_ty, table)?;
            }
            else_env.pop_scope();
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            let st = infer_expr_type(*scrutinee, arena, env, table, ret_ty)?;
            if st != Type::Quad {
                return Err(FrontendError {
                    pos: 0,
                    message: "match is allowed only for quad scrutinee".to_string(),
                });
            }
            if default.is_empty() {
                return Err(FrontendError {
                    pos: 0,
                    message: "match requires default arm '_'".to_string(),
                });
            }

            for arm in arms {
                let mut arm_env = env.clone();
                arm_env.push_scope();
                check_match_guard(arm.guard, arena, &arm_env, table, ret_ty)?;
                for s in &arm.block {
                    check_stmt(*s, arena, &mut arm_env, ret_ty, table)?;
                }
                arm_env.pop_scope();
            }

            let mut def_env = env.clone();
            def_env.push_scope();
            for s in default {
                check_stmt(*s, arena, &mut def_env, ret_ty, table)?;
            }
            def_env.pop_scope();
            Ok(())
        }
        Stmt::Return(v) => {
            check_return_payload(*v, arena, env, table, ret_ty)
        }
        Stmt::Expr(e) => {
            let _ = infer_expr_type(*e, arena, env, table, ret_ty)?;
            Ok(())
        }
    }
}

fn infer_expr_type(
    expr_id: ExprId,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
) -> Result<Type, FrontendError> {
    let expr = arena.expr(expr_id);
    match expr {
        Expr::QuadLiteral(_) => Ok(Type::Quad),
        Expr::BoolLiteral(_) => Ok(Type::Bool),
        Expr::Num(_) => Ok(Type::I32),
        Expr::Float(_) => Ok(Type::F64),
        Expr::Var(v) => env.get(*v).ok_or(FrontendError {
            pos: 0,
            message: format!("unknown variable '{}'", resolve_symbol_name(arena, *v)?),
        }),
        Expr::Block(block) => infer_value_block_type(block, arena, env, table, ret_ty),
        Expr::If(if_expr) => {
            let cond_ty = infer_expr_type(if_expr.condition, arena, env, table, ret_ty)?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "if expression condition must be bool; explicit compare is required for quad"
                            .to_string(),
                });
            }
            let then_ty = infer_value_block_type(&if_expr.then_block, arena, env, table, ret_ty)?;
            let else_ty = infer_value_block_type(&if_expr.else_block, arena, env, table, ret_ty)?;
            if then_ty != else_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "if expression branch type mismatch: then {:?}, else {:?}",
                        then_ty, else_ty
                    ),
                });
            }
            Ok(then_ty)
        }
        Expr::Match(match_expr) => infer_match_expr_type(match_expr, arena, env, table, ret_ty),
        Expr::Call(name, args) => {
            let sig = if let Some(s) = table.get(name) {
                s.clone()
            } else if let Some(s) = builtin_sig(resolve_symbol_name(arena, *name)?) {
                s
            } else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("unknown function '{}'", resolve_symbol_name(arena, *name)?),
                });
            };
            if sig.params.len() != args.len() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "function '{}' expects {} args, got {}",
                        resolve_symbol_name(arena, *name)?,
                        sig.params.len(),
                        args.len()
                    ),
                });
            }
            for (i, arg) in args.iter().enumerate() {
                let at = infer_expr_type(*arg, arena, env, table, ret_ty)?;
                if at != sig.params[i] {
                    if sig.params[i] == Type::Fx && is_numeric_for_fx_gap(at) {
                        if !is_fx_literal_expr(*arg, arena) {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "{}; arg {} for '{}' currently requires an fx literal or an existing fx-typed value",
                                    fx_coercion_gap_message(),
                                    i,
                                    resolve_symbol_name(arena, *name)?,
                                ),
                            });
                        }
                    } else {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "arg {} for '{}' has type {:?}, expected {:?}",
                                i,
                                resolve_symbol_name(arena, *name)?,
                                at,
                                sig.params[i]
                            ),
                        });
                    }
                }
            }
            Ok(sig.ret)
        }
        Expr::Unary(op, inner) => {
            let t = infer_expr_type(*inner, arena, env, table, ret_ty)?;
            match op {
                UnaryOp::Not => match t {
                    Type::Quad | Type::Bool => Ok(t),
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!("operator ! unsupported for {:?}", t),
                    }),
                },
                UnaryOp::Pos | UnaryOp::Neg => {
                    if t == Type::F64 {
                        Ok(Type::F64)
                    } else if t == Type::Fx {
                        Err(FrontendError {
                            pos: 0,
                            message: fx_unary_gap_message().to_string(),
                        })
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("operator +/- unsupported for {:?}", t),
                        })
                    }
                }
            }
        }
        Expr::Binary(l, op, r) => {
            let lt = infer_expr_type(*l, arena, env, table, ret_ty)?;
            let rt = infer_expr_type(*r, arena, env, table, ret_ty)?;
            match op {
                BinaryOp::Eq | BinaryOp::Ne => {
                    if lt == rt {
                        Ok(Type::Bool)
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("cannot compare {:?} and {:?}", lt, rt),
                        })
                    }
                }
                BinaryOp::AndAnd | BinaryOp::OrOr => {
                    if lt != rt {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator type mismatch: {:?} vs {:?}", lt, rt),
                        });
                    }
                    match lt {
                        Type::Quad | Type::Bool => Ok(lt),
                        _ => Err(FrontendError {
                            pos: 0,
                            message: format!("operator unsupported for {:?}", lt),
                        }),
                    }
                }
                BinaryOp::Implies => {
                    if lt == Type::Quad && rt == Type::Quad {
                        Ok(Type::Quad)
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: "operator '->' is allowed only for quad".to_string(),
                        })
                    }
                }
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    if lt == Type::Fx && rt == Type::Fx {
                        return Err(FrontendError {
                            pos: 0,
                            message: fx_arithmetic_gap_message().to_string(),
                        });
                    }
                    if lt == Type::F64 && rt == Type::F64 {
                        Ok(Type::F64)
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "f64 arithmetic requires f64 operands, got {:?} and {:?}",
                                lt, rt
                            ),
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typecheck_source(src: &str) -> Result<(), FrontendError> {
        let program = parse_program(src)?;
        type_check_program(&program)
    }

    #[test]
    fn fx_identity_surface_typechecks() {
        let src = r#"
            fn id(x: fx) -> fx {
                let y: fx = x;
                return y;
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("fx passthrough surface should typecheck");
    }

    #[test]
    fn fx_literal_surface_typechecks() {
        let src = r#"
            fn id(x: fx) -> fx {
                return x;
            }

            fn value() -> fx {
                return -1.25;
            }

            fn main() {
                let x: fx = 1.0;
                let y: fx = id(2);
                let z: fx = value();
                let same = x == x;
                let also_same = y == z;
                if same == also_same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("fx literal/call/return surface should typecheck");
    }

    #[test]
    fn fx_arithmetic_reports_explicit_gap() {
        let src = r#"
            fn add(x: fx, y: fx) -> fx {
                return x + y;
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("fx arithmetic should reject");
        assert!(err.message.contains("fx arithmetic is not implemented"));
    }

    #[test]
    fn block_expression_tail_typechecks() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let base: f64 = 1.0;
                    base + 2.0
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("block expression tail should typecheck");
    }

    #[test]
    fn block_expression_scope_does_not_escape() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let base: f64 = 1.0;
                    base + 2.0
                };
                let leak = base;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("block-local name must not escape");
        assert!(err.message.contains("unknown variable 'base'"));
    }

    #[test]
    fn if_expression_typechecks_when_branches_match() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { 2.0 };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("if expression should typecheck");
    }

    #[test]
    fn if_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { true };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("mismatched if expression branches must reject");
        assert!(err
            .message
            .contains("if expression branch type mismatch"));
    }

    #[test]
    fn if_expression_requires_bool_condition() {
        let src = r#"
            fn main() {
                let total: f64 = if T { 1.0 } else { 2.0 };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("quad condition must reject");
        assert!(err
            .message
            .contains("if expression condition must be bool"));
    }

    #[test]
    fn match_expression_typechecks_when_arms_match() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                    F => { 2.0 }
                    _ => { 3.0 }
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("match expression should typecheck");
    }

    #[test]
    fn match_expression_requires_quad_scrutinee() {
        let src = r#"
            fn main() {
                let total: f64 = match true {
                    T => { 1.0 }
                    _ => { 2.0 }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-quad match expression must reject");
        assert!(err
            .message
            .contains("match expression is allowed only for quad scrutinee"));
    }

    #[test]
    fn match_expression_requires_default_arm() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("match expression without default must reject");
        assert!(err
            .message
            .contains("match expression requires default arm '_'"));
    }

    #[test]
    fn match_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                    _ => { true }
                };
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("mismatched match expression branches must reject");
        assert!(err
            .message
            .contains("match expression branch type mismatch"));
    }

    #[test]
    fn match_expression_guard_requires_bool() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T if T => { 1.0 }
                    _ => { 2.0 }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-bool guard must reject");
        assert!(err.message.contains("match guard condition must be bool"));
    }

    #[test]
    fn guard_clause_typechecks_with_unit_return() {
        let src = r#"
            fn main() {
                guard true else return;
                return;
            }
        "#;

        typecheck_source(src).expect("guard clause should typecheck");
    }

    #[test]
    fn guard_clause_requires_bool_condition() {
        let src = r#"
            fn main() {
                guard T else return;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-bool guard clause must reject");
        assert!(err.message.contains("guard clause condition must be bool"));
    }

    #[test]
    fn guard_clause_reuses_return_type_contract() {
        let src = r#"
            fn main() {
                guard true else return true;
            }
        "#;

        let err = typecheck_source(src).expect_err("guard return payload must typecheck");
        assert!(err.message.contains("return type mismatch"));
    }

    #[test]
    fn expression_bodied_function_reuses_return_typing() {
        let src = r#"
            fn id(x: f64) -> f64 = x;

            fn main() {
                let same: f64 = id(1.0);
                let ok = same == same;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("expression-bodied function should typecheck");
    }

    #[test]
    fn expression_bodied_function_reports_return_mismatch() {
        let src = r#"
            fn bad() -> f64 = true;

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("expression-bodied return mismatch must reject");
        assert!(err.message.contains("return type mismatch"));
    }

    #[test]
    fn pipeline_chain_typechecks_via_existing_call_rules() {
        let src = r#"
            fn inc(x: f64) -> f64 = x + 1.0;
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = 1.0 |> inc() |> scale(3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("pipeline desugaring should typecheck");
    }
}

fn infer_value_block_type(
    block: &BlockExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
) -> Result<Type, FrontendError> {
    let mut block_env = env.clone();
    block_env.push_scope();
    for stmt in &block.statements {
        match arena.stmt(*stmt) {
            Stmt::Let { .. } | Stmt::Expr(_) => {
                check_stmt(*stmt, arena, &mut block_env, ret_ty, table)?;
            }
            _ => {
                return Err(FrontendError {
                    pos: 0,
                    message: "value-producing block currently supports only let-bindings and expression statements before the tail value".to_string(),
                });
            }
        }
    }
    let tail_ty = infer_expr_type(block.tail, arena, &block_env, table, ret_ty)?;
    block_env.pop_scope();
    Ok(tail_ty)
}

fn infer_match_expr_type(
    match_expr: &MatchExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
) -> Result<Type, FrontendError> {
    let scrutinee_ty = infer_expr_type(match_expr.scrutinee, arena, env, table, ret_ty)?;
    if scrutinee_ty != Type::Quad {
        return Err(FrontendError {
            pos: 0,
            message: "match expression is allowed only for quad scrutinee".to_string(),
        });
    }
    let default = match_expr.default.as_ref().ok_or(FrontendError {
        pos: 0,
        message: "match expression requires default arm '_'".to_string(),
    })?;

    let mut result_ty = None;
    for arm in &match_expr.arms {
        check_match_guard(arm.guard, arena, env, table, ret_ty)?;
        let arm_ty = infer_value_block_type(&arm.block, arena, env, table, ret_ty)?;
        if let Some(expected) = result_ty {
            if expected != arm_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "match expression branch type mismatch: expected {:?}, got {:?}",
                        expected, arm_ty
                    ),
                });
            }
        } else {
            result_ty = Some(arm_ty);
        }
    }

    let default_ty = infer_value_block_type(default, arena, env, table, ret_ty)?;
    if let Some(expected) = result_ty {
        if expected != default_ty {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "match expression branch type mismatch: expected {:?}, got {:?}",
                    expected, default_ty
                ),
            });
        }
        Ok(expected)
    } else {
        Ok(default_ty)
    }
}

fn check_match_guard(
    guard: Option<ExprId>,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
) -> Result<(), FrontendError> {
    if let Some(expr_id) = guard {
        let guard_ty = infer_expr_type(expr_id, arena, env, table, ret_ty)?;
        if guard_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: "match guard condition must be bool; explicit compare is required for quad"
                    .to_string(),
            });
        }
    }
    Ok(())
}

fn check_return_payload(
    value: Option<ExprId>,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
) -> Result<(), FrontendError> {
    let got = if let Some(expr_id) = value {
        infer_expr_type(expr_id, arena, env, table, ret_ty)?
    } else {
        Type::Unit
    };
    if got != ret_ty {
        if ret_ty == Type::Fx && is_numeric_for_fx_gap(got) {
            if let Some(expr_id) = value {
                if is_fx_literal_expr(expr_id, arena) {
                    return Ok(());
                }
            }
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "{}; function return currently requires an fx literal or an existing fx-typed value",
                    fx_coercion_gap_message(),
                ),
            });
        }
        return Err(FrontendError {
            pos: 0,
            message: format!("return type mismatch: expected {:?}, got {:?}", ret_ty, got),
        });
    }
    Ok(())
}
