//! CSS Nesting 1 style rules, nested group rules and their source order through
//! the public stylesheet parser.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn nesting_retains_descendant_and_parent_selectors_in_source_order() {
    let sheet = parse_sheet(".card { color: black; .title { color: white; } background-color: white; &:hover { opacity: 0.8; } }").expect("clean parse report");
    let [parent] = sheet.rules() else {
        panic!("expected one authored parent");
    };
    let parent = style_rule(parent);
    assert_eq!(
        parent.selectors().selectors()[0].selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        parent.declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
    let [title, CssRule::NestedDeclarations(after), hover] = parent.rules() else {
        panic!("expected child, declarations, child");
    };
    assert_eq!(
        style_rule(title).selectors().selectors()[0].selector(),
        &CssSelector::Class("title".to_owned())
    );
    assert_eq!(
        after.declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::BackgroundColor)
    );
    let CssSelector::Compound(hover) = style_rule(hover).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored hover selector");
    };
    assert_eq!(hover.nesting_selectors(), 1);
    assert!(hover.classes().is_empty());
    assert_eq!(hover.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_retains_selector_lists_and_relative_combinators() {
    let sheet = parse_sheet(
        ".button, .link { &.active[aria-current=true] { color: black; } > .icon { opacity: 1; } }",
    )
    .expect("clean parse report");
    let [parent] = sheet.rules() else {
        panic!("expected one authored parent");
    };
    let parent = style_rule(parent);
    assert_eq!(parent.selectors().selectors().len(), 2);
    assert_eq!(
        parent.selectors().selectors()[0].selector(),
        &CssSelector::Class("button".to_owned())
    );
    assert_eq!(
        parent.selectors().selectors()[1].selector(),
        &CssSelector::Class("link".to_owned())
    );
    let [active, icon] = parent.rules() else {
        panic!("expected two children without multiplication");
    };
    let CssSelector::Compound(active) = style_rule(active).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored compound");
    };
    assert_eq!(active.nesting_selectors(), 1);
    assert_eq!(active.classes(), &["active".to_owned()]);
    let [attribute] = active.attributes() else {
        panic!("expected attribute");
    };
    assert_eq!(attribute.name().as_str(), "aria-current");
    assert_eq!(
        attribute.matcher(),
        &CssAttributeMatcher::Equals("true".to_owned())
    );
    let [CssStyleSelector::Relative(icon)] = style_rule(icon).selectors().selectors() else {
        panic!("expected relative child");
    };
    assert_eq!(icon.combinator(), CssSelectorCombinator::Child);
    assert_eq!(icon.selector(), &CssSelector::Class("icon".to_owned()));
}

