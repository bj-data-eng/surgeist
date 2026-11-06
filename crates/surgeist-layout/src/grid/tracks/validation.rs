use super::repeat_extent::RepeatExtent;
use super::*;
use core::cmp::Ordering;

use crate::{
    LengthResolutionOf, LengthResolutionStatus, PercentageBasisOf, SizingCalculationOf,
    TrackRepetitionOf,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) struct AutoRepeatTrackOrigin {
    pub(in crate::grid) kind: TrackRepeat,
    pub(in crate::grid) repeat_group: usize,
    pub(in crate::grid) repetition_index: usize,
    pub(in crate::grid) track_index: usize,
}

pub(in crate::grid) fn resolve_track_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> LengthResolutionOf<S> {
    let basis = match basis {
        Some(value) => match PercentageBasisOf::definite(value) {
            Ok(basis) => basis,
            Err(_) => {
                return LengthResolutionOf::invalid_numeric(value, calculation.depends_on_basis());
            }
        },
        None => PercentageBasisOf::MISSING,
    };
    let resolution = calculation.resolve_against(basis);
    match resolution.status() {
        LengthResolutionStatus::Resolved => LengthResolutionOf::definite(
            resolution
                .value
                .expect("resolved sizing calculation must carry a value")
                .max(S::ZERO),
            calculation.depends_on_basis(),
        ),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::InvalidNumeric { .. } => {
            resolution
        }
        LengthResolutionStatus::NonNumeric => {
            unreachable!("a sizing calculation always has numeric program semantics")
        }
    }
}

pub(in crate::grid) fn track_has_percent_sizing<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    track.depends_on_basis()
}

pub(in crate::grid) fn extend_auto_tracks<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    auto_tracks: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    required_count: usize,
) -> Result<(), LengthResolutionStatus<S>> {
    let auto_tracks = expand_track_components(auto_tracks, basis, gap, None)?;
    let mut index = 0;
    while tracks.len() < required_count {
        let track = if auto_tracks.is_empty() {
            TrackSizingOf::AUTO
        } else {
            auto_tracks[index].clone()
        };
        tracks.push(track);
        if !auto_tracks.is_empty() {
            index = (index + 1) % auto_tracks.len();
        }
    }
    Ok(())
}

