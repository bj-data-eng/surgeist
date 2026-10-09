use super::color::parse_color;
use super::values::{parse_hinted_number_calculation, parse_length_percentage};
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{CalculationRoot, next_is_comma, parse_numeric_function};
use crate::error::{CssFeatureId, Error, basic, unsupported_value, unsupported_value_at};
use crate::font_variant::CssFontVariant;
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;
use crate::{
    CssAbsoluteFontWeight, CssFontObliqueAngle, CssFontSize, CssFontStyle, CssFontStyleKeyword,
    CssFontSynthesis, CssFontSynthesisPosition, CssFontSynthesisSmallCaps, CssFontSynthesisStyle,
    CssFontSynthesisValues, CssFontSynthesisWeight, CssFontWeight, CssFontWeightNumber,
    CssFontWidth, CssFontWidthKeyword, CssLengthPercentageCalculation, CssLineHeight,
    CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLengthPercentage,
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
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("font-size", ident.as_ref()),
            )),
        };
    }

    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let checked = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid font-size component"))?;
            crate::CssSpecifiedNonNegativeLengthPercentage::from_property_component(
                component, numeric,
            )
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            crate::CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    checked.map(CssFontSize::LengthPercentage).map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "font-size requires a nonnegative length-percentage or size keyword",
        )
    })
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

    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let checked = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid line-height component")
            })?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    checked
        .map(CssLineHeight::LengthPercentage)
        .map_err(|error| {
            unsupported_value_at(
                numeric.error_location(&error, location, root_offset),
                None,
                "line-height requires a nonnegative number or length-percentage",
            )
        })
}

fn parse_line_height_number<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSpecifiedNonNegativeNumber, ParseError<'i, Error>> {
    input.skip_whitespace();
    let numeric_start = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&numeric_start);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid line-height number"))?;
            CssSpecifiedNonNegativeNumber::try_from_component(component).map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "line-height must be non-negative",
                )
            })
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Number)
                .map(CssNumberCalculation::from_expression)
                .and_then(|calculation| {
                    CssSpecifiedNonNegativeNumber::try_from_calculation(calculation).map_err(
                        |error| {
                            unsupported_value_at(
                                numeric.error_location(&error, location, root_offset),
                                None,
                                "line-height must be non-negative",
                            )
                        },
                    )
                })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_writing_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWritingMode, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "horizontal-tb" => Ok(CssWritingMode::HorizontalTb),
        "vertical-rl" => Ok(CssWritingMode::VerticalRl),
        "vertical-lr" => Ok(CssWritingMode::VerticalLr),
        "sideways-rl" => Ok(CssWritingMode::SidewaysRl),
        "sideways-lr" => Ok(CssWritingMode::SidewaysLr),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("writing-mode", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_combine_upright<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextCombineUpright, ParseError<'i, Error>> {
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
                    count.ok_or_else(|| unsupported_value_at(location, None,
                        "text-combine-upright literal count must be between two and four"))?
                }
                Token::Function(name) if crate::numeric::is_math_function(name) => {
                    let expression = parse_numeric_function(input, &start, numeric, CalculationRoot::Integer)?;
                    CssTextCombineDigitCount::from_calculation(CssIntegerCalculation::from_expression(expression))
                }
                token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
            };
            Ok(CssTextCombineUpright::Digits(Some(count)))
        },
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-combine-upright", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_orientation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextOrientation, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "mixed" => Ok(CssTextOrientation::Mixed),
        "upright" => Ok(CssTextOrientation::Upright),
        "sideways" => Ok(CssTextOrientation::Sideways),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-orientation", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_unicode_bidi<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUnicodeBidi, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssUnicodeBidi::Normal),
        "embed" => Ok(CssUnicodeBidi::Embed),
        "isolate" => Ok(CssUnicodeBidi::Isolate),
        "bidi-override" => Ok(CssUnicodeBidi::BidiOverride),
        "isolate-override" => Ok(CssUnicodeBidi::IsolateOverride),
        "plaintext" => Ok(CssUnicodeBidi::Plaintext),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("unicode-bidi", ident.as_ref()),
        )),
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
            let offset = input.position().byte_index();
            let component = numeric.collect(input).map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, offset),
                    None,
                    "invalid glyph orientation dimension",
                )
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
    value.ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            "glyph-orientation-vertical accepts only auto, 0deg, 90deg, 0, or 90",
        )
    })
}

