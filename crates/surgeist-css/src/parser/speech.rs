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

pub(super) fn parse_voice_stress<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<crate::CssVoiceStress, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    cssparser::match_ignore_ascii_case! { &ident,
        "normal" => Ok(crate::CssVoiceStress::Normal),
        "strong" => Ok(crate::CssVoiceStress::Strong),
        "moderate" => Ok(crate::CssVoiceStress::Moderate),
        "none" => Ok(crate::CssVoiceStress::None),
        "reduced" => Ok(crate::CssVoiceStress::Reduced),
        _ => Err(location.new_unexpected_token_error(Token::Ident(ident))),
    }
}

pub(super) fn parse_voice_family<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoiceFamily, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("preserve"))
        .is_ok()
    {
        return Ok(crate::CssVoiceFamily::Preserve);
    }
    let mut entries = Vec::new();
    loop {
        entries.push(parse_voice_entry(input, numeric)?);
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
    }
    Ok(crate::CssVoiceFamily::Voices(
        crate::CssVoiceFamilyList::from_parser(entries).expect("parsed nonempty list"),
    ))
}
fn voice_age(value: &str) -> Option<crate::CssVoiceAge> {
    cssparser::match_ignore_ascii_case! { value, "child" => Some(crate::CssVoiceAge::Child), "young" => Some(crate::CssVoiceAge::Young), "old" => Some(crate::CssVoiceAge::Old), _ => None }
}
fn voice_gender(value: &str) -> Option<crate::CssVoiceGender> {
    cssparser::match_ignore_ascii_case! { value, "male" => Some(crate::CssVoiceGender::Male), "female" => Some(crate::CssVoiceGender::Female), "neutral" => Some(crate::CssVoiceGender::Neutral), _ => None }
}
fn parse_voice_entry<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoiceFamilyEntry, ParseError<'i, Error>> {
    use crate::{CssGenericVoice, CssVoiceFamilyEntry, CssVoiceFamilyName};
    input.skip_whitespace();
    let location = input.current_source_location();
    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return CssVoiceFamilyName::try_quoted(value.to_string())
            .map(CssVoiceFamilyEntry::Name)
            .map_err(|_| crate::error::unsupported_value_at(location, None, "invalid voice name"));
    }
    let start = input.state();
    let first = input.expect_ident_cloned().map_err(basic)?;
    let age = voice_age(&first);
    let gender = if age.is_some() {
        input
            .try_parse(|input| {
                let ident = input.expect_ident_cloned()?;
                voice_gender(&ident)
                    .ok_or_else(|| input.new_unexpected_token_error::<Error>(Token::Ident(ident)))
            })
            .ok()
    } else {
        voice_gender(&first)
    };
    if let Some(gender) = gender {
        let variant = if input.is_exhausted() || super::values::next_is_comma(input) {
            None
        } else {
            Some(super::values::parse_positive_integer_value(
                input,
                numeric,
                "voice-family variant",
            )?)
        };
        return CssGenericVoice::from_parser(age, gender, variant)
            .map(CssVoiceFamilyEntry::Generic)
            .map_err(|_| {
                crate::error::unsupported_value_at(location, None, "invalid voice variant")
            });
    }
    input.reset(&start);
    let mut identifiers = Vec::new();
    while !input.is_exhausted() && !super::values::next_is_comma(input) {
        input.skip_whitespace();
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if crate::speech::reserved_voice_identifier(&ident) {
            return Err(location.new_unexpected_token_error(Token::Ident(ident)));
        }
        identifiers.push(crate::CssIdent::try_new(ident.to_string()).map_err(|_| {
            crate::error::unsupported_value_at(location, None, "invalid voice identifier")
        })?);
    }
    CssVoiceFamilyName::try_identifiers(identifiers)
        .map(CssVoiceFamilyEntry::Name)
        .ok_or_else(|| input.new_error(cssparser::BasicParseErrorKind::EndOfInput))
}
