#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactValue {
    Bool(bool),
    I32(i32),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactResolution {
    Certain(FactValue),
    Uncertain(Vec<FactValue>),
    Conflicted(Vec<FactValue>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextWindow {
    pub name: String,
}

impl ContextWindow {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateEpoch(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionMetadata {
    pub key: String,
    pub from_epoch: StateEpoch,
    pub to_epoch: StateEpoch,
    pub reason: String,
}

pub type StateTransitionMetadata = TransitionMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateRecord {
    pub key: String,
    pub resolution: FactResolution,
    pub context: ContextWindow,
    pub epoch: StateEpoch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateUpdate {
    pub key: String,
    pub resolution: FactResolution,
    pub context: ContextWindow,
    pub reason: String,
}

impl StateUpdate {
    pub fn new(
        key: impl Into<String>,
        resolution: FactResolution,
        context: ContextWindow,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            resolution,
            context,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshot {
    pub epoch: StateEpoch,
    pub records: BTreeMap<String, StateRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateValidationCode {
    EmptyKey,
    EmptyContext,
    EmptyReason,
    EmptyAlternatives,
    DuplicateAlternatives,
    NotEnoughAlternatives,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateValidationError {
    pub code: StateValidationCode,
    pub message: String,
}

impl StateValidationError {
    pub fn new(code: StateValidationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl core::fmt::Display for StateValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StateValidationError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemanticStateStore {
    epoch: StateEpoch,
    records: BTreeMap<String, StateRecord>,
    transitions: Vec<TransitionMetadata>,
}

impl SemanticStateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn epoch(&self) -> StateEpoch {
        self.epoch
    }

    pub fn get(&self, key: &str) -> Option<&StateRecord> {
        self.records.get(key)
    }

    pub fn records(&self) -> &BTreeMap<String, StateRecord> {
        &self.records
    }

    pub fn transitions(&self) -> &[TransitionMetadata] {
        &self.transitions
    }

    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            epoch: self.epoch,
            records: self.records.clone(),
        }
    }

    pub fn apply(&mut self, update: StateUpdate) -> Result<TransitionMetadata, StateValidationError> {
        validate_update(&update)?;
        let from_epoch = self.epoch;
        let to_epoch = StateEpoch(self.epoch.0 + 1);
        let record = StateRecord {
            key: update.key.clone(),
            resolution: update.resolution,
            context: update.context,
            epoch: to_epoch,
        };
        self.records.insert(update.key.clone(), record);
        let transition = TransitionMetadata {
            key: update.key,
            from_epoch,
            to_epoch,
            reason: update.reason,
        };
        self.transitions.push(transition.clone());
        self.epoch = to_epoch;
        Ok(transition)
    }

    pub fn restore(&mut self, snapshot: StateSnapshot) {
        self.epoch = snapshot.epoch;
        self.records = snapshot.records;
    }
}

fn validate_update(update: &StateUpdate) -> Result<(), StateValidationError> {
    if update.key.trim().is_empty() {
        return Err(StateValidationError::new(
            StateValidationCode::EmptyKey,
            "state key must not be empty",
        ));
    }
    if update.context.name.trim().is_empty() {
        return Err(StateValidationError::new(
            StateValidationCode::EmptyContext,
            "context window must not be empty",
        ));
    }
    if update.reason.trim().is_empty() {
        return Err(StateValidationError::new(
            StateValidationCode::EmptyReason,
            "transition reason must not be empty",
        ));
    }

    match &update.resolution {
        FactResolution::Certain(_) => Ok(()),
        FactResolution::Uncertain(values) | FactResolution::Conflicted(values) => {
            if values.is_empty() {
                return Err(StateValidationError::new(
                    StateValidationCode::EmptyAlternatives,
                    "uncertain/conflicted resolution requires alternatives",
                ));
            }
            if values.len() < 2 {
                return Err(StateValidationError::new(
                    StateValidationCode::NotEnoughAlternatives,
                    "uncertain/conflicted resolution requires at least two alternatives",
                ));
            }
            let unique = values.iter().cloned().collect::<BTreeSet<_>>();
            if unique.len() != values.len() {
                return Err(StateValidationError::new(
                    StateValidationCode::DuplicateAlternatives,
                    "uncertain/conflicted resolution alternatives must be unique",
                ));
            }
            Ok(())
        }
    }
}

impl Default for StateEpoch {
    fn default() -> Self {
        Self(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_store_applies_updates_and_advances_epoch() {
        let mut store = SemanticStateStore::new();
        let transition = store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed fact",
            ))
            .expect("apply");

        assert_eq!(transition.from_epoch, StateEpoch(0));
        assert_eq!(transition.to_epoch, StateEpoch(1));
        assert_eq!(store.epoch(), StateEpoch(1));
        assert_eq!(
            store.get("fact.alpha").expect("record").resolution,
            FactResolution::Certain(FactValue::Bool(true))
        );
    }

    #[test]
    fn state_store_rejects_duplicate_alternatives_in_conflict() {
        let mut store = SemanticStateStore::new();
        let err = store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Conflicted(vec![
                    FactValue::I32(1),
                    FactValue::I32(1),
                ]),
                ContextWindow::new("root"),
                "conflict",
            ))
            .expect_err("must reject duplicate alternatives");
        assert_eq!(err.code, StateValidationCode::DuplicateAlternatives);
    }

    #[test]
    fn state_store_requires_context_and_reason() {
        let mut store = SemanticStateStore::new();
        let err = store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new(""),
                "",
            ))
            .expect_err("must reject empty context");
        assert_eq!(err.code, StateValidationCode::EmptyContext);
    }

    #[test]
    fn state_store_snapshot_roundtrip_restores_epoch_and_records() {
        let mut store = SemanticStateStore::new();
        store
            .apply(StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Text("ready".to_string())),
                ContextWindow::new("window.alpha"),
                "set initial fact",
            ))
            .expect("apply");
        let snapshot = store.snapshot();

        store
            .apply(StateUpdate::new(
                "fact.beta",
                FactResolution::Certain(FactValue::Bool(false)),
                ContextWindow::new("window.beta"),
                "mutate after snapshot",
            ))
            .expect("apply");
        store.restore(snapshot);

        assert_eq!(store.epoch(), StateEpoch(1));
        assert!(store.get("fact.alpha").is_some());
        assert!(store.get("fact.beta").is_none());
    }
}
