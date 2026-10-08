#![forbid(unsafe_code)]

//! Public characterization. Fullscreen snapshot
//! 7c38d773117aa1e6bfa13754afe77483f40c908f §5.1 names the nonfunctional
//! :fullscreen pseudo-class; tree/top-layer matching is downstream.

use surgeist_css::{
    CssNamespaceContext, CssPseudoClass, CssRecoveryAction, CssRule, CssSelector, CssSelectorList,
    parse_selector, parse_sheet,
};

#[test]
fn fullscreen_spelling_case_and_escapes_retain_one_symbolic_identity() {
    for source in [
        ":fullscreen",
        ":FULLSCREEN",
        r":full\73 creen",
        " /*é*/:fullscreen ",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let selector = report.syntax().as_ref().expect("one authored selector");
        assert_eq!(
            selector,
            &CssSelector::PseudoClass(CssPseudoClass::Fullscreen)
        );
        assert_eq!(selector.to_specified_css().unwrap(), ":fullscreen");
        assert!(!selector.has_pseudo_elements());
        assert!(report.into_validation_result().is_ok());
    }
}

#[test]
fn checked_fullscreen_selector_preserves_its_symbolic_public_value() {
    let selector = CssSelector::PseudoClass(CssPseudoClass::Fullscreen);
    let list = CssSelectorList::try_new(vec![selector.clone()]).unwrap();
    assert_eq!(list.selectors(), &[selector]);
    assert_eq!(list.to_specified_css().unwrap(), ":fullscreen");
    let reparsed = parse_selector(
        &list.to_specified_css().unwrap(),
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
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Style(_)] = report.syntax().rules() else {
            panic!("one authored style rule: {source}");
        };
        assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
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
        let direct = parse_selector(source, &CssNamespaceContext::default());
        assert!(direct.syntax().is_none(), "{source}: {direct:?}");
        assert!(!direct.is_clean());
        assert!(direct.into_validation_result().is_err());
        let sheet = parse_sheet(&format!("{source}{{}}.after{{color:red}}"));
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
    let extra = parse_selector(":fullscreen,.viewer", &CssNamespaceContext::default());
    assert!(extra.syntax().is_none());
    assert!(extra.into_validation_result().is_err());
    let forgiving = parse_selector(
        ":is(:fullscreen,:fullscreen())",
        &CssNamespaceContext::default(),
    );
    assert!(!forgiving.is_clean());
    assert_eq!(
        forgiving
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        ":is(:fullscreen)"
    );
    assert!(forgiving.into_validation_result().is_err());
}
