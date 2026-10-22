use super::color::parse_color;
use super::combinators::{Combinator, Tail, commit, fill, keyword, parse_group};
use super::property_input::Parser;
use super::values::{parse_hinted_number_calculation, parse_length_percentage};
use crate::error::property::{CssFeatureId, Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, LiteralTokens, NumericType, next_is_comma, parse_numeric_function,
    parse_numeric_type,
};
use crate::font_variant::CssFontVariant;
use crate::syntax::*;
use crate::{
    CssAbsoluteFontWeight, CssFontObliqueAngle, CssFontSize, CssFontStyle, CssFontStyleKeyword,
    CssFontSynthesis, CssFontSynthesisPosition, CssFontSynthesisSmallCaps, CssFontSynthesisStyle,
    CssFontSynthesisValues, CssFontSynthesisWeight, CssFontWeight, CssFontWeightNumber,
    CssFontWidth, CssFontWidthKeyword, CssLineHeight, CssSpecifiedLengthPercentage,
    CssSpecifiedNonNegativeNumber, CssTextSpacingAdjustment,
};

pub(super) static IMPLEMENTED_PROPERTY_EXTENSIONS: &[CssFeatureId] =
    &[CssFeatureId::new("ext.property.font-weight-range")];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.opentype-tag")];

pub(super) fn parse_caret_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssCaretColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssCaretColor::Auto);
    }
    let color = parse_color(input, numeric)?;
    Ok(CssCaretColor::Color(Box::new(color)))
}

pub(super) fn parse_font_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontSize, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "xx-small" => Ok(CssFontSize::XxSmall),
            "x-small" => Ok(CssFontSize::XSmall),
            "small" => Ok(CssFontSize::Small),
            "medium" => Ok(CssFontSize::Medium),
            "large" => Ok(CssFontSize::Large),
            "x-large" => Ok(CssFontSize::XLarge),
            "xx-large" => Ok(CssFontSize::XxLarge),
            "xxx-large" => Ok(CssFontSize::XxxLarge),
            "larger" => Ok(CssFontSize::Larger),
            "smaller" => Ok(CssFontSize::Smaller),
            "math" => Ok(CssFontSize::Math),
            _ => Err(unexpected_at(keyword_location)),
        };
    }

    super::values::parse_nonnegative_length_percentage(input, numeric)
        .map(CssFontSize::LengthPercentage)
}

pub(super) fn parse_line_height<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssLineHeight, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssLineHeight::Normal);
    }

    input.skip_whitespace();

    if let Ok(calculation) =
        input.try_parse(|input| parse_hinted_number_calculation(input, numeric))
    {
        return Ok(CssLineHeight::HintedNumberCalculation(calculation));
    }

    if let Ok(number) = input.try_parse(|input| parse_line_height_number(input, numeric)) {
        return Ok(CssLineHeight::Number(number));
    }

    super::values::parse_nonnegative_length_percentage(input, numeric)
        .map(CssLineHeight::LengthPercentage)
}

fn parse_line_height_number<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSpecifiedNonNegativeNumber, ParseError<'i, Error>> {
    super::values::parse_nonnegative_number(input, numeric)
}

