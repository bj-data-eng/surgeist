//! Existing-API expectations from CSS Lists 3 (2020-11-17) §§4–4.2:
//! https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#counter-reset
//! https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#counter-increment
//! https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#counter-set
//! Names follow Values 4 custom-ident exclusions; integer math remains symbolic.

use crate::css::support::named_property::longhand_contributions;
use crate::css::support::named_property::validated_important_property_declaration as declaration;
use crate::css::support::named_property::{
    assert_checked_value_rejected_at, assert_programmatic_rejection, assert_value_diagnostic_at,
};
use surgeist_css::*;

const PROPERTIES: [CssKnownProperty; 3] = [
    CssKnownProperty::CounterReset,
    CssKnownProperty::CounterIncrement,
    CssKnownProperty::CounterSet,
];

fn ordinary(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    longhand_contributions(source).items().to_vec()
}

fn rejected<'a>(property: CssKnownProperty, value: &str, rest: impl Into<Option<&'a str>>) {
    let rest = rest.into();
    let source = format!("color:red;{}:{value};height:2px", property.canonical_name());
    let report = CssParser::default().parse_block_contents(&source);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|item| item.known().unwrap().property())
            .collect::<Vec<_>>(),
        [CssKnownProperty::Color, CssKnownProperty::Height],
        "{source}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one dropped declaration: {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_value_diagnostic_at(
        diagnostic.error(),
        property,
        format!("color:red;{}:", property.canonical_name()).len(),
        value,
        rest,
    );
    assert!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .is_err(),
        "{source}"
    );
    assert_checked_value_rejected_at(property, value, rest);
}

#[test]
fn three_names_are_noninherited_lists3_longhands_with_none_initials() {
    for (property, production) in [
        (CssKnownProperty::CounterReset, "#propdef-counter-reset"),
        (
            CssKnownProperty::CounterIncrement,
            "#propdef-counter-increment",
        ),
        (CssKnownProperty::CounterSet, "#propdef-counter-set"),
    ] {
        let feature = feature_metadata(property.grammar().feature_id().as_str()).unwrap();
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.source().id().as_str(), "I-LISTS3");
        assert_eq!(feature.production(), production);
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("{} is one longhand", property.canonical_name())
        };
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("none is a fixed initial")
        };
        let parsed_none = ordinary(&declaration(property, "none"));
        assert_eq!(parsed_none.len(), 1);
        assert_eq!(initial, parsed_none[0].ordinary_value().unwrap());
    }
}

#[test]
fn names_numbers_math_and_duplicate_entries_remain_authored_and_ordered() {
    for property in PROPERTIES {
        for value in [
            "none",
            "chapter",
            "chapter 0 section -2 item +3",
            "item item 3 item -4",
            "MyCounter 4 mycounter -1",
            r"\31 chapter -2 chapter\ name 3",
            "chapter 2147483648",
            "chapter -9223372036854775809",
            "chapter 1234567890123456789012345678901234567890",
            "chapter calc(2.5)",
            "chapter calc(1 + 2) section -3",
        ] {
            let source = declaration(property, value);
            assert!(
                source.known().unwrap().property_value().is_some(),
                "{value}"
            );
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|component| matches!(
                        component.origin(),
                        CssValueOrigin::Parsed(origin) if origin.source().as_str().contains(value)
                    ))
            );
        }
    }
}

#[test]
fn small_integer_values_keep_omitted_operands_and_duplicates_distinct() {
    let source = declaration(
        CssKnownProperty::CounterIncrement,
        "chapter section -2 chapter 0",
    );
    let CssKnownPropertyValueRef::CounterIncrement(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("counter increment wrapper")
    };
    let [first, second, third] = wrapper.value().changes().unwrap() else {
        panic!("three ordered authored entries")
    };
    assert_eq!(first.name().as_str(), "chapter");
    assert_eq!(first.value(), None);
    assert_eq!(second.name().as_str(), "section");
    assert!(
        matches!(second.value(), Some(value) if value.literal().is_some_and(|value| value.numeric().representation() == "-2"))
    );
    assert_eq!(third.name().as_str(), "chapter");
    assert!(
        matches!(third.value(), Some(value) if value.literal().is_some_and(|value| value.numeric().representation() == "0"))
    );
}

