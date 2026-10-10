use super::color::parse_color;
use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use super::values::{parse_shadow_length, parse_shadow_nonnegative_length};
use crate::error::{CssFeatureId, Error, basic, unexpected_at};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("official.value.box-edge-keywords")];

pub(super) fn parse_box_decoration_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBoxDecorationBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "slice" => Ok(CssBoxDecorationBreak::Slice),
        "clone" => Ok(CssBoxDecorationBreak::Clone),
        _ => Err(unexpected_at(input.current_source_location())),
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
            return Err(unexpected_at(input.current_source_location()));
        }
    }

    Ok(CssBoxShadow::Shadows(
        CssBoxShadowList::try_new(shadows).expect("box-shadow parser records at least one shadow"),
    ))
}

pub(super) fn parse_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssShadow, ParseError<'i, Error>> {
    let mut inset = false;
    let mut color = None;
    let mut lengths = None;
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
                return Err(unexpected_at(input.current_source_location()));
            }
            inset = true;
            continue;
        }
        if let Ok(parsed) = input.try_parse(|input| parse_color(input, numeric)) {
            if color.replace(parsed).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            continue;
        }
        if lengths.is_some() {
            return Err(unexpected_at(input.current_source_location()));
        }
        // The && grammar reorders whole groups, never the lengths within one.
        let x = parse_shadow_length(input, numeric)?;
        let y = parse_shadow_length(input, numeric)?;
        let blur = input
            .try_parse(|input| parse_shadow_nonnegative_length(input, numeric))
            .ok();
        let spread = if blur.is_some() {
            input
                .try_parse(|input| parse_shadow_length(input, numeric))
                .ok()
        } else {
            None
        };
        lengths = Some((x, y, blur, spread));
    }
    let Some((x, y, blur, spread)) = lengths else {
        return Err(unexpected_at(input.current_source_location()));
    };
    Ok(CssShadow::try_new(inset, x, y, blur, spread, color)
        .expect("parser requires blur before spread"))
}

pub(super) fn parse_drop_shadow<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssDropShadow, ParseError<'i, Error>> {
    let mut color = None;
    let mut lengths = None;
    while !input.is_exhausted() {
        if let Ok(parsed) = input.try_parse(|input| parse_color(input, numeric)) {
            if color.replace(parsed).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            continue;
        }
        if lengths.is_some() {
            return Err(unexpected_at(input.current_source_location()));
        }
        let x = parse_shadow_length(input, numeric)?;
        let y = parse_shadow_length(input, numeric)?;
        let standard_deviation = input
            .try_parse(|input| parse_shadow_nonnegative_length(input, numeric))
            .ok();
        lengths = Some((x, y, standard_deviation));
    }
    let Some((x, y, standard_deviation)) = lengths else {
        return Err(unexpected_at(input.current_source_location()));
    };
    Ok(CssDropShadow::new(x, y, standard_deviation, color))
}
