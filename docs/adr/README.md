# Architecture Decision Records

Nygard format, in-repo, append-only. After `Accepted`, an ADR is immutable;
supersede it with a new one. New ADRs: copy
[`0000-template.md`](0000-template.md), use the next id, link it here.

| ADR | Title | Status |
| --- | --- | --- |
| [0000](0000-template.md) | Template | — |
| [0001](0001-doc-template.md) | Use SemantixTrace doc/lint template; record open blockers | Accepted |
| [0002](0002-tier1-treesitter-vs-astgrep.md) | Tier-1 context backend: Tree-sitter vs ast-grep | Proposed |
| [0003](0003-representation-formats.md) | Normative representation formats and binary format deferral | Accepted |
| [0004](0004-follow-rust-api-guidelines-on-public-surfaces.md) | Follow the Rust API Guidelines on public surfaces | Accepted |
| [0005](0005-prefix-postfix-conflict-strategy.md) | Prefix/postfix conflict-resolution strategy | Accepted |
| [0006](0006-surround-prime-directive.md) | Surround expansion trigger and prime-directive enforcement | Accepted |
| [0007](0007-roslyn-sidecar-protocol.md) | Roslyn sidecar IPC protocol | Accepted |
| [0008](0008-editor-extension-packaging.md) | Package editor extensions as thin LSP-client wrappers | Proposed |

See also: [`../architecture.md`](../architecture.md),
[`../decisions.log.md`](../decisions.log.md).
