//! Backgrounds 3 CRD 2024-03-11 §5.2 and Values 4 WD 2024-03-12 §2.2:
//! `[ <number [0,∞]> | <percentage [0,∞]> ]{1,4} && fill?` makes the
//! repeated numeric group one component; fill can precede or follow that group.
//! Masking 1 CRD 2021-08-05 §8.3 instead selects trailing `fill?`.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-slice
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#component-combinators

use crate::css::support::check_property_text as checked;
use crate::css::support::property_admission::{
    assert_checked_rejected, assert_style_attribute_rejected,
};
use surgeist_css::*;

const PROPERTIES: [CssKnownProperty; 2] = [
    CssKnownProperty::BorderImageSlice,
    CssKnownProperty::BorderImage,
];

fn parsed(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let css = format!("{}: {text} !important;", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed longhands")
    };
    values
}

fn slice(values: &CssLonghandContributions) -> &CssBorderImageSlice {
    let item = values
        .items()
        .iter()
        .find(|item| item.property() == CssKnownProperty::BorderImageSlice)
        .unwrap();
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageSlice(value)) =
        item.value()
    else {
        panic!("typed border-image slice")
    };
    value
}

/// Each split group with the numeric token that the group cannot extend to.
fn split_groups() -> [(&'static str, &'static str); 6] {
    [
        ("1 fill 2", "2"),
        ("1 fill 2 3", "2"),
        ("1 2 fill 3", "3"),
        ("1 fill 2 3 4", "2"),
        ("1 2 fill 3 4", "3"),
        ("1 2 3 fill 4", "4"),
    ]
}

#[test]
fn parsed_slice_and_shorthand_reject_fill_inside_the_numeric_group() {
    for property in PROPERTIES {
        for (text, token) in split_groups() {
            let css = format!("{}: {text};", property.canonical_name());
            let report = assert_style_attribute_rejected(&css, property, Some(token));
            assert!(
                report.syntax().is_empty(),
                "invalid declaration retained: {css}"
            );
            assert_eq!(
                report.diagnostics()[0].error().code(),
                CssErrorCode::InvalidPropertyValue
            );
        }
    }
}

#[test]
fn checked_slice_and_shorthand_reject_fill_inside_the_numeric_group() {
    for property in PROPERTIES {
        for (text, token) in split_groups() {
            assert_checked_rejected(checked(property, text), property, Some(token));
        }
    }
}

#[test]
fn shorthand_slash_suffix_cannot_make_a_split_slice_group_valid() {
    for (text, token) in [
        ("none 1 fill 2 / 3 / 4 round", "2"),
        ("round 1 2 fill 3 // 4", "3"),
        ("1 2 3 fill 4 / auto space", "4"),
    ] {
        assert_checked_rejected(
            checked(CssKnownProperty::BorderImage, text),
            CssKnownProperty::BorderImage,
            Some(token),
        );
        assert_style_attribute_rejected(
            &format!("border-image: {text};"),
            CssKnownProperty::BorderImage,
            Some(token),
        );
    }
}

#[test]
fn pending_reentry_rejects_split_groups_atomically_and_remains_reusable() {
    for property in PROPERTIES {
        for source in [
            parsed(property, "var(--slice)"),
            checked(property, "var(--slice)").unwrap(),
        ] {
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending substitution")
            };
            let before = source.serialize_specified(Default::default()).unwrap();
            for _ in 0..2 {
                for (text, _) in split_groups() {
                    let replacement = parse_component_values(text).unwrap();
                    let error = handle
                        .reenter(replacement)
                        .expect_err("pending split group rejection");
                    let direct = checked(property, text).expect_err("checked group rejection");
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("intrinsic grammar error: {error:?}")
                    };
                    assert_eq!(actual.kind(), direct.kind());
                    assert_eq!(actual.origin(), direct.origin());
                }
                let replacement = parse_component_values("fill 1 2 3 4").unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed reentry")
                };
                assert_eq!(
                    slice(&values)
                        .serialize_specified(Default::default())
                        .unwrap(),
                    "1 2 3 4 fill"
                );
                for item in values.items() {
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    let retained = item.replacement_components().unwrap();
                    assert_eq!(
                        retained
                            .serialize_tokens(Default::default())
                            .unwrap()
                            .as_css(),
                        replacement
                            .serialize_tokens(Default::default())
                            .unwrap()
                            .as_css()
                    );
                    for (a, b) in retained.items().iter().zip(replacement.items()) {
                        assert_eq!(a.origin(), b.origin());
                    }
                }
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(
                    source.serialize_specified(Default::default()).unwrap(),
                    before
                );
                let error = handle
                    .reenter(parse_component_values("var(--remaining)").unwrap())
                    .unwrap_err();
                assert!(matches!(
                    error.kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
        }
    }
}

