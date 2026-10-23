//! Fonts 4, Text 3/4 and Text Decoration 4 property families keep authored
//! syntax and reject cross-family values.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn rejects_line_height_auto() {
    let error = parse_sheet(".panel { line-height: auto; }").expect_err("rejected parse report");
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn rejects_line_height_min_content() {
    let error =
        parse_sheet(".panel { line-height: min-content; }").expect_err("rejected parse report");
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn rejects_font_size_auto() {
    let error = parse_sheet(".panel { font-size: auto; }").expect_err("rejected parse report");
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn parses_typography_and_text_keyword_families() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { writing-mode: vertical-rl; }",
                CssKnownProperty::WritingMode
            ),
            WritingMode,
            value
        ),
        CssWritingMode::VerticalRl
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { text-align: start; }", CssKnownProperty::TextAlign),
            TextAlign,
            value
        ),
        CssTextAlignValue::Alignment(CssTextAlignAllValue::Keyword(CssTextAlign::Start))
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-align-last: justify; }",
                CssKnownProperty::TextAlignLast
            ),
            TextAlignLast,
            value
        ),
        CssTextAlignLastValue::Keyword(CssTextAlign::Justify)
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { text-wrap: balance; }", CssKnownProperty::TextWrap),
            TextWrap,
            value
        ),
        CssTextWrap::try_new(None, Some(CssTextWrapStyle::Balance)).unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { white-space: pre-wrap; }",
                CssKnownProperty::WhiteSpace
            ),
            WhiteSpace,
            value
        ),
        CssWhiteSpace::from_keyword(CssWhiteSpaceKeyword::PreWrap)
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { word-break: keep-all; }",
                CssKnownProperty::WordBreak
            ),
            WordBreak,
            value
        ),
        CssWordBreak::KeepAll
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { overflow-wrap: anywhere; }",
                CssKnownProperty::OverflowWrap
            ),
            OverflowWrap,
            value
        ),
        CssOverflowWrap::Anywhere
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-overflow: ellipsis; }",
                CssKnownProperty::TextOverflow
            ),
            TextOverflow,
            value
        ),
        CssTextOverflow::Ellipsis
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-transform: uppercase; }",
                CssKnownProperty::TextTransform
            ),
            TextTransform,
            value
        ),
        CssTextTransform::Transforms(
            CssTextTransformSet::try_new(Some(CssTextTransformCase::Uppercase), false, false)
                .unwrap()
        )
    );
}

#[test]
fn parses_typography_and_text_length_families() {
    assert_eq!(
        property_payload!(
            declaration(".panel { text-indent: 2em; }", CssKnownProperty::TextIndent),
            TextIndent,
            value
        ),
        CssTextIndent::new(signed_length_percentage("2em"), false, false)
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { vertical-align: 4px; }",
                CssKnownProperty::VerticalAlign
            ),
            VerticalAlign,
            value
        ),
        CssVerticalAlign::Length(signed_length_percentage("4px"))
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { letter-spacing: normal; }",
                CssKnownProperty::LetterSpacing
            ),
            LetterSpacing,
            value
        ),
        CssTextSpacingAdjustment::Normal
    );
    let authored = declaration(
        ".panel { letter-spacing: 0.1em; }",
        CssKnownProperty::LetterSpacing,
    );
    let CssKnownPropertyValueRef::LetterSpacing(spacing) = authored
        .known()
        .and_then(|known| known.property_value())
        .expect("ordinary letter spacing")
    else {
        panic!("expected letter spacing");
    };
    assert_eq!(spacing.as_css(), "0.1em");
    assert!(matches!(
        spacing.value(),
        CssTextSpacingAdjustment::LengthPercentage(value)
            if value.serialize_specified(Default::default()).unwrap() == "0.1em"
    ));
    let CssTextSpacingAdjustment::LengthPercentage(length) = spacing.value() else {
        panic!("exact letter spacing")
    };
    assert_length_literal(length.literal_component().unwrap(), "0.1", "em");
    let spacing = property_payload!(
        declaration(
            ".panel { letter-spacing: 0.25em; }",
            CssKnownProperty::LetterSpacing
        ),
        LetterSpacing,
        value
    );
    let CssTextSpacingAdjustment::LengthPercentage(length) = spacing else {
        panic!("exact letter spacing")
    };
    assert_length_literal(length.literal_component().unwrap(), "0.25", "em");
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-decoration-thickness: from-font; }",
                CssKnownProperty::TextDecorationThickness
            ),
            TextDecorationThickness,
            value
        ),
        CssTextDecorationThickness::FromFont
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-decoration-thickness: 2px; }",
                CssKnownProperty::TextDecorationThickness
            ),
            TextDecorationThickness,
            value
        ),
        CssTextDecorationThickness::Length(signed_length_percentage("2px"))
    );
}

