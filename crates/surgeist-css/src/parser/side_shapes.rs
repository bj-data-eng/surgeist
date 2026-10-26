//! The one parser for `logical? <T>{1,4}`, `<T>{1,2}` and `auto | <T>`.
//!
//! Each family supplies only the parser of one element; the `logical` switch, the count bounds,
//! the location of a surplus value and the `auto` keyword are decided here once.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, unexpected_at};
use crate::{CssAutoOr, CssBoxSideKind, CssFourSides, CssSidePair};

/// Consumes an optional leading `logical` and returns the side assignment it selects.
pub(super) fn parse_side_kind<'i, 't>(input: &mut Parser<'i, 't>) -> CssBoxSideKind {
    if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    }
}

/// Parses `logical? <T>{1,4}` to the end of the input.
pub(super) fn parse_four_sides<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    mut element: impl FnMut(&mut Parser<'i, 't>) -> Result<T, ParseError<'i, Error>>,
) -> Result<CssFourSides<T>, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(element(input)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssFourSides::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

/// Parses `<T>{1,2}` to the end of the input, retaining whether the second value was authored.
pub(super) fn parse_side_pair<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    mut element: impl FnMut(&mut Parser<'i, 't>) -> Result<T, ParseError<'i, Error>>,
) -> Result<CssSidePair<T>, ParseError<'i, Error>> {
    let first = element(input)?;
    let second = if input.is_exhausted() {
        None
    } else {
        Some(element(input)?)
    };
    Ok(CssSidePair::from_parts(first, second))
}

/// Parses `auto | <T>`: the keyword, or else one element.
pub(super) fn parse_auto_or<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    element: impl FnOnce(&mut Parser<'i, 't>) -> Result<T, ParseError<'i, Error>>,
) -> Result<CssAutoOr<T>, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssAutoOr::Auto)
    } else {
        element(input).map(CssAutoOr::Value)
    }
}
