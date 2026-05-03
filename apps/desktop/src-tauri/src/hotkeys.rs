use tauri::{App, AppHandle, Runtime};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::app_service::{AppService, HotkeyEventState};

pub fn install<R: Runtime>(
    app: &mut App<R>,
    service: AppService,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        let handler_service = service.clone();
        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, _shortcut, event| match event.state() {
                    ShortcutState::Pressed => {
                        let _ = handler_service.handle_hotkey_event(HotkeyEventState::Pressed);
                    }
                    ShortcutState::Released => {
                        let _ = handler_service.handle_hotkey_event(HotkeyEventState::Released);
                    }
                })
                .build(),
        )?;
        let _ = register_from_settings(&app.handle(), &service);
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
        let configured_shortcut = settings.hotkey;
        let accelerator = normalize_hotkey_accelerator(&configured_shortcut)?;

        if let Some(active_accelerator) = service.active_hotkey_accelerator() {
            if active_accelerator == accelerator {
                service.set_hotkey_registered(configured_shortcut, accelerator);
                return Ok(());
            }

            app.global_shortcut()
                .unregister(active_accelerator.as_str())
                .map_err(|error| error.to_string())?;
        }

        app.global_shortcut()
            .register(accelerator.as_str())
            .map_err(|error| {
                let error = error.to_string();
                service.set_hotkey_registration_failed(configured_shortcut.clone(), error.clone());
                error
            })?;

        service.set_hotkey_registered(configured_shortcut, accelerator);
        Ok(())
    }
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
        let configured_shortcut = service
            .settings()
            .map(|settings| settings.hotkey)
            .unwrap_or_else(|_| "Unknown hotkey".to_string());

        if let Some(active_accelerator) = service.active_hotkey_accelerator() {
            app.global_shortcut()
                .unregister(active_accelerator.as_str())
                .map_err(|error| error.to_string())?;
        }

        service.clear_hotkey_registration(configured_shortcut);
        Ok(())
    }
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
    fn normalize_reports_unsupported_modifier() {
        let error = normalize_hotkey_accelerator("Hyper+Space").unwrap_err();

        assert!(error.contains("unsupported hotkey modifier"));
    }
}
