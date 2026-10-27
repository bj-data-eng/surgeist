use super::combinators::{Combinator, Operand, Tail, commit, fill, parse_group};
use super::property_input::Parser;
use crate::diagnostic::CssFeatureId;
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use super::values::{parse_custom_ident_from_str_at, parse_nonnegative_number};
use crate::numeric::{CssNonNegative, CssNumeric, CssPositive, CssTime};
use crate::rule::keyframes::{CssKeyframesIdent, CssKeyframesString};
use crate::values::animation::{
    CssAnimation, CssAnimationComponents, CssAnimationDirection, CssAnimationDirectionList,
    CssAnimationFillMode, CssAnimationFillModeList, CssAnimationIterationCount,
    CssAnimationIterationCountList, CssAnimationList, CssAnimationName, CssAnimationNameList,
    CssAnimationPlayState, CssAnimationPlayStateList, CssCubicBezier, CssCubicBezierX,
    CssDelayList, CssDurationList, CssEasing, CssEasingKeyword, CssEasingList, CssLinearEasing,
    CssLinearStop, CssStepPosition, CssSteps, CssTransition, CssTransitionList,
    CssTransitionProperty, CssTransitionPropertyList, CssTransitionPropertyName,
};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.easing-function"),
    CssFeatureId::new("official.value.cubic-bezier-easing"),
    CssFeatureId::new("official.value.linear-easing"),
    CssFeatureId::new("official.value.step-easing"),
    CssFeatureId::new("official.value.step-position"),
];

pub(crate) fn parse_duration_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssDurationList, ParseError<'i, Error>> {
    let values = input.parse_comma_separated(|input| parse_duration(input, numeric))?;
    Ok(CssDurationList::from_parser(values))
}

pub(crate) fn parse_delay_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssDelayList, ParseError<'i, Error>> {
    let values = input.parse_comma_separated(|input| parse_delay(input, numeric))?;
    Ok(CssDelayList::from_parser(values))
}

pub(super) fn parse_duration<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssNumeric<CssTime, CssNonNegative>, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let offset = input.position();
    let time = parse_delay(input, numeric)?;
    time.try_narrow()
        .map_err(|error| time_error(numeric, &error, location, offset))
}

pub(super) fn parse_delay<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::numeric::CssNumeric<crate::numeric::CssTime>, ParseError<'i, Error>>
{
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| time_error(numeric, &error, location, offset))?;
    crate::numeric::CssNumeric::<crate::numeric::CssTime>::from_parser_component(component, numeric)
        .map_err(|error| time_error(numeric, &error, location, offset))
}
fn time_error<'i>(
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
    error: &crate::numeric::CssNumericConstructionError,
    fallback: super::property_input::Site<'i>,
    offset: super::property_input::Site<'i>,
) -> ParseError<'i, Error> {
    let location = numeric.property_error_location(error, fallback, offset);
    if let Some(component) = error.component_error()
        && crate::diagnostic::error::is_component_resource_error(component)
    {
        return crate::diagnostic::error::property::invalid_component_value(
            location,
            component.clone(),
        );
    }
    unexpected_at(location)
}

pub(crate) fn parse_easing_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssEasingList, ParseError<'i, Error>> {
    let values = input.parse_comma_separated(|input| parse_easing(input, numeric))?;
    CssEasingList::try_new(values).ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_easing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssEasing, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let keyword = match_ignore_ascii_case! { &ident,
            "ease" => CssEasingKeyword::Ease,
            "linear" => CssEasingKeyword::Linear,
            "ease-in" => CssEasingKeyword::EaseIn,
            "ease-out" => CssEasingKeyword::EaseOut,
            "ease-in-out" => CssEasingKeyword::EaseInOut,
            "step-start" => CssEasingKeyword::StepStart,
            "step-end" => CssEasingKeyword::StepEnd,
            _ => Err(unexpected_at(keyword_location))?,
        };
        return Ok(CssEasing::Keyword(keyword));
    }
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    let Some(kind) = easing_function_kind(&name) else {
        // Easing 2 §2: an unknown function is itself responsible.
        return Err(location.new_unexpected_token_error::<Error>(Token::Function(name)));
    };
    input.parse_nested_block(|input| match kind {
        CssEasingFunctionKind::CubicBezier => {
            parse_cubic_bezier(input, numeric).map(CssEasing::CubicBezier)
        }
        CssEasingFunctionKind::Steps => parse_steps(input, numeric).map(CssEasing::Steps),
        CssEasingFunctionKind::Linear => parse_linear(input, numeric).map(CssEasing::Linear),
    })
}

