//! Grid 2 (CRD 2025-03-26, unchanged in the current draft) `<grid-line>`
//! grouping:
//!
//! ```text
//! <grid-line> = auto | <custom-ident> |
//!   [ [ <integer [-∞,-1]> | <integer [1,∞]> ] && <custom-ident>? ] |
//!   [ span && [ <integer [1,∞]> || <custom-ident> ] ]
//! ```
//!
//! The bracketed `||` group is one operand of `&&`, so by Values 4 §2.2 (not
//! associative) `span` precedes or follows the whole integer/name group but
//! never separates its members. The integer and name of the non-span form
//! are `&&` operands and so are order-free. Specified serialization uses the
//! grammar's canonical order (`span`, integer, name).

use crate::css::support::authored_property::{ParserFront, invalid};
use surgeist_css::property::CssKnownProperty;

const LONGHANDS: [&str; 4] = [
    "grid-row-start",
    "grid-row-end",
    "grid-column-start",
    "grid-column-end",
];
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];
const SPLIT_SPAN: [&str; 6] = [
    "2 span foo",
    "foo span 2",
    "3 SPAN bar",
    "bar span 3",
    "calc(2) span foo",
    "foo span calc(2)",
];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap_or_else(|| panic!("known property {name}"))
}

#[test]
fn span_before_or_after_the_whole_group_is_accepted() {
    for (input, expected) in [
        ("span 2 foo", "span 2 foo"),
        ("span foo 2", "span 2 foo"),
        ("2 foo span", "span 2 foo"),
        ("foo 2 span", "span 2 foo"),
        ("span foo", "span foo"),
        ("foo span", "span foo"),
        ("span 3", "span 3"),
        ("3 span", "span 3"),
    ] {
        for name in LONGHANDS {
            for front in FRONTS {
                front.valid(property(name), input, expected);
            }
        }
    }
}

#[test]
fn span_between_the_integer_and_the_name_is_rejected_in_longhands() {
    for name in LONGHANDS {
        for value in SPLIT_SPAN {
            invalid(property(name), value);
        }
        for value in ["span 2 span", "span foo span", "span 2 foo span"] {
            invalid(property(name), value);
        }
    }
}

#[test]
fn span_between_the_integer_and_the_name_is_rejected_in_shorthands() {
    for value in SPLIT_SPAN {
        for (name, template) in [
            ("grid-row", ["{} / 3", "1 / {}"].as_slice()),
            ("grid-column", ["{} / 3", "1 / {}"].as_slice()),
            (
                "grid-area",
                ["{}", "{} / 1", "1 / {}", "1 / 2 / {}", "1 / 2 / 3 / {}"].as_slice(),
            ),
        ] {
            for shape in template {
                invalid(property(name), &shape.replace("{}", value));
            }
        }
    }
    for (name, value) in [
        ("grid-row", "span 2 foo / foo 2 span"),
        ("grid-column", "2 foo span / span foo 2"),
        ("grid-area", "span 2 foo / 2 foo span / foo span / 3 span"),
    ] {
        for front in FRONTS {
            front.parse(property(name), value);
        }
    }
}

#[test]
fn integer_and_name_without_span_are_order_free_and_single() {
    for (input, expected) in [
        ("2 foo", "2 foo"),
        ("foo 2", "2 foo"),
        ("-1 foo", "-1 foo"),
        ("foo -1", "-1 foo"),
        ("foo", "foo"),
        ("4", "4"),
    ] {
        for name in LONGHANDS {
            for front in FRONTS {
                front.valid(property(name), input, expected);
            }
        }
    }
    for name in LONGHANDS {
        for value in ["2 foo 3", "foo 2 bar", "2 3", "foo bar", "0 foo", "foo 0"] {
            invalid(property(name), value);
        }
    }
}
