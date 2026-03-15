mod adapter;
mod docs;
mod snapshot;

use adapter::{
  adapter_contract,
  execute_job,
  resolve_workspace,
  AdapterContract,
  JobRequest,
  JobResult,
  WorkspaceSummary,
};
use snapshot::{read_overview_snapshot, OverviewSnapshot};
use docs::{read_spec_catalog, read_spec_document, SpecCatalogSection, SpecDocumentView};

#[tauri::command]
fn get_adapter_contract() -> Result<AdapterContract, String> {
  adapter_contract()
}

#[tauri::command]
async fn run_cli_job(request: JobRequest) -> Result<JobResult, String> {
  tauri::async_runtime::spawn_blocking(move || execute_job(request))
    .await
    .map_err(|error| format!("command task failed: {error}"))?
}

#[tauri::command]
fn resolve_workspace_root(candidate: Option<String>) -> Result<WorkspaceSummary, String> {
  resolve_workspace(candidate)
}

#[tauri::command]
fn get_overview_snapshot() -> Result<OverviewSnapshot, String> {
  read_overview_snapshot()
}

#[tauri::command]
fn get_spec_catalog() -> Result<Vec<SpecCatalogSection>, String> {
  read_spec_catalog()
}

#[tauri::command]
fn get_spec_document(relative_path: String) -> Result<SpecDocumentView, String> {
  read_spec_document(relative_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_adapter_contract,
      run_cli_job,
      resolve_workspace_root,
      get_overview_snapshot,
      get_spec_catalog,
      get_spec_document
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
