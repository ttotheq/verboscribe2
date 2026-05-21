//! Platform-specific adapters for macOS and Windows.
//!
//! This crate is the boundary for hotkeys, tray/menu-bar integration, target app
//! tracking, paste insertion, launch-at-login, and secret storage.

use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use verboscribe_core::{DictationError, TargetApp, TargetAppTracker, TextInsertionService};

const APP_IDENTIFIER: &str = "local.verboscribe2";
const APP_NAME: &str = "VerboScribe 2";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const ACTIVATE_SETTLE_DELAY: Duration = Duration::from_millis(120);
const MACOS_DIRECT_PASTE_PROGRAM: &str = "__verboscribe_macos_direct_paste__";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Macos,
    Windows,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCommand {
    pub program: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub success: bool,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner: Clone {
    fn run(&self, command: &PlatformCommand, timeout: Duration) -> Result<CommandOutput, String>;
}

#[derive(Debug, Clone, Default)]
pub struct StdCommandRunner;

impl CommandRunner for StdCommandRunner {
    fn run(&self, command: &PlatformCommand, timeout: Duration) -> Result<CommandOutput, String> {
        if let Some(output) = run_builtin_command(command)? {
            return Ok(output);
        }

        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(if command.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not launch {}: {error}", command.program))?;

        if let Some(input) = &command.stdin {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_bytes()).map_err(|error| {
                    format!("could not write stdin for {}: {error}", command.program)
                })?;
            }
        }

        let started_at = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let output = child.wait_with_output().map_err(|error| {
                        format!("could not wait for {}: {error}", command.program)
                    })?;
                    return Ok(CommandOutput {
                        success: output.status.success(),
                        status: output.status.to_string(),
                        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                    });
                }
                Ok(None) if started_at.elapsed() >= timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "command timed out after {}s: {}",
                        timeout.as_secs(),
                        command.program
                    ));
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("could not poll {}: {error}", command.program));
                }
            }
        }
    }
}

fn run_builtin_command(command: &PlatformCommand) -> Result<Option<CommandOutput>, String> {
    if command.program != MACOS_DIRECT_PASTE_PROGRAM {
        return Ok(None);
    }

    run_macos_direct_paste().map(|_| {
        Some(CommandOutput {
            success: true,
            status: "builtin".to_string(),
            stdout: String::new(),
            stderr: String::new(),
        })
    })
}

#[derive(Debug, Clone)]
pub struct DesktopTargetTracker<R = StdCommandRunner> {
    platform: PlatformKind,
    runner: R,
    last_target: Option<TargetApp>,
}

impl Default for DesktopTargetTracker<StdCommandRunner> {
    fn default() -> Self {
        Self::new_current()
    }
}

impl DesktopTargetTracker<StdCommandRunner> {
    pub fn new_current() -> Self {
        Self::new(current_platform(), StdCommandRunner)
    }
}

impl<R: CommandRunner> DesktopTargetTracker<R> {
    pub fn new(platform: PlatformKind, runner: R) -> Self {
        Self {
            platform,
            runner,
            last_target: None,
        }
    }
}

impl<R: CommandRunner> TargetAppTracker for DesktopTargetTracker<R> {
    fn capture_target(&mut self) -> Option<TargetApp> {
        let Some(command) = frontmost_target_command(self.platform) else {
            return self.last_target.clone();
        };

        let output = match self.runner.run(&command, COMMAND_TIMEOUT) {
            Ok(output) if output.success => output,
            _ => return self.last_target.clone(),
        };
        let target = match parse_frontmost_target_output(self.platform, &output.stdout) {
            Some(target) => target,
            None => return self.last_target.clone(),
        };

        if is_self_target(&target) {
            return self.last_target.clone();
        }

        self.last_target = Some(target.clone());
        Some(target)
    }
}

#[derive(Debug, Clone)]
pub struct DesktopTextInserter<R = StdCommandRunner> {
    platform: PlatformKind,
    runner: R,
}

impl Default for DesktopTextInserter<StdCommandRunner> {
    fn default() -> Self {
        Self::new_current()
    }
}

impl DesktopTextInserter<StdCommandRunner> {
    pub fn new_current() -> Self {
        Self::new(current_platform(), StdCommandRunner)
    }
}

