use super::{
    AtomicInlineParticipation, AvailableOf, Clear, DefaultScalar, Direction, Edges,
    InlineBoundaryEdgeOf, InlineBoundaryKind, InlineBreakOpportunity, InlineMetricsOf,
    LayoutScalar, Size, TextAlign, VerticalAlign, WritingMode,
};
#[cfg(test)]
use crate::Point;
use crate::geometry::FlowAxes;
#[cfg(test)]
use crate::geometry::{LogicalPointOf, LogicalSizeOf};
#[cfg(test)]
use std::cell::Cell;
use std::ops::Range;
use surgeist_text::{BidiLevel, SuppliedInlineParticipant, SuppliedLine};

mod input;

pub(crate) use input::{
    InlineParticipantKindOf, InlineParticipantProjection, continues_run, run_ranges, starts_run,
};

#[cfg(test)]
thread_local! {
    static INLINE_CANDIDATE_SCAN_VISITS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn observe_inline_candidate_scan_visit() {
    INLINE_CANDIDATE_SCAN_VISITS.with(|visits| visits.set(visits.get() + 1));
}

#[cfg(test)]
pub(super) fn reset_inline_candidate_scan_visits() {
    INLINE_CANDIDATE_SCAN_VISITS.with(|visits| visits.set(0));
}

#[cfg(test)]
pub(super) fn inline_candidate_scan_visits() -> usize {
    INLINE_CANDIDATE_SCAN_VISITS.with(Cell::get)
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct InlineRunInput<S: LayoutScalar = DefaultScalar> {
    pub available_width: AvailableOf<S>,
    pub writing_mode: WritingMode,
    pub direction: Direction,
    pub items: Vec<InlineParticipant<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MixedInlineParticipantOf<S: LayoutScalar = DefaultScalar> {
    Atomic {
        item: AtomicInlineBoxParticipant<S>,
        participation: AtomicInlineParticipation,
    },
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MixedInlineRunInputOf<S: LayoutScalar = DefaultScalar> {
    pub line_box_context: crate::line_box::LineBoxContext,
    pub observe_tail: bool,
    pub available_inline_extent: AvailableOf<S>,
    pub flow_axes: FlowAxes,
    pub text_align: TextAlign,
    pub text_align_last: crate::TextAlignLast,
    pub indent: crate::text_indent::ResolvedTextIndent<S>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub participants: Vec<MixedInlineParticipantOf<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineSourceOf<S: LayoutScalar = DefaultScalar> {
    pub item: AtomicInlineBoxParticipant<S>,
    pub inline_start: S,
    pub block_start: S,
    pub line_index: usize,
    /// Present only when this geometry came from the numeric mixed-order owner.
    pub visual_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PostLineClearIntent {
    None,
    LineStart,
    LineEnd,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LogicalLineBandQueryResultOf<S: LayoutScalar = DefaultScalar> {
    pub inline_start: S,
    pub inline_end: S,
    pub next_transition: Option<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LogicalLineBandOf<S: LayoutScalar = DefaultScalar> {
    pub inline_start: S,
    pub inline_end: S,
    pub block_start: S,
    pub block_end: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineControlSourceOf<S: LayoutScalar = DefaultScalar> {
    pub kind: InlineParticipantLayoutKind,
    pub source_index: usize,
    pub inline_start: S,
    pub inline_extent: S,
    pub endpoint: Option<InlineEndpointOf<S>>,
    pub block_start: S,
    pub line_index: usize,
    pub visual_index: Option<usize>,
}

/// Actual endpoint components and their own physical-side basis. Its strip
/// occupies one contribution; a Text interaction fragment is a separate view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineEndpointOf<S: LayoutScalar> {
    pub kind: InlineBoundaryKind,
    pub axes: FlowAxes,
    pub edge: InlineBoundaryEdgeOf<S>,
}

impl<S: LayoutScalar> InlineEndpointOf<S> {
    pub(super) fn from_input(input: crate::InlineBoundaryInputOf<S>) -> Self {
        Self {
            kind: input.kind(),
            axes: FlowAxes::new(input.writing_mode(), input.direction()),
            edge: input.edge(),
        }
    }

    pub(super) fn physical_components(self) -> (Edges<S>, Edges<S>) {
        let logical = |value| match self.kind {
            InlineBoundaryKind::Start => {
                crate::geometry::LogicalEdgesOf::new(value, S::ZERO, S::ZERO, S::ZERO)
            }
            InlineBoundaryKind::End => {
                crate::geometry::LogicalEdgesOf::new(S::ZERO, value, S::ZERO, S::ZERO)
            }
        };
        (
            self.axes.physical_edges(logical(self.edge.border())),
            self.axes.physical_edges(logical(self.edge.padding())),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MixedInlineRunReportOf<S: LayoutScalar = DefaultScalar> {
    pub formatted_lines: Option<Vec<crate::text_layout::formatted::NumericFormattedLine>>,
    pub line_box_context: crate::line_box::LineBoxContext,
    pub lines: Vec<crate::hyphenation::CommittedInlineLine>,
    pub hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub indent_state: crate::text_indent::IndentLineState,
    pub inline_extent: S,
    pub block_extent: S,
    pub float_edge_phase: Option<S>,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub atomics: Vec<AtomicInlineSourceOf<S>>,
    pub controls: Vec<InlineControlSourceOf<S>>,
    pub post_line_clear_intents: Vec<PostLineClearIntent>,
    pub line_bands: Vec<LogicalLineBandOf<S>>,
}

#[derive(Clone, Copy, Default)]
struct InlineSourceCursor {
    source: usize,
    descriptor: usize,
}

#[derive(Clone, Copy)]
struct ProjectedInlineRun<'a, S: LayoutScalar> {
    participants: &'a [MixedInlineParticipantOf<S>],
    supplied: &'a [SuppliedInlineParticipant],
}

#[derive(Clone, Copy)]
struct InlineLineInput<'a, S: LayoutScalar> {
    participants: &'a [MixedInlineParticipantOf<S>],
    supplied: &'a [SuppliedInlineParticipant],
    descriptor_start: usize,
}

impl<'a, S: LayoutScalar> ProjectedInlineRun<'a, S> {
    // A selected range never contains a forced break. The paired cursor skips
    // only those controls, so the two slices have exactly the same ordinal map.
    fn line(self, source: Range<usize>, descriptor_start: usize) -> InlineLineInput<'a, S> {
        let participants = &self.participants[source];
        InlineLineInput {
            participants,
            supplied: &self.supplied[descriptor_start..descriptor_start + participants.len()],
            descriptor_start,
        }
    }
}

fn project_inline_participants<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
) -> Vec<SuppliedInlineParticipant> {
    participants
        .iter()
        .filter_map(|participant| {
            #[cfg(test)]
            observe_inline_work_visit(0);
            match *participant {
                MixedInlineParticipantOf::Atomic { participation, .. } => {
                    Some(SuppliedInlineParticipant::Atomic {
                        level: participation.bidi_level(),
                    })
                }
                MixedInlineParticipantOf::Boundary(control) => {
                    Some(SuppliedInlineParticipant::Boundary {
                        level: BidiLevel::try_new(u8::from(
                            control.flow().direction() == Direction::Rtl,
                        ))
                        .expect("base direction has a valid resolved level"),
                    })
                }
                MixedInlineParticipantOf::ForcedLineBreak(_) => None,
            }
        })
        .collect()
}

#[cfg(test)]
thread_local! {
    // Actual source entries projected, source entries searched for hard line ends,
    // and calls into Text's leading-edge query, respectively.
    static INLINE_WORK_VISITS: Cell<[usize; 3]> = const { Cell::new([0; 3]) };
    static INLINE_LEADING_QUERY_UNITS: Cell<usize> = const { Cell::new(0) };
}
#[cfg(test)]
fn observe_inline_work_visit(kind: usize) {
    INLINE_WORK_VISITS.with(|visits| {
        let mut counts = visits.get();
        counts[kind] += 1;
        visits.set(counts);
    });
}
#[cfg(test)]
pub(super) fn reset_inline_work_visits() {
    INLINE_WORK_VISITS.with(|visits| visits.set([0; 3]));
    INLINE_LEADING_QUERY_UNITS.with(|visits| visits.set(0));
}
#[cfg(test)]
pub(super) fn inline_work_visits() -> [usize; 3] {
    INLINE_WORK_VISITS.with(Cell::get)
}
#[cfg(test)]
pub(super) fn inline_leading_query_units() -> usize {
    INLINE_LEADING_QUERY_UNITS.with(Cell::get)
}

#[derive(Clone, Copy)]
struct SelectedInlineUnitOf<S: LayoutScalar> {
    participant: MixedInlineParticipantOf<S>,
    discarded: bool,
}

#[derive(Clone)]
struct SelectedInlineLineOf<S: LayoutScalar> {
    ending: surgeist_text::LineEnd,
    alignment: TextAlign,
    indent: S,
    units: Vec<SelectedInlineUnitOf<S>>,
    descriptor_range: Range<usize>,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    post_line_clear_intent: PostLineClearIntent,
    baseline: S,
    after_baseline: S,
    fallback_line_band: Option<InlineMetricContributionOf<S>>,
    used_inline_extent: S,
    split_baseline_edge_phase: Option<S>,
    inline_start_override: Option<S>,
    uses_float_strut_phase: bool,
    band: LogicalLineBandOf<S>,
}

#[derive(Clone, Copy)]
struct InlineLineSummaryOf<S: LayoutScalar> {
    discarded_start_end: usize,
    discarded_end_start: usize,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    post_line_clear_intent: PostLineClearIntent,
    baseline: S,
    after_baseline: S,
    fallback_line_band: Option<InlineMetricContributionOf<S>>,
    used_inline_extent: S,
    split_baseline_edge_phase: Option<S>,
}

impl<S: LayoutScalar> InlineLineSummaryOf<S> {
    fn is_discarded(self, index: usize) -> bool {
        index < self.discarded_start_end || index >= self.discarded_end_start
    }
}

/// Actual selection outcome, independent of whether a hard ending has its own
/// control occurrence. Mandatory following-atomic breaks have no such control.
#[derive(Clone, Copy, PartialEq)]
enum SelectedInlineLineEnd {
    Soft,
    Forced,
    Paragraph,
}

#[derive(Clone)]
struct SelectedInlineLineCandidateOf<S: LayoutScalar> {
    end: SelectedInlineLineEnd,
    line: SelectedInlineLineOf<S>,
    next_cursor: InlineSourceCursor,
    pending_strut: Option<InlineMetricContributionOf<S>>,
}

#[derive(Clone, Copy)]
struct InlineMetricContributionOf<S: LayoutScalar> {
    baseline: S,
    after_baseline: S,
}

impl<S: LayoutScalar> InlineMetricContributionOf<S> {
    fn extent(self) -> S {
        self.baseline + self.after_baseline
    }
}

#[derive(Clone, Copy)]
struct InlineLineMetricGroupsOf<S: LayoutScalar> {
    baseline: S,
    after_baseline: S,
    zero_after_baseline_peak: S,
    has_positive_after_baseline: bool,
    line_over_extent: S,
    line_under_extent: S,
}

impl<S: LayoutScalar> InlineLineMetricGroupsOf<S> {
    fn from_strut(strut: Option<InlineMetricContributionOf<S>>) -> Self {
        Self {
            baseline: strut.map_or(S::ZERO, |metrics| metrics.baseline),
            after_baseline: strut.map_or(S::ZERO, |metrics| metrics.after_baseline),
            zero_after_baseline_peak: strut
                .filter(|metrics| metrics.after_baseline == S::ZERO)
                .map_or(S::ZERO, |metrics| metrics.baseline),
            has_positive_after_baseline: strut
                .is_some_and(|metrics| metrics.after_baseline > S::ZERO),
            line_over_extent: S::ZERO,
            line_under_extent: S::ZERO,
        }
    }

    fn include(
        &mut self,
        alignment: InlineControlAlignment,
        metrics: InlineMetricContributionOf<S>,
    ) {
        match alignment {
            InlineControlAlignment::Baseline => {
                self.baseline = self.baseline.max(metrics.baseline);
                self.after_baseline = self.after_baseline.max(metrics.after_baseline);
                if metrics.after_baseline == S::ZERO {
                    self.zero_after_baseline_peak =
                        self.zero_after_baseline_peak.max(metrics.baseline);
                } else {
                    self.has_positive_after_baseline = true;
                }
            }
            InlineControlAlignment::Top => {
                self.line_over_extent = self
                    .line_over_extent
                    .max(metrics.baseline + metrics.after_baseline);
            }
            InlineControlAlignment::Bottom => {
                self.line_under_extent = self
                    .line_under_extent
                    .max(metrics.baseline + metrics.after_baseline);
            }
        }
    }

    fn resolve(self) -> InlineMetricContributionOf<S> {
        let line_under = (self.baseline + self.after_baseline).max(self.line_over_extent);
        let line_over = (line_under - self.line_under_extent).min(S::ZERO);
        InlineMetricContributionOf {
            baseline: self.baseline - line_over,
            after_baseline: line_under - self.baseline,
        }
    }

    fn split_baseline_edge_phase(self) -> Option<S> {
        let phase_span = self.baseline + self.zero_after_baseline_peak;
        (self.has_positive_after_baseline
            && self.zero_after_baseline_peak > S::ZERO
            && phase_span > S::ZERO)
            .then(|| self.zero_after_baseline_peak / phase_span)
    }
}

impl<S: LayoutScalar> MixedInlineParticipantOf<S> {
    fn following_break(self) -> Option<InlineBreakOpportunity> {
        match self {
            Self::Atomic { participation, .. } => Some(participation.following_break()),
            Self::ForcedLineBreak(_) | Self::Boundary(_) => None,
        }
    }

    fn inline_advance(self, flow_axes: FlowAxes) -> S {
        match self {
            Self::Atomic { item, .. } => {
                let margin = flow_axes.logical_edges(item.margin);
                flow_axes.logical_size(item.size).inline + margin.inline_sum()
            }
            Self::ForcedLineBreak(_) => S::ZERO,
            Self::Boundary(control) => control.edge().advance(),
        }
    }

    fn metrics(self, flow_axes: FlowAxes) -> InlineMetricContributionOf<S> {
        match self {
            Self::Atomic { item, .. } => item.metrics(flow_axes),
            Self::ForcedLineBreak(control) => {
                let metrics = control.metrics();
                InlineMetricContributionOf {
                    baseline: metrics.baseline(),
                    after_baseline: metrics.after_baseline(),
                }
            }
            Self::Boundary(control) => {
                let metrics = control.metrics();
                InlineMetricContributionOf {
                    baseline: metrics.baseline(),
                    after_baseline: metrics.after_baseline(),
                }
            }
        }
    }

    fn alignment(self) -> InlineControlAlignment {
        match self {
            Self::Atomic { item, .. } => item.alignment,
            Self::ForcedLineBreak(control) => control.alignment(),
            Self::Boundary(control) => control.alignment(),
        }
    }
}

fn resolve_completed_fallback_envelope<S: LayoutScalar>(
    line: &mut SelectedInlineLineOf<S>,
    flow_axes: FlowAxes,
    uses_float_band: bool,
) {
    let Some(fallback_band) = line.fallback_line_band else {
        return;
    };
    let has_fallback_atomic = line.units.iter().copied().any(|unit| {
        !unit.discarded
            && matches!(
                unit.participant,
                MixedInlineParticipantOf::Atomic { item, .. }
                    if item
                        .fallback_block_start_in_band(flow_axes, fallback_band)
                        .is_some()
            )
    });
    if !has_fallback_atomic {
        return;
    }

    if uses_float_band {
        line.baseline = fallback_band.baseline;
        line.after_baseline = fallback_band.after_baseline;
        return;
    }

    line.fallback_line_band = None;
}

#[must_use]
pub(super) const fn mapped_post_line_clear_intent(
    flow_axes: FlowAxes,
    clear: Clear,
) -> PostLineClearIntent {
    let _ = flow_axes.inline_start();
    match clear {
        Clear::None => PostLineClearIntent::None,
        Clear::Left => PostLineClearIntent::LineStart,
        Clear::Right => PostLineClearIntent::LineEnd,
        Clear::Both => PostLineClearIntent::Both,
    }
}

fn summarize_inline_line<S: LayoutScalar>(
    input: InlineLineInput<'_, S>,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> InlineLineSummaryOf<S> {
    let participants = input.participants;
    let edges = SuppliedLine::new(input.supplied).edges();
    let mut metric_groups = InlineLineMetricGroupsOf::from_strut(strut);
    if let Some(control) = line_break {
        metric_groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    let fallback_line_band =
        (strut.is_some() || line_break.is_some()).then(|| metric_groups.resolve());
    let mut used_inline_extent = S::ZERO;
    let discarded_start_end = edges.leading_discard_len();
    let discarded_end_start = edges.trailing_discard_start();
    let mut before_trailing_metric_groups = metric_groups;
    let mut before_trailing_inline_extent = used_inline_extent;
    for (index, participant) in participants.iter().copied().enumerate() {
        if index == discarded_end_start {
            before_trailing_metric_groups = metric_groups;
            before_trailing_inline_extent = used_inline_extent;
        }
        if index >= discarded_start_end {
            metric_groups.include(participant.alignment(), participant.metrics(flow_axes));
            used_inline_extent = used_inline_extent + participant.inline_advance(flow_axes);
        }
    }
    if discarded_end_start < participants.len() {
        metric_groups = before_trailing_metric_groups;
        used_inline_extent = before_trailing_inline_extent;
    }
    let metrics = metric_groups.resolve();

    InlineLineSummaryOf {
        discarded_start_end,
        discarded_end_start,
        line_break,
        post_line_clear_intent: line_break.map_or(PostLineClearIntent::None, |control| {
            mapped_post_line_clear_intent(flow_axes, control.clear())
        }),
        baseline: metrics.baseline,
        after_baseline: metrics.after_baseline,
        fallback_line_band,
        used_inline_extent,
        split_baseline_edge_phase: metric_groups.split_baseline_edge_phase(),
    }
}

fn select_inline_line<S: LayoutScalar>(
    input: InlineLineInput<'_, S>,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> SelectedInlineLineOf<S> {
    let participants = input.participants;
    let summary = summarize_inline_line(input, line_break, strut, flow_axes);
    let units = participants
        .iter()
        .copied()
        .enumerate()
        .map(|(index, participant)| SelectedInlineUnitOf {
            participant,
            discarded: summary.is_discarded(index),
        })
        .collect();

    SelectedInlineLineOf {
        ending: surgeist_text::LineEnd::Paragraph,
        alignment: TextAlign::Start,
        indent: S::ZERO,
        units,
        descriptor_range: input.descriptor_start..input.descriptor_start + participants.len(),
        line_break: summary.line_break,
        post_line_clear_intent: summary.post_line_clear_intent,
        baseline: summary.baseline,
        after_baseline: summary.after_baseline,
        fallback_line_band: summary.fallback_line_band,
        used_inline_extent: summary.used_inline_extent,
        split_baseline_edge_phase: summary.split_baseline_edge_phase,
        inline_start_override: None,
        uses_float_strut_phase: false,
        band: LogicalLineBandOf {
            inline_start: S::ZERO,
            inline_end: S::ZERO,
            block_start: S::ZERO,
            block_end: S::ZERO,
        },
    }
}

fn resolve_float_strut_phase<S: LayoutScalar>(
    line: &mut SelectedInlineLineOf<S>,
    carried_strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> Option<InlineMetricContributionOf<S>> {
    let mut metric_groups = InlineLineMetricGroupsOf::from_strut(None);
    let mut active_strut = carried_strut;
    let mut uses_phase = carried_strut.is_some();
    if let Some(control) = line.line_break {
        metric_groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    for selected in line.units.iter().copied().filter(|unit| !unit.discarded) {
        match selected.participant {
            MixedInlineParticipantOf::Boundary(control) => {
                uses_phase = true;
                active_strut = match control.kind() {
                    InlineBoundaryKind::Start => Some(InlineMetricContributionOf {
                        baseline: control.metrics().baseline(),
                        after_baseline: control.metrics().after_baseline(),
                    }),
                    InlineBoundaryKind::End => None,
                };
            }
            participant => {
                metric_groups.include(participant.alignment(), participant.metrics(flow_axes));
            }
        }
    }
    let metrics = metric_groups.resolve();
    let line_extent = active_strut.map_or(metrics.extent(), |strut| {
        metrics.extent().max(strut.extent())
    });
    line.baseline = metrics.baseline;
    line.after_baseline = line_extent - metrics.baseline;
    line.fallback_line_band = None;
    line.uses_float_strut_phase = uses_phase;
    active_strut
}

fn starts_inline_strut_phase<S: LayoutScalar>(line: &SelectedInlineLineOf<S>) -> bool {
    line.units.iter().any(|selected| {
        matches!(
            selected.participant,
            MixedInlineParticipantOf::Boundary(control)
                if control.kind() == InlineBoundaryKind::Start
        )
    })
}

fn inline_min_content<S: LayoutScalar>(run: ProjectedInlineRun<'_, S>, flow_axes: FlowAxes) -> S {
    let participants = run.participants;
    let mut descriptor = 0;
    let mut group_descriptor_start = 0;
    let mut maximum = S::ZERO;
    let mut group_start = 0;
    for (index, participant) in participants.iter().enumerate() {
        if matches!(participant, MixedInlineParticipantOf::ForcedLineBreak(_)) {
            maximum = maximum.max(
                summarize_inline_line(
                    run.line(group_start..index, group_descriptor_start),
                    None,
                    None,
                    flow_axes,
                )
                .used_inline_extent,
            );
            group_start = index + 1;
            group_descriptor_start = descriptor;
            continue;
        }
        descriptor += 1;
        let Some(following_break) = participant.following_break() else {
            continue;
        };
        if following_break == InlineBreakOpportunity::Prohibited {
            continue;
        }
        let line = summarize_inline_line(
            run.line(group_start..index + 1, group_descriptor_start),
            None,
            None,
            flow_axes,
        );
        maximum = maximum.max(line.used_inline_extent);
        group_start = index + 1;
        group_descriptor_start = descriptor;
    }
    if group_start < participants.len() {
        maximum = maximum.max(
            summarize_inline_line(
                run.line(group_start..participants.len(), group_descriptor_start),
                None,
                None,
                flow_axes,
            )
            .used_inline_extent,
        );
    }
    maximum
}

fn inline_max_content<S: LayoutScalar>(run: ProjectedInlineRun<'_, S>, flow_axes: FlowAxes) -> S {
    let participants = run.participants;
    let mut descriptor = 0;
    let mut group_descriptor_start = 0;
    let mut maximum = S::ZERO;
    let mut group_start = 0;
    for (index, participant) in participants.iter().enumerate() {
        if matches!(participant, MixedInlineParticipantOf::ForcedLineBreak(_)) {
            maximum = maximum.max(
                summarize_inline_line(
                    run.line(group_start..index, group_descriptor_start),
                    None,
                    None,
                    flow_axes,
                )
                .used_inline_extent,
            );
            group_start = index + 1;
            group_descriptor_start = descriptor;
            continue;
        }
        descriptor += 1;
        if participant.following_break() != Some(InlineBreakOpportunity::Mandatory) {
            continue;
        }
        maximum = maximum.max(
            summarize_inline_line(
                run.line(group_start..index + 1, group_descriptor_start),
                None,
                None,
                flow_axes,
            )
            .used_inline_extent,
        );
        group_start = index + 1;
        group_descriptor_start = descriptor;
    }
    if group_start < participants.len() {
        maximum = maximum.max(
            summarize_inline_line(
                run.line(group_start..participants.len(), group_descriptor_start),
                None,
                None,
                flow_axes,
            )
            .used_inline_extent,
        );
    }
    maximum
}

fn resolved_inline_available<S: LayoutScalar>(
    input: &MixedInlineRunInputOf<S>,
    supplied: &[SuppliedInlineParticipant],
) -> S {
    let run = ProjectedInlineRun {
        participants: &input.participants,
        supplied,
    };
    match input.available_inline_extent {
        AvailableOf::Definite(value) => value,
        AvailableOf::MinContent => inline_min_content(run, input.flow_axes),
        AvailableOf::MaxContent => inline_max_content(run, input.flow_axes),
    }
}

fn select_next_inline_line<S: LayoutScalar>(
    run: ProjectedInlineRun<'_, S>,
    cursor: InlineSourceCursor,
    next_boundary_descriptor: usize,
    available_inline_extent: S,
    wraps: bool,
    pending_strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> Option<SelectedInlineLineCandidateOf<S>> {
    let participants = run.participants;
    let source_cursor = cursor.source;
    if source_cursor == participants.len() {
        return pending_strut.map(|strut| SelectedInlineLineCandidateOf {
            end: SelectedInlineLineEnd::Paragraph,
            line: select_inline_line(
                run.line(source_cursor..source_cursor, cursor.descriptor),
                None,
                Some(strut),
                flow_axes,
            ),
            next_cursor: cursor,
            pending_strut: None,
        });
    }

    let mut scan = source_cursor;
    let mut latest_allowed = None;
    let mut candidate_inline_extent = S::ZERO;
    #[cfg(test)]
    observe_inline_work_visit(2);
    let leading_query = &run.supplied[cursor.descriptor..next_boundary_descriptor];
    #[cfg(test)]
    INLINE_LEADING_QUERY_UNITS.with(|units| units.set(units.get() + leading_query.len()));
    let leading_discard_len = SuppliedLine::new(leading_query).leading_discard_len();
    while scan < participants.len() {
        let participant = participants[scan];
        if let MixedInlineParticipantOf::ForcedLineBreak(control) = participant {
            return Some(SelectedInlineLineCandidateOf {
                end: SelectedInlineLineEnd::Forced,
                line: select_inline_line(
                    run.line(source_cursor..scan, cursor.descriptor),
                    Some(control),
                    pending_strut,
                    flow_axes,
                ),
                next_cursor: InlineSourceCursor {
                    source: scan + 1,
                    descriptor: cursor.descriptor + scan - source_cursor,
                },
                pending_strut: Some(participant.metrics(flow_axes)),
            });
        }

        #[cfg(test)]
        observe_inline_candidate_scan_visit();
        if scan - source_cursor >= leading_discard_len {
            candidate_inline_extent =
                candidate_inline_extent + participant.inline_advance(flow_axes);
        }
        if wraps
            && candidate_inline_extent > available_inline_extent
            && let Some(break_end) = latest_allowed
        {
            return Some(SelectedInlineLineCandidateOf {
                end: SelectedInlineLineEnd::Soft,
                line: select_inline_line(
                    run.line(source_cursor..break_end, cursor.descriptor),
                    None,
                    pending_strut,
                    flow_axes,
                ),
                next_cursor: InlineSourceCursor {
                    source: break_end,
                    descriptor: cursor.descriptor + break_end - source_cursor,
                },
                pending_strut: None,
            });
        }

        scan += 1;
        let Some(following_break) = participant.following_break() else {
            continue;
        };
        match following_break {
            InlineBreakOpportunity::Allowed => {
                latest_allowed = Some(scan);
            }
            InlineBreakOpportunity::Mandatory => {
                return Some(SelectedInlineLineCandidateOf {
                    end: SelectedInlineLineEnd::Forced,
                    line: select_inline_line(
                        run.line(source_cursor..scan, cursor.descriptor),
                        None,
                        pending_strut,
                        flow_axes,
                    ),
                    next_cursor: InlineSourceCursor {
                        source: scan,
                        descriptor: cursor.descriptor + scan - source_cursor,
                    },
                    pending_strut: Some(participant.metrics(flow_axes)),
                });
            }
            InlineBreakOpportunity::Prohibited => {}
        }
    }

    Some(SelectedInlineLineCandidateOf {
        end: SelectedInlineLineEnd::Paragraph,
        line: select_inline_line(
            run.line(source_cursor..participants.len(), cursor.descriptor),
            None,
            pending_strut,
            flow_axes,
        ),
        next_cursor: InlineSourceCursor {
            source: participants.len(),
            descriptor: run.supplied.len(),
        },
        pending_strut: None,
    })
}

fn text_line_offset<S: LayoutScalar>(
    used_inline_extent: S,
    available_inline_extent: S,
    flow_axes: FlowAxes,
    text_align: TextAlign,
) -> S {
    let free_space = (available_inline_extent - used_inline_extent).max(S::ZERO);
    let inline_decreases = flow_axes
        .logical_axis_progression(crate::LogicalAxis::Inline)
        .is_decreasing();
    match text_align {
        TextAlign::Start | TextAlign::Justify => S::ZERO,
        TextAlign::End => free_space,
        TextAlign::Left | TextAlign::LegacyLeft if inline_decreases => free_space,
        TextAlign::Right | TextAlign::LegacyRight if !inline_decreases => free_space,
        TextAlign::Center | TextAlign::LegacyCenter => free_space / S::from_f64(2.0),
        TextAlign::Left | TextAlign::Right | TextAlign::LegacyLeft | TextAlign::LegacyRight => {
            S::ZERO
        }
    }
}

fn selected_line_inline_start<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    flow_axes: FlowAxes,
    text_align: TextAlign,
    direction: Direction,
) -> S {
    let same_direction = direction == flow_axes.direction();
    let text_align = match (same_direction, text_align) {
        (false, TextAlign::Start | TextAlign::Justify) => TextAlign::End,
        (false, TextAlign::End) => TextAlign::Start,
        (_, alignment) => alignment,
    };
    line.inline_start_override.unwrap_or_else(|| {
        let capacity = line.band.inline_end - line.band.inline_start - line.indent;
        let free_space = capacity - line.used_inline_extent;
        // Overlong content aligns to the resolved line-box Start, regardless
        // of requested alignment. Opposed containing progression needs the
        // signed deficit to reach that edge; matching progression starts at0.
        let offset = if free_space < S::ZERO {
            if same_direction { S::ZERO } else { free_space }
        } else {
            text_line_offset(line.used_inline_extent, capacity, flow_axes, text_align)
        };
        line.band.inline_start + if same_direction { line.indent } else { S::ZERO } + offset
    })
}

fn resolved_float_terminal_block_extent<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    carried_strut: Option<InlineMetricContributionOf<S>>,
    float_transition: Option<S>,
    flow_axes: FlowAxes,
    line_inline_start: S,
    line_inline_end: S,
) -> S {
    let line_block_end = line.band.block_end;
    let Some(strut) = carried_strut.filter(|strut| strut.after_baseline > S::ZERO) else {
        return line_block_end;
    };
    let touches_float_edge = if flow_axes
        .logical_axis_progression(crate::LogicalAxis::Inline)
        .is_decreasing()
    {
        line_inline_start == line.band.inline_start
    } else {
        line_inline_end == line.band.inline_end
    };
    if float_transition != Some(line_block_end) || !touches_float_edge {
        return line_block_end;
    }
    let baseline_phase = (line.baseline - strut.baseline).max(S::ZERO) / strut.after_baseline;
    line_block_end + baseline_phase
}

fn resolved_inline_unit_slots<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    supplied: &[SuppliedInlineParticipant],
    flow_axes: FlowAxes,
    text_align: TextAlign,
    direction: Direction,
) -> (S, Vec<usize>, Vec<S>) {
    let line_inline_start = selected_line_inline_start(line, flow_axes, text_align, direction);
    let mut order = vec![0; line.units.len()];
    let visual_order = SuppliedLine::new(supplied)
        .reorder_into(&mut order)
        .expect("selected descriptor range has exactly one slot per selected unit");
    let mut visual_indices = vec![0; line.units.len()];
    let mut inline_starts = vec![S::ZERO; line.units.len()];
    for (visual_index, (source_index, _)) in visual_order.iter().enumerate() {
        visual_indices[source_index] = visual_index;
    }
    let mut inline_start = line_inline_start;
    for (source_index, _) in visual_order.iter() {
        let selected = line.units[source_index];
        let selected_inline_extent = if selected.discarded {
            S::ZERO
        } else {
            selected.participant.inline_advance(flow_axes)
        };
        inline_starts[source_index] = inline_start;
        inline_start = inline_start + selected_inline_extent;
    }
    (line_inline_start, visual_indices, inline_starts)
}

#[must_use]
#[cfg(test)]
pub(super) fn layout_mixed_inline_run<S: LayoutScalar>(
    input: MixedInlineRunInputOf<S>,
) -> MixedInlineRunReportOf<S> {
    let supplied = project_inline_participants(&input.participants);
    let available = resolved_inline_available(&input, &supplied);
    layout_mixed_inline_run_from_available(
        input,
        supplied,
        available,
        |_, _| LogicalLineBandQueryResultOf {
            inline_start: S::ZERO,
            inline_end: available,
            next_transition: None,
        },
        |block, _| block,
        false,
    )
}

#[must_use]
pub(super) fn layout_mixed_inline_run_with_band_source<S, BandSource, ClearSource>(
    input: MixedInlineRunInputOf<S>,
    band_source: BandSource,
    clear_source: ClearSource,
) -> MixedInlineRunReportOf<S>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
{
    let supplied = project_inline_participants(&input.participants);
    let available = resolved_inline_available(&input, &supplied);
    layout_mixed_inline_run_from_available(
        input,
        supplied,
        available,
        band_source,
        clear_source,
        false,
    )
}

pub(super) fn layout_mixed_inline_run_with_formatted_observation<S, BandSource, ClearSource>(
    input: MixedInlineRunInputOf<S>,
    band_source: BandSource,
    clear_source: ClearSource,
    observe: bool,
) -> MixedInlineRunReportOf<S>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
{
    if !observe {
        return layout_mixed_inline_run_with_band_source(input, band_source, clear_source);
    }
    let supplied = project_inline_participants(&input.participants);
    let available = resolved_inline_available(&input, &supplied);
    layout_mixed_inline_run_from_available(
        input,
        supplied,
        available,
        band_source,
        clear_source,
        true,
    )
}

fn layout_mixed_inline_run_from_available<S, BandSource, ClearSource>(
    input: MixedInlineRunInputOf<S>,
    supplied: Vec<SuppliedInlineParticipant>,
    available: S,
    mut band_source: BandSource,
    mut clear_source: ClearSource,
    observe_formatted: bool,
) -> MixedInlineRunReportOf<S>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
{
    let wraps = !matches!(input.available_inline_extent, AvailableOf::MaxContent);
    let mut selected_lines = Vec::new();
    let mut formatted_lines = observe_formatted.then(Vec::new);
    let mut cursor = InlineSourceCursor::default();
    let run = ProjectedInlineRun {
        participants: &input.participants,
        supplied: &supplied,
    };
    let mut boundary_descriptor = 0;
    // Mandatory following breaks also delimit a line. Looking only for forced
    // controls would repeatedly query a suffix of discardable mandatory units.
    let mut line_boundaries = input
        .participants
        .iter()
        .enumerate()
        .filter_map(|(index, participant)| {
            #[cfg(test)]
            observe_inline_work_visit(1);
            let is_forced = matches!(participant, MixedInlineParticipantOf::ForcedLineBreak(_));
            if !is_forced {
                boundary_descriptor += 1;
            }
            (is_forced
                || participant
                    .following_break()
                    .is_some_and(|following| following == InlineBreakOpportunity::Mandatory))
            .then_some(InlineSourceCursor {
                source: index + 1,
                descriptor: boundary_descriptor,
            })
        })
        .peekable();
    let mut indent_state = input.indent_state;
    let mut line_box_context = input.line_box_context;
    // This route has only atomic/control occurrences, so every actual line is
    // weak. Its fallback is constant until a prepared Text run supplies strong facts.
    let line_direction = line_box_context.weak_direction();
    let mut pending_strut = None;
    let mut block_cursor = S::ZERO;
    let mut float_strut = None;
    let mut continuation_inline_cursor = None;
    let mut advanced_phase_transition = false;
    let mut resolved_terminal_block_extent = S::ZERO;
    let mut float_edge_phase = None;

    loop {
        let line_indent = input.indent.at(indent_state);
        let next_boundary_descriptor = line_boundaries
            .peek()
            .map_or(supplied.len(), |boundary| boundary.descriptor);
        let Some(mut provisional) = select_next_inline_line(
            run,
            cursor,
            next_boundary_descriptor,
            available - line_indent,
            wraps,
            pending_strut,
            input.flow_axes,
        ) else {
            break;
        };
        if float_strut.is_some() {
            resolve_float_strut_phase(&mut provisional.line, float_strut, input.flow_axes);
        }
        let provisional_block_end =
            block_cursor + provisional.line.baseline + provisional.line.after_baseline;
        let queried_band = band_source(block_cursor, provisional_block_end);
        let band_inline_end = queried_band.inline_end.max(queried_band.inline_start);
        let band_available = band_inline_end - queried_band.inline_start;
        let has_float_transition = queried_band.next_transition.is_some();

        if band_available == S::ZERO
            && provisional.line.used_inline_extent > S::ZERO
            && let Some(next_transition) = queried_band
                .next_transition
                .filter(|transition| transition.is_finite() && *transition > block_cursor)
        {
            block_cursor = next_transition;
            continue;
        }

        let use_containing_band = band_available == S::ZERO
            && provisional.line.used_inline_extent > S::ZERO
            && queried_band.next_transition.is_none();
        let (band_inline_start, band_inline_end, mut selected) = if use_containing_band {
            (S::ZERO, available, provisional)
        } else {
            let selected = select_next_inline_line(
                run,
                cursor,
                next_boundary_descriptor,
                band_available - line_indent,
                wraps,
                pending_strut,
                input.flow_axes,
            )
            .expect("a provisional line remains selectable against its queried band");
            (queried_band.inline_start, band_inline_end, selected)
        };
        if selected.line.used_inline_extent > band_inline_end - band_inline_start - line_indent
            && let Some(next_transition) = queried_band
                .next_transition
                .filter(|transition| transition.is_finite() && *transition > block_cursor)
        {
            // An indivisible first group cannot fit this genuine float band.
            // Re-select at its next real transition; an ordinary fitting break
            // above already remains selected and does not enter this retry.
            block_cursor = next_transition;
            continue;
        }
        let next_float_strut = if float_strut.is_some()
            || (has_float_transition && starts_inline_strut_phase(&selected.line))
        {
            resolve_float_strut_phase(&mut selected.line, float_strut, input.flow_axes)
        } else {
            float_strut
        };
        resolve_completed_fallback_envelope(
            &mut selected.line,
            input.flow_axes,
            has_float_transition,
        );
        let selected_block_end =
            block_cursor + selected.line.baseline + selected.line.after_baseline;
        let continuation_overflows = continuation_inline_cursor
            .is_some_and(|cursor| cursor + selected.line.used_inline_extent > band_inline_end);
        if !advanced_phase_transition
            && float_strut.is_some()
            && continuation_overflows
            && let Some(next_transition) = queried_band.next_transition.filter(|transition| {
                transition.is_finite()
                    && *transition > block_cursor
                    && *transition < selected_block_end
            })
        {
            block_cursor = next_transition;
            advanced_phase_transition = true;
            continue;
        }

        let mut line = selected.line;
        line.indent = line_indent;
        if advanced_phase_transition && continuation_inline_cursor.is_some() {
            line.inline_start_override = Some(band_inline_start + line_indent);
        }
        line.band = LogicalLineBandOf {
            inline_start: band_inline_start,
            inline_end: band_inline_end,
            block_start: block_cursor,
            block_end: selected_block_end,
        };
        if line_boundaries
            .peek()
            .is_some_and(|boundary| boundary.source == selected.next_cursor.source)
        {
            let consumed = line_boundaries
                .next()
                .expect("selected hard line end is present in source");
            debug_assert_eq!(consumed.descriptor, selected.next_cursor.descriptor);
        }
        cursor = selected.next_cursor;
        line.alignment = input.text_align.for_line(
            input.text_align_last,
            selected.end != SelectedInlineLineEnd::Soft,
        );
        if line.alignment == TextAlign::Justify {
            line.alignment = input.text_align.unexpandable(input.text_align_last);
        }
        line.ending = match selected.end {
            SelectedInlineLineEnd::Soft => surgeist_text::LineEnd::Soft,
            SelectedInlineLineEnd::Forced => surgeist_text::LineEnd::Forced,
            SelectedInlineLineEnd::Paragraph => surgeist_text::LineEnd::Paragraph,
        };
        if let Some(lines) = &mut formatted_lines {
            lines.push(crate::text_layout::formatted::NumericFormattedLine {
                source_end: selected.next_cursor.source,
                descriptor_end: selected.next_cursor.descriptor,
                ending: line.ending,
            });
        }
        indent_state.commit(selected.end == SelectedInlineLineEnd::Forced);
        line_box_context.commit(line_box_context.resolve(None));
        pending_strut = selected.pending_strut;
        let line_block_end = block_cursor + line.baseline + line.after_baseline;
        let line_inline_start =
            selected_line_inline_start(&line, input.flow_axes, line.alignment, line_direction);
        let line_inline_end = line_inline_start + line.used_inline_extent;
        if has_float_transition && line.split_baseline_edge_phase.is_some() {
            float_edge_phase = line.split_baseline_edge_phase;
        }
        resolved_terminal_block_extent = resolved_float_terminal_block_extent(
            &line,
            float_strut,
            queried_band.next_transition,
            input.flow_axes,
            line_inline_start,
            line_inline_end,
        );
        continuation_inline_cursor = float_strut.is_some().then_some(line_inline_end);
        float_strut = next_float_strut;
        advanced_phase_transition = false;
        block_cursor =
            clear_source(line_block_end, line.post_line_clear_intent).max(line_block_end);
        selected_lines.push(line);
    }

    let mut inline_extent = S::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut atomics = Vec::new();
    let mut controls = Vec::new();
    let mut post_line_clear_intents = Vec::new();
    let mut line_bands = Vec::new();
    let mut lines = Vec::new();
    for (line_index, line) in selected_lines.into_iter().enumerate() {
        if input.observe_tail {
            lines.push(crate::hyphenation::CommittedInlineLine {
                end: line.ending,
                content: line.units.iter().any(|unit| {
                    !unit.discarded
                        && matches!(unit.participant, MixedInlineParticipantOf::Atomic { .. })
                }),
                hyphenated: false,
            });
        }
        let block_start = line.band.block_start;
        let line_baseline = block_start + line.baseline;
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);
        let (line_inline_start, visual_indices, inline_starts) = resolved_inline_unit_slots(
            &line,
            &supplied[line.descriptor_range.clone()],
            input.flow_axes,
            line.alignment,
            line_direction,
        );
        inline_extent = inline_extent.max(line_inline_start + line.used_inline_extent);
        post_line_clear_intents.push(line.post_line_clear_intent);
        line_bands.push(line.band);
        for (source_index, selected) in line.units.into_iter().enumerate() {
            let inline_start = inline_starts[source_index];
            match selected.participant {
                MixedInlineParticipantOf::Atomic { item, .. } => {
                    let logical_margin = input.flow_axes.logical_edges(item.margin);
                    let logical_size = input.flow_axes.logical_size(item.size);
                    let line_block_extent = line.baseline + line.after_baseline;
                    let item_block_start = match item.alignment {
                        InlineControlAlignment::Baseline => line
                            .fallback_line_band
                            .and_then(|band| {
                                item.fallback_block_start_in_band(input.flow_axes, band)
                            })
                            .map_or_else(
                                || line_baseline - item.baseline_offset(input.flow_axes),
                                |offset| block_start + offset,
                            ),
                        InlineControlAlignment::Top => block_start - logical_margin.block_start,
                        InlineControlAlignment::Bottom => {
                            block_start + line_block_extent
                                - logical_margin.block_end
                                - logical_size.block
                        }
                    };
                    atomics.push(AtomicInlineSourceOf {
                        item,
                        inline_start: inline_start + logical_margin.inline_start,
                        block_start: item_block_start,
                        line_index,
                        visual_index: Some(visual_indices[source_index]),
                    });
                }
                MixedInlineParticipantOf::Boundary(control) => {
                    let control_block = if line.uses_float_strut_phase {
                        block_start + control.metrics().baseline()
                    } else {
                        control_block_position(
                            control.alignment(),
                            control.metrics(),
                            block_start,
                            line_baseline,
                            line.baseline + line.after_baseline,
                        )
                    };
                    controls.push(InlineControlSourceOf {
                        kind: inline_boundary_layout_kind(control.kind()),
                        source_index: control.source_index(),
                        inline_start,
                        inline_extent: control.edge().advance(),
                        endpoint: Some(InlineEndpointOf {
                            kind: control.kind(),
                            axes: control.flow().flow_axes,
                            edge: control.edge(),
                        }),
                        block_start: control_block,
                        line_index,
                        visual_index: Some(visual_indices[source_index]),
                    });
                }
                MixedInlineParticipantOf::ForcedLineBreak(_) => {
                    unreachable!("visible line breaks commit before line reordering")
                }
            }
        }
        if let Some(control) = line.line_break {
            let control_block = control_block_position(
                control.alignment(),
                control.metrics(),
                block_start,
                line_baseline,
                line.baseline + line.after_baseline,
            );
            controls.push(InlineControlSourceOf {
                kind: InlineParticipantLayoutKind::ForcedLineBreak,
                source_index: control.source_index(),
                inline_start: line_inline_start + line.used_inline_extent,
                inline_extent: S::ZERO,
                endpoint: None,
                block_start: control_block,
                line_index,
                visual_index: None,
            });
        }
    }

    MixedInlineRunReportOf {
        formatted_lines,
        line_box_context,
        lines,
        hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
        indent_state,
        inline_extent,
        block_extent: block_cursor.max(resolved_terminal_block_extent),
        float_edge_phase,
        first_baseline,
        last_baseline,
        atomics,
        controls,
        post_line_clear_intents,
        line_bands,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineBoxParticipant<S: LayoutScalar = DefaultScalar> {
    pub source_index: usize,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
    pub first_baseline: Option<S>,
    pub alignment: InlineControlAlignment,
}

impl<S: LayoutScalar> AtomicInlineBoxParticipant<S> {
    fn baseline_offset(self, flow_axes: FlowAxes) -> S {
        let logical_size = flow_axes.logical_size(self.size);
        let logical_margin = flow_axes.logical_edges(self.margin);
        self.first_baseline
            .unwrap_or(logical_size.block + logical_margin.block_end)
    }

    fn metrics(self, flow_axes: FlowAxes) -> InlineMetricContributionOf<S> {
        let logical_size = flow_axes.logical_size(self.size);
        let logical_margin = flow_axes.logical_edges(self.margin);
        InlineMetricContributionOf {
            baseline: logical_margin.block_start + self.baseline_offset(flow_axes),
            after_baseline: self.first_baseline.map_or(S::ZERO, |inner| {
                logical_size.block - inner + logical_margin.block_end
            }),
        }
    }

    fn fallback_block_start_in_band(
        self,
        flow_axes: FlowAxes,
        line_band: InlineMetricContributionOf<S>,
    ) -> Option<S> {
        if self.alignment != InlineControlAlignment::Baseline || self.first_baseline.is_some() {
            return None;
        }
        let logical_size = flow_axes.logical_size(self.size);
        let logical_margin = flow_axes.logical_edges(self.margin);
        let margin_box_extent =
            logical_margin.block_start + logical_size.block + logical_margin.block_end;
        let fallback_metrics = self.metrics(flow_axes);
        (margin_box_extent >= S::ZERO
            && margin_box_extent < line_band.extent()
            && fallback_metrics.baseline > line_band.baseline)
            .then(|| {
                (line_band.extent() - margin_box_extent) / S::from_f64(2.0)
                    + logical_margin.block_start
            })
    }
}

fn control_block_position<S: LayoutScalar>(
    alignment: InlineControlAlignment,
    metrics: InlineMetricsOf<S>,
    line_block_start: S,
    line_baseline: S,
    line_block_extent: S,
) -> S {
    match alignment {
        InlineControlAlignment::Baseline => line_baseline,
        InlineControlAlignment::Top => line_block_start + metrics.baseline(),
        InlineControlAlignment::Bottom => {
            line_block_start + line_block_extent - metrics.after_baseline()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineFlowOf<S: LayoutScalar = DefaultScalar> {
    flow_axes: FlowAxes,
    available_inline_extent: AvailableOf<S>,
}

impl<S: LayoutScalar> InlineFlowOf<S> {
    #[must_use]
    pub(super) const fn new(
        writing_mode: WritingMode,
        direction: Direction,
        available_inline_extent: AvailableOf<S>,
    ) -> Self {
        Self {
            flow_axes: FlowAxes::new(writing_mode, direction),
            available_inline_extent,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(super) const fn writing_mode(self) -> WritingMode {
        self.flow_axes.writing_mode()
    }

    #[must_use]
    pub(super) const fn direction(self) -> Direction {
        self.flow_axes.direction()
    }

    #[cfg(test)]
    #[must_use]
    pub(super) const fn available_inline_extent(self) -> AvailableOf<S> {
        self.available_inline_extent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InlineControlAlignment {
    Baseline,
    Top,
    Bottom,
}

impl From<VerticalAlign> for InlineControlAlignment {
    fn from(value: VerticalAlign) -> Self {
        match value {
            VerticalAlign::Baseline => Self::Baseline,
            VerticalAlign::Top => Self::Top,
            VerticalAlign::Bottom => Self::Bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ForcedLineBreakControlOf<S: LayoutScalar = DefaultScalar> {
    source_index: usize,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
    clear: Clear,
}

impl<S: LayoutScalar> ForcedLineBreakControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        source_index: usize,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment,
        clear: Clear,
    ) -> Self {
        Self {
            source_index,
            flow,
            metrics,
            alignment,
            clear,
        }
    }

    #[must_use]
    pub(super) const fn source_index(self) -> usize {
        self.source_index
    }

    #[cfg(test)]
    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }

    #[must_use]
    pub(super) const fn clear(self) -> Clear {
        self.clear
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineBoundaryControlOf<S: LayoutScalar = DefaultScalar> {
    source_index: usize,
    kind: InlineBoundaryKind,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment,
    edge: InlineBoundaryEdgeOf<S>,
}

impl<S: LayoutScalar> InlineBoundaryControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        source_index: usize,
        kind: InlineBoundaryKind,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment,
        edge: InlineBoundaryEdgeOf<S>,
    ) -> Self {
        Self {
            source_index,
            kind,
            flow,
            metrics,
            alignment,
            edge,
        }
    }

    #[must_use]
    pub(super) const fn source_index(self) -> usize {
        self.source_index
    }

    #[must_use]
    pub(super) const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[must_use]
    pub(super) const fn edge(self) -> InlineBoundaryEdgeOf<S> {
        self.edge
    }

    #[must_use]
    pub(super) const fn flow(self) -> InlineFlowOf<S> {
        self.flow
    }

    #[must_use]
    pub(super) const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }

    #[must_use]
    pub(super) const fn alignment(self) -> InlineControlAlignment {
        self.alignment
    }
}

#[cfg(test)]
impl<S: LayoutScalar> InlineParticipant<S> {
    pub(super) const fn new(
        source_index: usize,
        size: Size<S>,
        margin: Edges<S>,
        first_baseline: Option<S>,
    ) -> Self {
        Self::Box(AtomicInlineBoxParticipant {
            source_index,
            size,
            content_size: size,
            margin,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline,
            alignment: InlineControlAlignment::Baseline,
        })
    }

    #[must_use]
    pub(super) const fn forced_line_break(control: ForcedLineBreakControlOf<S>) -> Self {
        Self::ForcedLineBreak(control)
    }

    #[must_use]
    pub(super) const fn inline_boundary(control: InlineBoundaryControlOf<S>) -> Self {
        Self::Boundary(control)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineParticipant<S: LayoutScalar = DefaultScalar> {
    Box(AtomicInlineBoxParticipant<S>),
    ForcedLineBreak(ForcedLineBreakControlOf<S>),
    Boundary(InlineBoundaryControlOf<S>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InlineParticipantLayoutKind {
    SemanticControl,
    #[cfg(test)]
    Box,
    ForcedLineBreak,
    InlineBoundaryStart,
    InlineBoundaryEnd,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineParticipantLayoutItem<S: LayoutScalar = DefaultScalar> {
    pub kind: InlineParticipantLayoutKind,
    pub source_index: usize,
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct InlineRunReport<S: LayoutScalar = DefaultScalar> {
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub first_baseline: Option<S>,
    pub last_baseline: Option<S>,
    pub items: Vec<InlineParticipantLayoutItem<S>>,
}

fn inline_boundary_layout_kind(kind: InlineBoundaryKind) -> InlineParticipantLayoutKind {
    match kind {
        InlineBoundaryKind::Start => InlineParticipantLayoutKind::InlineBoundaryStart,
        InlineBoundaryKind::End => InlineParticipantLayoutKind::InlineBoundaryEnd,
    }
}

#[cfg(test)]
#[must_use]
pub(super) fn layout_inline_run<S: LayoutScalar>(input: InlineRunInput<S>) -> InlineRunReport<S> {
    let flow_axes = FlowAxes::new(input.writing_mode, input.direction);
    let participants = input
        .items
        .into_iter()
        .map(|participant| match participant {
            InlineParticipant::Box(item) => MixedInlineParticipantOf::Atomic {
                item,
                participation: AtomicInlineParticipation::new(
                    super::BidiLevel::try_new(0).expect("base bidi level is valid"),
                    InlineBreakOpportunity::Allowed,
                ),
            },
            InlineParticipant::ForcedLineBreak(control) => {
                MixedInlineParticipantOf::ForcedLineBreak(control)
            }
            InlineParticipant::Boundary(control) => MixedInlineParticipantOf::Boundary(control),
        })
        .collect();
    let report = layout_mixed_inline_run(MixedInlineRunInputOf {
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            flow_axes.direction(),
        ),
        observe_tail: false,
        available_inline_extent: input.available_width,
        flow_axes,
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants,
    });
    let logical_report_size = LogicalSizeOf::new(report.inline_extent, report.block_extent);
    let report_size = flow_axes.physical_size(logical_report_size);
    let mut items = report
        .atomics
        .into_iter()
        .map(|source| {
            let logical_size = flow_axes.logical_size(source.item.size);
            InlineParticipantLayoutItem {
                kind: InlineParticipantLayoutKind::Box,
                source_index: source.item.source_index,
                location: flow_axes.physical_point(
                    LogicalPointOf::new(source.inline_start, source.block_start),
                    logical_size,
                    report_size,
                ),
                size: source.item.size,
                content_size: source.item.content_size,
                margin: source.item.margin,
                padding: source.item.padding,
                border: source.item.border,
                scrollbar_size: source.item.scrollbar_size,
            }
        })
        .chain(report.controls.into_iter().map(|source| {
            InlineParticipantLayoutItem {
                kind: source.kind,
                source_index: source.source_index,
                location: flow_axes.physical_point(
                    LogicalPointOf::new(source.inline_start, source.block_start),
                    LogicalSizeOf::new(source.inline_extent, S::ZERO),
                    report_size,
                ),
                size: flow_axes.physical_size(LogicalSizeOf::new(source.inline_extent, S::ZERO)),
                content_size: Size::ZERO,
                margin: Edges::ZERO,
                padding: source
                    .endpoint
                    .map_or(Edges::ZERO, |endpoint| endpoint.physical_components().1),
                border: source
                    .endpoint
                    .map_or(Edges::ZERO, |endpoint| endpoint.physical_components().0),
                scrollbar_size: Size::ZERO,
            }
        }))
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.source_index);

    InlineRunReport {
        size: report_size,
        content_size: report_size,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
        items,
    }
}

#[cfg(test)]
#[must_use]
pub(super) fn inline_run_min_content_width<S: LayoutScalar>(items: &[InlineParticipant<S>]) -> S {
    layout_inline_run(InlineRunInput {
        available_width: AvailableOf::MIN_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: items.to_vec(),
    })
    .size
    .width
}

#[must_use]
#[cfg(test)]
pub(super) fn inline_run_max_content_width<S: LayoutScalar>(items: &[InlineParticipant<S>]) -> S {
    layout_inline_run(InlineRunInput {
        available_width: AvailableOf::MAX_CONTENT,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        items: items.to_vec(),
    })
    .size
    .width
}
