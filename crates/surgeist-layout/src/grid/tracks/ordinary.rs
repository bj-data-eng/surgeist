use super::*;
use crate::scalar::{
    from_nonnegative_finite_rank, max_nonnegative_finite_rank, nonnegative_finite_rank,
};

use super::flexible::span_contribution_with_gutters;
use crate::LengthResolutionOf;

fn resolution_or_zero<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> S {
    resolution_or_fallback(resolution, S::ZERO)
}

fn resolution_or_fallback<S: LayoutScalar>(resolution: LengthResolutionOf<S>, fallback: S) -> S {
    resolution_or_else(resolution, || fallback)
}

fn resolution_or_else<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
    fallback: impl FnOnce() -> S,
) -> S {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .expect("resolved length resolution must carry a value"),
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => fallback(),
    }
}

pub(super) fn resolution_optional<S: LayoutScalar>(resolution: LengthResolutionOf<S>) -> Option<S> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution.value,
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => None,
    }
}

#[derive(Clone)]
pub(super) struct OrdinaryTrackState<'a, S: LayoutScalar> {
    sizing_functions: &'a TrackSizingOf<S>,
    pub(super) base_size: S,
    pub(super) growth_limit: Option<S>,
    fit_content_limit: Option<S>,
    infinitely_growable: bool,
    pub(super) flex_factor: Option<S>,
    pub(super) auto_max_stretch_eligible: bool,
    pub(super) collapsed: bool,
    pub(super) fr_frozen: bool,
}

impl<'a, S: LayoutScalar> OrdinaryTrackState<'a, S> {
    pub(super) fn new(
        sizing_functions: &'a TrackSizingOf<S>,
        auto_max_stretch_eligible: bool,
        collapsed: bool,
    ) -> Self {
        Self {
            sizing_functions,
            base_size: S::ZERO,
            growth_limit: None,
            fit_content_limit: None,
            infinitely_growable: false,
            flex_factor: track_flex_factor(sizing_functions),
            auto_max_stretch_eligible,
            collapsed,
            fr_frozen: false,
        }
    }

    pub(super) fn apply_intrinsic_contributions(
        &mut self,
        basis: Option<S>,
        available: AvailableOf<S>,
        contributions: GridTrackContributions<S>,
    ) {
        self.infinitely_growable = false;
        if self.collapsed {
            self.base_size = S::ZERO;
            self.growth_limit = Some(S::ZERO);
            self.fit_content_limit = None;
            self.flex_factor = None;
            self.auto_max_stretch_eligible = false;
            return;
        }
        let GridTrackContributions {
            min_content,
            max_content,
            ..
        } = contributions;
        self.fit_content_limit = match &self.sizing_functions.max {
            MaxTrackSizingOf::FitContent(limit) => {
                resolution_optional(resolve_track_calculation(limit, basis))
            }
            _ => None,
        };
        self.base_size =
            track_min_size_for_intrinsics(self.sizing_functions, basis, available, contributions);
        self.growth_limit = match &self.sizing_functions.max {
            MaxTrackSizingOf::Calculation(calculation) => {
                resolution_optional(resolve_track_calculation(calculation, basis))
                    .or_else(|| calculation.depends_on_basis().then_some(max_content))
            }
            MaxTrackSizingOf::FitContent(_) => Some(
                self.fit_content_limit
                    .map_or(max_content, |limit| max_content.min(limit)),
            ),
            MaxTrackSizingOf::MinContent => Some(min_content),
            MaxTrackSizingOf::Auto | MaxTrackSizingOf::MaxContent => Some(max_content),
            MaxTrackSizingOf::Flex(_) => None,
        };
        if let Some(limit) = self.growth_limit {
            self.growth_limit = Some(limit.max(self.base_size));
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct OrdinaryIntrinsicContributionInput<'a, S: LayoutScalar = Scalar> {
    pub(in crate::grid) tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) start: usize,
    pub(in crate::grid) end: usize,
    pub(in crate::grid) kind: IntrinsicSpanContribution,
    pub(in crate::grid) percent_basis: Option<S>,
    pub(in crate::grid) contribution: S,
    pub(in crate::grid) gap: S,
    pub(in crate::grid) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
}

pub(in crate::grid) fn apply_ordinary_intrinsic_contribution<S: LayoutScalar>(
    sizes: &mut [S],
    input: OrdinaryIntrinsicContributionInput<'_, S>,
) {
    let OrdinaryIntrinsicContributionInput {
        tracks,
        start,
        end,
        kind,
        percent_basis,
        contribution,
        gap,
        gutters,
    } = input;
    let Some(span_tracks) = tracks.get(start..end) else {
        return;
    };
    let Some(span_sizes) = sizes.get_mut(start..end) else {
        return;
    };
    if span_tracks.is_empty() || !span_tracks.iter().any(track_has_intrinsic_sizing_function) {
        return;
    }
    if span_tracks.len() == 1 {
        span_sizes[0] = span_sizes[0].max(contribution);
        return;
    }
    let mut target = span_contribution_with_gutters(contribution, start, end, gap, gutters);
    if matches!(kind, IntrinsicSpanContribution::MinContent { .. }) {
        target = (target - intrinsic_span_minimum_floor_space(span_tracks)).max(S::ZERO);
    }
    distribute_intrinsic_span(span_sizes, span_tracks, kind, percent_basis, target);
}

// The full track and actual containing constraint are necessary: an Auto
// minimum under an intrinsic constraint also depends on its real maximum cap.
pub(in crate::grid) fn track_min_size_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    available: AvailableOf<S>,
    contributions: GridTrackContributions<S>,
) -> S {
    let GridTrackContributions {
        minimum,
        min_content,
        max_content,
    } = contributions;
    match &track.min {
        MinTrackSizingOf::Calculation(calculation)
            if !track_calculation_is_cyclic_auto(calculation, basis) =>
        {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MinTrackSizingOf::Calculation(_) | MinTrackSizingOf::Auto => match available {
            AvailableOf::Definite(_) => minimum,
            AvailableOf::MinContent | AvailableOf::MaxContent => {
                let limited = match &track.max {
                    MaxTrackSizingOf::Calculation(limit) | MaxTrackSizingOf::FitContent(limit) => {
                        resolution_optional(resolve_track_calculation(limit, basis))
                            .map_or(min_content, |limit| min_content.min(limit))
                    }
                    _ => min_content,
                };
                limited.max(minimum)
            }
        },
        MinTrackSizingOf::MinContent => min_content,
        MinTrackSizingOf::MaxContent => max_content,
    }
}

// Numeric-resolution test utility; final sizing goes through the phase owner.
#[cfg(test)]
pub(in crate::grid) fn track_base_size_for_intrinsics<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    available: AvailableOf<S>,
    contributions: GridTrackContributions<S>,
) -> S {
    let min = track_min_size_for_intrinsics(track, basis, available, contributions);
    let GridTrackContributions {
        min_content,
        max_content,
        ..
    } = contributions;
    let max_base = match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_or_else(resolve_track_calculation(calculation, basis), || {
                if calculation.depends_on_basis() {
                    max_content
                } else {
                    resolution_or_zero(resolve_track_calculation(calculation, None))
                }
            })
        }
        MaxTrackSizingOf::Flex(_) => S::ZERO,
        MaxTrackSizingOf::Auto | MaxTrackSizingOf::MaxContent => max_content,
        MaxTrackSizingOf::MinContent => min_content,
        MaxTrackSizingOf::FitContent(limit) => {
            let limit =
                resolution_or_fallback(resolve_track_calculation(limit, basis), max_content);
            max_content.min(limit)
        }
    };
    min.max(max_base)
}

// Fixed/percentage-floor inspection uses a uniform starting scalar (actual
// production callers pass zero), not a selected measured contribution channel.
pub(in crate::grid) fn track_base_size<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    uniform_intrinsic: S,
) -> S {
    let min = track_min_size(&track.min, basis, uniform_intrinsic);
    let max_base = match &track.max {
        MaxTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MaxTrackSizingOf::Flex(_) => S::ZERO,
        MaxTrackSizingOf::Auto | MaxTrackSizingOf::MinContent | MaxTrackSizingOf::MaxContent => {
            uniform_intrinsic
        }
        MaxTrackSizingOf::FitContent(limit) => {
            let limit =
                resolution_or_fallback(resolve_track_calculation(limit, basis), uniform_intrinsic);
            uniform_intrinsic.min(limit)
        }
    };
    min.max(max_base)
}

