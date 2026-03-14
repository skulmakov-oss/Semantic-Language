#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use prom_cap::{CapabilityKind, CapabilityManifestMetadata};
use sm_runtime_core::ExecutionContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuditEventId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditSessionMetadata {
    pub context: ExecutionContext,
    pub capability_manifest: CapabilityManifestMetadata,
    pub gate_registry_bound: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditEventKind {
    SessionStarted { entry: String },
    SessionFinished,
    RuleActivated { rule_id: String, salience: i32 },
    StateTransition {
        key: String,
        from_epoch: u64,
        to_epoch: u64,
    },
    CapabilityDenied {
        capability: CapabilityKind,
        call: Option<String>,
    },
    GateRead { device_id: u16, port: u16 },
    GateWrite { device_id: u16, port: u16 },
    PulseEmit { signal: String },
    Note { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub kind: AuditEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMetadata {
    pub session: AuditSessionMetadata,
    pub event_count: usize,
    pub last_event_id: Option<AuditEventId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditTrail {
    session: AuditSessionMetadata,
    next_id: u64,
    events: Vec<AuditEvent>,
}

impl AuditTrail {
    pub fn new(session: AuditSessionMetadata) -> Self {
        Self {
            session,
            next_id: 0,
            events: Vec::new(),
        }
    }

    pub fn session(&self) -> &AuditSessionMetadata {
        &self.session
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn record(&mut self, kind: AuditEventKind) -> AuditEventId {
        let id = AuditEventId(self.next_id);
        self.next_id += 1;
        self.events.push(AuditEvent { id, kind });
        id
    }

    pub fn replay_metadata(&self) -> ReplayMetadata {
        ReplayMetadata {
            session: self.session.clone(),
            event_count: self.events.len(),
            last_event_id: self.events.last().map(|event| event.id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session() -> AuditSessionMetadata {
        AuditSessionMetadata {
            context: ExecutionContext::KernelBound,
            capability_manifest: CapabilityManifestMetadata {
                schema: "prom.cap.manifest".to_string(),
                version: prom_cap::CapabilityManifestVersion::V1,
            },
            gate_registry_bound: true,
        }
    }

    #[test]
    fn audit_trail_assigns_monotonic_event_ids() {
        let mut trail = AuditTrail::new(sample_session());
        let first = trail.record(AuditEventKind::SessionStarted {
            entry: "main".to_string(),
        });
        let second = trail.record(AuditEventKind::SessionFinished);
        assert_eq!(first, AuditEventId(0));
        assert_eq!(second, AuditEventId(1));
    }

    #[test]
    fn replay_metadata_reflects_session_and_event_count() {
        let session = sample_session();
        let mut trail = AuditTrail::new(session.clone());
        trail.record(AuditEventKind::SessionStarted {
            entry: "main".to_string(),
        });
        trail.record(AuditEventKind::GateRead {
            device_id: 7,
            port: 3,
        });

        let replay = trail.replay_metadata();
        assert_eq!(replay.session, session);
        assert_eq!(replay.event_count, 2);
        assert_eq!(replay.last_event_id, Some(AuditEventId(1)));
    }

    #[test]
    fn capability_denial_event_retains_call_context() {
        let mut trail = AuditTrail::new(sample_session());
        trail.record(AuditEventKind::CapabilityDenied {
            capability: CapabilityKind::PulseEmit,
            call: Some("PulseEmit".to_string()),
        });
        match &trail.events()[0].kind {
            AuditEventKind::CapabilityDenied { capability, call } => {
                assert_eq!(*capability, CapabilityKind::PulseEmit);
                assert_eq!(call.as_deref(), Some("PulseEmit"));
            }
            other => panic!("unexpected event {other:?}"),
        }
    }

    #[test]
    fn audit_trail_preserves_rule_activation_and_state_transition_events() {
        let mut trail = AuditTrail::new(sample_session());
        trail.record(AuditEventKind::RuleActivated {
            rule_id: "rule.alpha".to_string(),
            salience: 9,
        });
        trail.record(AuditEventKind::StateTransition {
            key: "fact.alpha".to_string(),
            from_epoch: 1,
            to_epoch: 2,
        });

        assert!(matches!(
            &trail.events()[0].kind,
            AuditEventKind::RuleActivated { rule_id, salience }
                if rule_id == "rule.alpha" && *salience == 9
        ));
        assert!(matches!(
            &trail.events()[1].kind,
            AuditEventKind::StateTransition {
                key,
                from_epoch,
                to_epoch
            } if key == "fact.alpha" && *from_epoch == 1 && *to_epoch == 2
        ));
    }
}
