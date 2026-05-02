use tauri::State;

use crate::app_service::{AppService, AppStatusDto, DictationStatusDto};

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
