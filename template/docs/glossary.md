---
title: Glossary
---

# Glossary

Domain terms for this project, sorted alphabetically.

[TODO: Replace the placeholder terms below with your project's actual domain
terms. Remove this notice when done.]

---

## A

**ADR** (Architecture Decision Record) — A document in `docs/adr/` that
records a design decision: context, the decision, and consequences.
Append-only; superseded ADRs gain status _Superseded_ and link to the
replacement.

## C

**corpus** — The directory of seed inputs for a fuzz target, stored under
`fuzz/corpus/<target-name>/` and excluded from git.

## F

**fuzz target** — A function exercised by the fuzzer with randomized inputs.
Implements the libFuzzer protocol via the `fuzz_target!` macro.

## I

**INV** (Invariant) — A numbered property that must hold at all times.
Defined in `AGENTS.md` and verified by property tests and fuzz oracles.

## P

**proptest** — A property-based testing library. Generates random inputs and
checks that structural properties hold for all of them.

## W

**workspace** — The Cargo workspace root. Contains all publishable crates as
members; `fuzz/` is excluded and uses its own nightly toolchain.
