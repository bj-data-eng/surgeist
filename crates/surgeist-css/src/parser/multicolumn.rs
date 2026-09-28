use super::border_style::parse_border_style;
use super::border_width::{parse_exact_line_triple, parse_exact_line_width};
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{CalculationRoot, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) fn parse_column_count<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssColumnCount, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssColumnCount::Auto)
    } else {
        parse_positive_integer_value(input, numeric, "column-count").map(CssColumnCount::Count)
    }
}

fn parse_positive_integer_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    context: &str,
) -> Result<CssPositiveIntegerValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&numeric_start);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, format!("invalid {context} integer"))
            })?;
            let literal =
                crate::CssIntegerLiteral::try_from_component(component).map_err(|_| {
                    unsupported_value_at(location, None, format!("{context} must be an integer"))
                })?;
            let positive = CssPositiveIntegerLiteral::try_new(literal).ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    format!("{context} must be a positive integer"),
                )
            })?;
            Ok(
                match crate::integer_value::exact_i32(positive.integer().numeric().representation())
                    .and_then(CssPositiveInteger::try_new)
                {
                    Some(value) => CssPositiveIntegerValue::Literal(value),
                    None => CssPositiveIntegerValue::ExactLiteral(positive),
                },
            )
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .map(CssPositiveIntegerValue::Calculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_column_fill<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssColumnFill, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssColumnFill::Auto),
        "balance" => Ok(CssColumnFill::Balance),
        "balance-all" => Ok(CssColumnFill::BalanceAll),
        _ => Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("column-fill", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_line_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssLineStyle, ParseError<'i, Error>> {
    parse_border_style(input)
}

pub(super) fn parse_line_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssLineWidth, ParseError<'i, Error>> {
    parse_exact_line_width(input, numeric, "column-rule-width")
}

pub(super) fn parse_column_rule<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssColumnRule, ParseError<'i, Error>> {
    let (width, style, color) =
        parse_exact_line_triple(input, numeric, "column-rule", "column-rule-width")?;
    Ok(
        CssColumnRule::try_new(width, style, color.map(|value| value.into_parts().0))
            .expect("parsed nonempty column rule"),
    )
}

pub(super) fn parse_column_span<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssColumnSpan, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssColumnSpan::None),
        "all" => Ok(CssColumnSpan::All),
        _ => Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("column-span", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_column_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssColumnWidth, ParseError<'i, Error>> {
    super::sizing::parse_size_value(input, numeric)
}

pub(super) fn parse_columns<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssColumns, ParseError<'i, Error>> {
    let mut width: Option<CssColumnWidth> = None;
    let mut count = None;
    let mut autos = 0_u8;

    while !input.is_exhausted() {
        let location = input.current_source_location();
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            autos += 1;
        } else if width.is_none()
            && let Ok(value) = input.try_parse(|input| parse_column_width(input, numeric))
        {
            width = Some(value);
        } else if count.is_none()
            && let Ok(value) =
                input.try_parse(|input| parse_positive_integer_value(input, numeric, "columns"))
        {
            count = Some(CssColumnCount::Count(value));
        } else {
            return Err(unsupported_value_at(
                location,
                None,
                "unsupported or duplicate columns component",
            ));
        }

        let component_count = u8::from(width.is_some()) + u8::from(count.is_some()) + autos;
        if component_count > 2 {
            return Err(unsupported_value_at(
                location,
                None,
                "columns shorthand has too many components",
            ));
        }
    }

    if width.is_none() && count.is_none() && autos == 0 {
        return Err(unsupported_value(
            input,
            None,
            "columns shorthand is missing a component",
        ));
    }

    Ok(CssColumns::new(
        width.unwrap_or(CssColumnWidth::Auto),
        count.unwrap_or(CssColumnCount::Auto),
    ))
}
