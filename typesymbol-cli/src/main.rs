use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use typesymbol_config::{load_config, TypeSymbolConfig};
use typesymbol_core::CoreEngine;

#[derive(Parser)]
#[command(name = "typesymbol", version = "0.1", about = "System-wide math shorthand daemon")]
struct Cli {
    /// Optional path to TOML config file
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test an expression against the current rule set
    Test {
        /// The shorthand expression to test
        expression: String,
    },
    /// Start the background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Inspect or manage config
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Run interactive CLI app mode (REPL)
    App,
}

#[derive(Subcommand)]
enum DaemonAction {
    Start,
    Stop,
    Status,
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Init {
        /// Optional output path for config file
        #[arg(long)]
        path: Option<PathBuf>,
        /// Overwrite existing file if present
        #[arg(long)]
        force: bool,
    },
}

struct LoadedConfig {
    config: TypeSymbolConfig,
    source: String,
}

fn resolve_config(cli: &Cli) -> LoadedConfig {
    match &cli.config {
        Some(path) => match load_config(path.to_str().unwrap_or_default()) {
            Ok(cfg) => LoadedConfig {
                config: cfg,
                source: format!("file ({})", path.display()),
            },
            Err(err) => {
                eprintln!(
                    "Failed to load config from {}: {}. Falling back to defaults.",
                    path.display(),
                    err
                );
                LoadedConfig {
                    config: TypeSymbolConfig::default(),
                    source: "defaults (fallback after load failure)".to_string(),
                }
            }
        },
        None => LoadedConfig {
            config: TypeSymbolConfig::default(),
            source: "defaults".to_string(),
        },
    }
}

fn main() {
    let cli = Cli::parse();
    let loaded = resolve_config(&cli);
    let config = loaded.config;

    match &cli.command {
        Some(Commands::Test { expression }) => {
            let engine = CoreEngine::new(config);
            let result = engine.format(expression);
            println!("{}", result);
        }
        Some(Commands::Daemon { action }) => match action {
            DaemonAction::Start => {
                println!("Starting daemon...");
                typesymbol_daemon::run(config);
            }
            DaemonAction::Stop => {
                println!("Stop command is not implemented yet (launchd wiring pending).");
            }
            DaemonAction::Status => {
                println!("Status command is not implemented yet (PID/status tracking pending).");
            }
        },
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => {
                println!("Config source: {}", loaded.source);
                println!("mode = {}", config.mode);
                println!("trigger_mode = {}", config.trigger_mode);
                println!("trigger_key = {}", config.trigger_key);
                println!("live_suggestions = {}", config.live_suggestions);
                println!("auto_replace_safe_rules = {}", config.auto_replace_safe_rules);
                println!("aliases = {}", config.aliases.len());
                println!("operators = {}", config.operators.len());
                println!("excluded_apps = {}", config.excluded_apps.len());
                println!(
                    "features = greek_letters={}, operators={}, superscripts={}, subscripts={}, sqrt={}, integrals={}, summations={}, limits={}",
                    config.features.greek_letters,
                    config.features.operators,
                    config.features.superscripts,
                    config.features.subscripts,
                    config.features.sqrt,
                    config.features.integrals,
                    config.features.summations,
                    config.features.limits
                );
            }
            ConfigAction::Init { path, force } => {
                let target = path.clone().unwrap_or_else(default_config_path);
                if let Some(parent) = target.parent() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        eprintln!("Failed to create config directory {}: {}", parent.display(), err);
                        std::process::exit(1);
                    }
                }

                if target.exists() && !force {
                    eprintln!(
                        "Config file already exists at {}. Use --force to overwrite.",
                        target.display()
                    );
                    std::process::exit(1);
                }

                let config_text = render_default_config();
                if let Err(err) = fs::write(&target, config_text) {
                    eprintln!("Failed to write config at {}: {}", target.display(), err);
                    std::process::exit(1);
                }
                println!("Wrote config to {}", target.display());
            }
        },
        Some(Commands::App) => run_app_mode(config),
        None => {
            println!("Starting daemon...");
            typesymbol_daemon::run(config);
        }
    }
}

fn default_config_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("typesymbol")
            .join("config.toml")
    } else {
        PathBuf::from(".")
            .join(".config")
            .join("typesymbol")
            .join("config.toml")
    }
}

