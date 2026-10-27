//! Specified text-combine counts retain omission and defer computed math semantics.
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::CssImportance;
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::numeric::{CssCalculation, CssInteger};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef,
    CssLonghandValueRef, CssPropertyKindRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::text::{CssTextCombineDigitCount, CssTextCombineUpright};

fn parsed(text: &str) -> CssTextCombineUpright {
    let report = CssParser::default().parse_block_contents(format!("text-combine-upright:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::TextCombineUpright(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("text-combine-upright")
    };
    value.value().clone()
}
fn output(value: &CssTextCombineUpright, expected: &str) {
    let before = value.clone();
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        expected
    );
    assert_eq!(
        parsed(expected)
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(65_536, 262_144, expected.len()))
            .unwrap(),
        expected
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(
                65_536,
                262_144,
                expected.len() - 1
            ))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(value, &before);
}

#[test]
fn checked_counts_reject_invalid_literals_and_preserve_explicit_omission() {
    for n in [i32::MIN, -1, 0, 1, 5, i32::MAX] {
        assert!(CssTextCombineDigitCount::try_literal(n).is_none());
    }
    for n in [2, 3, 4] {
        let count = CssTextCombineDigitCount::try_literal(n).unwrap();
        assert_eq!(count.literal(), Some(n));
        assert!(count.calculation().is_none());
        let value = CssTextCombineUpright::Digits(Some(count));
        output(&value, &format!("digits {n}"));
        assert_eq!(parsed(&format!("digits +000{n}")), value);
    }
    for (value, text) in [
        (CssTextCombineUpright::None, "none"),
        (CssTextCombineUpright::All, "all"),
        (CssTextCombineUpright::Digits(None), "digits"),
    ] {
        output(&value, text);
    }
    assert_eq!(parsed("digits"), CssTextCombineUpright::Digits(None));
    assert_ne!(parsed("digits"), parsed("digits 2"));
}

#[test]
fn integer_calculation_counts_remain_unrounded_unclamped_and_owned() {
    for (input, expected) in [
        ("calc(1)", "digits calc(1)"),
        ("calc(8)", "digits calc(8)"),
        ("calc(2.5)", "digits calc(2.5)"),
        ("calc(1 + 2)", "digits calc(3)"),
        ("min(1,5)", "digits calc(1)"),
        ("calc(infinity)", "digits calc(infinity)"),
        ("calc(NaN)", "digits calc(NaN)"),
        ("calc(1px / 1em)", "digits calc(1px / 1em)"),
    ] {
        let components = CssComponentValues::parse(&CssParser::default(), input).unwrap();
        let calculation = CssCalculation::<CssInteger>::try_from_components(
            components.clone(),
            Default::default(),
        )
        .unwrap();
        let count = CssTextCombineDigitCount::from_calculation(calculation.clone());
        assert!(count.literal().is_none());
        assert_eq!(count.calculation().unwrap(), &calculation);
        assert_eq!(count.calculation().unwrap().components(), &components);
        output(&CssTextCombineUpright::Digits(Some(count)), expected);
        let value = parsed(&format!("digits {input}"));
        let CssTextCombineUpright::Digits(Some(ref count)) = value else {
            panic!("explicit count")
        };
        assert!(count.literal().is_none());
        assert!(matches!(
            count.calculation().unwrap().origin(),
            CssValueOrigin::Parsed(_)
        ));
        output(&value, expected);
    }
    let count =
        CssTextCombineDigitCount::from_calculation(CssCalculation::<CssInteger>::literal(1));
    assert!(matches!(
        count.calculation().unwrap().origin(),
        CssValueOrigin::Programmatic
    ));
    output(
        &CssTextCombineUpright::Digits(Some(count)),
        "digits calc(1)",
    );
}

