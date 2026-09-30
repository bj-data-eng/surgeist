#![forbid(unsafe_code)]
//! Shared nonnegative scalar consumers, checked through functional public APIs.
//! Exact retention and equality are Surgeist contracts. Genuine math stays
//! symbolic; ordinary nonnegative admission is not deferred range checking.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-images
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#supported-filter-functions
//! https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#animation-iteration-count

use surgeist_css::*;

fn number(text: &str) -> CssSpecifiedNonNegativeNumber {
    CssSpecifiedNonNegativeNumber::try_from_component(CssComponentValue::try_number(text).unwrap())
        .unwrap()
}
fn percentage(text: &str) -> CssSpecifiedNonNegativePercentage {
    CssSpecifiedNonNegativePercentage::try_from_component(
        CssComponentValue::try_token(text).unwrap(),
    )
    .unwrap()
}
fn number_math(text: &str) -> CssSpecifiedNonNegativeNumber {
    CssSpecifiedNonNegativeNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values(text).unwrap()).unwrap(),
    )
    .unwrap()
}
fn percentage_math(text: &str) -> CssSpecifiedNonNegativePercentage {
    CssSpecifiedNonNegativePercentage::try_from_calculation(
        CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap(),
    )
    .unwrap()
}
fn token(text: &str) -> CssComponentValue {
    parse_component_values(text)
        .unwrap()
        .items()
        .iter()
        .find(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .unwrap()
        .clone()
}
fn number_text(value: &CssSpecifiedNonNegativeNumber) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
        value.literal_component().unwrap().view()
    else {
        panic!("number")
    };
    number.representation()
}
fn percentage_text(value: &CssSpecifiedNonNegativePercentage) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) =
        value.literal_component().unwrap().view()
    else {
        panic!("percentage")
    };
    number.representation()
}

#[test]
fn shared_nonnegative_scalars_keep_exact_huge_tiny_positive_and_signed_zero_lexemes() {
    for (text, kind) in [
        ("1e999", CssNumericTokenKind::Number),
        ("1e-999", CssNumericTokenKind::Number),
        ("+01.250", CssNumericTokenKind::Number),
        ("-0", CssNumericTokenKind::Integer),
        (
            "-0e9999999999999999999999999999999999999999",
            CssNumericTokenKind::Number,
        ),
        (
            "-0e-9999999999999999999999999999999999999999",
            CssNumericTokenKind::Number,
        ),
    ] {
        let n = number(text);
        let p = percentage(&format!("{text}%"));
        assert_eq!(number_text(&n), text);
        assert_eq!(percentage_text(&p), text);
        assert!(
            matches!(n.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.kind() == kind)
        );
        assert!(
            matches!(p.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) if number.kind() == kind)
        );
        assert_eq!(n.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(p.origin(), &CssValueOrigin::Programmatic);
    }
    assert_eq!(percentage("250%").serialize_specified().unwrap(), "250%");
}

#[test]
fn ordinary_tiny_negatives_and_wrong_domains_return_exact_errors_with_origins() {
    for text in ["-1e-999", "-0.00000000000000000001", "-1"] {
        let input = token(&format!("  {text}"));
        let origin = input.origin().clone();
        let error = CssSpecifiedNonNegativeNumber::try_from_component(input).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(&origin));
        let input = token(&format!("  {text}%"));
        let origin = input.origin().clone();
        let error = CssSpecifiedNonNegativePercentage::try_from_component(input).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(&origin));
    }
    for text in ["1px", "1deg", "auto"] {
        assert_eq!(
            CssSpecifiedNonNegativeNumber::try_from_component(token(text))
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(
            CssSpecifiedNonNegativePercentage::try_from_component(token(text))
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
    }
    assert!(CssSpecifiedNonNegativeNumber::try_from_component(token("1%")).is_err());
    assert!(CssSpecifiedNonNegativePercentage::try_from_component(token("0")).is_err());
}

#[test]
fn bare_nonnegative_roots_recheck_range_and_actual_math_remains_symbolic() {
    for text in ["0", "-0", "1e999", "1e-999"] {
        assert_eq!(number_text(&number_math(text)), text);
        assert_eq!(percentage_text(&percentage_math(&format!("{text}%"))), text);
    }
    let number_root =
        CssNumberCalculation::try_from_components(parse_component_values("-1e-999").unwrap())
            .unwrap();
    assert_eq!(
        CssSpecifiedNonNegativeNumber::try_from_calculation(number_root)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange
    );
    let percentage_root =
        CssPercentageCalculation::try_from_components(parse_component_values("-1e-999%").unwrap())
            .unwrap();
    assert_eq!(
        CssSpecifiedNonNegativePercentage::try_from_calculation(percentage_root)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange
    );
    let n = number_math("calc(-1)");
    let p = percentage_math("calc(-1%)");
    assert!(n.literal_component().is_none());
    assert!(p.literal_component().is_none());
    assert_eq!(
        n.calculation().unwrap().result_type(),
        CssCalculationType::Number
    );
    assert_eq!(
        p.calculation().unwrap().result_type(),
        CssCalculationType::Percentage
    );
    assert_eq!(n.serialize_specified().unwrap(), "calc(-1)");
    assert_eq!(p.serialize_specified().unwrap(), "calc(-1%)");
    assert_eq!(
        n.calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(-1)"
    );
    assert_eq!(
        p.calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(-1%)"
    );
    let CssComponentValueRef::Function(function) =
        n.calculation().unwrap().components().items()[0].view()
    else {
        panic!("number function root");
    };
    assert_eq!(function.name(), "calc");
    assert!(
        matches!(function.values().items(), [component] if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == "-1"))
    );
}

