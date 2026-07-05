use tauri::{App, AppHandle, Runtime};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app_service::{AppService, HotkeyEventState, HotkeyRole};

fn debug_hotkeys(message: impl AsRef<str>) {
    if std::env::var_os("VERBOSCRIBE_DEBUG_HOTKEYS").is_some() {
        eprintln!("[verboscribe hotkeys] {}", message.as_ref());
    }
}

pub fn install<R: Runtime>(
    app: &mut App<R>,
    service: AppService,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        let handler_service = service.clone();
        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, shortcut, event| {
                    let Some(role) = resolve_hotkey_role(&handler_service, shortcut) else {
                        debug_hotkeys("ignoring event for unrecognized shortcut");
                        return;
                    };
                    let state = match event.state() {
                        ShortcutState::Pressed => HotkeyEventState::Pressed,
                        ShortcutState::Released => HotkeyEventState::Released,
                    };
                    debug_hotkeys(format!("received {state:?} event for {role:?}"));
                    let result = match role {
                        HotkeyRole::Dictation => handler_service.handle_hotkey_event(state),
                        HotkeyRole::Toggle => handler_service.handle_toggle_hotkey_event(state),
                        HotkeyRole::Cancel => handler_service.handle_cancel_hotkey_event(state),
                        HotkeyRole::PasteLast => {
                            handler_service.handle_paste_last_hotkey_event(state)
                        }
                        HotkeyRole::RetryLastFailed => {
                            handler_service.handle_retry_last_failed_hotkey_event(state)
                        }
                    };
                    if let Err(error) = result {
                        debug_hotkeys(format!("event failed: {error}"));
                    }
                })
                .build(),
        )?;
        debug_hotkeys("plugin installed");
        if let Err(error) = register_from_settings(&app.handle(), &service) {
            debug_hotkeys(format!("initial registration failed: {error}"));
        }
    }

    Ok(())
}

pub fn register_from_settings<R: Runtime>(
    app: &AppHandle<R>,
    service: &AppService,
) -> Result<(), String> {
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (app, service);
        return Err("global shortcuts are not supported on this platform".to_string());
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        let settings = service.settings()?;
        // Register both hotkeys even if one fails, so a conflict on one does not
        // suppress the other. Per-role failures are recorded on the service and
        // surfaced through app status; the first error is returned for the caller.
        let dictation = register_role(app, service, HotkeyRole::Dictation, settings.hotkey);
        let toggle = register_role(app, service, HotkeyRole::Toggle, settings.toggle_hotkey);
        let cancel = register_role(app, service, HotkeyRole::Cancel, settings.cancel_hotkey);
        let paste_last = register_role(
            app,
            service,
            HotkeyRole::PasteLast,
            settings.paste_last_hotkey,
        );
        let retry_last_failed = register_role(
            app,
            service,
            HotkeyRole::RetryLastFailed,
            settings.retry_last_failed_hotkey,
        );
        dictation
            .and(toggle)
            .and(cancel)
            .and(paste_last)
            .and(retry_last_failed)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn register_role<R: Runtime>(
    app: &AppHandle<R>,
    service: &AppService,
    role: HotkeyRole,
    configured_shortcut: String,
) -> Result<(), String> {
    let accelerator = normalize_hotkey_accelerator(&configured_shortcut)?;
    debug_hotkeys(format!(
        "register requested: {role:?} configured='{configured_shortcut}' accelerator='{accelerator}'"
    ));

    if let Some(active_accelerator) = service.active_hotkey_accelerator(role) {
        if active_accelerator == accelerator {
            service.set_hotkey_registered(role, configured_shortcut, accelerator);
            debug_hotkeys("register skipped: accelerator already active");
            return Ok(());
        }

        app.global_shortcut()
            .unregister(active_accelerator.as_str())
            .map_err(|error| error.to_string())?;
        debug_hotkeys(format!(
            "unregistered previous accelerator '{active_accelerator}'"
        ));
    }

    app.global_shortcut()
        .register(accelerator.as_str())
        .map_err(|error| {
            let error = error.to_string();
            service.set_hotkey_registration_failed(
                role,
                configured_shortcut.clone(),
                error.clone(),
            );
            error
        })?;

    service.set_hotkey_registered(role, configured_shortcut, accelerator);
    debug_hotkeys("register succeeded");
    Ok(())
}

/// Match a fired global shortcut back to the role it was registered under by
/// comparing it against each role's currently active accelerator.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn resolve_hotkey_role(service: &AppService, shortcut: &Shortcut) -> Option<HotkeyRole> {
    for role in [
        HotkeyRole::Dictation,
        HotkeyRole::Toggle,
        HotkeyRole::Cancel,
        HotkeyRole::PasteLast,
        HotkeyRole::RetryLastFailed,
    ] {
        if let Some(accelerator) = service.active_hotkey_accelerator(role) {
            if let Ok(parsed) = accelerator.parse::<Shortcut>() {
                if &parsed == shortcut {
                    return Some(role);
                }
            }
        }
    }
    None
}