impl<R: CommandRunner> DesktopTextInserter<R> {
    pub fn new(platform: PlatformKind, runner: R) -> Self {
        Self { platform, runner }
    }

    fn run_checked(&self, command: PlatformCommand, label: &str) -> Result<(), DictationError> {
        let output = self
            .runner
            .run(&command, COMMAND_TIMEOUT)
            .map_err(|error| DictationError::Paste(format!("{label}: {error}")))?;

        if output.success {
            Ok(())
        } else {
            Err(DictationError::Paste(format!(
                "{label}: {}",
                output_detail(&output)
            )))
        }
    }
}

impl<R: CommandRunner> TextInsertionService for DesktopTextInserter<R> {
    fn insert(&mut self, text: &str, target: Option<&TargetApp>) -> Result<(), DictationError> {
        let clipboard_command = clipboard_write_command(self.platform, text)?;
        self.run_checked(clipboard_command, "clipboard write failed")?;

        let target = target.ok_or_else(|| {
            DictationError::Paste("target app was not captured before recording".to_string())
        })?;

        if let Some(command) = activate_target_command(self.platform, target)? {
            self.run_checked(command, "target activation failed")?;
            thread::sleep(ACTIVATE_SETTLE_DELAY);
        }

        let paste_command = paste_command(self.platform, target)?;
        self.run_checked(paste_command, "paste shortcut failed")
    }
}

fn current_platform() -> PlatformKind {
    if cfg!(target_os = "macos") {
        PlatformKind::Macos
    } else if cfg!(target_os = "windows") {
        PlatformKind::Windows
    } else {
        PlatformKind::Other
    }
}

fn output_detail(output: &CommandOutput) -> String {
    if !output.stderr.is_empty() {
        output.stderr.clone()
    } else if !output.stdout.is_empty() {
        output.stdout.clone()
    } else {
        format!("command exited with status {}", output.status)
    }
}

fn clipboard_write_command(
    platform: PlatformKind,
    text: &str,
) -> Result<PlatformCommand, DictationError> {
    match platform {
        PlatformKind::Macos => Ok(PlatformCommand {
            program: "pbcopy".to_string(),
            args: Vec::new(),
            stdin: Some(text.to_string()),
        }),
        PlatformKind::Windows => Ok(PlatformCommand {
            program: "cmd".to_string(),
            args: vec!["/C".to_string(), "clip".to_string()],
            stdin: Some(text.to_string()),
        }),
        PlatformKind::Other => Err(DictationError::Paste(
            "clipboard write is not implemented on this platform".to_string(),
        )),
    }
}

fn frontmost_target_command(platform: PlatformKind) -> Option<PlatformCommand> {
    match platform {
        PlatformKind::Macos => Some(PlatformCommand {
            program: "/bin/sh".to_string(),
            args: vec![
                "-c".to_string(),
                "lsappinfo info -only bundleID,LSDisplayName $(lsappinfo front | awk '{print $1}')"
                    .to_string(),
            ],
            stdin: None,
        }),
        PlatformKind::Windows => Some(PlatformCommand {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                windows_target_capture_script(),
            ],
            stdin: None,
        }),
        PlatformKind::Other => None,
    }
}

fn parse_frontmost_target_output(platform: PlatformKind, output: &str) -> Option<TargetApp> {
    match platform {
        PlatformKind::Macos => parse_macos_target_output(output),
        PlatformKind::Windows => parse_windows_target_output(output),
        PlatformKind::Other => None,
    }
}