#[test]
fn invalid_names_operands_and_reversed_function_drop_only_their_declaration() {
    for property in PROPERTIES {
        // The `)` cannot continue the sum, so it is responsible.
        rejected(property, "chapter calc(1 +)", ")");
        for (value, rest) in [
            ("", ""),
            ("1", "1"),
            ("none chapter", "chapter"),
            ("chapter none", "none"),
            ("chapter, section", ", section"),
            ("inherit chapter", "chapter"),
            ("chapter default", "default"),
            ("default", "default"),
            ("DEFAULT", "DEFAULT"),
            (r"\64 efault", r#"\64 efault"#),
            (r"\69 nherit 1", "1"),
            ("chapter 1.5", "1.5"),
            ("chapter 1e2", "1e2"),
            ("chapter 1px", "1px"),
            ("chapter 2%", "2%"),
            ("chapter calc(2px)", "calc(2px)"),
            ("reversed(chapter)", "reversed(chapter)"),
        ] {
            rejected(property, value, rest);
        }
    }
}

#[test]
fn programmatic_components_share_grammar_and_have_programmatic_origins() {
    for property in PROPERTIES {
        let components = CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("chapter").unwrap(),
                CssComponentValue::try_token("-2").unwrap(),
            ],
            Default::default(),
        )
        .unwrap();
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            components,
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.known().unwrap().property(), property);
        assert!(
            checked
                .value_components()
                .items()
                .iter()
                .all(|component| matches!(component.origin(), CssValueOrigin::Programmatic))
        );
        let invalid = CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("chapter").unwrap(),
                CssComponentValue::try_token("1px").unwrap(),
            ],
            Default::default(),
        )
        .unwrap();
        assert_programmatic_rejection(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                invalid,
                CssImportance::Normal,
            ),
            property,
        );
    }
}

#[test]
fn ordinary_and_global_contributions_preserve_identity_importance_and_order() {
    for property in PROPERTIES {
        let source = declaration(property, "chapter -2 section");
        let values = ordinary(&source);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].property(), property);
        assert!(values[0].ordinary_value().is_some());
        assert!(values[0].source().same_occurrence(&source));
        assert_eq!(values[0].source().importance(), CssImportance::Important);
        assert!(values[0].replacement_components().is_none());
        for (spelling, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(property, spelling);
            let values = ordinary(&source);
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].property(), property);
            assert_eq!(values[0].value(), CssContributionValueRef::Global(keyword));
            assert!(values[0].source().same_occurrence(&source));
            assert_eq!(values[0].source().importance(), CssImportance::Important);
        }
        let escaped = declaration(property, r"\69 nherit");
        let values = ordinary(&escaped);
        assert_eq!(values.len(), 1);
        assert_eq!(
            values[0].value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
        );
        assert!(values[0].source().same_occurrence(&escaped));
    }
    let source = declaration(CssKnownProperty::All, "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        source.expand().unwrap()
    else {
        panic!("all supplies a reset")
    };
    for property in PROPERTIES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
    }
}

#[test]
fn pending_reentry_is_strict_and_reuses_the_original_occurrence() {
    for property in PROPERTIES {
        let source = declaration(property, "var(--counter)");
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("known counter property is pending")
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in ["chapter 1px", "chapter, section", "reversed(chapter)"] {
            assert!(
                pending
                    .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
                    .unwrap_err()
                    .replacement_error()
                    .is_some()
            );
        }
        assert!(
            pending
                .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
                .unwrap_err()
                .is_residual_substitution()
        );
        for replacement_css in ["none", "chapter -2 section", "chapter calc(2.5)", "inherit"] {
            let replacement =
                CssComponentValues::parse(&CssParser::default(), replacement_css).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one reentered longhand")
            };
            let [item] = values.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), property);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalization_keeps_three_counter_longhands_in_order_and_fails_atomically_at_limit() {
    let report = CssParser::default()
        .parse_sheet(".a{counter-reset:a;color:red;counter-increment:a 2;counter-set:b}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 4);
    for (order, property) in [
        CssKnownProperty::CounterReset,
        CssKnownProperty::Color,
        CssKnownProperty::CounterIncrement,
        CssKnownProperty::CounterSet,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[order].order(), order);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declarations[order].expansion()
        else {
            panic!("one normalized ordinary declaration")
        };
        let [item] = values.items() else {
            panic!("one member per declaration")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(declarations[order].source()));
    }
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 3).unwrap();
    let error = report.syntax().normalize(limits).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 3))
    );
    assert_eq!(error.declaration_order(), Some(3));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::CounterSet
    );
}
