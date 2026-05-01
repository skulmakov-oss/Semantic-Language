//! Public facade for the Semantic execution core capsule.
//!
//! This crate intentionally keeps a narrow API surface and delegates the
//! internal execution model to sibling crates.

use semantic_core_exec::{validate_program, CoreExecutor, CoreProgram, CoreValidationError};
use semantic_core_runtime::CoreAdmissionProfile;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CoreCapsule {
    inner: sealed::CoreInner,
}

pub use semantic_core_exec::{CoreResult, CoreStatus};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    #[cfg_attr(feature = "serde", serde(alias = "Scalar", alias = "scalar"))]
    Reference,
    #[cfg_attr(feature = "serde", serde(alias = "Auto", alias = "auto"))]
    Adaptive,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreConfig {
    pub fuel: u64,
    pub max_call_depth: u16,
    #[cfg_attr(feature = "serde", serde(alias = "backend"))]
    pub engine: ExecutionMode,
    pub validate_before_execute: bool,
    pub profile: CoreAdmissionProfile,
}

impl Default for CoreConfig {
    fn default() -> Self {
        let profile = CoreAdmissionProfile::default();
        Self {
            fuel: profile.max_fuel,
            max_call_depth: profile.max_call_depth,
            engine: ExecutionMode::Adaptive,
            validate_before_execute: true,
            profile,
        }
    }
}

#[derive(Debug)]
pub enum CoreError {
    Validation(CoreValidationError),
}

mod sealed {
    use semantic_core_exec::CoreExecutor;

    use crate::CoreConfig;

    #[derive(Debug)]
    pub(crate) struct CoreInner {
        pub(crate) config: CoreConfig,
        pub(crate) executor: CoreExecutor,
    }
}

impl CoreCapsule {
    pub fn new(config: CoreConfig) -> Self {
        Self {
            inner: sealed::CoreInner {
                executor: CoreExecutor::new(config.into()),
                config,
            },
        }
    }

    pub fn config(&self) -> CoreConfig {
        self.inner.config
    }

    pub fn validate(&self, program: &CoreProgram) -> Result<(), CoreError> {
        validate_program(program, self.inner.config.profile).map_err(CoreError::Validation)
    }

    pub fn run(&self, program: &CoreProgram) -> Result<CoreResult, CoreError> {
        if self.inner.config.validate_before_execute {
            self.validate(program)?;
        }
        Ok(self.inner.executor.execute_outcome(program))
    }
}

impl From<ExecutionMode> for semantic_core_exec::ExecutionMode {
    fn from(value: ExecutionMode) -> Self {
        match value {
            ExecutionMode::Reference => Self::Reference,
            ExecutionMode::Adaptive => Self::Adaptive,
        }
    }
}

impl From<CoreConfig> for semantic_core_exec::CoreConfig {
    fn from(value: CoreConfig) -> Self {
        Self {
            fuel: value.fuel,
            max_call_depth: value.max_call_depth,
            engine: value.engine.into(),
            validate_before_execute: value.validate_before_execute,
            profile: value.profile,
        }
    }
}