pub(super) fn parse_writing_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWritingMode, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "horizontal-tb" => Ok(CssWritingMode::HorizontalTb),
        "vertical-rl" => Ok(CssWritingMode::VerticalRl),
        "vertical-lr" => Ok(CssWritingMode::VerticalLr),
        "sideways-rl" => Ok(CssWritingMode::SidewaysRl),
        "sideways-lr" => Ok(CssWritingMode::SidewaysLr),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_text_combine_upright<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextCombineUpright, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssTextCombineUpright::None),
        "all" => Ok(CssTextCombineUpright::All),
        "digits" => {
            if input.is_exhausted() {
                return Ok(CssTextCombineUpright::Digits(None));
            }
            let start = input.state();
            let location = input.current_source_location();
            let count = match input.next().map_err(basic)? {
                Token::Number { .. } => {
                    input.reset(&start);
                    let value = super::values::parse_integer_literal(input, numeric)?;
                    let count = crate::integer_value::exact_i32(value.numeric().representation())
                        .and_then(CssTextCombineDigitCount::try_literal);
                    count.ok_or_else(|| unexpected_at(location))?
                }
                Token::Function(name) if crate::numeric::is_math_function(name) => {
                    let expression = parse_numeric_function(input, &start, numeric, CalculationRoot::Integer)?;
                    CssTextCombineDigitCount::from_calculation(CssIntegerCalculation::from_expression(expression))
                }
                token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
            };
            Ok(CssTextCombineUpright::Digits(Some(count)))
        },
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_text_orientation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextOrientation, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "mixed" => Ok(CssTextOrientation::Mixed),
        "upright" => Ok(CssTextOrientation::Upright),
        "sideways" => Ok(CssTextOrientation::Sideways),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_unicode_bidi<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUnicodeBidi, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssUnicodeBidi::Normal),
        "embed" => Ok(CssUnicodeBidi::Embed),
        "isolate" => Ok(CssUnicodeBidi::Isolate),
        "bidi-override" => Ok(CssUnicodeBidi::BidiOverride),
        "isolate-override" => Ok(CssUnicodeBidi::IsolateOverride),
        "plaintext" => Ok(CssUnicodeBidi::Plaintext),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_glyph_orientation_vertical<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextOrientation, ParseError<'i, Error>> {
    let start = input.state();
    let location = input.current_source_location();
    let value = match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
            Some(CssTextOrientation::Mixed)
        }
        Token::Number {
            int_value: Some(0), ..
        } => Some(CssTextOrientation::Upright),
        Token::Number {
            int_value: Some(90),
            ..
        } => Some(CssTextOrientation::Sideways),
        Token::Dimension { unit, .. } if unit.eq_ignore_ascii_case("deg") => {
            input.reset(&start);
            input.skip_whitespace();
            let offset = input.position();
            let component = input.collect_numeric_component(numeric).map_err(|error| {
                unexpected_at(numeric.property_error_location(&error, location, offset))
            })?;
            match component.view() {
                crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension {
                    number,
                    ..
                }) => match crate::exact_decimal::exact_binary32_value(number.representation()) {
                    Some(0.0) => Some(CssTextOrientation::Upright),
                    Some(90.0) => Some(CssTextOrientation::Sideways),
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    };
    value.ok_or_else(|| unexpected_at(location))
}

pub(super) fn parse_text_indent<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextIndent, ParseError<'i, Error>> {
    // `<length-percentage> && hanging? && each-line?`; any non-identifier
    // commits the length.
    let (mut hanging, mut each_line, mut length) = (None, None, None);
    let [hanging, each_line, _] = parse_group::<3>(
        input,
        Combinator::AllOf {
            optional: [true, true, false],
        },
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => keyword(&mut hanging, input, &[("hanging", ())]),
            1 => keyword(&mut each_line, input, &[("each-line", ())]),
            2 => commit(
                &mut length,
                input,
                |input| {
                    let state = input.state();
                    let ident = input.expect_ident().is_ok();
                    input.reset(&state);
                    !ident
                },
                |input| parse_length_percentage(input, numeric),
            ),
            _ => unreachable!("three text-indent operands"),
        },
    )?;
    let length = length.expect("required text-indent length");
    Ok(CssTextIndent::new(length, hanging, each_line))
}

pub(super) fn parse_vertical_align<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssVerticalAlign, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "baseline" => Ok(CssVerticalAlign::Baseline),
            "sub" => Ok(CssVerticalAlign::Sub),
            "super" => Ok(CssVerticalAlign::Super),
            "text-top" => Ok(CssVerticalAlign::TextTop),
            "text-bottom" => Ok(CssVerticalAlign::TextBottom),
            "middle" => Ok(CssVerticalAlign::Middle),
            "top" => Ok(CssVerticalAlign::Top),
            "bottom" => Ok(CssVerticalAlign::Bottom),
            _ => Err(unexpected_at(keyword_location)),
        };
    }

    parse_length_percentage(input, numeric).map(CssVerticalAlign::Length)
}

pub(super) fn parse_font_family_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFamilyList, ParseError<'i, Error>> {
    let families = input.parse_comma_separated(|input| parse_font_family_name(input))?;

    CssFontFamilyList::try_new(families)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_font_family_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFamilyName, ParseError<'i, Error>> {
    parse_font_family_name_with_generics(input, true)
}

pub(super) fn parse_non_generic_font_family_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFamilyName, ParseError<'i, Error>> {
    parse_font_family_name_with_generics(input, false)
}

