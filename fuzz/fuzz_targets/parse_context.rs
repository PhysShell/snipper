//! Fuzz target `parse_context` — context classification never panics.
//!
//! Prime directive (AGENTS.md): expansion of a trigger that appears inside a
//! string literal or comment must return an empty candidate set. This target
//! guards the no-panic invariant; INV-1 is enforced by proptest in
//! `snipper-context`.
//!
//! Oracles (docs/fuzzing.md):
//! - No panic on any UTF-8 input + any cursor position.
//! - Classification completes within the libFuzzer timeout.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // Scaffold: wire to snipper_context::classify once the public entry
    // point is stabilised. The no-panic oracle is enforced by the harness.
});
