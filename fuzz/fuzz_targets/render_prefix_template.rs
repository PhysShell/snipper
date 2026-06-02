//! Fuzz target `render_prefix_template` — prefix template matching never panics.
//!
//! Oracles:
//! - `match_prefix` does not panic on any trigger + rule body combination.
//! - All returned `Candidate` fields are accessible without panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use snippercore::{match_prefix, Position, PrefixContext, Range, Rule, RuleKind};

#[derive(Debug, arbitrary::Arbitrary)]
struct Input {
    trigger: String,
    rule_trigger: String,
    rule_body: String,
}

fuzz_target!(|input: Input| {
    let trigger_len = u32::try_from(input.trigger.len()).unwrap_or(u32::MAX);
    let prefix = PrefixContext {
        trigger: input.trigger,
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: trigger_len },
        },
    };
    let rules = [Rule {
        kind: RuleKind::Prefix,
        trigger: input.rule_trigger,
        label: String::new(),
        body: input.rule_body,
        requires: None,
    }];
    let candidates = match_prefix(&prefix, &rules);
    for c in &candidates {
        let _ = c.edit.new_text.len();
        let _ = c.trigger.len();
    }
});