fn parse_font_family_name_with_generics<'i, 't>(
    input: &mut Parser<'i, 't>,
    allow_generic: bool,
) -> std::result::Result<CssFontFamilyName, ParseError<'i, Error>> {
    if let Ok(name) = input.try_parse(Parser::expect_string_cloned) {
        return CssFontFamilyName::try_quoted(name.to_string())
            .ok_or_else(|| unexpected_at(input.current_source_location()));
    }

    if allow_generic
        && input
            .try_parse(|input| input.expect_function_matching("generic"))
            .is_ok()
    {
        return input.parse_nested_block(|input| {
            let keyword_location = input.current_source_location();
            let keyword = input.expect_ident_cloned().map_err(basic)?;
            let generic = CssGenericFontFamily::from_script_keyword(&keyword)
                .ok_or_else(|| unexpected_at(keyword_location))?;
            input.expect_exhausted().map_err(basic)?;
            Ok(CssFontFamilyName::generic(generic))
        });
    }

    let mut parts = Vec::new();
    let mut locations = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) {
        input.skip_whitespace();
        let location = input.current_source_location();
        match input.next().map_err(basic)? {
            Token::Ident(ident) => {
                parts.push(ident.to_string());
                locations.push(location);
            }
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }

    if allow_generic
        && let [keyword] = parts.as_slice()
        && let Some(generic) = CssGenericFontFamily::from_keyword(keyword)
    {
        return Ok(CssFontFamilyName::generic(generic));
    }

    // Fonts 4 <family-name>: an excluded identifier is responsible; one that
    // is excluded only alone (a generic family) is the whole sequence's start.
    let responsible = parts
        .iter()
        .position(|part| {
            CssFontFamilyName::try_ident_sequence(vec![part.clone(), "a".to_owned()]).is_none()
        })
        .or((!parts.is_empty()).then_some(0))
        .map(|index| locations[index]);
    let end = input.current_source_location();
    CssFontFamilyName::try_ident_sequence(parts)
        .ok_or_else(|| unexpected_at(responsible.unwrap_or(end)))
}

pub(super) fn parse_font<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontValue, ParseError<'i, Error>> {
    if let Ok(system) = input.try_parse(parse_system_font) {
        return Ok(CssFontValue::System(system));
    }

    // `[ <'font-style'> || <font-variant-css2> || <'font-weight'> ||
    // <font-width-css3> ]? <'font-size'>`. Every member admits `normal`, so up
    // to four `normal` operands are collected as ambiguous and assigned after
    // the explicit members; the group ends where a font size parses.
    let (mut style, mut variant, mut weight, mut stretch) = (None, None, None, None);
    let mut normals = [None; 4];
    // The boundary consumes the font size it finds, so the size parses once.
    let mut size = None;
    // The fifth prefix component in authored order: the group has four
    // members, so it is the first component no remaining member admits.
    let mut prefix_count = 0_usize;
    let mut fifth = None;
    let matched = parse_group::<8>(
        input,
        Combinator::AllOf {
            optional: [true; 8],
        },
        Tail::Reject,
        |input| {
            size = input
                .try_parse(|input| parse_font_size(input, numeric))
                .ok();
            size.is_some()
        },
        |operand, input| {
            let location = input.current_source_location();
            let matched = match operand {
                0..=3 => keyword(&mut normals[operand], input, &[("normal", ())]),
                4 => fill(&mut style, input, |input| parse_font_style(input, numeric)),
                5 => fill(&mut variant, input, parse_css2_font_variant),
                6 => fill(&mut weight, input, |input| {
                    parse_font_weight(input, numeric)
                }),
                7 => fill(&mut stretch, input, parse_font_stretch),
                _ => unreachable!("eight font prefix operands"),
            }?;
            if matched {
                prefix_count += 1;
                if prefix_count == 5 {
                    fifth = Some(location);
                }
            }
            Ok(matched)
        },
    )?;
    let Some(size) = size else {
        return Err(unexpected_at(input.current_source_location()));
    };

    for _ in matched[..4].iter().filter(|matched| **matched) {
        if style.is_none() {
            style = Some(CssFontStyle::Keyword(CssFontStyleKeyword::Normal));
        } else if variant.is_none() {
            variant = Some(CssFontVariant::Normal);
        } else if weight.is_none() {
            weight = Some(CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal));
        } else if stretch.is_none() {
            stretch = Some(CssFontWidthKeyword::Normal);
        } else {
            return Err(unexpected_at(
                fifth.unwrap_or_else(|| input.current_source_location()),
            ));
        }
    }

    let line_height = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_line_height(input, numeric)?)
    } else {
        None
    };
    let families = parse_font_family_list(input)?;

    CssExplicitFont::try_new(style, variant, weight, stretch, size, line_height, families)
        .map(CssFontValue::Explicit)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_system_font<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssSystemFont, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    input.expect_exhausted().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "caption" => Ok(CssSystemFont::Caption),
        "icon" => Ok(CssSystemFont::Icon),
        "menu" => Ok(CssSystemFont::Menu),
        "message-box" => Ok(CssSystemFont::MessageBox),
        "small-caption" => Ok(CssSystemFont::SmallCaption),
        "status-bar" => Ok(CssSystemFont::StatusBar),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_font_weight<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontWeight, ParseError<'i, Error>> {
    let start = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "bolder" => return Ok(CssFontWeight::Bolder),
            "lighter" => return Ok(CssFontWeight::Lighter),
            _ => {}
        }
    }
    input.reset(&start);
    parse_absolute_font_weight(input, numeric).map(CssFontWeight::Absolute)
}