pub(in crate::grid) fn prepend_auto_tracks<S: LayoutScalar>(
    tracks: &mut Vec<TrackSizingOf<S>>,
    auto_tracks: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    required_count: usize,
    auto_fit_limit: Option<usize>,
) -> Result<(), LengthResolutionStatus<S>> {
    if required_count == 0 {
        return Ok(());
    }

    let auto_tracks = expand_track_components(auto_tracks, basis, gap, auto_fit_limit)?;
    if auto_tracks.is_empty() {
        tracks.extend(std::iter::repeat_n(TrackSizingOf::AUTO, required_count));
    } else {
        let phase = (auto_tracks.len() - required_count % auto_tracks.len()) % auto_tracks.len();
        tracks.extend(
            auto_tracks
                .iter()
                .cycle()
                .skip(phase)
                .take(required_count)
                .cloned(),
        );
    }
    tracks.rotate_right(required_count);
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct ExpandedTrackOf<S: LayoutScalar = Scalar> {
    pub(in crate::grid) sizing: TrackSizingOf<S>,
    pub(in crate::grid) auto_repeat: Option<AutoRepeatTrackOrigin>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct TrackExpansionOf<S: LayoutScalar = Scalar> {
    pub(in crate::grid) tracks: Vec<ExpandedTrackOf<S>>,
}

impl<S: LayoutScalar> TrackExpansionOf<S> {
    pub(in crate::grid) fn inherited(tracks: Vec<TrackSizingOf<S>>) -> Self {
        Self {
            tracks: tracks
                .into_iter()
                .map(|sizing| ExpandedTrackOf {
                    sizing,
                    auto_repeat: None,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::grid) enum AutoRepeatSizing {
    FitAvailable,
    SingleIntrinsicGroup,
    ResolvedGroupCount(usize),
}

#[derive(Clone, Copy)]
pub(in crate::grid) enum AutoRepeatConstraint<S> {
    One,
    Fit(S),
    FillMinimum(S),
}

impl AutoRepeatSizing {
    pub(in crate::grid) fn for_container(display: Display) -> Self {
        if display.establishes_grid_lanes_formatting_context() {
            Self::FitAvailable
        } else {
            Self::SingleIntrinsicGroup
        }
    }
}

pub(in crate::grid) fn expand_track_components_with_origins<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
    auto_repeat_sizing: AutoRepeatSizing,
) -> Result<TrackExpansionOf<S>, LengthResolutionStatus<S>> {
    expand_track_components_into(
        components,
        basis,
        gap,
        auto_fit_limit,
        auto_repeat_sizing,
        Vec::new(),
    )
}

pub(in crate::grid) fn expand_track_components_into<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
    auto_repeat_sizing: AutoRepeatSizing,
    mut tracks: Vec<ExpandedTrackOf<S>>,
) -> Result<TrackExpansionOf<S>, LengthResolutionStatus<S>> {
    debug_assert!(tracks.is_empty());
    if subgrid_components(components) {
        return Ok(TrackExpansionOf { tracks });
    }

    validate_track_components(components, basis)?;
    let mut auto_repeat_group = 0;
    let reserved = (!matches!(auto_repeat_sizing, AutoRepeatSizing::ResolvedGroupCount(_)))
        .then(|| reserved_track_space(components, basis, gap));
    for component in components {
        match component {
            TrackComponentOf::Track(sizing) => tracks.push(ExpandedTrackOf {
                sizing: sizing.clone(),
                auto_repeat: None,
            }),
            TrackComponentOf::Repeat(repetition) => {
                let repeated_tracks = repetition.sizing_tracks();
                let count = match repetition.repeat() {
                    TrackRepeat::Count(count) => count.get(),
                    // Ordinary Grid uses one group when a repeated track lacks
                    // a fixed count operand. A fixed minimum or maximum keeps
                    // numeric counting for that typed track.
                    kind @ (TrackRepeat::AutoFill | TrackRepeat::AutoFit) => {
                        match auto_repeat_sizing {
                            AutoRepeatSizing::ResolvedGroupCount(count) => count,
                            AutoRepeatSizing::SingleIntrinsicGroup
                                if repeated_tracks
                                    .iter()
                                    .any(track_needs_intrinsic_repeat_size) =>
                            {
                                1
                            }
                            _ => {
                                let count = auto_repeat_count(
                                    &repeated_tracks,
                                    basis,
                                    gap,
                                    reserved
                                        .expect("unresolved counts reserve the surrounding tracks"),
                                );
                                if kind == TrackRepeat::AutoFit {
                                    count.min(auto_fit_limit.unwrap_or(usize::MAX)).max(1)
                                } else {
                                    count
                                }
                            }
                        }
                    }
                };
                let repeat_group = auto_repeat_group;
                let auto_repeat_kind = match repetition.repeat() {
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit => {
                        auto_repeat_group += 1;
                        Some(repetition.repeat())
                    }
                    TrackRepeat::Count(_) => None,
                };
                if repeated_tracks.is_empty() {
                    continue;
                }
                for repetition_index in 0..count {
                    tracks.extend(repeated_tracks.iter().cloned().enumerate().map(
                        |(track_index, sizing)| ExpandedTrackOf {
                            sizing,
                            auto_repeat: auto_repeat_kind.map(|kind| AutoRepeatTrackOrigin {
                                kind,
                                repeat_group,
                                repetition_index,
                                track_index,
                            }),
                        },
                    ));
                }
            }
            TrackComponentOf::LineNames(_) => {}
            TrackComponentOf::Subgrid(_) => {
                unreachable!("subgrid templates return before expansion")
            }
        }
    }
    Ok(TrackExpansionOf { tracks })
}

pub(in crate::grid) fn expand_track_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    auto_fit_limit: Option<usize>,
) -> Result<Vec<TrackSizingOf<S>>, LengthResolutionStatus<S>> {
    Ok(expand_track_components_with_origins(
        components,
        basis,
        gap,
        auto_fit_limit,
        AutoRepeatSizing::FitAvailable,
    )?
    .tracks
    .into_iter()
    .map(|track| track.sizing)
    .collect())
}

fn validate_track_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    for component in components {
        match component {
            TrackComponentOf::Track(track) => validate_track_sizing(track, basis)?,
            TrackComponentOf::Repeat(repetition) => {
                validate_track_components(repetition.components(), basis)?;
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Ok(())
}

fn validate_track_sizing<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    if let MinTrackSizingOf::Calculation(calculation) = &track.min {
        validate_track_calculation(calculation, basis)?;
    }
    match &track.max {
        MaxTrackSizingOf::Calculation(calculation) | MaxTrackSizingOf::FitContent(calculation) => {
            validate_track_calculation(calculation, basis)?;
        }
        MaxTrackSizingOf::Flex(_)
        | MaxTrackSizingOf::Auto
        | MaxTrackSizingOf::MinContent
        | MaxTrackSizingOf::MaxContent => {}
    }
    Ok(())
}

fn validate_track_calculation<S: LayoutScalar>(
    calculation: &SizingCalculationOf<S>,
    basis: Option<S>,
) -> Result<(), LengthResolutionStatus<S>> {
    let resolution = resolve_track_calculation(calculation, basis);
    match resolution.status() {
        LengthResolutionStatus::InvalidNumeric { .. } => Err(resolution.status()),
        LengthResolutionStatus::Resolved | LengthResolutionStatus::MissingBasis => Ok(()),
        LengthResolutionStatus::NonNumeric => Err(LengthResolutionStatus::NonNumeric),
    }
}

pub(in crate::grid) fn track_expansion_basis<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    node_basis: Option<S>,
    available_basis: Option<S>,
) -> Option<S> {
    if subgrid_components(components) {
        return None;
    }

    node_basis.or_else(|| {
        auto_repeat_components(components)
            .then_some(available_basis)
            .flatten()
    })
}

pub(in crate::grid) fn subgrid_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> bool {
    components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Subgrid(_)))
}

pub(in crate::grid) fn auto_repeat_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> bool {
    components.iter().any(|component| {
        matches!(
            component,
            TrackComponentOf::Repeat(repetition)
                if matches!(repetition.repeat(), TrackRepeat::AutoFill | TrackRepeat::AutoFit)
        )
    })
}

fn track_needs_intrinsic_repeat_size<S: LayoutScalar>(track: &TrackSizingOf<S>) -> bool {
    !matches!(track.min, MinTrackSizingOf::Calculation(_))
        && !matches!(track.max, MaxTrackSizingOf::Calculation(_))
}

pub(in crate::grid) fn definite_repeat_track_size<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
) -> Option<S> {
    let min = match &track.min {
        MinTrackSizingOf::Calculation(length) => resolve_track_calculation(length, basis).value,
        _ => None,
    };
    let max = match &track.max {
        MaxTrackSizingOf::Calculation(length) => resolve_track_calculation(length, basis).value,
        _ => None,
    };
    max.zip(min).map(|(max, min)| max.max(min)).or(max).or(min)
}

