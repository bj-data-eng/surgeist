//! Sizing 4 authored contained intrinsic-size declarations.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError};

use super::values::parse_nonnegative_length;
use crate::numeric::NumericInputContext;
use crate::{
    CssContainIntrinsicSize, CssContainIntrinsicSizeFallback, CssContainIntrinsicSizeValue,
};

pub(super) fn parse_contain_intrinsic_size_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssContainIntrinsicSizeValue, ParseError<'i, Error>> {
    let auto = input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok();
    let fallback = if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        CssContainIntrinsicSizeFallback::None
    } else {
        CssContainIntrinsicSizeFallback::Length(parse_nonnegative_length(input, numeric)?)
    };
    Ok(if auto {
        CssContainIntrinsicSizeValue::with_auto(fallback)
    } else {
        CssContainIntrinsicSizeValue::new(fallback)
    })
}

pub(super) fn parse_contain_intrinsic_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssContainIntrinsicSize, ParseError<'i, Error>> {
    let width = parse_contain_intrinsic_size_value(input, numeric)?;
    let height = if input.is_exhausted() {
        None
    } else {
        Some(parse_contain_intrinsic_size_value(input, numeric)?)
    };
    Ok(CssContainIntrinsicSize::new(width, height))
}