fn track_min_size<S: LayoutScalar>(
    min: &MinTrackSizingOf<S>,
    basis: Option<S>,
    uniform_intrinsic: S,
) -> S {
    match min {
        MinTrackSizingOf::Calculation(calculation) => {
            resolution_or_zero(resolve_track_calculation(calculation, basis))
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            uniform_intrinsic
        }
    }
}

#[cfg(test)]
pub(in crate::grid) fn track_growth_limit<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
    available: AvailableOf<S>,
    contributions: GridTrackContributions<S>,
) -> Option<S> {
    let mut state = OrdinaryTrackState::new(track, false, false);
    state.apply_intrinsic_contributions(basis, available, contributions);
    state.growth_limit
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct SpanningTrackContext<'a, S: LayoutScalar> {
    pub(in crate::grid) tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) basis: Option<S>,
    pub(in crate::grid) available: AvailableOf<S>,
    pub(in crate::grid) gap: S,
    pub(in crate::grid) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(in crate::grid) base_floors: &'a [S],
}

#[derive(Debug, PartialEq)]
pub(in crate::grid) struct GridAxisTrackPhases<S: LayoutScalar> {
    pub(in crate::grid) base_sizes: Vec<S>,
    pub(in crate::grid) growth_limits: Vec<Option<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) enum SpanningContributionError<S: LayoutScalar> {
    Allocation,
    InvalidIncidence,
    MissingContribution { purpose: GridContributionPurpose },
    DuplicateContribution { purpose: GridContributionPurpose },
    InvalidNumeric { value: S },
}

fn phase_number<S: LayoutScalar>(value: S) -> Result<S, SpanningContributionError<S>> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(SpanningContributionError::InvalidNumeric { value })
    }
}

fn phase_buffer<T: Clone, S: LayoutScalar>(
    count: usize,
    value: T,
) -> Result<Vec<T>, SpanningContributionError<S>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| SpanningContributionError::Allocation)?;
    result.resize(count, value);
    Ok(result)
}

#[derive(Clone, Copy)]
struct SpanningItem<S: LayoutScalar> {
    start: usize,
    end: usize,
    facts: GridTrackContributions<S>,
    flexible: bool,
}

fn same_receipt<Node: Eq, S: LayoutScalar>(
    left: &GridContributionReceipt<Node, S>,
    right: &GridContributionReceipt<Node, S>,
) -> bool {
    left.source == right.source && left.axis == right.axis && left.span == right.span
}

fn matched_receipt<Node: Eq, S: LayoutScalar>(
    wanted: &GridContributionReceipt<Node, S>,
    receipts: &[GridContributionReceipt<Node, S>],
    purpose: GridContributionPurpose,
) -> Result<S, SpanningContributionError<S>> {
    let mut result = None;
    for receipt in receipts {
        if receipt.purpose != purpose {
            return Err(SpanningContributionError::InvalidIncidence);
        }
        if same_receipt(wanted, receipt) {
            if result.is_some() {
                return Err(SpanningContributionError::DuplicateContribution { purpose });
            }
            result = Some(phase_number(receipt.value)?);
        }
    }
    result.ok_or(SpanningContributionError::MissingContribution { purpose })
}