fn single_automatic_repeat<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Option<(&TrackRepetitionOf<S>, bool)> {
    let mut automatic = None;
    for component in components {
        match component {
            TrackComponentOf::Repeat(repetition) => {
                let intrinsic = fixed_repeat_has_intrinsic_tracks(repetition.components())?;
                if matches!(
                    repetition.repeat(),
                    TrackRepeat::AutoFill | TrackRepeat::AutoFit
                ) {
                    if automatic.is_some() {
                        return None;
                    }
                    automatic = Some((repetition, intrinsic));
                }
            }
            TrackComponentOf::Subgrid(_) => return None,
            TrackComponentOf::Track(_) | TrackComponentOf::LineNames(_) => {}
        }
    }
    automatic
}

// The named context supports nested positive repeats and refuses nested
// automatic groups. Leave the latter to its existing diagnostic route.
fn fixed_repeat_has_intrinsic_tracks<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Option<bool> {
    let mut intrinsic = false;
    for component in components {
        match component {
            TrackComponentOf::Track(track) => intrinsic |= track_needs_intrinsic_repeat_size(track),
            TrackComponentOf::Repeat(repetition) => {
                if !matches!(repetition.repeat(), TrackRepeat::Count(_)) {
                    return None;
                }
                intrinsic |= fixed_repeat_has_intrinsic_tracks(repetition.components())?;
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Some(intrinsic)
}

pub(in crate::grid) fn intrinsic_auto_repeat_track_count<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Result<Option<usize>, SpanningContributionError<S>> {
    single_automatic_repeat(components)
        .filter(|(_, intrinsic)| *intrinsic)
        .map(|(repetition, _)| {
            checked_track_capacity(repetition.components(), AutomaticRepeatCapacity::FixedOnly)
        })
        .transpose()
}

pub(in crate::grid) fn unresolved_repeat_track_capacity<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> Result<Option<usize>, SpanningContributionError<S>> {
    if !components
        .iter()
        .any(|component| matches!(component, TrackComponentOf::Repeat(_)))
        || subgrid_components(components)
    {
        return Ok(None);
    }

    // Fallback expansion materializes at least one group for any automatic
    // repeat with tracks. Keep every fixed group in this minimum count.
    checked_track_capacity(components, AutomaticRepeatCapacity::Minimum).map(Some)
}

pub(in crate::grid) fn auto_repeat_track_capacity<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    count: usize,
) -> Result<usize, SpanningContributionError<S>> {
    if count == 0 || single_automatic_repeat(components).is_none() {
        return Err(SpanningContributionError::InvalidIncidence);
    }
    checked_track_capacity(components, AutomaticRepeatCapacity::Resolved(count))
}

pub(in crate::grid) fn standalone_auto_repeat_count<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
    constraint: AutoRepeatConstraint<S>,
) -> Result<Option<usize>, SpanningContributionError<S>> {
    let Some((repetition, intrinsic)) = single_automatic_repeat(components) else {
        return Ok(None);
    };
    let track_count =
        checked_track_capacity(repetition.components(), AutomaticRepeatCapacity::FixedOnly)?;
    if track_count == 0 {
        return Ok(None);
    }
    if intrinsic || matches!(constraint, AutoRepeatConstraint::One) {
        return Ok(Some(1));
    }
    let mut group = RepeatExtent::default();
    if !accumulate_counted_operands(repetition.components(), Some(1), &mut group, &|track| {
        definite_repeat_track_size(track, basis).map(|size| size.max(S::ONE))
    })? {
        return Ok(Some(1));
    }
    let reserved = standalone_reserved_track_space(components, track_count, basis, gap)?;
    repeat_count_from_extent(group, track_count, constraint, gap, &reserved).map(Some)
}

// Scale each operand by its authored multiplicity without expanding tracks.
// A name-only group may have no representable multiplicity and no operands.
fn accumulate_counted_operands<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    multiplicity: Option<usize>,
    extent: &mut RepeatExtent,
    track_operand: &impl Fn(&TrackSizingOf<S>) -> Option<S>,
) -> Result<bool, SpanningContributionError<S>> {
    for component in components {
        match component {
            TrackComponentOf::Track(track) => {
                let Some(value) = track_operand(track) else {
                    return Ok(false);
                };
                let count = multiplicity.ok_or(SpanningContributionError::Allocation)?;
                add_repeat_operand(extent, value, count)?;
            }
            TrackComponentOf::Repeat(repetition) => {
                let TrackRepeat::Count(count) = repetition.repeat() else {
                    return Err(SpanningContributionError::InvalidIncidence);
                };
                let multiplicity = multiplicity.and_then(|outer| outer.checked_mul(count.get()));
                if !accumulate_counted_operands(
                    repetition.components(),
                    multiplicity,
                    extent,
                    track_operand,
                )? {
                    return Ok(false);
                }
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }
    Ok(true)
}

fn add_repeat_operand<S: LayoutScalar>(
    extent: &mut RepeatExtent,
    value: S,
    count: usize,
) -> Result<(), SpanningContributionError<S>> {
    if !value.is_finite() {
        return Err(SpanningContributionError::InvalidNumeric { value });
    }
    extent.add(value.to_f64(), count);
    Ok(())
}

pub(in crate::grid) struct StandaloneReservedTrackSpace {
    count: usize,
    extent: RepeatExtent,
}

pub(in crate::grid) fn standalone_reserved_track_space<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    group_tracks: usize,
    basis: Option<S>,
    gap: S,
) -> Result<StandaloneReservedTrackSpace, SpanningContributionError<S>> {
    let count = auto_repeat_track_capacity(components, 1)?
        .checked_sub(group_tracks)
        .ok_or(SpanningContributionError::InvalidIncidence)?;
    let mut extent = RepeatExtent::default();
    for component in components {
        if matches!(component, TrackComponentOf::Repeat(repetition)
            if !matches!(repetition.repeat(), TrackRepeat::Count(_)))
        {
            continue;
        }
        accumulate_counted_operands(
            core::slice::from_ref(component),
            Some(1),
            &mut extent,
            &|track| Some(track_base_size(track, basis, S::ZERO)),
        )?;
    }
    add_repeat_operand(&mut extent, gap, count.saturating_sub(1))?;
    Ok(StandaloneReservedTrackSpace { count, extent })
}

#[derive(Clone, Copy)]
enum AutomaticRepeatCapacity {
    FixedOnly,
    Resolved(usize),
    Minimum,
}

fn checked_track_capacity<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    automatic: AutomaticRepeatCapacity,
) -> Result<usize, SpanningContributionError<S>> {
    let (tracks, entries) = checked_expanded_components(components, automatic)?;
    // Name-only repeats can keep one line while materializing many entries.
    core::alloc::Layout::array::<crate::grid::named::LineNameEntry>(entries)
        .map_err(|_| SpanningContributionError::Allocation)?;
    Ok(tracks)
}

