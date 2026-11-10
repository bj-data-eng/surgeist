//! Exact authored Speech 1 keyword grammars; no speech execution.

use super::combinators::{Combinator, Tail, commit, parse_group};
use super::side_shapes::parse_side_pair;
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic};
use crate::parser::property_input::StreamExt;
use crate::parser::syntax_types::SyntaxError;
use crate::values::common::CssSidePair;
use crate::values::speech::CssCue;
use crate::values::speech::{
    CssSpeak, CssSpeakAs, CssSpeakAsPunctuation, CssSpeechBreak, CssSpeechBreakStrength,
};
use surgeist_css_parser::Token;

pub(super) fn parse_speak<'i>(input: &mut Stream<'i>) -> Result<CssSpeak, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let token = input.consume_after_whitespace().map_err(basic)?;
    match &token {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssSpeak::Auto),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("never") => Ok(CssSpeak::Never),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("always") => Ok(CssSpeak::Always),
        _ => Err(location.unexpected(token)),
    }
}

pub(super) fn parse_speak_as<'i>(
    input: &mut Stream<'i>,
) -> Result<CssSpeakAs, SyntaxError<'i, Error>> {
    let mut normal = false;
    let mut spell_out = false;
    let mut digits = false;
    let mut punctuation = None;
    while !input.only_whitespace_remains() {
        input.discard_whitespace();
        let location = input.site();
        let token = input.consume_after_whitespace().map_err(basic)?;
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
            return Err(location.unexpected(token));
        }
    }
    if normal {
        return Ok(CssSpeakAs::normal());
    }
    CssSpeakAs::try_new(spell_out, digits, punctuation).ok_or_else(|| input.end_of_stream())
}

pub(super) fn parse_speech_break<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssSpeechBreak, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let value = surgeist_css_parser::match_ignore_ascii_case! { &ident,
            "none" => CssSpeechBreak::None,
            "x-weak" => CssSpeechBreak::Strength(CssSpeechBreakStrength::XWeak),
            "weak" => CssSpeechBreak::Strength(CssSpeechBreakStrength::Weak),
            "medium" => CssSpeechBreak::Strength(CssSpeechBreakStrength::Medium),
            "strong" => CssSpeechBreak::Strength(CssSpeechBreakStrength::Strong),
            "x-strong" => CssSpeechBreak::Strength(CssSpeechBreakStrength::XStrong),
            _ => return Err(location.unexpected(Token::Ident(ident))),
        };
        return Ok(value);
    }
    super::timing::parse_duration(input, numeric).map(CssSpeechBreak::Time)
}

pub(super) fn parse_speech_break_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssSidePair<CssSpeechBreak>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_speech_break(input, numeric))
}

pub(super) fn parse_cue<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssCue, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(crate::values::speech::CssCue::None);
    }
    let url = super::url::parse_url(input, numeric)?;
    input.discard_whitespace();
    let state = input.mark();
    let is_decibel = matches!(input.consume_after_whitespace(), Ok(Token::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("dB"));
    input.restore(state);
    let decibel = if is_decibel {
        Some(parse_decibel(input, numeric)?)
    } else {
        None
    };
    Ok(crate::values::speech::CssCue::Audio(
        crate::values::speech::CssAudioCue::from_parser(url, decibel),
    ))
}

pub(super) fn parse_cue_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::common::CssSidePair<CssCue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_cue(input, numeric))
}

pub(super) fn parse_voice_stress<'i>(
    input: &mut Stream<'i>,
) -> Result<crate::values::speech::CssVoiceStress, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    surgeist_css_parser::match_ignore_ascii_case! { &ident,
        "normal" => Ok(crate::values::speech::CssVoiceStress::Normal),
        "strong" => Ok(crate::values::speech::CssVoiceStress::Strong),
        "moderate" => Ok(crate::values::speech::CssVoiceStress::Moderate),
        "none" => Ok(crate::values::speech::CssVoiceStress::None),
        "reduced" => Ok(crate::values::speech::CssVoiceStress::Reduced),
        _ => Err(location.unexpected(Token::Ident(ident))),
    }
}

