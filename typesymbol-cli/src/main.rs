use clap::{Parser, Subcommand};
use fs2::FileExt;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
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
    /// Turn TypeSymbol on (enable autostart + start now)
    On,
    /// Turn TypeSymbol off (stop now + disable autostart)
    Off,
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
    Stop,
    Status,
    Enable,
    Disable,
    #[command(hide = true)]
    RunInternal,
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Set {
        /// Config key (e.g. trigger_key, features.integrals, aliases.alpha)
        key: String,
        /// Config value (e.g. enter, true, α)
        value: String,
    },
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
    let config = loaded.config.clone();

    match &cli.command {
        Some(Commands::Test { expression }) => {
            let engine = CoreEngine::new(config);
            let result = engine.format(expression);
            println!("{}", result);
        }
        Some(Commands::Daemon { action }) => match action {
            DaemonAction::Stop => {
                stop_daemon();
            }
            DaemonAction::Status => {
                print_daemon_status();
            }
            DaemonAction::Enable => {
                enable_autostart(cli.config.clone());
            }
            DaemonAction::Disable => {
                disable_autostart();
            }
            DaemonAction::RunInternal => {
                run_daemon_single_instance(config);
            }
        },
        Some(Commands::On) => {
            ensure_background_service(cli.config.clone());
        }
        Some(Commands::Off) => {
            stop_daemon();
            disable_autostart();
        }
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => {
                print_config(&loaded);
            }
            ConfigAction::Set { key, value } => {
                set_config_value(cli.config.clone(), key, value);
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
            ensure_background_service(cli.config.clone());
            if io::stdin().is_terminal() && io::stdout().is_terminal() {
                run_settings_shell(resolve_config(&cli), cli.config.clone());
            }
        }
    }
}

fn ensure_background_service(config_path: Option<PathBuf>) {
    // One command should make TypeSymbol "just work":
    // - keep daemon always-on after reboot/login
    // - start daemon now if it's not already running
    enable_autostart(config_path.clone());
    if read_live_pid().is_none() {
        let _ = start_daemon_background(config_path);
    } else {
        println!("TypeSymbol is already running in background.");
    }
}

fn run_settings_shell(mut loaded: LoadedConfig, config_path: Option<PathBuf>) {
    print_settings_landing(&loaded);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    loop {
        print!("typesymbol> ");
        if stdout.flush().is_err() {
            return;
        }
        let Some(line) = lines.next() else {
            println!();
            return;
        };
        let line = match line {
            Ok(v) => v.trim().to_string(),
            Err(_) => return,
        };
        if line.is_empty() {
            continue;
        }
        match line.as_str() {
            "exit" | "quit" | ":q" => return,
            "help" | "?" => {
                println!("Commands:");
                println!("  on                           Turn TypeSymbol on now");
                println!("  off                          Turn TypeSymbol off now");
                println!("  config show");
                println!("  config set <key> <value>");
                println!("  config init [--force]");
                println!("  daemon status|stop|enable|disable");
                println!("  test <expression>");
                println!("  exit");
            }
            "config show" => print_config(&loaded),
            "config init" => init_config_file(Some(default_config_path()), false),
            "config init --force" => init_config_file(Some(default_config_path()), true),
            "daemon status" => print_daemon_status(),
            "daemon stop" => stop_daemon(),
            "off" => {
                stop_daemon();
                disable_autostart();
            }
            "daemon enable" => enable_autostart(config_path.clone()),
            "daemon disable" => disable_autostart(),
            "on" => {
                ensure_background_service(config_path.clone());
            }
            "config reload" => {
                let cli = Cli {
                    config: config_path.clone(),
                    command: None,
                };
                loaded = resolve_config(&cli);
                println!("Reloaded config from {}", loaded.source);
            }
            _ => {
                if let Some(expr) = line.strip_prefix("test ") {
                    let engine = CoreEngine::new(loaded.config.clone());
                    println!("{}", engine.format(expr.trim()));
                } else if let Some(rest) = line.strip_prefix("config set ") {
                    let mut parts = rest.splitn(2, ' ');
                    let key = parts.next().unwrap_or("").trim();
                    let value = parts.next().unwrap_or("").trim();
                    if key.is_empty() || value.is_empty() {
                        eprintln!("Usage: config set <key> <value>");
                    } else {
                        set_config_value(config_path.clone(), key, value);
                        let cli = Cli {
                            config: config_path.clone(),
                            command: None,
                        };
                        loaded = resolve_config(&cli);
                    }
                } else {
                    println!("Unknown command. Type 'help'.");
                }
            }
        }
    }
}

