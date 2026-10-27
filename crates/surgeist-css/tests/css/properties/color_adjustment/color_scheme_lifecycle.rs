//! Existing-API RED expectations from CSS Color Adjustment 1 CR (2025-12-16), §2.1 and Values 4 §4.2.
//! https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/
//! Contextual adjustment, palettes, output-device policy and used colors are downstream.
//! Canonical writers and new typed payload accessors are tested with their implementation;
//! this candidate exercises admission and the already-existing authored lifecycle APIs.

use crate::css::support::known_property;
use crate::css::support::property_admission::assert_invalid_value;
use crate::css::support::validated_important_declaration as declaration;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssLonghandContribution,
    CssPropertyGrammar, CssPropertyKindRef, CssPropertyNameRef, CssPropertyTarget,
};

use crate::css::support::authored_property;

type PropertyCase = (
    &'static str,
    &'static str,
    bool,
    &'static [&'static str],
    &'static [(&'static str, Option<&'static str>)],
);

const CASES: &[PropertyCase] = &[(
    "color-scheme",
    "normal",
    true,
    &[
        "normal",
        "light",
        "dark",
        "light dark",
        "dark light",
        "only light",
        "light only",
        "only light dark",
        "light dark only",
        "light light",
        "Future future",
        "none",
        "light none",
        r"only l\69 ght Future",
        r"f\75 ture dark only",
    ],
    &[
        ("", None),
        ("only", Some("only")),
        ("only only", Some("only")),
        ("normal light", Some("light")),
        ("light normal", Some("normal")),
        ("normal only", Some("only")),
        ("light only dark", Some("dark")),
        ("only light only", Some("only")),
        ("light,dark", Some(",")),
        ("default", Some("default")),
        ("light default", Some("default")),
        ("light inherit", Some("inherit")),
        ("light revert-layer", Some("revert-layer")),
        ("1", Some("1")),
        ("10%", Some("10%")),
    ],
)];

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        source.expand().expect("selected authored longhand expands")
    else {
        panic!("one completed longhand")
    };
    let [item] = values.items() else {
        panic!("exactly one terminal contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    item.clone()
}

#[test]
fn source_derived_valid_values_are_admitted_with_original_components() {
    for &(name, _, _, valid, _) in CASES {
        for &text in valid {
            let source = declaration(name, text);
            assert!(source.known().unwrap().property_value().is_some());
            assert!(source.parsed_value().is_some());
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|item| { !matches!(item.origin(), CssValueOrigin::Programmatic) })
            );
        }
    }
}

#[test]
fn checked_construction_admits_the_same_grammar_and_preserves_input_origins() {
    for &(name, _, _, valid, _) in CASES {
        // Obtain identity only after real successful admission; no new symbols are needed.
        let admitted = declaration(name, valid[0]);
        let property = admitted.known().unwrap().property();
        let grammar = CssPropertyGrammar::from_name(name).unwrap();
        assert_eq!(grammar.target_property(), property);
        for &text in valid {
            let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
            for source in [
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Grammar(grammar),
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
        let components = CssComponentValues::try_new(
            vec![CssComponentValue::try_ident(valid[0]).unwrap()],
            Default::default(),
        )
        .unwrap();
        let source = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert!(matches!(
            source.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        let invalid = CssComponentValues::try_new(
            vec![CssComponentValue::try_ident("default").unwrap()],
            Default::default(),
        )
        .unwrap();
        let error = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            invalid,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(error.code().is_some());
        let responsible = match error.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
            other => panic!("invalid checked value lost its original component origin: {other:?}"),
        };
        assert_eq!(responsible, CssValueOrigin::Programmatic);
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
fn ordinary_values_contribute_once_without_losing_occurrence_or_importance() {
    for &(name, _, _, valid, _) in CASES {
        for &text in valid {
            let parsed = declaration(name, text);
            let property = parsed.known().unwrap().property();
            let checked = CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                parsed.value_components().clone(),
                CssImportance::Important,
            )
            .unwrap();
            let parsed_item = contribution(&parsed);
            let checked_item = contribution(&checked);
            assert!(parsed_item.ordinary_value().is_some());
            assert_eq!(parsed_item.ordinary_value(), checked_item.ordinary_value());
            assert!(parsed_item.replacement_components().is_none());
            assert!(checked_item.replacement_components().is_none());
        }
    }
}

#[test]
fn pending_reentry_is_strict_repeatable_and_retains_replacement_provenance() {
    for &(name, _, _, valid, invalid) in CASES {
        let terminal = name;
        let invalid: Vec<_> = invalid.iter().map(|&(value, _)| value).collect();
        authored_property::pending_longhand_reentry(name, valid, &invalid, terminal, declaration);
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
        for &(value, token) in invalid {
            let text = format!("color:red;{name}:{value};height:2px!important");
            let report = CssParser::default().parse_block_contents(&text);
            let snapshot = report.syntax().to_vec();
            let [diagnostic] = report.diagnostics() else {
                panic!("one dropped invalid occurrence: {text}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_invalid_value(diagnostic.error(), known_property(name), token);
            let error_offset = diagnostic.error().position().byte_offset().value();
            let property_start = "color:red;".len();
            let value_end = property_start + name.len() + 1 + value.len();
            assert!(
                (property_start..=value_end).contains(&error_offset),
                "diagnostic must refer to the original invalid occurrence: {text}"
            );
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
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                "red"
            );
            assert_eq!(
                report.syntax()[1]
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                "2px"
            );
            assert_eq!(
                CssParser::default()
                    .parse_block_contents(&text)
                    .into_validation_result()
                    .unwrap_err()
                    .diagnostics(),
                report.diagnostics()
            );
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
    let CssExpansion::Pending(handle) = source.expand().unwrap() else {
        panic!("existing color control is pending")
    };
    assert!(matches!(
        handle
            .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
            .unwrap(),
        CssContributions::Longhands(_)
    ));
    assert!(
        CssParser::default()
            .parse_sheet(".a{color:red!important;color:blue}")
            .syntax()
            .normalize(Default::default())
            .is_ok()
    );
}

#[test]
fn ordered_repeated_custom_schemes_and_only_positions_keep_the_complete_authored_graph() {
    for text in [
        "light dark",
        "dark light",
        "light light",
        "Future future Future",
        "only light Future",
        "light Future only",
        "none",
        "only none",
    ] {
        let source = declaration("color-scheme", text);
        let components = source.value_components().clone();
        let item = contribution(&source);
        assert!(item.ordinary_value().is_some());
        assert_eq!(item.source().value_components(), &components);
        assert_eq!(
            item.source()
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            text
        );
        assert!(item.source().same_occurrence(&source));
    }
}
