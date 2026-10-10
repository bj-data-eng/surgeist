use super::values::parse_length_percentage;
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{next_is_delim, parse_positive_integer_value};
use crate::error::{Error, basic, unexpected_at};
use crate::syntax::*;
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
            _ => Err(unexpected_at(input.current_source_location())),
        };
    }

    let length = parse_length_percentage(input, numeric)?;
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
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssGridTrackList::none());
    }
    if input
        .try_parse(|input| input.expect_ident_matching("subgrid"))
        .is_ok()
    {
        return parse_grid_subgrid(input, numeric, stop_at_slash);
    }
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
            return Err(unexpected_at(location));
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
        return Err(unexpected_at(input.current_source_location()));
    }

    build_grid_track_list(components)
}

fn parse_grid_subgrid<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    stop_at_slash: bool,
) -> Result<CssGridTrackList, ParseError<'i, Error>> {
    let mut components = Vec::new();
    let mut has_auto_fill = false;
    while !input.is_exhausted() && !(stop_at_slash && next_is_delim(input, '/')) {
        let location = input.current_source_location();
        let component = match input.next().map_err(basic)? {
            Token::SquareBracketBlock => {
                CssGridSubgridComponent::LineNames(input.parse_nested_block(parse_grid_line_names)?)
            }
            Token::Function(name) if name.eq_ignore_ascii_case("repeat") => {
                let repeat = input.parse_nested_block(|input| {
                    let count = if input
                        .try_parse(|input| input.expect_ident_matching("auto-fill"))
                        .is_ok()
                    {
                        None
                    } else {
                        Some(parse_positive_integer_value(input, numeric)?)
                    };
                    input.expect_comma().map_err(basic)?;
                    let mut groups = Vec::new();
                    while !input.is_exhausted() {
                        groups.push(
                            parse_optional_grid_line_names(input)?
                                .ok_or_else(|| unexpected_at(input.current_source_location()))?,
                        );
                    }
                    let value = match count {
                        Some(count) => CssGridNameRepeat::try_new(count, groups),
                        None => CssGridNameRepeat::try_auto_fill(groups),
                    };
                    value.ok_or_else(|| unexpected_at(input.current_source_location()))
                })?;
                if repeat.is_auto_fill() && has_auto_fill {
                    return Err(unexpected_at(location));
                }
                has_auto_fill |= repeat.is_auto_fill();
                CssGridSubgridComponent::Repeat(repeat)
            }
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        };
        components.push(component);
    }
    Ok(CssGridTrackList::try_subgrid(components).expect("checked subgrid name repetitions"))
}

fn parse_optional_grid_line_names<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Option<CssGridLineNames>, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(None);
    }
    let state = input.state();
    if matches!(input.next().map_err(basic)?, Token::SquareBracketBlock) {
        input.parse_nested_block(parse_grid_line_names).map(Some)
    } else {
        input.reset(&state);
        Ok(None)
    }
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
        names.push(
            CssGridLineName::try_new(CssIdent::new(ident.as_ref()))
                .ok_or_else(|| unexpected_at(location))?,
        );
    }
    Ok(CssGridLineNames::new(names))
}

fn parse_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    enum Count {
        Integer(CssPositiveIntegerValue),
        Auto(CssGridAutoRepeatKind),
    }

    let count = if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "auto-fill" => Count::Auto(CssGridAutoRepeatKind::AutoFill),
            "auto-fit" => Count::Auto(CssGridAutoRepeatKind::AutoFit),
            _ => return Err(unexpected_at(input.current_source_location())),
        }
    } else {
        Count::Integer(parse_positive_integer_value(input, numeric)?)
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
    count: CssPositiveIntegerValue,
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
                return Err(unexpected_at(location));
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
        return Err(unexpected_at(input.current_source_location()));
    }
    Ok(ParsedGridTrackComponent::IntegerRepeat {
        track: CssGridIntegerTrackRepeat::try_new(
            count.clone(),
            CssGridTrackRepeatContent::try_new(track_components).expect("checked repeat content"),
        )
        .expect("parsed positive repeat count"),
        fixed: fixed.then(|| {
            CssGridIntegerFixedRepeat::try_new(
                count,
                CssGridFixedRepeatContent::try_new(fixed_components)
                    .expect("checked fixed repeat content"),
            )
            .expect("parsed positive fixed repeat count")
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
                return Err(unexpected_at(location));
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
        return Err(unexpected_at(input.current_source_location()));
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
        Token::Function(name) if name.eq_ignore_ascii_case("repeat") => {
            Err(unexpected_at(location))
        }
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
        Err(unexpected_at(location))
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
            _ => Err(unexpected_at(location)),
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
        Token::Function(_) => Err(unexpected_at(location)),
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
        return Err(unexpected_at(location));
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
                        return Err(unexpected_at(located.location));
                    };
                    CssGridAutoTrackComponent::FixedSize(value)
                }
                ParsedGridTrackComponent::IntegerRepeat { fixed, .. } => {
                    let Some(value) = fixed else {
                        return Err(unexpected_at(located.location));
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
        return Err(unexpected_at(input.current_source_location()));
    }
    Ok(CssGridTrackSizeList::try_new(sizes).expect("nonempty implicit tracks"))
}

pub(super) fn parse_grid_template_areas<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridTemplateAreas, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "none" => Ok(CssGridTemplateAreas::None),
            _ => Err(unexpected_at(input.current_source_location())),
        };
    }

    let mut rows = Vec::new();
    while !input.is_exhausted() {
        let location = input.current_source_location();
        let row = input.expect_string_cloned().map_err(basic)?;
        rows.push(
            crate::grid_template_areas::parse_decoded_row(row.as_ref())
                .map_err(|_| unexpected_at(location))?,
        );
    }
    CssGridTemplateAreas::try_rows(rows).map_err(|_| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_grid_template<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGridTemplate, ParseError<'i, Error>> {
    if input
        .try_parse(|input| {
            input.expect_ident_matching("none")?;
            input.expect_exhausted()
        })
        .is_ok()
    {
        return Ok(CssGridTemplate::none());
    }
    if next_is_grid_area_row(input) {
        return parse_grid_area_template(input, numeric);
    }
    let rows = parse_grid_track_list_with_mode(input, numeric, true)?;
    input.expect_delim('/').map_err(basic)?;
    let columns = parse_grid_track_list_with_mode(input, numeric, false)?;
    Ok(CssGridTemplate::rows_columns(rows, columns))
}