pub(super) fn parse_text_indent<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextIndent, ParseError<'i, Error>> {
    let mut length = None;
    let mut hanging = false;
    let mut each_line = false;

    while !input.is_exhausted() {
        if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
            match_ignore_ascii_case! { &ident,
                "hanging" if !hanging => hanging = true,
                "each-line" if !each_line => each_line = true,
                _ => return Err(unsupported_value(
                    input,
                    None,
                    unsupported_keyword_reason("text-indent", ident.as_ref()),
                )),
            }
            continue;
        }
        if length.is_some() {
            return Err(unsupported_value(
                input,
                None,
                "text-indent has more than one length-percentage",
            ));
        }
        length = Some(parse_length_percentage(input, numeric, "text-indent")?);
    }

    let length = length.ok_or_else(|| {
        unsupported_value(input, None, "text-indent requires one length-percentage")
    })?;
    Ok(CssTextIndent::new(length, hanging, each_line))
}

pub(super) fn parse_vertical_align<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssVerticalAlign, ParseError<'i, Error>> {
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
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("vertical-align", ident.as_ref()),
            )),
        };
    }

    parse_length_percentage(input, numeric, "vertical-align").map(CssVerticalAlign::Length)
}

pub(super) fn parse_font_family_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontFamilyList, ParseError<'i, Error>> {
    let mut families = Vec::new();
    loop {
        families.push(parse_font_family_name(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font-family list has an empty item",
            ));
        }
    }

    CssFontFamilyList::try_new(families)
        .ok_or_else(|| unsupported_value(input, None, "font-family list is empty"))
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
            .ok_or_else(|| unsupported_value(input, None, "invalid decoded font family string"));
    }

    if allow_generic
        && input
            .try_parse(|input| input.expect_function_matching("generic"))
            .is_ok()
    {
        return input.parse_nested_block(|input| {
            let keyword = input.expect_ident_cloned().map_err(basic)?;
            let generic = CssGenericFontFamily::from_script_keyword(&keyword)
                .ok_or_else(|| unsupported_value(input, None, "unknown generic font family"))?;
            input.expect_exhausted().map_err(basic)?;
            Ok(CssFontFamilyName::generic(generic))
        });
    }

    let mut parts = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) {
        let location = input.current_source_location();
        match input.next().map_err(basic)? {
            Token::Ident(ident) => parts.push(ident.to_string()),
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }

    if allow_generic
        && let [keyword] = parts.as_slice()
        && let Some(generic) = CssGenericFontFamily::from_keyword(keyword)
    {
        return Ok(CssFontFamilyName::generic(generic));
    }

    CssFontFamilyName::try_ident_sequence(parts).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "font family names require nonempty identifier tokens without reserved keywords",
        )
    })
}

