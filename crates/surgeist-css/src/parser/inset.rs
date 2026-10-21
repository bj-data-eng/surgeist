//! Authored positioned offsets from Position 3 and Logical Properties 1.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, unexpected_at};

use super::values::parse_length_percentage;
use crate::numeric::NumericInputContext;
use crate::{CssBoxSideKind, CssInsetPair, CssInsetShorthand, CssInsetValue};

pub(super) fn parse_inset_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssInsetValue::Auto);
    }
    parse_length_percentage(input, numeric).map(CssInsetValue::LengthPercentage)
}

pub(super) fn parse_inset_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetPair, ParseError<'i, Error>> {
    let start = parse_inset_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_inset_value(input, numeric)?)
    };
    Ok(CssInsetPair::new(start, end))
}

pub(super) fn parse_inset_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetShorthand, ParseError<'i, Error>> {
    let kind = if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    };
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_inset_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssInsetShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