pub(super) fn parse_absolute_font_weight<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssAbsoluteFontWeight, ParseError<'i, Error>> {
    /// Fonts 4 `<number [1,1000]>`.
    const FONT_WEIGHT_NUMBER: NumericType<CssFontWeightNumber> = NumericType {
        root: CalculationRoot::Number,
        tokens: LiteralTokens::Numeric,
        literal: |component, _| CssFontWeightNumber::try_from_component(component),
        calculation: |expression| {
            CssFontWeightNumber::try_from_calculation(crate::CssNumberCalculation::from_expression(
                expression,
            ))
        },
    };
    input.skip_whitespace();
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "normal" => Ok(CssAbsoluteFontWeight::Normal),
            "bold" => Ok(CssAbsoluteFontWeight::Bold),
            _ => Err(unexpected_at(location)),
        };
    }
    parse_numeric_type(input, numeric, &FONT_WEIGHT_NUMBER).map(CssAbsoluteFontWeight::Number)
}

pub(super) fn parse_font_style<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontStyle, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("oblique") {
        let angle = input
            .try_parse(|input| parse_font_oblique_angle(input, numeric))
            .ok();
        return Ok(CssFontStyle::Oblique { angle });
    }
    common_font_style_keyword(&ident)
        .map(CssFontStyle::Keyword)
        .ok_or_else(|| unexpected_at(keyword_location))
}

pub(super) fn common_font_style_keyword(name: &str) -> Option<CssFontStyleKeyword> {
    if name.eq_ignore_ascii_case("normal") {
        Some(CssFontStyleKeyword::Normal)
    } else if name.eq_ignore_ascii_case("italic") {
        Some(CssFontStyleKeyword::Italic)
    } else if name.eq_ignore_ascii_case("left") {
        Some(CssFontStyleKeyword::Left)
    } else if name.eq_ignore_ascii_case("right") {
        Some(CssFontStyleKeyword::Right)
    } else {
        None
    }
}

pub(super) fn parse_font_oblique_angle<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontObliqueAngle, ParseError<'i, Error>> {
    /// Fonts 4 `<angle [-90deg,90deg]>`.
    const OBLIQUE_ANGLE: NumericType<CssFontObliqueAngle> = NumericType {
        root: CalculationRoot::Angle,
        tokens: LiteralTokens::Dimension,
        literal: |component, _| CssFontObliqueAngle::try_from_component(component),
        calculation: |expression| {
            CssFontObliqueAngle::try_from_calculation(crate::CssAngleCalculation::from_expression(
                expression,
            ))
        },
    };
    parse_numeric_type(input, numeric, &OBLIQUE_ANGLE)
}

pub(super) fn parse_font_stretch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontWidthKeyword, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssFontWidthKeyword::Normal),
        "ultra-condensed" => Ok(CssFontWidthKeyword::UltraCondensed),
        "extra-condensed" => Ok(CssFontWidthKeyword::ExtraCondensed),
        "condensed" => Ok(CssFontWidthKeyword::Condensed),
        "semi-condensed" => Ok(CssFontWidthKeyword::SemiCondensed),
        "semi-expanded" => Ok(CssFontWidthKeyword::SemiExpanded),
        "expanded" => Ok(CssFontWidthKeyword::Expanded),
        "extra-expanded" => Ok(CssFontWidthKeyword::ExtraExpanded),
        "ultra-expanded" => Ok(CssFontWidthKeyword::UltraExpanded),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontWidth, ParseError<'i, Error>> {
    if let Ok(keyword) = input.try_parse(parse_font_stretch) {
        return Ok(CssFontWidth::Keyword(keyword));
    }
    /// Fonts 4 `<percentage [0,∞]>`; other numeric tokens fail its literal check.
    const FONT_WIDTH_PERCENTAGE: NumericType<crate::CssSpecifiedNonNegativePercentage> =
        NumericType {
            tokens: LiteralTokens::Numeric,
            ..super::values::NONNEGATIVE_PERCENTAGE
        };
    parse_numeric_type(input, numeric, &FONT_WIDTH_PERCENTAGE).map(CssFontWidth::Percentage)
}

