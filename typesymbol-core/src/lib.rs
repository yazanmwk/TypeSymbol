use regex::Regex;
use typesymbol_config::TypeSymbolConfig;

pub struct CoreEngine {
    config: TypeSymbolConfig,
    alias_regex: Option<Regex>,
    operator_regex: Option<Regex>,
    integral_bound_regex: Regex,
    integral_verbose_regex: Regex,
    summation_regex: Regex,
    sqrt_group_regex: Regex,
    sqrt_word_regex: Regex,
    superscript_regex: Regex,
    subscript_regex: Regex,
}

impl CoreEngine {
    pub fn new(config: TypeSymbolConfig) -> Self {
        let alias_regex = compile_alias_regex(&config);
        let operator_regex = compile_operator_regex(&config);

        Self {
            config,
            alias_regex,
            operator_regex,
            integral_bound_regex: Regex::new(r"\bint_0\^(?:inf|infinity)\b").expect("valid regex"),
            integral_verbose_regex: Regex::new(r"\bintegral0-(?:inf|infinity)\(([^)]+)\)").expect("valid regex"),
            summation_regex: Regex::new(r"\bsum_\(i=1\)\^n\b").expect("valid regex"),
            sqrt_group_regex: Regex::new(r"\bsqrt\(([^)]+)\)").expect("valid regex"),
            sqrt_word_regex: Regex::new(r"\bsqrt\s+([A-Za-z0-9]+)\b").expect("valid regex"),
            superscript_regex: Regex::new(r"\^([A-Za-z0-9+\-=]+)").expect("valid regex"),
            subscript_regex: Regex::new(r"_([A-Za-z0-9+\-=]+)").expect("valid regex"),
        }
    }

    pub fn format(&self, input: &str) -> String {
        let mut output = input.to_owned();

        if self.config.features.integrals {
            output = self.apply_integrals(&output);
        }

        if self.config.features.operators {
            output = self.apply_operators(&output);
        }

        if self.config.features.greek_letters {
            output = self.apply_aliases(&output);
        }

        if self.config.features.summations {
            output = self
                .summation_regex
                .replace_all(&output, "∑ᵢ₌₁ⁿ")
                .to_string();
        }

        if self.config.features.sqrt {
            output = self
                .sqrt_group_regex
                .replace_all(&output, |caps: &regex::Captures| format!("√({})", &caps[1]))
                .to_string();
            output = self
                .sqrt_word_regex
                .replace_all(&output, |caps: &regex::Captures| format!("√{}", &caps[1]))
                .to_string();
        }

        if self.config.features.superscripts {
            output = self
                .superscript_regex
                .replace_all(&output, |caps: &regex::Captures| to_super(&caps[1]))
                .to_string();
        }

        if self.config.features.subscripts {
            output = self
                .subscript_regex
                .replace_all(&output, |caps: &regex::Captures| to_sub(&caps[1]))
                .to_string();
        }

        output
    }

    fn apply_operators(&self, input: &str) -> String {
        match &self.operator_regex {
            Some(regex) => regex
                .replace_all(input, |caps: &regex::Captures| {
                    self.config
                        .operators
                        .get(&caps[0])
                        .cloned()
                        .unwrap_or_else(|| caps[0].to_string())
                })
                .to_string(),
            None => input.to_string(),
        }
    }

    fn apply_aliases(&self, input: &str) -> String {
        match &self.alias_regex {
            Some(regex) => regex
                .replace_all(input, |caps: &regex::Captures| {
                    self.config
                        .aliases
                        .get(&caps[0])
                        .cloned()
                        .unwrap_or_else(|| caps[0].to_string())
                })
                .to_string(),
            None => input.to_string(),
        }
    }

    fn apply_integrals(&self, input: &str) -> String {
        let mut output = self
            .integral_bound_regex
            .replace_all(input, "∫₀^∞")
            .to_string();

        output = self
            .integral_verbose_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let expr = caps[1].trim();
                format!("∫₀^∞ {} dx", expr)
            })
            .to_string();

        Regex::new(r"\bint\b")
            .expect("valid regex")
            .replace_all(&output, "∫")
            .to_string()
    }
}

fn compile_alias_regex(config: &TypeSymbolConfig) -> Option<Regex> {
    if config.aliases.is_empty() {
        return None;
    }
    let mut keys: Vec<String> = config.aliases.keys().map(|k| regex::escape(k)).collect();
    keys.sort_by_key(|b| std::cmp::Reverse(b.len()));
    Regex::new(&format!(r"\b({})\b", keys.join("|"))).ok()
}

fn compile_operator_regex(config: &TypeSymbolConfig) -> Option<Regex> {
    if config.operators.is_empty() {
        return None;
    }
    let mut keys = config.operators.keys().cloned().collect::<Vec<_>>();
    keys.sort_by_key(|b| std::cmp::Reverse(b.len()));
    let pattern = keys
        .iter()
        .map(|k| regex::escape(k))
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&pattern).ok()
}

fn to_super(raw: &str) -> String {
    raw.chars().map(map_superscript_char).collect()
}

fn to_sub(raw: &str) -> String {
    raw.chars().map(map_subscript_char).collect()
}

fn map_superscript_char(ch: char) -> char {
    match ch {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'n' => 'ⁿ',
        'i' => 'ⁱ',
        _ => ch,
    }
}

fn map_subscript_char(ch: char) -> char {
    match ch {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'i' => 'ᵢ',
        _ => ch,
    }
}

#[cfg(test)]
mod tests {
    use super::CoreEngine;
    use typesymbol_config::TypeSymbolConfig;

    fn test_config() -> TypeSymbolConfig {
        let mut cfg = TypeSymbolConfig::default();
        cfg.aliases.insert("alpha".to_string(), "α".to_string());
        cfg.aliases.insert("beta".to_string(), "β".to_string());
        cfg.aliases.insert("infinity".to_string(), "∞".to_string());
        cfg.operators.insert("->".to_string(), "→".to_string());
        cfg.operators.insert("!=".to_string(), "≠".to_string());
        cfg
    }

    #[test]
    fn converts_aliases_and_operators() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("alpha -> beta != gamma"), "α → β ≠ γ");
    }

    #[test]
    fn converts_integral_shorthand() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(
            engine.format("integral0-infinity(x/x^2)"),
            "∫₀^∞ x/x² dx"
        );
    }

    #[test]
    fn converts_sum_and_scripts() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("sum_(i=1)^n i^2"), "∑ᵢ₌₁ⁿ i²");
    }

    #[test]
    fn converts_sqrt_forms() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("sqrt(x) + sqrt y"), "√(x) + √y");
    }
}
