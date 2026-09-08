use std::collections::HashMap;

use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{
    LengthGrammar, checked_percentage_value, next_is_delim, parse_box_size_value,
    parse_calc_length_with_grammar, parse_custom_ident_from_str_at, parse_length_with_context,
    parse_positive_integer,
};
use crate::CssFeatureId;
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::properties::CssGridFlowTolerancePropertyValueRepresentation;
use crate::syntax::*;
use crate::validation::{LengthUnitStatus, classify_length_unit, unsupported_keyword_reason};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] =
    &[CssFeatureId::new("ext.value.grid-repeat")];

pub(super) fn parse_grid_flow_tolerance<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridFlowTolerancePropertyValueRepresentation, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "normal" => Ok(CssGridFlowTolerancePropertyValueRepresentation::new(
                CssGridFlowToleranceValue::Normal,
                Some(CssGridFlowTolerance::Normal),
            )),
            "infinite" => Ok(CssGridFlowTolerancePropertyValueRepresentation::new(
                CssGridFlowToleranceValue::Infinite,
                Some(CssGridFlowTolerance::Infinite),
            )),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("grid-flow-tolerance", ident.as_ref()),
            )),
        };
    }

    let length = parse_box_size_value(input)?;
    let current = CssGridFlowToleranceValue::from_length(length.clone());
    let i01_subset = match &length {
        CssLength::Percent(value) => Some(CssGridFlowTolerance::Percent(value.value())),
        CssLength::Calc(CssCalcLength::Typed(_)) => None,
        length => Some(CssGridFlowTolerance::Length(length.clone())),
    };
    Ok(CssGridFlowTolerancePropertyValueRepresentation::new(
        current, i01_subset,
    ))
}

pub(super) fn parse_grid_track_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedGridTrackList, ParseError<'i, Error>> {
    parse_grid_track_list_with_mode(input, false)
}

fn parse_grid_track_list_with_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
    stop_at_slash: bool,
) -> std::result::Result<CssParsedGridTrackList, ParseError<'i, Error>> {
    let mut components = Vec::new();
    while !input.is_exhausted() {
        if stop_at_slash && next_is_delim(input, '/') {
            break;
        }
        let location = input.current_source_location();
        components.push(LocatedGridTrackComponent {
            location,
            component: parse_grid_track_component(input)?,
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
    TrackSize(CssAuthoredGridTrackSize),
    IntegerRepeat {
        track: CssAuthoredGridIntegerTrackRepeat,
        fixed: Option<CssAuthoredGridIntegerFixedRepeat>,
        i01: Option<CssGridRepeat>,
    },
    AutoRepeat {
        value: CssAuthoredGridAutoRepeat,
        i01: Option<CssGridRepeat>,
    },
}

fn parse_grid_track_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    let state = input.state();
    match input.next().map_err(basic)? {
        Token::SquareBracketBlock => {
            return input
                .parse_nested_block(parse_grid_line_names)
                .map(ParsedGridTrackComponent::LineNames);
        }
        Token::Function(name) if name.eq_ignore_ascii_case("repeat") => {
            return input.parse_nested_block(parse_grid_repeat);
        }
        _ => input.reset(&state),
    }

    parse_grid_track_size(input).map(ParsedGridTrackComponent::TrackSize)
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
    if names.is_empty() {
        Err(unsupported_value(input, None, "grid line names are empty"))
    } else {
        Ok(CssGridLineNames::new(names))
    }
}

fn parse_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    enum Count {
        Integer(i32),
        Auto(CssAuthoredGridAutoRepeatKind),
    }

    let count = if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "auto-fill" => Count::Auto(CssAuthoredGridAutoRepeatKind::AutoFill),
            "auto-fit" => Count::Auto(CssAuthoredGridAutoRepeatKind::AutoFit),
            _ => return Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("grid repeat count", ident.as_ref()),
            )),
        }
    } else {
        let count = parse_positive_integer(input, "grid repeat count")?;
        Count::Integer(count)
    };

    input.expect_comma().map_err(basic)?;
    match count {
        Count::Integer(count) => parse_integer_grid_repeat(input, count),
        Count::Auto(kind) => parse_auto_grid_repeat(input, kind),
    }
}

