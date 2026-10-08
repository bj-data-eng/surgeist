#![forbid(unsafe_code)]
//! Existing public string fronts only: VT1 CRD20240328 §§2.1,3.1,3.2;
//! selected Selectors4 §3.6.4 and the adopted scoped attachment disposition.
//! Unexecuted behavioral RED candidate; no proposed API symbols appear here.
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [value] = report.syntax().as_slice() else {
        panic!("one authored declaration")
    };
    let known = value
        .known()
        .expect("supported typed view-transition-name declaration");
    assert_eq!(known.property().canonical_name(), "view-transition-name");
    value.clone()
}

fn exact(source: &str, specified: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    let value = report
        .syntax()
        .as_ref()
        .expect("selected selector retained")
        .clone();
    assert_eq!(value.to_specified_css().unwrap(), specified);
    assert_eq!(
        parse_selector(specified, &CssNamespaceContext::default()).syntax(),
        &Some(value.clone())
    );
    assert!(report.into_validation_result().is_ok());
    value
}

#[test]
fn ordinary_names_are_supported_and_property_auto_is_invalid() {
    for source in [
        "view-transition-name:none",
        "view-transition-name:Card",
        r"view-transition-name:\43 ard",
    ] {
        assert!(
            declaration(source)
                .known()
                .unwrap()
                .property_value()
                .is_some()
        );
    }
    for invalid in [
        "auto",
        "AUTO",
        r"\61 uto",
        "default",
        "card extra",
        "1",
        "10%",
    ] {
        let source = format!("view-transition-name:{invalid};color:red");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.error().code() == CssErrorCode::InvalidPropertyValue
                    && d.action() == CssRecoveryAction::DropDeclaration)
        );
        let [sibling] = report.syntax().as_slice() else {
            panic!("valid later declaration survives")
        };
        assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    }
}

#[test]
fn css_wide_words_are_complete_global_alternatives() {
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let value = declaration(&format!("view-transition-name:{text}"));
        assert_eq!(value.known().unwrap().global(), Some(expected));
        assert!(value.known().unwrap().property_value().is_none());
    }
}

#[test]
fn substitution_values_keep_the_occurrence_and_strict_reentry() {
    for text in ["var(--tag)", "env(tag)", "attr(data-tag)"] {
        let source = declaration(&format!("view-transition-name:{text}!important"));
        assert!(source.known().unwrap().substitution_dependent().is_some());
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending authored name")
        };
        assert!(
            pending
                .reenter(parse_component_values("auto").unwrap())
                .is_err()
        );
        let replacement = parse_component_values("Card").unwrap();
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one ordinary longhand")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property().canonical_name(), "view-transition-name");
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}

#[test]
fn named_functions_keep_wildcard_and_unrestricted_custom_ident_arguments() {
    for name in [
        "view-transition-group",
        "view-transition-image-pair",
        "view-transition-old",
        "view-transition-new",
    ] {
        for argument in ["*", "Card", "none", "auto"] {
            let text = format!("::{name}({argument})");
            exact(&text, &text);
        }
    }
    exact(
        r"::VIEW-TRANSITION-OLD(\43 ard)",
        "::view-transition-old(Card)",
    );
}

#[test]
fn the_nonfunctional_root_has_canonical_identity() {
    exact("::VIEW-TRANSITION", "::view-transition");
    exact(r"::view-\74 ransition", "::view-transition");
}

#[test]
fn only_child_permission_is_scoped_and_propagates_to_logical_suffixes() {
    for name in ["group", "image-pair", "old", "new"] {
        for suffix in [
            ":only-child",
            ":not(:only-child)",
            ":is(:only-child,:hover)",
            ":where(:only-child)",
        ] {
            let text = format!("::view-transition-{name}(Card){suffix}");
            let report = parse_selector(&text, &CssNamespaceContext::default());
            assert!(report.is_clean(), "{text}: {report:?}");
            assert!(report.syntax().is_some());
        }
    }
    for source in [
        "::view-transition:only-child",
        "::before:only-child",
        "::view-transition-old(Card):current",
        "::view-transition-old(Card):first-child",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
    }
}

#[test]
fn defined_compound_chains_and_imported_receiving_permissions_are_admitted() {
    for source in [
        "::view-transition::view-transition-group(A)",
        "::view-transition-group(A)::view-transition-image-pair(B)",
        "::view-transition-image-pair(A)::view-transition-old(B)",
        "::view-transition-image-pair(A)::view-transition-new(B)",
        "::view-transition::view-transition-group(A)::view-transition-image-pair(B)::view-transition-old(C):only-child",
        "::part(label)::view-transition-old(A):only-child",
        "::slotted(.item)::view-transition",
        "::slotted(.item)::view-transition-group(A)",
        "::slotted(.item)::view-transition-image-pair(A)",
        "::slotted(.item)::view-transition-old(A)",
        "::slotted(.item)::view-transition-new(A)",
    ] {
        exact(source, source);
    }
}

#[test]
fn malformed_function_forms_and_existing_lexical_controls_stay_rejected() {
    exact(".control::before:hover", ".control::before:hover");
    for source in [
        ":view-transition",
        "::view-transition()",
        "::view-transition-group",
        "::view-transition-group()",
        "::view-transition-group(A B)",
        "::view-transition-group(A,B)",
        "::view-transition-group(\"A\")",
        "::view-transition-group(1)",
        "::view-transition-group(default)",
        "::view-transition-group(initial)",
        "::view-transition-group(**)",
        "::view-transition-group(A.x)",
        "::view-transition-group(.x)",
        "::view-transition-group (A)",
        ":: view-transition",
        ":unknown()",
        "::before > .child",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
    }
}
