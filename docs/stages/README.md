# Staged Delivery

Each stage has a single goal, explicit inputs/outputs, and measurable
acceptance criteria. Stages complete in order; a later stage may only
begin once all prior stages' acceptance criteria are green.

| Stage | Goal | Status |
|---|---|---|
| [S0](S0-baseline-workspace.md) | Baseline workspace | Done |
| [S1](S1-csharp-cst-classifier.md) | C# CST classifier | Done |
| [S2](S2-test-harness.md) | Golden fixtures + fuzz | Not started |
| [S3](S3-postfix-template-engine.md) | Postfix template engine | Not started |
| [S4](S4-lsp-adapter-mvp.md) | LSP adapter MVP | Not started |
| [S5](S5-prefix-expansions.md) | Prefix expansions + conflict strategy | Not started |
| [S6](S6-surround-selection.md) | Surround & selection expansions | Not started |
| [S7](S7-multi-language.md) | Multi-language + backend resolution | Not started |
| [S8](S8-roslyn-sidecar.md) | Roslyn sidecar + semantic enrichment | Not started |
| [S9](S9-command-expansions.md) | Command expansions | Not started |
| [S10](S10-reactor-core.md) | Reactor portable core hardening | Not started |
| [S11](S11-smart-ranking.md) | Smart ranking + local usage stats | Deferred |

## Expansion types

Snipper supports five expansion types, implemented across stages S3–S9:

| Type | Trigger | Stage |
|---|---|---|
| Postfix | `<receiver>.trigger` | S3 |
| Prefix | bare keyword at expression start | S5 |
| Surround | wraps selected text | S6 |
| Selection | selection-aware context | S6 |
| Command | editor command palette | S9 |
