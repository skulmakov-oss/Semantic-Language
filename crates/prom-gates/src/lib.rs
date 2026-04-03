#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use prom_abi::{AbiError, AbiFailureKind, AbiValue, HostCallId, PrometheusHostAbi};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GateId {
    pub device_id: u16,
    pub port: u16,
}

impl GateId {
    pub const fn new(device_id: u16, port: u16) -> Self {
        Self { device_id, port }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateDescriptor {
    pub id: GateId,
    pub name: String,
    pub access: GateAccess,
}

impl GateDescriptor {
    pub fn read_only(device_id: u16, port: u16, name: impl Into<String>) -> Self {
        Self {
            id: GateId::new(device_id, port),
            name: name.into(),
            access: GateAccess::ReadOnly,
        }
    }

    pub fn read_write(device_id: u16, port: u16, name: impl Into<String>) -> Self {
        Self {
            id: GateId::new(device_id, port),
            name: name.into(),
            access: GateAccess::ReadWrite,
        }
    }

    pub const fn allows_read(&self) -> bool {
        true
    }

    pub const fn allows_write(&self) -> bool {
        matches!(self.access, GateAccess::ReadWrite)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateBindingError {
    pub gate: GateId,
    pub message: String,
}

impl GateBindingError {
    pub fn new(gate: GateId, message: impl Into<String>) -> Self {
        Self {
            gate,
            message: message.into(),
        }
    }

    pub fn to_abi_error(&self, call: HostCallId, kind: AbiFailureKind) -> AbiError {
        AbiError::new(call, kind, self.message.clone())
    }
}

impl core::fmt::Display for GateBindingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "gate {}:{} binding error: {}",
            self.gate.device_id, self.gate.port, self.message
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GateBindingError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateRegistry {
    descriptors: BTreeMap<GateId, GateDescriptor>,
}

impl GateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: GateDescriptor) -> Result<(), GateBindingError> {
        let gate = descriptor.id;
        if self.descriptors.contains_key(&gate) {
            return Err(GateBindingError::new(
                gate,
                "gate descriptor already registered",
            ));
        }
        self.descriptors.insert(gate, descriptor);
        Ok(())
    }

    pub fn descriptor(&self, gate: GateId) -> Option<&GateDescriptor> {
        self.descriptors.get(&gate)
    }

    pub fn validate_read(&self, gate: GateId) -> Result<&GateDescriptor, GateBindingError> {
        self.descriptor(gate)
            .ok_or_else(|| GateBindingError::new(gate, "gate is not registered"))
    }

    pub fn validate_write(&self, gate: GateId) -> Result<&GateDescriptor, GateBindingError> {
        let descriptor = self
            .descriptor(gate)
            .ok_or_else(|| GateBindingError::new(gate, "gate is not registered"))?;
        if !descriptor.allows_write() {
            return Err(GateBindingError::new(
                gate,
                "gate does not allow writes",
            ));
        }
        Ok(descriptor)
    }
}

pub trait GateBinding {
    fn gate_read(&mut self, descriptor: &GateDescriptor) -> Result<AbiValue, GateBindingError>;
    fn gate_write(
        &mut self,
        descriptor: &GateDescriptor,
        value: AbiValue,
    ) -> Result<(), GateBindingError>;
}

pub struct GateHostAdapter<'a, B: GateBinding> {
    registry: &'a GateRegistry,
    binding: &'a mut B,
}

impl<'a, B: GateBinding> GateHostAdapter<'a, B> {
    pub fn new(registry: &'a GateRegistry, binding: &'a mut B) -> Self {
        Self { registry, binding }
    }
}

impl<'a, B: GateBinding> PrometheusHostAbi for GateHostAdapter<'a, B> {
    fn gate_read(&mut self, device_id: u16, port: u16) -> Result<AbiValue, AbiError> {
        let gate = GateId::new(device_id, port);
        let descriptor = self
            .registry
            .validate_read(gate)
            .map_err(|err| err.to_abi_error(HostCallId::GateRead, AbiFailureKind::Unavailable))?;
        self.binding
            .gate_read(descriptor)
            .map_err(|err| err.to_abi_error(HostCallId::GateRead, AbiFailureKind::HostFault))
    }

