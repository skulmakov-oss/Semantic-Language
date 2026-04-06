//! Desktop session ownership, event polling, and frame lifecycle for the
//! Semantic UI application boundary.
//!
//! This crate owns the runtime side of the first-wave UI boundary: desktop
//! session lifecycle, input event polling, frame token ownership, and the
//! backend adapter contract.
//!
//! # Current Wave Status
//!
//! Wave 0 scaffolding only. All types are inert markers. Desktop session
//! creation, event polling, drawing, and backend adapter wiring are deferred
//! to Wave 2+ (desktop lifecycle) and Wave 3 (minimal drawing surface).
//!
//! # Backend Policy
//!
//! Backend selection is an internal implementation detail of this crate.
//! No backend library becomes a language-level promise in the first wave.
//!
//! # Non-Commitments
//!
//! This crate does not claim:
//! - a specific graphics backend or wgpu fork
//! - multi-window, browser, or mobile support
//! - a widget/layout framework
//! - that UI runtime support is already part of the published `v1.1.1` line
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use prom_ui::{UiCapabilityKind, UiOperationId};

/// Inert marker for a desktop window session handle.
///
/// Lifecycle ownership (create, run, close) is deferred to Wave 2.
#[derive(Debug)]
pub struct DesktopSessionHandle {
    _private: (),
}

/// Inert marker for a single input event polled from the desktop session.
///
/// Event taxonomy and polling semantics are deferred to Wave 2.
#[derive(Debug, Clone)]
pub struct InputEvent {
    _private: (),
}

/// Inert marker for a frame token representing one submitted draw frame.
///
/// Draw command family and submission semantics are deferred to Wave 3.
#[derive(Debug)]
pub struct FrameToken {
    _private: (),
}

/// Error type for UI runtime operations.
///
/// Extended in Wave 1+ as admission and lifecycle paths are wired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiRuntimeError {
    /// The required UI capability was not admitted for this session.
    CapabilityDenied(UiCapabilityKind),
    /// The requested UI operation is not yet admitted in this wave.
    OperationNotAdmitted(UiOperationId),
}

impl core::fmt::Display for UiRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UiRuntimeError::CapabilityDenied(k) => {
                write!(f, "UI capability denied: {:?}", k)
            }
            UiRuntimeError::OperationNotAdmitted(op) => {
                write!(f, "UI operation not yet admitted: {:?}", op)
            }
        }
    }
}
