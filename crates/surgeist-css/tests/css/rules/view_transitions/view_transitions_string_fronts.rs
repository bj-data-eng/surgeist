//! Public string fronts: VT1 CRD20240328 §§2.1,3.1,3.2;
//! selected Selectors4 §3.6.4 and the adopted scoped attachment disposition.
use crate::css::support::rules::ExpectedRejection;
use crate::css::support::sole_declaration;
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let value = sole_declaration(source);
    let known = value
        .known()
        .expect("supported typed view-transition-name declaration");
    assert_eq!(known.property().canonical_name(), "view-transition-name");
    value
}

fn exact(source: &str, specified: &str) -> CssSelector {
    let report = CssSelector::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{source}: {report:?}");
    let value = report
        .syntax()
        .as_ref()
        .expect("selected selector retained")
        .clone();
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        specified
    );
    assert_eq!(
        CssSelector::parse(
            &CssParser::default(),
            specified,
            &CssNamespaceContext::default()
        )
        .syntax(),
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
        let report = CssParser::default().parse_block_contents(&source);
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
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("pending authored name")
        };
        // VT1 §3.1: view-transition-name is none | <custom-ident> without auto.
        let error = pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "auto").unwrap())
            .unwrap_err();
        let Some(replacement) = error.replacement_error() else {
            panic!("{text}: invalid replacement: {error:?}")
        };
        ExpectedRejection::new(CssErrorCode::InvalidPropertyValue)
            .named("view-transition-name")
            .at("auto")
            .assert_checked(replacement);
        let replacement = CssComponentValues::parse(&CssParser::default(), "Card").unwrap();
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
            let report = CssSelector::parse(
                &CssParser::default(),
                &text,
                &CssNamespaceContext::default(),
            );
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
        let report = CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        ExpectedRejection::new(CssErrorCode::InvalidSelector).assert_first(&report);
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
fn malformed_function_forms_and_lexical_controls_stay_rejected() {
    exact(".control::before:hover", ".control::before:hover");
    // VT1 §3.2: the named pseudo-elements take one <pt-name-selector>,
    // whose <custom-ident> excludes CSS-wide keywords and `default`, and
    // Selectors 4 §3.6.3 admits no combinator after a pseudo-element.
    let invalid = ExpectedRejection::new(CssErrorCode::InvalidSelector);
    for (source, expected) in [
        (":view-transition", invalid.clone()),
        ("::view-transition()", invalid.clone()),
        ("::view-transition-group", invalid.clone()),
        ("::view-transition-group()", invalid.clone()),
        ("::view-transition-group(A B)", invalid.clone().at("B")),
        ("::view-transition-group(A,B)", invalid.clone().at(",")),
        (
            "::view-transition-group(\"A\")",
            invalid.clone().at("\"A\""),
        ),
        ("::view-transition-group(1)", invalid.clone().at("1")),
        (
            "::view-transition-group(default)",
            invalid.clone().at("default"),
        ),
        (
            "::view-transition-group(initial)",
            invalid.clone().at("initial"),
        ),
        ("::view-transition-group(**)", invalid.clone()),
        ("::view-transition-group(A.x)", invalid.clone()),
        ("::view-transition-group(.x)", invalid.clone().at(".")),
        ("::view-transition-group (A)", invalid.clone()),
        (":: view-transition", invalid.clone()),
        (":unknown()", invalid.clone()),
        // The combinator after the pseudo-element is responsible.
        ("::before > .child", invalid.clone().at(">")),
    ] {
        let report = CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        expected.assert_first(&report);
    }
}
