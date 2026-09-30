use super::values::parse_length_percentage;
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{next_is_delim, parse_custom_ident_from_str_at, parse_integer_literal};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;
use crate::{
    CssComponentValueRef, CssFeatureId, CssFlexCalculation, CssGridTemplateAreas,
    CssLengthPercentageCalculation, CssSpecifiedNonNegativeFlex,
    CssSpecifiedNonNegativeLengthPercentage, CssValueTokenRef,
};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("ext.value.grid-repeat")];

pub(super) fn parse_flow_tolerance<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFlowTolerance, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "normal" => Ok(CssFlowTolerance::normal()),
            "infinite" => Ok(CssFlowTolerance::infinite()),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("flow-tolerance", ident.as_ref()),
            )),
        };
    }

    let length = parse_length_percentage(input, numeric, "flow-tolerance")?;
    Ok(CssFlowTolerance::length_percentage(length))
}

pub(super) fn parse_grid_track_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTrackList, ParseError<'i, Error>> {
    parse_grid_track_list_with_mode(input, numeric, false)
}

fn parse_grid_track_list_with_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    stop_at_slash: bool,
) -> std::result::Result<CssGridTrackList, ParseError<'i, Error>> {
    let mut components = Vec::new();
    while !input.is_exhausted() {
        if stop_at_slash && next_is_delim(input, '/') {
            break;
        }
        let location = input.current_source_location();
        let component = parse_grid_track_component(input, numeric)?;
        if adjacent_line_names(
            components
                .last()
                .map(|item: &LocatedGridTrackComponent| &item.component),
            &component,
            |item| matches!(item, ParsedGridTrackComponent::LineNames(_)),
        ) {
            return Err(unsupported_value_at(
                location,
                None,
                "adjacent grid line-name blocks",
            ));
        }
        components.push(LocatedGridTrackComponent {
            location,
            component,
        });
    }
    if components.is_empty()
        || !components
            .iter()
            .any(|component| !matches!(component.component, ParsedGridTrackComponent::LineNames(_)))
    {
        return Err(unsupported_value(
            input,
            None,
            "grid track list is missing a track",
        ));
    }

    build_grid_track_list(components)
}

#[derive(Clone)]
struct LocatedGridTrackComponent {
    location: cssparser::SourceLocation,
    component: ParsedGridTrackComponent,
}

#[derive(Clone)]
enum ParsedGridTrackComponent {
    LineNames(CssGridLineNames),
    TrackSize(CssGridTrackSize),
    IntegerRepeat {
        track: CssGridIntegerTrackRepeat,
        fixed: Option<CssGridIntegerFixedRepeat>,
    },
    AutoRepeat {
        value: CssGridAutoRepeat,
    },
}

fn parse_grid_track_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    let state = input.state();
    match input.next().map_err(basic)? {
        Token::SquareBracketBlock => {
            return input
                .parse_nested_block(parse_grid_line_names)
                .map(ParsedGridTrackComponent::LineNames);
        }
        Token::Function(name) if name.eq_ignore_ascii_case("repeat") => {
            return input.parse_nested_block(|input| parse_grid_repeat(input, numeric));
        }
        _ => input.reset(&state),
    }

    parse_grid_track_size(input, numeric).map(ParsedGridTrackComponent::TrackSize)
}

pub(super) fn parse_grid_line_names<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridLineNames, ParseError<'i, Error>> {
    let mut names = Vec::new();
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        names.push(parse_custom_ident_from_str_at(
            "grid line name",
            ident.as_ref(),
            location,
        )?);
    }
    Ok(CssGridLineNames::new(names))
}