pub(super) fn parse_voice_family<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoiceFamily, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("preserve"))
        .is_ok()
    {
        return Ok(crate::values::speech::CssVoiceFamily::Preserve);
    }
    let mut entries = Vec::new();
    loop {
        entries.push(parse_voice_entry(input, numeric)?);
        if input.only_whitespace_remains() {
            break;
        }
        input.consume_comma().map_err(basic)?;
    }
    Ok(crate::values::speech::CssVoiceFamily::Voices(
        crate::values::speech::CssVoiceFamilyList::from_parser(entries)
            .expect("parsed nonempty list"),
    ))
}
fn voice_age(value: &str) -> Option<crate::values::speech::CssVoiceAge> {
    surgeist_css_parser::match_ignore_ascii_case! { value, "child" => Some(crate::values::speech::CssVoiceAge::Child), "young" => Some(crate::values::speech::CssVoiceAge::Young), "old" => Some(crate::values::speech::CssVoiceAge::Old), _ => None }
}
fn voice_gender(value: &str) -> Option<crate::values::speech::CssVoiceGender> {
    surgeist_css_parser::match_ignore_ascii_case! { value, "male" => Some(crate::values::speech::CssVoiceGender::Male), "female" => Some(crate::values::speech::CssVoiceGender::Female), "neutral" => Some(crate::values::speech::CssVoiceGender::Neutral), _ => None }
}
fn parse_voice_entry<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoiceFamilyEntry, SyntaxError<'i, Error>> {
    use crate::values::speech::{CssGenericVoice, CssVoiceFamilyEntry, CssVoiceFamilyName};
    input.discard_whitespace();
    let location = input.site();
    if let Ok(value) = input.attempt(Stream::consume_string) {
        return CssVoiceFamilyName::try_quoted(value.to_string())
            .map(CssVoiceFamilyEntry::Name)
            .map_err(|_| crate::diagnostic::error::property::unexpected_at(location));
    }
    let start = input.mark();
    let first = input.consume_ident().map_err(basic)?;
    let age = voice_age(first);
    let gender = if age.is_some() {
        input
            .attempt(|input| {
                let ident = input.consume_ident()?;
                voice_gender(ident).ok_or_else(|| input.unexpected::<Error>(Token::Ident(ident)))
            })
            .ok()
    } else {
        voice_gender(first)
    };
    if let Some(gender) = gender {
        let variant = if input.only_whitespace_remains() || super::values::next_is_comma(input) {
            None
        } else {
            Some(super::values::parse_positive_integer_value(input, numeric)?)
        };
        return CssGenericVoice::from_parser(age, gender, variant)
            .map(CssVoiceFamilyEntry::Generic)
            .map_err(|_| crate::diagnostic::error::property::unexpected_at(location));
    }
    input.restore(start);
    let mut identifiers = Vec::new();
    while !input.only_whitespace_remains() && !super::values::next_is_comma(input) {
        input.discard_whitespace();
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        if crate::values::speech::values::reserved_voice_identifier(ident) {
            return Err(location.unexpected(Token::Ident(ident)));
        }
        identifiers.push(
            crate::values::common::CssIdent::try_new(ident.to_string())
                .map_err(|_| crate::diagnostic::error::property::unexpected_at(location))?,
        );
    }
    CssVoiceFamilyName::try_identifiers(identifiers)
        .map(CssVoiceFamilyEntry::Name)
        .ok_or_else(|| input.end_of_stream())
}