#[derive(Clone, Copy)]
enum CssEasingFunctionKind {
    CubicBezier,
    Linear,
    Steps,
}

fn easing_function_kind(name: &str) -> Option<CssEasingFunctionKind> {
    Some(match name.to_ascii_lowercase().as_str() {
        "cubic-bezier" => CssEasingFunctionKind::CubicBezier,
        "steps" => CssEasingFunctionKind::Steps,
        "linear" => CssEasingFunctionKind::Linear,
        _ => return None,
    })
}

/// The `<easing-function>` operand of a `||` group. An easing function token
/// commits the operand (Values 4 §2.2), so a failure inside it is the
/// value's failure rather than a component no operand admits; keywords stay
/// recoverable.
fn easing_operand<'i, 't>(
    slot: &mut Option<CssEasing>,
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Operand<'i> {
    let starts = |input: &mut Parser<'i, 't>| {
        let state = input.state();
        let starts = matches!(
            input.next(),
            Ok(Token::Function(name)) if easing_function_kind(name).is_some()
        );
        input.reset(&state);
        starts
    };
    if commit(slot, input, starts, |input| parse_easing(input, numeric))? {
        return Ok(true);
    }
    fill(slot, input, |input| parse_easing(input, numeric))
}

fn parse_linear<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssLinearEasing, ParseError<'i, Error>> {
    let stops = input.parse_comma_separated(|input| parse_linear_stop(input, numeric))?;
    CssLinearEasing::from_parser(stops)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_linear_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssLinearStop, ParseError<'i, Error>> {
    use super::values::{parse_specified_number, parse_specified_percentage};

    let mut inputs = Vec::new();
    let output = if let Ok(output) = input.try_parse(|input| parse_specified_number(input, numeric))
    {
        for _ in 0..2 {
            if input.is_exhausted() {
                break;
            }
            inputs.push(parse_specified_percentage(input, numeric)?);
        }
        output
    } else {
        inputs.push(parse_specified_percentage(input, numeric)?);
        if let Ok(second) = input.try_parse(|input| parse_specified_percentage(input, numeric)) {
            inputs.push(second);
        }
        parse_specified_number(input, numeric)?
    };
    input.expect_exhausted().map_err(basic)?;
    CssLinearStop::from_parser(output, inputs)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_cubic_bezier<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssCubicBezier, ParseError<'i, Error>> {
    let x1_location = input.current_source_location();
    let x1 = super::values::parse_specified_number(input, numeric)?;
    input.expect_comma().map_err(basic)?;
    let y1 = super::values::parse_specified_number(input, numeric)?;
    input.expect_comma().map_err(basic)?;
    let x2_location = input.current_source_location();
    let x2 = super::values::parse_specified_number(input, numeric)?;
    input.expect_comma().map_err(basic)?;
    let y2 = super::values::parse_specified_number(input, numeric)?;
    input.expect_exhausted().map_err(basic)?;

    if CssCubicBezierX::try_new(x1.clone()).is_none() {
        return Err(unexpected_at(x1_location));
    }
    if CssCubicBezierX::try_new(x2.clone()).is_none() {
        return Err(unexpected_at(x2_location));
    }
    CssCubicBezier::try_new(x1, y1, x2, y2).ok_or_else(|| unexpected_at(x1_location))
}