fn print_settings_landing(loaded: &LoadedConfig) {
    println!("TypeSymbol");
    println!("----------------------------------------");
    println!("Service: {}", if read_live_pid().is_some() { "ON" } else { "OFF" });
    println!("Config: {}", loaded.source);
    println!("Trigger: {}", loaded.config.trigger_key);
    println!("----------------------------------------");
    println!("Quick commands:");
    println!("  on              start background daemon");
    println!("  off             stop daemon");
    println!("  config show     view active config");
    println!("  help            show all commands");
    println!("  exit            close settings shell");
    println!("----------------------------------------");
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

fn daemon_pid_path() -> PathBuf {
    state_dir().join("daemon.pid")
}

fn daemon_lock_path() -> PathBuf {
    state_dir().join("daemon.lock")
}

fn daemon_log_path() -> PathBuf {
    state_dir().join("daemon.log")
}

fn state_dir() -> PathBuf {
    let preferred = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("typesymbol")
    } else {
        PathBuf::from("/tmp").join("typesymbol-state")
    };
    if fs::create_dir_all(&preferred).is_ok() {
        preferred
    } else {
        let fallback = PathBuf::from("/tmp").join("typesymbol-state");
        let _ = fs::create_dir_all(&fallback);
        fallback
    }
}

fn write_pid_file(pid: u32) -> io::Result<()> {
    let path = daemon_pid_path();
    fs::write(path, pid.to_string())
}

fn acquire_daemon_lock() -> Result<fs::File, String> {
    let path = daemon_lock_path();
    let lock = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|err| format!("Failed to open daemon lock {}: {}", path.display(), err))?;
    lock.try_lock_exclusive().map_err(|_| {
        "Another TypeSymbol daemon instance is already running.".to_string()
    })?;
    Ok(lock)
}

fn run_daemon_single_instance(config: TypeSymbolConfig) {
    let _lock = match acquire_daemon_lock() {
        Ok(lock) => lock,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let pid = process::id();
    if let Err(err) = write_pid_file(pid) {
        eprintln!("Warning: failed to write pid file: {}", err);
    }
    typesymbol_daemon::run(config);
}

fn read_pid_file() -> Option<u32> {
    let path = daemon_pid_path();
    let raw = fs::read_to_string(path).ok()?;
    raw.trim().parse::<u32>().ok()
}

fn is_pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn read_live_pid() -> Option<u32> {
    let pid = read_pid_file()?;
    if is_pid_alive(pid) {
        Some(pid)
    } else {
        let _ = fs::remove_file(daemon_pid_path());
        None
    }
}

fn start_daemon_background(config_path: Option<PathBuf>) -> bool {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Failed to resolve executable path: {}", err);
            return false;
        }
    };

    let log_path = daemon_log_path();
    let log = match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Failed to open daemon log {}: {}", log_path.display(), err);
            return false;
        }
    };
    let log_err = match log.try_clone() {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Failed to duplicate daemon log handle: {}", err);
            return false;
        }
    };

    let mut cmd = Command::new(exe);
    if let Some(cfg) = config_path {
        cmd.arg("--config").arg(cfg);
    }
    cmd.arg("daemon")
        .arg("run-internal")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));

    match cmd.spawn() {
        Ok(child) => {
            println!(
                "Started TypeSymbol daemon in background (pid {}).",
                child.id()
            );
            println!("Logs: {}", log_path.display());
            true
        }
        Err(err) => {
            eprintln!("Failed to start daemon in background: {}", err);
            false
        }
    }
}

