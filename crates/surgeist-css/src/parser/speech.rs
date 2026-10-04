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

pub(super) fn parse_cue<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssCue, ParseError<'i, Error>> {
    input.skip_whitespace();
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(crate::CssCue::None);
    }
    let url = super::url::parse_url(input, numeric)?;
    input.skip_whitespace();
    let state = input.state();
    let is_decibel = matches!(input.next(), Ok(Token::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("dB"));
    input.reset(&state);
    let decibel = if is_decibel {
        Some(parse_decibel(input, numeric)?)
    } else {
        None
    };
    Ok(crate::CssCue::Audio(crate::CssAudioCue::from_parser(
        url, decibel,
    )))
}

pub(super) fn parse_cue_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssCuePair, ParseError<'i, Error>> {
    let before = parse_cue(input, numeric)?;
    let after = if input.is_exhausted() {
        None
    } else {
        Some(parse_cue(input, numeric)?)
    };
    Ok(crate::CssCuePair::from_parser(before, after))
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

pub(super) fn parse_voice_duration<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoiceDuration, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(crate::CssVoiceDuration::Auto);
    }
    super::timing::parse_duration(input, numeric).map(crate::CssVoiceDuration::Time)
}
fn voice_level(value: &str) -> Option<crate::CssVoiceLevel> {
    cssparser::match_ignore_ascii_case! { value, "x-low" => Some(crate::CssVoiceLevel::XLow), "low" => Some(crate::CssVoiceLevel::Low), "medium" => Some(crate::CssVoiceLevel::Medium), "high" => Some(crate::CssVoiceLevel::High), "x-high" => Some(crate::CssVoiceLevel::XHigh), _ => None }
}
pub(super) fn parse_voice_pitch_range<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoicePitchRange, ParseError<'i, Error>> {
    let mut level = None;
    let mut absolute = None;
    let mut offset = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let location = input.current_source_location();
        let byte_offset = input.position().byte_index();
        if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
            if let Some(value) = voice_level(&ident)
                && level.is_none()
            {
                level = Some(value);
            } else if ident.eq_ignore_ascii_case("absolute") && absolute.is_none() {
                absolute = Some(location);
            } else {
                return Err(location.new_unexpected_token_error(Token::Ident(ident)));
            }
        } else {
            if offset.is_some() {
                return Err(crate::error::unsupported_value_at(
                    location,
                    None,
                    "only one voice offset is allowed",
                ));
            }
            let component = numeric
                .collect(input)
                .map_err(|error| speech_numeric_error(numeric, &error, location, byte_offset))?;
            offset = Some((component, location, byte_offset));
        }
    }
    if let Some(location) = absolute {
        if level.is_some() {
            return Err(crate::error::unsupported_value_at(
                location,
                None,
                "absolute frequency cannot carry a voice level",
            ));
        }
        let Some((component, location, byte_offset)) = offset else {
            return Err(crate::error::unsupported_value_at(
                location,
                None,
                "absolute requires a frequency",
            ));
        };
        let frequency = crate::CssFrequencyValue::from_parser_component(component, numeric)
            .map_err(|error| speech_numeric_error(numeric, &error, location, byte_offset))?;
        crate::CssVoicePitchRange::from_parser_absolute(frequency)
            .map_err(|error| speech_numeric_error(numeric, &error, location, byte_offset))
    } else {
        let offset = offset
            .map(|(component, location, byte_offset)| {
                crate::CssVoiceOffset::from_parser_component(component, numeric)
                    .map_err(|error| speech_numeric_error(numeric, &error, location, byte_offset))
            })
            .transpose()?;
        crate::CssVoicePitchRange::from_parser_relative(level, offset)
            .map_err(|_| input.new_error(cssparser::BasicParseErrorKind::EndOfInput))
    }
}
fn voice_rate_keyword(value: &str) -> Option<crate::CssVoiceRateKeyword> {
    cssparser::match_ignore_ascii_case! { value, "normal" => Some(crate::CssVoiceRateKeyword::Normal), "x-slow" => Some(crate::CssVoiceRateKeyword::XSlow), "slow" => Some(crate::CssVoiceRateKeyword::Slow), "medium" => Some(crate::CssVoiceRateKeyword::Medium), "fast" => Some(crate::CssVoiceRateKeyword::Fast), "x-fast" => Some(crate::CssVoiceRateKeyword::XFast), _ => None }
}
pub(super) fn parse_voice_rate<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoiceRate, ParseError<'i, Error>> {
    let mut keyword = None;
    let mut percentage = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let location = input.current_source_location();
        let byte_offset = input.position().byte_index();
        if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
            if let Some(value) = voice_rate_keyword(&ident)
                && keyword.is_none()
            {
                keyword = Some(value);
            } else {
                return Err(location.new_unexpected_token_error(Token::Ident(ident)));
            }
        } else {
            if percentage.is_some() {
                return Err(crate::error::unsupported_value_at(
                    location,
                    None,
                    "only one voice-rate percentage is allowed",
                ));
            }
            let component = numeric
                .collect(input)
                .map_err(|error| speech_numeric_error(numeric, &error, location, byte_offset))?;
            let value = if matches!(component.view(), crate::CssComponentValueRef::Token(_)) {
                crate::CssSpecifiedNonNegativePercentage::try_from_component(component)
            } else {
                let values = crate::CssComponentValues::try_new(vec![component])
                    .map_err(|error| crate::error::invalid_component_value(location, error))?;
                numeric
                    .admit(values, crate::numeric::CalculationRoot::Percentage)
                    .and_then(|expression| {
                        crate::CssSpecifiedNonNegativePercentage::try_from_calculation(
                            crate::CssPercentageCalculation::from_expression(expression),
                        )
                    })
            }
            .map_err(|error| speech_numeric_error(numeric, &error, location, byte_offset))?;
            percentage = Some(value);
        }
    }
    crate::CssVoiceRate::from_parser(keyword, percentage)
        .map_err(|_| input.new_error(cssparser::BasicParseErrorKind::EndOfInput))
}
fn speech_numeric_error<'i>(
    numeric: &crate::numeric::NumericInputContext<'_>,
    error: &crate::CssNumericConstructionError,
    fallback: cssparser::SourceLocation,
    offset: usize,
) -> ParseError<'i, Error> {
    let location = numeric.error_location(error, fallback, offset);
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::invalid_component_value(location, component.clone());
    }
    crate::error::unsupported_value_at(location, None, "invalid authored speech numeric value")
}

