#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use sm_emit::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8,
    read_utf8, Opcode, SemcodeFormatError, SemcodeHeaderSpec, CAP_DEBUG_SYMBOLS, CAP_F64_MATH,
    CAP_FX_VALUES, CAP_GATE_SURFACE,
};
use sm_runtime_core::RuntimeQuotas;
use std::collections::HashSet;

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationCode {
    BadHeader,
    UnsupportedVersion,
    TruncatedFunction,
    InvalidFunctionName,
    InvalidStringTable,
    InvalidDebugSection,
    UnknownOpcode,
    OperandOutOfBounds,
    InvalidJumpTarget,
    InvalidStringReference,
    InvalidRegisterReference,
    UnknownCallTarget,
    ResourceLimitExceeded,
    CapabilityViolation,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationDiagnostic {
    pub code: VerificationCode,
    pub function: Option<String>,
    pub offset: Option<usize>,
    pub message: String,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectReport {
    pub diagnostics: Vec<VerificationDiagnostic>,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedFunction {
    pub name: String,
    pub code_len: usize,
    pub string_count: usize,
    pub debug_symbol_count: usize,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProgram {
    pub header: SemcodeHeaderSpec,
    pub functions: Vec<VerifiedFunction>,
}

#[cfg(feature = "std")]
impl core::fmt::Display for RejectReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (idx, diag) in self.diagnostics.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "verify error [{:?}]", diag.code)?;
            if let Some(function) = &diag.function {
                write!(f, " in '{}'", function)?;
            }
            if let Some(offset) = diag.offset {
                write!(f, " @0x{:04x}", offset)?;
            }
            write!(f, ": {}", diag.message)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RejectReport {}

#[cfg(feature = "std")]
pub fn verify_semcode(bytes: &[u8]) -> Result<VerifiedProgram, RejectReport> {
    let mut diagnostics = Vec::new();
    let quotas = RuntimeQuotas::verified_local();

    if bytes.len() < 8 {
        diagnostics.push(diag(
            VerificationCode::BadHeader,
            None,
            None,
            "SemCode file is shorter than the 8-byte header",
        ));
        return Err(RejectReport { diagnostics });
    }

    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[..8]);
    let Some(header) = header_spec_from_magic(&magic) else {
        diagnostics.push(diag(
            VerificationCode::UnsupportedVersion,
            None,
            Some(0),
            format!(
                "unsupported SemCode header '{}'",
                String::from_utf8_lossy(&magic)
            ),
        ));
        return Err(RejectReport { diagnostics });
    };

    let mut cursor = 8usize;
    let mut functions = Vec::new();
    let mut pending_functions = Vec::new();
    while cursor < bytes.len() {
        let function_start = cursor;
        let name_len = match read_u16_le(bytes, &mut cursor) {
            Ok(v) => v as usize,
            Err(_) => {
                diagnostics.push(diag(
                    VerificationCode::TruncatedFunction,
                    None,
                    Some(function_start),
                    "truncated function header while reading name length",
                ));
                break;
            }
        };
        if name_len == 0 {
            diagnostics.push(diag(
                VerificationCode::InvalidFunctionName,
                None,
                Some(function_start),
                "function name must not be empty",
            ));
            break;
        }
        let name = match read_utf8(bytes, &mut cursor, name_len) {
            Ok(v) => v,
            Err(_) => {
                diagnostics.push(diag(
                    VerificationCode::InvalidFunctionName,
                    None,
                    Some(function_start),
                    "function name is truncated or invalid utf-8",
                ));
                break;
            }
        };
        let code_len = match read_u32_le(bytes, &mut cursor) {
            Ok(v) => v as usize,
            Err(_) => {
                diagnostics.push(diag(
                    VerificationCode::TruncatedFunction,
                    Some(name.clone()),
                    Some(function_start),
                    "truncated function header while reading code length",
                ));
                break;
            }
        };
        if cursor + code_len > bytes.len() {
            diagnostics.push(diag(
                VerificationCode::TruncatedFunction,
                Some(name.clone()),
                Some(cursor),
                "function code extends past end of file",
            ));
            break;
        }
        let code = &bytes[cursor..cursor + code_len];
        cursor += code_len;

        match verify_function_code(&name, code, &header, &quotas) {
            Ok(function) => {
                functions.push(function.verified.clone());
                pending_functions.push(function);
            }
            Err(report) => diagnostics.extend(report.diagnostics),
        }
    }

    if pending_functions.len() > quotas.max_frames {
        diagnostics.push(diag(
            VerificationCode::ResourceLimitExceeded,
            None,
            None,
            format!(
                "program defines {} functions, which exceeds the verified-local frame budget of {}",
                pending_functions.len(),
                quotas.max_frames
            ),
        ));
    }

    let known_functions = pending_functions
        .iter()
        .map(|function| function.verified.name.as_str())
        .collect::<HashSet<_>>();
    for function in &pending_functions {
        for (offset, callee) in &function.call_targets {
            if known_functions.contains(callee.as_str()) {
                continue;
            }

            if let Some(required_capabilities) = builtin_call_required_capabilities(callee) {
                if header.capabilities & required_capabilities != required_capabilities {
                    diagnostics.push(diag(
                        VerificationCode::CapabilityViolation,
                        Some(function.verified.name.clone()),
                        Some(*offset),
                        format!(
                            "builtin call target '{}' requires capability bits 0x{required_capabilities:08x}",
                            callee
                        ),
                    ));
                }
                continue;
            }

            diagnostics.push(diag(
                VerificationCode::UnknownCallTarget,
                Some(function.verified.name.clone()),
                Some(*offset),
                format!(
                    "call target '{}' does not resolve to any function in this SemCode program",
                    callee
                ),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(VerifiedProgram { header, functions })
    } else {
        Err(RejectReport { diagnostics })
    }
}

#[cfg(feature = "std")]
fn verify_function_code(
    name: &str,
    code: &[u8],
    header: &SemcodeHeaderSpec,
    quotas: &RuntimeQuotas,
) -> Result<PendingVerifiedFunction, RejectReport> {
    let mut cursor = 0usize;
    let string_count = read_u16_le(code, &mut cursor)
        .map_err(|_| reject_one(name, VerificationCode::InvalidStringTable, 0, "missing string table header"))?
        as usize;
    if string_count > quotas.max_symbol_table {
        return Err(reject_one(
            name,
            VerificationCode::ResourceLimitExceeded,
            0,
            format!(
                "function string table uses {} entries, exceeding the verified-local symbol budget of {}",
                string_count, quotas.max_symbol_table
            ),
        ));
    }

    let mut strings = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        let len = read_u16_le(code, &mut cursor)
            .map_err(|_| reject_one(name, VerificationCode::InvalidStringTable, cursor, "truncated string length"))?
            as usize;
        let string = read_utf8(code, &mut cursor, len)
            .map_err(|_| reject_one(name, VerificationCode::InvalidStringTable, cursor, "invalid function string entry"))?;
        strings.push(string);
    }

    let mut debug_symbol_count = 0usize;
    let mut debug_pcs = Vec::new();
    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == b"DBG0" {
        cursor += 4;
        debug_symbol_count = read_u16_le(code, &mut cursor)
            .map_err(|_| reject_one(name, VerificationCode::InvalidDebugSection, cursor, "truncated debug section header"))?
            as usize;
        if debug_symbol_count > quotas.max_trace_entries {
            return Err(reject_one(
                name,
                VerificationCode::ResourceLimitExceeded,
                cursor,
                format!(
                    "debug section uses {} entries, exceeding the verified-local trace budget of {}",
                    debug_symbol_count, quotas.max_trace_entries
                ),
            ));
        }
        for _ in 0..debug_symbol_count {
            let pc = read_u32_le(code, &mut cursor)
                .map_err(|_| reject_one(name, VerificationCode::InvalidDebugSection, cursor, "truncated debug pc"))?;
            read_u32_le(code, &mut cursor)
                .map_err(|_| reject_one(name, VerificationCode::InvalidDebugSection, cursor, "truncated debug line"))?;
            read_u16_le(code, &mut cursor)
                .map_err(|_| reject_one(name, VerificationCode::InvalidDebugSection, cursor, "truncated debug column"))?;
            debug_pcs.push(pc as usize);
        }
    }

    let instr_start = cursor;
    let instr_len = code.len().saturating_sub(instr_start);
    let mut instr_starts = Vec::new();
    let mut jump_targets = Vec::new();
    let mut string_refs = Vec::new();
    let mut max_register: Option<usize> = None;
    let mut used_caps = 0u32;
    while cursor < code.len() {
        let offset = cursor - instr_start;
        instr_starts.push(offset);
        let opcode = read_u8(code, &mut cursor)
            .map_err(|_| reject_one(name, VerificationCode::UnknownOpcode, offset, "missing opcode byte"))?;
        let opcode = Opcode::from_byte(opcode).map_err(|err| match err {
            SemcodeFormatError::UnknownOpcode(_) => {
                reject_one(name, VerificationCode::UnknownOpcode, offset, err.to_string())
            }
            _ => reject_one(name, VerificationCode::OperandOutOfBounds, offset, err.to_string()),
        })?;
        let refs = decode_operands(name, code, &mut cursor, offset, opcode)?;
        jump_targets.extend(refs.jump_targets);
        string_refs.extend(refs.string_refs);
        used_caps |= refs.required_capabilities;
        max_register = match (max_register, refs.max_register) {
            (Some(lhs), Some(rhs)) => Some(lhs.max(rhs)),
            (None, some) => some,
            (some, None) => some,
        };
    }

    for pc in debug_pcs {
        if pc >= instr_len {
            return Err(reject_one(
                name,
                VerificationCode::InvalidDebugSection,
                pc,
                "debug symbol pc points past the instruction stream",
            ));
        }
    }

    if debug_symbol_count > 0 {
        used_caps |= CAP_DEBUG_SYMBOLS;
    }

    let missing_caps = used_caps & !header.capabilities;
    if missing_caps != 0 {
        return Err(reject_one(
            name,
            VerificationCode::CapabilityViolation,
            0,
            format!(
                "function requires capability bits 0x{missing_caps:08x}, but header '{}' provides only 0x{:08x}",
                String::from_utf8_lossy(&header.magic),
                header.capabilities
            ),
        ));
    }

    for target in jump_targets {
        if target >= instr_len {
            return Err(reject_one(
                name,
                VerificationCode::InvalidJumpTarget,
                target,
                "jump target points past the instruction stream",
            ));
        }
        if !instr_starts.contains(&target) {
            return Err(reject_one(
                name,
                VerificationCode::InvalidJumpTarget,
                target,
                "jump target does not land on an instruction boundary",
            ));
        }
    }

    let mut call_targets = Vec::new();
    for (offset, sid, usage) in string_refs {
        if sid >= string_count {
            return Err(reject_one(
                name,
                VerificationCode::InvalidStringReference,
                offset,
                format!("{usage} uses missing string id s{sid}"),
            ));
        }
        if usage == "call target" {
            call_targets.push((offset, strings[sid].clone()));
        }
    }

    if let Some(max_register) = max_register {
        let used = max_register + 1;
        if used > quotas.max_registers {
            return Err(reject_one(
                name,
                VerificationCode::InvalidRegisterReference,
                used - 1,
                format!(
                    "function references register r{}, exceeding the verified-local register budget of {}",
                    max_register, quotas.max_registers
                ),
            ));
        }
    }

    Ok(PendingVerifiedFunction {
        verified: VerifiedFunction {
            name: name.to_string(),
            code_len: code.len(),
            string_count,
            debug_symbol_count,
        },
        call_targets,
    })
}

#[cfg(feature = "std")]
fn decode_operands(
    function: &str,
    code: &[u8],
    cursor: &mut usize,
    offset: usize,
    opcode: Opcode,
) -> Result<OperandRefs, RejectReport> {
    let invalid = |msg: &str| reject_one(function, VerificationCode::OperandOutOfBounds, offset, msg);
    let mut refs = OperandRefs::default();
    let mut mark_reg = |reg: u16| {
        let reg = reg as usize;
        refs.max_register = Some(refs.max_register.map_or(reg, |current| current.max(reg)));
    };

    match opcode {
        Opcode::LoadQ => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_u8(code, cursor).map_err(|_| invalid("truncated quad literal"))?;
        }
        Opcode::LoadBool => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_u8(code, cursor).map_err(|_| invalid("truncated bool literal"))?;
        }
        Opcode::LoadI32 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            read_i32_le(code, cursor).map_err(|_| invalid("truncated i32 literal"))?;
        }
        Opcode::LoadF64 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_F64_MATH;
            read_f64_le(code, cursor).map_err(|_| invalid("truncated f64 literal"))?;
        }
        Opcode::LoadFx => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_FX_VALUES;
            read_i32_le(code, cursor).map_err(|_| invalid("truncated fx literal"))?;
        }
        Opcode::LoadVar => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            mark_reg(dst);
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated variable string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "variable reference"));
        }
        Opcode::StoreVar => {
            let sid = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated variable string id"))?;
            refs.string_refs
                .push((offset, sid as usize, "variable reference"));
            let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated src register"))?;
            mark_reg(src);
        }
        Opcode::QNot | Opcode::BoolNot => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated src register"))?;
            mark_reg(dst);
            mark_reg(src);
        }
        Opcode::QAnd
        | Opcode::QOr
        | Opcode::QImpl
        | Opcode::BoolAnd
        | Opcode::BoolOr
        | Opcode::CmpEq
        | Opcode::CmpNe
        | Opcode::AddF64
        | Opcode::SubF64
        | Opcode::MulF64
        | Opcode::DivF64 => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated dst register"))?;
            let lhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated lhs register"))?;
            let rhs = read_u16_le(code, cursor).map_err(|_| invalid("truncated rhs register"))?;
            mark_reg(dst);
            mark_reg(lhs);
            mark_reg(rhs);
            if matches!(
                opcode,
                Opcode::AddF64 | Opcode::SubF64 | Opcode::MulF64 | Opcode::DivF64
            ) {
                refs.required_capabilities |= CAP_F64_MATH;
            }
        }
        Opcode::Jmp => {
            let target = read_u32_le(code, cursor).map_err(|_| invalid("truncated jump target"))?;
            refs.jump_targets.push(target as usize);
        }
        Opcode::JmpIf => {
            let cond = read_u16_le(code, cursor).map_err(|_| invalid("truncated condition register"))?;
            mark_reg(cond);
            let target = read_u32_le(code, cursor).map_err(|_| invalid("truncated jump target"))?;
            refs.jump_targets.push(target as usize);
        }
        Opcode::Call => {
            read_u8(code, cursor).map_err(|_| invalid("truncated call destination flag"))?;
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated call dst register"))?;
            mark_reg(dst);
            let sid = read_u16_le(code, cursor).map_err(|_| invalid("truncated callee string id"))?;
            refs.string_refs.push((offset, sid as usize, "call target"));
            let argc = read_u16_le(code, cursor).map_err(|_| invalid("truncated argc"))? as usize;
            for _ in 0..argc {
                let arg = read_u16_le(code, cursor).map_err(|_| invalid("truncated call arg register"))?;
                mark_reg(arg);
            }
        }
        Opcode::Assert => {
            let cond = read_u16_le(code, cursor)
                .map_err(|_| invalid("truncated assert condition register"))?;
            mark_reg(cond);
        }
        Opcode::GateRead => {
            let dst = read_u16_le(code, cursor).map_err(|_| invalid("truncated gate dst register"))?;
            mark_reg(dst);
            refs.required_capabilities |= CAP_GATE_SURFACE;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate device id"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate port"))?;
        }
        Opcode::GateWrite => {
            refs.required_capabilities |= CAP_GATE_SURFACE;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate device id"))?;
            read_u16_le(code, cursor).map_err(|_| invalid("truncated gate port"))?;
            let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated gate src register"))?;
            mark_reg(src);
        }
        Opcode::PulseEmit => {
            refs.required_capabilities |= CAP_GATE_SURFACE;
            let sid = read_u16_le(code, cursor).map_err(|_| invalid("truncated signal string id"))?;
            refs.string_refs.push((offset, sid as usize, "pulse signal"));
        }
        Opcode::Ret => {
            let has_src = read_u8(code, cursor).map_err(|_| invalid("truncated return flag"))?;
            if has_src != 0 {
                let src = read_u16_le(code, cursor).map_err(|_| invalid("truncated return src register"))?;
                mark_reg(src);
            }
        }
    }

    Ok(refs)
}

