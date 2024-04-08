use cargo_selector::run;
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

fn main() -> std::io::Result<()> {
    let _ = Cli::parse();
    run()
}