#[test]
fn parses_font_families_and_font_shorthand_as_authored_syntax() {
    let family_declaration = declaration(
        ".panel { font-family: \"Avenir Next\", Gill Sans, sans-serif; }",
        CssKnownProperty::FontFamily,
    );
    let CssKnownPropertyValueRef::FontFamily(value) = family_declaration
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected authored font-family");
    };
    let family = value.families();
    assert_eq!(
        family.families(),
        [
            CssFontFamilyName::try_quoted("Avenir Next").unwrap(),
            CssFontFamilyName::try_ident_sequence(vec!["Gill".into(), "Sans".into()]).unwrap(),
            CssFontFamilyName::generic(CssGenericFontFamily::SansSerif),
        ]
    );

    let weight_declaration =
        declaration(".panel { font-weight: 725; }", CssKnownProperty::FontWeight);
    let Some(CssKnownPropertyValueRef::FontWeight(weight)) = weight_declaration
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("expected authored font-weight");
    };
    assert_eq!(
        weight.value(),
        &CssFontWeight::Absolute(CssAbsoluteFontWeight::Number(weight_number("725")))
    );
    let style_declaration = declaration(
        ".panel { font-style: italic; }",
        CssKnownProperty::FontStyle,
    );
    let Some(CssKnownPropertyValueRef::FontStyle(style)) = style_declaration
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("expected authored font-style");
    };
    assert_eq!(
        style.value(),
        &CssFontStyle::Keyword(CssFontStyleKeyword::Italic)
    );
    let width_declaration = declaration(
        ".panel { font-stretch: semi-condensed; }",
        CssKnownProperty::FontWidth,
    );
    let Some(CssKnownPropertyValueRef::FontWidth(width)) = width_declaration
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("expected authored font-width");
    };
    assert_eq!(
        width.value(),
        &CssFontWidth::Keyword(CssFontWidthKeyword::SemiCondensed)
    );
    let variant_declaration = declaration(
        ".panel { font-variant: small-caps; }",
        CssKnownProperty::FontVariant,
    );
    let Some(CssKnownPropertyValueRef::FontVariant(variant)) = variant_declaration
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("expected authored font-variant");
    };
    let CssFontVariantValue::Values(values) = variant.variant() else {
        panic!("expected small-caps subgroup");
    };
    assert_eq!(values.caps(), Some(CssFontVariantCaps::SmallCaps));
    let feature_declaration = declaration(
        ".panel { font-feature-settings: \"kern\" on, \"liga\" 0; }",
        CssKnownProperty::FontFeatureSettings,
    );
    let Some(CssKnownPropertyValueRef::FontFeatureSettings(features)) = feature_declaration
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("expected authored font-feature-settings");
    };
    let CssAuthoredFontFeatureSettings::Features(list) = features.settings() else {
        panic!("expected feature list");
    };
    assert_eq!(list.features()[0].tag().as_str(), "kern");
    assert!(matches!(
        list.features()[0].value(),
        CssAuthoredFontFeatureValue::On
    ));
    assert_eq!(list.features()[1].tag().as_str(), "liga");
    assert!(
        matches!(list.features()[1].value(), CssAuthoredFontFeatureValue::Index(index) if index.i32_value() == Some(0))
    );

    let font_declaration = declaration(
        ".panel { font: italic small-caps 700 condensed 16px/normal \"Avenir Next\", sans-serif; }",
        CssKnownProperty::Font,
    );
    let CssKnownPropertyValueRef::Font(value) =
        font_declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected authored font shorthand");
    };
    let CssFontValue::Explicit(font) = value.font() else {
        panic!("expected an explicit font");
    };
    assert_eq!(
        font.style(),
        Some(&CssFontStyle::Keyword(CssFontStyleKeyword::Italic))
    );
    assert_eq!(font.variant(), Some(CssFontVariant::SmallCaps));
    assert_eq!(
        font.weight(),
        Some(&CssFontWeight::Absolute(CssAbsoluteFontWeight::Number(
            weight_number("700")
        )))
    );
    assert_eq!(font.stretch(), Some(CssFontWidthKeyword::Condensed));
    assert_eq!(
        font.size(),
        &CssFontSize::LengthPercentage(
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                CssComponentValue::try_token("16px").unwrap()
            )
            .unwrap()
        )
    );
    assert_eq!(font.line_height(), Some(&CssLineHeight::Normal));
    assert_eq!(
        font.families().families(),
        [
            CssFontFamilyName::try_quoted("Avenir Next").unwrap(),
            CssFontFamilyName::generic(CssGenericFontFamily::SansSerif),
        ]
    );
}