pub(super) fn parse_font<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontValue, ParseError<'i, Error>> {
    if let Ok(system) = input.try_parse(parse_system_font) {
        return Ok(CssFontValue::System(system));
    }

    let mut style = None;
    let mut variant = None;
    let mut weight = None;
    let mut stretch = None;
    let mut normal_count = 0;
    let size;

    loop {
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font shorthand is missing a size",
            ));
        }

        if let Ok(parsed_size) = input.try_parse(|input| parse_font_size(input, numeric)) {
            size = parsed_size;
            break;
        }

        if let Ok(()) = input.try_parse(|input| {
            input.expect_ident_matching("normal").map_err(basic)?;
            if normal_count == 4 {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate font normal component",
                ));
            }
            normal_count += 1;
            Ok(())
        }) {
            continue;
        }

        if style.is_none()
            && let Ok(parsed_style) = input.try_parse(|input| parse_font_style(input, numeric))
        {
            style = Some(parsed_style);
            continue;
        }
        if variant.is_none()
            && let Ok(parsed_variant) = input.try_parse(parse_css2_font_variant)
        {
            variant = Some(parsed_variant);
            continue;
        }
        if weight.is_none()
            && let Ok(parsed_weight) = input.try_parse(|input| parse_font_weight(input, numeric))
        {
            weight = Some(parsed_weight);
            continue;
        }
        if stretch.is_none()
            && let Ok(parsed_stretch) = input.try_parse(parse_font_stretch)
        {
            stretch = Some(parsed_stretch);
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported font shorthand component before size",
        ));
    }

    for _ in 0..normal_count {
        if style.is_none() {
            style = Some(CssFontStyle::Keyword(CssFontStyleKeyword::Normal));
        } else if variant.is_none() {
            variant = Some(CssFontVariant::Normal);
        } else if weight.is_none() {
            weight = Some(CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal));
        } else if stretch.is_none() {
            stretch = Some(CssFontWidthKeyword::Normal);
        } else {
            return Err(unsupported_value(
                input,
                None,
                "duplicate font normal component",
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
        .ok_or_else(|| unsupported_value(input, None, "invalid font shorthand"))
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("font", ident.as_ref()),
        )),
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
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "normal" => Ok(CssAbsoluteFontWeight::Normal),
            "bold" => Ok(CssAbsoluteFontWeight::Bold),
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("absolute font-weight", ident.as_ref()),
            )),
        },
        Token::Number { .. } | Token::Percentage { .. } | Token::Dimension { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid font-weight number"))?;
            CssFontWeightNumber::try_from_component(component)
                .map(CssAbsoluteFontWeight::Number)
                .map_err(|error| {
                    unsupported_value_at(
                        numeric.error_location(&error, location, root_offset),
                        None,
                        "font-weight requires a number between 1 and 1000",
                    )
                })
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Number)?;
            CssFontWeightNumber::try_from_calculation(crate::CssNumberCalculation::from_expression(
                expression,
            ))
            .map(CssAbsoluteFontWeight::Number)
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "font-weight requires number math",
                )
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_font_style<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("oblique") {
        let angle = input
            .try_parse(|input| parse_font_oblique_angle(input, numeric))
            .ok();
        return Ok(CssFontStyle::Oblique { angle });
    }
    common_font_style_keyword(&ident)
        .map(CssFontStyle::Keyword)
        .ok_or_else(|| {
            unsupported_value(
                input,
                None,
                unsupported_keyword_reason("font-style", ident.as_ref()),
            )
        })
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
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let result = match input.next().map_err(basic)? {
        Token::Dimension { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid font-style angle component")
            })?;
            CssFontObliqueAngle::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Angle)?;
            CssFontObliqueAngle::try_from_calculation(crate::CssAngleCalculation::from_expression(
                expression,
            ))
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    result.map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "font-style requires an oblique angle between -90deg and 90deg",
        )
    })
}

pub(super) fn parse_font_stretch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontWidthKeyword, ParseError<'i, Error>> {
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("font-stretch", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_font_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFontWidth, ParseError<'i, Error>> {
    if let Ok(keyword) = input.try_parse(parse_font_stretch) {
        return Ok(CssFontWidth::Keyword(keyword));
    }
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let percentage = match input.next().map_err(basic)? {
        Token::Percentage { .. } | Token::Number { .. } | Token::Dimension { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid font-width percentage")
            })?;
            crate::CssSpecifiedNonNegativePercentage::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Percentage)?;
            crate::CssSpecifiedNonNegativePercentage::try_from_calculation(
                CssPercentageCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
    .map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "font-width requires a nonnegative percentage or width keyword",
        )
    })?;
    Ok(CssFontWidth::Percentage(percentage))
}

