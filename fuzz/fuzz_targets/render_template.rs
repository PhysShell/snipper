//! Fuzz target `render_template` — template matching never panics.
//!
//! Oracles:
//! - `match_postfix` does not panic on any receiver + trigger + rule body.
//! - All returned `Candidate` fields are accessible without panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use snippercore::{match_postfix, Position, PostfixContext, Range, Rule};

#[derive(Debug, arbitrary::Arbitrary)]
struct Input {
    receiver: String,
    trigger: String,
    rule_trigger: String,
    rule_body: String,
}

fuzz_target!(|input: Input| {
    let postfix = PostfixContext {
        receiver: input.receiver,
        trigger: input.trigger,
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 1 },
        },
    };
    let rules = [Rule {
        trigger: input.rule_trigger,
        label: String::new(),
        body: input.rule_body,
    }];
    let candidates = match_postfix(&postfix, &rules);
    for c in &candidates {
        let _ = c.edit.new_text.len();
        let _ = c.trigger.len();
    }
});
