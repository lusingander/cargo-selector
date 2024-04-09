mod cargo;

use clap::{Args, Parser};

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

#[derive(Debug)]
pub struct Target {
    name: String,
    kind: TargetKind,
    path: String,
    required_features: Vec<String>,
}

#[derive(Debug)]
pub enum TargetKind {
    Bin,
    Example,
}

fn main() -> std::io::Result<()> {
    let _ = Cli::parse();

    let targets = cargo::get_all_targets();
    println!("{:?}", targets);
    Ok(())
}
