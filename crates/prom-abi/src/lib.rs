#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::{String, ToString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostCallId {
    GateRead,
    GateWrite,
    PulseEmit,
    StateQuery,
    StateUpdate,
    EventPost,
    ClockRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectClass {
    HostQuery,
    HostWrite,
    EventEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismClass {
    Deterministic,
    HostBound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCallStability {
    StableV1,
    PlannedPostStable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostCallDescriptor {
    pub id: HostCallId,
    pub effect: EffectClass,
    pub determinism: DeterminismClass,
    pub returns_value: bool,
    pub stability: HostCallStability,
}

pub const fn descriptor_for_call(id: HostCallId) -> HostCallDescriptor {
    match id {
        HostCallId::GateRead => HostCallDescriptor {
            id,
            effect: EffectClass::HostQuery,
            determinism: DeterminismClass::HostBound,
            returns_value: true,
            stability: HostCallStability::StableV1,
        },
        HostCallId::GateWrite => HostCallDescriptor {
            id,
            effect: EffectClass::HostWrite,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::StableV1,
        },
        HostCallId::PulseEmit => HostCallDescriptor {
            id,
            effect: EffectClass::EventEmit,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::StableV1,
        },
        HostCallId::StateQuery => HostCallDescriptor {
            id,
            effect: EffectClass::HostQuery,
            determinism: DeterminismClass::HostBound,
            returns_value: true,
            stability: HostCallStability::PlannedPostStable,
        },
        HostCallId::StateUpdate => HostCallDescriptor {
            id,
            effect: EffectClass::HostWrite,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::PlannedPostStable,
        },
        HostCallId::EventPost => HostCallDescriptor {
            id,
            effect: EffectClass::EventEmit,
            determinism: DeterminismClass::HostBound,
            returns_value: false,
            stability: HostCallStability::PlannedPostStable,
        },
        HostCallId::ClockRead => HostCallDescriptor {
            id,
            effect: EffectClass::HostQuery,
            determinism: DeterminismClass::HostBound,
            returns_value: true,
            stability: HostCallStability::PlannedPostStable,
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbiValue {
    Quad(u8),
    Bool(bool),
    I32(i32),
    U32(u32),
    Fx(i32),
    F64(f64),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiFailureKind {
    Unavailable,
    InvalidInput,
    HostFault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiError {
    pub call: HostCallId,
    pub kind: AbiFailureKind,
    pub message: String,
}

impl AbiError {
    pub fn new(call: HostCallId, kind: AbiFailureKind, message: impl Into<String>) -> Self {
        Self {
            call,
            kind,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for AbiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ABI {:?} failure [{:?}]: {}", self.call, self.kind, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AbiError {}

pub trait PrometheusHostAbi {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<AbiValue, AbiError>;
    fn gate_write(&mut self, device_id: u16, port: u16, value: AbiValue) -> Result<(), AbiError>;
    fn pulse_emit(&mut self, signal: &str) -> Result<(), AbiError>;
}

#[derive(Debug, Default)]
pub struct RecordingHostAbi {
    pub reads: alloc::vec::Vec<(u16, u16)>,
    pub writes: alloc::vec::Vec<(u16, u16, AbiValue)>,
    pub pulses: alloc::vec::Vec<String>,
    pub next_read: AbiValue,
}

impl RecordingHostAbi {
    pub fn with_read_value(next_read: AbiValue) -> Self {
        Self {
            next_read,
            ..Self::default()
        }
    }
}

impl PrometheusHostAbi for RecordingHostAbi {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<AbiValue, AbiError> {
        self.reads.push((device_id, port));
        Ok(self.next_read.clone())
    }

    fn gate_write(&mut self, device_id: u16, port: u16, value: AbiValue) -> Result<(), AbiError> {
        self.writes.push((device_id, port, value));
        Ok(())
    }

    fn pulse_emit(&mut self, signal: &str) -> Result<(), AbiError> {
        self.pulses.push(signal.to_string());
        Ok(())
    }
}

impl Default for AbiValue {
    fn default() -> Self {
        Self::Unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_table_is_explicit() {
        assert!(descriptor_for_call(HostCallId::GateRead).returns_value);
        assert_eq!(
            descriptor_for_call(HostCallId::GateWrite).effect,
            EffectClass::HostWrite
        );
        assert_eq!(
            descriptor_for_call(HostCallId::PulseEmit).determinism,
            DeterminismClass::HostBound
        );
        assert_eq!(
            descriptor_for_call(HostCallId::StateQuery).stability,
            HostCallStability::PlannedPostStable
        );
        assert_eq!(
            descriptor_for_call(HostCallId::ClockRead).returns_value,
            true
        );
    }
}