#[test]
fn direct_number_provenance_and_established_percentage_equality_remain_distinct() {
    let n = CssSpecifiedNonNegativeNumber::try_from_component(token("  1.250")).unwrap();
    let p = CssSpecifiedNonNegativePercentage::try_from_component(token("  1.250%")).unwrap();
    let CssValueOrigin::Parsed(origin) = n.origin() else {
        panic!("parsed number origin")
    };
    assert_eq!(origin.source().as_str(), "  1.250");
    assert_eq!(origin.span().start().byte_offset().value(), 2);
    assert_ne!(n, number("1.250"));
    assert_eq!(p, percentage("1.250%"));
    assert_ne!(p, percentage("1.25%"));
    let a = number_math("calc(1 + 2)");
    let b = number_math("  calc(1 + 2)");
    assert_ne!(a, b);
    assert_ne!(a.calculation().unwrap(), b.calculation().unwrap());
    let a = percentage_math("calc(1% + 2%)");
    let b = percentage_math("  calc(1% + 2%)");
    assert_eq!(a, b);
    assert_ne!(a.calculation().unwrap(), b.calculation().unwrap());
}

fn filter_amounts(value: &CssFilter) -> [&CssFilterAmount; 7] {
    let CssFilter::Functions(functions) = value else {
        panic!("function list")
    };
    let [
        CssFilterFunction::Brightness(a),
        CssFilterFunction::Contrast(b),
        CssFilterFunction::Grayscale(c),
        CssFilterFunction::Invert(d),
        CssFilterFunction::Opacity(e),
        CssFilterFunction::Saturate(f),
        CssFilterFunction::Sepia(g),
    ] = functions.functions()
    else {
        panic!("seven ordered filter identities")
    };
    [a, b, c, d, e, f, g]
}

#[test]
fn seven_filter_amount_roles_retain_default_number_percentage_and_actual_math() {
    let constructors: [fn(CssFilterAmount) -> CssFilterFunction; 7] = [
        CssFilterFunction::Brightness,
        CssFilterFunction::Contrast,
        CssFilterFunction::Grayscale,
        CssFilterFunction::Invert,
        CssFilterFunction::Opacity,
        CssFilterFunction::Saturate,
        CssFilterFunction::Sepia,
    ];
    for amount in [
        CssFilterAmount::Default,
        CssFilterAmount::Number(number("1e999")),
        CssFilterAmount::Percentage(percentage("250%")),
        CssFilterAmount::Number(number_math("calc(-1)")),
        CssFilterAmount::Percentage(percentage_math("calc(-1%)")),
    ] {
        let value = CssFilter::Functions(
            CssFilterFunctionList::try_new(
                constructors
                    .iter()
                    .map(|construct| construct(amount.clone()))
                    .collect(),
            )
            .unwrap(),
        );
        for actual in filter_amounts(&value) {
            assert_eq!(actual, &amount);
        }
    }
    assert!(CssFilterFunctionList::try_new(vec![]).is_none());
    assert_ne!(
        CssFilterAmount::Default,
        CssFilterAmount::Number(number("1"))
    );
    assert_ne!(
        CssFilterAmount::Number(number("1")),
        CssFilterAmount::Percentage(percentage("100%"))
    );
}

