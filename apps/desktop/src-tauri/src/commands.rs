use tauri::{AppHandle, State};

use crate::app_service::{
    AppService, AppStatusDto, DictationStatusDto, RuntimeEventDto, SettingsDto,
};
use crate::hotkeys;

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
pub fn start_dictation(service: State<'_, AppService>) -> Result<DictationStatusDto, String> {
    service.start_dictation()
}

#[tauri::command]
pub fn stop_dictation(service: State<'_, AppService>) -> Result<DictationStatusDto, String> {
    service.stop_dictation()
}

#[tauri::command]
pub fn cancel_dictation(service: State<'_, AppService>) -> Result<DictationStatusDto, String> {
    service.cancel_dictation()
}

#[tauri::command]
pub fn paste_last_transcript(service: State<'_, AppService>) -> Result<DictationStatusDto, String> {
    service.paste_last_transcript()
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

#[tauri::command]
pub fn register_dictation_hotkey(
    app: AppHandle,
    service: State<'_, AppService>,
) -> Result<(), String> {
    hotkeys::register_from_settings(&app, &service)
}

#[tauri::command]
pub fn unregister_dictation_hotkey(
    app: AppHandle,
    service: State<'_, AppService>,
) -> Result<(), String> {
    hotkeys::unregister_current(&app, &service)
}
