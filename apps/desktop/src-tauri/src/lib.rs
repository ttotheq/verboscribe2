mod app_service;
mod commands;
mod hotkeys;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let service = app_service::AppService::default();
    tauri::Builder::default()
        .manage(service.clone())
        .setup(move |app| hotkeys::install(app, service.clone()))
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::dry_run_dictation_state,
            commands::runtime_status,
            commands::dry_run_dictation_events,
            commands::settings,
            commands::save_settings,
            commands::register_dictation_hotkey,
            commands::unregister_dictation_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running VerboScribe 2");
}