fn activate_target_command(
    platform: PlatformKind,
    target: &TargetApp,
) -> Result<Option<PlatformCommand>, DictationError> {
    match platform {
        PlatformKind::Macos => {
            let bundle_id = target
                .identifier
                .as_deref()
                .and_then(|identifier| identifier.strip_prefix("macos:bundle:"))
                .ok_or_else(|| {
                    DictationError::Paste("target app bundle identifier is missing".to_string())
                })?;

            Ok(Some(PlatformCommand {
                program: "/usr/bin/open".to_string(),
                args: vec!["-b".to_string(), bundle_id.to_string()],
                stdin: None,
            }))
        }
        PlatformKind::Windows => {
            let hwnd = target
                .identifier
                .as_deref()
                .and_then(|identifier| identifier.strip_prefix("windows:hwnd:"))
                .ok_or_else(|| {
                    DictationError::Paste("target app window handle is missing".to_string())
                })?;

            Ok(Some(PlatformCommand {
                program: "powershell".to_string(),
                args: vec![
                    "-NoProfile".to_string(),
                    "-NonInteractive".to_string(),
                    "-Command".to_string(),
                    windows_activate_script(hwnd),
                ],
                stdin: None,
            }))
        }
        PlatformKind::Other => Err(DictationError::Paste(
            "target app activation is not implemented on this platform".to_string(),
        )),
    }
}

fn paste_command(
    platform: PlatformKind,
    target: &TargetApp,
) -> Result<PlatformCommand, DictationError> {
    let _ = target;

    match platform {
        PlatformKind::Macos => Ok(PlatformCommand {
            program: MACOS_DIRECT_PASTE_PROGRAM.to_string(),
            args: Vec::new(),
            stdin: None,
        }),
        PlatformKind::Windows => Ok(PlatformCommand {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                windows_paste_script(),
            ],
            stdin: None,
        }),
        PlatformKind::Other => Err(DictationError::Paste(
            "paste automation is not implemented on this platform".to_string(),
        )),
    }
}

fn parse_macos_target_output(output: &str) -> Option<TargetApp> {
    let mut bundle_id = None;
    let mut display_name = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("\"CFBundleIdentifier\"=") {
            bundle_id = parse_quoted_value(value);
        } else if let Some(value) = trimmed.strip_prefix("\"LSDisplayName\"=") {
            display_name = parse_quoted_value(value);
        }
    }

    let bundle_id = bundle_id?;
    Some(TargetApp {
        name: display_name,
        identifier: Some(format!("macos:bundle:{bundle_id}")),
    })
}

