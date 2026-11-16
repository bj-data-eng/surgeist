//! A `@supports` declaration test whose value holds a top-level `{}` block
//! beside another non-whitespace value has no typed view, in every front that
//! reads it. Syntax 3 section 5.5.6 (consume a declaration), step 8 returns no
//! declaration for such a value of a non-custom name; Conditional 3 keeps an
//! unsupported declaration as an authored test with no known typed view, as for
//! `(width: {} 1px)`, which no grammar of `width` admits. A `{}` block as the
//! entire value, a block nested in another component and a custom property name
//! keep their typed or authored result.

use surgeist_css::CssParser;
use surgeist_css::query::supports::{CssSupportsDeclaration, CssSupportsLeaf};
use surgeist_css::query::{CssCondition, CssConditionKind, CssWhenLeaf};
use surgeist_css::rule::CssRule;

/// Values with a top-level `{}` block beside another value, with a
/// substitution function or without one.
const BESIDE: &[&str] = &[
    "{} var(--x)",
    "var(--x) {}",
    "{} env(x)",
    "{} {} var(--x)",
    "{a b} var(--x)",
    "{} 1px",
    "1px {}",
];

/// Values that are no top-level `{}` block beside another value.
const NOT_BESIDE: &[&str] = &["{var(--x)}", "[] var(--x)", "() var(--x)", "var(--x, {})"];

/// Whether the declaration test of each front has a known typed view, for
/// `(name: value)`: the sheet `@supports` rule, `@import supports()`, the
/// `@when supports()` rule and the standards-mode condition text front.
fn known_by_front(name: &str, value: &str) -> [(&'static str, bool); 4] {
    let parser = CssParser::default();
    let declaration = format!("{name}: {value}");

    let sheet = parser.parse_sheet(format!("@supports ({declaration}) {{ a{{}} }}"));
    assert!(sheet.is_clean(), "{declaration}: {:?}", sheet.diagnostics());
    let [CssRule::Supports(rule)] = sheet.syntax().rules() else {
        panic!("{declaration}: one supports rule")
    };
    let CssConditionKind::Leaf(CssSupportsLeaf::Declaration(supports)) = rule.condition().kind()
    else {
        panic!("{declaration}: a declaration test")
    };
    let supports = supports.known().is_some();

    let import = parser.parse_sheet(format!("@import url(x.css) supports({declaration});"));
    assert!(
        import.is_clean(),
        "{declaration}: {:?}",
        import.diagnostics()
    );
    let [CssRule::Import(rule)] = import.syntax().rules() else {
        panic!("{declaration}: one import rule")
    };
    let CssConditionKind::Leaf(CssSupportsLeaf::Declaration(imported)) =
        rule.supports().unwrap().condition().kind()
    else {
        panic!("{declaration}: a declaration test")
    };
    let imported = imported.known().is_some();

    let when = parser.parse_sheet(format!("@when supports({declaration}) {{ a{{}} }}"));
    assert!(when.is_clean(), "{declaration}: {:?}", when.diagnostics());
    let [CssRule::When(rule)] = when.syntax().rules() else {
        panic!("{declaration}: one when rule")
    };
    let CssConditionKind::Leaf(CssWhenLeaf::SupportsDeclaration(when)) = rule.condition().kind()
    else {
        panic!("{declaration}: a supports declaration test")
    };
    let when = when.known().is_some();

    let text =
        CssCondition::<CssSupportsLeaf>::parse_condition_text(&parser, format!("({declaration})"))
            .unwrap();
    let CssConditionKind::Leaf(CssSupportsLeaf::Declaration(text)) = text.kind() else {
        panic!("{declaration}: a declaration test")
    };
    [
        ("@supports", supports),
        ("@import supports()", imported),
        ("@when supports()", when),
        ("condition text", text.known().is_some()),
    ]
}

#[test]
fn a_block_beside_another_value_has_no_known_view_in_every_supports_front() {
    for name in ["width", "margin-top"] {
        for value in BESIDE {
            for (front, known) in known_by_front(name, value) {
                assert!(!known, "{front}: ({name}: {value})");
            }
        }
    }
}

#[test]
fn the_property_value_overload_has_no_known_view_for_a_block_beside_another_value() {
    let parser = CssParser::default();
    for value in BESIDE {
        let declaration = CssSupportsDeclaration::parse(&parser, "width", value).unwrap();
        assert!(declaration.known().is_none(), "width, {value}");
    }
}

#[test]
fn a_whole_value_block_nested_blocks_and_custom_names_keep_their_view() {
    for value in NOT_BESIDE.iter().chain(&["1px"]) {
        for (front, known) in known_by_front("width", value) {
            assert!(known, "{front}: (width: {value})");
        }
    }
    // A custom property name keeps any value as a clean authored test in every
    // front; the helper asserts the clean report and the retained test.
    for value in BESIDE.iter().chain(NOT_BESIDE) {
        known_by_front("--c", value);
    }
}

#[test]
fn the_fronts_agree_on_every_value() {
    for value in BESIDE.iter().chain(NOT_BESIDE) {
        let views = known_by_front("width", value);
        assert!(
            views.iter().all(|(_, known)| *known == views[3].1),
            "(width: {value}): {views:?}"
        );
    }
}

#[test]
fn container_style_features_keep_every_value_as_authored_syntax() {
    // A `style()` feature identifies its property but validates no value
    // (Containment 3 leaves style evaluation external): a value no grammar of
    // the property admits, `{} 1px`, is retained exactly as `{} var(--x)` is.
    let parser = CssParser::default();
    for name in ["width", "--c"] {
        for value in BESIDE.iter().chain(NOT_BESIDE) {
            let source = format!("@container style({name}: {value}) {{ a{{}} }}");
            let sheet = parser.parse_sheet(&source);
            assert!(sheet.is_clean(), "{source}: {:?}", sheet.diagnostics());
            assert!(
                matches!(sheet.syntax().rules(), [CssRule::Container(_)]),
                "{source}"
            );
        }
    }
}
