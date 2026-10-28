use super::combinators::{Combinator, Operand, Tail, commit, fill, parse_group};
use super::syntax_types::Stream;
use crate::diagnostic::CssFeatureId;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::property_input::StreamExt;
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

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

pub(crate) fn parse_duration_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssDurationList, SyntaxError<'i, Error>> {
    let values = input.comma_segments(|input| parse_duration(input, numeric))?;
    Ok(CssDurationList::from_parser(values))
}

pub(crate) fn parse_delay_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssDelayList, SyntaxError<'i, Error>> {
    let values = input.comma_segments(|input| parse_delay(input, numeric))?;
    Ok(CssDelayList::from_parser(values))
}

pub(super) fn parse_duration<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssNumeric<CssTime, CssNonNegative>, SyntaxError<'i, Error>> {
    let location = input.site();
    let offset = input.site();
    let time = parse_delay(input, numeric)?;
    time.try_narrow()
        .map_err(|error| time_error(numeric, &error, location, offset))
}

pub(super) fn parse_delay<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::numeric::CssNumeric<crate::numeric::CssTime>, SyntaxError<'i, Error>>
{
    input.discard_whitespace();
    let location = input.site();
    let offset = input.site();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| time_error(numeric, &error, location, offset))?;
    crate::numeric::CssNumeric::<crate::numeric::CssTime>::from_parser_component(component, numeric)
        .map_err(|error| time_error(numeric, &error, location, offset))
}
fn time_error<'i>(
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
    unexpected_at(location)
}

pub(crate) fn parse_easing_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssEasingList, SyntaxError<'i, Error>> {
    let values = input.comma_segments(|input| parse_easing(input, numeric))?;
    CssEasingList::try_new(values).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_easing<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssEasing, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
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
    let location = input.site();
    let name = match input.consume_after_whitespace().map_err(basic)? {
        Token::Function(name) => name,
        token => return Err(location.unexpected::<Error>(token)),
    };
    let Some(kind) = easing_function_kind(name) else {
        // Easing 2 §2: an unknown function is itself responsible.
        return Err(location.unexpected::<Error>(Token::Function(name)));
    };
    input.within_block(|input| match kind {
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
fn easing_operand<'i>(
    slot: &mut Option<CssEasing>,
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Operand<'i> {
    let starts = |input: &mut Stream<'i>| {
        let state = input.mark();
        let starts = matches!(
            input.consume_after_whitespace(),
            Ok(Token::Function(name)) if easing_function_kind(name).is_some()
        );
        input.restore(state);
        starts
    };
    if commit(slot, input, starts, |input| parse_easing(input, numeric))? {
        return Ok(true);
    }
    fill(slot, input, |input| parse_easing(input, numeric))
}

fn parse_linear<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssLinearEasing, SyntaxError<'i, Error>> {
    let stops = input.comma_segments(|input| parse_linear_stop(input, numeric))?;
    CssLinearEasing::from_parser(stops).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_linear_stop<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssLinearStop, SyntaxError<'i, Error>> {
    use super::values::{parse_specified_number, parse_specified_percentage};

    let mut inputs = Vec::new();
    let output = if let Ok(output) = input.attempt(|input| parse_specified_number(input, numeric)) {
        for _ in 0..2 {
            if input.only_whitespace_remains() {
                break;
            }
            inputs.push(parse_specified_percentage(input, numeric)?);
        }
        output
    } else {
        inputs.push(parse_specified_percentage(input, numeric)?);
        if let Ok(second) = input.attempt(|input| parse_specified_percentage(input, numeric)) {
            inputs.push(second);
        }
        parse_specified_number(input, numeric)?
    };
    input.require_empty().map_err(basic)?;
    CssLinearStop::from_parser(output, inputs).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_cubic_bezier<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssCubicBezier, SyntaxError<'i, Error>> {
    let x1_location = input.site();
    let x1 = super::values::parse_specified_number(input, numeric)?;
    input.consume_comma().map_err(basic)?;
    let y1 = super::values::parse_specified_number(input, numeric)?;
    input.consume_comma().map_err(basic)?;
    let x2_location = input.site();
    let x2 = super::values::parse_specified_number(input, numeric)?;
    input.consume_comma().map_err(basic)?;
    let y2 = super::values::parse_specified_number(input, numeric)?;
    input.require_empty().map_err(basic)?;

    if CssCubicBezierX::try_new(x1.clone()).is_none() {
        return Err(unexpected_at(x1_location));
    }
    if CssCubicBezierX::try_new(x2.clone()).is_none() {
        return Err(unexpected_at(x2_location));
    }
    CssCubicBezier::try_new(x1, y1, x2, y2).ok_or_else(|| unexpected_at(x1_location))
}

fn parse_steps<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssSteps, SyntaxError<'i, Error>> {
    let count_location = input.site();
    let count = super::values::parse_integer_value(input, numeric)?
        .try_narrow::<CssPositive>()
        .map_err(|_| unexpected_at(count_location))?;

    let mut position_location = None;
    let position = if input.only_whitespace_remains() {
        None
    } else {
        input.consume_comma().map_err(basic)?;
        input.discard_whitespace();
        let location = input.site();
        position_location = Some(location);
        let ident = input.consume_ident().map_err(basic)?;
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
    input.require_empty().map_err(basic)?;
    // Easing 2 §2.3: a count valid by itself makes the position, which needs
    // more steps (jump-none), responsible.
    let location = match position_location {
        Some(location) if CssSteps::try_new(count.clone(), None).is_some() => location,
        _ => count_location,
    };
    CssSteps::try_new(count, position).ok_or_else(|| unexpected_at(location))
}

