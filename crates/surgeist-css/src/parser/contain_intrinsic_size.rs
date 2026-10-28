//! Sizing 4 authored contained intrinsic-size declarations.

use super::property_input::Stream;
use super::side_shapes::parse_side_pair;
use crate::diagnostic::error::property::{Error, SyntaxError};

use super::values::parse_nonnegative_length;
use crate::numeric::expression::NumericInputContext;
use crate::values::box_model::{CssContainIntrinsicSizeFallback, CssContainIntrinsicSizeValue};
use crate::values::common::CssSidePair;

pub(super) fn parse_contain_intrinsic_size_value<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssContainIntrinsicSizeValue, SyntaxError<'i, Error>> {
    let auto = input.attempt(|input| input.consume_keyword("auto")).is_ok();
    let fallback = if input.attempt(|input| input.consume_keyword("none")).is_ok() {
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

pub(super) fn parse_contain_intrinsic_size<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssContainIntrinsicSizeValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| {
        parse_contain_intrinsic_size_value(input, numeric)
    })
}
