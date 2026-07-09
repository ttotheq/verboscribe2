mod app_service;
mod commands;
mod hotkeys;

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Runtime,
};

pub use app_service::{AppService, AppStatusDto, DictationStatusDto, RuntimeEventDto, SettingsDto};

const TRAY_SHOW_ID: &str = "tray-show";
const TRAY_QUIT_ID: &str = "tray-quit";
const TRAY_TOOLTIP: &str = "VerboScribe 2";
const TRAY_ID: &str = "main";
const TRAY_ICON_BYTES: &[u8] =
    include_bytes!("../icons/concepts/verboscribe2-mark-concept-v1-32.png");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let service = app_service::AppService::default();
    tauri::Builder::default()
        .manage(service.clone())
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Regular);
            install_tray(app)?;
            install_window_close_guard(app);
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
            commands::paste_last_transcript,
            commands::retry_last_failed_transcript,
            commands::settings,
            commands::save_settings,
            commands::register_dictation_hotkey,
            commands::unregister_dictation_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running VerboScribe 2");
}

fn install_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let tray_menu = MenuBuilder::new(app)
        .text(TRAY_SHOW_ID, "Show VerboScribe 2")
        .separator()
        .text(TRAY_QUIT_ID, "Quit VerboScribe 2")
        .build()?;

    let tray_icon = load_tray_icon()?;

    let tray_builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&tray_menu)
        .icon(tray_icon)
        .icon_as_template(false)
        .tooltip(TRAY_TOOLTIP)
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
        });

    let _tray = tray_builder.build(app)?;
    sync_window_icon(app);
    Ok(())
}

fn install_window_close_guard<R: Runtime>(app: &App<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let window_handle = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window_handle.hide();
            }
        });
    }
}

fn load_tray_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(TRAY_ICON_BYTES).map(Image::to_owned)
}

fn sync_window_icon<R: Runtime>(app: &App<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Some(icon) = app.default_window_icon().cloned() {
            let _ = window.set_icon(icon);
        }
    }
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "macos")]
    let _ = app.show();
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
