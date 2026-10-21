//! Text Decoration 4 (current editor's draft) `text-decoration` grouping.
//!
//! `text-decoration` is `<'text-decoration-line'> || <'text-decoration-thickness'>
//! || <'text-decoration-style'> || <'text-decoration-color'>`, and
//! `text-decoration-line` is `none | [ underline || overline || line-through ||
//! blink ] | spelling-error | grammar-error`. Values 4 §2.2: combinators are not
//! associative, so the whole line value is one operand of the outer `||`. Its
//! keywords may appear in any internal order but never around another operand.
//! Specified serialization follows the grammar's canonical order (line,
//! thickness, style, color; line keywords in their grammar order).

use crate::css::support::authored_property::{ParserFront, invalid};
use surgeist_css::*;

const PROPERTY: CssKnownProperty = CssKnownProperty::TextDecoration;
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];

#[test]
fn contiguous_line_keywords_are_accepted_in_any_internal_order() {
    for (input, expected) in [
        ("underline overline red", "underline overline red"),
        ("overline underline red", "underline overline red"),
        ("red overline underline", "underline overline red"),
        ("wavy blink line-through 2px", "line-through blink 2px wavy"),
        (
            "dotted from-font underline blink overline red",
            "underline overline blink from-font dotted red",
        ),
        ("red none", "none red"),
        ("wavy spelling-error", "spelling-error wavy"),
        ("grammar-error 1px", "grammar-error 1px"),
    ] {
        for front in FRONTS {
            front.valid(PROPERTY, input, expected);
        }
    }
}

#[test]
fn line_keywords_split_by_another_operand_are_rejected() {
    for value in [
        // Split by color, style and thickness (length and keyword forms).
        "underline red overline",
        "underline wavy overline",
        "underline 2px overline",
        "underline auto line-through",
        "overline from-font blink",
        // Split with the other operands also present around the line value.
        "red underline wavy overline 1px",
        "underline red blink wavy",
        "line-through dotted underline currentcolor",
    ] {
        invalid(PROPERTY, value);
    }
}

#[test]
fn single_keyword_line_alternatives_cannot_join_or_repeat_across_operands() {
    for value in [
        "none red none",
        "none red underline",
        "underline red none",
        "spelling-error red grammar-error",
        "spelling-error red underline",
        "grammar-error wavy grammar-error",
        "blink red blink",
    ] {
        invalid(PROPERTY, value);
    }
}
