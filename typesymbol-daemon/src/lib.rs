use typesymbol_config::TypeSymbolConfig;
use typesymbol_core::CoreEngine;
use typesymbol_platform_macos::{inject_replacement, MacOSAdapter, PlatformEvent};
use std::sync::{Arc, Mutex};

pub fn run(config: TypeSymbolConfig) {
    println!("Starting TypeSymbol daemon with config: {:?}", config.mode);
    println!("App Exclusions loaded: {:?}", config.excluded_apps);

    let trigger_label = match config.trigger_key.trim().to_lowercase().as_str() {
        "ctrl-space" | "control-space" | "ctrl+space" | "control+space" => "Ctrl+Space",
        _ => "Enter",
    };

    let state = Arc::new(Mutex::new(DaemonState {
        daemon: TypeSymbolDaemon::new(config.clone()),
        last_prompt: None,
        trigger_label,
    }));
    let mut adapter = MacOSAdapter::new(config);
    adapter.start_listening(move |event| {
        let mut state = match state.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        state.handle_event(event)
    });
}

struct DaemonState {
    daemon: TypeSymbolDaemon,
    last_prompt: Option<String>,
    trigger_label: &'static str,
}

impl DaemonState {
    fn handle_event(&mut self, event: PlatformEvent) -> bool {
        match event {
            PlatformEvent::Char(ch) => {
                self.daemon.on_char_typed(ch);
                self.refresh_prompt();
                false
            }
            PlatformEvent::Backspace => {
                self.daemon.on_backspace();
                self.refresh_prompt();
                false
            }
            PlatformEvent::AcceptTrigger => {
                if let Some(candidate) = self.daemon.preview_replacement() {
                    match inject_replacement(&candidate.original, &candidate.replacement, 0) {
                        Ok(()) => {
                            println!("Replaced: {} -> {}", candidate.original, candidate.replacement);
                            self.daemon.reset_buffer();
                            self.last_prompt = None;
                            true
                        }
                        Err(err) => {
                            eprintln!("Replacement injection failed: {}", err);
                            false
                        }
                    }
                } else {
                    self.daemon.reset_buffer();
                    self.last_prompt = None;
                    false
                }
            }
        }
    }

    fn refresh_prompt(&mut self) {
        if let Some(candidate) = self.daemon.preview_replacement() {
            if self.last_prompt.as_deref() != Some(candidate.replacement.as_str()) {
                println!("Press {} for {}", self.trigger_label, candidate.replacement);
                self.last_prompt = Some(candidate.replacement);
            }
        } else {
            self.last_prompt = None;
        }
    }
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
            if formatted != suffix && is_high_confidence_math_replacement(&suffix, &formatted) {
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

fn is_high_confidence_math_replacement(original: &str, replacement: &str) -> bool {
    let original_trimmed = original.trim();
    if original_trimmed.is_empty() || original_trimmed.eq_ignore_ascii_case("in") {
        return false;
    }

    let lower = original_trimmed.to_lowercase();
    let has_keyword = [
        "alpha",
        "beta",
        "gamma",
        "theta",
        "lambda",
        "pi",
        "infinity",
        "integral",
        "int ",
        "sum",
        "summation",
        "product",
        "sqrt",
        "laplace",
        "fourier",
        "limit",
        "lim ",
        "partial",
        "probability",
        "expected value",
        "variance",
        "for all",
        "forall",
        "there exists",
        "exists",
        "subset",
        "union",
        "intersection",
        "not in",
        "power of",
    ]
    .iter()
    .any(|token| lower.contains(token));

    let has_operator_syntax = ["->", "<-", "<->", "!=", "<=", ">=", "+-", "^", "_"]
        .iter()
        .any(|token| original_trimmed.contains(token));

    let has_math_output_symbol = replacement.chars().any(|ch| {
        matches!(
            ch,
            '∫'
                | '∑'
                | '∏'
                | '√'
                | '∞'
                | 'ℒ'
                | 'ℱ'
                | '∂'
                | '∀'
                | '∃'
                | '⊆'
                | '∪'
                | '∩'
                | '∉'
                | '≤'
                | '≥'
                | '≠'
                | '→'
                | '←'
                | '↔'
                | '±'
                | 'α'
                | 'β'
                | 'γ'
                | 'θ'
                | 'λ'
                | 'π'
        )
    });

    let words: Vec<&str> = lower.split_whitespace().collect();
    let has_membership_phrase = words.len() >= 3 && words.contains(&"in");

    has_keyword || has_operator_syntax || has_math_output_symbol || has_membership_phrase
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

    #[test]
    fn does_not_replace_common_text_suffixes() {
        let mut daemon = TypeSymbolDaemon::new(TypeSymbolConfig::default());
        for ch in "check in".chars() {
            daemon.on_char_typed(ch);
        }
        assert!(daemon.preview_replacement().is_none());
    }

    #[test]
    fn still_replaces_set_membership_phrases() {
        let mut daemon = TypeSymbolDaemon::new(TypeSymbolConfig::default());
        for ch in "x in A".chars() {
            daemon.on_char_typed(ch);
        }
        let candidate = daemon.preview_replacement().expect("candidate exists");
        assert_eq!(candidate.replacement, "x ∈ A");
    }
}
