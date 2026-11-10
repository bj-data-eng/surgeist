use super::border_style::parse_border_style;
use super::border_width::{parse_exact_line_triple, parse_exact_line_width};
use super::combinators::{Combinator, Tail, fill, keyword, parse_group};
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::{next_is_ident, parse_positive_integer_value};
use crate::values::border::CssLineStyle;
use crate::values::border::CssLineWidth;
use crate::values::box_model::CssSizeValue;
use crate::values::layout::{
    CssColumnCount, CssColumnFill, CssColumnRule, CssColumnSpan, CssColumns,
};

pub(crate) fn parse_column_count<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssColumnCount, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        Ok(CssColumnCount::Auto)
    } else {
        parse_positive_integer_value(input, numeric).map(CssColumnCount::Count)
    }
}

pub(crate) fn parse_column_fill<'i>(
    input: &mut Stream<'i>,
) -> Result<CssColumnFill, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssColumnFill::Auto),
        "balance" => Ok(CssColumnFill::Balance),
        "balance-all" => Ok(CssColumnFill::BalanceAll),
        _ => Err(unexpected_at(location)),
    }
}

pub(crate) fn parse_line_style<'i>(
    input: &mut Stream<'i>,
) -> Result<CssLineStyle, SyntaxError<'i, Error>> {
    parse_border_style(input)
}

pub(crate) fn parse_line_width<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssLineWidth, SyntaxError<'i, Error>> {
    parse_exact_line_width(input, numeric)
}

pub(crate) fn parse_column_rule<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssColumnRule, SyntaxError<'i, Error>> {
    let (width, style, color) = parse_exact_line_triple(input, numeric)?;
    Ok(CssColumnRule::try_new(width, style, color).expect("parsed nonempty column rule"))
}

pub(crate) fn parse_column_span<'i>(
    input: &mut Stream<'i>,
) -> Result<CssColumnSpan, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssColumnSpan::None),
        "all" => Ok(CssColumnSpan::All),
        _ => Err(unexpected_at(location)),
    }
}

pub(crate) fn parse_column_width<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssSizeValue, SyntaxError<'i, Error>> {
    super::sizing::parse_size_value(input, numeric)
}

pub(crate) fn parse_columns<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssColumns, SyntaxError<'i, Error>> {
    // `<'column-width'> || <'column-count'>`. Both admit `auto`, so `auto`
    // operands are ambiguous and the group holds at most two components.
    let (mut width, mut count): (Option<CssSizeValue>, _) = (None, None);
    let mut autos = [None; 2];
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| {
            let components = autos.iter().flatten().count()
                + usize::from(width.is_some())
                + usize::from(count.is_some());
            match operand {
                _ if components == 2 => Ok(false),
                0 | 1 => keyword(&mut autos[operand], input, &[("auto", ())]),
                _ if next_is_ident(input, "auto") => Ok(false),
                2 => fill(&mut width, input, |input| {
                    parse_column_width(input, numeric)
                }),
                3 => fill(&mut count, input, |input| {
                    parse_positive_integer_value(input, numeric).map(CssColumnCount::Count)
                }),
                _ => unreachable!("four columns operands"),
            }
        },
    )?;

    Ok(CssColumns::new(
        width.unwrap_or(CssSizeValue::Auto),
        count.unwrap_or(CssColumnCount::Auto),
    ))
}
