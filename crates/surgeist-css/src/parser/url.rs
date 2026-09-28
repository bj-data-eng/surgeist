//! Shared authored Values 4 `<url>` parsing for properties, imports, and fonts.

use cssparser::{ParseError, Parser, Token};

use crate::error::{Error, basic};
use crate::syntax::*;

pub(super) fn parse_url<'i, 't>(
    input: &mut Parser<'i, 't>,
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
                    let modifier_location = input.current_source_location();
                    match input.next().map_err(basic)?.clone() {
                        Token::Ident(value) => {
                            modifiers.push(CssUrlModifier::Ident(CssIdent::new(value.to_string())));
                        }
                        Token::Function(name) => {
                            let arguments = input.parse_nested_block(|input| {
                                let start = input.position();
                                consume_url_modifier_components(input)?;
                                Ok(CssAuthoredFunctionArguments::new(
                                    input.slice_from(start).to_owned(),
                                ))
                            })?;
                            modifiers.push(CssUrlModifier::Function(CssUrlModifierFunction::new(
                                CssIdent::new(name.to_string()),
                                arguments,
                            )));
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