fn checked_expanded_components<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    automatic: AutomaticRepeatCapacity,
) -> Result<(usize, usize), SpanningContributionError<S>> {
    let (auto_repeat_count, nested_automatic) = match automatic {
        AutomaticRepeatCapacity::FixedOnly => (None, AutomaticRepeatCapacity::FixedOnly),
        AutomaticRepeatCapacity::Resolved(count) => {
            (Some(count), AutomaticRepeatCapacity::FixedOnly)
        }
        AutomaticRepeatCapacity::Minimum => (Some(1), AutomaticRepeatCapacity::Minimum),
    };
    let allocation = SpanningContributionError::Allocation;
    let mut track_count = 0usize;
    let mut entry_count = 0usize;
    for component in components {
        let (tracks, entries) = match component {
            TrackComponentOf::Track(_) => (1, 1),
            TrackComponentOf::Repeat(repetition) => {
                let group_count = match repetition.repeat() {
                    TrackRepeat::Count(explicit_count) => explicit_count.get(),
                    TrackRepeat::AutoFit | TrackRepeat::AutoFill => {
                        auto_repeat_count.ok_or(SpanningContributionError::InvalidIncidence)?
                    }
                };
                let (tracks, entries) =
                    checked_expanded_components(repetition.components(), nested_automatic)?;
                let name_groups = if tracks == 0 {
                    group_count.min(1)
                } else {
                    group_count
                };
                (
                    tracks.checked_mul(group_count).ok_or(allocation)?,
                    entries
                        .checked_mul(name_groups)
                        .and_then(|count| count.checked_add(1))
                        .ok_or(allocation)?,
                )
            }
            TrackComponentOf::LineNames(names) => (0, names.len().max(1)),
            TrackComponentOf::Subgrid(_) => (0, 1),
        };
        track_count = track_count.checked_add(tracks).ok_or(allocation)?;
        entry_count = entry_count.checked_add(entries).ok_or(allocation)?;
    }
    Ok((track_count, entry_count))
}

