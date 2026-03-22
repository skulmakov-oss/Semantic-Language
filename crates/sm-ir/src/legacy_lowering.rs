use super::*;
use crate::semcode_format::{
    write_f64_le, write_i32_le, write_u16_le, write_u32_le, Opcode, MAGIC0, MAGIC1, MAGIC2,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IrInstr {
    Label {
        name: String,
    },
    LoadQ {
        dst: u16,
        val: QuadVal,
    },
    LoadBool {
        dst: u16,
        val: bool,
    },
    LoadI32 {
        dst: u16,
        val: i32,
    },
    LoadF64 {
        dst: u16,
        val: f64,
    },
    LoadFx {
        dst: u16,
        val: i32,
    },
    LoadVar {
        dst: u16,
        name: String,
    },
    StoreVar {
        name: String,
        src: u16,
    },
    QAnd {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QOr {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QNot {
        dst: u16,
        src: u16,
    },
    QImpl {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    BoolAnd {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    BoolOr {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    BoolNot {
        dst: u16,
        src: u16,
    },
    CmpEq {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    CmpNe {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    AddF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    SubF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    MulF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    DivF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    Jmp {
        label: String,
    },
    JmpIf {
        cond: u16,
        label: String,
    },
    Assert {
        cond: u16,
    },
    Call {
        dst: Option<u16>,
        name: String,
        args: Vec<u16>,
    },
    GateRead {
        dst: u16,
        device_id: u16,
        port: u16,
    },
    GateWrite {
        device_id: u16,
        port: u16,
        src: u16,
    },
    PulseEmit {
        signal: String,
    },
    Ret {
        src: Option<u16>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub instrs: Vec<IrInstr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImmutableIrProgram {
    funcs: Vec<IrFunction>,
}

impl ImmutableIrProgram {
    pub fn from_vec(funcs: Vec<IrFunction>) -> Self {
        Self { funcs }
    }

    pub fn functions(&self) -> &[IrFunction] {
        &self.funcs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogosIrLaw {
    pub name: String,
    pub priority: u32,
    pub when_count: usize,
}

const FX_SCALE: i32 = 1_000;

fn encode_fx_literal(value: f64) -> Result<i32, FrontendError> {
    let scaled = value * FX_SCALE as f64;
    if !scaled.is_finite() {
        return Err(FrontendError {
            pos: 0,
            message: "fx literal is not finite".to_string(),
        });
    }
    let rounded = scaled.round();
    if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
        return Err(FrontendError {
            pos: 0,
            message: "fx literal is out of range for the v1 fixed-point carrier".to_string(),
        });
    }
    Ok(rounded as i32)
}

fn try_encode_fx_literal_expr(
    expr_id: ExprId,
    arena: &AstArena,
) -> Result<Option<i32>, FrontendError> {
    match arena.expr(expr_id) {
        Expr::Num(n) => {
            let value = i32::try_from(*n).map_err(|_| FrontendError {
                pos: 0,
                message: format!("numeric literal {} does not fit in i32", n),
            })?;
            value
                .checked_mul(FX_SCALE)
                .ok_or(FrontendError {
                    pos: 0,
                    message: "fx literal is out of range for the v1 fixed-point carrier"
                        .to_string(),
                })
                .map(Some)
        }
        Expr::Float(n) => encode_fx_literal(*n).map(Some),
        Expr::Unary(UnaryOp::Pos, inner) => try_encode_fx_literal_expr(*inner, arena),
        Expr::Unary(UnaryOp::Neg, inner) => {
            let Some(value) = try_encode_fx_literal_expr(*inner, arena)? else {
                return Ok(None);
            };
            value
                .checked_neg()
                .ok_or(FrontendError {
                    pos: 0,
                    message: "fx literal is out of range for the v1 fixed-point carrier"
                        .to_string(),
                })
                .map(Some)
        }
        _ => Ok(None),
    }
}

fn is_builtin_assert_name(
    name: SymbolId,
    arena: &AstArena,
    fn_table: &FnTable,
) -> Result<bool, FrontendError> {
    Ok(!fn_table.contains_key(&name) && resolve_symbol_name(arena, name)? == "assert")
}

pub fn lower_logos_laws_to_ir(program: &LogosProgram) -> Vec<LogosIrLaw> {
    let mut laws = program.laws.clone();
    laws.sort_by(|a, b| b.priority.cmp(&a.priority));
    laws.into_iter()
        .map(|law| LogosIrLaw {
            name: law.name,
            priority: law.priority,
            when_count: law.whens.len(),
        })
        .collect()
}

pub fn lower_expr_to_ir(
    expr: ExprId,
    arena: &AstArena,
    var_types: &HashMap<SymbolId, Type>,
    fn_table: &FnTable,
) -> Result<Vec<IrInstr>, FrontendError> {
    let mut out = Vec::new();
    let mut next = 0u16;
    let mut env = ScopeEnv::new();
    for (name, ty) in var_types {
        env.insert(*name, *ty);
    }
    let _ = lower_expr(expr, arena, &mut next, &mut out, &env, fn_table, Type::Unit)?;
    Ok(out)
}

pub fn lower_function_to_ir(
    func: &Function,
    arena: &AstArena,
    fn_table: &FnTable,
) -> Result<IrFunction, FrontendError> {
    type_check_function_with_table(func, arena, fn_table)?;

    let mut ctx = LoweringCtx::new();
    let mut env = ScopeEnv::with_params(&func.params);
    ctx.next_reg = u16::try_from(func.params.len()).map_err(|_| FrontendError {
        pos: 0,
        message: "too many function parameters for register space".to_string(),
    })?;
    for (idx, (name, _)) in func.params.iter().enumerate() {
        ctx.instrs.push(IrInstr::StoreVar {
            name: resolve_symbol_name(arena, *name)?.to_string(),
            src: idx as u16,
        });
    }
    for stmt in &func.body {
        lower_stmt(*stmt, arena, &mut ctx, &mut env, func.ret, fn_table)?;
    }

    if !ctx.ends_with_ret() {
        if func.ret == Type::Unit {
            ctx.instrs.push(IrInstr::Ret { src: None });
        } else {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{}' may exit without returning {:?}",
                    resolve_symbol_name(arena, func.name)?,
                    func.ret
                ),
            });
        }
    }

    Ok(IrFunction {
        name: resolve_symbol_name(arena, func.name)?.to_string(),
        instrs: ctx.instrs,
    })
}

pub fn compile_program_to_ir(input: &str) -> Result<Vec<IrFunction>, FrontendError> {
    let profile = ParserProfile::foundation_default();
    compile_program_to_ir_with_options_and_profile(
        input,
        CompileProfile::RustLike,
        OptLevel::O0,
        &profile,
    )
}

pub fn compile_program_to_immutable_ir(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<ImmutableIrProgram, FrontendError> {
    let parser_profile = ParserProfile::foundation_default();
    Ok(ImmutableIrProgram::from_vec(
        compile_program_to_ir_with_options_and_profile(input, profile, opt, &parser_profile)?,
    ))
}

pub fn compile_program_to_ir_with_options(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<Vec<IrFunction>, FrontendError> {
    let parser_profile = ParserProfile::foundation_default();
    compile_program_to_ir_with_options_and_profile(input, profile, opt, &parser_profile)
}

pub fn compile_program_to_ir_with_profile(
    input: &str,
    parser_profile: &ParserProfile,
) -> Result<Vec<IrFunction>, FrontendError> {
    compile_program_to_ir_with_options_and_profile(
        input,
        CompileProfile::RustLike,
        OptLevel::O0,
        parser_profile,
    )
}

pub fn compile_program_to_ir_with_options_and_profile(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
    parser_profile: &ParserProfile,
) -> Result<Vec<IrFunction>, FrontendError> {
    match profile {
        CompileProfile::RustLike if !cfg!(feature = "profile-rust") => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "RustLike profile is disabled at compile time (enable feature 'profile-rust')"
                        .to_string(),
            });
        }
        CompileProfile::Logos if !cfg!(feature = "profile-logos") => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "Logos profile is disabled at compile time (enable feature 'profile-logos')"
                        .to_string(),
            });
        }
        _ => {}
    }
    let logos_detected = parse_logos_program_with_profile(input, parser_profile)
        .map(|p| p.system.is_some() || !p.entities.is_empty() || !p.laws.is_empty())
        .unwrap_or(false);
    if (matches!(profile, CompileProfile::Logos)
        || (matches!(profile, CompileProfile::Auto) && logos_detected))
        && cfg!(feature = "profile-logos")
    {
        return Err(FrontendError {
            pos: 0,
            message: "Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend".to_string(),
        });
    }
    if matches!(profile, CompileProfile::Auto) && logos_detected && !cfg!(feature = "profile-logos")
    {
        return Err(FrontendError {
            pos: 0,
            message: "Logos input detected, but Logos profile is disabled at compile time"
                .to_string(),
        });
    }
    if !cfg!(feature = "profile-rust") {
        return Err(FrontendError {
            pos: 0,
            message: "RustLike lowering is disabled at compile time".to_string(),
        });
    }
    let program = parse_program_with_profile(input, parser_profile)?;
    let fn_table = build_fn_table(&program)?;
    type_check_program(&program)?;
    let mut out = Vec::new();
    for f in &program.functions {
        out.push(lower_function_to_ir(f, &program.arena, &fn_table)?);
    }
    if matches!(opt, OptLevel::O1) {
        let _ = crate::passes::run_default_opt_passes(&mut out);
    }
    Ok(out)
}

pub fn compile_program_to_ir_optimized(input: &str) -> Result<Vec<IrFunction>, FrontendError> {
    let profile = ParserProfile::foundation_default();
    compile_program_to_ir_with_options_and_profile(
        input,
        CompileProfile::RustLike,
        OptLevel::O1,
        &profile,
    )
}

pub fn validate_ir(f: &IrFunction) -> Result<(), FrontendError> {
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut has_ret = false;

    for (idx, instr) in f.instrs.iter().enumerate() {
        if let IrInstr::Label { name } = instr {
            if labels.insert(name.clone(), idx).is_some() {
                return Err(FrontendError {
                    pos: idx,
                    message: format!("duplicate label '{}' in '{}'", name, f.name),
                });
            }
        }
        if matches!(instr, IrInstr::Ret { .. }) {
            has_ret = true;
        }
    }

    if !has_ret {
        return Err(FrontendError {
            pos: 0,
            message: format!("function '{}' has no RET", f.name),
        });
    }

    for (idx, instr) in f.instrs.iter().enumerate() {
        match instr {
            IrInstr::Jmp { label } | IrInstr::JmpIf { label, .. } => {
                if !labels.contains_key(label) {
                    return Err(FrontendError {
                        pos: idx,
                        message: format!("jump to unknown label '{}' in '{}'", label, f.name),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn compile_program_to_semcode(input: &str) -> Result<Vec<u8>, FrontendError> {
    compile_program_to_semcode_with_options(input, CompileProfile::RustLike, OptLevel::O0)
}

pub fn compile_program_to_semcode_with_options(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<Vec<u8>, FrontendError> {
    compile_program_to_semcode_with_options_debug(input, profile, opt, false)
}

pub fn compile_program_to_semcode_with_options_debug(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
    debug_symbols: bool,
) -> Result<Vec<u8>, FrontendError> {
    if debug_symbols && !cfg!(feature = "debug-symbols") {
        return Err(FrontendError {
            pos: 0,
            message: "debug symbols are disabled at compile time (enable feature 'debug-symbols')"
                .to_string(),
        });
    }
    let ir = compile_program_to_immutable_ir(input, profile, opt)?;
    for f in ir.functions() {
        validate_ir(f)?;
    }
    emit_semcode(ir.functions(), debug_symbols)
}

pub fn emit_ir_to_semcode(
    funcs: &[IrFunction],
    debug_symbols: bool,
) -> Result<Vec<u8>, FrontendError> {
    emit_semcode(funcs, debug_symbols)
}

fn emit_semcode(funcs: &[IrFunction], debug_symbols: bool) -> Result<Vec<u8>, FrontendError> {
    let mut out = Vec::new();
    if has_v2_fx_instr(funcs) {
        out.extend_from_slice(&MAGIC2);
    } else if has_v1_math_instr(funcs) {
        out.extend_from_slice(&MAGIC1);
    } else {
        out.extend_from_slice(&MAGIC0);
    }
    for f in funcs {
        let name_bytes = f.name.as_bytes();
        write_u16_le(
            &mut out,
            u16::try_from(name_bytes.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "function name too long".to_string(),
            })?,
        );
        out.extend_from_slice(name_bytes);
        let code = emit_semcode_function(f, debug_symbols)?;
        write_u32_le(
            &mut out,
            u32::try_from(code.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "function code too large".to_string(),
            })?,
        );
        out.extend_from_slice(&code);
    }
    Ok(out)
}

fn emit_semcode_function(f: &IrFunction, debug_symbols: bool) -> Result<Vec<u8>, FrontendError> {
    let mut interner = StringInterner::new();
    for instr in &f.instrs {
        match instr {
            IrInstr::LoadVar { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::StoreVar { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::Call { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::PulseEmit { signal } => {
                let _ = interner.id(signal)?;
            }
            _ => {}
        }
    }

    let mut label_pc: HashMap<String, u32> = HashMap::new();
    let mut pc: u32 = 0;
    for instr in &f.instrs {
        match instr {
            IrInstr::Label { name } => {
                label_pc.insert(name.clone(), pc);
            }
            _ => {
                pc = pc
                    .checked_add(encoded_size(instr).ok_or(FrontendError {
                        pos: 0,
                        message: "label has no encoded size".to_string(),
                    })? as u32)
                    .ok_or(FrontendError {
                        pos: 0,
                        message: "bytecode size overflow".to_string(),
                    })?;
            }
        }
    }

    let mut instr_stream = Vec::new();
    let mut dbg = Vec::<(u32, u32, u16)>::new();
    for instr in &f.instrs {
        if matches!(instr, IrInstr::Label { .. }) {
            continue;
        }
        let pc = u32::try_from(instr_stream.len()).map_err(|_| FrontendError {
            pos: 0,
            message: "instruction stream too large".to_string(),
        })?;
        emit_instr(instr, &label_pc, &interner, &mut instr_stream)?;
        if debug_symbols {
            let line = u32::try_from(dbg.len() + 1).map_err(|_| FrontendError {
                pos: 0,
                message: "debug table too large".to_string(),
            })?;
            dbg.push((pc, line, 1));
        }
    }

    let mut code = Vec::new();
    interner.emit_table(&mut code)?;
    if debug_symbols {
        code.extend_from_slice(b"DBG0");
        write_u16_le(
            &mut code,
            u16::try_from(dbg.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "too many debug symbols".to_string(),
            })?,
        );
        for (pc, line, col) in dbg {
            write_u32_le(&mut code, pc);
            write_u32_le(&mut code, line);
            write_u16_le(&mut code, col);
        }
    }
    code.extend_from_slice(&instr_stream);
    Ok(code)
}

fn encoded_size(instr: &IrInstr) -> Option<usize> {
    let s = match instr {
        IrInstr::Label { .. } => return None,
        IrInstr::LoadQ { .. } => 1 + 2 + 1,
        IrInstr::LoadBool { .. } => 1 + 2 + 1,
        IrInstr::LoadI32 { .. } => 1 + 2 + 4,
        IrInstr::LoadF64 { .. } => 1 + 2 + 8,
        IrInstr::LoadFx { .. } => 1 + 2 + 4,
        IrInstr::LoadVar { .. } => 1 + 2 + 2,
        IrInstr::StoreVar { .. } => 1 + 2 + 2,
        IrInstr::QAnd { .. }
        | IrInstr::QOr { .. }
        | IrInstr::QImpl { .. }
        | IrInstr::BoolAnd { .. }
        | IrInstr::BoolOr { .. }
        | IrInstr::CmpEq { .. }
        | IrInstr::CmpNe { .. }
        | IrInstr::AddF64 { .. }
        | IrInstr::SubF64 { .. }
        | IrInstr::MulF64 { .. }
        | IrInstr::DivF64 { .. } => 1 + 2 + 2 + 2,
        IrInstr::QNot { .. } | IrInstr::BoolNot { .. } => 1 + 2 + 2,
        IrInstr::Jmp { .. } => 1 + 4,
        IrInstr::JmpIf { .. } => 1 + 2 + 4,
        IrInstr::Assert { .. } => 1 + 2,
        IrInstr::Call { args, .. } => 1 + 1 + 2 + 2 + 2 + (args.len() * 2),
        IrInstr::GateRead { .. } => 1 + 2 + 2 + 2,
        IrInstr::GateWrite { .. } => 1 + 2 + 2 + 2,
        IrInstr::PulseEmit { .. } => 1 + 2,
        IrInstr::Ret { src: Some(_) } => 1 + 1 + 2,
        IrInstr::Ret { src: None } => 1 + 1,
    };
    Some(s)
}

fn emit_instr(
    instr: &IrInstr,
    label_pc: &HashMap<String, u32>,
    interner: &StringInterner,
    out: &mut Vec<u8>,
) -> Result<(), FrontendError> {
    match instr {
        IrInstr::Label { .. } => {}
        IrInstr::LoadQ { dst, val } => {
            out.push(Opcode::LoadQ.byte());
            write_u16_le(out, *dst);
            out.push(match val {
                QuadVal::N => 0,
                QuadVal::F => 1,
                QuadVal::T => 2,
                QuadVal::S => 3,
            });
        }
        IrInstr::LoadBool { dst, val } => {
            out.push(Opcode::LoadBool.byte());
            write_u16_le(out, *dst);
            out.push(if *val { 1 } else { 0 });
        }
        IrInstr::LoadI32 { dst, val } => {
            out.push(Opcode::LoadI32.byte());
            write_u16_le(out, *dst);
            write_i32_le(out, *val);
        }
        IrInstr::LoadF64 { dst, val } => {
            out.push(Opcode::LoadF64.byte());
            write_u16_le(out, *dst);
            write_f64_le(out, *val);
        }
        IrInstr::LoadFx { dst, val } => {
            out.push(Opcode::LoadFx.byte());
            write_u16_le(out, *dst);
            write_i32_le(out, *val);
        }
        IrInstr::LoadVar { dst, name } => {
            out.push(Opcode::LoadVar.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(name)?);
        }
        IrInstr::StoreVar { name, src } => {
            out.push(Opcode::StoreVar.byte());
            write_u16_le(out, interner.lookup(name)?);
            write_u16_le(out, *src);
        }
        IrInstr::QAnd { dst, lhs, rhs } => emit_3reg(Opcode::QAnd, *dst, *lhs, *rhs, out),
        IrInstr::QOr { dst, lhs, rhs } => emit_3reg(Opcode::QOr, *dst, *lhs, *rhs, out),
        IrInstr::QNot { dst, src } => emit_2reg(Opcode::QNot, *dst, *src, out),
        IrInstr::QImpl { dst, lhs, rhs } => emit_3reg(Opcode::QImpl, *dst, *lhs, *rhs, out),
        IrInstr::BoolAnd { dst, lhs, rhs } => emit_3reg(Opcode::BoolAnd, *dst, *lhs, *rhs, out),
        IrInstr::BoolOr { dst, lhs, rhs } => emit_3reg(Opcode::BoolOr, *dst, *lhs, *rhs, out),
        IrInstr::BoolNot { dst, src } => emit_2reg(Opcode::BoolNot, *dst, *src, out),
        IrInstr::CmpEq { dst, lhs, rhs } => emit_3reg(Opcode::CmpEq, *dst, *lhs, *rhs, out),
        IrInstr::CmpNe { dst, lhs, rhs } => emit_3reg(Opcode::CmpNe, *dst, *lhs, *rhs, out),
        IrInstr::AddF64 { dst, lhs, rhs } => emit_3reg(Opcode::AddF64, *dst, *lhs, *rhs, out),
        IrInstr::SubF64 { dst, lhs, rhs } => emit_3reg(Opcode::SubF64, *dst, *lhs, *rhs, out),
        IrInstr::MulF64 { dst, lhs, rhs } => emit_3reg(Opcode::MulF64, *dst, *lhs, *rhs, out),
        IrInstr::DivF64 { dst, lhs, rhs } => emit_3reg(Opcode::DivF64, *dst, *lhs, *rhs, out),
        IrInstr::Jmp { label } => {
            out.push(Opcode::Jmp.byte());
            let addr = *label_pc.get(label).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown label '{}'", label),
            })?;
            write_u32_le(out, addr);
        }
        IrInstr::JmpIf { cond, label } => {
            out.push(Opcode::JmpIf.byte());
            write_u16_le(out, *cond);
            let addr = *label_pc.get(label).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown label '{}'", label),
            })?;
            write_u32_le(out, addr);
        }
        IrInstr::Assert { cond } => {
            out.push(Opcode::Assert.byte());
            write_u16_le(out, *cond);
        }
        IrInstr::Call { dst, name, args } => {
            out.push(Opcode::Call.byte());
            match dst {
                Some(r) => {
                    out.push(1);
                    write_u16_le(out, *r);
                }
                None => {
                    out.push(0);
                    write_u16_le(out, 0);
                }
            }
            write_u16_le(out, interner.lookup(name)?);
            write_u16_le(
                out,
                u16::try_from(args.len()).map_err(|_| FrontendError {
                    pos: 0,
                    message: "too many call args".to_string(),
                })?,
            );
            for a in args {
                write_u16_le(out, *a);
            }
        }
        IrInstr::GateRead {
            dst,
            device_id,
            port,
        } => {
            out.push(Opcode::GateRead.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *device_id);
            write_u16_le(out, *port);
        }
        IrInstr::GateWrite {
            device_id,
            port,
            src,
        } => {
            out.push(Opcode::GateWrite.byte());
            write_u16_le(out, *device_id);
            write_u16_le(out, *port);
            write_u16_le(out, *src);
        }
        IrInstr::PulseEmit { signal } => {
            out.push(Opcode::PulseEmit.byte());
            write_u16_le(out, interner.lookup(signal)?);
        }
        IrInstr::Ret { src } => {
            out.push(Opcode::Ret.byte());
            match src {
                Some(r) => {
                    out.push(1);
                    write_u16_le(out, *r);
                }
                None => {
                    out.push(0);
                }
            }
        }
    }
    Ok(())
}

fn emit_3reg(op: Opcode, dst: u16, lhs: u16, rhs: u16, out: &mut Vec<u8>) {
    out.push(op.byte());
    write_u16_le(out, dst);
    write_u16_le(out, lhs);
    write_u16_le(out, rhs);
}

fn emit_2reg(op: Opcode, dst: u16, src: u16, out: &mut Vec<u8>) {
    out.push(op.byte());
    write_u16_le(out, dst);
    write_u16_le(out, src);
}

fn has_v1_math_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::LoadF64 { .. }
                    | IrInstr::AddF64 { .. }
                    | IrInstr::SubF64 { .. }
                    | IrInstr::MulF64 { .. }
                    | IrInstr::DivF64 { .. }
            )
        })
    })
}

fn has_v2_fx_instr(funcs: &[IrFunction]) -> bool {
    funcs
        .iter()
        .any(|f| f.instrs.iter().any(|i| matches!(i, IrInstr::LoadFx { .. })))
}

#[derive(Debug, Default)]
struct StringInterner {
    ids: HashMap<String, u16>,
    by_id: Vec<String>,
}

impl StringInterner {
    fn new() -> Self {
        Self::default()
    }

    fn id(&mut self, s: &str) -> Result<u16, FrontendError> {
        if let Some(id) = self.ids.get(s) {
            return Ok(*id);
        }
        let id = u16::try_from(self.by_id.len()).map_err(|_| FrontendError {
            pos: 0,
            message: "string table overflow".to_string(),
        })?;
        self.ids.insert(s.to_string(), id);
        self.by_id.push(s.to_string());
        Ok(id)
    }

    fn lookup(&self, s: &str) -> Result<u16, FrontendError> {
        self.ids.get(s).copied().ok_or(FrontendError {
            pos: 0,
            message: format!("string '{}' not interned", s),
        })
    }

    fn emit_table(&self, out: &mut Vec<u8>) -> Result<(), FrontendError> {
        write_u16_le(
            out,
            u16::try_from(self.by_id.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "string table too large".to_string(),
            })?,
        );
        for s in &self.by_id {
            let b = s.as_bytes();
            write_u16_le(
                out,
                u16::try_from(b.len()).map_err(|_| FrontendError {
                    pos: 0,
                    message: "string too long".to_string(),
                })?,
            );
            out.extend_from_slice(b);
        }
        Ok(())
    }
}

fn lower_expr(
    expr_id: ExprId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    ret_ty: Type,
) -> Result<(u16, Type), FrontendError> {
    lower_expr_with_expected(expr_id, arena, next, out, env, fn_table, None, ret_ty)
}

fn lower_expr_with_expected(
    expr_id: ExprId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    expected: Option<Type>,
    ret_ty: Type,
) -> Result<(u16, Type), FrontendError> {
    match arena.expr(expr_id) {
        Expr::QuadLiteral(v) => {
            let r = alloc(next);
            out.push(IrInstr::LoadQ { dst: r, val: *v });
            Ok((r, Type::Quad))
        }
        Expr::BoolLiteral(v) => {
            let r = alloc(next);
            out.push(IrInstr::LoadBool { dst: r, val: *v });
            Ok((r, Type::Bool))
        }
        Expr::Num(n) => {
            let r = alloc(next);
            if expected == Some(Type::Fx) {
                let val = try_encode_fx_literal_expr(expr_id, arena)?.ok_or(FrontendError {
                    pos: 0,
                    message: "expected fx literal".to_string(),
                })?;
                out.push(IrInstr::LoadFx { dst: r, val });
                Ok((r, Type::Fx))
            } else {
                let val = i32::try_from(*n).map_err(|_| FrontendError {
                    pos: 0,
                    message: format!("numeric literal {} does not fit in i32", n),
                })?;
                out.push(IrInstr::LoadI32 { dst: r, val });
                Ok((r, Type::I32))
            }
        }
        Expr::Float(n) => {
            let r = alloc(next);
            if expected == Some(Type::Fx) {
                out.push(IrInstr::LoadFx {
                    dst: r,
                    val: encode_fx_literal(*n)?,
                });
                Ok((r, Type::Fx))
            } else {
                out.push(IrInstr::LoadF64 { dst: r, val: *n });
                Ok((r, Type::F64))
            }
        }
        Expr::Var(name) => {
            let ty = env.get(*name).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown variable '{}'", resolve_symbol_name(arena, *name)?),
            })?;
            let r = alloc(next);
            out.push(IrInstr::LoadVar {
                dst: r,
                name: resolve_symbol_name(arena, *name)?.to_string(),
            });
            Ok((r, ty))
        }
        Expr::Block(block) => {
            lower_value_block_expr(block, arena, next, out, env, fn_table, expected, ret_ty)
        }
        Expr::If(if_expr) => {
            let (cond_reg, cond_ty) =
                lower_expr(if_expr.condition, arena, next, out, env, fn_table, ret_ty)?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if expression condition must be bool".to_string(),
                });
            }

            let id = alloc_if_expr_id(next);
            let then_label = format!("if_expr_{}_then", id);
            let else_label = format!("if_expr_{}_else", id);
            let end_label = format!("if_expr_{}_end", id);
            let result_name = format!("__if_expr_{}_result", id);

            out.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: then_label.clone(),
            });
            out.push(IrInstr::Jmp {
                label: else_label.clone(),
            });

            out.push(IrInstr::Label { name: then_label });
            let (then_reg, then_ty) = lower_value_block_expr(
                &if_expr.then_block,
                arena,
                next,
                out,
                env,
                fn_table,
                expected,
                ret_ty,
            )?;
            out.push(IrInstr::StoreVar {
                name: result_name.clone(),
                src: then_reg,
            });
            out.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            out.push(IrInstr::Label { name: else_label });
            let (else_reg, else_ty) = lower_value_block_expr(
                &if_expr.else_block,
                arena,
                next,
                out,
                env,
                fn_table,
                expected,
                ret_ty,
            )?;
            if then_ty != else_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "if expression branch type mismatch in lowering: then {:?}, else {:?}",
                        then_ty, else_ty
                    ),
                });
            }
            out.push(IrInstr::StoreVar {
                name: result_name.clone(),
                src: else_reg,
            });
            out.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            out.push(IrInstr::Label { name: end_label });
            let dst = alloc(next);
            out.push(IrInstr::LoadVar {
                dst,
                name: result_name,
            });
            Ok((dst, then_ty))
        }
        Expr::Match(match_expr) => lower_match_expr(
            match_expr, arena, next, out, env, fn_table, expected, ret_ty,
        ),
        Expr::Call(name, args) => {
            if is_builtin_assert_name(*name, arena, fn_table)? {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "assert builtin is statement-only and cannot be used as expression value"
                            .to_string(),
                });
            }
            let sig = if let Some(s) = fn_table.get(name) {
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
            let mut regs = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                let (r, t) = lower_expr_with_expected(
                    *arg,
                    arena,
                    next,
                    out,
                    env,
                    fn_table,
                    Some(sig.params[i]),
                    ret_ty,
                )?;
                if t != sig.params[i] {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "arg {} for '{}' has type {:?}, expected {:?}",
                            i,
                            resolve_symbol_name(arena, *name)?,
                            t,
                            sig.params[i]
                        ),
                    });
                }
                regs.push(r);
            }
            if sig.ret == Type::Unit {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unit-returning call '{}' cannot be used as expression value",
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            let r = alloc(next);
            out.push(IrInstr::Call {
                dst: Some(r),
                name: resolve_symbol_name(arena, *name)?.to_string(),
                args: regs,
            });
            Ok((r, sig.ret))
        }
        Expr::Unary(op, inner) => {
            if expected == Some(Type::Fx) {
                if let Some(value) = try_encode_fx_literal_expr(expr_id, arena)? {
                    let dst = alloc(next);
                    out.push(IrInstr::LoadFx { dst, val: value });
                    return Ok((dst, Type::Fx));
                }
            }
            let (src, ty) = lower_expr_with_expected(
                *inner, arena, next, out, env, fn_table, expected, ret_ty,
            )?;
            match op {
                UnaryOp::Not => {
                    let dst = alloc(next);
                    match ty {
                        Type::Quad => out.push(IrInstr::QNot { dst, src }),
                        Type::Bool => out.push(IrInstr::BoolNot { dst, src }),
                        _ => {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!("operator ! unsupported for {:?}", ty),
                            })
                        }
                    }
                    Ok((dst, ty))
                }
                UnaryOp::Pos => {
                    if ty == Type::F64 {
                        Ok((src, Type::F64))
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("operator + unsupported for {:?}", ty),
                        })
                    }
                }
                UnaryOp::Neg => {
                    if ty != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator - unsupported for {:?}", ty),
                        });
                    }
                    let zero = alloc(next);
                    out.push(IrInstr::LoadF64 {
                        dst: zero,
                        val: 0.0,
                    });
                    let dst = alloc(next);
                    out.push(IrInstr::SubF64 {
                        dst,
                        lhs: zero,
                        rhs: src,
                    });
                    Ok((dst, Type::F64))
                }
            }
        }
        Expr::Binary(left, op, right) => {
            let (lr, lt) =
                lower_expr_with_expected(*left, arena, next, out, env, fn_table, expected, ret_ty)?;
            let (rr, rt) = lower_expr_with_expected(
                *right, arena, next, out, env, fn_table, expected, ret_ty,
            )?;
            if lt != rt {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("operator type mismatch: {:?} vs {:?}", lt, rt),
                });
            }
            let dst = alloc(next);
            match op {
                BinaryOp::AndAnd => match lt {
                    Type::Quad => out.push(IrInstr::QAnd {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    Type::Bool => out.push(IrInstr::BoolAnd {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    _ => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator && unsupported for {:?}", lt),
                        })
                    }
                },
                BinaryOp::OrOr => match lt {
                    Type::Quad => out.push(IrInstr::QOr {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    Type::Bool => out.push(IrInstr::BoolOr {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    _ => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator || unsupported for {:?}", lt),
                        })
                    }
                },
                BinaryOp::Implies => {
                    if lt != Type::Quad {
                        return Err(FrontendError {
                            pos: 0,
                            message: "operator '->' is allowed only for quad".to_string(),
                        });
                    }
                    out.push(IrInstr::QImpl {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::Quad));
                }
                BinaryOp::Eq => {
                    out.push(IrInstr::CmpEq {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::Bool));
                }
                BinaryOp::Ne => {
                    out.push(IrInstr::CmpNe {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::Bool));
                }
                BinaryOp::Add => {
                    if lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator + unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::AddF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::F64));
                }
                BinaryOp::Sub => {
                    if lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator - unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::SubF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::F64));
                }
                BinaryOp::Mul => {
                    if lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator * unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::MulF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::F64));
                }
                BinaryOp::Div => {
                    if lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator / unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::DivF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::F64));
                }
            }
            Ok((dst, lt))
        }
    }
}

