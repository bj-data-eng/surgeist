use super::combinators::{Combinator, Tail, commit, fill, group_operand, parse_group};
use super::property_input::Parser;
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::Token;

use super::values::{next_is_delim, parse_integer_value};
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssNumeric, CssPositive};
use crate::values::common::CssIdent;
use crate::values::layout::{CssGridArea, CssGridLine, CssGridLineName, CssGridLineRange};

/// `auto | <custom-ident> | [ <integer> && <custom-ident>? ] |
/// [ span && [ <integer [1,∞]> || <custom-ident> ] ]`, ending at `/`.
///
/// The non-auto forms share one table: `span? && [ <integer> || <custom-ident> ]`.
/// The bracketed `||` is a single `&&` operand, so `span` may precede or
/// follow the integer and name but never separate them (Values 4 §2.2).
/// Integer sign and the span requirements are post-conditions.
pub(crate) fn parse_grid_line<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridLine, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        if !input.is_exhausted() && !next_is_delim(input, '/') {
            return Err(unexpected_at(input.current_source_location()));
        }
        return Ok(CssGridLine::Auto);
    }
    let (mut integer, mut name) = (None, None);
    let [span, _] = parse_group::<2>(
        input,
        Combinator::AllOf {
            optional: [true, false],
        },
        Tail::Reject,
        |input| next_is_delim(input, '/'),
        |operand, input| match operand {
            0 => Ok(input
                .try_parse(|input| input.expect_ident_matching("span"))
                .is_ok()),
            1 => group_operand::<2>(
                input,
                |input| next_is_delim(input, '/'),
                |operand, input| match operand {
                    0 => commit(&mut integer, input, next_starts_integer, |input| {
                        parse_integer_value(input, numeric)
                    }),
                    1 => fill(&mut name, input, parse_grid_line_name),
                    _ => unreachable!("two grid-line group operands"),
                },
            ),
            _ => unreachable!("two grid-line operands"),
        },
    )?;
    if span {
        let positive = integer
            .map(CssNumeric::try_narrow::<CssPositive>)
            .transpose()
            .map_err(|_| unexpected_at(input.current_source_location()))?;
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

/// A number or function commits the integer operand.
fn next_starts_integer(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let starts = matches!(input.next(), Ok(Token::Number { .. } | Token::Function(_)));
    input.reset(&state);
    starts
}

fn parse_grid_line_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssGridLineName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssGridLineName::try_new(CssIdent::new(ident.as_ref())).ok_or_else(|| unexpected_at(location))
}

pub(crate) fn parse_grid_line_range<'i, 't>(
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

pub(crate) fn parse_grid_area<'i, 't>(
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
