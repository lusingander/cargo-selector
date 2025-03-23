mod cargo;
mod config;
mod event;
mod matcher;
mod tui;
mod util;

use std::{
    io::{stderr, BufWriter, Stderr},
    panic,
    process::{ExitCode, ExitStatus},
};

use clap::{Args, Parser, ValueEnum};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal, TerminalOptions, Viewport,
};
use serde::Deserialize;

use crate::{
    config::Config,
    matcher::Matcher,
    tui::{Ret, Tui},
};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt as _;

#[derive(Debug, Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cli {
    Selector(SelectorArgs),
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
struct SelectorArgs {
    /// Display list inline
    #[arg(short, long)]
    inline: bool,

    /// List size
    #[arg(short = 'n', long, default_value = "10", value_name = "SIZE")]
    inline_list_size: u16,

    /// Target kind
    #[arg(short, long, value_name = "NAME")]
    kind: Option<TargetKind>,

    /// Match type
    #[arg(short = 't', long, value_name = "TYPE")]
    match_type: Option<MatchType>,

    /// Additional arguments
    #[arg(short, long, value_name = "ARGS", allow_hyphen_values = true)]
    additional_args: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Target {
    name: String,
    kind: TargetKind,
    path: String,
    required_features: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TargetKind {
    Bin,
    Example,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    #[default]
    Substring,
    Fuzzy,
}

impl MatchType {
    fn matcher(self) -> Matcher {
        match self {
            MatchType::Substring => Matcher::substring(),
            MatchType::Fuzzy => Matcher::fuzzy(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum Action {
    #[default]
    Run,
    Build,
}

fn setup(
    inline: bool,
    inline_list_size: u16,
) -> std::io::Result<Terminal<CrosstermBackend<BufWriter<Stderr>>>> {
    enable_raw_mode()?;
    if !inline {
        execute!(stderr(), EnterAlternateScreen)?;
    }

    let backend = CrosstermBackend::new(BufWriter::new(stderr()));
    let viewport = if inline {
        Viewport::Inline(inline_list_size + 1)
    } else {
        Viewport::Fullscreen
    };
    Terminal::with_options(backend, TerminalOptions { viewport })
}

fn shutdown(inline: bool) -> std::io::Result<()> {
    if !inline {
        execute!(stderr(), LeaveAlternateScreen)?;
    }
    disable_raw_mode()?;
    Ok(())
}

fn initialize_panic_handler(inline: bool) {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        shutdown(inline).unwrap();
        original_hook(panic_info);
    }));
}

#[cfg(unix)]
fn to_exit_code(status: ExitStatus) -> ExitCode {
    if let Some(code) = status.code() {
        ExitCode::from(code as u8)
    } else if let Some(signal) = status.signal() {
        ExitCode::from(signal as u8 + 128)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(windows)]
fn to_exit_code(status: ExitStatus) -> ExitCode {
    if let Some(code) = status.code() {
        ExitCode::from(code as u8)
    } else {
        ExitCode::FAILURE
    }
}

fn main() -> std::io::Result<ExitCode> {
    let Cli::Selector(args) = Cli::parse();
    let SelectorArgs {
        inline,
        inline_list_size,
        kind,
        match_type,
        additional_args,
    } = args;

    let config = Config::load();
    let match_type = match_type.or(config.match_type).unwrap_or_default();
    let theme = config.color;

    let mut targets = cargo::get_all_targets();
    if let Some(kind) = kind {
        targets.retain(|t| t.kind == kind);
    }

    initialize_panic_handler(inline);
    let mut terminal = setup(inline, inline_list_size)?;
    let term_size = terminal.get_frame().area();
    let matcher = match_type.matcher();
    let ret = Tui::new(targets, term_size, matcher, theme).run(&mut terminal);
    shutdown(inline)?;

    if inline {
        terminal.clear()?;
    }

    ret.map(|t| match t {
        Ret::Quit => ExitCode::SUCCESS,
        Ret::Selected(t, a) => {
            let status = cargo::exec_cargo_run(&t, &a, additional_args);
            to_exit_code(status)
        }
        Ret::NotSelected => {
            eprintln!("no command selected");
            ExitCode::SUCCESS
        }
    })
}