#[test]
fn recovery_drops_only_the_split_group_and_preserves_siblings_and_coordinates() {
    for property in PROPERTIES {
        let declaration = format!("{}: 1 fill 2;", property.canonical_name());
        let css = format!("--😀: kept; {declaration} color: red;");
        let report = parse_style_attribute(&css);
        let [diagnostic] = report.diagnostics() else {
            panic!("one split-group diagnostic: {:?}", report.diagnostics())
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(report.syntax().len(), 2);
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Color
        );
        let start = css.find(&declaration).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + declaration.len()
        );
        let offset = start + property.canonical_name().len() + 2 + "1 fill ".len();
        assert_eq!(diagnostic.error().position().byte_offset().value(), offset);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            css[..offset].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn fill_before_or_after_complete_groups_preserves_edges_and_round_trips() {
    for property in PROPERTIES {
        for (group, canonical) in [
            ("+1", "1"),
            ("-0 2%", "0 2%"),
            ("1 2% 3", "1 2% 3"),
            ("1 2% 3 4%", "1 2% 3 4%"),
        ] {
            for text in [
                group.to_owned(),
                format!("fill {group}"),
                format!("{group} fill"),
            ] {
                for source in [parsed(property, &text), checked(property, &text).unwrap()] {
                    let values = contributions(&source);
                    let value = slice(&values);
                    assert_eq!(value.fill(), text.contains("fill"));
                    let expected =
                        format!("{canonical}{}", if value.fill() { " fill" } else { "" });
                    assert_eq!(
                        value.serialize_specified(Default::default()).unwrap(),
                        expected
                    );
                    for edge in value.values() {
                        let origin = match edge {
                            CssBorderImageSliceComponent::Number(number) => number.origin(),
                            CssBorderImageSliceComponent::Percentage(number) => number.origin(),
                            _ => panic!("number or percentage slice component"),
                        };
                        assert!(matches!(origin, CssValueOrigin::Parsed(_)));
                    }
                    let css = source.serialize_specified(Default::default()).unwrap();
                    let reparsed = parse_style_attribute(&css);
                    assert!(reparsed.is_clean(), "{css}: {:?}", reparsed.diagnostics());
                    assert_eq!(
                        slice(&contributions(&reparsed.syntax()[0]))
                            .serialize_specified(Default::default())
                            .unwrap(),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn range_cardinality_duplicate_and_slash_controls_stay_rejected() {
    for property in PROPERTIES {
        // The slice longhand has no slash; the shorthand's slash suffix needs
        // a nonnegative width or outset after it.
        let shorthand = property == CssKnownProperty::BorderImage;
        for (text, slice_token, shorthand_token) in [
            ("", None, None),
            ("fill", Some("fill"), Some("fill")),
            ("fill fill", Some("fill"), Some("fill")),
            ("1 fill fill", Some("fill"), Some("fill")),
            ("fill 1 fill", Some("fill"), Some("fill")),
            ("1 2 3 4 5", Some("5"), Some("5")),
            ("fill 1 2 3 4 5", Some("5"), Some("5")),
            ("-1", Some("-1"), Some("-1")),
            ("-1%", Some("-1%"), Some("-1%")),
            ("1 -2 fill", Some("-2"), Some("-2")),
            ("fill -2%", Some("-2%"), Some("-2%")),
            ("1 /", Some("/"), Some("/")),
            ("1 //", Some("/"), Some("/")),
            ("1 / 2 /", Some("/"), Some("/")),
            ("1 / -2", Some("/"), Some("-2")),
        ] {
            let token = if shorthand {
                shorthand_token
            } else {
                slice_token
            };
            assert_checked_rejected(checked(property, text), property, token);
            assert_style_attribute_rejected(
                &format!("{}: {text};", property.canonical_name()),
                property,
                token,
            );
        }
    }
    for text in [
        "none fill 1 2 / 3 / 4 round",
        "round 1 2 fill // 4 none",
        "fill 1 2 3 4 / auto space",
    ] {
        for source in [
            parsed(CssKnownProperty::BorderImage, text),
            checked(CssKnownProperty::BorderImage, text).unwrap(),
        ] {
            assert!(slice(&contributions(&source)).fill());
        }
    }
}

#[test]
fn masking_slice_keeps_its_separate_trailing_fill_grammar() {
    for property in [
        CssKnownProperty::MaskBorderSlice,
        CssKnownProperty::MaskBorder,
    ] {
        for text in ["1", "1 2%", "1 2 3 4 fill"] {
            assert!(checked(property, text).is_ok(), "mask control: {text}");
            assert!(
                parse_style_attribute(&format!("{}: {text};", property.canonical_name()))
                    .is_clean()
            );
        }
        for (text, token) in [
            ("fill 1", "fill"),
            ("1 fill 2", "2"),
            ("1 2 fill 3", "3"),
            ("1 2 3 fill 4", "4"),
        ] {
            assert_checked_rejected(checked(property, text), property, Some(token));
            assert_style_attribute_rejected(
                &format!("{}: {text};", property.canonical_name()),
                property,
                Some(token),
            );
        }
    }
}

#[test]
fn mixed_origin_checked_slice_groups_retain_nonnegative_scalar_ownership() {
    for property in PROPERTIES {
        let parsed = parse_component_values("fill 1 2% 3 4%").unwrap();
        let mut items = parsed.items().to_vec();
        let index = items.iter().position(|item| matches!(item.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == "3")).unwrap();
        items[index] = CssComponentValue::try_number("+3").unwrap();
        let components = CssComponentValues::try_new(items, Default::default()).unwrap();
        let source = parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
        .unwrap();
        let values = contributions(&source);
        let value = slice(&values);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            "1 2% 3 4% fill"
        );
        let CssBorderImageSliceComponent::Number(third) = &value.values()[2] else {
            panic!("third number")
        };
        assert_eq!(third.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(third.serialize_specified(Default::default()).unwrap(), "3");
    }
}

#[test]
fn slice_serialization_budgets_charge_the_complete_group_and_fill_atomically() {
    let source = checked(CssKnownProperty::BorderImageSlice, "fill 1 2% 3 4%").unwrap();
    let values = contributions(&source);
    let value = slice(&values);
    let expected = "1 2% 3 4% fill";
    type Limits = CssSerializationLimits;
    type Kind = CssSpecifiedValueSerializationErrorKind;
    assert_eq!(
        value
            .serialize_specified(Limits::new(10, 10, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(9, 10, expected.len()), Kind::InputNodeLimit),
        (
            Limits::new(10, 9, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(10, 10, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(value.serialize_specified(limits).unwrap_err().kind(), kind);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
}
