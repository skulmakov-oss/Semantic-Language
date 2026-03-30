use std::fmt::Write;

pub const GENERATED_API_CONTRACT_FORMAT_VERSION: u32 = 1;
pub const GENERATED_API_CONTRACT_GENERATOR: &str = env!("CARGO_PKG_NAME");
pub const GENERATED_API_CONTRACT_GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedApiSchemaRole {
    Api,
    Wire,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiVariant {
    pub name: String,
    pub fields: Vec<GeneratedApiField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratedApiSchemaShape {
    Record(Vec<GeneratedApiField>),
    TaggedUnion(Vec<GeneratedApiVariant>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiSchema {
    pub name: String,
    pub role: GeneratedApiSchemaRole,
    pub shape: GeneratedApiSchemaShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiContractArtifact {
    pub format_version: u32,
    pub generator_name: String,
    pub generator_version: String,
    pub schemas: Vec<GeneratedApiSchema>,
}

impl GeneratedApiContractArtifact {
    pub fn new(schemas: Vec<GeneratedApiSchema>) -> Self {
        Self {
            format_version: GENERATED_API_CONTRACT_FORMAT_VERSION,
            generator_name: GENERATED_API_CONTRACT_GENERATOR.to_string(),
            generator_version: GENERATED_API_CONTRACT_GENERATOR_VERSION.to_string(),
            schemas,
        }
    }
}

pub fn format_generated_api_contract(artifact: &GeneratedApiContractArtifact) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "semantic_api_contract v{}",
        artifact.format_version
    );
    let _ = writeln!(
        out,
        "generator {} {}",
        artifact.generator_name, artifact.generator_version
    );

    for schema in &artifact.schemas {
        out.push('\n');
        let _ = write!(
            out,
            "{} schema {} ",
            display_generated_api_role(schema.role),
            schema.name
        );
        match &schema.shape {
            GeneratedApiSchemaShape::Record(fields) => {
                out.push_str("{\n");
                for field in fields {
                    let _ = writeln!(out, "    {}: {}", field.name, field.ty);
                }
                out.push_str("}\n");
            }
            GeneratedApiSchemaShape::TaggedUnion(variants) => {
                out.push_str("{\n");
                for variant in variants {
                    let _ = writeln!(out, "    {} {{", variant.name);
                    for field in &variant.fields {
                        let _ = writeln!(out, "        {}: {}", field.name, field.ty);
                    }
                    out.push_str("    }\n");
                }
                out.push_str("}\n");
            }
        }
    }

    out
}

fn display_generated_api_role(role: GeneratedApiSchemaRole) -> &'static str {
    match role {
        GeneratedApiSchemaRole::Api => "api",
        GeneratedApiSchemaRole::Wire => "wire",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_api_contract_artifact_uses_canonical_metadata() {
        let artifact = GeneratedApiContractArtifact::new(Vec::new());

        assert_eq!(artifact.format_version, GENERATED_API_CONTRACT_FORMAT_VERSION);
        assert_eq!(artifact.generator_name, GENERATED_API_CONTRACT_GENERATOR);
        assert_eq!(
            artifact.generator_version,
            GENERATED_API_CONTRACT_GENERATOR_VERSION
        );
    }

    #[test]
    fn format_generated_api_contract_preserves_schema_and_field_order() {
        let artifact = GeneratedApiContractArtifact::new(vec![
            GeneratedApiSchema {
                name: "Telemetry".to_string(),
                role: GeneratedApiSchemaRole::Api,
                shape: GeneratedApiSchemaShape::Record(vec![
                    GeneratedApiField {
                        name: "enabled".to_string(),
                        ty: "bool".to_string(),
                    },
                    GeneratedApiField {
                        name: "interval_ms".to_string(),
                        ty: "u32[ms]".to_string(),
                    },
                ]),
            },
            GeneratedApiSchema {
                name: "Envelope".to_string(),
                role: GeneratedApiSchemaRole::Wire,
                shape: GeneratedApiSchemaShape::TaggedUnion(vec![
                    GeneratedApiVariant {
                        name: "Empty".to_string(),
                        fields: Vec::new(),
                    },
                    GeneratedApiVariant {
                        name: "Data".to_string(),
                        fields: vec![GeneratedApiField {
                            name: "sample_count".to_string(),
                            ty: "i32".to_string(),
                        }],
                    },
                ]),
            },
        ]);

        let formatted = format_generated_api_contract(&artifact);
        let expected = "\
semantic_api_contract v1
generator smc-cli 0.1.0

api schema Telemetry {
    enabled: bool
    interval_ms: u32[ms]
}

wire schema Envelope {
    Empty {
    }
    Data {
        sample_count: i32
    }
}
";
        assert_eq!(formatted, expected);
    }
}
