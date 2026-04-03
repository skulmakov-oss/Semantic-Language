use crate::semcode_format::{
    header_spec_from_magic, read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8,
    supported_headers, Opcode, SemcodeFormatError, SemcodeHeaderSpec,
};
use crate::QuadVal;
use prom_abi::{AbiError, AbiValue, HostCallId, PrometheusHostAbi};
use prom_cap::{CapabilityChecker, CapabilityDenied};
use sm_runtime_core::{
    AdtCarrier, ExecutionConfig, ExecutionContext, QuotaExceeded, QuotaKind, RecordCarrier,
    RuntimeQuotas, RuntimeSymbolTable, RuntimeTrap, SymbolId,
};
use sm_verify::verify_semcode;
use sm_verify::RejectReport;
use std::collections::{HashMap, HashSet};

const MAX_FUNCTIONS: usize = 4096;
const MAX_STRINGS_PER_FUNCTION: usize = 4096;
const MAX_STRING_LEN: usize = 8192;
const MAX_DEBUG_SYMBOLS_PER_FUNCTION: usize = 8192;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Quad(QuadVal),
    Bool(bool),
    I32(i32),
    F64(f64),
    U32(u32),
    Fx(i32),
    Tuple(Vec<Value>),
    Record(RecordCarrier<Value>),
    Adt(AdtCarrier<Value>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub pc: usize,
    pub regs: Vec<Value>,
    pub locals: HashMap<SymbolId, Value>,
    pub func: String,
    pub return_dst: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct FunctionBytecode {
    pub name: String,
    pub strings: Vec<String>,
    pub symbol_ids: Vec<SymbolId>,
    pub debug_symbols: Vec<DebugSymbol>,
    pub code: Vec<u8>,
    pub instr_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSymbol {
    pub pc: usize,
    pub line: u32,
    pub col: u16,
}

#[derive(Debug, Clone)]
pub struct VM {
    pub functions: HashMap<String, FunctionBytecode>,
    pub callstack: Vec<Frame>,
    pub config: ExecutionConfig,
    pub effect_calls: usize,
    pub symbols: RuntimeSymbolTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    BadHeader,
    UnsupportedBytecodeVersion { found: String, supported: String },
    BadFormat(String),
    UnknownFunction(String),
    InvalidJumpAddress { func: String, addr: usize },
    TypeMismatchRuntime(String),
    StackUnderflow,
    StackOverflow,
    QuotaExceeded(QuotaExceeded),
    VerifierRejected(RejectReport),
    UnknownVariable(String),
    InvalidStringId(u16),
    HostAbi(AbiError),
    CapabilityDenied(CapabilityDenied),
    Trap(RuntimeTrap),
}

impl core::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::BadHeader => write!(f, "bad SemCode header"),
            RuntimeError::UnsupportedBytecodeVersion { found, supported } => write!(
                f,
                "unsupported SemCode version '{}'; supported versions: {}. hint: recompile source with current semantic",
                found, supported
            ),
            RuntimeError::BadFormat(m) => write!(f, "bad SemCode format: {}", m),
            RuntimeError::UnknownFunction(n) => write!(f, "unknown function '{}'", n),
            RuntimeError::InvalidJumpAddress { func, addr } => {
                write!(f, "invalid jump address {} in '{}'", addr, func)
            }
            RuntimeError::TypeMismatchRuntime(m) => write!(f, "runtime type mismatch: {}", m),
            RuntimeError::StackUnderflow => write!(f, "stack underflow"),
            RuntimeError::StackOverflow => write!(f, "stack overflow"),
            RuntimeError::QuotaExceeded(exceeded) => write!(
                f,
                "quota exceeded: {:?} limit={} used={}",
                exceeded.kind, exceeded.limit, exceeded.used
            ),
            RuntimeError::VerifierRejected(report) => write!(f, "{report}"),
            RuntimeError::UnknownVariable(n) => write!(f, "unknown variable '{}'", n),
            RuntimeError::InvalidStringId(id) => write!(f, "invalid string id {}", id),
            RuntimeError::HostAbi(err) => write!(f, "{err}"),
            RuntimeError::CapabilityDenied(err) => write!(f, "{err}"),
            RuntimeError::Trap(RuntimeTrap::AssertionFailed) => write!(f, "assertion failed"),
            RuntimeError::Trap(trap) => write!(f, "runtime trap: {:?}", trap),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub fn run_semcode(bytes: &[u8]) -> Result<(), RuntimeError> {
    run_semcode_with_config(
        bytes,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_verified_semcode(bytes: &[u8]) -> Result<(), RuntimeError> {
    run_verified_semcode_with_config(
        bytes,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_semcode_with_entry(bytes: &[u8], entry: &str) -> Result<(), RuntimeError> {
    run_semcode_with_entry_and_config(
        bytes,
        entry,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_semcode_with_config(bytes: &[u8], config: ExecutionConfig) -> Result<(), RuntimeError> {
    run_semcode_with_entry_and_config(bytes, "main", config)
}

pub fn run_verified_semcode_with_config(
    bytes: &[u8],
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    run_verified_semcode_with_entry_and_config(bytes, "main", config)
}

pub fn run_verified_semcode_with_entry(bytes: &[u8], entry: &str) -> Result<(), RuntimeError> {
    run_verified_semcode_with_entry_and_config(
        bytes,
        entry,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
}

pub fn run_verified_semcode_with_entry_and_config(
    bytes: &[u8],
    entry: &str,
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    verify_semcode(bytes).map_err(RuntimeError::VerifierRejected)?;
    run_semcode_with_entry_and_config(bytes, entry, config)
}

pub fn run_verified_semcode_with_host_and_capabilities<
    H: PrometheusHostAbi,
    C: CapabilityChecker,
>(
    bytes: &[u8],
    host: &mut H,
    capabilities: &C,
) -> Result<(), RuntimeError> {
    run_verified_semcode_with_host_and_capabilities_and_config(
        bytes,
        "main",
        host,
        capabilities,
        ExecutionConfig::for_context(ExecutionContext::KernelBound),
    )
}

pub fn run_verified_semcode_with_host_and_capabilities_and_config<
    H: PrometheusHostAbi,
    C: CapabilityChecker,
>(
    bytes: &[u8],
    entry: &str,
    host: &mut H,
    capabilities: &C,
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    verify_semcode(bytes).map_err(RuntimeError::VerifierRejected)?;
    let (_, symbols, functions) = parse_semcode(bytes)?;
    let mut vm = VM {
        functions,
        callstack: Vec::new(),
        config,
        effect_calls: 0,
        symbols,
    };
    push_frame(&mut vm, entry, Vec::new(), None)?;
    let mut bridge = PrometheusVmHost { host, capabilities };
    exec_loop(&mut vm, &mut bridge)
}

pub fn run_semcode_with_entry_and_config(
    bytes: &[u8],
    entry: &str,
    config: ExecutionConfig,
) -> Result<(), RuntimeError> {
    let (_, symbols, functions) = parse_semcode(bytes)?;
    let mut vm = VM {
        functions,
        callstack: Vec::new(),
        config,
        effect_calls: 0,
        symbols,
    };
    push_frame(&mut vm, entry, Vec::new(), None)?;
    let mut host = LegacyVmHost;
    exec_loop(&mut vm, &mut host)
}

pub fn disasm_semcode(bytes: &[u8]) -> Result<String, RuntimeError> {
    let (spec, _, functions) = parse_semcode(bytes)?;
    let mut out = String::new();
    let header = if bytes.len() >= 8 { &bytes[0..8] } else { &[] };
    out.push_str(&format!(
        "{} epoch={}.{} caps=0x{:08x}\n",
        String::from_utf8_lossy(header),
        spec.epoch,
        spec.rev,
        spec.capabilities
    ));
    for f in functions.values() {
        out.push_str(&format!(
            "fn {}: code={} bytes, strings={}, debug={}\n",
            f.name,
            f.code.len(),
            f.strings.len(),
            f.debug_symbols.len()
        ));
        let mut pc = 0usize;
        while pc < f.code.len().saturating_sub(f.instr_start) {
            let (line, next) = disasm_one(f, pc)?;
            out.push_str(&format!("  {:04x}: {}\n", pc, line));
            pc = next;
        }
    }
    Ok(out)
}

fn parse_semcode(
    bytes: &[u8],
) -> Result<
    (
        SemcodeHeaderSpec,
        RuntimeSymbolTable,
        HashMap<String, FunctionBytecode>,
    ),
    RuntimeError,
> {
    if bytes.len() < 8 {
        return Err(RuntimeError::BadHeader);
    }
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let Some(header) = header_spec_from_magic(&magic) else {
        if &magic[0..7] == b"SEMCODE" {
            return Err(RuntimeError::UnsupportedBytecodeVersion {
                found: String::from_utf8_lossy(&magic).to_string(),
                supported: supported_headers()
                    .iter()
                    .map(|h| String::from_utf8_lossy(&h.magic).to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
        return Err(RuntimeError::BadHeader);
    };
    let mut i = 8usize;
    let mut out = HashMap::new();
    let mut runtime_symbols = RuntimeSymbolTable::new();
    while i < bytes.len() {
        if out.len() >= MAX_FUNCTIONS {
            return Err(RuntimeError::BadFormat(format!(
                "too many functions (>{})",
                MAX_FUNCTIONS
            )));
        }
        let name_len = read_u16_le(bytes, &mut i).map_err(map_format_err)? as usize;
        if name_len == 0 {
            return Err(RuntimeError::BadFormat("empty function name".to_string()));
        }
        if name_len > MAX_STRING_LEN {
            return Err(RuntimeError::BadFormat(format!(
                "function name too long: {}",
                name_len
            )));
        }
        let name = read_utf8(bytes, &mut i, name_len).map_err(map_format_err)?;
        let code_len = read_u32_le(bytes, &mut i).map_err(map_format_err)? as usize;
        if i + code_len > bytes.len() {
            return Err(RuntimeError::BadFormat(
                "function code out of bounds".to_string(),
            ));
        }
        let code = bytes[i..i + code_len].to_vec();
        i += code_len;

        let (strings, debug_symbols, instr_start) = parse_string_table_and_debug(&code)?;
        let symbol_ids = strings
            .iter()
            .map(|name| runtime_symbols.intern(name))
            .collect::<Vec<_>>();
        let f = FunctionBytecode {
            name: name.clone(),
            strings,
            symbol_ids,
            debug_symbols,
            code,
            instr_start,
        };
        validate_function_bytecode(&f)?;
        if out.insert(name.clone(), f).is_some() {
            return Err(RuntimeError::BadFormat(format!(
                "duplicate function '{}'",
                name
            )));
        }
    }
    Ok((header, runtime_symbols, out))
}

fn parse_string_table_and_debug(
    code: &[u8],
) -> Result<(Vec<String>, Vec<DebugSymbol>, usize), RuntimeError> {
    let mut i = 0usize;
    let count = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
    if count > MAX_STRINGS_PER_FUNCTION {
        return Err(RuntimeError::BadFormat(format!(
            "too many strings in function: {} (max {})",
            count, MAX_STRINGS_PER_FUNCTION
        )));
    }
    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let len = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
        if len > MAX_STRING_LEN {
            return Err(RuntimeError::BadFormat(format!(
                "string too long in function string table: {}",
                len
            )));
        }
        strings.push(read_utf8(code, &mut i, len).map_err(map_format_err)?);
    }
    let mut debug_symbols = Vec::new();
    if i + 4 <= code.len() && &code[i..i + 4] == b"DBG0" {
        i += 4;
        let count = read_u16_le(code, &mut i).map_err(map_format_err)? as usize;
        if count > MAX_DEBUG_SYMBOLS_PER_FUNCTION {
            return Err(RuntimeError::BadFormat(format!(
                "too many debug symbols in function: {} (max {})",
                count, MAX_DEBUG_SYMBOLS_PER_FUNCTION
            )));
        }
        debug_symbols.reserve(count);
        for _ in 0..count {
            let pc = read_u32_le(code, &mut i).map_err(map_format_err)? as usize;
            let line = read_u32_le(code, &mut i).map_err(map_format_err)?;
            let col = read_u16_le(code, &mut i).map_err(map_format_err)?;
            debug_symbols.push(DebugSymbol { pc, line, col });
        }
    }
    Ok((strings, debug_symbols, i))
}

fn map_format_err(err: SemcodeFormatError) -> RuntimeError {
    match err {
        SemcodeFormatError::UnexpectedEof => RuntimeError::BadFormat("unexpected EOF".to_string()),
        SemcodeFormatError::InvalidUtf8 => RuntimeError::BadFormat("invalid utf8".to_string()),
        SemcodeFormatError::UnknownOpcode(op) => {
            RuntimeError::BadFormat(format!("unknown opcode 0x{:02x}", op))
        }
    }
}

fn validate_function_bytecode(f: &FunctionBytecode) -> Result<(), RuntimeError> {
    if f.instr_start > f.code.len() {
        return Err(RuntimeError::BadFormat(format!(
            "invalid instr_start in '{}'",
            f.name
        )));
    }
    let mut cur = f.instr_start;
    let mut starts: HashSet<usize> = HashSet::new();
    let mut jumps: Vec<usize> = Vec::new();
    while cur < f.code.len() {
        starts.insert(cur - f.instr_start);
        let opcode = Opcode::from_byte(read_u8(&f.code, &mut cur).map_err(map_format_err)?)
            .map_err(map_format_err)?;
        match opcode {
            Opcode::LoadQ => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadBool => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadI32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::AddI32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadU32 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadF64 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_f64_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadFx => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::MakeTuple => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                if count < 2 {
                    return Err(RuntimeError::BadFormat(format!(
                        "tuple literal arity must be at least 2 in '{}'",
                        f.name
                    )));
                }
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::MakeRecord => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                if count == 0 {
                    return Err(RuntimeError::BadFormat(format!(
                        "record literal slot count must be at least 1 in '{}'",
                        f.name
                    )));
                }
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::MakeAdt => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                for _ in 0..count {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::AdtTag => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::AdtGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::RecordGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::TupleGet => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::LoadVar => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::StoreVar => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::QNot | Opcode::BoolNot => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::QAnd
            | Opcode::QOr
            | Opcode::QImpl
            | Opcode::BoolAnd
            | Opcode::BoolOr
            | Opcode::CmpEq
            | Opcode::CmpNe
            | Opcode::CmpI32Lt
            | Opcode::CmpI32Le
            | Opcode::AddF64
            | Opcode::SubF64
            | Opcode::MulF64
            | Opcode::DivF64 => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::Jmp => {
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                jumps.push(addr);
            }
            Opcode::JmpIf => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                jumps.push(addr);
            }
            Opcode::Call => {
                let _ = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let argc = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                for _ in 0..argc {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
            Opcode::Assert => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::GateRead => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::GateWrite => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::PulseEmit => {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            Opcode::Ret => {
                let has_src = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                if has_src {
                    let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                }
            }
        }
    }
    if cur != f.code.len() {
        return Err(RuntimeError::BadFormat(format!(
            "trailing bytes in '{}'",
            f.name
        )));
    }
    let instr_len = f.code.len().saturating_sub(f.instr_start);
    for ds in &f.debug_symbols {
        if ds.pc >= instr_len {
            return Err(RuntimeError::BadFormat(format!(
                "debug pc out of range in '{}': {}",
                f.name, ds.pc
            )));
        }
    }
    for addr in jumps {
        if addr >= instr_len {
            return Err(RuntimeError::BadFormat(format!(
                "jump out of range in '{}': {}",
                f.name, addr
            )));
        }
        if !starts.contains(&addr) {
            return Err(RuntimeError::BadFormat(format!(
                "jump to non-instruction boundary in '{}': {}",
                f.name, addr
            )));
        }
    }
    Ok(())
}

trait VmHostBridge {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError>;
    fn gate_write(&mut self, device_id: u16, port: u16, value: Value) -> Result<(), RuntimeError>;
    fn pulse_emit(&mut self, signal: &str) -> Result<(), RuntimeError>;
}

struct LegacyVmHost;

impl VmHostBridge for LegacyVmHost {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError> {
        Ok(Value::I32(((device_id as i32) << 16) | (port as i32)))
    }

    fn gate_write(
        &mut self,
        _device_id: u16,
        _port: u16,
        _value: Value,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn pulse_emit(&mut self, _signal: &str) -> Result<(), RuntimeError> {
        Ok(())
    }
}

struct PrometheusVmHost<'a, H: PrometheusHostAbi, C: CapabilityChecker> {
    host: &'a mut H,
    capabilities: &'a C,
}

impl<'a, H: PrometheusHostAbi, C: CapabilityChecker> VmHostBridge for PrometheusVmHost<'a, H, C> {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<Value, RuntimeError> {
        self.capabilities
            .require_call(HostCallId::GateRead)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .gate_read(device_id, port)
            .map(value_from_abi)
            .map_err(RuntimeError::HostAbi)
    }

    fn gate_write(&mut self, device_id: u16, port: u16, value: Value) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::GateWrite)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host
            .gate_write(device_id, port, value_to_abi(value)?)
            .map_err(RuntimeError::HostAbi)
    }

    fn pulse_emit(&mut self, signal: &str) -> Result<(), RuntimeError> {
        self.capabilities
            .require_call(HostCallId::PulseEmit)
            .map_err(RuntimeError::CapabilityDenied)?;
        self.host.pulse_emit(signal).map_err(RuntimeError::HostAbi)
    }
}

fn exec_loop<H: VmHostBridge>(vm: &mut VM, host: &mut H) -> Result<(), RuntimeError> {
    loop {
        let Some(frame_idx) = vm.callstack.len().checked_sub(1) else {
            return Ok(());
        };
        let func_name = vm.callstack[frame_idx].func.clone();
        let f = vm
            .functions
            .get(&func_name)
            .cloned()
            .ok_or_else(|| RuntimeError::UnknownFunction(func_name.clone()))?;
        let pc = vm.callstack[frame_idx].pc;
        let instr_rel_len = f.code.len().saturating_sub(f.instr_start);
        if pc >= instr_rel_len {
            return Err(RuntimeError::BadFormat(format!(
                "pc out of range in '{}': {}",
                func_name, pc
            )));
        }
        let mut cur = f.instr_start + pc;
        let opcode = Opcode::from_byte(read_u8(&f.code, &mut cur).map_err(map_format_err)?)
            .map_err(map_format_err)?;
        let next_pc: usize;

        match opcode {
            Opcode::LoadQ => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let q = match read_u8(&f.code, &mut cur).map_err(map_format_err)? {
                    0 => QuadVal::N,
                    1 => QuadVal::F,
                    2 => QuadVal::T,
                    3 => QuadVal::S,
                    v => {
                        return Err(RuntimeError::BadFormat(format!(
                            "invalid quad literal {}",
                            v
                        )))
                    }
                };
                set_reg(vm, frame_idx, dst, Value::Quad(q))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadBool => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let b = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                set_reg(vm, frame_idx, dst, Value::Bool(b))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::I32(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AddI32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = l.wrapping_add(r);
                set_reg(vm, frame_idx, dst, Value::I32(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadU32 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::U32(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadF64 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_f64_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::F64(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadFx => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let v = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
                set_reg(vm, frame_idx, dst, Value::Fx(v))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeTuple => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    items.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(vm, frame_idx, dst, Value::Tuple(items))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeRecord => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let type_name = lookup_str(&f, sid)?.to_string();
                let mut slots = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    slots.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(
                    vm,
                    frame_idx,
                    dst,
                    Value::Record(RecordCarrier { type_name, slots }),
                )?;
                next_pc = cur - f.instr_start;
            }
            Opcode::MakeAdt => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let variant_sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let tag = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let type_name = lookup_str(&f, sid)?.to_string();
                let variant_name = lookup_str(&f, variant_sid)?.to_string();
                let mut payload = Vec::with_capacity(count);
                for _ in 0..count {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    payload.push(get_reg(vm, frame_idx, src)?);
                }
                set_reg(
                    vm,
                    frame_idx,
                    dst,
                    Value::Adt(AdtCarrier {
                        type_name,
                        variant_name,
                        tag,
                        payload,
                    }),
                )?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AdtTag => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let expected_name = lookup_str(&f, sid)?.to_string();
                let adt = get_reg(vm, frame_idx, src)?;
                let Value::Adt(adt) = adt else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "ADT_TAG source must be enum".to_string(),
                    ));
                };
                if adt.type_name != expected_name {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "ADT_TAG expected enum '{}', got '{}'",
                        expected_name, adt.type_name
                    )));
                }
                set_reg(vm, frame_idx, dst, Value::I32(i32::from(adt.tag)))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AdtGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let expected_name = lookup_str(&f, sid)?.to_string();
                let adt = get_reg(vm, frame_idx, src)?;
                let Value::Adt(adt) = adt else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "ADT_GET source must be enum".to_string(),
                    ));
                };
                if adt.type_name != expected_name {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "ADT_GET expected enum '{}', got '{}'",
                        expected_name, adt.type_name
                    )));
                }
                let item = adt.payload.get(index).cloned().ok_or_else(|| {
                    RuntimeError::BadFormat(format!("adt-get index out of bounds: {}", index))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::RecordGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let expected_name = lookup_str(&f, sid)?.to_string();
                let record = get_reg(vm, frame_idx, src)?;
                let Value::Record(record) = record else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "RECORD_GET source must be record".to_string(),
                    ));
                };
                if record.type_name != expected_name {
                    return Err(RuntimeError::TypeMismatchRuntime(format!(
                        "RECORD_GET expected record '{}', got '{}'",
                        expected_name, record.type_name
                    )));
                }
                let item = record.slots.get(index).cloned().ok_or_else(|| {
                    RuntimeError::BadFormat(format!("record-get index out of bounds: {}", index))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::TupleGet => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let index = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let tuple = get_reg(vm, frame_idx, src)?;
                let Value::Tuple(items) = tuple else {
                    return Err(RuntimeError::TypeMismatchRuntime(
                        "TUPLE_GET source must be tuple".to_string(),
                    ));
                };
                let item = items.get(index).cloned().ok_or_else(|| {
                    RuntimeError::BadFormat(format!("tuple-get index out of bounds: {}", index))
                })?;
                set_reg(vm, frame_idx, dst, item)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::LoadVar => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let symbol = lookup_symbol(&f, sid)?;
                let val = vm.callstack[frame_idx]
                    .locals
                    .get(&symbol)
                    .cloned()
                    .ok_or_else(|| {
                        let name = lookup_str(&f, sid).unwrap_or("<unknown>");
                        RuntimeError::UnknownVariable(name.to_string())
                    })?;
                set_reg(vm, frame_idx, dst, val)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::StoreVar => {
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let symbol = lookup_symbol(&f, sid)?;
                let val = get_reg(vm, frame_idx, src)?;
                vm.callstack[frame_idx].locals.insert(symbol, val);
                next_pc = cur - f.instr_start;
            }
            Opcode::QAnd | Opcode::QOr | Opcode::QImpl => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lq = as_quad(get_reg(vm, frame_idx, lhs)?)?;
                let rq = as_quad(get_reg(vm, frame_idx, rhs)?)?;
                let out_q = match opcode {
                    Opcode::QAnd => quad_and(lq, rq),
                    Opcode::QOr => quad_or(lq, rq),
                    Opcode::QImpl => quad_or(quad_not(lq), rq),
                    _ => unreachable!(),
                };
                set_reg(vm, frame_idx, dst, Value::Quad(out_q))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::QNot => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let q = as_quad(get_reg(vm, frame_idx, src)?)?;
                set_reg(vm, frame_idx, dst, Value::Quad(quad_not(q)))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::BoolAnd | Opcode::BoolOr => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lb = as_bool(get_reg(vm, frame_idx, lhs)?)?;
                let rb = as_bool(get_reg(vm, frame_idx, rhs)?)?;
                let out_b = if opcode == Opcode::BoolAnd {
                    lb && rb
                } else {
                    lb || rb
                };
                set_reg(vm, frame_idx, dst, Value::Bool(out_b))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::BoolNot => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let b = as_bool(get_reg(vm, frame_idx, src)?)?;
                set_reg(vm, frame_idx, dst, Value::Bool(!b))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::CmpEq | Opcode::CmpNe => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lv = get_reg(vm, frame_idx, lhs)?;
                let rv = get_reg(vm, frame_idx, rhs)?;
                let eq = value_eq(&lv, &rv)?;
                let out = if opcode == Opcode::CmpEq { eq } else { !eq };
                set_reg(vm, frame_idx, dst, Value::Bool(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::CmpI32Lt | Opcode::CmpI32Le => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_i32(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_i32(get_reg(vm, frame_idx, rhs)?)?;
                let out = if opcode == Opcode::CmpI32Lt {
                    l < r
                } else {
                    l <= r
                };
                set_reg(vm, frame_idx, dst, Value::Bool(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AddF64 | Opcode::SubF64 | Opcode::MulF64 | Opcode::DivF64 => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_f64(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_f64(get_reg(vm, frame_idx, rhs)?)?;
                let out = match opcode {
                    Opcode::AddF64 => l + r,
                    Opcode::SubF64 => l - r,
                    Opcode::MulF64 => l * r,
                    Opcode::DivF64 => l / r,
                    _ => unreachable!(),
                };
                set_reg(vm, frame_idx, dst, Value::F64(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx => {
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let lhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let rhs = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let l = as_fx(get_reg(vm, frame_idx, lhs)?)?;
                let r = as_fx(get_reg(vm, frame_idx, rhs)?)?;
                let out = match opcode {
                    Opcode::AddFx => fx_add_raw(l, r)?,
                    Opcode::SubFx => fx_sub_raw(l, r)?,
                    Opcode::MulFx => fx_mul_raw(l, r)?,
                    Opcode::DivFx => fx_div_raw(l, r)?,
                    _ => unreachable!(),
                };
                set_reg(vm, frame_idx, dst, Value::Fx(out))?;
                next_pc = cur - f.instr_start;
            }
            Opcode::Jmp => {
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                if addr >= instr_rel_len {
                    return Err(RuntimeError::InvalidJumpAddress {
                        func: func_name,
                        addr,
                    });
                }
                next_pc = addr;
            }
            Opcode::JmpIf => {
                let cond = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let addr = read_u32_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let b = as_bool(get_reg(vm, frame_idx, cond)?)?;
                if b {
                    if addr >= instr_rel_len {
                        return Err(RuntimeError::InvalidJumpAddress {
                            func: func_name,
                            addr,
                        });
                    }
                    next_pc = addr;
                } else {
                    next_pc = cur - f.instr_start;
                }
            }
            Opcode::Call => {
                let has_dst = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let callee_sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let argc = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
                let callee = lookup_str(&f, callee_sid)?.to_string();
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    args.push(get_reg(vm, frame_idx, r)?);
                }
                if let Some(result) = try_eval_builtin_call(&callee, &args)? {
                    if has_dst {
                        set_reg(vm, frame_idx, dst, result)?;
                    }
                    next_pc = cur - f.instr_start;
                } else {
                    vm.callstack[frame_idx].pc = cur - f.instr_start;
                    push_frame(vm, &callee, args, if has_dst { Some(dst) } else { None })?;
                    continue;
                }
            }
            Opcode::Assert => {
                let cond = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                match get_reg(vm, frame_idx, cond)? {
                    Value::Bool(true) => {
                        next_pc = cur - f.instr_start;
                    }
                    Value::Bool(false) => {
                        return Err(RuntimeError::Trap(RuntimeTrap::AssertionFailed));
                    }
                    other => {
                        return Err(RuntimeError::TypeMismatchRuntime(format!(
                            "ASSERT requires bool register, got {:?}",
                            other
                        )));
                    }
                }
            }
            Opcode::GateRead => {
                bump_effect_calls(vm)?;
                let dst = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let device_id = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let value = host.gate_read(device_id, port)?;
                set_reg(vm, frame_idx, dst, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::GateWrite => {
                bump_effect_calls(vm)?;
                let device_id = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let value = get_reg(vm, frame_idx, src)?;
                host.gate_write(device_id, port, value)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::PulseEmit => {
                bump_effect_calls(vm)?;
                let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                let signal = lookup_str(&f, sid)?;
                host.pulse_emit(signal)?;
                next_pc = cur - f.instr_start;
            }
            Opcode::Ret => {
                let has_src = read_u8(&f.code, &mut cur).map_err(map_format_err)? != 0;
                let ret_val = if has_src {
                    let src = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                    get_reg(vm, frame_idx, src)?
                } else {
                    Value::Unit
                };
                let finished = vm.callstack.pop().ok_or(RuntimeError::StackUnderflow)?;
                if let Some(caller) = vm.callstack.last_mut() {
                    if let Some(dst) = finished.return_dst {
                        write_reg(caller, dst as usize, ret_val, &vm.config.quotas)?;
                    }
                } else {
                    return Ok(());
                }
                continue;
            }
        }

        vm.callstack[frame_idx].pc = next_pc;
    }
}

fn value_to_abi(value: Value) -> Result<AbiValue, RuntimeError> {
    match value {
        Value::Quad(q) => Ok(AbiValue::Quad(quad_to_u8(q))),
        Value::Bool(v) => Ok(AbiValue::Bool(v)),
        Value::I32(v) => Ok(AbiValue::I32(v)),
        Value::F64(v) => Ok(AbiValue::F64(v)),
        Value::U32(v) => Ok(AbiValue::U32(v)),
        Value::Fx(v) => Ok(AbiValue::Fx(v)),
        Value::Tuple(_) => Err(RuntimeError::TypeMismatchRuntime(
            "tuple values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Record(_) => Err(RuntimeError::TypeMismatchRuntime(
            "record values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Adt(_) => Err(RuntimeError::TypeMismatchRuntime(
            "enum values are not part of the PROMETHEUS host ABI surface".to_string(),
        )),
        Value::Unit => Ok(AbiValue::Unit),
    }
}

fn value_from_abi(value: AbiValue) -> Value {
    match value {
        AbiValue::Quad(q) => Value::Quad(u8_to_quad(q)),
        AbiValue::Bool(v) => Value::Bool(v),
        AbiValue::I32(v) => Value::I32(v),
        AbiValue::F64(v) => Value::F64(v),
        AbiValue::U32(v) => Value::U32(v),
        AbiValue::Fx(v) => Value::Fx(v),
        AbiValue::Unit => Value::Unit,
    }
}

fn push_frame(
    vm: &mut VM,
    func_name: &str,
    args: Vec<Value>,
    return_dst: Option<u16>,
) -> Result<(), RuntimeError> {
    let f = vm
        .functions
        .get(func_name)
        .ok_or_else(|| RuntimeError::UnknownFunction(func_name.to_string()))?;
    let next_depth = vm.callstack.len() + 1;
    enforce_quota(&vm.config.quotas, QuotaKind::Frames, next_depth)?;
    match enforce_quota(&vm.config.quotas, QuotaKind::StackDepth, next_depth) {
        Err(RuntimeError::QuotaExceeded(_)) => return Err(RuntimeError::StackOverflow),
        other => other?,
    }
    let initial_reg_count = 16usize.max(args.len());
    enforce_quota(&vm.config.quotas, QuotaKind::Registers, initial_reg_count)?;
    let mut regs = vec![Value::Unit; initial_reg_count];
    for (i, v) in args.into_iter().enumerate() {
        regs[i] = v;
    }
    let frame = Frame {
        pc: 0,
        regs,
        locals: HashMap::new(),
        func: f.name.clone(),
        return_dst,
    };
    vm.callstack.push(frame);
    Ok(())
}

fn lookup_str<'a>(f: &'a FunctionBytecode, sid: u16) -> Result<&'a str, RuntimeError> {
    f.strings
        .get(sid as usize)
        .map(|s| s.as_str())
        .ok_or(RuntimeError::InvalidStringId(sid))
}

fn lookup_symbol(f: &FunctionBytecode, sid: u16) -> Result<SymbolId, RuntimeError> {
    f.symbol_ids
        .get(sid as usize)
        .copied()
        .ok_or(RuntimeError::InvalidStringId(sid))
}

fn get_reg(vm: &VM, frame_idx: usize, r: u16) -> Result<Value, RuntimeError> {
    vm.callstack
        .get(frame_idx)
        .and_then(|fr| fr.regs.get(r as usize))
        .cloned()
        .ok_or_else(|| RuntimeError::BadFormat(format!("read invalid reg r{}", r)))
}

fn set_reg(vm: &mut VM, frame_idx: usize, r: u16, v: Value) -> Result<(), RuntimeError> {
    if let Some(frame) = vm.callstack.get_mut(frame_idx) {
        write_reg(frame, r as usize, v, &vm.config.quotas)?;
    }
    Ok(())
}

fn write_reg(
    frame: &mut Frame,
    r: usize,
    v: Value,
    quotas: &RuntimeQuotas,
) -> Result<(), RuntimeError> {
    if frame.regs.len() <= r {
        let required = r + 1;
        enforce_quota(quotas, QuotaKind::Registers, required)?;
        frame.regs.resize(required, Value::Unit);
    }
    frame.regs[r] = v;
    Ok(())
}

fn enforce_quota(quotas: &RuntimeQuotas, kind: QuotaKind, used: usize) -> Result<(), RuntimeError> {
    if let Some(exceeded) = quotas.exceed(kind, used) {
        return Err(RuntimeError::QuotaExceeded(exceeded));
    }
    Ok(())
}

fn bump_effect_calls(vm: &mut VM) -> Result<(), RuntimeError> {
    let next = vm.effect_calls + 1;
    enforce_quota(&vm.config.quotas, QuotaKind::EffectCalls, next)?;
    vm.effect_calls = next;
    Ok(())
}

fn as_quad(v: Value) -> Result<QuadVal, RuntimeError> {
    if let Value::Quad(q) = v {
        Ok(q)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected quad".to_string(),
        ))
    }
}

fn as_bool(v: Value) -> Result<bool, RuntimeError> {
    if let Value::Bool(b) = v {
        Ok(b)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected bool".to_string(),
        ))
    }
}

fn as_i32(v: Value) -> Result<i32, RuntimeError> {
    if let Value::I32(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected i32".to_string(),
        ))
    }
}

fn as_f64(v: Value) -> Result<f64, RuntimeError> {
    if let Value::F64(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime(
            "expected f64".to_string(),
        ))
    }
}

fn as_fx(v: Value) -> Result<i32, RuntimeError> {
    if let Value::Fx(x) = v {
        Ok(x)
    } else {
        Err(RuntimeError::TypeMismatchRuntime("expected fx".to_string()))
    }
}

fn fx_add_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    i32::try_from(i64::from(lhs) + i64::from(rhs))
        .map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn fx_sub_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    i32::try_from(i64::from(lhs) - i64::from(rhs))
        .map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn fx_mul_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    let scaled = (i64::from(lhs) * i64::from(rhs)) / 1_000;
    i32::try_from(scaled).map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn fx_div_raw(lhs: i32, rhs: i32) -> Result<i32, RuntimeError> {
    if rhs == 0 {
        return Err(RuntimeError::Trap(RuntimeTrap::DivisionByZero));
    }
    let scaled = (i64::from(lhs) * 1_000) / i64::from(rhs);
    i32::try_from(scaled).map_err(|_| RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow))
}

fn quad_to_u8(q: QuadVal) -> u8 {
    match q {
        QuadVal::N => 0,
        QuadVal::F => 1,
        QuadVal::T => 2,
        QuadVal::S => 3,
    }
}

fn u8_to_quad(v: u8) -> QuadVal {
    match v & 0b11 {
        0 => QuadVal::N,
        1 => QuadVal::F,
        2 => QuadVal::T,
        _ => QuadVal::S,
    }
}

fn quad_and(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad(quad_to_u8(a) & quad_to_u8(b))
}

fn quad_or(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad(quad_to_u8(a) | quad_to_u8(b))
}

fn quad_not(a: QuadVal) -> QuadVal {
    let v = quad_to_u8(a);
    let r = ((v & 0b10) >> 1) | ((v & 0b01) << 1);
    u8_to_quad(r)
}

fn value_eq(a: &Value, b: &Value) -> Result<bool, RuntimeError> {
    match (a, b) {
        (Value::Quad(x), Value::Quad(y)) => Ok(x == y),
        (Value::Bool(x), Value::Bool(y)) => Ok(x == y),
        (Value::I32(x), Value::I32(y)) => Ok(x == y),
        (Value::F64(x), Value::F64(y)) => Ok(x == y),
        (Value::U32(x), Value::U32(y)) => Ok(x == y),
        (Value::Fx(x), Value::Fx(y)) => Ok(x == y),
        (Value::Tuple(xs), Value::Tuple(ys)) => {
            if xs.len() != ys.len() {
                return Ok(false);
            }
            for (x, y) in xs.iter().zip(ys.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Record(xs), Value::Record(ys)) => {
            if xs.type_name != ys.type_name {
                return Ok(false);
            }
            if xs.slots.len() != ys.slots.len() {
                return Ok(false);
            }
            for (x, y) in xs.slots.iter().zip(ys.slots.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Adt(xs), Value::Adt(ys)) => {
            if xs.type_name != ys.type_name
                || xs.variant_name != ys.variant_name
                || xs.tag != ys.tag
                || xs.payload.len() != ys.payload.len()
            {
                return Ok(false);
            }
            for (x, y) in xs.payload.iter().zip(ys.payload.iter()) {
                if !value_eq(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Unit, Value::Unit) => Ok(true),
        _ => Err(RuntimeError::TypeMismatchRuntime(
            "CmpEq/CmpNe operands must have same runtime type".to_string(),
        )),
    }
}

fn try_eval_builtin_call(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let value = match name {
        "sin" => Value::F64(expect_builtin_unary_f64(name, args)?.sin()),
        "cos" => Value::F64(expect_builtin_unary_f64(name, args)?.cos()),
        "tan" => Value::F64(expect_builtin_unary_f64(name, args)?.tan()),
        "sqrt" => Value::F64(expect_builtin_unary_f64(name, args)?.sqrt()),
        "abs" => Value::F64(expect_builtin_unary_f64(name, args)?.abs()),
        "pow" => {
            let (lhs, rhs) = expect_builtin_binary_f64(name, args)?;
            Value::F64(lhs.powf(rhs))
        }
        _ => return Ok(None),
    };
    Ok(Some(value))
}

fn expect_builtin_unary_f64(name: &str, args: &[Value]) -> Result<f64, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeMismatchRuntime(format!(
            "builtin '{name}' expects 1 f64 argument, got {}",
            args.len()
        )));
    }
    as_f64(args[0].clone())
}

fn expect_builtin_binary_f64(name: &str, args: &[Value]) -> Result<(f64, f64), RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeMismatchRuntime(format!(
            "builtin '{name}' expects 2 f64 arguments, got {}",
            args.len()
        )));
    }
    Ok((as_f64(args[0].clone())?, as_f64(args[1].clone())?))
}

fn disasm_one(f: &FunctionBytecode, pc: usize) -> Result<(String, usize), RuntimeError> {
    let mut cur = f.instr_start + pc;
    let opcode = Opcode::from_byte(read_u8(&f.code, &mut cur).map_err(map_format_err)?)
        .map_err(map_format_err)?;
    let text = match opcode {
        Opcode::LoadQ => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let q = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_Q r{}, {}", d, q)
        }
        Opcode::LoadBool => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let b = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_BOOL r{}, {}", d, b)
        }
        Opcode::LoadI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_I32 r{}, {}", d, n)
        }
        Opcode::AddI32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("ADD_I32 r{}, r{}, r{}", d, l, r)
        }
        Opcode::LoadU32 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_U32 r{}, {}", d, n)
        }
        Opcode::LoadF64 => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_f64_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_F64 r{}, {}", d, n)
        }
        Opcode::LoadFx => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_i32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_FX r{}, raw:{}", d, n)
        }
        Opcode::AddFx | Opcode::SubFx | Opcode::MulFx | Opcode::DivFx => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let op = match opcode {
                Opcode::AddFx => "ADD_FX",
                Opcode::SubFx => "SUB_FX",
                Opcode::MulFx => "MUL_FX",
                Opcode::DivFx => "DIV_FX",
                _ => unreachable!(),
            };
            format!("{} r{}, r{}, r{}", op, d, l, r)
        }
        Opcode::MakeTuple => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("MAKE_TUPLE r{}, [{}]", d, regs)
        }
        Opcode::MakeRecord => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            let name = lookup_str(f, sid)?;
            format!("MAKE_RECORD r{}, {}, [{}]", d, name, regs)
        }
        Opcode::MakeAdt => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let variant_sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let tag = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let count = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            let mut regs = Vec::with_capacity(count);
            for _ in 0..count {
                regs.push(read_u16_le(&f.code, &mut cur).map_err(map_format_err)?);
            }
            let regs = regs
                .iter()
                .map(|reg| format!("r{}", reg))
                .collect::<Vec<_>>()
                .join(", ");
            let name = lookup_str(f, sid)?;
            let variant = lookup_str(f, variant_sid)?;
            format!(
                "MAKE_ADT r{}, {}::{}, tag={}, [{}]",
                d, name, variant, tag, regs
            )
        }
        Opcode::AdtTag => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let name = lookup_str(f, sid)?;
            format!("ADT_TAG r{}, r{}, {}", d, s, name)
        }
        Opcode::AdtGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let name = lookup_str(f, sid)?;
            format!("ADT_GET r{}, r{}, {}, {}", d, s, name, i)
        }
        Opcode::RecordGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let name = lookup_str(f, sid)?;
            format!("RECORD_GET r{}, r{}, {}, {}", d, s, name, i)
        }
        Opcode::TupleGet => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let i = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("TUPLE_GET r{}, r{}, {}", d, s, i)
        }
        Opcode::LoadVar => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("LOAD_VAR r{}, s{}", d, s)
        }
        Opcode::StoreVar => {
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("STORE_VAR s{}, r{}", s, r)
        }
        Opcode::QAnd
        | Opcode::QOr
        | Opcode::QImpl
        | Opcode::BoolAnd
        | Opcode::BoolOr
        | Opcode::CmpI32Lt
        | Opcode::CmpI32Le
        | Opcode::AddF64
        | Opcode::SubF64
        | Opcode::MulF64
        | Opcode::DivF64
        | Opcode::CmpEq
        | Opcode::CmpNe => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let l = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let op = match opcode {
                Opcode::QAnd => "Q_AND",
                Opcode::QOr => "Q_OR",
                Opcode::QImpl => "Q_IMPL",
                Opcode::BoolAnd => "BOOL_AND",
                Opcode::BoolOr => "BOOL_OR",
                Opcode::CmpI32Lt => "CMP_I32_LT",
                Opcode::CmpI32Le => "CMP_I32_LE",
                Opcode::AddF64 => "ADD_F64",
                Opcode::SubF64 => "SUB_F64",
                Opcode::MulF64 => "MUL_F64",
                Opcode::DivF64 => "DIV_F64",
                Opcode::AddFx => "ADD_FX",
                Opcode::SubFx => "SUB_FX",
                Opcode::MulFx => "MUL_FX",
                Opcode::DivFx => "DIV_FX",
                Opcode::CmpEq => "CMP_EQ",
                _ => "CMP_NE",
            };
            format!("{} r{}, r{}, r{}", op, d, l, r)
        }
        Opcode::QNot | Opcode::BoolNot => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let op = if opcode == Opcode::QNot {
                "Q_NOT"
            } else {
                "BOOL_NOT"
            };
            format!("{} r{}, r{}", op, d, s)
        }
        Opcode::Jmp => {
            let a = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("JMP {}", a)
        }
        Opcode::JmpIf => {
            let c = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let a = read_u32_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("JMP_IF r{}, {}", c, a)
        }
        Opcode::Call => {
            let has = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let n = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let argc = read_u16_le(&f.code, &mut cur).map_err(map_format_err)? as usize;
            for _ in 0..argc {
                let _ = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            }
            format!("CALL dst?{} r{} fn#{} argc={}", has, d, n, argc)
        }
        Opcode::Assert => {
            let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("ASSERT r{}", r)
        }
        Opcode::GateRead => {
            let d = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let dev = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("GATE_READ r{}, dev={}, port={}", d, dev, port)
        }
        Opcode::GateWrite => {
            let dev = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let port = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            let s = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("GATE_WRITE dev={}, port={}, r{}", dev, port, s)
        }
        Opcode::PulseEmit => {
            let sid = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
            format!("PULSE_EMIT s{}", sid)
        }
        Opcode::Ret => {
            let has = read_u8(&f.code, &mut cur).map_err(map_format_err)?;
            if has != 0 {
                let r = read_u16_le(&f.code, &mut cur).map_err(map_format_err)?;
                format!("RET r{}", r)
            } else {
                "RET".to_string()
            }
        }
    };
    Ok((text, cur - f.instr_start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_emit::compile_program_to_semcode;
    use sm_runtime_core::{
        ExecutionConfig, ExecutionContext, QuotaExceeded, QuotaKind, RuntimeTrap,
    };

    #[test]
    fn vm_runs_empty_main() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_assert_statement_when_condition_holds() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("assert(true) should pass");
    }

    #[test]
    fn vm_traps_on_failed_assert() {
        let src = r#"
            fn main() {
                assert(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("assert(false) should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_function_requires_clause_when_condition_holds() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext, expected: quad) -> quad
                requires(ctx.camera == expected)
                requires(ctx.quality == 0.75) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx, T);
                assert(seen == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("requires clause should pass");
    }

    #[test]
    fn vm_traps_on_failed_function_requires_clause() {
        let src = r#"
            fn must_be_true(flag: bool) -> bool requires(flag == true) {
                return flag;
            }

            fn main() {
                let seen: bool = must_be_true(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("requires clause should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_function_ensures_clause_when_condition_holds() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext) -> quad
                ensures(result == ctx.camera)
                ensures(ctx.quality == 0.75) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx);
                assert(seen == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("ensures clause should pass");
    }

    #[test]
    fn vm_traps_on_failed_function_ensures_clause() {
        let src = r#"
            fn must_return_true(flag: bool) -> bool ensures(result == true) {
                return flag;
            }

            fn main() {
                let seen: bool = must_return_true(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("ensures clause should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_function_invariant_clauses_when_conditions_hold() {
        let src = r#"
            fn keep(flag: bool) -> bool
                invariant(flag == true)
                invariant(result == flag) {
                return flag;
            }

            fn main() {
                let seen: bool = keep(true);
                assert(seen == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("invariant clauses should pass");
    }

    #[test]
    fn vm_traps_on_failed_function_invariant_clause() {
        let src = r#"
            fn must_stay_true(flag: bool) -> bool invariant(result == true) {
                return flag;
            }

            fn main() {
                let seen: bool = must_stay_true(false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("invariant clause should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::AssertionFailed)
        ));
    }

    #[test]
    fn vm_runs_bool_ops() {
        let src = r#"
			fn main() {
				let a: bool = true;
				let b: bool = false;
				let c = a && b;
				if c == false { return; } else { return; }
			}
		"#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_quad_ops() {
        let src = r#"
			fn main() {
				let a: quad = T;
				let b: quad = S;
				let c = a && b;
				if c == T { return; } else { return; }
			}
		"#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_call_ret() {
        let src = r#"
			fn one() -> i32 { return 1; }
			fn main() { let x: i32 = one(); return; }
		"#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_fx_literal_call_and_compare_path() {
        let src = r#"
            fn id(x: fx) -> fx { return x; }

            fn make() -> fx {
                return -1.25;
            }

            fn main() {
                let x: fx = 1.25;
                let y: fx = id(2);
                let z: fx = make();
                let a = x == x;
                let b = y != z;
                if a == b { return; } else { return; }
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("LOAD_FX"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_plain_fx_arithmetic_path() {
        let src = r#"
            fn main() {
                let a: fx = 2.5;
                let b: fx = 1.5;
                let sum: fx = a + b;
                let diff: fx = a - b;
                let prod: fx = a * b;
                let quo: fx = a / b;
                let neg: fx = -a;
                let expected_sum: fx = 4.0;
                let expected_diff: fx = 1.0;
                let expected_prod: fx = 3.75;
                let expected_quo: fx = 1.666;
                let expected_neg: fx = -2.5;
                assert(sum == expected_sum);
                assert(diff == expected_diff);
                assert(prod == expected_prod);
                assert(quo == expected_quo);
                assert(neg == expected_neg);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADD_FX"));
        assert!(disasm.contains("SUB_FX"));
        assert!(disasm.contains("MUL_FX"));
        assert!(disasm.contains("DIV_FX"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_traps_on_fx_division_by_zero() {
        let src = r#"
            fn main() {
                let a: fx = 1.0;
                let b: fx = 0.0;
                let bad: fx = a / b;
                assert(bad == a);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let err = run_semcode(&bytes).expect_err("fx division by zero should trap");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::DivisionByZero)
        ));
    }

    #[test]
    fn vm_runs_u32_literal_compare_path() {
        let src = r#"
            fn main() {
                let left: u32 = 1_000u32;
                let right: u32 = 0x3e8u32;
                assert(left == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("LOAD_U32"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_tuple_return_and_equality_path() {
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
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_TUPLE"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_tuple_destructuring_bind_path() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let (count, ready): (i32, bool) = pair(true);
                assert(count == 1);
                assert(ready == true);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("TUPLE_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_stage1_record_literal_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let shadow: DecisionContext = ctx;
                let _ = shadow;
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_RECORD"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_stage1_enum_constructor_path() {
        let src = r#"
            enum Maybe {
                None,
                Some(bool),
            }

            fn choose(flag: bool) -> Maybe {
                return Maybe::Some(flag);
            }

            fn main() {
                let left: Maybe = choose(true);
                let right: Maybe = Maybe::None;
                let _ = left;
                let _ = right;
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_ADT"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_option_and_result_standard_form_paths() {
        let src = r#"
            fn keep(flag: bool) -> Option(bool) {
                let fallback: Option(bool) = Option::None;
                let _ = fallback;
                return Option::Some(flag);
            }

            fn settle(flag: bool) -> Result(bool, quad) {
                if flag {
                    let value: Result(bool, quad) = Result::Ok(true);
                    return value;
                }
                let value: Result(bool, quad) = Result::Err(N);
                return value;
            }

            fn main() {
                let left: Option(bool) = keep(true);
                let right: Result(bool, quad) = settle(false);
                let _ = left;
                let _ = right;
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("MAKE_ADT"));
        run_semcode(&bytes).expect("Option/Result standard-form paths should run");
    }

    #[test]
    fn vm_runs_option_and_result_match_ergonomics_paths() {
        let src = r#"
            fn unwrap(opt: Option(bool)) -> bool {
                let out: bool = match opt {
                    Option::Some(value) => { value }
                    Option::None => { false }
                };
                return out;
            }

            fn settle(res: Result(quad, quad)) -> quad {
                let out: quad = match res {
                    Result::Ok(value) => { value }
                    Result::Err(code) => { code }
                };
                return out;
            }

            fn main() {
                let left: bool = unwrap(Option::Some(true));
                let right: bool = unwrap(Option::None);
                let code: quad = settle(Result::Err(S));
                assert(left == true);
                assert(right == false);
                assert(code == S);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ADT_GET"));
        run_semcode(&bytes).expect("Option/Result match ergonomics paths should run");
    }

    #[test]
    fn vm_runs_stage1_adt_match_path() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn unwrap(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::Some(inner) => { inner }
                    _ => { 0.0 }
                };
                return total;
            }

            fn main() {
                let total: f64 = unwrap(Maybe::Some(2.5));
                assert(total == 2.5);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ADT_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_exhaustive_adt_match_without_default_path() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn unwrap(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::None => { 0.0 }
                    Maybe::Some(inner) => { inner }
                };
                return total;
            }

            fn main() {
                let total: f64 = unwrap(Maybe::Some(2.5));
                assert(total == 2.5);
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("ADT_TAG"));
        assert!(disasm.contains("ASSERT"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_stage1_record_field_access_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let seen: quad = ctx.camera;
                assert(seen == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("RECORD_GET"));
        run_semcode(&bytes).expect("run");
    }

    #[test]
    fn vm_runs_record_pass_return_and_safe_equality_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn echo(ctx: DecisionContext) -> DecisionContext {
                return ctx;
            }

            fn main() {
                let left: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let right: DecisionContext = echo(left);
                assert(right == right);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("record pass/return/equality should run");
    }

    #[test]
    fn vm_runs_record_access_policy_scenario() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                badge: quad,
                override_state: quad,
                tamper: quad,
                quality: f64,
            }

            fn allow(ctx: DecisionContext) -> quad {
                if ctx.tamper == T || ctx.tamper == S {
                    return S;
                }
                if ctx.override_state == T {
                    return T;
                }
                if ctx.camera == T && ctx.badge == T {
                    return T;
                }
                return N;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext {
                    quality: 0.50,
                    tamper: F,
                    override_state: N,
                    badge: T,
                    camera: T,
                };
                let decision: quad = allow(ctx);
                assert(decision == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        run_semcode(&bytes).expect("record access-policy scenario should run");
    }

    #[test]
    fn vm_runs_record_destructuring_bind_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: seen_camera, quality: _ } =
                    DecisionContext { quality: 0.75, camera: T };
                assert(seen_camera == T);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        run_semcode(&bytes).expect("record destructuring bind path should run");
    }

    #[test]
    fn vm_runs_record_let_else_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: T, quality: score } =
                    DecisionContext { quality: 0.75, camera: T } else return;
                assert(score == 0.75);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        run_semcode(&bytes).expect("record let-else path should run");
    }

    #[test]
    fn vm_runs_record_copy_with_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                assert(patched.quality == 1.0);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        assert!(disasm.contains("MAKE_RECORD"));
        run_semcode(&bytes).expect("record copy-with path should run");
    }

    #[test]
    fn vm_runs_record_stage2_ergonomics_scenario() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                override_state: quad,
                quality: f64,
            }

            fn main() {
                let camera: quad = T;
                let override_state: quad = N;
                let quality: f64 = 0.75;
                let ctx: DecisionContext = DecisionContext { camera, override_state, quality };
                let DecisionContext { camera, quality: _ } = ctx;
                let patched: DecisionContext = ctx with { camera };
                let DecisionContext { camera: T, override_state, quality } =
                    patched else return;
                assert(camera == T);
                assert(override_state == N);
                assert(quality == 0.75);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disassemble");
        assert!(disasm.contains("RECORD_GET"));
        assert!(disasm.contains("MAKE_RECORD"));
        run_semcode(&bytes).expect("record stage-2 ergonomics scenario should run");
    }

    #[test]
    fn vm_runs_for_range_inclusive_path() {
        let src = r#"
            fn main() {
                let saw_start: bool = false;
                let saw_end: bool = false;
                for i in 0..=2 {
                    if i == 0 {
                        saw_start ||= true;
                    } else {
                        saw_start ||= false;
                    }
                    if i == 2 {
                        saw_end ||= true;
                    } else {
                        saw_end ||= false;
                    }
                }
                assert(saw_start == true);
                assert(saw_end == true);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("CMP_I32_LE"));
        assert!(disasm.contains("ADD_I32"));
        run_semcode(&bytes).expect("inclusive for-range should run");
    }

    #[test]
    fn vm_runs_for_range_empty_half_open_path() {
        let src = r#"
            fn main() {
                let visited: bool = false;
                for i in 3..3 {
                    visited ||= true;
                }
                assert(visited == false);
                return;
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let disasm = disasm_semcode(&bytes).expect("disasm");
        assert!(disasm.contains("CMP_I32_LT"));
        run_semcode(&bytes).expect("empty half-open for-range should skip body");
    }

    #[test]
    fn vm_rejects_unknown_opcode_on_load() {
        let src = "fn main() { return; }";
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;
        bytes[opcode_pos] = 0xff;
        let err = run_semcode(&bytes).expect_err("must fail");
        assert!(matches!(err, RuntimeError::BadFormat(_)));
    }

    #[test]
    fn vm_rejects_unsupported_bytecode_version_with_hint() {
        let src = "fn main() { return; }";
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        bytes[7] = b'9';
        let err = run_semcode(&bytes).expect_err("must fail");
        match err {
            RuntimeError::UnsupportedBytecodeVersion { found, supported } => {
                assert!(found.starts_with("SEMCODE"));
                assert!(supported.contains("SEMCODE0"));
                assert!(supported.contains("SEMCODE1"));
                assert!(supported.contains("SEMCODE2"));
                assert!(supported.contains("SEMCODE3"));
            }
            other => panic!("expected UnsupportedBytecodeVersion, got {other:?}"),
        }
    }

    #[test]
    fn vm_enforces_configured_stack_depth() {
        let src = r#"
            fn helper() { return; }
            fn main() { helper(); return; }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let mut config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
        config.quotas.max_stack_depth = 1;
        let err = run_semcode_with_config(&bytes, config).expect_err("must fail");
        assert_eq!(err, RuntimeError::StackOverflow);
    }

    #[test]
    fn vm_enforces_configured_register_budget() {
        let src = r#"
            fn main() {
                let a: bool = true;
                let b: bool = false;
                let c = a && b;
                if c == false { return; } else { return; }
            }
        "#;
        let bytes = compile_program_to_semcode(src).expect("compile");
        let mut config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
        config.quotas.max_registers = 2;
        let err = run_semcode_with_config(&bytes, config).expect_err("must fail");
        assert_eq!(
            err,
            RuntimeError::QuotaExceeded(QuotaExceeded {
                kind: QuotaKind::Registers,
                limit: 2,
                used: 16,
            })
        );
    }

    #[test]
    fn verified_run_rejects_invalid_bytecode_before_execution() {
        let src = "fn main() { return; }";
        let mut bytes = compile_program_to_semcode(src).expect("compile");
        let opcode_pos = 8 + 2 + 4 + 4 + 2;
        bytes[opcode_pos] = 0xff;
        let err = run_verified_semcode(&bytes).expect_err("must fail");
        assert!(matches!(err, RuntimeError::VerifierRejected(_)));
    }
}
