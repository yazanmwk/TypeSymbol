use regex::Regex;
use typesymbol_config::TypeSymbolConfig;

const INTEGRAL_UPPER_INF: &str = "^∞";

pub struct CoreEngine {
    config: TypeSymbolConfig,
    alias_regex: Option<Regex>,
    operator_regex: Option<Regex>,
    integral_bound_regex: Regex,
    integral_verbose_regex: Regex,
    integral_phrase_regex: Regex,
    summation_regex: Regex,
    summation_generic_regex: Regex,
    summation_phrase_regex: Regex,
    summation_phrase_implicit_var_regex: Regex,
    product_generic_regex: Regex,
    product_phrase_regex: Regex,
    laplace_regex: Regex,
    inv_laplace_regex: Regex,
    fourier_regex: Regex,
    inv_fourier_regex: Regex,
    limit_phrase_regex: Regex,
    limit_arrow_regex: Regex,
    limit_arrow_unicode_regex: Regex,
    partial_derivative_regex: Regex,
    forall_regex: Regex,
    exists_regex: Regex,
    in_regex: Regex,
    not_in_regex: Regex,
    subseteq_regex: Regex,
    union_regex: Regex,
    intersection_regex: Regex,
    probability_regex: Regex,
    expectation_regex: Regex,
    variance_regex: Regex,
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
            integral_bound_regex: Regex::new(r"(?i)\bint_0\^(?:inf|infinity)\b").expect("valid regex"),
            integral_verbose_regex: Regex::new(r"(?i)\bintegral0-(?:inf|infinity)\(([^)]+)\)").expect("valid regex"),
            integral_phrase_regex: Regex::new(
                r"(?i)\b(?:integral|int)\s*(?:from\s+)?([A-Za-z0-9]+)\s*(?:to|->)\s*([A-Za-z0-9]+)\s*(.*)$",
            )
            .expect("valid regex"),
            summation_regex: Regex::new(r"\bsum_\(i=1\)\^n\b").expect("valid regex"),
            summation_generic_regex: Regex::new(
                r"(?i)\bsum_\(\s*([A-Za-z])\s*=\s*([A-Za-z0-9∞]+)\s*\)\s*\^\s*([A-Za-z0-9∞]+)(\s|$)",
            )
            .expect("valid regex"),
            summation_phrase_regex: Regex::new(
                r"(?i)\b(?:sum|summation|sumnation)\s*(?:from\s+)?([A-Za-z])\s*=\s*([A-Za-z0-9∞]+)\s*(?:to|->)\s*([A-Za-z0-9∞]+)\s*(.*)$",
            )
            .expect("valid regex"),
            summation_phrase_implicit_var_regex: Regex::new(
                r"(?i)\b(?:sum|summation|sumnation)\s*(?:from\s+)?([A-Za-z0-9∞]+)\s*(?:to|->)\s*([A-Za-z0-9∞]+)\s*(.*)$",
            )
            .expect("valid regex"),
            product_generic_regex: Regex::new(
                r"(?i)\b(?:prod|product)_\(\s*([A-Za-z])\s*=\s*([A-Za-z0-9∞]+)\s*\)\s*\^\s*([A-Za-z0-9∞]+)(\s|$)",
            )
            .expect("valid regex"),
            product_phrase_regex: Regex::new(
                r"(?i)\b(?:product|prod)\s*(?:from\s+)?([A-Za-z])\s*=\s*([A-Za-z0-9∞]+)\s*(?:to|->)\s*([A-Za-z0-9∞]+)\s*(.*)$",
            )
            .expect("valid regex"),
            laplace_regex: Regex::new(r"(?i)\blaplace(?:\s+transform)?\s+(?:of\s+)?(.+)$").expect("valid regex"),
            inv_laplace_regex: Regex::new(
                r"(?i)\b(?:inverse\s+laplace|inv\s+laplace)(?:\s+transform)?\s+(?:of\s+)?(.+)$",
            )
            .expect("valid regex"),
            fourier_regex: Regex::new(r"(?i)\bfourier(?:\s+transform)?\s+of\s+(.+)$").expect("valid regex"),
            inv_fourier_regex: Regex::new(
                r"(?i)\b(?:inverse\s+fourier|inv\s+fourier)(?:\s+transform)?\s+of\s+(.+)$",
            )
            .expect("valid regex"),
            limit_phrase_regex: Regex::new(
                r"(?i)\b(?:limit|lim)\s+([A-Za-z])\s+(?:to|->)\s+([A-Za-z0-9∞]+)\s+(?:of\s+)?(.+)$",
            )
            .expect("valid regex"),
            limit_arrow_regex: Regex::new(r"(?i)\blim\s*\(\s*([A-Za-z])\s*->\s*([A-Za-z0-9∞]+)\s*\)\s*(.+)$")
                .expect("valid regex"),
            // Matches normalized arrow form after operator replacement.
            limit_arrow_unicode_regex: Regex::new(
                r"(?i)\blim\s*\(\s*([A-Za-z])\s*→\s*([A-Za-z0-9∞]+)\s*\)\s*(.+)$",
            )
            .expect("valid regex"),
            partial_derivative_regex: Regex::new(r"(?i)\bpartial\s*/\s*partial\s*([A-Za-z])\s+(.+)$")
                .expect("valid regex"),
            forall_regex: Regex::new(r"(?i)\bfor\s+all\b|\bforall\b").expect("valid regex"),
            exists_regex: Regex::new(r"(?i)\bthere\s+exists\b|\bexists\b").expect("valid regex"),
            in_regex: Regex::new(r"(?i)\b([A-Za-z0-9_]{1,3})\s+in\s+([A-Z][A-Za-z0-9_]*)\b")
                .expect("valid regex"),
            not_in_regex: Regex::new(r"(?i)\b([A-Za-z0-9_]{1,3})\s+not\s+in\s+([A-Z][A-Za-z0-9_]*)\b")
                .expect("valid regex"),
            subseteq_regex: Regex::new(r"(?i)\bsubset\s*eq\b|\bsubseteq\b").expect("valid regex"),
            union_regex: Regex::new(r"(?i)\bunion\b").expect("valid regex"),
            intersection_regex: Regex::new(r"(?i)\bintersection\b").expect("valid regex"),
            probability_regex: Regex::new(r"(?i)\b(?:probability|prob)\s+of\s+(.+)$").expect("valid regex"),
            expectation_regex: Regex::new(r"(?i)\b(?:expectation|expected\s+value)\s+of\s+(.+)$")
                .expect("valid regex"),
            variance_regex: Regex::new(r"(?i)\b(?:variance|var)\s+of\s+(.+)$").expect("valid regex"),
            sqrt_group_regex: Regex::new(r"\bsqrt\(([^)]+)\)").expect("valid regex"),
            sqrt_word_regex: Regex::new(r"\bsqrt\s+([A-Za-z0-9]+)\b").expect("valid regex"),
            superscript_regex: Regex::new(r"\^([A-Za-z0-9+\-=]+)").expect("valid regex"),
            subscript_regex: Regex::new(r"_([A-Za-z0-9+\-=]+)").expect("valid regex"),
        }
    }

    pub fn format(&self, input: &str) -> String {
        let mut output = input.to_owned();

        output = normalize_power_phrase(&output);

        if self.config.features.integrals {
            output = self.apply_integrals(&output);
        }

        if self.config.features.operators {
            output = self.apply_operators(&output);
        }

        if self.config.features.greek_letters {
            output = self.apply_aliases(&output);
        }

        // Run a second pass after operator/alias normalization so variants
        // like "lim (t->inf)" that become "lim (t→∞)" are still recognized.
        output = self.apply_math_pack(&output);

        if self.config.features.summations {
            output = self.apply_summations(&output);
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
            .replace_all(input, &format!("∫₀{}", INTEGRAL_UPPER_INF))
            .to_string();

        output = self
            .integral_verbose_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let expr = caps[1].trim();
                format!("∫₀{} {} dx", INTEGRAL_UPPER_INF, expr)
            })
            .to_string();

        output = self
            .integral_phrase_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let lower_raw = caps[1].trim();
                let upper_raw = caps[2].trim();
                let tail_raw = caps[3].trim();
                let tail = strip_optional_of_prefix(tail_raw);

                let lower = format_lower_bound(lower_raw);
                let upper = format_upper_bound(upper_raw);

                if tail.is_empty() {
                    format!("∫{}{}", lower, upper)
                } else {
                    let diff = extract_or_default_differential(&tail);
                    let expr = normalize_integral_expression(&tail);
                    let expr = strip_trailing_differential(&expr, diff);
                    format!("∫{}{} {} d{}", lower, upper, expr, diff)
                }
            })
            .to_string();

        Regex::new(r"(?i)\bint\b")
            .expect("valid regex")
            .replace_all(&output, "∫")
            .to_string()
    }

    fn apply_summations(&self, input: &str) -> String {
        let mut output = self
            .summation_regex
            .replace_all(input, "∑ᵢ₌₁ⁿ")
            .to_string();
        output = self
            .summation_generic_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = caps[1].to_lowercase();
                let start = normalize_bound_token(caps[2].trim());
                let end = normalize_bound_token(caps[3].trim());
                format!("{}{}", format_sum_core(&var, &start, &end), &caps[4])
            })
            .to_string();

        output = self
            .summation_phrase_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = caps[1].to_lowercase();
                let start = normalize_bound_token(caps[2].trim());
                let end = normalize_bound_token(caps[3].trim());
                let tail = strip_optional_of_prefix(caps[4].trim());

                let core = format_sum_core(&var, &start, &end);

                if tail.is_empty() {
                    core
                } else {
                    format!("{} {}", core, normalize_sum_expression(&tail))
                }
            })
            .to_string();

        output = self
            .summation_phrase_implicit_var_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = "n".to_string();
                let start = normalize_bound_token(caps[1].trim());
                let end = normalize_bound_token(caps[2].trim());
                let tail = strip_optional_of_prefix(caps[3].trim());

                let core = format_sum_core(&var, &start, &end);

                if tail.is_empty() {
                    core
                } else {
                    format!("{} {}", core, normalize_sum_expression(&tail))
                }
            })
            .to_string();

        output
    }

    fn apply_math_pack(&self, input: &str) -> String {
        let mut output = input.to_string();

        output = self
            .inv_laplace_regex
            .replace_all(&output, |caps: &regex::Captures| format!("ℒ⁻¹{{{}}}", caps[1].trim()))
            .to_string();
        output = self
            .laplace_regex
            .replace_all(&output, |caps: &regex::Captures| format!("ℒ{{{}}}", caps[1].trim()))
            .to_string();
        output = self
            .inv_fourier_regex
            .replace_all(&output, |caps: &regex::Captures| format!("ℱ⁻¹{{{}}}", caps[1].trim()))
            .to_string();
        output = self
            .fourier_regex
            .replace_all(&output, |caps: &regex::Captures| format!("ℱ{{{}}}", caps[1].trim()))
            .to_string();

        output = self
            .limit_arrow_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = format_limit_variable(&caps[1]);
                let target = normalize_bound_token(caps[2].trim());
                let target_fmt = if target == "inf" { "∞".to_string() } else { target };
                format!("lim{}→{} {}", var, target_fmt, caps[3].trim())
            })
            .to_string();
        output = self
            .limit_arrow_unicode_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = format_limit_variable(&caps[1]);
                let target = normalize_bound_token(caps[2].trim());
                let target_fmt = if target == "inf" { "∞".to_string() } else { target };
                format!("lim{}→{} {}", var, target_fmt, caps[3].trim())
            })
            .to_string();
        output = self
            .limit_phrase_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = format_limit_variable(&caps[1]);
                let target = normalize_bound_token(caps[2].trim());
                let target_fmt = if target == "inf" { "∞".to_string() } else { target };
                format!("lim{}→{} {}", var, target_fmt, caps[3].trim())
            })
            .to_string();

        output = self
            .partial_derivative_regex
            .replace_all(&output, |caps: &regex::Captures| format!("∂/∂{} {}", caps[1].to_lowercase(), caps[2].trim()))
            .to_string();

        output = self
            .product_generic_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = caps[1].to_lowercase();
                let start = normalize_bound_token(caps[2].trim());
                let end = normalize_bound_token(caps[3].trim());
                format!("{}{}", format_product_core(&var, &start, &end), &caps[4])
            })
            .to_string();
        output = self
            .product_phrase_regex
            .replace_all(&output, |caps: &regex::Captures| {
                let var = caps[1].to_lowercase();
                let start = normalize_bound_token(caps[2].trim());
                let end = normalize_bound_token(caps[3].trim());
                let tail = strip_optional_of_prefix(caps[4].trim());

                let core = format_product_core(&var, &start, &end);

                if tail.is_empty() {
                    core
                } else {
                    format!("{} {}", core, normalize_sum_expression(&tail))
                }
            })
            .to_string();

        output = self
            .probability_regex
            .replace_all(&output, |caps: &regex::Captures| format!("P({})", caps[1].trim()))
            .to_string();
        output = self
            .expectation_regex
            .replace_all(&output, |caps: &regex::Captures| format!("E[{}]", caps[1].trim()))
            .to_string();
        output = self
            .variance_regex
            .replace_all(&output, |caps: &regex::Captures| format!("Var({})", caps[1].trim()))
            .to_string();

        output = self
            .not_in_regex
            .replace_all(&output, |caps: &regex::Captures| {
                format!("{} ∉ {}", &caps[1], &caps[2])
            })
            .to_string();
        output = self
            .subseteq_regex
            .replace_all(&output, "⊆")
            .to_string();
        output = self
            .union_regex
            .replace_all(&output, "∪")
            .to_string();
        output = self
            .intersection_regex
            .replace_all(&output, "∩")
            .to_string();
        output = self
            .forall_regex
            .replace_all(&output, "∀")
            .to_string();
        output = self
            .exists_regex
            .replace_all(&output, "∃")
            .to_string();
        output = self
            .in_regex
            .replace_all(&output, |caps: &regex::Captures| {
                format!("{} ∈ {}", &caps[1], &caps[2])
            })
            .to_string();

        output
    }
}

