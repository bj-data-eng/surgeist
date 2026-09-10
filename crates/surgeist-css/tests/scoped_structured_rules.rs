//! CSS Nesting 1 §3 permits nested style rules in a style rule's body, including
//! when the containing style rule is scoped. Scope changes selector context,
//! not the style body's acceptance of nested rules and declaration runs.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting

use surgeist_css::parse_sheet;

#[test]
fn scoped_style_body_accepts_nested_rules_and_surrounding_declarations() {
    let source = "@scope { .parent { color: red; .child { width: 1px; } opacity: 1; } }";
    let report = parse_sheet(source);
    assert!(
        report.is_clean(),
        "valid scoped style nesting must not discard syntax during recovery: {:?}",
        report.diagnostics(),
    );
}

fn scoped_parent(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
    scope_depth: usize,
) -> &surgeist_css::CssScopedStyleRule {
    use surgeist_css::{CssRule, CssScopedRule};

    let [CssRule::Scope(mut_scope)] = report.syntax().rules() else {
        panic!("expected one outer scope");
    };
    let mut scope = mut_scope;
    for _ in 1..scope_depth {
        let [CssScopedRule::Scope(child)] = scope.rules().rules() else {
            panic!("expected retained scope ancestor");
        };
        scope = child;
    }
    let [CssScopedRule::Style(parent)] = scope.rules().rules() else {
        panic!("expected one authored scoped style parent");
    };
    parent
}

#[test]
fn scoped_style_children_and_trailing_declarations_survive_structural_splits() {
    use surgeist_css::{CssKnownProperty, CssRule};

    for scope_depth in [1, 62, 63, 64, 126, 127, 128, 254] {
        let source = format!(
            "{}.parent {{ color: red; .child {{ width: 1px; }} opacity: 1; }}{}",
            "@scope{".repeat(scope_depth),
            "}".repeat(scope_depth),
        );
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {scope_depth}: {:?}",
            report.diagnostics()
        );
        let parent = scoped_parent(&report, scope_depth);
        assert_eq!(parent.declarations().len(), 1);
        assert_eq!(
            parent.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        let [CssRule::Style(child), CssRule::NestedDeclarations(after)] = parent.rules() else {
            panic!("depth {scope_depth}: expected retained child then trailing declarations");
        };
        assert_eq!(child.declarations().len(), 1);
        assert_eq!(
            child.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Width
        );
        assert_eq!(after.declarations().len(), 1);
        assert_eq!(
            after.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
        assert_eq!(
            parent.position().byte_offset().value(),
            source.find(".parent").unwrap()
        );
        assert_eq!(
            child.position().byte_offset().value(),
            source.find(".child").unwrap()
        );
        assert_eq!(
            after.position().byte_offset().value(),
            source.find("opacity").unwrap()
        );
    }
}

#[test]
fn nested_conditionals_keep_scoped_parent_context_and_order_across_splits() {
    use surgeist_css::{CssKnownProperty, CssRule, CssSelectorCombinator, CssStyleSelector};

    for scope_depth in [1, 61, 62, 63, 64] {
        let source = format!(
            "{}.parent {{ color: red; @media screen {{ width: 1px; > .child {{ height: 2px; }} opacity: 1; }} color: blue; }}{}",
            "@scope{".repeat(scope_depth),
            "}".repeat(scope_depth),
        );
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {scope_depth}: {:?}",
            report.diagnostics()
        );
        let parent = scoped_parent(&report, scope_depth);
        let [
            CssRule::Media(media),
            CssRule::NestedDeclarations(after_media),
        ] = parent.rules()
        else {
            panic!("depth {scope_depth}: expected nested media and trailing declarations");
        };
        let [
            CssRule::NestedDeclarations(before),
            CssRule::Style(child),
            CssRule::NestedDeclarations(after),
        ] = media.rules()
        else {
            panic!("depth {scope_depth}: expected nested declarations, child, declarations");
        };
        assert_eq!(
            before.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Width
        );
        assert_eq!(
            child.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Height
        );
        assert_eq!(
            after.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
        assert_eq!(
            after_media.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        let [CssStyleSelector::Relative(relative)] = child.selectors().selectors() else {
            panic!("expected symbolic child combinator");
        };
        assert_eq!(relative.combinator(), CssSelectorCombinator::Child);
        assert_eq!(
            before.position().byte_offset().value(),
            source.find("width").unwrap()
        );
        assert_eq!(
            after.position().byte_offset().value(),
            source.find("opacity").unwrap()
        );
        assert_eq!(
            after_media.position().byte_offset().value(),
            source.find("color: blue").unwrap()
        );
    }
}
