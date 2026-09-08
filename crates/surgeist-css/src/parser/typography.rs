use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, LengthGrammar, next_is_comma, parse_color, parse_integer, parse_length_with,
    parse_typed_calculation,
};
use crate::error::{CssFeatureId, Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) static IMPLEMENTED_PROPERTY_EXTENSIONS: &[CssFeatureId] =
    &[CssFeatureId::new("ext.property.font-weight-range")];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.opentype-tag")];

pub(super) fn parse_caret_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCaretColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssCaretColor::Auto);
    }
    let (color, _) = parse_color(input)?.into_parts();
    Ok(CssCaretColor::Color(Box::new(color)))
}

pub(super) fn parse_font_size<'i, 't>(
    input: &mut Parser<'i, 't>,
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
            "larger" => Ok(CssFontSize::Larger),
            "smaller" => Ok(CssFontSize::Smaller),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("font-size", ident.as_ref()),
            )),
        };
    }

    let value = parse_length_with(input, LengthGrammar::FontSize)?;
    CssFontSizeLengthPercentage::try_new(value)
        .map(CssFontSize::LengthPercentage)
        .ok_or_else(|| unsupported_value(input, None, "font-size must be non-negative"))
}

pub(super) fn parse_line_height<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLineHeight, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssLineHeight::Normal);
    }

    if let Ok(number) = input.try_parse(parse_line_height_number) {
        return Ok(CssLineHeight::Number(number));
    }

    let value = parse_length_with(input, LengthGrammar::LineHeight)?;
    CssLineHeightLengthPercentage::try_new(value)
        .map(CssLineHeight::LengthPercentage)
        .ok_or_else(|| unsupported_value(input, None, "line-height must be non-negative"))
}

fn parse_line_height_number<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssNonNegativeNumberValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => CssNonNegativeNumber::try_new(*value)
            .map(CssNonNegativeNumberValue::Literal)
            .ok_or_else(|| {
                unsupported_value_at(location, None, "line-height must be non-negative")
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
            .map(CssNumberCalculation::from_expression)
            .map(CssNonNegativeNumberValue::Calculation),
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
) -> std::result::Result<CssTextCombineUpright, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssTextCombineUpright::None),
        "all" => Ok(CssTextCombineUpright::All),
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
) -> std::result::Result<CssTextOrientation, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssTextOrientation::Mixed),
        Token::Number {
            int_value: Some(0), ..
        } => Ok(CssTextOrientation::Upright),
        Token::Dimension {
            value: 0.0, unit, ..
        } if unit.eq_ignore_ascii_case("deg") => Ok(CssTextOrientation::Upright),
        Token::Number {
            int_value: Some(90),
            ..
        } => Ok(CssTextOrientation::Sideways),
        Token::Dimension {
            value: 90.0, unit, ..
        } if unit.eq_ignore_ascii_case("deg") => Ok(CssTextOrientation::Sideways),
        _ => Err(unsupported_value_at(
            location,
            None,
            "glyph-orientation-vertical accepts only auto, 0deg, 90deg, 0, or 90",
        )),
    }
}

pub(super) fn parse_text_align<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextAlign, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "start" => Ok(CssTextAlign::Start),
        "end" => Ok(CssTextAlign::End),
        "left" => Ok(CssTextAlign::Left),
        "right" => Ok(CssTextAlign::Right),
        "center" => Ok(CssTextAlign::Center),
        "justify" => Ok(CssTextAlign::Justify),
        "match-parent" => Ok(CssTextAlign::MatchParent),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-align", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_align_last<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextAlignLast, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssTextAlignLast::Auto),
        "start" => Ok(CssTextAlignLast::Start),
        "end" => Ok(CssTextAlignLast::End),
        "left" => Ok(CssTextAlignLast::Left),
        "right" => Ok(CssTextAlignLast::Right),
        "center" => Ok(CssTextAlignLast::Center),
        "justify" => Ok(CssTextAlignLast::Justify),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-align-last", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_text_indent<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextIndent, ParseError<'i, Error>> {
    let length = parse_length_with(input, LengthGrammar::TextIndent)?;
    let mut hanging = false;
    let mut each_line = false;

    while !input.is_exhausted() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        match_ignore_ascii_case! { &ident,
            "hanging" if !hanging => hanging = true,
            "each-line" if !each_line => each_line = true,
            _ => return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("text-indent", ident.as_ref()),
            )),
        }
    }

    Ok(CssTextIndent::new(length, hanging, each_line))
}