fn normalize_integral_expression(expr: &str) -> String {
    let mut out = strip_optional_of_prefix(expr);
    out = Regex::new(r"(?i)\bover\b")
        .expect("valid regex")
        .replace_all(&out, "/")
        .to_string();
    out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    Regex::new(r"\s*/\s*")
        .expect("valid regex")
        .replace_all(&out, "/")
        .to_string()
}

fn normalize_sum_expression(expr: &str) -> String {
    normalize_integral_expression(expr)
}

fn strip_optional_of_prefix(input: &str) -> String {
    Regex::new(r"(?i)^of\s+")
        .expect("valid regex")
        .replace(input, "")
        .into_owned()
}

fn normalize_bound_token(raw: &str) -> String {
    if raw.eq_ignore_ascii_case("inf") || raw.eq_ignore_ascii_case("infinity") || raw == "∞" {
        "inf".to_string()
    } else {
        raw.to_lowercase()
    }
}

fn format_limit_variable(raw: &str) -> String {
    let lowered = raw.to_lowercase();
    if can_render_subscript(&lowered) {
        to_sub(&lowered)
    } else {
        format!("_{{{}}}", lowered)
    }
}

fn format_sum_core(var: &str, start: &str, end: &str) -> String {
    let var_sub = if can_render_subscript(var) {
        to_sub(var)
    } else {
        format!("_{{{}}}", var)
    };
    let start_sub = if can_render_subscript(start) {
        to_sub(start)
    } else {
        format!("_{{{}}}", start)
    };
    let end_sup = format_upper_bound(end);
    format!("∑{}₌{}{}", var_sub, start_sub, end_sup)
}