#[test]
fn nesting_retains_ampersand_combinator_chain_and_suffixes() {
    let sheet = parse_sheet(".tabs { & > .tab { color: black; } } .button { &.active[aria-current=true]:hover { opacity: 1; } }").expect("clean parse report");
    let [tabs, button] = sheet.rules() else {
        panic!("expected two authored parents");
    };
    let [tab] = style_rule(tabs).rules() else {
        panic!("expected nested tab");
    };
    let [CssStyleSelector::Selector(CssSelector::Complex(tab))] =
        style_rule(tab).selectors().selectors()
    else {
        panic!("expected explicit parent compound and child combinator");
    };
    assert_eq!(tab.first().nesting_selectors(), 1);
    assert!(tab.first().classes().is_empty());
    let [child] = tab.rest() else {
        panic!("expected one authored child combinator");
    };
    assert_eq!(child.combinator(), CssSelectorCombinator::Child);
    assert_eq!(child.selector().nesting_selectors(), 0);
    assert_eq!(child.selector().classes(), &["tab".to_owned()]);
    let [active] = style_rule(button).rules() else {
        panic!("expected active child");
    };
    let CssSelector::Compound(active) = style_rule(active).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored suffix");
    };
    assert_eq!(active.nesting_selectors(), 1);
    assert_eq!(active.classes(), &["active".to_owned()]);
    assert_eq!(active.attributes()[0].name().as_str(), "aria-current");
    assert_eq!(active.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_retains_style_rules_inside_conditional_groups() {
    let sheet = parse_sheet("@media screen { .card { .title { color: white; } } } @container sidebar (inline-size > 30rem) { .tabs { & > .tab { color: black; } } }").expect("clean parse report");
    let [media, container] = sheet.rules() else {
        panic!("expected media and container");
    };
    let [card] = media_rule(media).rules() else {
        panic!("expected authored card");
    };
    let [title] = style_rule(card).rules() else {
        panic!("expected nested title");
    };
    assert_eq!(
        style_rule(title).selectors().selectors()[0].selector(),
        &CssSelector::Class("title".to_owned())
    );
    let [tabs] = container_rule(container).rules() else {
        panic!("expected authored tabs");
    };
    let [tab] = style_rule(tabs).rules() else {
        panic!("expected nested tab");
    };
    let [CssStyleSelector::Selector(CssSelector::Complex(tab))] =
        style_rule(tab).selectors().selectors()
    else {
        panic!("expected explicit parent compound and child combinator");
    };
    assert_eq!(tab.first().nesting_selectors(), 1);
    assert!(tab.first().classes().is_empty());
    let [child] = tab.rest() else {
        panic!("expected one authored child combinator");
    };
    assert_eq!(child.combinator(), CssSelectorCombinator::Child);
    assert_eq!(child.selector().nesting_selectors(), 0);
    assert_eq!(child.selector().classes(), &["tab".to_owned()]);
}

#[test]
fn nesting_retains_media_and_container_inside_style_rules() {
    let sheet = parse_sheet(".card { color: black; @media (min-width: 600px) { background-color: white; > .title { color: black; } } @container sidebar (inline-size > 30rem) { &:hover { opacity: 0.9; } } }").expect("clean parse report");
    let [card] = sheet.rules() else {
        panic!("expected authored card");
    };
    let card = style_rule(card);
    assert_eq!(
        card.declarations()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
    let [media, container] = card.rules() else {
        panic!("expected nested conditionals");
    };
    let [CssRule::NestedDeclarations(background), title] = media_rule(media).rules() else {
        panic!("expected declarations then style");
    };
    assert_eq!(
        background.declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::BackgroundColor)
    );
    let [CssStyleSelector::Relative(title)] = style_rule(title).selectors().selectors() else {
        panic!("expected relative title");
    };
    assert_eq!(title.combinator(), CssSelectorCombinator::Child);
    let [hover] = container_rule(container).rules() else {
        panic!("expected hover child");
    };
    let CssSelector::Compound(hover) = style_rule(hover).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored hover");
    };
    assert_eq!(hover.nesting_selectors(), 1);
    assert_eq!(hover.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_preserves_terminal_pseudo_elements_on_anchored_selectors() {
    let sheet = parse_sheet(".card { &::before { color: red; } }").expect("clean parse report");
    let [card] = sheet.rules() else {
        panic!("expected authored parent");
    };
    let [before] = style_rule(card).rules() else {
        panic!("expected pseudo-element child");
    };
    let CssSelector::Compound(before) = style_rule(before).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored compound");
    };
    assert_eq!(before.nesting_selectors(), 1);
    assert!(before.classes().is_empty());
    assert_pseudo_elements(
        before.pseudo_elements().unwrap(),
        &[CssPseudoElement::Before],
    );
}

#[test]
fn nesting_validates_each_authored_selector_without_combining_pseudo_elements() {
    assert!(parse_sheet(".card::before { .icon { color: red; } }").is_clean());
    assert_rejection(
        ".card { &::before .icon { color: red; } }",
        Rejection::selector(Some("{")),
    );
}

#[test]
fn nesting_retains_media_before_later_declaration_run_in_source_order() {
    let sheet = parse_sheet(".card { color: black; @media (min-width: 600px) { opacity: 0.8; } background-color: white; }").expect("clean parse report");
    let [card] = sheet.rules() else {
        panic!("expected authored parent");
    };
    let card = style_rule(card);
    assert_eq!(
        card.declarations()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
    let [media, CssRule::NestedDeclarations(after)] = card.rules() else {
        panic!("expected media then declarations");
    };
    let [CssRule::NestedDeclarations(opacity)] = media_rule(media).rules() else {
        panic!("expected nested declarations");
    };
    assert_eq!(
        opacity.declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::Opacity)
    );
    assert_eq!(
        after.declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::BackgroundColor)
    );
}

