//! A style-attribute declaration that starts with anything other than an
//! identifier is malformed, and its remnants are discarded through the next
//! root `;` or EOF.
//!
//! - CSS Style Attributes (REC 2013-11-07) §3: the value is parsed as declaration
//!   block contents, and a `}` "does not terminate the style data: it is merely
//!   an invalid token"; CSS 2.1 §4.2 "Malformed declarations" reads such a
//!   declaration "until the end of the declaration", observing matching pairs.
//! - Syntax 3 (2026 CRD) §5.5.6 "consume a declaration": a first token that is
//!   not an `<ident-token>` consumes the remnants of a bad declaration, which, not
//!   nested, discards a `}` and consumes component values (whole blocks and
//!   functions) until `;` or EOF.
//!   <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-the-remnants-of-a-bad-declaration>
use surgeist_css::*;

fn names(report: &CssParseReport<CssDeclarationList>) -> Vec<String> {
    report
        .syntax()
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name().to_owned(),
            CssPropertyNameRef::Custom(name) => name.as_str().to_owned(),
            _ => panic!("ordinary test name"),
        })
        .collect()
}

fn assert_remnant(source: &str, unit: &str, expected: &[&str]) {
    let report = parse_style_attribute(source);
    assert_eq!(names(&report), expected, "{source:?}: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("{source:?}: one malformed declaration: {report:?}");
    };
    let start = source.find(unit).unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
    assert_eq!(diagnostic.error().position().byte_offset().value(), start);
    assert_eq!(
        (
            diagnostic.span().start().byte_offset().value(),
            diagnostic.span().end().byte_offset().value()
        ),
        (start, start + unit.len()),
        "{source:?}"
    );
    assert_eq!(
        validate_style_attribute(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

const STARTS: [&str; 12] = [
    "}", ")", "]", "1px", "-2", "50%", "\"text\"", "'text'", "f(x)", "(a)", "[a]", "#hash",
];

#[test]
fn a_malformed_start_discards_its_remnants_through_the_next_semicolon() {
    for start in STARTS {
        let unit = format!("{start} color: red");
        assert_remnant(&format!("{unit}; width: 2px"), &unit, &["width"]);
        assert_remnant(&format!("{unit}; width: 2px;"), &unit, &["width"]);
        assert_remnant(
            &format!("top: 1px; {unit}; width: 2px"),
            &unit,
            &["top", "width"],
        );
    }
}

#[test]
fn a_malformed_start_without_a_semicolon_discards_through_eof() {
    for start in STARTS {
        let unit = format!("{start} color: red");
        assert_remnant(&unit, &unit, &[]);
        assert_remnant(&format!("top: 1px; {unit}"), &unit, &["top"]);
    }
}

#[test]
fn blocks_and_functions_inside_the_remnants_are_consumed_whole() {
    for unit in [
        "} {a: b; width: 1px} color: red",
        ") f(a; width: 1px) color: red",
        "] [a; width: 1px] color: red",
        "1px (a; width: 1px) color: red",
        "} } } color: red",
        "} \";\" /* ; */ color: red",
    ] {
        let source = format!("{unit}; width: 2px");
        assert_remnant(&source, unit, &["width"]);
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax()[0]
                .serialize_specified(Default::default())
                .unwrap(),
            "width: 2px;"
        );
    }
}

#[test]
fn the_raw_declaration_list_front_drops_the_same_remnant() {
    for start in STARTS {
        let unit = format!("{start} color: red");
        let source = format!("{unit}; width: 2px");
        let attribute = parse_style_attribute(&source);
        let raw = parse_declaration_list_text(&source);
        assert_eq!(attribute.syntax(), raw.syntax(), "{source:?}");
        assert_eq!(attribute.diagnostics(), raw.diagnostics(), "{source:?}");
    }
}

// CSS Style Attributes (REC 2013-11-07) §3 admits only declarations, and CSS 2.1
// §4.2 "Malformed declarations" reads an ident-started unit that is not a valid
// declaration "until the end of the declaration": its `{}` block is a matched
// pair inside that one malformed declaration, not a qualified rule ending at `}`.
const IDENT_STARTS: [&str; 5] = [
    "a {} color: red",
    "a { x: y } color: red",
    "a } color: red",
    "a b {c: d} color: red",
    "color {} color: red",
];

#[test]
fn an_ident_start_that_is_not_a_declaration_discards_through_the_next_semicolon() {
    for unit in IDENT_STARTS {
        let report = parse_style_attribute(&format!("{unit}; width: 2px"));
        assert_eq!(names(&report), ["width"], "{unit:?}: {report:?}");
        let source = format!("top: 1px; {unit}; width: 2px");
        let report = parse_style_attribute(&source);
        assert_eq!(names(&report), ["top", "width"], "{unit:?}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{unit:?}: one malformed declaration: {report:?}");
        };
        let start = source.find(unit).unwrap();
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        // As for any other identifier-started declaration this front drops
        // (`broken;`), the span covers the unit and the `;` that ends it.
        assert_eq!(
            (
                diagnostic.span().start().byte_offset().value(),
                diagnostic.span().end().byte_offset().value()
            ),
            (start, start + unit.len() + 1),
            "{unit:?}"
        );
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn an_ident_start_that_is_not_a_declaration_without_a_semicolon_discards_through_eof() {
    for unit in IDENT_STARTS {
        let report = parse_style_attribute(unit);
        assert!(report.syntax().is_empty(), "{unit:?}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{unit:?}: one malformed declaration: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(diagnostic.span().end().byte_offset().value(), unit.len());
    }
}

#[test]
fn ident_and_delimiter_starts_drop_the_same_following_declaration() {
    // `.a {}` cannot start a declaration; `a {}` starts one that fails. Both
    // are one malformed declaration, so `color` is dropped either way.
    for prefix in [".a {}", "a {}"] {
        let report = parse_style_attribute(&format!("{prefix} color: red; width: 2px"));
        assert_eq!(names(&report), ["width"], "{prefix:?}: {report:?}");
    }
}

#[test]
fn the_raw_declaration_list_front_drops_the_same_ident_started_unit() {
    for unit in IDENT_STARTS {
        let source = format!("{unit}; width: 2px");
        let attribute = parse_style_attribute(&source);
        let raw = parse_declaration_list_text(&source);
        assert_eq!(attribute.syntax(), raw.syntax(), "{source:?}");
        assert_eq!(raw.diagnostics().len(), 1, "{source:?}: {raw:?}");
        assert_eq!(
            attribute.diagnostics()[0].span().start(),
            raw.diagnostics()[0].span().start(),
            "{source:?}"
        );
    }
}
