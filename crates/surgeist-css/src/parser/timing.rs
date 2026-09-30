use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{
    CalculationRoot, next_is_comma, parse_custom_ident_from_str_at, parse_numeric_function,
};
use crate::error::{CssFeatureId, Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.easing-function"),
    CssFeatureId::new("official.value.cubic-bezier-easing"),
    CssFeatureId::new("official.value.step-easing"),
    CssFeatureId::new("official.value.step-position"),
];

pub(super) fn parse_duration_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDurationList, ParseError<'i, Error>> {
    let mut values = Vec::new();
    loop {
        values.push(parse_duration(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "duration list has an empty item",
            ));
        }
    }
    CssDurationList::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "duration list is empty"))
}

pub(super) fn parse_delay_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDelayList, ParseError<'i, Error>> {
    let mut values = Vec::new();
    loop {
        values.push(parse_delay(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "delay list has an empty item",
            ));
        }
    }
    CssDelayList::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "delay list is empty"))
}

pub(super) fn parse_duration<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDuration, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("s") => {
            CssDurationLiteral::try_new(*value, CssTimeUnit::Seconds)
                .map(CssDuration::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(
                        location,
                        None,
                        "CSS duration must be finite and non-negative",
                    )
                })
        }
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("ms") => {
            CssDurationLiteral::try_new(*value, CssTimeUnit::Milliseconds)
                .map(CssDuration::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(
                        location,
                        None,
                        "CSS duration must be finite and non-negative",
                    )
                })
        }
        Token::Dimension { unit, .. } => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported duration unit `{unit}`"),
        )),
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Time)
                .map(CssTimeCalculation::from_expression)
                .map(CssDuration::Calculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_delay<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssDelay, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("s") => {
            CssDelayLiteral::try_new(*value, CssTimeUnit::Seconds)
                .map(CssDelay::Literal)
                .ok_or_else(|| unsupported_value_at(location, None, "CSS delay must be finite"))
        }
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("ms") => {
            CssDelayLiteral::try_new(*value, CssTimeUnit::Milliseconds)
                .map(CssDelay::Literal)
                .ok_or_else(|| unsupported_value_at(location, None, "CSS delay must be finite"))
        }
        Token::Dimension { unit, .. } => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported delay unit `{unit}`"),
        )),
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Time)
                .map(CssTimeCalculation::from_expression)
                .map(CssDelay::Calculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_easing_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssEasingList, ParseError<'i, Error>> {
    let mut values = Vec::new();
    loop {
        values.push(parse_easing(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "easing list has an empty item",
            ));
        }
    }
    CssEasingList::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "easing list is empty"))
}

pub(super) fn parse_easing<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssEasing, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let keyword = match_ignore_ascii_case! { &ident,
            "ease" => CssEasingKeyword::Ease,
            "linear" => CssEasingKeyword::Linear,
            "ease-in" => CssEasingKeyword::EaseIn,
            "ease-out" => CssEasingKeyword::EaseOut,
            "ease-in-out" => CssEasingKeyword::EaseInOut,
            "step-start" => CssEasingKeyword::StepStart,
            "step-end" => CssEasingKeyword::StepEnd,
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("easing", ident.as_ref()),
            ))?,
        };
        return Ok(CssEasing::Keyword(keyword));
    }
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    let kind = match name.to_ascii_lowercase().as_str() {
        "cubic-bezier" => CssEasingFunctionKind::CubicBezier,
        "steps" => CssEasingFunctionKind::Steps,
        _ => {
            return Err(unsupported_value(
                input,
                None,
                format!("unsupported easing function `{name}`"),
            ));
        }
    };
    input.parse_nested_block(|input| match kind {
        CssEasingFunctionKind::CubicBezier => {
            parse_cubic_bezier(input, numeric).map(CssEasing::CubicBezier)
        }
        CssEasingFunctionKind::Steps => parse_steps(input, numeric).map(CssEasing::Steps),
    })
}

#[derive(Clone, Copy)]
enum CssEasingFunctionKind {
    CubicBezier,
    Steps,
}

fn parse_cubic_bezier<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssCubicBezier, ParseError<'i, Error>> {
    let x1_location = input.current_source_location();
    let x1 = super::values::parse_specified_number(input, numeric, "easing")?;
    input.expect_comma().map_err(basic)?;
    let y1 = super::values::parse_specified_number(input, numeric, "easing")?;
    input.expect_comma().map_err(basic)?;
    let x2_location = input.current_source_location();
    let x2 = super::values::parse_specified_number(input, numeric, "easing")?;
    input.expect_comma().map_err(basic)?;
    let y2 = super::values::parse_specified_number(input, numeric, "easing")?;
    input.expect_exhausted().map_err(basic)?;

    if CssCubicBezierX::try_new(x1.clone()).is_none() {
        return Err(unsupported_value_at(
            x1_location,
            None,
            "cubic-bezier x coordinate must be between zero and one",
        ));
    }
    if CssCubicBezierX::try_new(x2.clone()).is_none() {
        return Err(unsupported_value_at(
            x2_location,
            None,
            "cubic-bezier x coordinate must be between zero and one",
        ));
    }
    CssCubicBezier::try_new(x1, y1, x2, y2).ok_or_else(|| {
        unsupported_value_at(
            x1_location,
            None,
            "cubic-bezier x coordinates must be between zero and one",
        )
    })
}