fn parse_css2_font_variant<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariant, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssFontVariant::Normal),
        "small-caps" => Ok(CssFontVariant::SmallCaps),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("font-variant", ident.as_ref()),
        )),
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

    let mut weight = false;
    let mut style = false;
    let mut small_caps = false;
    let mut position = false;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let token = input.next().map_err(basic)?;
        match token {
            Token::Ident(ident) if ident.eq_ignore_ascii_case("weight") && !weight => {
                weight = true;
            }
            Token::Ident(ident) if ident.eq_ignore_ascii_case("style") && !style => {
                style = true;
            }
            Token::Ident(ident) if ident.eq_ignore_ascii_case("small-caps") && !small_caps => {
                small_caps = true;
            }
            Token::Ident(ident) if ident.eq_ignore_ascii_case("position") && !position => {
                position = true;
            }
            token => {
                return Err(location.new_unexpected_token_error::<Error>(token.clone()));
            }
        }
    }

    CssFontSynthesisValues::try_new(weight, style, small_caps, position)
        .map(CssFontSynthesis::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-synthesis requires a value"))
}

pub(super) fn parse_font_synthesis_weight<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisWeight, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisWeight::Auto),
        "none" => Ok(CssFontSynthesisWeight::None),
        _ => Err(unsupported_value(input, None, unsupported_keyword_reason("font-synthesis-weight", ident.as_ref()))),
    }
}

pub(super) fn parse_font_synthesis_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisStyle::Auto),
        "none" => Ok(CssFontSynthesisStyle::None),
        "oblique-only" => Ok(CssFontSynthesisStyle::ObliqueOnly),
        _ => Err(unsupported_value(input, None, unsupported_keyword_reason("font-synthesis-style", ident.as_ref()))),
    }
}

pub(super) fn parse_font_synthesis_small_caps<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisSmallCaps, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisSmallCaps::Auto),
        "none" => Ok(CssFontSynthesisSmallCaps::None),
        _ => Err(unsupported_value(input, None, unsupported_keyword_reason("font-synthesis-small-caps", ident.as_ref()))),
    }
}

pub(super) fn parse_font_synthesis_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSynthesisPosition, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontSynthesisPosition::Auto),
        "none" => Ok(CssFontSynthesisPosition::None),
        _ => Err(unsupported_value(input, None, unsupported_keyword_reason("font-synthesis-position", ident.as_ref()))),
    }
}

pub(super) fn parse_letter_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextSpacingAdjustment, ParseError<'i, Error>> {
    parse_text_spacing_adjustment(input, numeric, "letter-spacing")
}

pub(super) fn parse_word_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextSpacingAdjustment, ParseError<'i, Error>> {
    parse_text_spacing_adjustment(input, numeric, "word-spacing")
}

fn parse_text_spacing_adjustment<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    property: &str,
) -> std::result::Result<CssTextSpacingAdjustment, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssTextSpacingAdjustment::Normal);
    }
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let value = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, format!("invalid {property} component"))
            })?;
            CssSpecifiedLengthPercentage::from_property_component(component, numeric)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
    .map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            format!("{property} requires normal or a length-percentage"),
        )
    })?;
    Ok(CssTextSpacingAdjustment::LengthPercentage(value))
}

fn text_wrap_mode_keyword(ident: &str) -> Option<CssTextWrapMode> {
    match_ignore_ascii_case! { ident,
        "wrap" => Some(CssTextWrapMode::Wrap), "nowrap" => Some(CssTextWrapMode::NoWrap),
        _ => None,
    }
}
fn text_wrap_style_keyword(ident: &str) -> Option<CssTextWrapStyle> {
    match_ignore_ascii_case! { ident,
        "auto" => Some(CssTextWrapStyle::Auto), "balance" => Some(CssTextWrapStyle::Balance),
        "stable" => Some(CssTextWrapStyle::Stable), "pretty" => Some(CssTextWrapStyle::Pretty),
        "avoid-short-last-line" => Some(CssTextWrapStyle::AvoidShortLastLine), _ => None,
    }
}
fn white_space_collapse_keyword(ident: &str) -> Option<CssWhiteSpaceCollapse> {
    match_ignore_ascii_case! { ident,
        "collapse" => Some(CssWhiteSpaceCollapse::Collapse), "discard" => Some(CssWhiteSpaceCollapse::Discard),
        "preserve" => Some(CssWhiteSpaceCollapse::Preserve), "preserve-breaks" => Some(CssWhiteSpaceCollapse::PreserveBreaks),
        "preserve-spaces" => Some(CssWhiteSpaceCollapse::PreserveSpaces), "break-spaces" => Some(CssWhiteSpaceCollapse::BreakSpaces),
        _ => None,
    }
}

