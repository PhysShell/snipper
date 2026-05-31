# Project Specification

> One sentence that defines the essence of the project.

## What my-project is

- **TODO**: bullet describing what the project *is*
- A Rust library / CLI / service that does X
- Designed for Y use-case

## What my-project is not

- **TODO**: bullet describing non-goals
- Not a general-purpose solution for Z

## Delivery shape

| Artefact | Kind | Notes |
|---|---|---|
| `my-project-core` | library crate | core logic |
| `my-project-cli` | binary crate | CLI interface |

## Hard rules

1. No `unsafe` code outside explicitly reviewed modules.
2. All public items must have doc comments.
3. `cargo deny check` must pass before any release.
4. **TODO**: add project-specific hard rules here.

## Current state

Stage S0 — baseline workspace wired up. See [stages/](stages/).

## See also

- [Architecture](architecture.md)
- [ADR index](adr/README.md)
- [Glossary](glossary.md)
