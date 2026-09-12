//! Named pseudo admission from pinned Selectors 4, sections 3.6.1 and 3.9.
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#pseudo-element-syntax
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid
//! https://www.w3.org/TR/2025/WD-css-pseudo-4-20250627/
use surgeist_css::{
    CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssPseudoElement, CssRecoveryAction,
    CssSelector, parse_selector,
};

#[test]
fn named_pseudos_retain_defined_elements_and_reject_unknown_names() {
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    for (input, expected) in [
        (":after", CssPseudoElement::After),
        (":before", CssPseudoElement::Before),
        (":bEfOrE", CssPseudoElement::Before),
        (":first-letter", CssPseudoElement::FirstLetter),
        (":first-line", CssPseudoElement::FirstLine),
        ("::before", CssPseudoElement::Before),
    ] {
        let report = parse_selector(input, &context);
        assert!(report.is_clean(), "{input}: {report:?}");
        let Some(CssSelector::Compound(compound)) = report.syntax() else {
            panic!("{input}: expected retained pseudo-element compound: {report:?}");
        };
        assert_eq!(
            compound
                .pseudo_elements()
                .map(|sequence| sequence.pseudo_elements()),
            Some([expected].as_slice()),
            "{input}",
        );
        assert!(compound.pseudo_classes().is_empty(), "{input}");
    }
    for input in [":test", ":test-test", "::test", "::test-test"] {
        let report = parse_selector(input, &context);
        assert!(report.syntax().is_none(), "{input}: {report:?}");
        assert!(!report.is_clean(), "{input}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput),
            "{input}: {report:?}",
        );
    }
}

// Undefined function names are invalid independently of argument contents.
#[test]
fn unknown_functional_pseudos_reject_complete_raw_inputs() {
    let context = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").unwrap()),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )]);
    for input in [
        ":unknown(1 + 2)",
        ":unknown([{}()[{[()]}]])",
        ":unknown([{}()[{[)]}]])",
        ":unknown( 1 + 2 /* comment */ + 3 )",
        "::test(1 + 2)",
    ] {
        let report = parse_selector(input, &context);
        assert!(report.syntax().is_none(), "{input}: {report:?}");
        assert!(!report.is_clean(), "{input}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput),
            "{input}: {report:?}",
        );
    }
}