fn parse_steps<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssSteps, ParseError<'i, Error>> {
    let count_location = input.current_source_location();
    let count = super::values::parse_integer_value(input, numeric)?
        .try_narrow::<CssPositive>()
        .map_err(|_| unexpected_at(count_location))?;

    let mut position_location = None;
    let position = if input.is_exhausted() {
        None
    } else {
        input.expect_comma().map_err(basic)?;
        input.skip_whitespace();
        let location = input.current_source_location();
        position_location = Some(location);
        let ident = input.expect_ident_cloned().map_err(basic)?;
        let position = match_ignore_ascii_case! { &ident,
            "jump-start" => CssStepPosition::JumpStart,
            "jump-end" => CssStepPosition::JumpEnd,
            "jump-none" => CssStepPosition::JumpNone,
            "jump-both" => CssStepPosition::JumpBoth,
            "start" => CssStepPosition::Start,
            "end" => CssStepPosition::End,
            // The unknown keyword itself cannot continue steps().
            _ => return Err(unexpected_at(location)),
        };
        Some(position)
    };
    input.expect_exhausted().map_err(basic)?;
    // Easing 2 §2.3: a count valid by itself makes the position, which needs
    // more steps (jump-none), responsible.
    let location = match position_location {
        Some(location) if CssSteps::try_new(count.clone(), None).is_some() => location,
        _ => count_location,
    };
    CssSteps::try_new(count, position).ok_or_else(|| unexpected_at(location))
}

pub(crate) fn parse_transition_property_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransitionPropertyList, ParseError<'i, Error>> {
    // List post-condition: `none` is admitted only alone (Transitions 1
    // §2.1). A leading `none` is a valid prefix, so the comma after it is
    // responsible; a later `none` is itself responsible.
    let mut none_location = None;
    let mut index = 0_usize;
    let properties = input.parse_comma_separated(|input| {
        let location = input.current_source_location();
        let property = parse_transition_property(input)?;
        if matches!(property, CssTransitionProperty::None) {
            let responsible = if index == 0 {
                input.current_source_location()
            } else {
                location
            };
            none_location.get_or_insert(responsible);
        }
        index += 1;
        Ok(property)
    })?;
    CssTransitionPropertyList::try_new(properties).ok_or_else(|| {
        unexpected_at(none_location.unwrap_or_else(|| input.current_source_location()))
    })
}

pub(super) fn parse_transition_property<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransitionProperty, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "all" => Ok(CssTransitionProperty::All),
        "none" => Ok(CssTransitionProperty::None),
        _ => parse_custom_ident_from_str_at(ident.as_ref(), location)
            .and_then(|name| {
                CssTransitionPropertyName::try_new(name).ok_or_else(|| {
                    unexpected_at(location)
                })
            })
            .map(CssTransitionProperty::Custom),
    }
}

pub(crate) fn parse_transition_value_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTransitionList, ParseError<'i, Error>> {
    // List post-condition: a `none` transition property is admitted only
    // alone (Transitions 1 §2.5). A leading item with `none` is a valid
    // prefix, so the comma after it is responsible.
    let mut none_location = None;
    let mut index = 0_usize;
    let items = input.parse_comma_separated(|input| {
        let (item, location) = parse_single_transition_value(input, numeric)?;
        if none_location.is_none() && location.is_some() {
            none_location = if index == 0 {
                Some(input.current_source_location())
            } else {
                location
            };
        }
        index += 1;
        Ok(item)
    })?;
    CssTransitionList::from_parser(items).ok_or_else(|| {
        unexpected_at(none_location.unwrap_or_else(|| input.current_source_location()))
    })
}

pub(super) fn parse_single_transition_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<
    (CssTransition, Option<super::property_input::Site<'i>>),
    ParseError<'i, Error>,
> {
    // `[ none | <single-transition-property> ] || <time> || <easing-function>
    // || <time>`: the first time is the duration, a second the delay.
    let (mut property, mut none_location) = (None, None);
    let (mut duration, mut delay, mut timing_function) = (None, None, None);
    parse_group::<4>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut duration, input, |input| parse_duration(input, numeric)),
            1 if duration.is_none() => Ok(false),
            1 => fill(&mut delay, input, |input| parse_delay(input, numeric)),
            2 => easing_operand(&mut timing_function, input, numeric),
            3 => {
                let location = input.current_source_location();
                let matched = fill(&mut property, input, parse_transition_property)?;
                if matches!(property, Some(CssTransitionProperty::None)) {
                    none_location = Some(location);
                }
                Ok(matched)
            }
            _ => unreachable!("four transition operands"),
        },
    )?;
    CssTransition::from_parser(property, duration, delay, timing_function)
        .map(|value| (value, none_location))
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(crate) fn parse_animation_name_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationNameList, ParseError<'i, Error>> {
    let names = input.parse_comma_separated(|input| parse_animation_name(input))?;
    CssAnimationNameList::try_new(names)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_animation_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return Ok(CssAnimationName::String(CssKeyframesString::new(
            value.to_string(),
        )));
    }

    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        Ok(CssAnimationName::None)
    } else {
        parse_custom_ident_from_str_at(ident.as_ref(), location)
            .and_then(|name| {
                CssKeyframesIdent::try_new(name).ok_or_else(|| unexpected_at(location))
            })
            .map(CssAnimationName::Custom)
    }
}

