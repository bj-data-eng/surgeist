#![forbid(unsafe_code)]

//! External-consumer assertions for intrinsic declaration expansion.
//!
//! Independent shorthand/default expectations follow CSS Box 3 and Backgrounds 3:
//! https://www.w3.org/TR/css-box-3/#margin-shorthand
//! https://www.w3.org/TR/css-box-3/#padding-shorthand
//! https://www.w3.org/TR/css-backgrounds-3/#border-shorthands
//! https://www.w3.org/TR/css-backgrounds-3/#border-images
//! Global/reset-only behavior follows Cascade 5; substitution follows Variables 1:
//! https://www.w3.org/TR/css-cascade-5/#shorthand
//! https://www.w3.org/TR/css-cascade-5/#all-shorthand
//! https://www.w3.org/TR/css-variables-1/#variables-in-shorthands
//! Exact typed values, occurrence identity, and original token origins are
//! Surgeist contracts. This consumer neither resolves style nor loads resources.

use surgeist_css::{CssKnownProperty as Property, *};

const MARGINS: [Property; 4] = [
    Property::MarginTop,
    Property::MarginRight,
    Property::MarginBottom,
    Property::MarginLeft,
];
const PADDINGS: [Property; 4] = [
    Property::PaddingTop,
    Property::PaddingRight,
    Property::PaddingBottom,
    Property::PaddingLeft,
];
const MARGIN_BLOCK: [Property; 2] = [Property::MarginBlockStart, Property::MarginBlockEnd];
const PADDING_BLOCK: [Property; 2] = [Property::PaddingBlockStart, Property::PaddingBlockEnd];
const WIDTHS: [Property; 4] = [
    Property::BorderTopWidth,
    Property::BorderRightWidth,
    Property::BorderBottomWidth,
    Property::BorderLeftWidth,
];
const STYLES: [Property; 4] = [
    Property::BorderTopStyle,
    Property::BorderRightStyle,
    Property::BorderBottomStyle,
    Property::BorderLeftStyle,
];
const COLORS: [Property; 4] = [
    Property::BorderTopColor,
    Property::BorderRightColor,
    Property::BorderBottomColor,
    Property::BorderLeftColor,
];
const SIDE_BORDERS: [Property; 4] = [
    Property::BorderTop,
    Property::BorderRight,
    Property::BorderBottom,
    Property::BorderLeft,
];
const IMAGE_LONGHANDS: [Property; 5] = [
    Property::BorderImageSource,
    Property::BorderImageSlice,
    Property::BorderImageWidth,
    Property::BorderImageOutset,
    Property::BorderImageRepeat,
];
const MODERN_COLOR: &str = "color(display-p3 1 0.5 0 / 150%)";

fn declaration(property: Property, value: &str, importance: CssImportance) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(value).expect("valid component syntax"),
        importance,
    )
    .expect("valid authored property grammar")
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    match expand_declaration(source).expect("supported expansion") {
        CssExpansion::Contributions(CssContributions::Longhands(values)) => {
            for item in values.items() {
                assert!(item.source().same_occurrence(source));
                assert!(item.replacement_components().is_none());
            }
            values
        }
        other => panic!("expected completed longhand contributions: {other:?}"),
    }
}

fn pending_expansion(source: &CssDeclaration) -> CssPendingSubstitution {
    match expand_declaration(source).expect("supported pending declaration") {
        CssExpansion::Pending(value) => value,
        other => panic!("expected pending substitution: {other:?}"),
    }
}

fn member(values: &CssLonghandContributions, property: Property) -> &CssLonghandContribution {
    let mut matching = values
        .items()
        .iter()
        .filter(|item| item.property() == property);
    let result = matching
        .next()
        .unwrap_or_else(|| panic!("missing {property:?}"));
    assert!(matching.next().is_none(), "duplicate {property:?}");
    result
}

fn assert_members(values: &CssLonghandContributions, expected: &[Property]) {
    assert_eq!(values.items().len(), expected.len());
    for property in expected {
        let _ = member(values, *property);
    }
}

fn border_px(number: &str) -> CssBorderWidth {
    CssBorderWidth::Length(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension(number, "px").unwrap(),
        )
        .unwrap(),
    )
}

fn length(item: &CssLonghandContribution) -> &CssBorderWidth {
    match (item.property(), item.value()) {
        (
            Property::BorderTopWidth,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderTopWidth(value)),
        )
        | (
            Property::BorderRightWidth,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderRightWidth(value)),
        )
        | (
            Property::BorderBottomWidth,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderBottomWidth(value)),
        )
        | (
            Property::BorderLeftWidth,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderLeftWidth(value)),
        ) => value,
        other => panic!("expected coupled length contribution: {other:?}"),
    }
}

