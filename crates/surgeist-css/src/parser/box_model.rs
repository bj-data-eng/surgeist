use super::color::parse_color;
use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use super::values::{parse_shadow_blur_length, parse_shadow_length};
use crate::box_values::CssParsedBorderColors;
use crate::error::{CssFeatureId, Error, basic, unsupported_value};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.box-edge-keywords")];

pub(super) fn parse_box_decoration_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBoxDecorationBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "slice" => Ok(CssBoxDecorationBreak::Slice),
        "clone" => Ok(CssBoxDecorationBreak::Clone),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("box-decoration-break", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_border_colors<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssParsedBorderColors, ParseError<'i, Error>> {
    let mut colors = Vec::new();
    while !input.is_exhausted() {
        colors.push(parse_color(input, numeric)?);
        if colors.len() == 4 {
            input.expect_exhausted().map_err(basic)?;
            break;
        }
    }
    CssParsedBorderColors::try_new(colors).ok_or_else(|| {
        unsupported_value(input, None, "border-color requires one through four colors")
    })
}

pub(super) fn expand_radius_components(
    values: Vec<CssLength>,
) -> (CssLength, CssLength, CssLength, CssLength) {
    match values.as_slice() {
        [all] => (all.clone(), all.clone(), all.clone(), all.clone()),
        [vertical, horizontal] => (
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ),
        [top_left, top_right_bottom_left, bottom_right] => (
            top_left.clone(),
            top_right_bottom_left.clone(),
            bottom_right.clone(),
            top_right_bottom_left.clone(),
        ),
        [top_left, top_right, bottom_right, bottom_left] => (
            top_left.clone(),
            top_right.clone(),
            bottom_right.clone(),
            bottom_left.clone(),
        ),
        [] => unreachable!("caller validates non-empty border-radius components"),
        _ => unreachable!("border-radius component parser caps values at four"),
    }
}

pub(super) fn parse_box_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBoxShadow, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned)
        && ident.eq_ignore_ascii_case("none")
        && input.is_exhausted()
    {
        return Ok(CssBoxShadow::None);
    }
    input.reset(&state);

    let mut shadows = Vec::new();
    loop {
        shadows.push(parse_shadow(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "box-shadow list has an empty item",
            ));
        }
    }

    Ok(CssBoxShadow::Shadows(
        CssBoxShadowList::new(shadows).expect("box-shadow parser records at least one shadow"),
    ))
}

pub(super) fn parse_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssShadow, ParseError<'i, Error>> {
    let mut inset = false;
    let mut color = None;
    let mut lengths = Vec::new();

    while !input.is_exhausted() {
        let state = input.state();
        if input.try_parse(Parser::expect_comma).is_ok() {
            input.reset(&state);
            break;
        }

        if input
            .try_parse(|input| input.expect_ident_matching("inset"))
            .is_ok()
        {
            if inset {
                return Err(unsupported_value(input, None, "duplicate box-shadow inset"));
            }
            inset = true;
            continue;
        }

        if let Ok(parsed_color) = input.try_parse(|input| parse_color(input, numeric)) {
            if color.replace(parsed_color).is_some() {
                return Err(unsupported_value(input, None, "duplicate box-shadow color"));
            }
            continue;
        }

        let parsed_length = match lengths.len() {
            0 | 1 | 3 => input
                .try_parse(|input| parse_shadow_length(input, numeric))
                .ok(),
            2 => input
                .try_parse(|input| parse_shadow_blur_length(input, numeric))
                .ok(),
            _ => None,
        };
        if let Some(parsed_length) = parsed_length {
            lengths.push(parsed_length);
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported box-shadow component",
        ));
    }

    match lengths.as_slice() {
        [offset_x, offset_y] => Ok(CssShadow::new_current(
            inset,
            offset_x.clone(),
            offset_y.clone(),
            None,
            None,
            color,
        )),
        [offset_x, offset_y, blur] => Ok(CssShadow::new_current(
            inset,
            offset_x.clone(),
            offset_y.clone(),
            Some(blur.clone()),
            None,
            color,
        )),
        [offset_x, offset_y, blur, spread] => Ok(CssShadow::new_current(
            inset,
            offset_x.clone(),
            offset_y.clone(),
            Some(blur.clone()),
            Some(spread.clone()),
            color,
        )),
        _ => Err(unsupported_value(
            input,
            None,
            "box-shadow requires at least two offsets",
        )),
    }
}

pub(super) fn parse_drop_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDropShadow, ParseError<'i, Error>> {
    let mut color = None;
    let mut lengths = Vec::new();

    while !input.is_exhausted() {
        if let Ok(parsed_color) = input.try_parse(|input| parse_color(input, numeric)) {
            if color.replace(parsed_color).is_some() {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate drop-shadow color",
                ));
            }
            continue;
        }

        let parsed_length = match lengths.len() {
            0 | 1 => input
                .try_parse(|input| parse_shadow_length(input, numeric))
                .ok(),
            2 => input
                .try_parse(|input| parse_shadow_blur_length(input, numeric))
                .ok(),
            _ => None,
        };
        if let Some(parsed_length) = parsed_length {
            lengths.push(parsed_length);
            continue;
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported drop-shadow component",
        ));
    }

    match lengths.as_slice() {
        [offset_x, offset_y] => {
            CssDropShadow::try_new_current(offset_x.clone(), offset_y.clone(), None, color)
        }
        [offset_x, offset_y, blur] => CssDropShadow::try_new_current(
            offset_x.clone(),
            offset_y.clone(),
            Some(blur.clone()),
            color,
        ),
        _ => None,
    }
    .ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "drop-shadow requires two offsets and an optional non-negative blur",
        )
    })
}
