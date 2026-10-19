#![forbid(unsafe_code)]

//! Variables1 CR20220616 §§2/2.2 supplies every literal intrinsic fact and
//! the empty-authored-value versus guaranteed-invalid-initial distinction.

use surgeist_css::*;

#[test]
fn all_valid_names_share_the_complete_intrinsic_definition_and_selected_source() {
    let first = CssCustomPropertyName::try_new("--Family").unwrap();
    let metadata = first.metadata();
    for authored in ["--Other", "--family", "--空", r"\2d \2d Family"] {
        let name = CssCustomPropertyName::try_new(authored).unwrap();
        let actual = name.metadata();
        assert!(std::ptr::eq(actual, metadata));
        assert!(actual.accepts_declaration_values());
        assert!(actual.allows_empty_value());
        let initial: CssGuaranteedInvalidInitial = actual.initial_value();
        assert_eq!(initial, metadata.initial_value());
        assert!(actual.applies_to_all_elements_and_pseudo_elements());
        assert!(actual.inherited_by_default());
        assert!(!actual.has_percentage_basis());
        assert!(actual.computed_value_is_tokens_or_guaranteed_invalid());
        assert!(actual.canonical_order_is_grammar_order());
        assert!(actual.animation_is_discrete());
        assert!(actual.supports_all_media());
        let source = actual.source();
        assert_eq!(source.id().as_str(), "O-VARIABLES1");
        assert_eq!(
            source.module(),
            "CSS Custom Properties for Cascading Variables"
        );
        assert_eq!(source.level(), "1");
        assert_eq!(source.tier(), CssSpecificationTier::Snapshot2026Official);
        assert_eq!(
            source.url(),
            Some("https://www.w3.org/TR/2022/CR-css-variables-1-20220616/")
        );
        assert_eq!(source.repository_provenance(), None);
        assert_eq!(source, *specification_source("O-VARIABLES1").unwrap());
        // The family definition does not turn dynamic names into built-in rows.
        assert!(property_support_metadata(name.as_str()).is_none());
        assert!(CssKnownProperty::from_name(name.as_str()).is_none());
    }
}

fn assert_authored_initial_distinction(
    declaration: &CssDeclaration,
    initial: bool,
    expected: &str,
) {
    let before = declaration.clone();
    let components = declaration.value_components().clone();
    let custom = declaration.custom().unwrap();
    assert_eq!(custom.name().as_str(), "--x");
    let metadata = custom.name().metadata();
    let _: CssGuaranteedInvalidInitial = metadata.initial_value();
    if initial {
        assert!(matches!(
            custom.value(),
            CssCustomPropertyDeclaredValue::Global(CssGlobalKeyword::Initial)
        ));
        assert_eq!(custom.value().global(), Some(CssGlobalKeyword::Initial));
        assert!(custom.value().value().is_none());
        assert_eq!(declaration.importance(), CssImportance::Important);
    } else {
        let CssCustomPropertyDeclaredValue::Value(value) = custom.value() else {
            panic!("valid empty token value, not a computed invalid value");
        };
        assert!(value.is_empty());
        assert_eq!(value.as_css(), "");
        assert_eq!(custom.value().global(), None);
        assert_eq!(declaration.importance(), CssImportance::Normal);
    }
    let CssExpansion::Contributions(CssContributions::Custom(contribution)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("completed symbolic custom contribution");
    };
    assert!(contribution.source().same_occurrence(declaration));
    assert_eq!(contribution.declaration(), custom);
    assert_eq!(contribution.source().importance(), declaration.importance());
    assert_eq!(contribution.source().value_components(), &components);
    assert!(std::ptr::eq(
        contribution.declaration().name().metadata(),
        metadata
    ));
    assert_eq!(declaration.to_specified_css().unwrap(), expected);
    assert_eq!(declaration, &before);
    assert!(declaration.same_occurrence(&before));
    assert_eq!(declaration.value_components(), &components);
}

#[test]
fn empty_tokens_and_authored_initial_remain_distinct_through_checked_and_parsed_contributions() {
    const SOURCE: &str = "--x:;--x:initial!important";
    let report = parse_style_attribute(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [empty, initial] = report.syntax().as_slice() else {
        panic!("two ordered authored occurrences with distinct declared values");
    };
    assert_authored_initial_distinction(empty, false, "--x: ;");
    assert_authored_initial_distinction(initial, true, "--x: initial !important;");
    assert_ne!(
        empty.custom().unwrap().value(),
        initial.custom().unwrap().value()
    );
    assert_eq!(
        validate_style_attribute(SOURCE).unwrap(),
        report.syntax().clone()
    );

    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for (text, importance, is_initial, expected) in [
        ("", CssImportance::Normal, false, "--x: ;"),
        (
            "initial",
            CssImportance::Important,
            true,
            "--x: initial !important;",
        ),
    ] {
        let values = parse_component_values(text).unwrap();
        let original_values = values.clone();
        let checked =
            parse_property_value(CssPropertyNameRef::Custom(&name), values, importance).unwrap();
        assert!(checked.parsed_name().is_none());
        assert!(checked.parsed_value().is_none());
        assert_eq!(checked.value_components(), &original_values);
        assert_authored_initial_distinction(&checked, is_initial, expected);
        let reparsed = validate_style_attribute(expected).unwrap();
        let [reparsed] = reparsed.as_slice() else {
            panic!("one complete canonical custom declaration");
        };
        assert_authored_initial_distinction(reparsed, is_initial, expected);
    }
}