fn spacing(item: &CssLonghandContribution) -> String {
    match item.value() {
        CssContributionValueRef::Ordinary(value) => match value {
            CssLonghandValueRef::MarginTop(value)
            | CssLonghandValueRef::MarginRight(value)
            | CssLonghandValueRef::MarginBottom(value)
            | CssLonghandValueRef::MarginLeft(value)
            | CssLonghandValueRef::MarginBlockStart(value)
            | CssLonghandValueRef::MarginBlockEnd(value) => value.serialize_specified().unwrap(),
            CssLonghandValueRef::PaddingTop(value)
            | CssLonghandValueRef::PaddingRight(value)
            | CssLonghandValueRef::PaddingBottom(value)
            | CssLonghandValueRef::PaddingLeft(value)
            | CssLonghandValueRef::PaddingBlockStart(value)
            | CssLonghandValueRef::PaddingBlockEnd(value) => value.serialize_specified().unwrap(),
            other => panic!("expected spacing value: {other:?}"),
        },
        other => panic!("expected ordinary spacing value: {other:?}"),
    }
}

fn style(item: &CssLonghandContribution) -> CssBorderStyle {
    match (item.property(), item.value()) {
        (
            Property::BorderTopStyle,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderTopStyle(value)),
        )
        | (
            Property::BorderRightStyle,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderRightStyle(value)),
        )
        | (
            Property::BorderBottomStyle,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderBottomStyle(value)),
        )
        | (
            Property::BorderLeftStyle,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderLeftStyle(value)),
        ) => *value,
        other => panic!("expected coupled border-style contribution: {other:?}"),
    }
}

fn color(item: &CssLonghandContribution) -> &CssColor {
    match (item.property(), item.value()) {
        (
            Property::BorderTopColor,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderTopColor(value)),
        )
        | (
            Property::BorderRightColor,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderRightColor(value)),
        )
        | (
            Property::BorderBottomColor,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderBottomColor(value)),
        )
        | (
            Property::BorderLeftColor,
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderLeftColor(value)),
        ) => value,
        other => panic!("expected coupled color contribution: {other:?}"),
    }
}

fn assert_modern_color(value: &CssColor) {
    let value = value
        .predefined_value()
        .expect("preserved predefined color");
    assert_eq!(value.color_space(), CssPredefinedColorSpace::DisplayP3);
    for (channel, expected) in value.channels().iter().zip(["1", "0.5", "0"]) {
        assert!(matches!(channel,
            CssColorComponent::Number(number) if number.numeric().representation() == expected));
    }
    assert!(matches!(value.alpha(),
        Some(CssColorComponent::Percentage(number)) if number.numeric().representation() == "150"));
}

fn border_members() -> Vec<Property> {
    WIDTHS
        .into_iter()
        .chain(STYLES)
        .chain(COLORS)
        .chain(IMAGE_LONGHANDS)
        .collect()
}

fn assert_image_initials(values: &CssLonghandContributions) {
    assert!(matches!(
        member(values, Property::BorderImageSource).value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageSource(
            CssImageValue::None
        ))
    ));
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageSlice(slice)) =
        member(values, Property::BorderImageSlice).value()
    else {
        panic!("typed border-image-slice initial");
    };
    assert!(!slice.fill());
    assert!(slice.values().iter().all(|value| matches!(value,
        CssBorderImageSliceComponent::Percentage(number) if exact_literal(number.literal_component(), "100%"))));
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageWidth(width)) =
        member(values, Property::BorderImageWidth).value()
    else {
        panic!("typed border-image-width initial");
    };
    assert!(width.values().iter().all(|value| matches!(value,
        CssBorderImageWidthComponent::Number(number) if exact_literal(number.literal_component(), "1"))));
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageOutset(outset)) =
        member(values, Property::BorderImageOutset).value()
    else {
        panic!("typed border-image-outset initial");
    };
    assert!(outset.values().iter().all(|value| matches!(value,
        CssBorderImageOutsetComponent::Number(number) if exact_literal(number.literal_component(), "0"))));
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageRepeat(repeat)) =
        member(values, Property::BorderImageRepeat).value()
    else {
        panic!("typed border-image-repeat initial");
    };
    assert_eq!(repeat.horizontal(), CssBorderImageRepeatKeyword::Stretch);
    assert_eq!(repeat.vertical(), CssBorderImageRepeatKeyword::Stretch);
}