    fn gate_write(
        &mut self,
        device_id: u16,
        port: u16,
        value: AbiValue,
    ) -> Result<(), AbiError> {
        let gate = GateId::new(device_id, port);
        let descriptor = self.registry.validate_write(gate).map_err(|err| {
            err.to_abi_error(HostCallId::GateWrite, AbiFailureKind::InvalidInput)
        })?;
        self.binding
            .gate_write(descriptor, value)
            .map_err(|err| err.to_abi_error(HostCallId::GateWrite, AbiFailureKind::HostFault))
    }

    fn pulse_emit(&mut self, signal: &str) -> Result<(), AbiError> {
        Err(AbiError::new(
            HostCallId::PulseEmit,
            AbiFailureKind::Unavailable,
            format!("pulse emission '{}' is not bound by gate adapter", signal),
        ))
    }

    fn state_query(&mut self, key: &str) -> Result<AbiValue, AbiError> {
        Err(AbiError::new(
            HostCallId::StateQuery,
            AbiFailureKind::Unavailable,
            format!("state query '{}' is not bound by gate adapter", key),
        ))
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeterministicGateMock {
    reads: BTreeMap<GateId, AbiValue>,
    writes: Vec<(GateId, AbiValue)>,
}

impl DeterministicGateMock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_read(&mut self, gate: GateId, value: AbiValue) {
        self.reads.insert(gate, value);
    }

    pub fn writes(&self) -> &[(GateId, AbiValue)] {
        &self.writes
    }
}

impl GateBinding for DeterministicGateMock {
    fn gate_read(&mut self, descriptor: &GateDescriptor) -> Result<AbiValue, GateBindingError> {
        self.reads
            .get(&descriptor.id)
            .cloned()
            .ok_or_else(|| GateBindingError::new(descriptor.id, "no deterministic read value"))
    }

    fn gate_write(
        &mut self,
        descriptor: &GateDescriptor,
        value: AbiValue,
    ) -> Result<(), GateBindingError> {
        self.writes.push((descriptor.id, value));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_rejects_duplicate_descriptor() {
        let mut registry = GateRegistry::new();
        registry
            .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
            .expect("register");
        let err = registry
            .register(GateDescriptor::read_write(7, 3, "sensor.alpha.shadow"))
            .expect_err("must reject duplicate");
        assert!(err.message.contains("already registered"));
    }

    #[test]
    fn registry_write_validation_respects_access() {
        let mut registry = GateRegistry::new();
        registry
            .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
            .expect("register");
        let err = registry
            .validate_write(GateId::new(7, 3))
            .expect_err("read-only gate must reject writes");
        assert!(err.message.contains("does not allow writes"));
    }

    #[test]
    fn adapter_routes_gate_calls_through_registry_and_mock() {
        let mut registry = GateRegistry::new();
        registry
            .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
            .expect("register read");
        registry
            .register(GateDescriptor::read_write(7, 4, "actuator.beta"))
            .expect("register write");

        let mut binding = DeterministicGateMock::new();
        binding.seed_read(GateId::new(7, 3), AbiValue::I32(41));
        let mut adapter = GateHostAdapter::new(&registry, &mut binding);

        let value = adapter.gate_read(7, 3).expect("read");
        assert_eq!(value, AbiValue::I32(41));
        adapter
            .gate_write(7, 4, AbiValue::I32(41))
            .expect("write");
        assert_eq!(
            binding.writes(),
            &[(GateId::new(7, 4), AbiValue::I32(41))]
        );
    }

    #[test]
    fn adapter_blocks_write_to_read_only_gate_before_backend() {
        let mut registry = GateRegistry::new();
        registry
            .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
            .expect("register");

        let mut binding = DeterministicGateMock::new();
        let mut adapter = GateHostAdapter::new(&registry, &mut binding);
        let err = adapter
            .gate_write(7, 3, AbiValue::I32(1))
            .expect_err("must reject");
        assert_eq!(err.call, HostCallId::GateWrite);
        assert_eq!(err.kind, AbiFailureKind::InvalidInput);
        assert!(binding.writes().is_empty());
    }
}
