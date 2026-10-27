//! CSS Sizing 4 §4.1: `auto || <ratio>`; CSS Values 4 §5.7: a ratio is
//! `<number [0,∞]> [ / <number [0,∞]> ]?` and serializes both components.
//! The expected spellings below come from those grammars, not parser output.

use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::{
    assert_checked_rejected, assert_component_rejected, assert_invalid_value,
    assert_numeric_rejected,
};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::CssNormalizedItem;
use surgeist_css::numeric::{
    CssCalculation, CssNonNegative, CssNumber, CssNumeric, CssSpecifiedRatio,
};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandValueRef, CssPropertyKindRef, CssPropertyNameRef,
    CssPropertyTarget,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::box_model::CssAspectRatioValue;

fn parsed(value: &str) -> surgeist_css::declaration::CssDeclaration {
    let source = format!("aspect-ratio:{value}");
    let report = CssParser::default().parse_block_contents(&source);
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

fn checked(value: &str) -> surgeist_css::declaration::CssDeclaration {
    let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
    let declaration = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::AspectRatio)),
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

fn assert_authored_ratio(declaration: &surgeist_css::declaration::CssDeclaration, value: &str) {
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected aspect-ratio: {value}")
    };
    assert_eq!(authored.as_css(), value);
    let ratio = match authored.value() {
        CssAspectRatioValue::Auto => {
            assert!(value.eq_ignore_ascii_case("auto"));
            return;
        }
        CssAspectRatioValue::Ratio(ratio) => {
            assert!(!value.to_ascii_lowercase().contains("auto"));
            ratio
        }
        CssAspectRatioValue::AutoRatio(ratio) => {
            assert!(value.to_ascii_lowercase().contains("auto"));
            ratio
        }
        _ => panic!("selected ratio domain"),
    };
    assert_eq!(ratio.denominator().is_some(), value.contains('/'));
    let numeric_text = value
        .trim()
        .trim_start_matches("auto ")
        .trim_start_matches("AUTO ")
        .trim_end_matches(" auto")
        .trim_end_matches(" AUTO");
    for (operand, spelling) in std::iter::once((
        ratio.numerator(),
        numeric_text.split('/').next().unwrap().trim(),
    ))
    .chain(
        ratio
            .denominator()
            .map(|operand| (operand, numeric_text.split('/').nth(1).unwrap().trim())),
    ) {
        if let Some(component) = operand.literal().map(|literal| literal.component()) {
            assert!(
                matches!(component.view(), surgeist_css::component::CssComponentValueRef::Token(surgeist_css::component::CssValueTokenRef::Number(number)) if number.representation() == spelling)
            );
        } else {
            assert!(
                operand.calculation().is_some(),
                "symbolic operand: {spelling}"
            );
        }
    }
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
        assert_authored_ratio(&parsed(value), value);
        assert_authored_ratio(&checked(value), value);
    }
    let sheet = CssParser::default().parse_sheet(".x{aspect-ratio:auto 16/9}");
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
        assert_authored_ratio(&parsed(value), value);
        assert_authored_ratio(&checked(value), value);
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
        assert_authored_ratio(&parsed(value), value);
        assert_authored_ratio(&checked(value), value);
    }
}

#[test]
fn ordinary_numbers_keep_their_authored_single_operand() {
    for value in ["1.5", "16"] {
        for declaration in [parsed(value), checked(value)] {
            let CssKnownPropertyValueRef::AspectRatio(authored) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("expected aspect-ratio: {value}")
            };
            let CssAspectRatioValue::Ratio(ratio) = authored.value() else {
                panic!("ratio")
            };
            assert!(ratio.denominator().is_none());
            assert!(
                matches!(ratio.numerator().literal().map(|literal| literal.component()).unwrap().view(), surgeist_css::component::CssComponentValueRef::Token(surgeist_css::component::CssValueTokenRef::Number(number)) if number.representation() == value)
            );
        }
    }
    for value in ["1.5/1", "auto 1.5", "calc(1.5)"] {
        assert_authored_ratio(&parsed(value), value);
        assert_authored_ratio(&checked(value), value);
    }
}

#[test]
fn malformed_ratio_drops_only_its_declaration() {
    for (value, token) in [
        ("-1e-999", Some("-1e-999")),
        ("-1e-999 / 2", Some("-1e-999")),
        ("2 / -1e-999", Some("-1e-999")),
        ("auto auto", Some("auto")),
        ("auto 16/9 auto", Some("auto")),
        ("16/9/2", Some("/")),
        ("/9", Some("/")),
        ("16/", Some("/")),
        ("auto / 9", Some("/")),
        ("16 / auto", Some("auto")),
        ("calc(1px) / 2", Some("calc(")),
    ] {
        let source = format!("color:red;aspect-ratio:{value};width:1px");
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(report.diagnostics().len(), 1, "{value}: {report:?}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{value}"
        );
        assert_invalid_value(
            report.diagnostics()[0].error(),
            CssKnownProperty::AspectRatio,
            token,
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
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::AspectRatio)),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        );
        assert_checked_rejected(checked, CssKnownProperty::AspectRatio, token);
    }
}