fn four_sided_length_shorthands() {
    for (property, sides) in [(Property::Margin, MARGINS), (Property::Padding, PADDINGS)] {
        for (css, expected) in [
            ("1px", ["1px", "1px", "1px", "1px"]),
            ("1px 2px", ["1px", "2px", "1px", "2px"]),
            ("1px 2px 3px", ["1px", "2px", "3px", "2px"]),
            ("1px 2px 3px 4px", ["1px", "2px", "3px", "4px"]),
        ] {
            let source = declaration(property, css, CssImportance::Normal);
            let current = source.known().unwrap().property_value().unwrap();
            let assigned = match current {
                CssKnownPropertyValueRef::Margin(value) => {
                    assert_eq!(value.as_css(), css);
                    value
                        .value()
                        .assigned_values()
                        .map(|side| side.serialize_specified().unwrap())
                }
                CssKnownPropertyValueRef::Padding(value) => {
                    assert_eq!(value.as_css(), css);
                    value
                        .value()
                        .assigned_values()
                        .map(|side| side.serialize_specified().unwrap())
                }
                other => panic!("expected checked box spacing: {other:?}"),
            };
            assert_eq!(assigned, expected);
            assert_eq!(sides.len(), assigned.len());
            assert_eq!(
                expand_declaration(&source).unwrap_err().kind(),
                &CssExpansionErrorKind::UnresolvedStandard {
                    property,
                    reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
                }
            );
        }
    }
    for (css, expected) in [
        ("1px", ["1", "1", "1", "1"]),
        ("1px 2px", ["1", "2", "1", "2"]),
        ("1px 2px 3px", ["1", "2", "3", "2"]),
        ("1px 2px 3px 4px", ["1", "2", "3", "4"]),
    ] {
        let source = declaration(Property::BorderWidth, css, CssImportance::Normal);
        let Some(CssKnownPropertyValueRef::BorderWidth(authored)) =
            source.known().unwrap().property_value()
        else {
            panic!("typed border widths")
        };
        assert_eq!(
            authored
                .value()
                .assigned_values()
                .map(|width| width.serialize_specified().unwrap()),
            expected.map(|n| format!("{n}px"))
        );
        assert_eq!(
            expand_declaration(&source).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: Property::BorderWidth,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
    }
    let source = declaration(Property::Margin, "auto 10% -3px", CssImportance::Normal);
    let Some(CssKnownPropertyValueRef::Margin(authored)) = source.known().unwrap().property_value()
    else {
        panic!("authored margin");
    };
    assert_eq!(
        authored
            .value()
            .assigned_values()
            .map(|side| side.serialize_specified().unwrap()),
        ["auto", "10%", "-3px", "10%"]
    );
    assert_eq!(authored.as_css(), "auto 10% -3px");
    assert_eq!(
        expand_declaration(&source).unwrap_err().kind(),
        &CssExpansionErrorKind::UnresolvedStandard {
            property: Property::Margin,
            reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
        }
    );
    println!("four-sided authored values and reset boundary: ok");
}

fn four_sided_styles_and_current_colors() {
    for (css, expected) in [
        ("solid", [CssBorderStyle::Solid; 4]),
        (
            "solid dashed",
            [
                CssBorderStyle::Solid,
                CssBorderStyle::Dashed,
                CssBorderStyle::Solid,
                CssBorderStyle::Dashed,
            ],
        ),
        (
            "solid dashed dotted",
            [
                CssBorderStyle::Solid,
                CssBorderStyle::Dashed,
                CssBorderStyle::Dotted,
                CssBorderStyle::Dashed,
            ],
        ),
        (
            "solid dashed dotted double",
            [
                CssBorderStyle::Solid,
                CssBorderStyle::Dashed,
                CssBorderStyle::Dotted,
                CssBorderStyle::Double,
            ],
        ),
    ] {
        let source = declaration(Property::BorderStyle, css, CssImportance::Normal);
        let Some(CssKnownPropertyValueRef::BorderStyle(authored)) =
            source.known().unwrap().property_value()
        else {
            panic!("authored border style")
        };
        assert_eq!(
            authored.value().kind(),
            surgeist_css::CssBoxSideKind::Physical
        );
        assert_eq!(
            authored.value().assigned_values().map(|value| *value),
            expected
        );
        assert_eq!(
            expand_declaration(&source).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: Property::BorderStyle,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
    }
    for (css, expected) in [
        ("red", ["red", "red", "red", "red"]),
        ("red blue", ["red", "blue", "red", "blue"]),
        ("red blue green", ["red", "blue", "green", "blue"]),
        ("red blue green black", ["red", "blue", "green", "black"]),
    ] {
        let source = declaration(Property::BorderColor, css, CssImportance::Normal);
        let Some(CssKnownPropertyValueRef::BorderColor(authored)) =
            source.known().unwrap().property_value()
        else {
            panic!("authored border-color")
        };
        assert_eq!(
            authored.value().kind(),
            surgeist_css::CssBoxSideKind::Physical
        );
        for (side, expected) in authored.value().assigned_values().into_iter().zip(expected) {
            assert_eq!(side.named().unwrap().name(), expected);
        }
        assert_eq!(
            expand_declaration(&source).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: Property::BorderColor,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
    }
    let source = declaration(
        Property::BorderColor,
        &format!("{MODERN_COLOR} currentcolor"),
        CssImportance::Normal,
    );
    let Some(CssKnownPropertyValueRef::BorderColor(authored)) =
        source.known().unwrap().property_value()
    else {
        panic!("authored current border-color");
    };
    let [top, right, bottom, left] = authored.value().assigned_values();
    assert_modern_color(top);
    assert_modern_color(bottom);
    assert!(right.is_current_color());
    assert!(left.is_current_color());
    println!("four-sided styles and current colors: ok");
}

fn side_borders_supply_defaults_without_image_resets() {
    for index in 0..4 {
        for (css, expected_width, expected_style, modern) in [
            (
                "solid".to_owned(),
                CssBorderWidth::Medium,
                CssBorderStyle::Solid,
                false,
            ),
            (
                "2px".to_owned(),
                border_px("2"),
                CssBorderStyle::None,
                false,
            ),
            (
                MODERN_COLOR.to_owned(),
                CssBorderWidth::Medium,
                CssBorderStyle::None,
                true,
            ),
            (
                format!("{MODERN_COLOR} dashed 3px"),
                border_px("3"),
                CssBorderStyle::Dashed,
                true,
            ),
        ] {
            let source = declaration(SIDE_BORDERS[index], &css, CssImportance::Normal);
            let values = expanded(&source);
            assert_members(&values, &[WIDTHS[index], STYLES[index], COLORS[index]]);
            assert_eq!(length(member(&values, WIDTHS[index])), &expected_width);
            assert_eq!(style(member(&values, STYLES[index])), expected_style);
            let value = color(member(&values, COLORS[index]));
            if modern {
                assert_modern_color(value);
            } else {
                assert!(value.is_current_color());
            }
        }
    }
    println!("side border defaults: ok");
}

fn full_border_resets_all_five_image_values() {
    for (css, expected_width, expected_style, modern) in [
        (
            "none".to_owned(),
            CssBorderWidth::Medium,
            CssBorderStyle::None,
            false,
        ),
        (
            "solid".to_owned(),
            CssBorderWidth::Medium,
            CssBorderStyle::Solid,
            false,
        ),
        (
            format!("2px dashed {MODERN_COLOR}"),
            border_px("2"),
            CssBorderStyle::Dashed,
            true,
        ),
    ] {
        let source = declaration(Property::Border, &css, CssImportance::Normal);
        if modern {
            let Some(CssKnownPropertyValueRef::Border(authored)) =
                source.known().unwrap().property_value()
            else {
                panic!("authored border");
            };
            let border = authored.value();
            assert_eq!(border.style(), Some(CssBorderStyle::Dashed));
        }
        let values = expanded(&source);
        assert_members(&values, &border_members());
        for index in 0..4 {
            assert_eq!(length(member(&values, WIDTHS[index])), &expected_width);
            assert_eq!(style(member(&values, STYLES[index])), expected_style);
            let value = color(member(&values, COLORS[index]));
            if modern {
                assert_modern_color(value);
            } else {
                assert!(value.is_current_color());
            }
        }
        assert_image_initials(&values);
    }
    println!("full border image resets: ok");
}

fn ordinary_longhands_retain_typed_values() {
    for property in MARGINS.into_iter().chain(PADDINGS) {
        let values = expanded(&declaration(property, "7px", CssImportance::Normal));
        assert_members(&values, &[property]);
        assert_eq!(spacing(member(&values, property)), "7px");
    }
    for property in WIDTHS {
        let values = expanded(&declaration(property, "7px", CssImportance::Normal));
        assert_members(&values, &[property]);
        assert_eq!(length(member(&values, property)), &border_px("7"));
    }
    for property in STYLES {
        let values = expanded(&declaration(property, "dashed", CssImportance::Normal));
        assert_members(&values, &[property]);
        assert_eq!(style(member(&values, property)), CssBorderStyle::Dashed);
    }
    for property in COLORS {
        let values = expanded(&declaration(property, MODERN_COLOR, CssImportance::Normal));
        assert_members(&values, &[property]);
        assert_modern_color(color(member(&values, property)));
    }
    for (property, css) in [
        (Property::BorderImageSource, "url(example.svg)"),
        (Property::BorderImageSlice, "7% 11 13% 17 fill"),
        (Property::BorderImageWidth, "2 auto 3% 4px"),
        (Property::BorderImageOutset, "1 2px 3 4px"),
        (Property::BorderImageRepeat, "round space"),
    ] {
        let values = expanded(&declaration(property, css, CssImportance::Normal));
        assert_members(&values, &[property]);
        match member(&values, property).value() {
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageSource(
                CssImageValue::Url(url),
            )) => {
                assert_eq!(property, Property::BorderImageSource);
                assert_eq!(url.as_str(), "example.svg");
            }
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageSlice(value)) => {
                assert_eq!(property, Property::BorderImageSlice);
                assert!(value.fill());
                for (component, (percentage, expected)) in value.values().iter().zip([
                    (true, "7%"),
                    (false, "11"),
                    (true, "13%"),
                    (false, "17"),
                ]) {
                    match component {
                        CssBorderImageSliceComponent::Percentage(number) if percentage => {
                            assert!(exact_literal(number.literal_component(), expected))
                        }
                        CssBorderImageSliceComponent::Number(number) if !percentage => {
                            assert!(exact_literal(number.literal_component(), expected))
                        }
                        other => panic!("preserved slice component: {other:?}"),
                    }
                }
            }
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageWidth(value)) => {
                assert_eq!(property, Property::BorderImageWidth);
                let [first, second, third, fourth] = value.values();
                assert!(
                    matches!(first, CssBorderImageWidthComponent::Number(number) if exact_literal(number.literal_component(), "2"))
                );
                assert!(matches!(second, CssBorderImageWidthComponent::Auto));
                assert!(
                    matches!(third, CssBorderImageWidthComponent::LengthPercentage(value) if exact_literal(value.literal_component(), "3%"))
                );
                assert!(
                    matches!(fourth, CssBorderImageWidthComponent::LengthPercentage(value) if exact_literal(value.literal_component(), "4px"))
                );
            }
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageOutset(value)) => {
                assert_eq!(property, Property::BorderImageOutset);
                let [first, second, third, fourth] = value.values();
                assert!(
                    matches!(first, CssBorderImageOutsetComponent::Number(number) if exact_literal(number.literal_component(), "1"))
                );
                assert!(
                    matches!(second, CssBorderImageOutsetComponent::Length(value) if exact_literal(value.literal_component(), "2px"))
                );
                assert!(
                    matches!(third, CssBorderImageOutsetComponent::Number(number) if exact_literal(number.literal_component(), "3"))
                );
                assert!(
                    matches!(fourth, CssBorderImageOutsetComponent::Length(value) if exact_literal(value.literal_component(), "4px"))
                );
            }
            CssContributionValueRef::Ordinary(CssLonghandValueRef::BorderImageRepeat(value)) => {
                assert_eq!(property, Property::BorderImageRepeat);
                assert_eq!(value.horizontal(), CssBorderImageRepeatKeyword::Round);
                assert_eq!(value.vertical(), CssBorderImageRepeatKeyword::Space);
            }
            other => panic!("exact image longhand value: {other:?}"),
        }
    }
    println!("ordinary longhands: ok");
}

