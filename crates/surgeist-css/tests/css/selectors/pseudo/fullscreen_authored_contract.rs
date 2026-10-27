//! Public characterization. Fullscreen snapshot
//! 7c38d773117aa1e6bfa13754afe77483f40c908f §5.1 names the nonfunctional
//! :fullscreen pseudo-class; tree/top-layer matching is downstream.

use crate::css::support::color_and_selector_fixtures::{
    assert_selector_input_rejected, assert_selector_item_dropped,
};
use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::rule::CssRule;
use surgeist_css::selector::{CssNamespaceContext, CssPseudoClass, CssSelector, CssSelectorList};

#[test]
fn fullscreen_spelling_case_and_escapes_retain_one_symbolic_identity() {
    for source in [
        ":fullscreen",
        ":FULLSCREEN",
        r":full\73 creen",
        " /*é*/:fullscreen ",
    ] {
        let report = CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let selector = report.syntax().as_ref().expect("one authored selector");
        assert_eq!(
            selector,
            &CssSelector::PseudoClass(CssPseudoClass::Fullscreen)
        );
        assert_eq!(
            selector.serialize_specified(Default::default()).unwrap(),
            ":fullscreen"
        );
        assert!(!selector.has_pseudo_elements());
        assert!(report.into_validation_result().is_ok());
    }
}

#[test]
fn checked_fullscreen_selector_preserves_its_symbolic_public_value() {
    let selector = CssSelector::PseudoClass(CssPseudoClass::Fullscreen);
    let list = CssSelectorList::try_new(vec![selector.clone()], Default::default()).unwrap();
    assert_eq!(list.selectors(), &[selector]);
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        ":fullscreen"
    );
    let reparsed = CssSelector::parse(
        &CssParser::default(),
        list.serialize_specified(Default::default()).unwrap(),
        &CssNamespaceContext::default(),
    );
    assert!(reparsed.is_clean());
    assert_eq!(reparsed.syntax().as_ref(), Some(&list.selectors()[0]));
}

#[test]
fn fullscreen_compounds_and_lists_have_canonical_authored_output() {
    for (source, expected) in [
        (
            "video:FULLSCREEN{color:red}",
            "video:fullscreen { color: red; }",
        ),
        (
            ".viewer:fullscreen > .controls{}",
            ".viewer:fullscreen > .controls { }",
        ),
        (":fullscreen, .viewer{}", ":fullscreen, .viewer { }"),
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Style(_)] = report.syntax().rules() else {
            panic!("one authored style rule: {source}");
        };
        assert_eq!(
            report
                .syntax()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn invalid_fullscreen_forms_drop_their_rule_and_preserve_the_following_sibling() {
    for source in [
        ":fullscreen()",
        ":fullscreen(x)",
        "::fullscreen",
        ": fullscreen",
        ":full-screen",
    ] {
        assert_selector_input_rejected(
            source,
            CssSelector::parse(
                &CssParser::default(),
                source,
                &CssNamespaceContext::default(),
            ),
        );
        let sheet = CssParser::default().parse_sheet(format!("{source}{{}}.after{{color:red}}"));
        let [CssRule::Style(after)] = sheet.syntax().rules() else {
            panic!("retain only the valid sibling: {source}: {sheet:?}");
        };
        assert_eq!(
            after.selectors().selectors()[0].selector(),
            &CssSelector::Class("after".into())
        );
        assert_eq!(after.declarations().len(), 1);
        assert!(
            sheet
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
        );
    }
}

#[test]
fn fullscreen_exact_front_rejects_a_second_selector_and_forgiving_recovery_stays_unclean() {
    let extra = ":fullscreen,.viewer";
    assert_selector_input_rejected(
        extra,
        CssSelector::parse(
            &CssParser::default(),
            extra,
            &CssNamespaceContext::default(),
        ),
    );
    let source = ":is(:fullscreen,:fullscreen())";
    let forgiving = CssSelector::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    assert_eq!(
        forgiving
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        ":is(:fullscreen)"
    );
    assert_selector_item_dropped(source, forgiving, ":fullscreen()");
}