#[test]
fn specified_serialization_keeps_both_components_and_canonical_auto_order() {
    let huge = format!("1{}", "0".repeat(999));
    let huge_expected = format!("{huge} / 1");
    for (authored, expected) in [
        ("auto", "auto"),
        ("16", "16 / 1"),
        ("16/9", "16 / 9"),
        ("16/9 auto", "auto 16 / 9"),
        ("0/0", "0 / 0"),
        ("1/0", "1 / 0"),
        ("auto 1.5", "auto 1.5 / 1"),
        ("1e999", huge_expected.as_str()),
        ("1e-999", "0 / 1"),
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
            value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{authored}"
        );
    }
}

#[test]
fn checked_operands_cannot_hide_invalid_literals_in_typed_calculations() {
    for (text, kind) in [
        ("-1e-999", NumericFailure::OutOfRange),
        ("-1", NumericFailure::OutOfRange),
        ("2px", NumericFailure::RootDomainMismatch),
        ("auto", NumericFailure::RootDomainMismatch),
    ] {
        let component = CssComponentValues::parse(&CssParser::default(), text)
            .unwrap()
            .items()[0]
            .clone();
        assert_component_rejected(
            CssNumeric::<CssNumber, CssNonNegative>::try_from_component,
            component,
            kind,
        );
    }
    let exact = CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
        CssComponentValue::try_number("1e999").unwrap(),
    )
    .unwrap();
    assert!(matches!(exact.origin(), CssValueOrigin::Programmatic));
    assert!(exact.literal().map(|literal| literal.component()).is_some());
    assert!(exact.calculation().is_none());
    let bare_negative = CssCalculation::<CssNumber>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "-1e-999").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert_numeric_rejected(
        CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(bare_negative),
        NumericFailure::OutOfRange,
    );
    let math = CssCalculation::<CssNumber>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "calc(-1)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let deferred = CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(math).unwrap();
    assert!(deferred.calculation().is_some());
    let ratio = CssSpecifiedRatio::new(exact, Some(deferred));
    assert!(
        ratio
            .numerator()
            .literal()
            .map(|literal| literal.component())
            .is_some()
    );
    assert!(ratio.denominator().unwrap().calculation().is_some());
    assert_eq!(
        ratio.serialize_specified(Default::default()).unwrap(),
        format!("1{} / calc(-1)", "0".repeat(999))
    );
    let source = parsed("16/9");
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed ratio")
    };
    let CssAspectRatioValue::Ratio(parsed_ratio) = authored.value() else {
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
            CssSerializationLimits::new(2, 3, 11),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 11),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 10),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.value().serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    assert_eq!(
        value
            .value()
            .serialize_specified(CssSerializationLimits::new(3, 3, 11))
            .unwrap(),
        "auto 16 / 9"
    );
    let math_pair = parsed("calc(8 * 2)/9");
    let CssKnownPropertyValueRef::AspectRatio(math) =
        math_pair.known().unwrap().property_value().unwrap()
    else {
        panic!("math ratio")
    };
    let full = math
        .value()
        .serialize_specified(Default::default())
        .unwrap();
    assert_eq!(full, "calc(16) / 9");
    assert!(
        math.value()
            .serialize_specified(CssSerializationLimits::new(100, 100, full.len() - 1))
            .unwrap_err()
            .is_byte_limit()
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
    let CssExpansion::Contributions(_) = source.expand().unwrap() else {
        panic!("aspect-ratio terminal contribution")
    };
}

#[test]
fn ratio_expansion_retains_typed_payload_importance_and_source_occurrence() {
    let report = CssParser::default().parse_block_contents("aspect-ratio:auto 16/9!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
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
    assert_eq!(
        ratio.serialize_specified(Default::default()).unwrap(),
        "16 / 9"
    );
    for (keyword, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let global =
            CssParser::default().parse_block_contents(format!("aspect-ratio:{keyword}!important"));
        assert!(global.is_clean());
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            global.syntax()[0].expand().unwrap()
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
    let report =
        CssParser::default().parse_block_contents("aspect-ratio:var(--preferred)!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = source.expand().unwrap() else {
        panic!("pending ratio")
    };
    for text in ["-1e-999/2", "16/9/2", "auto auto", "16/"] {
        let error = handle
            .reenter(CssComponentValues::parse(&CssParser::default(), text).unwrap())
            .unwrap_err();
        assert!(error.replacement_error().is_some(), "{text}");
    }
    assert!(
        handle
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "16/9 auto").unwrap();
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
    assert_eq!(
        ratio.serialize_specified(Default::default()).unwrap(),
        "16 / 9"
    );
    let CssContributions::Longhands(globals) = handle
        .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
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
    let report = CssParser::default()
        .parse_sheet(".a{aspect-ratio:16/9;aspect-ratio:-1;aspect-ratio:auto!important}");
    assert_eq!(report.diagnostics().len(), 1);
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
        assert_eq!(
            ratio.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
}
