use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use fs2::FileExt;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::Duration;
use typesymbol_config::{load_config, TypeSymbolConfig};
use typesymbol_core::CoreEngine;

const COMPACT_MIN_WIDTH: usize = 70;
const DEFAULT_TERMINAL_WIDTH: usize = 78;
const PANEL_INNER_WIDTH: usize = 62;
const PROMPT_SYMBOL: &str = "∫";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "typesymbol",
    version = APP_VERSION,
    about = "System-wide math shorthand daemon"
)]
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
    /// Check for updates or update this installation
    Update {
        /// Only check whether an update is available
        #[arg(long)]
        check: bool,
    },
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Dashboard,
    Help,
    Config,
}

struct TuiApp {
    screen: Screen,
    selected: usize,
    config_selected: usize,
    service_running: bool,
    loaded: LoadedConfig,
    config_path: Option<PathBuf>,
    capture_trigger: bool,
    flash: Option<String>,
    error: Option<String>,
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
                        eprintln!(
                            "Failed to create config directory {}: {}",
                            parent.display(),
                            err
                        );
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
        Some(Commands::Update { check }) => run_self_update(*check),
        None => {
            if cfg!(windows) {
                run_settings_shell(resolve_config(&cli), cli.config.clone());
            } else if io::stdin().is_terminal() && io::stdout().is_terminal() {
                if let Err(err) = run_interactive_tui(resolve_config(&cli), cli.config.clone()) {
                    eprintln!("Interactive shell failed: {}", err);
                    process::exit(1);
                }
            } else {
                ensure_background_service(cli.config.clone());
            }
        }
    }
}

