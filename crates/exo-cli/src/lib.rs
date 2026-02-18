#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use exo_core::diagnostics::diagnostic_catalog;
#[cfg(feature = "std")]
use exo_emit::compile_program_to_exobyte_with_options_debug;
#[cfg(feature = "std")]
use exo_ir::{
    compile_program_to_ir_with_options, CompileProfile, OptLevel,
};
#[cfg(feature = "std")]
use exo_semantics::{check_file_with_provider, check_source, ModuleProvider, SemanticReport};
#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
pub struct CliPipeline;

#[cfg(feature = "std")]
struct CliFsProvider;

#[cfg(feature = "std")]
impl ModuleProvider for CliFsProvider {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
        std::fs::read(module_id).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "std")]
impl CliPipeline {
    pub fn compile_source(
        src: &str,
        profile: CompileProfile,
        opt: OptLevel,
        debug_symbols: bool,
    ) -> Result<Vec<u8>, String> {
        compile_program_to_exobyte_with_options_debug(src, profile, opt, debug_symbols)
            .map_err(|e| e.to_string())
    }

    pub fn build_ir(
        src: &str,
        profile: CompileProfile,
        opt: OptLevel,
    ) -> Result<Vec<exo_ir::IrFunction>, String> {
        compile_program_to_ir_with_options(src, profile, opt).map_err(|e| e.to_string())
    }

    pub fn semantic_check_source(src: &str) -> Result<SemanticReport, String> {
        check_source(src).map_err(|e| e.to_string())
    }

    pub fn semantic_check_file(path: &Path) -> Result<SemanticReport, String> {
        let provider = CliFsProvider;
        let root = path
            .canonicalize()
            .map_err(|e| format!("failed to resolve '{}': {}", path.display(), e))?;
        check_file_with_provider(&root, &provider).map_err(|e| e.to_string())
    }

    pub fn explain(code: &str) -> Option<&'static str> {
        let upper = code.trim().to_ascii_uppercase();
        diagnostic_catalog()
            .iter()
            .find(|(c, _)| *c == upper)
            .map(|(_, msg)| *msg)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn compile_pipeline_smoke() {
        let src = "fn main() { return; }";
        let bytes = CliPipeline::compile_source(src, CompileProfile::RustLike, OptLevel::O0, false)
            .expect("compile");
        assert_eq!(&bytes[0..8], b"EXOBYTE0");
    }

    #[test]
    fn semantic_pipeline_smoke() {
        let src = r#"
Law "L" [priority 1]:
    When true ->
        System.recovery()
"#;
        let rep = CliPipeline::semantic_check_source(src).expect("check");
        assert_eq!(rep.scheduled_laws.len(), 1);
    }

    #[test]
    fn explain_smoke() {
        let text = CliPipeline::explain("E0101").expect("known code");
        assert!(text.contains("indent"));
    }
}
