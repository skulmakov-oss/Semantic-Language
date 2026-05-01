//! Public facade for the Semantic execution core capsule.
//!
//! This crate intentionally keeps a narrow API surface and delegates the
//! internal execution model to sibling crates.

use semantic_core_exec::{validate_program, CoreExecutor, CoreProgram, CoreValidationError};

#[derive(Debug)]
pub struct CoreCapsule {
    inner: sealed::CoreInner,
}

pub use semantic_core_exec::{CoreConfig, CoreEnginePolicy, CoreResult, CoreStatus};

#[derive(Debug)]
pub enum CoreError {
    Validation(CoreValidationError),
}

mod sealed {
    use semantic_core_exec::{CoreConfig, CoreExecutor};

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
                executor: CoreExecutor::new(config),
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
