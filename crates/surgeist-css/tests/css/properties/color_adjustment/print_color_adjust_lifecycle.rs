//! Existing-API RED expectations from CSS Color Adjustment 1 CR (2025-12-16), §§4.1–4.2.
//! https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/
//! Contextual adjustment, palettes, output-device policy and used colors are downstream.
//! Canonical writers and new typed payload accessors are tested with their implementation;
//! this candidate exercises admission and the already-existing authored lifecycle APIs.

use crate::css::support::known_property;
use crate::css::support::property_admission::assert_invalid_value;
use crate::css::support::validated_important_declaration as declaration;
use surgeist_css::*;

use crate::css::support::authored_property;

type PropertyCase = (
    &'static str,
    &'static str,
    bool,
    &'static [&'static str],
    &'static [(&'static str, Option<&'static str>)],
);

const CASES: &[PropertyCase] = &[
    (
        "print-color-adjust",
        "economy",
        true,
        &["economy", "exact", "EXACT", r"ec\6f nomy"],
        &[
            ("", None),
            ("auto", Some("auto")),
            ("none", Some("none")),
            ("economy exact", Some("exact")),
            ("exact,economy", Some(",")),
            ("1", Some("1")),
            ("10%", Some("10%")),
        ],
    ),
    (
        "color-adjust",
        "economy",
        true,
        &["economy", "exact", "EXACT", r"ec\6f nomy"],
        &[
            ("", None),
            ("auto", Some("auto")),
            ("none", Some("none")),
            ("economy exact", Some("exact")),
            ("exact,economy", Some(",")),
            ("1", Some("1")),
            ("10%", Some("10%")),
        ],
    ),
];

fn terminal_name(source: &CssDeclaration) -> &'static str {
    match source.known().unwrap().property().canonical_name() {
        "color-adjust" => "print-color-adjust",
        name => name,
    }
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        source.expand().expect("selected authored longhand expands")
    else {
        panic!("one completed longhand")
    };
    let [item] = values.items() else {
        panic!("exactly one terminal contribution")
    };
    assert_eq!(item.property().canonical_name(), terminal_name(source));
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    item.clone()
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
            vec![CssComponentValue::try_ident("definitely-invalid-keyword").unwrap()],
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
        if name == "color-adjust" {
            continue; // The dedicated shorthand test asserts its referenced longhand metadata.
        }
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
fn deprecated_color_adjust_keeps_distinct_shorthand_identity_and_one_print_member() {
    let source = declaration("color-adjust", "exact");
    let print = declaration("print-color-adjust", "exact");
    let shorthand = source.known().unwrap().property();
    let longhand = print.known().unwrap().property();
    assert_ne!(shorthand, longhand);
    assert_eq!(source.known().unwrap().grammar().name(), "color-adjust");
    assert_eq!(
        print.known().unwrap().grammar().name(),
        "print-color-adjust"
    );
    let CssPropertyKindRef::Shorthand(metadata) = shorthand.metadata().unwrap().kind() else {
        panic!("color-adjust is a standard shorthand, not a name-equivalent alias")
    };
    assert_eq!(metadata.members().len(), 1);
    assert_eq!(metadata.members()[0].known_property(), longhand);
    assert_eq!(metadata.settable_members(), metadata.members());
    assert!(metadata.reset_only_members().is_empty());
    assert!(!metadata.is_legacy());
    let item = contribution(&source);
    assert_eq!(item.property(), longhand);
    assert!(item.source().same_occurrence(&source));
    assert_eq!(
        item.source()
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "exact"
    );
}
