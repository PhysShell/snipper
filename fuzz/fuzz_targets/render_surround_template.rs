//! Fuzz target `render_surround_template` — surround template matching never panics.
//!
//! Oracles:
//! - `match_surround` does not panic on any selected text + rule body combination.
//! - All returned `Candidate` fields are accessible without panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use snippercore::{match_surround, Position, Range, Rule, RuleKind, SurroundContext};

#[derive(Debug, arbitrary::Arbitrary)]
struct Input {
    selected_text: String,
    rule_body: String,
}

fuzz_target!(|input: Input| {
    let sel_len = u32::try_from(input.selected_text.len()).unwrap_or(u32::MAX);
    let ctx = SurroundContext {
        selected_text: input.selected_text,
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: sel_len },
        },
    };
    let rules = [Rule {
        kind: RuleKind::Surround,
        trigger: "wrap".to_owned(),
        label: String::new(),
        body: input.rule_body,
        requires: None,
    }];
    let candidates = match_surround(&ctx, &rules);
    for c in &candidates {
        let _ = c.edit.new_text.len();
        let _ = c.trigger.len();
    }
});