#[test]
fn parses_text_decoration_family() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-decoration-line: underline overline; }",
                CssKnownProperty::TextDecorationLine
            ),
            TextDecorationLine,
            value
        ),
        CssTextDecorationLine::try_new(vec![
            CssTextDecorationLineComponent::Underline,
            CssTextDecorationLineComponent::Overline,
        ])
        .unwrap()
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-decoration-color: black; }",
                CssKnownProperty::TextDecorationColor
            ),
            TextDecorationColor,
            value
        ),
        CssColor::from_named(CssNamedColor::try_new("black").unwrap())
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { text-decoration-style: wavy; }",
                CssKnownProperty::TextDecorationStyle
            ),
            TextDecorationStyle,
            value
        ),
        CssTextDecorationStyle::Wavy
    );

    let value = property_payload!(
        declaration(
            ".panel { text-decoration: underline dotted white 3px; }",
            CssKnownProperty::TextDecoration
        ),
        TextDecoration,
        value
    );
    assert_eq!(
        value,
        CssTextDecoration::try_new(
            Some(
                CssTextDecorationLine::try_new(vec![CssTextDecorationLineComponent::Underline])
                    .unwrap()
            ),
            Some(CssColor::from_named(
                CssNamedColor::try_new("white").unwrap()
            )),
            Some(CssTextDecorationStyle::Dotted),
            Some(CssTextDecorationThickness::Length(
                signed_length_percentage("3px")
            )),
        )
        .unwrap()
    );
}

