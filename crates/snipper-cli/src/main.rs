//! `snipper` CLI entry point.
//!
//! Subcommands:
//! - `snipper context` — classify cursor context.
//!   Supports `--format {tree,sexpr,json}`.
//! - `snipper expand`  — apply expansion at a cursor position.

use clap::{Parser, Subcommand};
use sysexits::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "snipper", version, about = "Portable structural expansion engine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Classify cursor context and print the result.
    Context(ContextArgs),
}

#[derive(Debug, clap::Args)]
struct ContextArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "tree")]
    format: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Tree,
    Sexpr,
    Json,
}

fn main() -> ExitCode {
    let _cli = Cli::parse();
    // Scaffold only — implementation follows the main engine spec.
    ExitCode::Ok
}
