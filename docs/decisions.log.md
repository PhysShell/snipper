---
title: Decisions Log
status: stable
owners: []
updated: 2026-06-02
---

# Decisions Log

Small decisions that do not warrant a full ADR. Append-only. Y-statement
format: _In the context of X, facing Y, we decided Z, to achieve W, accepting
drawback D._

---

- **2026-05-30** — In the context of the initial scaffold, facing a choice of
  MSRV, we decided to use `1.85` (matching SemantixTrace), to minimise
  divergence, accepting that it may need updating when Tree-sitter bindings
  require a newer edition.

- **2026-05-30** — In the context of the initial scaffold, facing a choice of
  license, we decided to use MIT (matching SemantixTrace), to keep supply-chain
  simple, accepting that contributors must agree to MIT.

- **2026-05-30** — In the context of fuzz corpus layout, facing the need to
  track seed files but not generated artifacts, we decided to git-ignore
  `fuzz/corpus/` and require `git add -f` for deliberate seed commits,
  accepting the extra step in exchange for keeping the repo lean.

- **2026-06-02** — In the context of surfacing Snipper to real editors (S12),
  facing a choice between shipping a universal LSP configuration guide versus
  dedicated editor extensions, we decided to ship thin wrapper extensions for
  VS Code and Visual Studio first (ADR-0008), to achieve zero-config
  installation for the primary C# developer audience, accepting the
  maintenance overhead of two separate extension projects and deferred support
  for other editors.

- **2026-06-02** — In the context of S12 editor extensions needing command
  registrations that mirror `snippets/csharp/commands.toml`, facing the
  choice between hand-maintaining VS Code `package.json` and Visual Studio C#
  command constants versus code generation, we decided to generate both
  artefacts from TOML via `xtask generate-extension-manifests` (ADR-0009),
  to have a single source of truth, accepting the added `xtask` crate and the
  requirement to re-run the generator after TOML changes.

- **2026-06-02** — In the context of S12 settings for binary paths, facing
  the choice between env-var-only configuration versus a three-tier hierarchy,
  we decided on IDE setting → LSP `initializationOptions` → env fallback, to
  give IDE users a first-class settings experience, accepting that
  `snipper-lsp` must parse `initializationOptions` on `initialize`.

- **2026-05-31** — In the context of C# receiver-type information for
  type-aware template filtering, facing a choice between tree-sitter
  heuristics (unreliable: gives structure, not types) and the Roslyn semantic
  API (accurate, requires a .NET sidecar), we decided to position the Roslyn
  sidecar as a first-class planned component (S8) rather than a deferred
  nice-to-have, to achieve reliable type-aware filtering (e.g. `fod` only for
  collections) and lay the groundwork for smart ranking (S11), accepting the
  .NET dependency and sidecar startup latency as engineering costs.
