#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec::Vec;

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq)]
pub struct RecordCarrier<T> {
    pub type_name: String,
    pub slots: Vec<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub u32);

impl SymbolId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionContext {
    PureCompute,
    VerifiedLocal,
    RuleExecution,
    KernelBound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaKind {
    Steps,
    Calls,
    StackDepth,
    Frames,
    Registers,
    ConstPool,
    SymbolTable,
    EffectCalls,
    TraceEntries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuotaExceeded {
    pub kind: QuotaKind,
    pub limit: usize,
    pub used: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTrap {
    AssertionFailed,
    StackOverflow,
    StackUnderflow,
    TypeMismatch,
    InvalidOpcode,
    InvalidJump,
    CapabilityDenied,
    AbiViolation,
    VerifierRejected,
    QuotaExceeded(QuotaExceeded),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeQuotas {
    pub max_steps: usize,
    pub max_calls: usize,
    pub max_stack_depth: usize,
    pub max_frames: usize,
    pub max_registers: usize,
    pub max_const_pool: usize,
    pub max_symbol_table: usize,
    pub max_effect_calls: usize,
    pub max_trace_entries: usize,
}

impl RuntimeQuotas {
    pub const fn verified_local() -> Self {
        Self {
            max_steps: 100_000,
            max_calls: 16_384,
            max_stack_depth: 256,
            max_frames: 256,
            max_registers: 4_096,
            max_const_pool: 65_536,
            max_symbol_table: 16_384,
            max_effect_calls: 1_024,
            max_trace_entries: 8_192,
        }
    }

    pub const fn pure_compute() -> Self {
        Self {
            max_steps: 100_000,
            max_calls: 16_384,
            max_stack_depth: 256,
            max_frames: 256,
            max_registers: 4_096,
            max_const_pool: 65_536,
            max_symbol_table: 16_384,
            max_effect_calls: 0,
            max_trace_entries: 4_096,
        }
    }

    pub const fn kernel_bound() -> Self {
        Self {
            max_steps: 250_000,
            max_calls: 32_768,
            max_stack_depth: 256,
            max_frames: 256,
            max_registers: 8_192,
            max_const_pool: 65_536,
            max_symbol_table: 16_384,
            max_effect_calls: 4_096,
            max_trace_entries: 16_384,
        }
    }

    pub fn exceed(self, kind: QuotaKind, used: usize) -> Option<QuotaExceeded> {
        let limit = match kind {
            QuotaKind::Steps => self.max_steps,
            QuotaKind::Calls => self.max_calls,
            QuotaKind::StackDepth => self.max_stack_depth,
            QuotaKind::Frames => self.max_frames,
            QuotaKind::Registers => self.max_registers,
            QuotaKind::ConstPool => self.max_const_pool,
            QuotaKind::SymbolTable => self.max_symbol_table,
            QuotaKind::EffectCalls => self.max_effect_calls,
            QuotaKind::TraceEntries => self.max_trace_entries,
        };
        (used > limit).then_some(QuotaExceeded { kind, limit, used })
    }
}

impl Default for RuntimeQuotas {
    fn default() -> Self {
        Self::verified_local()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionConfig {
    pub context: ExecutionContext,
    pub quotas: RuntimeQuotas,
    pub trace_enabled: bool,
}

impl ExecutionConfig {
    pub const fn new(context: ExecutionContext, quotas: RuntimeQuotas) -> Self {
        Self {
            context,
            quotas,
            trace_enabled: false,
        }
    }

    pub const fn for_context(context: ExecutionContext) -> Self {
        let quotas = match context {
            ExecutionContext::PureCompute => RuntimeQuotas::pure_compute(),
            ExecutionContext::VerifiedLocal | ExecutionContext::RuleExecution => {
                RuntimeQuotas::verified_local()
            }
            ExecutionContext::KernelBound => RuntimeQuotas::kernel_bound(),
        };
        Self::new(context, quotas)
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeSymbolTable {
    ordered: Vec<String>,
    index: BTreeMap<String, SymbolId>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl RuntimeSymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, name: &str) -> SymbolId {
        if let Some(id) = self.index.get(name) {
            return *id;
        }
        let id = SymbolId(self.ordered.len() as u32);
        self.ordered.push(name.to_string());
        self.index.insert(name.to_string(), id);
        id
    }

    pub fn resolve(&self, id: SymbolId) -> Option<&str> {
        self.ordered.get(id.0 as usize).map(|name| name.as_str())
    }

    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DebugNameMap {
    names: Vec<String>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl DebugNameMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, name: &str) -> SymbolId {
        let id = SymbolId(self.names.len() as u32);
        self.names.push(name.to_string());
        id
    }

    pub fn resolve(&self, id: SymbolId) -> Option<&str> {
        self.names.get(id.0 as usize).map(|name| name.as_str())
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_symbol_table_assigns_deterministic_ids() {
        let mut table = RuntimeSymbolTable::new();
        let alpha = table.intern("alpha");
        let beta = table.intern("beta");
        let alpha_again = table.intern("alpha");

        assert_eq!(alpha, SymbolId(0));
        assert_eq!(beta, SymbolId(1));
        assert_eq!(alpha_again, alpha);
        assert_eq!(table.resolve(beta), Some("beta"));
    }

    #[test]
    fn debug_name_map_is_append_only_and_stable() {
        let mut names = DebugNameMap::new();
        let first = names.push("main");
        let second = names.push("helper");

        assert_eq!(first, SymbolId(0));
        assert_eq!(second, SymbolId(1));
        assert_eq!(names.resolve(first), Some("main"));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn verified_local_quotas_keep_stack_depth_at_contract_value() {
        let quotas = RuntimeQuotas::verified_local();
        assert_eq!(quotas.max_stack_depth, 256);
        assert_eq!(quotas.max_effect_calls, 1_024);
    }

    #[test]
    fn quota_exceed_reports_precise_limit_and_usage() {
        let quotas = RuntimeQuotas::pure_compute();
        let exceed = quotas
            .exceed(QuotaKind::EffectCalls, 1)
            .expect("must exceed");

        assert_eq!(
            exceed,
            QuotaExceeded {
                kind: QuotaKind::EffectCalls,
                limit: 0,
                used: 1,
            }
        );
    }

    #[test]
    fn execution_config_uses_context_defaults() {
        let config = ExecutionConfig::for_context(ExecutionContext::KernelBound);
        assert_eq!(config.context, ExecutionContext::KernelBound);
        assert_eq!(config.quotas.max_effect_calls, 4_096);
    }
}
