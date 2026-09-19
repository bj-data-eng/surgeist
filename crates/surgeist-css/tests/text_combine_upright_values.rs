#![forbid(unsafe_code)]
//! Specified text-combine counts retain omission and defer computed math semantics.
use surgeist_css::*;

fn parsed(text: &str) -> CssTextCombineUpright {
    let report = parse_style_attribute(&format!("text-combine-upright:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::TextCombineUpright(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("text-combine-upright")
    };
    value.combine().clone()
}
fn output(value: &CssTextCombineUpright, expected: &str) {
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(parsed(expected).serialize_specified().unwrap(), expected);
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                65_536,
                262_144,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                65_536,
                262_144,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
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
        let components = parse_component_values(input).unwrap();
        let calculation = CssIntegerCalculation::try_from_components(components.clone()).unwrap();
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
    let count = CssTextCombineDigitCount::from_calculation(CssIntegerCalculation::literal(1));
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
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input_nodes,
                    projection_nodes,
                    text.len()
                ))
                .unwrap(),
            text
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(
                    input_nodes - 1,
                    projection_nodes,
                    text.len(),
                ),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(
                    input_nodes,
                    projection_nodes - 1,
                    text.len(),
                ),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(
                    input_nodes,
                    projection_nodes,
                    text.len() - 1,
                ),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 0),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
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
        assert_eq!(value, before);
    }
    let value =
        CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(2).unwrap()));
    // The outer prefix is checked before entering the child's residual budgets.
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
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
    let report = parse_style_attribute("text-combine-upright:var(--count)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("pending")
    };
    for (text, expected) in [
        ("digits", CssTextCombineUpright::Digits(None)),
        (
            "digits 4",
            CssTextCombineUpright::Digits(Some(CssTextCombineDigitCount::try_literal(4).unwrap())),
        ),
    ] {
        let components = parse_component_values(text).unwrap();
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
        CssIntegerCalculation::literal(1),
    )));
    let before = value.clone();
    let text = "digits calc(1)";
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                2,
                2,
                text.len()
            ))
            .unwrap(),
        text
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(1, 2, text.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 1, text.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 2, text.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 6),
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
    assert_eq!(value, before);
}

#[test]
fn recovered_counts_normalize_in_order_with_exact_contribution_budget() {
    let text = ".a{text-combine-upright:digits;text-combine-upright:digits 1;text-combine-upright:digits 4!important}";
    let report = parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(text).is_err());
    let sheet = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap(),
    )
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
            normalize_sheet_with_limits(report.syntax(), limits)
                .unwrap_err()
                .kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
    }
}
