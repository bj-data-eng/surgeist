//! Exact authored Speech 1 keyword grammars; no speech execution.

use crate::error::{Error, basic};
use crate::{
    CssSpeak, CssSpeakAs, CssSpeakAsPunctuation, CssSpeechBreak, CssSpeechBreakPair,
    CssSpeechBreakStrength,
};
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

pub(super) fn parse_speech_break<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpeechBreak, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let value = cssparser::match_ignore_ascii_case! { &ident,
            "none" => CssSpeechBreak::None,
            "x-weak" => CssSpeechBreak::Strength(CssSpeechBreakStrength::XWeak),
            "weak" => CssSpeechBreak::Strength(CssSpeechBreakStrength::Weak),
            "medium" => CssSpeechBreak::Strength(CssSpeechBreakStrength::Medium),
            "strong" => CssSpeechBreak::Strength(CssSpeechBreakStrength::Strong),
            "x-strong" => CssSpeechBreak::Strength(CssSpeechBreakStrength::XStrong),
            _ => return Err(location.new_unexpected_token_error(Token::Ident(ident))),
        };
        return Ok(value);
    }
    super::timing::parse_duration(input, numeric).map(CssSpeechBreak::Time)
}

pub(super) fn parse_speech_break_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpeechBreakPair, ParseError<'i, Error>> {
    let before = parse_speech_break(input, numeric)?;
    let after = if input.is_exhausted() {
        None
    } else {
        Some(parse_speech_break(input, numeric)?)
    };
    Ok(CssSpeechBreakPair::from_parser(before, after))
}
