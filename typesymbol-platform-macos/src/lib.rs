use rdev::{listen, Event, EventType, Key};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use typesymbol_config::TypeSymbolConfig;

pub struct MacOSAdapter {
    config: TypeSymbolConfig,
    is_dormant: Arc<AtomicBool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformEvent {
    Char(char),
    Backspace,
    Enter,
    Space,
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
            if is_dormant.load(Ordering::Relaxed) {
                return;
            }

            match event.event_type {
                EventType::KeyPress(Key::Backspace) => on_event(PlatformEvent::Backspace),
                EventType::KeyPress(Key::Return) => on_event(PlatformEvent::Enter),
                EventType::KeyPress(Key::Space) => on_event(PlatformEvent::Space),
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