fn run_interactive_tui(loaded: LoadedConfig, config_path: Option<PathBuf>) -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.autoresize()?;
    terminal.clear()?;

    let mut app = TuiApp {
        screen: Screen::Dashboard,
        selected: 0,
        config_selected: 0,
        service_running: read_live_pid().is_some(),
        loaded,
        config_path,
        capture_trigger: false,
        flash: None,
        error: None,
    };

    let result = (|| -> io::Result<()> {
        loop {
            terminal.draw(|f| draw_tui(f, &app))?;
            if event::poll(Duration::from_millis(120))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        if handle_tui_key(&mut app, key.code)? {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {
                        // Keep ratatui's cached viewport synchronized with terminal dimensions.
                        terminal.autoresize()?;
                        terminal.clear()?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn handle_tui_key(app: &mut TuiApp, code: KeyCode) -> io::Result<bool> {
    if app.capture_trigger {
        match code {
            KeyCode::Esc | KeyCode::Backspace => {
                app.capture_trigger = false;
                app.flash = Some("Trigger change canceled".to_string());
            }
            other => {
                if let Some(trigger) = trigger_key_from_code(other) {
                    match set_config_value_silent(app.config_path.clone(), "trigger_key", &trigger) {
                        Ok(_) => {
                            app.loaded.config.trigger_key = trigger.clone();
                            app.flash = Some(format!("Trigger key changed to {}", trigger));
                        }
                        Err(err) => app.error = Some(err),
                    }
                    app.capture_trigger = false;
                } else {
                    app.error = Some("Unsupported key for trigger".to_string());
                }
            }
        }
        return Ok(false);
    }

    match code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('?') => {
            app.screen = Screen::Help;
            app.error = None;
            app.flash = None;
        }
        KeyCode::Esc | KeyCode::Backspace => {
            if app.screen == Screen::Dashboard {
                return Ok(true);
            }
            app.screen = Screen::Dashboard;
            app.error = None;
            app.flash = None;
        }
        KeyCode::Up | KeyCode::Char('k') => match app.screen {
            Screen::Dashboard => app.selected = app.selected.saturating_sub(1),
            Screen::Config => app.config_selected = app.config_selected.saturating_sub(1),
            Screen::Help => {}
        },
        KeyCode::Down | KeyCode::Char('j') => match app.screen {
            Screen::Dashboard => app.selected = (app.selected + 1).min(4),
            Screen::Config => app.config_selected = (app.config_selected + 1).min(3),
            Screen::Help => {}
        },
        KeyCode::Enter => match app.screen {
            Screen::Dashboard => {
                if app.selected == 4 {
                    return Ok(true);
                }
                handle_dashboard_action(app)
            }
            Screen::Config => handle_config_action(app)?,
            Screen::Help => app.screen = Screen::Dashboard,
        },
        _ => {}
    }
    Ok(false)
}

fn handle_dashboard_action(app: &mut TuiApp) {
    app.error = None;
    app.flash = None;
    match app.selected {
        0 => {
            if app.service_running {
                app.flash = Some("Service already running".to_string());
                return;
            }
            match ensure_background_service_silent(app.config_path.clone()) {
                Ok(msg) => {
                    app.service_running = read_live_pid().is_some();
                    app.flash = Some(msg.to_string());
                }
                Err(err) => app.error = Some(err),
            }
        }
        1 => {
            if !app.service_running {
                app.flash = Some("Service already stopped".to_string());
                return;
            }
            match stop_daemon_silent() {
                Ok(_) => {
                    app.service_running = false;
                    app.flash = Some("Background daemon stopped".to_string());
                }
                Err(err) => app.error = Some(err),
            }
        }
        2 => app.screen = Screen::Config,
        3 => app.screen = Screen::Help,
        4 => {}
        _ => {}
    }
}

fn handle_config_action(app: &mut TuiApp) -> io::Result<()> {
    app.error = None;
    app.flash = None;
    match app.config_selected {
        0 => {
            app.capture_trigger = true;
            app.flash = Some("Press a key to set the trigger".to_string());
        }
        1 => {
            let next_mode = if app.loaded.config.mode == "unicode" {
                "raw"
            } else {
                "unicode"
            };
            match set_config_value_silent(app.config_path.clone(), "mode", next_mode) {
                Ok(_) => {
                    app.loaded.config.mode = next_mode.to_string();
                    app.flash = Some(format!(
                        "Unicode output {}",
                        if next_mode == "unicode" {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ));
                }
                Err(err) => app.error = Some(err),
            }
        }
        2 => {
            let target = app.config_path.clone().unwrap_or_else(default_config_path);
            let content = render_default_config();
            match fs::write(&target, content) {
                Ok(_) => {
                    let reloaded = load_config(&target.to_string_lossy())
                        .map_err(|err| err.to_string())
                        .unwrap_or_else(|_| TypeSymbolConfig::default());
                    app.loaded = LoadedConfig {
                        config: reloaded,
                        source: format!("file ({})", target.display()),
                    };
                    app.flash = Some("Config reset to defaults".to_string());
                }
                Err(err) => app.error = Some(format!(
                    "Failed to reset config at {}: {}",
                    target.display(),
                    err
                )),
            }
        }
        3 => app.screen = Screen::Dashboard,
        _ => {}
    }
    Ok(())
}

fn draw_tui(frame: &mut ratatui::Frame<'_>, app: &TuiApp) {
    let size = frame.area();
    let color_enabled = supports_color();
    let body_min_height = 13u16;
    let footer_height = 1u16;
    let header_border = 2u16;
    let header_line_budget = size
        .height
        .saturating_sub(body_min_height.saturating_add(footer_height).saturating_add(header_border));
    let header_lines =
        build_tui_header_lines(size.width, header_line_budget.max(3), color_enabled);
    let header_height = (header_lines.len() as u16).saturating_add(header_border).max(6);
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(body_min_height),
            Constraint::Length(1),
        ])
        .split(size);

    let header = Paragraph::new(header_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if color_enabled {
                Style::default().fg(Color::Rgb(186, 83, 230))
            } else {
                Style::default()
            }),
    );
    frame.render_widget(header, root[0]);

    match app.screen {
        Screen::Dashboard => draw_dashboard(frame, root[1], app, color_enabled),
        Screen::Help => draw_help(frame, root[1], color_enabled),
        Screen::Config => draw_config(frame, root[1], app, color_enabled),
    }

    let help_text = "↑/↓ move  Enter select  Esc back  ? help  q quit";
    let version_text = format!("v{}", APP_VERSION);
    let footer_width = root[2].width as usize;
    let gap = footer_width.saturating_sub(help_text.len() + version_text.len() + 2).max(2);
    let footer_line = Line::from(vec![
        Span::styled(
            help_text,
            if color_enabled {
                Style::default().fg(Color::Rgb(210, 150, 225))
            } else {
                Style::default()
            },
        ),
        Span::raw(" ".repeat(gap)),
        Span::styled(
            version_text,
            if color_enabled {
                Style::default()
                    .fg(Color::Rgb(255, 92, 203))
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            },
        ),
    ]);
    let footer = Paragraph::new(footer_line)
        .block(
            Block::default().borders(Borders::TOP).border_style(if color_enabled {
                Style::default().fg(Color::Rgb(186, 83, 230))
            } else {
                Style::default()
            }),
        );
    frame.render_widget(footer, root[2]);

    if let Some(message) = &app.flash {
        draw_modal(frame, size, "Status", message, false, color_enabled);
    }
    if let Some(message) = &app.error {
        draw_modal(frame, size, "Error", message, true, color_enabled);
    }
    if app.capture_trigger {
        draw_modal(
            frame,
            size,
            "Change Trigger",
            "Press a key (Enter/Tab/Space/Letter). Esc to cancel.",
            false,
            color_enabled,
        );
    }
}

fn trigger_key_from_code(code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Enter => Some("enter".to_string()),
        KeyCode::Tab => Some("tab".to_string()),
        KeyCode::Char(' ') => Some("space".to_string()),
        KeyCode::Char(c) if c.is_ascii_alphanumeric() => Some(c.to_string()),
        _ => None,
    }
}

fn build_tui_header_lines(width: u16, line_budget: u16, color_enabled: bool) -> Vec<Line<'static>> {
    // Keep room for dashboard controls first, then spend remaining lines on branding.
    // This keeps the full logo visible whenever possible without clipping menu content.
    if width >= 110 {
        let type_rows = [
            "████████╗██╗   ██╗██████╗ ███████╗",
            "╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝",
            "   ██║    ╚████╔╝ ██████╔╝█████╗  ",
            "   ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ",
            "   ██║      ██║   ██║     ███████╗",
            "   ╚═╝      ╚═╝   ╚═╝     ╚══════╝",
        ];
        let symbol_rows = [
            "███████╗██╗   ██╗███╗   ███╗██████╗  ██████╗ ██╗     ",
            "██╔════╝╚██╗ ██╔╝████╗ ████║██╔══██╗██╔═══██╗██║     ",
            "███████╗ ╚████╔╝ ██╔████╔██║██████╔╝██║   ██║██║     ",
            "╚════██║  ╚██╔╝  ██║╚██╔╝██║██╔══██╗██║   ██║██║     ",
            "███████║   ██║   ██║ ╚═╝ ██║██████╔╝╚██████╔╝███████╗",
            "╚══════╝   ╚═╝   ╚═╝     ╚═╝╚═════╝  ╚═════╝ ╚══════╝",
        ];
        let mut lines: Vec<Line<'static>> = Vec::new();
        for row in type_rows {
            let line = if color_enabled {
                gradient_wordmark_line(row)
            } else {
                Line::from(Span::raw(row))
            };
            lines.push(line.alignment(Alignment::Center));
        }
        lines.push(Line::from(""));
        for row in symbol_rows {
            let line = if color_enabled {
                gradient_wordmark_line(row)
            } else {
                Line::from(Span::raw(row))
            };
            lines.push(line.alignment(Alignment::Center));
        }
        lines.push(Line::from(""));
        let tagline = vec![
            Span::styled(
                "∫ ",
                if color_enabled {
                    Style::default()
                        .fg(Color::LightMagenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                "system-wide math typing helper",
                if color_enabled {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default()
                },
            ),
        ];
        lines.push(Line::from(tagline).alignment(Alignment::Center));
        if (lines.len() as u16) <= line_budget {
            return lines;
        }
    }

    if width >= 78 {
        let type_rows = [
            "████████╗██╗   ██╗██████╗ ███████╗",
            "╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝",
            "   ██║    ╚████╔╝ ██████╔╝█████╗  ",
            "   ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ",
            "   ██║      ██║   ██║     ███████╗",
            "   ╚═╝      ╚═╝   ╚═╝     ╚══════╝",
        ];
        let mut lines: Vec<Line<'static>> = Vec::new();
        for row in type_rows {
            let line = if color_enabled {
                gradient_wordmark_line(row)
            } else {
                Line::from(Span::raw(row))
            };
            lines.push(line.alignment(Alignment::Center));
        }
        lines.push(Line::from(""));
        lines.push(
            Line::from(vec![
                Span::styled(
                    "∫ ",
                    if color_enabled {
                        Style::default()
                            .fg(Color::LightMagenta)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(
                    "TypeSymbol",
                    if color_enabled {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ])
            .alignment(Alignment::Center),
        );
        lines.push(
            Line::from(Span::styled(
                "system-wide math typing helper",
                if color_enabled {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default()
                },
            ))
            .alignment(Alignment::Center),
        );
        if (lines.len() as u16) <= line_budget {
            return lines;
        }
    }

    vec![
        Line::from(vec![
            Span::styled(
                "∫ ",
                if color_enabled {
                    Style::default()
                        .fg(Color::LightMagenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                "TypeSymbol",
                if color_enabled {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
        ])
        .alignment(Alignment::Center),
        Line::from(Span::styled(
            "system-wide math typing helper",
            if color_enabled {
                Style::default().fg(Color::Gray)
            } else {
                Style::default()
            },
        ))
        .alignment(Alignment::Center),
    ]
}

fn gradient_wordmark_line(row: &str) -> Line<'static> {
    let palette = [
        (255, 92, 203),
        (235, 143, 224),
        (190, 170, 235),
        (130, 205, 240),
        (70, 230, 240),
    ];
    let chars: Vec<char> = row.chars().collect();
    let span_count = chars.len().max(1);
    let mut spans = Vec::with_capacity(span_count);
    for (idx, ch) in chars.into_iter().enumerate() {
        if ch == ' ' {
            spans.push(Span::raw(" "));
            continue;
        }
        let t = if span_count <= 1 {
            0.0
        } else {
            idx as f32 / (span_count - 1) as f32
        };
        let (r, g, b) = gradient_rgb_at(&palette, t);
        let mut style = Style::default()
            .fg(Color::Rgb(r, g, b))
            .add_modifier(Modifier::BOLD);
        if matches!(ch, '╔' | '╗' | '╚' | '╝' | '═') {
            style = style.add_modifier(Modifier::DIM);
        }
        spans.push(Span::styled(ch.to_string(), style));
    }
    Line::from(spans)
}

fn gradient_rgb_at(palette: &[(u8, u8, u8)], t: f32) -> (u8, u8, u8) {
    if palette.len() <= 1 {
        return palette[0];
    }
    let clamped = t.clamp(0.0, 1.0);
    let scaled = clamped * (palette.len() - 1) as f32;
    let left = scaled.floor() as usize;
    let right = (left + 1).min(palette.len() - 1);
    let frac = scaled - left as f32;
    let (lr, lg, lb) = palette[left];
    let (rr, rg, rb) = palette[right];
    let lerp = |a: u8, b: u8| -> u8 { ((a as f32) + (b as f32 - a as f32) * frac) as u8 };
    (lerp(lr, rr), lerp(lg, rg), lerp(lb, rb))
}

fn draw_dashboard(
    frame: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    app: &TuiApp,
    color_enabled: bool,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(25), Constraint::Min(20)])
        .split(area);
    let integral = [
        "       ▄████",
        "      ██",
        "      ██",
        "     ██",
        "     ██",
        "     ██",
        "    ██",
        "    ██",
        " ██ ██",
        "  ███",
    ];
    let left_lines: Vec<Line> = integral
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let color = if color_enabled {
                let (r, g, b) = gradient_rgb_at(
                    &[
                        (170, 220, 245), // pale blue
                        (190, 170, 235), // lavender
                        (240, 120, 205), // pink-magenta
                    ],
                    if integral.len() <= 1 {
                        0.0
                    } else {
                        i as f32 / (integral.len() - 1) as f32
                    },
                );
                Color::Rgb(r, g, b)
            } else {
                Color::Reset
            };
            Line::from(Span::styled(*row, Style::default().fg(color)))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(left_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if color_enabled {
                    Style::default().fg(Color::Rgb(160, 95, 190))
                } else {
                    Style::default()
                }),
        ),
        cols[0],
    );

    let items = ["Start daemon", "Stop daemon", "Configuration", "Help", "Quit"];
    let mut lines = vec![
        Line::from(Span::styled(
            "Status",
            if color_enabled {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            },
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Service   ",
                if color_enabled {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                "●",
                if app.service_running {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else if color_enabled {
                    Style::default().fg(Color::Rgb(200, 120, 120)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::raw(" "),
            Span::styled(
                if app.service_running { "running" } else { "stopped" },
                if color_enabled {
                    Style::default().fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Config    ",
                if color_enabled {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                app.loaded.source.clone(),
                if color_enabled {
                    Style::default().fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Trigger   ",
                if color_enabled {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                format!("↵ {}", app.loaded.config.trigger_key),
                if color_enabled {
                    Style::default().fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Menu",
            if color_enabled {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            },
        )),
    ];
    for (idx, item) in items.iter().enumerate() {
        let disabled = (idx == 0 && app.service_running) || (idx == 1 && !app.service_running);
        let marker = if idx == app.selected { "›" } else { " " };
        let mut style = Style::default().fg(Color::White);
        if disabled {
            style = Style::default().fg(Color::DarkGray);
        } else if idx == app.selected {
            style = Style::default()
                .fg(if color_enabled {
                    Color::Rgb(255, 92, 203)
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD);
        }
        let marker_style = if idx == app.selected && !disabled && color_enabled {
            Style::default()
                .fg(Color::Rgb(255, 92, 203))
                .add_modifier(Modifier::BOLD)
        } else if disabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", marker), marker_style),
            Span::styled(*item, style),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if color_enabled {
                    Style::default().fg(Color::Rgb(186, 83, 230))
                } else {
                    Style::default()
                }),
        ),
        cols[1],
    );
}

fn draw_help(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, color_enabled: bool) {
    let lines = vec![
        Line::from(Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  ↑/↓        Move selection"),
        Line::from("  Enter      Select item"),
        Line::from("  Esc        Go back"),
        Line::from("  q          Quit"),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Start daemon        Turn TypeSymbol on"),
        Line::from("  Stop daemon         Turn TypeSymbol off"),
        Line::from("  Configuration       View or edit settings"),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if color_enabled {
                    Style::default().fg(Color::Rgb(170, 95, 200))
                } else {
                    Style::default()
                }),
        ),
        area,
    );
}

fn draw_config(
    frame: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    app: &TuiApp,
    color_enabled: bool,
) {
    let items = ["Change trigger", "Toggle Unicode", "Reset config", "Back"];
    let mut lines = vec![
        Line::from(Span::styled("Configuration", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!("  Config profile     {}", app.loaded.source)),
        Line::from(format!("  Trigger key        {}", app.loaded.config.trigger_key)),
        Line::from(format!(
            "  Unicode output     {}",
            if app.loaded.config.mode == "unicode" {
                "enabled"
            } else {
                "disabled"
            }
        )),
        Line::from(format!(
            "  Daemon autostart   {}",
            if launch_agent_path().exists() { "enabled" } else { "disabled" }
        )),
        Line::from(""),
    ];
    for (idx, item) in items.iter().enumerate() {
        let style = if idx == app.config_selected {
            Style::default()
                .fg(if color_enabled {
                    Color::Rgb(255, 92, 203)
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::styled(
                if idx == app.config_selected { "  › " } else { "    " },
                if idx == app.config_selected && color_enabled {
                    Style::default()
                        .fg(Color::Rgb(255, 92, 203))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(*item, style),
        ]));
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if color_enabled {
                    Style::default().fg(Color::Rgb(170, 95, 200))
                } else {
                    Style::default()
                }),
        ),
        area,
    );
}

fn draw_modal(
    frame: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    title: &str,
    message: &str,
    is_error: bool,
    color_enabled: bool,
) {
    let popup = centered_rect(60, 20, area);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(vec![Line::from(""), Line::from(message), Line::from(""), Line::from("Press Esc/Backspace")])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(if color_enabled {
                        Style::default().fg(if is_error {
                            Color::Rgb(255, 92, 140)
                        } else {
                            Color::Rgb(225, 110, 215)
                        })
                    } else {
                        Style::default()
                    }),
            ),
        popup,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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

fn ensure_background_service_silent(config_path: Option<PathBuf>) -> Result<&'static str, String> {
    enable_autostart_silent(config_path.clone())?;
    if read_live_pid().is_none() {
        start_daemon_background_silent(config_path)?;
        Ok("Background daemon started")
    } else {
        Ok("Background daemon already running")
    }
}

fn run_settings_shell(mut loaded: LoadedConfig, config_path: Option<PathBuf>) {
    print_settings_landing(&loaded);
    let theme = RenderTheme::detect();
    let width = terminal_width().max(48);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    loop {
        print!("{}", render_prompt());
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
                print!("{}", render_help(&theme, width));
            }
            "config show" => {
                print!("{}", render_active_config_panel(&theme, width, &loaded));
            }
            "config init" => init_config_file(Some(default_config_path()), false),
            "config init --force" => init_config_file(Some(default_config_path()), true),
            "daemon status" => {
                let status = if read_live_pid().is_some() {
                    "● running"
                } else {
                    "● stopped"
                };
                print!(
                    "{}",
                    render_info_panel(&theme, width, "Service", &[("Status", status)])
                );
            }
            "daemon stop" => {
                stop_daemon();
                print!(
                    "{}",
                    render_info_panel(
                        &theme,
                        width,
                        "Service",
                        &[("Status", "● stopped"), ("Message", "Stop background daemon")]
                    )
                );
            }
            "off" => {
                stop_daemon();
                disable_autostart();
                print!(
                    "{}",
                    render_info_panel(
                        &theme,
                        width,
                        "Service",
                        &[("Status", "● stopped"), ("Message", "Stop daemon")]
                    )
                );
            }
            "daemon enable" => {
                enable_autostart(config_path.clone());
                print!(
                    "{}",
                    render_info_panel(
                        &theme,
                        width,
                        "Service",
                        &[("Status", "enabled"), ("Message", "Enable daemon autostart")]
                    )
                );
            }
            "daemon disable" => {
                disable_autostart();
                print!(
                    "{}",
                    render_info_panel(
                        &theme,
                        width,
                        "Service",
                        &[("Status", "disabled"), ("Message", "Disable daemon autostart")]
                    )
                );
            }
            "on" => {
                ensure_background_service(config_path.clone());
                print!(
                    "{}",
                    render_info_panel(
                        &theme,
                        width,
                        "Service",
                        &[("Status", "● running"), ("Message", "Start background daemon")]
                    )
                );
            }
            "config reload" => {
                let cli = Cli {
                    config: config_path.clone(),
                    command: None,
                };
                loaded = resolve_config(&cli);
                print!(
                    "{}",
                    render_info_panel(
                        &theme,
                        width,
                        "Configuration",
                        &[("Status", "reloaded"), ("Source", &loaded.source)]
                    )
                );
            }
            "update" => run_self_update(false),
            "update check" => run_self_update(true),
            _ => {
                if let Some(expr) = line.strip_prefix("test ") {
                    let engine = CoreEngine::new(loaded.config.clone());
                    println!("{}", engine.format(expr.trim()));
                } else if let Some(rest) = line.strip_prefix("config set ") {
                    let mut parts = rest.splitn(2, ' ');
                    let key = parts.next().unwrap_or("").trim();
                    let value = parts.next().unwrap_or("").trim();
                    if key.is_empty() || value.is_empty() {
                        print!(
                            "{}",
                            render_error_panel(
                                &theme,
                                width,
                                "config set",
                                "Missing key or value",
                                Some("config set <key> <value>")
                            )
                        );
                    } else {
                        set_config_value(config_path.clone(), key, value);
                        let cli = Cli {
                            config: config_path.clone(),
                            command: None,
                        };
                        loaded = resolve_config(&cli);
                    }
                } else {
                    print!(
                        "{}",
                        render_error_panel(
                            &theme,
                            width,
                            "command",
                            "Unknown command",
                            Some("help")
                        )
                    );
                }
            }
        }
    }
}

fn print_settings_landing(loaded: &LoadedConfig) {
    let service_on = read_live_pid().is_some();
    let theme = RenderTheme::detect();
    let width = terminal_width().max(48);
    print!("{}", render_header(&theme, width));
    print!(
        "{}",
        render_dashboard(
            &theme,
            width,
            service_on,
            &loaded.source,
            &loaded.config.trigger_key,
        )
    );
}

#[derive(Clone, Copy)]
struct RenderTheme {
    color: bool,
    unicode: bool,
}

impl RenderTheme {
    fn detect() -> Self {
        Self {
            color: supports_color(),
            unicode: supports_unicode(),
        }
    }
}

struct BoxChars {
    tl: &'static str,
    tr: &'static str,
    bl: &'static str,
    br: &'static str,
    h: &'static str,
    v: &'static str,
}

struct CommandItem<'a> {
    command: &'a str,
    description: &'a str,
}

struct CommandGroup<'a> {
    title: &'a str,
    items: &'a [CommandItem<'a>],
}

impl BoxChars {
    fn for_theme(theme: &RenderTheme) -> Self {
        if theme.unicode {
            Self {
                tl: "╭",
                tr: "╮",
                bl: "╰",
                br: "╯",
                h: "─",
                v: "│",
            }
        } else {
            Self {
                tl: "+",
                tr: "+",
                bl: "+",
                br: "+",
                h: "-",
                v: "|",
            }
        }
    }
}

fn supports_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var("TERM").map(|v| v == "dumb").unwrap_or(false) {
        return false;
    }
    io::stdout().is_terminal()
}

fn supports_unicode() -> bool {
    if std::env::var_os("TYPESYMBOL_ASCII").is_some() {
        return false;
    }
    let mut locale = String::new();
    if let Ok(v) = std::env::var("LC_ALL") {
        locale.push_str(&v);
    }
    if let Ok(v) = std::env::var("LC_CTYPE") {
        locale.push_str(&v);
    }
    if let Ok(v) = std::env::var("LANG") {
        locale.push_str(&v);
    }
    locale.to_uppercase().contains("UTF-8")
}

fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_TERMINAL_WIDTH)
}

fn render_header(theme: &RenderTheme, width: usize) -> String {
    let inner_width = panel_inner_width(width);
    let compact = width < COMPACT_MIN_WIDTH;
    let mut lines = Vec::new();
    lines.push(String::new());
    if compact {
        lines.push(format!("  {PROMPT_SYMBOL} TypeSymbol"));
        lines.push("  system-wide math typing helper".to_string());
    } else {
        lines.extend([
            "        ████████╗██╗   ██╗██████╗ ███████╗".to_string(),
            "        ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝".to_string(),
            "           ██║    ╚████╔╝ ██████╔╝█████╗".to_string(),
            "           ██║     ╚██╔╝  ██╔═══╝ ██╔══╝".to_string(),
            "           ██║      ██║   ██║     ███████╗".to_string(),
            "           ╚═╝      ╚═╝   ╚═╝     ╚══════╝".to_string(),
            String::new(),
            format!("                    {PROMPT_SYMBOL} TypeSymbol"),
            "            system-wide math typing helper".to_string(),
        ]);
    }
    let version_badge = format!("v{}", APP_VERSION);
    lines.push(format!("{:>width$}", version_badge, width = inner_width.saturating_sub(2)));
    lines.push(String::new());
    let mut out = render_box(&lines, inner_width, theme);
    out.push('\n');
    if theme.color {
        out = colorize_header(out);
    }
    out
}

fn render_dashboard(
    theme: &RenderTheme,
    width: usize,
    service_on: bool,
    config_name: &str,
    trigger: &str,
) -> String {
    let inner_width = panel_inner_width(width);
    let compact = width < COMPACT_MIN_WIDTH;
    let service_dot = if theme.unicode { "●" } else { "*" };
    let service_state = if service_on { "on" } else { "off" };
    let config_summary = if config_name == "defaults" {
        "defaults".to_string()
    } else {
        "custom".to_string()
    };
    let trigger_label = if theme.unicode {
        format!("↵ {}", trigger)
    } else {
        format!("enter {}", trigger)
    };
    let mut lines = Vec::new();
    lines.push(String::new());
    if compact {
        lines.push("  Status".to_string());
        lines.push(format!("  Service      {service_dot} {service_state}"));
        lines.push(format!("  Config       {config_summary}"));
        lines.push(format!("  Trigger      {trigger_label}"));
        lines.push(String::new());
        lines.push("  Quick Commands".to_string());
    } else {
        let art = vec![
            "       ▄████",
            "      ██",
            "      ██",
            "     ██",
            "     ██",
            "     ██",
            "    ██",
            "    ██",
            " ██ ██",
            "  ███",
        ];
        let art = if theme.color {
            render_integral_gradient(&art, supports_truecolor())
        } else {
            art.iter().map(|line| line.to_string()).collect()
        };
        let integral_col = 6usize;
        let text_col = 20usize;
        let mut right_rows = vec![String::new(); art.len() + 2];
        right_rows[0] = "Status".to_string();
        right_rows[1] = format!("{:<12} {}", "Service", format!("{service_dot} {service_state}"));
        right_rows[2] = format!("{:<12} {}", "Config", config_summary);
        right_rows[3] = format!("{:<12} {}", "Trigger", trigger_label);
        right_rows[5] = "Quick Commands".to_string();
        right_rows[6] = format!("{:<12} {}", "on", "Start daemon");
        right_rows[7] = format!("{:<12} {}", "off", "Stop daemon");
        right_rows[8] = format!("{:<12} {}", "config show", "View active config");
        right_rows[9] = format!("{:<12} {}", "update", "Upgrade via Homebrew");
        right_rows[10] = format!("{:<12} {}", "help", "Show all commands");
        right_rows[11] = format!("{:<12} {}", "exit", "Close settings shell");

        for i in 0..right_rows.len() {
            let mut line = String::new();
            let art_row = art.get(i).map(String::as_str).unwrap_or("");
            if !art_row.is_empty() {
                line.push_str(&" ".repeat(integral_col));
                line.push_str(art_row);
            }
            line = pad_to_width(&line, text_col);
            if !right_rows[i].is_empty() {
                line.push_str(&right_rows[i]);
            }
            lines.push(line);
        }
    }
    if compact {
        lines.extend([
            "    on           Start daemon".to_string(),
            "    off          Stop daemon".to_string(),
            "    config show  View active config".to_string(),
            "    update       Upgrade via Homebrew".to_string(),
            "    help         Show all commands".to_string(),
            "    exit         Close settings shell".to_string(),
        ]);
    }
    lines.push(String::new());
    let mut out = render_box(&lines, inner_width, theme);
    out.push('\n');
    if theme.color {
        out = colorize_dashboard(out, service_on);
    }
    out
}

fn render_prompt() -> String {
    format!("{PROMPT_SYMBOL} typesymbol › ")
}

fn render_box(lines: &[String], inner_width: usize, theme: &RenderTheme) -> String {
    let border = BoxChars::for_theme(theme);
    let mut out = String::new();
    out.push_str(border.tl);
    out.push_str(&border.h.repeat(inner_width));
    out.push_str(border.tr);
    out.push('\n');
    for line in lines {
        out.push_str(border.v);
        out.push_str(&pad_to_width(line, inner_width));
        out.push_str(border.v);
        out.push('\n');
    }
    out.push_str(border.bl);
    out.push_str(&border.h.repeat(inner_width));
    out.push_str(border.br);
    out.push('\n');
    out
}

fn render_help(theme: &RenderTheme, width: usize) -> String {
    if width < COMPACT_MIN_WIDTH {
        return "TypeSymbol Help\n\nQuick Commands\n  on        Start daemon\n  off       Stop daemon\n  update    Upgrade via Homebrew (macOS)\n  help      Show help\n  exit      Close shell\n\nConfig\n  config show\n  config set <key> <value>\n  config init\n\nDaemon\n  daemon status\n  daemon stop\n  daemon enable\n  daemon disable\n\nTesting\n  test <expr>\n".to_string();
    }

    let quick = [
        CommandItem {
            command: "on",
            description: "Start background daemon",
        },
        CommandItem {
            command: "off",
            description: "Stop daemon",
        },
        CommandItem {
            command: "help",
            description: "Show this help screen",
        },
        CommandItem {
            command: "update",
            description: "Upgrade via Homebrew (macOS)",
        },
        CommandItem {
            command: "exit",
            description: "Close settings shell",
        },
    ];
    let config = [
        CommandItem {
            command: "config show",
            description: "View active config",
        },
        CommandItem {
            command: "config set",
            description: "Update a config value",
        },
        CommandItem {
            command: "config init",
            description: "Create a default config file",
        },
    ];
    let daemon = [
        CommandItem {
            command: "daemon status",
            description: "Show daemon status",
        },
        CommandItem {
            command: "daemon stop",
            description: "Stop background daemon",
        },
        CommandItem {
            command: "daemon enable",
            description: "Enable daemon autostart",
        },
        CommandItem {
            command: "daemon disable",
            description: "Disable daemon autostart",
        },
    ];
    let testing = [CommandItem {
        command: "test <expr>",
        description: "Preview how an expression expands",
    }];
    let groups = [
        CommandGroup {
            title: "Quick Commands",
            items: &quick,
        },
        CommandGroup {
            title: "Configuration",
            items: &config,
        },
        CommandGroup {
            title: "Daemon",
            items: &daemon,
        },
        CommandGroup {
            title: "Testing",
            items: &testing,
        },
    ];
    render_command_table(theme, width, "Help", &groups)
}

fn render_command_table(
    theme: &RenderTheme,
    width: usize,
    title: &str,
    groups: &[CommandGroup<'_>],
) -> String {
    let mut lines = vec![title.to_string(), String::new()];
    for (idx, group) in groups.iter().enumerate() {
        lines.push(group.title.to_string());
        for item in group.items {
            lines.push(format!("  {:<16} {}", item.command, item.description));
        }
        if idx + 1 != groups.len() {
            lines.push(String::new());
        }
    }
    let mut out = render_box(&lines, panel_inner_width(width), theme);
    if theme.color {
        out = colorize_help(out);
    }
    out
}

fn render_info_panel(
    theme: &RenderTheme,
    width: usize,
    title: &str,
    rows: &[(&str, &str)],
) -> String {
    let mut lines = vec![title.to_string(), String::new()];
    for (k, v) in rows {
        lines.push(format!("  {:<12} {}", k, v));
    }
    let mut out = render_box(&lines, panel_inner_width(width), theme);
    if theme.color {
        out = colorize_info_panel(out, title, rows);
    }
    out
}

fn render_error_panel(
    theme: &RenderTheme,
    width: usize,
    command: &str,
    message: &str,
    hint: Option<&str>,
) -> String {
    let mut lines = vec!["Error".to_string(), String::new()];
    lines.push(format!("  {:<12} {}", "Command", command));
    lines.push(format!("  {:<12} {}", "Message", message));
    if let Some(h) = hint {
        lines.push(String::new());
        lines.push(format!("  {:<12} {}", "Try", h));
    }
    let mut out = render_box(&lines, panel_inner_width(width), theme);
    if theme.color {
        let mut rows = vec![("Command", command), ("Message", message)];
        if let Some(h) = hint {
            rows.push(("Try", h));
        }
        out = colorize_info_panel(out, "Error", &rows);
    }
    out
}

fn render_active_config_panel(theme: &RenderTheme, width: usize, loaded: &LoadedConfig) -> String {
    let daemon_state = if launch_agent_path().exists() {
        "enabled"
    } else {
        "disabled"
    };
    let unicode_state = if theme.unicode { "enabled" } else { "disabled" };
    render_info_panel(
        theme,
        width,
        "Active Config",
        &[
            ("Config", &loaded.source),
            ("Trigger", &loaded.config.trigger_key),
            ("Daemon", daemon_state),
            ("Unicode", unicode_state),
        ],
    )
}

fn pad_to_width(text: &str, width: usize) -> String {
    let visible = visible_width(text);
    if visible >= width {
        truncate_visible(text, width)
    } else {
        format!("{text}{}", " ".repeat(width - visible))
    }
}

fn visible_width(text: &str) -> usize {
    let mut count = 0usize;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for esc in chars.by_ref() {
                if esc == 'm' {
                    break;
                }
            }
            continue;
        }
        count += 1;
    }
    count
}

fn truncate_visible(text: &str, max_visible: usize) -> String {
    let mut out = String::new();
    let mut visible = 0usize;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            out.push(ch);
            if let Some(next) = chars.next() {
                out.push(next);
            }
            for esc in chars.by_ref() {
                out.push(esc);
                if esc == 'm' {
                    break;
                }
            }
            continue;
        }
        if visible >= max_visible {
            break;
        }
        out.push(ch);
        visible += 1;
    }
    out
}

fn panel_inner_width(width: usize) -> usize {
    let available = width.saturating_sub(2).max(34);
    available.min(PANEL_INNER_WIDTH)
}

fn colorize_header(mut rendered: String) -> String {
    rendered = colorize_borders(rendered);
    for token in [
        "████████╗██╗   ██╗██████╗ ███████╗",
        "╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝",
        "██║    ╚████╔╝ ██████╔╝█████╗",
        "██║     ╚██╔╝  ██╔═══╝ ██╔══╝",
        "██║      ██║   ██║     ███████╗",
        "╚═╝      ╚═╝   ╚═╝     ╚══════╝",
    ] {
        rendered = rendered.replace(token, &ansi(token, "1;96"));
    }
    rendered = rendered.replace(
        "∫ TypeSymbol",
        &format!("{} {}", ansi("∫", "1;95"), ansi("TypeSymbol", "1;97")),
    );
    rendered = rendered.replace(
        "system-wide math typing helper",
        &ansi("system-wide math typing helper", "2;37"),
    );
    rendered = rendered.replace(&format!("v{}", APP_VERSION), &ansi(&format!("v{}", APP_VERSION), "2;95"));
    rendered
}

fn colorize_dashboard(mut rendered: String, service_on: bool) -> String {
    rendered = colorize_borders(rendered);
    rendered = rendered.replace("Status", &ansi("Status", "1;97"));
    rendered = rendered.replace("Quick Commands", &ansi("Quick Commands", "1;97"));
    for label in ["Service", "Config", "Trigger"] {
        rendered = rendered.replace(label, &ansi(label, "1;96"));
    }
    if service_on {
        rendered = rendered.replace(
            "● on",
            &format!("{} {}", ansi("●", "1;92"), ansi("on", "1;97")),
        );
        rendered = rendered.replace(
            "* on",
            &format!("{} {}", ansi("*", "1;92"), ansi("on", "1;97")),
        );
    } else {
        rendered = rendered.replace(
            "● off",
            &format!("{} {}", ansi("●", "1;91"), ansi("off", "1;97")),
        );
        rendered = rendered.replace(
            "* off",
            &format!("{} {}", ansi("*", "1;91"), ansi("off", "1;97")),
        );
    }
    for command in ["on", "off", "config show", "update", "help", "exit"] {
        rendered = rendered.replace(command, &ansi(command, "1;97"));
    }
    for desc in [
        "Start daemon",
        "Stop daemon",
        "View active config",
        "Upgrade via Homebrew",
        "Show all commands",
        "Close settings shell",
    ] {
        rendered = rendered.replace(desc, &ansi(desc, "2;37"));
    }
    rendered
}

fn colorize_help(mut rendered: String) -> String {
    rendered = colorize_borders(rendered);
    for heading in ["Help", "Quick Commands", "Configuration", "Daemon", "Testing"] {
        rendered = rendered.replace(heading, &ansi(heading, "1;97"));
    }
    for cmd in [
        "on",
        "off",
        "help",
        "update",
        "exit",
        "config show",
        "config set",
        "config init",
        "daemon status",
        "daemon stop",
        "daemon enable",
        "daemon disable",
        "test <expr>",
    ] {
        rendered = rendered.replace(cmd, &ansi(cmd, "1;97"));
    }
    for desc in [
        "Start background daemon",
        "Stop daemon",
        "Show this help screen",
        "Upgrade via Homebrew (macOS)",
        "Close settings shell",
        "View active config",
        "Update a config value",
        "Create a default config file",
        "Show daemon status",
        "Stop background daemon",
        "Enable daemon autostart",
        "Disable daemon autostart",
        "Preview how an expression expands",
    ] {
        rendered = rendered.replace(desc, &ansi(desc, "2;37"));
    }
    rendered
}

fn colorize_info_panel(mut rendered: String, title: &str, rows: &[(&str, &str)]) -> String {
    rendered = colorize_borders(rendered);
    rendered = rendered.replace(title, &ansi(title, "1;97"));
    for (k, v) in rows {
        rendered = rendered.replace(k, &ansi(k, "1;96"));
        rendered = rendered.replace(v, &ansi(v, "2;37"));
    }
    rendered
}

fn colorize_borders(mut rendered: String) -> String {
    for border in ["╭", "╮", "╰", "╯", "─", "│", "+", "-", "|"] {
        rendered = rendered.replace(border, &ansi(border, "2;35"));
    }
    rendered
}

fn ansi(text: &str, code: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

fn supports_truecolor() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if let Ok(color_term) = std::env::var("COLORTERM") {
        let lowered = color_term.to_lowercase();
        if lowered.contains("truecolor") || lowered.contains("24bit") {
            return true;
        }
    }
    std::env::var("TERM")
        .map(|term| {
            let lowered = term.to_lowercase();
            lowered.contains("truecolor") || lowered.contains("direct")
        })
        .unwrap_or(false)
}

fn render_integral_gradient(lines: &[&str], use_truecolor: bool) -> Vec<String> {
    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| gradient_line(line, idx, lines.len(), use_truecolor))
        .collect()
}

fn gradient_line(line: &str, row_index: usize, total_rows: usize, use_truecolor: bool) -> String {
    let code = gradient_code(row_index, total_rows, use_truecolor);
    colorize_non_space_runs(line, &code)
}

fn gradient_code(row_index: usize, total_rows: usize, use_truecolor: bool) -> String {
    if use_truecolor {
        let palette = [
            (86, 182, 255),
            (93, 164, 255),
            (115, 139, 255),
            (139, 116, 255),
            (164, 96, 245),
            (186, 83, 230),
            (207, 75, 210),
            (226, 74, 190),
            (240, 88, 175),
            (255, 115, 190),
        ];
        let idx = row_index.min(total_rows.saturating_sub(1)).min(palette.len() - 1);
        let (r, g, b) = palette[idx];
        format!("38;2;{r};{g};{b}")
    } else {
        let fallback = ["96", "96", "94", "34", "95", "35", "95", "35", "95", "95"];
        let idx = row_index.min(total_rows.saturating_sub(1)).min(fallback.len() - 1);
        fallback[idx].to_string()
    }
}

fn colorize_non_space_runs(text: &str, code: &str) -> String {
    let mut out = String::new();
    let mut run = String::new();
    for ch in text.chars() {
        if ch == ' ' {
            if !run.is_empty() {
                out.push_str(&ansi(&run, code));
                run.clear();
            }
            out.push(ch);
        } else {
            run.push(ch);
        }
    }
    if !run.is_empty() {
        out.push_str(&ansi(&run, code));
    }
    out
}

fn default_config_path() -> PathBuf {
    if cfg!(windows) {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("TypeSymbol")
                .join("config.toml");
        }
    } else if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("typesymbol")
            .join("config.toml");
    }
    PathBuf::from(".")
        .join(".config")
        .join("typesymbol")
        .join("config.toml")
}

