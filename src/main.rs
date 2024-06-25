mod cargo;
mod macros;
mod tui;
mod util;

use std::{
    io::{stderr, BufWriter, Stderr},
    panic,
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
use tui::{Ret, Tui};

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

#[derive(Clone, Copy)]
pub enum Action {
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

fn main() -> std::io::Result<()> {
    let Cli::Selector(args) = Cli::parse();
    let SelectorArgs {
        inline,
        inline_list_size,
        kind,
    } = args;

    let mut targets = cargo::get_all_targets();
    if let Some(kind) = kind {
        targets.retain(|t| t.kind == kind);
    }

    initialize_panic_handler(inline);
    let mut terminal = setup(inline, inline_list_size)?;
    let term_size = terminal.get_frame().size();
    let ret = Tui::new(targets, term_size).run(&mut terminal);
    shutdown(inline)?;

    if inline {
        terminal.clear()?;
    }

    ret.map(|t| match t {
        Ret::Quit => {}
        Ret::Selected(t, a) => cargo::exec_cargo_run(&t, &a),
        Ret::NotSelected => eprintln!("no command selected"),
    })
}
