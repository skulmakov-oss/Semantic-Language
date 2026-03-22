use crate::*;
use crate::types::NumericLiteral;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::ToString;

fn fx_coercion_gap_message() -> &'static str {
    "fx coercion from non-literal numeric expressions is not implemented in the canonical Rust-like path yet"
}

fn fx_arithmetic_gap_message() -> &'static str {
    "fx arithmetic is not implemented in the canonical Rust-like path yet"
}

fn fx_unary_gap_message() -> &'static str {
    "fx unary +/- is not implemented in the canonical Rust-like path yet"
}

fn is_numeric_for_fx_gap(ty: &Type) -> bool {
    matches!(ty, Type::I32 | Type::U32 | Type::F64)
}

fn is_fx_literal_expr(expr_id: ExprId, arena: &AstArena) -> bool {
    match arena.expr(expr_id) {
        Expr::NumericLiteral(_) => true,
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
            params: func.params.iter().map(|(_, t)| t.clone()).collect(),
            param_names: Some(func.params.iter().map(|(name, _)| *name).collect()),
            param_defaults: Some(func.param_defaults.clone()),
            ret: func.ret.clone(),
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
    if func.params.len() != func.param_defaults.len() {
        return Err(FrontendError {
            pos: 0,
            message: "function parameter/default metadata length mismatch".to_string(),
        });
    }
    let empty_env = ScopeEnv::new();
    let mut default_loop_stack = Vec::new();
    for ((name, ty), default_expr) in func.params.iter().zip(func.param_defaults.iter()) {
        if let Some(default_expr) = default_expr {
            let default_ty =
                infer_expr_type(
                    *default_expr,
                    arena,
                    &empty_env,
                    table,
                    Type::Unit,
                    &mut default_loop_stack,
                )?;
            if let Err(err) = ensure_const_initializer_safe(*default_expr, arena, &empty_env) {
                return Err(FrontendError {
                    pos: err.pos,
                    message: format!(
                        "default parameter '{}' {}",
                        resolve_symbol_name(arena, *name)?,
                        err.message
                    ),
                });
            }
            ensure_binding_value_type(
                ty.clone(),
                default_ty,
                *default_expr,
                arena,
                format!("default parameter '{}'", resolve_symbol_name(arena, *name)?),
            )?;
        }
    }
    let mut env = ScopeEnv::with_params(&func.params);
    let mut loop_stack = Vec::new();
    for stmt in &func.body {
        check_stmt(*stmt, arena, &mut env, func.ret.clone(), table, &mut loop_stack)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopTypeFrame {
    break_ty: Option<Type>,
}

fn check_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<(), FrontendError> {
    let stmt = arena.stmt(stmt_id);
    match stmt {
        Stmt::Const { name, ty, value } => {
            let vt = infer_expr_type(*value, arena, env, table, ret_ty, loop_stack)?;
            ensure_const_initializer_safe(*value, arena, env)?;
            let final_ty = if let Some(ann) = ty {
                ensure_binding_value_type(
                    ann.clone(),
                    vt,
                    *value,
                    arena,
                    format!("const '{}'", resolve_symbol_name(arena, *name)?),
                )?;
                ann.clone()
            } else {
                vt
            };
            env.insert_const(*name, final_ty);
            Ok(())
        }
        Stmt::Let { name, ty, value } => {
            let vt = infer_expr_type(*value, arena, env, table, ret_ty, loop_stack)?;
            let final_ty = if let Some(ann) = ty {
                ensure_binding_value_type(
                    ann.clone(),
                    vt,
                    *value,
                    arena,
                    format!("let '{}'", resolve_symbol_name(arena, *name)?),
                )?;
                ann.clone()
            } else {
                vt
            };
            env.insert(*name, final_ty);
            Ok(())
        }
        Stmt::LetTuple { items, ty, value } => {
            let vt = infer_expr_type(*value, arena, env, table, ret_ty, loop_stack)?;
            let final_ty = if let Some(ann) = ty {
                ensure_binding_value_type(
                    ann.clone(),
                    vt,
                    *value,
                    arena,
                    "tuple destructuring bind".to_string(),
                )?;
                ann.clone()
            } else {
                vt
            };
            let Type::Tuple(item_tys) = final_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: "tuple destructuring bind requires tuple value".to_string(),
                });
            };
            if item_tys.len() != items.len() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "tuple destructuring bind arity mismatch: expected {}, got {}",
                        items.len(),
                        item_tys.len()
                    ),
                });
            }
            for (item, item_ty) in items.iter().zip(item_tys.into_iter()) {
                if let Some(name) = item {
                    env.insert(*name, item_ty);
                }
            }
            Ok(())
        }
        Stmt::Discard { ty, value } => {
            let vt = infer_expr_type(*value, arena, env, table, ret_ty, loop_stack)?;
            if let Some(ann) = ty {
                ensure_binding_value_type(
                    ann.clone(),
                    vt,
                    *value,
                    arena,
                    "discard binding".to_string(),
                )?;
            }
            Ok(())
        }
        Stmt::Assign { name, value } => {
            let target_ty = env.get(*name).ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "unknown assignment target '{}'",
                    resolve_symbol_name(arena, *name)?
                ),
            })?;
            if env.is_const(*name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "cannot assign to const binding '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            let value_ty = infer_expr_type(*value, arena, env, table, ret_ty.clone(), loop_stack)?;
            ensure_binding_value_type(
                target_ty,
                value_ty,
                *value,
                arena,
                format!("assignment to '{}'", resolve_symbol_name(arena, *name)?),
            )
        }
        Stmt::AssignTuple { items, value } => {
            let value_ty = infer_expr_type(*value, arena, env, table, ret_ty.clone(), loop_stack)?;
            let Type::Tuple(item_tys) = value_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: "tuple destructuring assignment requires tuple value".to_string(),
                });
            };
            if item_tys.len() != items.len() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "tuple destructuring assignment arity mismatch: expected {}, got {}",
                        items.len(),
                        item_tys.len()
                    ),
                });
            }
            for (item, item_ty) in items.iter().zip(item_tys.into_iter()) {
                let Some(name) = item else {
                    continue;
                };
                let target_ty = env.get(*name).ok_or(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown tuple assignment target '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                })?;
                if env.is_const(*name) {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "cannot assign to const binding '{}' in tuple destructuring assignment",
                            resolve_symbol_name(arena, *name)?
                        ),
                    });
                }
                ensure_binding_value_type(
                    target_ty,
                    item_ty,
                    *value,
                    arena,
                    format!(
                        "tuple assignment to '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                )?;
            }
            Ok(())
        }
        Stmt::Break(value) => {
            let break_ty = infer_expr_type(*value, arena, env, table, ret_ty, loop_stack)?;
            let frame = loop_stack.last_mut().ok_or(FrontendError {
                pos: 0,
                message: "break with value is allowed only inside loop expression".to_string(),
            })?;
            if let Some(expected) = &frame.break_ty {
                if *expected != break_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "loop expression break type mismatch: expected {:?}, got {:?}",
                            expected, break_ty
                        ),
                    });
                }
            } else {
                frame.break_ty = Some(break_ty);
            }
            Ok(())
        }
        Stmt::Guard {
            condition,
            else_return,
        } => {
            let condition_ty =
                infer_expr_type(*condition, arena, env, table, ret_ty.clone(), loop_stack)?;
            if condition_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "guard clause condition must be bool; explicit compare is required for quad"
                            .to_string(),
                });
            }
            check_return_payload(*else_return, arena, env, table, ret_ty, loop_stack)
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            let ct = infer_expr_type(*condition, arena, env, table, ret_ty.clone(), loop_stack)?;
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
                check_stmt(*s, arena, &mut then_env, ret_ty.clone(), table, loop_stack)?;
            }
            then_env.pop_scope();

            let mut else_env = env.clone();
            else_env.push_scope();
            for s in else_block {
                check_stmt(*s, arena, &mut else_env, ret_ty.clone(), table, loop_stack)?;
            }
            else_env.pop_scope();
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            let st = infer_expr_type(*scrutinee, arena, env, table, ret_ty.clone(), loop_stack)?;
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
                check_match_guard(arm.guard, arena, &arm_env, table, ret_ty.clone(), loop_stack)?;
                for s in &arm.block {
                    check_stmt(*s, arena, &mut arm_env, ret_ty.clone(), table, loop_stack)?;
                }
                arm_env.pop_scope();
            }

            let mut def_env = env.clone();
            def_env.push_scope();
            for s in default {
                check_stmt(*s, arena, &mut def_env, ret_ty.clone(), table, loop_stack)?;
            }
            def_env.pop_scope();
            Ok(())
        }
        Stmt::Return(v) => check_return_payload(*v, arena, env, table, ret_ty, loop_stack),
        Stmt::Expr(e) => {
            if check_builtin_assert_stmt(*e, arena, env, table, ret_ty.clone(), loop_stack)? {
                return Ok(());
            }
            let _ = infer_expr_type(*e, arena, env, table, ret_ty, loop_stack)?;
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
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<Type, FrontendError> {
    let expr = arena.expr(expr_id);
    match expr {
        Expr::QuadLiteral(_) => Ok(Type::Quad),
        Expr::BoolLiteral(_) => Ok(Type::Bool),
        Expr::NumericLiteral(literal) => match literal {
            NumericLiteral::I32(_) => Ok(Type::I32),
            NumericLiteral::U32(_) => Ok(Type::U32),
            NumericLiteral::F64(_) => Ok(Type::F64),
            NumericLiteral::Fx(_) => Ok(Type::Fx),
        },
        Expr::Tuple(items) => {
            let mut item_tys = Vec::with_capacity(items.len());
            for item in items {
                item_tys.push(infer_expr_type(
                    *item,
                    arena,
                    env,
                    table,
                    ret_ty.clone(),
                    loop_stack,
                )?);
            }
            Ok(Type::Tuple(item_tys))
        }
        Expr::Var(v) => env.get(*v).ok_or(FrontendError {
            pos: 0,
            message: format!("unknown variable '{}'", resolve_symbol_name(arena, *v)?),
        }),
        Expr::Block(block) => infer_value_block_type(block, arena, env, table, ret_ty, loop_stack),
        Expr::If(if_expr) => {
            let cond_ty =
                infer_expr_type(if_expr.condition, arena, env, table, ret_ty.clone(), loop_stack)?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "if expression condition must be bool; explicit compare is required for quad"
                            .to_string(),
                });
            }
            let then_ty =
                infer_value_block_type(
                    &if_expr.then_block,
                    arena,
                    env,
                    table,
                    ret_ty.clone(),
                    loop_stack,
                )?;
            let else_ty =
                infer_value_block_type(
                    &if_expr.else_block,
                    arena,
                    env,
                    table,
                    ret_ty.clone(),
                    loop_stack,
                )?;
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
        Expr::Match(match_expr) => {
            infer_match_expr_type(match_expr, arena, env, table, ret_ty, loop_stack)
        }
        Expr::Loop(loop_expr) => infer_loop_expr_type(loop_expr, arena, env, table, ret_ty, loop_stack),
        Expr::Call(name, args) => {
            if is_builtin_assert_name(*name, arena, table)? {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "assert builtin is statement-only and cannot be used as expression value"
                            .to_string(),
                });
            }
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
            let ordered_args = reorder_call_args(*name, args, &sig, arena)?;
            for (i, arg) in ordered_args.iter().enumerate() {
                let at = infer_expr_type(*arg, arena, env, table, ret_ty.clone(), loop_stack)?;
                let expected_ty = sig.params[i].clone();
                if at != expected_ty {
                    if expected_ty == Type::Fx && is_numeric_for_fx_gap(&at) {
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
                                expected_ty
                            ),
                        });
                    }
                }
            }
            Ok(sig.ret.clone())
        }
        Expr::Unary(op, inner) => {
            let t = infer_expr_type(*inner, arena, env, table, ret_ty.clone(), loop_stack)?;
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
                    } else if t == Type::Fx && is_fx_literal_expr(expr_id, arena) {
                        Ok(Type::Fx)
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
            let lt = infer_expr_type(*l, arena, env, table, ret_ty.clone(), loop_stack)?;
            let rt = infer_expr_type(*r, arena, env, table, ret_ty.clone(), loop_stack)?;
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
    fn extended_numeric_literal_surface_typechecks() {
        let src = r#"
            fn main() {
                let decimal: i32 = 1_000;
                let hex: i32 = 0xff;
                let unsigned: u32 = 1_000u32;
                let fx_value: fx = 1.25fx;
                let neg_fx: fx = -1.25fx;
                let same = unsigned == unsigned;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("extended numeric literal surface should typecheck");
    }

    #[test]
    fn explicit_fx_literal_bypasses_f64_gap_at_same_type() {
        let src = r#"
            fn main() {
                let value: fx = 2fx;
                let same = value == value;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("explicit fx literal should typecheck as fx directly");
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
        assert!(err.message.contains("if expression branch type mismatch"));
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
        assert!(err.message.contains("if expression condition must be bool"));
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

    #[test]
    fn named_arguments_typecheck_via_parameter_reorder() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(factor = 3.0, x = 2.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("named arguments should typecheck");
    }

    #[test]
    fn pipeline_named_arguments_typecheck_after_positional_prefix() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = 2.0 |> scale(factor = 3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("pipeline named arguments should typecheck");
    }

    #[test]
    fn default_parameters_fill_omitted_trailing_arguments() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("default parameters should fill omitted trailing arguments");
    }

    #[test]
    fn named_arguments_can_override_remaining_default_parameters() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(x = 3.0, factor = 4.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("named arguments should override defaulted parameters");
    }

    #[test]
    fn builtin_named_arguments_are_rejected() {
        let src = r#"
            fn main() {
                let total: f64 = sqrt(x = 4.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("builtin named arguments must reject");
        assert!(err
            .message
            .contains("named arguments are not supported for builtin 'sqrt'"));
    }

    #[test]
    fn duplicate_named_arguments_are_rejected() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(x = 2.0, x = 3.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("duplicate named arguments must reject");
        assert!(err
            .message
            .contains("duplicate named argument 'x'"));
    }

    #[test]
    fn missing_named_argument_is_rejected() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(x = 2.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("missing named argument must reject");
        assert!(err
            .message
            .contains("is missing argument for parameter 'factor'"));
    }

    #[test]
    fn required_parameter_still_rejects_when_default_is_missing() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale();
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("required non-default parameter must reject");
        assert!(err
            .message
            .contains("is missing argument for parameter 'x'"));
    }

    #[test]
    fn default_parameter_initializer_must_be_const_safe() {
        let src = r#"
            fn scale(x: f64, factor: f64 = sqrt(4.0)) -> f64 = x * factor;

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-const-safe default parameter must reject");
        assert!(err
            .message
            .contains("default parameter 'factor'"));
    }

    #[test]
    fn default_parameter_initializer_cannot_reference_previous_parameter() {
        let src = r#"
            fn scale(x: f64, factor: f64 = x) -> f64 = x * factor;

            fn main() {
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("default parameter cannot reference earlier param");
        assert!(err.message.contains("'x'"));
    }

    #[test]
    fn immediate_short_lambda_typechecks_via_block_desugaring() {
        let src = r#"
            fn main() {
                let total: f64 = (x => x + 1.0)(2.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("immediate short lambda should typecheck");
    }

    #[test]
    fn pipeline_short_lambda_typechecks_via_block_desugaring() {
        let src = r#"
            fn main() {
                let total: f64 = 2.0 |> (x => x + 1.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("pipeline short lambda should typecheck");
    }

    #[test]
    fn const_declaration_typechecks_for_literal_expression_subset() {
        let src = r#"
            fn main() {
                const two: f64 = 1.0 + 1.0;
                const four: f64 = two + two;
                let ok = four == four;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("const declarations should typecheck");
    }

    #[test]
    fn const_declaration_rejects_non_const_initializer() {
        let src = r#"
            fn main() {
                let base: f64 = 1.0;
                const total: f64 = base + 1.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("const initializer must reject runtime binding");
        assert!(err.message.contains("is not const"));
    }

    #[test]
    fn const_binding_rejects_assignment_target() {
        let src = r#"
            fn main() {
                const total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assignment to const must reject");
        assert!(err.message.contains("cannot assign to const binding 'total'"));
    }

    #[test]
    fn const_declaration_is_allowed_inside_value_block_body() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    const offset: f64 = 2.0;
                    1.0 + offset
                };
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("const should be accepted in value block body");
    }

    #[test]
    fn captureful_short_lambda_is_rejected() {
        let src = r#"
            fn main() {
                let offset: f64 = 1.0;
                let total: f64 = (x => x + offset)(2.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("captureful short lambda must reject");
        assert!(err.message.contains("capture-free only"));
    }

    #[test]
    fn compound_assignment_typechecks_for_existing_scalar_rules() {
        let src = r#"
            fn main() {
                let total: f64 = 1.0;
                total += 2.0;
                let ready: bool = true;
                ready &&= false;
                return;
            }
        "#;

        typecheck_source(src).expect("compound assignment should typecheck");
    }

    #[test]
    fn compound_assignment_requires_existing_binding() {
        let src = r#"
            fn main() {
                total += 1.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown assignment target must reject");
        assert!(err.message.contains("unknown assignment target 'total'"));
    }

    #[test]
    fn compound_assignment_reuses_operator_type_rules() {
        let src = r#"
            fn main() {
                let total: f64 = 1.0;
                total += true;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("compound assignment operator mismatch must reject");
        assert!(err.message.contains("f64 arithmetic requires f64 operands"));
    }

    #[test]
    fn repeated_discard_binds_typecheck_without_name_collisions() {
        let src = r#"
            fn main() {
                let _ = 1.0;
                let _ = 2.0;
                return;
            }
        "#;

        typecheck_source(src).expect("discard binds should not create conflicting visible names");
    }

    #[test]
    fn typed_discard_bind_reuses_type_mismatch_rules() {
        let src = r#"
            fn main() {
                let _: f64 = true;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("typed discard bind must check rhs type");
        assert!(err.message.contains("discard binding"));
    }

    #[test]
    fn discard_bind_is_allowed_inside_value_block_body() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let _ = 1.0;
                    2.0
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("discard bind should be accepted in value block body");
    }

    #[test]
    fn assert_builtin_statement_typechecks() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;

        typecheck_source(src).expect("assert builtin statement should typecheck");
    }

    #[test]
    fn assert_builtin_requires_bool_condition() {
        let src = r#"
            fn main() {
                assert(1.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assert builtin must require bool");
        assert!(err
            .message
            .contains("assert builtin requires bool condition"));
    }

    #[test]
    fn assert_builtin_requires_single_argument() {
        let src = r#"
            fn main() {
                assert(true, false);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assert builtin arity must reject");
        assert!(err.message.contains("assert builtin expects 1 arg"));
    }

    #[test]
    fn assert_builtin_is_statement_only() {
        let src = r#"
            fn main() {
                let ok: bool = assert(true);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assert builtin should reject value position");
        assert!(err
            .message
            .contains("assert builtin is statement-only and cannot be used as expression value"));
    }

    #[test]
    fn tuple_literals_and_tuple_types_typecheck_through_call_and_return_paths() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) {
                return (1, flag);
            }

            fn main() {
                let left: (i32, bool) = pair(true);
                let right: (i32, bool) = (1, true);
                assert(left == right);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple literal/type surface should typecheck");
    }

    #[test]
    fn tuple_destructuring_bind_typechecks() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let (count, ready): (i32, bool) = pair(true);
                assert(ready == true);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple destructuring bind should typecheck");
    }

    #[test]
    fn tuple_destructuring_bind_rejects_non_tuple_value() {
        let src = r#"
            fn main() {
                let (count, ready) = 1;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-tuple destructuring must reject");
        assert!(err.message.contains("tuple destructuring bind requires tuple value"));
    }

    #[test]
    fn tuple_destructuring_assignment_typechecks() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let count: i32 = 0;
                let ready: bool = false;
                (count, ready) = pair(true);
                assert(count == 1);
                assert(ready == true);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple destructuring assignment should typecheck");
    }

    #[test]
    fn tuple_destructuring_assignment_rejects_unknown_target() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let count: i32 = 0;
                (count, ready) = pair(true);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown tuple assignment target must reject");
        assert!(err
            .message
            .contains("unknown tuple assignment target 'ready'"));
    }

    #[test]
    fn where_clause_typechecks_via_block_desugaring() {
        let src = r#"
            fn magnitude_sq(x: f64, y: f64) -> f64 =
                total where
                    xx = x * x,
                    yy = y * y,
                    total = xx + yy;

            fn main() {
                let value: f64 = magnitude_sq(3.0, 4.0);
                return;
            }
        "#;

        typecheck_source(src).expect("where-clause should typecheck");
    }

    #[test]
    fn where_clause_reuses_let_type_mismatch_rules() {
        let src = r#"
            fn main() {
                let value: f64 = total where total: bool = true;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("typed where binding mismatch must reject");
        assert!(err.message.contains("type mismatch in let"));
    }

    #[test]
    fn loop_expression_typechecks_with_break_value() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    if true {
                        break 1.0;
                    } else {
                        break 2.0;
                    }
                };
                return;
            }
        "#;

        typecheck_source(src).expect("loop expression should typecheck");
    }

    #[test]
    fn loop_expression_rejects_break_outside_loop() {
        let src = r#"
            fn main() {
                break 1.0;
            }
        "#;

        let err = typecheck_source(src).expect_err("break outside loop must reject");
        assert!(err
            .message
            .contains("break with value is allowed only inside loop expression"));
    }

    #[test]
    fn loop_expression_rejects_mismatched_break_types() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    if true {
                        break 1.0;
                    } else {
                        break true;
                    }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("mismatched break types must reject");
        assert!(err.message.contains("loop expression break type mismatch"));
    }

    #[test]
    fn loop_expression_rejects_return_in_body() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    return;
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("return in loop expression body must reject");
        assert!(err
            .message
            .contains("loop expression body currently does not allow guard clause or return"));
    }

    #[test]
    fn ufcs_method_call_typechecks_via_ordinary_call_contract() {
        let src = r#"
            fn scale(value: f64, factor: f64) -> f64 = value * factor;

            fn main() {
                let total: f64 = 2.0.scale(3.0);
                return;
            }
        "#;

        typecheck_source(src).expect("UFCS method-call sugar should typecheck");
    }

    #[test]
    fn ufcs_named_arguments_reuse_parameter_reorder_rules() {
        let src = r#"
            fn clamp(value: f64, min: f64, max: f64) -> f64 = value;

            fn main() {
                let total: f64 = 2.0.clamp(min = 0.0, max = 10.0);
                return;
            }
        "#;

        typecheck_source(src).expect("UFCS named arguments should typecheck");
    }

    #[test]
    fn ufcs_builtin_named_arguments_still_reject() {
        let src = r#"
            fn main() {
                let total: f64 = 2.0.pow(exp = 3.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("builtin named arguments must still reject");
        assert!(err
            .message
            .contains("named arguments are not supported for builtin 'pow'"));
    }
}

fn is_builtin_assert_name(
    name: SymbolId,
    arena: &AstArena,
    table: &FnTable,
) -> Result<bool, FrontendError> {
    Ok(!table.contains_key(&name) && resolve_symbol_name(arena, name)? == "assert")
}

fn check_builtin_assert_stmt(
    expr_id: ExprId,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<bool, FrontendError> {
    let Expr::Call(name, args) = arena.expr(expr_id) else {
        return Ok(false);
    };
    if !is_builtin_assert_name(*name, arena, table)? {
        return Ok(false);
    }
    if args.len() != 1 {
        return Err(FrontendError {
            pos: 0,
            message: format!("assert builtin expects 1 arg, got {}", args.len()),
        });
    }
    let cond_ty = infer_expr_type(args[0].value, arena, env, table, ret_ty, loop_stack)?;
    if cond_ty != Type::Bool {
        return Err(FrontendError {
            pos: 0,
            message: format!("assert builtin requires bool condition, got {:?}", cond_ty),
        });
    }
    Ok(true)
}

fn infer_value_block_type(
    block: &BlockExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<Type, FrontendError> {
    let mut block_env = env.clone();
    block_env.push_scope();
    for stmt in &block.statements {
        match arena.stmt(*stmt) {
            Stmt::Const { .. }
            | Stmt::Let { .. }
            | Stmt::LetTuple { .. }
            | Stmt::Discard { .. }
            | Stmt::Expr(_) => {
                check_stmt(*stmt, arena, &mut block_env, ret_ty.clone(), table, loop_stack)?;
            }
            _ => {
                return Err(FrontendError {
                    pos: 0,
                    message: "value-producing block currently supports only const-bindings, let-bindings, discard binds, and expression statements before the tail value".to_string(),
                });
            }
        }
    }
    let tail_ty = infer_expr_type(block.tail, arena, &block_env, table, ret_ty, loop_stack)?;
    block_env.pop_scope();
    Ok(tail_ty)
}

fn infer_match_expr_type(
    match_expr: &MatchExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<Type, FrontendError> {
    let scrutinee_ty =
        infer_expr_type(match_expr.scrutinee, arena, env, table, ret_ty.clone(), loop_stack)?;
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
        check_match_guard(arm.guard, arena, env, table, ret_ty.clone(), loop_stack)?;
        let arm_ty = infer_value_block_type(
            &arm.block,
            arena,
            env,
            table,
            ret_ty.clone(),
            loop_stack,
        )?;
        if let Some(ref expected) = result_ty {
            if *expected != arm_ty {
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

    let default_ty = infer_value_block_type(default, arena, env, table, ret_ty, loop_stack)?;
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

fn infer_loop_expr_type(
    loop_expr: &LoopExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<Type, FrontendError> {
    let mut body_env = env.clone();
    body_env.push_scope();
    loop_stack.push(LoopTypeFrame { break_ty: None });
    for stmt in &loop_expr.body {
        check_loop_expr_stmt(*stmt, arena, &mut body_env, table, ret_ty.clone(), loop_stack)?;
    }
    body_env.pop_scope();
    let frame = loop_stack.pop().expect("loop frame must exist");
    frame.break_ty.ok_or(FrontendError {
        pos: 0,
        message: "loop expression requires at least one break value".to_string(),
    })
}

fn check_loop_expr_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<(), FrontendError> {
    match arena.stmt(stmt_id) {
        Stmt::Guard { .. } | Stmt::Return(..) => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow guard clause or return"
                .to_string(),
        }),
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            let cond_ty =
                infer_expr_type(*condition, arena, env, table, ret_ty.clone(), loop_stack)?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool; explicit compare is required for quad"
                        .to_string(),
                });
            }

            let mut then_env = env.clone();
            then_env.push_scope();
            for stmt in then_block {
                check_loop_expr_stmt(*stmt, arena, &mut then_env, table, ret_ty.clone(), loop_stack)?;
            }
            then_env.pop_scope();

            let mut else_env = env.clone();
            else_env.push_scope();
            for stmt in else_block {
                check_loop_expr_stmt(*stmt, arena, &mut else_env, table, ret_ty.clone(), loop_stack)?;
            }
            else_env.pop_scope();
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            let st = infer_expr_type(*scrutinee, arena, env, table, ret_ty.clone(), loop_stack)?;
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
                check_match_guard(arm.guard, arena, &arm_env, table, ret_ty.clone(), loop_stack)?;
                for stmt in &arm.block {
                    check_loop_expr_stmt(*stmt, arena, &mut arm_env, table, ret_ty.clone(), loop_stack)?;
                }
                arm_env.pop_scope();
            }

            let mut def_env = env.clone();
            def_env.push_scope();
            for stmt in default {
                check_loop_expr_stmt(*stmt, arena, &mut def_env, table, ret_ty.clone(), loop_stack)?;
            }
            def_env.pop_scope();
            Ok(())
        }
        _ => check_stmt(stmt_id, arena, env, ret_ty, table, loop_stack),
    }
}

fn check_match_guard(
    guard: Option<ExprId>,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<(), FrontendError> {
    if let Some(expr_id) = guard {
        let guard_ty = infer_expr_type(expr_id, arena, env, table, ret_ty, loop_stack)?;
        if guard_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message:
                    "match guard condition must be bool; explicit compare is required for quad"
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
    loop_stack: &mut Vec<LoopTypeFrame>,
) -> Result<(), FrontendError> {
    let got = if let Some(expr_id) = value {
        infer_expr_type(expr_id, arena, env, table, ret_ty.clone(), loop_stack)?
    } else {
        Type::Unit
    };
    if got != ret_ty {
        if ret_ty == Type::Fx && is_numeric_for_fx_gap(&got) {
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

fn ensure_binding_value_type(
    expected: Type,
    actual: Type,
    value_expr: ExprId,
    arena: &AstArena,
    context: String,
) -> Result<(), FrontendError> {
    if expected == actual {
        return Ok(());
    }
    if expected == Type::Fx && is_numeric_for_fx_gap(&actual) {
        if is_fx_literal_expr(value_expr, arena) {
            return Ok(());
        }
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "{}; {} currently accepts only fx literals or existing fx-typed values",
                fx_coercion_gap_message(),
                context,
            ),
        });
    }
    Err(FrontendError {
        pos: 0,
        message: format!(
            "type mismatch in {}: {:?} vs {:?}",
            context, expected, actual
        ),
    })
}

fn ensure_const_initializer_safe(
    expr_id: ExprId,
    arena: &AstArena,
    env: &ScopeEnv,
) -> Result<(), FrontendError> {
    match arena.expr(expr_id) {
        Expr::QuadLiteral(_) | Expr::BoolLiteral(_) | Expr::NumericLiteral(_) => Ok(()),
        Expr::Tuple(items) => {
            for item in items {
                ensure_const_initializer_safe(*item, arena, env)?;
            }
            Ok(())
        }
        Expr::Var(name) => {
            if env.is_const(*name) {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "const initializer currently allows only literals, unary/binary operations, and references to earlier const bindings; '{}' is not const",
                        resolve_symbol_name(arena, *name)?
                    ),
                })
            }
        }
        Expr::Unary(_, inner) => ensure_const_initializer_safe(*inner, arena, env),
        Expr::Binary(lhs, _, rhs) => {
            ensure_const_initializer_safe(*lhs, arena, env)?;
            ensure_const_initializer_safe(*rhs, arena, env)
        }
        _ => Err(FrontendError {
            pos: 0,
            message: "const initializer currently supports only pure literal/const expression forms"
                .to_string(),
        }),
    }
}