fn render_default_config() -> String {
    let cfg = TypeSymbolConfig::default();
    let excluded_apps_block = if cfg!(windows) {
        r#"excluded_apps = [
  "WindowsTerminal.exe",
  "Code.exe",
  "rustrover64.exe",
]"#
    } else {
        r#"excluded_apps = [
  "com.apple.Terminal",
  "com.microsoft.VSCode",
  "com.jetbrains.rustrover",
]"#
    };
    format!(
        r#"mode = "{mode}"
trigger_mode = "{trigger_mode}"
trigger_key = "{trigger_key}"
live_suggestions = {live_suggestions}
auto_replace_safe_rules = {auto_replace_safe_rules}
{excluded_apps}

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
        excluded_apps = excluded_apps_block,
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
        infinity = cfg
            .aliases
            .get("infinity")
            .map(String::as_str)
            .unwrap_or("∞"),
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
    let preferred = if cfg!(windows) {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("TypeSymbol").join("state")
        } else {
            PathBuf::from(".").join(".typesymbol-state")
        }
    } else if let Ok(home) = std::env::var("HOME") {
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
        let fallback = if cfg!(windows) {
            PathBuf::from(".").join(".typesymbol-state")
        } else {
            PathBuf::from("/tmp").join("typesymbol-state")
        };
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
    lock.try_lock_exclusive()
        .map_err(|_| "Another TypeSymbol daemon instance is already running.".to_string())?;
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
    if cfg!(windows) {
        let filter = format!("PID eq {}", pid);
        return Command::new("tasklist")
            .args(["/FI", &filter])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map(|o| {
                o.status.success()
                    && String::from_utf8_lossy(&o.stdout)
                        .to_lowercase()
                        .contains(&pid.to_string())
            })
            .unwrap_or(false);
    }

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
    match start_daemon_background_silent(config_path) {
        Ok((pid, log_path)) => {
            println!("Started TypeSymbol daemon in background (pid {}).", pid);
            println!("Logs: {}", log_path.display());
            true
        }
        Err(err) => {
            eprintln!("{}", err);
            false
        }
    }
}

