//! Desktop session ownership, event polling, and frame lifecycle for the
//! Semantic UI application boundary.
//!
//! This crate owns the runtime side of the first-wave UI boundary: desktop
//! session lifecycle, input event polling, frame token ownership, and the
//! backend adapter contract.
//!
//! # Current Wave Status
//!
//! Wave 2: single-window session ownership, lifecycle API, deterministic event
//! polling, and frame-token ownership. All types are owner-layer contracts.
//! Actual backend wiring (winit/wgpu or equivalent) is deferred to Wave 3.
//!
//! # Backend Policy
//!
//! Backend selection is an internal implementation detail of this crate.
//! No backend library becomes a language-level promise in the first wave.
//! `UiBackendAdapter` is the only seam — no platform crate name crosses it.
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

// ── Error type ───────────────────────────────────────────────────────────────

/// Error type for UI runtime operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiRuntimeError {
    /// The required UI capability was not admitted for this session.
    CapabilityDenied(UiCapabilityKind),
    /// The requested UI operation is not yet admitted in this wave.
    OperationNotAdmitted(UiOperationId),
    /// The backend failed to create the window.
    WindowCreationFailed,
    /// The event loop terminated with a backend-level error.
    EventLoopFailed,
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
            UiRuntimeError::WindowCreationFailed => {
                write!(f, "backend failed to create window")
            }
            UiRuntimeError::EventLoopFailed => {
                write!(f, "event loop terminated with backend error")
            }
        }
    }
}

// ── PR 5: Single-window session ownership and lifecycle API ───────────────────

/// Configuration for creating a single desktop window.
///
/// Passed to `UiBackendAdapter::create_window` at session creation.
/// Backend selection is an internal detail; this struct is the public
/// contract between the owner layer and whatever backend is wired in Wave 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowConfig {
    pub title: alloc::string::String,
    pub width: u32,
    pub height: u32,
}

impl WindowConfig {
    pub fn new(title: impl Into<alloc::string::String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }
}

/// Continuation signal returned by the per-frame callback.
///
/// `Continue` keeps the loop alive; `ExitRequested` drains the loop cleanly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopControl {
    Continue,
    ExitRequested,
}

/// Lifecycle state of a `DesktopSession`.
///
/// Transitions: `Created` → `Running` → `Closed`.
/// No backward transitions are permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Created,
    Running,
    Closed,
}

/// Internal contract a backend must implement to be driven by `DesktopSession`.
///
/// This trait is the backend seam — no platform crate name appears in the
/// public API. Actual backend implementations are wired in Wave 3.
pub trait UiBackendAdapter {
    fn create_window(&mut self, config: &WindowConfig) -> Result<(), UiRuntimeError>;
    fn close_window(&mut self);
    /// Drive the event/frame loop, calling `on_event` once per iteration.
    ///
    /// The backend controls iteration timing; the closure signals whether to
    /// continue. In Wave 3 the backend reconciles `LoopControl::ExitRequested`
    /// with platform-native close events.
    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        on_event: F,
    ) -> Result<(), UiRuntimeError>;
}

/// Owner of a single desktop window session.
///
/// Wraps a `UiBackendAdapter`. Lifecycle: `create` → `run` → `close`.
///
/// `B` is the backend type. The concrete backend is supplied by the caller
/// through dependency injection, keeping platform crates out of the
/// public API surface.
pub struct DesktopSession<B: UiBackendAdapter> {
    backend: B,
    state: SessionState,
}

impl<B: UiBackendAdapter> DesktopSession<B> {
    /// Create a new session, initialising the backend window.
    ///
    /// Returns the session in `SessionState::Created` on success.
    pub fn create(mut backend: B, config: WindowConfig) -> Result<Self, UiRuntimeError> {
        backend.create_window(&config)?;
        Ok(Self {
            backend,
            state: SessionState::Created,
        })
    }

    /// Run the event/frame loop until the callback returns `LoopControl::ExitRequested`.
    ///
    /// Transitions the session to `SessionState::Running` on entry.
    /// The per-frame callback receives a mutable `EventBuffer`; events pushed
    /// into it during the frame are drained before the next iteration.
    ///
    /// NOTE: In Wave 2 the caller's `LoopControl` return governs termination
    /// via a captured flag. Wave 3 reconciles this with backend-native close
    /// events.
    pub fn run<F>(&mut self, mut on_frame: F) -> Result<(), UiRuntimeError>
    where
        F: FnMut(&mut EventBuffer) -> LoopControl,
    {
        self.state = SessionState::Running;
        let mut buffer = EventBuffer::new();
        let exit_requested = core::cell::Cell::new(false);
        let result = self.backend.run_event_loop(|_backend_control| {
            if exit_requested.get() {
                return;
            }
            let signal = on_frame(&mut buffer);
            let _ = buffer.drain();
            if signal == LoopControl::ExitRequested {
                exit_requested.set(true);
            }
        });
        result
    }

    /// Close the window and mark the session as closed.
    pub fn close(&mut self) {
        self.backend.close_window();
        self.state = SessionState::Closed;
    }

    /// Current lifecycle state of this session.
    pub fn state(&self) -> SessionState {
        self.state
    }
}

