#![forbid(unsafe_code)]
//! Backgrounds 3 CRD 2024-03-11 §§5.1–5.5, 5.7 and CSSOM WD 2021-08-26
//! §6.7.2 supply grammar/order/compression oracles. Exact resource formulas and
//! immutable, branch-preserving checked construction are Surgeist contracts.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Error = CssSpecifiedValueSerializationError;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn number(text: &str) -> CssSpecifiedNonNegativeNumber {
    if text.starts_with("calc(") {
        CssSpecifiedNonNegativeNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap()
    }
}

fn slice(texts: &[&str], fill: bool) -> CssBorderImageSlice {
    CssBorderImageSlice::try_new(
        texts
            .iter()
            .map(|text| {
                if text.ends_with('%') {
                    CssBorderImageSliceComponent::Percentage(
                        CssSpecifiedNonNegativePercentage::try_from_component(
                            CssComponentValue::try_token(text).unwrap(),
                        )
                        .unwrap(),
                    )
                } else {
                    CssBorderImageSliceComponent::Number(number(text))
                }
            })
            .collect(),
        fill,
    )
    .unwrap()
}

fn width(texts: &[&str]) -> CssBorderImageWidth {
    CssBorderImageWidth::try_new(
        texts
            .iter()
            .map(|text| CssBorderImageWidthComponent::Number(number(text)))
            .collect(),
    )
    .unwrap()
}

fn outset(texts: &[&str]) -> CssBorderImageOutset {
    CssBorderImageOutset::try_new(
        texts
            .iter()
            .map(|text| CssBorderImageOutsetComponent::Number(number(text)))
            .collect(),
    )
    .unwrap()
}

fn declaration(text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BorderImage),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap()
}

fn image(text: &str) -> CssBorderImage {
    let declaration = declaration(text);
    let CssKnownPropertyValueRef::BorderImage(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("border-image typed value");
    };
    value.border_image().clone()
}

