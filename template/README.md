---
title: Rust Project Template
---

# Rust Project Template

Batteries-included starting point for Rust workspace projects.

## What is included

- **Lint gate**: markdownlint-cli2, typos, rustfmt, clippy (pedantic + nursery +
  cargo lint groups), cargo-deny
- **Fuzz scaffold**: cargo-fuzz target with CI smoke run (~60 s) and scheduled
  nightly deep run (~15 min)
- **Documentation skeleton**: AGENTS.md agent guide, ADR log, architecture notes,
  glossary with placeholder terms
- **CI**: `ci.yml` (lint / test / fuzz-smoke / public-api) and `fuzz-nightly.yml`
- **Rust API Guidelines**: workspace-level lint attrs enforce C-* guidelines from
  day one

## Getting started

**Step 1.** Click _Use this template_ → _Create a new repository_.

**Step 2.** Clone your new repository, then replace `my-project` with your
project name throughout all files:

```sh
grep -rl 'my-project' . \
  --include='*.toml' --include='*.rs' \
  --include='*.md' --include='*.yml' \
  | xargs sed -i 's/my-project/your-project/g'
```

**Step 3.** Update `Cargo.toml` — fill in `authors`, `repository`, `homepage`,
`documentation`, `description`, `keywords`, and `categories` under
`[workspace.package]`.

**Step 4.** Customize the documentation skeletons:

- `AGENTS.md` — project description, prime directive, invariants
- `docs/glossary.md` — replace placeholder terms with your domain terms
- `docs/architecture.md` — system overview and crate dependencies
- `docs/adr/0001-doc-template.md` — fill in the date and content

**Step 5.** Run the full gate locally to confirm everything passes:

```sh
just verify
```

## Tooling requirements

| Tool              | Install command                    |
|-------------------|------------------------------------|
| Rust (≥ 1.85)     | <https://rustup.rs>                |
| just              | `cargo install just`               |
| cargo-deny        | `cargo install cargo-deny`         |
| markdownlint-cli2 | `npm install -g markdownlint-cli2` |
| typos             | `cargo install typos-cli`          |

## Quick reference

```sh
just verify      # full gate: lint + test + fuzz-smoke
just lint        # markdownlint + typos + fmt + clippy + deny
just test        # cargo test --workspace --all-features
just fuzz-smoke  # 60-second smoke run of all fuzz targets
```

## Repository layout

```text
.
├── crates/
│   └── my-project-core/   # example core crate — rename or replace
├── docs/
│   ├── adr/               # Architecture Decision Records
│   ├── architecture.md
│   ├── fuzzing.md
│   └── glossary.md
├── fuzz/                  # isolated cargo-fuzz harness (nightly)
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── fuzz-nightly.yml
└── AGENTS.md              # guide for AI coding agents
```

## Enabling the template flag

In your repository: **Settings → General → Template repository** —
check the box so others can use it as a starting point.