pub(super) fn parse_vertical_align<'i, 't>(
    input: &mut Parser<'i, 't>,
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

    parse_length_with(input, LengthGrammar::VerticalAlign)
        .map(CssVerticalAlignLength::new)
        .map(CssVerticalAlign::Length)
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
        if name.is_empty() {
            return Err(unsupported_value(
                input,
                None,
                "font family string is empty",
            ));
        }
        return Ok(CssFontFamilyName::quoted(name.to_string()));
    }

    let mut parts = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) {
        let location = input.current_source_location();
        match input.next().map_err(basic)? {
            Token::Ident(ident) => parts.push(ident.to_string()),
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }

    if parts.is_empty() {
        return Err(unsupported_value(input, None, "font family name is empty"));
    }

    if allow_generic
        && parts.len() == 1
        && let Some(generic) = generic_font_family(&parts[0])
    {
        return Ok(CssFontFamilyName::generic(generic, parts.remove(0)));
    }

    if parts
        .iter()
        .any(|part| generic_font_family(part).is_some() || is_css_wide_keyword(part))
    {
        return Err(unsupported_value(
            input,
            None,
            "font family identifier sequences cannot contain reserved keywords",
        ));
    }

    Ok(CssFontFamilyName::ident_sequence(parts.join(" ")))
}

fn generic_font_family(ident: &str) -> Option<CssGenericFontFamily> {
    if ident.eq_ignore_ascii_case("serif") {
        Some(CssGenericFontFamily::Serif)
    } else if ident.eq_ignore_ascii_case("sans-serif") {
        Some(CssGenericFontFamily::SansSerif)
    } else if ident.eq_ignore_ascii_case("cursive") {
        Some(CssGenericFontFamily::Cursive)
    } else if ident.eq_ignore_ascii_case("fantasy") {
        Some(CssGenericFontFamily::Fantasy)
    } else if ident.eq_ignore_ascii_case("monospace") {
        Some(CssGenericFontFamily::Monospace)
    } else {
        None
    }
}

fn is_css_wide_keyword(ident: &str) -> bool {
    ["initial", "inherit", "unset", "revert", "revert-layer"]
        .iter()
        .any(|keyword| ident.eq_ignore_ascii_case(keyword))
}

pub(super) fn parse_font<'i, 't>(
    input: &mut Parser<'i, 't>,
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

        if let Ok(parsed_size) = input.try_parse(parse_font_size) {
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
            && let Ok(parsed_style) = input.try_parse(parse_font_style)
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
            && let Ok(parsed_weight) = input.try_parse(parse_font_weight)
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
            style = Some(CssFontStyle::Normal);
        } else if variant.is_none() {
            variant = Some(CssFontVariant::Normal);
        } else if weight.is_none() {
            weight = Some(CssFontWeight::Normal);
        } else if stretch.is_none() {
            stretch = Some(CssFontStretch::Normal);
        } else {
            return Err(unsupported_value(
                input,
                None,
                "duplicate font normal component",
            ));
        }
    }

    let line_height = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_line_height(input)?)
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
) -> std::result::Result<CssFontWeight, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "normal" => Ok(CssFontWeight::Normal),
            "bold" => Ok(CssFontWeight::Bold),
            "bolder" => Ok(CssFontWeight::Bolder),
            "lighter" => Ok(CssFontWeight::Lighter),
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("font-weight", ident.as_ref()),
            )),
        },
        Token::Number {
            int_value: Some(value),
            ..
        } if CssFontWeightNumber::try_new(*value).is_some() => {
            Ok(CssFontWeight::Number(CssFontWeightNumber::new(*value)))
        }
        Token::Number {
            int_value: Some(_), ..
        } => Err(unsupported_value_at(
            location,
            None,
            "font-weight must be 1 through 1000",
        )),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            "font-weight number must be an integer",
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_font_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssFontStyle::Normal),
        "italic" => Ok(CssFontStyle::Italic),
        "oblique" => Ok(CssFontStyle::Oblique),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("font-style", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_font_stretch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontStretch, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssFontStretch::Normal),
        "ultra-condensed" => Ok(CssFontStretch::UltraCondensed),
        "extra-condensed" => Ok(CssFontStretch::ExtraCondensed),
        "condensed" => Ok(CssFontStretch::Condensed),
        "semi-condensed" => Ok(CssFontStretch::SemiCondensed),
        "semi-expanded" => Ok(CssFontStretch::SemiExpanded),
        "expanded" => Ok(CssFontStretch::Expanded),
        "extra-expanded" => Ok(CssFontStretch::ExtraExpanded),
        "ultra-expanded" => Ok(CssFontStretch::UltraExpanded),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("font-stretch", ident.as_ref()),
        )),
    }
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

