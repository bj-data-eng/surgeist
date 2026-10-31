use super::{
    AtomicInlineParticipation, AvailableOf, Clear, DefaultScalar, Direction, Edges,
    InlineBoundaryEdgeOf, InlineBoundaryKind, InlineBreakOpportunity, InlineMetricsOf,
    LayoutScalar, Size, TextAlign, WritingMode,
};
use crate::Point;
#[cfg(test)]
use crate::geometry::LogicalSizeOf;
use crate::geometry::{FlowAxes, LogicalPointOf};
use crate::inline_wrapping::NumericWrappingError;
use crate::inline_wrapping::balance::{BalanceGoal, NaturalResidual};
use crate::inline_wrapping::quality::{LineQualityGoal, LineQualityKey};
#[cfg(test)]
use std::cell::Cell;
use std::ops::Range;
use surgeist_text::{BidiLevel, SuppliedInlineParticipant, SuppliedLine};

mod input;

#[cfg(test)]
mod baseline_offset_tests;
#[cfg(test)]
mod named_float_phase_tests;
#[cfg(test)]
mod no_strut_tests;
#[cfg(test)]
mod wrap_edges_tests;
#[cfg(test)]
mod wrap_inside_auto_tests;
#[cfg(test)]
mod wrap_inside_tests;

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
    pub inline_baseline_context: crate::InlineBaselineContext,
    pub groups: crate::group_alignment::Snapshot,
    pub text_wrap_style: crate::TextWrapStyle,
    pub line_box_context: crate::line_box::LineBoxContext,
    pub observe_tail: bool,
    pub wrapping: Option<crate::inline_wrapping::NumericWrapping>,
    pub available_inline_extent: AvailableOf<S>,
    pub flow_axes: FlowAxes,
    pub text_align: TextAlign,
    pub text_align_last: crate::TextAlignLast,
    pub indent: crate::text_indent::ResolvedTextIndent<S>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub participants: Vec<MixedInlineParticipantOf<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineSourcePositionOf<S: LayoutScalar = DefaultScalar> {
    /// Numeric run-local coordinates, before the parent-frame projection.
    Logical(LogicalPointOf<S>),
    /// Native parent-relative coordinates from the actual accepted frame.
    Physical(Point<S>),
}

