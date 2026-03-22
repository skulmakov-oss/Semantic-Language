pub const MAGIC0: [u8; 8] = *b"SEMCODE0";
pub const MAGIC1: [u8; 8] = *b"SEMCODE1";
pub const MAGIC2: [u8; 8] = *b"SEMCODE2";

pub const CAP_DEBUG_SYMBOLS: u32 = 1 << 0;
pub const CAP_F64_MATH: u32 = 1 << 1;
pub const CAP_GATE_SURFACE: u32 = 1 << 2;
pub const CAP_FX_VALUES: u32 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemcodeHeaderSpec {
    pub magic: [u8; 8],
    pub epoch: u16,
    pub rev: u16,
    pub capabilities: u32,
}

pub const HEADER_V0: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC0,
    epoch: 0,
    rev: 1,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_GATE_SURFACE,
};

pub const HEADER_V1: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC1,
    epoch: 0,
    rev: 2,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_F64_MATH | CAP_GATE_SURFACE,
};

pub const HEADER_V2: SemcodeHeaderSpec = SemcodeHeaderSpec {
    magic: MAGIC2,
    epoch: 0,
    rev: 3,
    capabilities: CAP_DEBUG_SYMBOLS | CAP_F64_MATH | CAP_GATE_SURFACE | CAP_FX_VALUES,
};

pub fn supported_headers() -> &'static [SemcodeHeaderSpec] {
    &[HEADER_V0, HEADER_V1, HEADER_V2]
}

pub fn header_spec_from_magic(magic: &[u8; 8]) -> Option<SemcodeHeaderSpec> {
    supported_headers().iter().copied().find(|h| &h.magic == magic)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    LoadQ = 0x01,
    LoadBool = 0x02,
    LoadI32 = 0x03,
    AddI32 = 0x07,
    LoadU32 = 0x06,
    LoadVar = 0x04,
    StoreVar = 0x05,
    QAnd = 0x10,
    QOr = 0x11,
    QNot = 0x12,
    QImpl = 0x13,
    BoolAnd = 0x14,
    BoolOr = 0x15,
    BoolNot = 0x16,
    CmpEq = 0x20,
    CmpNe = 0x21,
    CmpI32Lt = 0x22,
    CmpI32Le = 0x23,
    Jmp = 0x30,
    JmpIf = 0x31,
    Call = 0x40,
    Ret = 0x41,
    Assert = 0x42,
    MakeTuple = 0x43,
    TupleGet = 0x44,
    LoadF64 = 0x50,
    AddF64 = 0x51,
    SubF64 = 0x52,
    MulF64 = 0x53,
    DivF64 = 0x54,
    LoadFx = 0x55,
    GateRead = 0x60,
    GateWrite = 0x61,
    PulseEmit = 0x62,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemcodeFormatError {
    UnexpectedEof,
    InvalidUtf8,
    UnknownOpcode(u8),
}

impl core::fmt::Display for SemcodeFormatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SemcodeFormatError::UnexpectedEof => write!(f, "unexpected EOF"),
            SemcodeFormatError::InvalidUtf8 => write!(f, "invalid utf8"),
            SemcodeFormatError::UnknownOpcode(v) => write!(f, "unknown opcode 0x{:02x}", v),
        }
    }
}

impl std::error::Error for SemcodeFormatError {}

impl Opcode {
    pub fn byte(self) -> u8 {
        self as u8
    }

