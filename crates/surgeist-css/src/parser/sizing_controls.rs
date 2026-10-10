//! Sizing 4 physical size pairs and finite intrinsic controls.

use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use super::sizing::{parse_max_size_value, parse_size_value};
use crate::error::{Error, basic, unexpected_at};
use crate::numeric::NumericInputContext;
use crate::{CssFrameSizing, CssMaxSizePair, CssMinIntrinsicSizing, CssSizePair};

pub(super) fn parse_size_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSizePair, ParseError<'i, Error>> {
    let width = parse_size_value(input, numeric)?;
    let height = if input.is_exhausted() {
        None
    } else {
        Some(parse_size_value(input, numeric)?)
    };
    Ok(CssSizePair::new(width, height))
}

pub(super) fn parse_max_size_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMaxSizePair, ParseError<'i, Error>> {
    let width = parse_max_size_value(input, numeric)?;
    let height = if input.is_exhausted() {
        None
    } else {
        Some(parse_max_size_value(input, numeric)?)
    };
    Ok(CssMaxSizePair::new(width, height))
}

pub(super) fn parse_frame_sizing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssFrameSizing, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFrameSizing::Auto),
        "content-width" => Ok(CssFrameSizing::ContentWidth),
        "content-height" => Ok(CssFrameSizing::ContentHeight),
        "content-block-size" => Ok(CssFrameSizing::ContentBlockSize),
        "content-inline-size" => Ok(CssFrameSizing::ContentInlineSize),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_min_intrinsic_sizing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssMinIntrinsicSizing, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("legacy") {
        return Ok(CssMinIntrinsicSizing::Legacy);
    }
    let first_scroll = if ident.eq_ignore_ascii_case("zero-if-scroll") {
        true
    } else if ident.eq_ignore_ascii_case("zero-if-extrinsic") {
        false
    } else {
        return Err(unexpected_at(input.current_source_location()));
    };
    if input.is_exhausted() {
        return Ok(if first_scroll {
            CssMinIntrinsicSizing::ZeroIfScroll
        } else {
            CssMinIntrinsicSizing::ZeroIfExtrinsic
        });
    }
    let second = input.expect_ident_cloned().map_err(basic)?;
    if (first_scroll && second.eq_ignore_ascii_case("zero-if-extrinsic"))
        || (!first_scroll && second.eq_ignore_ascii_case("zero-if-scroll"))
    {
        Ok(CssMinIntrinsicSizing::ZeroIfScrollAndExtrinsic)
    } else {
        Err(unexpected_at(input.current_source_location()))
    }
}
