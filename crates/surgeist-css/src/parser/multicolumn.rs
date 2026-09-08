use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, LengthGrammar, parse_color, parse_length_with_context, parse_typed_calculation,
};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) fn parse_column_count<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssColumnCount, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssColumnCount::Auto)
    } else {
        parse_positive_integer_value(input, "column-count").map(CssColumnCount::Count)
    }
}

fn parse_positive_integer_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> Result<CssPositiveIntegerValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => CssPositiveInteger::try_new(*value)
            .map(CssPositiveIntegerValue::Literal)
            .ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    format!("{context} must be a positive integer"),
                )
            }),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            format!("{context} must be an integer"),
        )),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Integer))
            .map(CssIntegerCalculation::from_expression)
            .map(CssPositiveIntegerValue::Calculation),
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
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssLineStyle::None),
        "hidden" => Ok(CssLineStyle::Hidden),
        "dotted" => Ok(CssLineStyle::Dotted),
        "dashed" => Ok(CssLineStyle::Dashed),
        "solid" => Ok(CssLineStyle::Solid),
        "double" => Ok(CssLineStyle::Double),
        "groove" => Ok(CssLineStyle::Groove),
        "ridge" => Ok(CssLineStyle::Ridge),
        "inset" => Ok(CssLineStyle::Inset),
        "outset" => Ok(CssLineStyle::Outset),
        _ => Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("line-style", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_line_width<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssLineWidth, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "thin" => Ok(CssLineWidth::Thin),
            "medium" => Ok(CssLineWidth::Medium),
            "thick" => Ok(CssLineWidth::Thick),
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("line-width", ident.as_ref()),
            )),
        };
    }

    parse_non_negative_length(input, "column-rule-width").map(CssLineWidth::Length)
}

fn parse_non_negative_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> Result<CssNonNegativeLength, ParseError<'i, Error>> {
    let value = parse_length_with_context(input, LengthGrammar::NonNegativeLength, context)?;
    CssNonNegativeLength::try_new(value)
        .ok_or_else(|| unsupported_value(input, None, format!("invalid {context}")))
}

pub(super) fn parse_column_rule<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssColumnRule, ParseError<'i, Error>> {
    let mut width = None;
    let mut style = None;
    let mut color = None;

    while !input.is_exhausted() {
        if width.is_none()
            && let Ok(value) = input.try_parse(parse_line_width)
        {
            width = Some(value);
            continue;
        }
        if style.is_none()
            && let Ok(value) = input.try_parse(parse_line_style)
        {
            style = Some(value);
            continue;
        }
        if color.is_none()
            && let Ok(value) = input.try_parse(parse_color)
        {
            color = Some(value);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported or duplicate column-rule component",
        ));
    }

    if width.is_none() && style.is_none() && color.is_none() {
        Err(unsupported_value(
            input,
            None,
            "column-rule shorthand is missing a component",
        ))
    } else {
        Ok(CssColumnRule::new(width, style, color))
    }
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
) -> Result<CssColumnWidth, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssColumnWidth::Auto)
    } else {
        parse_non_negative_length(input, "column-width").map(CssColumnWidth::Length)
    }
}

pub(super) fn parse_columns<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssColumns, ParseError<'i, Error>> {
    let mut width = None;
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
            && let Ok(value) = input.try_parse(parse_column_width)
        {
            width = Some(value);
        } else if count.is_none()
            && let Ok(value) =
                input.try_parse(|input| parse_positive_integer_value(input, "columns"))
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