fn format_product_core(var: &str, start: &str, end: &str) -> String {
    let var_sub = if can_render_subscript(var) {
        to_sub(var)
    } else {
        format!("_{{{}}}", var)
    };
    let start_sub = if can_render_subscript(start) {
        to_sub(start)
    } else {
        format!("_{{{}}}", start)
    };
    let end_sup = format_upper_bound(end);
    format!("∏{}₌{}{}", var_sub, start_sub, end_sup)
}

fn format_lower_bound(raw: &str) -> String {
    if raw.eq_ignore_ascii_case("inf") || raw.eq_ignore_ascii_case("infinity") {
        return "_∞".to_string();
    }
    let lowered = raw.to_lowercase();
    if can_render_subscript(&lowered) {
        to_sub(&lowered)
    } else {
        format!("_{{{}}}", lowered)
    }
}

fn format_upper_bound(raw: &str) -> String {
    if raw.eq_ignore_ascii_case("inf") || raw.eq_ignore_ascii_case("infinity") {
        return INTEGRAL_UPPER_INF.to_string();
    }
    let lowered = raw.to_lowercase();
    if can_render_superscript(&lowered) {
        to_super(&lowered)
    } else {
        format!("^{{{}}}", lowered)
    }
}

fn extract_or_default_differential(expr: &str) -> char {
    let re = Regex::new(r"(?i)\bd([a-z])\b").expect("valid regex");
    if let Some(caps) = re.captures(expr) {
        caps[1].chars().next().unwrap_or('x')
    } else {
        'x'
    }
}

