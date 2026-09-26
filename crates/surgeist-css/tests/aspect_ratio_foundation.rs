#![forbid(unsafe_code)]
//! CSS Sizing 4 §4.1: `auto || <ratio>`; CSS Values 4 §5.7: a ratio is
//! `<number [0,∞]> [ / <number [0,∞]> ]?` and serializes both components.
//! The expected spellings below come from those grammars, not parser output.

use surgeist_css::{
    CssAspectRatio, CssAspectRatioValue, CssComponentValue, CssContributionValueRef,
    CssContributions, CssExpansion, CssExpansionErrorKind, CssGlobalKeyword, CssImportance,
    CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLonghandValueRef,
    CssNormalizedItem, CssNumberCalculation, CssPropertyKindRef, CssPropertyNameRef,
    CssRatioOperand, CssRecoveryAction, CssSpecifiedRatio, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, expand_declaration, normalize_sheet,
    parse_component_values, parse_property_value, parse_sheet, parse_style_attribute,
};

fn parsed(value: &str) -> surgeist_css::CssDeclaration {
    let source = format!("aspect-ratio:{value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{value}");
    let declaration = report.syntax()[0].clone();
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected an ordinary aspect-ratio: {value}")
    };
    assert_eq!(authored.as_css(), value);
    declaration
}

fn checked(value: &str) -> surgeist_css::CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::AspectRatio),
        components,
        CssImportance::Normal,
    )
    .unwrap_or_else(|error| panic!("checked aspect-ratio {value}: {error:?}"));
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected checked aspect-ratio: {value}")
    };
    assert_eq!(authored.as_css(), value);
    declaration
}

fn assert_no_i01_projection(declaration: &surgeist_css::CssDeclaration, value: &str) {
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected aspect-ratio: {value}")
    };
    assert!(authored.i01_subset().is_none(), "{value}");
}

#[test]
fn preferred_ratio_accepts_auto_optional_pair_and_both_keyword_orders() {
    for value in [
        "auto",
        "16/9",
        "auto 16/9",
        "16/9 auto",
        "AUTO 16 / 9",
        "16 / 9 AUTO",
    ] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
    let sheet = parse_sheet(".x{aspect-ratio:auto 16/9}");
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
}

#[test]
fn preferred_ratio_accepts_zero_and_exact_extreme_number_components() {
    for value in [
        "0",
        "0/0",
        "0/9",
        "16/0",
        "-0/2",
        "1e999",
        "1e-999",
        "0.1",
        "16777217",
        "1e999 / 1e-999",
        "auto 1e999 / 1e-999",
    ] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
}

#[test]
fn preferred_ratio_accepts_numeric_math_in_each_component() {
    for value in [
        "calc(8 * 2) / 9",
        "16 / calc(3 * 3)",
        "auto calc(8 * 2) / calc(3 * 3)",
        "min(16, 9) / max(3, 2) auto",
        "calc(-1)",
    ] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
}

#[test]
fn only_a_positive_ordinary_number_has_the_frozen_i01_projection() {
    for value in ["1.5", "16"] {
        for declaration in [parsed(value), checked(value)] {
            let CssKnownPropertyValueRef::AspectRatio(authored) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("expected aspect-ratio: {value}")
            };
            let expected = CssAspectRatio::try_new(value.parse().unwrap()).unwrap();
            assert_eq!(authored.i01_subset(), Some(&expected));
        }
    }
    for value in ["1.5/1", "auto 1.5", "calc(1.5)"] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
}

#[test]
fn malformed_ratio_drops_only_its_declaration() {
    for value in [
        "-1e-999",
        "-1e-999 / 2",
        "2 / -1e-999",
        "auto auto",
        "auto 16/9 auto",
        "16/9/2",
        "/9",
        "16/",
        "auto / 9",
        "16 / auto",
        "calc(1px) / 2",
    ] {
        let source = format!("color:red;aspect-ratio:{value};width:1px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.diagnostics().len(), 1, "{value}: {report:?}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{value}"
        );
        let kept: Vec<_> = report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property())
            .collect();
        assert_eq!(
            kept,
            [CssKnownProperty::Color, CssKnownProperty::Width],
            "{value}"
        );
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::AspectRatio),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        );
        assert!(checked.is_err(), "checked invalid ratio: {value}");
    }
}

#[test]
fn specified_serialization_keeps_both_components_and_canonical_auto_order() {
    let huge = format!("1{}", "0".repeat(999));
    let tiny = format!("0.{}1", "0".repeat(998));
    let huge_expected = format!("{huge} / 1");
    let tiny_expected = format!("{tiny} / 1");
    for (authored, expected) in [
        ("auto", "auto"),
        ("16", "16 / 1"),
        ("16/9", "16 / 9"),
        ("16/9 auto", "auto 16 / 9"),
        ("0/0", "0 / 0"),
        ("1/0", "1 / 0"),
        ("auto 1.5", "auto 1.5 / 1"),
        ("1e999", huge_expected.as_str()),
        ("1e-999", tiny_expected.as_str()),
        ("0.1", "0.1 / 1"),
        ("16777217", "16777217 / 1"),
        ("calc(-1)", "calc(-1) / 1"),
        ("calc(8 * 2) / calc(3 * 3)", "calc(16) / calc(9)"),
        ("calc(1em / 1px) / 2", "calc(1em / 1px) / 2"),
    ] {
        let declaration = parsed(authored);
        let CssKnownPropertyValueRef::AspectRatio(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("aspect-ratio: {authored}")
        };
        assert_eq!(
            value.ratio().serialize_specified().unwrap(),
            expected,
            "{authored}"
        );
    }
}

