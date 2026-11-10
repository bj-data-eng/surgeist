use super::combinators::{Combinator, Tail, commit, fill, group_operand, parse_group};
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;

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
pub(crate) fn parse_grid_line<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridLine, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        if !input.only_whitespace_remains() && !next_is_delim(input, '/') {
            return Err(unexpected_at(input.site()));
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
            0 => Ok(input.attempt(|input| input.consume_keyword("span")).is_ok()),
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
            .map_err(|_| unexpected_at(input.site()))?;
        CssGridLine::try_span(positive, name).ok_or_else(|| unexpected_at(input.site()))
    } else if let Some(integer) = integer {
        CssGridLine::try_indexed(integer, name).ok_or_else(|| unexpected_at(input.site()))
    } else {
        name.map(CssGridLine::Name)
            .ok_or_else(|| unexpected_at(input.site()))
    }
}

/// A number or function commits the integer operand.
fn next_starts_integer(input: &mut Stream<'_>) -> bool {
    let state = input.mark();
    let starts = matches!(
        input.consume_after_whitespace(),
        Ok(Token::Number(_) | Token::Function(_))
    );
    input.restore(state);
    starts
}

fn parse_grid_line_name<'i>(
    input: &mut Stream<'i>,
) -> Result<CssGridLineName, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    CssGridLineName::try_new(CssIdent::new(ident)).ok_or_else(|| unexpected_at(location))
}

pub(crate) fn parse_grid_line_range<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridLineRange, SyntaxError<'i, Error>> {
    let start = parse_grid_line(input, numeric)?;
    let end = if input.attempt(|input| input.consume_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    Ok(CssGridLineRange::new(start, end))
}

pub(crate) fn parse_grid_area<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGridArea, SyntaxError<'i, Error>> {
    let row_start = parse_grid_line(input, numeric)?;
    let column_start = if input.attempt(|input| input.consume_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    let row_end = if input.attempt(|input| input.consume_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    let column_end = if input.attempt(|input| input.consume_delim('/')).is_ok() {
        Some(parse_grid_line(input, numeric)?)
    } else {
        None
    };
    CssGridArea::try_new(row_start, column_start, row_end, column_end)
        .ok_or_else(|| unexpected_at(input.site()))
}