fn parse_css2_font_variant<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariant, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssFontVariant::Normal),
        "small-caps" => Ok(CssFontVariant::SmallCaps),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_synthesis<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesis, ParseError<'i, Error>> {
    let state = input.state();
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFontSynthesis::None);
    }
    input.reset(&state);

    // `weight || style || small-caps || position`.
    const KEYWORDS: [&str; 4] = ["weight", "style", "small-caps", "position"];
    let mut slots = [None; 4];
    let [weight, style, small_caps, position] = parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| keyword(&mut slots[operand], input, &[(KEYWORDS[operand], ())]),
    )?;

    CssFontSynthesisValues::try_new(weight, style, small_caps, position)
        .map(CssFontSynthesis::Values)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_font_synthesis_weight<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisWeight, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisWeight::Auto),
        "none" => Ok(CssFontSynthesisWeight::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_synthesis_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisStyle, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisStyle::Auto),
        "none" => Ok(CssFontSynthesisStyle::None),
        "oblique-only" => Ok(CssFontSynthesisStyle::ObliqueOnly),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_synthesis_small_caps<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisSmallCaps, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisSmallCaps::Auto),
        "none" => Ok(CssFontSynthesisSmallCaps::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_synthesis_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisPosition, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisPosition::Auto),
        "none" => Ok(CssFontSynthesisPosition::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_letter_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextSpacingAdjustment, ParseError<'i, Error>> {
    parse_text_spacing_adjustment(input, numeric)
}

pub(super) fn parse_word_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextSpacingAdjustment, ParseError<'i, Error>> {
    parse_text_spacing_adjustment(input, numeric)
}

fn parse_text_spacing_adjustment<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextSpacingAdjustment, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssTextSpacingAdjustment::Normal);
    }
    parse_length_percentage(input, numeric).map(CssTextSpacingAdjustment::LengthPercentage)
}

/// `<'text-decoration-line'> || <'text-decoration-thickness'> ||
/// <'text-decoration-style'> || <'text-decoration-color'>`. The line operand
/// is that longhand's whole grammar, so its keywords stay contiguous.
pub(super) fn parse_text_decoration<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextDecoration, ParseError<'i, Error>> {
    let (mut line, mut style, mut thickness, mut color) = (None, None, None, None);
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut line, input, |input| {
                parse_text_decoration_line_value(input, Tail::Leave)
            }),
            1 => fill(&mut style, input, parse_text_decoration_style),
            2 => fill(&mut thickness, input, |input| {
                parse_text_decoration_thickness(input, numeric)
            }),
            3 => fill(&mut color, input, |input| parse_color(input, numeric)),
            _ => unreachable!("four text-decoration operands"),
        },
    )?;
    Ok(CssTextDecoration::new(line, color, style, thickness))
}

pub(super) fn parse_text_decoration_line<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationLine, ParseError<'i, Error>> {
    parse_text_decoration_line_value(input, Tail::Reject)
}

/// `none | [ underline || overline || line-through || blink ] |
/// spelling-error | grammar-error`, retaining authored keyword order.
fn parse_text_decoration_line_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    tail: Tail,
) -> std::result::Result<CssTextDecorationLine, ParseError<'i, Error>> {
    const KEYWORDS: [(&str, CssTextDecorationLineComponent); 4] = [
        ("underline", CssTextDecorationLineComponent::Underline),
        ("overline", CssTextDecorationLineComponent::Overline),
        ("line-through", CssTextDecorationLineComponent::LineThrough),
        ("blink", CssTextDecorationLineComponent::Blink),
    ];
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTextDecorationLine::none());
    }
    if let Ok(error) = input.try_parse(super::text_decoration::parse_decoration_error) {
        return Ok(CssTextDecorationLine::error(error));
    }
    let mut components = Vec::new();
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        tail,
        |_| false,
        |operand, input| {
            let (keyword, component) = KEYWORDS[operand];
            let matched = input
                .try_parse(|input| input.expect_ident_matching(keyword))
                .is_ok();
            if matched {
                components.push(component);
            }
            Ok(matched)
        },
    )?;
    CssTextDecorationLine::try_new(components)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_text_decoration_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationStyle, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "solid" => Ok(CssTextDecorationStyle::Solid),
        "double" => Ok(CssTextDecorationStyle::Double),
        "dotted" => Ok(CssTextDecorationStyle::Dotted),
        "dashed" => Ok(CssTextDecorationStyle::Dashed),
        "wavy" => Ok(CssTextDecorationStyle::Wavy),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_text_decoration_thickness<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextDecorationThickness, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "auto" => Ok(CssTextDecorationThickness::Auto),
            "from-font" => Ok(CssTextDecorationThickness::FromFont),
            _ => Err(unexpected_at(keyword_location)),
        };
    }

    parse_length_percentage(input, numeric).map(CssTextDecorationThickness::Length)
}