#[test]
fn checked_operands_cannot_hide_invalid_literals_in_typed_calculations() {
    for text in ["-1e-999", "-1", "2px", "auto"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        assert!(
            CssRatioOperand::try_from_component(component).is_err(),
            "{text}"
        );
    }
    let exact =
        CssRatioOperand::try_from_component(CssComponentValue::try_number("1e999").unwrap())
            .unwrap();
    assert!(matches!(exact.origin(), CssValueOrigin::Programmatic));
    assert!(exact.literal_component().is_some());
    assert!(exact.calculation().is_none());
    let bare_negative =
        CssNumberCalculation::try_from_components(parse_component_values("-1e-999").unwrap())
            .unwrap();
    assert!(CssRatioOperand::try_from_calculation(bare_negative).is_err());
    let math =
        CssNumberCalculation::try_from_components(parse_component_values("calc(-1)").unwrap())
            .unwrap();
    let deferred = CssRatioOperand::try_from_calculation(math).unwrap();
    assert!(deferred.calculation().is_some());
    let ratio = CssSpecifiedRatio::new(exact, Some(deferred));
    assert!(ratio.numerator().literal_component().is_some());
    assert!(ratio.denominator().unwrap().calculation().is_some());
    assert_eq!(
        ratio.serialize_specified().unwrap(),
        format!("1{} / calc(-1)", "0".repeat(999))
    );
    let source = parsed("16/9");
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed ratio")
    };
    let CssAspectRatioValue::Ratio(parsed_ratio) = authored.ratio() else {
        panic!("ratio pair")
    };
    assert!(matches!(
        parsed_ratio.numerator().origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert!(matches!(
        parsed_ratio.denominator().unwrap().origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn ratio_serialization_charges_one_budget_across_auto_and_both_operands() {
    let declaration = parsed("auto 16/9");
    let CssKnownPropertyValueRef::AspectRatio(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("aspect-ratio")
    };
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 11),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 11),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 10),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .ratio()
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
    assert_eq!(
        value
            .ratio()
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 11))
            .unwrap(),
        "auto 16 / 9"
    );
    let math_pair = parsed("calc(8 * 2)/9");
    let CssKnownPropertyValueRef::AspectRatio(math) =
        math_pair.known().unwrap().property_value().unwrap()
    else {
        panic!("math ratio")
    };
    let full = math.ratio().serialize_specified().unwrap();
    assert_eq!(full, "calc(16) / 9");
    assert_eq!(
        math.ratio()
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                full.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn aspect_ratio_is_a_noninherited_terminal_with_auto_initial() {
    let metadata = CssKnownProperty::AspectRatio.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("aspect-ratio terminal longhand")
    };
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("aspect-ratio initial")
    };
    let CssLonghandValueRef::AspectRatio(CssAspectRatioValue::Auto) = initial.view() else {
        panic!("auto initial")
    };
    let source = parsed("16/9");
    let CssExpansion::Contributions(_) = expand_declaration(&source).unwrap() else {
        panic!("aspect-ratio terminal contribution")
    };
}

#[test]
fn ratio_expansion_retains_typed_payload_importance_and_source_occurrence() {
    let report = parse_style_attribute("aspect-ratio:auto 16/9!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("terminal contribution")
    };
    let [value] = values.items() else {
        panic!("one terminal")
    };
    assert_eq!(value.property(), CssKnownProperty::AspectRatio);
    assert_eq!(value.source().importance(), CssImportance::Important);
    assert!(value.source().same_occurrence(source));
    assert!(value.replacement_components().is_none());
    let CssLonghandValueRef::AspectRatio(CssAspectRatioValue::AutoRatio(ratio)) =
        value.ordinary_value().unwrap().view()
    else {
        panic!("typed auto pair")
    };
    assert_eq!(ratio.serialize_specified().unwrap(), "16 / 9");
    for (keyword, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let global = parse_style_attribute(&format!("aspect-ratio:{keyword}!important"));
        assert!(global.is_clean());
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&global.syntax()[0]).unwrap()
        else {
            panic!("global longhand")
        };
        let [item] = items.items() else {
            panic!("one global terminal")
        };
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_ratio_reentry_checks_full_grammar_and_retains_source() {
    let report = parse_style_attribute("aspect-ratio:var(--preferred)!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("pending ratio")
    };
    for text in ["-1e-999/2", "16/9/2", "auto auto", "16/"] {
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
    let replacement = parse_component_values("16/9 auto").unwrap();
    let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("reentered ratio")
    };
    let [item] = items.items() else {
        panic!("one terminal")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssLonghandValueRef::AspectRatio(CssAspectRatioValue::AutoRatio(ratio)) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("typed reentered ratio")
    };
    assert_eq!(ratio.serialize_specified().unwrap(), "16 / 9");
    let CssContributions::Longhands(globals) = handle
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("global reentry")
    };
    assert_eq!(
        globals.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn normalization_orders_surviving_ratio_occurrences_after_recovery() {
    let report = parse_sheet(".a{aspect-ratio:16/9;aspect-ratio:-1;aspect-ratio:auto!important}");
    assert_eq!(report.diagnostics().len(), 1);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    for (index, expected) in ["16 / 9", "auto"].into_iter().enumerate() {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Normal
            } else {
                CssImportance::Important
            }
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("normalized ratio terminal")
        };
        let [value] = values.items() else {
            panic!("one ratio terminal")
        };
        assert_eq!(value.property(), CssKnownProperty::AspectRatio);
        assert!(value.source().same_occurrence(item.source()));
        let CssLonghandValueRef::AspectRatio(ratio) = value.ordinary_value().unwrap().view() else {
            panic!("normalized typed ratio")
        };
        assert_eq!(ratio.serialize_specified().unwrap(), expected);
    }
}