fn stop_daemon() {
    let Some(pid) = read_live_pid() else {
        println!("TypeSymbol daemon is not running.");
        return;
    };

    let status = Command::new("kill").arg(pid.to_string()).status();
    match status {
        Ok(s) if s.success() => {
            println!("Sent stop signal to daemon pid {}.", pid);
            let _ = fs::remove_file(daemon_pid_path());
        }
        Ok(_) => {
            eprintln!("Failed to stop daemon pid {}.", pid);
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Failed to execute kill for pid {}: {}", pid, err);
            process::exit(1);
        }
    }
}

fn print_daemon_status() {
    if let Some(pid) = read_live_pid() {
        println!("TypeSymbol daemon is running (pid {}).", pid);
    } else {
        println!("TypeSymbol daemon is not running.");
    }
}

fn launch_agent_label() -> &'static str {
    "com.typesymbol.daemon"
}

fn launch_agent_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{}.plist", launch_agent_label()))
    } else {
        PathBuf::from("/tmp").join(format!("{}.plist", launch_agent_label()))
    }
}

fn enable_autostart(config_path: Option<PathBuf>) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Failed to resolve executable path: {}", err);
            return;
        }
    };
    let agent_path = launch_agent_path();
    if let Some(parent) = agent_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "Failed to create LaunchAgents directory {}: {}",
                parent.display(),
                err
            );
            return;
        }
    }

    let log_path = daemon_log_path();
    let cfg_arg = config_path
        .map(|p| format!("<string>--config</string><string>{}</string>", xml_escape(&p.display().to_string())))
        .unwrap_or_default();
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    {cfg_arg}
    <string>daemon</string>
    <string>run-internal</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{log}</string>
</dict>
</plist>
"#,
        label = launch_agent_label(),
        exe = xml_escape(&exe.display().to_string()),
        cfg_arg = cfg_arg,
        log = xml_escape(&log_path.display().to_string()),
    );

    if let Err(err) = fs::write(&agent_path, plist) {
        eprintln!("Failed to write launch agent {}: {}", agent_path.display(), err);
        return;
    }

    let _ = Command::new("launchctl")
        .arg("bootout")
        .arg(format!("gui/{}/{}", current_uid(), launch_agent_label()))
        .status();
    let status = Command::new("launchctl")
        .arg("bootstrap")
        .arg(format!("gui/{}", current_uid()))
        .arg(&agent_path)
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("Autostart enabled.");
            println!("LaunchAgent: {}", agent_path.display());
            println!("Daemon will stay on after login/reboot.");
        }
        _ => {
            let legacy = Command::new("launchctl").arg("load").arg(&agent_path).status();
            if matches!(legacy, Ok(s) if s.success()) {
                println!("Autostart enabled (legacy launchctl load).");
            } else {
                eprintln!("Autostart plist written, but launchctl load failed.");
                eprintln!("Try manually: launchctl bootstrap gui/$(id -u) {}", agent_path.display());
            }
        }
    }
}

fn disable_autostart() {
    let agent_path = launch_agent_path();
    let _ = Command::new("launchctl")
        .arg("bootout")
        .arg(format!("gui/{}/{}", current_uid(), launch_agent_label()))
        .status();
    let _ = Command::new("launchctl").arg("unload").arg(&agent_path).status();
    if agent_path.exists() {
        if let Err(err) = fs::remove_file(&agent_path) {
            eprintln!(
                "Failed to remove launch agent {}: {}",
                agent_path.display(),
                err
            );
            return;
        }
    }
    println!("Autostart disabled.");
}