fn render_default_config() -> String {
    let cfg = TypeSymbolConfig::default();
    format!(
        r#"mode = "{mode}"
trigger_mode = "{trigger_mode}"
trigger_key = "{trigger_key}"
live_suggestions = {live_suggestions}
auto_replace_safe_rules = {auto_replace_safe_rules}
excluded_apps = [
  "com.apple.Terminal",
  "com.microsoft.VSCode",
  "com.jetbrains.rustrover",
]

[features]
greek_letters = {greek_letters}
operators = {operators}
superscripts = {superscripts}
subscripts = {subscripts}
sqrt = {sqrt}
integrals = {integrals}
summations = {summations}
limits = {limits}

[aliases]
alpha = "{alpha}"
beta = "{beta}"
gamma = "{gamma}"
theta = "{theta}"
lambda = "{lambda}"
pi = "{pi}"
inf = "{inf}"
infinity = "{infinity}"

[operators]
"<->" = "{op_bidir}"
"<-" = "{op_left}"
"->" = "{op_right}"
"!=" = "{op_ne}"
"<=" = "{op_le}"
">=" = "{op_ge}"
"+-" = "{op_pm}"
"#,
        mode = cfg.mode,
        trigger_mode = cfg.trigger_mode,
        trigger_key = cfg.trigger_key,
        live_suggestions = cfg.live_suggestions,
        auto_replace_safe_rules = cfg.auto_replace_safe_rules,
        greek_letters = cfg.features.greek_letters,
        operators = cfg.features.operators,
        superscripts = cfg.features.superscripts,
        subscripts = cfg.features.subscripts,
        sqrt = cfg.features.sqrt,
        integrals = cfg.features.integrals,
        summations = cfg.features.summations,
        limits = cfg.features.limits,
        alpha = cfg.aliases.get("alpha").map(String::as_str).unwrap_or("α"),
        beta = cfg.aliases.get("beta").map(String::as_str).unwrap_or("β"),
        gamma = cfg.aliases.get("gamma").map(String::as_str).unwrap_or("γ"),
        theta = cfg.aliases.get("theta").map(String::as_str).unwrap_or("θ"),
        lambda = cfg.aliases.get("lambda").map(String::as_str).unwrap_or("λ"),
        pi = cfg.aliases.get("pi").map(String::as_str).unwrap_or("π"),
        inf = cfg.aliases.get("inf").map(String::as_str).unwrap_or("∞"),
        infinity = cfg.aliases.get("infinity").map(String::as_str).unwrap_or("∞"),
        op_bidir = cfg.operators.get("<->").map(String::as_str).unwrap_or("↔"),
        op_left = cfg.operators.get("<-").map(String::as_str).unwrap_or("←"),
        op_right = cfg.operators.get("->").map(String::as_str).unwrap_or("→"),
        op_ne = cfg.operators.get("!=").map(String::as_str).unwrap_or("≠"),
        op_le = cfg.operators.get("<=").map(String::as_str).unwrap_or("≤"),
        op_ge = cfg.operators.get(">=").map(String::as_str).unwrap_or("≥"),
        op_pm = cfg.operators.get("+-").map(String::as_str).unwrap_or("±"),
    )
}

fn run_app_mode(config: TypeSymbolConfig) {
    let engine = CoreEngine::new(config);
    println!("TypeSymbol App Mode");
    println!("Type math shorthand and press Enter to convert.");
    println!("Commands: :help, :raw <text>, :quit");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    loop {
        print!("typesymbol> ");
        if let Err(err) = stdout.flush() {
            eprintln!("Failed to flush prompt: {}", err);
            return;
        }

        let Some(line) = lines.next() else {
            println!();
            return;
        };
        let line = match line {
            Ok(l) => l,
            Err(err) => {
                eprintln!("Failed to read input: {}", err);
                continue;
            }
        };

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            ":quit" | ":q" | "quit" | "exit" => {
                println!("Bye.");
                return;
            }
            ":help" => {
                println!("Enter any shorthand expression to see Unicode conversion.");
                println!("Examples:");
                println!("  alpha -> beta");
                println!("  sum_(i=1)^n i^2");
                println!("Commands:");
                println!("  :help         Show this help");
                println!("  :raw <text>   Print input unchanged");
                println!("  :quit         Exit app mode");
            }
            _ => {
                if let Some(raw) = input.strip_prefix(":raw ") {
                    println!("{}", raw);
                } else {
                    println!("{}", engine.format(input));
                }
            }
        }
    }
}