fn parse_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    enum Count {
        Integer(CssPositiveIntegerLiteral),
        Auto(CssGridAutoRepeatKind),
    }

    let count = if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "auto-fill" => Count::Auto(CssGridAutoRepeatKind::AutoFill),
            "auto-fit" => Count::Auto(CssGridAutoRepeatKind::AutoFit),
            _ => return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("grid repeat count", ident.as_ref()),
            )),
        }
    } else {
        let location = input.current_source_location();
        let integer = parse_integer_literal(input, numeric)?;
        let count = CssPositiveIntegerLiteral::try_new(integer).ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                "grid repeat count must be a positive integer",
            )
        })?;
        Count::Integer(count)
    };

    input.expect_comma().map_err(basic)?;
    match count {
        Count::Integer(count) => parse_integer_grid_repeat(input, numeric, count),
        Count::Auto(kind) => parse_auto_grid_repeat(input, numeric, kind),
    }
}

fn parse_integer_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    count: CssPositiveIntegerLiteral,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    let mut track_components = Vec::new();
    let mut fixed_components = Vec::new();
    let mut fixed = true;
    let mut has_track = false;
    while !input.is_exhausted() {
        let state = input.state();
        let location = input.current_source_location();
        if matches!(input.next().map_err(basic)?, Token::SquareBracketBlock) {
            let names = input.parse_nested_block(parse_grid_line_names)?;
            let component = CssGridTrackRepeatComponent::LineNames(names.clone());
            if adjacent_line_names(track_components.last(), &component, |item| {
                matches!(item, CssGridTrackRepeatComponent::LineNames(_))
            }) {
                return Err(unsupported_value_at(
                    location,
                    None,
                    "adjacent grid line-name blocks",
                ));
            }
            track_components.push(component);
            fixed_components.push(CssGridFixedRepeatComponent::LineNames(names.clone()));
            continue;
        }
        input.reset(&state);
        let size = parse_grid_track_size(input, numeric)?;
        has_track = true;
        if let Some(fixed_size) = grid_fixed_size(&size) {
            fixed_components.push(CssGridFixedRepeatComponent::FixedSize(fixed_size));
        } else {
            fixed = false;
        }
        track_components.push(CssGridTrackRepeatComponent::TrackSize(size));
    }
    if !has_track {
        return Err(unsupported_value(
            input,
            None,
            "grid repeat content is missing a track",
        ));
    }
    Ok(ParsedGridTrackComponent::IntegerRepeat {
        track: CssGridIntegerTrackRepeat::new(
            count.clone(),
            CssGridTrackRepeatContent::try_new(track_components).expect("checked repeat content"),
        ),
        fixed: fixed.then(|| {
            CssGridIntegerFixedRepeat::new(
                count,
                CssGridFixedRepeatContent::try_new(fixed_components)
                    .expect("checked fixed repeat content"),
            )
        }),
    })
}

fn parse_auto_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    kind: CssGridAutoRepeatKind,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    let mut components = Vec::new();
    let mut has_track = false;
    while !input.is_exhausted() {
        let state = input.state();
        let location = input.current_source_location();
        if matches!(input.next().map_err(basic)?, Token::SquareBracketBlock) {
            let names = input.parse_nested_block(parse_grid_line_names)?;
            let component = CssGridTrackRepeatComponent::LineNames(names.clone());
            if adjacent_line_names(components.last(), &component, |item| {
                matches!(item, CssGridTrackRepeatComponent::LineNames(_))
            }) {
                return Err(unsupported_value_at(
                    location,
                    None,
                    "adjacent grid line-name blocks",
                ));
            }
            components.push(component);
            continue;
        }
        input.reset(&state);
        let size = parse_grid_track_size(input, numeric)?;
        has_track = true;
        components.push(CssGridTrackRepeatComponent::TrackSize(size));
    }
    if !has_track {
        return Err(unsupported_value(
            input,
            None,
            "grid repeat content is missing a track",
        ));
    }
    Ok(ParsedGridTrackComponent::AutoRepeat {
        value: CssGridAutoRepeat::new(
            kind,
            CssGridTrackRepeatContent::try_new(components).expect("checked auto repeat content"),
        ),
    })
}

