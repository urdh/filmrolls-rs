#![doc = include_str!("../README.md")]
use std::process::ExitCode;

use ::clap::Parser;
use color_eyre::eyre::Result;

mod clap;
mod cmds;
mod rolls;
mod types;

fn main() -> Result<ExitCode> {
    let args = clap::Cli::parse();
    args.init_colors()?;
    args.init_logging();
    args.run_command()
}