fn start_daemon_background_silent(config_path: Option<PathBuf>) -> Result<(u32, PathBuf), String> {
    if cfg!(windows) {
        enable_autostart_silent(config_path)?;
        let status = Command::new("schtasks")
            .args(["/Run", "/TN", launch_agent_label()])
            .status()
            .map_err(|err| format!("Failed to run Windows autostart task: {}", err))?;
        if !status.success() {
            return Err("Failed to start daemon via Scheduled Task.".to_string());
        }

        // Give the task a moment to spawn and write pid file.
        thread::sleep(Duration::from_millis(800));
        if let Some(pid) = read_live_pid() {
            return Ok((pid, daemon_log_path()));
        }
        return Err(
            "Scheduled Task started, but daemon did not report a live pid. Check daemon logs."
                .to_string(),
        );
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(err) => return Err(format!("Failed to resolve executable path: {}", err)),
    };

    let log_path = daemon_log_path();
    let log = match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(err) => return Err(format!("Failed to open daemon log {}: {}", log_path.display(), err)),
    };
    let log_err = match log.try_clone() {
        Ok(f) => f,
        Err(err) => return Err(format!("Failed to duplicate daemon log handle: {}", err)),
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
        Ok(child) => Ok((child.id(), log_path)),
        Err(err) => Err(format!("Failed to start daemon in background: {}", err)),
    }
}