fn parse_steps<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSteps, ParseError<'i, Error>> {
    let count_location = input.current_source_location();
    let count = match super::values::parse_integer_value(input, numeric)? {
        CssIntegerValue::Literal(literal) => {
            let literal = CssPositiveIntegerLiteral::try_new(literal).ok_or_else(|| {
                unsupported_value_at(
                    count_location,
                    None,
                    "steps() count must be a positive integer",
                )
            })?;
            CssPositiveIntegerValue::Literal(literal)
        }
        CssIntegerValue::Calculation(value) => CssPositiveIntegerValue::Calculation(value),
    };

    let position = if input.is_exhausted() {
        None
    } else {
        input.expect_comma().map_err(basic)?;
        let ident = input.expect_ident_cloned().map_err(basic)?;
        let position = match_ignore_ascii_case! { &ident,
            "jump-start" => CssStepPosition::JumpStart,
            "jump-end" => CssStepPosition::JumpEnd,
            "jump-none" => CssStepPosition::JumpNone,
            "jump-both" => CssStepPosition::JumpBoth,
            "start" => CssStepPosition::Start,
            "end" => CssStepPosition::End,
            _ => return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("step position", ident.as_ref()),
            )),
        };
        Some(position)
    };
    input.expect_exhausted().map_err(basic)?;
    CssSteps::try_new(count, position).ok_or_else(|| {
        unsupported_value_at(
            count_location,
            None,
            "steps() with jump-none requires at least two intervals",
        )
    })
}

pub(super) fn parse_transition_property_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransitionPropertyList, ParseError<'i, Error>> {
    let mut properties = Vec::new();
    loop {
        properties.push(parse_transition_property(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "transition-property list has an empty item",
            ));
        }
    }
    CssTransitionPropertyList::try_new(properties)
        .ok_or_else(|| unsupported_value(input, None, "transition-property list is empty"))
}

pub(super) fn parse_transition_property<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransitionProperty, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "all" => Ok(CssTransitionProperty::All),
        "none" => Ok(CssTransitionProperty::None),
        _ => parse_custom_ident_from_str_at("transition property", ident.as_ref(), location)
            .map(CssTransitionProperty::Custom),
    }
}

pub(super) fn parse_transition_value_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransitionList, ParseError<'i, Error>> {
    let mut items = Vec::new();
    loop {
        items.push(parse_single_transition_value(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "transition list has an empty item",
            ));
        }
    }
    CssTransitionList::try_new(items)
        .ok_or_else(|| unsupported_value(input, None, "transition list is empty"))
}

pub(super) fn parse_single_transition_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransition, ParseError<'i, Error>> {
    let mut property = None;
    let mut duration = None;
    let mut delay = None;
    let mut timing_function = None;
    while !input.is_exhausted() && !next_is_comma(input) {
        if duration.is_none()
            && let Ok(value) = input.try_parse(|input| parse_duration(input, numeric))
        {
            duration = Some(value);
            continue;
        }
        if duration.is_some()
            && delay.is_none()
            && let Ok(value) = input.try_parse(|input| parse_delay(input, numeric))
        {
            delay = Some(value);
            continue;
        }
        if timing_function.is_none()
            && let Ok(easing) = input.try_parse(|input| parse_easing(input, numeric))
        {
            timing_function = Some(easing);
            continue;
        }
        if property.is_none()
            && let Ok(parsed_property) = input.try_parse(parse_transition_property)
        {
            property = Some(parsed_property);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported transition component",
        ));
    }
    CssTransition::try_new(property, duration, delay, timing_function)
        .ok_or_else(|| unsupported_value(input, None, "transition item is empty"))
}

pub(super) fn parse_animation_name_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationNameList, ParseError<'i, Error>> {
    let mut names = Vec::new();
    loop {
        names.push(parse_animation_name(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "animation-name list has an empty item",
            ));
        }
    }
    CssAnimationNameList::try_new(names)
        .ok_or_else(|| unsupported_value(input, None, "animation-name list is empty"))
}

pub(super) fn parse_animation_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(value) = input.try_parse(Parser::expect_string_cloned) {
        return CssKeyframesString::try_new(value.to_string())
            .map(CssAnimationName::String)
            .ok_or_else(|| unsupported_value_at(location, None, "animation string name is empty"));
    }

    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        Ok(CssAnimationName::None)
    } else {
        parse_custom_ident_from_str_at("animation name", ident.as_ref(), location)
            .map(CssAnimationName::Custom)
    }
}

pub(super) fn parse_animation_iteration_value_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationIterationCountList, ParseError<'i, Error>> {
    let mut counts = Vec::new();
    loop {
        counts.push(parse_animation_iteration_value(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "animation-iteration-count list has an empty item",
            ));
        }
    }
    CssAnimationIterationCountList::try_new(counts)
        .ok_or_else(|| unsupported_value(input, None, "animation-iteration-count list is empty"))
}

