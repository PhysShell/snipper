---
title: "ADR 0001: Documentation and tooling setup"
---

# ADR 0001: Documentation and tooling setup

Date: [YYYY-MM-DD]
Status: Accepted

## Context

A new Rust workspace project needs consistent tooling, documentation
conventions, and CI from day one. Setting these up ad-hoc leads to
inconsistency and technical debt that compounds as the project grows.

This repository was created from `physshell/rust-project-template`, which
provides an opinionated scaffold covering:

- Lint gate: markdownlint-cli2, typos, rustfmt, clippy (pedantic + nursery +
  cargo lint groups), cargo-deny
- Fuzz scaffold: cargo-fuzz with CI smoke and nightly deep runs
- Documentation: AGENTS.md, ADR log, architecture notes, glossary
- Rust API Guidelines enforcement via workspace lint attrs

## Decision

Adopt the template scaffold as-is. Customize all placeholder content
(`my-project` → actual project name; `[TODO: ...]` → real content).
Record deviations from the template defaults as follow-up ADRs.

## Consequences

- The full lint gate runs on every push and pull request.
- Documentation structure is established from day one.
- Future ADRs build on this numbered log.
