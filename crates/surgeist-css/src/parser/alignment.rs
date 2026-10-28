//! Box Alignment 3 grammar for the selected six longhands and three shorthands.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;

use crate::values::layout::alignment::*;

fn keyword<'i>(input: &mut Stream<'i>) -> Result<String, SyntaxError<'i, Error>> {
    Ok(input.consume_ident().map_err(basic)?.to_ascii_lowercase())
}

fn maybe<'i>(input: &mut Stream<'i>, expected: &str) -> bool {
    input.attempt(|p| p.consume_keyword(expected)).is_ok()
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

/// One alignment value and the location of its keyword after any
/// `<overflow-position>`, which a property's own grammar may reject.
fn parse_value<'i>(
    input: &mut Stream<'i>,
    justify_items: bool,
) -> Result<(CssAlignmentValue, super::syntax_types::Site<'i>), SyntaxError<'i, Error>> {
    let mut word_location = input.site();
    let first = keyword(input)?;
    let overflow = match first.as_str() {
        "safe" => Some(CssOverflowPosition::Safe),
        "unsafe" => Some(CssOverflowPosition::Unsafe),
        _ => None,
    };
    let word = if overflow.is_some() {
        input.discard_whitespace();
        word_location = input.site();
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
            let state = input.mark();
            let second = input.attempt(keyword).ok().and_then(|word| legacy(&word));
            if second.is_none() {
                input.restore(state);
            }
            CssAlignmentValue::Legacy(second)
        }
        word => {
            let Some(position) = position(word) else {
                return Err(unexpected_at(word_location));
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
    Ok((value, word_location))
}

macro_rules! longhand {
    ($name:ident, $type:ident, $legacy:expr) => {
        pub(super) fn $name<'i>(input: &mut Stream<'i>) -> Result<$type, SyntaxError<'i, Error>> {
            // Align 3: the keyword this property's grammar excludes is
            // responsible; an admitted `<overflow-position>` is a valid prefix.
            let (value, location) = parse_value(input, $legacy)?;
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

pub(crate) fn parse_place_content_value<'i>(
    input: &mut Stream<'i>,
) -> Result<CssPlaceContentValue, SyntaxError<'i, Error>> {
    let align = parse_align_content_value(input)?;
    if input.only_whitespace_remains() {
        Ok(CssPlaceContentValue::from_align(align))
    } else {
        Ok(CssPlaceContentValue::new(
            align,
            parse_justify_content_value(input)?,
        ))
    }
}

pub(crate) fn parse_place_items_value<'i>(
    input: &mut Stream<'i>,
) -> Result<CssPlaceItemsValue, SyntaxError<'i, Error>> {
    if let Ok(pair) = input.attempt(|input| {
        input.consume_keyword("baseline").map_err(basic)?;
        let align =
            CssAlignItemsValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Baseline))
                .expect("bare baseline is valid on align-items");
        let justify = parse_justify_items_value(input)?;
        input.require_empty().map_err(basic)?;
        Ok::<_, SyntaxError<'i, Error>>(CssPlaceItemsValue::new(align, justify))
    }) {
        return Ok(pair);
    }
    let align = parse_align_items_value(input)?;
    if input.only_whitespace_remains() {
        Ok(CssPlaceItemsValue::from_align(align))
    } else {
        Ok(CssPlaceItemsValue::new(
            align,
            parse_justify_items_value(input)?,
        ))
    }
}

pub(crate) fn parse_place_self_value<'i>(
    input: &mut Stream<'i>,
) -> Result<CssPlaceSelfValue, SyntaxError<'i, Error>> {
    if let Ok(pair) = input.attempt(|input| {
        input.consume_keyword("baseline").map_err(basic)?;
        let align =
            CssAlignSelfValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Baseline))
                .expect("bare baseline is valid on align-self");
        let justify = parse_justify_self_value(input)?;
        input.require_empty().map_err(basic)?;
        Ok::<_, SyntaxError<'i, Error>>(CssPlaceSelfValue::new(align, justify))
    }) {
        return Ok(pair);
    }
    let align = parse_align_self_value(input)?;
    if input.only_whitespace_remains() {
        Ok(CssPlaceSelfValue::from_align(align))
    } else {
        Ok(CssPlaceSelfValue::new(
            align,
            parse_justify_self_value(input)?,
        ))
    }
}