#[test]
fn nesting_inside_media_and_container_stays_inside_group() {
    let sheet = parse_sheet("@media (prefers-color-scheme: dark) { .card { .title { color: white; } } } @container sidebar (inline-size > 30rem) { .card { &:hover { opacity: 0.9; } } }").expect("clean parse report");
    let [media, container] = sheet.rules() else {
        panic!("expected media and container");
    };
    let [card] = media_rule(media).rules() else {
        panic!("expected authored card");
    };
    let [title] = style_rule(card).rules() else {
        panic!("expected child title");
    };
    assert_eq!(
        style_rule(title).selectors().selectors()[0].selector(),
        &CssSelector::Class("title".to_owned())
    );
    let [card] = container_rule(container).rules() else {
        panic!("expected authored card");
    };
    let [hover] = style_rule(card).rules() else {
        panic!("expected child hover");
    };
    let CssSelector::Compound(hover) = style_rule(hover).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored hover");
    };
    assert_eq!(hover.nesting_selectors(), 1);
    assert_eq!(hover.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_rejects_unsupported_at_rules_inside_style_blocks() {
    for (css, rejection) in [
        (
            r#".card { @import "x.css"; }"#,
            Rejection::placement("import"),
        ),
        (
            r#".card { @font-face { font-family: Inter; src: url("inter.woff2"); } }"#,
            Rejection::placement("font-face"),
        ),
        (
            ".card { @keyframes fade { from { opacity: 0; } } }",
            Rejection::placement("keyframes"),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn nesting_rejects_unsupported_nested_selector_forms() {
    for css in [
        ".card { svg|a { color: black; } }",
        ".card { [svg|href] { color: black; } }",
        ".card { .col || { color: black; } }",
    ] {
        let error = parse_sheet(css).expect_err(css);
        assert!(
            matches!(error.kind(), ErrorKind::InvalidSelector { .. }),
            "{css} should reject as an invalid selector, got {error:?}"
        );
    }
}

#[test]
fn keyframes_and_nesting_reject_browser_recovery_forms() {
    let rejected = [
        (
            "@-webkit-keyframes fade { from { opacity: 0; } }",
            Rejection::unknown_at_rule("-webkit-keyframes"),
        ),
        (
            "@keyframes fade { from { @media screen { opacity: 0; } } }",
            Rejection::placement("media"),
        ),
        (
            "@keyframes fade { from { .nested { opacity: 0; } } }",
            Rejection::new(CssErrorCode::InvalidQualifiedRule, None, Some(".")),
        ),
        (
            ".card { svg|a { color: black; } }",
            Rejection::selector(Some("a")),
        ),
        (
            ".card { .col || { color: black; } }",
            Rejection::selector(Some("{")),
        ),
        (
            r#".card { @import url("theme.css"); }"#,
            Rejection::placement("import"),
        ),
        (
            r#".card { @font-face { font-family: Inter; src: url("inter.woff2"); } }"#,
            Rejection::placement("font-face"),
        ),
        (
            ".card { @keyframes fade { from { opacity: 0; } } }",
            Rejection::placement("keyframes"),
        ),
    ];

    for (css, rejection) in rejected {
        assert_rejection(css, rejection);
    }
    assert_drops_invalid_keyframe_block("@keyframes fade { 0 { opacity: 0; } }");
}

#[test]
fn keyframes_and_nesting_accept_practical_surface_matrix() {
    let accepted = [
        // CSS Nesting permits repeated anchors and anchors after other compounds.
        ".card { .theme & { color: black; } }",
        ".card { && { color: black; } }",
        ".card { & & { color: black; } }",
        r#"@keyframes fade { from { opacity: 0; } to { opacity: 1; } }"#,
        r#"@keyframes "fade in" { 0%, 100% { opacity: 1; } }"#,
        r#"@keyframes duplicate { from, 0% { } from { opacity: 1; } }"#,
        ".card { color: black; .title { color: white; } }",
        ".card { &:hover { opacity: 0.9; } }",
        ".card { > .title[aria-current=true] { color: white; } }",
        ".card { @media (min-width: 600px) { &:hover { opacity: 0.9; } } }",
        "@media screen { .card { .title { color: black; } } }",
        "@container sidebar (inline-size > 30rem) { .card { &:hover { opacity: 1; } } }",
        ".card { &:not(.field .icon) { color: black; } }",
    ];

    for css in accepted {
        assert!(parse_sheet(css).is_clean(), "{css} should parse");
    }
}