fn exact_limits(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(Limits) -> Result<String, Error>,
) {
    assert_eq!(
        serialize(Limits::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(input - 1, projection, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(input, projection - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(input, projection, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
}

fn expanded_texts(text: &str) -> Vec<String> {
    let declaration = declaration(text);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&declaration).unwrap()
    else {
        panic!("completed border-image longhands");
    };
    values
        .items()
        .iter()
        .map(|item| {
            let CssContributionValueRef::Ordinary(value) = item.value() else {
                panic!("ordinary typed value");
            };
            match value {
                CssLonghandValueRef::BorderImageSource(value) => value.serialize_specified(),
                CssLonghandValueRef::BorderImageSlice(value) => value.serialize_specified(),
                CssLonghandValueRef::BorderImageWidth(value) => value.serialize_specified(),
                CssLonghandValueRef::BorderImageOutset(value) => value.serialize_specified(),
                CssLonghandValueRef::BorderImageRepeat(value) => value.serialize_specified(),
                _ => panic!("ordered border-image member"),
            }
            .unwrap()
        })
        .collect()
}

#[test]
fn checked_construction_preserves_fields_origins_and_expanded_edges() {
    let slice = slice(&["10", "20%", "30"], true);
    let width = width(&["2", "3"]);
    let outset = outset(&["4"]);
    let repeat = CssBorderImageRepeat::new(
        CssBorderImageRepeatKeyword::Round,
        CssBorderImageRepeatKeyword::Space,
    );
    let value = CssBorderImage::try_new(
        Some(CssImageValue::None),
        Some(slice.clone()),
        Some(width.clone()),
        Some(outset.clone()),
        Some(repeat),
    )
    .unwrap();
    assert_eq!(value.source(), Some(&CssImageValue::None));
    assert_eq!(value.slice(), Some(&slice));
    assert_eq!(value.width(), Some(&width));
    assert_eq!(value.outset(), Some(&outset));
    assert_eq!(value.repeat(), Some(repeat));
    assert!(value.slice().unwrap().fill());
    for (edge, (expected, percentage)) in value.slice().unwrap().values().iter().zip([
        ("10", false),
        ("20", true),
        ("30", false),
        ("20", true),
    ]) {
        let token = match edge {
            CssBorderImageSliceComponent::Number(number) if !percentage => {
                assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
                number.literal_component().unwrap().view()
            }
            CssBorderImageSliceComponent::Percentage(number) if percentage => {
                assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
                number.literal_component().unwrap().view()
            }
            _ => panic!("independent expected slice branch"),
        };
        let representation = match token {
            CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if !percentage => {
                number.representation()
            }
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) if percentage => {
                number.representation()
            }
            _ => panic!("independent expected literal token branch"),
        };
        assert_eq!(representation, expected);
    }
    for (edge, expected) in value
        .width()
        .unwrap()
        .values()
        .iter()
        .zip(["2", "3", "2", "3"])
    {
        let CssBorderImageWidthComponent::Number(number) = edge else {
            panic!("number width edge");
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Number(token)) =
            number.literal_component().unwrap().view()
        else {
            panic!("literal number width edge");
        };
        assert_eq!(token.representation(), expected);
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    }
    for edge in value.outset().unwrap().values() {
        let CssBorderImageOutsetComponent::Number(number) = edge else {
            panic!("number outset edge");
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Number(token)) =
            number.literal_component().unwrap().view()
        else {
            panic!("literal number outset edge");
        };
        assert_eq!(token.representation(), "4");
        assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    }
    assert_eq!(
        value.repeat().unwrap().horizontal(),
        CssBorderImageRepeatKeyword::Round
    );
    assert_eq!(
        value.repeat().unwrap().vertical(),
        CssBorderImageRepeatKeyword::Space
    );
    let CssBorderImageSliceComponent::Number(top) = &slice.values()[0] else {
        panic!("number");
    };
    assert_eq!(top.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(slice.serialize_specified().unwrap(), "10 20% 30 fill");
    assert_eq!(width.serialize_specified().unwrap(), "2 3");
    assert_eq!(outset.serialize_specified().unwrap(), "4");

    let parsed = image("fill 10 20% 30 / 2 3 / 4 round space");
    let copy = parsed.clone();
    let CssBorderImageSliceComponent::Number(top) = &parsed.slice().unwrap().values()[0] else {
        panic!("number");
    };
    let CssBorderImageSliceComponent::Number(copied_top) = &copy.slice().unwrap().values()[0]
    else {
        panic!("number");
    };
    let CssValueOrigin::Parsed(origin) = top.origin() else {
        panic!("parsed top edge origin");
    };
    assert_eq!(
        origin.source().as_str(),
        "fill 10 20% 30 / 2 3 / 4 round space"
    );
    assert_eq!(origin.span().start().byte_offset().value(), 5);
    assert_eq!(origin.span().end().byte_offset().value(), 7);
    assert_eq!(top.origin(), copied_top.origin());
    assert_eq!(parsed, copy);
    assert_eq!(
        parsed.serialize_specified().unwrap(),
        "10 20% 30 fill / 2 3 / 4 round space"
    );
}

#[test]
fn empty_overlong_and_missing_slice_construction_is_rejected() {
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
    assert!(CssBorderImage::try_new(None, None, None, None, None).is_none());
    assert!(
        CssBorderImage::try_new(
            Some(CssImageValue::None),
            None,
            Some(width(&["2"])),
            None,
            None
        )
        .is_none()
    );
    assert!(
        CssBorderImage::try_new(
            None,
            None,
            None,
            Some(outset(&["2"])),
            Some(CssBorderImageRepeat::new(
                CssBorderImageRepeatKeyword::Repeat,
                CssBorderImageRepeatKeyword::Repeat
            ))
        )
        .is_none()
    );
}

#[test]
fn canonical_equal_edges_use_shortest_one_to_four_sequences() {
    for (input, expected) in [
        (["+1.0", "1e0", "01", "1"], "1"),
        (["1", "2", "1.0", "2e0"], "1 2"),
        (["1", "2", "3", "2.0"], "1 2 3"),
        (["1", "2", "3", "4"], "1 2 3 4"),
    ] {
        for result in [
            slice(&input, false).serialize_specified(),
            width(&input).serialize_specified(),
            outset(&input).serialize_specified(),
        ] {
            assert_eq!(result.unwrap(), expected);
        }
    }
    assert_eq!(
        slice(&["0", "0%", "0", "0%"], false)
            .serialize_specified()
            .unwrap(),
        "0 0%"
    );
    assert_eq!(
        slice(&["100%"], true).serialize_specified().unwrap(),
        "100% fill"
    );
}

#[test]
fn scalar_limits_charge_all_four_edges_even_when_one_is_emitted() {
    let value = slice(&["+1.0", "1e0", "01", "1"], false);
    exact_limits("1", 9, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = slice(&["1"], true);
    exact_limits("1 fill", 10, 10, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = width(&["1"]);
    exact_limits("1", 9, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = outset(&["0"]);
    exact_limits("0", 9, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::Auto]).unwrap();
    exact_limits("auto", 5, 5, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    for (horizontal, vertical, expected) in [
        (
            CssBorderImageRepeatKeyword::Stretch,
            CssBorderImageRepeatKeyword::Stretch,
            "stretch",
        ),
        (
            CssBorderImageRepeatKeyword::Round,
            CssBorderImageRepeatKeyword::Space,
            "round space",
        ),
    ] {
        let value = CssBorderImageRepeat::new(horizontal, vertical);
        exact_limits(expected, 3, 3, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
}

#[test]
fn canonical_math_compression_preserves_symbolic_roots_and_all_projection_costs() {
    // Each calc group/product/two leaves visits four inputs and projects four
    // nodes (two values, inverse and product result). No duplicate is skipped.
    let value = slice(
        &["calc(1 / 2)", "calc(0.5)", "calc(2 / 4)", "calc(0.50)"],
        false,
    );
    exact_limits("calc(0.5)", 17, 15, |limits| {
        value.serialize_specified_with_limits(limits)
    });

    let values = [
        "calc(1em + 2px)",
        "calc(2px + 1em)",
        "calc(1.0em + 2.0px)",
        "calc(2px + 1em)",
    ]
    .iter()
    .map(|text| {
        CssBorderImageWidthComponent::LengthPercentage(
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::try_from_components(
                    parse_component_values(text).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    })
    .collect();
    let value = CssBorderImageWidth::try_new(values).unwrap();
    // Each unresolved sum visits group/sum/two values, then projects two leaves,
    // two same-unit combined values and the sum: four inputs/five projections.
    exact_limits("calc(1em + 2px)", 21, 25, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = image("calc(100%) / calc(1) / calc(0) stretch");
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc(100%) / calc(1) / calc(0)"
    );
}

#[test]
fn slot_serialization_keeps_number_percentage_and_length_zero_branches() {
    let width = CssBorderImageWidth::try_new(vec![
        CssBorderImageWidthComponent::Number(number("0")),
        CssBorderImageWidthComponent::LengthPercentage(
            CssSpecifiedNonNegativeLengthPercentage::zero(),
        ),
        CssBorderImageWidthComponent::Number(number("0")),
        CssBorderImageWidthComponent::LengthPercentage(
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                CssComponentValue::try_token("0%").unwrap(),
            )
            .unwrap(),
        ),
    ])
    .unwrap();
    exact_limits("0 0px 0 0%", 9, 9, |limits| {
        width.serialize_specified_with_limits(limits)
    });
    let outset = CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Length(
        CssSpecifiedNonNegativeLength::zero(),
    )])
    .unwrap();
    exact_limits("0px", 9, 9, |limits| {
        outset.serialize_specified_with_limits(limits)
    });
    let value = CssBorderImage::try_new(
        None,
        Some(slice(&["100%"], false)),
        Some(width),
        Some(outset),
        None,
    )
    .unwrap();
    let text = value.serialize_specified().unwrap();
    assert_eq!(text, "100% / 0 0px 0 0% / 0px");
    let reentered = image(&text);
    assert!(matches!(
        reentered.width().unwrap().values()[0],
        CssBorderImageWidthComponent::Number(_)
    ));
    for index in [1, 3] {
        assert!(matches!(
            reentered.width().unwrap().values()[index],
            CssBorderImageWidthComponent::LengthPercentage(_)
        ));
    }
    assert!(
        reentered
            .outset()
            .unwrap()
            .values()
            .iter()
            .all(|value| matches!(value, CssBorderImageOutsetComponent::Length(_)))
    );
}

#[test]
fn shorthand_initials_coupling_fill_and_order_have_independent_expected_text() {
    for (authored, expected) in [
        ("none", "none"),
        ("100%", "none"),
        ("none 100% / 1 / 0 stretch stretch", "none"),
        ("100% / 2", "100% / 2"),
        ("100% / 1 / 2", "100% / / 2"),
        ("100% // 2", "100% / / 2"),
        ("100% fill / 1 / 0 stretch", "100% fill"),
        (
            "round space 10 20% 30 fill / auto 2 25% 4px / 0 2px 3 4px url(frame.png)",
            "url(\"frame.png\") 10 20% 30 fill / auto 2 25% 4px / 0 2px 3 4px round space",
        ),
    ] {
        let value = image(authored);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(image(expected).serialize_specified().unwrap(), expected);
        assert_eq!(expanded_texts(authored), expanded_texts(expected));
    }
    assert_eq!(
        expanded_texts("none"),
        ["none", "100%", "1", "0", "stretch"]
    );
    assert_eq!(
        expanded_texts("100% // 2"),
        ["none", "100%", "1", "2", "stretch"]
    );
}

#[test]
fn omitted_initials_visit_all_children_without_consuming_discarded_bytes() {
    let value = image("none 100% / 1 / 0 stretch stretch");
    exact_limits("none", 32, 33, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = image("none");
    exact_limits("none", 2, 3, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    // The source consumes the complete final byte allowance. Initial numeric
    // groups must be visited without attempting even one discarded byte.
    let value = image("url(x) 100% / 1 / 0 stretch stretch");
    exact_limits("url(\"x\")", 33, 33, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let exponent = "9".repeat(100);
    let value = CssBorderImage::try_new(
        Some(CssImageValue::None),
        Some(slice(&["100%"], false)),
        Some(width(&["1"])),
        Some(outset(&[&format!("-0e{exponent}")])),
        Some(CssBorderImageRepeat::new(
            CssBorderImageRepeatKeyword::Stretch,
            CssBorderImageRepeatKeyword::Stretch,
        )),
    )
    .unwrap();
    exact_limits("none", 32, 33, |limits| {
        value.serialize_specified_with_limits(limits)
    });
}

#[test]
fn huge_exact_literals_bound_expansion_and_preserve_input_after_failure() {
    let value = slice(&["1e20"], false);
    exact_limits("100000000000000000000", 9, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let value = slice(&["1e20", "1", "1e20", "1"], false);
    exact_limits("100000000000000000000 1", 9, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let huge = format!("1e{}", "9".repeat(100));
    let value = slice(&[&huge], false);
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(100, 100, 4))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, before);
    let value = slice(&[&format!("-0e{}", "9".repeat(100))], false);
    exact_limits("0", 9, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
}

#[test]
fn cumulative_source_prefix_and_later_groups_fail_atomically() {
    let value = image("url(x) 100% / 2 / 3 round space");
    // Root1 + URL2 + three groups9 each + Repeat3 =33; no fallback.
    exact_limits("url(\"x\") 100% / 2 / 3 round space", 33, 33, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(33, 33, 20))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, before);
    assert_eq!(
        value.serialize_specified().unwrap(),
        "url(\"x\") 100% / 2 / 3 round space"
    );
}
