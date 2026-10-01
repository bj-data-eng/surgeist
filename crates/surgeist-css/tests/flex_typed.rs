#![forbid(unsafe_code)]

//! Public checked flex construction and intrinsic specified-value behavior.

use surgeist_css::*;

fn one_component(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    let [component] = values.items() else {
        panic!("expected one component: {text}")
    };
    component.clone()
}

fn factor(text: &str) -> CssSpecifiedNonNegativeNumber {
    CssSpecifiedNonNegativeNumber::try_from_component(CssComponentValue::try_number(text).unwrap())
        .unwrap()
}

fn basis(text: &str) -> CssFlexBasisValue {
    let source = format!("flex-basis:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::FlexBasis(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed flex basis: {source}")
    };
    value.value().clone()
}

fn flex(text: &str) -> CssFlexValue {
    let source = format!("flex:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Flex(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed flex: {source}")
    };
    value.value().clone()
}

fn contributions(text: &str) -> Vec<CssLonghandContribution> {
    let source = format!("flex:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("three flex contributions: {source}")
    };
    assert_eq!(values.items().len(), 3);
    for (item, name) in values
        .items()
        .iter()
        .zip(["flex-grow", "flex-shrink", "flex-basis"])
    {
        assert_eq!(item.property().canonical_name(), name);
        assert!(item.source().same_occurrence(declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    values.items().to_vec()
}

#[test]
fn checked_components_reject_empty_and_orphan_shrink_but_admit_basis_only() {
    assert!(CssFlexComponents::try_new(None, None, None).is_none());
    assert!(CssFlexComponents::try_new(None, Some(factor("2")), None).is_none());
    assert!(CssFlexComponents::try_new(None, Some(factor("2")), Some(basis("10px"))).is_none());

    let components = CssFlexComponents::try_new(None, None, Some(basis("10px"))).unwrap();
    assert!(components.grow().is_none());
    assert!(components.shrink().is_none());
    assert_eq!(
        components.basis().unwrap().serialize_specified().unwrap(),
        "10px"
    );
}

#[test]
fn checked_factors_preserve_exact_lexical_numbers_and_symbolic_math() {
    for (text, expected) in [
        ("0", "0".to_owned()),
        ("16777217", "16777217".to_owned()),
        ("1e-100", "0".to_owned()),
    ] {
        let value = factor(text);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let Some(component) = value.literal_component() else {
            panic!("exact lexical literal: {text}")
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            panic!("number token: {text}")
        };
        assert_eq!(number.representation(), text);
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    }
    for text in ["-1", "-1e-100"] {
        let error = CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    }
    let math =
        CssNumberCalculation::try_from_components(parse_component_values("calc(1 + 2)").unwrap())
            .unwrap();
    let value = CssSpecifiedNonNegativeNumber::try_from_calculation(math).unwrap();
    assert!(value.calculation().is_some());
    assert_eq!(value.serialize_specified().unwrap(), "calc(3)");
}

#[test]
fn flex_basis_borrows_content_size_and_calc_size_without_erasing_structure() {
    assert!(matches!(
        CssFlexBasisValue::content().view(),
        CssFlexBasisRef::Content
    ));
    assert_eq!(
        CssFlexBasisValue::content().serialize_specified().unwrap(),
        "content"
    );

    let size = CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_dimension("10", "px").unwrap(),
        )
        .unwrap(),
    ));
    let from_size = CssFlexBasisValue::from(size);
    assert!(matches!(from_size.view(), CssFlexBasisRef::Size(_)));
    assert_eq!(from_size.serialize_specified().unwrap(), "10px");

    let calc = CssCalcSize::try_from_component(one_component("calc-size(content, size)")).unwrap();
    let direct = CssFlexBasisValue::from(calc.clone());
    let through_box_size = CssFlexBasisValue::from(CssSizeValue::BoxSize(CssBoxSize::CalcSize(
        CssBoxCalcSize::try_from(
            CssCalcSize::try_from_component(one_component("calc-size(min-content, size)")).unwrap(),
        )
        .unwrap(),
    )));
    assert!(matches!(direct.view(), CssFlexBasisRef::CalcSize(_)));
    assert!(matches!(
        through_box_size.view(),
        CssFlexBasisRef::CalcSize(_)
    ));
    assert_eq!(direct, CssFlexBasisValue::from(calc));
    assert_eq!(through_box_size, basis("calc-size(min-content, size)"));
}