fn parse_integer_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
    count: i32,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    let mut track_components = Vec::new();
    let mut fixed_components = Vec::new();
    let mut fixed = true;
    let mut legacy_components = Some(Vec::new());
    let mut has_track = false;
    while !input.is_exhausted() {
        let state = input.state();
        if matches!(input.next().map_err(basic)?, Token::SquareBracketBlock) {
            let names = input.parse_nested_block(parse_grid_line_names)?;
            track_components.push(CssAuthoredGridTrackRepeatComponent::LineNames(
                names.clone(),
            ));
            fixed_components.push(CssAuthoredGridFixedRepeatComponent::LineNames(
                names.clone(),
            ));
            legacy_components
                .as_mut()
                .expect("line names preserve projection")
                .push(CssGridTrackComponent::LineNames(names));
            continue;
        }
        input.reset(&state);
        let size = parse_grid_track_size(input)?;
        has_track = true;
        match (legacy_components.as_mut(), size.i01_projection()) {
            (Some(components), Some(value)) => {
                components.push(CssGridTrackComponent::TrackSize(value));
            }
            (Some(_), None) => legacy_components = None,
            (None, _) => {}
        }
        if let Some(fixed_size) = grid_fixed_size(&size) {
            fixed_components.push(CssAuthoredGridFixedRepeatComponent::FixedSize(fixed_size));
        } else {
            fixed = false;
        }
        track_components.push(CssAuthoredGridTrackRepeatComponent::TrackSize(size));
    }
    if !has_track {
        return Err(unsupported_value(
            input,
            None,
            "grid repeat content is missing a track",
        ));
    }
    let count_value = CssGridRepeatInteger::try_new(count).expect("positive repeat count");
    let legacy = legacy_components.map(|components| {
        CssGridRepeat::new(
            CssGridRepeatCount::integer(count),
            CssGridTrackList::new(components),
        )
    });
    Ok(ParsedGridTrackComponent::IntegerRepeat {
        track: CssAuthoredGridIntegerTrackRepeat::new(
            count_value,
            CssAuthoredGridTrackRepeatContent::new(track_components),
        ),
        fixed: fixed.then(|| {
            CssAuthoredGridIntegerFixedRepeat::new(
                count_value,
                CssAuthoredGridFixedRepeatContent::new(fixed_components),
            )
        }),
        i01: legacy,
    })
}

fn parse_auto_grid_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
    kind: CssAuthoredGridAutoRepeatKind,
) -> std::result::Result<ParsedGridTrackComponent, ParseError<'i, Error>> {
    let mut components = Vec::new();
    let mut legacy_components = Some(Vec::new());
    let mut has_track = false;
    while !input.is_exhausted() {
        let state = input.state();
        if matches!(input.next().map_err(basic)?, Token::SquareBracketBlock) {
            let names = input.parse_nested_block(parse_grid_line_names)?;
            components.push(CssAuthoredGridFixedRepeatComponent::LineNames(
                names.clone(),
            ));
            legacy_components
                .as_mut()
                .expect("line names preserve projection")
                .push(CssGridTrackComponent::LineNames(names));
            continue;
        }
        input.reset(&state);
        let location = input.current_source_location();
        let size = parse_grid_track_size(input)?;
        let Some(fixed) = grid_fixed_size(&size) else {
            return Err(unsupported_value_at(
                location,
                None,
                "automatic grid repetition requires a fixed track size",
            ));
        };
        has_track = true;
        match (legacy_components.as_mut(), size.i01_projection()) {
            (Some(components), Some(value)) => {
                components.push(CssGridTrackComponent::TrackSize(value));
            }
            (Some(_), None) => legacy_components = None,
            (None, _) => {}
        }
        components.push(CssAuthoredGridFixedRepeatComponent::FixedSize(fixed));
    }
    if !has_track {
        return Err(unsupported_value(
            input,
            None,
            "grid repeat content is missing a track",
        ));
    }
    let old_count = match kind {
        CssAuthoredGridAutoRepeatKind::AutoFill => CssGridRepeatCount::AutoFill,
        CssAuthoredGridAutoRepeatKind::AutoFit => CssGridRepeatCount::AutoFit,
    };
    let legacy = legacy_components
        .map(|components| CssGridRepeat::new(old_count, CssGridTrackList::new(components)));
    Ok(ParsedGridTrackComponent::AutoRepeat {
        value: CssAuthoredGridAutoRepeat::new(
            kind,
            CssAuthoredGridFixedRepeatContent::new(components),
        ),
        i01: legacy,
    })
}

