use cssparser::{ParseError, Parser, Token};

use super::values::{next_is_delim, parse_integer_value};
use crate::error::{Error, basic, unexpected_at};
use crate::numeric::NumericInputContext;
use crate::syntax::{
    CssGridArea, CssGridLine, CssGridLineName, CssGridLineRange, CssIdent, CssIntegerValue,
    CssPositiveIntegerLiteral, CssPositiveIntegerValue,
};

pub(super) fn parse_grid_line<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridLine, ParseError<'i, Error>> {
    let mut span = false;
    let mut name = None;
    let mut integer = None;
    while !input.is_exhausted() && !next_is_delim(input, '/') {
        let location = input.current_source_location();
        let state = input.state();
        match input.next().map_err(basic)? {
            Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
                if span || name.is_some() || integer.is_some() {
                    return Err(unexpected_at(location));
                }
                if !input.is_exhausted() && !next_is_delim(input, '/') {
                    return Err(unexpected_at(input.current_source_location()));
                }
                return Ok(CssGridLine::Auto);
            }
            Token::Ident(ident) if ident.eq_ignore_ascii_case("span") => {
                if span {
                    return Err(unexpected_at(location));
                }
                span = true;
            }
            Token::Ident(ident) => {
                if name.is_some() {
                    return Err(unexpected_at(location));
                }
                name = Some(
                    CssGridLineName::try_new(CssIdent::new(ident.as_ref()))
                        .ok_or_else(|| unexpected_at(location))?,
                );
            }
            Token::Number { .. } | Token::Function(_) => {
                input.reset(&state);
                if integer.is_some() {
                    return Err(unexpected_at(location));
                }
                integer = Some(parse_integer_value(input, numeric)?);
            }
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }
    if span {
        let positive = match integer {
            Some(CssIntegerValue::Literal(value)) => Some(CssPositiveIntegerValue::Literal(
                CssPositiveIntegerLiteral::try_new(value)
                    .ok_or_else(|| unexpected_at(input.current_source_location()))?,
            )),
            Some(CssIntegerValue::Calculation(value)) => {
                Some(CssPositiveIntegerValue::Calculation(value))
            }
            None => None,
        };
        CssGridLine::try_span(positive, name)
            .ok_or_else(|| unexpected_at(input.current_source_location()))
    } else if let Some(integer) = integer {
        CssGridLine::try_indexed(integer, name)
            .ok_or_else(|| unexpected_at(input.current_source_location()))
    } else {
        name.map(CssGridLine::Name)
            .ok_or_else(|| unexpected_at(input.current_source_location()))
    }
}

pub(super) fn parse_grid_line_range<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridLineRange, ParseError<'i, Error>> {
    let start = parse_grid_line(input, numeric)?;
    let end = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    Ok(CssGridLineRange::new(start, end))
}

pub(super) fn parse_grid_area<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridArea, ParseError<'i, Error>> {
    let row_start = parse_grid_line(input, numeric)?;
    let column_start = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    let row_end = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    let column_end = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    CssGridArea::try_new(row_start, column_start, row_end, column_end)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
