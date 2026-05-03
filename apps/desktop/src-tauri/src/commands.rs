use tauri::State;

use crate::app_service::{
    AppService, AppStatusDto, DictationStatusDto, RuntimeEventDto, SettingsDto,
};

#[tauri::command]
pub fn app_status(service: State<'_, AppService>) -> AppStatusDto {
    service.app_status()
}

#[tauri::command]
pub fn dry_run_dictation_state(
    service: State<'_, AppService>,
) -> Result<DictationStatusDto, String> {
    service.dry_run_dictation_state()
}

#[tauri::command]
pub fn runtime_status(service: State<'_, AppService>) -> RuntimeEventDto {
    service.runtime_status()
}

#[tauri::command]
pub fn dry_run_dictation_events(
    service: State<'_, AppService>,
) -> Result<Vec<RuntimeEventDto>, String> {
    service.dry_run_dictation_events()
}

#[tauri::command]
pub fn settings(service: State<'_, AppService>) -> Result<SettingsDto, String> {
    service.settings()
}

#[tauri::command]
pub fn save_settings(
    service: State<'_, AppService>,
    settings: SettingsDto,
) -> Result<SettingsDto, String> {
    service.save_settings(settings)
}