#[test]
fn filter_and_backdrop_parsers_preserve_all_amount_roles_and_exact_payloads() {
    for property in ["filter", "backdrop-filter"] {
        for amount in ["", "1e999", "250%", "calc(-1)", "calc(-1%)"] {
            let value = [
                "brightness",
                "contrast",
                "grayscale",
                "invert",
                "opacity",
                "saturate",
                "sepia",
            ]
            .map(|name| format!("{name}({amount})"))
            .join(" ");
            let source = format!("{property}: {value}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let value = match report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            {
                CssKnownPropertyValueRef::Filter(value) => value.value(),
                CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
                _ => panic!("filter property"),
            };
            for (index, value) in filter_amounts(value).into_iter().enumerate() {
                let name = [
                    "brightness",
                    "contrast",
                    "grayscale",
                    "invert",
                    "opacity",
                    "saturate",
                    "sepia",
                ][index];
                let start = source.find(&format!("{name}(")).unwrap() + name.len() + 1;
                match (amount, value) {
                    ("", CssFilterAmount::Default) => {}
                    ("1e999", CssFilterAmount::Number(value)) => {
                        assert_eq!(number_text(value), "1e999");
                        assert_origin(value.origin(), &source, start, amount.len());
                    }
                    ("250%", CssFilterAmount::Percentage(value)) => {
                        assert_eq!(percentage_text(value), "250");
                        assert_origin(value.origin(), &source, start, amount.len());
                    }
                    ("calc(-1)", CssFilterAmount::Number(value)) => {
                        let calculation = value.calculation().expect("symbolic calculation");
                        assert_eq!(
                            calculation.components().serialize().unwrap().as_css(),
                            amount
                        );
                        assert_origin(calculation.origin(), &source, start, 5);
                    }
                    ("calc(-1%)", CssFilterAmount::Percentage(value)) => {
                        let calculation = value.calculation().expect("symbolic calculation");
                        assert_eq!(
                            calculation.components().serialize().unwrap().as_css(),
                            amount
                        );
                        assert_origin(calculation.origin(), &source, start, 5);
                    }
                    _ => panic!("expected authored {amount} branch"),
                }
            }
        }
    }
}

#[test]
fn filter_aggregate_equality_ignores_numeric_origins_without_erasing_roles_or_order() {
    let parsed = CssSpecifiedNonNegativeNumber::try_from_component(token("  1")).unwrap();
    assert_eq!(
        CssFilterAmount::Number(parsed),
        CssFilterAmount::Number(number("1"))
    );
    assert_eq!(
        CssFilterAmount::Number(number_math("calc(1 + 2)")),
        CssFilterAmount::Number(number_math("  calc(1 + 2)"))
    );
    assert_ne!(
        CssFilterAmount::Number(number("1")),
        CssFilterAmount::Number(number("1.0"))
    );
    assert_ne!(
        CssFilterAmount::Number(number("1")),
        CssFilterAmount::Number(number_math("calc(1)"))
    );
    let first = CssFilterFunction::Brightness(CssFilterAmount::Default);
    let second = CssFilterFunction::Contrast(CssFilterAmount::Number(number("2")));
    assert_ne!(
        CssFilterFunctionList::try_new(vec![first.clone(), second.clone()]),
        CssFilterFunctionList::try_new(vec![second, first])
    );
}