fn lower_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
) -> Result<(), FrontendError> {
    let stmt = arena.stmt(stmt_id);
    match stmt {
        Stmt::Const { name, ty, value } => {
            let (reg, vty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                *ty,
                ret_ty,
            )?;
            let final_ty = if let Some(ann) = ty { *ann } else { vty };
            env.insert_const(*name, final_ty);
            ctx.instrs.push(IrInstr::StoreVar {
                name: resolve_symbol_name(arena, *name)?.to_string(),
                src: reg,
            });
            Ok(())
        }
        Stmt::Let { name, ty, value } => {
            let (reg, vty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                *ty,
                ret_ty,
            )?;
            let final_ty = if let Some(ann) = ty { *ann } else { vty };
            env.insert(*name, final_ty);
            ctx.instrs.push(IrInstr::StoreVar {
                name: resolve_symbol_name(arena, *name)?.to_string(),
                src: reg,
            });
            Ok(())
        }
        Stmt::Discard { ty, value } => {
            let _ = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                *ty,
                ret_ty,
            )?;
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
            let (reg, _) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                Some(target_ty),
                ret_ty,
            )?;
            ctx.instrs.push(IrInstr::StoreVar {
                name: resolve_symbol_name(arena, *name)?.to_string(),
                src: reg,
            });
            Ok(())
        }
        Stmt::Guard {
            condition,
            else_return,
        } => {
            let (cond_reg, cond_ty) = lower_expr(
                *condition,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                ret_ty,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "guard clause condition must be bool".to_string(),
                });
            }

            let id = ctx.next_if_id();
            let continue_label = format!("guard_{}_continue", id);
            ctx.instrs.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: continue_label.clone(),
            });
            lower_return_payload(
                *else_return,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                ret_ty,
            )?;
            ctx.instrs.push(IrInstr::Label {
                name: continue_label,
            });
            Ok(())
        }
        Stmt::Expr(expr) => {
            lower_expr_stmt(*expr, arena, ctx, env, fn_table, ret_ty)?;
            Ok(())
        }
        Stmt::Return(v) => lower_return_payload(
            *v,
            arena,
            &mut ctx.next_reg,
            &mut ctx.instrs,
            env,
            fn_table,
            ret_ty,
        ),
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            let (cond_reg, cond_ty) = lower_expr(
                *condition,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                ret_ty,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool".to_string(),
                });
            }

            let id = ctx.next_if_id();
            let then_label = format!("if_{}_then", id);
            let else_label = format!("if_{}_else", id);
            let end_label = format!("if_{}_end", id);

            ctx.instrs.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: then_label.clone(),
            });
            ctx.instrs.push(IrInstr::Jmp {
                label: else_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: then_label });
            let mut then_env = env.clone();
            then_env.push_scope();
            for s in then_block {
                lower_stmt(*s, arena, ctx, &mut then_env, ret_ty, fn_table)?;
            }
            then_env.pop_scope();
            ctx.instrs.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: else_label });
            let mut else_env = env.clone();
            else_env.push_scope();
            for s in else_block {
                lower_stmt(*s, arena, ctx, &mut else_env, ret_ty, fn_table)?;
            }
            else_env.pop_scope();
            ctx.instrs.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: end_label });
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            let (scr_reg, scr_ty) = lower_expr(
                *scrutinee,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
                ret_ty,
            )?;
            if scr_ty != Type::Quad {
                return Err(FrontendError {
                    pos: 0,
                    message: "match scrutinee must be quad".to_string(),
                });
            }
            if default.is_empty() {
                return Err(FrontendError {
                    pos: 0,
                    message: "match requires default arm '_'".to_string(),
                });
            }

            let mid = ctx.next_if_id();
            let end_label = format!("match_{}_end", mid);
            let default_label = format!("match_{}_default", mid);
            let arm_labels: Vec<String> = (0..arms.len())
                .map(|i| format!("match_{}_arm_{}", mid, i))
                .collect();
            if arms.iter().all(|arm| arm.guard.is_none()) {
                for (i, arm) in arms.iter().enumerate() {
                    let lit_reg = alloc(&mut ctx.next_reg);
                    ctx.instrs.push(IrInstr::LoadQ {
                        dst: lit_reg,
                        val: arm.pat,
                    });
                    let cmp_reg = alloc(&mut ctx.next_reg);
                    ctx.instrs.push(IrInstr::CmpEq {
                        dst: cmp_reg,
                        lhs: scr_reg,
                        rhs: lit_reg,
                    });
                    ctx.instrs.push(IrInstr::JmpIf {
                        cond: cmp_reg,
                        label: arm_labels[i].clone(),
                    });
                }
                ctx.instrs.push(IrInstr::Jmp {
                    label: default_label.clone(),
                });

                for (i, arm) in arms.iter().enumerate() {
                    ctx.instrs.push(IrInstr::Label {
                        name: arm_labels[i].clone(),
                    });
                    let mut arm_env = env.clone();
                    arm_env.push_scope();
                    for s in &arm.block {
                        lower_stmt(*s, arena, ctx, &mut arm_env, ret_ty, fn_table)?;
                    }
                    arm_env.pop_scope();
                    ctx.instrs.push(IrInstr::Jmp {
                        label: end_label.clone(),
                    });
                }
            } else {
                for (i, arm) in arms.iter().enumerate() {
                    if i > 0 {
                        ctx.instrs.push(IrInstr::Label {
                            name: format!("match_{}_check_{}", mid, i),
                        });
                    }
                    let next_label = if i + 1 < arms.len() {
                        format!("match_{}_check_{}", mid, i + 1)
                    } else {
                        default_label.clone()
                    };

                    let lit_reg = alloc(&mut ctx.next_reg);
                    ctx.instrs.push(IrInstr::LoadQ {
                        dst: lit_reg,
                        val: arm.pat,
                    });
                    let cmp_reg = alloc(&mut ctx.next_reg);
                    ctx.instrs.push(IrInstr::CmpEq {
                        dst: cmp_reg,
                        lhs: scr_reg,
                        rhs: lit_reg,
                    });
                    ctx.instrs.push(IrInstr::JmpIf {
                        cond: cmp_reg,
                        label: arm_labels[i].clone(),
                    });
                    ctx.instrs.push(IrInstr::Jmp {
                        label: next_label.clone(),
                    });

                    ctx.instrs.push(IrInstr::Label {
                        name: arm_labels[i].clone(),
                    });
                    let mut arm_env = env.clone();
                    arm_env.push_scope();
                    if let Some(guard_reg) = lower_match_guard(
                        arm.guard,
                        arena,
                        &mut ctx.next_reg,
                        &mut ctx.instrs,
                        &arm_env,
                        fn_table,
                        ret_ty,
                    )? {
                        let guarded_body_label = format!("match_{}_body_{}", mid, i);
                        ctx.instrs.push(IrInstr::JmpIf {
                            cond: guard_reg,
                            label: guarded_body_label.clone(),
                        });
                        ctx.instrs.push(IrInstr::Jmp { label: next_label });
                        ctx.instrs.push(IrInstr::Label {
                            name: guarded_body_label,
                        });
                    }
                    for s in &arm.block {
                        lower_stmt(*s, arena, ctx, &mut arm_env, ret_ty, fn_table)?;
                    }
                    arm_env.pop_scope();
                    ctx.instrs.push(IrInstr::Jmp {
                        label: end_label.clone(),
                    });
                }
            }

            ctx.instrs.push(IrInstr::Label {
                name: default_label,
            });
            let mut def_env = env.clone();
            def_env.push_scope();
            for s in default {
                lower_stmt(*s, arena, ctx, &mut def_env, ret_ty, fn_table)?;
            }
            def_env.pop_scope();
            ctx.instrs.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: end_label });
            Ok(())
        }
    }
}

