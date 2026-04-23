use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TypeSymbolConfig {
    pub mode: String,
    pub trigger_mode: String,
    pub trigger_key: String,
    pub live_suggestions: bool,
    pub auto_replace_safe_rules: bool,
    
    pub features: FeatureSet,
    pub aliases: HashMap<String, String>,
    pub operators: HashMap<String, String>,
    
    #[serde(default)]
    pub excluded_apps: HashSet<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FeatureSet {
    pub greek_letters: bool,
    pub operators: bool,
    pub superscripts: bool,
    pub subscripts: bool,
    pub sqrt: bool,
    pub integrals: bool,
    pub summations: bool,
    pub limits: bool,
}

impl Default for TypeSymbolConfig {
    fn default() -> Self {
        Self {
            mode: "unicode".to_string(),
            trigger_mode: "manual".to_string(),
            trigger_key: "ctrl-space".to_string(),
            live_suggestions: false,
            auto_replace_safe_rules: true,
            features: FeatureSet {
                greek_letters: true,
                operators: true,
                superscripts: true,
                subscripts: true,
                sqrt: true,
                integrals: true,
                summations: true,
                limits: true,
            },
            aliases: default_aliases(),
            operators: default_operators(),
            excluded_apps: HashSet::from([
                "com.apple.Terminal".to_string(),
                "com.microsoft.VSCode".to_string(),
                "com.jetbrains.rustrover".to_string(),
            ]),
        }
    }
}

pub fn load_config(path: &str) -> Result<TypeSymbolConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: TypeSymbolConfig = toml::from_str(&content)?;
    Ok(config)
}

fn default_aliases() -> HashMap<String, String> {
    HashMap::from([
        ("alpha".to_string(), "α".to_string()),
        ("beta".to_string(), "β".to_string()),
        ("gamma".to_string(), "γ".to_string()),
        ("theta".to_string(), "θ".to_string()),
        ("lambda".to_string(), "λ".to_string()),
        ("pi".to_string(), "π".to_string()),
        ("inf".to_string(), "∞".to_string()),
        ("infinity".to_string(), "∞".to_string()),
    ])
}

fn default_operators() -> HashMap<String, String> {
    HashMap::from([
        ("->".to_string(), "→".to_string()),
        ("<-".to_string(), "←".to_string()),
        ("<->".to_string(), "↔".to_string()),
        ("!=".to_string(), "≠".to_string()),
        ("<=".to_string(), "≤".to_string()),
        (">=".to_string(), "≥".to_string()),
        ("+-".to_string(), "±".to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::{load_config, TypeSymbolConfig};

    #[test]
    fn default_includes_mvp_rules() {
        let cfg = TypeSymbolConfig::default();
        assert_eq!(cfg.aliases.get("alpha").map(String::as_str), Some("α"));
        assert_eq!(cfg.aliases.get("infinity").map(String::as_str), Some("∞"));
        assert_eq!(cfg.operators.get("->").map(String::as_str), Some("→"));
    }

    #[test]
    fn toml_config_deserializes() {
        let dir = std::env::temp_dir();
        let path = dir.join("typesymbol-config-test.toml");
        let raw = r#"
mode = "unicode"
trigger_mode = "manual"
trigger_key = "ctrl-space"
live_suggestions = false
auto_replace_safe_rules = true

[features]
greek_letters = true
operators = true
superscripts = true
subscripts = true
sqrt = true
integrals = true
summations = true
limits = true

[aliases]
alpha = "α"

[operators]
"->" = "→"
"#;
        std::fs::write(&path, raw).expect("write test config");
        let cfg = load_config(path.to_str().expect("temp path utf8")).expect("load config");
        assert_eq!(cfg.aliases.get("alpha").map(String::as_str), Some("α"));
        assert_eq!(cfg.operators.get("->").map(String::as_str), Some("→"));
        let _ = std::fs::remove_file(path);
    }
}