#[test]
fn border_numeric_constructors_expand_one_to_four_edges_with_distinct_scalar_branches() {
    for (input, expected) in [
        (vec!["1"], ["1", "1", "1", "1"]),
        (vec!["1", "2"], ["1", "2", "1", "2"]),
        (vec!["1", "2", "3"], ["1", "2", "3", "2"]),
        (vec!["1", "2", "3", "4"], ["1", "2", "3", "4"]),
    ] {
        let slice = CssBorderImageSlice::try_new(
            input
                .iter()
                .map(|text| CssBorderImageSliceComponent::Number(number(text)))
                .collect(),
            false,
        )
        .unwrap();
        let width = CssBorderImageWidth::try_new(
            input
                .iter()
                .map(|text| CssBorderImageWidthComponent::Number(number(text)))
                .collect(),
        )
        .unwrap();
        let outset = CssBorderImageOutset::try_new(
            input
                .iter()
                .map(|text| CssBorderImageOutsetComponent::Number(number(text)))
                .collect(),
        )
        .unwrap();
        for (index, expected) in expected.iter().enumerate() {
            assert!(
                matches!(&slice.values()[index], CssBorderImageSliceComponent::Number(value) if number_text(value) == *expected)
            );
            assert!(
                matches!(&width.values()[index], CssBorderImageWidthComponent::Number(value) if number_text(value) == *expected)
            );
            assert!(
                matches!(&outset.values()[index], CssBorderImageOutsetComponent::Number(value) if number_text(value) == *expected)
            );
        }
    }
    for count in [0, 5] {
        assert!(
            CssBorderImageSlice::try_new(
                vec![CssBorderImageSliceComponent::Number(number("1")); count],
                false
            )
            .is_none()
        );
        assert!(
            CssBorderImageWidth::try_new(vec![
                CssBorderImageWidthComponent::Number(number("1"));
                count
            ])
            .is_none()
        );
        assert!(
            CssBorderImageOutset::try_new(vec![
                CssBorderImageOutsetComponent::Number(number("1"));
                count
            ])
            .is_none()
        );
    }
}

#[test]
fn border_slice_width_and_outset_retain_percentage_length_keyword_and_math_roles() {
    let slice = CssBorderImageSlice::try_new(
        vec![
            CssBorderImageSliceComponent::Percentage(percentage("1e999%")),
            CssBorderImageSliceComponent::Number(number_math("calc(-1)")),
        ],
        true,
    )
    .unwrap();
    assert!(slice.fill());
    assert!(
        matches!(&slice.values()[0], CssBorderImageSliceComponent::Percentage(value) if percentage_text(value) == "1e999")
    );
    assert!(
        matches!(&slice.values()[1], CssBorderImageSliceComponent::Number(value) if value.calculation().is_some())
    );
    let length_percentage = CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_token("25%").unwrap(),
    )
    .unwrap();
    let width = CssBorderImageWidth::try_new(vec![
        CssBorderImageWidthComponent::Auto,
        CssBorderImageWidthComponent::LengthPercentage(length_percentage),
        CssBorderImageWidthComponent::Number(number_math("calc(-1)")),
    ])
    .unwrap();
    assert!(matches!(
        &width.values()[0],
        CssBorderImageWidthComponent::Auto
    ));
    assert!(
        matches!(&width.values()[1], CssBorderImageWidthComponent::LengthPercentage(value) if matches!(value.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) if number.representation() == "25"))
    );
    assert!(
        matches!(&width.values()[2], CssBorderImageWidthComponent::Number(value) if value.calculation().is_some())
    );
    let length = CssSpecifiedNonNegativeLength::try_from_component(
        CssComponentValue::try_token("2px").unwrap(),
    )
    .unwrap();
    let outset = CssBorderImageOutset::try_new(vec![
        CssBorderImageOutsetComponent::Length(length),
        CssBorderImageOutsetComponent::Number(number("1e999")),
    ])
    .unwrap();
    assert!(matches!(
        &outset.values()[0],
        CssBorderImageOutsetComponent::Length(_)
    ));
    assert!(
        matches!(&outset.values()[1], CssBorderImageOutsetComponent::Number(value) if number_text(value) == "1e999")
    );
}