#[test]
fn content_calc_size_is_flex_valid_and_box_invalid_at_the_offending_basis() {
    for text in [
        "calc-size(content, size)",
        "calc-size(calc-size(content, size), size + 1px)",
    ] {
        let calc = CssCalcSize::try_from_component(one_component(text)).unwrap();
        assert!(matches!(
            CssFlexBasisValue::from(calc.clone()).view(),
            CssFlexBasisRef::CalcSize(_)
        ));
        let expected_origin = match calc.basis() {
            CssCalcSizeBasisRef::Nested(child) => child.basis_origin(),
            _ => calc.basis_origin(),
        }
        .clone();
        let error = CssBoxCalcSize::try_from(calc).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::InvalidArgumentType
        );
        assert_eq!(error.origin(), Some(&expected_origin));
    }

    let constructed = CssComponentValue::try_function(
        "calc-size",
        parse_component_values("content, size").unwrap(),
    )
    .unwrap();
    let calc = CssCalcSize::try_from_component(constructed).unwrap();
    let error = CssBoxCalcSize::try_from(calc.clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::InvalidArgumentType
    );
    assert_eq!(error.origin(), Some(calc.basis_origin()));
    assert!(matches!(error.origin(), Some(CssValueOrigin::Parsed(_))));

    let nested_auto =
        CssCalcSize::try_from_component(one_component("calc-size(calc-size(auto, size), size)"))
            .unwrap();
    let box_value = CssBoxCalcSize::try_from(nested_auto).unwrap();
    assert_eq!(
        CssMaxSizeValue::try_box_size(CssBoxSize::CalcSize(box_value))
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::InvalidArgumentType
    );
}

#[test]
fn typed_flex_serialization_materializes_the_specified_default_triple() {
    for (source, expected) in [
        ("0%", "1 1 0%"),
        ("2", "2 1 0"),
        ("10px", "1 1 10px"),
        ("2 3 10px", "2 3 10px"),
    ] {
        assert_eq!(
            flex(source).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
    assert_eq!(CssFlexValue::None.serialize_specified().unwrap(), "none");
    assert_eq!(CssFlexValue::Auto.serialize_specified().unwrap(), "auto");

    let constructed = CssFlexValue::Components(
        CssFlexComponents::try_new(Some(factor("2")), None, None).unwrap(),
    );
    assert_eq!(constructed.serialize_specified().unwrap(), "2 1 0");
    let CssFlexValue::Components(components) = &constructed else {
        panic!("directly constructed flex components")
    };
    assert_eq!(
        components.grow().unwrap().serialize_specified().unwrap(),
        "2"
    );
    assert!(components.shrink().is_none());
    assert!(components.basis().is_none());
    let basis_only_zero =
        CssFlexValue::Components(CssFlexComponents::try_new(None, None, Some(basis("0"))).unwrap());
    assert_eq!(basis_only_zero.serialize_specified().unwrap(), "1 1 0");
}

#[test]
fn serialized_flex_reparses_with_the_same_effective_terminals() {
    for source in ["0", "0%", "2", "10px", "2 3 10px", "none", "auto"] {
        let serialized = flex(source).serialize_specified().unwrap();
        assert_eq!(flex(&serialized).serialize_specified().unwrap(), serialized);
        let original = contributions(source);
        let reparsed = contributions(&serialized);
        for (left, right) in original.iter().zip(reparsed.iter()) {
            match (
                left.ordinary_value().unwrap().view(),
                right.ordinary_value().unwrap().view(),
            ) {
                (CssLonghandValueRef::FlexGrow(a), CssLonghandValueRef::FlexGrow(b))
                | (CssLonghandValueRef::FlexShrink(a), CssLonghandValueRef::FlexShrink(b)) => {
                    assert_eq!(
                        a.serialize_specified().unwrap(),
                        b.serialize_specified().unwrap()
                    );
                }
                (CssLonghandValueRef::FlexBasis(a), CssLonghandValueRef::FlexBasis(b)) => {
                    assert_eq!(
                        a.serialize_specified().unwrap(),
                        b.serialize_specified().unwrap()
                    );
                }
                _ => panic!("mismatched effective flex terminal: {source}"),
            }
        }
    }
}

#[test]
fn intrinsic_flex_initials_are_zero_one_auto() {
    for (name, expected) in [
        ("flex-grow", "0"),
        ("flex-shrink", "1"),
        ("flex-basis", "auto"),
    ] {
        let property = CssPropertyGrammar::from_name(name)
            .unwrap()
            .target_property();
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("longhand metadata: {name}")
        };
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("ordinary initial: {name}")
        };
        assert_eq!(value.property().known_property(), property);
        let serialized = match value.view() {
            CssLonghandValueRef::FlexGrow(v) | CssLonghandValueRef::FlexShrink(v) => {
                assert_eq!(v.origin(), &CssValueOrigin::Programmatic);
                v.serialize_specified().unwrap()
            }
            CssLonghandValueRef::FlexBasis(v) => v.serialize_specified().unwrap(),
            _ => panic!("typed initial: {name}"),
        };
        assert_eq!(serialized, expected);
    }
}

