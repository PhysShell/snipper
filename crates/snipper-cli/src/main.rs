//! `snipper` CLI entry point.
//!
//! Subcommands:
//! - `snipper context` — classify cursor context.
//!   Supports `--format {tree,sexpr,json}`.
//! - `snipper expand`  — apply expansion at a cursor position.

use std::io::Read as _;

use clap::{Parser, Subcommand};
use snippercontext::{Backend as _, LexicalClass, TreeSitterBackend};
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

    /// Source language (only "csharp" is supported right now).
    #[arg(long, default_value = "csharp")]
    language: String,

    /// Byte offset of the cursor in the source text.
    #[arg(long)]
    offset: usize,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Tree,
    Sexpr,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Context(ref args) => run_context(args),
    }
}

fn run_context(args: &ContextArgs) -> ExitCode {
    let mut source = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut source) {
        eprintln!("error: failed to read stdin: {e}");
        return ExitCode::IoErr;
    }

    let class = match args.language.as_str() {
        "csharp" => {
            let backend = TreeSitterBackend::csharp();
            match backend.classify(&source, args.offset) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::DataErr;
                }
            }
        }
        other => {
            eprintln!("error: unsupported language '{other}' (only 'csharp' is available)");
            return ExitCode::Usage;
        }
    };

    let lexical_str = lexical_class_str(class);

    match args.format {
        OutputFormat::Tree => {
            println!("context");
            println!("├── language: {}", args.language);
            println!("├── offset: {}", args.offset);
            println!("└── lexical: {lexical_str}");
        }
        OutputFormat::Sexpr => {
            println!(
                "(context\n  (language {:?})\n  (offset {})\n  (lexical {lexical_str}))",
                args.language, args.offset
            );
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "language": args.language,
                    "offset": args.offset,
                    "lexical": lexical_str,
                })
            );
        }
    }

    ExitCode::Ok
}

const fn lexical_class_str(class: LexicalClass) -> &'static str {
    match class {
        LexicalClass::CodeAfterDot => "code_after_dot",
        LexicalClass::StringLiteral => "string_literal",
        LexicalClass::Comment => "comment",
        LexicalClass::IdentifierDeclaration => "identifier_declaration",
        LexicalClass::Other => "other",
        // Exhaustiveness: LexicalClass is #[non_exhaustive], this arm is unreachable
        // in practice but required by the compiler when matching outside the crate.
        _ => "unknown",
    }
}