#[test]
fn border_shorthand_construction_preserves_coupling_omission_and_exact_structure() {
    let slice = || {
        CssBorderImageSlice::try_new(
            vec![CssBorderImageSliceComponent::Number(number("1"))],
            false,
        )
        .unwrap()
    };
    let width = || {
        CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Number(number("1"))])
            .unwrap()
    };
    let outset = || {
        CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Number(number("0"))])
            .unwrap()
    };
    assert!(CssBorderImage::try_new(None, None, None, None, None).is_none());
    assert!(CssBorderImage::try_new(None, None, Some(width()), None, None).is_none());
    assert!(CssBorderImage::try_new(None, None, None, Some(outset()), None).is_none());
    let omitted = CssBorderImage::try_new(None, Some(slice()), None, None, None).unwrap();
    assert!(omitted.width().is_none());
    assert!(omitted.outset().is_none());
    let explicit =
        CssBorderImage::try_new(None, Some(slice()), Some(width()), Some(outset()), None).unwrap();
    assert!(explicit.source().is_none());
    assert!(explicit.repeat().is_none());
    assert_ne!(omitted, explicit);
    let parsed = CssSpecifiedNonNegativeNumber::try_from_component(token("  1")).unwrap();
    assert_eq!(
        CssBorderImageSliceComponent::Number(parsed.clone()),
        CssBorderImageSliceComponent::Number(number("1"))
    );
    assert_eq!(
        CssBorderImageWidthComponent::Number(parsed.clone()),
        CssBorderImageWidthComponent::Number(number("1"))
    );
    assert_eq!(
        CssBorderImageOutsetComponent::Number(parsed),
        CssBorderImageOutsetComponent::Number(number("1"))
    );
    let a = number_math("calc(1 + 2)");
    let b = number_math("  calc(1 + 2)");
    assert_eq!(
        CssBorderImageSliceComponent::Number(a.clone()),
        CssBorderImageSliceComponent::Number(b.clone())
    );
    assert_eq!(
        CssBorderImageWidthComponent::Number(a.clone()),
        CssBorderImageWidthComponent::Number(b.clone())
    );
    assert_eq!(
        CssBorderImageOutsetComponent::Number(a),
        CssBorderImageOutsetComponent::Number(b)
    );
    assert_ne!(
        CssBorderImageSliceComponent::Number(number("1")),
        CssBorderImageSliceComponent::Percentage(percentage("1%"))
    );
    assert_ne!(
        CssBorderImageOutsetComponent::Number(number("2")),
        CssBorderImageOutsetComponent::Length(
            CssSpecifiedNonNegativeLength::try_from_component(
                CssComponentValue::try_token("2px").unwrap()
            )
            .unwrap()
        )
    );
    assert_ne!(
        CssBorderImageWidthComponent::Number(number("1")),
        CssBorderImageWidthComponent::Number(number("1.0"))
    );
    assert_ne!(
        slice(),
        CssBorderImageSlice::try_new(
            vec![CssBorderImageSliceComponent::Number(number("1"))],
            true
        )
        .unwrap()
    );
    assert_eq!(
        slice(),
        CssBorderImageSlice::try_new(
            vec![CssBorderImageSliceComponent::Number(number("1")); 4],
            false
        )
        .unwrap()
    );
    assert_ne!(
        CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Auto]),
        CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Number(number("1"))])
    );
    assert_ne!(
        CssBorderImageSlice::try_new(
            vec![
                CssBorderImageSliceComponent::Number(number("1")),
                CssBorderImageSliceComponent::Number(number("2"))
            ],
            false
        ),
        CssBorderImageSlice::try_new(
            vec![
                CssBorderImageSliceComponent::Number(number("2")),
                CssBorderImageSliceComponent::Number(number("1"))
            ],
            false
        )
    );
}

#[test]
fn border_expansion_initials_use_exact_programmatic_zero_hundred_percent_and_one() {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Border),
        parse_component_values("solid").unwrap(),
        CssImportance::Important,
    )
    .unwrap();
    let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
        expand_declaration(&declaration).unwrap()
    else {
        panic!("border-image contributions")
    };
    for item in contributions.items() {
        assert!(item.source().same_occurrence(&declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
        match item.value() {
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageSlice(slice)) => {
                assert!(!slice.fill());
                for edge in slice.values() {
                    let CssBorderImageSliceComponent::Percentage(value) = edge else {
                        panic!("percentage initial")
                    };
                    assert_eq!(percentage_text(value), "100");
                    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
                }
            }
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageWidth(width)) => {
                for edge in width.values() {
                    let CssBorderImageWidthComponent::Number(value) = edge else {
                        panic!("number width initial")
                    };
                    assert_eq!(number_text(value), "1");
                    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
                }
            }
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageOutset(outset)) => {
                for edge in outset.values() {
                    let CssBorderImageOutsetComponent::Number(value) = edge else {
                        panic!("number outset initial")
                    };
                    assert_eq!(number_text(value), "0");
                    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
                }
            }
            _ => {}
        }
    }
    assert_eq!(
        contributions
            .items()
            .iter()
            .filter(|item| matches!(
                item.property(),
                CssKnownProperty::BorderImageSlice
                    | CssKnownProperty::BorderImageWidth
                    | CssKnownProperty::BorderImageOutset
            ))
            .count(),
        3
    );
}

