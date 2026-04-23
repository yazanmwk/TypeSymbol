use clap::{Parser, Subcommand};
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
    command: Commands,
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
}

#[derive(Subcommand)]
enum DaemonAction {
    Start,
    Stop,
    Status,
}

fn resolve_config(cli: &Cli) -> TypeSymbolConfig {
    match &cli.config {
        Some(path) => match load_config(path.to_str().unwrap_or_default()) {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!(
                    "Failed to load config from {}: {}. Falling back to defaults.",
                    path.display(),
                    err
                );
                TypeSymbolConfig::default()
            }
        },
        None => TypeSymbolConfig::default(),
    }
}

fn main() {
    let cli = Cli::parse();
    let config = resolve_config(&cli);

    match &cli.command {
        Commands::Test { expression } => {
            let engine = CoreEngine::new(config);
            let result = engine.format(expression);
            println!("{}", result);
        }
        Commands::Daemon { action } => match action {
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
    }
}