pub(in crate::grid) fn hypothetical_repeat_groups<S: LayoutScalar>(
    largest_span: usize,
    group_tracks: usize,
) -> Result<usize, SpanningContributionError<S>> {
    if largest_span == 0 || group_tracks == 0 {
        return Err(SpanningContributionError::InvalidIncidence);
    }
    let largest_span =
        isize::try_from(largest_span).map_err(|_| SpanningContributionError::Allocation)?;
    let group_tracks =
        isize::try_from(group_tracks).map_err(|_| SpanningContributionError::Allocation)?;
    // A one-track span supplies a negative numerator. Euclidean division by
    // the positive group length rounds toward negative infinity.
    let count = largest_span
        .checked_sub(2)
        .and_then(|numerator| numerator.checked_div_euclid(group_tracks))
        .and_then(|quotient| quotient.checked_add(2))
        .ok_or(SpanningContributionError::Allocation)?;
    usize::try_from(count).map_err(|_| SpanningContributionError::Allocation)
}

pub(in crate::grid) fn tracks_need_available_basis<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
) -> bool {
    tracks
        .iter()
        .any(|track| matches!(track.max, MaxTrackSizingOf::Flex(_)))
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct ReservedTrackSpace<S: LayoutScalar = Scalar> {
    pub(in crate::grid) count: usize,
    pub(in crate::grid) size: S,
}

pub(in crate::grid) fn reserved_track_space<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
    basis: Option<S>,
    gap: S,
) -> ReservedTrackSpace<S> {
    let mut count = 0;
    let mut size = S::ZERO;
    for component in components {
        match component {
            TrackComponentOf::Track(track) => {
                count += 1;
                size = size + track_base_size(track, basis, S::ZERO);
            }
            TrackComponentOf::Repeat(repetition) => {
                if let TrackRepeat::Count(repeat_count) = repetition.repeat() {
                    let repeated_tracks = repetition.sizing_tracks();
                    count += repeat_count.get() * repeated_tracks.len();
                    size = size
                        + S::from_usize(repeat_count.get())
                            * repeated_tracks
                                .iter()
                                .map(|track| track_base_size(track, basis, S::ZERO))
                                .fold(S::ZERO, |sum, value| sum + value);
                }
            }
            TrackComponentOf::LineNames(_) | TrackComponentOf::Subgrid(_) => {}
        }
    }

    if count > 1 {
        size = size + gap * S::from_usize(count - 1);
    }

    ReservedTrackSpace { count, size }
}