pub(super) fn parse_text_wrap_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrapMode, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    text_wrap_mode_keyword(&ident).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-wrap-mode", &ident),
        )
    })
}
pub(super) fn parse_text_wrap_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrapStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    text_wrap_style_keyword(&ident).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-wrap-style", &ident),
        )
    })
}
pub(super) fn parse_white_space_collapse<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpaceCollapse, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    white_space_collapse_keyword(&ident).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            unsupported_keyword_reason("white-space-collapse", &ident),
        )
    })
}

pub(super) fn parse_white_space_trim<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpaceTrim, ParseError<'i, Error>> {
    let mut ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        return Ok(CssWhiteSpaceTrim::none());
    }
    let (mut before, mut after, mut inner) = (false, false, false);
    loop {
        let flag = match_ignore_ascii_case! { &ident,
            "discard-before" => &mut before, "discard-after" => &mut after, "discard-inner" => &mut inner,
            _ => return Err(unsupported_value(input, None, unsupported_keyword_reason("white-space-trim", &ident))),
        };
        if *flag {
            return Err(unsupported_value(
                input,
                None,
                "white-space-trim flags cannot repeat",
            ));
        }
        *flag = true;
        let state = input.state();
        let Ok(next) = input.try_parse(|input| input.expect_ident_cloned()) else {
            break;
        };
        if next.eq_ignore_ascii_case("discard-before")
            || next.eq_ignore_ascii_case("discard-after")
            || next.eq_ignore_ascii_case("discard-inner")
        {
            ident = next;
        } else {
            input.reset(&state);
            break;
        }
    }
    Ok(CssWhiteSpaceTrim::new(before, after, inner))
}

pub(super) fn parse_text_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrap, ParseError<'i, Error>> {
    let (mut mode, mut style) = (None, None);
    while !input.is_exhausted() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if let Some(value) = text_wrap_mode_keyword(&ident) {
            if mode.replace(value).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "text-wrap mode cannot repeat",
                ));
            }
        } else if let Some(value) = text_wrap_style_keyword(&ident) {
            if style.replace(value).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "text-wrap style cannot repeat",
                ));
            }
        } else {
            return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("text-wrap", &ident),
            ));
        }
    }
    CssTextWrap::try_new(mode, style)
        .ok_or_else(|| unsupported_value(input, None, "text-wrap requires a mode or style"))
}

pub(super) fn parse_white_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpace, ParseError<'i, Error>> {
    let start = input.state();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let keyword = match_ignore_ascii_case! { &ident,
        "normal" => Some(CssWhiteSpaceKeyword::Normal), "pre" => Some(CssWhiteSpaceKeyword::Pre),
        "pre-wrap" => Some(CssWhiteSpaceKeyword::PreWrap), "pre-line" => Some(CssWhiteSpaceKeyword::PreLine), _ => None,
    };
    if let Some(keyword) = keyword {
        return Ok(CssWhiteSpace::from_keyword(keyword));
    }
    input.reset(&start);
    let (mut collapse, mut mode, mut trim) = (None, None, None);
    while !input.is_exhausted() {
        let state = input.state();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if let Some(value) = white_space_collapse_keyword(&ident) {
            if collapse.replace(value).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "white-space collapse cannot repeat",
                ));
            }
        } else if let Some(value) = text_wrap_mode_keyword(&ident) {
            if mode.replace(value).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "white-space mode cannot repeat",
                ));
            }
        } else if ident.eq_ignore_ascii_case("none")
            || ident.eq_ignore_ascii_case("discard-before")
            || ident.eq_ignore_ascii_case("discard-after")
            || ident.eq_ignore_ascii_case("discard-inner")
        {
            // Quoted property ranges preserve their grouping: the entire trim
            // constituent is contiguous and can occur only once (Values4 §2.2).
            if trim.is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "white-space trim must form one nonrepeated group",
                ));
            }
            input.reset(&state);
            trim = Some(parse_white_space_trim(input)?);
        } else {
            return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("white-space", &ident),
            ));
        }
    }
    CssWhiteSpace::try_new(collapse, mode, trim)
        .ok_or_else(|| unsupported_value(input, None, "white-space requires a constituent"))
}

