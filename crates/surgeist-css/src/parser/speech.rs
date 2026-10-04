//! Exact authored Speech 1 keyword grammars; no speech execution.

use crate::error::{Error, basic};
use crate::{CssSpeak, CssSpeakAs, CssSpeakAsPunctuation};
use cssparser::{ParseError, Parser, Token};

pub(super) fn parse_speak<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssSpeak, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match &token {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssSpeak::Auto),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("never") => Ok(CssSpeak::Never),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("always") => Ok(CssSpeak::Always),
        _ => Err(location.new_unexpected_token_error(token)),
    }
}

pub(super) fn parse_speak_as<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssSpeakAs, ParseError<'i, Error>> {
    let mut normal = false;
    let mut spell_out = false;
    let mut digits = false;
    let mut punctuation = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let location = input.current_source_location();
        let token = input.next().map_err(basic)?.clone();
        let valid = match &token {
            Token::Ident(ident) if !normal && ident.eq_ignore_ascii_case("normal") => {
                if spell_out || digits || punctuation.is_some() {
                    false
                } else {
                    normal = true;
                    true
                }
            }
            Token::Ident(ident)
                if !normal && !spell_out && ident.eq_ignore_ascii_case("spell-out") =>
            {
                spell_out = true;
                true
            }
            Token::Ident(ident) if !normal && !digits && ident.eq_ignore_ascii_case("digits") => {
                digits = true;
                true
            }
            Token::Ident(ident)
                if !normal
                    && punctuation.is_none()
                    && ident.eq_ignore_ascii_case("literal-punctuation") =>
            {
                punctuation = Some(CssSpeakAsPunctuation::Literal);
                true
            }
            Token::Ident(ident)
                if !normal
                    && punctuation.is_none()
                    && ident.eq_ignore_ascii_case("no-punctuation") =>
            {
                punctuation = Some(CssSpeakAsPunctuation::None);
                true
            }
            _ => false,
        };
        if !valid {
            return Err(location.new_unexpected_token_error(token));
        }
    }
    if normal {
        return Ok(CssSpeakAs::normal());
    }
    CssSpeakAs::try_new(spell_out, digits, punctuation)
        .ok_or_else(|| input.new_error(cssparser::BasicParseErrorKind::EndOfInput))
}