fn current_uid() -> String {
    std::env::var("UID").unwrap_or_else(|_| "501".to_string())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
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


fn print_config(loaded: &LoadedConfig) {
    let config = &loaded.config;
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

fn init_config_file(path: Option<PathBuf>, force: bool) {
    let target = path.unwrap_or_else(default_config_path);
    if let Some(parent) = target.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("Failed to create config directory {}: {}", parent.display(), err);
            return;
        }
    }

    if target.exists() && !force {
        eprintln!(
            "Config file already exists at {}. Use --force to overwrite.",
            target.display()
        );
        return;
    }

    let config_text = render_default_config();
    if let Err(err) = fs::write(&target, config_text) {
        eprintln!("Failed to write config at {}: {}", target.display(), err);
        return;
    }
    println!("Wrote config to {}", target.display());
}

fn set_config_value(config_path_arg: Option<PathBuf>, key: &str, value: &str) {
    let path = config_path_arg.unwrap_or_else(default_config_path);
    if !path.exists() {
        init_config_file(Some(path.clone()), false);
    }

    let path_str = path.to_string_lossy().to_string();
    let mut cfg = match load_config(&path_str) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to load config from {}: {}", path.display(), err);
            return;
        }
    };

    if let Err(err) = apply_config_update(&mut cfg, key, value) {
        eprintln!("{}", err);
        return;
    }

    let out = match toml::to_string_pretty(&cfg) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to serialize config: {}", err);
            return;
        }
    };

    if let Err(err) = fs::write(&path, out) {
        eprintln!("Failed to write config at {}: {}", path.display(), err);
        return;
    }

    println!("Updated {} in {}", key, path.display());
}

fn apply_config_update(cfg: &mut TypeSymbolConfig, key: &str, value: &str) -> Result<(), String> {
    match key {
        "mode" => cfg.mode = value.to_string(),
        "trigger_mode" => cfg.trigger_mode = value.to_string(),
        "trigger_key" => cfg.trigger_key = value.to_string(),
        "live_suggestions" => cfg.live_suggestions = parse_bool(value)?,
        "auto_replace_safe_rules" => cfg.auto_replace_safe_rules = parse_bool(value)?,
        k if k.starts_with("features.") => {
            let f = &k["features.".len()..];
            let v = parse_bool(value)?;
            match f {
                "greek_letters" => cfg.features.greek_letters = v,
                "operators" => cfg.features.operators = v,
                "superscripts" => cfg.features.superscripts = v,
                "subscripts" => cfg.features.subscripts = v,
                "sqrt" => cfg.features.sqrt = v,
                "integrals" => cfg.features.integrals = v,
                "summations" => cfg.features.summations = v,
                "limits" => cfg.features.limits = v,
                _ => return Err(format!("Unknown feature key: {}", k)),
            }
        }
        k if k.starts_with("aliases.") => {
            let alias = &k["aliases.".len()..];
            if alias.is_empty() {
                return Err("aliases.<name> key cannot be empty".to_string());
            }
            cfg.aliases.insert(alias.to_string(), value.to_string());
        }
        k if k.starts_with("operators.") => {
            let op = &k["operators.".len()..];
            if op.is_empty() {
                return Err("operators.<token> key cannot be empty".to_string());
            }
            cfg.operators.insert(op.to_string(), value.to_string());
        }
        "excluded_apps.add" => {
            cfg.excluded_apps.insert(value.to_string());
        }
        "excluded_apps.remove" => {
            cfg.excluded_apps.remove(value);
        }
        _ => {
            return Err(format!(
                "Unknown config key '{}'. Supported: mode, trigger_mode, trigger_key, live_suggestions, auto_replace_safe_rules, features.*, aliases.*, operators.*, excluded_apps.add/remove",
                key
            ))
        }
    }
    Ok(())
}

fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("Expected boolean value, got '{}'", raw)),
    }
}