fn lower_value_block_expr(
    block: &BlockExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    expected: Option<Type>,
    ret_ty: Type,
) -> Result<(u16, Type), FrontendError> {
    let mut block_env = env.clone();
    block_env.push_scope();
    for stmt in &block.statements {
        match arena.stmt(*stmt) {
            Stmt::Const { name, ty, value } => {
                let (reg, vty) = lower_expr_with_expected(
                    *value,
                    arena,
                    next,
                    out,
                    &block_env,
                    fn_table,
                    *ty,
                    ret_ty,
                )?;
                let final_ty = if let Some(ann) = ty { *ann } else { vty };
                block_env.insert_const(*name, final_ty);
                out.push(IrInstr::StoreVar {
                    name: resolve_symbol_name(arena, *name)?.to_string(),
                    src: reg,
                });
            }
            Stmt::Let { name, ty, value } => {
                let (reg, vty) = lower_expr_with_expected(
                    *value, arena, next, out, &block_env, fn_table, *ty, ret_ty,
                )?;
                let final_ty = if let Some(ann) = ty { *ann } else { vty };
                block_env.insert(*name, final_ty);
                out.push(IrInstr::StoreVar {
                    name: resolve_symbol_name(arena, *name)?.to_string(),
                    src: reg,
                });
            }
            Stmt::Discard { ty, value } => {
                let _ = lower_expr_with_expected(
                    *value, arena, next, out, &block_env, fn_table, *ty, ret_ty,
                )?;
            }
            Stmt::Expr(expr) => {
                lower_expr_stmt_with_parts(*expr, arena, next, out, &block_env, fn_table, ret_ty)?;
            }
            _ => {
                return Err(FrontendError {
                    pos: 0,
                    message: "value-producing block currently supports only const-bindings, let-bindings, discard binds, and expression statements before the tail value".to_string(),
                });
            }
        }
    }
    let tail = lower_expr_with_expected(
        block.tail, arena, next, out, &block_env, fn_table, expected, ret_ty,
    )?;
    block_env.pop_scope();
    Ok(tail)
}

