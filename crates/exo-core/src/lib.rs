#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod diagnostics;
pub mod ids;
pub mod source;
#[cfg(feature = "alloc")]
pub mod arena;
#[cfg(feature = "alloc")]
pub mod sigtable;

#[cfg(feature = "alloc")]
pub use arena::{Arena, ArenaId};
#[cfg(feature = "alloc")]
pub use sigtable::SigTable;
pub use diagnostics::{DiagLevel, Diagnostic};
pub use ids::{ExprId, StmtId, SymbolId};
pub use source::{FileId, SourceMark, Span};
#[cfg(feature = "alloc")]
pub use source::{SourceFile, SourceMap};