    pub fn from_byte(v: u8) -> Result<Self, SemcodeFormatError> {
        match v {
            x if x == Self::LoadQ as u8 => Ok(Self::LoadQ),
            x if x == Self::LoadBool as u8 => Ok(Self::LoadBool),
            x if x == Self::LoadI32 as u8 => Ok(Self::LoadI32),
            x if x == Self::AddI32 as u8 => Ok(Self::AddI32),
            x if x == Self::LoadU32 as u8 => Ok(Self::LoadU32),
            x if x == Self::LoadVar as u8 => Ok(Self::LoadVar),
            x if x == Self::StoreVar as u8 => Ok(Self::StoreVar),
            x if x == Self::QAnd as u8 => Ok(Self::QAnd),
            x if x == Self::QOr as u8 => Ok(Self::QOr),
            x if x == Self::QNot as u8 => Ok(Self::QNot),
            x if x == Self::QImpl as u8 => Ok(Self::QImpl),
            x if x == Self::BoolAnd as u8 => Ok(Self::BoolAnd),
            x if x == Self::BoolOr as u8 => Ok(Self::BoolOr),
            x if x == Self::BoolNot as u8 => Ok(Self::BoolNot),
            x if x == Self::CmpEq as u8 => Ok(Self::CmpEq),
            x if x == Self::CmpNe as u8 => Ok(Self::CmpNe),
            x if x == Self::CmpI32Lt as u8 => Ok(Self::CmpI32Lt),
            x if x == Self::CmpI32Le as u8 => Ok(Self::CmpI32Le),
            x if x == Self::Jmp as u8 => Ok(Self::Jmp),
            x if x == Self::JmpIf as u8 => Ok(Self::JmpIf),
            x if x == Self::Call as u8 => Ok(Self::Call),
            x if x == Self::Ret as u8 => Ok(Self::Ret),
            x if x == Self::Assert as u8 => Ok(Self::Assert),
            x if x == Self::MakeTuple as u8 => Ok(Self::MakeTuple),
            x if x == Self::TupleGet as u8 => Ok(Self::TupleGet),
            x if x == Self::LoadF64 as u8 => Ok(Self::LoadF64),
            x if x == Self::AddF64 as u8 => Ok(Self::AddF64),
            x if x == Self::SubF64 as u8 => Ok(Self::SubF64),
            x if x == Self::MulF64 as u8 => Ok(Self::MulF64),
            x if x == Self::DivF64 as u8 => Ok(Self::DivF64),
            x if x == Self::LoadFx as u8 => Ok(Self::LoadFx),
            x if x == Self::GateRead as u8 => Ok(Self::GateRead),
            x if x == Self::GateWrite as u8 => Ok(Self::GateWrite),
            x if x == Self::PulseEmit as u8 => Ok(Self::PulseEmit),
            _ => Err(SemcodeFormatError::UnknownOpcode(v)),
        }
    }
}

pub fn write_u16_le(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn write_i32_le(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn write_f64_le(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn read_u8(bytes: &[u8], i: &mut usize) -> Result<u8, SemcodeFormatError> {
    if *i >= bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let v = bytes[*i];
    *i += 1;
    Ok(v)
}

pub fn read_u16_le(bytes: &[u8], i: &mut usize) -> Result<u16, SemcodeFormatError> {
    if *i + 2 > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let v = u16::from_le_bytes([bytes[*i], bytes[*i + 1]]);
    *i += 2;
    Ok(v)
}

pub fn read_u32_le(bytes: &[u8], i: &mut usize) -> Result<u32, SemcodeFormatError> {
    if *i + 4 > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let v = u32::from_le_bytes([bytes[*i], bytes[*i + 1], bytes[*i + 2], bytes[*i + 3]]);
    *i += 4;
    Ok(v)
}

pub fn read_i32_le(bytes: &[u8], i: &mut usize) -> Result<i32, SemcodeFormatError> {
    Ok(read_u32_le(bytes, i)? as i32)
}

pub fn read_f64_le(bytes: &[u8], i: &mut usize) -> Result<f64, SemcodeFormatError> {
    if *i + 8 > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*i..*i + 8]);
    *i += 8;
    Ok(f64::from_le_bytes(raw))
}

pub fn read_utf8(bytes: &[u8], i: &mut usize, len: usize) -> Result<String, SemcodeFormatError> {
    if *i + len > bytes.len() {
        return Err(SemcodeFormatError::UnexpectedEof);
    }
    let s = std::str::from_utf8(&bytes[*i..*i + len])
        .map_err(|_| SemcodeFormatError::InvalidUtf8)?
        .to_string();
    *i += len;
    Ok(s)
}