#[test]
fn iteration_constructor_retains_fraction_zero_infinite_math_nonempty_order_and_omission() {
    assert!(CssAnimationIterationCountList::try_new(vec![]).is_none());
    let counts = CssAnimationIterationCountList::try_new(vec![
        CssAnimationIterationCount::Number(number(".5")),
        CssAnimationIterationCount::Infinite,
        CssAnimationIterationCount::Number(number("-0")),
        CssAnimationIterationCount::Number(number_math("calc(-1)")),
    ])
    .unwrap();
    assert!(
        matches!(&counts.values()[0], CssAnimationIterationCount::Number(value) if number_text(value) == ".5")
    );
    assert!(matches!(
        &counts.values()[1],
        CssAnimationIterationCount::Infinite
    ));
    assert!(
        matches!(&counts.values()[2], CssAnimationIterationCount::Number(value) if number_text(value) == "-0")
    );
    assert!(
        matches!(&counts.values()[3], CssAnimationIterationCount::Number(value) if value.calculation().is_some())
    );
    let animation = CssAnimation::try_new(CssAnimationComponents {
        iteration_count: Some(counts.values()[0].clone()),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    assert!(
        matches!(animation.iteration_count(), Some(CssAnimationIterationCount::Number(value)) if number_text(value) == ".5")
    );
    assert!(animation.name().is_none());
    assert!(animation.duration().is_none());
    let omitted = CssAnimation::try_new(CssAnimationComponents {
        name: Some(CssAnimationName::None),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    assert!(omitted.iteration_count().is_none());
    let explicit = CssAnimation::try_new(CssAnimationComponents {
        name: Some(CssAnimationName::None),
        iteration_count: Some(CssAnimationIterationCount::Number(number("1"))),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    assert_ne!(omitted, explicit);
}

#[test]
fn iteration_aggregate_equality_ignores_only_numeric_origins_and_preserves_ast_roles_order() {
    let parsed = CssSpecifiedNonNegativeNumber::try_from_component(token("  .5")).unwrap();
    assert_eq!(
        CssAnimationIterationCount::Number(parsed),
        CssAnimationIterationCount::Number(number(".5"))
    );
    assert_eq!(
        CssAnimationIterationCount::Number(number_math("calc(1 + 2)")),
        CssAnimationIterationCount::Number(number_math("  calc(1 + 2)"))
    );
    assert_ne!(
        CssAnimationIterationCount::Number(number("1")),
        CssAnimationIterationCount::Number(number("1.0"))
    );
    assert_ne!(
        CssAnimationIterationCount::Number(number("1")),
        CssAnimationIterationCount::Number(number_math("calc(1)"))
    );
    assert_ne!(
        CssAnimationIterationCount::Number(number_math("calc(1 + 2)")),
        CssAnimationIterationCount::Number(number_math("calc(2 + 1)"))
    );
    let number = CssAnimationIterationCount::Number(number("1"));
    assert_ne!(
        CssAnimationIterationCountList::try_new(vec![
            number.clone(),
            CssAnimationIterationCount::Infinite
        ]),
        CssAnimationIterationCountList::try_new(vec![CssAnimationIterationCount::Infinite, number])
    );
}

#[test]
fn iteration_longhand_and_shorthand_keep_shared_exact_counts_and_authored_omission() {
    let report = parse_style_attribute(
        "animation-iteration-count: .5, infinite, 1e999, calc(-1); animation: fade 1s .5; animation: fade 1s",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 3);
    let CssKnownPropertyValueRef::AnimationIterationCount(counts) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("iteration longhand")
    };
    let [
        CssAnimationIterationCount::Number(fraction),
        CssAnimationIterationCount::Infinite,
        CssAnimationIterationCount::Number(huge),
        CssAnimationIterationCount::Number(math),
    ] = counts.iteration_counts().values()
    else {
        panic!("ordered counts")
    };
    assert_eq!(number_text(fraction), ".5");
    assert_eq!(number_text(huge), "1e999");
    assert!(matches!(fraction.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        math.calculation().unwrap().result_type(),
        CssCalculationType::Number
    );
    for (index, supplied) in [(1, true), (2, false)] {
        let CssKnownPropertyValueRef::Animation(wrapper) = report.syntax()[index]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("animation shorthand")
        };
        let [animation] = wrapper.animations().values() else {
            panic!("one animation")
        };
        assert!(
            matches!(animation.name(), Some(CssAnimationName::Custom(name)) if name.as_str() == "fade")
        );
        assert!(
            matches!(animation.duration(), Some(CssDuration::Literal(value)) if value.value() == 1.0 && value.unit() == CssTimeUnit::Seconds)
        );
        if supplied {
            assert!(
                matches!(animation.iteration_count(), Some(CssAnimationIterationCount::Number(value)) if number_text(value) == ".5")
            );
        } else {
            assert!(animation.iteration_count().is_none());
        }
    }
}

#[test]
fn shared_nonnegative_projection_limits_are_atomic_and_percentage_ranges_share_budgets() {
    let n = number("12.50");
    let p = percentage("12.50%");
    assert_eq!(
        n.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 4))
            .unwrap(),
        "12.5"
    );
    assert_eq!(
        p.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 5))
            .unwrap(),
        "12.5%"
    );
    for (input, projection, bytes, kind) in [
        (
            0,
            1,
            10,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            1,
            0,
            10,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (1, 1, 3, CssSpecifiedValueSerializationErrorKind::ByteLimit),
    ] {
        assert_eq!(
            n.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input, projection, bytes
            ))
            .unwrap_err()
            .kind(),
            kind
        );
        assert_eq!(
            p.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input, projection, bytes
            ))
            .unwrap_err()
            .kind(),
            kind
        );
    }
    assert_eq!(number_text(&n), "12.50");
    assert_eq!(percentage_text(&p), "12.50");
    let range = CssFontFaceWidth::Range {
        start: CssFontWidth::Percentage(percentage("75%")),
        end: Some(CssFontWidth::Percentage(percentage("125%"))),
    };
    assert_eq!(
        range
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 8))
            .unwrap(),
        "75% 125%"
    );
    for (input, projection, bytes, kind) in [
        (
            2,
            3,
            8,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            3,
            2,
            8,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (3, 3, 7, CssSpecifiedValueSerializationErrorKind::ByteLimit),
    ] {
        assert_eq!(
            range
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input, projection, bytes
                ))
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(range.serialize_specified().unwrap(), "75% 125%");
}

