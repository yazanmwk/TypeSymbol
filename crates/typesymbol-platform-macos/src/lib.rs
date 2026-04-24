use rdev::{grab, Event, EventType, Key};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use typesymbol_config::TypeSymbolConfig;

static SUPPRESS_EVENTS: AtomicBool = AtomicBool::new(false);

pub struct MacOSAdapter {
    config: TypeSymbolConfig,
    is_dormant: Arc<AtomicBool>,
    ctrl_pressed: Arc<AtomicBool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformEvent {
    Char(char),
    Backspace,
    AcceptTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TriggerKind {
    Enter,
    CtrlSpace,
}

impl MacOSAdapter {
    pub fn new(config: TypeSymbolConfig) -> Self {
        Self {
            config,
            is_dormant: Arc::new(AtomicBool::new(false)),
            ctrl_pressed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_listening<F>(&mut self, on_event: F)
    where
        F: Fn(PlatformEvent) -> bool + Send + Sync + 'static,
    {
        println!("MacOSAdapter: Starting event loop listener...");
        println!("MacOSAdapter: trigger key = {}", self.config.trigger_key);
        println!(
            "MacOSAdapter: exclusions active for {} bundle identifiers",
            self.config.excluded_apps.len()
        );
        let trigger = trigger_kind_from_config(&self.config.trigger_key);

        self.spawn_active_app_monitor();

        let is_dormant = Arc::clone(&self.is_dormant);
        let ctrl_pressed = Arc::clone(&self.ctrl_pressed);
        if let Err(error) = grab(move |event: Event| {
            if is_dormant.load(Ordering::Relaxed) || SUPPRESS_EVENTS.load(Ordering::Relaxed) {
                return Some(event);
            }

            match event.event_type {
                EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::ControlRight) => {
                    ctrl_pressed.store(true, Ordering::Relaxed);
                    Some(event)
                }
                EventType::KeyRelease(Key::ControlLeft)
                | EventType::KeyRelease(Key::ControlRight) => {
                    ctrl_pressed.store(false, Ordering::Relaxed);
                    Some(event)
                }
                EventType::KeyPress(Key::Backspace) => {
                    on_event(PlatformEvent::Backspace);
                    Some(event)
                }
                EventType::KeyPress(Key::Return) => {
                    if trigger == TriggerKind::Enter {
                        if on_event(PlatformEvent::AcceptTrigger) {
                            None
                        } else {
                            Some(event)
                        }
                    } else {
                        Some(event)
                    }
                }
                EventType::KeyPress(Key::Space) => {
                    if trigger == TriggerKind::CtrlSpace && ctrl_pressed.load(Ordering::Relaxed) {
                        let _ = on_event(PlatformEvent::AcceptTrigger);
                        None
                    } else {
                        on_event(PlatformEvent::Char(' '));
                        Some(event)
                    }
                }
                EventType::KeyPress(_) => {
                    if let Some(ref text) = event.name {
                        for ch in text.chars() {
                            on_event(PlatformEvent::Char(ch));
                        }
                    }
                    Some(event)
                }
                _ => Some(event),
            }
        }) {
            eprintln!("MacOSAdapter grab listener failed: {error:?}");
        }
    }

    fn spawn_active_app_monitor(&self) {
        let is_dormant = Arc::clone(&self.is_dormant);
        let excluded = self.config.excluded_apps.clone();
        thread::spawn(move || loop {
            let bundle = frontmost_bundle_id();
            let dormant = bundle
                .as_deref()
                .map(|id| excluded.contains(id))
                .unwrap_or(false);
            is_dormant.store(dormant, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(500));
        });
    }
}

pub fn inject_replacement(
    original: &str,
    replacement: &str,
    extra_backspaces: usize,
) -> Result<(), String> {
    let backspace_count = original.chars().count() + extra_backspaces;
    if backspace_count == 0 {
        return Ok(());
    }
    SUPPRESS_EVENTS.store(true, Ordering::Relaxed);

    let escaped = escape_applescript_string(replacement);
    let script = format!(
        r#"
set previousClipboard to the clipboard
set the clipboard to "{}"
tell application "System Events"
    repeat {} times
        key code 51
    end repeat
    keystroke "v" using command down
end tell
delay 0.05
set the clipboard to previousClipboard
"#,
        escaped, backspace_count
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| format!("failed to spawn osascript: {err}"))?;

    if output.status.success() {
        // Let synthetic key events flush before re-enabling listener handling.
        thread::sleep(Duration::from_millis(120));
        SUPPRESS_EVENTS.store(false, Ordering::Relaxed);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        SUPPRESS_EVENTS.store(false, Ordering::Relaxed);
        Err(format!(
            "osascript failed (status {}): {}",
            output.status, stderr
        ))
    }
}

fn frontmost_bundle_id() -> Option<String> {
    let script =
        r#"tell application "System Events" to get bundle identifier of first application process whose frontmost is true"#;
    let output = Command::new("osascript").arg("-e").arg(script).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let bundle = String::from_utf8(output.stdout).ok()?;
    let trimmed = bundle.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn escape_applescript_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn trigger_kind_from_config(raw: &str) -> TriggerKind {
    match raw.trim().to_lowercase().as_str() {
        "ctrl-space" | "control-space" | "ctrl+space" | "control+space" => TriggerKind::CtrlSpace,
        _ => TriggerKind::Enter,
    }
}
