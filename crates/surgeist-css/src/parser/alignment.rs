//! Box Alignment 3 grammar for the selected six longhands and three shorthands.

use cssparser::{ParseError, Parser};

use crate::alignment::*;
use crate::error::{Error, basic, unexpected_at};

fn keyword<'i, 't>(input: &mut Parser<'i, 't>) -> Result<String, ParseError<'i, Error>> {
    Ok(input
        .expect_ident_cloned()
        .map_err(basic)?
        .to_ascii_lowercase())
}

fn maybe<'i, 't>(input: &mut Parser<'i, 't>, expected: &str) -> bool {
    input
        .try_parse(|p| p.expect_ident_matching(expected))
        .is_ok()
}

fn position(name: &str) -> Option<CssAlignmentPosition> {
    Some(match name {
        "center" => CssAlignmentPosition::Center,
        "start" => CssAlignmentPosition::Start,
        "end" => CssAlignmentPosition::End,
        "self-start" => CssAlignmentPosition::SelfStart,
        "self-end" => CssAlignmentPosition::SelfEnd,
        "flex-start" => CssAlignmentPosition::FlexStart,
        "flex-end" => CssAlignmentPosition::FlexEnd,
        "left" => CssAlignmentPosition::Left,
        "right" => CssAlignmentPosition::Right,
        _ => return None,
    })
}

fn legacy(name: &str) -> Option<CssLegacyAlignment> {
    Some(match name {
        "left" => CssLegacyAlignment::Left,
        "right" => CssLegacyAlignment::Right,
        "center" => CssLegacyAlignment::Center,
        _ => return None,
    })
}

fn parse_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    justify_items: bool,
) -> Result<CssAlignmentValue, ParseError<'i, Error>> {
    let first = keyword(input)?;
    let overflow = match first.as_str() {
        "safe" => Some(CssOverflowPosition::Safe),
        "unsafe" => Some(CssOverflowPosition::Unsafe),
        _ => None,
    };
    let word = if overflow.is_some() {
        keyword(input)?
    } else {
        first
    };

    let value = match word.as_str() {
        "normal" => CssAlignmentValue::Normal { overflow },
        "auto" if overflow.is_none() => CssAlignmentValue::Auto,
        "stretch" if overflow.is_none() => CssAlignmentValue::Stretch,
        "space-between" if overflow.is_none() => CssAlignmentValue::SpaceBetween,
        "space-around" if overflow.is_none() => CssAlignmentValue::SpaceAround,
        "space-evenly" if overflow.is_none() => CssAlignmentValue::SpaceEvenly,
        "baseline" if overflow.is_none() => {
            let which = if maybe(input, "first") {
                CssBaselinePosition::First
            } else if maybe(input, "last") {
                CssBaselinePosition::Last
            } else {
                CssBaselinePosition::Baseline
            };
            CssAlignmentValue::Baseline(which)
        }
        "first" if overflow.is_none() && maybe(input, "baseline") => {
            CssAlignmentValue::Baseline(CssBaselinePosition::First)
        }
        "last" if overflow.is_none() && maybe(input, "baseline") => {
            CssAlignmentValue::Baseline(CssBaselinePosition::Last)
        }
        "legacy" if justify_items && overflow.is_none() => {
            let state = input.state();
            let second = input.try_parse(keyword).ok().and_then(|word| legacy(&word));
            if second.is_none() {
                input.reset(&state);
            }
            CssAlignmentValue::Legacy(second)
        }
        word => {
            let Some(position) = position(word) else {
                return Err(unexpected_at(input.current_source_location()));
            };
            if justify_items
                && overflow.is_none()
                && legacy(word).is_some()
                && maybe(input, "legacy")
            {
                CssAlignmentValue::Legacy(legacy(word))
            } else {
                CssAlignmentValue::Position { overflow, position }
            }
        }
    };
    Ok(value)
}

macro_rules! longhand {
    ($name:ident, $type:ident, $legacy:expr) => {
        pub(super) fn $name<'i, 't>(
            input: &mut Parser<'i, 't>,
        ) -> Result<$type, ParseError<'i, Error>> {
            let location = input.current_source_location();
            let value = parse_value(input, $legacy)?;
            $type::try_new(value).ok_or_else(|| unexpected_at(location))
        }
    };
}

longhand!(parse_align_content_value, CssAlignContentValue, false);
longhand!(parse_justify_content_value, CssJustifyContentValue, false);
longhand!(parse_align_items_value, CssAlignItemsValue, false);
longhand!(parse_justify_items_value, CssJustifyItemsValue, true);
longhand!(parse_align_self_value, CssAlignSelfValue, false);
longhand!(parse_justify_self_value, CssJustifySelfValue, false);

pub(super) fn parse_place_content_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssPlaceContentValue, ParseError<'i, Error>> {
    let align = parse_align_content_value(input)?;
    if input.is_exhausted() {
        Ok(CssPlaceContentValue::from_align(align))
    } else {
        Ok(CssPlaceContentValue::new(
            align,
            parse_justify_content_value(input)?,
        ))
    }
}

pub(super) fn parse_place_items_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssPlaceItemsValue, ParseError<'i, Error>> {
    if let Ok(pair) = input.try_parse(|input| {
        input.expect_ident_matching("baseline").map_err(basic)?;
        let align =
            CssAlignItemsValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Baseline))
                .expect("bare baseline is valid on align-items");
        let justify = parse_justify_items_value(input)?;
        input.expect_exhausted().map_err(basic)?;
        Ok::<_, ParseError<'i, Error>>(CssPlaceItemsValue::new(align, justify))
    }) {
        return Ok(pair);
    }
    let align = parse_align_items_value(input)?;
    if input.is_exhausted() {
        Ok(CssPlaceItemsValue::from_align(align))
    } else {
        Ok(CssPlaceItemsValue::new(
            align,
            parse_justify_items_value(input)?,
        ))
    }
}

pub(super) fn parse_place_self_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssPlaceSelfValue, ParseError<'i, Error>> {
    if let Ok(pair) = input.try_parse(|input| {
        input.expect_ident_matching("baseline").map_err(basic)?;
        let align =
            CssAlignSelfValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Baseline))
                .expect("bare baseline is valid on align-self");
        let justify = parse_justify_self_value(input)?;
        input.expect_exhausted().map_err(basic)?;
        Ok::<_, ParseError<'i, Error>>(CssPlaceSelfValue::new(align, justify))
    }) {
        return Ok(pair);
    }
    let align = parse_align_self_value(input)?;
    if input.is_exhausted() {
        Ok(CssPlaceSelfValue::from_align(align))
    } else {
        Ok(CssPlaceSelfValue::new(
            align,
            parse_justify_self_value(input)?,
        ))
    }
}