#[derive(Default)]
struct FontVariantComponents {
    common: Option<CssFontVariantLigatureState>,
    discretionary: Option<CssFontVariantLigatureState>,
    historical: Option<CssFontVariantLigatureState>,
    contextual: Option<CssFontVariantLigatureState>,
    position: Option<CssFontVariantPosition>,
    caps: Option<CssFontVariantCaps>,
    figure: Option<CssFontVariantNumericFigure>,
    spacing: Option<CssFontVariantNumericSpacing>,
    fraction: Option<CssFontVariantNumericFraction>,
    ordinal: bool,
    slashed_zero: bool,
    east_asian_variant: Option<CssFontVariantEastAsianVariant>,
    east_asian_width: Option<CssFontVariantEastAsianWidth>,
    ruby: bool,
}

impl FontVariantComponents {
    fn apply_ligature(&mut self, ident: &str) -> Option<bool> {
        if ident.eq_ignore_ascii_case("common-ligatures") {
            Some(set_once(
                &mut self.common,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-common-ligatures") {
            Some(set_once(
                &mut self.common,
                CssFontVariantLigatureState::Disabled,
            ))
        } else if ident.eq_ignore_ascii_case("discretionary-ligatures") {
            Some(set_once(
                &mut self.discretionary,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-discretionary-ligatures") {
            Some(set_once(
                &mut self.discretionary,
                CssFontVariantLigatureState::Disabled,
            ))
        } else if ident.eq_ignore_ascii_case("historical-ligatures") {
            Some(set_once(
                &mut self.historical,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-historical-ligatures") {
            Some(set_once(
                &mut self.historical,
                CssFontVariantLigatureState::Disabled,
            ))
        } else if ident.eq_ignore_ascii_case("contextual") {
            Some(set_once(
                &mut self.contextual,
                CssFontVariantLigatureState::Enabled,
            ))
        } else if ident.eq_ignore_ascii_case("no-contextual") {
            Some(set_once(
                &mut self.contextual,
                CssFontVariantLigatureState::Disabled,
            ))
        } else {
            None
        }
    }

    fn apply_caps(&mut self, ident: &str) -> Option<bool> {
        let value = if ident.eq_ignore_ascii_case("small-caps") {
            CssFontVariantCaps::SmallCaps
        } else if ident.eq_ignore_ascii_case("all-small-caps") {
            CssFontVariantCaps::AllSmallCaps
        } else if ident.eq_ignore_ascii_case("petite-caps") {
            CssFontVariantCaps::PetiteCaps
        } else if ident.eq_ignore_ascii_case("all-petite-caps") {
            CssFontVariantCaps::AllPetiteCaps
        } else if ident.eq_ignore_ascii_case("unicase") {
            CssFontVariantCaps::Unicase
        } else if ident.eq_ignore_ascii_case("titling-caps") {
            CssFontVariantCaps::TitlingCaps
        } else {
            return None;
        };
        Some(set_once(&mut self.caps, value))
    }

    fn apply_position(&mut self, ident: &str) -> Option<bool> {
        let value = if ident.eq_ignore_ascii_case("sub") {
            CssFontVariantPosition::Sub
        } else if ident.eq_ignore_ascii_case("super") {
            CssFontVariantPosition::Super
        } else {
            return None;
        };
        Some(set_once(&mut self.position, value))
    }

    fn apply_numeric(&mut self, ident: &str) -> Option<bool> {
        if ident.eq_ignore_ascii_case("lining-nums") {
            Some(set_once(
                &mut self.figure,
                CssFontVariantNumericFigure::LiningNums,
            ))
        } else if ident.eq_ignore_ascii_case("oldstyle-nums") {
            Some(set_once(
                &mut self.figure,
                CssFontVariantNumericFigure::OldstyleNums,
            ))
        } else if ident.eq_ignore_ascii_case("proportional-nums") {
            Some(set_once(
                &mut self.spacing,
                CssFontVariantNumericSpacing::ProportionalNums,
            ))
        } else if ident.eq_ignore_ascii_case("tabular-nums") {
            Some(set_once(
                &mut self.spacing,
                CssFontVariantNumericSpacing::TabularNums,
            ))
        } else if ident.eq_ignore_ascii_case("diagonal-fractions") {
            Some(set_once(
                &mut self.fraction,
                CssFontVariantNumericFraction::DiagonalFractions,
            ))
        } else if ident.eq_ignore_ascii_case("stacked-fractions") {
            Some(set_once(
                &mut self.fraction,
                CssFontVariantNumericFraction::StackedFractions,
            ))
        } else if ident.eq_ignore_ascii_case("ordinal") {
            Some(set_flag_once(&mut self.ordinal))
        } else if ident.eq_ignore_ascii_case("slashed-zero") {
            Some(set_flag_once(&mut self.slashed_zero))
        } else {
            None
        }
    }

    fn apply_east_asian(&mut self, ident: &str) -> Option<bool> {
        let variant = if ident.eq_ignore_ascii_case("jis78") {
            Some(CssFontVariantEastAsianVariant::Jis78)
        } else if ident.eq_ignore_ascii_case("jis83") {
            Some(CssFontVariantEastAsianVariant::Jis83)
        } else if ident.eq_ignore_ascii_case("jis90") {
            Some(CssFontVariantEastAsianVariant::Jis90)
        } else if ident.eq_ignore_ascii_case("jis04") {
            Some(CssFontVariantEastAsianVariant::Jis04)
        } else if ident.eq_ignore_ascii_case("simplified") {
            Some(CssFontVariantEastAsianVariant::Simplified)
        } else if ident.eq_ignore_ascii_case("traditional") {
            Some(CssFontVariantEastAsianVariant::Traditional)
        } else {
            None
        };
        if let Some(variant) = variant {
            return Some(set_once(&mut self.east_asian_variant, variant));
        }

        let width = if ident.eq_ignore_ascii_case("full-width") {
            Some(CssFontVariantEastAsianWidth::FullWidth)
        } else if ident.eq_ignore_ascii_case("proportional-width") {
            Some(CssFontVariantEastAsianWidth::ProportionalWidth)
        } else {
            None
        };
        if let Some(width) = width {
            return Some(set_once(&mut self.east_asian_width, width));
        }

        ident
            .eq_ignore_ascii_case("ruby")
            .then(|| set_flag_once(&mut self.ruby))
    }

    fn ligatures(&self) -> Option<CssFontVariantLigatureValues> {
        CssFontVariantLigatureValues::try_new(
            self.common,
            self.discretionary,
            self.historical,
            self.contextual,
        )
    }

    fn numeric(&self) -> Option<CssFontVariantNumericValues> {
        CssFontVariantNumericValues::try_new(
            self.figure,
            self.spacing,
            self.fraction,
            self.ordinal,
            self.slashed_zero,
        )
    }

    fn east_asian(&self) -> Option<CssFontVariantEastAsianValues> {
        CssFontVariantEastAsianValues::try_new(
            self.east_asian_variant,
            self.east_asian_width,
            self.ruby,
        )
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> bool {
    if slot.is_some() {
        false
    } else {
        *slot = Some(value);
        true
    }
}

fn set_flag_once(slot: &mut bool) -> bool {
    if *slot {
        false
    } else {
        *slot = true;
        true
    }
}

fn next_font_variant_ident<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(cssparser::SourceLocation, String), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    Ok((location, ident.as_ref().to_owned()))
}

fn invalid_font_variant_keyword<'i>(
    location: cssparser::SourceLocation,
    ident: String,
) -> ParseError<'i, Error> {
    location.new_unexpected_token_error::<Error>(Token::Ident(ident.into()))
}

pub(super) fn parse_font_variant_caps<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantCaps, ParseError<'i, Error>> {
    let (location, ident) = next_font_variant_ident(input)?;
    let value = if ident.eq_ignore_ascii_case("normal") {
        CssFontVariantCaps::Normal
    } else if ident.eq_ignore_ascii_case("small-caps") {
        CssFontVariantCaps::SmallCaps
    } else if ident.eq_ignore_ascii_case("all-small-caps") {
        CssFontVariantCaps::AllSmallCaps
    } else if ident.eq_ignore_ascii_case("petite-caps") {
        CssFontVariantCaps::PetiteCaps
    } else if ident.eq_ignore_ascii_case("all-petite-caps") {
        CssFontVariantCaps::AllPetiteCaps
    } else if ident.eq_ignore_ascii_case("unicase") {
        CssFontVariantCaps::Unicase
    } else if ident.eq_ignore_ascii_case("titling-caps") {
        CssFontVariantCaps::TitlingCaps
    } else {
        return Err(invalid_font_variant_keyword(location, ident));
    };
    Ok(value)
}

pub(super) fn parse_font_variant_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantPosition, ParseError<'i, Error>> {
    let (location, ident) = next_font_variant_ident(input)?;
    if ident.eq_ignore_ascii_case("normal") {
        Ok(CssFontVariantPosition::Normal)
    } else if ident.eq_ignore_ascii_case("sub") {
        Ok(CssFontVariantPosition::Sub)
    } else if ident.eq_ignore_ascii_case("super") {
        Ok(CssFontVariantPosition::Super)
    } else {
        Err(invalid_font_variant_keyword(location, ident))
    }
}

pub(super) fn parse_font_variant_ligatures<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantLigatures, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantLigatures::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFontVariantLigatures::None);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        if components.apply_ligature(&ident) != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }
    components
        .ligatures()
        .map(CssFontVariantLigatures::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-ligatures requires a value"))
}

pub(super) fn parse_font_variant_numeric<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantNumeric, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantNumeric::Normal);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        if components.apply_numeric(&ident) != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }
    components
        .numeric()
        .map(CssFontVariantNumeric::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-numeric requires a value"))
}