fn parse_decibel<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssDecibelLiteral, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = numeric
        .collect(input)
        .map_err(|error| speech_numeric_error(numeric, &error, location, offset))?;
    crate::CssDecibelLiteral::try_from_component(component)
        .map_err(|error| crate::error::invalid_component_value(location, error))
}

pub(super) fn parse_voice_balance<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoiceBalance, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let keyword = cssparser::match_ignore_ascii_case! { &ident,
            "left" => crate::CssVoiceBalanceKeyword::Left,
            "center" => crate::CssVoiceBalanceKeyword::Center,
            "right" => crate::CssVoiceBalanceKeyword::Right,
            "leftwards" => crate::CssVoiceBalanceKeyword::Leftwards,
            "rightwards" => crate::CssVoiceBalanceKeyword::Rightwards,
            _ => return Err(location.new_unexpected_token_error(Token::Ident(ident))),
        };
        return Ok(crate::CssVoiceBalance::from_keyword(keyword));
    }
    super::values::parse_specified_number(input, numeric, "voice-balance")
        .map(crate::CssVoiceBalance::from_parser_number)
}

pub(super) fn parse_voice_volume<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssVoiceVolume, ParseError<'i, Error>> {
    input.skip_whitespace();
    if input
        .try_parse(|input| input.expect_ident_matching("silent"))
        .is_ok()
    {
        return Ok(crate::CssVoiceVolume::Silent);
    }
    let mut level = None;
    let mut decibel = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let state = input.state();
        let location = input.current_source_location();
        let token = input.next().map_err(basic)?.clone();
        match &token {
            Token::Ident(ident) if level.is_none() => {
                level = Some(cssparser::match_ignore_ascii_case! { &ident,
                    "x-soft" => crate::CssVoiceVolumeLevel::XSoft,
                    "soft" => crate::CssVoiceVolumeLevel::Soft,
                    "medium" => crate::CssVoiceVolumeLevel::Medium,
                    "loud" => crate::CssVoiceVolumeLevel::Loud,
                    "x-loud" => crate::CssVoiceVolumeLevel::XLoud,
                    _ => return Err(location.new_unexpected_token_error(token)),
                });
            }
            Token::Dimension { unit, .. }
                if decibel.is_none() && unit.eq_ignore_ascii_case("dB") =>
            {
                input.reset(&state);
                decibel = Some(parse_decibel(input, numeric)?);
            }
            _ => return Err(location.new_unexpected_token_error(token)),
        }
    }
    match (level, decibel) {
        (Some(level), decibel) => Ok(crate::CssVoiceVolume::Level { level, decibel }),
        (None, Some(value)) => Ok(crate::CssVoiceVolume::Offset(value)),
        (None, None) => Err(input.new_error(cssparser::BasicParseErrorKind::EndOfInput)),
    }
}