pub(crate) fn parse_transition_property_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTransitionPropertyList, SyntaxError<'i, Error>> {
    // List post-condition: `none` is admitted only alone (Transitions 1
    // §2.1). A leading `none` is a valid prefix, so the comma after it is
    // responsible; a later `none` is itself responsible.
    let mut none_location = None;
    let mut index = 0_usize;
    let properties = input.comma_segments(|input| {
        let location = input.site();
        let property = parse_transition_property(input)?;
        if matches!(property, CssTransitionProperty::None) {
            let responsible = if index == 0 { input.site() } else { location };
            none_location.get_or_insert(responsible);
        }
        index += 1;
        Ok(property)
    })?;
    CssTransitionPropertyList::try_new(properties)
        .ok_or_else(|| unexpected_at(none_location.unwrap_or_else(|| input.site())))
}

pub(super) fn parse_transition_property<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTransitionProperty, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "all" => Ok(CssTransitionProperty::All),
        "none" => Ok(CssTransitionProperty::None),
        _ => parse_custom_ident_from_str_at(ident, location)
            .and_then(|name| {
                CssTransitionPropertyName::try_new(name).ok_or_else(|| {
                    unexpected_at(location)
                })
            })
            .map(CssTransitionProperty::Custom),
    }
}

pub(crate) fn parse_transition_value_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssTransitionList, SyntaxError<'i, Error>> {
    // List post-condition: a `none` transition property is admitted only
    // alone (Transitions 1 §2.5). A leading item with `none` is a valid
    // prefix, so the comma after it is responsible.
    let mut none_location = None;
    let mut index = 0_usize;
    let items = input.comma_segments(|input| {
        let (item, location) = parse_single_transition_value(input, numeric)?;
        if none_location.is_none() && location.is_some() {
            none_location = if index == 0 {
                Some(input.site())
            } else {
                location
            };
        }
        index += 1;
        Ok(item)
    })?;
    CssTransitionList::from_parser(items)
        .ok_or_else(|| unexpected_at(none_location.unwrap_or_else(|| input.site())))
}

pub(super) fn parse_single_transition_value<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<
    (CssTransition, Option<super::syntax_types::Site<'i>>),
    SyntaxError<'i, Error>,
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
                let location = input.site();
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
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_animation_name_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationNameList, SyntaxError<'i, Error>> {
    let names = input.comma_segments(|input| parse_animation_name(input))?;
    CssAnimationNameList::try_new(names).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_animation_name<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationName, SyntaxError<'i, Error>> {
    let location = input.site();
    if let Ok(value) = input.attempt(Stream::consume_string) {
        return Ok(CssAnimationName::String(CssKeyframesString::new(
            value.to_string(),
        )));
    }

    let ident = input.consume_ident().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        Ok(CssAnimationName::None)
    } else {
        parse_custom_ident_from_str_at(ident, location)
            .and_then(|name| {
                CssKeyframesIdent::try_new(name).ok_or_else(|| unexpected_at(location))
            })
            .map(CssAnimationName::Custom)
    }
}

pub(crate) fn parse_animation_iteration_value_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationIterationCountList, SyntaxError<'i, Error>> {
    let counts = input.comma_segments(|input| parse_animation_iteration_value(input, numeric))?;
    CssAnimationIterationCountList::try_new(counts).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_animation_iteration_value<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationIterationCount, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("infinite"))
        .is_ok()
    {
        return Ok(CssAnimationIterationCount::Infinite);
    }
    parse_nonnegative_number(input, numeric).map(CssAnimationIterationCount::Number)
}

pub(crate) fn parse_animation_direction_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationDirectionList, SyntaxError<'i, Error>> {
    let directions = input.comma_segments(|input| parse_animation_direction(input))?;
    CssAnimationDirectionList::try_new(directions).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_animation_direction<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationDirection, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssAnimationDirection::Normal),
        "reverse" => Ok(CssAnimationDirection::Reverse),
        "alternate" => Ok(CssAnimationDirection::Alternate),
        "alternate-reverse" => Ok(CssAnimationDirection::AlternateReverse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_animation_fill_mode_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationFillModeList, SyntaxError<'i, Error>> {
    let modes = input.comma_segments(|input| parse_animation_fill_mode(input))?;
    CssAnimationFillModeList::try_new(modes).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_animation_fill_mode<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationFillMode, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssAnimationFillMode::None),
        "forwards" => Ok(CssAnimationFillMode::Forwards),
        "backwards" => Ok(CssAnimationFillMode::Backwards),
        "both" => Ok(CssAnimationFillMode::Both),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_animation_play_state_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationPlayStateList, SyntaxError<'i, Error>> {
    let states = input.comma_segments(|input| parse_animation_play_state(input))?;
    CssAnimationPlayStateList::try_new(states).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_animation_play_state<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssAnimationPlayState, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "running" => Ok(CssAnimationPlayState::Running),
        "paused" => Ok(CssAnimationPlayState::Paused),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_animation_value_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationList, SyntaxError<'i, Error>> {
    let items = input.comma_segments(|input| parse_single_animation_value(input, numeric))?;
    Ok(CssAnimationList::from_parser(items))
}

pub(super) fn parse_single_animation_value<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssAnimation, SyntaxError<'i, Error>> {
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
    .ok_or_else(|| unexpected_at(input.site()))
}