#[cfg(feature = "std")]
fn builtin_call_required_capabilities(name: &str) -> Option<u32> {
    match name {
        "sin" | "cos" | "tan" | "sqrt" | "abs" | "pow" => Some(CAP_F64_MATH),
        _ => None,
    }
}

#[cfg(feature = "std")]
#[derive(Default)]
struct OperandRefs {
    jump_targets: Vec<usize>,
    string_refs: Vec<(usize, usize, &'static str)>,
    max_register: Option<usize>,
    required_capabilities: u32,
}

#[cfg(feature = "std")]
struct PendingVerifiedFunction {
    verified: VerifiedFunction,
    call_targets: Vec<(usize, String)>,
}

#[cfg(feature = "std")]
fn reject_one(
    function: &str,
    code: VerificationCode,
    offset: usize,
    message: impl Into<String>,
) -> RejectReport {
    RejectReport {
        diagnostics: vec![diag(code, Some(function.to_string()), Some(offset), message)],
    }
}

#[cfg(feature = "std")]
fn diag(
    code: VerificationCode,
    function: Option<String>,
    offset: Option<usize>,
    message: impl Into<String>,
) -> VerificationDiagnostic {
    VerificationDiagnostic {
        code,
        function,
        offset,
        message: message.into(),
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_emit::{
        compile_program_to_semcode, compile_program_to_semcode_with_options_debug, CompileProfile,
        OptLevel,
    };

    #[test]
    fn verifier_accepts_valid_semcode() {
        let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_accepts_fx_semcode() {
        let src = r#"
            fn id(x: fx) -> fx { return x; }
            fn main() {
                let x: fx = 1.25;
                let y: fx = id(-2.0);
                if x == x { return; } else { return; }
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 3);
    }

    #[test]
    fn verifier_accepts_cli_o0_f64_arithmetic_storevar_layout() {
        let src = r#"
            fn main() {
                let y: f64 = 1.0 + 2.0;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode_with_options_debug(
            src,
            CompileProfile::Auto,
            OptLevel::O0,
            false,
        )
        .expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 2);
    }

    #[test]
    fn verifier_accepts_builtin_f64_call_targets() {
        let src = r#"
            fn main() {
                let y: f64 = sqrt(16.0);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode_with_options_debug(
            src,
            CompileProfile::Auto,
            OptLevel::O0,
            false,
        )
        .expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.header.rev, 2);
    }

    #[test]
    fn verifier_accepts_assert_opcode() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let verified = verify_semcode(&bytes).expect("verify");
        assert_eq!(verified.functions.len(), 1);
    }

    #[test]
    fn verifier_rejects_short_header() {
        let report = verify_semcode(b"SEMC").expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::BadHeader);
    }

    #[test]
    fn verifier_rejects_unknown_opcode() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;
        bytes[opcode_pos] = 0xff;
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::UnknownOpcode);
    }

    #[test]
    fn verifier_rejects_truncated_function_body() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        bytes.truncate(bytes.len() - 1);
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::TruncatedFunction);
    }

    #[test]
    fn verifier_rejects_truncated_string_table() {
        let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
        let code_len_pos = 8 + 2 + 4;
        bytes[code_len_pos..code_len_pos + 4].copy_from_slice(&1u32.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::InvalidStringTable);
    }

    #[test]
    fn verifier_rejects_jump_past_instruction_stream() {
        let mut bytes = compile_program_to_semcode("fn main() { if true { return; } return; }")
            .expect("compile");
        let opcode_pos = find_opcode(&bytes, Opcode::JmpIf.byte()).expect("jmpif");
        let target_pos = opcode_pos + 1 + 2;
        bytes[target_pos..target_pos + 4].copy_from_slice(&999u32.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::InvalidJumpTarget);
    }

    #[test]
    fn verifier_rejects_bad_string_reference() {
        let mut bytes = compile_program_to_semcode(
            "fn helper() { return; } fn main() { helper(); return; }",
        )
        .expect("compile");
        let opcode_pos = find_opcode(&bytes, Opcode::Call.byte()).expect("call");
        let sid_pos = opcode_pos + 1 + 1 + 2;
        bytes[sid_pos..sid_pos + 2].copy_from_slice(&99u16.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidStringReference
        );
    }

    #[test]
    fn verifier_rejects_register_past_verified_local_budget() {
        let mut bytes =
            compile_program_to_semcode("fn main() { let a: bool = true; return; }").expect("compile");
        let opcode_pos = find_opcode(&bytes, Opcode::LoadBool.byte()).expect("load bool");
        let dst_pos = opcode_pos + 1;
        bytes[dst_pos..dst_pos + 2].copy_from_slice(&5000u16.to_le_bytes());
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(
            report.diagnostics[0].code,
            VerificationCode::InvalidRegisterReference
        );
    }

    #[test]
    fn verifier_rejects_unknown_call_target() {
        let mut bytes = compile_program_to_semcode(
            "fn helper() { return; } fn main() { helper(); return; }",
        )
        .expect("compile");
        let helper_pos = bytes
            .windows(b"helper".len())
            .rposition(|window| window == b"helper")
            .expect("helper string");
        bytes[helper_pos..helper_pos + b"helper".len()].copy_from_slice(b"gh0st!");
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::UnknownCallTarget);
    }

    #[test]
    fn verifier_rejects_f64_ops_under_v0_capabilities() {
        let src = r#"
            fn main() {
                let x: f64 = 1.0 + 2.0;
                return;
            }
        "#;
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        bytes[7] = b'0';
        let report = verify_semcode(&bytes).expect_err("must reject");
        assert_eq!(report.diagnostics[0].code, VerificationCode::CapabilityViolation);
    }

    fn find_opcode(bytes: &[u8], opcode: u8) -> Option<usize> {
        bytes.iter().position(|byte| *byte == opcode)
    }
}