fn next_is_grid_area_row(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let result = input
        .try_parse(|input| {
            parse_optional_grid_line_names(input)?;
            input.expect_string_cloned().map_err(basic)
        })
        .is_ok();
    input.reset(&state);
    result
}

fn parse_grid_area_template<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssGridTemplate, ParseError<'i, Error>> {
    let mut rows = Vec::new();
    while !input.is_exhausted() && !next_is_delim(input, '/') {
        let before = parse_optional_grid_line_names(input)?;
        let location = input.current_source_location();
        let text = input.expect_string_cloned().map_err(basic)?;
        let area = crate::grid_template_areas::parse_decoded_row(text.as_ref())
            .map_err(|_| unexpected_at(location))?;
        let state = input.state();
        let omitted = input.is_exhausted()
            || matches!(
                input.next(),
                Ok(Token::QuotedString(_) | Token::SquareBracketBlock | Token::Delim('/'))
            );
        input.reset(&state);
        let size = if omitted {
            None
        } else {
            Some(parse_grid_track_size(input, numeric)?)
        };
        // Greedy trailing group gives a deterministic view of a single shared
        // boundary; a second group belongs to the following row's leading slot.
        let after = parse_optional_grid_line_names(input)?;
        rows.push(CssGridTemplateAreaTrack::new(area, size, before, after));
    }
    let columns = if !input.is_exhausted() {
        input.expect_delim('/').map_err(basic)?;
        let mut components = Vec::new();
        while !input.is_exhausted() {
            let location = input.current_source_location();
            let component = match parse_optional_grid_line_names(input)? {
                Some(names) => CssGridTrackRepeatComponent::LineNames(names),
                None => {
                    CssGridTrackRepeatComponent::TrackSize(parse_grid_track_size(input, numeric)?)
                }
            };
            if adjacent_line_names(components.last(), &component, |item| {
                matches!(item, CssGridTrackRepeatComponent::LineNames(_))
            }) {
                return Err(unexpected_at(location));
            }
            components.push(component);
        }
        Some(
            CssGridTrackRepeatContent::try_new(components)
                .ok_or_else(|| unexpected_at(input.current_source_location()))?,
        )
    } else {
        None
    };
    CssGridTemplate::try_areas(rows, columns)
        .map_err(|_| unexpected_at(input.current_source_location()))
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
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_grid<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGrid, ParseError<'i, Error>> {
    if input
        .try_parse(|input| {
            input.expect_ident_matching("none")?;
            input.expect_exhausted()
        })
        .is_ok()
    {
        return Ok(CssGrid::template(CssGridTemplate::none()));
    }
    if next_is_grid_area_row(input) {
        return parse_grid_area_template(input, numeric).map(CssGrid::template);
    }
    if next_is_auto_flow_prefix(input) {
        let dense = parse_grid_auto_flow_prefix(input)?;
        let auto_tracks = parse_optional_grid_auto_tracks(input, numeric, true)?;
        input.expect_delim('/').map_err(basic)?;
        let explicit_tracks = parse_grid_track_list_with_mode(input, numeric, false)?;
        return Ok(CssGrid::from_auto_flow(
            CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Row, dense),
            auto_tracks,
            explicit_tracks,
        ));
    }
    let rows = parse_grid_track_list_with_mode(input, numeric, true)?;
    input.expect_delim('/').map_err(basic)?;
    if next_is_auto_flow_prefix(input) {
        let dense = parse_grid_auto_flow_prefix(input)?;
        let auto_tracks = parse_optional_grid_auto_tracks(input, numeric, false)?;
        Ok(CssGrid::from_auto_flow(
            CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Column, dense),
            auto_tracks,
            rows,
        ))
    } else {
        let columns = parse_grid_track_list_with_mode(input, numeric, false)?;
        Ok(CssGrid::template(CssGridTemplate::rows_columns(
            rows, columns,
        )))
    }
}

fn next_is_auto_flow_prefix(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let result = input
        .try_parse(|input| input.expect_ident_cloned())
        .is_ok_and(|ident| {
            ident.eq_ignore_ascii_case("auto-flow") || ident.eq_ignore_ascii_case("dense")
        });
    input.reset(&state);
    result
}

fn parse_grid_auto_flow_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<bool, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("dense"))
        .is_ok()
    {
        input.expect_ident_matching("auto-flow").map_err(basic)?;
        Ok(true)
    } else {
        input.expect_ident_matching("auto-flow").map_err(basic)?;
        Ok(input
            .try_parse(|input| input.expect_ident_matching("dense"))
            .is_ok())
    }
}

fn parse_optional_grid_auto_tracks<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    stop_at_slash: bool,
) -> std::result::Result<Option<CssGridTrackSizeList>, ParseError<'i, Error>> {
    if input.is_exhausted() || (stop_at_slash && next_is_delim(input, '/')) {
        Ok(None)
    } else {
        parse_grid_auto_track_sizes_with_mode(input, numeric, stop_at_slash).map(Some)
    }
}