fn strip_trailing_differential(expr: &str, diff: char) -> String {
    let pattern = format!(r"(?i)\s+d{}\s*$", regex::escape(&diff.to_string()));
    Regex::new(&pattern)
        .expect("valid regex")
        .replace(expr, "")
        .trim()
        .to_string()
}

fn can_render_superscript(raw: &str) -> bool {
    raw.chars().all(has_superscript_char)
}

fn can_render_subscript(raw: &str) -> bool {
    raw.chars().all(has_subscript_char)
}

fn has_superscript_char(ch: char) -> bool {
    matches!(
        ch,
        '0'..='9'
            | '+'
            | '-'
            | '='
            | '('
            | ')'
            | 'a'
            | 'b'
            | 'c'
            | 'd'
            | 'e'
            | 'f'
            | 'g'
            | 'h'
            | 'i'
            | 'j'
            | 'k'
            | 'l'
            | 'm'
            | 'n'
            | 'o'
            | 'p'
            | 'r'
            | 's'
            | 't'
            | 'u'
            | 'v'
            | 'w'
            | 'x'
            | 'y'
            | 'z'
    )
}

fn has_subscript_char(ch: char) -> bool {
    matches!(ch, '0'..='9' | '+' | '-' | '=' | '(' | ')' | 'i' | 'n')
}

