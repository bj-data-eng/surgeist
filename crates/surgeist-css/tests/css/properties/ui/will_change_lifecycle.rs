//! Existing-API RED expectations from CSS Will Change 1 CRD (2022-05-05), §2, and Values 4 §4.2: https://www.w3.org/TR/2022/CRD-css-will-change-1-20220505/#will-change.
//! Canonical writers and new typed payload accessors are tested with their implementation;
//! this candidate exercises admission and the already-existing authored lifecycle APIs.

use crate::css::support::validated_important_declaration as declaration;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssPropertyGrammar,
    CssPropertyKindRef, CssPropertyNameRef, CssPropertyTarget,
};

use crate::css::support::authored_property;
use crate::css::support::named_property::sole_longhand_contribution as contribution;

type PropertyCase = (
    &'static str,
    &'static str,
    bool,
    &'static [&'static str],
    &'static [&'static str],
);

const CASES: &[PropertyCase] = &[(
    "will-change",
    "auto",
    false,
    &[
        "auto",
        "scroll-position",
        "contents",
        "transform, opacity",
        "FutureProperty, --custom",
        r"f\75 ture, contents, future",
    ],
    &[
        "",
        "none",
        "all",
        "will-change",
        "default",
        "auto,opacity",
        "inherit,contents",
        "scroll-position contents",
        "opacity,",
        ",opacity",
        "1",
        "\"opacity\"",
    ],
)];

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
        // Unknown future identifiers are valid; excluded keyword "none" supplies the failure oracle.
        let invalid = CssComponentValues::try_new(
            vec![CssComponentValue::try_ident("none").unwrap()],
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
            let report = CssParser::default().parse_block_contents(&text);
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
            assert!(
                CssParser::default()
                    .parse_block_contents(&text)
                    .into_validation_result()
                    .is_err()
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
