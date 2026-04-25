use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use rdev::{grab, Event, EventType};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use typesymbol_config::TypeSymbolConfig;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

static SUPPRESS_EVENTS: AtomicBool = AtomicBool::new(false);

pub struct WindowsAdapter {
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

impl WindowsAdapter {
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
        println!("WindowsAdapter: Starting event loop listener...");
        println!("WindowsAdapter: trigger key = {}", self.config.trigger_key);
        println!(
            "WindowsAdapter: exclusions active for {} process names",
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
                EventType::KeyPress(rdev::Key::ControlLeft)
                | EventType::KeyPress(rdev::Key::ControlRight) => {
                    ctrl_pressed.store(true, Ordering::Relaxed);
                    Some(event)
                }
                EventType::KeyRelease(rdev::Key::ControlLeft)
                | EventType::KeyRelease(rdev::Key::ControlRight) => {
                    ctrl_pressed.store(false, Ordering::Relaxed);
                    Some(event)
                }
                EventType::KeyPress(rdev::Key::Backspace) => {
                    on_event(PlatformEvent::Backspace);
                    Some(event)
                }
                EventType::KeyPress(rdev::Key::Return) => {
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
                EventType::KeyPress(rdev::Key::Space) => {
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
            eprintln!("WindowsAdapter grab listener failed: {error:?}");
        }
    }

    fn spawn_active_app_monitor(&self) {
        let is_dormant = Arc::clone(&self.is_dormant);
        let excluded = self.config.excluded_apps.clone();
        thread::spawn(move || loop {
            let process_name = frontmost_process_name();
            let dormant = process_name
                .as_deref()
                .map(|name| excluded.contains(name))
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

    let mut clipboard = Clipboard::new().map_err(|err| format!("clipboard init failed: {err}"))?;
    let previous = clipboard.get_text().ok();
    clipboard
        .set_text(replacement.to_string())
        .map_err(|err| format!("clipboard set failed: {err}"))?;

    let mut enigo = Enigo::new(&Settings::default()).map_err(|err| format!("enigo init failed: {err}"))?;
    for _ in 0..backspace_count {
        let _ = enigo.key(Key::Backspace, Direction::Click);
    }
    let _ = enigo.key(Key::Control, Direction::Press);
    // On Windows, enigo expects virtual keys for modified shortcuts (Ctrl+V).
    let _ = enigo.key(Key::V, Direction::Click);
    let _ = enigo.key(Key::Control, Direction::Release);
    thread::sleep(Duration::from_millis(50));

    if let Some(previous) = previous {
        let _ = clipboard.set_text(previous);
    }

    thread::sleep(Duration::from_millis(120));
    SUPPRESS_EVENTS.store(false, Ordering::Relaxed);
    Ok(())
}

fn frontmost_process_name() -> Option<String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return None;
    }

    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }

    let mut size: u32 = 1024;
    let mut buffer = vec![0u16; size as usize];
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size) } != 0;
    unsafe {
        CloseHandle(handle);
    }
    if !ok || size == 0 {
        return None;
    }

    let path = String::from_utf16(&buffer[..size as usize]).ok()?;
    path.rsplit('\\').next().map(|s| s.to_string())
}

fn trigger_kind_from_config(raw: &str) -> TriggerKind {
    match raw.trim().to_lowercase().as_str() {
        "ctrl-space" | "control-space" | "ctrl+space" | "control+space" => TriggerKind::CtrlSpace,
        _ => TriggerKind::Enter,
    }
}