pub(super) fn parse_word_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWordBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssWordBreak::Normal),
        "break-all" => Ok(CssWordBreak::BreakAll),
        "keep-all" => Ok(CssWordBreak::KeepAll),
        "manual" => Ok(CssWordBreak::Manual),
        "auto-phrase" => Ok(CssWordBreak::AutoPhrase),
        "break-word" => Ok(CssWordBreak::BreakWord),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("word-break", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_overflow_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOverflowWrap, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssOverflowWrap::Normal),
        "break-word" => Ok(CssOverflowWrap::BreakWord),
        "anywhere" => Ok(CssOverflowWrap::Anywhere),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("overflow-wrap", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_overflow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextOverflow, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "clip" => Ok(CssTextOverflow::Clip),
        "ellipsis" => Ok(CssTextOverflow::Ellipsis),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-overflow", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_decoration<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextDecoration, ParseError<'i, Error>> {
    let mut line_components = Vec::new();
    let mut line_none = false;
    let mut line_error = None;
    let mut color = None;
    let mut style = None;
    let mut thickness = None;

    while !input.is_exhausted() {
        if let Ok(component) = input.try_parse(parse_text_decoration_line_component) {
            if line_none || line_error.is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "text-decoration line mixes none with line components",
                ));
            }
            if line_components.contains(&component) {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate text-decoration-line component",
                ));
            }
            line_components.push(component);
            continue;
        }
        if input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            if line_none || line_error.is_some() || !line_components.is_empty() {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate text-decoration-line none",
                ));
            }
            line_none = true;
            continue;
        }
        if let Ok(error) = input.try_parse(super::text_decoration::parse_decoration_error) {
            if line_none || line_error.is_some() || !line_components.is_empty() {
                return Err(unsupported_value(input, None, "exclusive error line"));
            }
            line_error = Some(error);
            continue;
        }
        if style.is_none()
            && let Ok(parsed_style) = input.try_parse(parse_text_decoration_style)
        {
            style = Some(parsed_style);
            continue;
        }
        if thickness.is_none()
            && let Ok(parsed_thickness) =
                input.try_parse(|input| parse_text_decoration_thickness(input, numeric))
        {
            thickness = Some(parsed_thickness);
            continue;
        }
        if color.is_none()
            && let Ok(parsed_color) = input.try_parse(|input| parse_color(input, numeric))
        {
            color = Some(parsed_color);
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported text-decoration component",
        ));
    }

    let line = if let Some(error) = line_error {
        Some(CssTextDecorationLine::error(error))
    } else if line_none {
        Some(CssTextDecorationLine::none())
    } else if line_components.is_empty() {
        None
    } else {
        Some(CssTextDecorationLine::new(line_components))
    };

    if line.is_none() && color.is_none() && style.is_none() && thickness.is_none() {
        None
    } else {
        Some(CssTextDecoration::new(line, color, style, thickness))
    }
    .ok_or_else(|| unsupported_value(input, None, "text-decoration shorthand is empty"))
}

pub(super) fn parse_text_decoration_line<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationLine, ParseError<'i, Error>> {
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
    while !input.is_exhausted() {
        let component = parse_text_decoration_line_component(input)?;
        if components.contains(&component) {
            return Err(unsupported_value(
                input,
                None,
                "duplicate text-decoration-line component",
            ));
        }
        components.push(component);
    }

    CssTextDecorationLine::try_new(components)
        .ok_or_else(|| unsupported_value(input, None, "text-decoration-line is empty"))
}