pub(in crate::grid) fn auto_repeat_count<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
    gap: S,
    reserved: ReservedTrackSpace<S>,
) -> usize {
    let Some(basis) = basis else {
        return 1;
    };
    if tracks.is_empty() {
        return 0;
    }
    let track_sum = tracks
        .iter()
        .map(|track| track_base_size(track, Some(basis), S::ZERO).max(S::ONE))
        .fold(S::ZERO, |sum, value| sum + value);
    let (available, repeat_size) =
        repeat_count_operands(track_sum, tracks.len(), basis, gap, reserved);
    if repeat_size <= S::ZERO {
        1
    } else {
        (available / repeat_size)
            .floor()
            .max(S::ONE)
            .floor_to_usize_saturating()
    }
}

pub(in crate::grid) fn auto_repeat_count_from_hypothetical<S: LayoutScalar>(
    hypothetical: &[S],
    constraint: AutoRepeatConstraint<S>,
    gap: S,
    reserved: &StandaloneReservedTrackSpace,
) -> Result<usize, SpanningContributionError<S>> {
    if hypothetical.is_empty() {
        return Err(SpanningContributionError::InvalidIncidence);
    }
    let mut group = RepeatExtent::default();
    for &size in hypothetical {
        if !size.is_finite() {
            return Err(SpanningContributionError::InvalidNumeric { value: size });
        }
        group.add(size.max(S::ONE).to_f64(), 1);
    }
    repeat_count_from_extent(group, hypothetical.len(), constraint, gap, reserved)
}

