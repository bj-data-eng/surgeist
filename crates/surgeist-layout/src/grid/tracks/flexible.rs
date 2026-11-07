use super::ordinary::{GridAxisTrackPhases, OrdinaryTrackState, SpanningContributionError};
use super::*;

use crate::geometry::FlowAxes;
use crate::grid::input::GridOverflowFacts;
use crate::scalar::{from_nonnegative_finite_rank, nonnegative_finite_rank};

pub(in crate::grid) fn grid_axis_computed_overflow(
    style: &impl GridOverflowFacts,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> Overflow {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => style.computed_overflow().x(),
        crate::geometry::PhysicalAxis::Vertical => style.computed_overflow().y(),
    }
}

pub(in crate::grid) fn grid_axis_physical_axis(
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> crate::geometry::PhysicalAxis {
    match axis.logical_axis() {
        crate::LogicalAxis::Inline => flow_axes.inline_axis(),
        crate::LogicalAxis::Block => flow_axes.block_axis(),
    }
}

pub(in crate::grid) fn grid_axis_size<T>(
    flow_axes: FlowAxes,
    size: Size<T>,
    axis: GridAxisKind,
) -> T {
    match grid_axis_physical_axis(flow_axes, axis) {
        crate::geometry::PhysicalAxis::Horizontal => size.width,
        crate::geometry::PhysicalAxis::Vertical => size.height,
    }
}

fn flex_number<S: LayoutScalar>(value: S) -> Result<S, SpanningContributionError<S>> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(SpanningContributionError::InvalidNumeric { value })
    }
}