fn lower_match_guard(
    guard: Option<ExprId>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    ret_ty: Type,
) -> Result<Option<u16>, FrontendError> {
    let Some(guard_expr) = guard else {
        return Ok(None);
    };
    let (guard_reg, guard_ty) = lower_expr(guard_expr, arena, next, out, env, fn_table, ret_ty)?;
    if guard_ty != Type::Bool {
        return Err(FrontendError {
            pos: 0,
            message: "match guard condition must be bool".to_string(),
        });
    }
    Ok(Some(guard_reg))
}

fn lower_return_payload(
    value: Option<ExprId>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    ret_ty: Type,
) -> Result<(), FrontendError> {
    match value {
        Some(expr_id) => {
            let (reg, ty) = lower_expr_with_expected(
                expr_id,
                arena,
                next,
                out,
                env,
                fn_table,
                Some(ret_ty),
                ret_ty,
            )?;
            if ty != ret_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "return type mismatch in lowering: expected {:?}, got {:?}",
                        ret_ty, ty
                    ),
                });
            }
            out.push(IrInstr::Ret { src: Some(reg) });
            Ok(())
        }
        None => {
            if ret_ty != Type::Unit {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("return without value in non-unit function ({:?})", ret_ty),
                });
            }
            out.push(IrInstr::Ret { src: None });
            Ok(())
        }
    }
}