#[test]
fn expanded_flex_keywords_and_omissions_have_distinct_effective_values() {
    for (source, expected) in [
        ("none", ["0", "0", "auto"]),
        ("auto", ["1", "1", "auto"]),
        ("2", ["2", "1", "0"]),
        ("10px", ["1", "1", "10px"]),
    ] {
        let actual = contributions(source)
            .iter()
            .map(|item| match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::FlexGrow(v) | CssLonghandValueRef::FlexShrink(v) => {
                    v.serialize_specified().unwrap()
                }
                CssLonghandValueRef::FlexBasis(v) => v.serialize_specified().unwrap(),
                _ => panic!("unexpected flex terminal"),
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "flex:{source}");
    }
    let omitted = contributions("2");
    let CssLonghandValueRef::FlexShrink(shrink) = omitted[1].ordinary_value().unwrap().view()
    else {
        panic!("synthetic shrink")
    };
    assert_eq!(shrink.origin(), &CssValueOrigin::Programmatic);
    let CssLonghandValueRef::FlexBasis(basis) = omitted[2].ordinary_value().unwrap().view() else {
        panic!("synthetic basis")
    };
    let CssFlexBasisRef::Size(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(zero))) =
        basis.view()
    else {
        panic!("omitted basis is unitless zero")
    };
    assert_eq!(
        zero.literal_component().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
}

#[test]
fn flex_serialization_uses_one_shared_resource_budget() {
    let value = flex("calc(1 + 2) 3 calc-size(content, size)");
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(1, 100, 1000),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 1, 1000),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 100, 8),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert!(
        value
            .serialize_specified()
            .unwrap()
            .contains("calc-size(content, size)")
    );
}

#[test]
fn global_and_pending_flex_declarations_keep_all_three_contributions() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let report = parse_style_attribute(&format!("flex:{text}!important"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let CssExpansion::Contributions(CssContributions::Longhands(globals)) =
            expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("global flex: {text}")
        };
        assert_eq!(globals.items().len(), 3);
        for item in globals.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&report.syntax()[0]));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
    }
    let report = parse_style_attribute("flex:var(--f)!important");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssExpansion::Pending(pending) = expand_declaration(&report.syntax()[0]).unwrap() else {
        panic!("pending flex")
    };
    assert!(matches!(
        pending
            .reenter(parse_component_values("2 content 3").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let replacement = parse_component_values("10px 2 3").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("reentered flex")
    };
    assert_eq!(values.items().len(), 3);
    for item in values.items() {
        assert!(item.source().same_occurrence(&report.syntax()[0]));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}

#[test]
fn normalization_retains_valid_flex_contributions_around_a_dropped_declaration() {
    let report = parse_sheet(".a{flex:2;flex:2 content 3;flex:none!important}");
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 2);
    for (index, declaration) in declarations.into_iter().enumerate() {
        assert_eq!(declaration.order(), index);
        assert_eq!(
            declaration.source().importance(),
            if index == 0 {
                CssImportance::Normal
            } else {
                CssImportance::Important
            }
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("normalized flex contributions")
        };
        assert_eq!(values.items().len(), 3);
        assert!(
            values
                .items()
                .iter()
                .all(|item| item.source().same_occurrence(declaration.source()))
        );
    }
}

#[test]
fn normalization_preserves_mixed_flex_occurrence_order_and_rejects_member_limit() {
    let report = parse_sheet(".a{flex-grow:4;flex:2;flex-basis:10px}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, expected) in [1, 3, 1].into_iter().enumerate() {
        assert_eq!(declarations[index].order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declarations[index].expansion()
        else {
            panic!("ordered flex contributions")
        };
        assert_eq!(values.items().len(), expected);
        assert!(
            values
                .items()
                .iter()
                .all(|item| item.source().same_occurrence(declarations[index].source()))
        );
    }

    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 3).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3
        }
    ));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::Flex
    );
}