fn stop_daemon() {
    match stop_daemon_silent() {
        Ok(message) => println!("{}", message),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn stop_daemon_silent() -> Result<&'static str, String> {
    let Some(pid) = read_live_pid() else {
        return Ok("TypeSymbol daemon is not running.");
    };
    let status = if cfg!(windows) {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(|err| format!("Failed to execute taskkill for pid {}: {}", pid, err))?
    } else {
        Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map_err(|err| format!("Failed to execute kill for pid {}: {}", pid, err))?
    };
    if status.success() {
        let _ = fs::remove_file(daemon_pid_path());
        Ok("Background daemon stopped")
    } else {
        Err(format!("Failed to stop daemon pid {}.", pid))
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
    if cfg!(windows) {
        "TypeSymbolDaemon"
    } else {
        "com.typesymbol.daemon"
    }
}

fn launch_agent_path() -> PathBuf {
    if cfg!(windows) {
        state_dir().join("autostart.windows")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{}.plist", launch_agent_label()))
    } else {
        PathBuf::from("/tmp").join(format!("{}.plist", launch_agent_label()))
    }
}

fn windows_run_registry_path() -> &'static str {
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
}

fn windows_run_registry_value_name() -> &'static str {
    "TypeSymbolDaemon"
}

fn windows_daemon_command(exe: &std::path::Path, config_path: Option<&PathBuf>) -> String {
    let mut command = format!("\"{}\" ", exe.display());
    if let Some(cfg) = config_path {
        command.push_str(&format!("--config \"{}\" ", cfg.display()));
    }
    command.push_str("daemon run-internal");
    command
}

