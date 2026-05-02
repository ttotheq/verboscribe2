mod app_service;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(app_service::AppService)
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::dry_run_dictation_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running VerboScribe 2");
}
