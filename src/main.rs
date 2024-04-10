mod cargo;
mod macros;
mod tui;
mod util;

use std::{
    io::{stderr, Stderr},
    panic,
};

use clap::{Args, Parser};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::{Ret, Tui};

// https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands
// https://docs.rs/clap/latest/clap/_derive/_cookbook/cargo_example_derive/index.html

#[derive(Debug, Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cli {
    Selector(SelectorArgs),
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
struct SelectorArgs {}

#[derive(Debug, Clone)]
pub struct Target {
    name: String,
    kind: TargetKind,
    path: String,
    required_features: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TargetKind {
    Bin,
    Example,
}

fn setup() -> std::io::Result<Terminal<CrosstermBackend<Stderr>>> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr());
    Terminal::new(backend)
}

fn shutdown() -> std::io::Result<()> {
    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn initialize_panic_handler() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        shutdown().unwrap();
        original_hook(panic_info);
    }));
}

fn main() -> std::io::Result<()> {
    let _ = Cli::parse();

    let targets = cargo::get_all_targets();

    initialize_panic_handler();
    let mut terminal = setup()?;
    let term_size = terminal.get_frame().size();
    let ret = Tui::new(targets, term_size).run(&mut terminal);
    shutdown()?;

    ret.map(|t| match t {
        Ret::Quit => {}
        Ret::Selected(t) => cargo::exec_cargo_run(&t),
        Ret::NotSelected => eprintln!("no command selected"),
    })
}