fn enable_autostart(config_path: Option<PathBuf>) {
    match enable_autostart_silent(config_path) {
        Ok(agent_path) => {
            println!("Autostart enabled.");
            println!("LaunchAgent: {}", agent_path.display());
            println!("Daemon will stay on after login/reboot.");
        }
        Err(err) => eprintln!("{}", err),
    }
}

fn enable_autostart_silent(config_path: Option<PathBuf>) -> Result<PathBuf, String> {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(err) => return Err(format!("Failed to resolve executable path: {}", err)),
    };
    let agent_path = launch_agent_path();
    if let Some(parent) = agent_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create autostart directory {}: {}", parent.display(), err));
        }
    }

    if cfg!(windows) {
        let task_command = windows_daemon_command(&exe, config_path.as_ref());

        let status = Command::new("schtasks")
            .args([
                "/Create",
                "/TN",
                launch_agent_label(),
                "/SC",
                "ONLOGON",
                "/TR",
                &task_command,
                "/F",
            ])
            .status();
        match status {
            Ok(s) if s.success() => {
                let _ = fs::write(&agent_path, task_command);
                return Ok(agent_path);
            }
            Ok(_) | Err(_) => {
                let reg_status = Command::new("reg")
                    .args([
                        "add",
                        windows_run_registry_path(),
                        "/v",
                        windows_run_registry_value_name(),
                        "/t",
                        "REG_SZ",
                        "/d",
                        &task_command,
                        "/f",
                    ])
                    .status();

                match reg_status {
                    Ok(s) if s.success() => {
                        let _ = fs::write(&agent_path, task_command);
                        return Ok(agent_path);
                    }
                    Ok(_) | Err(_) => {
                        return Err(
                            "Failed to configure Windows autostart (both Task Scheduler and HKCU Run key). Try running elevated or create it manually with schtasks."
                                .to_string(),
                        )
                    }
                }
            }
        }
    }

    let log_path = daemon_log_path();
    let cfg_arg = config_path
        .map(|p| {
            format!(
                "<string>--config</string><string>{}</string>",
                xml_escape(&p.display().to_string())
            )
        })
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
        return Err(format!(
            "Failed to write launch agent {}: {}",
            agent_path.display(),
            err
        ));
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
        Ok(s) if s.success() => Ok(agent_path),
        _ => {
            let legacy = Command::new("launchctl")
                .arg("load")
                .arg(&agent_path)
                .status();
            if matches!(legacy, Ok(s) if s.success()) {
                Ok(agent_path)
            } else {
                Err(format!(
                    "Autostart plist written, but launchctl load failed. Try manually: launchctl bootstrap gui/$(id -u) {}",
                    agent_path.display()
                ))
            }
        }
    }
}