#[test]
fn keyword_and_literal_budgets_charge_outer_and_child_nodes_atomically() {
    for (value, input_nodes, projection_nodes, text) in [
        (CssTextCombineUpright::None, 1, 1, "none"),
        (CssTextCombineUpright::All, 1, 1, "all"),
        (CssTextCombineUpright::Digits(None), 1, 1, "digits"),
        (
            CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(2).unwrap())),
            2,
            2,
            "digits 2",
        ),
    ] {
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(
                    input_nodes,
                    projection_nodes,
                    text.len()
                ))
                .unwrap(),
            text
        );
        for (limits, kind) in [
            (
                CssSerializationLimits::new(input_nodes - 1, projection_nodes, text.len()),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(input_nodes, projection_nodes - 1, text.len()),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(input_nodes, projection_nodes, text.len() - 1),
                SerializationFailure::ByteLimit,
            ),
            (
                CssSerializationLimits::new(0, 0, 0),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 0, 0),
                SerializationFailure::ProjectionNodeLimit,
            ),
        ] {
            assert_eq!(
                serialization_failure(&value.serialize_specified(limits).unwrap_err()),
                kind
            );
        }
        assert_eq!(value, before);
    }
    let value =
        CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(2).unwrap()));
    // The outer prefix is checked before entering the child's residual budgets.
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(1, 1, 6))
            .unwrap_err()
            .is_byte_limit()
    );
}

#[test]
fn metadata_initial_and_reentry_keep_exact_current_payloads() {
    let property = CssKnownProperty::TextCombineUpright;
    let metadata = property.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert!(longhand.inherited_by_default());
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::TextCombineUpright(value) = initial.view() else {
        panic!("text combine")
    };
    assert_eq!(value, &CssTextCombineUpright::None);
    let report =
        CssParser::default().parse_block_contents("text-combine-upright:var(--count)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("pending")
    };
    for (text, expected) in [
        ("digits", CssTextCombineUpright::Digits(None)),
        (
            "digits 4",
            CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(4).unwrap())),
        ),
    ] {
        let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(components.clone()).unwrap()
        else {
            panic!("longhands")
        };
        let [value] = values.items() else {
            panic!("one")
        };
        let CssLonghandValueRef::TextCombineUpright(actual) =
            value.ordinary_value().unwrap().view()
        else {
            panic!("text combine")
        };
        assert_eq!(actual, &expected);
        assert_eq!(value.replacement_components(), Some(&components));
        assert!(value.source().same_occurrence(source));
        assert_eq!(value.source().importance(), CssImportance::Important);
    }
}

#[test]
fn calculation_leaf_budgets_include_outer_nodes_and_prefix_bytes() {
    let value = CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::from_calculation(
        CssCalculation::<CssInteger>::literal(1),
    )));
    let before = value.clone();
    let text = "digits calc(1)";
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(2, 2, text.len()))
            .unwrap(),
        text
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(1, 2, text.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 1, text.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 2, text.len() - 1),
            SerializationFailure::ByteLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, 6),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    assert_eq!(value, before);
}

#[test]
fn recovered_counts_normalize_in_order_with_exact_contribution_budget() {
    let text = ".a{text-combine-upright:digits;text-combine-upright:digits 1;text-combine-upright:digits 4!important}";
    let report = CssParser::default().parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(
        CssParser::default()
            .parse_sheet(text)
            .into_validation_result()
            .is_err()
    );
    let sheet = report
        .syntax()
        .normalize(CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap())
        .unwrap();
    let items: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 2);
    for (index, (item, expected)) in items
        .iter()
        .zip([
            CssTextCombineUpright::Digits(None),
            CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(4).unwrap())),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary")
        };
        let [value] = values.items() else {
            panic!("one")
        };
        let CssLonghandValueRef::TextCombineUpright(actual) =
            value.ordinary_value().unwrap().view()
        else {
            panic!("combine")
        };
        assert_eq!(actual, &expected);
        assert!(value.source().same_occurrence(item.source()));
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Normal
            } else {
                CssImportance::Important
            }
        );
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 0, 2, 2).unwrap(),
            CssNormalizationResource::Rules,
            0,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 1, 2).unwrap(),
            CssNormalizationResource::Declarations,
            1,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 2, 1).unwrap(),
            CssNormalizationResource::Contributions,
            1,
        ),
    ] {
        assert_eq!(
            report
                .syntax()
                .normalize(limits)
                .unwrap_err()
                .exhausted_limit(),
            Some((resource, limit))
        );
    }
}
