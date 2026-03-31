use std::fmt::Write;

pub const GENERATED_WIRE_CONTRACT_FORMAT_VERSION: u32 = 1;
pub const GENERATED_WIRE_CONTRACT_GENERATOR: &str = env!("CARGO_PKG_NAME");
pub const GENERATED_WIRE_CONTRACT_GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedWireUnionField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedWireUnionVariant {
    pub name: String,
    pub fields: Vec<TaggedWireUnionField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedWireUnionContract {
    pub schema_name: String,
    pub variants: Vec<TaggedWireUnionVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirePatchField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirePatchTypeContract {
    pub schema_name: String,
    pub fields: Vec<WirePatchField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedWireContractArtifact {
    pub format_version: u32,
    pub generator_name: String,
    pub generator_version: String,
    pub tagged_unions: Vec<TaggedWireUnionContract>,
    pub patch_types: Vec<WirePatchTypeContract>,
}

impl GeneratedWireContractArtifact {
    pub fn new(
        tagged_unions: Vec<TaggedWireUnionContract>,
        patch_types: Vec<WirePatchTypeContract>,
    ) -> Self {
        Self {
            format_version: GENERATED_WIRE_CONTRACT_FORMAT_VERSION,
            generator_name: GENERATED_WIRE_CONTRACT_GENERATOR.to_string(),
            generator_version: GENERATED_WIRE_CONTRACT_GENERATOR_VERSION.to_string(),
            tagged_unions,
            patch_types,
        }
    }
}

pub fn format_generated_wire_contract(artifact: &GeneratedWireContractArtifact) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "semantic_wire_contract v{}", artifact.format_version);
    let _ = writeln!(
        out,
        "generator {} {}",
        artifact.generator_name, artifact.generator_version
    );

    for tagged_union in &artifact.tagged_unions {
        out.push('\n');
        let _ = writeln!(out, "wire union {} {{", tagged_union.schema_name);
        for variant in &tagged_union.variants {
            let _ = writeln!(out, "    {} {{", variant.name);
            for field in &variant.fields {
                let _ = writeln!(out, "        {}: {}", field.name, field.ty);
            }
            out.push_str("    }\n");
        }
        out.push_str("}\n");
    }

    for patch_type in &artifact.patch_types {
        out.push('\n');
        let _ = writeln!(out, "wire patch {} {{", patch_type.schema_name);
        for field in &patch_type.fields {
            let _ = writeln!(out, "    {}?: {}", field.name, field.ty);
        }
        out.push_str("}\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_wire_contract_artifact_uses_canonical_metadata() {
        let artifact = GeneratedWireContractArtifact::new(Vec::new(), Vec::new());

        assert_eq!(
            artifact.format_version,
            GENERATED_WIRE_CONTRACT_FORMAT_VERSION
        );
        assert_eq!(
            artifact.generator_name,
            GENERATED_WIRE_CONTRACT_GENERATOR
        );
        assert_eq!(
            artifact.generator_version,
            GENERATED_WIRE_CONTRACT_GENERATOR_VERSION
        );
    }

    #[test]
    fn format_generated_wire_contract_preserves_contract_order() {
        let artifact = GeneratedWireContractArtifact::new(
            vec![TaggedWireUnionContract {
                schema_name: "Envelope".to_string(),
                variants: vec![
                    TaggedWireUnionVariant {
                        name: "Empty".to_string(),
                        fields: Vec::new(),
                    },
                    TaggedWireUnionVariant {
                        name: "Data".to_string(),
                        fields: vec![TaggedWireUnionField {
                            name: "count".to_string(),
                            ty: "i32".to_string(),
                        }],
                    },
                ],
            }],
            vec![WirePatchTypeContract {
                schema_name: "Telemetry".to_string(),
                fields: vec![
                    WirePatchField {
                        name: "enabled".to_string(),
                        ty: "bool".to_string(),
                    },
                    WirePatchField {
                        name: "interval_ms".to_string(),
                        ty: "u32[ms]".to_string(),
                    },
                ],
            }],
        );

        let formatted = format_generated_wire_contract(&artifact);
        let expected = "\
semantic_wire_contract v1
generator smc-cli 0.1.0

wire union Envelope {
    Empty {
    }
    Data {
        count: i32
    }
}

wire patch Telemetry {
    enabled?: bool
    interval_ms?: u32[ms]
}
";

        assert_eq!(formatted, expected);
    }
}