// Facts have already been normalized into the owning Grid frame. This owner
// never consumes envelope-converted vectors or rewrites actual query receipts.
// Ordinary placement receipts retain their actual source identity in this
// adapter. The numeric owner receives only already-normalized constraint rows.
pub(in crate::grid) fn grid_contribution_rows<Node: Copy + Eq, S: LayoutScalar>(
    contributions: &GridAxisContributions<Node, S>,
) -> Result<Vec<GridIntrinsicContributionRow<S>>, SpanningContributionError<S>> {
    if contributions.minimum.purpose != GridContributionPurpose::Minimum
        || contributions.min_content.purpose != GridContributionPurpose::MinContent
        || contributions.max_content.purpose != GridContributionPurpose::MaxContent
    {
        return Err(SpanningContributionError::InvalidIncidence);
    }
    let receipts = &contributions.minimum.receipts;
    let mut rows = Vec::new();
    rows.try_reserve_exact(receipts.len())
        .map_err(|_| SpanningContributionError::Allocation)?;
    for receipt in receipts {
        let minimum = matched_receipt(receipt, receipts, GridContributionPurpose::Minimum)?;
        let min_content = matched_receipt(
            receipt,
            &contributions.min_content.receipts,
            GridContributionPurpose::MinContent,
        )?;
        let max_content = matched_receipt(
            receipt,
            &contributions.max_content.receipts,
            GridContributionPurpose::MaxContent,
        )?;
        if receipts
            .first()
            .is_some_and(|first| first.axis != receipt.axis)
        {
            return Err(SpanningContributionError::InvalidIncidence);
        }
        rows.push(GridIntrinsicContributionRow {
            span: receipt.span,
            contributions: GridTrackContributions {
                minimum,
                min_content,
                max_content,
            },
        });
    }
    // Orphan content receipts cannot silently acquire a made-up Minimum.
    for (purpose, set) in [
        (
            GridContributionPurpose::MinContent,
            &contributions.min_content.receipts,
        ),
        (
            GridContributionPurpose::MaxContent,
            &contributions.max_content.receipts,
        ),
    ] {
        for receipt in set {
            if receipt.purpose != purpose {
                return Err(SpanningContributionError::InvalidIncidence);
            }
            matched_receipt(receipt, receipts, GridContributionPurpose::Minimum)?;
        }
    }
    Ok(rows)
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct GridIntrinsicContributionRow<S: LayoutScalar> {
    pub(in crate::grid) span: GridTrackSpan,
    pub(in crate::grid) contributions: GridTrackContributions<S>,
}

// Lane owns its authentic hypothetical rows; ordinary placement owns its real
// receipt rows. Neither adapter invents source identities for the other domain.
pub(in crate::grid) fn resolve_grid_contribution_rows<S: LayoutScalar>(
    context: SpanningTrackContext<'_, S>,
    rows: &[GridIntrinsicContributionRow<S>],
) -> Result<GridAxisTrackPhases<S>, SpanningContributionError<S>> {
    let count = context.tracks.len();
    if !context.base_floors.is_empty() && context.base_floors.len() != count {
        return Err(SpanningContributionError::InvalidIncidence);
    }
    phase_number(context.gap)?;
    if let Some(basis) = context.basis {
        phase_number(basis)?;
    }
    if let AvailableOf::Definite(value) = context.available {
        phase_number(value)?;
    }
    for track in context.tracks {
        let minimum = match &track.min {
            MinTrackSizingOf::Calculation(value) => Some(value),
            _ => None,
        };
        let maximum = match &track.max {
            MaxTrackSizingOf::Calculation(value) | MaxTrackSizingOf::FitContent(value) => {
                Some(value)
            }
            _ => None,
        };
        for calculation in [minimum, maximum].into_iter().flatten() {
            if let LengthResolutionStatus::InvalidNumeric { value } =
                resolve_track_calculation(calculation, context.basis).status()
            {
                return Err(SpanningContributionError::InvalidNumeric { value });
            }
        }
    }
    let mut singles = phase_buffer::<_, S>(count, false)?;
    let mut single_facts = phase_buffer::<_, S>(
        count,
        GridTrackContributions {
            minimum: S::ZERO,
            min_content: S::ZERO,
            max_content: S::ZERO,
        },
    )?;
    let mut items = Vec::new();
    for row in rows {
        let start = row
            .span
            .start
            .checked_sub(1)
            .ok_or(SpanningContributionError::InvalidIncidence)?;
        let end = row
            .span
            .end
            .checked_sub(1)
            .ok_or(SpanningContributionError::InvalidIncidence)?;
        let tracks = context
            .tracks
            .get(start..end)
            .filter(|span| !span.is_empty())
            .ok_or(SpanningContributionError::InvalidIncidence)?;
        let facts = row.contributions;
        for value in [facts.minimum, facts.min_content, facts.max_content] {
            phase_number(value)?;
        }
        if end - start == 1 {
            singles[start] = true;
            let single = &mut single_facts[start];
            single.minimum = single.minimum.max(facts.minimum);
            single.min_content = single.min_content.max(facts.min_content);
            single.max_content = single.max_content.max(facts.max_content);
        } else {
            items
                .try_reserve(1)
                .map_err(|_| SpanningContributionError::Allocation)?;
            items.push(SpanningItem {
                start,
                end,
                facts,
                flexible: tracks
                    .iter()
                    .any(|track| track_flex_factor(track).is_some()),
            });
        }
    }
    let mut states = Vec::new();
    states
        .try_reserve_exact(count)
        .map_err(|_| SpanningContributionError::Allocation)?;
    for (index, track) in context.tracks.iter().enumerate() {
        let collapsed = context
            .gutters
            .and_then(|g| g.collapsed().get(index))
            .copied()
            .unwrap_or(false);
        let floor = phase_number(context.base_floors.get(index).copied().unwrap_or(S::ZERO))?;
        let mut state = OrdinaryTrackState::new(track, false, collapsed);
        state.apply_intrinsic_contributions(context.basis, context.available, single_facts[index]);
        if !collapsed {
            state.base_size = state.base_size.max(floor);
            if let Some(limit) = state.growth_limit {
                state.growth_limit = Some(limit.max(state.base_size));
            }
            // Base-floor constraints do not assert the existence of a single
            // item. Only actual supplied row incidence distinguishes zero.
            if !singles[index] && intrinsic_maximum(track, context.basis) {
                state.growth_limit = None;
            }
        }
        phase_number(state.base_size)?;
        if state.base_size < S::ZERO {
            return Err(SpanningContributionError::InvalidNumeric {
                value: state.base_size,
            });
        }
        if let Some(value) = state.fit_content_limit {
            phase_number(value)?;
        }
        if let Some(value) = state.growth_limit {
            phase_number(value)?;
        }
        states.push(state);
    }
    items.sort_unstable_by_key(|item| item.end - item.start);
    let mut scratch = PhaseScratch {
        planned_sizes: phase_buffer::<_, S>(count, S::ZERO)?,
        item_sizes: phase_buffer::<_, S>(count, S::ZERO)?,
        touched: phase_buffer::<_, S>(count, false)?,
    };
    let mut start = 0;
    while start < items.len() {
        let length = items[start].end - items[start].start;
        let mut end = start + 1;
        while end < items.len() && items[end].end - items[end].start == length {
            end += 1;
        }
        run_spanning_group(
            context,
            &mut states,
            &items[start..end],
            false,
            &mut scratch,
        )?;
        start = end;
    }
    // Flexible spans are considered together after non-flexible spans.
    run_spanning_group(context, &mut states, &items, true, &mut scratch)?;
    let mut result = GridAxisTrackPhases {
        base_sizes: phase_buffer::<_, S>(count, S::ZERO)?,
        growth_limits: phase_buffer::<_, S>(count, None)?,
    };
    for (index, state) in states.into_iter().enumerate() {
        result.base_sizes[index] = state.base_size;
        result.growth_limits[index] = Some(
            state
                .growth_limit
                .unwrap_or(state.base_size)
                .max(state.base_size),
        );
    }
    Ok(result)
}

fn intrinsic_maximum<S: LayoutScalar>(track: &TrackSizingOf<S>, basis: Option<S>) -> bool {
    match &track.max {
        MaxTrackSizingOf::Auto
        | MaxTrackSizingOf::MinContent
        | MaxTrackSizingOf::MaxContent
        | MaxTrackSizingOf::FitContent(_) => true,
        MaxTrackSizingOf::Calculation(value) => track_calculation_is_cyclic_auto(value, basis),
        MaxTrackSizingOf::Flex(_) => false,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SpanPhase {
    Minimum,
    ContentMinimum,
    LimitedMaximum,
    MaximumMinimum,
    ContentGrowth,
    MaximumGrowth,
}
impl SpanPhase {
    fn growth(self) -> bool {
        matches!(self, Self::ContentGrowth | Self::MaximumGrowth)
    }
}
struct PhaseScratch<S> {
    planned_sizes: Vec<S>,
    item_sizes: Vec<S>,
    touched: Vec<bool>,
}

fn affects<S: LayoutScalar>(phase: SpanPhase, track: &TrackSizingOf<S>, basis: Option<S>) -> bool {
    match phase {
        SpanPhase::Minimum => track_minimum_is_intrinsic(track, basis),
        SpanPhase::ContentMinimum => matches!(
            track.min,
            MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent
        ),
        SpanPhase::LimitedMaximum => {
            matches!(
                track.min,
                MinTrackSizingOf::Auto | MinTrackSizingOf::MaxContent
            ) || matches!(&track.min, MinTrackSizingOf::Calculation(value) if track_calculation_is_cyclic_auto(value, basis))
        }
        SpanPhase::MaximumMinimum => matches!(track.min, MinTrackSizingOf::MaxContent),
        SpanPhase::ContentGrowth => intrinsic_maximum(track, basis),
        SpanPhase::MaximumGrowth => {
            matches!(
                track.max,
                MaxTrackSizingOf::Auto
                    | MaxTrackSizingOf::MaxContent
                    | MaxTrackSizingOf::FitContent(_)
            ) || matches!(&track.max, MaxTrackSizingOf::Calculation(value) if track_calculation_is_cyclic_auto(value, basis))
        }
    }
}

fn run_spanning_group<'tracks, S: LayoutScalar>(
    context: SpanningTrackContext<'tracks, S>,
    states: &mut [OrdinaryTrackState<'tracks, S>],
    items: &[SpanningItem<S>],
    flexible: bool,
    scratch: &mut PhaseScratch<S>,
) -> Result<(), SpanningContributionError<S>> {
    for phase in [
        SpanPhase::Minimum,
        SpanPhase::ContentMinimum,
        SpanPhase::LimitedMaximum,
        SpanPhase::MaximumMinimum,
        SpanPhase::ContentGrowth,
        SpanPhase::MaximumGrowth,
    ] {
        if phase == SpanPhase::LimitedMaximum && context.available != AvailableOf::MAX_CONTENT {
            continue;
        }
        if phase == SpanPhase::ContentGrowth {
            for state in states.iter_mut() {
                if let Some(limit) = state.growth_limit {
                    state.growth_limit = Some(limit.max(state.base_size));
                }
            }
        }
        apply_span_phase(context, states, items, flexible, phase, scratch)?;
    }
    Ok(())
}

fn item_target<S: LayoutScalar>(
    context: SpanningTrackContext<'_, S>,
    item: SpanningItem<S>,
    phase: SpanPhase,
) -> Result<S, SpanningContributionError<S>> {
    let gap = context.gutters.map_or_else(
        || context.gap * S::from_usize(item.end - item.start - 1),
        |gutters| gutters.span_gap_total(item.start, item.end),
    );
    phase_number(gap)?;
    let target = |value: S| phase_number::<S>(value - gap).map(|value| value.max(S::ZERO));
    let minimum = target(item.facts.minimum)?;
    let content = match phase {
        SpanPhase::Minimum if matches!(context.available, AvailableOf::Definite(_)) => {
            return Ok(minimum);
        }
        SpanPhase::Minimum | SpanPhase::ContentMinimum | SpanPhase::ContentGrowth => {
            target(item.facts.min_content)?
        }
        _ => target(item.facts.max_content)?,
    };
    if !matches!(phase, SpanPhase::Minimum | SpanPhase::LimitedMaximum) {
        return Ok(content);
    }
    let mut cap = S::ZERO;
    for track in &context.tracks[item.start..item.end] {
        let value = match &track.max {
            MaxTrackSizingOf::Calculation(_) => effective_fixed_track_maximum(track, context.basis)
                .map_err(|status| match status {
                    LengthResolutionStatus::InvalidNumeric { value } => {
                        SpanningContributionError::InvalidNumeric { value }
                    }
                    _ => SpanningContributionError::InvalidIncidence,
                })?,
            MaxTrackSizingOf::FitContent(value) => {
                resolution_optional(resolve_track_calculation(value, context.basis))
            }
            _ => None,
        };
        let Some(value) = value else {
            return Ok(content.max(minimum));
        };
        cap = phase_number(cap + value)?;
    }
    Ok(content.min(cap).max(minimum))
}

fn apply_span_phase<'tracks, S: LayoutScalar>(
    context: SpanningTrackContext<'tracks, S>,
    states: &mut [OrdinaryTrackState<'tracks, S>],
    items: &[SpanningItem<S>],
    flexible: bool,
    phase: SpanPhase,
    scratch: &mut PhaseScratch<S>,
) -> Result<(), SpanningContributionError<S>> {
    for (planned, state) in scratch.planned_sizes.iter_mut().zip(states.iter()) {
        *planned = phase_affected_size(state, phase);
    }
    scratch.touched.fill(false);
    for item in items
        .iter()
        .copied()
        .filter(|item| item.flexible == flexible)
    {
        let eligible = |index: usize| {
            !states[index].collapsed
                && (!flexible || states[index].flex_factor.is_some())
                && affects(phase, &context.tracks[index], context.basis)
        };
        if !(item.start..item.end).any(eligible) {
            continue;
        }
        for (size, state) in scratch.item_sizes.iter_mut().zip(states.iter()) {
            *size = phase_affected_size(state, phase);
        }
        let occupied = phase_span_size(&scratch.item_sizes, item)?;
        let mut remaining =
            phase_number(item_target(context, item, phase)? - occupied)?.max(S::ZERO);
        for selection in [
            DistributionPass::Affected,
            DistributionPass::Other,
            DistributionPass::Beyond,
        ] {
            distribute_phase_space(
                PhaseDistribution {
                    context,
                    states,
                    item,
                    phase,
                    flexible,
                    selection,
                },
                &mut remaining,
                &mut scratch.item_sizes,
            )?;
        }
        for (index, &size) in scratch
            .item_sizes
            .iter()
            .enumerate()
            .take(item.end)
            .skip(item.start)
        {
            if eligible(index) || size > phase_affected_size(&states[index], phase) {
                scratch.touched[index] = true;
            }
            scratch.planned_sizes[index] = scratch.planned_sizes[index].max(size);
        }
    }
    for (index, state) in states.iter_mut().enumerate() {
        if phase.growth() {
            if scratch.touched[index] {
                let was_infinite = state.growth_limit.is_none();
                state.growth_limit = Some(scratch.planned_sizes[index]);
                if phase == SpanPhase::ContentGrowth {
                    state.infinitely_growable = was_infinite;
                }
            }
        } else {
            state.base_size = scratch.planned_sizes[index];
        }
    }
    if phase == SpanPhase::MaximumGrowth {
        for state in states {
            state.infinitely_growable = false;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum DistributionPass {
    Affected,
    Other,
    Beyond,
}

struct PhaseDistribution<'a, 'tracks, S: LayoutScalar> {
    context: SpanningTrackContext<'tracks, S>,
    states: &'a [OrdinaryTrackState<'tracks, S>],
    item: SpanningItem<S>,
    phase: SpanPhase,
    flexible: bool,
    selection: DistributionPass,
}

fn phase_affected_size<S: LayoutScalar>(state: &OrdinaryTrackState<'_, S>, phase: SpanPhase) -> S {
    if phase.growth() {
        state.growth_limit.unwrap_or(state.base_size)
    } else {
        state.base_size
    }
}

fn phase_span_size<S: LayoutScalar>(
    sizes: &[S],
    item: SpanningItem<S>,
) -> Result<S, SpanningContributionError<S>> {
    sizes[item.start..item.end]
        .iter()
        .try_fold(S::ZERO, |sum, &size| phase_number(sum + size))
}

// Replays the real ordered suffix. Overflow is an infeasible trial value,
// not a reason to abandon finite lower candidates.
fn phase_trial_size<S: LayoutScalar>(
    sizes: &[S],
    item: SpanningItem<S>,
    recipient: usize,
    candidate: S,
) -> Option<S> {
    let mut total = S::ZERO;
    for (index, &size) in sizes.iter().enumerate().take(item.end).skip(item.start) {
        total = total + if index == recipient { candidate } else { size };
        if !total.is_finite() {
            return None;
        }
    }
    Some(total)
}

fn phase_residual_candidate<S: LayoutScalar>(
    sizes: &[S],
    item: SpanningItem<S>,
    recipient: usize,
    target: S,
) -> S {
    let residual = sizes[recipient + 1..item.end]
        .iter()
        .rev()
        .fold(target, |residual, &size| (residual - size).max(S::ZERO));
    let prefix = sizes[item.start..recipient]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, size| sum + size);
    (residual - prefix).max(S::ZERO)
}

// Greatest feasible ordered fold within these actual recipient bounds. A
// residual candidate is preferred within that fold's preimage, while a real
// plateau advance is retained instead of mistaking equal aggregates for no work.
fn phase_feasible_size<S: LayoutScalar>(
    sizes: &[S],
    item: SpanningItem<S>,
    recipient: usize,
    upper: S,
    preferred: S,
    target: S,
) -> S {
    let lower = sizes[recipient];
    let preferred = preferred.max(lower).min(upper);
    if preferred > lower && phase_trial_size(sizes, item, recipient, preferred) == Some(target) {
        return preferred;
    }
    let mut low = nonnegative_finite_rank(lower);
    let mut high = nonnegative_finite_rank(upper);
    if !phase_trial_size(sizes, item, recipient, upper).is_some_and(|total| total <= target) {
        // Positive finite IEEE ranks are ordered integers. Upper-midpoint
        // bisection strictly shrinks the interval: at most32/64 integer
        // iterations, plus fixed endpoint/preferred validation calls.
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            let candidate = from_nonnegative_finite_rank::<S>(middle);
            if phase_trial_size(sizes, item, recipient, candidate)
                .is_some_and(|total| total <= target)
            {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
    } else {
        low = high;
    }
    let best = from_nonnegative_finite_rank::<S>(low);
    let best_fold = phase_trial_size(sizes, item, recipient, best);
    if preferred > lower && phase_trial_size(sizes, item, recipient, preferred) == best_fold {
        preferred
    } else {
        best
    }
}

fn distribute_phase_space<S: LayoutScalar>(
    input: PhaseDistribution<'_, '_, S>,
    remaining: &mut S,
    sizes: &mut [S],
) -> Result<(), SpanningContributionError<S>> {
    let PhaseDistribution {
        context,
        states,
        item,
        phase,
        flexible,
        selection,
    } = input;
    let target = item_target(context, item, phase)?;
    loop {
        if *remaining <= S::ZERO {
            break;
        }
        let preferred = |index: usize, values: &[S]| {
            states[index]
                .fit_content_limit
                .is_none_or(|cap| values[index] < cap)
                && if matches!(phase, SpanPhase::MaximumMinimum | SpanPhase::LimitedMaximum) {
                    matches!(
                        context.tracks[index].max,
                        MaxTrackSizingOf::MaxContent
                            | MaxTrackSizingOf::Auto
                            | MaxTrackSizingOf::FitContent(_)
                            | MaxTrackSizingOf::Flex(_)
                    ) || matches!(&context.tracks[index].max, MaxTrackSizingOf::Calculation(value) if track_calculation_is_cyclic_auto(value, context.basis))
                } else {
                    intrinsic_maximum(&context.tracks[index], context.basis)
                }
        };
        let has_preferred = (item.start..item.end).any(|index| {
            !states[index].collapsed
                && (!flexible || states[index].flex_factor.is_some())
                && affects(phase, &context.tracks[index], context.basis)
                && preferred(index, sizes)
        });
        let capacity_limit = |index: usize, values: &[S]| -> Option<S> {
            let state = &states[index];
            if state.collapsed || (flexible && state.flex_factor.is_none()) {
                return Some(values[index]);
            }
            let affected = affects(phase, &context.tracks[index], context.basis);
            if (selection == DistributionPass::Affected && !affected)
                || (selection == DistributionPass::Other && affected)
                || (selection == DistributionPass::Beyond
                    && (!affected || (has_preferred && !preferred(index, values))))
                || (selection == DistributionPass::Beyond
                    && phase.growth()
                    && !preferred(index, values))
            {
                return Some(values[index]);
            }
            if selection == DistributionPass::Beyond {
                if preferred(index, values) {
                    state.fit_content_limit
                } else {
                    None
                }
            } else if phase.growth() {
                if state.growth_limit.is_some() && !state.infinitely_growable {
                    state.growth_limit
                } else {
                    state.fit_content_limit
                }
            } else {
                match (state.growth_limit, state.fit_content_limit) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                }
            }
        };
        let mut count = 0;
        let mut factors = S::ZERO;
        for (index, state) in states.iter().enumerate().take(item.end).skip(item.start) {
            if capacity_limit(index, sizes).is_none_or(|limit| limit > sizes[index]) {
                count += 1;
                factors = phase_number(factors + state.flex_factor.unwrap_or(S::ZERO))?;
            }
        }
        if count == 0 {
            break;
        }
        let occupied_before = phase_span_size(sizes, item)?;
        let round = *remaining;
        let mut nominal_used = S::ZERO;
        let mut capped_spill = S::ZERO;
        let mut recipients = count;
        let mut advanced = false;
        for (index, state) in states.iter().enumerate().take(item.end).skip(item.start) {
            let limit = capacity_limit(index, sizes);
            if limit.is_some_and(|limit| limit <= sizes[index]) {
                continue;
            }
            let last = recipients == 1;
            let nominal = if last {
                phase_number(round - nominal_used)?.max(S::ZERO)
            } else if flexible && factors > S::ZERO {
                let factor = state.flex_factor.unwrap_or(S::ZERO);
                let fraction = if factors >= S::ONE {
                    factor / factors
                } else {
                    factor + (S::ONE - factors) / S::from_usize(count)
                };
                phase_number(round * fraction)?
            } else {
                phase_number(round / S::from_usize(count))?
            };
            let nominal = nominal.min(phase_number(round - nominal_used)?.max(S::ZERO));
            nominal_used = phase_number(nominal_used + nominal)?;
            recipients -= 1;
            let previous = sizes[index];
            if let Some(limit) = limit {
                let room = (limit - previous).max(S::ZERO);
                capped_spill =
                    phase_number(capped_spill + (nominal - room).max(S::ZERO))?.min(round);
            }
            let nominal_final = previous + nominal;
            let upper = if nominal_final.is_finite() {
                nominal_final
            } else {
                from_nonnegative_finite_rank(max_nonnegative_finite_rank::<S>())
            };
            let upper = limit.map_or(upper, |limit| upper.min(limit)).max(previous);
            let candidate = if last {
                phase_residual_candidate(sizes, item, index, (target - capped_spill).max(S::ZERO))
            } else {
                upper
            };
            let selected = phase_feasible_size(sizes, item, index, upper, candidate, target);
            advanced |= selected > previous;
            sizes[index] = selected;
        }
        let mut occupied = phase_span_size(sizes, item)?;
        *remaining = phase_number(target - occupied)?.max(S::ZERO);
        if *remaining <= S::ZERO {
            break;
        }
        if !advanced || occupied <= occupied_before {
            // Retain all completed nominal values, including plateau advances.
            // A plateau aggregate also needs the current pass's full-interval
            // residual sweep; rank-by-rank nominal rounds can otherwise be huge.
            // Eligibility can change when the next round recomputes preferred
            // groups. This sweep does not certify unvisited actors or passes.
            for (index, _) in states.iter().enumerate().take(item.end).skip(item.start) {
                let limit = capacity_limit(index, sizes);
                if limit.is_some_and(|limit| limit <= sizes[index]) {
                    continue;
                }
                let previous = sizes[index];
                let upper = limit
                    .unwrap_or_else(|| {
                        from_nonnegative_finite_rank(max_nonnegative_finite_rank::<S>())
                    })
                    .max(previous);
                let candidate = phase_residual_candidate(sizes, item, index, target);
                let selected = phase_feasible_size(sizes, item, index, upper, candidate, target);
                advanced |= selected > previous;
                sizes[index] = selected;
                occupied = phase_span_size(sizes, item)?;
                *remaining = phase_number(target - occupied)?.max(S::ZERO);
                if *remaining <= S::ZERO {
                    break;
                }
            }
        }
        if !advanced {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod spanning_phases_tests {
    use super::*;
    use crate::{LengthPercentageOf, SizingCalculationOf};

    fn resolve_spanning_contributions<Node: Copy + Eq, S: LayoutScalar>(
        context: SpanningTrackContext<'_, S>,
        contributions: &GridAxisContributions<Node, S>,
    ) -> Result<GridAxisTrackPhases<S>, SpanningContributionError<S>> {
        let rows = grid_contribution_rows(contributions)?;
        resolve_grid_contribution_rows(context, &rows)
    }

    fn input(
        count: usize,
        items: &[(usize, usize, f64, f64, f64)],
    ) -> GridAxisContributions<u32, f64> {
        let mut minimum = vec![0.0_f64; count];
        let mut content = minimum.clone();
        let mut maximum = minimum.clone();
        let mut min_receipts = Vec::new();
        let mut content_receipts = Vec::new();
        let mut max_receipts = Vec::new();
        for (source, &(start, end, min, mc, mx)) in items.iter().enumerate() {
            let receipt = |purpose, value| GridContributionReceipt {
                purpose,
                source: u32::try_from(source).unwrap(),
                axis: GridAxisKind::Column,
                span: GridTrackSpan::new(start + 1, end + 1),
                value,
            };
            min_receipts.push(receipt(GridContributionPurpose::Minimum, min));
            content_receipts.push(receipt(GridContributionPurpose::MinContent, mc));
            max_receipts.push(receipt(GridContributionPurpose::MaxContent, mx));
            if end == start + 1 {
                minimum[start] = minimum[start].max(min);
                content[start] = content[start].max(mc);
                maximum[start] = maximum[start].max(mx);
            }
        }
        GridAxisContributions::new(
            IntrinsicAxisContributions::column(
                GridContributionPurpose::Minimum,
                minimum,
                min_receipts,
                Vec::new(),
            ),
            IntrinsicAxisContributions::column(
                GridContributionPurpose::MinContent,
                content,
                content_receipts,
                Vec::new(),
            ),
            IntrinsicAxisContributions::column(
                GridContributionPurpose::MaxContent,
                maximum,
                max_receipts,
                Vec::new(),
            ),
        )
    }
    fn context(
        tracks: &[TrackSizingOf<f64>],
        available: AvailableOf<f64>,
    ) -> SpanningTrackContext<'_, f64> {
        SpanningTrackContext {
            tracks,
            basis: None,
            available,
            gap: 0.0,
            gutters: None,
            base_floors: &[],
        }
    }

    #[test]
    fn nonaffected_fixed_minimum_grows_before_beyond_limits() {
        let tracks = [
            TrackSizingOf::minmax(MinTrackSizingOf::AUTO, MaxTrackSizingOf::px(10.0)),
            TrackSizingOf::minmax(MinTrackSizingOf::px(0.0), MaxTrackSizingOf::MAX_CONTENT),
        ];
        let facts = input(2, &[(0, 2, 0.0, 40.0, 80.0)]);
        let phases =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &facts)
                .unwrap();
        assert_eq!(phases.base_sizes, vec![10.0, 30.0]);
        assert_eq!(phases.growth_limits, vec![Some(10.0), Some(70.0)]);
        let result = super::super::flexible::resolve_tracks_from_phases(
            super::super::flexible::ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &grid_contribution_rows(&facts).unwrap(),
                available: AvailableOf::MIN_CONTENT,
                stretch_size: None,
                gap: 0.0,
                alignment: AlignContent::Start,
                gutters: None,
            },
        )
        .unwrap();
        assert_eq!(result, vec![10.0, 30.0]);
    }

    #[test]
    fn infinitely_growable_track_receives_remaining_maximum() {
        // Literal Grid1 §11.5 example: actual first item10; spanning30/100.
        let track =
            TrackSizingOf::minmax(MinTrackSizingOf::MIN_CONTENT, MaxTrackSizingOf::MAX_CONTENT);
        let tracks = [track.clone(), track];
        let facts = input(2, &[(0, 1, 10.0, 10.0, 10.0), (0, 2, 0.0, 30.0, 100.0)]);
        let phases =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &facts)
                .unwrap();
        assert_eq!(phases.base_sizes, vec![10.0, 20.0]);
        assert_eq!(phases.growth_limits, vec![Some(10.0), Some(90.0)]);
    }

    #[test]
    fn same_span_planned_increases_are_independent_of_item_order() {
        let tracks = [TrackSizingOf::<f64>::AUTO; 3];
        let one = input(3, &[(0, 2, 0.0, 40.0, 80.0), (1, 3, 0.0, 40.0, 80.0)]);
        let two = input(3, &[(1, 3, 0.0, 40.0, 80.0), (0, 2, 0.0, 40.0, 80.0)]);
        let first =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &one)
                .unwrap();
        let second =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &two)
                .unwrap();
        // Each span independently incurs20 per base, then planned maxima merge.
        assert_eq!(first.base_sizes, vec![20.0, 20.0, 20.0]);
        assert_eq!(first, second);
    }

    #[test]
    fn actual_span_gutter_is_counted_once() {
        let tracks = [
            TrackSizingOf::minmax(MinTrackSizingOf::AUTO, MaxTrackSizingOf::px(10.0)),
            TrackSizingOf::minmax(MinTrackSizingOf::px(0.0), MaxTrackSizingOf::MAX_CONTENT),
        ];
        let facts = input(2, &[(0, 2, 0.0, 45.0, 85.0)]);
        let gutters = OrdinaryGridAxisGuttersOf::new(2, &[], 5.0);
        let mut ctx = context(&tracks, AvailableOf::MIN_CONTENT);
        ctx.gap = 100.0; // Real gutter custody, not this unused fallback, applies.
        ctx.gutters = Some(&gutters);
        let phases = resolve_spanning_contributions(ctx, &facts).unwrap();
        assert_eq!(phases.base_sizes, vec![10.0, 30.0]);
        assert_eq!(phases.growth_limits, vec![Some(10.0), Some(70.0)]);
    }

    #[test]
    fn fit_content_limit_does_not_receive_an_uncapped_donation() {
        let fit = TrackSizingOf::fit_content(SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(30.0, 0.0).unwrap(),
        ));
        let tracks = [fit.clone(), fit];
        let facts = input(2, &[(0, 2, 0.0, 80.0, 100.0)]);
        let phases =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &facts)
                .unwrap();
        assert_eq!(phases.base_sizes, vec![30.0, 30.0]);
        assert_eq!(phases.growth_limits, vec![Some(30.0), Some(30.0)]);
    }

    #[test]
    fn missing_minimum_is_not_invented_from_content() {
        let tracks = [TrackSizingOf::<f64>::AUTO; 2];
        let mut facts = input(2, &[(0, 2, 0.0, 40.0, 80.0)]);
        facts.minimum.receipts.clear();
        assert_eq!(
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &facts),
            Err(SpanningContributionError::MissingContribution {
                purpose: GridContributionPurpose::Minimum
            })
        );
    }

    #[test]
    fn duplicate_purpose_receipt_is_not_guessed() {
        let tracks = [TrackSizingOf::<f64>::AUTO; 2];
        let mut facts = input(2, &[(0, 2, 0.0, 40.0, 80.0)]);
        facts.minimum.receipts.push(facts.minimum.receipts[0]);
        assert_eq!(
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &facts),
            Err(SpanningContributionError::DuplicateContribution {
                purpose: GridContributionPurpose::Minimum
            })
        );
    }
    #[test]
    fn genuine_zero_single_item_is_not_an_absent_growth_limit() {
        let tracks = [
            TrackSizingOf::minmax(MinTrackSizingOf::AUTO, MaxTrackSizingOf::px(10.0)),
            TrackSizingOf::minmax(MinTrackSizingOf::px(0.0), MaxTrackSizingOf::MAX_CONTENT),
        ];
        let facts = input(2, &[(0, 2, 0.0, 40.0, 80.0), (1, 2, 0.0, 0.0, 0.0)]);
        let phases =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MIN_CONTENT), &facts)
                .unwrap();
        // B now really starts at finite growth0. It cannot absorb base space
        // before beyond-limits expansion of affected A. A zero marker is not
        // interchangeable with there being no item in B.
        assert_eq!(phases.base_sizes, vec![40.0, 0.0]);
        assert_eq!(phases.growth_limits, vec![Some(40.0), Some(40.0)]);
    }

    #[test]
    fn hypothetical_single_row_retains_its_real_maximum_without_tree_receipt() {
        let tracks = [TrackSizingOf::<f64>::AUTO];
        let rows = [GridIntrinsicContributionRow {
            span: GridTrackSpan::new(1, 2),
            contributions: GridTrackContributions {
                minimum: 0.0,
                min_content: 20.0,
                max_content: 20.0,
            },
        }];
        let phases =
            resolve_grid_contribution_rows(context(&tracks, AvailableOf::definite(100.0)), &rows)
                .unwrap();
        assert_eq!(phases.base_sizes, vec![0.0]);
        assert_eq!(phases.growth_limits, vec![Some(20.0)]);
        let sizes = super::super::flexible::resolve_tracks_from_phases(
            super::super::flexible::ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &rows,
                available: AvailableOf::definite(100.0),
                stretch_size: None,
                gap: 0.0,
                alignment: AlignContent::Start,
                gutters: None,
            },
        )
        .unwrap();
        assert_eq!(sizes, vec![20.0]);
    }

    #[test]
    fn edge_base_floor_does_not_fabricate_zero_single_item_presence() {
        let tracks = [
            TrackSizingOf::minmax(MinTrackSizingOf::AUTO, MaxTrackSizingOf::px(10.0)),
            TrackSizingOf::minmax(MinTrackSizingOf::px(0.0), MaxTrackSizingOf::MAX_CONTENT),
        ];
        let rows = [GridIntrinsicContributionRow {
            span: GridTrackSpan::new(1, 3),
            contributions: GridTrackContributions {
                minimum: 0.0,
                min_content: 40.0,
                max_content: 80.0,
            },
        }];
        let floors = [5.0, 0.0];
        let mut ctx = context(&tracks, AvailableOf::MIN_CONTENT);
        ctx.base_floors = &floors;
        let phases = resolve_grid_contribution_rows(ctx, &rows).unwrap();
        // A's real edge floor5 is part of its base occupancy, not added again
        // to the normalized40 target. Zero B floor is not a zero item in B.
        assert_eq!(phases.base_sizes, vec![10.0, 30.0]);
        assert_eq!(phases.growth_limits, vec![Some(10.0), Some(70.0)]);
    }

    #[test]
    fn empty_subgrid_real_edge_floor_survives_without_item_receipts() {
        let tracks = [TrackSizingOf::<f64>::AUTO];
        let floors = [20.0];
        let mut ctx = context(&tracks, AvailableOf::definite(100.0));
        ctx.base_floors = &floors;
        let phases = resolve_grid_contribution_rows(ctx, &[]).unwrap();
        assert_eq!(phases.base_sizes, vec![20.0]);
        assert_eq!(phases.growth_limits, vec![Some(20.0)]);
    }

    #[test]
    fn finite_phase_consumer_rejects_nonfinite_total_before_container_clamp() {
        let tracks = [TrackSizingOf::px(f64::MAX), TrackSizingOf::px(f64::MAX)];
        let facts = input(2, &[]);
        let phases =
            resolve_spanning_contributions(context(&tracks, AvailableOf::MAX_CONTENT), &facts)
                .unwrap();
        let result = super::super::flexible::resolve_tracks_from_phases(
            super::super::flexible::ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &grid_contribution_rows(&facts).unwrap(),
                available: AvailableOf::MAX_CONTENT,
                stretch_size: None,
                gap: 0.0,
                alignment: AlignContent::Start,
                gutters: None,
            },
        );
        assert_eq!(
            result,
            Err(SpanningContributionError::InvalidNumeric {
                value: f64::INFINITY
            })
        );
    }

    fn percentage_track<S: LayoutScalar>(fraction: S) -> TrackSizingOf<S> {
        let calculation = SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(S::ZERO, fraction).unwrap(),
        );
        TrackSizingOf::minmax(
            MinTrackSizingOf::Calculation(calculation.clone()),
            MaxTrackSizingOf::Calculation(calculation),
        )
    }

    fn assert_cyclic_percentage_minimum<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let tracks = [
            percentage_track(scalar(0.2)),
            TrackSizingOf::px(scalar(10.0)),
        ];
        let rows = [GridIntrinsicContributionRow {
            span: GridTrackSpan::new(1, 3),
            contributions: GridTrackContributions {
                minimum: S::ZERO,
                min_content: scalar(40.0),
                max_content: scalar(80.0),
            },
        }];
        let context = SpanningTrackContext {
            tracks: &tracks,
            basis: None,
            available: AvailableOf::MIN_CONTENT,
            gap: S::ZERO,
            gutters: None,
            base_floors: &[],
        };
        let phases = resolve_grid_contribution_rows(context, &rows).unwrap();
        // The percentage is Auto intrinsically. Fixed10 is occupied once;
        // the genuine limited min-content40 leaves30 for that Auto minimum.
        assert_eq!(phases.base_sizes, vec![scalar(30.0), scalar(10.0)]);
        let invalid = resolve_grid_contribution_rows(
            SpanningTrackContext {
                basis: Some(S::INFINITY),
                ..context
            },
            &rows,
        );
        assert!(matches!(
            invalid,
            Err(SpanningContributionError::InvalidNumeric { .. })
        ));
    }

    fn assert_cyclic_percentage_thirteen_track_phases<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let fixed_minimum =
            |maximum| TrackSizingOf::minmax(MinTrackSizingOf::px(scalar(2.0)), maximum);
        let tracks = [
            TrackSizingOf::<S>::MIN_CONTENT,
            TrackSizingOf::MAX_CONTENT,
            TrackSizingOf::fit_content(SizingCalculationOf::value(
                LengthPercentageOf::px(scalar(20.0)).unwrap(),
            )),
            TrackSizingOf::AUTO,
            TrackSizingOf::px(scalar(10.0)),
            percentage_track(scalar(0.2)),
            fixed_minimum(MaxTrackSizingOf::AUTO),
            fixed_minimum(MaxTrackSizingOf::px(scalar(4.0))),
            fixed_minimum(MaxTrackSizingOf::MIN_CONTENT),
            fixed_minimum(MaxTrackSizingOf::MAX_CONTENT),
            TrackSizingOf::minmax(MinTrackSizingOf::MIN_CONTENT, MaxTrackSizingOf::MAX_CONTENT),
            TrackSizingOf::minmax(MinTrackSizingOf::MIN_CONTENT, MaxTrackSizingOf::AUTO),
            TrackSizingOf::minmax(MinTrackSizingOf::MAX_CONTENT, MaxTrackSizingOf::AUTO),
        ];
        // A scrollable zero minimum and a content-based160 minimum both use
        // the same limited min-content160 under this intrinsic MAX constraint.
        for minimum in [0.0, 160.0] {
            let mut rows = (0..tracks.len())
                .map(|index| GridIntrinsicContributionRow {
                    span: GridTrackSpan::new(index + 1, index + 2),
                    contributions: GridTrackContributions {
                        minimum: S::ZERO,
                        min_content: S::ZERO,
                        max_content: S::ZERO,
                    },
                })
                .collect::<Vec<_>>();
            rows.push(GridIntrinsicContributionRow {
                span: GridTrackSpan::new(1, 14),
                contributions: GridTrackContributions {
                    minimum: scalar(minimum),
                    min_content: scalar(160.0),
                    max_content: scalar(320.0),
                },
            });
            let phases = resolve_grid_contribution_rows(
                SpanningTrackContext {
                    tracks: &tracks,
                    basis: None,
                    available: AvailableOf::MAX_CONTENT,
                    gap: S::ZERO,
                    gutters: None,
                    base_floors: &[],
                },
                &rows,
            )
            .unwrap();
            // Fixed occupancy starts18; the nonaffected2..4 track takes2 first.
            // Remaining140/8=17.5. The capped Fit track takes2.5 of the next160;
            // the other four eligible maxima each receive157.5/4=39.375.
            let expected = [
                17.5, 56.875, 20.0, 56.875, 10.0, 56.875, 2.0, 4.0, 2.0, 2.0, 17.5, 17.5, 56.875,
            ];
            assert_eq!(
                phases.base_sizes,
                expected.into_iter().map(scalar).collect::<Vec<_>>()
            );
            assert_eq!(
                phases
                    .base_sizes
                    .iter()
                    .copied()
                    .fold(S::ZERO, |sum, value| sum + value),
                scalar(320.0)
            );
        }
    }

    #[test]
    fn cyclic_percentage_minimum_f32() {
        assert_cyclic_percentage_minimum::<f32>();
    }
    #[test]
    fn cyclic_percentage_minimum_f64() {
        assert_cyclic_percentage_minimum::<f64>();
    }
    #[test]
    fn cyclic_percentage_thirteen_track_phases_f32() {
        assert_cyclic_percentage_thirteen_track_phases::<f32>();
    }
    #[test]
    fn cyclic_percentage_thirteen_track_phases_f64() {
        assert_cyclic_percentage_thirteen_track_phases::<f64>();
    }

    fn assert_six_track_phase_conserves_actual_space<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let tracks = [
            TrackSizingOf::<S>::MIN_CONTENT,
            TrackSizingOf::MAX_CONTENT,
            TrackSizingOf::fit_content(SizingCalculationOf::value(
                LengthPercentageOf::px(scalar(20.0)).unwrap(),
            )),
            TrackSizingOf::AUTO,
            TrackSizingOf::px(scalar(10.0)),
            percentage_track(scalar(0.2)),
        ];
        let mut rows = (0..tracks.len())
            .map(|index| GridIntrinsicContributionRow {
                span: GridTrackSpan::new(index + 1, index + 2),
                contributions: GridTrackContributions {
                    minimum: S::ZERO,
                    min_content: S::ZERO,
                    max_content: S::ZERO,
                },
            })
            .collect::<Vec<_>>();
        rows.push(GridIntrinsicContributionRow {
            span: GridTrackSpan::new(1, 7),
            contributions: GridTrackContributions {
                minimum: scalar(80.0),
                min_content: scalar(80.0),
                max_content: scalar(160.0),
            },
        });
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks: &tracks,
                basis: None,
                available: AvailableOf::MAX_CONTENT,
                gap: S::ZERO,
                gutters: None,
                base_floors: &[],
            },
            &rows,
        )
        .unwrap();
        // MIN80 reserves fixed10 and gives14 to each of five intrinsic
        // minima. MAX160 gives nominal20 to four destinations; Fit takes6,
        // leaving14 to share among three. Its representation must conserve160.
        assert_eq!(phases.base_sizes[0], scalar(14.0));
        assert_eq!(phases.base_sizes[2], scalar(20.0));
        assert_eq!(phases.base_sizes[4], scalar(10.0));
        assert_eq!(
            phases
                .base_sizes
                .iter()
                .copied()
                .fold(S::ZERO, |sum, value| sum + value),
            scalar(160.0),
        );
        assert_eq!(
            phases
                .growth_limits
                .iter()
                .map(|limit| limit.unwrap())
                .fold(S::ZERO, |sum, value| sum + value),
            scalar(160.0),
        );
        let sizes = super::super::flexible::resolve_tracks_from_phases(
            super::super::flexible::ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &rows,
                available: AvailableOf::MAX_CONTENT,
                stretch_size: None,
                gap: S::ZERO,
                alignment: AlignContent::Start,
                gutters: None,
            },
        )
        .unwrap();
        assert_eq!(track_sum_with_gutters(&sizes, S::ZERO, None), scalar(160.0));
    }

    #[test]
    fn six_track_phase_conserves_actual_space_f32() {
        assert_six_track_phase_conserves_actual_space::<f32>();
    }

    #[test]
    fn six_track_phase_conserves_actual_space_f64() {
        assert_six_track_phase_conserves_actual_space::<f64>();
    }

    fn assert_quantized_phase_retains_feasible_increase<S: LayoutScalar>(base: S) {
        let scalar = S::from_f64;
        let target = base + scalar(4.0);
        let floor = scalar(2.0);
        // Both lanes use a finite binade with ULP4. A legal final small
        // track4 reaches the target; adding nominal4 to floor2 overshoots it.
        assert!(base.is_finite() && target.is_finite());
        assert_eq!(base + floor, base);
        assert_eq!(base + scalar(4.0), target);
        assert_eq!(base + scalar(6.0), base + scalar(8.0));
        let tracks = [TrackSizingOf::px(base), TrackSizingOf::AUTO];
        let floors = [S::ZERO, floor];
        let rows = [GridIntrinsicContributionRow {
            span: GridTrackSpan::new(1, 3),
            contributions: GridTrackContributions {
                minimum: target,
                min_content: target,
                max_content: target,
            },
        }];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks: &tracks,
                basis: None,
                available: AvailableOf::MAX_CONTENT,
                gap: S::ZERO,
                gutters: None,
                base_floors: &floors,
            },
            &rows,
        )
        .unwrap();
        let sizes = super::super::flexible::resolve_tracks_from_phases(
            super::super::flexible::ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &rows,
                available: AvailableOf::MAX_CONTENT,
                stretch_size: None,
                gap: S::ZERO,
                alignment: AlignContent::Start,
                gutters: None,
            },
        )
        .unwrap();
        assert_eq!(phases.base_sizes[0], base);
        assert_eq!(phases.base_sizes[1], scalar(4.0));
        assert_eq!(sizes, vec![base, scalar(4.0)]);
        assert_eq!(
            phases
                .base_sizes
                .iter()
                .copied()
                .fold(S::ZERO, |sum, value| sum + value),
            target,
        );
        assert_eq!(track_sum_with_gutters(&sizes, S::ZERO, None), target);
    }

    #[test]
    fn quantized_phase_retains_feasible_increase_f32() {
        assert_quantized_phase_retains_feasible_increase::<f32>(33_554_432.0);
    }

    #[test]
    fn quantized_phase_retains_feasible_increase_f64() {
        assert_quantized_phase_retains_feasible_increase::<f64>(18_014_398_509_481_984.0);
    }

    fn phase_target_fixture<S: LayoutScalar>(
        tracks: &[TrackSizingOf<S>],
        floors: &[S],
        target: S,
    ) -> (GridAxisTrackPhases<S>, Vec<S>) {
        let rows = [GridIntrinsicContributionRow {
            span: GridTrackSpan::new(1, tracks.len() + 1),
            contributions: GridTrackContributions {
                minimum: target,
                min_content: target,
                max_content: target,
            },
        }];
        let phases = resolve_grid_contribution_rows(
            SpanningTrackContext {
                tracks,
                basis: None,
                available: AvailableOf::MAX_CONTENT,
                gap: S::ZERO,
                gutters: None,
                base_floors: floors,
            },
            &rows,
        )
        .unwrap();
        let sizes = super::super::flexible::resolve_tracks_from_phases(
            super::super::flexible::ResolvedAxisTrackInput {
                tracks,
                phases: &phases,
                items: &rows,
                available: AvailableOf::MAX_CONTENT,
                stretch_size: None,
                gap: S::ZERO,
                alignment: AlignContent::Start,
                gutters: None,
            },
        )
        .unwrap();
        (phases, sizes)
    }

    fn assert_phase_keeps_individual_plateau_advances<S: LayoutScalar>(base: S) {
        let scalar = S::from_f64;
        let target = base + scalar(4.0);
        assert_eq!(scalar(2.0) + base, base);
        assert_eq!((scalar(2.0) + scalar(2.0)) + base, target);
        let tracks = [
            TrackSizingOf::AUTO,
            TrackSizingOf::AUTO,
            TrackSizingOf::px(base),
        ];
        let (phases, sizes) = phase_target_fixture(&tracks, &[S::ZERO; 3], target);
        let expected = vec![scalar(2.0), scalar(2.0), base];
        assert_eq!(phases.base_sizes, expected);
        assert_eq!(sizes, expected);
        assert_eq!(track_sum_with_gutters(&sizes, S::ZERO, None), target);
    }

    fn assert_phase_keeps_greatest_feasible_no_exact_fold<S: LayoutScalar>(half_ulp: S, ulp: S) {
        let target = S::ONE + ulp;
        // The two consecutive free values around1 straddle an absent target
        // image under the real prefix's nearest-even addition.
        assert_eq!(half_ulp + S::ONE, S::ONE);
        assert_eq!(half_ulp + (S::ONE + ulp), S::ONE + (ulp + ulp));
        let tracks = [TrackSizingOf::px(half_ulp), TrackSizingOf::AUTO];
        let (phases, sizes) = phase_target_fixture(&tracks, &[S::ZERO; 2], target);
        assert_eq!(phases.base_sizes, vec![half_ulp, S::ONE]);
        assert_eq!(sizes, vec![half_ulp, S::ONE]);
        let occupied = track_sum_with_gutters(&sizes, S::ZERO, None);
        assert_eq!(occupied, S::ONE);
        assert_eq!(target - occupied, ulp);
        assert!(
            target - occupied > S::ZERO,
            "unpaid represented residual is not reported as spent"
        );
    }

    #[test]
    fn phase_keeps_individual_plateau_advances_f32() {
        assert_phase_keeps_individual_plateau_advances::<f32>(33_554_432.0);
    }
    #[test]
    fn phase_keeps_individual_plateau_advances_f64() {
        assert_phase_keeps_individual_plateau_advances::<f64>(18_014_398_509_481_984.0);
    }
    #[test]
    fn phase_keeps_greatest_feasible_no_exact_fold_f32() {
        assert_phase_keeps_greatest_feasible_no_exact_fold::<f32>(
            1.0 / 16_777_216.0,
            1.0 / 8_388_608.0,
        );
    }
    #[test]
    fn phase_keeps_greatest_feasible_no_exact_fold_f64() {
        assert_phase_keeps_greatest_feasible_no_exact_fold::<f64>(
            1.0 / 9_007_199_254_740_992.0,
            1.0 / 4_503_599_627_370_496.0,
        );
    }

    fn assert_many_auto_plateau_reaches_target<S: LayoutScalar>(base: S) {
        let scalar = S::from_f64;
        let count = 4096;
        let nominal = scalar(4.0) / S::from_usize(count);
        assert!(nominal > S::ZERO);
        assert_eq!(nominal * S::from_usize(count), scalar(4.0));
        assert_eq!(base + nominal, base);
        let target = base + scalar(4.0);
        let tracks = vec![TrackSizingOf::AUTO; count];
        let mut floors = vec![S::ZERO; count];
        floors[0] = base;
        let (phases, sizes) = phase_target_fixture(&tracks, &floors, target);
        assert!(phases.base_sizes[0] >= base);
        assert_eq!(sizes, phases.base_sizes);
        assert!(
            phases
                .base_sizes
                .iter()
                .skip(1)
                .all(|&size| size >= nominal),
            "the completed nominal round's legitimate trailing shares survive the residual sweep",
        );
        assert_eq!(
            phases
                .base_sizes
                .iter()
                .copied()
                .fold(S::ZERO, |sum, size| sum + size),
            target,
        );
        assert_eq!(track_sum_with_gutters(&sizes, S::ZERO, None), target);
    }

    #[test]
    fn many_auto_plateau_reaches_target_f32() {
        assert_many_auto_plateau_reaches_target::<f32>(33_554_432.0);
    }

    #[test]
    fn many_auto_plateau_reaches_target_f64() {
        assert_many_auto_plateau_reaches_target::<f64>(18_014_398_509_481_984.0);
    }

    fn assert_effective_fixed_maximum_caps<S: LayoutScalar>() {
        let n = S::from_f64;
        for raw in [false, true] {
            let tracks = [
                if raw {
                    TrackSizingOf::minmax(
                        MinTrackSizingOf::px(n(100.0)),
                        MaxTrackSizingOf::px(n(10.0)),
                    )
                } else {
                    TrackSizingOf::px(n(100.0))
                },
                TrackSizingOf::minmax(MinTrackSizingOf::AUTO, MaxTrackSizingOf::px(n(10.0))),
            ];
            let authored = tracks.clone();
            assert_eq!(
                effective_fixed_track_maximum(&tracks[0], None).unwrap(),
                Some(n(100.0))
            );
            assert_eq!(
                GridMinimumTrackFunctions::new(&tracks, None)
                    .span_facts(S::ZERO)
                    .unwrap()
                    .fixed_max_span(),
                Some(n(110.0))
            );
            for (available, minimum, expected_second) in [
                (AvailableOf::MIN_CONTENT, S::ZERO, n(10.0)),
                (AvailableOf::MAX_CONTENT, S::ZERO, n(10.0)),
                (AvailableOf::Definite(n(100.0)), n(110.0), n(10.0)),
                (AvailableOf::Definite(n(100.0)), S::ZERO, S::ZERO),
            ] {
                let rows = [
                    GridIntrinsicContributionRow {
                        span: GridTrackSpan::new(1, 3),
                        contributions: GridTrackContributions {
                            minimum,
                            min_content: n(120.0),
                            max_content: n(120.0),
                        },
                    },
                    GridIntrinsicContributionRow {
                        span: GridTrackSpan::new(2, 3),
                        contributions: GridTrackContributions {
                            minimum: S::ZERO,
                            min_content: S::ZERO,
                            max_content: S::ZERO,
                        },
                    },
                ];
                let phases = resolve_grid_contribution_rows(
                    SpanningTrackContext {
                        tracks: &tracks,
                        basis: None,
                        available,
                        gap: S::ZERO,
                        gutters: None,
                        base_floors: &[],
                    },
                    &rows,
                )
                .unwrap();
                // Actual occupied fixed100 is counted once. Formal intrinsic
                // content120 is capped at effective fixed maxima100+10;
                // definite used-minimum0 still permits no second-track base.
                assert_eq!(phases.base_sizes, [n(100.0), expected_second]);
            }
            assert_eq!(
                tracks, authored,
                "qualification does not rewrite authored functions"
            );
        }
        let track = TrackSizingOf::minmax(
            MinTrackSizingOf::Calculation(SizingCalculationOf::value(
                LengthPercentageOf::from_coefficients(S::ZERO, n(0.2)).unwrap(),
            )),
            MaxTrackSizingOf::px(n(10.0)),
        );
        assert_eq!(
            effective_fixed_track_maximum(&track, None).unwrap(),
            Some(n(10.0))
        );
        assert_eq!(
            effective_fixed_track_maximum(&track, Some(n(100.0))).unwrap(),
            Some(n(20.0))
        );
        assert!(matches!(
            effective_fixed_track_maximum(&track, Some(S::INFINITY)),
            Err(LengthResolutionStatus::InvalidNumeric { .. })
        ));
        assert_eq!(
            effective_fixed_track_maximum(&percentage_track(n(0.2)), None).unwrap(),
            None
        );
    }
    #[test]
    fn effective_fixed_maximum_caps_f32() {
        assert_effective_fixed_maximum_caps::<f32>();
    }
    #[test]
    fn effective_fixed_maximum_caps_f64() {
        assert_effective_fixed_maximum_caps::<f64>();
    }
}
