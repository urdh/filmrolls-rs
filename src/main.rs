#![doc = include_str!("../README.md")]
use std::process::ExitCode;

use ::clap::Parser;
use color_eyre::eyre::Result;

mod clap;
mod cmds;
mod metadata;
mod negative;
mod rolls;
mod types;

/// Application entry point
///
/// See [`clap::Cli`] for a description of the CLI itself.
fn main() -> Result<ExitCode> {
    let args = clap::Cli::parse();
    args.init_colors()?;
    args.init_logging();
    args.run_command()
}
