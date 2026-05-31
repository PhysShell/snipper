---
title: Fuzzing
---

# Fuzzing

[TODO: Describe the fuzz testing strategy for my-project.]

## Hard rules

1. Fuzz targets live in `fuzz/fuzz_targets/` and are excluded from the main
   workspace (`exclude = ["fuzz"]` in `Cargo.toml`).
2. Fuzz targets must not panic on valid input — panics are treated as bugs.
3. Corpus seeds live in `fuzz/corpus/<target>/`. Generated corpora are excluded
   from git via `.gitignore`.
4. A crash on any seed is a P0 bug; the fix must include the crashing input as
   a regression seed.

## Targets

| Target          | Covers           | Entry point         |
|-----------------|------------------|---------------------|
| `fuzz_target_1` | [TODO: describe] | [TODO: entry point] |

## Running locally

```sh
# 60-second smoke run (same as CI)
cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=60

# Unlimited run (Ctrl-C to stop)
cargo +nightly fuzz run fuzz_target_1
```

## CI policy

- `ci.yml` — `fuzz-build` verifies the harness compiles; `fuzz-smoke` runs
  each target for 60 seconds. Both are blocking gates.
- `fuzz-nightly.yml` — scheduled at 02:00 UTC daily; runs for 900 seconds;
  uploads artifacts for 30 days.