fn repeat_count_from_extent<S: LayoutScalar>(
    mut group: RepeatExtent,
    track_count: usize,
    constraint: AutoRepeatConstraint<S>,
    gap: S,
    reserved: &StandaloneReservedTrackSpace,
) -> Result<usize, SpanningContributionError<S>> {
    let (bound, fit) = match constraint {
        AutoRepeatConstraint::Fit(bound) => (bound, true),
        AutoRepeatConstraint::FillMinimum(bound) => (bound, false),
        AutoRepeatConstraint::One => return Ok(1),
    };
    let mut available = reserved.extent.negated();
    add_repeat_operand(&mut available, bound, 1)?;
    // Each repeated track contributes a gap. Without surrounding tracks,
    // restore the final gap that does not appear in the topology.
    add_repeat_operand(&mut available, gap, usize::from(reserved.count == 0))?;
    add_repeat_operand(&mut group, gap, track_count)?;
    let crosses_bound = |count| {
        let comparison = group.compare_scaled(count, &available);
        if fit {
            comparison == Ordering::Greater
        } else {
            comparison != Ordering::Less
        }
    };
    if crosses_bound(1) || group.compare_scaled(1, &RepeatExtent::default()) != Ordering::Greater {
        return Ok(1);
    }

    let mut low = 1;
    let mut high = 2_usize;
    while !crosses_bound(high) {
        if high == usize::MAX {
            return Err(SpanningContributionError::Allocation);
        }
        low = high;
        high = high.saturating_mul(2);
    }
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if crosses_bound(middle) {
            high = middle;
        } else {
            low = middle;
        }
    }
    Ok(if fit { low } else { high })
}