pub(super) fn parse_voice_duration<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoiceDuration, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(crate::values::speech::CssVoiceDuration::Auto);
    }
    super::timing::parse_duration(input, numeric).map(crate::values::speech::CssVoiceDuration::Time)
}
fn voice_level(value: &str) -> Option<crate::values::speech::CssVoiceLevel> {
    surgeist_css_parser::match_ignore_ascii_case! { value, "x-low" => Some(crate::values::speech::CssVoiceLevel::XLow), "low" => Some(crate::values::speech::CssVoiceLevel::Low), "medium" => Some(crate::values::speech::CssVoiceLevel::Medium), "high" => Some(crate::values::speech::CssVoiceLevel::High), "x-high" => Some(crate::values::speech::CssVoiceLevel::XHigh), _ => None }
}
pub(super) fn parse_voice_pitch_range<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoicePitchRange, SyntaxError<'i, Error>> {
    let mut level = None;
    let mut absolute = None;
    let mut offset = None;
    while !input.only_whitespace_remains() {
        input.discard_whitespace();
        let location = input.site();
        let root_site = input.site();
        if let Ok(ident) = input.attempt(Stream::consume_ident) {
            if let Some(value) = voice_level(ident)
                && level.is_none()
            {
                level = Some(value);
            } else if ident.eq_ignore_ascii_case("absolute") && absolute.is_none() {
                absolute = Some(location);
            } else {
                return Err(location.unexpected(Token::Ident(ident)));
            }
        } else {
            if offset.is_some() {
                return Err(crate::diagnostic::error::property::unexpected_at(location));
            }
            let component = input
                .collect_component()
                .map_err(crate::numeric::CssNumericConstructionError::component)
                .map_err(|error| speech_numeric_error(numeric, &error, location, root_site))?;
            offset = Some((component, location, root_site));
        }
    }
    if let Some(location) = absolute {
        if level.is_some() {
            return Err(crate::diagnostic::error::property::unexpected_at(location));
        }
        let Some((component, location, root_site)) = offset else {
            return Err(crate::diagnostic::error::property::unexpected_at(location));
        };
        let frequency = match component {
            std::borrow::Cow::Borrowed(component)
                if !matches!(component.view(), crate::component::CssComponentValueRef::Token(_)) =>
            {
                numeric
                    .admit(component, crate::numeric::expression::CalculationRoot::Frequency)
                    .and_then(|expression| {
                        crate::numeric::CssNumeric::<crate::numeric::CssFrequency>::from_parser_calculation(
                            crate::numeric::CssCalculation::<crate::numeric::CssFrequency>::from_expression(
                                expression,
                            ),
                        )
                    })
            }
            component => crate::numeric::CssNumeric::<crate::numeric::CssFrequency>::from_parser_component(
                component.into_owned(),
                numeric,
            ),
        }
        .map_err(|error| speech_numeric_error(numeric, &error, location, root_site))?;
        crate::values::speech::CssVoicePitchRange::from_parser_absolute(frequency)
            .map_err(|error| speech_numeric_error(numeric, &error, location, root_site))
    } else {
        let offset = offset
            .map(|(component, location, root_site)| {
                match component {
                    std::borrow::Cow::Borrowed(component)
                        if !matches!(
                            component.view(),
                            crate::component::CssComponentValueRef::Token(_)
                        ) =>
                    {
                        numeric
                            .admit(
                                component,
                                crate::numeric::expression::CalculationRoot::FrequencyPercentage,
                            )
                            .map(|expression| {
                                crate::values::speech::CssVoiceOffset::Calculation(
                                    crate::numeric::CssCalculation::<
                                        crate::numeric::CssFrequencyPercentage,
                                    >::from_expression(
                                        expression
                                    ),
                                )
                            })
                    }
                    component => crate::values::speech::CssVoiceOffset::from_parser_component(
                        component.into_owned(),
                        numeric,
                    ),
                }
                .map_err(|error| speech_numeric_error(numeric, &error, location, root_site))
            })
            .transpose()?;
        crate::values::speech::CssVoicePitchRange::from_parser_relative(level, offset)
            .map_err(|_| input.end_of_stream())
    }
}
fn voice_rate_keyword(value: &str) -> Option<crate::values::speech::CssVoiceRateKeyword> {
    surgeist_css_parser::match_ignore_ascii_case! { value, "normal" => Some(crate::values::speech::CssVoiceRateKeyword::Normal), "x-slow" => Some(crate::values::speech::CssVoiceRateKeyword::XSlow), "slow" => Some(crate::values::speech::CssVoiceRateKeyword::Slow), "medium" => Some(crate::values::speech::CssVoiceRateKeyword::Medium), "fast" => Some(crate::values::speech::CssVoiceRateKeyword::Fast), "x-fast" => Some(crate::values::speech::CssVoiceRateKeyword::XFast), _ => None }
}
pub(super) fn parse_voice_rate<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoiceRate, SyntaxError<'i, Error>> {
    // `[ normal | x-slow | slow | medium | fast | x-fast ] || <percentage [0,∞]>`:
    // an identifier commits the keyword and anything else the percentage.
    let (mut keyword, mut percentage) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => commit(&mut keyword, input, next_is_any_ident, |input| {
                input.discard_whitespace();
                let location = input.site();
                let ident = input.consume_ident().map_err(basic)?;
                voice_rate_keyword(ident).ok_or_else(|| location.unexpected(Token::Ident(ident)))
            }),
            1 => commit(
                &mut percentage,
                input,
                |input| !next_is_any_ident(input),
                |input| parse_voice_rate_percentage(input, numeric),
            ),
            _ => unreachable!("two voice-rate operands"),
        },
    )?;
    crate::values::speech::CssVoiceRate::from_parser(keyword, percentage)
        .map_err(|_| input.end_of_stream())
}
fn next_is_any_ident(input: &mut Stream<'_>) -> bool {
    let state = input.mark();
    let ident = input.consume_ident().is_ok();
    input.restore(state);
    ident
}
fn parse_voice_rate_percentage<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<
    crate::numeric::CssNumeric<crate::numeric::CssPercentage, crate::numeric::CssNonNegative>,
    SyntaxError<'i, Error>,