// ── PR 6: Deterministic event polling and frame-token ownership ───────────────

/// Taxonomy of first-wave input events admitted for polling.
///
/// Mouse, touch, and gamepad events are deferred to a future wave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEventKind {
    KeyDown { key_code: u32 },
    KeyUp { key_code: u32 },
    CloseRequested,
}

/// A single input event polled from the desktop session.
///
/// Replaces the Wave 0 inert marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEvent {
    pub kind: InputEventKind,
}

impl InputEvent {
    pub fn new(kind: InputEventKind) -> Self {
        Self { kind }
    }
}

/// Vec-backed accumulator for input events within a single frame.
///
/// The session's per-frame callback receives a `&mut EventBuffer`.
/// Events pushed by the backend (Wave 3) are drained each frame.
#[derive(Debug)]
pub struct EventBuffer {
    events: alloc::vec::Vec<InputEvent>,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self {
            events: alloc::vec::Vec::new(),
        }
    }

    /// Push a single event into the buffer.
    pub fn push(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    /// Drain all accumulated events, returning them as a `Vec`.
    ///
    /// After draining the buffer is empty.
    pub fn drain(&mut self) -> alloc::vec::Vec<InputEvent> {
        core::mem::replace(&mut self.events, alloc::vec::Vec::new())
    }

    /// Returns `true` if no events have been pushed since the last drain.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Token representing a single submitted draw frame.
///
/// Replaces the Wave 0 inert marker. Draw command submission semantics are
/// deferred to Wave 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameToken {
    pub frame_id: u64,
}

/// Issues monotonically increasing `FrameToken` values.
///
/// A single issuer should be held per session. Frame IDs start at 0.
pub struct FrameTokenIssuer {
    next_id: u64,
}

impl FrameTokenIssuer {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Issue the next sequential `FrameToken`.
    pub fn next(&mut self) -> FrameToken {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        FrameToken { frame_id: id }
    }
}

impl Default for FrameTokenIssuer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_config_holds_title_and_dimensions() {
        let cfg = WindowConfig::new("Hello Wave 2", 1280, 720);
        assert_eq!(cfg.title, "Hello Wave 2");
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
    }

    #[test]
    fn loop_control_continue_and_exit_are_distinct() {
        assert_ne!(LoopControl::Continue, LoopControl::ExitRequested);
        let a = LoopControl::Continue;
        let _b = a; // Copy — no move
        assert_eq!(a, LoopControl::Continue);
    }

    #[test]
    fn event_buffer_push_and_drain_roundtrip() {
        let mut buf = EventBuffer::new();
        assert!(buf.is_empty());
        buf.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));
        buf.push(InputEvent::new(InputEventKind::KeyUp { key_code: 65 }));
        buf.push(InputEvent::new(InputEventKind::CloseRequested));
        assert!(!buf.is_empty());
        let drained = buf.drain();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0].kind, InputEventKind::KeyDown { key_code: 65 });
        assert_eq!(drained[2].kind, InputEventKind::CloseRequested);
        assert!(buf.is_empty());
    }

    #[test]
    fn frame_token_issuer_issues_sequential_ids() {
        let mut issuer = FrameTokenIssuer::new();
        let t0 = issuer.next();
        let t1 = issuer.next();
        let t2 = issuer.next();
        assert_eq!(t0.frame_id, 0);
        assert_eq!(t1.frame_id, 1);
        assert_eq!(t2.frame_id, 2);
        assert!(t0 < t1);
        assert!(t1 < t2);
    }

    #[test]
    fn session_state_transitions_are_explicit() {
        let s = SessionState::Created;
        assert_ne!(s, SessionState::Running);
        assert_ne!(s, SessionState::Closed);
        assert_ne!(SessionState::Running, SessionState::Closed);
        let a = SessionState::Running;
        let _b = a; // Copy
        assert_eq!(a, SessionState::Running);
    }

    #[test]
    fn desktop_session_lifecycle_via_mock_backend() {
        struct MockBackend {
            created: bool,
            closed: bool,
            loop_iters: usize,
        }
        impl UiBackendAdapter for MockBackend {
            fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
                self.created = true;
                Ok(())
            }
            fn close_window(&mut self) {
                self.closed = true;
            }
            fn run_event_loop<F: FnMut(LoopControl)>(
                &mut self,
                mut on_event: F,
            ) -> Result<(), UiRuntimeError> {
                for _ in 0..self.loop_iters {
                    on_event(LoopControl::Continue);
                }
                Ok(())
            }
        }

        let backend = MockBackend {
            created: false,
            closed: false,
            loop_iters: 3,
        };
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session =
            DesktopSession::create(backend, cfg).expect("mock backend create must succeed");
        assert_eq!(session.state(), SessionState::Created);

        let mut frame_count = 0u32;
        session
            .run(|_buf| {
                frame_count += 1;
                LoopControl::Continue
            })
            .expect("mock run must succeed");
        assert_eq!(session.state(), SessionState::Running);
        assert_eq!(frame_count, 3);

        session.close();
        assert_eq!(session.state(), SessionState::Closed);
    }
}