fn globals_include_reset_only_members_and_all_stays_symbolic() {
    for (css, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for (property, expected) in [
            (Property::MarginBlock, MARGIN_BLOCK.to_vec()),
            (Property::PaddingBlock, PADDING_BLOCK.to_vec()),
            (
                Property::BorderBlockColor,
                vec![
                    Property::BorderBlockStartColor,
                    Property::BorderBlockEndColor,
                ],
            ),
            (Property::Border, border_members()),
            (
                Property::BorderLeft,
                vec![
                    Property::BorderLeftWidth,
                    Property::BorderLeftStyle,
                    Property::BorderLeftColor,
                ],
            ),
            (Property::MarginLeft, vec![Property::MarginLeft]),
            (Property::BorderImageSlice, vec![Property::BorderImageSlice]),
        ] {
            let source = declaration(property, css, CssImportance::Important);
            let values = expanded(&source);
            assert_members(&values, &expected);
            for item in values.items() {
                assert!(
                    matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                );
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        let border_width = declaration(Property::BorderWidth, css, CssImportance::Important);
        assert_eq!(
            expand_declaration(&border_width).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: Property::BorderWidth,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
        let border_style = declaration(Property::BorderStyle, css, CssImportance::Important);
        assert_eq!(
            expand_declaration(&border_style).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: Property::BorderStyle,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
        let border_color = declaration(Property::BorderColor, css, CssImportance::Important);
        assert_eq!(
            expand_declaration(&border_color).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: Property::BorderColor,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
        let source = declaration(Property::All, css, CssImportance::Important);
        let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("all remains a symbolic universal reset");
        };
        assert_eq!(reset.keyword(), keyword);
        assert!(reset.source().same_occurrence(&source));
        assert!(reset.replacement_components().is_none());
        for property in [Property::Direction, Property::UnicodeBidi] {
            assert!(reset.excludes(CssPropertyNameRef::Known(property)));
        }
        for name in ["--theme", "--direction", "--UnicodeBidi"] {
            let name = CssCustomPropertyName::try_new(name).unwrap();
            assert!(reset.excludes(CssPropertyNameRef::Custom(&name)));
        }
        for property in [
            Property::MarginTop,
            Property::BorderImageSource,
            Property::TextAlign,
            Property::Display,
        ] {
            assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
        }
    }
    println!("global and universal resets: ok");
}

fn contribution_sources_preserve_occurrence_and_importance() {
    let css = "/*😀*/ margin-block: 1px 2px !important";
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let source = report.syntax()[0].clone();
    let source_clone = source.clone();
    let values = expanded(&source);
    for item in values.items() {
        assert!(item.source().same_occurrence(&source_clone));
        assert_eq!(item.source().importance(), CssImportance::Important);
        let position = item.source().position().expect("parsed source position");
        assert_eq!(position.byte_offset().value(), css.find("margin").unwrap());
        assert_eq!(
            position.column().value() as usize,
            css[..css.find("margin").unwrap()].encode_utf16().count()
        );
        assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
        assert!(
            item.source()
                .parsed_value()
                .unwrap()
                .source()
                .same_snapshot(source.parsed_value().unwrap().source())
        );
    }
    let first = declaration(
        Property::MarginBlock,
        "var(--gap)",
        CssImportance::Important,
    );
    let separate = declaration(
        Property::MarginBlock,
        "var(--gap)",
        CssImportance::Important,
    );
    assert_eq!(first, separate);
    assert!(!first.same_occurrence(&separate));
    let first_pending = pending_expansion(&first);
    let cloned_pending = pending_expansion(&first.clone());
    let separate_pending = pending_expansion(&separate);
    assert!(
        first_pending
            .source()
            .same_occurrence(cloned_pending.source())
    );
    assert!(
        !first_pending
            .source()
            .same_occurrence(separate_pending.source())
    );
    assert_eq!(
        first_pending.source().importance(),
        CssImportance::Important
    );
    assert!(first_pending.source().position().is_none());
    println!("source and occurrence identity: ok");
}

fn pending_reentry_preserves_original_and_replacement_provenance() {
    let source_text = "margin-block: var(--gap) !important";
    let report = parse_style_attribute(source_text);
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let pending = pending_expansion(source);
    let first = parse_component_values("7px").unwrap();
    let second = parse_component_values("11%").unwrap();
    let replacement = CssComponentValues::try_new(vec![
        first.items()[0].clone(),
        CssComponentValue::try_token(" ").unwrap(),
        second.items()[0].clone(),
    ])
    .unwrap();
    let completed: CssContributions = pending.reenter(replacement.clone()).unwrap();
    let CssContributions::Longhands(values) = completed else {
        panic!("completed margin members");
    };
    assert_members(&values, &MARGIN_BLOCK);
    assert_eq!(spacing(member(&values, Property::MarginBlockStart)), "7px");
    assert_eq!(spacing(member(&values, Property::MarginBlockEnd)), "11%");
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert!(
            item.source()
                .known()
                .unwrap()
                .substitution_dependent()
                .is_some()
        );
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(
            item.source().parsed_name().unwrap().source().as_str(),
            source_text
        );
        let actual = item
            .replacement_components()
            .expect("retained replacement input");
        assert_eq!(actual.items().len(), 3);
        for (actual, expected) in actual.items().iter().zip(replacement.items()) {
            assert_eq!(actual.origin(), expected.origin());
            if let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
                (actual.origin(), expected.origin())
            {
                assert!(actual.source().same_snapshot(expected.source()));
            }
        }
        assert_eq!(actual.serialize().unwrap().as_css(), "7px 11%");
    }
    for property in [Property::Border, Property::All] {
        let source = declaration(property, "var(--reset)", CssImportance::Important);
        let pending = pending_expansion(&source);
        let replacement = parse_component_values("revert-layer").unwrap();
        let result: CssContributions = pending.reenter(replacement.clone()).unwrap();
        match result {
            CssContributions::Longhands(values) if property == Property::Border => {
                assert_members(&values, &border_members());
                for item in values.items() {
                    assert!(matches!(
                        item.value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
                    ));
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(
                        item.replacement_components().unwrap().items()[0].origin(),
                        replacement.items()[0].origin()
                    );
                }
            }
            CssContributions::UniversalReset(reset) if property == Property::All => {
                assert_eq!(reset.keyword(), CssGlobalKeyword::RevertLayer);
                assert!(reset.source().same_occurrence(&source));
                assert_eq!(
                    reset.replacement_components().unwrap().items()[0].origin(),
                    replacement.items()[0].origin()
                );
            }
            other => panic!("completed keyword contribution: {other:?}"),
        }
    }
    println!("strict reentry success and origins: ok");
}

fn strict_reentry_rejects_atomically_and_preserves_out_of_slice_identity() {
    let source = declaration(Property::PaddingBlock, "var(--gap)", CssImportance::Normal);
    let pending = pending_expansion(&source);
    for css in [
        "var(--still-pending)",
        "1px var(--still-pending)",
        "calc(var(--still-pending) + 1px)",
    ] {
        let error = pending
            .reenter(parse_component_values(css).unwrap())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::ResidualSubstitution
        ));
        assert!(pending.source().same_occurrence(&source));
        let completed: CssContributions = pending
            .reenter(parse_component_values("3px").unwrap())
            .unwrap();
        let CssContributions::Longhands(values) = completed else {
            panic!("complete valid retry after residual substitution");
        };
        assert_members(&values, &PADDING_BLOCK);
        for property in PADDING_BLOCK {
            assert_eq!(spacing(member(&values, property)), "3px");
        }
    }
    for (css, responsible) in [
        ("-1px", Some("-1px")),
        ("1px blue", Some("blue")),
        ("1px 2px 3px", None),
        ("1px;color:red", Some(";")),
        ("1px!important", Some("!")),
    ] {
        let replacement = parse_component_values(css).unwrap();
        let expected_error = parse_property_value(
            CssPropertyNameRef::Known(Property::PaddingBlock),
            replacement.clone(),
            CssImportance::Normal,
        )
        .expect_err("the owning property grammar rejects this replacement");
        let error = pending.reenter(replacement).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
            panic!("replacement grammar rejection: {error:?}");
        };
        assert_eq!(
            error, &expected_error,
            "strict reentry preserves the owner's complete mapped error"
        );
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        if let Some(responsible) = responsible {
            let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
                panic!("responsible replacement token: {error:?}");
            };
            assert_eq!(origin.source().as_str(), css);
            let span = origin.span();
            assert_eq!(
                &css[span.start().byte_offset().value()..span.end().byte_offset().value()],
                responsible
            );
        }
        assert!(pending.source().same_occurrence(&source));
        let completed: CssContributions = pending
            .reenter(parse_component_values("3px").unwrap())
            .unwrap();
        let CssContributions::Longhands(values) = completed else {
            panic!("complete valid retry");
        };
        assert_members(&values, &PADDING_BLOCK);
        for property in PADDING_BLOCK {
            assert_eq!(spacing(member(&values, property)), "3px");
        }
    }
    let source = declaration(Property::Cursor, "auto", CssImportance::Normal);
    let error = expand_declaration(&source).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssExpansionErrorKind::UnsupportedProperty(Property::Cursor)
    ));
    assert_eq!(source.known().unwrap().property(), Property::Cursor);
    let source = declaration(Property::TextAlign, "justify-all", CssImportance::Normal);
    let Some(CssKnownPropertyValueRef::TextAlign(value)) = source.known().unwrap().property_value()
    else {
        panic!("original text-align declaration");
    };
    assert_eq!(value.value(), &CssTextAlignValue::JustifyAll);
    assert_eq!(value.as_css(), "justify-all");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("two alignment targets")
    };
    let [all, last] = values.items() else {
        panic!("two alignment targets")
    };
    assert!(matches!(
        all.value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::TextAlignAll(
            CssTextAlignAllValue::Keyword(CssTextAlign::Justify)
        ))
    ));
    assert!(matches!(
        last.value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::TextAlignLast(
            CssTextAlignLastValue::Keyword(CssTextAlign::Justify)
        ))
    ));
    println!("strict rejection and text alignment expansion: ok");
}

