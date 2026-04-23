use rdev::{listen, Event, EventType, Key};
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformEvent {
    Char(char),
    Backspace,
    Enter,
}

impl MacOSAdapter {
    pub fn new(config: TypeSymbolConfig) -> Self {
        Self {
            config,
            is_dormant: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_listening<F>(&mut self, mut on_event: F)
    where
        F: FnMut(PlatformEvent) + Send + 'static,
    {
        println!("MacOSAdapter: Starting event loop listener...");
        println!("MacOSAdapter: trigger key = {}", self.config.trigger_key);
        println!(
            "MacOSAdapter: exclusions active for {} bundle identifiers",
            self.config.excluded_apps.len()
        );

        self.spawn_active_app_monitor();

        let is_dormant = Arc::clone(&self.is_dormant);
        if let Err(error) = listen(move |event: Event| {
            if is_dormant.load(Ordering::Relaxed) || SUPPRESS_EVENTS.load(Ordering::Relaxed) {
                return;
            }

            match event.event_type {
                EventType::KeyPress(Key::Backspace) => on_event(PlatformEvent::Backspace),
                EventType::KeyPress(Key::Return) => on_event(PlatformEvent::Enter),
                EventType::KeyPress(Key::Space) => on_event(PlatformEvent::Char(' ')),
                EventType::KeyPress(_) => {
                    if let Some(text) = event.name {
                        for ch in text.chars() {
                            on_event(PlatformEvent::Char(ch));
                        }
                    }
                }
                _ => {}
            }
        }) {
            eprintln!("MacOSAdapter listener failed: {error:?}");
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
