//! The one parser for `logical? <T>{1,4}`, `<T>{1,2}` and `auto | <T>`.
//!
//! Each family supplies only the parser of one element; the `logical` switch, the count bounds,
//! the location of a surplus value and the `auto` keyword are decided here once.

use super::property_input::Stream;
use crate::diagnostic::error::property::{Error, SyntaxError, unexpected_at};
use crate::values::common::{CssAutoOr, CssBoxSideKind, CssFourSides, CssSidePair};

/// Consumes an optional leading `logical` and returns the side assignment it selects.
pub(super) fn parse_side_kind<'i>(input: &mut Stream<'i>) -> CssBoxSideKind {
    if input
        .attempt(|input| input.consume_keyword("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    }
}

/// Parses `logical? <T>{1,4}` to the end of the input.
pub(super) fn parse_four_sides<'i, T>(
    input: &mut Stream<'i>,
    mut element: impl FnMut(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Result<CssFourSides<T>, SyntaxError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.only_whitespace_remains() {
        authored.push(element(input)?);
        if authored.len() == 4 && !input.only_whitespace_remains() {
            return Err(unexpected_at(input.site()));
        }
    }
    CssFourSides::try_new(kind, authored).ok_or_else(|| unexpected_at(input.site()))
}

/// Parses `<T>{1,2}` to the end of the input, retaining whether the second value was authored.
pub(super) fn parse_side_pair<'i, T>(
    input: &mut Stream<'i>,
    mut element: impl FnMut(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Result<CssSidePair<T>, SyntaxError<'i, Error>> {
    let first = element(input)?;
    let second = if input.only_whitespace_remains() {
        None
    } else {
        Some(element(input)?)
    };
    Ok(CssSidePair::from_parts(first, second))
}

/// Parses `auto | <T>`: the keyword, or else one element.
pub(super) fn parse_auto_or<'i, T>(
    input: &mut Stream<'i>,
    element: impl FnOnce(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Result<CssAutoOr<T>, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        Ok(CssAutoOr::Auto)
    } else {
        element(input).map(CssAutoOr::Value)
    }
}