fn lower_match_expr(
    match_expr: &MatchExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    expected: Option<Type>,
    ret_ty: Type,
) -> Result<(u16, Type), FrontendError> {
    let (scr_reg, scr_ty) = lower_expr(
        match_expr.scrutinee,
        arena,
        next,
        out,
        env,
        fn_table,
        ret_ty,
    )?;
    if scr_ty != Type::Quad {
        return Err(FrontendError {
            pos: 0,
            message: "match expression scrutinee must be quad".to_string(),
        });
    }
    let default = match_expr.default.as_ref().ok_or(FrontendError {
        pos: 0,
        message: "match expression requires default arm '_'".to_string(),
    })?;

    let id = alloc_match_expr_id(next);
    let end_label = format!("match_expr_{}_end", id);
    let default_label = format!("match_expr_{}_default", id);
    let arm_labels: Vec<String> = (0..match_expr.arms.len())
        .map(|i| format!("match_expr_{}_arm_{}", id, i))
        .collect();
    let result_name = format!("__match_expr_{}_result", id);

    let mut result_ty = None;
    for (i, arm) in match_expr.arms.iter().enumerate() {
        if i > 0 {
            out.push(IrInstr::Label {
                name: format!("match_expr_{}_check_{}", id, i),
            });
        }
        let next_label = if i + 1 < match_expr.arms.len() {
            format!("match_expr_{}_check_{}", id, i + 1)
        } else {
            default_label.clone()
        };

        let lit_reg = alloc(next);
        out.push(IrInstr::LoadQ {
            dst: lit_reg,
            val: arm.pat,
        });
        let cmp_reg = alloc(next);
        out.push(IrInstr::CmpEq {
            dst: cmp_reg,
            lhs: scr_reg,
            rhs: lit_reg,
        });
        out.push(IrInstr::JmpIf {
            cond: cmp_reg,
            label: arm_labels[i].clone(),
        });
        out.push(IrInstr::Jmp {
            label: next_label.clone(),
        });

        out.push(IrInstr::Label {
            name: arm_labels[i].clone(),
        });
        let mut arm_env = env.clone();
        arm_env.push_scope();
        if let Some(guard_reg) =
            lower_match_guard(arm.guard, arena, next, out, &arm_env, fn_table, ret_ty)?
        {
            let guarded_body_label = format!("match_expr_{}_body_{}", id, i);
            out.push(IrInstr::JmpIf {
                cond: guard_reg,
                label: guarded_body_label.clone(),
            });
            out.push(IrInstr::Jmp { label: next_label });
            out.push(IrInstr::Label {
                name: guarded_body_label,
            });
        }
        let (arm_reg, arm_ty) = lower_value_block_expr(
            &arm.block, arena, next, out, &arm_env, fn_table, expected, ret_ty,
        )?;
        arm_env.pop_scope();
        if let Some(expected_ty) = result_ty {
            if expected_ty != arm_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "match expression branch type mismatch in lowering: expected {:?}, got {:?}",
                        expected_ty, arm_ty
                    ),
                });
            }
        } else {
            result_ty = Some(arm_ty);
        }
        out.push(IrInstr::StoreVar {
            name: result_name.clone(),
            src: arm_reg,
        });
        out.push(IrInstr::Jmp {
            label: end_label.clone(),
        });
    }

    out.push(IrInstr::Label {
        name: default_label,
    });
    let (default_reg, default_ty) =
        lower_value_block_expr(default, arena, next, out, env, fn_table, expected, ret_ty)?;
    if let Some(expected_ty) = result_ty {
        if expected_ty != default_ty {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "match expression branch type mismatch in lowering: expected {:?}, got {:?}",
                    expected_ty, default_ty
                ),
            });
        }
    } else {
        result_ty = Some(default_ty);
    }
    out.push(IrInstr::StoreVar {
        name: result_name.clone(),
        src: default_reg,
    });
    out.push(IrInstr::Jmp {
        label: end_label.clone(),
    });

    out.push(IrInstr::Label { name: end_label });
    let dst = alloc(next);
    out.push(IrInstr::LoadVar {
        dst,
        name: result_name,
    });
    Ok((dst, result_ty.expect("default arm guarantees result type")))
}