pub(crate) fn parse_animation_iteration_value_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationIterationCountList, ParseError<'i, Error>> {
    let counts =
        input.parse_comma_separated(|input| parse_animation_iteration_value(input, numeric))?;
    CssAnimationIterationCountList::try_new(counts)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_animation_iteration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationIterationCount, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("infinite"))
        .is_ok()
    {
        return Ok(CssAnimationIterationCount::Infinite);
    }
    parse_nonnegative_number(input, numeric).map(CssAnimationIterationCount::Number)
}

pub(crate) fn parse_animation_direction_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationDirectionList, ParseError<'i, Error>> {
    let directions = input.parse_comma_separated(|input| parse_animation_direction(input))?;
    CssAnimationDirectionList::try_new(directions)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_animation_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationDirection, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssAnimationDirection::Normal),
        "reverse" => Ok(CssAnimationDirection::Reverse),
        "alternate" => Ok(CssAnimationDirection::Alternate),
        "alternate-reverse" => Ok(CssAnimationDirection::AlternateReverse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_animation_fill_mode_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationFillModeList, ParseError<'i, Error>> {
    let modes = input.parse_comma_separated(|input| parse_animation_fill_mode(input))?;
    CssAnimationFillModeList::try_new(modes)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_animation_fill_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationFillMode, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssAnimationFillMode::None),
        "forwards" => Ok(CssAnimationFillMode::Forwards),
        "backwards" => Ok(CssAnimationFillMode::Backwards),
        "both" => Ok(CssAnimationFillMode::Both),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_animation_play_state_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationPlayStateList, ParseError<'i, Error>> {
    let states = input.parse_comma_separated(|input| parse_animation_play_state(input))?;
    CssAnimationPlayStateList::try_new(states)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_animation_play_state<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationPlayState, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "running" => Ok(CssAnimationPlayState::Running),
        "paused" => Ok(CssAnimationPlayState::Paused),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_animation_value_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationList, ParseError<'i, Error>> {
    let items =
        input.parse_comma_separated(|input| parse_single_animation_value(input, numeric))?;
    Ok(CssAnimationList::from_parser(items))
}

pub(super) fn parse_single_animation_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimation, ParseError<'i, Error>> {
    // `<time> || <easing-function> || <time> || <iteration-count> ||
    // <direction> || <fill-mode> || <play-state> || <keyframes-name>`: the
    // first time is the duration, a second the delay; keywords precede names.
    let (mut name, mut duration, mut delay, mut timing_function) = (None, None, None, None);
    let (mut iteration_count, mut direction, mut fill_mode, mut play_state) =
        (None, None, None, None);
    parse_group::<8>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut duration, input, |input| parse_duration(input, numeric)),
            1 if duration.is_none() => Ok(false),
            1 => fill(&mut delay, input, |input| parse_delay(input, numeric)),
            2 => easing_operand(&mut timing_function, input, numeric),
            3 => fill(&mut iteration_count, input, |input| {
                parse_animation_iteration_value(input, numeric)
            }),
            4 => fill(&mut direction, input, parse_animation_direction),
            5 => fill(&mut fill_mode, input, parse_animation_fill_mode),
            6 => fill(&mut play_state, input, parse_animation_play_state),
            7 => fill(&mut name, input, parse_animation_name),
            _ => unreachable!("eight animation operands"),
        },
    )?;

    CssAnimation::from_parser(CssAnimationComponents {
        name,
        duration,
        delay,
        timing_function,
        iteration_count,
        direction,
        fill_mode,
        play_state,
    })
    .ok_or_else(|| unexpected_at(input.current_source_location()))
}