> {
    input.discard_whitespace();
    let location = input.site();
    let root_site = input.site();
    let component = input
        .collect_component()
        .map_err(crate::numeric::CssNumericConstructionError::component)
        .map_err(|error| speech_numeric_error(numeric, &error, location, root_site))?;
    if matches!(component.view(), crate::component::CssComponentValueRef::Token(_)) {
        crate::numeric::CssNumeric::<crate::numeric::CssPercentage, crate::numeric::CssNonNegative>::try_from_component(component.into_owned())
    } else {
        let values = crate::numeric::expression::CalculationComponents::single(component)
            .map_err(|error| crate::diagnostic::error::property::invalid_component_value(location, error))?;
        numeric
            .admit(values, crate::numeric::expression::CalculationRoot::Percentage)
            .and_then(|expression| {
                crate::numeric::CssNumeric::<crate::numeric::CssPercentage, crate::numeric::CssNonNegative>::from_parser_calculation(
                    crate::numeric::CssCalculation::<crate::numeric::CssPercentage>::from_expression(expression),
                )
            })
    }
    .map_err(|error| speech_numeric_error(numeric, &error, location, root_site))
}
fn speech_numeric_error<'i>(
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
    error: &crate::numeric::CssNumericConstructionError,
    fallback: super::syntax_types::Site<'i>,
    offset: super::syntax_types::Site<'i>,
) -> SyntaxError<'i, Error> {
    let location = numeric.property_error_location(error, fallback, offset);
    if let Some(component) = error.component_error()
        && crate::diagnostic::error::is_component_resource_error(component)
    {
        return crate::diagnostic::error::property::invalid_component_value(
            location,
            component.clone(),
        );
    }
    crate::diagnostic::error::property::unexpected_at(location)
}

fn parse_decibel<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssDecibelLiteral, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let offset = input.site();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| speech_numeric_error(numeric, &error, location, offset))?;
    crate::values::speech::CssDecibelLiteral::try_from_component(component).map_err(|error| {
        crate::diagnostic::error::property::invalid_component_value(location, error)
    })
}

pub(super) fn parse_voice_balance<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoiceBalance, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let keyword = surgeist_css_parser::match_ignore_ascii_case! { &ident,
            "left" => crate::values::speech::CssVoiceBalanceKeyword::Left,
            "center" => crate::values::speech::CssVoiceBalanceKeyword::Center,
            "right" => crate::values::speech::CssVoiceBalanceKeyword::Right,
            "leftwards" => crate::values::speech::CssVoiceBalanceKeyword::Leftwards,
            "rightwards" => crate::values::speech::CssVoiceBalanceKeyword::Rightwards,
            _ => return Err(location.unexpected(Token::Ident(ident))),
        };
        return Ok(crate::values::speech::CssVoiceBalance::from_keyword(
            keyword,
        ));
    }
    super::values::parse_specified_number(input, numeric)
        .map(crate::values::speech::CssVoiceBalance::from_parser_number)
}

pub(super) fn parse_voice_volume<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<crate::values::speech::CssVoiceVolume, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    if input
        .attempt(|input| input.consume_keyword("silent"))
        .is_ok()
    {
        return Ok(crate::values::speech::CssVoiceVolume::Silent);
    }
    // `[ x-soft | soft | medium | loud | x-loud ] || <decibel>`: an identifier
    // commits the level and a dB dimension the offset.
    let (mut level, mut decibel) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => commit(&mut level, input, next_is_any_ident, |input| {
                input.discard_whitespace();
                let location = input.site();
                let ident = input.consume_ident().map_err(basic)?;
                Ok(surgeist_css_parser::match_ignore_ascii_case! { &ident,
                    "x-soft" => crate::values::speech::CssVoiceVolumeLevel::XSoft,
                    "soft" => crate::values::speech::CssVoiceVolumeLevel::Soft,
                    "medium" => crate::values::speech::CssVoiceVolumeLevel::Medium,
                    "loud" => crate::values::speech::CssVoiceVolumeLevel::Loud,
                    "x-loud" => crate::values::speech::CssVoiceVolumeLevel::XLoud,
                    _ => return Err(location.unexpected(Token::Ident(ident))),
                })
            }),
            1 => commit(
                &mut decibel,
                input,
                |input| {
                    let state = input.mark();
                    let starts = matches!(input.consume_after_whitespace(), Ok(Token::Dimension { unit, .. })
                        if unit.eq_ignore_ascii_case("dB"));
                    input.restore(state);
                    starts
                },
                |input| parse_decibel(input, numeric),
            ),
            _ => unreachable!("two voice-volume operands"),
        },
    )?;
    match (level, decibel) {
        (Some(level), decibel) => {
            Ok(crate::values::speech::CssVoiceVolume::Level { level, decibel })
        }
        (None, Some(value)) => Ok(crate::values::speech::CssVoiceVolume::Offset(value)),
        (None, None) => Err(input.end_of_stream()),
    }
}