// Operation-local freezing: each restart freezes at least one additional track.
// Zero-factor tracks consume their base and never evaluate 0 * infinity.
fn find_size_of_fr<S: LayoutScalar>(
    states: &mut [OrdinaryTrackState<'_, S>],
    space: S,
) -> Result<S, SpanningContributionError<S>> {
    for state in states.iter_mut() {
        state.fr_frozen = false;
    }
    loop {
        let mut used = S::ZERO;
        let mut factors = S::ZERO;
        for state in states.iter() {
            match state.flex_factor {
                Some(factor) if factor > S::ZERO && !state.fr_frozen => {
                    factors = flex_number(factors + factor)?;
                }
                _ => used = flex_number(used + state.base_size)?,
            }
        }
        let fraction = flex_number((space - used).max(S::ZERO) / factors.max(S::ONE))?;
        let mut newly_frozen = false;
        for state in states.iter_mut() {
            if let Some(factor) = state.flex_factor
                && factor > S::ZERO
                && !state.fr_frozen
                && factor * fraction < state.base_size
            {
                state.fr_frozen = true;
                newly_frozen = true;
            }
        }
        if !newly_frozen {
            return Ok(fraction);
        }
    }
}

// Replay the actual products and original ordered span/gutter fold. Overflow
// here only rejects a trial; a lower finite represented fraction may still fit.
fn flex_span_fold<S: LayoutScalar>(
    states: &[OrdinaryTrackState<'_, S>],
    fraction: S,
    gap_total: S,
) -> Option<S> {
    let mut sum = S::ZERO;
    for state in states {
        let value = match state.flex_factor {
            Some(factor) => {
                let product = factor * fraction;
                if !product.is_finite() {
                    return None;
                }
                state.base_size.max(product)
            }
            None => state.base_size,
        };
        sum = sum + value;
        if !sum.is_finite() {
            return None;
        }
    }
    let total = sum + gap_total;
    total.is_finite().then_some(total)
}

fn feasible_flex_fraction<S: LayoutScalar>(
    states: &[OrdinaryTrackState<'_, S>],
    nominal: S,
    gap_total: S,
    budget: S,
) -> S {
    // No free space admits no additional expansion, including on an aggregate
    // plateau. Existing floors remain intact through max(base, factor * 0).
    if flex_span_fold(states, S::ZERO, gap_total).is_none_or(|sum| sum >= budget) {
        return S::ZERO;
    }
    if flex_span_fold(states, nominal, gap_total).is_some_and(|sum| sum <= budget) {
        return nominal;
    }
    let mut low = 0;
    let mut high = nonnegative_finite_rank(nominal);
    // Positive finite IEEE ranks are monotone. This takes at most 32/64 probes;
    // it does not depend on the magnitude or spacing of the numeric interval.
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        let value = from_nonnegative_finite_rank(middle);
        if flex_span_fold(states, value, gap_total).is_some_and(|sum| sum <= budget) {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    from_nonnegative_finite_rank(low)
}

fn resolve_flex_fraction<S: LayoutScalar>(
    states: &mut [OrdinaryTrackState<'_, S>],
    items: &[GridIntrinsicContributionRow<S>],
    available: GridTrackSizingConstraint<S>,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Result<S, SpanningContributionError<S>> {
    if !states.iter().any(|state| state.flex_factor.is_some())
        || (available.operation == SizingOperation::MinContent
            && available.available_extent.is_none())
    {
        return Ok(S::ZERO);
    }
    let span_gap = |start, end| {
        gutters.map_or_else(
            || gap * S::from_usize(end - start - 1),
            |gutters| gutters.span_gap_total(start, end),
        )
    };
    if let Some(budget) = available.available_extent {
        let gaps = flex_number(span_gap(0, states.len()))?;
        let occupied = flex_span_fold(states, S::ZERO, gaps)
            .ok_or(SpanningContributionError::InvalidNumeric { value: S::INFINITY })?;
        if occupied >= budget {
            return Ok(S::ZERO);
        }
        let nominal = find_size_of_fr(states, flex_number(budget - gaps)?.max(S::ZERO))?;
        return Ok(feasible_flex_fraction(states, nominal, gaps, budget));
    }
    let mut fraction = S::ZERO;
    for state in states.iter() {
        if let Some(factor) = state.flex_factor {
            let candidate = if factor > S::ONE {
                state.base_size / factor
            } else {
                state.base_size
            };
            fraction = fraction.max(flex_number(candidate)?);
        }
    }
    // Every candidate comes from an item span in the current owning
    // frame. A candidate from another item may legitimately exceed this demand.
    for item in items {
        let start = item.span.start - 1;
        let end = item.span.end - 1;
        let span = &mut states[start..end];
        if !span.iter().any(|state| state.flex_factor.is_some()) {
            continue;
        }
        let gaps = flex_number(span_gap(start, end))?;
        // Signed outer contributions remain authentic input. This operation
        // consumes only their nonnegative sizing demand.
        let budget = item.contributions.max_content.max(S::ZERO);
        let nominal = find_size_of_fr(span, flex_number(budget - gaps)?.max(S::ZERO))?;
        fraction = fraction.max(feasible_flex_fraction(span, nominal, gaps, budget));
    }
    Ok(fraction)
}

pub(in crate::grid) fn track_flex_factor<S: LayoutScalar>(track: &TrackSizingOf<S>) -> Option<S> {
    if let MaxTrackSizingOf::Flex(value) = &track.max {
        Some(value.get())
    } else {
        None
    }
}

fn track_has_auto_maximum<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    matches!(track.max, MaxTrackSizingOf::Auto)
}

fn maximize_tracks<S: LayoutScalar>(
    states: &mut [OrdinaryTrackState<'_, S>],
    available: GridTrackSizingConstraint<S>,
    gap_total: S,
) {
    if available.operation == SizingOperation::MinContent && available.available_extent.is_none() {
        return;
    }
    let Some(budget) = available.available_extent else {
        for state in states {
            if let Some(limit) = state.growth_limit {
                state.base_size = state.base_size.max(limit);
            }
        }
        return;
    };
    let mut remaining = (budget
        - gap_total
        - states
            .iter()
            .map(|state| state.base_size)
            .fold(S::ZERO, |sum, size| sum + size))
    .max(S::ZERO);
    while remaining > S::ZERO {
        let eligible = states
            .iter()
            .filter(|state| {
                !state.collapsed
                    && state.flex_factor.is_none()
                    && state
                        .growth_limit
                        .is_some_and(|limit| limit > state.base_size)
            })
            .count();
        if eligible == 0 {
            break;
        }
        let increment = remaining / S::from_usize(eligible);
        let mut used = S::ZERO;
        for state in states.iter_mut() {
            if !state.collapsed
                && state.flex_factor.is_none()
                && let Some(limit) = state.growth_limit
                && limit > state.base_size
            {
                let next = (state.base_size + increment).min(limit);
                used = used + (next - state.base_size);
                state.base_size = next;
            }
        }
        let next = (remaining - used).max(S::ZERO);
        if next >= remaining {
            break;
        }
        remaining = next;
    }
}

fn finish_ordinary_track_phases<S: LayoutScalar>(
    mut states: Vec<OrdinaryTrackState<'_, S>>,
    input: ResolvedAxisTrackInput<'_, S>,
) -> Result<Vec<S>, SpanningContributionError<S>> {
    let ResolvedAxisTrackInput {
        items,
        available,
        flex_bounds,
        stretch_size,
        gap,
        gutters,
        alignment,
        ..
    } = input;
    let gap_total = flex_number(gutters.map_or_else(
        || gap * S::from_usize(states.len().saturating_sub(1)),
        OrdinaryGridAxisGuttersOf::active_gap_total,
    ))?;
    let GridTrackFinishingBounds { minimum, maximum } = flex_bounds;
    // A maximum limits maximization without becoming the initial flex space.
    let maximize_available = GridTrackSizingConstraint {
        available_extent: if available.operation == SizingOperation::MaxContent {
            maximum.or(available.available_extent)
        } else {
            available.available_extent
        },
        ..available
    };
    maximize_tracks(&mut states, maximize_available, gap_total);
    let mut fraction = resolve_flex_fraction(&mut states, items, available, gap, gutters)?;
    let bound = if minimum.is_some() || maximum.is_some() {
        let extent = flex_span_fold(&states, fraction, gap_total);
        // An overflowing nonnegative sum exceeds every finite maximum.
        minimum
            .filter(|minimum| extent.is_some_and(|extent| extent < *minimum))
            .or_else(|| maximum.filter(|maximum| extent.is_none_or(|extent| extent > *maximum)))
    } else {
        None
    };
    if let Some(bound) = bound {
        fraction = resolve_flex_fraction(
            &mut states,
            items,
            GridTrackSizingConstraint {
                available_extent: Some(bound),
                ..available
            },
            gap,
            gutters,
        )?;
    }
    let mut used = S::ZERO;
    for state in &mut states {
        if let Some(factor) = state.flex_factor {
            state.base_size = state.base_size.max(flex_number(factor * fraction)?);
        }
        used = flex_number(used + state.base_size)?;
    }
    let auto_count = states
        .iter()
        .filter(|s| s.auto_max_stretch_eligible && !s.collapsed)
        .count();
    let auto_size = if alignment == AlignContent::Stretch && auto_count > 0 {
        match stretch_size {
            Some(size) => {
                flex_number((size - gap_total - used).max(S::ZERO) / S::from_usize(auto_count))?
            }
            None => S::ZERO,
        }
    } else {
        S::ZERO
    };
    let mut result = Vec::new();
    result
        .try_reserve_exact(states.len())
        .map_err(|_| SpanningContributionError::Allocation)?;
    for state in states {
        result.push(flex_number(
            state.base_size
                + if state.auto_max_stretch_eligible {
                    auto_size
                } else {
                    S::ZERO
                },
        )?);
    }
    Ok(result)
}

#[cfg(test)]
pub(in crate::grid) fn track_sum<S: LayoutScalar>(sizes: &[S], gap: S) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gap * S::from_usize(sizes.len().saturating_sub(1))
}

pub(in crate::grid) fn track_sum_with_gutters<S: LayoutScalar>(
    sizes: &[S],
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    sizes
        .iter()
        .copied()
        .fold(S::ZERO, |sum, value| sum + value)
        + gutters.map_or_else(
            || gap * S::from_usize(sizes.len().saturating_sub(1)),
            OrdinaryGridAxisGuttersOf::active_gap_total,
        )
}

pub(in crate::grid) fn track_span_sum_with_gutters<S: LayoutScalar>(
    sizes: &[S],
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    track_span_sum_with_boundary_gaps(
        sizes,
        start,
        end,
        gap,
        gutters.map(OrdinaryGridAxisGuttersOf::gutter_after),
    )
}

pub(in crate::grid) fn track_span_sum_with_boundary_gaps<S: LayoutScalar>(
    sizes: &[S],
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&[S]>,
) -> S {
    if start >= end || end > sizes.len() {
        return S::ZERO;
    }
    sizes[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, size| sum + size)
        + gutters.map_or_else(
            || gap * S::from_usize(end.saturating_sub(start + 1)),
            |gutters| {
                gutters
                    .get(start..end - 1)
                    .unwrap_or_default()
                    .iter()
                    .copied()
                    .fold(S::ZERO, |sum, gutter| sum + gutter)
            },
        )
}

pub(super) fn span_contribution_with_gutters<S: LayoutScalar>(
    contribution: S,
    start: usize,
    end: usize,
    gap: S,
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> S {
    let gutter_total = gutters.map_or_else(
        || gap * S::from_usize(end.saturating_sub(start + 1)),
        |gutters| gutters.span_gap_total(start, end),
    );
    (contribution - gutter_total).max(S::ZERO)
}

pub(in crate::grid) fn offsets<S: LayoutScalar>(sizes: &[S], start: S, gap: S) -> Vec<S> {
    let mut cursor = start;
    sizes
        .iter()
        .map(|size| {
            let offset = cursor;
            cursor = cursor + *size + gap;
            offset
        })
        .collect()
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct GridTrackFinishingBounds<S: LayoutScalar> {
    pub(in crate::grid) minimum: Option<S>,
    pub(in crate::grid) maximum: Option<S>,
}

impl<S: LayoutScalar> GridTrackFinishingBounds<S> {
    pub(in crate::grid) const NONE: Self = Self {
        minimum: None,
        maximum: None,
    };
}

// Explicit resolved phases are mandatory here; fixed minima are not reapplied.
pub(in crate::grid) struct ResolvedAxisTrackInput<'a, S: LayoutScalar> {
    pub(in crate::grid) tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) phases: &'a GridAxisTrackPhases<S>,
    pub(in crate::grid) items: &'a [GridIntrinsicContributionRow<S>],
    pub(in crate::grid) available: GridTrackSizingConstraint<S>,
    pub(in crate::grid) flex_bounds: GridTrackFinishingBounds<S>,
    pub(in crate::grid) stretch_size: Option<S>,
    pub(in crate::grid) gap: S,
    pub(in crate::grid) alignment: AlignContent,
    pub(in crate::grid) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
}

pub(in crate::grid) fn resolve_tracks_from_phases<S: LayoutScalar>(
    input: ResolvedAxisTrackInput<'_, S>,
) -> Result<Vec<S>, SpanningContributionError<S>> {
    let ResolvedAxisTrackInput {
        tracks,
        phases,
        items,
        available,
        flex_bounds,
        stretch_size,
        gap,
        gutters,
        ..
    } = input;
    if phases.base_sizes.len() != tracks.len() || phases.growth_limits.len() != tracks.len() {
        return Err(SpanningContributionError::InvalidIncidence);
    }
    let gap_total = gutters.map_or_else(
        || gap * S::from_usize(tracks.len().saturating_sub(1)),
        OrdinaryGridAxisGuttersOf::active_gap_total,
    );
    if !gap_total.is_finite() {
        return Err(SpanningContributionError::InvalidNumeric { value: gap_total });
    }
    if !gap.is_finite() || gap < S::ZERO {
        return Err(SpanningContributionError::InvalidNumeric { value: gap });
    }
    for value in [
        available.available_extent,
        flex_bounds.minimum,
        flex_bounds.maximum,
        stretch_size,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() || value < S::ZERO {
            return Err(SpanningContributionError::InvalidNumeric { value });
        }
    }
    for item in items {
        if item.span.start == 0
            || item.span.end <= item.span.start
            || item.span.end - 1 > tracks.len()
        {
            return Err(SpanningContributionError::InvalidIncidence);
        }
        for value in [
            item.contributions.minimum,
            item.contributions.min_content,
            item.contributions.max_content,
        ] {
            if !value.is_finite() {
                return Err(SpanningContributionError::InvalidNumeric { value });
            }
        }
    }
    let mut states = Vec::new();
    states
        .try_reserve_exact(tracks.len())
        .map_err(|_| SpanningContributionError::Allocation)?;
    for (index, ((track, base), growth)) in tracks
        .iter()
        .zip(phases.base_sizes.iter().copied())
        .zip(phases.growth_limits.iter().copied())
        .enumerate()
    {
        if !base.is_finite() || base < S::ZERO {
            return Err(SpanningContributionError::InvalidNumeric { value: base });
        }
        if let Some(value) = growth {
            if !value.is_finite() {
                return Err(SpanningContributionError::InvalidNumeric { value });
            }
            if value < base {
                return Err(SpanningContributionError::InvalidIncidence);
            }
        }
        let collapsed = gutters
            .and_then(|g| g.collapsed().get(index))
            .copied()
            .unwrap_or(false);
        if collapsed && base != S::ZERO {
            return Err(SpanningContributionError::InvalidIncidence);
        }
        let mut state = OrdinaryTrackState::new(track, track_has_auto_maximum(track), collapsed);
        state.base_size = base;
        state.growth_limit = growth;
        if collapsed {
            state.flex_factor = None;
            state.auto_max_stretch_eligible = false;
        }
        states.push(state);
    }
    let result = finish_ordinary_track_phases(states, input)?;
    let total = track_sum_with_gutters(&result, gap, gutters);
    if !total.is_finite() {
        return Err(SpanningContributionError::InvalidNumeric { value: total });
    }
    Ok(result)
}

// An explicitly definite source-prefix estimate has no issued item inventory.
// It cannot be used for indefinite/max-content resolution or published facts.
pub(in crate::grid) struct DefiniteAxisTrackEstimateInput<'a, S: LayoutScalar> {
    pub(in crate::grid) tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) basis: Option<S>,
    pub(in crate::grid) available: S,
    pub(in crate::grid) stretch_size: Option<S>,
    pub(in crate::grid) gap: S,
    pub(in crate::grid) alignment: AlignContent,
    pub(in crate::grid) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
}

pub(in crate::grid) fn estimate_definite_axis_tracks<S: LayoutScalar>(
    input: DefiniteAxisTrackEstimateInput<'_, S>,
) -> Result<Vec<S>, SpanningContributionError<S>> {
    let available = GridTrackSizingConstraint {
        operation: SizingOperation::Normal,
        available_extent: Some(input.available),
    };
    let phases = resolve_grid_contribution_rows(
        SpanningTrackContext {
            tracks: input.tracks,
            basis: input.basis,
            available,
            gap: input.gap,
            gutters: input.gutters,
            base_floors: &[],
        },
        &[],
    )?;
    resolve_tracks_from_phases(ResolvedAxisTrackInput {
        tracks: input.tracks,
        phases: &phases,
        items: &[],
        available,
        flex_bounds: GridTrackFinishingBounds::NONE,
        stretch_size: input.stretch_size,
        gap: input.gap,
        alignment: input.alignment,
        gutters: input.gutters,
    })
}

#[cfg(test)]
mod flex_item_tests {
    use super::*;
    use crate::TrackFlexFactorOf;

    fn flex<S: LayoutScalar>(factor: S) -> TrackSizingOf<S> {
        TrackSizingOf::minmax(
            MinTrackSizingOf::px(S::ZERO),
            MaxTrackSizingOf::flex(TrackFlexFactorOf::try_new(factor).unwrap()),
        )
    }
    fn item<S: LayoutScalar>(
        start: usize,
        end: usize,
        min: S,
        mc: S,
        mx: S,
    ) -> GridIntrinsicContributionRow<S> {
        GridIntrinsicContributionRow {
            span: GridTrackSpan::new(start + 1, end + 1),
            contributions: GridTrackContributions {
                minimum: min,
                min_content: mc,
                max_content: mx,
            },
        }
    }
    fn solve<S: LayoutScalar>(
        tracks: &[TrackSizingOf<S>],
        items: &[GridIntrinsicContributionRow<S>],
        available: GridTrackSizingConstraint<S>,
        floors: &[S],
        gap: S,
        gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
    ) -> Vec<S> {
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available,
                gap,
                gutters,
                base_floors: floors,
            },
            items,
        )
        .unwrap();
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items,
            available,
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: None,
            gap,
            alignment: AlignContent::Start,
            gutters,
        })
        .unwrap()
    }
    fn assert_flex_item_matrix<S: LayoutScalar>() {
        let n = S::from_f64;
        // §11.5 flex spans below1 distribute .2+.25 / .3+.25, not .4/.6.
        let tracks = [
            TrackSizingOf::minmax(
                MinTrackSizingOf::AUTO,
                MaxTrackSizingOf::flex(TrackFlexFactorOf::try_new(n(0.2)).unwrap()),
            ),
            TrackSizingOf::minmax(
                MinTrackSizingOf::AUTO,
                MaxTrackSizingOf::flex(TrackFlexFactorOf::try_new(n(0.3)).unwrap()),
            ),
        ];
        let rows = [item(0, 2, n(60.0), n(60.0), n(60.0))];
        let indefinite = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            &[],
            S::ZERO,
            None,
        );
        assert_eq!(indefinite, [n(27.0), n(33.0)]);
        assert_eq!(track_sum_with_gutters(&indefinite, S::ZERO, None), n(60.0));
        // Below-one factors leave real host space; it is not track content.
        let definite = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(n(200.0)),
            },
            &[],
            S::ZERO,
            None,
        );
        let products = [n(0.2) * n(200.0), n(0.3) * n(200.0)];
        assert_eq!(definite, products);
        assert_eq!(
            track_sum_with_gutters(&definite, S::ZERO, None),
            products[0] + products[1]
        );

        let tracks = [flex(n(1.0)), flex(n(2.0))];
        let rows = [item(0, 2, S::ZERO, n(30.0), n(90.0))];
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[],
                S::ZERO,
                None
            ),
            [n(30.0), n(60.0)]
        );
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MinContent,
                    available_extent: None
                },
                &[],
                S::ZERO,
                None
            ),
            [S::ZERO; 2]
        );
        // The same actual rows and immutable zero-minimum phases support a
        // changed definite operation and then the original indefinite one.
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks: &tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &rows,
        )
        .unwrap();
        for (available, expected) in [
            (
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                [n(30.0), n(60.0)],
            ),
            (
                GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(n(60.0)),
                },
                [n(20.0), n(40.0)],
            ),
            (
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                [n(30.0), n(60.0)],
            ),
        ] {
            assert_eq!(
                resolve_tracks_from_phases(ResolvedAxisTrackInput {
                    tracks: &tracks,
                    phases: &phases,
                    items: &rows,
                    available,
                    flex_bounds: GridTrackFinishingBounds::NONE,
                    stretch_size: None,
                    gap: S::ZERO,
                    alignment: AlignContent::Start,
                    gutters: None,
                })
                .unwrap(),
                expected
            );
            assert_eq!(phases.base_sizes, [S::ZERO; 2]);
        }
        // A different item's larger max-content candidate is not capped by90.
        let rows = [rows[0], item(0, 1, S::ZERO, S::ZERO, n(120.0))];
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[],
                S::ZERO,
                None
            ),
            [n(120.0), n(240.0)]
        );

        let tracks = [flex(S::ONE), flex(S::ONE)];
        let rows = [item(0, 2, S::ZERO, S::ZERO, n(60.0))];
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[n(40.0), S::ZERO],
                S::ZERO,
                None
            ),
            [n(40.0), n(40.0)]
        );
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(n(60.0))
                },
                &[n(40.0), S::ZERO],
                S::ZERO,
                None
            ),
            [n(40.0), n(20.0)]
        );
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(n(20.0))
                },
                &[n(40.0), S::ZERO],
                S::ZERO,
                None
            ),
            [n(40.0), S::ZERO]
        );
        // Freezing one operation cannot leak into a later differently spanned item.
        let rows = [rows[0], item(1, 2, S::ZERO, S::ZERO, n(100.0))];
        let expected = [n(100.0), n(100.0)];
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[n(40.0), S::ZERO],
                S::ZERO,
                None
            ),
            expected
        );
        assert_eq!(
            solve(
                &tracks,
                &[rows[1], rows[0]],
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[n(40.0), S::ZERO],
                S::ZERO,
                None
            ),
            expected
        );

        let tracks = [TrackSizingOf::px(n(20.0)), flex(S::ONE), flex(n(2.0))];
        let rows = [item(0, 3, S::ZERO, S::ZERO, n(120.0))];
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[],
                n(5.0),
                None
            ),
            [n(20.0), n(30.0), n(60.0)]
        );
        let tracks = [flex(S::ONE), TrackSizingOf::px(n(80.0)), flex(n(2.0))];
        let gutters = OrdinaryGridAxisGuttersOf::new(3, &[false, true, false], n(10.0));
        let rows = [item(0, 3, S::ZERO, S::ZERO, n(100.0))];
        let sizes = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            &[],
            n(999.0),
            Some(&gutters),
        );
        assert_eq!(sizes, [n(30.0), S::ZERO, n(60.0)]);
        assert_eq!(
            track_sum_with_gutters(&sizes, n(999.0), Some(&gutters)),
            n(100.0)
        );

        let tracks = [flex(S::ZERO), flex(S::ONE)];
        let rows = [item(0, 2, S::ZERO, S::ZERO, n(60.0))];
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[n(10.0), S::ZERO],
                S::ZERO,
                None
            ),
            [n(10.0), n(50.0)]
        );
        assert_eq!(
            solve(
                &tracks,
                &rows,
                GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(n(60.0))
                },
                &[n(10.0), S::ZERO],
                S::ZERO,
                None
            ),
            [n(10.0), n(50.0)]
        );

        let tracks = [flex(S::ONE)];
        let rows = [item(0, 1, S::ZERO, S::ZERO, n(10.0))];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks: &tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &rows,
        )
        .unwrap();
        let bad = [item(0, 1, S::ZERO, S::ZERO, S::INFINITY)];
        assert!(matches!(
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &bad,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap: S::ZERO,
                alignment: AlignContent::Start,
                gutters: None,
            }),
            Err(SpanningContributionError::InvalidNumeric { .. })
        ));
        assert_eq!(
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &rows,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap: S::ZERO,
                alignment: AlignContent::Start,
                gutters: None,
            })
            .unwrap(),
            [n(10.0)]
        );
        assert_eq!(phases.base_sizes, [S::ZERO]);
    }
    #[test]
    fn flex_item_matrix_f32() {
        assert_flex_item_matrix::<f32>();
    }
    #[test]
    fn flex_item_matrix_f64() {
        assert_flex_item_matrix::<f64>();
    }

    fn assert_equal_budget<S: LayoutScalar>(count: usize, expected: S) {
        let tracks = vec![flex(S::ONE); count];
        let rows = (0..count)
            .map(|index| item(index, index + 1, S::ZERO, S::ZERO, S::ZERO))
            .collect::<Vec<_>>();
        let sizes = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(S::ONE),
            },
            &[],
            S::ZERO,
            None,
        );
        assert!(sizes.iter().all(|&size| size == expected));
        assert!(track_sum_with_gutters(&sizes, S::ZERO, None) <= S::ONE);
        let next = from_nonnegative_finite_rank::<S>(nonnegative_finite_rank(expected) + 1);
        let nominal = S::ONE / S::from_usize(count);
        assert_eq!(next, nominal);
        assert!((0..count).fold(S::ZERO, |sum, _| sum + next) > S::ONE);
    }
    #[test]
    fn flex_equal_budget_f32() {
        assert_equal_budget::<f32>(10, f32::from_bits(0x3dcc_cccc));
    }
    #[test]
    fn flex_equal_budget_f64() {
        assert_equal_budget::<f64>(9, f64::from_bits(0x3fbc_71c7_1c71_c71b));
    }

    fn assert_no_exact_budget<S: LayoutScalar>(half: S, ulp: S) {
        let budget = S::ONE + ulp;
        assert_eq!(half + S::ONE, S::ONE);
        assert_eq!(half + (S::ONE + ulp), S::ONE + ulp + ulp);
        let tracks = [TrackSizingOf::px(half), flex(S::ONE)];
        let rows = [item(1, 2, S::ZERO, S::ZERO, S::ZERO)];
        let sizes = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(budget),
            },
            &[],
            S::ZERO,
            None,
        );
        assert_eq!(sizes, [half, S::ONE]);
        let occupied = track_sum_with_gutters(&sizes, S::ZERO, None);
        assert_eq!(occupied, S::ONE);
        assert_eq!(budget - occupied, ulp);
        // The next possible recipient value overspends; unpaid residual stays real.
        assert!(half + (S::ONE + ulp) > budget);
    }
    #[test]
    fn flex_no_exact_budget_f32() {
        assert_no_exact_budget::<f32>(1.0 / 16_777_216.0, 1.0 / 8_388_608.0);
    }
    #[test]
    fn flex_no_exact_budget_f64() {
        assert_no_exact_budget::<f64>(1.0 / 9_007_199_254_740_992.0, 1.0 / 4_503_599_627_370_496.0);
    }

    fn assert_extreme_budget<S: LayoutScalar>() {
        let budget =
            from_nonnegative_finite_rank(crate::scalar::max_nonnegative_finite_rank::<S>());
        let tracks = [flex(S::from_f64(0.6)), flex(S::from_f64(0.6))];
        let rows = [
            item(0, 1, S::ZERO, S::ZERO, S::ZERO),
            item(1, 2, S::ZERO, S::ZERO, S::ZERO),
        ];
        let sizes = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(budget),
            },
            &[],
            S::ZERO,
            None,
        );
        assert!(
            sizes
                .iter()
                .all(|value| value.is_finite() && *value > S::ZERO)
        );
        let occupied = track_sum_with_gutters(&sizes, S::ZERO, None);
        assert!(occupied.is_finite() && occupied <= budget);
    }
    #[test]
    fn flex_extreme_budget_f32() {
        assert_extreme_budget::<f32>();
    }
    #[test]
    fn flex_extreme_budget_f64() {
        assert_extreme_budget::<f64>();
    }

    fn assert_signed_outer_contributions<S: LayoutScalar>() {
        let n = S::from_f64;
        // An ordinary border20 with inline margins totaling-30 contributes-10.
        // The input remains signed; the track-sizing operation floors demand.
        let negative = n(20.0) + n(-30.0);
        assert_eq!(negative, n(-10.0));
        let rows = [item(0, 1, negative, negative, negative)];
        for track in [
            TrackSizingOf::AUTO,
            TrackSizingOf::MIN_CONTENT,
            TrackSizingOf::MAX_CONTENT,
        ] {
            for available in [
                GridTrackSizingConstraint {
                    operation: SizingOperation::MinContent,
                    available_extent: None,
                },
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None,
                },
                GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(n(100.0)),
                },
            ] {
                assert_eq!(
                    solve(
                        core::slice::from_ref(&track),
                        &rows,
                        available,
                        &[],
                        S::ZERO,
                        None
                    ),
                    [S::ZERO]
                );
            }
        }
        for available in [
            GridTrackSizingConstraint {
                operation: SizingOperation::MinContent,
                available_extent: None,
            },
            GridTrackSizingConstraint {
                operation: SizingOperation::MaxContent,
                available_extent: None,
            },
            GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(n(100.0)),
            },
        ] {
            assert_eq!(
                solve(
                    &[TrackSizingOf::px(n(20.0))],
                    &rows,
                    available,
                    &[],
                    S::ZERO,
                    None
                ),
                [n(20.0)]
            );
        }
        let tracks = [TrackSizingOf::px(n(20.0)), flex(S::ONE)];
        let span = [item(0, 2, negative, negative, negative)];
        assert_eq!(
            solve(
                &tracks,
                &span,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[],
                S::ZERO,
                None
            ),
            [n(20.0), S::ZERO]
        );
        assert_eq!(
            solve(
                &tracks,
                &span,
                GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(n(100.0))
                },
                &[],
                S::ZERO,
                None
            ),
            [n(20.0), n(80.0)]
        );
        let mixed = [item(0, 1, negative, negative, n(30.0))];
        assert_eq!(
            solve(
                &[flex(S::ONE)],
                &mixed,
                GridTrackSizingConstraint {
                    operation: SizingOperation::MaxContent,
                    available_extent: None
                },
                &[],
                S::ZERO,
                None
            ),
            [n(30.0)]
        );
        assert_eq!(rows[0].contributions.minimum, negative);
        assert_eq!(rows[0].contributions.min_content, negative);
        assert_eq!(rows[0].contributions.max_content, negative);
    }
    #[test]
    fn flex_signed_outer_contributions_f32() {
        assert_signed_outer_contributions::<f32>();
    }
    #[test]
    fn flex_signed_outer_contributions_f64() {
        assert_signed_outer_contributions::<f64>();
    }

    fn assert_no_available_extent_preserves_represented_bases<S: LayoutScalar>(
        count: usize,
        a: S,
        b: S,
    ) {
        assert!(a > b && b > S::ZERO);
        let mut minima = vec![a; count];
        minima[count - 1] = b;
        let occupied = minima
            .iter()
            .copied()
            .fold(S::ZERO, |sum, value| sum + value);
        assert!(
            occupied > S::ONE,
            "the real ordered bases already exhaust the budget"
        );
        assert_eq!(
            S::ONE / S::from_usize(count),
            a,
            "the old nominal quotient could grow the last base"
        );
        let tracks = minima
            .iter()
            .map(|&minimum| {
                TrackSizingOf::minmax(
                    MinTrackSizingOf::px(minimum),
                    MaxTrackSizingOf::flex(TrackFlexFactorOf::try_new(S::ONE).unwrap()),
                )
            })
            .collect::<Vec<_>>();
        let rows = (0..count)
            .map(|index| item(index, index + 1, S::ZERO, S::ZERO, S::ZERO))
            .collect::<Vec<_>>();
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks: &tracks,
                basis: None,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(S::ONE),
                },
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &rows,
        )
        .unwrap();
        assert_eq!(phases.base_sizes, minima);
        let sizes = resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks: &tracks,
            phases: &phases,
            items: &rows,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(S::ONE),
            },
            flex_bounds: GridTrackFinishingBounds::NONE,
            stretch_size: None,
            gap: S::ZERO,
            alignment: AlignContent::Start,
            gutters: None,
        })
        .unwrap();
        assert_eq!(
            sizes, minima,
            "no extra expansion or shrinking of legitimate bases"
        );
        assert_eq!(track_sum_with_gutters(&sizes, S::ZERO, None), occupied);
        assert_eq!(phases.base_sizes, minima);

        let half = S::from_f64(0.5);
        let tracks = [flex(S::ONE), flex(S::ONE)];
        let rows = [item(0, 2, S::ZERO, S::ZERO, S::ZERO)];
        let gutters = OrdinaryGridAxisGuttersOf::new(2, &[], S::from_f64(0.125));
        let sizes = solve(
            &tracks,
            &rows,
            GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(S::ONE),
            },
            &[half, half],
            S::ZERO,
            Some(&gutters),
        );
        assert_eq!(sizes, [half, half]);
        assert_eq!(
            track_sum_with_gutters(&sizes, S::ZERO, Some(&gutters)),
            S::from_f64(1.125)
        );
    }
    #[test]
    fn flex_no_available_extent_preserves_bases_f32() {
        assert_no_available_extent_preserves_represented_bases::<f32>(
            10,
            f32::from_bits(0x3dcc_cccd),
            f32::from_bits(0x3dcc_cccc),
        );
    }
    #[test]
    fn flex_no_available_extent_preserves_bases_f64() {
        // Eight1/9 bases plus its immediately lower neighbor still left-fold
        // above1; the ninth value must not grow to the old quotient1/9.
        assert_no_available_extent_preserves_represented_bases::<f64>(
            9,
            f64::from_bits(0x3fbc_71c7_1c71_c71c),
            f64::from_bits(0x3fbc_71c7_1c71_c71b),
        );
    }
}
