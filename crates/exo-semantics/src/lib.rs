#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
pub mod alloc_core;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc_core::*;
#[cfg(feature = "std")]
pub use alloc_core::ModuleProvider;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use alloc_core::{
    build_export_sets_core, collect_local_exports_core, default_import_alias,
    fold_fx_const_call_core, has_magic_number_core, infer_atom_type_core,
    infer_law_entity_core, infer_when_condition_type_core, is_assignment_compatible,
    is_compatible_cmp, is_dead_when_condition, is_large_law_core, is_law_name_style_ok,
    is_valid_when_result_type_core, parse_import_directives, parse_law_local_decl,
    parse_select_items, track_entity_field_usage_core, evaluate_law_header_policy_core,
    insert_name_core, insert_scoped_name_core, LawHeaderPolicy,
    validate_import_bindings_core, validate_import_namespace_rules, validate_when_non_empty_core,
    validate_select_imports_core, ConditionInferError, ExportBuildError, ExportBuildModule,
    ExportItem, ExportKind, ExportOrigin, ExportSet, GateInstr, ImmutableIr, ImportDirective,
    ImportPolicyError, LawScheduler, LocalExportDecl, SelectImportModule,
    SelectImportPolicyError, SemanticType, ScopeKind, Symbol, SymbolError, SymbolTable, TypeId,
    TypeRegistry, WhenValidationError, diagnostic_help_core,
};

#[cfg(feature = "std")]
mod frontend {
    pub use exo_core::SourceMark;
    pub use exo_frontend::{
        parse_logos_program, parse_program, type_check_program, LogosEntity, LogosEntityFieldKind,
        LogosProgram, Type,
    };
}

#[cfg(feature = "std")]
mod std_adapters;

#[cfg(feature = "std")]
pub use std_adapters::*;