fn lower_expr_stmt(
    expr_id: ExprId,
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &ScopeEnv,
    fn_table: &FnTable,
    ret_ty: Type,
) -> Result<(), FrontendError> {
    lower_expr_stmt_with_parts(
        expr_id,
        arena,
        &mut ctx.next_reg,
        &mut ctx.instrs,
        env,
        fn_table,
        ret_ty,
    )
}

fn alloc_if_expr_id(next: &mut u16) -> u16 {
    let id = *next;
    *next += 1;
    id
}

fn alloc_match_expr_id(next: &mut u16) -> u16 {
    let id = *next;
    *next += 1;
    id
}

fn lower_expr_stmt_with_parts(
    expr_id: ExprId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    ret_ty: Type,
) -> Result<(), FrontendError> {
    let expr = arena.expr(expr_id);
    if let Expr::Call(name, args) = expr {
        if is_builtin_assert_name(*name, arena, fn_table)? {
            if args.len() != 1 {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("assert builtin expects 1 arg, got {}", args.len()),
                });
            }
            let (cond, cond_ty) = lower_expr_with_expected(
                args[0],
                arena,
                next,
                out,
                env,
                fn_table,
                Some(Type::Bool),
                ret_ty,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("assert builtin requires bool condition, got {:?}", cond_ty),
                });
            }
            out.push(IrInstr::Assert { cond });
            return Ok(());
        }
        let sig = if let Some(s) = fn_table.get(name) {
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
        let mut regs = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let (r, t) = lower_expr_with_expected(
                *arg,
                arena,
                next,
                out,
                env,
                fn_table,
                Some(sig.params[i]),
                ret_ty,
            )?;
            if t != sig.params[i] {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "arg {} for '{}' type mismatch",
                        i,
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            regs.push(r);
        }
        let dst = if sig.ret == Type::Unit {
            None
        } else {
            Some(alloc(next))
        };
        out.push(IrInstr::Call {
            dst,
            name: resolve_symbol_name(arena, *name)?.to_string(),
            args: regs,
        });
        return Ok(());
    }

    let _ = lower_expr(expr_id, arena, next, out, env, fn_table, ret_ty)?;
    Ok(())
}