pub(super) fn parse_text_transform<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextTransform, ParseError<'i, Error>> {
    let first = input.state();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        return Ok(CssTextTransform::None);
    }
    if ident.eq_ignore_ascii_case("math-auto") {
        return Ok(CssTextTransform::MathAuto);
    }
    input.reset(&first);
    // `[ capitalize | uppercase | lowercase ] || full-width || full-size-kana`.
    let (mut case, mut full_width, mut full_size_kana) = (None, None, None);
    let [_, full_width, full_size_kana] = parse_group::<3>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => keyword(
                &mut case,
                input,
                &[
                    ("capitalize", CssTextTransformCase::Capitalize),
                    ("uppercase", CssTextTransformCase::Uppercase),
                    ("lowercase", CssTextTransformCase::Lowercase),
                ],
            ),
            1 => keyword(&mut full_width, input, &[("full-width", ())]),
            2 => keyword(&mut full_size_kana, input, &[("full-size-kana", ())]),
            _ => unreachable!("three text-transform operands"),
        },
    )?;
    CssTextTransformSet::try_new(case, full_width, full_size_kana)
        .map(CssTextTransform::Transforms)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_wrap_inside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWrapInside, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssWrapInside::Auto),
        "avoid" => Ok(CssWrapInside::Avoid),
        _ => Err(location.new_unexpected_token_error::<Error>(Token::Ident(ident))),
    }
}
pub(super) fn parse_wrap_boundary<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWrapBoundary, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssWrapBoundary::Auto), "avoid" => Ok(CssWrapBoundary::Avoid),
        "avoid-line" => Ok(CssWrapBoundary::AvoidLine), "avoid-flex" => Ok(CssWrapBoundary::AvoidFlex),
        "line" => Ok(CssWrapBoundary::Line), "flex" => Ok(CssWrapBoundary::Flex),
        _ => Err(location.new_unexpected_token_error::<Error>(Token::Ident(ident))),
    }
}
pub(super) fn parse_line_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLineBreak, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssLineBreak::Auto), "loose" => Ok(CssLineBreak::Loose),
        "normal" => Ok(CssLineBreak::Normal), "strict" => Ok(CssLineBreak::Strict),
        "anywhere" => Ok(CssLineBreak::Anywhere),
        _ => Err(location.new_unexpected_token_error::<Error>(Token::Ident(ident))),
    }
}
pub(super) fn parse_word_space_transform<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWordSpaceTransform, ParseError<'i, Error>> {
    let first = input.state();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        return Ok(CssWordSpaceTransform::None);
    }
    input.reset(&first);
    // `[ space | ideographic-space ] && auto-phrase?`.
    let (mut ideographic, mut auto_phrase) = (None, None);
    let [_, auto_phrase] = parse_group::<2>(
        input,
        Combinator::AllOf {
            optional: [false, true],
        },
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => keyword(
                &mut ideographic,
                input,
                &[("space", false), ("ideographic-space", true)],
            ),
            1 => keyword(&mut auto_phrase, input, &[("auto-phrase", ())]),
            _ => unreachable!("two word-space-transform operands"),
        },
    )?;
    match ideographic {
        Some(false) => Ok(CssWordSpaceTransform::Space { auto_phrase }),
        Some(true) => Ok(CssWordSpaceTransform::IdeographicSpace { auto_phrase }),
        None => Err(unexpected_at(input.current_source_location())),
    }
}
pub(super) fn parse_tab_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTabSize, ParseError<'i, Error>> {
    // Number-first preserves the authored bare-zero choice. Typed math tries
    // pure roots through the shared original numeric context, never a reparse.
    if let Ok(number) =
        input.try_parse(|input| super::values::parse_nonnegative_number(input, numeric))
    {
        return Ok(CssTabSize::Number(number));
    }
    super::values::parse_nonnegative_length(input, numeric).map(CssTabSize::Length)
}

