//! Developer task runner for the Snipper workspace.
//!
//! Usage: `cargo run -p xtask -- <subcommand>`

use std::path::{Path, PathBuf};

use serde::Deserialize;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("generate-extension-manifests") => {
            let root = workspace_root();
            if let Err(e) = generate_extension_manifests(&root) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some(cmd) => {
            eprintln!("unknown command: {cmd}");
            eprintln!("usage: cargo run -p xtask -- generate-extension-manifests");
            std::process::exit(1);
        }
        None => {
            eprintln!("usage: cargo run -p xtask -- generate-extension-manifests");
        }
    }
}

/// Resolve the workspace root from `CARGO_MANIFEST_DIR` (set by cargo when
/// running `cargo run -p xtask`). The xtask crate lives at
/// `<root>/crates/xtask`, so we go up two levels.
fn workspace_root() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("must run via `cargo run -p xtask`");
    PathBuf::from(manifest_dir)
        .parent() // crates/
        .and_then(Path::parent) // workspace root
        .expect("workspace root exists")
        .to_owned()
}

// ── TOML rule types ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RuleFile {
    #[serde(default)]
    rules: Vec<RuleEntry>,
}

#[derive(Deserialize)]
struct RuleEntry {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    trigger: String,
    #[serde(default)]
    label: String,
}

// ── Collection ───────────────────────────────────────────────────────────────

fn collect_command_rules(
    snippets_dir: &Path,
) -> Result<Vec<RuleEntry>, Box<dyn std::error::Error>> {
    let mut rules = Vec::new();
    collect_toml_rules_recursive(snippets_dir, &mut rules)?;
    Ok(rules)
}

fn collect_toml_rules_recursive(
    dir: &Path,
    out: &mut Vec<RuleEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    // Deterministic order across OS/filesystems.
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_toml_rules_recursive(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            let content = std::fs::read_to_string(&path)?;
            let file: RuleFile = toml::from_str(&content)?;
            for rule in file.rules {
                if rule.kind == "command" {
                    out.push(rule);
                }
            }
        }
    }
    Ok(())
}

// ── Generation ───────────────────────────────────────────────────────────────

fn generate_extension_manifests(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let snippets_dir = root.join("snippets");
    let rules = collect_command_rules(&snippets_dir)?;

    if rules.is_empty() {
        eprintln!("warning: no command rules found in {}", snippets_dir.display());
    }

    generate_vscode_manifest(root, &rules)?;
    generate_vscode_commands_ts(root, &rules)?;
    generate_vs_commands(root, &rules)?;

    println!("generated {} command(s):", rules.len());
    for r in &rules {
        println!("  snipper.{}  —  {}", r.trigger, r.label);
    }

    Ok(())
}

fn generate_vscode_manifest(
    root: &Path,
    rules: &[RuleEntry],
) -> Result<(), Box<dyn std::error::Error>> {
    let pkg_path = root
        .join("extensions")
        .join("snipper-vscode")
        .join("package.json");

    let content = std::fs::read_to_string(&pkg_path)?;
    let mut pkg: serde_json::Value = serde_json::from_str(&content)?;

    let commands: Vec<serde_json::Value> = rules
        .iter()
        .map(|r| {
            serde_json::json!({
                "command": format!("snipper.{}", r.trigger),
                "title": r.label,
                "category": "Snipper"
            })
        })
        .collect();

    // Update only contributes.commands; activationEvents and settings are
    // managed by hand in package.json and must not be overwritten here.
    pkg["contributes"]["commands"] = serde_json::Value::Array(commands);

    std::fs::write(&pkg_path, format!("{}\n", serde_json::to_string_pretty(&pkg)?))?;
    println!("  updated {}", pkg_path.display());

    Ok(())
}

fn generate_vscode_commands_ts(
    root: &Path,
    rules: &[RuleEntry],
) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = root
        .join("extensions")
        .join("snipper-vscode")
        .join("src")
        .join("commands.generated.ts");

    let ids: Vec<String> = rules
        .iter()
        .map(|r| format!("  \"snipper.{}\",", r.trigger))
        .collect();

    let content = format!(
        "// auto-generated — do not edit. Run: cargo run -p xtask -- generate-extension-manifests\n\nexport const SNIPPER_COMMANDS = [\n{}\n] as const;\n",
        ids.join("\n")
    );

    std::fs::write(&out_path, content)?;
    println!("  wrote {}", out_path.display());

    Ok(())
}

fn generate_vs_commands(
    root: &Path,
    rules: &[RuleEntry],
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = root
        .join("extensions")
        .join("snipper-vs")
        .join("Generated");
    std::fs::create_dir_all(&out_dir)?;

    let mut cs = String::new();
    cs.push_str("// <auto-generated>\n");
    cs.push_str(
        "// Do not edit — generated by: cargo run -p xtask -- generate-extension-manifests\n",
    );
    cs.push_str("// </auto-generated>\n");
    cs.push('\n');
    cs.push_str("namespace Snipper.VisualStudio;\n");
    cs.push('\n');
    cs.push_str("internal static class SnipperCommands\n");
    cs.push_str("{\n");

    for rule in rules {
        let name = pascal_case(&rule.trigger);
        cs.push_str(&format!(
            "    public const string {name}Id = \"snipper.{}\";\n",
            rule.trigger
        ));
        cs.push_str(&format!(
            "    public const string {name}Title = {:?};\n",
            rule.label
        ));
        cs.push('\n');
    }

    cs.push_str("}\n");

    let out_path = out_dir.join("SnipperCommands.cs");
    std::fs::write(&out_path, cs)?;
    println!("  wrote {}", out_path.display());

    Ok(())
}

/// Capitalise the first character; leave the rest unchanged (trigger is
/// already camelCase so the result is PascalCase without further changes).
fn pascal_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_case_converts_camel() {
        assert_eq!(pascal_case("scaffoldConstructor"), "ScaffoldConstructor");
        assert_eq!(pascal_case("implementInterface"), "ImplementInterface");
        assert_eq!(pascal_case(""), "");
    }
}