#[derive(Debug, Default)]
struct LoweringCtx {
    next_reg: u16,
    next_label_id: u32,
    instrs: Vec<IrInstr>,
}

impl LoweringCtx {
    fn new() -> Self {
        Self {
            next_reg: 0,
            next_label_id: 0,
            instrs: Vec::new(),
        }
    }

    fn next_if_id(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    fn ends_with_ret(&self) -> bool {
        matches!(self.instrs.last(), Some(IrInstr::Ret { .. }))
    }
}

#[inline]
fn alloc(next: &mut u16) -> u16 {
    let out = *next;
    *next += 1;
    out
}

#[cfg(test)]
mod opt_tests {
    use super::*;
    use crate::passes::run_default_opt_passes;

    #[test]
    fn lower_block_expression_tail_to_ir() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let base: f64 = 1.0;
                    base + 2.0
                };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("block expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name == "base"
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name == "total"
        )));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
    }

    #[test]
    fn block_expression_rejects_control_statements_in_body() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    if true { return; } else { return; }
                    1.0
                };
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("control statements must reject");
        assert!(err.message.contains(
            "value-producing block currently supports only const-bindings, let-bindings, discard binds, and expression statements before the tail value"
        ));
    }

    #[test]
    fn lower_if_expression_to_ir() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { 2.0 };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("if expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("if_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__if_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadVar { name, .. } if name.starts_with("__if_expr_")
        )));
    }

    #[test]
    fn lowering_if_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { true };
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("mismatched branch types must reject");
        assert!(err.message.contains("if expression branch type mismatch"));
    }

    #[test]
    fn lower_match_expression_to_ir() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T if true => { 1.0 }
                    _ => { 2.0 }
                };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("match expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("match_expr_")
        )));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadBool { .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__match_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadVar { name, .. } if name.starts_with("__match_expr_")
        )));
    }

    #[test]
    fn lower_guard_clause_to_ir() {
        let src = r#"
            fn main() {
                guard true else return;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("guard clause should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::JmpIf { label, .. } if label.starts_with("guard_")
        )));
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::Ret { .. }))
                .count()
                >= 2
        );
    }

    #[test]
    fn lower_pipeline_expression_to_ordinary_calls() {
        let src = r#"
            fn inc(x: f64) -> f64 = x + 1.0;
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = 1.0 |> inc() |> scale(3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        let ir = compile_program_to_ir(src).expect("pipeline should lower through ordinary calls");
        let main = &ir[2];
        let call_names: Vec<_> = main
            .instrs
            .iter()
            .filter_map(|instr| match instr {
                IrInstr::Call { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert!(call_names.contains(&"inc"));
        assert!(call_names.contains(&"scale"));
    }

    #[test]
    fn lower_immediate_short_lambda_without_indirect_call_path() {
        let src = r#"
            fn main() {
                let total: f64 = (x => x + 1.0)(2.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("short lambda should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name == "x")));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(!main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Call { .. })));
    }

    #[test]
    fn lower_pipeline_short_lambda_without_indirect_call_path() {
        let src = r#"
            fn main() {
                let total: f64 = 2.0 |> (x => x + 1.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("pipeline short lambda should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name == "x")));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(!main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Call { .. })));
    }

    #[test]
    fn lower_const_declaration_to_existing_store_path() {
        let src = r#"
            fn main() {
                const total: f64 = 1.0 + 2.0;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("const declaration should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name == "total")));
    }

    #[test]
    fn lowering_rejects_assignment_to_const_binding() {
        let src = r#"
            fn main() {
                const total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("assignment to const must reject");
        assert!(err.message.contains("cannot assign to const binding 'total'"));
    }

    #[test]
    fn lower_compound_assignment_to_read_modify_write() {
        let src = r#"
            fn main() {
                let total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("compound assignment should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadVar { name, .. } if name == "total")));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name == "total"))
                .count()
                >= 2
        );
    }

    #[test]
    fn lower_discard_bind_evaluates_rhs_without_store() {
        let src = r#"
            fn main() {
                let _ = 1.0 + 2.0;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("discard bind should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(!main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name == "_"
        )));
    }

    #[test]
    fn lower_assert_statement_to_assert_ir() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("assert statement should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Assert { .. })));
    }

    #[test]
    fn lowering_match_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                    _ => { true }
                };
                return;
            }
        "#;

        let err = compile_program_to_ir(src)
            .expect_err("mismatched match expression branches must reject");
        assert!(err
            .message
            .contains("match expression branch type mismatch"));
    }

    #[test]
    fn opt_removes_unreachable_and_noop_jmp() {
        let mut ir = vec![IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::Label {
                    name: "entry".to_string(),
                },
                IrInstr::Jmp {
                    label: "l1".to_string(),
                },
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::Label {
                    name: "l1".to_string(),
                },
                IrInstr::Ret { src: None },
            ],
        }];
        let report = run_default_opt_passes(&mut ir);
        assert!(report.changed);
        assert!(matches!(ir[0].instrs[0], IrInstr::Label { .. }));
        assert!(ir[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::LoadBool { dst: 0, val: true })));
    }

    #[test]
    fn opt_removes_redundant_consecutive_loads() {
        let mut ir = vec![IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadI32 { dst: 1, val: 10 },
                IrInstr::LoadI32 { dst: 1, val: 11 },
                IrInstr::Ret { src: Some(1) },
            ],
        }];
        let report = run_default_opt_passes(&mut ir);
        assert!(report.changed);
        let loads = ir[0]
            .instrs
            .iter()
            .filter(|i| matches!(i, IrInstr::LoadI32 { dst: 1, .. }))
            .count();
        assert_eq!(loads, 1);
        assert!(matches!(
            ir[0].instrs[0],
            IrInstr::LoadI32 { dst: 1, val: 11 }
        ));
    }

    #[test]
    fn opt_folds_bool_and_f64_constants() {
        let f = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::LoadBool { dst: 1, val: false },
                IrInstr::BoolAnd {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::LoadF64 { dst: 3, val: 2.0 },
                IrInstr::LoadF64 { dst: 4, val: 3.0 },
                IrInstr::AddF64 {
                    dst: 5,
                    lhs: 3,
                    rhs: 4,
                },
                IrInstr::Ret { src: Some(5) },
            ],
        };
        let mut ir = vec![f];
        let report = crate::passes::run_default_opt_passes(&mut ir);
        assert!(report.changed);
        let f = &ir[0];
        assert!(f
            .instrs
            .iter()
            .any(|i| matches!(i, IrInstr::LoadBool { dst: 2, val: false })));
        assert!(f.instrs.iter().any(|i| matches!(
            i,
            IrInstr::LoadF64 { dst: 5, val } if (*val - 5.0).abs() < f64::EPSILON
        )));
    }
}