// Text4's remaining authored families share finite role models and the existing
// exact numeric/component owners. Nested reorderable groups stay contiguous.
macro_rules! text_keyword_parser {
    ($parser:ident, $ty:ident, $($text:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $parser<'i, 't>(input: &mut Parser<'i, 't>) -> std::result::Result<$ty, ParseError<'i, Error>> {
            let keyword_location = input.current_source_location();
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident,
                $($text => Ok($ty::$variant),)+
                _ => Err(unexpected_at(keyword_location)),
            }
        }
    };
}
text_keyword_parser!(parse_hyphens, CssHyphens, "none" => None, "manual" => Manual, "auto" => Auto);
text_keyword_parser!(parse_hyphenate_limit_last, CssHyphenateLimitLast,
    "none" => None, "always" => Always, "column" => Column, "page" => Page, "spread" => Spread);
text_keyword_parser!(parse_text_justify_base, CssTextJustifyBase,
    "auto" => Auto, "none" => None, "inter-word" => InterWord, "inter-character" => InterCharacter,
    "ruby" => Ruby, "distribute" => Distribute);
text_keyword_parser!(parse_text_group_align, CssTextGroupAlign,
    "none" => None, "start" => Start, "end" => End, "left" => Left, "right" => Right, "center" => Center);
text_keyword_parser!(parse_autospace_mode, CssAutospaceMode, "insert" => Insert, "replace" => Replace);
text_keyword_parser!(parse_spacing_trim, CssSpacingTrim,
    "space-all" => SpaceAll, "normal" => Normal, "space-first" => SpaceFirst,
    "trim-start" => TrimStart, "trim-both" => TrimBoth, "trim-all" => TrimAll);

pub(super) fn parse_hyphenate_character<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssHyphenateCharacter, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssHyphenateCharacter::Auto);
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|_| unexpected_at(location))?;
    CssHyphenateString::try_from_component(component)
        .map(CssHyphenateCharacter::String)
        .ok_or_else(|| unexpected_at(location))
}
fn parse_hyphenate_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssHyphenateLimitInteger, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let integer = super::values::parse_integer_value(input, numeric)?;
    CssHyphenateLimitInteger::try_new(integer).ok_or_else(|| unexpected_at(location))
}
fn parse_hyphenate_chars_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssHyphenateLimitCharsComponent, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssHyphenateLimitCharsComponent::Auto);
    }
    parse_hyphenate_integer(input, numeric).map(CssHyphenateLimitCharsComponent::Integer)
}
pub(super) fn parse_hyphenate_limit_chars<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssHyphenateLimitChars, ParseError<'i, Error>> {
    let total = parse_hyphenate_chars_component(input, numeric)?;
    let before = if input.is_exhausted() {
        None
    } else {
        Some(parse_hyphenate_chars_component(input, numeric)?)
    };
    let after = if input.is_exhausted() {
        None
    } else {
        Some(parse_hyphenate_chars_component(input, numeric)?)
    };
    Ok(CssHyphenateLimitChars::try_new(total, before, after).expect("ordered authored slots"))
}
pub(super) fn parse_hyphenate_limit_lines<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssHyphenateLimitLines, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("no-limit"))
        .is_ok()
    {
        return Ok(CssHyphenateLimitLines::NoLimit);
    }
    parse_hyphenate_integer(input, numeric).map(CssHyphenateLimitLines::Integer)
}
pub(super) fn parse_hyphenate_limit_zone<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSpecifiedLengthPercentage, ParseError<'i, Error>> {
    parse_length_percentage(input, numeric)
}
pub(super) fn parse_line_padding<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSpecifiedLength, ParseError<'i, Error>> {
    super::values::parse_length(input, numeric)
}
pub(super) fn parse_text_justify<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextJustify, ParseError<'i, Error>> {
    // `[ auto | none | inter-word | inter-character | ruby ] || no-compress`.
    let (mut no_compress, mut base) = (None, None);
    let [no_compress, _] = parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => keyword(&mut no_compress, input, &[("no-compress", ())]),
            1 => fill(&mut base, input, parse_text_justify_base),
            _ => unreachable!("two text-justify operands"),
        },
    )?;
    CssTextJustify::try_new(base, no_compress)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