pub(super) fn parse_animation_iteration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationIterationCount, ParseError<'i, Error>> {
    let numeric_start = input.state();
    if input
        .try_parse(|input| input.expect_ident_matching("infinite"))
        .is_ok()
    {
        return Ok(CssAnimationIterationCount::Infinite);
    }
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => CssAnimationIterationNumber::try_new(*value)
            .map(CssAnimationIterationCount::Number)
            .ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    "animation iteration count must be finite and non-negative",
                )
            }),
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Number)
                .map(CssNumberCalculation::from_expression)
                .map(CssAnimationIterationCount::Calculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_animation_direction_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationDirectionList, ParseError<'i, Error>> {
    let mut directions = Vec::new();
    loop {
        directions.push(parse_animation_direction(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "animation-direction list has an empty item",
            ));
        }
    }
    CssAnimationDirectionList::try_new(directions)
        .ok_or_else(|| unsupported_value(input, None, "animation-direction list is empty"))
}

pub(super) fn parse_animation_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationDirection, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssAnimationDirection::Normal),
        "reverse" => Ok(CssAnimationDirection::Reverse),
        "alternate" => Ok(CssAnimationDirection::Alternate),
        "alternate-reverse" => Ok(CssAnimationDirection::AlternateReverse),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("animation-direction", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_animation_fill_mode_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationFillModeList, ParseError<'i, Error>> {
    let mut modes = Vec::new();
    loop {
        modes.push(parse_animation_fill_mode(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "animation-fill-mode list has an empty item",
            ));
        }
    }
    CssAnimationFillModeList::try_new(modes)
        .ok_or_else(|| unsupported_value(input, None, "animation-fill-mode list is empty"))
}

pub(super) fn parse_animation_fill_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationFillMode, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssAnimationFillMode::None),
        "forwards" => Ok(CssAnimationFillMode::Forwards),
        "backwards" => Ok(CssAnimationFillMode::Backwards),
        "both" => Ok(CssAnimationFillMode::Both),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("animation-fill-mode", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_animation_play_state_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationPlayStateList, ParseError<'i, Error>> {
    let mut states = Vec::new();
    loop {
        states.push(parse_animation_play_state(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "animation-play-state list has an empty item",
            ));
        }
    }
    CssAnimationPlayStateList::try_new(states)
        .ok_or_else(|| unsupported_value(input, None, "animation-play-state list is empty"))
}

pub(super) fn parse_animation_play_state<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAnimationPlayState, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "running" => Ok(CssAnimationPlayState::Running),
        "paused" => Ok(CssAnimationPlayState::Paused),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("animation-play-state", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_animation_value_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssAnimationList, ParseError<'i, Error>> {
    let mut items = Vec::new();
    loop {
        items.push(parse_single_animation_value(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "animation list has an empty item",
            ));
        }
    }
    CssAnimationList::try_new(items)
        .ok_or_else(|| unsupported_value(input, None, "animation list is empty"))
}

pub(super) fn parse_single_animation_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssAnimation, ParseError<'i, Error>> {
    let mut name = None;
    let mut duration = None;
    let mut delay = None;
    let mut timing_function = None;
    let mut iteration_count = None;
    let mut direction = None;
    let mut fill_mode = None;
    let mut play_state = None;

    while !input.is_exhausted() && !next_is_comma(input) {
        if duration.is_none()
            && let Ok(value) = input.try_parse(|input| parse_duration(input, numeric))
        {
            duration = Some(value);
            continue;
        }
        if duration.is_some()
            && delay.is_none()
            && let Ok(value) = input.try_parse(|input| parse_delay(input, numeric))
        {
            delay = Some(value);
            continue;
        }
        if timing_function.is_none()
            && let Ok(easing) = input.try_parse(|input| parse_easing(input, numeric))
        {
            timing_function = Some(easing);
            continue;
        }
        if iteration_count.is_none()
            && let Ok(count) =
                input.try_parse(|input| parse_animation_iteration_value(input, numeric))
        {
            iteration_count = Some(count);
            continue;
        }
        if direction.is_none()
            && let Ok(parsed_direction) = input.try_parse(parse_animation_direction)
        {
            direction = Some(parsed_direction);
            continue;
        }
        if fill_mode.is_none()
            && let Ok(parsed_fill_mode) = input.try_parse(parse_animation_fill_mode)
        {
            fill_mode = Some(parsed_fill_mode);
            continue;
        }
        if play_state.is_none()
            && let Ok(parsed_play_state) = input.try_parse(parse_animation_play_state)
        {
            play_state = Some(parsed_play_state);
            continue;
        }
        if name.is_none()
            && let Ok(parsed_name) = input.try_parse(parse_animation_name)
        {
            name = Some(parsed_name);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported animation component",
        ));
    }

    CssAnimation::try_new(CssAnimationComponents {
        name,
        duration,
        delay,
        timing_function,
        iteration_count,
        direction,
        fill_mode,
        play_state,
    })
    .ok_or_else(|| unsupported_value(input, None, "animation item is empty"))
}