pub fn unregister_current<R: Runtime>(
    app: &AppHandle<R>,
    service: &AppService,
) -> Result<(), String> {
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (app, service);
        return Ok(());
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        let settings = service.settings().ok();
        let dictation_shortcut = settings
            .as_ref()
            .map(|settings| settings.hotkey.clone())
            .unwrap_or_else(|| "Unknown hotkey".to_string());
        let toggle_shortcut = settings
            .as_ref()
            .map(|settings| settings.toggle_hotkey.clone())
            .unwrap_or_else(|| "Unknown hotkey".to_string());

        let cancel_shortcut = settings
            .as_ref()
            .map(|settings| settings.cancel_hotkey.clone())
            .unwrap_or_else(|| "Unknown hotkey".to_string());

        let paste_last_shortcut = settings
            .as_ref()
            .map(|settings| settings.paste_last_hotkey.clone())
            .unwrap_or_else(|| "Unknown hotkey".to_string());

        let retry_last_failed_shortcut = settings
            .as_ref()
            .map(|settings| settings.retry_last_failed_hotkey.clone())
            .unwrap_or_else(|| "Unknown hotkey".to_string());

        let dictation = unregister_role(app, service, HotkeyRole::Dictation, dictation_shortcut);
        let toggle = unregister_role(app, service, HotkeyRole::Toggle, toggle_shortcut);
        let cancel = unregister_role(app, service, HotkeyRole::Cancel, cancel_shortcut);
        let paste_last = unregister_role(app, service, HotkeyRole::PasteLast, paste_last_shortcut);
        let retry_last_failed = unregister_role(
            app,
            service,
            HotkeyRole::RetryLastFailed,
            retry_last_failed_shortcut,
        );
        dictation
            .and(toggle)
            .and(cancel)
            .and(paste_last)
            .and(retry_last_failed)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn unregister_role<R: Runtime>(
    app: &AppHandle<R>,
    service: &AppService,
    role: HotkeyRole,
    configured_shortcut: String,
) -> Result<(), String> {
    if let Some(active_accelerator) = service.active_hotkey_accelerator(role) {
        app.global_shortcut()
            .unregister(active_accelerator.as_str())
            .map_err(|error| error.to_string())?;
        debug_hotkeys(format!(
            "unregister succeeded: {role:?} '{active_accelerator}'"
        ));
    }

    service.clear_hotkey_registration(role, configured_shortcut);
    Ok(())
}

fn normalize_hotkey_accelerator(configured_shortcut: &str) -> Result<String, String> {
    let mut tokens = configured_shortcut
        .split('+')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    let key = tokens
        .pop()
        .ok_or_else(|| "hotkey must include at least one key".to_string())?;
    let modifiers = tokens
        .into_iter()
        .map(normalize_modifier)
        .collect::<Result<Vec<_>, _>>()?;
    let key = normalize_key(key)?;

    let mut accelerator = modifiers.join("+");
    if !accelerator.is_empty() {
        accelerator.push('+');
    }
    accelerator.push_str(&key);
    Ok(accelerator)
}

fn normalize_modifier(token: &str) -> Result<String, String> {
    match token.to_ascii_lowercase().as_str() {
        "control" | "ctrl" => Ok("ctrl".to_string()),
        "option" | "alt" => Ok("alt".to_string()),
        "shift" => Ok("shift".to_string()),
        "command" | "cmd" | "super" | "meta" => Ok("super".to_string()),
        "commandorcontrol" | "cmdorcontrol" | "cmdorctrl" => {
            if cfg!(target_os = "macos") {
                Ok("super".to_string())
            } else {
                Ok("ctrl".to_string())
            }
        }
        unsupported => Err(format!("unsupported hotkey modifier: {unsupported}")),
    }
}

fn normalize_key(token: &str) -> Result<String, String> {
    let token = token.trim();
    if token.len() == 1 {
        return Ok(token.to_ascii_lowercase());
    }

    match token.to_ascii_lowercase().as_str() {
        "space" | "spacebar" => Ok("space".to_string()),
        "enter" | "return" => Ok("enter".to_string()),
        "tab" => Ok("tab".to_string()),
        "escape" | "esc" => Ok("esc".to_string()),
        "backspace" => Ok("backspace".to_string()),
        key if key.starts_with('f') && key[1..].chars().all(|ch| ch.is_ascii_digit()) => {
            Ok(key.to_string())
        }
        unsupported => Err(format!("unsupported hotkey key: {unsupported}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_default_hotkey_for_plugin_registration() {
        assert_eq!(
            normalize_hotkey_accelerator("Control+Option+Space").unwrap(),
            "ctrl+alt+space"
        );
    }

    #[test]
    fn normalize_shift_letter_hotkey() {
        assert_eq!(
            normalize_hotkey_accelerator("Control+Shift+D").unwrap(),
            "ctrl+shift+d"
        );
    }

    #[test]
    fn normalize_escape_hotkey_for_plugin_registration() {
        assert_eq!(
            normalize_hotkey_accelerator("Control+Option+Escape").unwrap(),
            "ctrl+alt+esc"
        );
    }

    #[test]
    fn normalize_reports_unsupported_modifier() {
        let error = normalize_hotkey_accelerator("Hyper+Space").unwrap_err();

        assert!(error.contains("unsupported hotkey modifier"));
    }
}
