//! A style attribute is read as CSS Syntax 3 (2026 CRD) §5.5.5 "consume a
//! block's contents" (#1353 decision 1, #1396), the same algorithm as every
//! other declaration list.
//!
//! - §5.5.5: a root `}` ends the contents; a root `;` is discarded; an
//!   at-keyword is consumed as an at-rule; anything else is tried as a
//!   declaration and, if that returns nothing, reparsed as a qualified rule
//!   with `;` as the stop token and `nested` set.
//! - §5.5.3 "consume a qualified rule": the rule ends at its `{}` block, so
//!   input after that block starts a fresh item; EOF or the stop token returns
//!   nothing, and so does `}` when `nested` is true (the `}` is not consumed).
//! - §5.5.6 "consume a declaration": a first token that is not an
//!   `<ident-token>` returns nothing, and the component values of a nested
//!   declaration end at `}`.
//!   <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents>
//!
//! This supersedes the CSS Style Attributes 2013 / CSS 2.1 §4.2 reading, in
//! which a `}` is an ordinary invalid token and a malformed declaration is read
//! through its `;`.
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

/// A dropped unit that reached no `{}` block: one `DropDeclaration` whose span is
/// the unit and its `;`.
fn assert_dropped_unit(source: &str, unit: &str, expected: &[&str]) {
    assert_dropped(source, unit, expected, CssRecoveryAction::DropDeclaration);
}

/// A dropped qualified rule that ended at its `{}` block.
fn assert_dropped_rule(source: &str, unit: &str, expected: &[&str]) {
    assert_dropped(source, unit, expected, CssRecoveryAction::DropQualifiedRule);
}