pub(super) fn parse_font_variant_east_asian<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantEastAsian, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantEastAsian::Normal);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        if components.apply_east_asian(&ident) != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }
    components
        .east_asian()
        .map(CssFontVariantEastAsian::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-variant-east-asian requires a value"))
}

pub(super) fn parse_font_variant<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontVariantValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontVariantValue::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFontVariantValue::None);
    }

    let mut components = FontVariantComponents::default();
    while !input.is_exhausted() {
        let (location, ident) = next_font_variant_ident(input)?;
        let applied = components
            .apply_ligature(&ident)
            .or_else(|| components.apply_position(&ident))
            .or_else(|| components.apply_caps(&ident))
            .or_else(|| components.apply_numeric(&ident))
            .or_else(|| components.apply_east_asian(&ident));
        if applied != Some(true) {
            return Err(invalid_font_variant_keyword(location, ident));
        }
    }

    CssFontVariantValues::try_new(
        components.ligatures(),
        components.position,
        components.caps,
        components.numeric(),
        components.east_asian(),
    )
    .map(CssFontVariantValue::Values)
    .ok_or_else(|| unsupported_value(input, None, "font-variant requires a value"))
}

pub(super) fn parse_font_kerning<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontKerning, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "auto" => Ok(CssFontKerning::Auto),
            "normal" => Ok(CssFontKerning::Normal),
            "none" => Ok(CssFontKerning::None),
            _ => Err(location.new_unexpected_token_error::<Error>(Token::Ident(ident.clone()))),
        },
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_font_size_adjust<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFontSizeAdjust, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("none") => Ok(CssFontSizeAdjust::None),
        token @ Token::Number { value, .. } => CssNonNegativeNumber::try_new(*value)
            .map(CssFontSizeAdjust::Number)
            .ok_or_else(|| location.new_unexpected_token_error::<Error>(token.clone())),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
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
            token => {
                return Err(location.new_unexpected_token_error::<Error>(token.clone()));
            }
        }
    }

    CssFontSynthesisValues::try_new(weight, style)
        .map(CssFontSynthesis::Values)
        .ok_or_else(|| unsupported_value(input, None, "font-synthesis requires a value"))
}

