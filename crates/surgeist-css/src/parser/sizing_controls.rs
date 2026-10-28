//! Sizing 4 physical size pairs and finite intrinsic controls.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::match_ignore_ascii_case;

use super::side_shapes::parse_side_pair;
use super::sizing::{parse_max_size_value, parse_size_value};
use crate::numeric::expression::NumericInputContext;
use crate::values::box_model::{CssFrameSizing, CssMinIntrinsicSizing};
use crate::values::box_model::{CssMaxSizeValue, CssSizeValue};
use crate::values::common::CssSidePair;

pub(super) fn parse_size_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssSizeValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_size_value(input, numeric))
}

pub(super) fn parse_max_size_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssMaxSizeValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_max_size_value(input, numeric))
}

pub(super) fn parse_frame_sizing<'i>(
    input: &mut Stream<'i>,
) -> Result<CssFrameSizing, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFrameSizing::Auto),
        "content-width" => Ok(CssFrameSizing::ContentWidth),
        "content-height" => Ok(CssFrameSizing::ContentHeight),
        "content-block-size" => Ok(CssFrameSizing::ContentBlockSize),
        "content-inline-size" => Ok(CssFrameSizing::ContentInlineSize),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_min_intrinsic_sizing<'i>(
    input: &mut Stream<'i>,
) -> Result<CssMinIntrinsicSizing, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    if ident.eq_ignore_ascii_case("legacy") {
        return Ok(CssMinIntrinsicSizing::Legacy);
    }
    let first_scroll = if ident.eq_ignore_ascii_case("zero-if-scroll") {
        true
    } else if ident.eq_ignore_ascii_case("zero-if-extrinsic") {
        false
    } else {
        return Err(unexpected_at(keyword_location));
    };
    if input.only_whitespace_remains() {
        return Ok(if first_scroll {
            CssMinIntrinsicSizing::ZeroIfScroll
        } else {
            CssMinIntrinsicSizing::ZeroIfExtrinsic
        });
    }
    let keyword_location = input.site();
    let second = input.consume_ident().map_err(basic)?;
    if (first_scroll && second.eq_ignore_ascii_case("zero-if-extrinsic"))
        || (!first_scroll && second.eq_ignore_ascii_case("zero-if-scroll"))
    {
        Ok(CssMinIntrinsicSizing::ZeroIfScrollAndExtrinsic)
    } else {
        Err(unexpected_at(keyword_location))
    }
}