fn assert_dropped(source: &str, unit: &str, expected: &[&str], action: CssRecoveryAction) {
    let report = CssParser::default().parse_block_contents(source);
    assert_eq!(names(&report), expected, "{source:?}: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("{source:?}: one dropped rule: {report:?}");
    };
    let start = source.find(unit).unwrap();
    assert_eq!(diagnostic.action(), action);
    // A dropped declaration's span includes the `;` that terminates it, as in
    // every declaration list; a rule that ended at its block has no such stop.
    let stop = usize::from(
        action == CssRecoveryAction::DropDeclaration
            && source[start + unit.len()..].starts_with(';'),
    );
    assert_eq!(
        (
            diagnostic.span().start().byte_offset().value(),
            diagnostic.span().end().byte_offset().value()
        ),
        (start, start + unit.len() + stop),
        "{source:?}"
    );
    assert_eq!(
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

/// Starts that are not an identifier and not a root closer.
const STARTS: [&str; 11] = [
    ")", "]", "1px", "-2", "50%", "\"text\"", "'text'", "f(x)", "(a)", "[a]", "#hash",
];

#[test]
fn a_non_ident_start_drops_a_qualified_rule_through_the_stop_semicolon() {
    for start in STARTS {
        let unit = format!("{start} color: red");
        assert_dropped_unit(&format!("{unit}; width: 2px"), &unit, &["width"]);
        assert_dropped_unit(&format!("{unit}; width: 2px;"), &unit, &["width"]);
        assert_dropped_unit(
            &format!("top: 1px; {unit}; width: 2px"),
            &unit,
            &["top", "width"],
        );
    }
}

#[test]
fn a_non_ident_start_without_a_semicolon_drops_through_eof() {
    for start in STARTS {
        let unit = format!("{start} color: red");
        assert_dropped_unit(&unit, &unit, &[]);
        assert_dropped_unit(&format!("top: 1px; {unit}"), &unit, &["top"]);
    }
}

#[test]
fn blocks_and_functions_before_the_stop_semicolon_are_consumed_whole() {
    for unit in [
        ") f(a; width: 1px) color: red",
        "] [a; width: 1px] color: red",
        "1px (a; width: 1px) color: red",
        "\"a;\" /* ; */ color: red",
    ] {
        let source = format!("{unit}; width: 2px");
        assert_dropped_unit(&source, unit, &["width"]);
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(
            report.syntax()[0]
                .serialize_specified(Default::default())
                .unwrap(),
            "width: 2px;"
        );
    }
}

#[test]
fn a_root_closer_ends_the_contents_and_discards_the_rest() {
    for (source, expected) in [
        ("} color: red; width: 2px", &[][..]),
        ("top: 1px; } width: 2px", &["top"][..]),
        ("color: red } width: 1px", &["color"][..]),
        ("top: 1px } } width: 1px", &["top"][..]),
        ("}", &[][..]),
    ] {
        let report = CssParser::default().parse_block_contents(source);
        assert_eq!(names(&report), expected, "{source:?}: {report:?}");
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert_eq!(
            &CssParser::default()
                .parse_block_contents(source)
                .into_validation_result()
                .unwrap(),
            report.syntax(),
            "{source:?}"
        );
    }
}

// Syntax 3 (2026 CRD) §5.5.3: a qualified rule ends at its block, so what
// follows the block is a fresh item and may be a declaration. `a {}` is a rule
// (an identifier that is not followed by a colon), as are the others.
const RULES: [&str; 6] = [
    "a {}",
    "a { x: y }",
    "a b {c: d}",
    "color {}",
    ".a {}",
    ") {a: b; width: 1px}",
];

#[test]
fn a_qualified_rule_ends_at_its_block_and_the_following_declaration_is_kept() {
    for rule in RULES {
        assert_dropped_rule(
            &format!("{rule} color: red; width: 2px"),
            rule,
            &["color", "width"],
        );
        assert_dropped_rule(
            &format!("top: 1px; {rule} color: red"),
            rule,
            &["top", "color"],
        );
        assert_dropped_rule(&format!("{rule} color: red"), rule, &["color"]);
        assert_dropped_rule(&format!("{rule}; color: red"), rule, &["color"]);
    }
}

#[test]
fn an_ident_start_without_a_colon_drops_a_rule_through_the_stop_semicolon() {
    // `broken color: red` is not a declaration (no colon after the name), so
    // it is reparsed as a qualified rule that the semicolon stops.
    assert_dropped_unit(
        "broken color: red; width: 2px",
        "broken color: red",
        &["width"],
    );
    assert_dropped_unit("broken color: red", "broken color: red", &[]);
}

#[test]
fn an_ident_start_before_a_root_closer_drops_a_rule_and_ends_the_contents() {
    // `a` is dropped (§5.5.3: `}` returns nothing when nested) and the `}`
    // then ends the contents, so `color` is never read.
    let report = CssParser::default().parse_block_contents("a } color: red");
    assert!(report.syntax().is_empty(), "{report:?}");
    assert!(!report.is_clean(), "{report:?}");
    let report = CssParser::default().parse_block_contents("top: 1px; a } color: red");
    assert_eq!(names(&report), ["top"], "{report:?}");
    assert!(!report.is_clean(), "{report:?}");
}

#[test]
fn the_style_attribute_text_front_agrees_with_the_utf16_front() {
    use std::sync::Arc;
    let units = |source: &str| Arc::new(source.encode_utf16().collect::<Vec<u16>>());
    for source in [
        "a {} color: red; width: 2px",
        "color: red } width: 1px",
        "} color: red; width: 2px",
        "1px color: red; width: 2px",
        "a } color: red",
        "top: 1px; .a {} color: red",
    ] {
        let text = CssParser::default().parse_block_contents(source);
        let utf16 = CssParser::default()
            .parse_block_contents_utf16(units(source), CssUtf16InputLimits::default(), || {
                Ok::<_, ()>(())
            })
            .unwrap();
        assert_eq!(text.syntax(), utf16.report().syntax(), "{source:?}");
        assert_eq!(
            text.diagnostics(),
            utf16.report().diagnostics(),
            "{source:?}"
        );
    }
}