pub(super) fn parse_font_feature_settings<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredFontFeatureSettings, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if ident.eq_ignore_ascii_case("normal") && input.is_exhausted() {
            return Ok(CssAuthoredFontFeatureSettings::Normal);
        }
        input.reset(&state);
    }

    let mut features = Vec::new();
    loop {
        features.push(parse_font_feature(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font-feature-settings list has an empty item",
            ));
        }
    }

    CssAuthoredFontFeatureList::try_new(features)
        .map(CssAuthoredFontFeatureSettings::Features)
        .ok_or_else(|| unsupported_value(input, None, "font-feature-settings list is empty"))
}

pub(super) fn parse_font_feature<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredFontFeature, ParseError<'i, Error>> {
    let tag_location = input.current_source_location();
    let tag = input.expect_string_cloned().map_err(basic)?.to_string();
    if tag.is_empty() {
        return Err(unsupported_value(input, None, "font feature tag is empty"));
    }
    if tag.chars().count() == 4 && !tag.is_ascii() {
        return Err(unsupported_value_at(
            tag_location,
            None,
            "font feature tag must be four ASCII characters",
        ));
    }

    let value = if input.is_exhausted() || next_is_comma(input) {
        CssAuthoredFontFeatureValue::Omitted
    } else if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "on" => CssAuthoredFontFeatureValue::On,
            "off" => CssAuthoredFontFeatureValue::Off,
            _ => return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("font feature value", ident.as_ref()),
            )),
        }
    } else {
        let index_location = input.current_source_location();
        let value = parse_integer(input, "font feature value")?;
        let value = CssFontFeatureIndex::try_new(value).ok_or_else(|| {
            unsupported_value_at(
                index_location,
                None,
                "font feature index must be non-negative",
            )
        })?;
        CssAuthoredFontFeatureValue::Index(value)
    };

    let tag = CssOpenTypeTag::try_new(tag).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "font feature tag must be four ASCII characters",
        )
    })?;
    Ok(CssAuthoredFontFeature::new(tag, value))
}

