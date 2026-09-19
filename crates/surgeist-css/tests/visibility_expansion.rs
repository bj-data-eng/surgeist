#![forbid(unsafe_code)]
//! Selected Display3 2026-06-05 §4 supplies
//! visible | hidden | collapse, inherited yes, initial visible, one longhand.
//! New longhand payload and canonical serializer assertions require functional APIs.
use surgeist_css::*;

fn declaration(value: &str, important: bool) -> CssDeclaration {
    let suffix = if important { "!important" } else { "" };
    let report = parse_style_attribute(&format!("visibility:{value}{suffix}"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one visibility declaration")
    };
    declaration.clone()
}

fn ordinary(source: &CssDeclaration) {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("Visibility has intrinsic expansion")
    else {
        panic!("ordinary longhand contribution")
    };
    let [value] = values.items() else {
        panic!("one terminal")
    };
    assert_eq!(value.property(), CssKnownProperty::Visibility);
    assert!(matches!(
        value.value(),
        CssContributionValueRef::Ordinary(_)
    ));
    assert_eq!(
        value.ordinary_value().unwrap().property().known_property(),
        CssKnownProperty::Visibility
    );
    assert!(value.source().same_occurrence(source));
    assert_eq!(value.source().importance(), source.importance());
    assert!(value.replacement_components().is_none());
}

#[test]
fn visibility_metadata_is_inherited_with_an_ordinary_initial() {
    let metadata = CssKnownProperty::Visibility
        .metadata()
        .expect("Visibility metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("Visibility longhand")
    };
    assert!(longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::Visibility
    );
    assert!(matches!(initial.view(), CssInitialValueRef::Value(_)));
    // This proves initial shape only. Direct Visible payload proof waits for
    // the real new Visibility longhand variant, not a parser-derived oracle.
}

#[test]
fn all_visibility_keywords_expand_once_with_original_occurrence() {
    for text in ["visible", "hidden", "collapse"] {
        for important in [false, true] {
            ordinary(&declaration(text, important));
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()])
                    .unwrap();
            let source = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Visibility),
                components.clone(),
                if important {
                    CssImportance::Important
                } else {
                    CssImportance::Normal
                },
            )
            .unwrap();
            assert_eq!(source.value_components(), &components);
            assert!(matches!(
                components.items()[0].origin(),
                CssValueOrigin::Programmatic
            ));
            ordinary(&source);
        }
    }
}

#[test]
fn visibility_globals_target_the_same_terminal_without_resolving_inheritance() {
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for important in [false, true] {
            let source = declaration(text, important);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("global terminal")
            };
            let [value] = values.items() else {
                panic!("one terminal")
            };
            assert_eq!(value.property(), CssKnownProperty::Visibility);
            assert_eq!(value.value(), CssContributionValueRef::Global(expected));
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), source.importance());
        }
    }
}

#[test]
fn visibility_pending_reentry_is_strict_and_keeps_original_and_replacement_origins() {
    let source = declaration("var(--visibility)", true);
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending visibility")
    };
    assert!(handle.source().same_occurrence(&source));
    for text in ["visible", "hidden", "collapse"] {
        let replacement = parse_component_values(text).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("replacement longhand")
        };
        let [value] = values.items() else {
            panic!("one replacement")
        };
        assert_eq!(value.property(), CssKnownProperty::Visibility);
        assert!(matches!(
            value.value(),
            CssContributionValueRef::Ordinary(_)
        ));
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&replacement));
    }
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("global replacement")
    };
    assert_eq!(values.items().len(), 1);
    assert_eq!(
        values.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
    for text in [
        "auto",
        "force-hidden",
        "visible hidden",
        "inherit hidden",
        "hidden; color:red",
        "hidden!important",
    ] {
        let error = handle
            .reenter(parse_component_values(text).unwrap())
            .unwrap_err();
        assert!(
            matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
            "{text}"
        );
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    assert!(handle.source().same_occurrence(&source));
}

#[test]
fn visibility_existing_keyword_grammar_identity_and_recovery_are_preserved() {
    for (text, expected) in [
        ("visible", CssVisibility::Visible),
        ("hidden", CssVisibility::Hidden),
        ("collapse", CssVisibility::Collapse),
        ("HIDDEN", CssVisibility::Hidden),
        (r"v\69 sible", CssVisibility::Visible),
    ] {
        let parsed = declaration(text, false);
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Visibility),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        for source in [parsed, checked] {
            let CssKnownPropertyValueRef::Visibility(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("visibility wrapper")
            };
            assert_eq!(value.i01_subset(), Some(&expected));
            assert_eq!(value.as_css(), text);
        }
    }
    for text in [
        "auto",
        "force-hidden",
        "visible hidden",
        "hidden collapse",
        "none",
        "0",
        "",
        "inherit visible",
    ] {
        let report = parse_style_attribute(&format!("color:red;visibility:{text};color:blue"));
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid visibility: {text}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2);
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Visibility),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}
