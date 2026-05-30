# ADR 0001: Use SemantixTrace doc/lint template; record open blockers

Date: 2026-05-30
Status: Accepted

## Context

The task spec requires documentation and lint configuration to mirror the
`PhysShell/SemantixTrace` repository. SemantixTrace is publicly accessible;
all root config files (`rustfmt.toml`, `clippy.toml`, `deny.toml`,
`rust-toolchain.toml`, `.editorconfig`, `.gitattributes`, CI workflow
structure) have been read directly from the repository at commit
`472c0167`. The markdown linter config (`.markdownlint-cli2.jsonc`) and
`typos.toml` are Snipper-specific additions not present in SemantixTrace.

Five blockers from task spec §11 require human confirmation before they can
be finalised:

1. MSRV — working default: `1.85`.
2. License — working default: `MIT`.
3. Tier-1 backend (Tree-sitter vs ast-grep) — open; see ADR-0002.
4. Documentation language — working default: English.
5. Markdown linter config source — SemantixTrace has no
   `.markdownlint-cli2.jsonc`; the file in this repo uses the defaults
   from task spec §7.1.

## Decision

We adopt SemantixTrace config files verbatim where they exist and use the
task-spec defaults for files not present in SemantixTrace. All `[DEFAULT]`
items in the task spec are replaced by actual files. Open blockers are
recorded above and remain open until the repository owner provides answers.

## Consequences

- Config files track SemantixTrace; divergence requires a new ADR.
- The five blockers above may cause minor rework when resolved (e.g. MSRV
  bump, license change).
- Until ADR-0002 is resolved, the Tier-1 backend choice is deferred and
  the differential fuzz test is the decision mechanism.
