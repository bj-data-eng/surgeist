//! Shared authored Values 4 `<url>` parsing for properties, imports, and fonts.

use super::property_input::Stream;
use crate::diagnostic::error::property::SyntaxError;
use cssparser::Token;
use std::borrow::Cow;

use crate::component::CssComponentValueRef;
use crate::declaration::CssAuthoredFunctionArguments;
use crate::diagnostic::error::property::{Error, basic};
use crate::values::common::{
    CssIdent, CssUrl, CssUrlFunction, CssUrlModifier, CssUrlModifierFunction,
};

pub(super) fn parse_url<'i>(
    input: &mut Stream<'i>,
    _numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssUrl, SyntaxError<'i, Error>> {
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)?.clone() {
        Token::UnquotedUrl(value) => Ok(CssUrl::new(value.to_string())),
        Token::Function(name)
            if name.eq_ignore_ascii_case("url") || name.eq_ignore_ascii_case("src") =>
        {
            let function = if name.eq_ignore_ascii_case("src") {
                CssUrlFunction::Src
            } else {
                CssUrlFunction::Url
            };
            let (value, modifiers) = input.within_block(|input| {
                let value = input.consume_string().map_err(basic)?.to_string();
                let mut modifiers = Vec::new();
                while !input.only_whitespace_remains() {
                    let modifier_start = input.mark();
                    let modifier_location = input.site();
                    match input.consume_after_whitespace().map_err(basic)?.clone() {
                        Token::Ident(value) => {
                            modifiers.push(CssUrlModifier::Ident(CssIdent::new(value.to_string())));
                        }
                        Token::Function(name) => {
                            let arguments = input.within_block(|input| {
                                let start = input.site();
                                consume_url_modifier_components(input)?;
                                Ok(CssAuthoredFunctionArguments::new(input.text_since(start)))
                            })?;
                            let modifier_end = input.mark();
                            input.restore(modifier_start);
                            let collected = input.collect_component().map_err(|error| {
                                crate::diagnostic::error::property::invalid_component_value(
                                    modifier_location,
                                    error,
                                )
                            })?;
                            input.restore(modifier_end);
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
                                return Err(crate::diagnostic::error::property::unexpected_at(
                                    input.site(),
                                ));
                            };
                            modifiers.push(CssUrlModifier::Function(
                                CssUrlModifierFunction::from_parsed(
                                    CssIdent::new(name.to_string()),
                                    arguments,
                                    argument_components,
                                )
                                .map_err(|error| {
                                    crate::diagnostic::error::property::invalid_component_value(
                                        modifier_location,
                                        error,
                                    )
                                })?,
                            ));
                        }
                        token => {
                            return Err(modifier_location.unexpected::<Error>(token));
                        }
                    }
                }
                Ok((value, modifiers))
            })?;
            Ok(CssUrl::from_parts(function, value, modifiers))
        }
        token => Err(location.unexpected::<Error>(token)),
    }
}

fn consume_url_modifier_components<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<(), SyntaxError<'i, Error>> {
    while !input.only_whitespace_remains() {
        let location = input.site();
        match input.consume_token_with_comments().map_err(basic)?.clone() {
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                input.within_block(consume_url_modifier_components)?;
            }
            token @ (Token::BadString(_)
            | Token::BadUrl(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket) => {
                return Err(location.unexpected::<Error>(token));
            }
            _ => {}
        }
    }
    Ok(())
}