fn disable_autostart() {
    match disable_autostart_silent() {
        Ok(_) => println!("Autostart disabled."),
        Err(err) => eprintln!("{}", err),
    }
}

fn disable_autostart_silent() -> Result<(), String> {
    let agent_path = launch_agent_path();
    if cfg!(windows) {
        let _ = Command::new("schtasks")
            .args(["/Delete", "/TN", launch_agent_label(), "/F"])
            .status();
        let _ = Command::new("reg")
            .args([
                "delete",
                windows_run_registry_path(),
                "/v",
                windows_run_registry_value_name(),
                "/f",
            ])
            .status();
        if agent_path.exists() {
            let _ = fs::remove_file(&agent_path);
        }
        return Ok(());
    }

    let _ = Command::new("launchctl")
        .arg("bootout")
        .arg(format!("gui/{}/{}", current_uid(), launch_agent_label()))
        .status();
    let _ = Command::new("launchctl")
        .arg("unload")
        .arg(&agent_path)
        .status();
    if agent_path.exists() {
        if let Err(err) = fs::remove_file(&agent_path) {
            return Err(format!(
                "Failed to remove launch agent {}: {}",
                agent_path.display(),
                err
            ));
        }
    }
    Ok(())
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

fn run_self_update(check_only: bool) {
    if cfg!(target_os = "windows") {
        let package_id = "yazanmwk.TypeSymbol";
        let source = "winget";
        if check_only {
            let output = Command::new("winget")
                .args(["upgrade", "--id", package_id, "--exact", "--source", source])
                .output();
            match output {
                Ok(result) if result.status.success() => {
                    let stdout = String::from_utf8_lossy(&result.stdout).to_lowercase();
                    if stdout.contains("no installed package found")
                        || stdout.contains("no available upgrade found")
                    {
                        println!("TypeSymbol is up to date.");
                    } else {
                        println!("Update available. Run: typesymbol update");
                    }
                }
                Ok(_) => {
                    eprintln!(
                        "Failed to check updates with winget. Run `winget --version` and verify the package id."
                    );
                    process::exit(1);
                }
                Err(err) => {
                    eprintln!("Failed to launch winget: {}", err);
                    process::exit(1);
                }
            }
            return;
        }

        println!("Running winget upgrade...");
        let status = Command::new("winget")
            .args([
                "upgrade",
                "--id",
                package_id,
                "--exact",
                "--source",
                source,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("TypeSymbol update complete.");
                println!("Tip: run `typesymbol --version` to verify.");
            }
            Ok(_) => {
                eprintln!("winget upgrade exited with an error.");
                process::exit(1);
            }
            Err(err) => {
                eprintln!("Failed to launch winget: {}", err);
                process::exit(1);
            }
        }
        return;
    }

    if !cfg!(target_os = "macos") {
        println!("Automatic update is currently supported on macOS Homebrew installs.");
        println!("Download the latest release from:");
        println!("https://github.com/yazanmwk/TypeSymbol/releases/latest");
        return;
    }

    if !command_exists("brew") {
        eprintln!("Homebrew is not installed or not on PATH.");
        process::exit(1);
    }

    if check_only {
        let output = Command::new("brew")
            .args(["outdated", "--formula", "typesymbol"])
            .output();
        match output {
            Ok(result) if result.status.success() => {
                if String::from_utf8_lossy(&result.stdout).trim().is_empty() {
                    println!("TypeSymbol is up to date.");
                } else {
                    println!("Update available. Run: typesymbol update");
                }
            }
            Ok(_) => {
                eprintln!("Failed to check updates via Homebrew.");
                process::exit(1);
            }
            Err(err) => {
                eprintln!("Failed to run Homebrew: {}", err);
                process::exit(1);
            }
        }
        return;
    }

    println!("Updating Homebrew metadata...");
    let update_status = Command::new("brew").arg("update").status();
    match update_status {
        Ok(status) if status.success() => {}
        Ok(_) => {
            eprintln!("brew update failed.");
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Failed to run brew update: {}", err);
            process::exit(1);
        }
    }

    println!("Upgrading TypeSymbol...");
    let upgrade_status = Command::new("brew")
        .args(["upgrade", "typesymbol"])
        .status();
    match upgrade_status {
        Ok(status) if status.success() => {
            println!("TypeSymbol update complete.");
            println!("Tip: run `typesymbol --version` to verify.");
        }
        Ok(_) => {
            eprintln!("brew upgrade typesymbol failed.");
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Failed to run brew upgrade: {}", err);
            process::exit(1);
        }
    }
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn print_config(loaded: &LoadedConfig) {
    let config = &loaded.config;
    println!("Config source: {}", loaded.source);
    println!("mode = {}", config.mode);
    println!("trigger_mode = {}", config.trigger_mode);
    println!("trigger_key = {}", config.trigger_key);
    println!("live_suggestions = {}", config.live_suggestions);
    println!(
        "auto_replace_safe_rules = {}",
        config.auto_replace_safe_rules
    );
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
            eprintln!(
                "Failed to create config directory {}: {}",
                parent.display(),
                err
            );
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
    match set_config_value_silent(config_path_arg, key, value) {
        Ok(path) => println!("Updated {} in {}", key, path),
        Err(err) => eprintln!("{}", err),
    }
}

fn set_config_value_silent(
    config_path_arg: Option<PathBuf>,
    key: &str,
    value: &str,
) -> Result<String, String> {
    let path = config_path_arg.unwrap_or_else(default_config_path);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!("Failed to create config directory {}: {}", parent.display(), err)
            })?;
        }
        fs::write(&path, render_default_config())
            .map_err(|err| format!("Failed to write config at {}: {}", path.display(), err))?;
    }

    let path_str = path.to_string_lossy().to_string();
    let mut cfg = match load_config(&path_str) {
        Ok(c) => c,
        Err(err) => return Err(format!("Failed to load config from {}: {}", path.display(), err)),
    };

    if let Err(err) = apply_config_update(&mut cfg, key, value) {
        return Err(err);
    }

    let out = match toml::to_string_pretty(&cfg) {
        Ok(s) => s,
        Err(err) => return Err(format!("Failed to serialize config: {}", err)),
    };

    if let Err(err) = fs::write(&path, out) {
        return Err(format!("Failed to write config at {}: {}", path.display(), err));
    }

    Ok(path.display().to_string())
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

