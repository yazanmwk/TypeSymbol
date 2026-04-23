use typesymbol_config::TypeSymbolConfig;
use typesymbol_core::CoreEngine;
use typesymbol_platform_macos::{inject_replacement, MacOSAdapter, PlatformEvent};

pub fn run(config: TypeSymbolConfig) {
    println!("Starting TypeSymbol daemon with config: {:?}", config.mode);
    println!("App Exclusions loaded: {:?}", config.excluded_apps);

    let mut daemon = TypeSymbolDaemon::new(config.clone());
    let mut adapter = MacOSAdapter::new(config);
    adapter.start_listening(move |event| match event {
        PlatformEvent::Char(ch) => {
            daemon.on_char_typed(ch);
            if let Some(candidate) = daemon.preview_replacement() {
                println!("Press Enter for {}", candidate.replacement);
            }
        }
        PlatformEvent::Backspace => daemon.on_backspace(),
        PlatformEvent::Enter => {
            if let Some(candidate) = daemon.preview_replacement() {
                match inject_replacement(&candidate.original, &candidate.replacement, 1) {
                    Ok(()) => {
                        println!("Replaced: {} -> {}", candidate.original, candidate.replacement);
                        daemon.reset_buffer();
                    }
                    Err(err) => {
                        eprintln!("Replacement injection failed: {}", err);
                    }
                }
            } else {
                daemon.reset_buffer();
            }
        }
    });
}

pub struct TypeSymbolDaemon {
    engine: CoreEngine,
    text_buffer: String,
    max_buffer_chars: usize,
}

impl TypeSymbolDaemon {
    pub fn new(config: TypeSymbolConfig) -> Self {
        Self {
            engine: CoreEngine::new(config),
            text_buffer: String::new(),
            max_buffer_chars: 256,
        }
    }

    pub fn on_char_typed(&mut self, ch: char) {
        self.text_buffer.push(ch);
        self.trim_buffer_to_max();
    }

    pub fn on_backspace(&mut self) {
        self.text_buffer.pop();
    }

    pub fn reset_buffer(&mut self) {
        self.text_buffer.clear();
    }

    pub fn preview_replacement(&self) -> Option<ReplacementCandidate> {
        let chars: Vec<char> = self.text_buffer.chars().collect();
        let max_suffix = chars.len().min(64);

        for len in (1..=max_suffix).rev() {
            let start = chars.len() - len;
            let suffix: String = chars[start..].iter().collect();
            let formatted = self.engine.format(&suffix);
            if formatted != suffix {
                return Some(ReplacementCandidate {
                    original: suffix,
                    replacement: formatted,
                });
            }
        }
        None
    }

    pub fn current_buffer(&self) -> &str {
        &self.text_buffer
    }

    fn trim_buffer_to_max(&mut self) {
        let count = self.text_buffer.chars().count();
        if count <= self.max_buffer_chars {
            return;
        }
        let drop_count = count - self.max_buffer_chars;
        self.text_buffer = self.text_buffer.chars().skip(drop_count).collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacementCandidate {
    pub original: String,
    pub replacement: String,
}

#[cfg(test)]
mod tests {
    use super::TypeSymbolDaemon;
    use typesymbol_config::TypeSymbolConfig;

    #[test]
    fn finds_replacement_candidate() {
        let mut daemon = TypeSymbolDaemon::new(TypeSymbolConfig::default());
        for ch in "alpha -> beta".chars() {
            daemon.on_char_typed(ch);
        }
        let candidate = daemon.preview_replacement().expect("candidate exists");
        assert_eq!(candidate.original, "alpha -> beta");
        assert_eq!(candidate.replacement, "α → β");
    }

    #[test]
    fn returns_none_when_no_match() {
        let mut daemon = TypeSymbolDaemon::new(TypeSymbolConfig::default());
        for ch in "hello world".chars() {
            daemon.on_char_typed(ch);
        }
        assert!(daemon.preview_replacement().is_none());
    }
}
