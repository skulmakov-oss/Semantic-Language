mod adapter;

use adapter::{adapter_contract, execute_job, AdapterContract, JobRequest, JobResult};

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
      run_cli_job
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