pub(super) fn parse_text_decoration_line_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationLineComponent, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "underline" => Ok(CssTextDecorationLineComponent::Underline),
        "overline" => Ok(CssTextDecorationLineComponent::Overline),
        "line-through" => Ok(CssTextDecorationLineComponent::LineThrough),
        "blink" => Ok(CssTextDecorationLineComponent::Blink),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-decoration-line", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_decoration_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextDecorationStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "solid" => Ok(CssTextDecorationStyle::Solid),
        "double" => Ok(CssTextDecorationStyle::Double),
        "dotted" => Ok(CssTextDecorationStyle::Dotted),
        "dashed" => Ok(CssTextDecorationStyle::Dashed),
        "wavy" => Ok(CssTextDecorationStyle::Wavy),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-decoration-style", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_decoration_thickness<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTextDecorationThickness, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "auto" => Ok(CssTextDecorationThickness::Auto),
            "from-font" => Ok(CssTextDecorationThickness::FromFont),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("text-decoration-thickness", ident.as_ref()),
            )),
        };
    }

    parse_length_percentage(input, numeric, "text-decoration-thickness")
        .map(CssTextDecorationThickness::Length)
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
    let mut case = None;
    let mut full_width = false;
    let mut full_size_kana = false;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        match_ignore_ascii_case! { &ident,
            "capitalize" if case.is_none() => case = Some(CssTextTransformCase::Capitalize),
            "uppercase" if case.is_none() => case = Some(CssTextTransformCase::Uppercase),
            "lowercase" if case.is_none() => case = Some(CssTextTransformCase::Lowercase),
            "full-width" if !full_width => full_width = true,
            "full-size-kana" if !full_size_kana => full_size_kana = true,
            _ => return Err(location.new_unexpected_token_error::<Error>(Token::Ident(ident))),
        }
    }
    CssTextTransformSet::try_new(case, full_width, full_size_kana)
        .map(CssTextTransform::Transforms)
        .ok_or_else(|| {
            unsupported_value(
                input,
                None,
                "text-transform requires a nonempty transform set",
            )
        })
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
    let mut ideographic = None;
    let mut auto_phrase = false;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        match_ignore_ascii_case! { &ident,
            "space" if ideographic.is_none() => ideographic = Some(false),
            "ideographic-space" if ideographic.is_none() => ideographic = Some(true),
            "auto-phrase" if !auto_phrase => auto_phrase = true,
            _ => return Err(location.new_unexpected_token_error::<Error>(Token::Ident(ident))),
        }
    }
    match ideographic {
        Some(false) => Ok(CssWordSpaceTransform::Space { auto_phrase }),
        Some(true) => Ok(CssWordSpaceTransform::IdeographicSpace { auto_phrase }),
        None => Err(unsupported_value(
            input,
            None,
            "word-space-transform requires a separator base",
        )),
    }
}
pub(super) fn parse_tab_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTabSize, ParseError<'i, Error>> {
    // Number-first preserves the authored bare-zero choice. Typed math tries
    // pure roots through the shared original numeric context, never a reparse.
    if let Ok(number) =
        input.try_parse(|input| super::values::parse_nonnegative_number(input, numeric, "tab-size"))
    {
        return Ok(CssTabSize::Number(number));
    }
    super::values::parse_nonnegative_length(input, numeric, "tab-size").map(CssTabSize::Length)
}

