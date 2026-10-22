//! Existing-API RED expectations from CSS Scrollbars 1 CR (2021-12-09), §§2–3: https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/.
//! Canonical writers and new typed payload accessors are tested with their implementation;
//! this candidate exercises admission and the already-existing authored lifecycle APIs.

use crate::css::support::validated_important_declaration as declaration;
use surgeist_css::*;

use crate::css::support::authored_property;
use crate::css::support::named_property::sole_longhand_contribution as contribution;

type PropertyCase = (
    &'static str,
    &'static str,
    bool,
    &'static [&'static str],
    &'static [&'static str],
);

const CASES: &[PropertyCase] = &[
    (
        "scrollbar-width",
        "auto",
        false,
        &["auto", "thin", "NONE", r"th\69 n"],
        &["", "8", "10%", "auto thin", "thin,none", "wide"],
    ),
    (
        "scrollbar-color",
        "auto",
        true,
        &[
            "auto",
            "red blue",
            "currentcolor Canvas",
            "rgb(10 20 30) color(srgb .2 .3 .4)",
        ],
        &["", "red", "red blue green", "auto red", "red,blue", "thin"],
    ),
];

#[test]
fn checked_construction_admits_the_same_grammar_and_preserves_input_origins() {
    for &(name, _, _, valid, _) in CASES {
        // Obtain identity only after real successful admission; no new symbols are needed.
        let admitted = declaration(name, valid[0]);
        let property = admitted.known().unwrap().property();
        let grammar = CssPropertyGrammar::from_name(name).unwrap();
        assert_eq!(grammar.target_property(), property);
        for &text in valid {
            let components = parse_component_values(text).unwrap();
            for source in [
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
                parse_property_value_for_grammar(
                    grammar,
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
            ] {
                assert!(source.parsed_value().is_none());
                assert_eq!(source.value_components(), &components);
                assert_eq!(source.importance(), CssImportance::Important);
                assert_eq!(source.known().unwrap().property(), property);
            }
        }
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(valid[0]).unwrap()])
                .unwrap();
        let source = parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert!(matches!(
            source.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        let invalid = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("definitely-invalid-keyword").unwrap(),
        ])
        .unwrap();
        let error = parse_property_value(
            CssPropertyNameRef::Known(property),
            invalid,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        let responsible = match error.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
            other => panic!("invalid checked value lost its original component origin: {other:?}"),
        };
        assert_eq!(responsible, &CssValueOrigin::Programmatic);
    }
}

#[test]
fn intrinsic_longhand_metadata_has_the_pinned_initial_and_inheritance() {
    for &(name, initial_text, inherited, _, _) in CASES {
        let source = declaration(name, initial_text);
        let property = source.known().unwrap().property();
        let CssPropertyKindRef::Longhand(metadata) = property
            .metadata()
            .expect("intrinsic longhand metadata")
            .kind()
        else {
            panic!("{name} is a longhand")
        };
        assert_eq!(metadata.property().known_property(), property);
        assert_eq!(metadata.inherited_by_default(), inherited);
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} has a fixed intrinsic initial")
        };
        assert_eq!(initial, contribution(&source).ordinary_value().unwrap());
    }
}

#[test]
fn normalization_retains_order_and_importance_with_an_exact_contribution_limit() {
    for &(name, _, _, valid, _) in CASES {
        authored_property::normalized_longhand_order_and_limit(name, valid);
    }
}

#[test]
fn invalid_recovery_control_preserves_neighbors_and_the_original_snapshot() {
    // This is a recovery control on the baseline even where the property is absent.
    // Admission tests above independently require valid values to be supported.
    for &(name, _, _, _, invalid) in CASES {
        for &value in invalid {
            let text = format!("color:red;{name}:{value};height:2px!important");
            let report = parse_style_attribute(&text);
            let snapshot = report.syntax().to_vec();
            let [diagnostic] = report.diagnostics() else {
                panic!("one dropped invalid occurrence: {text}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_eq!(report.syntax().len(), 2);
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            assert_eq!(
                report.syntax()[1].known().unwrap().property(),
                CssKnownProperty::Height
            );
            assert_eq!(report.syntax()[1].importance(), CssImportance::Important);
            assert_eq!(
                report.syntax()[0]
                    .value_components()
                    .serialize()
                    .unwrap()
                    .as_css(),
                "red"
            );
            assert_eq!(
                report.syntax()[1]
                    .value_components()
                    .serialize()
                    .unwrap()
                    .as_css(),
                "2px"
            );
            assert!(validate_style_attribute(&text).is_err());
            for (source, original) in report.syntax().iter().zip(&snapshot) {
                assert!(source.same_occurrence(original));
                assert_eq!(source.value_components(), original.value_components());
            }
        }
    }
}

#[test]
fn color_lifecycle_control_exercises_the_same_shared_front_doors() {
    let source = declaration("color", "red");
    let item = contribution(&source);
    assert_eq!(item.property(), CssKnownProperty::Color);
    let source = declaration("color", "var(--control)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("existing color control is pending")
    };
    assert!(matches!(
        handle
            .reenter(parse_component_values("inherit").unwrap())
            .unwrap(),
        CssContributions::Longhands(_)
    ));
    assert!(normalize_sheet(parse_sheet(".a{color:red!important;color:blue}").syntax()).is_ok());
}

#[test]
fn scrollbar_width_admission_control_preserves_typed_keywords_and_origins() {
    for (text, expected) in [
        ("auto", CssScrollbarWidth::Auto),
        ("THIN", CssScrollbarWidth::Thin),
        (r"n\6f ne", CssScrollbarWidth::None),
    ] {
        let source = declaration("scrollbar-width", text);
        let CssKnownPropertyValueRef::ScrollbarWidth(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("existing scrollbar-width typed value")
        };
        assert_eq!(value.value(), &expected);
        assert_eq!(value.as_css(), text);
    }
}
