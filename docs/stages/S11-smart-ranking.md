# S11 — Smart Ranking + Local Usage Statistics

| Field | Value |
|---|---|
| Status | Deferred (research) |
| Depends on | [S8](S8-roslyn-sidecar.md) |
| ADRs | — |

## Goal

Personalise completion ranking by accumulating local statistics on which
templates the user picks in which semantic contexts, so frequently-chosen
templates surface first within a given receiver type.

## Inputs → Outputs

**In:** type-aware filtering (S8); real usage data.

**Out:** local ranking database; context-weighted scoring promoting the
most relevant template for each `(receiver_type, trigger_prefix)` pair.

## Approach

1. Define a local stats store (SQLite or flat JSON): key =
   `(language, receiver_type_hash, expansion_type, template_id)`,
   value = `pick_count`.
2. On each `completionItem/resolve`, increment the counter for the
   accepted item.
3. Integrate frequency score into ranking: highest `pick_count` in context
   ranks first.
4. Fall back to alphabetical when no statistics exist.
5. Add `snipper stats reset` CLI command to clear local data.

## Acceptance criteria

- After picking `fod` 5 times on a `List<T>` receiver, `fod` ranks first
  for subsequent `List<T>` receivers.
- Stats file is local-only; no network calls.
- `snipper stats reset` clears all counters.

## Open questions

- Storage location: XDG data dir vs workspace-local `.snipper/`?
- Privacy: should stats be opt-in?
- Decay: should old picks decay over time?

## See also

- [Architecture](../architecture.md)
- [S8 — Roslyn sidecar](S8-roslyn-sidecar.md)
