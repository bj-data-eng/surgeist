//! Bounded structural replay of many independently deep rules does work in
//! proportion to the sheet and holds replay frames bounded by nesting depth,
//! not by the number of deep rules.
//!
//! Each fixture is valid CSS: `count` sibling style rules, each nested `depth`
//! levels deep. The work measure is the tokens walked by structural preflight;
//! the stack measure is the deepest live nesting of bounded replay parses.

use super::reconstruction::{REPLAY_DEPTH, REPLAY_MAX_DEPTH, REPLAY_PARSES};
use super::recovery::PREFLIGHT_TOKENS;
use crate::rule::CssRule;
use crate::tokenization::PREPARED_BYTES;

fn sheet(depth: usize, count: usize) -> String {
    let rule = format!("{}{}", "a{".repeat(depth), "}".repeat(depth));
    rule.repeat(count)
}

struct Measure {
    tokens_per_byte: f64,
    parses: usize,
    max_depth: usize,
}

fn measure(depth: usize, count: usize) -> Measure {
    let source = sheet(depth, count);
    PREFLIGHT_TOKENS.with(|tokens| tokens.set(0));
    REPLAY_PARSES.with(|parses| parses.set(0));
    REPLAY_DEPTH.with(|live| live.set(0));
    REPLAY_MAX_DEPTH.with(|maximum| maximum.set(0));
    let report = crate::CssParser::default().parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), count);
    let mut rule = &report.syntax().rules()[count - 1];
    for _ in 1..depth {
        let CssRule::Style(style) = rule else {
            panic!("nested style chain")
        };
        let [child] = style.rules() else {
            panic!("one nested child")
        };
        rule = child;
    }
    Measure {
        tokens_per_byte: PREFLIGHT_TOKENS.with(std::cell::Cell::get) as f64 / source.len() as f64,
        parses: REPLAY_PARSES.with(std::cell::Cell::get),
        max_depth: REPLAY_MAX_DEPTH.with(std::cell::Cell::get),
    }
}

#[test]
fn deep_sibling_replay_work_is_linear_and_replay_nesting_is_independent_of_rule_count() {
    for depth in [32, 64] {
        let few = measure(depth, 25);
        for count in [50, 100] {
            let many = measure(depth, count);
            println!(
                "depth {depth}: 25 rules {:.2} tokens/byte, {} parses, nesting {}; {count} rules {:.2} tokens/byte, {} parses, nesting {}",
                few.tokens_per_byte,
                few.parses,
                few.max_depth,
                many.tokens_per_byte,
                many.parses,
                many.max_depth
            );
            // Every byte is walked a bounded number of times: a full-source
            // preflight per deep rule would grow with the rule count.
            assert!(
                many.tokens_per_byte <= few.tokens_per_byte + 1.0,
                "depth {depth}: {count} rules walk {:.2} tokens per byte, 25 walk {:.2}",
                many.tokens_per_byte,
                few.tokens_per_byte
            );
            // Live replay frames depend on nesting depth, not on siblings.
            assert_eq!(
                many.max_depth, few.max_depth,
                "depth {depth}, {count} rules"
            );
            // One outer parse plus a bounded number of parses per deep rule.
            assert!(
                many.parses <= 4 * count + 4,
                "depth {depth}: {count} rules start {} replay parses",
                many.parses
            );
        }
    }
}

// Bytes prepared for one parse, with the bounded replay parses it started.
fn prepared(depth: usize, count: usize) -> (usize, usize, usize) {
    let source = sheet(depth, count);
    PREPARED_BYTES.with(|prepared| prepared.set(0));
    REPLAY_PARSES.with(|parses| parses.set(0));
    let report = crate::CssParser::default().parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    (
        PREPARED_BYTES.with(std::cell::Cell::get),
        REPLAY_PARSES.with(std::cell::Cell::get),
        source.len(),
    )
}

// Every replay parse reads a view as long as the whole source, so prepared
// bytes are bounded by one source-length preparation per replay parse; they
// are not yet linear in the sheet when many deep siblings each start a child
// replay. This bound keeps a parse from preparing its view more than once.
#[test]
fn deep_child_replays_prepare_at_most_one_source_length_view_each() {
    for depth in [32, 64, 96] {
        for count in [25, 50, 100] {
            let (bytes, parses, length) = prepared(depth, count);
            println!(
                "depth {depth}, {count} rules: {length} bytes, {parses} parses, {bytes} prepared ({:.2} per byte)",
                bytes as f64 / length as f64
            );
            assert!(bytes > 0 && parses > 0, "depth {depth}, {count} rules");
            assert!(
                bytes <= parses * length,
                "depth {depth}, {count} rules: {bytes} prepared bytes for {parses} parses of {length}"
            );
        }
    }
}

#[test]
fn many_deep_siblings_parse_on_an_ordinary_test_thread() {
    for (depth, count) in [(32, 400), (64, 200)] {
        let source = sheet(depth, count);
        let report = crate::CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), count);
    }
}
