//! Shared authored Values 4 `<url>` parsing for properties, imports, and fonts.

use super::property_input::Parser;
use crate::error::property::ParseError;
use cssparser::Token;
use std::borrow::Cow;

use crate::CssComponentValueRef;
use crate::error::property::{Error, basic};
use crate::syntax::*;

pub(super) fn parse_url<'i, 't>(
    input: &mut Parser<'i, 't>,
    _numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssUrl, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::UnquotedUrl(value) => Ok(CssUrl::new(value.to_string())),
        Token::Function(name)
            if name.eq_ignore_ascii_case("url") || name.eq_ignore_ascii_case("src") =>
        {
            let function = if name.eq_ignore_ascii_case("src") {
                CssUrlFunction::Src
            } else {
                CssUrlFunction::Url
            };
            let (value, modifiers) = input.parse_nested_block(|input| {
                let value = input.expect_string_cloned().map_err(basic)?.to_string();
                let mut modifiers = Vec::new();
                while !input.is_exhausted() {
                    let modifier_start = input.state();
                    let modifier_location = input.current_source_location();
                    match input.next().map_err(basic)?.clone() {
                        Token::Ident(value) => {
                            modifiers.push(CssUrlModifier::Ident(CssIdent::new(value.to_string())));
                        }
                        Token::Function(name) => {
                            let arguments = input.parse_nested_block(|input| {
                                let start = input.position();
                                consume_url_modifier_components(input)?;
                                Ok(CssAuthoredFunctionArguments::new(input.slice_from(start)))
                            })?;
                            let modifier_end = input.state();
                            input.reset(&modifier_start);
                            let collected = input.collect_component().map_err(|error| {
                                crate::error::property::invalid_component_value(
                                    modifier_location,
                                    error,
                                )
                            })?;
                            input.reset(&modifier_end);
                            let argument_components = match collected {
                                Cow::Borrowed(component) => match component.view() {
                                    CssComponentValueRef::Function(function) => {
                                        Some(function.values().clone())
                                    }
                                    _ => None,
                                },
                                Cow::Owned(component) => component.into_function_values(),
                            };
                            let Some(argument_components) = argument_components else {
                                return Err(crate::error::property::unexpected_at(
                                    input.current_source_location(),
                                ));
                            };
                            modifiers.push(CssUrlModifier::Function(
                                CssUrlModifierFunction::from_parsed(
                                    CssIdent::new(name.to_string()),
                                    arguments,
                                    argument_components,
                                )
                                .map_err(|error| {
                                    crate::error::property::invalid_component_value(
                                        modifier_location,
                                        error,
                                    )
                                })?,
                            ));
                        }
                        token => {
                            return Err(
                                modifier_location.new_unexpected_token_error::<Error>(token)
                            );
                        }
                    }
                }
                Ok((value, modifiers))
            })?;
            Ok(CssUrl::from_parts(function, value, modifiers))
        }
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn consume_url_modifier_components<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    while !input.is_exhausted() {
        let location = input.current_source_location();
        match input
            .next_including_whitespace_and_comments()
            .map_err(basic)?
            .clone()
        {
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                input.parse_nested_block(consume_url_modifier_components)?;
            }
            token @ (Token::BadString(_)
            | Token::BadUrl(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket) => {
                return Err(location.new_unexpected_token_error::<Error>(token));
            }
            _ => {}
        }
    }
    Ok(())
}
