use sm_front::{
    build_adt_table, build_record_table, canonicalize_declared_type, parse_program,
    resolve_symbol_name, AstArena, FrontendError, SchemaDecl, SchemaShape, SchemaVersion, Type,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCompatibilityKind {
    Equivalent,
    Additive,
    Breaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaFieldChangeKind {
    Added,
    Removed,
    TypeChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaFieldChange {
    pub field_name: String,
    pub kind: SchemaFieldChangeKind,
    pub previous_type: Option<String>,
    pub next_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSchemaCompatibilityReport {
    pub schema_name: String,
    pub previous_version: u32,
    pub next_version: u32,
    pub compatibility: SchemaCompatibilityKind,
    pub changes: Vec<SchemaFieldChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaVariantChangeKind {
    Added,
    Removed,
    PayloadChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedUnionSchemaVariantChange {
    pub variant_name: String,
    pub kind: SchemaVariantChangeKind,
    pub field_changes: Vec<SchemaFieldChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedUnionSchemaCompatibilityReport {
    pub schema_name: String,
    pub previous_version: u32,
    pub next_version: u32,
    pub compatibility: SchemaCompatibilityKind,
    pub variant_changes: Vec<TaggedUnionSchemaVariantChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaCompatibilityBuildError {
    pub message: String,
}

impl fmt::Display for SchemaCompatibilityBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "schema compatibility build error: {}", self.message)
    }
}

impl Error for SchemaCompatibilityBuildError {}

pub fn classify_record_schema_compatibility(
    previous_src: &str,
    next_src: &str,
    schema_name: &str,
) -> Result<RecordSchemaCompatibilityReport, SchemaCompatibilityBuildError> {
    let previous_program = parse_program(previous_src).map_err(schema_compatibility_build_error)?;
    let next_program = parse_program(next_src).map_err(schema_compatibility_build_error)?;

    let previous_schema = find_named_schema(&previous_program, schema_name)?;
    let next_schema = find_named_schema(&next_program, schema_name)?;

    let previous_version = require_schema_version(previous_schema, &previous_program.arena)?;
    let next_version = require_schema_version(next_schema, &next_program.arena)?;
    if next_version.value <= previous_version.value {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility requires increasing versions; got {} -> {}",
                schema_name, previous_version.value, next_version.value
            ),
        });
    }
    if previous_schema.role != next_schema.role {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility requires stable schema role across versions",
                schema_name
            ),
        });
    }

    let SchemaShape::Record(previous_fields) = &previous_schema.shape else {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility slice currently supports only record-shaped schemas",
                schema_name
            ),
        });
    };
    let SchemaShape::Record(next_fields) = &next_schema.shape else {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility slice currently supports only record-shaped schemas",
                schema_name
            ),
        });
    };

    let previous_record_table =
        build_record_table(&previous_program).map_err(schema_compatibility_build_error)?;
    let previous_adt_table =
        build_adt_table(&previous_program).map_err(schema_compatibility_build_error)?;
    let next_record_table =
        build_record_table(&next_program).map_err(schema_compatibility_build_error)?;
    let next_adt_table = build_adt_table(&next_program).map_err(schema_compatibility_build_error)?;

    let mut next_by_name = BTreeMap::new();
    for field in next_fields {
        let field_name = resolve_symbol_name(&next_program.arena, field.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        next_by_name.insert(field_name, field);
    }
    let mut previous_names = BTreeSet::new();
    let mut changes = Vec::new();
    let mut compatibility = SchemaCompatibilityKind::Equivalent;

    for field in previous_fields {
        let field_name = resolve_symbol_name(&previous_program.arena, field.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        previous_names.insert(field_name.clone());
        let previous_type = canonicalize_declared_type(
            &field.ty,
            &previous_record_table,
            &previous_adt_table,
            &previous_program.arena,
        )
        .map_err(schema_compatibility_build_error)?;
        let previous_type_text =
            display_schema_compatibility_type(&previous_type, &previous_program.arena)
                .map_err(schema_compatibility_build_error)?;
        match next_by_name.get(&field_name) {
            Some(next_field) => {
                let next_type = canonicalize_declared_type(
                    &next_field.ty,
                    &next_record_table,
                    &next_adt_table,
                    &next_program.arena,
                )
                .map_err(schema_compatibility_build_error)?;
                let next_type_text = display_schema_compatibility_type(&next_type, &next_program.arena)
                    .map_err(schema_compatibility_build_error)?;
                if previous_type != next_type {
                    compatibility = SchemaCompatibilityKind::Breaking;
                    changes.push(SchemaFieldChange {
                        field_name,
                        kind: SchemaFieldChangeKind::TypeChanged,
                        previous_type: Some(previous_type_text),
                        next_type: Some(next_type_text),
                    });
                }
            }
            None => {
                compatibility = SchemaCompatibilityKind::Breaking;
                changes.push(SchemaFieldChange {
                    field_name,
                    kind: SchemaFieldChangeKind::Removed,
                    previous_type: Some(previous_type_text),
                    next_type: None,
                });
            }
        }
    }

    for field in next_fields {
        let field_name = resolve_symbol_name(&next_program.arena, field.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        if previous_names.contains(&field_name) {
            continue;
        }
        let next_type = canonicalize_declared_type(
            &field.ty,
            &next_record_table,
            &next_adt_table,
            &next_program.arena,
        )
        .map_err(schema_compatibility_build_error)?;
        let next_type_text = display_schema_compatibility_type(&next_type, &next_program.arena)
            .map_err(schema_compatibility_build_error)?;
        if compatibility != SchemaCompatibilityKind::Breaking {
            compatibility = SchemaCompatibilityKind::Additive;
        }
        changes.push(SchemaFieldChange {
            field_name,
            kind: SchemaFieldChangeKind::Added,
            previous_type: None,
            next_type: Some(next_type_text),
        });
    }

    Ok(RecordSchemaCompatibilityReport {
        schema_name: schema_name.to_string(),
        previous_version: previous_version.value,
        next_version: next_version.value,
        compatibility,
        changes,
    })
}

pub fn classify_tagged_union_schema_compatibility(
    previous_src: &str,
    next_src: &str,
    schema_name: &str,
) -> Result<TaggedUnionSchemaCompatibilityReport, SchemaCompatibilityBuildError> {
    let previous_program = parse_program(previous_src).map_err(schema_compatibility_build_error)?;
    let next_program = parse_program(next_src).map_err(schema_compatibility_build_error)?;

    let previous_schema = find_named_schema(&previous_program, schema_name)?;
    let next_schema = find_named_schema(&next_program, schema_name)?;

    let previous_version = require_schema_version(previous_schema, &previous_program.arena)?;
    let next_version = require_schema_version(next_schema, &next_program.arena)?;
    if next_version.value <= previous_version.value {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility requires increasing versions; got {} -> {}",
                schema_name, previous_version.value, next_version.value
            ),
        });
    }
    if previous_schema.role != next_schema.role {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility requires stable schema role across versions",
                schema_name
            ),
        });
    }

    let SchemaShape::TaggedUnion(previous_variants) = &previous_schema.shape else {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility slice currently supports only tagged-union schemas",
                schema_name
            ),
        });
    };
    let SchemaShape::TaggedUnion(next_variants) = &next_schema.shape else {
        return Err(SchemaCompatibilityBuildError {
            message: format!(
                "schema '{}' compatibility slice currently supports only tagged-union schemas",
                schema_name
            ),
        });
    };

    let previous_record_table =
        build_record_table(&previous_program).map_err(schema_compatibility_build_error)?;
    let previous_adt_table =
        build_adt_table(&previous_program).map_err(schema_compatibility_build_error)?;
    let next_record_table =
        build_record_table(&next_program).map_err(schema_compatibility_build_error)?;
    let next_adt_table = build_adt_table(&next_program).map_err(schema_compatibility_build_error)?;

    let mut next_by_name = BTreeMap::new();
    for variant in next_variants {
        let variant_name = resolve_symbol_name(&next_program.arena, variant.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        next_by_name.insert(variant_name, variant);
    }
    let mut previous_names = BTreeSet::new();
    let mut variant_changes = Vec::new();
    let mut compatibility = SchemaCompatibilityKind::Equivalent;

    for variant in previous_variants {
        let variant_name = resolve_symbol_name(&previous_program.arena, variant.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        previous_names.insert(variant_name.clone());
        match next_by_name.get(&variant_name) {
            Some(next_variant) => {
                let field_changes = classify_variant_field_changes(
                    &variant.fields,
                    &previous_program.arena,
                    &previous_record_table,
                    &previous_adt_table,
                    &next_variant.fields,
                    &next_program.arena,
                    &next_record_table,
                    &next_adt_table,
                )?;
                if !field_changes.is_empty() {
                    if field_changes.iter().any(|change| {
                        matches!(
                            change.kind,
                            SchemaFieldChangeKind::Removed | SchemaFieldChangeKind::TypeChanged
                        )
                    }) {
                        compatibility = SchemaCompatibilityKind::Breaking;
                    } else if compatibility != SchemaCompatibilityKind::Breaking {
                        compatibility = SchemaCompatibilityKind::Additive;
                    }
                    variant_changes.push(TaggedUnionSchemaVariantChange {
                        variant_name,
                        kind: SchemaVariantChangeKind::PayloadChanged,
                        field_changes,
                    });
                }
            }
            None => {
                compatibility = SchemaCompatibilityKind::Breaking;
                variant_changes.push(TaggedUnionSchemaVariantChange {
                    variant_name,
                    kind: SchemaVariantChangeKind::Removed,
                    field_changes: Vec::new(),
                });
            }
        }
    }

    for variant in next_variants {
        let variant_name = resolve_symbol_name(&next_program.arena, variant.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        if previous_names.contains(&variant_name) {
            continue;
        }
        if compatibility != SchemaCompatibilityKind::Breaking {
            compatibility = SchemaCompatibilityKind::Additive;
        }
        variant_changes.push(TaggedUnionSchemaVariantChange {
            variant_name,
            kind: SchemaVariantChangeKind::Added,
            field_changes: Vec::new(),
        });
    }

    Ok(TaggedUnionSchemaCompatibilityReport {
        schema_name: schema_name.to_string(),
        previous_version: previous_version.value,
        next_version: next_version.value,
        compatibility,
        variant_changes,
    })
}

fn schema_compatibility_build_error(error: FrontendError) -> SchemaCompatibilityBuildError {
    SchemaCompatibilityBuildError {
        message: error.message,
    }
}

fn find_named_schema<'a>(
    program: &'a sm_front::Program,
    schema_name: &str,
) -> Result<&'a SchemaDecl, SchemaCompatibilityBuildError> {
    program
        .schemas
        .iter()
        .find(|schema| {
            resolve_symbol_name(&program.arena, schema.name)
                .map(|name| name == schema_name)
                .unwrap_or(false)
        })
        .ok_or_else(|| SchemaCompatibilityBuildError {
            message: format!("unknown schema '{}'", schema_name),
        })
}