fn parse_grid_track_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTrackSize, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let state = input.state();
    match input.next().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("minmax") => {
            input.parse_nested_block(|input| {
                let min = parse_grid_inflexible_track_breadth(input, numeric)?;
                input.expect_comma().map_err(basic)?;
                let max = parse_grid_track_breadth(input, numeric)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(CssGridTrackSize::try_minmax(min, max).expect("checked inflexible minimum"))
            })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("fit-content") => input
            .parse_nested_block(|input| {
                let limit = parse_grid_length_percentage(input, numeric)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(CssGridTrackSize::from_fit_content(limit))
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("repeat") => Err(unsupported_value_at(
            location,
            None,
            "repeat() is a grid track list component, not a track size",
        )),
        _ => {
            input.reset(&state);
            parse_grid_track_breadth(input, numeric).map(CssGridTrackSize::from_breadth)
        }
    }
}

fn parse_grid_inflexible_track_breadth<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTrackBreadth, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let breadth = parse_grid_track_breadth(input, numeric)?;
    if breadth.is_inflexible() {
        Ok(breadth)
    } else {
        Err(unsupported_value_at(
            location,
            None,
            "minmax() minimum must be an inflexible track breadth",
        ))
    }
}

fn parse_grid_track_breadth<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTrackBreadth, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    input.skip_whitespace();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Dimension { .. } | Token::Percentage { .. } | Token::Number { .. } => {
            input.reset(&numeric_start);
            let component = grid_numeric_component(input, numeric)?;
            if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("fr"))
            {
                let flex = CssSpecifiedNonNegativeFlex::try_from_component(component)
                    .map_err(|error| grid_numeric_error(numeric, &error, location, root_offset))?;
                Ok(CssGridTrackBreadth::from_flex(flex))
            } else {
                let length = CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
                    .map_err(|error| grid_numeric_error(numeric, &error, location, root_offset))?;
                Ok(CssGridTrackBreadth::from_length_percentage(length))
            }
        }
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "min-content" => Ok(CssGridTrackBreadth::min_content()),
            "max-content" => Ok(CssGridTrackBreadth::max_content()),
            "auto" => Ok(CssGridTrackBreadth::auto()),
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("grid track", ident.as_ref()),
            )),
        },
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            input.reset(&numeric_start);
            let component = grid_numeric_component(input, numeric)?;
            let values = crate::CssComponentValues::try_new(vec![component])
                .map_err(|_| super::values::calculation_error(location))?;
            match numeric
                .admit_grid_track_math(values)
                .map_err(|error| grid_numeric_error(numeric, &error, location, root_offset))?
            {
                crate::numeric::GridTrackMath::Flex(expression) => {
                    let value = CssSpecifiedNonNegativeFlex::try_from_calculation(
                        CssFlexCalculation::from_expression(expression),
                    )
                    .map_err(|error| grid_numeric_error(numeric, &error, location, root_offset))?;
                    Ok(CssGridTrackBreadth::from_flex(value))
                }
                crate::numeric::GridTrackMath::LengthPercentage(expression) => {
                    let value = CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                        CssLengthPercentageCalculation::from_expression(expression),
                    )
                    .map_err(|error| grid_numeric_error(numeric, &error, location, root_offset))?;
                    Ok(CssGridTrackBreadth::from_length_percentage(value))
                }
            }
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported grid track function `{name}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn grid_numeric_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<crate::CssComponentValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    numeric
        .collect(input)
        .map_err(|error| grid_numeric_error(numeric, &error, location, offset))
}

fn grid_numeric_error<'i>(
    numeric: &crate::numeric::NumericInputContext<'_>,
    error: &crate::CssNumericConstructionError,
    location: cssparser::SourceLocation,
    offset: usize,
) -> ParseError<'i, Error> {
    super::values::calculation_error(numeric.error_location(error, location, offset))
}

fn parse_grid_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssSpecifiedNonNegativeLengthPercentage, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = grid_numeric_component(input, numeric)?;
    if matches!(component.view(), CssComponentValueRef::Function(_)) {
        let values = crate::CssComponentValues::try_new(vec![component])
            .map_err(|_| super::values::calculation_error(location))?;
        let expression = numeric
            .admit(values, crate::numeric::CalculationRoot::LengthPercentage)
            .map_err(|error| grid_numeric_error(numeric, &error, location, offset))?;
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::from_expression(expression),
        )
        .map_err(|error| grid_numeric_error(numeric, &error, location, offset))
    } else {
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
            .map_err(|error| grid_numeric_error(numeric, &error, location, offset))
    }
}

fn build_grid_track_list<'i>(
    components: Vec<LocatedGridTrackComponent>,
) -> std::result::Result<CssGridTrackList, ParseError<'i, Error>> {
    let auto_repeat_count = components
        .iter()
        .filter(|component| {
            matches!(
                component.component,
                ParsedGridTrackComponent::AutoRepeat { .. }
            )
        })
        .count();

    if auto_repeat_count > 1 {
        let location = components
            .iter()
            .filter(|component| {
                matches!(
                    component.component,
                    ParsedGridTrackComponent::AutoRepeat { .. }
                )
            })
            .nth(1)
            .expect("second auto repeat")
            .location;
        return Err(unsupported_value_at(
            location,
            None,
            "grid auto track list contains more than one automatic repetition",
        ));
    }

    let current = if auto_repeat_count == 0 {
        let values = components
            .into_iter()
            .map(|located| match located.component {
                ParsedGridTrackComponent::LineNames(value) => {
                    CssGridGeneralTrackComponent::LineNames(value)
                }
                ParsedGridTrackComponent::TrackSize(value) => {
                    CssGridGeneralTrackComponent::TrackSize(value)
                }
                ParsedGridTrackComponent::IntegerRepeat { track, .. } => {
                    CssGridGeneralTrackComponent::Repeat(track)
                }
                ParsedGridTrackComponent::AutoRepeat { .. } => {
                    unreachable!("general list has no auto repetition")
                }
            })
            .collect();
        CssGridTrackList::general(
            CssGridGeneralTrackList::try_new(values).expect("checked general list"),
        )
    } else {
        let mut values = Vec::with_capacity(components.len());
        for located in components {
            let value = match located.component {
                ParsedGridTrackComponent::LineNames(value) => {
                    CssGridAutoTrackComponent::LineNames(value)
                }
                ParsedGridTrackComponent::TrackSize(value) => {
                    let Some(value) = grid_fixed_size(&value) else {
                        return Err(unsupported_value_at(
                            located.location,
                            None,
                            "tracks surrounding automatic repetition must be fixed-size",
                        ));
                    };
                    CssGridAutoTrackComponent::FixedSize(value)
                }
                ParsedGridTrackComponent::IntegerRepeat { fixed, .. } => {
                    let Some(value) = fixed else {
                        return Err(unsupported_value_at(
                            located.location,
                            None,
                            "repetition surrounding automatic repetition must be fixed-size",
                        ));
                    };
                    CssGridAutoTrackComponent::Repeat(value)
                }
                ParsedGridTrackComponent::AutoRepeat { value, .. } => {
                    CssGridAutoTrackComponent::AutoRepeat(value)
                }
            };
            values.push(value);
        }
        CssGridTrackList::auto(CssGridAutoTrackList::try_new(values).expect("checked auto list"))
    };

    Ok(current)
}

fn grid_fixed_size(size: &CssGridTrackSize) -> Option<CssGridFixedSize> {
    CssGridFixedSize::try_new(size.clone())
}

pub(super) fn parse_grid_auto_track_sizes<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTrackSizeList, ParseError<'i, Error>> {
    parse_grid_auto_track_sizes_with_mode(input, numeric, false)
}

fn parse_grid_auto_track_sizes_with_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    stop_at_slash: bool,
) -> std::result::Result<CssGridTrackSizeList, ParseError<'i, Error>> {
    let mut sizes = Vec::new();
    while !input.is_exhausted() && !(stop_at_slash && next_is_delim(input, '/')) {
        sizes.push(parse_grid_track_size(input, numeric)?);
    }
    if sizes.is_empty() {
        return Err(unsupported_value(
            input,
            None,
            "grid automatic track list is missing a track size",
        ));
    }
    Ok(CssGridTrackSizeList::try_new(sizes).expect("nonempty implicit tracks"))
}

pub(super) fn parse_grid_template_areas<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridTemplateAreas, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "none" => Ok(CssGridTemplateAreas::None),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("grid-template-areas", ident.as_ref()),
            )),
        };
    }

    let mut rows = Vec::new();
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let row = input.expect_string_cloned().map_err(basic)?;
        rows.push(
            crate::grid_template_areas::parse_decoded_row(row.as_ref())
                .map_err(|error| unsupported_value_at(location, None, area_error_message(error)))?,
        );
    }
    CssGridTemplateAreas::try_rows(rows)
        .map_err(|error| unsupported_value(input, None, area_error_message(error)))
}

fn area_error_message(error: crate::CssGridTemplateAreaError) -> String {
    match error {
        crate::CssGridTemplateAreaError::InvalidName => {
            "invalid grid template area name".to_owned()
        }
        crate::CssGridTemplateAreaError::TrashCharacter(character) => {
            format!("invalid grid template area character {character}")
        }
        crate::CssGridTemplateAreaError::MissingRows => {
            "grid-template-areas is missing rows".to_owned()
        }
        crate::CssGridTemplateAreaError::EmptyRow => "grid template area row is empty".to_owned(),
        crate::CssGridTemplateAreaError::InconsistentWidths => {
            "grid-template-areas rows have inconsistent widths".to_owned()
        }
        crate::CssGridTemplateAreaError::NonRectangular(name) => {
            format!("grid template area {name} is not rectangular")
        }
    }
}

pub(super) fn parse_grid_template<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTemplate, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "none" => Ok(CssGridTemplate::none()),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("grid-template", ident.as_ref()),
            )),
        };
    }

    let rows = parse_grid_track_list_with_mode(input, numeric, true)?;
    let columns = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_track_list_with_mode(input, numeric, false)?)
    } else {
        None
    };
    Ok(CssGridTemplate::rows_columns(rows, columns))
}

pub(super) fn parse_grid_auto_flow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridAutoFlow, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssGridAutoFlow::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("dense"))
        .is_ok()
    {
        return Ok(match input.try_parse(parse_grid_auto_flow_axis) {
            Ok(axis) => CssGridAutoFlow::explicit_axis(axis, true),
            Err(_) => CssGridAutoFlow::Dense,
        });
    }
    let axis = parse_grid_auto_flow_axis(input)?;
    let dense = input
        .try_parse(|input| input.expect_ident_matching("dense"))
        .is_ok();
    Ok(CssGridAutoFlow::explicit_axis(axis, dense))
}

pub(super) fn parse_grid_auto_flow_axis<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridAutoFlowAxis, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "row" => Ok(CssGridAutoFlowAxis::Row),
        "column" => Ok(CssGridAutoFlowAxis::Column),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("grid-auto-flow", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_grid<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGrid, ParseError<'i, Error>> {
    let state = input.state();
    let is_auto_flow = input
        .try_parse(|input| input.expect_ident_matching("auto-flow"))
        .is_ok();
    input.reset(&state);
    if is_auto_flow {
        parse_grid_auto_flow_shorthand(input, numeric)
    } else {
        let template = parse_grid_template(input, numeric)?;
        Ok(CssGrid::template(template))
    }
}

pub(super) fn parse_grid_auto_flow_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGrid, ParseError<'i, Error>> {
    input.expect_ident_matching("auto-flow").map_err(basic)?;
    let dense = input
        .try_parse(|input| input.expect_ident_matching("dense"))
        .is_ok();
    let auto_tracks = if !input.is_exhausted() && !next_is_delim(input, '/') {
        Some(parse_grid_auto_track_sizes_with_mode(input, numeric, true)?)
    } else {
        None
    };
    input.expect_delim('/').map_err(basic)?;
    let explicit_tracks = parse_grid_track_list_with_mode(input, numeric, false)?;
    let flow = CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Row, dense);
    Ok(CssGrid::from_auto_flow(flow, auto_tracks, explicit_tracks))
}
