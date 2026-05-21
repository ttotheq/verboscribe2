mod app_service;
mod commands;
mod hotkeys;

use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};

pub use app_service::{AppService, AppStatusDto, DictationStatusDto, RuntimeEventDto, SettingsDto};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let service = app_service::AppService::default();
    tauri::Builder::default()
        .manage(service.clone())
        .setup(move |app| {
            install_tray(app)?;
            hotkeys::install(app, service.clone())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::dry_run_dictation_state,
            commands::runtime_status,
            commands::dry_run_dictation_events,
            commands::start_dictation,
            commands::stop_dictation,
            commands::cancel_dictation,
            commands::settings,
            commands::save_settings,
            commands::register_dictation_hotkey,
            commands::unregister_dictation_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running VerboScribe 2");
}

const TRAY_SHOW_ID: &str = "tray-show";
const TRAY_QUIT_ID: &str = "tray-quit";
const TRAY_TOOLTIP: &str = "VerboScribe 2";
const TRAY_ICON_BYTES: &[u8] =
    include_bytes!("../icons/concepts/verboscribe2-mark-concept-v1-32.png");

fn install_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let tray_menu = MenuBuilder::new(app)
        .text(TRAY_SHOW_ID, "Show VerboScribe 2")
        .separator()
        .text(TRAY_QUIT_ID, "Quit VerboScribe 2")
        .build()?;

    let tray_icon = load_tray_icon()?;

    TrayIconBuilder::with_id("main")
        .menu(&tray_menu)
        .icon(tray_icon)
        .tooltip(TRAY_TOOLTIP)
        .icon_as_template(false)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_SHOW_ID => show_main_window(app),
            TRAY_QUIT_ID => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    if let Some(window) = app.get_webview_window("main") {
        if let Some(icon) = app.default_window_icon().cloned() {
            let _ = window.set_icon(icon);
        }
    }

    Ok(())
}

fn load_tray_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(TRAY_ICON_BYTES).map(Image::to_owned)
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.show();
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