pub(super) fn parse_letter_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLetterSpacing, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        Ok(CssLetterSpacing::Normal)
    } else {
        parse_length_with(input, LengthGrammar::LetterSpacing)
            .map(CssLetterSpacingLength::new)
            .map(CssLetterSpacing::Length)
    }
}

pub(super) fn parse_word_spacing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWordSpacing, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        Ok(CssWordSpacing::Normal)
    } else {
        let location = input.current_source_location();
        let value = parse_length_with(input, LengthGrammar::WordSpacing)?;
        CssWordSpacingLength::try_new(value)
            .map(CssWordSpacing::Length)
            .ok_or_else(|| unsupported_value_at(location, None, "word-spacing requires a length"))
    }
}

pub(super) fn parse_text_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrap, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "wrap" => Ok(CssTextWrap::Wrap),
        "nowrap" => Ok(CssTextWrap::NoWrap),
        "balance" => Ok(CssTextWrap::Balance),
        "pretty" => Ok(CssTextWrap::Pretty),
        "stable" => Ok(CssTextWrap::Stable),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-wrap", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_white_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpace, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssWhiteSpace::Normal),
        "nowrap" => Ok(CssWhiteSpace::NoWrap),
        "pre" => Ok(CssWhiteSpace::Pre),
        "pre-wrap" => Ok(CssWhiteSpace::PreWrap),
        "pre-line" => Ok(CssWhiteSpace::PreLine),
        "break-spaces" => Ok(CssWhiteSpace::BreakSpaces),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("white-space", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_word_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWordBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssWordBreak::Normal),
        "break-all" => Ok(CssWordBreak::BreakAll),
        "keep-all" => Ok(CssWordBreak::KeepAll),
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
) -> std::result::Result<CssTextDecoration, ParseError<'i, Error>> {
    let mut line_components = Vec::new();
    let mut line_none = false;
    let mut color = None;
    let mut style = None;
    let mut thickness = None;

    while !input.is_exhausted() {
        if let Ok(component) = input.try_parse(parse_text_decoration_line_component) {
            if line_none {
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
            if line_none || !line_components.is_empty() {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate text-decoration-line none",
                ));
            }
            line_none = true;
            continue;
        }
        if style.is_none()
            && let Ok(parsed_style) = input.try_parse(parse_text_decoration_style)
        {
            style = Some(parsed_style);
            continue;
        }
        if thickness.is_none()
            && let Ok(parsed_thickness) = input.try_parse(parse_text_decoration_thickness)
        {
            thickness = Some(parsed_thickness);
            continue;
        }
        if color.is_none()
            && let Ok(parsed_color) = input.try_parse(parse_color)
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

    let line = if line_none {
        Some(CssTextDecorationLine::none())
    } else if line_components.is_empty() {
        None
    } else {
        Some(CssTextDecorationLine::new(line_components))
    };

    if line.is_none() && color.is_none() && style.is_none() && thickness.is_none() {
        None
    } else {
        Some(CssTextDecoration::new_current(
            line, color, style, thickness,
        ))
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

    parse_length_with(input, LengthGrammar::TextDecorationThickness)
        .map(CssTextDecorationThicknessLength::new)
        .map(CssTextDecorationThickness::Length)
}

pub(super) fn parse_text_transform<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextTransform, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssTextTransform::None),
        "capitalize" => Ok(CssTextTransform::Capitalize),
        "uppercase" => Ok(CssTextTransform::Uppercase),
        "lowercase" => Ok(CssTextTransform::Lowercase),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("text-transform", ident.as_ref()),
        )),
    }
}
