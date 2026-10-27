//! Supports wrapper expectations derive from retained WebKit revision
//! 73aa6c89e2cb77c46184a81aec944e4ab99d114d: CSSSupportsRule::cssText and
//! CSSGroupingRule::cssTextForRules/appendCSSTextForItemsInternal. Conditional3
//! 2024-08-15 requires preservation of specified logical and future-token structure.

use crate::css::support::{graph_cssom, graph_specified};
use surgeist_css::CssParser;
use surgeist_css::declaration::CssSpecifiedDeclarationBlock as Block;
use surgeist_css::rule::CssRule;
use surgeist_css::serialization::CssGroupPreludeRef as Prelude;
use surgeist_css::serialization::CssRuleCssomFormat;
use surgeist_css::serialization::CssRuleGraphInput as View;

#[test]
fn empty_parsed_supports_has_exactly_one_closing_line_feed() {
    let report = CssParser::default().parse_sheet("@supports (display: grid) {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("Supports rule");
    };
    assert_eq!(
        rule.condition()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        " (display: grid) "
    );
    assert!(rule.rules().is_empty());
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        "@supports (display: grid) { }"
    );
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        "@supports (display: grid) {\n}"
    );
    assert_eq!(report, before);
}

#[test]
fn multiline_children_receive_a_prefix_only_on_their_first_line() {
    let report = CssParser::default().parse_sheet(
        "@supports (display:grid) { @media screen { .child { color:red } } .last {} }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("Supports rule");
    };
    assert_eq!(
        rule.condition()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        " (display:grid) "
    );
    assert_eq!(
        rule.rules()[0].serialize_cssom(Default::default()).unwrap(),
        "@media screen {\n  .child { color: red; }\n}"
    );
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        "@supports (display:grid) { @media screen { .child { color: red; } } .last { } }"
    );
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        "@supports (display:grid) {\n  @media screen {\n  .child { color: red; }\n}\n  .last { }\n}"
    );
}

#[test]
fn supports_inside_a_style_preserves_actual_nested_selector_ancestry() {
    let report =
        CssParser::default().parse_sheet(".host { @supports (display:grid) { .child {} } }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        ".host { @supports (display:grid) { .child { } } }"
    );
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        ".host {\n  @supports (display:grid) {\n  & .child { }\n}\n}"
    );
    assert_eq!(report, before);
}

#[test]
fn borrowed_scoped_supports_formats_without_adopting_a_scope_wrapper() {
    let report = CssParser::default()
        .parse_sheet("@scope (.host) { @supports (display:grid) { & .child {} } }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope");
    };
    let [scoped @ CssRule::Supports(_)] = scope.rules().rules() else {
        panic!("scoped Supports");
    };
    let view = View::from_rule(scoped);
    assert_eq!(
        graph_specified(view, Default::default()).unwrap(),
        "@supports (display:grid) { & .child { } }"
    );
    assert_eq!(
        graph_cssom(view, Default::default()).unwrap(),
        "@supports (display:grid) {\n  & .child { }\n}"
    );
}

#[test]
fn edited_supports_preserves_selected_survivors_and_filters_empty_child_text() {
    let report = CssParser::default()
        .parse_sheet("@supports (display:grid) {} .child { margin:1px 2px 3px 4px }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(supports), CssRule::Style(style)] = report.syntax().rules() else {
        panic!("Supports and style");
    };
    let original = Block::try_from_declarations(style.declarations(), Default::default()).unwrap();
    let selected = [original.entries()[3].clone(), original.entries()[1].clone()];
    let current = Block::try_from_entries(&selected, Default::default()).unwrap();
    for (retained, authored) in current.entries().iter().zip(&selected) {
        assert!(retained.source().same_occurrence(authored.source()));
    }
    assert_eq!(
        current.serialize_cssom(Default::default()).unwrap(),
        "margin-left: 4px; margin-right: 2px;"
    );
    let child = View::style(style.selectors(), &current, &[]);
    let empty = Block::try_from_entries(&[], Default::default()).unwrap();
    let empty_child = View::nested_declarations(&empty);
    assert_eq!(graph_cssom(empty_child, Default::default()).unwrap(), "");
    let children = [empty_child, child, empty_child, child, empty_child];
    let group = View::group(Prelude::Supports(supports.condition()), &children);
    assert_eq!(
        graph_specified(group, Default::default()).unwrap(),
        "@supports (display:grid) { .child { margin-left: 4px; margin-right: 2px; } .child { margin-left: 4px; margin-right: 2px; } }"
    );
    assert_eq!(
        graph_cssom(group, Default::default()).unwrap(),
        "@supports (display:grid) {\n  .child { margin-left: 4px; margin-right: 2px; }\n  .child { margin-left: 4px; margin-right: 2px; }\n}"
    );
    assert_eq!(graph_cssom(children[0], Default::default()).unwrap(), "");
    assert_eq!(
        original.serialize_cssom(Default::default()).unwrap(),
        "margin: 1px 2px 3px 4px;"
    );
}

fn assert_empty_edited_frame(retain_empty_children: bool) {
    let report = CssParser::default().parse_sheet("@supports (display:grid) {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(supports)] = report.syntax().rules() else {
        panic!("Supports");
    };
    let empty = Block::try_from_entries(&[], Default::default()).unwrap();
    let empty_child = View::nested_declarations(&empty);
    let retained = [empty_child, empty_child, empty_child];
    let children = if retain_empty_children {
        &retained[..]
    } else {
        &[][..]
    };
    let group = View::group(Prelude::Supports(supports.condition()), children);
    assert_eq!(
        graph_specified(group, Default::default()).unwrap(),
        "@supports (display:grid) { }"
    );
    assert!(
        children
            .iter()
            .all(|child| { graph_cssom(*child, Default::default()).unwrap().is_empty() })
    );
    assert_eq!(
        graph_cssom(group, Default::default()).unwrap(),
        "@supports (display:grid) {\n}"
    );
}

#[test]
fn zero_edited_children_have_the_one_line_feed_frame() {
    assert_empty_edited_frame(false);
}

#[test]
fn all_empty_edited_children_have_the_one_line_feed_frame() {
    assert_empty_edited_frame(true);
}

#[test]
fn specified_condition_retains_logical_grouping_and_future_tokens() {
    let report = CssParser::default()
        .parse_sheet("@supports ((display:grid) or (display:flex)) and (future-token(foo)) {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let [CssRule::Supports(supports)] = report.syntax().rules() else {
        panic!("Supports");
    };
    // The value serializer preserves the exact authored prelude trivia; the
    // rule serializer may safely trim its boundary whitespace.
    let condition = " ((display:grid) or (display:flex)) and (future-token(foo)) ";
    assert_eq!(
        supports
            .condition()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        condition
    );
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        "@supports ((display:grid) or (display:flex)) and (future-token(foo)) { }"
    );
    assert_eq!(report, before);
}

#[test]
fn media_empty_frame_and_unselected_container_are_independent_controls() {
    let report = CssParser::default().parse_sheet("@media screen {} @container (width>1px) {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [media @ CssRule::Media(_), container @ CssRule::Container(_)] = report.syntax().rules()
    else {
        panic!("media and container");
    };
    assert_eq!(
        media.serialize_cssom(Default::default()).unwrap(),
        "@media screen {\n\n}"
    );
    assert!(
        container
            .serialize_cssom(Default::default())
            .unwrap_err()
            .unavailable_format()
            == Some(CssRuleCssomFormat::Container)
    );
    assert_eq!(
        container.serialize_specified(Default::default()).unwrap(),
        "@container (width>1px) { }"
    );
}