fn parse_grid_track_size<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredGridTrackSize, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let state = input.state();
    match input.next().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("minmax") => {
            input.parse_nested_block(|input| {
                let min = parse_grid_inflexible_track_breadth(input)?;
                input.expect_comma().map_err(basic)?;
                let max = parse_grid_track_breadth(input)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(CssAuthoredGridTrackSize::from_minmax(min, max))
            })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("fit-content") => input
            .parse_nested_block(|input| {
                let limit =
                    parse_length_with_context(input, LengthGrammar::GridTrack, "grid fit-content")?;
                input.expect_exhausted().map_err(basic)?;
                Ok(CssAuthoredGridTrackSize::from_fit_content(limit))
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("repeat") => Err(unsupported_value_at(
            location,
            None,
            "repeat() is a grid track list component, not a track size",
        )),
        _ => {
            input.reset(&state);
            parse_grid_track_breadth(input).map(CssAuthoredGridTrackSize::from_breadth)
        }
    }
}

fn parse_grid_inflexible_track_breadth<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredGridTrackBreadth, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let breadth = parse_grid_track_breadth(input)?;
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
) -> std::result::Result<CssAuthoredGridTrackBreadth, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, .. } if !value.is_finite() => Err(unsupported_value_at(
            location,
            None,
            "unsupported non-finite grid track dimension",
        )),
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("fr") => {
            if *value < 0.0 {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported negative grid flex fraction",
                ))
            } else {
                Ok(CssAuthoredGridTrackBreadth::from_fraction(
                    CssNonNegativeNumber::try_new(*value).expect("checked grid fraction"),
                ))
            }
        }
        Token::Dimension { value, unit, .. } => match classify_length_unit(unit) {
            LengthUnitStatus::Supported(_) if *value < 0.0 => Err(unsupported_value_at(
                location,
                None,
                "unsupported negative grid track length",
            )),
            LengthUnitStatus::Supported(unit) => Ok(CssAuthoredGridTrackBreadth::from_length(
                CssLength::dimension(*value, unit),
            )),
            LengthUnitStatus::Unknown => Err(unsupported_value_at(
                location,
                None,
                format!("unknown grid track unit `{unit}`"),
            )),
        },
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                "unsupported non-finite grid track percentage",
            )?;
            if value < 0.0 {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported negative grid track percentage",
                ))
            } else {
                Ok(CssAuthoredGridTrackBreadth::from_length(
                    CssLength::percent(value),
                ))
            }
        }
        Token::Number { value, .. } if *value == 0.0 => {
            Ok(CssAuthoredGridTrackBreadth::from_length(CssLength::Zero))
        }
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "min-content" => Ok(CssAuthoredGridTrackBreadth::min_content()),
            "max-content" => Ok(CssAuthoredGridTrackBreadth::max_content()),
            "auto" => Ok(CssAuthoredGridTrackBreadth::auto()),
            _ => Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("grid track", ident.as_ref()),
            )),
        },
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            let calc = input.parse_nested_block(|input| {
                parse_calc_length_with_grammar(input, LengthGrammar::GridTrack)
            })?;
            Ok(CssAuthoredGridTrackBreadth::from_length(CssLength::Calc(
                calc,
            )))
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported grid track function `{name}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn build_grid_track_list<'i>(
    components: Vec<LocatedGridTrackComponent>,
) -> std::result::Result<CssParsedGridTrackList, ParseError<'i, Error>> {
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

    let i01_components = components
        .iter()
        .map(|located| match &located.component {
            ParsedGridTrackComponent::LineNames(value) => {
                Some(CssGridTrackComponent::LineNames(value.clone()))
            }
            ParsedGridTrackComponent::TrackSize(value) => {
                value.i01_projection().map(CssGridTrackComponent::TrackSize)
            }
            ParsedGridTrackComponent::IntegerRepeat { i01, .. }
            | ParsedGridTrackComponent::AutoRepeat { i01, .. } => {
                i01.clone().map(CssGridTrackComponent::Repeat)
            }
        })
        .collect::<Option<Vec<_>>>()
        .map(CssGridTrackList::new);

    let current = if auto_repeat_count == 0 {
        let values = components
            .into_iter()
            .map(|located| match located.component {
                ParsedGridTrackComponent::LineNames(value) => {
                    CssAuthoredGridGeneralTrackComponent::LineNames(value)
                }
                ParsedGridTrackComponent::TrackSize(value) => {
                    CssAuthoredGridGeneralTrackComponent::TrackSize(value)
                }
                ParsedGridTrackComponent::IntegerRepeat { track, .. } => {
                    CssAuthoredGridGeneralTrackComponent::Repeat(track)
                }
                ParsedGridTrackComponent::AutoRepeat { .. } => {
                    unreachable!("general list has no auto repetition")
                }
            })
            .collect();
        CssAuthoredGridTrackList::general(CssAuthoredGridGeneralTrackList::new(values))
    } else {
        let mut values = Vec::with_capacity(components.len());
        for located in components {
            let value = match located.component {
                ParsedGridTrackComponent::LineNames(value) => {
                    CssAuthoredGridAutoTrackComponent::LineNames(value)
                }
                ParsedGridTrackComponent::TrackSize(value) => {
                    let Some(value) = grid_fixed_size(&value) else {
                        return Err(unsupported_value_at(
                            located.location,
                            None,
                            "tracks surrounding automatic repetition must be fixed-size",
                        ));
                    };
                    CssAuthoredGridAutoTrackComponent::FixedSize(value)
                }
                ParsedGridTrackComponent::IntegerRepeat { fixed, .. } => {
                    let Some(value) = fixed else {
                        return Err(unsupported_value_at(
                            located.location,
                            None,
                            "repetition surrounding automatic repetition must be fixed-size",
                        ));
                    };
                    CssAuthoredGridAutoTrackComponent::Repeat(value)
                }
                ParsedGridTrackComponent::AutoRepeat { value, .. } => {
                    CssAuthoredGridAutoTrackComponent::AutoRepeat(value)
                }
            };
            values.push(value);
        }
        CssAuthoredGridTrackList::auto(CssAuthoredGridAutoTrackList::new(values))
    };

    Ok(CssParsedGridTrackList::new(current, i01_components))
}

fn grid_fixed_size(size: &CssAuthoredGridTrackSize) -> Option<CssAuthoredGridFixedSize> {
    size.is_fixed()
        .then(|| CssAuthoredGridFixedSize::new(size.clone()))
}

pub(super) fn parse_grid_auto_track_sizes<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedGridTrackSizeList, ParseError<'i, Error>> {
    parse_grid_auto_track_sizes_with_mode(input, false)
}

fn parse_grid_auto_track_sizes_with_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
    stop_at_slash: bool,
) -> std::result::Result<CssParsedGridTrackSizeList, ParseError<'i, Error>> {
    let mut sizes = Vec::new();
    while !input.is_exhausted() && !(stop_at_slash && next_is_delim(input, '/')) {
        sizes.push(parse_grid_track_size(input)?);
    }
    if sizes.is_empty() {
        return Err(unsupported_value(
            input,
            None,
            "grid automatic track list is missing a track size",
        ));
    }
    let i01_subset = sizes
        .iter()
        .map(|size| size.i01_projection().map(CssGridTrackComponent::TrackSize))
        .collect::<Option<Vec<_>>>()
        .map(CssGridTrackList::new);
    Ok(CssParsedGridTrackSizeList::new(
        CssAuthoredGridTrackSizeList::new(sizes),
        i01_subset,
    ))
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
        rows.push(parse_grid_template_area_row(row.as_ref(), location)?);
    }
    validate_grid_template_area_rectangles(&rows, input)?;
    Ok(CssGridTemplateAreas::rows(rows))
}

pub(super) fn parse_grid_template_area_row<'i>(
    row: &str,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssGridTemplateAreaRow, ParseError<'i, Error>> {
    let cells = row
        .split_whitespace()
        .map(|token| {
            if token.chars().all(|ch| ch == '.') {
                Ok(CssGridTemplateAreaCell::Empty)
            } else if token.contains('.') {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("invalid grid template area token `{token}`"),
                ))
            } else {
                parse_custom_ident_from_str_at("grid template area", token, location)
                    .map(CssGridTemplateAreaCell::Named)
            }
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if cells.is_empty() {
        Err(unsupported_value_at(
            location,
            None,
            "grid template area row is empty",
        ))
    } else {
        Ok(CssGridTemplateAreaRow::new(cells))
    }
}

#[derive(Clone, Copy)]
pub(super) struct GridAreaBounds {
    min_row: usize,
    max_row: usize,
    min_col: usize,
    max_col: usize,
    count: usize,
}

pub(super) fn validate_grid_template_area_rectangles<'i, 't>(
    rows: &[CssGridTemplateAreaRow],
    input: &Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    if rows.is_empty() {
        return Err(unsupported_value(
            input,
            None,
            "grid-template-areas is missing rows",
        ));
    }

    let width = rows[0].cells().len();
    let mut bounds = HashMap::<String, GridAreaBounds>::new();
    for (row_index, row) in rows.iter().enumerate() {
        if row.cells().len() != width {
            return Err(unsupported_value(
                input,
                None,
                "grid-template-areas rows have inconsistent widths",
            ));
        }
        for (col_index, cell) in row.cells().iter().enumerate() {
            let CssGridTemplateAreaCell::Named(name) = cell else {
                continue;
            };
            bounds
                .entry(name.as_str().to_owned())
                .and_modify(|bounds| {
                    bounds.min_row = bounds.min_row.min(row_index);
                    bounds.max_row = bounds.max_row.max(row_index);
                    bounds.min_col = bounds.min_col.min(col_index);
                    bounds.max_col = bounds.max_col.max(col_index);
                    bounds.count += 1;
                })
                .or_insert(GridAreaBounds {
                    min_row: row_index,
                    max_row: row_index,
                    min_col: col_index,
                    max_col: col_index,
                    count: 1,
                });
        }
    }

    for (name, bounds) in bounds {
        let rectangle_area =
            (bounds.max_row - bounds.min_row + 1) * (bounds.max_col - bounds.min_col + 1);
        if rectangle_area != bounds.count {
            return Err(unsupported_value(
                input,
                None,
                format!("grid template area `{name}` is not rectangular"),
            ));
        }
    }
    Ok(())
}

pub(super) fn parse_grid_template<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedGridTemplate, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "none" => Ok(CssParsedGridTemplate::new(
                CssAuthoredGridTemplateValue::none(),
                Some(CssGridTemplate::None),
            )),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("grid-template", ident.as_ref()),
            )),
        };
    }

    let rows = parse_grid_track_list_with_mode(input, true)?;
    let columns = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_track_list_with_mode(input, false)?)
    } else {
        None
    };
    let (rows_current, rows_i01) = rows.into_parts();
    let (columns_current, columns_i01, columns_project) = match columns {
        Some(value) => {
            let (current, i01) = value.into_parts();
            let projects = i01.is_some();
            (Some(current), i01, projects)
        }
        None => (None, None, true),
    };
    let current = CssAuthoredGridTemplateValue::rows_columns(rows_current, columns_current);
    let i01_subset = match (rows_i01, columns_project) {
        (Some(rows), true) => Some(CssGridTemplate::RowsColumns {
            rows,
            columns: columns_i01,
        }),
        (None, _) | (_, false) => None,
    };
    Ok(CssParsedGridTemplate::new(current, i01_subset))
}

pub(super) fn parse_grid_auto_flow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridAutoFlow, ParseError<'i, Error>> {
    let axis = parse_grid_auto_flow_axis(input)?;
    let dense = input
        .try_parse(|input| input.expect_ident_matching("dense"))
        .is_ok();
    Ok(CssGridAutoFlow::new(axis, dense))
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

pub(super) fn parse_grid_line<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridLine, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(CssGridLine::Auto),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("span") => parse_grid_line_span(input),
        Token::Ident(ident) => {
            parse_custom_ident_from_str_at("grid line", ident.as_ref(), location)
                .map(CssGridLine::CustomIdent)
        }
        Token::Number {
            int_value: Some(value),
            ..
        } if *value != 0 => Ok(CssGridLine::integer(*value)),
        Token::Number {
            int_value: Some(_), ..
        } => Err(unsupported_value_at(
            location,
            None,
            "grid line integer must not be zero",
        )),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            "grid line number must be an integer",
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_grid_line_span<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridLine, ParseError<'i, Error>> {
    let mut integer = None;
    let mut name = None;

    while !input.is_exhausted() && !next_is_delim(input, '/') {
        if integer.is_none() {
            let parsed = input.try_parse(|input| parse_positive_integer(input, "grid span"));
            if let Ok(value) = parsed {
                integer = Some(value);
                continue;
            }
        }

        if name.is_none() {
            let location = input.current_source_location();
            let parsed = input.try_parse(Parser::expect_ident_cloned);
            if let Ok(ident) = parsed {
                name = Some(parse_custom_ident_from_str_at(
                    "grid span",
                    ident.as_ref(),
                    location,
                )?);
                continue;
            }
        }

        return Err(unsupported_value(
            input,
            None,
            "unsupported grid span component",
        ));
    }

    if integer.is_none() && name.is_none() {
        Err(unsupported_value(
            input,
            None,
            "grid span is missing an integer or name",
        ))
    } else {
        Ok(CssGridLine::span(integer, name))
    }
}

pub(super) fn parse_grid_line_range<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridLineRange, ParseError<'i, Error>> {
    let start = parse_grid_line(input)?;
    let end = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input)?)
    } else {
        None
    };
    Ok(CssGridLineRange::new(start, end))
}

pub(super) fn parse_grid_area<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridArea, ParseError<'i, Error>> {
    let row_start = parse_grid_line(input)?;
    let column_start = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input)?)
    } else {
        None
    };
    let row_end = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input)?)
    } else {
        None
    };
    let column_end = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_grid_line(input)?)
    } else {
        None
    };
    Ok(CssGridArea::new(
        row_start,
        column_start,
        row_end,
        column_end,
    ))
}

pub(super) fn parse_grid<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedGrid, ParseError<'i, Error>> {
    let state = input.state();
    let is_auto_flow = input
        .try_parse(|input| input.expect_ident_matching("auto-flow"))
        .is_ok();
    input.reset(&state);
    if is_auto_flow {
        parse_grid_auto_flow_shorthand(input)
    } else {
        let template = parse_grid_template(input)?;
        let (current, i01_subset) = template.into_parts();
        Ok(CssParsedGrid::new(
            CssAuthoredGridValue::template(current),
            i01_subset.map(CssGrid::Template),
        ))
    }
}

pub(super) fn parse_grid_auto_flow_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedGrid, ParseError<'i, Error>> {
    input.expect_ident_matching("auto-flow").map_err(basic)?;
    let dense = input
        .try_parse(|input| input.expect_ident_matching("dense"))
        .is_ok();
    let auto_tracks = if !input.is_exhausted() && !next_is_delim(input, '/') {
        Some(parse_grid_auto_track_sizes_with_mode(input, true)?)
    } else {
        None
    };
    input.expect_delim('/').map_err(basic)?;
    let explicit_tracks = parse_grid_track_list_with_mode(input, false)?;
    let flow = CssGridAutoFlow::new(CssGridAutoFlowAxis::Row, dense);
    let (auto_current, auto_i01, auto_projects) = match auto_tracks {
        Some(value) => {
            let (current, i01) = value.into_parts();
            let projects = i01.is_some();
            (Some(current), i01, projects)
        }
        None => (None, None, true),
    };
    let (explicit_current, explicit_i01) = explicit_tracks.into_parts();
    let current = CssAuthoredGridValue::from_auto_flow(flow, auto_current, explicit_current);
    let i01_subset = match (auto_projects, explicit_i01) {
        (true, Some(explicit_tracks)) => Some(CssGrid::AutoFlow {
            flow,
            auto_tracks: auto_i01,
            explicit_tracks,
        }),
        (false, _) | (_, None) => None,
    };
    Ok(CssParsedGrid::new(current, i01_subset))
}
