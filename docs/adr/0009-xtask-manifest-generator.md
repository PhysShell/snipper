# ADR-0009: Generate extension manifests from TOML rules

Date: 2026-06-02
Status: Proposed

## Context

VS Code extensions declare commands in `package.json` (under `contributes.commands`).
Visual Studio VSIX extensions register commands in C# (command IDs, titles, routing
table). Both must stay in sync with the canonical command definitions in
`snippets/csharp/commands.toml` (`trigger`, `label` fields).

Hand-maintaining three copies of the same list guarantees drift: a `trigger` renamed in
TOML silently breaks the VS Code command palette until someone notices the mismatch in
production. This is a machine-synchronisation problem, not a human-discipline problem.

## Decision

We add an `xtask` Cargo crate (`crates/xtask`) with a single subcommand:

```text
cargo run -p xtask -- generate-extension-manifests
```

The generator reads every `snippets/**/*.toml` file, collects rules with
`type = "command"`, and writes two artefacts:

- `extensions/snipper-vscode/package.generated.json` — a JSON fragment containing
  the VS Code `contributes.commands` array and `activationEvents` entries for each
  command. Merged into `package.json` at build time via `npm run build`.
- `extensions/snipper-vs/Generated/SnipperCommands.cs` — a `static class` with
  `const string` fields for each command ID (`snipper.<trigger>`) and title, consumed
  by `SnipperCommandPackage` and `SnipperCommandRouter`.

Generated files are committed to the repository. CI runs:

```text
cargo run -p xtask -- generate-extension-manifests
git diff --exit-code extensions/
```

A non-empty diff fails the build, enforcing that generated files are never stale.

## Consequences

- `snippets/csharp/commands.toml` is the single source of truth for command identity;
  adding a command requires editing only the TOML file.
- CI catches drift between TOML and generated artefacts automatically.
- Adding a third editor integration requires only a new generator output target; the
  TOML schema and `xtask` harness are already in place.
- Developers must run `cargo run -p xtask -- generate-extension-manifests` after
  changing command TOML files; the CI check makes forgetting visible immediately.
- The `xtask` crate has no production dependencies (no runtime, no tokio); it is a
  dev-time code-generation tool only.
