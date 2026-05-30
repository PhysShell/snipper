//! Fuzz target `render_template` — template rendering never panics.
//!
//! Oracles (docs/fuzzing.md):
//! - Render does not panic on any template body + captures.
//! - Tabstops (`$0`, `$1`, `${1:...}`) are syntactically valid in the output.
//! - `{{expr}}` is substituted exactly once per occurrence.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // Scaffold: wire to template renderer once implemented.
});
