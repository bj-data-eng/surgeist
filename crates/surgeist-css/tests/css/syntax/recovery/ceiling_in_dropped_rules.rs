//! The shared structural ceiling charges every kind of block inside a dropped
//! rule alike.
//!
//! Authored rules and component-value trees share one ceiling of 256 levels
//! (`STRUCTURAL_NESTING_LIMIT`; `CssErrorCode::NestingLimit` is "authored
//! nesting exceeded the parser's configured structural limit"), and a failure to
//! enter a block for that limit takes precedence over the failure of the rule
//! that holds the block. A rule dropped for an invalid selector is such a
//! holder: its block is charged for what entering it would have cost, so its
//! curly, parenthesis and square nesting reaches the ceiling at the same depth
//! and stops the unit with `NestingLimit`, where one level fewer leaves the drop
//! as it was.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::CssRule;
use surgeist_css::selector::CssNamespaceContext;

const LIMIT: usize = 256;

/// The nested blocks of the three kinds: opener and closer.
const KINDS: [(&str, &str, &str); 3] = [
    ("curly", "{", "}"),
    ("paren", "(", ")"),
    ("square", "[", "]"),
];

/// One way into the parser. `enclosing` is how many blocks the dropped rule
/// already stands in, so that its own block is the next level.
struct Entry {
    name: &'static str,
    head: &'static str,
    tail: &'static str,
    enclosing: usize,
    single_rule: bool,
}

const ENTRIES: [Entry; 4] = [
    Entry {
        name: "the sheet driver",
        head: "",
        tail: "",
        enclosing: 0,
        single_rule: false,
    },
    Entry {
        name: "CssRule::parse",
        head: "",
        tail: "",
        enclosing: 0,
        single_rule: true,
    },
    Entry {
        name: "a rule nested in a style rule",
        head: "a{",
        tail: "}",
        enclosing: 1,
        single_rule: false,
    },
    Entry {
        name: "an @media body",
        head: "@media x{",
        tail: "}",
        enclosing: 1,
        single_rule: false,
    },
];

/// What a parse reported about the ceiling and about the dropped rule.
struct Outcome {
    limit_offset: Option<usize>,
    limits: usize,
    dropped_for_selector: bool,
}

fn outcome(entry: &Entry, open: &str, close: &str, depth: usize) -> (Outcome, usize) {
    let rule_open = format!("{}b:{{", entry.head);
    let text = format!(
        "{rule_open}{}{}}}{}",
        open.repeat(depth),
        close.repeat(depth),
        entry.tail
    );
    let parser = CssParser::default();
    let diagnostics = if entry.single_rule {
        CssRule::parse(&parser, &text, &CssNamespaceContext::default())
            .diagnostics()
            .to_vec()
    } else {
        parser.parse_sheet(&text).diagnostics().to_vec()
    };
    let limits: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::StopAtNestingLimit)
        .collect();
    for limit in &limits {
        assert_eq!(
            limit.error().code(),
            CssErrorCode::NestingLimit,
            "{text:.40}"
        );
        assert_eq!(limit.error().nesting_limit(), Some(LIMIT as u32));
    }
    let result = Outcome {
        limit_offset: limits
            .first()
            .map(|limit| limit.error().position().byte_offset().value()),
        limits: limits.len(),
        dropped_for_selector: diagnostics
            .iter()
            .any(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidSelector),
    };
    (result, rule_open.len())
}

#[test]
fn every_kind_of_block_in_a_dropped_rule_reaches_the_ceiling_at_the_same_depth() {
    for entry in &ENTRIES {
        // The dropped rule's own block is one level; the nested blocks add to it.
        let first_over = LIMIT - entry.enclosing;
        for (kind, open, close) in KINDS {
            let name = format!("{} / {kind}", entry.name);
            // The last level that still fits.
            let (fits, _) = outcome(entry, open, close, first_over - 1);
            assert_eq!(fits.limits, 0, "{name}: 255 levels fit");
            assert!(
                fits.dropped_for_selector,
                "{name}: dropped for its selector"
            );
            // The first level past the ceiling stops the unit at that opening.
            for depth in [first_over, first_over + 1, first_over + 40] {
                let (over, opening_length) = outcome(entry, open, close, depth);
                assert_eq!(over.limits, 1, "{name}: one limit at depth {depth}");
                assert_eq!(
                    over.limit_offset,
                    Some(opening_length + first_over - 1),
                    "{name}: the first opening past the ceiling at depth {depth}"
                );
            }
        }
    }
}

#[test]
fn a_curly_block_and_a_parenthesis_block_at_the_same_depth_give_the_same_result() {
    for entry in &ENTRIES {
        let first_over = LIMIT - entry.enclosing;
        for depth in [first_over - 1, first_over, first_over + 7] {
            let mut results = KINDS.map(|(_, open, close)| {
                let (result, _) = outcome(entry, open, close, depth);
                (
                    result.limit_offset,
                    result.limits,
                    result.dropped_for_selector,
                )
            });
            let curly = results[0];
            for (index, result) in results.iter_mut().enumerate() {
                assert_eq!(*result, curly, "{} depth {depth} kind {index}", entry.name);
            }
        }
    }
}