// Text4's remaining authored families share finite role models and the existing
// exact numeric/component owners. Nested reorderable groups stay contiguous.
macro_rules! text_keyword_parser {
    ($parser:ident, $ty:ident, $($text:literal => $variant:ident),+ $(,)?) => {
        pub(super) fn $parser<'i, 't>(input: &mut Parser<'i, 't>) -> std::result::Result<$ty, ParseError<'i, Error>> {
            let ident = input.expect_ident_cloned().map_err(basic)?;
            match_ignore_ascii_case! { &ident,
                $($text => Ok($ty::$variant),)+
                _ => Err(unsupported_value(input, None, "invalid Text keyword")),
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
    let component = numeric
        .collect(input)
        .map_err(|_| unsupported_value_at(location, None, "expected hyphenation string"))?;
    CssHyphenateString::try_from_component(component)
        .map(CssHyphenateCharacter::String)
        .ok_or_else(|| unsupported_value_at(location, None, "expected hyphenation string or auto"))
}
fn parse_hyphenate_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssHyphenateLimitInteger, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let integer = super::values::parse_integer_value(input, numeric)?;
    CssHyphenateLimitInteger::try_new(integer).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            "expected nonnegative integer or Integer-root function",
        )
    })
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
    parse_length_percentage(input, numeric, "hyphenate-limit-zone")
}
pub(super) fn parse_line_padding<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSpecifiedLength, ParseError<'i, Error>> {
    super::values::parse_length(input, numeric, "line-padding")
}
pub(super) fn parse_text_justify<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextJustify, ParseError<'i, Error>> {
    let mut base = None;
    let mut no_compress = false;
    while !input.is_exhausted() {
        if input
            .try_parse(|input| input.expect_ident_matching("no-compress"))
            .is_ok()
        {
            if no_compress {
                return Err(unsupported_value(input, None, "duplicate no-compress"));
            }
            no_compress = true;
        } else {
            let value = parse_text_justify_base(input)?;
            if base.replace(value).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate justification base",
                ));
            }
        }
    }
    CssTextJustify::try_new(base, no_compress)
        .ok_or_else(|| unsupported_value(input, None, "empty justification value"))
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
            return Err(unsupported_value(input, None, "duplicate autospace flag"));
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
        .ok_or_else(|| unsupported_value(input, None, "empty autospace constituent"))
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
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssTextBoxTrim::None),
        "trim-start" => Ok(CssTextBoxTrim::TrimStart),
        "trim-end" => Ok(CssTextBoxTrim::TrimEnd),
        "trim-both" => Ok(CssTextBoxTrim::TrimBoth),
        _ => Err(unsupported_value(input, None, "a text-box-trim keyword")),
    }
}
fn parse_text_under_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextUnderEdge, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "text" => Ok(CssTextUnderEdge::Text),
        "ideographic" => Ok(CssTextUnderEdge::Ideographic),
        "ideographic-ink" => Ok(CssTextUnderEdge::IdeographicInk),
        "alphabetic" => Ok(CssTextUnderEdge::Alphabetic),
        _ => Err(unsupported_value(input, None, "an under text edge")),
    }
}
pub(super) fn parse_text_box_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextBoxEdge, ParseError<'i, Error>> {
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
        _ => return Err(unsupported_value(input, None, "an over text edge or auto")),
    };
    if let Ok(under) = input.try_parse(parse_text_under_edge) {
        return Ok(CssTextBoxEdge::Edge(CssTextEdge::Pair { over, under }));
    }
    single
        .map(|metric| CssTextBoxEdge::Edge(CssTextEdge::Single(metric)))
        .ok_or_else(|| {
            unsupported_value(input, None, "an explicit under text edge after cap or ex")
        })
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
        .ok_or_else(|| unsupported_value(input, None, "a nonempty text-box value"))
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
        .ok_or_else(|| unsupported_value(input, None, "empty text-spacing value"))
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
    let mut first = false;
    let mut end = None;
    let mut last = false;
    while !input.is_exhausted() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        match_ignore_ascii_case! { &ident,
            "first" if !first => first = true,
            "last" if !last => last = true,
            "force-end" if end.is_none() => end = Some(CssHangingPunctuationEnd::ForceEnd),
            "allow-end" if end.is_none() => end = Some(CssHangingPunctuationEnd::AllowEnd),
            _ => return Err(unsupported_value(input, None, "duplicate, conflicting or unknown hanging role")),
        };
    }
    CssHangingPunctuationValues::try_new(first, end, last)
        .map(CssHangingPunctuation::Hang)
        .ok_or_else(|| unsupported_value(input, None, "empty hanging-punctuation value"))
}