fn normalize_power_phrase(input: &str) -> String {
    Regex::new(r"(?i)\s+power\s+of\s+")
        .expect("valid regex")
        .replace_all(input, "^")
        .to_string()
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
        'a' => 'ᵃ',
        'b' => 'ᵇ',
        'c' => 'ᶜ',
        'd' => 'ᵈ',
        'e' => 'ᵉ',
        'f' => 'ᶠ',
        'g' => 'ᵍ',
        'h' => 'ʰ',
        'n' => 'ⁿ',
        'i' => 'ⁱ',
        'j' => 'ʲ',
        'k' => 'ᵏ',
        'l' => 'ˡ',
        'm' => 'ᵐ',
        'o' => 'ᵒ',
        'p' => 'ᵖ',
        'r' => 'ʳ',
        's' => 'ˢ',
        't' => 'ᵗ',
        'u' => 'ᵘ',
        'v' => 'ᵛ',
        'w' => 'ʷ',
        'x' => 'ˣ',
        'y' => 'ʸ',
        'z' => 'ᶻ',
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
        'n' => 'ₙ',
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
    fn converts_natural_language_integral_forms() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(
            engine.format("integral from 0 to infinity of x over x^2"),
            "∫₀^∞ x/x² dx"
        );
        assert_eq!(
            engine.format("int 0 -> inf x/x^2"),
            "∫₀^∞ x/x² dx"
        );
        assert_eq!(engine.format("integral from 0 to x"), "∫₀ˣ");
        assert_eq!(engine.format("int from 1 to n"), "∫₁ⁿ");
        assert_eq!(
            engine.format("INTEGRAL FROM 0 TO X OF x^2 dx"),
            "∫₀ˣ x² dx"
        );
        assert_eq!(
            engine.format("int from a to b of f(t) dt"),
            "∫_{a}ᵇ f(t) dt"
        );
        assert_eq!(
            engine.format("int 0 to infinity x over x^2"),
            "∫₀^∞ x/x² dx"
        );
        assert_eq!(engine.format("integral 0->inf of x"), "∫₀^∞ x dx");
        assert_eq!(engine.format("int from 0 to n of x"), "∫₀ⁿ x dx");
    }

    #[test]
    fn preserves_case_for_non_math_text() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(
            engine.format("THIS IS AN ALL CAPS MESSAGE"),
            "THIS IS AN ALL CAPS MESSAGE"
        );
    }

    #[test]
    fn converts_sum_and_scripts() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("sum_(i=1)^n i^2"), "∑ᵢ₌₁ⁿ i²");
        assert_eq!(engine.format("sum_(k=0)^inf k"), "∑_{k}₌₀^∞ k");
        assert_eq!(engine.format("sum from n = 0 to ∞"), "∑ₙ₌₀^∞");
        assert_eq!(engine.format("sum n=0 to inf n^2"), "∑ₙ₌₀^∞ n²");
        assert_eq!(
            engine.format("sum from n=0 to inf of n^2"),
            "∑ₙ₌₀^∞ n²"
        );
        assert_eq!(
            engine.format("summation from i = 1 to n of i"),
            "∑ᵢ₌₁ⁿ i"
        );
        assert_eq!(
            engine.format("sumnation from i = 1 to n of i"),
            "∑ᵢ₌₁ⁿ i"
        );
        assert_eq!(
            engine.format("sum from 0 to infinity of 1/x"),
            "∑ₙ₌₀^∞ 1/x"
        );
    }

    #[test]
    fn converts_multi_char_exponents() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("x^3x"), "x³ˣ");
        assert_eq!(engine.format("x power of 3x"), "x³ˣ");
    }

    #[test]
    fn converts_sqrt_forms() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("sqrt(x) + sqrt y"), "√(x) + √y");
    }

    #[test]
    fn converts_math_pack_notations() {
        let engine = CoreEngine::new(test_config());
        assert_eq!(engine.format("laplace of f(t)"), "ℒ{f(t)}");
        assert_eq!(engine.format("laplace f(t)"), "ℒ{f(t)}");
        assert_eq!(engine.format("inverse laplace of F(s)"), "ℒ⁻¹{F(s)}");
        assert_eq!(engine.format("inv laplace F(s)"), "ℒ⁻¹{F(s)}");
        assert_eq!(engine.format("fourier transform of f(t)"), "ℱ{f(t)}");
        assert_eq!(engine.format("inv fourier of X(w)"), "ℱ⁻¹{X(w)}");
        assert_eq!(
            engine.format("limit x to 0 of sin(x)/x"),
            "lim_{x}→0 sin(x)/x"
        );
        assert_eq!(
            engine.format("lim (t->inf) e^(-t)"),
            "lim_{t}→∞ e^(-t)"
        );
        assert_eq!(
            engine.format("partial/partial x f(x,y)"),
            "∂/∂x f(x,y)"
        );
        assert_eq!(
            engine.format("product from i = 1 to n of i"),
            "∏ᵢ₌₁ⁿ i"
        );
        assert_eq!(engine.format("prod_(i=1)^n i"), "∏ᵢ₌₁ⁿ i");
        assert_eq!(engine.format("product i=1 to n i"), "∏ᵢ₌₁ⁿ i");
        assert_eq!(
            engine.format("for all x in A"),
            "∀ x ∈ A"
        );
        assert_eq!(
            engine.format("there exists y not in B"),
            "∃ y ∉ B"
        );
        assert_eq!(
            engine.format("A subseteq B"),
            "A ⊆ B"
        );
        assert_eq!(
            engine.format("A union B intersection C"),
            "A ∪ B ∩ C"
        );
        assert_eq!(
            engine.format("probability of A|B"),
            "P(A|B)"
        );
        assert_eq!(
            engine.format("expected value of X"),
            "E[X]"
        );
        assert_eq!(
            engine.format("variance of X"),
            "Var(X)"
        );
        assert_eq!(
            engine.format("when i type in it"),
            "when i type in it"
        );
    }
}