fn require_schema_version<'a>(
    schema: &'a SchemaDecl,
    arena: &AstArena,
) -> Result<&'a SchemaVersion, SchemaCompatibilityBuildError> {
    schema.version.as_ref().ok_or_else(|| SchemaCompatibilityBuildError {
        message: format!(
            "schema '{}' compatibility requires explicit version metadata",
            resolve_symbol_name(arena, schema.name).unwrap_or("<invalid-schema>")
        ),
    })
}

fn classify_variant_field_changes(
    previous_fields: &[sm_front::SchemaField],
    previous_arena: &AstArena,
    previous_record_table: &sm_front::RecordTable,
    previous_adt_table: &sm_front::AdtTable,
    next_fields: &[sm_front::SchemaField],
    next_arena: &AstArena,
    next_record_table: &sm_front::RecordTable,
    next_adt_table: &sm_front::AdtTable,
) -> Result<Vec<SchemaFieldChange>, SchemaCompatibilityBuildError> {
    let mut next_by_name = BTreeMap::new();
    for field in next_fields {
        let field_name = resolve_symbol_name(next_arena, field.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        next_by_name.insert(field_name, field);
    }
    let mut previous_names = BTreeSet::new();
    let mut changes = Vec::new();

    for field in previous_fields {
        let field_name = resolve_symbol_name(previous_arena, field.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        previous_names.insert(field_name.clone());
        let previous_type = canonicalize_declared_type(
            &field.ty,
            previous_record_table,
            previous_adt_table,
            previous_arena,
        )
        .map_err(schema_compatibility_build_error)?;
        let previous_type_text = display_schema_compatibility_type(&previous_type, previous_arena)
            .map_err(schema_compatibility_build_error)?;

        match next_by_name.get(&field_name) {
            Some(next_field) => {
                let next_type = canonicalize_declared_type(
                    &next_field.ty,
                    next_record_table,
                    next_adt_table,
                    next_arena,
                )
                .map_err(schema_compatibility_build_error)?;
                let next_type_text = display_schema_compatibility_type(&next_type, next_arena)
                    .map_err(schema_compatibility_build_error)?;
                if previous_type != next_type {
                    changes.push(SchemaFieldChange {
                        field_name,
                        kind: SchemaFieldChangeKind::TypeChanged,
                        previous_type: Some(previous_type_text),
                        next_type: Some(next_type_text),
                    });
                }
            }
            None => {
                changes.push(SchemaFieldChange {
                    field_name,
                    kind: SchemaFieldChangeKind::Removed,
                    previous_type: Some(previous_type_text),
                    next_type: None,
                });
            }
        }
    }

    for field in next_fields {
        let field_name = resolve_symbol_name(next_arena, field.name)
            .map_err(schema_compatibility_build_error)?
            .to_string();
        if previous_names.contains(&field_name) {
            continue;
        }
        let next_type = canonicalize_declared_type(
            &field.ty,
            next_record_table,
            next_adt_table,
            next_arena,
        )
        .map_err(schema_compatibility_build_error)?;
        let next_type_text = display_schema_compatibility_type(&next_type, next_arena)
            .map_err(schema_compatibility_build_error)?;
        changes.push(SchemaFieldChange {
            field_name,
            kind: SchemaFieldChangeKind::Added,
            previous_type: None,
            next_type: Some(next_type_text),
        });
    }

    Ok(changes)
}

fn display_schema_compatibility_type(
    ty: &Type,
    arena: &AstArena,
) -> Result<String, FrontendError> {
    Ok(match ty {
        Type::Quad => "quad".to_string(),
        Type::QVec(width) => format!("qvec({})", width),
        Type::Bool => "bool".to_string(),
        Type::I32 => "i32".to_string(),
        Type::U32 => "u32".to_string(),
        Type::Fx => "fx".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Measured(base, unit) => format!(
            "{}[{}]",
            display_schema_compatibility_type(base, arena)?,
            resolve_symbol_name(arena, *unit)?
        ),
        Type::RangeI32 => "range<i32>".to_string(),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(|item| display_schema_compatibility_type(item, arena))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ),
        Type::Option(item) => format!("Option({})", display_schema_compatibility_type(item, arena)?),
        Type::Result(ok_ty, err_ty) => format!(
            "Result({}, {})",
            display_schema_compatibility_type(ok_ty, arena)?,
            display_schema_compatibility_type(err_ty, arena)?,
        ),
        Type::Record(name) | Type::Adt(name) => resolve_symbol_name(arena, *name)?.to_string(),
        Type::Unit => "()".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_record_schema_compatibility_reports_additive_field_growth() {
        let previous = r#"
api schema Telemetry version(1) {
    enabled: bool,
}

fn main() {
    return;
}
"#;
        let next = r#"
api schema Telemetry version(2) {
    enabled: bool,
    interval_ms: u32[ms],
}

fn main() {
    return;
}
"#;

        let report =
            classify_record_schema_compatibility(previous, next, "Telemetry")
                .expect("record-shaped schema compatibility should classify");

        assert_eq!(report.previous_version, 1);
        assert_eq!(report.next_version, 2);
        assert_eq!(report.compatibility, SchemaCompatibilityKind::Additive);
        assert_eq!(report.changes.len(), 1);
        assert_eq!(report.changes[0].field_name, "interval_ms");
        assert_eq!(report.changes[0].kind, SchemaFieldChangeKind::Added);
        assert_eq!(report.changes[0].previous_type, None);
        assert_eq!(report.changes[0].next_type.as_deref(), Some("u32[ms]"));
    }

    #[test]
    fn classify_record_schema_compatibility_reports_breaking_field_changes() {
        let previous = r#"
wire schema Envelope version(2) {
    count: i32,
    status: quad,
}

fn main() {
    return;
}
"#;
        let next = r#"
wire schema Envelope version(3) {
    count: u32,
}

fn main() {
    return;
}
"#;

        let report =
            classify_record_schema_compatibility(previous, next, "Envelope")
                .expect("breaking compatibility should still classify");

        assert_eq!(report.compatibility, SchemaCompatibilityKind::Breaking);
        assert_eq!(report.changes.len(), 2);
        assert_eq!(report.changes[0].field_name, "count");
        assert_eq!(report.changes[0].kind, SchemaFieldChangeKind::TypeChanged);
        assert_eq!(report.changes[0].previous_type.as_deref(), Some("i32"));
        assert_eq!(report.changes[0].next_type.as_deref(), Some("u32"));
        assert_eq!(report.changes[1].field_name, "status");
        assert_eq!(report.changes[1].kind, SchemaFieldChangeKind::Removed);
        assert_eq!(report.changes[1].previous_type.as_deref(), Some("quad"));
        assert_eq!(report.changes[1].next_type, None);
    }

    #[test]
    fn classify_record_schema_compatibility_rejects_missing_version_metadata() {
        let previous = r#"
schema Telemetry {
    enabled: bool,
}

fn main() {
    return;
}
"#;
        let next = r#"
schema Telemetry version(2) {
    enabled: bool,
}

fn main() {
    return;
}
"#;

        let err = classify_record_schema_compatibility(previous, next, "Telemetry")
            .expect_err("missing version metadata must reject");
        assert!(err
            .message
            .contains("compatibility requires explicit version metadata"));
    }

    #[test]
    fn classify_record_schema_compatibility_rejects_tagged_union_schemas_in_record_slice() {
        let previous = r#"
wire schema Envelope version(1) {
    Empty {},
}

fn main() {
    return;
}
"#;
        let next = r#"
wire schema Envelope version(2) {
    Empty {},
    Data {
        count: i32,
    },
}

fn main() {
    return;
}
"#;

        let err = classify_record_schema_compatibility(previous, next, "Envelope")
            .expect_err("tagged-union schemas are deferred");
        assert!(err
            .message
            .contains("currently supports only record-shaped schemas"));
    }

    #[test]
    fn classify_tagged_union_schema_compatibility_reports_additive_variant_growth() {
        let previous = r#"
wire schema Envelope version(1) {
    Empty {},
}

fn main() {
    return;
}
"#;
        let next = r#"
wire schema Envelope version(2) {
    Empty {},
    Data {
        count: i32,
    },
}

fn main() {
    return;
}
"#;

        let report = classify_tagged_union_schema_compatibility(previous, next, "Envelope")
            .expect("tagged-union compatibility should classify additive variant growth");

        assert_eq!(report.compatibility, SchemaCompatibilityKind::Additive);
        assert_eq!(report.variant_changes.len(), 1);
        assert_eq!(report.variant_changes[0].variant_name, "Data");
        assert_eq!(report.variant_changes[0].kind, SchemaVariantChangeKind::Added);
        assert!(report.variant_changes[0].field_changes.is_empty());
    }

    #[test]
    fn classify_tagged_union_schema_compatibility_reports_additive_payload_growth() {
        let previous = r#"
api schema Event version(2) {
    Data {
        count: i32,
    },
}

fn main() {
    return;
}
"#;
        let next = r#"
api schema Event version(3) {
    Data {
        count: i32,
        interval_ms: u32[ms],
    },
}

fn main() {
    return;
}
"#;

        let report = classify_tagged_union_schema_compatibility(previous, next, "Event")
            .expect("tagged-union payload growth should classify");

        assert_eq!(report.compatibility, SchemaCompatibilityKind::Additive);
        assert_eq!(report.variant_changes.len(), 1);
        assert_eq!(report.variant_changes[0].variant_name, "Data");
        assert_eq!(
            report.variant_changes[0].kind,
            SchemaVariantChangeKind::PayloadChanged
        );
        assert_eq!(report.variant_changes[0].field_changes.len(), 1);
        assert_eq!(
            report.variant_changes[0].field_changes[0].kind,
            SchemaFieldChangeKind::Added
        );
        assert_eq!(
            report.variant_changes[0].field_changes[0].field_name,
            "interval_ms"
        );
    }

    #[test]
    fn classify_tagged_union_schema_compatibility_reports_breaking_variant_and_payload_changes() {
        let previous = r#"
wire schema Envelope version(3) {
    Empty {},
    Data {
        count: i32,
        status: quad,
    },
}

fn main() {
    return;
}
"#;
        let next = r#"
wire schema Envelope version(4) {
    Data {
        count: u32,
    },
}

fn main() {
    return;
}
"#;

        let report = classify_tagged_union_schema_compatibility(previous, next, "Envelope")
            .expect("breaking tagged-union changes should still classify");

        assert_eq!(report.compatibility, SchemaCompatibilityKind::Breaking);
        assert_eq!(report.variant_changes.len(), 2);
        let removed = report
            .variant_changes
            .iter()
            .find(|change| change.variant_name == "Empty")
            .expect("removed variant should be present");
        assert_eq!(removed.kind, SchemaVariantChangeKind::Removed);

        let payload_changed = report
            .variant_changes
            .iter()
            .find(|change| change.variant_name == "Data")
            .expect("payload-changed variant should be present");
        assert_eq!(
            payload_changed.kind,
            SchemaVariantChangeKind::PayloadChanged
        );
        assert_eq!(payload_changed.field_changes.len(), 2);
        assert_eq!(
            payload_changed.field_changes[0].kind,
            SchemaFieldChangeKind::TypeChanged
        );
        assert_eq!(
            payload_changed.field_changes[1].kind,
            SchemaFieldChangeKind::Removed
        );
    }

    #[test]
    fn classify_tagged_union_schema_compatibility_rejects_record_schemas_in_union_slice() {
        let previous = r#"
api schema Telemetry version(1) {
    enabled: bool,
}

fn main() {
    return;
}
"#;
        let next = r#"
api schema Telemetry version(2) {
    enabled: bool,
    interval_ms: u32[ms],
}

fn main() {
    return;
}
"#;

        let err = classify_tagged_union_schema_compatibility(previous, next, "Telemetry")
            .expect_err("record-shaped schemas are deferred in tagged-union slice");
        assert!(err
            .message
            .contains("currently supports only tagged-union schemas"));
    }
}