// Variables 1 #defining-variables and #syntax keep specified custom values
// symbolic. The contribution carries the same authored occurrence, not an
// environment or an attempted variable-substitution result.
fn custom_properties_preserve_symbolic_values_and_occurrences() {
    for name in ["--Theme", "--theme", "--foó", "--foo\u{301}"] {
        let name = CssCustomPropertyName::try_new(name).unwrap();
        for value in [
            "",
            "Red",
            "var(--Theme)",
            "var(--missing, 1px)",
            "[a] / f(2)",
        ] {
            for importance in [CssImportance::Normal, CssImportance::Important] {
                let components = parse_component_values(value).unwrap();
                let checked = parse_property_value(
                    CssPropertyNameRef::Custom(&name),
                    components.clone(),
                    importance,
                )
                .unwrap();
                let suffix = if importance == CssImportance::Important {
                    "!important"
                } else {
                    ""
                };
                let source = format!("{}:{value}{suffix}", name.as_str());
                let report = parse_style_attribute(&source);
                assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
                for declaration in [&checked, &report.syntax()[0]] {
                    let CssExpansion::Contributions(CssContributions::Custom(contribution)) =
                        expand_declaration(declaration).unwrap()
                    else {
                        panic!("custom contribution");
                    };
                    assert!(contribution.source().same_occurrence(declaration));
                    assert_eq!(contribution.source().importance(), importance);
                    assert_eq!(contribution.declaration().name(), &name);
                    assert_eq!(
                        contribution.declaration().value().value().unwrap().as_css(),
                        value
                    );
                    assert!(std::ptr::eq(
                        contribution.source().value_components(),
                        declaration.value_components()
                    ));
                    assert_eq!(
                        contribution.source().parsed_value(),
                        declaration.parsed_value()
                    );
                    assert_eq!(contribution.source().position(), declaration.position());
                    let cloned = contribution.clone();
                    assert!(cloned.source().same_occurrence(declaration));
                }
                assert_eq!(checked.position(), None);
                assert_eq!(
                    checked.value_components().serialize().unwrap().as_css(),
                    value
                );
                for (actual, expected) in checked
                    .value_components()
                    .items()
                    .iter()
                    .zip(components.items())
                {
                    assert_eq!(actual.origin(), expected.origin());
                }
                assert_eq!(
                    report.syntax()[0].position().unwrap().byte_offset().value(),
                    0
                );
            }
        }
        for (css, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_ident(css).unwrap()])
                    .unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Custom(&name),
                components,
                CssImportance::Important,
            )
            .unwrap();
            let CssExpansion::Contributions(CssContributions::Custom(contribution)) =
                expand_declaration(&checked).unwrap()
            else {
                panic!("custom global contribution");
            };
            assert!(contribution.source().same_occurrence(&checked));
            assert_eq!(contribution.declaration().name(), &name);
            assert_eq!(contribution.declaration().value().global(), Some(keyword));
            assert!(contribution.declaration().value().value().is_none());
            assert_eq!(contribution.source().importance(), CssImportance::Important);
            assert_eq!(contribution.source().position(), None);
        }
    }
    println!("custom symbolic contributions: ok");
}