fn repeat_count_operands<S: LayoutScalar>(
    track_sum: S,
    track_count: usize,
    basis: S,
    gap: S,
    reserved: ReservedTrackSpace<S>,
) -> (S, S) {
    let available = if reserved.count == 0 {
        basis + gap
    } else {
        basis - reserved.size
    };
    (available, track_sum + gap * S::from_usize(track_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hypothetical_group_count_floors_the_signed_span_ratio() {
        for (span, tracks, expected) in [(1, 1, 1), (1, 2, 1), (2, 2, 2), (3, 2, 2), (4, 2, 3)] {
            assert_eq!(
                hypothetical_repeat_groups::<f64>(span, tracks),
                Ok(expected)
            );
        }
        assert_eq!(
            hypothetical_repeat_groups::<f64>(1, 0),
            Err(SpanningContributionError::InvalidIncidence)
        );
        assert_eq!(
            hypothetical_repeat_groups::<f64>(usize::MAX, 2),
            Err(SpanningContributionError::Allocation)
        );
    }

    fn qualify_hypothetical_repeat_count<S: LayoutScalar>() {
        let scalar = S::from_f64;
        let hypothetical = [scalar(0.25), scalar(10.0)];
        let gap = scalar(2.0);
        let mut prefix = StandaloneReservedTrackSpace {
            count: 1,
            extent: RepeatExtent::default(),
        };
        prefix.extent.add(7.0, 1);
        let no_prefix = StandaloneReservedTrackSpace {
            count: 0,
            extent: RepeatExtent::default(),
        };
        // After flooring each track at 1px, a group plus its two gaps
        // takes 15px. A 7px prefix leaves room for three groups in 65px.
        assert_eq!(
            auto_repeat_count_from_hypothetical(
                &hypothetical,
                AutoRepeatConstraint::Fit(scalar(65.0)),
                gap,
                &prefix
            ),
            Ok(3)
        );
        // With no prefix, four groups take only seven gaps and fit in 58px.
        assert_eq!(
            auto_repeat_count_from_hypothetical(
                &hypothetical,
                AutoRepeatConstraint::Fit(scalar(58.0)),
                gap,
                &no_prefix
            ),
            Ok(4)
        );
        assert_eq!(
            auto_repeat_count_from_hypothetical(
                &hypothetical,
                AutoRepeatConstraint::Fit(scalar(5.0)),
                gap,
                &prefix
            ),
            Ok(1)
        );
        assert_eq!(
            auto_repeat_count_from_hypothetical(
                &[S::INFINITY],
                AutoRepeatConstraint::Fit(scalar(10.0)),
                gap,
                &no_prefix
            ),
            Err(SpanningContributionError::InvalidNumeric { value: S::INFINITY })
        );
        assert_eq!(
            auto_repeat_count_from_hypothetical(
                &[S::ONE],
                AutoRepeatConstraint::Fit(S::from_usize(usize::MAX)),
                S::ZERO,
                &no_prefix,
            ),
            Err(SpanningContributionError::Allocation)
        );
    }

    #[test]
    fn hypothetical_repeat_count_fits_gaps_and_checks_numeric_capacity() {
        qualify_hypothetical_repeat_count::<f32>();
        qualify_hypothetical_repeat_count::<f64>();
    }

    #[test]
    fn intrinsic_repeat_capacity_counts_tracks_and_rejects_overflow() {
        let components = [
            TrackComponentOf::px(7.0_f64),
            TrackComponentOf::Repeat(
                TrackRepetitionOf::count(2, vec![TrackSizingOf::px(4.0), TrackSizingOf::px(5.0)])
                    .unwrap(),
            ),
            TrackComponentOf::Repeat(
                TrackRepetitionOf::auto_fill_components(vec![
                    TrackComponentOf::line_names(["inside"]),
                    TrackComponentOf::px(10.0),
                    TrackComponentOf::Repeat(
                        TrackRepetitionOf::count(2, vec![TrackSizingOf::AUTO]).unwrap(),
                    ),
                ])
                .unwrap(),
            ),
        ];
        assert_eq!(intrinsic_auto_repeat_track_count(&components), Ok(Some(3)));
        assert_eq!(auto_repeat_track_capacity(&components, 3), Ok(14));
        assert_eq!(
            auto_repeat_track_capacity(&components, usize::MAX),
            Err(SpanningContributionError::Allocation)
        );
    }

    #[test]
    fn intrinsic_repeat_keeps_name_only_repetitions_on_one_line() {
        let name_only = TrackComponentOf::Repeat(
            TrackRepetitionOf::count_components(
                usize::MAX,
                vec![TrackComponentOf::<f64>::line_names(["lane"])],
            )
            .unwrap(),
        );
        let automatic = |components| {
            TrackComponentOf::Repeat(TrackRepetitionOf::auto_fill_components(components).unwrap())
        };
        for components in [
            vec![automatic(vec![
                name_only.clone(),
                TrackComponentOf::MAX_CONTENT,
            ])],
            vec![name_only, automatic(vec![TrackComponentOf::MAX_CONTENT])],
        ] {
            assert_eq!(intrinsic_auto_repeat_track_count(&components), Ok(Some(1)));
            assert_eq!(auto_repeat_track_capacity(&components, 1), Ok(1));
            let expansion = expand_track_components_with_origins(
                &components,
                Some(100.0),
                0.0,
                None,
                AutoRepeatSizing::ResolvedGroupCount(1),
            )
            .unwrap();
            let lines = crate::grid::named::named_lines_from_track_components(
                GridAxisKind::Column,
                &components,
                expansion.tracks.len(),
            )
            .unwrap();
            assert_eq!(expansion.tracks.len(), 1);
            assert_eq!(lines.named_occurrences("lane"), [1]);
            assert_eq!(lines.entries_on_line(1).len(), 1);
        }
    }
}
