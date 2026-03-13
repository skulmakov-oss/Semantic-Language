use super::*;
use crate::semcode_format::{
    write_f64_le, write_i32_le, write_u16_le, write_u32_le, Opcode, MAGIC0, MAGIC1,
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
    let _ = lower_expr(expr, arena, &mut next, &mut out, &env, fn_table)?;
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
    compile_program_to_ir_with_options(input, CompileProfile::RustLike, OptLevel::O0)
}

pub fn compile_program_to_immutable_ir(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<ImmutableIrProgram, FrontendError> {
    Ok(ImmutableIrProgram::from_vec(compile_program_to_ir_with_options(
        input, profile, opt,
    )?))
}

pub fn compile_program_to_ir_with_options(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
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
    let logos_detected = parse_logos_program(input)
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
    let program = parse_program(input)?;
    let fn_table = build_fn_table(&program)?;
    type_check_program(&program)?;
    let mut out = Vec::new();
    for f in &program.functions {
        out.push(lower_function_to_ir(f, &program.arena, &fn_table)?);
    }
    if matches!(opt, OptLevel::O1) {
        for f in &mut out {
            canonicalize_ir_function(f);
            optimize_ir_function(f);
        }
        let _ = crate::passes::run_default_opt_passes(&mut out);
    }
    Ok(out)
}

pub fn compile_program_to_ir_optimized(input: &str) -> Result<Vec<IrFunction>, FrontendError> {
    compile_program_to_ir_with_options(input, CompileProfile::RustLike, OptLevel::O1)
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

fn canonicalize_ir_function(f: &mut IrFunction) {
    // Normalize obvious structural noise first.
    remove_duplicate_consecutive_labels(&mut f.instrs);
}

fn optimize_ir_function(f: &mut IrFunction) {
    remove_unreachable_until_label(&mut f.instrs);
    remove_noop_jumps(&mut f.instrs);
    remove_redundant_consecutive_loads(&mut f.instrs);
}

fn remove_duplicate_consecutive_labels(instrs: &mut Vec<IrInstr>) {
    let mut out = Vec::with_capacity(instrs.len());
    for instr in instrs.drain(..) {
        let dup = matches!(
            (out.last(), &instr),
            (
                Some(IrInstr::Label { name: a }),
                IrInstr::Label { name: b }
            ) if a == b
        );
        if !dup {
            out.push(instr);
        }
    }
    *instrs = out;
}

fn remove_unreachable_until_label(instrs: &mut Vec<IrInstr>) {
    let mut out = Vec::with_capacity(instrs.len());
    let mut unreachable = false;
    for instr in instrs.drain(..) {
        match &instr {
            IrInstr::Label { .. } => {
                unreachable = false;
                out.push(instr);
            }
            _ if unreachable => {}
            _ => {
                let terminal = matches!(instr, IrInstr::Ret { .. } | IrInstr::Jmp { .. });
                out.push(instr);
                if terminal {
                    unreachable = true;
                }
            }
        }
    }
    *instrs = out;
}

fn remove_noop_jumps(instrs: &mut Vec<IrInstr>) {
    let mut out = Vec::with_capacity(instrs.len());
    let mut i = 0usize;
    while i < instrs.len() {
        let skip = if let IrInstr::Jmp { label } = &instrs[i] {
            matches!(
                instrs.get(i + 1),
                Some(IrInstr::Label { name }) if name == label
            )
        } else {
            false
        };
        if !skip {
            out.push(instrs[i].clone());
        }
        i += 1;
    }
    *instrs = out;
}

fn load_dst_and_payload(instr: &IrInstr) -> Option<(u16, u64)> {
    match instr {
        IrInstr::LoadQ { dst, val } => Some((*dst, 0x1000 | (*val as u64))),
        IrInstr::LoadBool { dst, val } => Some((*dst, 0x2000 | (*val as u64))),
        IrInstr::LoadI32 { dst, val } => Some((*dst, 0x3000 | (*val as i64 as u64))),
        IrInstr::LoadF64 { dst, val } => Some((*dst, 0x4000 | val.to_bits())),
        IrInstr::LoadVar { dst, name } => {
            let mut h = 0xcbf29ce484222325u64;
            for b in name.as_bytes() {
                h ^= *b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            Some((*dst, 0x5000 ^ h))
        }
        _ => None,
    }
}

fn remove_redundant_consecutive_loads(instrs: &mut Vec<IrInstr>) {
    let mut out = Vec::with_capacity(instrs.len());
    let mut i = 0usize;
    while i < instrs.len() {
        let drop_curr = if let (Some(a), Some(b)) = (
            load_dst_and_payload(&instrs[i]),
            instrs.get(i + 1).and_then(load_dst_and_payload),
        ) {
            a.0 == b.0
        } else {
            false
        };
        if !drop_curr {
            out.push(instrs[i].clone());
        }
        i += 1;
    }
    *instrs = out;
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

pub fn emit_ir_to_semcode(funcs: &[IrFunction], debug_symbols: bool) -> Result<Vec<u8>, FrontendError> {
    emit_semcode(funcs, debug_symbols)
}

fn emit_semcode(funcs: &[IrFunction], debug_symbols: bool) -> Result<Vec<u8>, FrontendError> {
    let mut out = Vec::new();
    if has_v1_math_instr(funcs) {
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
            let val = i32::try_from(*n).map_err(|_| FrontendError {
                pos: 0,
                message: format!("numeric literal {} does not fit in i32", n),
            })?;
            out.push(IrInstr::LoadI32 { dst: r, val });
            Ok((r, Type::I32))
        }
        Expr::Float(n) => {
            let r = alloc(next);
            out.push(IrInstr::LoadF64 { dst: r, val: *n });
            Ok((r, Type::F64))
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
        Expr::Call(name, args) => {
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
                let (r, t) = lower_expr(*arg, arena, next, out, env, fn_table)?;
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
            let (src, ty) = lower_expr(*inner, arena, next, out, env, fn_table)?;
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
            let (lr, lt) = lower_expr(*left, arena, next, out, env, fn_table)?;
            let (rr, rt) = lower_expr(*right, arena, next, out, env, fn_table)?;
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
        Stmt::Let { name, ty, value } => {
            let (reg, vty) = lower_expr(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                fn_table,
            )?;
            let final_ty = if let Some(ann) = ty { *ann } else { vty };
            env.insert(*name, final_ty);
            ctx.instrs.push(IrInstr::StoreVar {
                name: resolve_symbol_name(arena, *name)?.to_string(),
                src: reg,
            });
            Ok(())
        }
        Stmt::Expr(expr) => {
            lower_expr_stmt(*expr, arena, ctx, env, fn_table)?;
            Ok(())
        }
        Stmt::Return(v) => {
            match v {
                Some(e) => {
                    let (reg, ty) = lower_expr(
                        *e,
                        arena,
                        &mut ctx.next_reg,
                        &mut ctx.instrs,
                        env,
                        fn_table,
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
                    ctx.instrs.push(IrInstr::Ret { src: Some(reg) });
                }
                None => {
                    if ret_ty != Type::Unit {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "return without value in non-unit function ({:?})",
                                ret_ty
                            ),
                        });
                    }
                    ctx.instrs.push(IrInstr::Ret { src: None });
                }
            }
            Ok(())
        }
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

fn lower_expr_stmt(
    expr_id: ExprId,
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &ScopeEnv,
    fn_table: &FnTable,
) -> Result<(), FrontendError> {
    let expr = arena.expr(expr_id);
    if let Expr::Call(name, args) = expr {
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
            let (r, t) = lower_expr(*arg, arena, &mut ctx.next_reg, &mut ctx.instrs, env, fn_table)?;
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
            Some(alloc(&mut ctx.next_reg))
        };
        ctx.instrs.push(IrInstr::Call {
            dst,
            name: resolve_symbol_name(arena, *name)?.to_string(),
            args: regs,
        });
        return Ok(());
    }

    let _ = lower_expr(expr_id, arena, &mut ctx.next_reg, &mut ctx.instrs, env, fn_table)?;
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

    #[test]
    fn opt_removes_unreachable_and_noop_jmp() {
        let mut f = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::Jmp {
                    label: "l1".to_string(),
                },
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::Label {
                    name: "l1".to_string(),
                },
                IrInstr::Ret { src: None },
            ],
        };
        canonicalize_ir_function(&mut f);
        optimize_ir_function(&mut f);
        assert!(matches!(f.instrs[0], IrInstr::Label { .. }));
        assert!(f
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::LoadBool { dst: 0, val: true })));
    }

    #[test]
    fn opt_removes_redundant_consecutive_loads() {
        let mut f = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadI32 { dst: 1, val: 10 },
                IrInstr::LoadI32 { dst: 1, val: 11 },
                IrInstr::Ret { src: Some(1) },
            ],
        };
        optimize_ir_function(&mut f);
        let loads = f
            .instrs
            .iter()
            .filter(|i| matches!(i, IrInstr::LoadI32 { dst: 1, .. }))
            .count();
        assert_eq!(loads, 1);
        assert!(matches!(
            f.instrs[0],
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
        assert!(f.instrs.iter().any(|i| matches!(
            i,
            IrInstr::LoadBool {
                dst: 2,
                val: false
            }
        )));
        assert!(f.instrs.iter().any(|i| matches!(
            i,
            IrInstr::LoadF64 { dst: 5, val } if (*val - 5.0).abs() < f64::EPSILON
        )));
    }
}