fn assert_origin(origin: &CssValueOrigin, source: &str, start: usize, length: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("retained parsed origin");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), start + length);
}

#[test]
fn migrated_border_and_iteration_payloads_keep_exact_source_coordinates_and_kinds() {
    let source = "border-image-slice: +01.25 2.50%; border-image-width: 3e2; border-image-outset: -0; animation-iteration-count: .5, calc(-1)";
    let report = parse_style_attribute(source);
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 4);
    let CssKnownPropertyValueRef::BorderImageSlice(slice) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("slice");
    };
    let [
        CssBorderImageSliceComponent::Number(number),
        CssBorderImageSliceComponent::Percentage(percentage),
        _,
        _,
    ] = slice.slice().values()
    else {
        panic!("number/percentage edges");
    };
    assert_eq!(number_text(number), "+01.25");
    assert_eq!(percentage_text(percentage), "2.50");
    assert_origin(number.origin(), source, 20, 6);
    assert_origin(percentage.origin(), source, 27, 5);
    let CssKnownPropertyValueRef::BorderImageWidth(width) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("width");
    };
    let CssBorderImageWidthComponent::Number(number) = &width.widths().values()[0] else {
        panic!("number width");
    };
    assert_eq!(number_text(number), "3e2");
    assert_origin(number.origin(), source, 54, 3);
    let CssKnownPropertyValueRef::BorderImageOutset(outset) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("outset");
    };
    let CssBorderImageOutsetComponent::Number(number) = &outset.outsets().values()[0] else {
        panic!("number outset");
    };
    assert_eq!(number_text(number), "-0");
    assert_origin(number.origin(), source, 80, 2);
    assert!(
        matches!(number.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.kind() == CssNumericTokenKind::Integer)
    );
    let CssKnownPropertyValueRef::AnimationIterationCount(counts) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("iteration");
    };
    let [
        CssAnimationIterationCount::Number(fraction),
        CssAnimationIterationCount::Number(math),
    ] = counts.iteration_counts().values()
    else {
        panic!("ordered scalar counts");
    };
    assert_eq!(number_text(fraction), ".5");
    assert_origin(fraction.origin(), source, 111, 2);
    assert!(
        matches!(fraction.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.kind() == CssNumericTokenKind::Number)
    );
    let calculation = math.calculation().expect("symbolic number function");
    assert_origin(calculation.origin(), source, 115, 5);
    assert_eq!(
        calculation.components().serialize().unwrap().as_css(),
        "calc(-1)"
    );
}
