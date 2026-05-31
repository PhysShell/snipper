//! Fuzz target `receiver_range` — TextEdit range invariants.
//!
//! Oracles (docs/fuzzing.md § INV-2, INV-3):
//! - (a) computed range is within document bounds;
//! - (b) range covers the full receiver expression;
//! - (c) applying the edit and re-parsing does not duplicate the receiver node.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // Scaffold: wire to snipper_context range extraction once implemented.
});
