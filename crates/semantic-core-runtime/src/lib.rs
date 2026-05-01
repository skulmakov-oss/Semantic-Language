#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub u32);

impl SymbolId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FunctionId(pub u16);

impl FunctionId {
    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CoreTrap {
    InvalidPc = 0,
    InvalidRegister = 1,
    TypeMismatch = 2,
    DivisionByZero = 3,
    IntegerOverflow = 4,
    FuelExceeded = 5,
    CallDepthExceeded = 6,
    InvalidFunction = 7,
    AssertFailed = 8,
    ExplicitTrap = 9,
}

impl CoreTrap {
    pub const fn code(self) -> u8 {
        self as u8
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelMeter {
    remaining: u64,
}

impl FuelMeter {
    pub const fn new(remaining: u64) -> Self {
        Self { remaining }
    }

    pub fn consume(&mut self, amount: u64) -> Result<(), CoreTrap> {
        if amount > self.remaining {
            self.remaining = 0;
            return Err(CoreTrap::FuelExceeded);
        }
        self.remaining -= amount;
        Ok(())
    }

    pub const fn remaining(self) -> u64 {
        self.remaining
    }

    pub const fn is_exhausted(self) -> bool {
        self.remaining == 0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreAdmissionProfile {
    pub max_registers: u16,
    pub max_functions: u16,
    pub max_call_depth: u16,
    pub max_instrs_per_function: u32,
    pub max_fuel: u64,
}

impl CoreAdmissionProfile {
    pub const MAX_REGISTERS_BOUND: u16 = 16_384;
    pub const MAX_FUNCTIONS_BOUND: u16 = 4_096;
    pub const MAX_CALL_DEPTH_BOUND: u16 = 1_024;
    pub const MAX_INSTRS_BOUND: u32 = 1_000_000;
    pub const MAX_FUEL_BOUND: u64 = 1_000_000_000;

    pub const fn safe() -> Self {
        Self {
            max_registers: 256,
            max_functions: 256,
            max_call_depth: 64,
            max_instrs_per_function: 8_192,
            max_fuel: 1_000_000,
        }
    }

    pub fn validate(self) -> Result<(), CoreProfileError> {
        if self.max_registers == 0 {
            return Err(CoreProfileError::ZeroLimit("max_registers"));
        }
        if self.max_functions == 0 {
            return Err(CoreProfileError::ZeroLimit("max_functions"));
        }
        if self.max_call_depth == 0 {
            return Err(CoreProfileError::ZeroLimit("max_call_depth"));
        }
        if self.max_instrs_per_function == 0 {
            return Err(CoreProfileError::ZeroLimit("max_instrs_per_function"));
        }
        if self.max_fuel == 0 {
            return Err(CoreProfileError::ZeroLimit("max_fuel"));
        }
        if self.max_registers > Self::MAX_REGISTERS_BOUND {
            return Err(CoreProfileError::UnboundedLimit("max_registers"));
        }
        if self.max_functions > Self::MAX_FUNCTIONS_BOUND {
            return Err(CoreProfileError::UnboundedLimit("max_functions"));
        }
        if self.max_call_depth > Self::MAX_CALL_DEPTH_BOUND {
            return Err(CoreProfileError::UnboundedLimit("max_call_depth"));
        }
        if self.max_instrs_per_function > Self::MAX_INSTRS_BOUND {
            return Err(CoreProfileError::UnboundedLimit("max_instrs_per_function"));
        }
        if self.max_fuel > Self::MAX_FUEL_BOUND {
            return Err(CoreProfileError::UnboundedLimit("max_fuel"));
        }
        Ok(())
    }
}

impl Default for CoreAdmissionProfile {
    fn default() -> Self {
        Self::safe()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreProfileError {
    ZeroLimit(&'static str),
    UnboundedLimit(&'static str),
}

impl fmt::Display for CoreProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroLimit(field) => write!(f, "{field} must be non-zero"),
            Self::UnboundedLimit(field) => write!(f, "{field} exceeds the public contract bound"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trap_codes_are_stable() {
        assert_eq!(CoreTrap::InvalidPc.code(), 0);
        assert_eq!(CoreTrap::InvalidRegister.code(), 1);
        assert_eq!(CoreTrap::TypeMismatch.code(), 2);
        assert_eq!(CoreTrap::DivisionByZero.code(), 3);
        assert_eq!(CoreTrap::IntegerOverflow.code(), 4);
        assert_eq!(CoreTrap::FuelExceeded.code(), 5);
        assert_eq!(CoreTrap::CallDepthExceeded.code(), 6);
        assert_eq!(CoreTrap::InvalidFunction.code(), 7);
        assert_eq!(CoreTrap::AssertFailed.code(), 8);
        assert_eq!(CoreTrap::ExplicitTrap.code(), 9);
    }

    #[test]
    fn trap_debug_is_deterministic() {
        assert_eq!(format!("{:?}", CoreTrap::DivisionByZero), "DivisionByZero");
    }

    #[test]
    fn fuel_consumed_per_instruction() {
        let mut fuel = FuelMeter::new(3);
        fuel.consume(1).unwrap();
        fuel.consume(1).unwrap();
        assert_eq!(fuel.remaining(), 1);
    }

    #[test]
    fn fuel_exceeded_returns_trap() {
        let mut fuel = FuelMeter::new(1);
        fuel.consume(1).unwrap();
        assert_eq!(fuel.consume(1), Err(CoreTrap::FuelExceeded));
        assert!(fuel.is_exhausted());
    }

    #[test]
    fn fuel_zero_rejects_execution() {
        let mut fuel = FuelMeter::new(0);
        assert_eq!(fuel.consume(1), Err(CoreTrap::FuelExceeded));
    }

    #[test]
    fn profile_default_safe() {
        let profile = CoreAdmissionProfile::default();
        assert!(profile.max_registers > 0);
        assert!(profile.max_functions > 0);
        assert!(profile.max_call_depth > 0);
        assert!(profile.max_instrs_per_function > 0);
        assert!(profile.max_fuel > 0);
    }

    #[test]
    fn profile_rejects_zero_limits() {
        let profile = CoreAdmissionProfile {
            max_registers: 0,
            ..CoreAdmissionProfile::default()
        };
        assert_eq!(
            profile.validate(),
            Err(CoreProfileError::ZeroLimit("max_registers"))
        );
    }

    #[test]
    fn profile_rejects_unbounded_limits() {
        let profile = CoreAdmissionProfile {
            max_registers: u16::MAX,
            ..CoreAdmissionProfile::default()
        };
        assert_eq!(
            profile.validate(),
            Err(CoreProfileError::UnboundedLimit("max_registers"))
        );
    }
}
