//! `snipper` CLI entry point.
//!
//! Subcommands:
//! - `snipper context` — classify cursor context.
//!   Supports `--format {tree,sexpr,json}`.
//! - `snipper expand`  — apply expansion at a cursor position;
//!   emits `TextEdit[]` JSON.

use std::io::Read as _;

use clap::{Parser, Subcommand};
use snippercontext::{Backend as _, ClassifiedContext, LexicalClass, TreeSitterBackend};
use sysexits::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "snipper",
    version,
    about = "Portable structural expansion engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Classify cursor context and print the result.
    Context(ContextArgs),
    /// Apply expansion at a cursor position and emit `TextEdit[]` JSON.
    Expand(ExpandArgs),
}

#[derive(Debug, clap::Args)]
struct ContextArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "tree")]
    format: OutputFormat,

    /// Source language (e.g. "csharp", "typescript").
    #[arg(long, default_value = "csharp")]
    language: String,

    /// Byte offset of the cursor in the source text.
    #[arg(long)]
    offset: usize,
}

#[derive(Debug, clap::Args)]
struct ExpandArgs {
    /// Source language (e.g. "csharp", "typescript").
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
        Command::Expand(ref args) => run_expand(args),
    }
}

fn classify_for_language(
    source: &str,
    offset: usize,
    language: &str,
) -> Result<ClassifiedContext, ExitCode> {
    let backend = match language {
        "csharp" | "cs" => TreeSitterBackend::csharp(),
        "typescript" | "ts" | "typescriptreact" | "tsx" => TreeSitterBackend::typescript(),
        other => {
            eprintln!("error: unsupported language '{other}' (supported: csharp, typescript)");
            return Err(ExitCode::Usage);
        }
    };
    backend.classify(source, offset).map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::DataErr
    })
}

fn run_context(args: &ContextArgs) -> ExitCode {
    let mut source = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut source) {
        eprintln!("error: failed to read stdin: {e}");
        return ExitCode::IoErr;
    }

    let classified = match classify_for_language(&source, args.offset, &args.language) {
        Ok(c) => c,
        Err(code) => return code,
    };

    let lexical_str = lexical_class_str(classified.lexical);

    match args.format {
        OutputFormat::Tree => {
            println!("context");
            println!("\u{251c}\u{2500}\u{2500} language: {}", args.language);
            println!("\u{251c}\u{2500}\u{2500} offset: {}", args.offset);
            if let Some(p) = &classified.postfix {
                println!("\u{251c}\u{2500}\u{2500} lexical: {lexical_str}");
                println!("\u{2514}\u{2500}\u{2500} postfix");
                println!("    \u{251c}\u{2500}\u{2500} receiver: {}", p.receiver);
                println!("    \u{2514}\u{2500}\u{2500} trigger:  {}", p.trigger);
            } else if let Some(p) = &classified.prefix {
                println!("\u{251c}\u{2500}\u{2500} lexical: {lexical_str}");
                println!("\u{2514}\u{2500}\u{2500} prefix");
                println!("    \u{2514}\u{2500}\u{2500} trigger:  {}", p.trigger);
            } else {
                println!("\u{2514}\u{2500}\u{2500} lexical: {lexical_str}");
            }
        }
        OutputFormat::Sexpr => {
            if let Some(p) = &classified.postfix {
                println!(
                    "(context\n  (language {:?})\n  (offset {})\n  (lexical {lexical_str})\n  (postfix (receiver {:?}) (trigger {:?})))",
                    args.language, args.offset, p.receiver, p.trigger
                );
            } else if let Some(p) = &classified.prefix {
                println!(
                    "(context\n  (language {:?})\n  (offset {})\n  (lexical {lexical_str})\n  (prefix (trigger {:?})))",
                    args.language, args.offset, p.trigger
                );
            } else {
                println!(
                    "(context\n  (language {:?})\n  (offset {})\n  (lexical {lexical_str}))",
                    args.language, args.offset
                );
            }
        }
        OutputFormat::Json => {
            let mut obj = serde_json::json!({
                "language": args.language,
                "offset": args.offset,
                "lexical": lexical_str,
            });
            if let Some(p) = &classified.postfix {
                obj["postfix"] = serde_json::json!({
                    "receiver": p.receiver,
                    "trigger": p.trigger,
                    "range": p.range,
                });
            }
            if let Some(p) = &classified.prefix {
                obj["prefix"] = serde_json::json!({
                    "trigger": p.trigger,
                    "range": p.range,
                });
            }
            println!("{obj}");
        }
    }

    ExitCode::Ok
}

fn run_expand(args: &ExpandArgs) -> ExitCode {
    let mut source = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut source) {
        eprintln!("error: failed to read stdin: {e}");
        return ExitCode::IoErr;
    }

    let classified = match classify_for_language(&source, args.offset, &args.language) {
        Ok(c) => c,
        Err(code) => return code,
    };

    let candidates = match args.language.as_str() {
        "csharp" | "cs" => {
            if let Some(postfix) = &classified.postfix {
                snippercore::match_postfix(postfix, &snippercore::built_in_csharp_postfix_rules())
            } else if let Some(prefix) = &classified.prefix {
                snippercore::match_prefix(prefix, &snippercore::built_in_csharp_prefix_rules())
            } else {
                vec![]
            }
        }
        "typescript" | "ts" | "typescriptreact" | "tsx" => {
            if let Some(postfix) = &classified.postfix {
                snippercore::match_postfix(
                    postfix,
                    &snippercore::built_in_typescript_postfix_rules(),
                )
            } else if let Some(prefix) = &classified.prefix {
                snippercore::match_prefix(prefix, &snippercore::built_in_typescript_prefix_rules())
            } else {
                vec![]
            }
        }
        _ => vec![],
    };

    // Emit TextEdit[] only — callers do not need trigger/label metadata.
    let edits: Vec<&snippercore::TextEdit> = candidates.iter().map(|c| &c.edit).collect();
    match serde_json::to_string_pretty(&edits) {
        Ok(json) => {
            println!("{json}");
            ExitCode::Ok
        }
        Err(e) => {
            eprintln!("error: serialization failed: {e}");
            ExitCode::DataErr
        }
    }
}

const fn lexical_class_str(class: LexicalClass) -> &'static str {
    match class {
        LexicalClass::CodeAfterDot => "code_after_dot",
        LexicalClass::CodeBareIdentifier => "code_bare_identifier",
        LexicalClass::StringLiteral => "string_literal",
        LexicalClass::Comment => "comment",
        LexicalClass::IdentifierDeclaration => "identifier_declaration",
        LexicalClass::Other => "other",
        // Exhaustiveness: LexicalClass is #[non_exhaustive], this arm is unreachable
        // in practice but required by the compiler when matching outside the crate.
        _ => "unknown",
    }
}
