#![forbid(unsafe_code)]
//! Existing-callable admission oracles from Selectors4 WD20260122 §3.10 and
//! normative Appendix B. Unknown nonfunctional -webkit- pseudo-elements are
//! valid authored selectors; their match-nothing meaning is downstream.
use surgeist_css::{
    CssNamespaceContext, CssRecoveryAction, CssRule, CssSelector, parse_selector, parse_sheet,
};

fn admitted(source: &str) {
    let selector = parse_selector(source, &CssNamespaceContext::default());
    let sheet_source = format!("{source}{{color:red}}");
    let sheet = parse_sheet(&sheet_source);
    // Both real fronts execute before the shared admission assertion.
    assert_eq!(
        (
            selector.is_clean(),
            selector.syntax().is_some(),
            sheet.is_clean(),
            matches!(sheet.syntax().rules(), [CssRule::Style(_)]),
        ),
        (true, true, true, true),
        "{source}: selector={selector:?}; sheet={sheet:?}"
    );
}

#[test]
fn webkit_autofill_alias_is_admitted_on_ordinary_selector_and_sheet_fronts() {
    for source in [
        ":-webkit-autofill",
        "input:-WEBKIT-AUTOFILL",
        r"input:-\77 ebkit-auto\66 ill",
    ] {
        admitted(source);
    }
}

#[test]
fn unknown_nonfunctional_webkit_pseudo_element_is_admitted_on_both_fronts() {
    for source in [
        "::-webkit-asdf",
        ".field::-WebKit-AsDf",
        r".field::-\77 ebkit-asdf",
    ] {
        admitted(source);
    }
}

#[test]
fn functional_unknowns_other_prefixes_and_colon_whitespace_remain_invalid() {
    for source in [
        "::-webkit-jkl()",
        "::-WebKit-AsDf(ignored)",
        "::unknown-future",
        ":-webkit-unknown",
        ": -webkit-autofill",
        ":: -webkit-asdf",
        ": :before",
    ] {
        let selector = parse_selector(source, &CssNamespaceContext::default());
        let sheet = parse_sheet(&format!("{source}{{}}.after{{color:red}}"));
        assert_eq!(
            (
                selector.is_clean(),
                selector.syntax().is_some(),
                sheet.is_clean()
            ),
            (false, false, false),
            "{source}: selector={selector:?}; sheet={sheet:?}"
        );
        let [CssRule::Style(after)] = sheet.syntax().rules() else {
            panic!("only the later valid sibling survives: {source}: {sheet:?}")
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
fn existing_standard_legacy_colon_and_supported_pseudo_segments_stay_admitted() {
    for source in [
        ".field:hover",
        "::before",
        ".field:BEFORE",
        ".field::first-letter",
        ".button:hover::part(label)",
        ".button::part(label):hover::before",
    ] {
        admitted(source);
    }
}
