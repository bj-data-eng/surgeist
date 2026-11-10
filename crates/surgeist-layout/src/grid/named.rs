use crate::{
    GridLine, GridPlacement, GridSpan, GridTemplateAreas, LayoutScalar, RawGridLine,
    RawGridPlacement, SubgridLineNameComponent, SubgridLineNameRepeatCount, TrackComponentOf,
    TrackRepeat,
};

use super::{
    GridAxisKind, GridContainerProjection, GridParentContext, InheritedGridAxis,
    placement::GridPlacementDemandError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NamedGridLines {
    pub(super) axis: GridAxisKind,
    pub(super) explicit_track_count: usize,
    pub(super) line_names: Vec<Vec<LineNameEntry>>,
    pub(super) area_facts: GridAreaNameFacts,
    used_frame: Option<UsedGridLineFrame>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UsedGridLineFrame {
    explicit_start: usize,
    track_count: usize,
}

impl NamedGridLines {
    pub(super) fn positioned_stacking(axis: GridAxisKind) -> Self {
        // Two labelled bounds use one numbering interval. This positioned
        // reference frame creates no tracks or placement demand.
        Self::new(axis, 1)
    }

    pub(super) fn new(axis: GridAxisKind, explicit_track_count: usize) -> Self {
        Self {
            axis,
            explicit_track_count,
            line_names: vec![Vec::new(); explicit_track_count + 1],
            area_facts: GridAreaNameFacts::default(),
            used_frame: None,
        }
    }

    pub(super) fn unnamed_used_frame(&self) -> Self {
        let mut unnamed = Self::new(self.axis, 0);
        unnamed.used_frame = Some(self.used_frame());
        unnamed
    }

    pub(super) fn set_used_frame(&mut self, explicit_start: usize, track_count: usize) {
        self.used_frame = (explicit_start != 0 || track_count != self.explicit_track_count)
            .then_some(UsedGridLineFrame {
                explicit_start,
                track_count,
            });
    }

    fn used_frame(&self) -> UsedGridLineFrame {
        self.used_frame.unwrap_or(UsedGridLineFrame {
            explicit_start: 0,
            track_count: self.explicit_track_count,
        })
    }

    fn ensure_track_count(&mut self, explicit_track_count: usize) {
        self.explicit_track_count = self.explicit_track_count.max(explicit_track_count);
        self.line_names
            .resize_with(self.explicit_track_count + 1, Vec::new);
    }

    fn add_line_names(&mut self, line_index: usize, names: &[String], origin: LineNameOrigin) {
        self.ensure_track_count(line_index);
        self.line_names[line_index].extend(
            names
                .iter()
                .cloned()
                .map(|name| LineNameEntry { name, origin }),
        );
    }

    fn add_area_name(&mut self, line_index: usize, name: String) {
        self.ensure_track_count(line_index);
        self.line_names[line_index].push(LineNameEntry {
            name,
            origin: LineNameOrigin::AreaGenerated,
        });
    }

    #[cfg(test)]
    pub(super) fn named_occurrences(&self, name: &str) -> Vec<usize> {
        self.line_names
            .iter()
            .enumerate()
            .filter_map(|(line_index, entries)| {
                entries
                    .iter()
                    .any(|entry| entry.name == name)
                    .then_some(line_index + 1)
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn entries_on_line(&self, line: usize) -> &[LineNameEntry] {
        if line == 0 {
            return &[];
        }

        self.line_names
            .get(line - 1)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LineNameEntry {
    pub(super) name: String,
    pub(super) origin: LineNameOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LineNameOrigin {
    Explicit,
    Inherited,
    AreaGenerated,
    LocalSubgrid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NamedGridError {
    ReservedLineName {
        name: String,
    },
    UnresolvedAutoRepeatNames {
        axis: GridAxisKind,
    },
    EmptyTemplateAreas,
    TemplateAreaRowLengthMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    NonRectangularTemplateArea {
        name: String,
    },
    ZeroRepeat {
        axis: GridAxisKind,
    },
    MultipleAutoFillRepeats {
        axis: GridAxisKind,
    },
    ZeroLine,
    ZeroSpan,
    AutoWithoutCursor,
    LineBeforeFirst {
        axis: GridAxisKind,
        line: isize,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NamedGridReport {
    errors: Vec<NamedGridErrorReport>,
}

impl NamedGridReport {
    pub fn errors(&self) -> &[NamedGridErrorReport] {
        &self.errors
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub(super) fn from_error(error: NamedGridError) -> Self {
        let mut report = Self::default();
        report.push_error(error);
        report
    }

    pub(super) fn push_error(&mut self, error: NamedGridError) {
        self.errors.push(error.into());
    }

    pub(super) fn extend(&mut self, other: NamedGridReport) {
        self.errors.extend(other.errors);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedGridErrorReport {
    ReservedLineName {
        name: String,
    },
    UnresolvedAutoRepeatNames {
        axis: GridAxisKind,
    },
    EmptyTemplateAreas,
    TemplateAreaRowLengthMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    NonRectangularTemplateArea {
        name: String,
    },
    ZeroRepeat {
        axis: GridAxisKind,
    },
    MultipleAutoFillRepeats {
        axis: GridAxisKind,
    },
    ZeroLine,
    ZeroSpan,
    AutoWithoutCursor,
    LineBeforeFirst {
        axis: GridAxisKind,
        line: isize,
    },
}

impl From<NamedGridError> for NamedGridErrorReport {
    fn from(error: NamedGridError) -> Self {
        match error {
            NamedGridError::ReservedLineName { name } => Self::ReservedLineName { name },
            NamedGridError::UnresolvedAutoRepeatNames { axis } => {
                Self::UnresolvedAutoRepeatNames { axis }
            }
            NamedGridError::EmptyTemplateAreas => Self::EmptyTemplateAreas,
            NamedGridError::TemplateAreaRowLengthMismatch {
                row,
                expected,
                actual,
            } => Self::TemplateAreaRowLengthMismatch {
                row,
                expected,
                actual,
            },
            NamedGridError::NonRectangularTemplateArea { name } => {
                Self::NonRectangularTemplateArea { name }
            }
            NamedGridError::ZeroRepeat { axis } => Self::ZeroRepeat { axis },
            NamedGridError::MultipleAutoFillRepeats { axis } => {
                Self::MultipleAutoFillRepeats { axis }
            }
            NamedGridError::ZeroLine => Self::ZeroLine,
            NamedGridError::ZeroSpan => Self::ZeroSpan,
            NamedGridError::AutoWithoutCursor => Self::AutoWithoutCursor,
            NamedGridError::LineBeforeFirst { axis, line } => Self::LineBeforeFirst { axis, line },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlacementSide {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpanSearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NamedSpanScope {
    Standalone,
    Inherited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedLine {
    css_line: i128,
    absolute_line: i128,
}

impl ResolvedLine {
    fn from_absolute(line: i128, explicit_end: i128) -> Self {
        let css_line = if line > 0 {
            line
        } else {
            line - explicit_end - 1
        };
        Self {
            css_line,
            absolute_line: line,
        }
    }

    fn grid_line(self) -> Option<GridLine> {
        isize::try_from(self.css_line).ok().and_then(GridLine::new)
    }

    fn require_grid_line(self) -> Result<GridLine, GridPlacementResolutionError> {
        self.grid_line()
            .ok_or(GridPlacementResolutionError::Capacity)
    }
}

#[derive(Debug)]
pub(super) enum GridPlacementResolutionError {
    Named(NamedGridError),
    Capacity,
}

impl From<NamedGridError> for GridPlacementResolutionError {
    fn from(error: NamedGridError) -> Self {
        Self::Named(error)
    }
}

fn normalized_raw_grid_lines(placement: &RawGridPlacement) -> (RawGridLine, RawGridLine) {
    let mut start = placement.start.clone();
    let mut end = placement.end.clone();

    if matches!(start, RawGridLine::NamedSpan { .. }) && matches!(end, RawGridLine::Auto) {
        start = RawGridLine::Span(1);
    }
    if matches!(end, RawGridLine::NamedSpan { .. }) && matches!(start, RawGridLine::Auto) {
        end = RawGridLine::Span(1);
    }
    if is_span(&start) && is_span(&end) {
        end = RawGridLine::Auto;
    }
    if matches!(start, RawGridLine::NamedSpan { .. }) && matches!(end, RawGridLine::Auto) {
        start = RawGridLine::Span(1);
    }
    (start, end)
}

pub(super) fn resolve_grid_placement(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> Result<GridPlacement, GridPlacementResolutionError> {
    let (start, end) = normalized_raw_grid_lines(placement);
    let explicit_end = lines.explicit_track_count as i128 + 1;

    match (&start, &end) {
        (RawGridLine::Auto, RawGridLine::Auto) => Ok(GridPlacement::AUTO),
        (RawGridLine::Auto, span) | (span, RawGridLine::Auto) if is_span(span) => {
            let span = resolve_span_count(span)?;
            if let Some(cursor) = auto_cursor_line {
                Ok(GridPlacement::try_line_span(cursor, span)
                    .expect("resolved cursor line/span must be valid"))
            } else {
                Ok(GridPlacement::try_span(span).expect("resolved grid span must be valid"))
            }
        }
        (RawGridLine::Auto, end) if is_definite_line(end) => {
            let end = resolve_line(lines, end, PlacementSide::End)?;
            validate_resolved_range(lines, end.absolute_line - 1, end.absolute_line)?;
            Ok(GridPlacement::end_line(end.require_grid_line()?))
        }
        (start, RawGridLine::Auto) if is_definite_line(start) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            validate_resolved_range(lines, start.absolute_line, start.absolute_line + 1)?;
            Ok(GridPlacement::line(start.require_grid_line()?))
        }
        (start, span) if is_definite_line(start) && is_span(span) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            match span {
                RawGridLine::NamedSpan { name, index } => {
                    let end = resolve_named_span(
                        lines,
                        start.absolute_line,
                        name,
                        *index,
                        SpanSearchDirection::Forward,
                        NamedSpanScope::Standalone,
                    )?;
                    normalize_resolved_lines(
                        lines,
                        start,
                        ResolvedLine::from_absolute(end, explicit_end),
                    )
                }
                _ => {
                    let span = resolve_span_count(span)?;
                    validate_resolved_range(
                        lines,
                        start.absolute_line,
                        start.absolute_line + span as i128,
                    )?;
                    Ok(
                        GridPlacement::try_line_span(start.require_grid_line()?.get(), span)
                            .expect("resolved grid line/span must be valid"),
                    )
                }
            }
        }
        (span, end) if is_span(span) && is_definite_line(end) => {
            let end = resolve_line(lines, end, PlacementSide::End)?;
            match span {
                RawGridLine::NamedSpan { name, index } => {
                    let start = resolve_named_span(
                        lines,
                        end.absolute_line,
                        name,
                        *index,
                        SpanSearchDirection::Backward,
                        NamedSpanScope::Standalone,
                    )?;
                    normalize_resolved_lines(
                        lines,
                        ResolvedLine::from_absolute(start, explicit_end),
                        end,
                    )
                }
                _ => {
                    let span = resolve_span_count(span)?;
                    validate_resolved_range(
                        lines,
                        end.absolute_line - span as i128,
                        end.absolute_line,
                    )?;
                    Ok(
                        GridPlacement::try_span_line(span, end.require_grid_line()?.get())
                            .expect("resolved grid span/end line must be valid"),
                    )
                }
            }
        }
        (start, end) if is_definite_line(start) && is_definite_line(end) => {
            let start = resolve_line(lines, start, PlacementSide::Start)?;
            let end = resolve_line(lines, end, PlacementSide::End)?;
            normalize_resolved_lines(lines, start, end)
        }
        _ => Err(NamedGridError::AutoWithoutCursor.into()),
    }
}

#[cfg(test)]
pub(super) fn resolve_grid_placement_or_auto(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
) -> GridPlacement {
    resolve_grid_placement_or_auto_with_report(lines, placement, auto_cursor_line, None)
        .expect("test grid placement is representable")
        .0
}

pub(super) fn resolve_grid_placement_or_auto_with_report(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
    automatic_span: Option<GridSpan>,
) -> Result<(GridPlacement, NamedGridReport), GridPlacementDemandError> {
    let anchor = match (&placement.start, &placement.end) {
        (RawGridLine::Auto, line) if is_definite_line(line) => Some((line, PlacementSide::End)),
        (line, RawGridLine::Auto) if is_definite_line(line) => Some((line, PlacementSide::Start)),
        _ => None,
    };
    let resolved = if automatic_span.is_some()
        && let Some((line, side)) = anchor
    {
        // The derived extent can legitimately grow tracks before the explicit grid.
        resolve_line(lines, line, side)
            .map_err(GridPlacementResolutionError::from)
            .and_then(ResolvedLine::require_grid_line)
            .map(|line| match side {
                PlacementSide::Start => GridPlacement::line(line),
                PlacementSide::End => GridPlacement::end_line(line),
            })
    } else {
        resolve_grid_placement(lines, placement, auto_cursor_line)
    };
    match resolved {
        Ok(placement) => Ok((placement, NamedGridReport::default())),
        Err(GridPlacementResolutionError::Named(error)) => {
            Ok((GridPlacement::AUTO, NamedGridReport::from_error(error)))
        }
        Err(GridPlacementResolutionError::Capacity) => {
            Err(GridPlacementDemandError::UnrepresentableAxisPlacement { axis: lines.axis })
        }
    }
}

pub(super) fn resolve_positioned_grid_placement_with_report(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
) -> (GridPlacement, NamedGridReport) {
    match resolve_positioned_grid_lines(lines, placement) {
        Ok(result) => result,
        Err(error) => (GridPlacement::AUTO, NamedGridReport::from_error(error)),
    }
}

fn resolve_positioned_grid_lines(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
) -> Result<(GridPlacement, NamedGridReport), NamedGridError> {
    let (start, end) = normalized_raw_grid_lines(placement);
    match (&start, &end) {
        (RawGridLine::Auto, span) | (span, RawGridLine::Auto) if is_span(span) => {
            resolve_span_count(span)?;
            return Ok((GridPlacement::AUTO, NamedGridReport::default()));
        }
        (anchor, span) if is_definite_line(anchor) && is_span(span) => {
            return resolve_positioned_grid_span(lines, anchor, span, PlacementSide::Start);
        }
        (span, anchor) if is_span(span) && is_definite_line(anchor) => {
            return resolve_positioned_grid_span(lines, anchor, span, PlacementSide::End);
        }
        _ => {}
    }
    let resolve_edge = |line: &RawGridLine, side| match line {
        RawGridLine::Auto => Ok(None),
        _ => resolve_line(lines, line, side).map(Some),
    };
    let mut start = resolve_edge(&start, PlacementSide::Start)?;
    let mut end = resolve_edge(&end, PlacementSide::End)?;
    let (start_line, end_line) = match (start, end) {
        (Some(start), Some(end)) => (start.absolute_line, end.absolute_line),
        (Some(start), None) => (start.absolute_line, start.absolute_line + 1),
        (None, Some(end)) => (end.absolute_line - 1, end.absolute_line),
        (None, None) => return Ok((GridPlacement::AUTO, NamedGridReport::default())),
    };
    let report = positioned_grid_range_report(lines, start_line, end_line);
    if start_line > end_line {
        std::mem::swap(&mut start, &mut end);
    } else if start_line == end_line {
        end = None;
    }

    // A line beyond the placement representation cannot exist in the used grid.
    let start = start.and_then(ResolvedLine::grid_line);
    let end = end.and_then(ResolvedLine::grid_line);
    let placement = match (start, end) {
        (Some(start), Some(end)) => GridPlacement::lines(start, end),
        (Some(start), None) => GridPlacement::line(start),
        (None, Some(end)) => GridPlacement::end_line(end),
        (None, None) => GridPlacement::AUTO,
    };
    Ok((placement, report))
}

fn resolve_positioned_grid_span(
    lines: &NamedGridLines,
    anchor: &RawGridLine,
    span: &RawGridLine,
    side: PlacementSide,
) -> Result<(GridPlacement, NamedGridReport), NamedGridError> {
    let anchor = resolve_line(lines, anchor, side)?;
    let count = resolve_span_count(span)?;
    let direction = match side {
        PlacementSide::Start => SpanSearchDirection::Forward,
        PlacementSide::End => SpanSearchDirection::Backward,
    };
    let opposite = match span {
        RawGridLine::NamedSpan { name, .. } => resolve_named_span(
            lines,
            anchor.absolute_line,
            name,
            count,
            direction,
            NamedSpanScope::Standalone,
        )?,
        _ => match side {
            PlacementSide::Start => anchor.absolute_line + count as i128,
            PlacementSide::End => anchor.absolute_line - count as i128,
        },
    };
    let report = positioned_grid_range_report(lines, anchor.absolute_line, opposite);
    let Some(line) = anchor.grid_line() else {
        return Ok((GridPlacement::AUTO, report));
    };
    let distance = (opposite - anchor.absolute_line).abs();
    let span = usize::try_from(distance).ok().and_then(GridSpan::new);
    // Keep the anchor attached until the used grid qualifies it. A distance
    // beyond usize cannot fit that grid, so only its derived edge becomes Auto.
    let placement = match (side, span) {
        (PlacementSide::Start, Some(span)) => GridPlacement::line_span(line, span),
        (PlacementSide::End, Some(span)) => GridPlacement::span_line(span, line),
        (PlacementSide::Start, None) => GridPlacement::line(line),
        (PlacementSide::End, None) => GridPlacement::end_line(line),
    };
    Ok((placement, report))
}

fn positioned_grid_range_report(lines: &NamedGridLines, start: i128, end: i128) -> NamedGridReport {
    // Leading used tracks can contain a line before the explicit grid.
    // Retain the diagnostic without rejecting either positioned reference.
    if let Err(GridPlacementResolutionError::Named(error)) =
        validate_resolved_range(lines, start, end)
    {
        NamedGridReport::from_error(error)
    } else {
        NamedGridReport::default()
    }
}

pub(super) fn resolve_subgrid_placement(
    lines: &NamedGridLines,
    placement: &RawGridPlacement,
    auto_cursor_line: Option<isize>,
    automatic_span: Option<GridSpan>,
) -> Result<GridPlacement, NamedGridError> {
    let (start, end) = normalized_raw_grid_lines(placement);
    let auto_cursor_line = auto_cursor_line.map(|line| line as i128);
    let default_tracks = automatic_span.map_or(1, GridSpan::get) as i128;
    let resolve_edge = |line: &RawGridLine, side| {
        resolve_line(lines, line, side).map(|resolved| resolved.absolute_line)
    };
    let (mut start_line, mut end_line) = match (&start, &end) {
        (RawGridLine::Auto, RawGridLine::Auto) => {
            let Some(cursor) = auto_cursor_line else {
                let bounded_span = automatic_span.and_then(|span| {
                    GridSpan::new(span.get().min(lines.explicit_track_count.max(1)))
                });
                return Ok(GridPlacement::AUTO.with_automatic_span(bounded_span));
            };
            (cursor, cursor + default_tracks)
        }
        (RawGridLine::Auto, span) | (span, RawGridLine::Auto) if is_span(span) => {
            let span = resolve_span_count(span)?;
            let Some(cursor) = auto_cursor_line else {
                return Ok(GridPlacement::try_span(span).expect("resolved grid span is valid"));
            };
            (cursor, cursor + span as i128)
        }
        (RawGridLine::Auto, end) if is_definite_line(end) => {
            let end = resolve_edge(end, PlacementSide::End)?;
            (end - default_tracks, end)
        }
        (start, RawGridLine::Auto) if is_definite_line(start) => {
            let start = resolve_edge(start, PlacementSide::Start)?;
            (start, start + default_tracks)
        }
        (start, span) if is_definite_line(start) && is_span(span) => {
            let start = resolve_edge(start, PlacementSide::Start)?;
            let end = match span {
                RawGridLine::NamedSpan { name, index } => resolve_named_span(
                    lines,
                    start,
                    name,
                    *index,
                    SpanSearchDirection::Forward,
                    NamedSpanScope::Inherited,
                )?,
                _ => start + resolve_span_count(span)? as i128,
            };
            (start, end)
        }
        (span, end) if is_span(span) && is_definite_line(end) => {
            let end = resolve_edge(end, PlacementSide::End)?;
            let start = match span {
                RawGridLine::NamedSpan { name, index } => resolve_named_span(
                    lines,
                    end,
                    name,
                    *index,
                    SpanSearchDirection::Backward,
                    NamedSpanScope::Inherited,
                )?,
                _ => end - resolve_span_count(span)? as i128,
            };
            (start, end)
        }
        (start, end) if is_definite_line(start) && is_definite_line(end) => (
            resolve_edge(start, PlacementSide::Start)?,
            resolve_edge(end, PlacementSide::End)?,
        ),
        _ => return Err(NamedGridError::AutoWithoutCursor),
    };

    if start_line > end_line {
        std::mem::swap(&mut start_line, &mut end_line);
    } else if start_line == end_line {
        end_line = start_line + 1;
    }

    let track_count = lines.explicit_track_count;
    let track_index = |line: i128| (line - 1).clamp(0, track_count as i128) as usize;
    let (start, end) = super::placement::clamp_subgrid_axis_range(
        track_index(start_line),
        track_index(end_line),
        track_count,
    );
    let css_line = |index: usize| {
        isize::try_from(index as i128 + 1).expect("inherited grid lines are representable")
    };
    Ok(GridPlacement::try_lines(css_line(start), css_line(end))
        .expect("clamped subgrid placement lines are valid"))
}

fn resolve_named_span(
    lines: &NamedGridLines,
    edge: i128,
    name: &str,
    count: usize,
    direction: SpanSearchDirection,
    scope: NamedSpanScope,
) -> Result<i128, NamedGridError> {
    validate_line_name(name)?;
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }

    let first_after_explicit = lines.explicit_track_count as i128 + 2;
    let trailing =
        if direction == SpanSearchDirection::Backward && scope == NamedSpanScope::Inherited {
            (edge - first_after_explicit).max(0)
        } else {
            0
        };
    let remaining = count as i128 - trailing;
    if remaining <= 0 {
        return Ok(edge - count as i128);
    }

    let candidates = named_span_candidates(lines, edge, name, direction);
    if remaining <= candidates.len() as i128 {
        return Ok(candidates[remaining as usize - 1] as i128);
    }

    let missing = remaining - candidates.len() as i128;
    Ok(match direction {
        SpanSearchDirection::Forward => first_after_explicit.max(edge + 1) + missing - 1,
        SpanSearchDirection::Backward => (edge - 1).min(0) - missing + 1,
    })
}

fn normalize_resolved_lines(
    lines: &NamedGridLines,
    mut start: ResolvedLine,
    mut end: ResolvedLine,
) -> Result<GridPlacement, GridPlacementResolutionError> {
    if start.absolute_line > end.absolute_line {
        std::mem::swap(&mut start, &mut end);
    }
    validate_resolved_range(lines, start.absolute_line, end.absolute_line)?;
    if start.absolute_line == end.absolute_line {
        return Ok(
            GridPlacement::try_line_span(start.require_grid_line()?.get(), 1)
                .expect("normalized equal grid lines produce a valid span"),
        );
    }

    Ok(GridPlacement::lines(
        start.require_grid_line()?,
        end.require_grid_line()?,
    ))
}

fn validate_resolved_range(
    lines: &NamedGridLines,
    start_line: i128,
    end_line: i128,
) -> Result<(), GridPlacementResolutionError> {
    let first_line = start_line.min(end_line);
    if first_line < 1 {
        let line =
            isize::try_from(first_line).map_err(|_| GridPlacementResolutionError::Capacity)?;
        return Err(NamedGridError::LineBeforeFirst {
            axis: lines.axis,
            line,
        }
        .into());
    }
    Ok(())
}

fn resolve_line(
    lines: &NamedGridLines,
    line: &RawGridLine,
    side: PlacementSide,
) -> Result<ResolvedLine, NamedGridError> {
    let explicit_end = lines.explicit_track_count as i128 + 1;
    match line {
        RawGridLine::Line(0) => Err(NamedGridError::ZeroLine),
        RawGridLine::Line(line) => {
            let css_line = *line as i128;
            let absolute_line = if css_line < 0 {
                explicit_end + 1 + css_line
            } else {
                css_line
            };
            Ok(ResolvedLine {
                css_line,
                absolute_line,
            })
        }
        RawGridLine::NamedLine { name, index } => {
            validate_line_name(name)?;
            resolve_named_occurrence(explicit_matches(lines, name), *index, explicit_end)
        }
        RawGridLine::BareIdent(name) => {
            validate_line_name(name)?;
            let side_name = match side {
                PlacementSide::Start => format!("{name}-start"),
                PlacementSide::End => format!("{name}-end"),
            };
            let mut matches = explicit_matches(lines, &side_name);
            if matches.is_empty() {
                matches = explicit_matches(lines, name);
            }
            resolve_named_occurrence(matches, 1, explicit_end)
        }
        _ => Err(NamedGridError::AutoWithoutCursor),
    }
}

fn resolve_named_occurrence(
    matches: Vec<isize>,
    occurrence: isize,
    explicit_end: i128,
) -> Result<ResolvedLine, NamedGridError> {
    let count = occurrence.unsigned_abs();
    if count == 0 {
        return Err(NamedGridError::ZeroLine);
    }
    let line = if count <= matches.len() {
        let index = if occurrence < 0 {
            matches.len() - count
        } else {
            count - 1
        };
        matches[index] as i128
    } else {
        let missing = (count - matches.len()) as i128;
        if occurrence < 0 {
            1 - missing
        } else {
            explicit_end + missing
        }
    };
    Ok(ResolvedLine::from_absolute(line, explicit_end))
}

fn named_span_candidates(
    lines: &NamedGridLines,
    edge: i128,
    name: &str,
    direction: SpanSearchDirection,
) -> Vec<isize> {
    let explicit_matches = explicit_matches(lines, name);
    match direction {
        SpanSearchDirection::Forward => explicit_matches
            .iter()
            .copied()
            .filter(|line| *line as i128 > edge)
            .collect::<Vec<_>>(),
        SpanSearchDirection::Backward => explicit_matches
            .iter()
            .rev()
            .copied()
            .filter(|line| (*line as i128) < edge)
            .collect::<Vec<_>>(),
    }
}

fn resolve_span_count(line: &RawGridLine) -> Result<usize, NamedGridError> {
    let count = match line {
        RawGridLine::Span(count) | RawGridLine::NamedSpan { index: count, .. } => *count,
        _ => return Err(NamedGridError::AutoWithoutCursor),
    };
    if count == 0 {
        return Err(NamedGridError::ZeroSpan);
    }
    Ok(count)
}

fn is_definite_line(line: &RawGridLine) -> bool {
    matches!(
        line,
        RawGridLine::Line(_) | RawGridLine::BareIdent(_) | RawGridLine::NamedLine { .. }
    )
}

fn is_span(line: &RawGridLine) -> bool {
    matches!(line, RawGridLine::Span(_) | RawGridLine::NamedSpan { .. })
}

fn validate_line_name(name: &str) -> Result<(), NamedGridError> {
    if matches!(name, "auto" | "span") {
        return Err(NamedGridError::ReservedLineName {
            name: name.to_string(),
        });
    }
    Ok(())
}

fn explicit_matches(lines: &NamedGridLines, name: &str) -> Vec<isize> {
    lines
        .line_names
        .iter()
        .enumerate()
        .filter_map(|(line_index, entries)| {
            entries
                .iter()
                .any(|entry| entry.name == name)
                .then_some(line_index as isize + 1)
        })
        .collect()
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct GridAreaNameFacts {
    pub(super) row_count: usize,
    pub(super) column_count: usize,
    pub(super) rows_valid: bool,
    pub(super) columns_valid: bool,
    pub(super) area_order: Vec<String>,
    pub(super) area_rectangles: Vec<GridAreaNameRectangle>,
}

impl GridAreaNameFacts {
    pub(super) fn is_valid_for_axis(&self, axis: GridAxisKind) -> bool {
        match axis {
            GridAxisKind::Column => self.columns_valid,
            GridAxisKind::Row => self.rows_valid,
        }
    }

    pub(super) fn from_specified_areas(areas: &GridTemplateAreas) -> Result<Self, NamedGridError> {
        if areas.rows.is_empty() {
            return Err(NamedGridError::EmptyTemplateAreas);
        }

        let column_count = areas.rows[0].cells.len();
        if column_count == 0 {
            return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                row: 1,
                expected: 1,
                actual: 0,
            });
        }

        let mut accumulators = Vec::<AreaAccumulator>::new();
        for (row_index, row) in areas.rows.iter().enumerate() {
            if row.cells.is_empty() {
                return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                    row: row_index + 1,
                    expected: column_count.max(1),
                    actual: 0,
                });
            }
            if row.cells.len() != column_count {
                return Err(NamedGridError::TemplateAreaRowLengthMismatch {
                    row: row_index + 1,
                    expected: column_count,
                    actual: row.cells.len(),
                });
            }

            for (column_index, cell) in row.cells.iter().enumerate() {
                let Some(name) = cell else {
                    continue;
                };

                if let Some(accumulator) = accumulators
                    .iter_mut()
                    .find(|area| area.rectangle.name == *name)
                {
                    accumulator.rectangle.row_start =
                        accumulator.rectangle.row_start.min(row_index + 1);
                    accumulator.rectangle.row_end =
                        accumulator.rectangle.row_end.max(row_index + 2);
                    accumulator.rectangle.column_start =
                        accumulator.rectangle.column_start.min(column_index + 1);
                    accumulator.rectangle.column_end =
                        accumulator.rectangle.column_end.max(column_index + 2);
                    accumulator.cell_count += 1;
                } else {
                    accumulators.push(AreaAccumulator {
                        rectangle: GridAreaNameRectangle {
                            name: name.clone(),
                            row_start: row_index + 1,
                            row_end: row_index + 2,
                            column_start: column_index + 1,
                            column_end: column_index + 2,
                            row_start_name: row_index + 1,
                            row_end_name: row_index + 2,
                            column_start_name: column_index + 1,
                            column_end_name: column_index + 2,
                        },
                        cell_count: 1,
                    });
                }
            }
        }

        let mut area_order = Vec::with_capacity(accumulators.len());
        let mut area_rectangles = Vec::with_capacity(accumulators.len());
        for accumulator in accumulators {
            let mut rectangle = accumulator.rectangle;
            let expected_cells = (rectangle.row_end - rectangle.row_start)
                * (rectangle.column_end - rectangle.column_start);
            if accumulator.cell_count != expected_cells {
                return Err(NamedGridError::NonRectangularTemplateArea {
                    name: rectangle.name,
                });
            }
            rectangle.row_start_name = rectangle.row_start;
            rectangle.row_end_name = rectangle.row_end;
            rectangle.column_start_name = rectangle.column_start;
            rectangle.column_end_name = rectangle.column_end;
            area_order.push(rectangle.name.clone());
            area_rectangles.push(rectangle);
        }

        Ok(Self {
            row_count: areas.rows.len(),
            column_count,
            rows_valid: true,
            columns_valid: true,
            area_order,
            area_rectangles,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GridAreaNameRectangle {
    pub(super) name: String,
    pub(super) row_start: usize,
    pub(super) row_end: usize,
    pub(super) column_start: usize,
    pub(super) column_end: usize,
    pub(super) row_start_name: usize,
    pub(super) row_end_name: usize,
    pub(super) column_start_name: usize,
    pub(super) column_end_name: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AreaAccumulator {
    rectangle: GridAreaNameRectangle,
    cell_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GridNamedContext {
    pub(super) columns: NamedGridLines,
    pub(super) rows: NamedGridLines,
    pub(super) area_facts: Option<GridAreaNameFacts>,
}

pub(super) fn build_grid_named_context<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    explicit_columns: usize,
    explicit_rows: usize,
    parent_context: &GridParentContext<S>,
) -> Result<GridNamedContext, NamedGridError> {
    let style_area_facts = if style.grid_template_areas.rows.is_empty() {
        None
    } else {
        Some(GridAreaNameFacts::from_specified_areas(
            style.grid_template_areas,
        )?)
    };

    let inherited_area_facts = inherited_subgrid_area_facts(parent_context);
    let style_area_facts = clamp_style_area_facts_to_subgrid_axes(style_area_facts, parent_context);
    let merged_area_facts = merge_subgrid_area_facts(
        style_area_facts.clone(),
        inherited_area_facts,
        parent_context,
    );

    let columns = if let Some(parent_axis) = &parent_context.columns {
        let mut columns = inherited_subgrid_axis_named_lines(
            GridAxisKind::Column,
            parent_axis,
            style.grid_template_columns,
            None,
        )?;
        if let Some(facts) = &merged_area_facts {
            columns =
                add_area_generated_lines_from_facts(GridAxisKind::Column, columns, facts.clone());
        }
        columns
    } else {
        let mut columns = named_lines_from_track_components(
            GridAxisKind::Column,
            style.grid_template_columns,
            explicit_columns,
        )?;
        if let Some(facts) = &style_area_facts {
            columns =
                add_area_generated_lines_from_facts(GridAxisKind::Column, columns, facts.clone());
        }
        columns
    };
    let rows = if let Some(parent_axis) = &parent_context.rows {
        let mut rows = inherited_subgrid_axis_named_lines(
            GridAxisKind::Row,
            parent_axis,
            style.grid_template_rows,
            None,
        )?;
        if let Some(facts) = &merged_area_facts {
            rows = add_area_generated_lines_from_facts(GridAxisKind::Row, rows, facts.clone());
        }
        rows
    } else {
        let mut rows = named_lines_from_track_components(
            GridAxisKind::Row,
            style.grid_template_rows,
            explicit_rows,
        )?;
        if let Some(facts) = &style_area_facts {
            rows = add_area_generated_lines_from_facts(GridAxisKind::Row, rows, facts.clone());
        }
        rows
    };

    Ok(GridNamedContext {
        columns,
        rows,
        area_facts: merged_area_facts,
    })
}

fn inherited_subgrid_axis_named_lines<S: LayoutScalar>(
    axis: GridAxisKind,
    parent_axis: &InheritedGridAxis<S>,
    components: &[TrackComponentOf<S>],
    parent_area_facts: Option<&GridAreaNameFacts>,
) -> Result<NamedGridLines, NamedGridError> {
    let used_track_count = parent_axis.tracks.len();
    let local_line_names = expand_subgrid_local_line_names(
        axis,
        used_track_count,
        subgrid_line_name_components(components).unwrap_or(&[]),
    )?;
    let mut lines = inherit_subgrid_named_lines(
        &parent_axis.named_lines,
        parent_axis.parent_start + 1,
        parent_axis.parent_end + 1,
        parent_axis.reversed,
        &local_line_names,
        parent_area_facts,
    )?;
    lines.axis = axis;
    Ok(lines)
}

fn subgrid_line_name_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Option<&[SubgridLineNameComponent]> {
    components.iter().find_map(|component| match component {
        TrackComponentOf::Subgrid(subgrid) => Some(subgrid.name_components.as_slice()),
        TrackComponentOf::LineNames(_)
        | TrackComponentOf::Track(_)
        | TrackComponentOf::Repeat(_) => None,
    })
}

pub(super) fn named_lines_from_track_components<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
    explicit_track_count: usize,
) -> Result<NamedGridLines, NamedGridError> {
    validate_track_component_line_names(components)?;

    if components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
    {
        return Ok(NamedGridLines::new(axis, explicit_track_count));
    }

    let mut lines = NamedGridLines::new(axis, explicit_track_count);
    let mut current_line = 0;
    append_track_component_names(
        &mut lines,
        components,
        explicit_track_count,
        &mut current_line,
    )?;
    lines.ensure_track_count(current_line);
    Ok(lines)
}

pub(super) fn subgrid_automatic_span<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
) -> Option<GridSpan> {
    let local_names = subgrid_line_name_components(components)?;
    validate_subgrid_local_line_names(axis, local_names).ok()?;
    GridSpan::new(
        fixed_subgrid_slots_after(local_names)
            .saturating_sub(1)
            .max(1),
    )
}

fn validate_subgrid_local_line_names(
    axis: GridAxisKind,
    components: &[SubgridLineNameComponent],
) -> Result<(), NamedGridError> {
    let mut auto_fill_count = 0;

    for component in components {
        match component {
            SubgridLineNameComponent::LineNames(names) => validate_line_names(names)?,
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(count),
                line_name_sets,
            } => {
                if *count == 0 {
                    return Err(NamedGridError::ZeroRepeat { axis });
                }
                validate_line_name_sets(line_name_sets)?;
            }
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                line_name_sets,
            } => {
                auto_fill_count += 1;
                if auto_fill_count > 1 {
                    return Err(NamedGridError::MultipleAutoFillRepeats { axis });
                }
                validate_line_name_sets(line_name_sets)?;
            }
        }
    }
    Ok(())
}

pub(super) fn expand_subgrid_local_line_names(
    axis: GridAxisKind,
    used_track_count: usize,
    components: &[SubgridLineNameComponent],
) -> Result<Vec<Vec<LineNameEntry>>, NamedGridError> {
    let slot_count = used_track_count + 1;
    validate_subgrid_local_line_names(axis, components)?;

    let mut local_line_names = Vec::with_capacity(slot_count);
    for (index, component) in components.iter().enumerate() {
        let remaining = slot_count - local_line_names.len();
        if remaining == 0 {
            break;
        }
        match component {
            SubgridLineNameComponent::LineNames(names) => {
                local_line_names.push(subgrid_line_name_entries(names));
            }
            SubgridLineNameComponent::Repeat {
                count,
                line_name_sets,
            } => {
                let group_slots = line_name_sets.len();
                let used_slots = match count {
                    SubgridLineNameRepeatCount::Count(count) => {
                        remaining.min(count.saturating_mul(group_slots))
                    }
                    SubgridLineNameRepeatCount::AutoFill => {
                        let trailing = fixed_subgrid_slots_after(&components[index + 1..]);
                        let available = remaining.saturating_sub(trailing);
                        available.checked_div(group_slots).unwrap_or(0) * group_slots
                    }
                };
                local_line_names.extend(
                    line_name_sets
                        .iter()
                        .cycle()
                        .take(used_slots)
                        .map(|names| subgrid_line_name_entries(names)),
                );
            }
        }
    }

    local_line_names.resize_with(slot_count, Vec::new);
    Ok(local_line_names)
}

pub(super) fn inherit_subgrid_named_lines(
    parent: &NamedGridLines,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
    local_line_names: &[Vec<LineNameEntry>],
    parent_area_facts: Option<&GridAreaNameFacts>,
) -> Result<NamedGridLines, NamedGridError> {
    let span_len = validate_subgrid_parent_span(parent, parent_start, parent_end)?;
    if local_line_names.len() != span_len + 1 {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis: parent.axis });
    }
    for entries in local_line_names {
        for entry in entries {
            validate_line_name(&entry.name)?;
        }
    }

    let mut lines = NamedGridLines::new(parent.axis, span_len);
    let explicit_start = parent.used_frame().explicit_start;
    for local_line in 0..=span_len {
        let parent_line = subgrid_parent_line(parent_start, parent_end, local_line, reversed);
        lines.line_names[local_line].extend(
            (parent_line - 1)
                .checked_sub(explicit_start)
                .and_then(|line| parent.line_names.get(line))
                .into_iter()
                .flatten()
                .filter(|entry| entry.origin != LineNameOrigin::AreaGenerated)
                .map(|entry| LineNameEntry {
                    name: entry.name.clone(),
                    origin: LineNameOrigin::Inherited,
                }),
        );
    }

    if let Some(facts) = parent_area_facts {
        add_clipped_subgrid_area_names(
            &mut lines,
            parent_start,
            parent_end,
            explicit_start,
            reversed,
            facts,
        );
    }

    for (line_index, entries) in local_line_names.iter().enumerate() {
        lines.line_names[line_index].extend(entries.iter().cloned());
    }

    Ok(lines)
}

fn inherited_subgrid_area_facts<S: LayoutScalar>(
    parent_context: &GridParentContext<S>,
) -> Option<GridAreaNameFacts> {
    let source = parent_context
        .columns
        .as_ref()
        .and_then(|axis| axis.area_facts.as_ref())
        .or_else(|| {
            parent_context
                .rows
                .as_ref()
                .and_then(|axis| axis.area_facts.as_ref())
        })?;
    let facts = clip_subgrid_area_facts(
        source,
        parent_context.columns.as_ref(),
        parent_context.rows.as_ref(),
    );
    (facts.columns_valid || facts.rows_valid).then_some(facts)
}

fn merge_subgrid_area_facts<S: LayoutScalar>(
    local: Option<GridAreaNameFacts>,
    inherited: Option<GridAreaNameFacts>,
    parent_context: &GridParentContext<S>,
) -> Option<GridAreaNameFacts> {
    match (local, inherited) {
        (None, None) => None,
        (Some(local), None) => Some(local),
        (None, Some(inherited)) => Some(inherited),
        (Some(mut local), Some(inherited)) => {
            let mut inherited_rectangles = inherited.area_rectangles;
            for name in inherited.area_order {
                if let Some(local_index) = local
                    .area_order
                    .iter()
                    .position(|local_name| local_name == &name)
                {
                    let Some(inherited_index) = inherited_rectangles
                        .iter()
                        .position(|rectangle| rectangle.name == name)
                    else {
                        continue;
                    };
                    merge_duplicate_subgrid_area_rectangle(
                        &mut local.area_rectangles[local_index],
                        &inherited_rectangles[inherited_index],
                        parent_context,
                    );
                    continue;
                }
                let Some(rectangle_index) = inherited_rectangles
                    .iter()
                    .position(|rectangle| rectangle.name == name)
                else {
                    continue;
                };
                local.area_order.push(name);
                local
                    .area_rectangles
                    .push(inherited_rectangles.remove(rectangle_index));
            }

            local.row_count = local.row_count.max(inherited.row_count);
            local.column_count = local.column_count.max(inherited.column_count);
            local.rows_valid |= inherited.rows_valid;
            local.columns_valid |= inherited.columns_valid;
            Some(local)
        }
    }
}

fn clamp_style_area_facts_to_subgrid_axes<S: LayoutScalar>(
    facts: Option<GridAreaNameFacts>,
    parent_context: &GridParentContext<S>,
) -> Option<GridAreaNameFacts> {
    let mut facts = facts?;

    let column_boundary = parent_context
        .columns
        .as_ref()
        .map(|axis| axis.tracks.len() + 1);
    let row_boundary = parent_context
        .rows
        .as_ref()
        .map(|axis| axis.tracks.len() + 1);

    if column_boundary.is_none() && row_boundary.is_none() {
        return Some(facts);
    }

    for rectangle in &mut facts.area_rectangles {
        if let Some(boundary) = column_boundary {
            clamp_area_axis_to_local_boundary(
                &mut rectangle.column_start,
                &mut rectangle.column_end,
                &mut rectangle.column_start_name,
                &mut rectangle.column_end_name,
                boundary,
            );
        }
        if let Some(boundary) = row_boundary {
            clamp_area_axis_to_local_boundary(
                &mut rectangle.row_start,
                &mut rectangle.row_end,
                &mut rectangle.row_start_name,
                &mut rectangle.row_end_name,
                boundary,
            );
        }
    }

    if let Some(boundary) = column_boundary {
        facts.column_count = facts.column_count.min(boundary.saturating_sub(1));
    }
    if let Some(boundary) = row_boundary {
        facts.row_count = facts.row_count.min(boundary.saturating_sub(1));
    }

    Some(facts)
}

fn clamp_area_axis_to_local_boundary(
    start: &mut usize,
    end: &mut usize,
    start_name: &mut usize,
    end_name: &mut usize,
    boundary: usize,
) {
    *start = (*start).clamp(1, boundary);
    *end = (*end).clamp(1, boundary);
    *start_name = (*start_name).clamp(1, boundary);
    *end_name = (*end_name).clamp(1, boundary);
}

fn merge_duplicate_subgrid_area_rectangle<S: LayoutScalar>(
    local: &mut GridAreaNameRectangle,
    inherited: &GridAreaNameRectangle,
    parent_context: &GridParentContext<S>,
) {
    if parent_context.columns.is_some() {
        local.column_start = local.column_start.min(inherited.column_start);
        local.column_end = local.column_end.min(inherited.column_end);
        local.column_start_name = local.column_start_name.min(inherited.column_start_name);
        local.column_end_name = local.column_end_name.min(inherited.column_end_name);
    }
    if parent_context.rows.is_some() {
        local.row_start = local.row_start.min(inherited.row_start);
        local.row_end = local.row_end.min(inherited.row_end);
        local.row_start_name = local.row_start_name.min(inherited.row_start_name);
        local.row_end_name = local.row_end_name.min(inherited.row_end_name);
    }
}

fn clip_subgrid_area_facts<S: LayoutScalar>(
    facts: &GridAreaNameFacts,
    columns: Option<&InheritedGridAxis<S>>,
    rows: Option<&InheritedGridAxis<S>>,
) -> GridAreaNameFacts {
    let columns_valid = columns.is_some_and(|axis| facts.is_valid_for_axis(axis.named_lines.axis));
    let rows_valid = rows.is_some_and(|axis| facts.is_valid_for_axis(axis.named_lines.axis));
    let column_count = columns
        .map(|axis| axis.parent_end.saturating_sub(axis.parent_start))
        .unwrap_or(0);
    let row_count = rows
        .map(|axis| axis.parent_end.saturating_sub(axis.parent_start))
        .unwrap_or(0);
    let mut area_order = Vec::new();
    let mut area_rectangles = Vec::new();

    for name in &facts.area_order {
        let Some(rectangle) = facts
            .area_rectangles
            .iter()
            .find(|rectangle| rectangle.name == *name)
        else {
            continue;
        };
        let Some(column) = clip_area_axis_lines(rectangle, columns_valid, columns) else {
            continue;
        };
        let Some(row) = clip_area_axis_lines(rectangle, rows_valid, rows) else {
            continue;
        };
        area_order.push(name.clone());
        area_rectangles.push(GridAreaNameRectangle {
            name: name.clone(),
            row_start: row.start,
            row_end: row.end,
            column_start: column.start,
            column_end: column.end,
            row_start_name: row.start_name,
            row_end_name: row.end_name,
            column_start_name: column.start_name,
            column_end_name: column.end_name,
        });
    }

    GridAreaNameFacts {
        row_count,
        column_count,
        rows_valid,
        columns_valid,
        area_order,
        area_rectangles,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClippedAreaAxisLines {
    start: usize,
    end: usize,
    start_name: usize,
    end_name: usize,
}

fn clip_area_axis_lines<S: LayoutScalar>(
    rectangle: &GridAreaNameRectangle,
    axis_valid: bool,
    inherited: Option<&InheritedGridAxis<S>>,
) -> Option<ClippedAreaAxisLines> {
    let Some(inherited) = inherited.filter(|_| axis_valid) else {
        return Some(ClippedAreaAxisLines {
            start: 1,
            end: 1,
            start_name: 1,
            end_name: 1,
        });
    };
    project_area_axis_lines(
        rectangle,
        inherited.named_lines.axis,
        inherited.named_lines.used_frame().explicit_start,
        inherited.parent_start + 1,
        inherited.parent_end + 1,
        inherited.reversed,
    )
}

fn project_area_axis_lines(
    rectangle: &GridAreaNameRectangle,
    axis: GridAxisKind,
    explicit_start: usize,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
) -> Option<ClippedAreaAxisLines> {
    let (start, end, start_name, end_name) = match axis {
        GridAxisKind::Column => (
            rectangle.column_start,
            rectangle.column_end,
            rectangle.column_start_name,
            rectangle.column_end_name,
        ),
        GridAxisKind::Row => (
            rectangle.row_start,
            rectangle.row_end,
            rectangle.row_start_name,
            rectangle.row_end_name,
        ),
    };
    let clipped_start = (start + explicit_start).max(parent_start);
    let clipped_end = (end + explicit_start).min(parent_end);
    if clipped_end <= clipped_start {
        return None;
    }

    let to_local = |line| subgrid_local_line_index(parent_start, parent_end, line, reversed) + 1;
    let start = to_local(clipped_start);
    let end = to_local(clipped_end);
    let local_start_name = to_local(clipped_semantic_area_edge(
        clipped_start,
        clipped_end,
        start_name,
        end_name,
        true,
    ));
    let local_end_name = to_local(clipped_semantic_area_edge(
        clipped_start,
        clipped_end,
        start_name,
        end_name,
        false,
    ));
    Some(ClippedAreaAxisLines {
        start: start.min(end),
        end: start.max(end),
        start_name: local_start_name,
        end_name: local_end_name,
    })
}

#[cfg(test)]
pub(super) fn add_area_generated_lines(
    axis: GridAxisKind,
    base: NamedGridLines,
    areas: &GridTemplateAreas,
) -> Result<NamedGridLines, NamedGridError> {
    if areas.rows.is_empty() {
        return Ok(base);
    }

    let facts = GridAreaNameFacts::from_specified_areas(areas)?;
    Ok(add_area_generated_lines_from_facts(axis, base, facts))
}

fn add_area_generated_lines_from_facts(
    axis: GridAxisKind,
    mut base: NamedGridLines,
    facts: GridAreaNameFacts,
) -> NamedGridLines {
    if !facts.is_valid_for_axis(axis) {
        base.area_facts = facts;
        return base;
    }
    let area_track_count = match axis {
        GridAxisKind::Column => facts.column_count,
        GridAxisKind::Row => facts.row_count,
    };
    base.ensure_track_count(area_track_count);

    for rectangle in &facts.area_rectangles {
        let (start_line, end_line) = match axis {
            GridAxisKind::Column => (rectangle.column_start_name, rectangle.column_end_name),
            GridAxisKind::Row => (rectangle.row_start_name, rectangle.row_end_name),
        };
        base.add_area_name(start_line - 1, format!("{}-start", rectangle.name));
        base.add_area_name(end_line - 1, format!("{}-end", rectangle.name));
    }

    base.area_facts = facts;
    base
}

fn append_track_component_names<S: LayoutScalar>(
    lines: &mut NamedGridLines,
    components: &[TrackComponentOf<S>],
    explicit_track_count: usize,
    current_line: &mut usize,
) -> Result<(), NamedGridError> {
    let auto_repeat_count =
        auto_repeat_expansion_count(lines.axis, components, explicit_track_count)?;

    for component in components {
        match component {
            TrackComponentOf::LineNames(names) => {
                lines.add_line_names(*current_line, names, LineNameOrigin::Explicit);
            }
            TrackComponentOf::Track(_) => {
                *current_line += 1;
                lines.ensure_track_count(*current_line);
            }
            TrackComponentOf::Repeat(repetition) => {
                let repeated_track_count = fixed_track_count(lines.axis, repetition.components())?;
                let count = match repetition.repeat() {
                    TrackRepeat::Count(count) => count.get(),
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => auto_repeat_count.unwrap_or(0),
                };
                // With no sizing tracks, every copy supplies the same names
                // to the same line. A name selects that line once.
                let count = if repeated_track_count == 0 {
                    count.min(1)
                } else {
                    count
                };
                for _ in 0..count {
                    append_track_component_names(
                        lines,
                        repetition.components(),
                        repeated_track_count,
                        current_line,
                    )?;
                }
            }
            TrackComponentOf::Subgrid(_) => {}
        }
    }

    Ok(())
}

fn validate_track_component_line_names<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Result<(), NamedGridError> {
    for component in components {
        match component {
            TrackComponentOf::LineNames(names) => validate_line_names(names)?,
            TrackComponentOf::Repeat(repetition) => {
                validate_track_component_line_names(repetition.components())?;
            }
            TrackComponentOf::Track(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Ok(())
}

fn validate_line_names(names: &[String]) -> Result<(), NamedGridError> {
    for name in names {
        if matches!(name.as_str(), "auto" | "span") {
            return Err(NamedGridError::ReservedLineName { name: name.clone() });
        }
    }
    Ok(())
}

fn validate_line_name_sets(line_name_sets: &[Vec<String>]) -> Result<(), NamedGridError> {
    for names in line_name_sets {
        validate_line_names(names)?;
    }
    Ok(())
}

fn subgrid_line_name_entries(names: &[String]) -> Vec<LineNameEntry> {
    names
        .iter()
        .cloned()
        .map(|name| LineNameEntry {
            name,
            origin: LineNameOrigin::LocalSubgrid,
        })
        .collect()
}

fn fixed_subgrid_slots_after(components: &[SubgridLineNameComponent]) -> usize {
    components
        .iter()
        .map(|component| match component {
            SubgridLineNameComponent::LineNames(_) => 1,
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::Count(count),
                line_name_sets,
            } => count.saturating_mul(line_name_sets.len()),
            SubgridLineNameComponent::Repeat {
                count: SubgridLineNameRepeatCount::AutoFill,
                ..
            } => 0,
        })
        .fold(0, usize::saturating_add)
}

fn validate_subgrid_parent_span(
    parent: &NamedGridLines,
    parent_start: usize,
    parent_end: usize,
) -> Result<usize, NamedGridError> {
    if parent_start == 0
        || parent_end <= parent_start
        || parent_end > parent.used_frame().track_count + 1
    {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis: parent.axis });
    }
    Ok(parent_end - parent_start)
}

fn subgrid_parent_line(
    parent_start: usize,
    parent_end: usize,
    local_line: usize,
    reversed: bool,
) -> usize {
    if reversed {
        parent_end - local_line
    } else {
        parent_start + local_line
    }
}

fn add_clipped_subgrid_area_names(
    lines: &mut NamedGridLines,
    parent_start: usize,
    parent_end: usize,
    explicit_start: usize,
    reversed: bool,
    facts: &GridAreaNameFacts,
) {
    if !facts.is_valid_for_axis(lines.axis) {
        return;
    }
    for area in &facts.area_order {
        let Some(rectangle) = facts
            .area_rectangles
            .iter()
            .find(|rectangle| rectangle.name == *area)
        else {
            continue;
        };
        let Some(projected) = project_area_axis_lines(
            rectangle,
            lines.axis,
            explicit_start,
            parent_start,
            parent_end,
            reversed,
        ) else {
            continue;
        };
        lines.line_names[projected.start_name - 1].push(LineNameEntry {
            name: format!("{area}-start"),
            origin: LineNameOrigin::AreaGenerated,
        });
        lines.line_names[projected.end_name - 1].push(LineNameEntry {
            name: format!("{area}-end"),
            origin: LineNameOrigin::AreaGenerated,
        });
    }
}

fn clipped_semantic_area_edge(
    clipped_start: usize,
    clipped_end: usize,
    area_start_name: usize,
    area_end_name: usize,
    start_edge: bool,
) -> usize {
    let forward = area_start_name <= area_end_name;
    match (forward, start_edge) {
        (true, true) | (false, false) => clipped_start,
        (true, false) | (false, true) => clipped_end,
    }
}

fn subgrid_local_line_index(
    parent_start: usize,
    parent_end: usize,
    parent_line: usize,
    reversed: bool,
) -> usize {
    if reversed {
        parent_end - parent_line
    } else {
        parent_line - parent_start
    }
}

fn auto_repeat_expansion_count<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
    explicit_track_count: usize,
) -> Result<Option<usize>, NamedGridError> {
    let mut fixed_tracks = 0;
    let mut auto_repeated_tracks = None;

    for component in components {
        match component {
            TrackComponentOf::Track(_) => fixed_tracks += 1,
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
            TrackComponentOf::Repeat(repetition) => match repetition.repeat() {
                TrackRepeat::Count(count) => {
                    fixed_tracks += fixed_track_count(axis, repetition.components())? * count.get();
                }
                TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                    if auto_repeated_tracks.is_some() {
                        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
                    }
                    auto_repeated_tracks = Some(fixed_track_count(axis, repetition.components())?);
                }
            },
        }
    }

    let Some(auto_repeated_tracks) = auto_repeated_tracks else {
        return Ok(None);
    };
    if auto_repeated_tracks == 0 || explicit_track_count < fixed_tracks {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
    }

    let remaining_tracks = explicit_track_count - fixed_tracks;
    if !remaining_tracks.is_multiple_of(auto_repeated_tracks) {
        return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
    }

    Ok(Some(remaining_tracks / auto_repeated_tracks))
}

fn fixed_track_count<S: LayoutScalar>(
    axis: GridAxisKind,
    components: &[TrackComponentOf<S>],
) -> Result<usize, NamedGridError> {
    let mut count = 0;
    for component in components {
        match component {
            TrackComponentOf::Track(_) => count += 1,
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
            TrackComponentOf::Repeat(repetition) => match repetition.repeat() {
                TrackRepeat::Count(repeat_count) => {
                    count += fixed_track_count(axis, repetition.components())? * repeat_count.get();
                }
                TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                    return Err(NamedGridError::UnresolvedAutoRepeatNames { axis });
                }
            },
        }
    }
    Ok(count)
}