#[cfg(test)]
mod tui_tests {
    use super::*;

    #[test]
    fn renders_compact_header_on_narrow_width() {
        let theme = RenderTheme {
            color: false,
            unicode: true,
        };
        let output = render_header(&theme, 60);
        assert!(output.contains("∫ TypeSymbol"));
        assert!(!output.contains("████████"));
    }

    #[test]
    fn renders_ascii_borders_when_unicode_disabled() {
        let theme = RenderTheme {
            color: false,
            unicode: false,
        };
        let output = render_dashboard(&theme, 78, false, "defaults", "enter");
        assert!(output.contains("+"));
        assert!(output.contains("|"));
        assert!(!output.contains("╭"));
    }

    #[test]
    fn prompt_matches_new_branding() {
        assert_eq!(render_prompt(), "∫ typesymbol › ");
    }

    #[test]
    fn dashboard_lines_have_consistent_visible_width_with_color() {
        let theme = RenderTheme {
            color: true,
            unicode: true,
        };
        let output = render_dashboard(&theme, 120, true, "defaults", "enter");
        let expected = panel_inner_width(120) + 2;
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() > 2);
        for line in lines {
            if line.is_empty() {
                continue;
            }
            assert_eq!(visible_width(line), expected);
        }
    }

    #[test]
    fn header_never_exceeds_available_line_budget() {
        let lines = build_tui_header_lines(120, 14, false);
        assert!(lines.len() <= 14);
    }

    #[test]
    fn full_header_is_used_when_budget_allows_it() {
        let lines = build_tui_header_lines(120, 15, false);
        assert_eq!(lines.len(), 15);
    }
}
