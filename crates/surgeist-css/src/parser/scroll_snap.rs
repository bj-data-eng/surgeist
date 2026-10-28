//! Checked authored grammars from the selected Scroll Snap 1 publication.

use super::side_shapes::{parse_auto_or, parse_four_sides, parse_side_pair};
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
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

pub(super) fn parse_scroll_snap_type<'i>(
    input: &mut Stream<'i>,
) -> Result<CssScrollSnapType, SyntaxError<'i, Error>> {
    // Scroll Snap 1 §5.1: the unadmitted keyword itself is responsible.
    let axis_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    let axis = match_ignore_ascii_case! { &ident,
        "none" => return Ok(CssScrollSnapType::None),
        "x" => CssScrollSnapAxis::X,
        "y" => CssScrollSnapAxis::Y,
        "block" => CssScrollSnapAxis::Block,
        "inline" => CssScrollSnapAxis::Inline,
        "both" => CssScrollSnapAxis::Both,
        _ => return Err(unexpected_at(axis_location)),
    };
    let strictness = if input.only_whitespace_remains() {
        None
    } else {
        let strictness_location = input.site();
        let strictness = input.consume_ident().map_err(basic)?;
        Some(match_ignore_ascii_case! { &strictness,
            "mandatory" => CssScrollSnapStrictness::Mandatory,
            "proximity" => CssScrollSnapStrictness::Proximity,
            _ => return Err(unexpected_at(strictness_location)),
        })
    };
    Ok(CssScrollSnapType::axis(axis, strictness))
}

fn parse_alignment<'i>(
    input: &mut Stream<'i>,
) -> Result<CssScrollSnapAlignment, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssScrollSnapAlignment::None),
        "start" => Ok(CssScrollSnapAlignment::Start),
        "end" => Ok(CssScrollSnapAlignment::End),
        "center" => Ok(CssScrollSnapAlignment::Center),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_scroll_snap_align<'i>(
    input: &mut Stream<'i>,
) -> Result<CssSidePair<CssScrollSnapAlignment>, SyntaxError<'i, Error>> {
    parse_side_pair(input, parse_alignment)
}

pub(super) fn parse_scroll_snap_stop<'i>(
    input: &mut Stream<'i>,
) -> Result<CssScrollSnapStop, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssScrollSnapStop::Normal),
        "always" => Ok(CssScrollSnapStop::Always),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_scroll_margin_length<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNumeric<CssLength>, SyntaxError<'i, Error>> {
    parse_length(input, numeric)
}

pub(super) fn parse_scroll_padding_value<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>, SyntaxError<'i, Error>> {
    parse_auto_or(input, |input| {
        parse_nonnegative_length_percentage(input, numeric)
    })
}

pub(super) fn parse_scroll_margin_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<ScrollMarginSide>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_scroll_margin_length(input, numeric))
}

pub(super) fn parse_scroll_padding_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<ScrollPaddingSide>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_scroll_padding_value(input, numeric))
}

pub(super) fn parse_scroll_margin_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<ScrollMarginSide>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_scroll_margin_length(input, numeric))
}

pub(super) fn parse_scroll_padding_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<ScrollPaddingSide>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_scroll_padding_value(input, numeric))
}