#[cfg(test)]
pub(super) fn expect_logical_position<S: LayoutScalar>(
    position: InlineSourcePositionOf<S>,
) -> LogicalPointOf<S> {
    let InlineSourcePositionOf::Logical(position) = position else {
        panic!("a numeric source must retain its logical position");
    };
    position
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineSourceOf<S: LayoutScalar = DefaultScalar> {
    pub item: AtomicInlineBoxParticipant<S>,
    pub position: InlineSourcePositionOf<S>,
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
pub(super) struct NonFormattedInlineControlOf<S: LayoutScalar = DefaultScalar> {
    /// The original numerical witness remains real despite zero used height.
    pub control: InlineBoundaryControlOf<S>,
    pub inline_start: S,
    pub block_start: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InlineControlSourceOf<S: LayoutScalar = DefaultScalar> {
    pub kind: InlineParticipantLayoutKind,
    pub source_index: usize,
    pub position: InlineSourcePositionOf<S>,
    pub inline_extent: S,
    pub endpoint: Option<InlineEndpointOf<S>>,
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

    pub(super) fn from_control(control: InlineBoundaryControlOf<S>) -> Self {
        Self {
            kind: control.kind(),
            axes: control.flow().flow_axes,
            edge: control.edge(),
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

/// Accepted numeric run-local baseline distances before parent-frame projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NumericBaselineReportOf<S: LayoutScalar = DefaultScalar> {
    pub context: crate::InlineBaselineContext,
    pub first: Option<S>,
    pub last: Option<S>,
    pub first_named: bool,
    pub last_named: bool,
}

/// Physical baseline coordinates retain the frame in which they were obtained.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PhysicalBaselineReport {
    pub context: crate::InlineBaselineContext,
    pub first: Option<f64>,
    pub last: Option<f64>,
    pub first_named: bool,
    pub last_named: bool,
    pub frame: crate::block::baseline_frame::BaselineFrame,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum InlineBaselineReportOf<S: LayoutScalar = DefaultScalar> {
    Numeric(NumericBaselineReportOf<S>),
    Native(PhysicalBaselineReport),
}

#[cfg(test)]
impl<S: LayoutScalar> InlineBaselineReportOf<S> {
    pub(super) fn numeric(&self) -> &NumericBaselineReportOf<S> {
        let Self::Numeric(baselines) = self else {
            panic!("a numeric report must retain numeric baseline facts");
        };
        baselines
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MixedInlineRunReportOf<S: LayoutScalar = DefaultScalar> {
    pub baselines: InlineBaselineReportOf<S>,
    pub group_lines: Vec<crate::group_alignment::NaturalLine<S>>,
    pub formatted_lines: Option<Vec<crate::text_layout::formatted::NumericFormattedLine>>,
    pub line_box_context: crate::line_box::LineBoxContext,
    pub lines: Vec<crate::hyphenation::CommittedInlineLine>,
    pub hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub indent_state: crate::text_indent::IndentLineState,
    pub inline_extent: S,
    pub block_extent: S,
    pub float_edge_phase: Option<S>,
    pub atomics: Vec<AtomicInlineSourceOf<S>>,
    pub controls: Vec<InlineControlSourceOf<S>>,
    pub nonformatted_controls: Vec<NonFormattedInlineControlOf<S>>,
    pub post_line_clear_intents: Vec<PostLineClearIntent>,
    pub line_bands: Vec<LogicalLineBandOf<S>>,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct InlineSourceCursor {
    source: usize,
    descriptor: usize,
    edge_consumed: bool,
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

#[derive(Clone, Copy, PartialEq)]
struct SelectedInlineUnitOf<S: LayoutScalar> {
    participant: MixedInlineParticipantOf<S>,
    discarded: bool,
}

#[derive(Clone, PartialEq)]
struct SelectedInlineLineOf<S: LayoutScalar> {
    ending: surgeist_text::LineEnd,
    next_cursor: InlineSourceCursor,
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
    has_in_flow_content: bool,
    split_baseline_edge_phase: Option<S>,
    inline_start_override: Option<S>,
    incoming_strut: Option<ReferencedInlineStrutOf<S>>,
    scope_band: Option<LogicalLineBandQueryResultOf<S>>,
    band: LogicalLineBandOf<S>,
    numeric_error: Option<crate::group_alignment::NumericError>,
    has_displacement: bool,
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
    has_in_flow_content: bool,
    split_baseline_edge_phase: Option<S>,
    numeric_error: Option<crate::group_alignment::NumericError>,
    has_displacement: bool,
}

impl<S: LayoutScalar> InlineLineSummaryOf<S> {
    fn is_discarded(self, index: usize) -> bool {
        index < self.discarded_start_end || index >= self.discarded_end_start
    }
}

/// Actual selection outcome, independent of whether a hard ending has its own
/// control occurrence. Mandatory following-atomic breaks have no such control.
#[derive(Clone, Copy, Debug, PartialEq)]
enum SelectedInlineLineEnd {
    Soft,
    Forced,
    Paragraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumericLineDisposition {
    Formatted,
    NonFormatted,
}

impl NumericLineDisposition {
    fn for_line(ending: surgeist_text::LineEnd, has_in_flow_content: bool) -> Self {
        if ending == surgeist_text::LineEnd::Forced || has_in_flow_content {
            Self::Formatted
        } else {
            Self::NonFormatted
        }
    }
}

impl SelectedInlineLineEnd {
    fn line_end(self) -> surgeist_text::LineEnd {
        match self {
            Self::Soft => surgeist_text::LineEnd::Soft,
            Self::Forced => surgeist_text::LineEnd::Forced,
            Self::Paragraph => surgeist_text::LineEnd::Paragraph,
        }
    }
}

fn numeric_line_alignment(
    normal: TextAlign,
    last: crate::TextAlignLast,
    ending: SelectedInlineLineEnd,
) -> TextAlign {
    let alignment = normal.for_line(last, ending != SelectedInlineLineEnd::Soft);
    if alignment == TextAlign::Justify {
        normal.unexpandable(last)
    } else {
        alignment
    }
}

/// A numeric run separates adjoining margins only if it can produce an actual
/// formatted line. This uses the same participant semantics as line summaries;
/// numerical struts, a previous force and zero/nonzero advances are not roles.
pub(super) fn numeric_run_has_formatted_line<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    wrapping: Option<&crate::inline_wrapping::NumericWrapping>,
) -> bool {
    participants.iter().copied().any(|participant| {
        participant.has_in_flow_content()
            || matches!(participant, MixedInlineParticipantOf::ForcedLineBreak(_))
    }) || wrapping.is_some_and(|wrapping| wrapping.forced(0) || wrapping.next_forced(0).is_some())
}

#[derive(Clone)]
struct SelectedInlineLineCandidateOf<S: LayoutScalar> {
    end: SelectedInlineLineEnd,
    line: SelectedInlineLineOf<S>,
    next_cursor: InlineSourceCursor,
    pending_strut: Option<ReferencedInlineStrutOf<S>>,
}

enum InlineSelectionStep<S: LayoutScalar> {
    Selected(Option<SelectedInlineLineCandidateOf<S>>),
    RetryReceiver(ReferencedInlineStrutOf<S>),
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct InlineMetricContributionOf<S: LayoutScalar> {
    pub baseline: S,
    pub after_baseline: S,
}

/// An active Start retains raw sides and its canonical operation. Its local
/// baseline distance never replaces the selected line's common reference.
#[derive(Clone, Copy, PartialEq)]
struct ActiveInlineStrutOf<S: LayoutScalar> {
    metrics: InlineMetricContributionOf<S>,
    alignment: InlineControlAlignment<S>,
}

/// The actual reference travels with carried geometry through checkpoints.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ReferencedInlineStrutOf<S: LayoutScalar> {
    pub metrics: InlineMetricContributionOf<S>,
    pub reference: crate::InlineBaselineContext,
    pub source: InlineStrutSource,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum InlineStrutSource {
    Direct,
    Atomic { source_index: usize },
}

type AtomicReceiverSource<'a, S> = dyn FnMut(
        usize,
        crate::InlineBaselineContext,
    ) -> Result<ReferencedInlineStrutOf<S>, crate::inline_wrapping::NumericWrappingError>
    + 'a;

impl<S: LayoutScalar> InlineMetricContributionOf<S> {
    fn extent(self) -> S {
        self.baseline + self.after_baseline
    }

    fn post_baseline(
        self,
        alignment: InlineControlAlignment<S>,
        over_at_block_start: bool,
    ) -> Result<Self, crate::group_alignment::NumericError> {
        let InlineControlAlignment::Shift(offset) = alignment else {
            return Ok(self);
        };
        let over = if over_at_block_start {
            offset.get()
        } else {
            -offset.get()
        };
        Ok(Self {
            baseline: crate::baseline_offset::displaced(self.baseline, over)?,
            after_baseline: crate::baseline_offset::displaced(self.after_baseline, -over)?,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InlineLineMetricGroupsOf<S: LayoutScalar> {
    has_baseline: bool,
    baseline: S,
    after_baseline: S,
    zero_after_baseline_peak: S,
    has_positive_after_baseline: bool,
    line_over_extent: S,
    line_under_extent: S,
    over_at_block_start: bool,
    has_displacement: bool,
    numeric_error: Option<crate::group_alignment::NumericError>,
}

impl<S: LayoutScalar> InlineLineMetricGroupsOf<S> {
    fn from_strut(strut: Option<InlineMetricContributionOf<S>>, axes: FlowAxes) -> Self {
        Self::with_reference(strut, axes.line_over() == axes.block_start())
    }

    /// Text's native local metric coordinates are already line-over/line-under.
    pub(crate) fn typographic() -> Self {
        Self::with_reference(None, true)
    }

    fn with_reference(
        strut: Option<InlineMetricContributionOf<S>>,
        over_at_block_start: bool,
    ) -> Self {
        Self {
            has_baseline: strut.is_some(),
            baseline: strut.map_or(S::ZERO, |metrics| metrics.baseline),
            after_baseline: strut.map_or(S::ZERO, |metrics| metrics.after_baseline),
            zero_after_baseline_peak: strut
                .filter(|metrics| metrics.after_baseline == S::ZERO)
                .map_or(S::ZERO, |metrics| metrics.baseline),
            has_positive_after_baseline: strut
                .is_some_and(|metrics| metrics.after_baseline > S::ZERO),
            line_over_extent: S::ZERO,
            line_under_extent: S::ZERO,
            over_at_block_start,
            has_displacement: false,
            numeric_error: None,
        }
    }

    pub(crate) fn include(
        &mut self,
        alignment: InlineControlAlignment<S>,
        mut metrics: InlineMetricContributionOf<S>,
    ) {
        if let InlineControlAlignment::Shift(offset) = alignment {
            self.has_displacement |= offset.get() != S::ZERO;
            match metrics.post_baseline(alignment, self.over_at_block_start) {
                Ok(value) => metrics = value,
                Err(error) => {
                    self.numeric_error = self.numeric_error.or(Some(error));
                    return;
                }
            }
        }
        match alignment {
            InlineControlAlignment::Baseline | InlineControlAlignment::Shift(_) => {
                if self.has_baseline {
                    self.baseline = self.baseline.max(metrics.baseline);
                    self.after_baseline = self.after_baseline.max(metrics.after_baseline);
                } else {
                    self.baseline = metrics.baseline;
                    self.after_baseline = metrics.after_baseline;
                    self.has_baseline = true;
                }
                if metrics.after_baseline == S::ZERO {
                    self.zero_after_baseline_peak =
                        self.zero_after_baseline_peak.max(metrics.baseline);
                } else if metrics.after_baseline > S::ZERO {
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

    pub(crate) fn checked(
        self,
    ) -> Result<InlineMetricContributionOf<S>, crate::group_alignment::NumericError> {
        if let Some(error) = self.numeric_error {
            return Err(error);
        }
        let metrics = self.resolve();
        if self.has_displacement {
            crate::baseline_offset::displaced(metrics.baseline, metrics.after_baseline)?;
        }
        Ok(metrics)
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
    fn has_in_flow_content(self) -> bool {
        match self {
            Self::Atomic { .. } => true,
            Self::Boundary(control) => {
                control.edge().border() != S::ZERO || control.edge().padding() != S::ZERO
            }
            Self::ForcedLineBreak(_) => false,
        }
    }

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

    fn referenced_strut(self, flow_axes: FlowAxes) -> ReferencedInlineStrutOf<S> {
        let reference = match self {
            Self::Atomic { item, .. } => item.baseline_reference,
            Self::ForcedLineBreak(control) => control.metrics().reference(),
            Self::Boundary(control) => control.metrics().reference(),
        };
        ReferencedInlineStrutOf {
            metrics: self.metrics(flow_axes),
            reference,
            source: match self {
                Self::Atomic { item, .. } => InlineStrutSource::Atomic {
                    source_index: item.source_index,
                },
                Self::ForcedLineBreak(_) | Self::Boundary(_) => InlineStrutSource::Direct,
            },
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

    fn alignment(self) -> InlineControlAlignment<S> {
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
        if !line.has_displacement {
            line.baseline = fallback_band.baseline;
            line.after_baseline = fallback_band.after_baseline;
        }
        return;
    }

    line.fallback_line_band = None;
}

fn unshifted_alignment<S: LayoutScalar>(
    alignment: InlineControlAlignment<S>,
) -> InlineControlAlignment<S> {
    match alignment {
        InlineControlAlignment::Shift(_) => InlineControlAlignment::Baseline,
        value => value,
    }
}

fn displaced_fallback_groups<S: LayoutScalar>(
    participants: &[MixedInlineParticipantOf<S>],
    control: Option<ForcedLineBreakControlOf<S>>,
    strut: Option<InlineMetricContributionOf<S>>,
    raw_band: InlineMetricContributionOf<S>,
    axes: FlowAxes,
) -> InlineLineMetricGroupsOf<S> {
    let mut groups = InlineLineMetricGroupsOf::from_strut(strut, axes);
    if let Some(control) = control {
        groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    for participant in participants.iter().copied() {
        let mut metrics = participant.metrics(axes);
        if let MixedInlineParticipantOf::Atomic { item, .. } = participant
            && let Some(center) = item.fallback_block_start_in_band(axes, raw_band)
        {
            let margin = axes.logical_edges(item.margin);
            let extent = metrics.extent();
            metrics.baseline = raw_band.baseline - (center - margin.block_start);
            metrics.after_baseline = extent - metrics.baseline;
        }
        groups.include(participant.alignment(), metrics);
    }
    groups
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
    let mut metric_groups = InlineLineMetricGroupsOf::from_strut(strut, flow_axes);
    if let Some(control) = line_break {
        metric_groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    let fallback_line_band = (strut.is_some() || line_break.is_some()).then(|| {
        let mut raw = InlineLineMetricGroupsOf::from_strut(strut, flow_axes);
        if let Some(control) = line_break {
            raw.include(
                unshifted_alignment(control.alignment()),
                InlineMetricContributionOf {
                    baseline: control.metrics().baseline(),
                    after_baseline: control.metrics().after_baseline(),
                },
            );
        }
        raw.resolve()
    });
    let mut used_inline_extent = S::ZERO;
    let discarded_start_end = edges.leading_discard_len();
    let discarded_end_start = edges.trailing_discard_start();
    let mut has_in_flow_content = false;
    let mut before_trailing_metric_groups = metric_groups;
    let mut before_trailing_inline_extent = used_inline_extent;
    for (index, participant) in participants.iter().copied().enumerate() {
        if index == discarded_end_start {
            before_trailing_metric_groups = metric_groups;
            before_trailing_inline_extent = used_inline_extent;
        }
        if discarded_start_end <= index && index < discarded_end_start {
            has_in_flow_content |= participant.has_in_flow_content();
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
        has_in_flow_content,
        split_baseline_edge_phase: metric_groups.split_baseline_edge_phase(),
        numeric_error: metric_groups.checked().err(),
        has_displacement: metric_groups.has_displacement,
    }
}

fn select_inline_line<S: LayoutScalar>(
    input: InlineLineInput<'_, S>,
    line_break: Option<ForcedLineBreakControlOf<S>>,
    strut: Option<InlineMetricContributionOf<S>>,
    flow_axes: FlowAxes,
) -> SelectedInlineLineOf<S> {
    let summary = summarize_inline_line(input, line_break, strut, flow_axes);
    selected_from_summary(input, summary)
}
fn selected_from_summary<S: LayoutScalar>(
    input: InlineLineInput<'_, S>,
    summary: InlineLineSummaryOf<S>,
) -> SelectedInlineLineOf<S> {
    let participants = input.participants;
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
        next_cursor: InlineSourceCursor::default(),
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
        has_in_flow_content: summary.has_in_flow_content,
        split_baseline_edge_phase: summary.split_baseline_edge_phase,
        inline_start_override: None,
        incoming_strut: None,
        scope_band: None,
        numeric_error: summary.numeric_error,
        has_displacement: summary.has_displacement,
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
    carried_strut: Option<ActiveInlineStrutOf<S>>,
    flow_axes: FlowAxes,
) -> Result<Option<ActiveInlineStrutOf<S>>, NumericWrappingError> {
    let mut groups = InlineLineMetricGroupsOf::from_strut(None, flow_axes);
    let mut active = carried_strut;
    if let Some(control) = line.line_break {
        groups.include(
            control.alignment(),
            InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            },
        );
    }
    for selected in line.units.iter().copied().filter(|unit| !unit.discarded) {
        let participant = selected.participant;
        groups.include(participant.alignment(), participant.metrics(flow_axes));
        if let MixedInlineParticipantOf::Boundary(control) = participant {
            active = match control.kind() {
                InlineBoundaryKind::Start => Some(ActiveInlineStrutOf {
                    metrics: participant.metrics(flow_axes),
                    alignment: control.alignment(),
                }),
                InlineBoundaryKind::End => None,
            };
        }
    }
    if let Some(strut) = active {
        groups.include(strut.alignment, strut.metrics);
    }
    let metrics = groups.checked()?;
    line.baseline = metrics.baseline;
    line.after_baseline = metrics.after_baseline;
    line.fallback_line_band = None;
    line.has_displacement |= groups.has_displacement;
    Ok(active)
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

fn inline_min_content<S: LayoutScalar>(
    run: ProjectedInlineRun<'_, S>,
    flow_axes: FlowAxes,
    wrapping: Option<&crate::inline_wrapping::NumericWrapping>,
) -> S {
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
        if !wrapping.is_some_and(|w| w.forced(index + 1))
            && participant
                .following_break()
                .is_none_or(|b| b == InlineBreakOpportunity::Prohibited)
        {
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

fn inline_max_content<S: LayoutScalar>(
    run: ProjectedInlineRun<'_, S>,
    flow_axes: FlowAxes,
    wrapping: Option<&crate::inline_wrapping::NumericWrapping>,
) -> S {
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
        if participant.following_break() != Some(InlineBreakOpportunity::Mandatory)
            && !wrapping.is_some_and(|w| w.forced(index + 1))
        {
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
        AvailableOf::MinContent => {
            inline_min_content(run, input.flow_axes, input.wrapping.as_ref())
        }
        AvailableOf::MaxContent => {
            inline_max_content(run, input.flow_axes, input.wrapping.as_ref())
        }
    }
}

fn select_next_inline_line<S: LayoutScalar>(
    run: ProjectedInlineRun<'_, S>,
    cursor: InlineSourceCursor,
    next_boundary_descriptor: usize,
    available_inline_extent: S,
    wraps: bool,
    pending_strut: Option<ReferencedInlineStrutOf<S>>,
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
                Some(strut.metrics),
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
                    pending_strut.map(|strut| strut.metrics),
                    flow_axes,
                ),
                next_cursor: InlineSourceCursor {
                    edge_consumed: false,
                    source: scan + 1,
                    descriptor: cursor.descriptor + scan - source_cursor,
                },
                pending_strut: Some(participant.referenced_strut(flow_axes)),
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
                    pending_strut.map(|strut| strut.metrics),
                    flow_axes,
                ),
                next_cursor: InlineSourceCursor {
                    edge_consumed: false,
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
                        pending_strut.map(|strut| strut.metrics),
                        flow_axes,
                    ),
                    next_cursor: InlineSourceCursor {
                        edge_consumed: false,
                        source: scan,
                        descriptor: cursor.descriptor + scan - source_cursor,
                    },
                    pending_strut: Some(participant.referenced_strut(flow_axes)),
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
            pending_strut.map(|strut| strut.metrics),
            flow_axes,
        ),
        next_cursor: InlineSourceCursor {
            edge_consumed: false,
            source: participants.len(),
            descriptor: run.supplied.len(),
        },
        pending_strut: None,
    })
}

#[derive(Clone, Copy)]
struct NumericCandidateReceiptOf<S: LayoutScalar> {
    end: SelectedInlineLineEnd,
    next_cursor: InlineSourceCursor,
    pending_strut: Option<ReferencedInlineStrutOf<S>>,
    summary: InlineLineSummaryOf<S>,
    groups: InlineLineMetricGroupsOf<S>,
    phase_groups: InlineLineMetricGroupsOf<S>,
    last_boundary: Option<Option<ActiveInlineStrutOf<S>>>,
    starts_phase: bool,
    has_fallback_atomic: bool,
    band: Option<LogicalLineBandQueryResultOf<S>>,
    capacity: Option<S>,
    displaced_fallback: Option<InlineMetricContributionOf<S>>,
}
/// Prefix accumulator over actual numeric participants. Neither numeric variant
/// is discardable Text, so these are the same ordered metric/advance folds as
/// summarize_inline_line, with no independent width estimator.
struct NumericReceiptAccumulatorOf<S: LayoutScalar> {
    axes: FlowAxes,
    groups: InlineLineMetricGroupsOf<S>,
    phase_groups: InlineLineMetricGroupsOf<S>,
    advance: S,
    has_in_flow_content: bool,
    last_boundary: Option<Option<ActiveInlineStrutOf<S>>>,
    starts_phase: bool,
    has_fallback_atomic: bool,
    fallback: Option<InlineMetricContributionOf<S>>,
}
impl<S: LayoutScalar> NumericReceiptAccumulatorOf<S> {
    fn new(strut: Option<ReferencedInlineStrutOf<S>>, axes: FlowAxes) -> Self {
        let strut = strut.map(|strut| strut.metrics);
        let groups = InlineLineMetricGroupsOf::from_strut(strut, axes);
        Self {
            groups,
            phase_groups: InlineLineMetricGroupsOf::from_strut(None, axes),
            advance: S::ZERO,
            has_in_flow_content: false,
            last_boundary: None,
            starts_phase: false,
            has_fallback_atomic: false,
            fallback: strut.map(|_| groups.resolve()),
            axes,
        }
    }
    fn push(&mut self, participant: MixedInlineParticipantOf<S>, axes: FlowAxes) {
        self.has_in_flow_content |= participant.has_in_flow_content();
        self.groups
            .include(participant.alignment(), participant.metrics(axes));
        self.advance = self.advance + participant.inline_advance(axes);
        match participant {
            MixedInlineParticipantOf::Boundary(control) => {
                self.phase_groups
                    .include(control.alignment(), participant.metrics(axes));
                self.last_boundary = Some(match control.kind() {
                    InlineBoundaryKind::Start => {
                        self.starts_phase = true;
                        Some(ActiveInlineStrutOf {
                            metrics: participant.metrics(axes),
                            alignment: control.alignment(),
                        })
                    }
                    InlineBoundaryKind::End => None,
                });
            }
            participant => self
                .phase_groups
                .include(participant.alignment(), participant.metrics(axes)),
        }
        if let MixedInlineParticipantOf::Atomic { item, .. } = participant {
            self.has_fallback_atomic |= self
                .fallback
                .is_some_and(|band| item.fallback_block_start_in_band(axes, band).is_some());
        }
    }
    fn receipt(
        &self,
        end: SelectedInlineLineEnd,
        cursor: InlineSourceCursor,
        control: Option<ForcedLineBreakControlOf<S>>,
        pending: Option<ReferencedInlineStrutOf<S>>,
        length: usize,
    ) -> NumericCandidateReceiptOf<S> {
        let mut groups = self.groups;
        let mut phase_groups = self.phase_groups;
        let mut fallback = self.fallback;
        if let Some(control) = control {
            let metrics = InlineMetricContributionOf {
                baseline: control.metrics().baseline(),
                after_baseline: control.metrics().after_baseline(),
            };
            groups.include(control.alignment(), metrics);
            phase_groups.include(control.alignment(), metrics);
            let mut initial =
                InlineLineMetricGroupsOf::from_strut(pending.map(|strut| strut.metrics), self.axes);
            initial.include(unshifted_alignment(control.alignment()), metrics);
            fallback = Some(initial.resolve());
        }
        let metrics = groups.resolve();
        NumericCandidateReceiptOf {
            end,
            next_cursor: cursor,
            pending_strut: None,
            summary: InlineLineSummaryOf {
                discarded_start_end: 0,
                discarded_end_start: length,
                line_break: control,
                post_line_clear_intent: PostLineClearIntent::None,
                baseline: metrics.baseline,
                after_baseline: metrics.after_baseline,
                fallback_line_band: fallback,
                used_inline_extent: self.advance,
                has_in_flow_content: self.has_in_flow_content,
                split_baseline_edge_phase: groups.split_baseline_edge_phase(),
                numeric_error: groups.checked().err(),
                has_displacement: groups.has_displacement,
            },
            groups,
            phase_groups,
            last_boundary: self.last_boundary,
            starts_phase: self.starts_phase,
            has_fallback_atomic: self.has_fallback_atomic,
            band: None,
            capacity: None,
            displaced_fallback: None,
        }
    }
}

fn for_each_unowned_box_metric<S: LayoutScalar>(
    wrapping: &crate::inline_wrapping::NumericWrapping,
    participants: &[MixedInlineParticipantOf<S>],
    start: usize,
    end: usize,
    mut include: impl FnMut(InlineControlAlignment<S>, InlineMetricContributionOf<S>),
) -> Result<(), crate::inline_wrapping::NumericWrappingError> {
    use crate::{InlineBoxMetricContribution as Contribution, WrappingScopeError};
    for allocation in wrapping.metrics(start, end) {
        let owned: Vec<_> = allocation
            .endpoints
            .into_iter()
            .flatten()
            .filter(|&index| start <= index && index < end)
            .map(|index| match participants.get(index) {
                Some(MixedInlineParticipantOf::Boundary(control)) => {
                    Ok((control.alignment(), control.metrics()))
                }
                _ => Err(WrappingScopeError::CaptureAssociation),
            })
            .collect::<Result<_, _>>()?;
        let (alignment, metrics) = match &allocation.contribution {
            Contribution::NoContribution => {
                if !owned.is_empty() {
                    return Err(WrappingScopeError::ConflictingBoxMetrics.into());
                }
                continue;
            }
            Contribution::Strut(strut) => {
                let baseline = S::from_f64(strut.metrics.baseline);
                let extent = S::from_f64(strut.metrics.line_extent);
                if !baseline.is_finite() || !extent.is_finite() || extent < baseline {
                    return Err(WrappingScopeError::InvalidBoxMetrics.into());
                }
                let alignment = match strut.alignment {
                    surgeist_text::LineBoxAlignment::Baseline => InlineControlAlignment::Baseline,
                    surgeist_text::LineBoxAlignment::Top => InlineControlAlignment::Top,
                    surgeist_text::LineBoxAlignment::Bottom => InlineControlAlignment::Bottom,
                    surgeist_text::LineBoxAlignment::Shift(offset) => {
                        InlineControlAlignment::Shift(
                            crate::BaselineOffsetOf::try_new(crate::group_alignment::scalar::<S>(
                                offset.get(),
                            )?)
                            .map_err(|_| WrappingScopeError::InvalidBoxMetrics)?,
                        )
                    }
                };
                if owned.iter().any(|(actual_alignment, metrics)| {
                    *actual_alignment != alignment
                        || metrics.baseline() != baseline
                        || metrics.after_baseline() != extent - baseline
                }) {
                    return Err(WrappingScopeError::ConflictingBoxMetrics.into());
                }
                (
                    alignment,
                    InlineMetricContributionOf {
                        baseline,
                        after_baseline: extent - baseline,
                    },
                )
            }
        };
        if owned.is_empty() {
            include(alignment, metrics);
        }
    }
    Ok(())
}

fn allocate_numeric_box_metrics<S: LayoutScalar>(
    receipt: &mut NumericCandidateReceiptOf<S>,
    wrapping: &crate::inline_wrapping::NumericWrapping,
    participants: &[MixedInlineParticipantOf<S>],
    start: usize,
    end: usize,
) -> Result<(), NumericWrappingError> {
    for_each_unowned_box_metric(wrapping, participants, start, end, |alignment, metrics| {
        receipt.groups.include(alignment, metrics);
        receipt.phase_groups.include(alignment, metrics);
    })?;
    let metrics = receipt.groups.checked()?;
    receipt.summary.baseline = metrics.baseline;
    receipt.summary.after_baseline = metrics.after_baseline;
    receipt.summary.split_baseline_edge_phase = receipt.groups.split_baseline_edge_phase();
    receipt.summary.numeric_error = None;
    receipt.summary.has_displacement = receipt.groups.has_displacement;
    Ok(())
}

fn resolve_numeric_candidate<S: LayoutScalar>(
    receipt: &mut NumericCandidateReceiptOf<S>,
    carried: Option<ActiveInlineStrutOf<S>>,
    block: S,
    indent: S,
    available: S,
    mut band_source: impl FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
) -> Result<bool, crate::inline_wrapping::NumericWrappingError> {
    if NumericLineDisposition::for_line(receipt.end.line_end(), receipt.summary.has_in_flow_content)
        == NumericLineDisposition::NonFormatted
    {
        return Ok(true);
    }
    receipt.groups.checked()?;
    receipt.phase_groups.checked()?;
    let original = receipt.summary;
    let mut floating = false;
    let mut seen = Vec::new();
    loop {
        let phase = carried.is_some() || floating && receipt.starts_phase;
        let (metrics, fallback) = if phase {
            let mut groups = receipt.phase_groups;
            let active = receipt.last_boundary.unwrap_or(carried);
            if let Some(strut) = active {
                groups.include(strut.alignment, strut.metrics);
            }
            let metrics = groups.checked()?;
            receipt.summary.has_displacement |= groups.has_displacement;
            (
                InlineMetricContributionOf {
                    baseline: metrics.baseline,
                    after_baseline: metrics.after_baseline,
                },
                None,
            )
        } else if floating && receipt.has_fallback_atomic {
            (
                receipt
                    .displaced_fallback
                    .or(original.fallback_line_band)
                    .unwrap_or(InlineMetricContributionOf {
                        baseline: original.baseline,
                        after_baseline: original.after_baseline,
                    }),
                original.fallback_line_band,
            )
        } else {
            (
                InlineMetricContributionOf {
                    baseline: original.baseline,
                    after_baseline: original.after_baseline,
                },
                if receipt.has_fallback_atomic {
                    None
                } else {
                    original.fallback_line_band
                },
            )
        };
        let end = consumed_line_end(
            block,
            metrics.baseline,
            metrics.after_baseline,
            receipt.summary.has_displacement,
        )?;
        let band = band_source(block, end);
        let basis = (
            metrics.baseline,
            metrics.after_baseline,
            band.inline_start,
            band.inline_end,
            band.next_transition,
            phase,
        );
        let next = band.next_transition.is_some();
        if next != floating {
            if seen.contains(&basis) {
                return Err(crate::inline_wrapping::NumericWrappingError::FeedbackCycle);
            }
            seen.push(basis);
            floating = next;
            continue;
        }
        receipt.summary.baseline = metrics.baseline;
        receipt.summary.after_baseline = metrics.after_baseline;
        receipt.summary.fallback_line_band = fallback;
        receipt.band = Some(band);
        let width = (band.inline_end - band.inline_start).max(S::ZERO);
        let width = if width == S::ZERO && band.next_transition.is_none() {
            available
        } else {
            width
        };
        receipt.capacity = Some(width);
        return Ok(receipt.summary.used_inline_extent <= width - indent);
    }
}

#[derive(Clone, Copy)]
struct NumericLineQuery<'a, S: LayoutScalar> {
    goal: Option<LineQualityGoal>,
    indent: S,
    run: ProjectedInlineRun<'a, S>,
    cursor: InlineSourceCursor,
    next_boundary_descriptor: usize,
    available: S,
    wraps: bool,
    pending: Option<ReferencedInlineStrutOf<S>>,
    axes: FlowAxes,
}
enum NumericCandidatePreference<S: LayoutScalar> {
    Auto(NumericCandidateReceiptOf<S>),
    Quality {
        receipt: NumericCandidateReceiptOf<S>,
        key: LineQualityKey,
    },
}
impl<S: LayoutScalar> NumericCandidatePreference<S> {
    fn new(
        receipt: NumericCandidateReceiptOf<S>,
        goal: Option<LineQualityGoal>,
        indent: S,
    ) -> Result<Self, crate::inline_wrapping::NumericWrappingError> {
        let Some(goal) = goal else {
            return Ok(Self::Auto(receipt));
        };
        let invalid = match goal {
            LineQualityGoal::Balance(_) => {
                crate::inline_wrapping::NumericWrappingError::InvalidBalanceBasis
            }
            LineQualityGoal::Tail(_) => {
                crate::inline_wrapping::NumericWrappingError::InvalidTailBasis
            }
            LineQualityGoal::Body(_) => {
                crate::inline_wrapping::NumericWrappingError::InvalidPrettyBasis
            }
        };
        let key = match receipt.capacity {
            Some(capacity) => goal
                .key(
                    capacity.to_f64(),
                    indent.to_f64(),
                    receipt.summary.used_inline_extent.to_f64(),
                )
                .ok_or(invalid)?,
            None => match goal {
                LineQualityGoal::Balance(_) => return Err(invalid),
                // A nonformatted completion has no optional quality measure;
                // it still retains its genuine ordinary completion semantics.
                LineQualityGoal::Tail(_) => LineQualityKey::Tail(None),
                LineQualityGoal::Body(_) => LineQualityKey::Body(None),
            },
        };
        Ok(Self::Quality { receipt, key })
    }
    fn receipt(&self) -> &NumericCandidateReceiptOf<S> {
        match self {
            Self::Auto(receipt) | Self::Quality { receipt, .. } => receipt,
        }
    }
    fn into_receipt(self) -> NumericCandidateReceiptOf<S> {
        match self {
            Self::Auto(receipt) | Self::Quality { receipt, .. } => receipt,
        }
    }
    fn compare(&self, other: &Self, goal: Option<LineQualityGoal>) -> std::cmp::Ordering {
        match (self, other, goal) {
            (Self::Auto(_), Self::Auto(_), None) => std::cmp::Ordering::Equal,
            (Self::Quality { key: left, .. }, Self::Quality { key: right, .. }, Some(goal)) => {
                goal.compare(*left, *right)
            }
            _ => unreachable!("one numeric candidate walk has one actual preference mode"),
        }
    }
}

fn select_scope_inline_line<S: LayoutScalar>(
    query: NumericLineQuery<'_, S>,
    wrapping: Option<&crate::inline_wrapping::NumericWrapping>,
    mut prepare_receiver: impl FnMut(
        SelectedInlineLineEnd,
        bool,
    ) -> Result<
        Option<ReferencedInlineStrutOf<S>>,
        crate::inline_wrapping::NumericWrappingError,
    >,
    mut native_fits: impl FnMut(
        &mut NumericCandidateReceiptOf<S>,
    ) -> Result<bool, crate::inline_wrapping::NumericWrappingError>,
) -> Result<InlineSelectionStep<S>, crate::inline_wrapping::NumericWrappingError> {
    let NumericLineQuery {
        goal,
        indent,
        run,
        cursor,
        next_boundary_descriptor,
        available,
        wraps,
        pending,
        axes,
    } = query;
    let mut fits = |receipt: &mut NumericCandidateReceiptOf<S>| {
        if receipt.summary.has_displacement
            && let Some(raw) = receipt.summary.fallback_line_band
        {
            let end =
                receipt.next_cursor.source - usize::from(receipt.summary.line_break.is_some());
            let participants = &run.participants[cursor.source..end];
            receipt.has_fallback_atomic = participants.iter().any(|participant| matches!(participant,
                MixedInlineParticipantOf::Atomic { item, .. } if item.fallback_block_start_in_band(axes, raw).is_some()));
            if receipt.has_fallback_atomic {
                let mut groups = displaced_fallback_groups(
                    participants,
                    receipt.summary.line_break,
                    pending.map(|strut| strut.metrics),
                    raw,
                    axes,
                );
                if let Some(wrapping) = wrapping {
                    for_each_unowned_box_metric(
                        wrapping,
                        run.participants,
                        cursor.source,
                        end,
                        |alignment, metrics| groups.include(alignment, metrics),
                    )?;
                }
                receipt.displaced_fallback = Some(groups.checked()?);
            }
        }
        native_fits(receipt)
    };
    let baseline = || {
        select_next_inline_line(
            run,
            cursor,
            next_boundary_descriptor,
            available,
            wraps,
            pending,
            axes,
        )
    };
    if wrapping.is_none() && goal.is_none() {
        let step = prepare_selected_receiver(baseline(), &mut prepare_receiver)?;
        let InlineSelectionStep::Selected(Some(mut candidate)) = step else {
            return Ok(step);
        };
        if !candidate.line.has_displacement {
            return Ok(InlineSelectionStep::Selected(Some(candidate)));
        }
        let end = candidate.next_cursor.source - usize::from(candidate.line.line_break.is_some());
        let mut accumulator = NumericReceiptAccumulatorOf::new(pending, axes);
        for participant in run.participants[cursor.source..end].iter().copied() {
            accumulator.push(participant, axes);
        }
        let mut receipt = accumulator.receipt(
            candidate.end,
            candidate.next_cursor,
            candidate.line.line_break,
            pending,
            end - cursor.source,
        );
        let _ = fits(&mut receipt)?;
        candidate.line = selected_from_summary(
            run.line(cursor.source..end, cursor.descriptor),
            receipt.summary,
        );
        candidate.line.scope_band = receipt.band;
        return Ok(InlineSelectionStep::Selected(Some(candidate)));
    }
    let edge_stop = wrapping.and_then(|wrapping| {
        if !cursor.edge_consumed && wrapping.forced(cursor.source) {
            Some(cursor.source)
        } else {
            wrapping.next_forced(cursor.source)
        }
    });
    if cursor.source == run.participants.len() && edge_stop.is_none() {
        return prepare_selected_receiver(baseline(), &mut prepare_receiver);
    }
    let mut stop = run.participants.len();
    let mut forced = None;
    let mut mandatory = false;
    let mut accumulator = NumericReceiptAccumulatorOf::new(pending, axes);
    for i in cursor.source..run.participants.len() {
        let participant = run.participants[i];
        if edge_stop == Some(i) {
            stop = i;
            mandatory = true;
            break;
        }
        if let MixedInlineParticipantOf::ForcedLineBreak(control) = participant {
            stop = i;
            forced = Some(control);
            mandatory = true;
            break;
        }
        accumulator.push(participant, axes);
        if participant.following_break() == Some(InlineBreakOpportunity::Mandatory) {
            stop = i + 1;
            mandatory = true;
            break;
        }
    }
    let edge_forced = edge_stop == Some(stop);
    mandatory |= edge_forced;
    let mut completion = accumulator.receipt(
        if mandatory {
            SelectedInlineLineEnd::Forced
        } else {
            SelectedInlineLineEnd::Paragraph
        },
        InlineSourceCursor {
            edge_consumed: edge_forced,
            source: stop + usize::from(forced.is_some()),
            descriptor: cursor.descriptor + stop - cursor.source,
        },
        forced,
        pending,
        stop - cursor.source,
    );
    completion.pending_strut = if let Some(control) = forced {
        Some(MixedInlineParticipantOf::ForcedLineBreak(control).referenced_strut(axes))
    } else if mandatory {
        stop.checked_sub(1)
            .map(|i| run.participants[i].referenced_strut(axes))
    } else {
        None
    };
    completion.summary.post_line_clear_intent = forced.map_or(PostLineClearIntent::None, |c| {
        mapped_post_line_clear_intent(axes, c.clear())
    });
    // Earlier legal Soft candidates end after an Atomic and are therefore real
    // too. Boundaries supply no following permission; a scope force is Forced.
    // Prepare before allocation, preference or any height-dependent observation.
    if let Some(seed) = prepare_receiver(completion.end, completion.summary.has_in_flow_content)? {
        return Ok(InlineSelectionStep::RetryReceiver(seed));
    }
    if let Some(fallback) = completion.summary.fallback_line_band {
        completion.has_fallback_atomic=run.participants[cursor.source..stop].iter().any(|p|matches!(*p,MixedInlineParticipantOf::Atomic {item,..} if item.fallback_block_start_in_band(axes,fallback).is_some()));
    }
    if let Some(wrapping) = wrapping {
        allocate_numeric_box_metrics(
            &mut completion,
            wrapping,
            run.participants,
            cursor.source,
            stop,
        )?;
    }
    let build = |receipt: NumericCandidateReceiptOf<S>| {
        let end = receipt.next_cursor.source - usize::from(receipt.summary.line_break.is_some());
        let mut line = selected_from_summary(
            run.line(cursor.source..end, cursor.descriptor),
            receipt.summary,
        );
        line.scope_band = receipt.band;
        SelectedInlineLineCandidateOf {
            end: receipt.end,
            line,
            next_cursor: receipt.next_cursor,
            pending_strut: receipt.pending_strut,
        }
    };
    let completion_fits = fits(&mut completion)?;
    if !wraps
        || completion_fits
            && (goal.is_none()
                || NumericLineDisposition::for_line(
                    completion.end.line_end(),
                    completion.summary.has_in_flow_content,
                ) == NumericLineDisposition::NonFormatted)
    {
        return Ok(InlineSelectionStep::Selected(Some(build(completion))));
    }
    let descriptor_stop = cursor.descriptor + stop - cursor.source;
    let bounded = ProjectedInlineRun {
        participants: &run.participants[..stop + usize::from(forced.is_some())],
        supplied: &run.supplied[..descriptor_stop],
    };
    let fallback = select_next_inline_line(
        bounded,
        cursor,
        descriptor_stop,
        available,
        wraps,
        pending,
        axes,
    )
    .map(|mut candidate| {
        if candidate.next_cursor.source == stop && mandatory {
            candidate.end = SelectedInlineLineEnd::Forced;
            candidate.next_cursor.edge_consumed = edge_forced;
            candidate.pending_strut = completion.pending_strut;
        }
        candidate
    })
    .or_else(|| Some(build(completion)));
    let fallback = if !wrapping.is_some_and(|wrapping| wrapping.has_metrics()) {
        fallback
    } else if let Some(candidate) = fallback {
        let end = candidate.next_cursor.source - usize::from(candidate.line.line_break.is_some());
        let mut accumulator = NumericReceiptAccumulatorOf::new(pending, axes);
        for participant in run.participants[cursor.source..end].iter().copied() {
            accumulator.push(participant, axes);
        }
        let mut receipt = accumulator.receipt(
            candidate.end,
            candidate.next_cursor,
            candidate.line.line_break,
            pending,
            end - cursor.source,
        );
        receipt.pending_strut = candidate.pending_strut;
        if let Some(seed) = prepare_receiver(receipt.end, receipt.summary.has_in_flow_content)? {
            return Ok(InlineSelectionStep::RetryReceiver(seed));
        }
        if let Some(wrapping) = wrapping {
            allocate_numeric_box_metrics(
                &mut receipt,
                wrapping,
                run.participants,
                cursor.source,
                end,
            )?;
        }
        fits(&mut receipt)?;
        Some(build(receipt))
    } else {
        None
    };
    let mut first: Option<NumericCandidatePreference<S>> = None;
    let mut frontier: Option<
        crate::inline_wrapping::PreferenceFrontier<NumericCandidatePreference<S>>,
    > = None;
    let prefer = |left: &NumericCandidatePreference<S>, right: &NumericCandidatePreference<S>| {
        left.compare(right, goal)
    };
    let mut consider = |receipt: NumericCandidateReceiptOf<S>| -> Result<(), crate::inline_wrapping::NumericWrappingError> {
        let next = NumericCandidatePreference::new(receipt, goal, indent)?;
        if let Some(wrapping) = wrapping {
            if let Some(frontier) = frontier.as_mut() {
                if goal.is_some() { frontier.consider_ranked(wrapping.class(next.receipt().next_cursor.source)?, next, prefer)?; }
                else { frontier.consider(wrapping.class(next.receipt().next_cursor.source)?, next)?; }
            } else if let Some(prior) = first.take() {
                let mut next_frontier = wrapping.frontier();
                if goal.is_some() {
                    next_frontier.consider_ranked(wrapping.class(prior.receipt().next_cursor.source)?, prior, prefer)?;
                    next_frontier.consider_ranked(wrapping.class(next.receipt().next_cursor.source)?, next, prefer)?;
                } else {
                    next_frontier.consider(wrapping.class(prior.receipt().next_cursor.source)?, prior)?;
                    next_frontier.consider(wrapping.class(next.receipt().next_cursor.source)?, next)?;
                }
                frontier = Some(next_frontier);
            } else { first = Some(next); }
        } else if first.as_ref().is_none_or(|prior| prefer(&next, prior) != std::cmp::Ordering::Greater) {
            first = Some(next);
        }
        Ok(())
    };
    let mut accumulator = NumericReceiptAccumulatorOf::new(pending, axes);
    for end in cursor.source + 1..stop {
        let participant = run.participants[end - 1];
        accumulator.push(participant, axes);
        if participant.following_break() != Some(InlineBreakOpportunity::Allowed) {
            continue;
        }
        let mut candidate = accumulator.receipt(
            SelectedInlineLineEnd::Soft,
            InlineSourceCursor {
                edge_consumed: false,
                source: end,
                descriptor: cursor.descriptor + end - cursor.source,
            },
            None,
            pending,
            end - cursor.source,
        );
        if let Some(seed) = prepare_receiver(candidate.end, candidate.summary.has_in_flow_content)?
        {
            return Ok(InlineSelectionStep::RetryReceiver(seed));
        }
        if let Some(wrapping) = wrapping {
            allocate_numeric_box_metrics(
                &mut candidate,
                wrapping,
                run.participants,
                cursor.source,
                end,
            )?;
        }
        if !fits(&mut candidate)? {
            continue;
        }
        consider(candidate)?;
    }
    if completion_fits {
        consider(completion)?;
    }
    let preferred = frontier
        .and_then(|frontier| {
            if goal.is_some() {
                frontier.into_preferred_ranked(prefer)
            } else {
                frontier.into_preferred()
            }
        })
        .or(first);
    Ok(InlineSelectionStep::Selected(
        preferred
            .map(|preferred| build(preferred.into_receipt()))
            .or(fallback),
    ))
}

fn prepare_selected_receiver<S: LayoutScalar>(
    candidate: Option<SelectedInlineLineCandidateOf<S>>,
    prepare: &mut impl FnMut(
        SelectedInlineLineEnd,
        bool,
    ) -> Result<
        Option<ReferencedInlineStrutOf<S>>,
        crate::inline_wrapping::NumericWrappingError,
    >,
) -> Result<InlineSelectionStep<S>, crate::inline_wrapping::NumericWrappingError> {
    if let Some(candidate) = &candidate
        && let Some(seed) = prepare(candidate.end, candidate.line.has_in_flow_content)?
    {
        return Ok(InlineSelectionStep::RetryReceiver(seed));
    }
    Ok(InlineSelectionStep::Selected(candidate))
}

fn text_line_offset<S: LayoutScalar>(
    used_inline_extent: S,
    available_inline_extent: S,
    flow_axes: FlowAxes,
    text_align: TextAlign,
) -> S {
    let free_space = (available_inline_extent - used_inline_extent).max(S::ZERO);
    match text_align {
        TextAlign::Start | TextAlign::Justify => S::ZERO,
        TextAlign::End => free_space,
        TextAlign::Left | TextAlign::LegacyLeft if flow_axes.direction() == Direction::Rtl => {
            free_space
        }
        TextAlign::Right | TextAlign::LegacyRight if flow_axes.direction() == Direction::Ltr => {
            free_space
        }
        TextAlign::Center | TextAlign::LegacyCenter => free_space / S::from_f64(2.0),
        TextAlign::Left | TextAlign::Right | TextAlign::LegacyLeft | TextAlign::LegacyRight => {
            S::ZERO
        }
    }
}

fn line_alignment_offset<S: LayoutScalar>(
    advance: S,
    capacity: S,
    axes: FlowAxes,
    alignment: TextAlign,
    same_direction: bool,
) -> S {
    let alignment = match (same_direction, alignment) {
        (false, TextAlign::Start | TextAlign::Justify) => TextAlign::End,
        (false, TextAlign::End) => TextAlign::Start,
        (_, alignment) => alignment,
    };
    let free_space = capacity - advance;
    if free_space < S::ZERO {
        if same_direction { S::ZERO } else { free_space }
    } else {
        text_line_offset(advance, capacity, axes, alignment)
    }
}
fn ordinary_line_inline_start<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    axes: FlowAxes,
    alignment: TextAlign,
    direction: Direction,
) -> S {
    let same_direction = direction == axes.direction();
    line.inline_start_override.unwrap_or_else(|| {
        let capacity = line.band.inline_end - line.band.inline_start - line.indent;
        let offset = line_alignment_offset(
            line.used_inline_extent,
            capacity,
            axes,
            alignment,
            same_direction,
        );
        line.band.inline_start + if same_direction { line.indent } else { S::ZERO } + offset
    })
}
fn selected_line_inline_start<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    axes: FlowAxes,
    alignment: TextAlign,
    direction: Direction,
    insets: surgeist_text::LineAlignmentInsets,
) -> Result<S, crate::inline_wrapping::NumericWrappingError> {
    let old = ordinary_line_inline_start(line, axes, alignment, direction);
    if insets == surgeist_text::LineAlignmentInsets::ZERO {
        return Ok(old);
    }
    let capacity = crate::group_alignment::canonical_capacity::<S>(
        (line.band.inline_end - line.band.inline_start).to_f64(),
        insets,
        line.indent.to_f64(),
    )?;
    crate::group_alignment::add::<S>(capacity, -line.used_inline_extent.to_f64())?;
    let same_direction = direction == axes.direction();
    let offset = line_alignment_offset(
        line.used_inline_extent,
        crate::group_alignment::scalar::<S>(capacity)?,
        axes,
        alignment,
        same_direction,
    );
    let start_inset = if axes.direction() == Direction::Ltr {
        insets.line_left()
    } else {
        insets.line_right()
    };
    let base = crate::group_alignment::add::<S>(
        line.band.inline_start.to_f64(),
        if same_direction {
            line.indent.to_f64()
        } else {
            0.0
        },
    )?;
    let shifted = crate::group_alignment::add::<S>(
        crate::group_alignment::add::<S>(base, offset.to_f64())?,
        start_inset,
    )?;
    let result = if let Some(overridden) = line.inline_start_override {
        let mut ordinary = line.clone();
        ordinary.inline_start_override = None;
        let delta = crate::group_alignment::add::<S>(
            shifted,
            -ordinary_line_inline_start(&ordinary, axes, alignment, direction).to_f64(),
        )?;
        crate::group_alignment::add::<S>(overridden.to_f64(), delta)?
    } else {
        shifted
    };
    Ok(crate::group_alignment::scalar(result)?)
}

fn resolved_float_terminal_block_extent<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    carried_strut: Option<ActiveInlineStrutOf<S>>,
    float_transition: Option<S>,
    flow_axes: FlowAxes,
    line_inline_start: S,
    line_inline_end: S,
) -> Result<S, NumericWrappingError> {
    let line_block_end = line.band.block_end;
    let Some(active) = carried_strut else {
        return Ok(line_block_end);
    };
    let strut = active.metrics.post_baseline(
        active.alignment,
        flow_axes.line_over() == flow_axes.block_start(),
    )?;
    if strut.after_baseline <= S::ZERO {
        return Ok(line_block_end);
    }
    let touches_float_edge = if flow_axes
        .logical_axis_progression(crate::LogicalAxis::Inline)
        .is_decreasing()
    {
        line_inline_start == line.band.inline_start
    } else {
        line_inline_end == line.band.inline_end
    };
    if float_transition != Some(line_block_end) || !touches_float_edge {
        return Ok(line_block_end);
    }
    let displaced = matches!(active.alignment, InlineControlAlignment::Shift(offset) if offset.get() != S::ZERO);
    let difference = if displaced {
        crate::baseline_offset::displaced(line.baseline, -strut.baseline)?
    } else {
        line.baseline - strut.baseline
    };
    let baseline_phase = difference.max(S::ZERO) / strut.after_baseline;
    if displaced {
        Ok(crate::baseline_offset::displaced(
            line_block_end,
            baseline_phase,
        )?)
    } else {
        Ok(line_block_end + baseline_phase)
    }
}

fn resolved_inline_unit_slots<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    supplied: &[SuppliedInlineParticipant],
    flow_axes: FlowAxes,
    text_align: TextAlign,
    direction: Direction,
    insets: surgeist_text::LineAlignmentInsets,
) -> Result<(S, Vec<usize>, Vec<S>), crate::inline_wrapping::NumericWrappingError> {
    let line_inline_start =
        selected_line_inline_start(line, flow_axes, text_align, direction, insets)?;
    let mut order = vec![0; line.units.len()];
    let visual_order = SuppliedLine::new(supplied)
        .reorder_into(&mut order)
        .expect("selected descriptor range has exactly one slot per selected unit");
    let mut visual_indices = vec![0; line.units.len()];
    let mut inline_starts = vec![S::ZERO; line.units.len()];
    for (visual_index, (source_index, _)) in visual_order.iter().enumerate() {
        visual_indices[source_index] = visual_index;
    }
    // L2 orders slots from line-left to line-right. A containing RTL flow
    // numbers logical coordinates from the opposite side of the occupied
    // interval, independently of its physical writing-mode progression.
    let rtl_flow = flow_axes.direction() == Direction::Rtl;
    let mut inline_start = if rtl_flow {
        line_inline_start + line.used_inline_extent
    } else {
        line_inline_start
    };
    for (source_index, _) in visual_order.iter() {
        let selected = line.units[source_index];
        let selected_inline_extent = if selected.discarded {
            S::ZERO
        } else {
            selected.participant.inline_advance(flow_axes)
        };
        if rtl_flow {
            inline_start = inline_start - selected_inline_extent;
        }
        inline_starts[source_index] = inline_start;
        if !rtl_flow {
            inline_start = inline_start + selected_inline_extent;
        }
    }
    Ok((line_inline_start, visual_indices, inline_starts))
}

#[cfg(test)]
pub(super) fn layout_mixed_inline_run<S: LayoutScalar>(
    input: MixedInlineRunInputOf<S>,
) -> Result<MixedInlineRunReportOf<S>, crate::inline_wrapping::NumericWrappingError> {
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
        unavailable_atomic_receiver,
    )
}

#[cfg(test)]
pub(super) fn layout_mixed_inline_run_with_band_source<S, BandSource, ClearSource>(
    input: MixedInlineRunInputOf<S>,
    band_source: BandSource,
    clear_source: ClearSource,
) -> Result<MixedInlineRunReportOf<S>, crate::inline_wrapping::NumericWrappingError>
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
        unavailable_atomic_receiver,
    )
}

pub(super) fn layout_mixed_inline_run_with_formatted_observation<
    S,
    BandSource,
    ClearSource,
    ReceiverSource,
>(
    input: MixedInlineRunInputOf<S>,
    band_source: BandSource,
    clear_source: ClearSource,
    observe: bool,
    receiver_source: ReceiverSource,
) -> Result<MixedInlineRunReportOf<S>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
    ReceiverSource:
        FnMut(
            usize,
            crate::InlineBaselineContext,
        )
            -> Result<ReferencedInlineStrutOf<S>, crate::inline_wrapping::NumericWrappingError>,
{
    let supplied = project_inline_participants(&input.participants);
    let available = resolved_inline_available(&input, &supplied);
    layout_mixed_inline_run_from_available(
        input,
        supplied,
        available,
        band_source,
        clear_source,
        observe,
        receiver_source,
    )
}

#[derive(Clone, Copy)]
struct NumericWalkInput<'a, S: LayoutScalar> {
    input: &'a MixedInlineRunInputOf<S>,
    supplied: &'a [SuppliedInlineParticipant],
    available: S,
    receiver_source: &'a std::cell::RefCell<AtomicReceiverSource<'a, S>>,
}

/// A metric-only private caller has supplied coordinates, not actual box facts.
#[cfg(test)]
pub(super) fn unavailable_atomic_receiver<S: LayoutScalar>(
    _: usize,
    expected: crate::InlineBaselineContext,
) -> Result<ReferencedInlineStrutOf<S>, crate::inline_wrapping::NumericWrappingError> {
    Err(crate::inline_wrapping::NumericWrappingError::Baseline(
        crate::InlineBaselineError::ReferenceMismatch {
            expected,
            actual: crate::InlineBaselineContext::Physical,
        },
    ))
}

#[derive(Clone, Copy, PartialEq)]
struct NumericFlowState<S: LayoutScalar> {
    cursor: InlineSourceCursor,
    boundary_index: usize,
    pending_strut: Option<ReferencedInlineStrutOf<S>>,
    indent_state: crate::text_indent::IndentLineState,
    line_box_context: crate::line_box::LineBoxContext,
    block_cursor: S,
    float_strut: Option<ActiveInlineStrutOf<S>>,
    continuation_inline_cursor: Option<S>,
    advanced_phase_transition: bool,
    resolved_terminal_block_extent: S,
    float_edge_phase: Option<S>,
}

struct NumericSequence<S: LayoutScalar> {
    state: NumericFlowState<S>,
    selected_lines: Vec<SelectedInlineLineOf<S>>,
    formatted_lines: Option<Vec<crate::text_layout::formatted::NumericFormattedLine>>,
    tail_frontier: Option<NumericFlowState<S>>,
}

#[derive(Clone, Copy)]
struct NumericGroupEntry<S: LayoutScalar> {
    state: NumericFlowState<S>,
    selected_len: usize,
    formatted_len: usize,
}

impl<S: LayoutScalar> NumericSequence<S> {
    fn new(input: &MixedInlineRunInputOf<S>, observe: bool) -> Self {
        Self {
            state: NumericFlowState {
                cursor: InlineSourceCursor::default(),
                boundary_index: 0,
                pending_strut: None,
                indent_state: input.indent_state,
                line_box_context: input.line_box_context,
                block_cursor: S::ZERO,
                float_strut: None,
                continuation_inline_cursor: None,
                advanced_phase_transition: false,
                resolved_terminal_block_extent: S::ZERO,
                float_edge_phase: None,
            },
            selected_lines: Vec::new(),
            formatted_lines: observe.then(Vec::new),
            tail_frontier: None,
        }
    }
    fn checkpoint(&self) -> NumericGroupEntry<S> {
        NumericGroupEntry {
            state: self.state,
            selected_len: self.selected_lines.len(),
            formatted_len: self.formatted_lines.as_ref().map_or(0, Vec::len),
        }
    }
    fn restore(&mut self, entry: NumericGroupEntry<S>) {
        self.state = entry.state;
        self.tail_frontier = None;
        self.selected_lines.truncate(entry.selected_len);
        if let Some(lines) = &mut self.formatted_lines {
            lines.truncate(entry.formatted_len);
        }
    }
}

#[derive(Clone, Copy)]
struct NumericWalkControl {
    goal: Option<LineQualityGoal>,
    goal_start: usize,
    goal_end: Option<usize>,
    observe_frontier: Option<usize>,
    stop_at_group: bool,
    max_formatted: Option<usize>,
}
impl NumericWalkControl {
    fn auto() -> Self {
        Self {
            goal: None,
            goal_start: 0,
            goal_end: None,
            observe_frontier: None,
            stop_at_group: false,
            max_formatted: None,
        }
    }
    fn group(goal: Option<LineQualityGoal>, max_formatted: Option<usize>) -> Self {
        Self {
            goal,
            goal_start: 0,
            goal_end: None,
            observe_frontier: None,
            stop_at_group: true,
            max_formatted,
        }
    }
    fn stop(self, ending: surgeist_text::LineEnd, count: usize) -> Option<NumericWalkOutcome> {
        if self.max_formatted.is_some_and(|limit| count > limit) {
            Some(NumericWalkOutcome::CountExceeded)
        } else if self.stop_at_group && ending != surgeist_text::LineEnd::Soft {
            Some(NumericWalkOutcome::GroupComplete)
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum NumericWalkOutcome {
    Exhausted,
    GroupComplete,
    CountExceeded,
}

fn walk_numeric_sequence<S, B, C>(
    request: NumericWalkInput<'_, S>,
    sequence: &mut NumericSequence<S>,
    line_boundaries: &mut std::iter::Peekable<impl Iterator<Item = InlineSourceCursor>>,
    control: NumericWalkControl,
    mut band_source: B,
    mut clear_source: C,
) -> Result<NumericWalkOutcome, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let NumericWalkInput {
        input,
        supplied,
        available,
        receiver_source,
    } = request;
    let run = ProjectedInlineRun {
        participants: &input.participants,
        supplied,
    };
    let wraps = !matches!(input.available_inline_extent, AvailableOf::MaxContent);
    let NumericSequence {
        state,
        selected_lines,
        formatted_lines,
        tail_frontier,
    } = sequence;
    let NumericFlowState {
        mut cursor,
        mut boundary_index,
        mut pending_strut,
        mut indent_state,
        mut line_box_context,
        mut block_cursor,
        mut float_strut,
        mut continuation_inline_cursor,
        mut advanced_phase_transition,
        mut resolved_terminal_block_extent,
        mut float_edge_phase,
    } = *state;
    let line_direction = line_box_context.weak_direction();
    let mut formatted_count = 0;
    let mut outcome = NumericWalkOutcome::Exhausted;
    'lines: loop {
        if control.observe_frontier == Some(formatted_count) && tail_frontier.is_none() {
            *tail_frontier = Some(NumericFlowState {
                cursor,
                boundary_index,
                pending_strut,
                indent_state,
                line_box_context,
                block_cursor,
                float_strut,
                continuation_inline_cursor,
                advanced_phase_transition,
                resolved_terminal_block_extent,
                float_edge_phase,
            });
        }
        let goal = control.goal.filter(|_| {
            formatted_count >= control.goal_start
                && control.goal_end.is_none_or(|end| formatted_count < end)
        });
        let line_indent = input.indent.at(indent_state);
        let next_boundary_descriptor = line_boundaries
            .peek()
            .map_or(supplied.len(), |boundary| boundary.descriptor);
        let provisional = loop {
            let step = select_scope_inline_line(
                NumericLineQuery {
                    goal,
                    indent: line_indent,
                    run,
                    cursor,
                    next_boundary_descriptor,
                    available: available - line_indent,
                    wraps,
                    pending: pending_strut,
                    axes: input.flow_axes,
                },
                input.wrapping.as_ref(),
                |ending, content| {
                    prepare_receiving_seed(
                        pending_strut,
                        input.inline_baseline_context,
                        ending,
                        content,
                        receiver_source,
                    )
                },
                |receipt| {
                    validate_carried_reference(
                        pending_strut,
                        input.inline_baseline_context,
                        receipt.end.line_end(),
                        receipt.summary.has_in_flow_content,
                    )?;
                    resolve_numeric_candidate(
                        receipt,
                        float_strut,
                        block_cursor,
                        line_indent,
                        available,
                        &mut band_source,
                    )
                },
            )?;
            match step {
                InlineSelectionStep::Selected(candidate) => break candidate,
                InlineSelectionStep::RetryReceiver(seed) => pending_strut = Some(seed),
            }
        };
        let Some(mut provisional) = provisional else {
            break;
        };
        if NumericLineDisposition::for_line(
            provisional.end.line_end(),
            provisional.line.has_in_flow_content,
        ) == NumericLineDisposition::NonFormatted
        {
            provisional.line.incoming_strut = pending_strut;
            provisional.line.ending = provisional.end.line_end();
            provisional.line.next_cursor = provisional.next_cursor;
            provisional.line.indent = line_indent;
            provisional.line.alignment =
                numeric_line_alignment(input.text_align, input.text_align_last, provisional.end);
            provisional.line.band = LogicalLineBandOf {
                inline_start: S::ZERO,
                inline_end: available,
                block_start: block_cursor,
                block_end: block_cursor,
            };
            cursor = provisional.next_cursor;
            pending_strut = provisional.pending_strut;
            let ending = provisional.line.ending;
            selected_lines.push(provisional.line);
            if let Some(stop) = control.stop(ending, formatted_count) {
                outcome = stop;
                break;
            }
            continue;
        }
        validate_carried_reference(
            pending_strut,
            input.inline_baseline_context,
            provisional.end.line_end(),
            provisional.line.has_in_flow_content,
        )?;
        if let Some(error) = provisional.line.numeric_error {
            return Err(error.into());
        }
        if float_strut.is_some() {
            let _ = resolve_float_strut_phase(&mut provisional.line, float_strut, input.flow_axes)?;
        }
        let provisional_block_end = consumed_line_end(
            block_cursor,
            provisional.line.baseline,
            provisional.line.after_baseline,
            provisional.line.has_displacement,
        )?;
        let mut queried_band = band_source(block_cursor, provisional_block_end);
        let band_inline_end = queried_band.inline_end.max(queried_band.inline_start);
        let band_available = band_inline_end - queried_band.inline_start;
        let mut has_float_transition = queried_band.next_transition.is_some();

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
        let (mut band_inline_start, mut band_inline_end, mut selected) = if use_containing_band {
            (S::ZERO, available, provisional)
        } else {
            let selected = select_scope_inline_line(
                NumericLineQuery {
                    goal,
                    indent: line_indent,
                    run,
                    cursor,
                    next_boundary_descriptor,
                    available: band_available - line_indent,
                    wraps,
                    pending: pending_strut,
                    axes: input.flow_axes,
                },
                input.wrapping.as_ref(),
                |ending, content| {
                    prepare_receiving_seed(
                        pending_strut,
                        input.inline_baseline_context,
                        ending,
                        content,
                        receiver_source,
                    )
                },
                |receipt| {
                    validate_carried_reference(
                        pending_strut,
                        input.inline_baseline_context,
                        receipt.end.line_end(),
                        receipt.summary.has_in_flow_content,
                    )?;
                    resolve_numeric_candidate(
                        receipt,
                        float_strut,
                        block_cursor,
                        line_indent,
                        available,
                        &mut band_source,
                    )
                },
            )?;
            let selected = match selected {
                InlineSelectionStep::Selected(candidate) => candidate
                    .expect("a provisional line remains selectable against its queried band"),
                InlineSelectionStep::RetryReceiver(seed) => {
                    pending_strut = Some(seed);
                    continue 'lines;
                }
            };
            (queried_band.inline_start, band_inline_end, selected)
        };
        if let Some(band) = selected.line.scope_band {
            queried_band = band;
            has_float_transition = band.next_transition.is_some();
            band_inline_start = band.inline_start;
            band_inline_end = band.inline_end.max(band.inline_start);
            if band_inline_end == band_inline_start && band.next_transition.is_none() {
                band_inline_start = S::ZERO;
                band_inline_end = available;
            }
        }
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
            resolve_float_strut_phase(&mut selected.line, float_strut, input.flow_axes)?
        } else {
            float_strut
        };
        resolve_completed_fallback_envelope(
            &mut selected.line,
            input.flow_axes,
            has_float_transition,
        );
        let selected_block_end = consumed_line_end(
            block_cursor,
            selected.line.baseline,
            selected.line.after_baseline,
            selected.line.has_displacement,
        )?;
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
        line.incoming_strut = pending_strut;
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
            boundary_index += 1;
        }
        cursor = selected.next_cursor;
        line.alignment =
            numeric_line_alignment(input.text_align, input.text_align_last, selected.end);
        line.ending = selected.end.line_end();
        line.next_cursor = selected.next_cursor;
        if let Some(lines) = formatted_lines.as_mut() {
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
        let line_inline_start = selected_line_inline_start(
            &line,
            input.flow_axes,
            line.alignment,
            line_direction,
            if matches!(input.available_inline_extent, AvailableOf::Definite(_)) {
                input.groups.insets::<S>(line_box_context.resolve(None))?
            } else {
                surgeist_text::LineAlignmentInsets::ZERO
            },
        )?;
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
        )?;
        continuation_inline_cursor = float_strut.is_some().then_some(line_inline_end);
        float_strut = next_float_strut;
        advanced_phase_transition = false;
        block_cursor =
            clear_source(line_block_end, line.post_line_clear_intent).max(line_block_end);
        let ending = line.ending;
        selected_lines.push(line);
        formatted_count += 1;
        if let Some(stop) = control.stop(ending, formatted_count) {
            outcome = stop;
            break;
        }
    }

    *state = NumericFlowState {
        cursor,
        boundary_index,
        pending_strut,
        indent_state,
        line_box_context,
        block_cursor,
        float_strut,
        continuation_inline_cursor,
        advanced_phase_transition,
        resolved_terminal_block_extent,
        float_edge_phase,
    };
    Ok(outcome)
}

#[derive(Clone, Copy, PartialEq)]
struct NumericGroupTerminal<S: LayoutScalar> {
    ending: surgeist_text::LineEnd,
    cursor: InlineSourceCursor,
    control: Option<ForcedLineBreakControlOf<S>>,
}
impl<S: LayoutScalar> NumericGroupTerminal<S> {
    fn of(line: &SelectedInlineLineOf<S>) -> Self {
        Self {
            ending: line.ending,
            cursor: line.next_cursor,
            control: line.line_break,
        }
    }
}
struct NumericReferenceGroup<S: LayoutScalar> {
    terminal: NumericGroupTerminal<S>,
    count: usize,
    residuals: Option<Vec<NaturalResidual>>,
}
#[derive(Clone, PartialEq)]
struct NumericGroupBasis<S: LayoutScalar> {
    lines: Vec<SelectedInlineLineOf<S>>,
    outgoing: NumericFlowState<S>,
}
struct NumericGroupProgram<S: LayoutScalar> {
    goal: Option<BalanceGoal>,
    basis: NumericGroupBasis<S>,
}
#[derive(Clone, Copy)]
struct NumericGroupAttempt<'a, S: LayoutScalar> {
    entry: NumericGroupEntry<S>,
    reference: &'a NumericReferenceGroup<S>,
    goal: Option<BalanceGoal>,
}
fn numeric_line_residual<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
) -> Result<NaturalResidual, crate::inline_wrapping::NumericWrappingError> {
    NaturalResidual::from_components(
        (line.band.inline_end - line.band.inline_start).to_f64(),
        line.indent.to_f64(),
        line.used_inline_extent.to_f64(),
    )
    .ok_or(crate::inline_wrapping::NumericWrappingError::InvalidBalanceBasis)
}
fn numeric_is_formatted<S: LayoutScalar>(line: &SelectedInlineLineOf<S>) -> bool {
    NumericLineDisposition::for_line(line.ending, line.has_in_flow_content)
        == NumericLineDisposition::Formatted
}
fn numeric_reference_groups<S: LayoutScalar>(
    lines: &[SelectedInlineLineOf<S>],
) -> Result<Vec<NumericReferenceGroup<S>>, crate::inline_wrapping::NumericWrappingError> {
    lines
        .split_inclusive(|line| line.ending != surgeist_text::LineEnd::Soft)
        .map(|group| {
            let last = group
                .last()
                .ok_or(crate::inline_wrapping::NumericWrappingError::InvalidBalanceBasis)?;
            let count = group
                .iter()
                .filter(|line| numeric_is_formatted(line))
                .count();
            let residuals = if (2..=10).contains(&count) {
                Some(
                    group
                        .iter()
                        .filter(|line| numeric_is_formatted(line))
                        .map(numeric_line_residual)
                        .collect::<Result<_, _>>()?,
                )
            } else {
                None
            };
            Ok(NumericReferenceGroup {
                terminal: NumericGroupTerminal::of(last),
                count,
                residuals,
            })
        })
        .collect()
}
fn numeric_group_trial<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    attempt: NumericGroupAttempt<'_, S>,
    band_source: &mut B,
    clear_source: &mut C,
) -> Result<
    Option<crate::inline_wrapping::balance::GroupTrial<NumericGroupBasis<S>>>,
    crate::inline_wrapping::NumericWrappingError,
>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let NumericGroupAttempt {
        entry,
        reference,
        goal,
    } = attempt;
    sequence.restore(entry);
    let mut remaining = boundaries[entry.state.boundary_index..]
        .iter()
        .copied()
        .peekable();
    let outcome = walk_numeric_sequence(
        request,
        sequence,
        &mut remaining,
        NumericWalkControl::group(goal.map(LineQualityGoal::Balance), Some(reference.count)),
        band_source,
        clear_source,
    )?;
    if outcome != NumericWalkOutcome::GroupComplete {
        return Ok(None);
    }
    let suffix = &sequence.selected_lines[entry.selected_len..];
    let Some(last) = suffix.last() else {
        return Ok(None);
    };
    if NumericGroupTerminal::of(last) != reference.terminal
        || suffix
            .iter()
            .filter(|line| numeric_is_formatted(line))
            .count()
            != reference.count
    {
        return Ok(None);
    }
    let residuals = if reference.residuals.is_some() {
        suffix
            .iter()
            .filter(|line| numeric_is_formatted(line))
            .map(numeric_line_residual)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    Ok(Some(crate::inline_wrapping::balance::GroupTrial {
        residuals,
        value: NumericGroupBasis {
            lines: suffix.to_vec(),
            outgoing: sequence.state,
        },
    }))
}
fn replay_numeric_auto<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    initial: NumericGroupEntry<S>,
    band_source: &mut B,
    clear_source: &mut C,
) -> Result<(), crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    sequence.restore(initial);
    let mut cursor = boundaries.iter().copied().peekable();
    walk_numeric_sequence(
        request,
        sequence,
        &mut cursor,
        NumericWalkControl::auto(),
        band_source,
        clear_source,
    )?;
    Ok(())
}
fn balance_numeric_sequence<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    mut band_source: B,
    mut clear_source: C,
) -> Result<(), crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    use crate::inline_wrapping::balance::{choose_group, variance_order};
    use std::cmp::Ordering;
    let invalid = crate::inline_wrapping::NumericWrappingError::InvalidBalanceBasis;
    let initial = sequence.checkpoint();
    replay_numeric_auto(
        request,
        boundaries,
        sequence,
        initial,
        &mut band_source,
        &mut clear_source,
    )?;
    let references = numeric_reference_groups(&sequence.selected_lines)?;
    sequence.restore(initial);
    let mut program = Vec::with_capacity(references.len());
    let mut improved = false;
    for reference in &references {
        let entry = sequence.checkpoint();
        let auto = numeric_group_trial(
            request,
            boundaries,
            sequence,
            NumericGroupAttempt {
                entry,
                reference,
                goal: None,
            },
            &mut band_source,
            &mut clear_source,
        )?;
        let (goal, trial) = if let Some(residuals) = &reference.residuals {
            let winner = choose_group(residuals, auto, |goal| {
                numeric_group_trial(
                    request,
                    boundaries,
                    sequence,
                    NumericGroupAttempt {
                        entry,
                        reference,
                        goal: Some(goal),
                    },
                    &mut band_source,
                    &mut clear_source,
                )
            })?;
            let Some(winner) = winner else {
                return replay_numeric_auto(
                    request,
                    boundaries,
                    sequence,
                    initial,
                    &mut band_source,
                    &mut clear_source,
                );
            };
            improved |= variance_order(&winner.trial.residuals, residuals).ok_or(invalid)?
                == Ordering::Less;
            (winner.goal, winner.trial)
        } else {
            let Some(auto) = auto else {
                return replay_numeric_auto(
                    request,
                    boundaries,
                    sequence,
                    initial,
                    &mut band_source,
                    &mut clear_source,
                );
            };
            // Noneligible groups use one actual local Auto invocation; do not
            // format a long unrelated group again for winner selection.
            program.push(NumericGroupProgram {
                goal: None,
                basis: auto.value,
            });
            continue;
        };
        let replay = numeric_group_trial(
            request,
            boundaries,
            sequence,
            NumericGroupAttempt {
                entry,
                reference,
                goal,
            },
            &mut band_source,
            &mut clear_source,
        )?;
        let Some(replay) = replay else {
            return replay_numeric_auto(
                request,
                boundaries,
                sequence,
                initial,
                &mut band_source,
                &mut clear_source,
            );
        };
        if replay.value != trial.value {
            return Err(invalid);
        }
        program.push(NumericGroupProgram {
            goal,
            basis: replay.value,
        });
    }
    // One whole-program replay starts at the true incoming state. Group slices
    // index the one authentic hard-boundary inventory; earlier prefixes are
    // borrowed and never replayed by another group's target portfolio.
    sequence.restore(initial);
    for (reference, program) in references.iter().zip(&program) {
        let entry = sequence.checkpoint();
        let replay = numeric_group_trial(
            request,
            boundaries,
            sequence,
            NumericGroupAttempt {
                entry,
                reference,
                goal: program.goal,
            },
            &mut band_source,
            &mut clear_source,
        )?;
        let Some(replay) = replay else {
            return replay_numeric_auto(
                request,
                boundaries,
                sequence,
                initial,
                &mut band_source,
                &mut clear_source,
            );
        };
        if replay.value != program.basis {
            return Err(invalid);
        }
        if let Some(reference) = &reference.residuals
            && variance_order(&replay.residuals, reference).ok_or(invalid)? == Ordering::Greater
        {
            return replay_numeric_auto(
                request,
                boundaries,
                sequence,
                initial,
                &mut band_source,
                &mut clear_source,
            );
        }
    }
    let before = sequence.selected_lines.len();
    let mut remaining = boundaries[sequence.state.boundary_index..]
        .iter()
        .copied()
        .peekable();
    if walk_numeric_sequence(
        request,
        sequence,
        &mut remaining,
        NumericWalkControl::auto(),
        &mut band_source,
        &mut clear_source,
    )? != NumericWalkOutcome::Exhausted
        || sequence.selected_lines.len() != before
    {
        return Err(invalid);
    }
    if !improved {
        return replay_numeric_auto(
            request,
            boundaries,
            sequence,
            initial,
            &mut band_source,
            &mut clear_source,
        );
    }
    Ok(())
}

fn layout_mixed_inline_run_from_available<S, BandSource, ClearSource, ReceiverSource>(
    input: MixedInlineRunInputOf<S>,
    supplied: Vec<SuppliedInlineParticipant>,
    available: S,
    mut band_source: BandSource,
    mut clear_source: ClearSource,
    observe_formatted: bool,
    receiver_source: ReceiverSource,
) -> Result<MixedInlineRunReportOf<S>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    BandSource: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    ClearSource: FnMut(S, PostLineClearIntent) -> S,
    ReceiverSource:
        FnMut(
            usize,
            crate::InlineBaselineContext,
        )
            -> Result<ReferencedInlineStrutOf<S>, crate::inline_wrapping::NumericWrappingError>,
{
    let mut boundary_descriptor = 0;
    // Mandatory following breaks also delimit a line. Looking only for forced
    // controls would repeatedly query a suffix of discardable mandatory units.
    let mut original_boundaries = input
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
                edge_consumed: false,
                source: index + 1,
                descriptor: boundary_descriptor,
            })
        })
        .peekable();
    let mut computed_boundaries = input
        .wrapping
        .as_ref()
        .into_iter()
        .flat_map(|wrapping| {
            let mut descriptor = 0;
            std::iter::once((0, 0))
                .chain(input.participants.iter().enumerate().map(move |(i, p)| {
                    if !matches!(p, MixedInlineParticipantOf::ForcedLineBreak(_)) {
                        descriptor += 1;
                    }
                    (i + 1, descriptor)
                }))
                .filter_map(move |(source, descriptor)| {
                    wrapping.forced(source).then_some(InlineSourceCursor {
                        source,
                        descriptor,
                        edge_consumed: false,
                    })
                })
        })
        .peekable();
    let mut line_boundaries = std::iter::from_fn(move || {
        match (original_boundaries.peek(), computed_boundaries.peek()) {
            (Some(a), Some(b)) if a.source == b.source => {
                computed_boundaries.next();
                original_boundaries.next()
            }
            (Some(a), Some(b)) if a.source < b.source => original_boundaries.next(),
            (_, Some(_)) => computed_boundaries.next(),
            (Some(_), None) => original_boundaries.next(),
            (None, None) => None,
        }
    })
    .peekable();
    let balance = match input.text_wrap_style {
        crate::TextWrapStyle::Auto | crate::TextWrapStyle::Stable => false,
        crate::TextWrapStyle::Balance => {
            matches!(input.available_inline_extent, AvailableOf::Definite(_))
        }
        crate::TextWrapStyle::AvoidShortLastLine | crate::TextWrapStyle::Pretty => false,
    };
    let repair_tail = input.text_wrap_style == crate::TextWrapStyle::AvoidShortLastLine
        && matches!(input.available_inline_extent, AvailableOf::Definite(_));
    let pretty = input.text_wrap_style == crate::TextWrapStyle::Pretty
        && matches!(input.available_inline_extent, AvailableOf::Definite(_));
    let mut sequence = NumericSequence::new(
        &input,
        observe_formatted || balance || repair_tail || pretty,
    );
    let receiver_source = std::cell::RefCell::new(receiver_source);
    let request = NumericWalkInput {
        input: &input,
        supplied: &supplied,
        available,
        receiver_source: &receiver_source,
    };
    if balance {
        let boundaries = line_boundaries.collect::<Vec<_>>();
        balance_numeric_sequence(
            request,
            &boundaries,
            &mut sequence,
            &mut band_source,
            &mut clear_source,
        )?;
    } else if repair_tail {
        let boundaries = line_boundaries.collect::<Vec<_>>();
        tail::repair_numeric_tail(
            request,
            &boundaries,
            &mut sequence,
            &mut band_source,
            &mut clear_source,
        )?;
    } else if pretty {
        let boundaries = line_boundaries.collect::<Vec<_>>();
        pretty::layout_pretty_sequence(
            request,
            &boundaries,
            &mut sequence,
            &mut band_source,
            &mut clear_source,
        )?;
    } else {
        walk_numeric_sequence(
            request,
            &mut sequence,
            &mut line_boundaries,
            NumericWalkControl::auto(),
            &mut band_source,
            &mut clear_source,
        )?;
    }
    let NumericSequence {
        state,
        selected_lines,
        formatted_lines,
        ..
    } = sequence;
    let NumericFlowState {
        line_box_context,
        indent_state,
        block_cursor,
        resolved_terminal_block_extent,
        float_edge_phase,
        ..
    } = state;
    let line_direction = input.line_box_context.weak_direction();

    let mut inline_extent = S::ZERO;
    let mut first_baseline = None;
    let mut last_baseline = None;
    let mut first_named_baseline = false;
    let mut last_named_baseline = false;
    let mut atomics = Vec::new();
    let mut controls = Vec::new();
    let mut nonformatted_controls = Vec::new();
    let mut group_lines = Vec::new();
    let group_insets = if matches!(input.available_inline_extent, AvailableOf::Definite(_)) {
        input
            .groups
            .insets::<S>(input.line_box_context.resolve(None))?
    } else {
        surgeist_text::LineAlignmentInsets::ZERO
    };
    let mut post_line_clear_intents = Vec::new();
    let mut line_bands = Vec::new();
    let mut lines = Vec::new();
    let mut line_index = 0;
    for line in selected_lines {
        if NumericLineDisposition::for_line(line.ending, line.has_in_flow_content)
            == NumericLineDisposition::NonFormatted
        {
            let (_, _, inline_starts) = resolved_inline_unit_slots(
                &line,
                &supplied[line.descriptor_range.clone()],
                input.flow_axes,
                line.alignment,
                line_direction,
                surgeist_text::LineAlignmentInsets::ZERO,
            )?;
            for (index, unit) in line.units.into_iter().enumerate() {
                let MixedInlineParticipantOf::Boundary(control) = unit.participant else {
                    unreachable!("nonformatted numeric participation contains only boundaries");
                };
                nonformatted_controls.push(NonFormattedInlineControlOf {
                    control,
                    inline_start: inline_starts[index],
                    block_start: line.band.block_start,
                });
            }
            continue;
        }
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
        let named_authority = matches!(
            input.inline_baseline_context,
            crate::InlineBaselineContext::Named(_)
        ) && (line.units.iter().any(|unit| {
            !unit.discarded
                && participant_has_named_baseline(unit.participant, input.inline_baseline_context)
        }) || line.line_break.is_some_and(|control| {
            participant_has_named_baseline(
                MixedInlineParticipantOf::ForcedLineBreak(control),
                input.inline_baseline_context,
            )
        }) || line.incoming_strut.is_some_and(|strut| {
            matches!(strut.reference, crate::InlineBaselineContext::Named(_))
        }));
        if first_baseline.is_none() {
            first_named_baseline = named_authority;
        }
        last_named_baseline = named_authority;
        first_baseline.get_or_insert(line_baseline);
        last_baseline = Some(line_baseline);
        let (line_inline_start, visual_indices, inline_starts) = resolved_inline_unit_slots(
            &line,
            &supplied[line.descriptor_range.clone()],
            input.flow_axes,
            line.alignment,
            line_direction,
            group_insets,
        )?;
        if input.groups.is_active()
            && matches!(input.available_inline_extent, AvailableOf::Definite(_))
        {
            group_lines.push(crate::group_alignment::NaturalLine {
                width: (line.band.inline_end - line.band.inline_start).to_f64(),
                indent: line.indent.to_f64(),
                advance: line.used_inline_extent.to_f64(),
                direction: input.line_box_context.resolve(None),
                used: group_insets,
                witness: crate::group_alignment::Witness::Numeric {
                    participants: line.units.iter().map(|unit| unit.participant).collect(),
                    discarded: line.units.iter().map(|unit| unit.discarded).collect(),
                    descriptor_start: line.descriptor_range.start,
                    next: (
                        line.next_cursor.source,
                        line.next_cursor.descriptor,
                        line.next_cursor.edge_consumed,
                    ),
                    end: line.ending,
                    band: line.band,
                },
            });
        }
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
                        InlineControlAlignment::Baseline | InlineControlAlignment::Shift(_) => line
                            .fallback_line_band
                            .and_then(|band| {
                                item.fallback_block_start_in_band(input.flow_axes, band)
                                    .map(|offset| (offset, band.baseline))
                            })
                            .map_or_else(
                                || line_baseline - item.baseline_offset(input.flow_axes),
                                |(offset, raw_baseline)| {
                                    block_start + offset + line.baseline - raw_baseline
                                },
                            ),
                        InlineControlAlignment::Top | InlineControlAlignment::Bottom => {
                            let edge = if item.alignment == InlineControlAlignment::Top {
                                input.flow_axes.line_over()
                            } else {
                                input.flow_axes.line_under()
                            };
                            // Align the margin edge; retain the border origin for
                            // the existing single physical projection.
                            if edge == input.flow_axes.block_start() {
                                block_start + logical_margin.block_start
                            } else {
                                block_start + line_block_extent
                                    - logical_margin.block_end
                                    - logical_size.block
                            }
                        }
                    };
                    let item_block_start = displaced_block_coordinate(
                        item.alignment,
                        input.flow_axes,
                        item_block_start,
                    )?;
                    atomics.push(AtomicInlineSourceOf {
                        item,
                        position: InlineSourcePositionOf::Logical(LogicalPointOf::new(
                            inline_start + logical_margin.inline_start,
                            item_block_start,
                        )),
                        line_index,
                        visual_index: Some(visual_indices[source_index]),
                    });
                }
                MixedInlineParticipantOf::Boundary(control) => {
                    let control_block = control_block_position(
                        control.alignment(),
                        control.metrics(),
                        block_start,
                        line_baseline,
                        line.baseline + line.after_baseline,
                        input.flow_axes,
                    )?;
                    controls.push(InlineControlSourceOf {
                        kind: inline_boundary_layout_kind(control.kind()),
                        source_index: control.source_index(),
                        position: InlineSourcePositionOf::Logical(LogicalPointOf::new(
                            inline_start,
                            control_block,
                        )),
                        inline_extent: control.edge().advance(),
                        endpoint: Some(InlineEndpointOf::from_control(control)),
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
                input.flow_axes,
            )?;
            controls.push(InlineControlSourceOf {
                kind: InlineParticipantLayoutKind::ForcedLineBreak,
                source_index: control.source_index(),
                position: InlineSourcePositionOf::Logical(LogicalPointOf::new(
                    line_inline_start + line.used_inline_extent,
                    control_block,
                )),
                inline_extent: S::ZERO,
                endpoint: None,
                line_index,
                visual_index: None,
            });
        }
        line_index += 1;
    }

    Ok(MixedInlineRunReportOf {
        baselines: InlineBaselineReportOf::Numeric(NumericBaselineReportOf {
            context: input.inline_baseline_context,
            first: first_baseline,
            last: last_baseline,
            first_named: first_named_baseline,
            last_named: last_named_baseline,
        }),
        group_lines,
        formatted_lines,
        line_box_context,
        lines,
        hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
        indent_state,
        inline_extent,
        block_extent: block_cursor.max(resolved_terminal_block_extent),
        float_edge_phase,
        atomics,
        controls,
        nonformatted_controls,
        post_line_clear_intents,
        line_bands,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AtomicInlineBoxParticipant<S: LayoutScalar = DefaultScalar> {
    pub baseline_reference: crate::InlineBaselineContext,
    pub source_index: usize,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub margin: Edges<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub scrollbar_size: Size<S>,
    pub first_baseline: Option<S>,
    pub alignment: InlineControlAlignment<S>,
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
        if !self.alignment.is_baseline_relative() || self.first_baseline.is_some() {
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
    alignment: InlineControlAlignment<S>,
    metrics: InlineMetricsOf<S>,
    line_block_start: S,
    line_baseline: S,
    line_block_extent: S,
    flow_axes: FlowAxes,
) -> Result<S, NumericWrappingError> {
    let position = match alignment {
        InlineControlAlignment::Baseline | InlineControlAlignment::Shift(_) => line_baseline,
        InlineControlAlignment::Top => line_block_start + metrics.baseline(),
        InlineControlAlignment::Bottom => {
            line_block_start + line_block_extent - metrics.after_baseline()
        }
    };
    displaced_block_coordinate(alignment, flow_axes, position)
}

/// Convert line-over displacement to the numeric fitter's block coordinate.
fn block_displacement<S: LayoutScalar>(alignment: InlineControlAlignment<S>, axes: FlowAxes) -> S {
    let InlineControlAlignment::Shift(offset) = alignment else {
        return S::ZERO;
    };
    if axes.line_over() == axes.block_start() {
        -offset.get()
    } else {
        offset.get()
    }
}

fn displaced_block_coordinate<S: LayoutScalar>(
    alignment: InlineControlAlignment<S>,
    axes: FlowAxes,
    position: S,
) -> Result<S, NumericWrappingError> {
    if let InlineControlAlignment::Shift(_) = alignment {
        Ok(crate::baseline_offset::displaced(
            position,
            block_displacement(alignment, axes),
        )?)
    } else {
        Ok(position)
    }
}

fn consumed_line_end<S: LayoutScalar>(
    start: S,
    baseline: S,
    after: S,
    displaced: bool,
) -> Result<S, NumericWrappingError> {
    if displaced {
        crate::baseline_offset::displaced(start, baseline)?;
        let extent = crate::baseline_offset::displaced(baseline, after)?;
        Ok(crate::baseline_offset::displaced(start, extent)?)
    } else {
        Ok(start + baseline + after)
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

pub(super) use crate::VerticalAlignOf as InlineControlAlignment;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ForcedLineBreakControlOf<S: LayoutScalar = DefaultScalar> {
    source_index: usize,
    flow: InlineFlowOf<S>,
    metrics: InlineMetricsOf<S>,
    alignment: InlineControlAlignment<S>,
    clear: Clear,
}

impl<S: LayoutScalar> ForcedLineBreakControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        source_index: usize,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment<S>,
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
    pub(super) const fn alignment(self) -> InlineControlAlignment<S> {
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
    alignment: InlineControlAlignment<S>,
    edge: InlineBoundaryEdgeOf<S>,
}

impl<S: LayoutScalar> InlineBoundaryControlOf<S> {
    #[must_use]
    pub(super) const fn new(
        source_index: usize,
        kind: InlineBoundaryKind,
        flow: InlineFlowOf<S>,
        metrics: InlineMetricsOf<S>,
        alignment: InlineControlAlignment<S>,
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
    pub(super) const fn alignment(self) -> InlineControlAlignment<S> {
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
            baseline_reference: crate::InlineBaselineContext::Physical,
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
        inline_baseline_context: crate::InlineBaselineContext::Physical,
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            flow_axes.direction(),
        ),
        observe_tail: false,
        wrapping: None,
        available_inline_extent: input.available_width,
        flow_axes,
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants,
    })
    .expect("test-only auto numeric layout has no scope demand");
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
                    expect_logical_position(source.position),
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
                    expect_logical_position(source.position),
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
        first_baseline: report.baselines.numeric().first,
        last_baseline: report.baselines.numeric().last,
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

#[cfg(test)]
mod balance_tests;
#[cfg(test)]
mod bidi_order_tests;
#[cfg(test)]
mod phantom_completion_tests;
mod pretty;
mod tail;

fn participant_has_named_baseline<S: LayoutScalar>(
    participant: MixedInlineParticipantOf<S>,
    reference: crate::InlineBaselineContext,
) -> bool {
    match participant {
        MixedInlineParticipantOf::Atomic { item, .. } => {
            item.alignment.is_baseline_relative() && item.first_baseline.is_some()
        }
        MixedInlineParticipantOf::Boundary(control) => {
            control.alignment().is_baseline_relative() && control.metrics().reference() == reference
        }
        MixedInlineParticipantOf::ForcedLineBreak(control) => {
            control.alignment().is_baseline_relative() && control.metrics().reference() == reference
        }
    }
}

fn prepare_receiving_seed<S: LayoutScalar>(
    strut: Option<ReferencedInlineStrutOf<S>>,
    expected: crate::InlineBaselineContext,
    ending: SelectedInlineLineEnd,
    content: bool,
    source: &std::cell::RefCell<AtomicReceiverSource<'_, S>>,
) -> Result<Option<ReferencedInlineStrutOf<S>>, crate::inline_wrapping::NumericWrappingError> {
    if NumericLineDisposition::for_line(ending.line_end(), content)
        == NumericLineDisposition::NonFormatted
    {
        return Ok(None);
    }
    let Some(strut) = strut else {
        return Ok(None);
    };
    if strut.reference == expected {
        return Ok(None);
    }
    if let InlineStrutSource::Atomic { source_index } = strut.source
        && strut.reference == crate::InlineBaselineContext::Physical
        && matches!(expected, crate::InlineBaselineContext::Named(_))
    {
        let resolved = source.borrow_mut()(source_index, expected)?;
        if resolved.source != strut.source
            || resolved.reference != expected
            || !resolved.metrics.baseline.is_finite()
            || !resolved.metrics.after_baseline.is_finite()
        {
            return Err(crate::inline_wrapping::NumericWrappingError::InvalidAtomicReceiverBasis);
        }
        return Ok(Some(resolved));
    }
    validate_carried_reference(Some(strut), expected, ending.line_end(), content)?;
    Ok(None)
}

fn validate_carried_reference<S: LayoutScalar>(
    strut: Option<ReferencedInlineStrutOf<S>>,
    expected: crate::InlineBaselineContext,
    ending: surgeist_text::LineEnd,
    content: bool,
) -> Result<(), crate::inline_wrapping::NumericWrappingError> {
    if NumericLineDisposition::for_line(ending, content) == NumericLineDisposition::NonFormatted {
        return Ok(());
    }
    if let Some(strut) = strut
        && strut.reference != expected
    {
        return Err(crate::inline_wrapping::NumericWrappingError::Baseline(
            crate::InlineBaselineError::ReferenceMismatch {
                expected,
                actual: strut.reference,
            },
        ));
    }
    Ok(())
}