fn parse_windows_target_output(output: &str) -> Option<TargetApp> {
    let mut hwnd = None;
    let mut process = None;
    let mut title = None;

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("HWND=") {
            let value = value.trim();
            if !value.is_empty() {
                hwnd = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("PROCESS=") {
            let value = value.trim();
            if !value.is_empty() {
                process = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("TITLE=") {
            let value = value.trim();
            if !value.is_empty() {
                title = Some(value.to_string());
            }
        }
    }

    let hwnd = hwnd?;
    let name = title.or(process.clone());

    Some(TargetApp {
        name,
        identifier: Some(format!("windows:hwnd:{hwnd}")),
    })
}

fn parse_quoted_value(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches(';');
    let trimmed = trimmed.strip_prefix('"')?.strip_suffix('"')?;
    Some(trimmed.to_string())
}

fn is_self_target(target: &TargetApp) -> bool {
    target
        .identifier
        .as_deref()
        .is_some_and(|identifier| identifier.contains(APP_IDENTIFIER))
        || target
            .name
            .as_deref()
            .is_some_and(|name| name.eq_ignore_ascii_case(APP_NAME))
}

fn windows_target_capture_script() -> String {
    r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ForegroundWindow {
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int count);
}
"@;
$handle = [ForegroundWindow]::GetForegroundWindow();
if ($handle -eq [IntPtr]::Zero) { exit 1 }
$pid = 0;
[void][ForegroundWindow]::GetWindowThreadProcessId($handle, [ref]$pid);
$process = Get-Process -Id $pid -ErrorAction Stop;
$builder = New-Object System.Text.StringBuilder 1024;
[void][ForegroundWindow]::GetWindowText($handle, $builder, $builder.Capacity);
Write-Output ("HWND=" + $handle.ToInt64());
Write-Output ("PROCESS=" + $process.ProcessName);
Write-Output ("TITLE=" + $builder.ToString().Trim());
"#
    .trim()
    .to_string()
}

fn windows_activate_script(hwnd: &str) -> String {
    format!(
        r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class WindowActivator {{
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
}}
"@;
$handle = [IntPtr]::new({hwnd});
if (-not [WindowActivator]::SetForegroundWindow($handle)) {{
    throw "SetForegroundWindow failed for handle {hwnd}"
}}
"#
    )
    .trim()
    .to_string()
}

fn windows_paste_script() -> String {
    r#"
Add-Type -AssemblyName System.Windows.Forms;
[System.Windows.Forms.SendKeys]::SendWait("^v");
"#
    .trim()
    .to_string()
}

#[cfg(target_os = "macos")]
fn run_macos_direct_paste() -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventFlags = u64;
    type CGEventRef = *mut c_void;
    type CGEventSourceRef = *mut c_void;
    type CGEventSourceStateID = i32;
    type CGEventTapLocation = u32;
    type CGKeyCode = u16;
    type Boolean = u8;

    const KCG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 0x0010_0000;
    const KCG_HID_EVENT_TAP: CGEventTapLocation = 0;
    const KCG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: CGEventSourceStateID = 1;
    const KVK_ANSI_V: CGKeyCode = 9;

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> Boolean;
        fn CGEventSourceCreate(state_id: CGEventSourceStateID) -> CGEventSourceRef;
        fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: CGKeyCode,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventSetFlags(event: CGEventRef, flags: CGEventFlags);
        fn CGEventPost(tap: CGEventTapLocation, event: CGEventRef);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(value: *const c_void);
    }

    struct CfGuard(*mut c_void);

    impl Drop for CfGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CFRelease(self.0.cast_const()) };
            }
        }
    }

    unsafe {
        if AXIsProcessTrusted() == 0 {
            return Err("Accessibility permission is not granted to VerboScribe 2".to_string());
        }

        let source = CfGuard(CGEventSourceCreate(KCG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE));
        if source.0.is_null() {
            return Err("could not create macOS event source".to_string());
        }

        let key_down = CfGuard(CGEventCreateKeyboardEvent(source.0, KVK_ANSI_V, true));
        if key_down.0.is_null() {
            return Err("could not create macOS key-down event".to_string());
        }
        CGEventSetFlags(key_down.0, KCG_EVENT_FLAG_MASK_COMMAND);

        let key_up = CfGuard(CGEventCreateKeyboardEvent(source.0, KVK_ANSI_V, false));
        if key_up.0.is_null() {
            return Err("could not create macOS key-up event".to_string());
        }
        CGEventSetFlags(key_up.0, KCG_EVENT_FLAG_MASK_COMMAND);

        CGEventPost(KCG_HID_EVENT_TAP, key_down.0);
        CGEventPost(KCG_HID_EVENT_TAP, key_up.0);
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn run_macos_direct_paste() -> Result<(), String> {
    Err("direct macOS paste is not available on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct SequenceRunner {
        results: Arc<Mutex<Vec<Result<CommandOutput, String>>>>,
        commands: Arc<Mutex<Vec<PlatformCommand>>>,
    }

    impl SequenceRunner {
        fn with_results(results: Vec<Result<CommandOutput, String>>) -> Self {
            Self {
                results: Arc::new(Mutex::new(results)),
                commands: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn recorded_commands(&self) -> Vec<PlatformCommand> {
            self.commands.lock().unwrap().clone()
        }
    }

    impl CommandRunner for SequenceRunner {
        fn run(
            &self,
            command: &PlatformCommand,
            _timeout: Duration,
        ) -> Result<CommandOutput, String> {
            self.commands.lock().unwrap().push(command.clone());
            self.results.lock().unwrap().remove(0)
        }
    }

    fn successful_output(stdout: &str) -> CommandOutput {
        CommandOutput {
            success: true,
            status: "exit status: 0".to_string(),
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    #[test]
    fn macos_target_output_parses_bundle_and_name() {
        let output =
            "\"CFBundleIdentifier\"=\"com.apple.TextEdit\"\n\"LSDisplayName\"=\"TextEdit\"";

        let target = parse_frontmost_target_output(PlatformKind::Macos, output).unwrap();

        assert_eq!(target.name.as_deref(), Some("TextEdit"));
        assert_eq!(
            target.identifier.as_deref(),
            Some("macos:bundle:com.apple.TextEdit")
        );
    }

    #[test]
    fn windows_target_output_parses_window_handle_and_name() {
        let output = "HWND=12345\nPROCESS=notepad\nTITLE=Untitled - Notepad";

        let target = parse_frontmost_target_output(PlatformKind::Windows, output).unwrap();

        assert_eq!(target.name.as_deref(), Some("Untitled - Notepad"));
        assert_eq!(target.identifier.as_deref(), Some("windows:hwnd:12345"));
    }

    #[test]
    fn tracker_reuses_last_non_self_target_when_self_is_frontmost() {
        let runner = SequenceRunner::with_results(vec![
            Ok(successful_output(
                "\"CFBundleIdentifier\"=\"com.apple.TextEdit\"\n\"LSDisplayName\"=\"TextEdit\"",
            )),
            Ok(successful_output(
                "\"CFBundleIdentifier\"=\"local.verboscribe2\"\n\"LSDisplayName\"=\"VerboScribe 2\"",
            )),
        ]);
        let mut tracker = DesktopTargetTracker::new(PlatformKind::Macos, runner);

        let first = tracker.capture_target();
        let second = tracker.capture_target();

        assert_eq!(first, second);
        assert_eq!(
            second.and_then(|target| target.identifier),
            Some("macos:bundle:com.apple.TextEdit".to_string())
        );
    }

    #[test]
    fn inserter_writes_clipboard_before_reporting_missing_target() {
        let runner = SequenceRunner::with_results(vec![Ok(successful_output(""))]);
        let mut inserter = DesktopTextInserter::new(PlatformKind::Macos, runner.clone());

        let error = inserter.insert("captured transcript", None).unwrap_err();
        let commands = runner.recorded_commands();

        assert!(error
            .to_string()
            .contains("target app was not captured before recording"));
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "pbcopy");
        assert_eq!(commands[0].stdin.as_deref(), Some("captured transcript"));
    }

    #[test]
    fn inserter_leaves_clipboard_step_first_when_paste_fails() {
        let runner = SequenceRunner::with_results(vec![
            Ok(successful_output("")),
            Ok(successful_output("")),
            Ok(CommandOutput {
                success: false,
                status: "exit status: 1".to_string(),
                stdout: String::new(),
                stderr: "access not allowed".to_string(),
            }),
        ]);
        let mut inserter = DesktopTextInserter::new(PlatformKind::Macos, runner.clone());
        let target = TargetApp {
            name: Some("TextEdit".to_string()),
            identifier: Some("macos:bundle:com.apple.TextEdit".to_string()),
        };

        let error = inserter
            .insert("captured transcript", Some(&target))
            .unwrap_err();
        let commands = runner.recorded_commands();

        assert!(error.to_string().contains("paste shortcut failed"));
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].program, "pbcopy");
        assert_eq!(commands[1].program, "/usr/bin/open");
        assert_eq!(commands[2].program, MACOS_DIRECT_PASTE_PROGRAM);
    }

    #[test]
    fn windows_paste_command_uses_send_keys_script() {
        let target = TargetApp {
            name: Some("Notepad".to_string()),
            identifier: Some("windows:hwnd:12345".to_string()),
        };

        let command = paste_command(PlatformKind::Windows, &target).unwrap();

        assert_eq!(command.program, "powershell");
        assert!(command.args.iter().any(|arg| arg.contains("SendWait")));
    }

    #[test]
    fn macos_activate_command_uses_open_by_bundle_identifier() {
        let target = TargetApp {
            name: Some("TextEdit".to_string()),
            identifier: Some("macos:bundle:com.apple.TextEdit".to_string()),
        };

        let command = activate_target_command(PlatformKind::Macos, &target)
            .unwrap()
            .unwrap();

        assert_eq!(command.program, "/usr/bin/open");
        assert_eq!(
            command.args,
            vec!["-b".to_string(), "com.apple.TextEdit".to_string()]
        );
    }

    #[test]
    fn macos_paste_command_uses_direct_paste_builtin() {
        let target = TargetApp {
            name: Some("TextEdit".to_string()),
            identifier: Some("macos:bundle:com.apple.TextEdit".to_string()),
        };

        let command = paste_command(PlatformKind::Macos, &target).unwrap();

        assert_eq!(command.program, MACOS_DIRECT_PASTE_PROGRAM);
        assert!(command.args.is_empty());
    }
}
