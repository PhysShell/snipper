---
title: Agent Conventions
status: stable
owners: []
updated: 2026-05-30
---

# Agent Conventions

Rules for writing documentation that agents parse unambiguously and that
passes markdownlint.

## Headings

- Strictly hierarchical: no level skips (H1 → H2 → H3, never H1 → H3).
- One H1 per file.
- ATX style only (`#`, not underline).

## Code fences

- Every fenced block has a language tag (`rust`, `toml`, `text`, `json`,
  `sh`, `yaml`).
- Bare fences (` ``` ` with no tag) are forbidden (MD040).

## Commands

Commands that an agent must execute go in a `text`-tagged code fence, one
command per line, no `$` prefix:

```text
cargo test --workspace
just lint
```

## Normative language

Requirements use **must / must not / required / forbidden**. Avoid
"should" or "ideally" for anything machine-checkable.

## Decisions and open questions

- Every architectural decision gets an ADR in `docs/adr/`.
- Open decisions are marked `DECISION NEEDED` and link to their ADR.
- Nothing checkable lives only in prose; checkable things go in
  Definition of Done checklists.

## Frontmatter

Every file in `docs/` starts with YAML frontmatter:

```yaml
---
title: <title>
status: draft | review | stable
owners: [<github-handle>]
updated: <YYYY-MM-DD>
---
```

## Line length

Prose lines are wrapped at 100 characters. Code blocks and tables are exempt
(markdownlint MD013 `code_blocks: false, tables: false`).