#[test]
fn checked_typography_constructors_reject_invalid_states() {
    assert_eq!(CssFontFamilyList::try_new(Vec::new()), None);
    assert!(
        CssFontWeightNumber::try_from_component(CssComponentValue::try_number("0").unwrap())
            .is_err()
    );
    assert!(
        CssFontWeightNumber::try_from_component(CssComponentValue::try_number("1001").unwrap())
            .is_err()
    );
    assert_eq!(
        CssFontWeightNumber::try_from_component(CssComponentValue::try_number("500").unwrap()),
        Ok(weight_number("500"))
    );
    assert_eq!(CssAuthoredFontFeatureList::try_new(Vec::new()), None);
    assert_eq!(CssTextDecorationLine::try_new(Vec::new()), None);
    assert!(
        CssExplicitFont::try_new(
            None,
            None,
            None,
            None,
            CssFontSize::LengthPercentage(
                CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                    CssComponentValue::try_token("12px").unwrap()
                )
                .unwrap()
            ),
            None,
            CssFontFamilyList::try_new(vec![CssFontFamilyName::generic(
                CssGenericFontFamily::SansSerif
            )])
            .unwrap(),
        )
        .is_some(),
    );
    assert!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_token("auto").unwrap()
        )
        .is_err()
    );
    let empty_quoted = CssFontFamilyName::try_quoted("").unwrap();
    assert_eq!(empty_quoted.as_str(), "");
    assert!(CssFontFamilyList::try_new(vec![empty_quoted]).is_some());
    assert_eq!(CssFontFamilyName::try_ident_sequence(Vec::new()), None);
    assert_eq!(
        CssFontFamilyName::try_ident_sequence(vec![String::new()]),
        None
    );
    assert!(CssOpenTypeTag::try_new("abc").is_none());
    assert!(CssOpenTypeTag::try_new("abcde").is_none());
    let feature = CssAuthoredFontFeature::new(
        CssOpenTypeTag::try_new("kern").unwrap(),
        CssAuthoredFontFeatureValue::On,
    );
    assert_eq!(feature.tag().as_str(), "kern");
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token("auto").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token("auto").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_dimension("-1", "px").unwrap()
        )
        .is_err()
    );
    assert_eq!(
        CssTextDecorationLine::try_new(vec![
            CssTextDecorationLineComponent::Underline,
            CssTextDecorationLineComponent::Underline,
        ]),
        None
    );
}

#[test]
fn typography_and_text_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            writing-mode: horizontal-tb;
            text-align: center;
            text-align-last: auto;
            text-indent: 1rem hanging each-line;
            vertical-align: super;
            font-family: \"Avenir Next\", sans-serif;
            font: italic 700 16px/normal \"Avenir Next\", sans-serif;
            font-weight: bold;
            font-style: oblique;
            font-stretch: expanded;
            font-variant: normal;
            font-feature-settings: normal;
            letter-spacing: 1px;
            text-wrap: wrap;
            white-space: nowrap;
            word-break: break-word;
            overflow-wrap: break-word;
            text-overflow: clip;
            text-decoration: underline solid black 1px;
            text-decoration-line: none;
            text-decoration-color: transparent;
            text-decoration-style: solid;
            text-decoration-thickness: auto;
            text-transform: capitalize;
        }",
    )
    .expect("clean parse report");
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssKnownProperty::WritingMode,
        CssKnownProperty::TextAlign,
        CssKnownProperty::TextAlignLast,
        CssKnownProperty::TextIndent,
        CssKnownProperty::VerticalAlign,
        CssKnownProperty::FontFamily,
        CssKnownProperty::Font,
        CssKnownProperty::FontWeight,
        CssKnownProperty::FontStyle,
        CssKnownProperty::FontWidth,
        CssKnownProperty::FontVariant,
        CssKnownProperty::FontFeatureSettings,
        CssKnownProperty::LetterSpacing,
        CssKnownProperty::TextWrap,
        CssKnownProperty::WhiteSpace,
        CssKnownProperty::WordBreak,
        CssKnownProperty::OverflowWrap,
        CssKnownProperty::TextOverflow,
        CssKnownProperty::TextDecoration,
        CssKnownProperty::TextDecorationLine,
        CssKnownProperty::TextDecorationColor,
        CssKnownProperty::TextDecorationStyle,
        CssKnownProperty::TextDecorationThickness,
        CssKnownProperty::TextTransform,
    ] {
        assert!(
            declarations.iter().any(|declaration| declaration
                .known()
                .map(|known| known.property())
                == Some(property)),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn typography_and_text_properties_reject_cross_family_values() {
    for input in [
        ".panel { font-size: auto; }",
        ".panel { font-weight: 1001; }",
        ".panel { font-style: bold; }",
        ".panel { font-family:; }",
        ".panel { letter-spacing: auto; }",
        ".panel { text-decoration-style: 2px; }",
        ".panel { text-transform: wrap; }",
        ".panel { font-feature-settings: \"abc\" on; }",
        ".panel { font-feature-settings: \"abcde\" on; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}
