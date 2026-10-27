//! Checked authored grammars from the selected Scroll Snap 1 publication.

use super::property_input::Parser;
use super::side_shapes::{parse_auto_or, parse_four_sides, parse_side_pair};
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::{parse_length, parse_nonnegative_length_percentage};
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssLength, CssNumeric};
use crate::numeric::{CssLengthPercentage, CssNonNegative};
use crate::values::common::CssAutoOr;
use crate::values::common::{CssFourSides, CssSidePair};
use crate::values::layout::scroll_snap::{ScrollMarginSide, ScrollPaddingSide};
use crate::values::layout::{
    CssScrollSnapAlignment, CssScrollSnapAxis, CssScrollSnapStop, CssScrollSnapStrictness,
    CssScrollSnapType,
};

pub(super) fn parse_scroll_snap_type<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapType, ParseError<'i, Error>> {
    // Scroll Snap 1 §5.1: the unadmitted keyword itself is responsible.
    let axis_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let axis = match_ignore_ascii_case! { &ident,
        "none" => return Ok(CssScrollSnapType::None),
        "x" => CssScrollSnapAxis::X,
        "y" => CssScrollSnapAxis::Y,
        "block" => CssScrollSnapAxis::Block,
        "inline" => CssScrollSnapAxis::Inline,
        "both" => CssScrollSnapAxis::Both,
        _ => return Err(unexpected_at(axis_location)),
    };
    let strictness = if input.is_exhausted() {
        None
    } else {
        let strictness_location = input.current_source_location();
        let strictness = input.expect_ident_cloned().map_err(basic)?;
        Some(match_ignore_ascii_case! { &strictness,
            "mandatory" => CssScrollSnapStrictness::Mandatory,
            "proximity" => CssScrollSnapStrictness::Proximity,
            _ => return Err(unexpected_at(strictness_location)),
        })
    };
    Ok(CssScrollSnapType::axis(axis, strictness))
}

fn parse_alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapAlignment, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssScrollSnapAlignment::None),
        "start" => Ok(CssScrollSnapAlignment::Start),
        "end" => Ok(CssScrollSnapAlignment::End),
        "center" => Ok(CssScrollSnapAlignment::Center),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_scroll_snap_align<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssSidePair<CssScrollSnapAlignment>, ParseError<'i, Error>> {
    parse_side_pair(input, parse_alignment)
}

pub(super) fn parse_scroll_snap_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapStop, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssScrollSnapStop::Normal),
        "always" => Ok(CssScrollSnapStop::Always),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_scroll_margin_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNumeric<CssLength>, ParseError<'i, Error>> {
    parse_length(input, numeric)
}

pub(super) fn parse_scroll_padding_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>, ParseError<'i, Error>> {
    parse_auto_or(input, |input| {
        parse_nonnegative_length_percentage(input, numeric)
    })
}

pub(super) fn parse_scroll_margin_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<ScrollMarginSide>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_scroll_margin_length(input, numeric))
}

pub(super) fn parse_scroll_padding_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<ScrollPaddingSide>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_scroll_padding_value(input, numeric))
}

pub(super) fn parse_scroll_margin_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<ScrollMarginSide>, ParseError<'i, Error>> {
    parse_four_sides(input, |input| parse_scroll_margin_length(input, numeric))
}

pub(super) fn parse_scroll_padding_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<ScrollPaddingSide>, ParseError<'i, Error>> {
    parse_four_sides(input, |input| parse_scroll_padding_value(input, numeric))
}
