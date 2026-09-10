//! Scope's authored selector context must survive the parser's structural chunk boundary.
//! The same supported scoped selector remains valid regardless of ancestor depth.

use surgeist_css::{
    CssRule, CssScopedRule, CssScopedStyleRule, CssScopedStyleSelector, CssSelector,
    CssSelectorCombinator, parse_sheet,
};

fn style_at_scope_depth(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
    scope_depth: usize,
) -> &CssScopedStyleRule {
    let [CssRule::Scope(first)] = report.syntax().rules() else {
        panic!("expected retained scope root");
    };
    let mut scope = first;
    for _ in 1..scope_depth {
        let [CssScopedRule::Scope(next)] = scope.rules().rules() else {
            panic!("expected retained scope ancestor");
        };
        scope = next;
    }
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("expected retained scoped style rule");
    };
    style
}

#[test]
fn scoped_anchor_remains_valid_when_its_rule_starts_a_structural_chunk() {
    for depth in [1, 62, 63, 64] {
        let source = format!("{}&.anchor {{ color: red; }}{}", "@scope{".repeat(depth), "}".repeat(depth));
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "depth {depth}: {:?}", report.diagnostics());
        let style = style_at_scope_depth(&report, depth);
        let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] = style.selectors().selectors() else {
            panic!("expected authored scoped anchor");
        };
        assert!(selector.has_scope_anchor());
        assert_eq!(selector.classes(), ["anchor"]);
        assert_eq!(style.position().byte_offset().value(), source.find('&').unwrap());
        assert_eq!(style.declarations().len(), 1);
    }
}

#[test]
fn scoped_relative_selector_remains_valid_when_its_rule_starts_a_structural_chunk() {
    for depth in [1, 62, 63, 64] {
        let source = format!("{}> .child {{ color: red; }}{}", "@scope{".repeat(depth), "}".repeat(depth));
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "depth {depth}: {:?}", report.diagnostics());
        let style = style_at_scope_depth(&report, depth);
        let [CssScopedStyleSelector::Relative(relative)] = style.selectors().selectors() else {
            panic!("expected authored relative scoped selector");
        };
        assert_eq!(relative.combinator(), CssSelectorCombinator::Child);
        assert_eq!(relative.selector(), &CssSelector::Class("child".to_owned()));
        assert_eq!(style.position().byte_offset().value(), source.find('>').unwrap());
        assert_eq!(style.declarations().len(), 1);
    }
}