fn parse_autospace_flags<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<[bool; 3], ParseError<'i, Error>> {
    let mut flags = [false; 3];
    while !input.is_exhausted() {
        let state = input.state();
        let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) else {
            break;
        };
        let index = if ident.eq_ignore_ascii_case("ideograph-alpha") {
            0
        } else if ident.eq_ignore_ascii_case("ideograph-numeric") {
            1
        } else if ident.eq_ignore_ascii_case("punctuation") {
            2
        } else {
            input.reset(&state);
            break;
        };
        if std::mem::replace(&mut flags[index], true) {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    Ok(flags)
}
fn parse_autospace<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAutospace, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("no-autospace"))
        .is_ok()
    {
        return Ok(CssAutospace::NoAutospace);
    }
    // The entire nested flag group appears either before or after the mode.
    let (flags, mode) = if let Ok(mode) = input.try_parse(parse_autospace_mode) {
        (parse_autospace_flags(input)?, Some(mode))
    } else {
        let flags = parse_autospace_flags(input)?;
        (flags, input.try_parse(parse_autospace_mode).ok())
    };
    CssAutospaceValues::try_new(flags[0], flags[1], flags[2], mode)
        .map(CssAutospace::Spacing)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_text_autospace<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextAutospace, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssTextAutospace::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssTextAutospace::Auto);
    }
    parse_autospace(input).map(CssTextAutospace::Autospace)
}
pub(super) fn parse_text_box_trim<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextBoxTrim, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssTextBoxTrim::None),
        "trim-start" => Ok(CssTextBoxTrim::TrimStart),
        "trim-end" => Ok(CssTextBoxTrim::TrimEnd),
        "trim-both" => Ok(CssTextBoxTrim::TrimBoth),
        _ => Err(unexpected_at(keyword_location)),
    }
}
fn parse_text_under_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextUnderEdge, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "text" => Ok(CssTextUnderEdge::Text),
        "ideographic" => Ok(CssTextUnderEdge::Ideographic),
        "ideographic-ink" => Ok(CssTextUnderEdge::IdeographicInk),
        "alphabetic" => Ok(CssTextUnderEdge::Alphabetic),
        _ => Err(unexpected_at(keyword_location)),
    }
}
pub(super) fn parse_text_box_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextBoxEdge, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("auto") {
        return Ok(CssTextBoxEdge::Auto);
    }
    let (over, single) = match_ignore_ascii_case! { &ident,
        "text" => (CssTextOverEdge::Text, Some(CssTextEdgeMetric::Text)),
        "ideographic" => (CssTextOverEdge::Ideographic, Some(CssTextEdgeMetric::Ideographic)),
        "ideographic-ink" => (CssTextOverEdge::IdeographicInk, Some(CssTextEdgeMetric::IdeographicInk)),
        "cap" => (CssTextOverEdge::Cap, None),
        "ex" => (CssTextOverEdge::Ex, None),
        _ => return Err(unexpected_at(keyword_location)),
    };
    if let Ok(under) = input.try_parse(parse_text_under_edge) {
        return Ok(CssTextBoxEdge::Edge(CssTextEdge::Pair { over, under }));
    }
    single
        .map(|metric| CssTextBoxEdge::Edge(CssTextEdge::Single(metric)))
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_text_box<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextBox, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssTextBox::Normal);
    }
    let mut trim = input.try_parse(parse_text_box_trim).ok();
    let edge = input.try_parse(parse_text_box_edge).ok();
    if trim.is_none() {
        trim = input.try_parse(parse_text_box_trim).ok();
    }
    CssTextBoxValues::try_new(trim, edge)
        .map(CssTextBox::Components)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_text_spacing_trim<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextSpacingTrim, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssTextSpacingTrim::Auto);
    }
    parse_spacing_trim(input).map(CssTextSpacingTrim::Trim)
}
pub(super) fn parse_text_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextSpacing, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTextSpacing::None);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssTextSpacing::Auto);
    }
    let mut trim = input.try_parse(parse_spacing_trim).ok();
    let autospace = input.try_parse(parse_autospace).ok();
    if trim.is_none() {
        trim = input.try_parse(parse_spacing_trim).ok();
    }
    CssTextSpacingValues::try_new(trim, autospace)
        .map(CssTextSpacing::Components)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_hanging_punctuation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssHangingPunctuation, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssHangingPunctuation::None);
    }
    // `first || [ force-end | allow-end ] || last`.
    let (mut first, mut end, mut last) = (None, None, None);
    let [first, _, last] = parse_group::<3>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => keyword(&mut first, input, &[("first", ())]),
            1 => keyword(
                &mut end,
                input,
                &[
                    ("force-end", CssHangingPunctuationEnd::ForceEnd),
                    ("allow-end", CssHangingPunctuationEnd::AllowEnd),
                ],
            ),
            2 => keyword(&mut last, input, &[("last", ())]),
            _ => unreachable!("three hanging-punctuation operands"),
        },
    )?;
    CssHangingPunctuationValues::try_new(first, end, last)
        .map(CssHangingPunctuation::Hang)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