fn main() {
    four_sided_length_shorthands();
    four_sided_styles_and_current_colors();
    side_borders_supply_defaults_without_image_resets();
    full_border_resets_all_five_image_values();
    ordinary_longhands_retain_typed_values();
    globals_include_reset_only_members_and_all_stays_symbolic();
    contribution_sources_preserve_occurrence_and_importance();
    pending_reentry_preserves_original_and_replacement_provenance();
    strict_reentry_rejects_atomically_and_preserves_out_of_slice_identity();
    custom_properties_preserve_symbolic_values_and_occurrences();
}

fn exact_literal(component: Option<&surgeist_css::CssComponentValue>, css: &str) -> bool {
    use surgeist_css::{CssComponentValueRef as Component, CssValueTokenRef as Token};
    let expected = surgeist_css::CssComponentValue::try_token(css).unwrap();
    match (
        component.map(surgeist_css::CssComponentValue::view),
        expected.view(),
    ) {
        (
            Some(Component::Token(Token::Number(actual))),
            Component::Token(Token::Number(expected)),
        )
        | (
            Some(Component::Token(Token::Percentage(actual))),
            Component::Token(Token::Percentage(expected)),
        ) => actual.representation() == expected.representation(),
        (
            Some(Component::Token(Token::Dimension {
                number: actual,
                unit: actual_unit,
            })),
            Component::Token(Token::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => actual.representation() == expected.representation() && actual_unit == expected_unit,
        _ => false,
    }
}
