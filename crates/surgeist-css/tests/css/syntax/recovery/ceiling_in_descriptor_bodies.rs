//! The structural ceiling charges a failed unit in a descriptor body like a
//! failed unit anywhere else.
//!
//! Authored rules and component-value trees share one ceiling of 256 levels
//! (`STRUCTURAL_NESTING_LIMIT`), and a failure to enter a block for that limit
//! takes precedence over the failure of the unit that holds the block. The
//! bodies of `@page` and its margin boxes, `@counter-style` and (as the
//! reference for the same input) `@font-face` stop a failed unit at the first
//! opening past the ceiling, for curly, parenthesis and square blocks alike,
//! and leave a unit one level shallower as it was.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};

const LIMIT: usize = 256;

/// The nested blocks of the three kinds: opener and closer.
const KINDS: [(&str, &str, &str); 3] = [
    ("curly", "{", "}"),
    ("paren", "(", ")"),
    ("square", "[", "]"),
];

/// A body: the text that opens it and closes it, and how many blocks a unit of
/// the body already stands in.
struct Body {
    name: &'static str,
    head: &'static str,
    tail: &'static str,
    enclosing: usize,
}

const PAGE: Body = Body {
    name: "@page",
    head: "@page{",
    tail: "}",
    enclosing: 1,
};

const MARGIN_BOX: Body = Body {
    name: "a page margin box",
    head: "@page{@top-left{",
    tail: "}}",
    enclosing: 2,
};

const COUNTER_STYLE: Body = Body {
    name: "@counter-style",
    head: "@counter-style x{",
    tail: "}",
    enclosing: 1,
};

const FONT_FACE: Body = Body {
    name: "@font-face",
    head: "@font-face{",
    tail: "}",
    enclosing: 1,
};

/// The units whose own block is the first level: a declaration look-alike, an
/// unknown at-rule and a qualified rule.
const UNITS: [(&str, &str); 3] = [
    ("a declaration look-alike", "b:{"),
    ("an unknown at-rule", "@x{"),
    ("a qualified rule", "x{"),
];

/// What a parse reported about the ceiling.
struct Outcome {
    limit_offset: Option<usize>,
    limits: usize,
    diagnostics: usize,
}

fn outcome(body: &Body, unit: &str, open: &str, close: &str, depth: usize) -> (Outcome, usize) {
    let unit_open = format!("{}{unit}", body.head);
    let text = format!(
        "{unit_open}{}{}}}{}",
        open.repeat(depth),
        close.repeat(depth),
        body.tail
    );
    let report = CssParser::default().parse_sheet(&text);
    let limits: Vec<_> = report
        .diagnostics()
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
        diagnostics: report.diagnostics().len(),
    };
    (result, unit_open.len())
}

/// Every unit shape and block kind reaches the ceiling at the same opening of
/// `body`: one level fewer leaves the unit dropped as it was.
fn assert_ceiling_in(body: &Body) {
    // The unit's own block is one level; the nested blocks add to it.
    let first_over = LIMIT - body.enclosing;
    for (unit_name, unit) in UNITS {
        for (kind, open, close) in KINDS {
            let name = format!("{} / {unit_name} / {kind}", body.name);
            let (fits, _) = outcome(body, unit, open, close, first_over - 1);
            assert_eq!(fits.limits, 0, "{name}: the last level that fits");
            assert!(fits.diagnostics >= 1, "{name}: the unit is still dropped");
            for depth in [first_over, first_over + 1, first_over + 40] {
                let (over, opening_length) = outcome(body, unit, open, close, depth);
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
fn a_failed_unit_in_a_page_body_stops_at_the_first_opening_past_the_ceiling() {
    assert_ceiling_in(&PAGE);
}

#[test]
fn a_failed_unit_in_a_margin_box_stops_at_the_first_opening_past_the_ceiling() {
    assert_ceiling_in(&MARGIN_BOX);
}

#[test]
fn a_failed_unit_in_a_counter_style_body_stops_at_the_first_opening_past_the_ceiling() {
    assert_ceiling_in(&COUNTER_STYLE);
}

#[test]
fn a_failed_unit_in_a_font_face_body_stops_at_the_first_opening_past_the_ceiling() {
    assert_ceiling_in(&FONT_FACE);
}
