//! Candidate fitting over the actual immutable Text operations.

use super::boxes::{
    BoxAssignment, BoxAuthority, BoxChoice, BoxEntry, BoxResolver, FinalizedBoxReceipt,
};
use super::formatted::FormattedTailSnapshot;
pub(crate) use super::roles::{RoleBasis, RoleUse};
use crate::inline_wrapping::balance::{BalanceGoal, NaturalResidual};
use crate::inline_wrapping::quality::{LineQualityGoal, LineQualityKey};
use crate::{TextAlign, TextAlignLast, TextJustification};

#[path = "fit_balance.rs"]
mod balance;
#[path = "fit_pretty.rs"]
mod pretty;
#[path = "fit_tail.rs"]
mod tail;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SelectionProgram {
    Balance(balance::BalanceBasis),
    Tail(tail::TailBasis),
    Pretty(pretty::PrettyBasis),
}
use surgeist_text::{
    Alignment, AtomicAllocation, ConditionalDisposition, ConditionalPurpose, ConditionalStage,
    ControlAllocation, FinalizedLine, HyphenationContinuation, LineContext, LineEnd,
    LineOrientation, LineSelectionRequest, LineStart, ParagraphBaseDirection,
    PhraseOverflowDecision, PreparedAlternative, PreparedBoundary, PreparedBreakKind,
    PreparedParticipant, PreparedPhrasePortion, PreparedTermination, PreparedText, SelectedLine,
    SpacingAllocation, SpacingAllocations, SpacingPolicy, SpacingSlotDisposition, TabContext,
    TabPreparation,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Band {
    pub start: f64,
    pub end: f64,
    /// Only an actual captured intrinsic-Max query lends this end-side receipt.
    pub intrinsic_end_occupation: Option<f64>,
    pub next_transition: Option<f64>,
}

/// A source-line encounter checkpoint is not the mutable text avoidance cursor.
#[derive(Clone)]
pub(crate) struct EncounterReplay {
    pub boxes: Option<BoxAssignment>,
    /// Actual preceding source encounters of a pure portion measurement.
    pub preceding: Option<PrecedingEncounter>,
    pub source_floor: f64,
    pub logical: f64,
    pub placed: f64,
    pub defer_preceding: bool,
}

impl EncounterReplay {
    fn with_boxes(mut self, boxes: Option<BoxAssignment>) -> Self {
        self.boxes = boxes;
        self
    }
}

/// Actual receipt and placed anchor belong to one accepted trial frame.
#[derive(Clone)]
pub(crate) struct PrecedingEncounter {
    pub boxes: Option<BoxAssignment>,
    pub line: FinalizedLine,
    pub placed: f64,
}

/// Constraints from the two actual footprints, never their bounding union.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineBands {
    pub allocation: Band,
    pub metrics: Band,
}

impl LineBands {
    pub(super) fn combined(self) -> Band {
        Band {
            start: self.allocation.start.max(self.metrics.start),
            end: self.allocation.end.min(self.metrics.end),
            intrinsic_end_occupation: match (
                self.allocation.intrinsic_end_occupation,
                self.metrics.intrinsic_end_occupation,
            ) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            },
            next_transition: earliest(
                self.allocation.next_transition,
                self.metrics.next_transition,
            ),
        }
    }

    fn next_transition(self, logical_start: f64, placed_start: f64) -> Option<f64> {
        earliest(
            self.allocation
                .next_transition
                .filter(|next| next.is_finite() && *next > logical_start),
            self.metrics
                .next_transition
                .filter(|next| next.is_finite() && *next > placed_start),
        )
    }
}

fn earliest(first: Option<f64>, second: Option<f64>) -> Option<f64> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (first, second) => first.or(second),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Available {
    Definite(f64),
    Minimum,
    Maximum,
}

/// Completion authority supplied by the actual owning block traversal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum TailContext {
    #[default]
    Unrestricted,
    CompleteBlock,
    Unresolved,
}

/// Reset one candidate or the entire unpublished sequence to its incoming ledger.
#[derive(Clone, Copy)]
pub(crate) enum EncounterReset {
    Candidate,
    Sequence,
    Group,
}

/// Actual source-event ordinal and accepted scalar geometry for cycle identity.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EncounterBasis {
    pub boxes: Option<BoxAssignment>,
    pub event: usize,
    pub prefix: f64,
    pub preceding: bool,
    pub encounter_floor: f64,
    pub intrinsic_inline_demand: Option<f64>,
    pub inline: f64,
    pub block: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct CandidateBasis {
    disposition: LineDisposition,
    encounters: Vec<EncounterBasis>,
    request: LineSelectionRequest,
    boxes: Option<BoxAssignment>,
    amounts: Vec<f64>,
    natural_advance: f64,
    logical_start: f64,
    placed_start: f64,
    bands: LineBands,
    height: f64,
    minimum_block: Option<f64>,
}

/// A proposed line position and genuine earlier float/clearance constraint.
pub(crate) struct BlockPosition {
    pub proposed: f64,
    pub minimum: Option<f64>,
    /// Actual used allocation height paired with the callback's finalized baseline.
    pub height: f64,
}

/// Accepted logical allocation and any genuine following clearance movement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineFlow {
    pub start: f64,
    pub end: f64,
    pub next: f64,
}

/// Numerical finalization and used CSS line existence are distinct. The
/// nonformatted capability retains the exact receipt without allocating a line.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LineDisposition {
    Formatted,
    NonFormatted(surgeist_text::NonFormattedLine),
}
impl LineDisposition {
    pub(crate) fn resolve(line: &FinalizedLine) -> Result<Self, FitError> {
        let selected = line.selected_line();
        let request = selected.request();
        if request.context.end == LineEnd::Forced
            || line.has_in_flow_content()
            || request
                .controls
                .iter()
                .any(|control| control.boundary.is_some_and(|edge| edge.advance() != 0.0))
        {
            Ok(Self::Formatted)
        } else {
            Ok(Self::NonFormatted(line.nonformatted()?))
        }
    }
    pub(crate) fn is_formatted(&self) -> bool {
        matches!(self, Self::Formatted)
    }
    pub(crate) fn height(&self, line: &FinalizedLine) -> f64 {
        if self.is_formatted() {
            line.feedback().line_height
        } else {
            0.0
        }
    }
    pub(crate) fn baseline(&self, line: &FinalizedLine) -> f64 {
        if self.is_formatted() {
            line.feedback().baseline
        } else {
            0.0
        }
    }
}

#[derive(Clone)]
pub(crate) struct FittedLine {
    pub disposition: LineDisposition,
    pub boxes: Option<BoxAssignment>,
    pub box_uses: Vec<super::boxes::BoxTrialUse>,
    pub first_formatted: bool,
    pub role_uses: Vec<RoleUse>,
    pub line_box_end: crate::line_box::LineBoxContext,
    pub zone_basis: ZoneBasis,
    pub phrase_basis: Vec<PhraseBasis>,
    pub hyphenation_end: crate::hyphenation::LineHyphenationState,
    pub encounters: Vec<EncounterBasis>,
    pub finalized: FinalizedLine,
    pub amounts: Vec<f64>,
    pub natural_advance: f64,
    pub program: Option<std::sync::Arc<SelectionProgram>>,
    pub inline_start: f64,
    pub placed_start: f64,
    pub available: f64,
    pub bands: LineBands,
    pub flow: LineFlow,
}

#[derive(Clone)]
pub(crate) struct ReusableLine {
    pub finalized: FinalizedLine,
    pub amounts: Vec<f64>,
}

pub(crate) enum FitError {
    Text(surgeist_text::Error),
    TextWritingMode(crate::WritingMode),
    Wrapping(crate::inline_wrapping::WrappingScopeError),
    FeedbackCycle,
    InvalidTailBasis,
    InvalidPrettyBasis,
    SpacingBudget { available: f64, advance: f64 },
    NonFinite(f64),
    Numeric(f64, crate::LayoutTextNumericError),
    InvalidGeometry,
    Dependency,
    MissingZoneBasis,
    MissingBlockTail,
    MissingComputedBoundaryIncidence,
}

impl From<crate::scalar::GeometryAddError> for FitError {
    fn from(error: crate::scalar::GeometryAddError) -> Self {
        match error.reason {
            crate::LayoutTextNumericError::NonFinite => Self::NonFinite(error.value),
            reason => Self::Numeric(error.value, reason),
        }
    }
}

impl From<surgeist_text::Error> for FitError {
    fn from(error: surgeist_text::Error) -> Self {
        Self::Text(error)
    }
}

impl From<crate::inline_wrapping::WrappingScopeError> for FitError {
    fn from(error: crate::inline_wrapping::WrappingScopeError) -> Self {
        Self::Wrapping(error)
    }
}

pub(crate) type WrappingClassifier<'a> = dyn BoxResolver + 'a;

pub(crate) struct FitInput<'a> {
    pub groups: crate::group_alignment::Snapshot,
    pub wrapping: Option<&'a WrappingClassifier<'a>>,
    pub formatted: &'a FormattedTailSnapshot,
    pub line_box_context: crate::line_box::LineBoxContext,
    pub prepared: &'a PreparedText,
    pub available: Available,
    pub base_direction: ParagraphBaseDirection,
    pub block_direction: ParagraphBaseDirection,
    pub alignment: TextAlign,
    pub wrap_style: crate::TextWrapStyle,
    pub last_alignment: TextAlignLast,
    pub justification: TextJustification,
    pub indent: crate::text_indent::ResolvedTextIndent<f64>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub hyphenate_limit_zone: crate::LengthPercentageOf<f64>,
    pub tail: TailContext,
    pub orientation: LineOrientation,
    pub atomics: &'a [AtomicAllocation],
    pub controls: &'a [ControlAllocation],
    pub line_strut: Option<surgeist_text::LineStrut>,
    pub reusable: &'a [ReusableLine],
}

pub(crate) struct LineAllocations {
    pub line_box_struts: Vec<surgeist_text::LineBoxStrut>,
    pub atomics: Vec<AtomicAllocation>,
    pub controls: Vec<ControlAllocation>,
    pub line_strut: Option<surgeist_text::LineStrut>,
}

struct FinalizedCandidate {
    request: LineSelectionRequest,
    finalized: FinalizedLine,
    amounts: Vec<f64>,
    natural_advance: f64,
}

fn text_alignment(alignment: TextAlign) -> Alignment {
    match alignment {
        TextAlign::Start => Alignment::Start,
        TextAlign::End => Alignment::End,
        TextAlign::Left | TextAlign::LegacyLeft => Alignment::Left,
        TextAlign::Right | TextAlign::LegacyRight => Alignment::Right,
        TextAlign::Center | TextAlign::LegacyCenter => Alignment::Center,
        TextAlign::Justify => Alignment::Justify,
    }
}

fn natural_line(
    prepared: &PreparedText,
    reusable: &[ReusableLine],
    request: &LineSelectionRequest,
) -> Result<FinalizedLine, FitError> {
    if let Some(line) = reusable.iter().find(|line| {
        line.finalized.selected_line().request() == request
            && line.amounts.iter().all(|amount| *amount == 0.0)
    }) {
        return Ok(line.finalized.clone());
    }
    Ok(prepared
        .select(request.clone())?
        .finalize(SpacingAllocations::None)?)
}

/// Rebuild the reached facts for this actual request. Context rows allocate no
/// flow; only Text chooses their authenticated encounter order and stopping edge.
fn resolve_boundary_context(
    prepared: &PreparedText,
    controls: &[ControlAllocation],
    request: &mut LineSelectionRequest,
) -> Result<(), FitError> {
    request.boundary_context.clear();
    while let Some(occurrence) = prepared.boundary_context_demand(request)? {
        let boundary = controls
            .iter()
            .find(|row| row.occurrence == occurrence)
            .and_then(|row| row.boundary)
            .ok_or(FitError::Dependency)?;
        // An issued demand must add a new fact. Preserve bounded progress if
        // an inconsistent preparation ever repeats an already supplied handle.
        if request
            .boundary_context
            .iter()
            .any(|row| row.occurrence == occurrence)
        {
            return Err(FitError::Dependency);
        }
        request
            .boundary_context
            .push(surgeist_text::InlineBoundaryContextAllocation {
                occurrence,
                boundary,
            });
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct FinalizationAlignment {
    normal: TextAlign,
    last: TextAlignLast,
    justification: TextJustification,
}

fn finalize_candidate(
    prepared: &PreparedText,
    controls: &[ControlAllocation],
    reusable: &[ReusableLine],
    mut request: LineSelectionRequest,
    alignment: FinalizationAlignment,
    active_request: &mut LineSelectionRequest,
) -> Result<FinalizedCandidate, FitError> {
    let FinalizationAlignment {
        normal,
        last,
        justification,
    } = alignment;
    let effective = normal.for_line(last, request.context.end != LineEnd::Soft);
    request.context.alignment = text_alignment(effective);
    *active_request = request.clone();
    resolve_boundary_context(prepared, controls, &mut request)?;
    let natural = natural_line(prepared, reusable, &request)?;
    let natural_advance = natural.measurements().alignment().advance();
    let selected = natural.selected_line();
    let mut amounts = vec![0.0; selected.spacing_slots().len()];
    if effective == TextAlign::Justify {
        let retained: Vec<_> = natural
            .spacing_slots()
            .enumerate()
            .filter_map(|(index, row)| {
                matches!(row.disposition, SpacingSlotDisposition::Retained { .. }).then_some(index)
            })
            .collect();
        let expansion = request
            .context
            .available_inline
            .map(|width| {
                if request.context.alignment_insets == surgeist_text::LineAlignmentInsets::ZERO {
                    finite(width - request.context.indent).and_then(|capacity| {
                        finite(capacity - natural.measurements().alignment().advance())
                            .map(|extra| (capacity, extra))
                    })
                } else {
                    let capacity = crate::group_alignment::canonical_capacity::<f64>(
                        width,
                        request.context.alignment_insets,
                        request.context.indent,
                    )
                    .map_err(|error| FitError::Numeric(error.value(), error.reason()))?;
                    let extra = crate::group_alignment::add::<f64>(
                        capacity,
                        -natural.measurements().alignment().advance(),
                    )
                    .map_err(|error| FitError::Numeric(error.value(), error.reason()))?;
                    Ok((capacity, extra))
                }
            })
            .transpose()?;
        if let Some((width, extra)) = expansion.filter(|(_, extra)| *extra >= 0.0)
            && (extra == 0.0
                || (justification == TextJustification::Spaces && !retained.is_empty()))
        {
            if extra > 0.0 {
                // Eligibility comes from the natural line; complete allocations
                // then remap its exclusions through the same Text finalizer.
                let share = finite(extra / retained.len() as f64)?;
                let mut supplied = 0.0;
                for (ordinal, index) in retained.iter().enumerate() {
                    amounts[*index] = if ordinal + 1 == retained.len() {
                        finite(extra - supplied)?
                    } else {
                        share
                    };
                    supplied = finite(supplied + amounts[*index])?;
                }
                let finalized = if let Some(line) = reusable.iter().find(|line| {
                    line.finalized.selected_line().request() == &request && line.amounts == amounts
                }) {
                    line.finalized.clone()
                } else {
                    finalize_spacing(&selected, &amounts)?
                };
                let finalized =
                    constrain_spacing_budget(&selected, &retained, width, &mut amounts, finalized)?;
                return Ok(FinalizedCandidate {
                    request,
                    finalized,
                    amounts,
                    natural_advance,
                });
            }
        } else {
            // Pinned CSS Text3 §6.4.3; a new alignment is a new actual request,
            // so no slot capabilities are transferred from the zero pass.
            request.context.alignment = text_alignment(normal.unexpandable(last));
            *active_request = request.clone();
            resolve_boundary_context(prepared, controls, &mut request)?;
            let finalized = natural_line(prepared, reusable, &request)?;
            amounts = vec![0.0; finalized.selected_line().spacing_slots().len()];
            return Ok(FinalizedCandidate {
                request,
                natural_advance: finalized.measurements().alignment().advance(),
                finalized,
                amounts,
            });
        }
    }
    Ok(FinalizedCandidate {
        request,
        finalized: natural,
        amounts,
        natural_advance,
    })
}

/// Both full and natural-only trials pass this boundary. A context change
/// rebuilds selection and complete spacing slots rather than transferring them.
fn finalize_with_roles(
    input: &FitInput<'_>,
    mut request: LineSelectionRequest,
    natural_only: bool,
    entry: &super::roles::RoleEntry<'_>,
) -> Result<(FinalizedCandidate, Option<RoleBasis>), FitError> {
    request.context.formatted_line = surgeist_text::FormattedLineContext::default();
    let mut basis = super::roles::initial_basis(&request, entry.current_first);
    for _ in 0..=2 {
        let mut active = request.clone();
        let result = if natural_only {
            resolve_boundary_context(input.prepared, input.controls, &mut active)?;
            natural_line(input.prepared, input.reusable, &active).map(|finalized| {
                FinalizedCandidate {
                    amounts: vec![0.0; finalized.selected_line().spacing_slots().len()],
                    request: active.clone(),
                    natural_advance: finalized.measurements().alignment().advance(),
                    finalized,
                }
            })
        } else {
            finalize_candidate(
                input.prepared,
                input.controls,
                input.reusable,
                request.clone(),
                FinalizationAlignment {
                    normal: input.alignment,
                    last: input.last_alignment,
                    justification: input.justification,
                },
                &mut active,
            )
        };
        match result {
            Ok(candidate) => return Ok((candidate, basis)),
            Err(FitError::Text(error)) => {
                let role = match error.detail() {
                    Some(surgeist_text::ErrorDetail::InvalidTextProtocol {
                        reason:
                            surgeist_text::ProtocolError::MissingPhaseTwoContext(
                                role @ (surgeist_text::PhaseTwoContext::FirstFormattedLine
                                | surgeist_text::PhaseTwoContext::LastFormattedLine),
                            ),
                    }) => *role,
                    _ => return Err(FitError::Text(error)),
                };
                let Some(value) = super::roles::resolve(input, &active, entry, role, &mut basis)?
                else {
                    return Err(FitError::Text(error));
                };
                let old = active.context.formatted_line;
                let next = match role {
                    surgeist_text::PhaseTwoContext::FirstFormattedLine => {
                        surgeist_text::FormattedLineContext::new(Some(value), old.last())
                    }
                    surgeist_text::PhaseTwoContext::LastFormattedLine => {
                        surgeist_text::FormattedLineContext::new(old.first(), Some(value))
                    }
                    _ => return Err(FitError::Text(error)),
                };
                if old == next {
                    return Err(FitError::Text(error));
                }
                active.context.formatted_line = next;
                request = active;
            }
            Err(error) => return Err(error),
        }
    }
    Err(FitError::FeedbackCycle)
}

/// Every probe executes the complete table on this actual selection. Never
/// reconstruct Text's slot additions or its logical advance fold in Layout.
fn finalize_spacing(selected: &SelectedLine, amounts: &[f64]) -> Result<FinalizedLine, FitError> {
    Ok(selected.finalize(SpacingAllocations::Complete(
        selected
            .spacing_slots()
            .zip(amounts)
            .map(|(slot, &amount)| SpacingAllocation { slot, amount })
            .collect(),
    ))?)
}

fn constrain_spacing_budget(
    selected: &SelectedLine,
    retained: &[usize],
    width: f64,
    amounts: &mut [f64],
    mut finalized: FinalizedLine,
) -> Result<FinalizedLine, FitError> {
    if finalized.measurements().alignment().advance() <= width {
        return Ok(finalized);
    }
    // The ideal sum of amounts does not capture rounding in each real slot's
    // natural+amount addition and the following Text-owned logical fold.
    if let Some(index) = retained
        .iter()
        .rev()
        .copied()
        .find(|&index| amounts[index] > 0.0)
    {
        let previous = amounts[index];
        let overshoot = finite(finalized.measurements().alignment().advance() - width)?;
        let reduced = finite(previous - overshoot)?.max(0.0);
        amounts[index] = if reduced < previous {
            reduced
        } else {
            previous.next_down()
        };
        finalized = finalize_spacing(selected, amounts)?;
        if finalized.measurements().alignment().advance() <= width {
            return Ok(finalized);
        }
    }

    // No search over assumed numeric admission: Text may refuse a nonzero
    // probe that vanishes in an actual slot or later logical addition. A still
    // over-budget completed correction is Layout's own truthful refusal.
    Err(FitError::SpacingBudget {
        available: width,
        advance: finalized.measurements().alignment().advance(),
    })
}

/// Candidate-local coordinates are distinct from the accepted source checkpoint.
#[derive(Clone, Debug, PartialEq)]
struct CandidatePosition {
    source_floor: f64,
    logical_start: f64,
    minimum_block: Option<f64>,
    deferred_from: Option<LineSelectionRequest>,
}

#[derive(Clone)]
struct EvaluatedCandidate {
    boxes: Option<BoxAssignment>,
    request: LineSelectionRequest,
    finalized: FinalizedLine,
    amounts: Vec<f64>,
    natural_advance: f64,
    encounters: Vec<EncounterBasis>,
    bands: LineBands,
    placed_start: f64,
    fits: bool,
    disposition: LineDisposition,
}

impl EvaluatedCandidate {
    fn basis(&self, position: &CandidatePosition) -> CandidateBasis {
        CandidateBasis {
            disposition: self.disposition.clone(),
            encounters: self.encounters.clone(),
            boxes: self.boxes.clone(),
            request: self.request.clone(),
            amounts: self.amounts.clone(),
            natural_advance: self.natural_advance,
            logical_start: position.logical_start,
            placed_start: self.placed_start,
            bands: self.bands,
            height: self.disposition.height(&self.finalized),
            minimum_block: position.minimum_block,
        }
    }

    fn quality_key(&self, goal: LineQualityGoal) -> Option<LineQualityKey> {
        goal.key(
            self.request.context.available_inline?,
            self.request.context.indent,
            self.natural_advance,
        )
    }

    fn observed(&self, position: &CandidatePosition) -> ObservedCandidateBasis {
        ObservedCandidateBasis {
            candidate: self.basis(position),
            fits: self.fits,
            formatted: self.disposition.is_formatted(),
            usage: self.finalized.measurements().formatted_line_usage(),
            feedback: self.finalized.feedback(),
            fit: self.finalized.measurements().fit(),
            alignment: self.finalized.measurements().alignment(),
            metric_bounds: self.finalized.metric_bounds(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ObservedCandidateBasis {
    candidate: CandidateBasis,
    fits: bool,
    formatted: bool,
    usage: surgeist_text::FormattedLineUsage,
    feedback: surgeist_text::LineFeedback,
    fit: surgeist_text::InlineMeasure,
    alignment: surgeist_text::InlineMeasure,
    metric_bounds: surgeist_text::Rect,
}

/// Only the first authentic fitting end and whether another distinct end was
/// reached are needed to decide whether a scope comparison is demanded.
#[derive(Clone, Debug, Default, PartialEq)]
struct FittingEnds {
    first: Option<PreparedBoundary>,
    multiple: bool,
}

impl FittingEnds {
    fn observe(&mut self, candidate: &EvaluatedCandidate, include_terminal: bool) {
        if !candidate.fits
            || !candidate.disposition.is_formatted()
            || (!include_terminal && candidate.request.context.end != LineEnd::Soft)
        {
            return;
        }
        if let Some(first) = &self.first {
            self.multiple |= *first != candidate.request.end;
        } else {
            self.first = Some(candidate.request.end.clone());
        }
    }
}

/// Exact facts actually retained by the ordinary chooser. Discarded first-pass
/// receipts are not a transcript: replay uses the same immutable operation and
/// accepted frame, and contradictions in these observations refuse feedback.
#[derive(Clone, Debug, PartialEq)]
struct ComparisonBasis {
    position: CandidatePosition,
    normal: Option<ObservedCandidateBasis>,
    terminal: Option<(PreparedBoundary, Option<ObservedCandidateBasis>)>,
    progress: Option<ObservedCandidateBasis>,
    ceiling: Option<PreparedBoundary>,
    fitting_none: bool,
    ends: FittingEnds,
}

#[derive(Default)]
struct ScopedChoices {
    goal: Option<LineQualityGoal>,
    frontier: Option<crate::inline_wrapping::PreferenceFrontier<EvaluatedCandidate>>,
    error: Option<FitError>,
}

impl ScopedChoices {
    fn consider(&mut self, classify: &WrappingClassifier<'_>, candidate: EvaluatedCandidate) {
        if self.error.is_some() {
            return;
        }
        let result = candidate
            .boxes
            .as_ref()
            .ok_or(FitError::InvalidGeometry)
            .and_then(|boxes| classify.classify(boxes))
            .and_then(|class| {
                let frontier = self.frontier.get_or_insert_with(|| class.frontier());
                if let Some(goal) = self.goal {
                    candidate
                        .quality_key(goal)
                        .ok_or(FitError::InvalidGeometry)?;
                    frontier.consider_ranked(class, candidate, |left, right| {
                        goal.compare(
                            left.quality_key(goal).expect("finite goal candidate"),
                            right.quality_key(goal).expect("finite goal candidate"),
                        )
                    })
                } else {
                    frontier.consider(class, candidate)
                }
                .map_err(FitError::Wrapping)
            });
        if let Err(error) = result {
            self.error = Some(error);
        }
    }

    fn preferred(self) -> Result<Option<EvaluatedCandidate>, FitError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(self.frontier.and_then(|frontier| {
            if let Some(goal) = self.goal {
                frontier.into_preferred_ranked(|left, right| {
                    goal.compare(
                        left.quality_key(goal).expect("finite goal candidate"),
                        right.quality_key(goal).expect("finite goal candidate"),
                    )
                })
            } else {
                frontier.into_preferred()
            }
        }))
    }
}

fn prefer_candidate(
    goal: Option<LineQualityGoal>,
    candidate: &EvaluatedCandidate,
    current: Option<&EvaluatedCandidate>,
) -> Result<bool, FitError> {
    let Some((goal, current)) = goal.zip(current).filter(|(_, current)| {
        candidate.fits
            && current.fits
            && candidate.disposition.is_formatted()
            && current.disposition.is_formatted()
    }) else {
        return Ok(true);
    };
    let candidate = candidate
        .quality_key(goal)
        .ok_or(FitError::InvalidGeometry)?;
    let current = current.quality_key(goal).ok_or(FitError::InvalidGeometry)?;
    Ok(goal.compare(candidate, current) != std::cmp::Ordering::Greater)
}

#[derive(Clone, Copy)]
struct EndpointContext<'a> {
    accepted_prefix: &'a [FittedLine],
    line_box_context: crate::line_box::LineBoxContext,
    start: &'a PreparedBoundary,
    incoming: Option<&'a HyphenationContinuation>,
    position: &'a CandidatePosition,
    indent_state: crate::text_indent::IndentLineState,
    hyphenation_state: crate::hyphenation::LineHyphenationState,
    rejected_choices: &'a [CandidateKey],
    conditional: bool,
}

#[derive(Clone, Copy)]
struct BoxCheckpoint<'a> {
    entry: BoxEntry<'a>,
    preceding: Option<FinalizedBoxReceipt<'a>>,
}

fn box_checkpoint<'a>(
    start: &PreparedBoundary,
    query: &'a EndpointContext<'_>,
    preceding: Option<&'a EvaluatedCandidate>,
) -> Result<Option<BoxCheckpoint<'a>>, FitError> {
    let checkpoint = |line: &'a FinalizedLine,
                      amounts: &'a [f64],
                      boxes: Option<&'a BoxAssignment>|
     -> Result<Option<BoxCheckpoint<'a>>, FitError> {
        let Some(boxes) = boxes else {
            return Ok(None);
        };
        let selected = line.selected_line();
        let request = selected.request();
        if request.end == *start {
            return Ok(Some(BoxCheckpoint {
                entry: BoxEntry::ConsumedTo(&boxes.outgoing),
                preceding: Some(FinalizedBoxReceipt {
                    line,
                    amounts,
                    boxes,
                }),
            }));
        }
        if request.start == *start {
            return Ok(Some(BoxCheckpoint {
                entry: BoxEntry::ConsumedTo(&boxes.incoming),
                preceding: None,
            }));
        }
        if request.start.prepared_text() == start.prepared_text()
            && request.start.view().participant_offset() < start.view().participant_offset()
            && start.view().participant_offset() < request.end.view().participant_offset()
        {
            let participant = start
                .prepared_text()
                .participants()
                .nth(start.view().participant_offset())
                .ok_or(FitError::InvalidGeometry)?;
            let prefix =
                line.prefix_at(&surgeist_text::TextSeam::BeforeParticipant(participant))?;
            return Ok(Some(BoxCheckpoint {
                entry: if prefix.has_retained_flow_participation() {
                    BoxEntry::ConsumedTo(&boxes.incoming)
                } else {
                    BoxEntry::UnconsumedFrom(&boxes.incoming)
                },
                preceding: None,
            }));
        }
        Ok(None)
    };
    if let Some(candidate) = preceding
        && let Some(entry) = checkpoint(
            &candidate.finalized,
            &candidate.amounts,
            candidate.boxes.as_ref(),
        )?
    {
        return Ok(Some(entry));
    }
    for line in query.accepted_prefix.iter().rev() {
        if let Some(entry) = checkpoint(&line.finalized, &line.amounts, line.boxes.as_ref())? {
            return Ok(Some(entry));
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq)]
struct CandidateKey {
    request: LineSelectionRequest,
    boxes: Option<BoxAssignment>,
}
impl EvaluatedCandidate {
    fn key(&self) -> CandidateKey {
        CandidateKey {
            request: self.request.clone(),
            boxes: self.boxes.clone(),
        }
    }
}
impl FittedLine {
    fn key(&self) -> CandidateKey {
        CandidateKey {
            request: self.finalized.selected_line().request().clone(),
            boxes: self.boxes.clone(),
        }
    }
}

struct EndpointChoice {
    candidate: Option<EvaluatedCandidate>,
    dispositions: Vec<ConditionalDisposition>,
    formatted: bool,
    fitting_none: bool,
}

/// These callbacks all operate on the same unpublished invocation. Reset only
/// restores its accepted encounter checkpoint; it is not a child publication.
struct CandidateEnvironment<B, C, A, R, X> {
    roles: Vec<RoleUse>,
    box_uses: Vec<super::boxes::BoxTrialUse>,
    observed: Option<balance::Observations>,
    band: B,
    canonical: C,
    allocate: A,
    replay: R,
    reset: X,
}

impl<B, C, A, R, X> CandidateEnvironment<B, C, A, R, X>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset),
{
    fn evaluate(
        &mut self,
        input: &FitInput<'_>,
        request: LineSelectionRequest,
        query: &EndpointContext<'_>,
        preceding: Option<&EvaluatedCandidate>,
    ) -> Result<EvaluatedCandidate, FitError> {
        self.evaluate_mode(input, request, query, preceding, false)
    }

    fn evaluate_mode(
        &mut self,
        input: &FitInput<'_>,
        request: LineSelectionRequest,
        query: &EndpointContext<'_>,
        preceding: Option<&EvaluatedCandidate>,
        natural_only: bool,
    ) -> Result<EvaluatedCandidate, FitError> {
        let Some(resolver) = input.wrapping else {
            return self.evaluate_assignment(input, request, query, preceding, natural_only, None);
        };
        let incoming = box_checkpoint(&request.start, query, preceding)?;
        let assignments = resolver.choices(BoxChoice {
            start: &request.start,
            end: &request.end,
            incoming: incoming.map(|checkpoint| checkpoint.entry),
            controls: &request.controls,
            ending: request.context.end,
            authority: BoxAuthority::Selected {
                request: &request,
                preceding: incoming.and_then(|checkpoint| checkpoint.preceding),
            },
        })?;
        let mut ordinary = None;
        let mut unrejected = None;
        let mut fitting = Vec::new();
        for assignment in assignments {
            let mut request = request.clone();
            request.line_box_struts = assignment.struts();
            let trial = self.evaluate_assignment(
                input,
                request,
                query,
                preceding,
                natural_only,
                Some(assignment),
            )?;
            if ordinary.is_none() {
                ordinary = Some(trial.clone());
            }
            if !query.rejected_choices.contains(&trial.key()) {
                if unrejected.is_none() {
                    unrejected = Some(trial.clone());
                }
                if trial.fits {
                    fitting.push(trial);
                }
            }
        }
        if fitting.len() > 1
            && matches!(input.available, Available::Definite(_))
            && request.context.end == LineEnd::Soft
        {
            let mut frontier = None;
            for trial in fitting {
                let class =
                    resolver.classify(trial.boxes.as_ref().ok_or(FitError::InvalidGeometry)?)?;
                frontier
                    .get_or_insert_with(|| class.frontier())
                    .consider(class, trial)?;
            }
            return frontier
                .and_then(|frontier| frontier.into_preferred())
                .ok_or(FitError::InvalidGeometry);
        }
        fitting
            .into_iter()
            .next()
            .or(unrejected)
            .or(ordinary)
            .ok_or(FitError::InvalidGeometry)
    }

    fn evaluate_assignment(
        &mut self,
        input: &FitInput<'_>,
        mut request: LineSelectionRequest,
        query: &EndpointContext<'_>,
        preceding: Option<&EvaluatedCandidate>,
        natural_only: bool,
        boxes: Option<BoxAssignment>,
    ) -> Result<EvaluatedCandidate, FitError> {
        let original_box_struts = boxes.as_ref().map_or_else(Vec::new, BoxAssignment::struts);
        let position = query.position;
        let entry = super::roles::RoleEntry {
            current_first: query.indent_state.first_formatted(),
            query_start: query.start,
            query_incoming: query.incoming,
            boxes: boxes.as_ref(),
            accepted: query.accepted_prefix,
            preceding: preceding.map(|candidate| {
                (
                    &candidate.finalized,
                    candidate.amounts.as_slice(),
                    candidate.boxes.as_ref(),
                )
            }),
        };
        (self.reset)(EncounterReset::Candidate);
        let mut height = 0.0;
        let mut placed_start = position.logical_start;
        let mut rejected = Vec::new();
        loop {
            let bands = query_bands(&mut self.band, position.logical_start, placed_start, height)?;
            let band = bands.combined();
            request.context.available_inline = selection_width(input.available, band)?;
            request.context.alignment_insets = if request.context.available_inline.is_some() {
                input
                    .groups
                    .insets::<f64>(request.context.line_box_direction)
                    .map_err(|error| FitError::Numeric(error.value(), error.reason()))?
            } else {
                surgeist_text::LineAlignmentInsets::ZERO
            };
            request.context.tabs = tab_context(input, band, request.context.available_inline)?;
            let mut allocation_bases = Vec::new();
            let (finalized, amounts, natural_advance, role_basis) = loop {
                let (candidate, role_basis) =
                    finalize_with_roles(input, request, natural_only, &entry)?;
                request = candidate.request;
                (self.replay)(
                    &candidate.finalized,
                    position
                        .replay(placed_start, preceding)
                        .with_boxes(boxes.clone()),
                )?;
                let actual = (self.allocate)(
                    &candidate.finalized,
                    band,
                    placed_start,
                    &original_box_struts,
                )?;
                if actual.atomics == request.atomics
                    && actual.controls == request.controls
                    && actual.line_strut == request.line_strut
                    && actual.line_box_struts == request.line_box_struts
                {
                    // A nonformatted semantic receipt is not a group member.
                    // Its real allocations were validated above; finalize its
                    // actual zero-used frame before retaining source custody.
                    if request.context.alignment_insets != surgeist_text::LineAlignmentInsets::ZERO
                        && !LineDisposition::resolve(&candidate.finalized)?.is_formatted()
                    {
                        request.context.alignment_insets = surgeist_text::LineAlignmentInsets::ZERO;
                        continue;
                    }
                    break (
                        candidate.finalized,
                        candidate.amounts,
                        candidate.natural_advance,
                        role_basis,
                    );
                }
                let basis = (request.clone(), candidate.amounts);
                if allocation_bases.contains(&basis) {
                    return Err(FitError::FeedbackCycle);
                }
                allocation_bases.push(basis);
                request.atomics = actual.atomics;
                request.controls = actual.controls;
                request.line_strut = actual.line_strut;
                request.line_box_struts = actual.line_box_struts;
            };
            let encounters = (self.replay)(
                &finalized,
                position
                    .replay(placed_start, preceding)
                    .with_boxes(boxes.clone()),
            )?;
            let disposition = LineDisposition::resolve(&finalized)?;
            let actual_height = finite(disposition.height(&finalized))?;
            let actual_start = (self.canonical)(
                BlockPosition {
                    proposed: position.logical_start,
                    minimum: position.minimum_block,
                    height: actual_height,
                },
                disposition.baseline(&finalized),
            )?;
            let actual_bands = query_bands(
                &mut self.band,
                position.logical_start,
                actual_start,
                actual_height,
            )?;
            let basis = CandidateBasis {
                disposition: disposition.clone(),
                encounters: encounters.clone(),
                boxes: boxes.clone(),
                request: request.clone(),
                amounts: amounts.clone(),
                natural_advance,
                logical_start: position.logical_start,
                placed_start,
                bands,
                height,
                minimum_block: position.minimum_block,
            };
            if actual_bands != bands || actual_start != placed_start {
                if rejected.contains(&basis) {
                    return Err(FitError::FeedbackCycle);
                }
                rejected.push(basis);
                height = actual_height;
                placed_start = actual_start;
                continue;
            }
            let fit_width = match input.available {
                Available::Minimum => Some(finite((band.end - band.start).max(0.0))?),
                Available::Definite(_) | Available::Maximum => request.context.available_inline,
            };
            let measurement = match input.available {
                Available::Minimum => finalized.measurements().min_content_contribution(),
                Available::Maximum => finalized.measurements().max_content_contribution(),
                Available::Definite(_) => finalized.measurements().fit(),
            };
            let occupied = finite(measurement.occupied_end() - measurement.occupied_start())?;
            let fits = !disposition.is_formatted()
                || fit_width.is_none_or(|width| {
                    measurement.advance() <= width - request.context.indent
                        && occupied <= width - request.context.indent
                });
            let usage = finalized.measurements().formatted_line_usage();
            if usage.first() || usage.last() {
                let basis = role_basis.ok_or(FitError::InvalidGeometry)?;
                let used = RoleUse {
                    disposition: disposition.clone(),
                    request: request.clone(),
                    boxes: boxes.clone(),
                    amounts: amounts.clone(),
                    usage,
                    outgoing: finalized.hyphenation_continuation(),
                    basis,
                    bands,
                    encounters: encounters.clone(),
                    logical_start: position.logical_start,
                    source_floor: position.source_floor,
                    minimum_block: position.minimum_block,
                    deferred_from: position.deferred_from.clone(),
                    placed_start,
                };
                if let Some(observed) = &mut self.observed {
                    observed.role(&used);
                }
                if !self.roles.contains(&used) {
                    self.roles.push(used);
                }
            }
            if let Some(assignment) = &boxes {
                let used = super::boxes::BoxTrialUse {
                    disposition: disposition.clone(),
                    assignment: assignment.clone(),
                    request: request.clone(),
                    amounts: amounts.clone(),
                    bands,
                    encounters: encounters.clone(),
                    placed_start,
                };
                if let Some(observed) = &mut self.observed {
                    observed.box_use(&used);
                }
                if !self.box_uses.contains(&used) {
                    self.box_uses.push(used);
                }
            }
            return Ok(EvaluatedCandidate {
                boxes,
                request,
                finalized,
                amounts,
                natural_advance,
                encounters,
                bands,
                placed_start,
                fits,
                disposition,
            });
        }
    }

    fn choose_endpoint(
        &mut self,
        input: &FitInput<'_>,
        query: &EndpointContext<'_>,
        variants: impl Iterator<Item = LineEndpoint>,
        zone_basis: &mut ZoneBasis,
    ) -> Result<EndpointChoice, FitError> {
        let mut fitting_none = false;
        let mut endpoint_choice: Option<EvaluatedCandidate> = None;
        let mut dispositions = Vec::new();
        let mut ordinary_formatted = false;
        for endpoint in variants {
            let mut zone_suppressed = false;
            let alternative = endpoint.alternative.clone();
            let automatic = alternative.as_ref().and_then(|alternative| {
                match alternative.view().definition() {
                    surgeist_text::PreparedAlternativeDefinition::Automatic { cut } => Some(cut),
                    _ => None,
                }
            });
            if automatic.is_some() && matches!(input.available, Available::Maximum) {
                continue;
            }
            if let Some(alternative) = alternative.as_ref().filter(|_| automatic.is_some())
                && let Some(limit) = query
                    .hyphenation_state
                    .suppressed_limit(matches!(input.available, Available::Minimum))
            {
                dispositions.push(ConditionalDisposition::AutoLineLimit {
                    alternative: alternative.clone(),
                    limit,
                });
                continue;
            }
            if let Some(cut) = &automatic
                && matches!(input.available, Available::Definite(_))
                && let Some(word) = cut.view().word()
            {
                let mut fallback: Option<EvaluatedCandidate> = None;
                for offer in word.view().ordinary_fallbacks(query.start)? {
                    let request = selection_request(
                        input,
                        query.start,
                        query.incoming,
                        LineEndpoint {
                            boundary: offer.view().boundary(),
                            termination: PreparedTermination::Ordinary(offer),
                            alternative: None,
                            ending: LineEnd::Soft,
                        },
                        query.indent_state,
                        query.line_box_context,
                    )?;
                    let trial = self.evaluate_mode(input, request, query, None, true)?;
                    if !trial.disposition.is_formatted() {
                        continue;
                    }
                    if fallback.is_none() || trial.fits {
                        fallback = Some(trial);
                    }
                }
                if let Some(fallback) = fallback {
                    let span = fallback
                        .request
                        .context
                        .available_inline
                        .ok_or(FitError::MissingZoneBasis)?;
                    let zone = used_zone(input.hyphenate_limit_zone, span)?;
                    let end = super::run::add(
                        fallback.request.context.indent,
                        fallback.finalized.measurements().alignment().advance(),
                    )?;
                    let gap = super::run::add(span, -end)?.max(0.0);
                    zone_basis.references.push(ZoneReference {
                        alternative: alternative.clone().expect("automatic variant"),
                        request: fallback.request.clone(),
                        zone,
                        measure: fallback.finalized.measurements().alignment(),
                        bands: fallback.bands,
                        encounters: fallback.encounters.clone(),
                        placed_start: fallback.placed_start,
                    });
                    if gap <= zone {
                        // The genuine meaningful fallback's earlier None variant
                        // is already the ordinary ceiling. It cannot be discharged
                        // as phantom or by an automatic-only disposition.
                        if !query.conditional {
                            continue;
                        }
                        zone_suppressed = true;
                    }
                }
            }
            let request = selection_request(
                input,
                query.start,
                query.incoming,
                endpoint,
                query.indent_state,
                query.line_box_context,
            )?;
            let candidate = self.evaluate(input, request, query, None)?;
            if zone_suppressed {
                dispositions.push(ConditionalDisposition::ConsumerHyphenationPolicy(
                    candidate.finalized,
                ));
                continue;
            }
            fitting_none |= candidate.disposition.is_formatted()
                && candidate.fits
                && candidate.request.alternative.is_none();
            if query.rejected_choices.contains(&candidate.key()) {
                if candidate.request.alternative.is_some() {
                    dispositions.push(ConditionalDisposition::ConsumerHyphenationPolicy(
                        candidate.finalized,
                    ));
                } else {
                    // An unselected meaningful None is still an ordinary
                    // barrier; rejection does not turn it into phantom.
                    ordinary_formatted |= candidate.disposition.is_formatted();
                }
                continue;
            }
            if let Some(cut) = &automatic
                && matches!(input.available, Available::Definite(_))
                && let Some(word) = cut.view().word()
            {
                let mut priority = None;
                let portions = match &candidate.request.termination {
                    PreparedTermination::Conditional(opportunity) => {
                        opportunity.manual_portions()?
                    }
                    _ => word.view().portions()?,
                };
                for portion in portions {
                    let boundary = cut.view().boundary().view().participant_offset();
                    if !portion.has_manual_partition()
                        || boundary <= portion.start().view().participant_offset()
                        || boundary > portion.end().view().participant_offset()
                    {
                        continue;
                    }
                    let request = selection_request(
                        input,
                        &portion.start(),
                        portion.incoming().as_ref(),
                        LineEndpoint {
                            boundary: portion.end(),
                            termination: PreparedTermination::Measurement(portion.clone()),
                            alternative: portion.alternative(),
                            ending: LineEnd::Soft,
                        },
                        query.indent_state,
                        query.line_box_context,
                    )?;
                    let trial = self.evaluate(input, request, query, Some(&candidate))?;
                    if trial.fits {
                        priority = Some(trial.finalized);
                        break;
                    }
                }
                if let Some(portion) = priority {
                    dispositions.push(ConditionalDisposition::AutoManualPriority {
                        alternative: alternative.clone().expect("automatic alternative"),
                        portion,
                    });
                    continue;
                }
            }
            if !candidate.disposition.is_formatted()
                && candidate.request.context.end == LineEnd::Soft
            {
                dispositions.push(ConditionalDisposition::Phantom(candidate.finalized));
                continue;
            }
            if let Some(alternative) = alternative.filter(|_| automatic.is_some()) {
                if matches!(input.available, Available::Minimum) {
                    dispositions.push(ConditionalDisposition::AutoIntrinsicRefinement(alternative));
                } else {
                    dispositions.push(ConditionalDisposition::AutoFitExhausted(
                        candidate.finalized.clone(),
                    ));
                }
            } else if query.conditional {
                dispositions.push(ConditionalDisposition::Exhausted(
                    candidate.finalized.clone(),
                ));
            } else if candidate.disposition.is_formatted() {
                ordinary_formatted = true;
            }
            let fits = candidate.fits;
            if endpoint_choice.is_none() || fits {
                endpoint_choice = Some(candidate);
            }
            if fits && !matches!(input.available, Available::Minimum) {
                break;
            }
        }
        Ok(EndpointChoice {
            candidate: endpoint_choice,
            dispositions,
            formatted: ordinary_formatted,
            fitting_none,
        })
    }

    fn measure_phrase(
        &mut self,
        input: &FitInput<'_>,
        query: &EndpointContext<'_>,
        ordinary: &EvaluatedCandidate,
        portion: PreparedPhrasePortion,
    ) -> Result<(PhraseOverflowDecision, PhraseBasis), FitError> {
        // A meaningful earlier None is the query ceiling, and a fitting real
        // prefix wins before search. Earlier portion prefixes are therefore
        // phantom; do not invent an accepted prefix line or consume its indent.
        let empty_position = query.position.clone();
        let empty_indent = query.indent_state;
        let preceding = (portion.start() != *query.start).then_some(ordinary);
        let incoming = portion.incoming();
        let request = selection_request(
            input,
            &portion.start(),
            incoming.as_ref(),
            LineEndpoint {
                boundary: portion.end(),
                termination: PreparedTermination::PhraseMeasurement(portion),
                alternative: None,
                ending: LineEnd::Soft,
            },
            empty_indent,
            query.line_box_context,
        )?;
        let phrase_query = EndpointContext {
            position: &empty_position,
            indent_state: empty_indent,
            ..*query
        };
        let trial = self.evaluate(input, request, &phrase_query, preceding)?;
        let basis = PhraseBasis {
            ordinary: ordinary.basis(query.position),
            position: empty_position.clone(),
            indent_state: empty_indent,
            trial: trial.basis(&empty_position),
            fits: trial.fits,
        };
        let decision = if trial.fits {
            PhraseOverflowDecision::Fits(trial.finalized)
        } else {
            PhraseOverflowDecision::Overflows(trial.finalized)
        };
        Ok((decision, basis))
    }

    /// A later rejected candidate may own the provisional ledger. Restore and
    /// verify the chosen candidate's actual complete receipt before accepting it.
    fn restore(
        &mut self,
        candidate: &EvaluatedCandidate,
        position: &CandidatePosition,
    ) -> Result<(), FitError> {
        (self.reset)(EncounterReset::Candidate);
        let encounters = (self.replay)(
            &candidate.finalized,
            position
                .replay(candidate.placed_start, None)
                .with_boxes(candidate.boxes.clone()),
        )?;
        let bands = query_bands(
            &mut self.band,
            position.logical_start,
            candidate.placed_start,
            candidate.disposition.height(&candidate.finalized),
        )?;
        let actual = (self.allocate)(
            &candidate.finalized,
            candidate.bands.combined(),
            candidate.placed_start,
            &candidate
                .boxes
                .as_ref()
                .map_or_else(Vec::new, BoxAssignment::struts),
        )?;
        if encounters != candidate.encounters
            || bands != candidate.bands
            || actual.atomics != candidate.request.atomics
            || actual.controls != candidate.request.controls
            || actual.line_strut != candidate.request.line_strut
            || actual.line_box_struts != candidate.request.line_box_struts
        {
            return Err(FitError::FeedbackCycle);
        }
        Ok(())
    }
}

impl CandidatePosition {
    fn replay(&self, placed: f64, preceding: Option<&EvaluatedCandidate>) -> EncounterReplay {
        EncounterReplay {
            boxes: None,
            preceding: preceding.map(|candidate| PrecedingEncounter {
                boxes: candidate.boxes.clone(),
                line: candidate.finalized.clone(),
                placed: candidate.placed_start,
            }),
            source_floor: self.source_floor,
            logical: self.logical_start,
            placed,
            defer_preceding: self.deferred_from.is_some(),
        }
    }
}

fn selection_width(available: Available, band: Band) -> Result<Option<f64>, FitError> {
    match available {
        Available::Definite(value) => {
            Ok(Some(finite((band.end - band.start).max(0.0))?.min(value)))
        }
        Available::Minimum | Available::Maximum => Ok(None),
    }
}

/// Intrinsic measurement selects its logical grid. Used layout resolves the
/// captured block edge relative to this actual allocation's paragraph start.
fn tab_context(
    input: &FitInput<'_>,
    band: Band,
    allocation: Option<f64>,
) -> Result<TabContext, FitError> {
    if input.prepared.preparation_options().tabs() == TabPreparation::FontAdvance {
        return Ok(TabContext::FontAdvance);
    }
    if matches!(input.available, Available::Minimum | Available::Maximum) {
        return Ok(TabContext::IntrinsicCssStops);
    }
    let displacement = if input.base_direction == input.block_direction {
        Some(band.start)
    } else {
        allocation
            .map(|width| super::run::add(band.start, width))
            .transpose()?
    };
    let origin = displacement.map(|value| match input.block_direction {
        ParagraphBaseDirection::LeftToRight => -value,
        ParagraphBaseDirection::RightToLeft => value,
    });
    Ok(TabContext::CssStops { origin })
}

/// The three operations share one accepted encounter checkpoint.
pub(crate) struct EncounterCallbacks<Replay, Accept, Reset, Checkpoint> {
    pub replay: Replay,
    pub accept: Accept,
    pub reset: Reset,
    pub checkpoint_group: Checkpoint,
}

struct LineEndpoint {
    boundary: PreparedBoundary,
    termination: PreparedTermination,
    alternative: Option<PreparedAlternative>,
    ending: LineEnd,
}

fn selection_request(
    input: &FitInput<'_>,
    start: &PreparedBoundary,
    incoming: Option<&HyphenationContinuation>,
    endpoint: LineEndpoint,
    indent_state: crate::text_indent::IndentLineState,
    line_box_context: crate::line_box::LineBoxContext,
) -> Result<LineSelectionRequest, FitError> {
    let facts = input
        .prepared
        .line_direction_facts(start, &endpoint.boundary)?;
    let strong = if facts.has_eligible_strong() {
        facts.paragraph_strong_direction()
    } else {
        None
    };
    let line_box_direction = line_box_context.resolve(strong);
    let from = start.view().participant_offset();
    let to = endpoint.boundary.view().participant_offset();
    let participants = input
        .prepared
        .participants()
        .skip(from)
        .take(to - from)
        .collect::<Vec<_>>();
    let atomics = input
        .atomics
        .iter()
        .filter(|allocation| {
            participants.iter().any(|participant| {
                matches!(participant,
        PreparedParticipant::Atomic(occurrence) if occurrence == &allocation.occurrence)
            })
        })
        .cloned()
        .collect();
    let controls = input
        .controls
        .iter()
        .filter(|allocation| {
            participants.iter().any(|participant| {
                matches!(participant,
        PreparedParticipant::Control(occurrence) if occurrence == &allocation.occurrence)
            })
        })
        .cloned()
        .collect();
    Ok(LineSelectionRequest {
        start: start.clone(),
        end: endpoint.boundary,
        participants,
        termination: endpoint.termination,
        alternative: endpoint.alternative,
        incoming: incoming.cloned(),
        context: LineContext {
            alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
            formatted_line: surgeist_text::FormattedLineContext::default(),
            line_box_direction,
            base_direction: input.base_direction,
            orientation: input.orientation,
            start: if from == 0 {
                LineStart::Paragraph
            } else {
                LineStart::Continuation
            },
            end: endpoint.ending,
            available_inline: None,
            indent: input.indent.at(indent_state),
            alignment: text_alignment(input.alignment),
            tabs: TabContext::FontAdvance,

            spacing: if input.justification == TextJustification::Spaces {
                SpacingPolicy::Spaces
            } else {
                SpacingPolicy::None
            },
        },
        atomics,
        controls,
        boundary_context: Vec::new(),
        line_strut: input.line_strut,
        line_box_struts: Vec::new(),
    })
}

/// Fit only preparation-issued opportunities. Every measured candidate executes
/// its own retained finalizer and complete convergence, not preliminary widths.
pub(crate) fn fit(
    input: FitInput<'_>,
    band_source: impl FnMut(f64, f64) -> Result<Band, FitError>,
    canonical_block: impl FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    allocate: impl FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    clear_source: impl FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
    encounters: EncounterCallbacks<
        impl FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
        impl FnMut() -> Result<(), FitError>,
        impl FnMut(EncounterReset),
        impl FnMut(),
    >,
) -> Result<Vec<FittedLine>, FitError> {
    let EncounterCallbacks {
        replay: replay_encounters,
        accept: accept_encounters,
        reset: reset_candidate,
        checkpoint_group,
    } = encounters;
    let boundaries: Vec<_> = input
        .prepared
        .boundary_views()
        .map(|view| (view.participant_offset(), view.handle()))
        .collect();
    let opportunities: Vec<_> = input
        .prepared
        .opportunity_views()
        .map(|view| {
            let boundary = view.boundary();
            let end = boundary.view().participant_offset();
            (
                end,
                view.handle(),
                view.kind(),
                view.alternatives()
                    .map(|view| view.handle())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let environment = CandidateEnvironment {
        roles: Vec::new(),
        box_uses: Vec::new(),
        observed: None,
        band: band_source,
        canonical: canonical_block,
        allocate,
        replay: replay_encounters,
        reset: reset_candidate,
    };
    let entry = SequenceEntry {
        boundary: boundaries
            .first()
            .ok_or(FitError::InvalidGeometry)?
            .1
            .clone(),
        logical_start: 0.0,
        minimum_block: None,
        indent_state: input.indent_state,
        line_box_context: input.line_box_context,
        hyphenation_state: input.hyphenation_state,
        incoming: None,
        prefix_len: 0,
    };
    let mut runner = SequenceRunner {
        input,
        environment,
        opportunities,
        accept: accept_encounters,
        clear: clear_source,
    };
    match runner.input.wrap_style {
        crate::TextWrapStyle::Balance
            if matches!(runner.input.available, Available::Definite(_)) =>
        {
            return balance::fit(&mut runner, entry, checkpoint_group);
        }
        crate::TextWrapStyle::AvoidShortLastLine
            if matches!(runner.input.available, Available::Definite(_)) =>
        {
            return tail::fit(&mut runner, entry, checkpoint_group);
        }
        crate::TextWrapStyle::Pretty
            if matches!(runner.input.available, Available::Definite(_)) =>
        {
            return runner.pretty(entry, checkpoint_group);
        }
        crate::TextWrapStyle::Auto
        | crate::TextWrapStyle::Stable
        | crate::TextWrapStyle::Balance
        | crate::TextWrapStyle::AvoidShortLastLine
        | crate::TextWrapStyle::Pretty => {}
    }
    let mut result = Vec::new();
    let scope = SequenceScope {
        reset: EncounterReset::Sequence,
        stop_at_forced: false,
        max_formatted: None,
        goal: None,
        goal_after: 0,
        goal_before: None,
        opportunity_start: 0,
        opportunity_end: runner.opportunities.len(),
    };
    match runner.run(&entry, scope, &mut result)? {
        SequenceOutcome::Complete(_) => Ok(result),
        SequenceOutcome::Inadmissible(_) => Err(FitError::FeedbackCycle),
    }
}

type Opportunity = (
    usize,
    surgeist_text::PreparedOpportunity,
    PreparedBreakKind,
    Vec<PreparedAlternative>,
);

#[derive(Clone, Debug, PartialEq)]
struct SequenceEntry {
    boundary: PreparedBoundary,
    logical_start: f64,
    minimum_block: Option<f64>,
    indent_state: crate::text_indent::IndentLineState,
    line_box_context: crate::line_box::LineBoxContext,
    hyphenation_state: crate::hyphenation::LineHyphenationState,
    incoming: Option<HyphenationContinuation>,
    prefix_len: usize,
}

#[derive(Clone, Copy)]
struct SequenceScope {
    reset: EncounterReset,
    stop_at_forced: bool,
    max_formatted: Option<usize>,
    goal: Option<LineQualityGoal>,
    goal_after: usize,
    goal_before: Option<usize>,
    opportunity_start: usize,
    opportunity_end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopeExhaustion {
    NoAdmissibleCandidate,
    TerminalConstraint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SequenceRejection {
    CountExceeded,
    ScopeExhausted(ScopeExhaustion),
}

enum SequenceOutcome {
    Complete(SequenceEntry),
    Inadmissible(SequenceRejection),
}

struct SequenceRunner<'a, B, C, A, R, X, Accept, Clear> {
    input: FitInput<'a>,
    environment: CandidateEnvironment<B, C, A, R, X>,
    opportunities: Vec<Opportunity>,
    accept: Accept,
    clear: Clear,
}

impl<B, C, A, R, X, Accept, Clear> SequenceRunner<'_, B, C, A, R, X, Accept, Clear>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset),
    Accept: FnMut() -> Result<(), FitError>,
    Clear: FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
{
    fn run(
        &mut self,
        entry: &SequenceEntry,
        scope: SequenceScope,
        result: &mut Vec<FittedLine>,
    ) -> Result<SequenceOutcome, FitError> {
        #[cfg(test)]
        balance::observe_work(scope, false);
        let input = &self.input;
        let environment = &mut self.environment;
        let opportunities = &self.opportunities[scope.opportunity_start..scope.opportunity_end];
        let accept_encounters = &mut self.accept;
        let clear_source = &mut self.clear;
        // Each frame records only choices already visited under its exact accepted
        // prefix. Backtracking discards all descendant frames; no abandoned ban is
        // transferred to a different line sequence. Every remaining actual variant
        // is still evaluated in the ordinary chooser's preference order.
        let constrained = input.tail == TailContext::CompleteBlock
            && matches!(input.available, Available::Definite(_));
        let wrapping = input
            .wrapping
            .filter(|_| matches!(input.available, Available::Definite(_)));
        let mut visited: Vec<Vec<CandidateKey>> = Vec::new();
        let mut terminal_constraint_rejected = false;
        'sequence: loop {
            environment.roles.clear();
            environment.box_uses.clear();
            (environment.reset)(scope.reset);
            result.truncate(entry.prefix_len);
            let mut start = entry.boundary.view().participant_offset();
            let mut logical_start = entry.logical_start;
            let mut minimum_block = entry.minimum_block;
            let mut indent_state = entry.indent_state;
            let mut line_box_context = entry.line_box_context;
            let mut hyphenation_state = entry.hyphenation_state;
            // A rebuilt sequence owns fresh continuation custody. Requests below
            // retain it in reuse, convergence and visited-choice identity.
            let mut incoming = entry.incoming.clone();
            let mut completed_end = false;
            let mut current_boundary = entry.boundary.clone();
            let mut formatted_count = 0;
            while !completed_end {
                let goal = scope.goal.filter(|_| {
                    formatted_count >= scope.goal_after
                        && scope.goal_before.is_none_or(|end| formatted_count < end)
                });
                let start_boundary = current_boundary.clone();
                let rejected_choices: &[CandidateKey] = if constrained {
                    if visited.len() <= result.len() - entry.prefix_len {
                        visited.push(Vec::new());
                    }
                    &visited[result.len() - entry.prefix_len]
                } else {
                    &[]
                };
                let mut position = CandidatePosition {
                    source_floor: logical_start,
                    logical_start,
                    minimum_block,
                    deferred_from: None,
                };
                let mut transitions = Vec::new();
                let role_start = environment.roles.len();
                let box_use_start = environment.box_uses.len();
                loop {
                    let mut phrase_basis = Vec::new();
                    let mut choice: Option<EvaluatedCandidate>;
                    let mut fitting_none;
                    let mut first_ceiling;
                    let mut skipped_ordinary = Vec::new();
                    let mut zone_basis = ZoneBasis {
                        policy: input.hyphenate_limit_zone,
                        references: Vec::new(),
                    };
                    let mut comparison_basis: Option<ComparisonBasis> = None;
                    loop {
                        choice = None;
                        fitting_none = false;
                        first_ceiling = None;
                        skipped_ordinary.clear();
                        zone_basis.references.clear();
                        let mut ends = FittingEnds::default();
                        let mut terminal = None;
                        let mut scoped = ScopedChoices {
                            goal,
                            ..ScopedChoices::default()
                        };
                        for (end, opportunity, kind, alternatives) in opportunities {
                            #[cfg(test)]
                            balance::observe_work(scope, true);
                            if *end < start
                                || (*end == start && *kind != PreparedBreakKind::ParagraphEnd)
                            {
                                continue;
                            }
                            let variants = (*kind != PreparedBreakKind::Discretionary)
                                .then_some(None)
                                .into_iter()
                                .chain(alternatives.iter().cloned().map(Some));
                            let query = EndpointContext {
                                accepted_prefix: result,
                                line_box_context,
                                start: &start_boundary,
                                incoming: incoming.as_ref(),
                                position: &position,
                                indent_state,
                                hyphenation_state,
                                rejected_choices,
                                conditional: false,
                            };
                            let endpoint = environment.choose_endpoint(
                                input,
                                &query,
                                variants.map(|alternative| LineEndpoint {
                                    boundary: opportunity.view().boundary(),
                                    termination: PreparedTermination::Ordinary(opportunity.clone()),
                                    alternative,
                                    ending: match kind {
                                        PreparedBreakKind::Allowed
                                        | PreparedBreakKind::Discretionary => LineEnd::Soft,
                                        PreparedBreakKind::Forced => LineEnd::Forced,
                                        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                                    },
                                }),
                                &mut zone_basis,
                            )?;
                            fitting_none |= endpoint.fitting_none;
                            let EndpointChoice {
                                candidate: endpoint_choice,
                                dispositions,
                                formatted: ordinary_formatted,
                                ..
                            } = endpoint;
                            if wrapping.is_some()
                                && !matches!(
                                    kind,
                                    PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                                )
                            {
                                terminal = Some((
                                    opportunity.view().boundary(),
                                    endpoint_choice
                                        .as_ref()
                                        .map(|candidate| candidate.observed(&position)),
                                ));
                            }
                            if let Some(candidate) = endpoint_choice {
                                if wrapping.is_some() {
                                    ends.observe(&candidate, goal.is_some());
                                }
                                if comparison_basis.is_some()
                                    && candidate.fits
                                    && candidate.disposition.is_formatted()
                                    && (goal.is_some()
                                        || candidate.request.context.end == LineEnd::Soft)
                                {
                                    scoped.consider(
                                        wrapping.expect("comparison has a scope loan"),
                                        candidate.clone(),
                                    );
                                }
                                if (choice.is_none()
                                    || (!matches!(input.available, Available::Minimum)
                                        && (candidate.fits
                                            || matches!(input.available, Available::Maximum))))
                                    && prefer_candidate(goal, &candidate, choice.as_ref())?
                                {
                                    choice = Some(candidate);
                                }
                            }
                            if first_ceiling.is_none() {
                                if ordinary_formatted {
                                    first_ceiling = Some(opportunity.clone());
                                } else {
                                    skipped_ordinary.extend(dispositions);
                                }
                            }
                            if matches!(input.available, Available::Minimum) && ordinary_formatted {
                                break;
                            }
                            if !matches!(
                                kind,
                                PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                            ) {
                                break;
                            }
                        }
                        if wrapping.is_some() {
                            let observed = ComparisonBasis {
                                position: position.clone(),
                                normal: choice
                                    .as_ref()
                                    .map(|candidate| candidate.observed(&position)),
                                terminal,
                                progress: None,
                                ceiling: first_ceiling.as_ref().map(
                                    |opportunity: &surgeist_text::PreparedOpportunity| {
                                        opportunity.view().boundary()
                                    },
                                ),
                                fitting_none,
                                ends,
                            };
                            if let Some(prior) = &comparison_basis {
                                if prior != &observed {
                                    return Err(FitError::FeedbackCycle);
                                }
                                if let Some(preferred) = scoped.preferred()? {
                                    choice = Some(preferred);
                                }
                            } else if observed.ends.multiple
                                && !choice.as_ref().is_some_and(|candidate| {
                                    candidate.fits && candidate.request.context.end != LineEnd::Soft
                                })
                            {
                                comparison_basis = Some(observed);
                                continue;
                            }
                        }
                        break;
                    }
                    // A rejected fitting nonhyphenating terminal branch does not
                    // authorize overflow/emergency in an otherwise fitting word.
                    let choice =
                        choice.filter(|candidate| !constrained || candidate.fits || !fitting_none);
                    let Some(mut candidate) = choice else {
                        if constrained
                            && result.len() > entry.prefix_len
                            && let Some(parent) = result.pop()
                        {
                            let depth = result.len() - entry.prefix_len;
                            visited.truncate(depth + 1);
                            visited[depth].push(parent.key());
                            continue 'sequence;
                        }
                        return if scope.stop_at_forced {
                            Ok(SequenceOutcome::Inadmissible(
                                SequenceRejection::ScopeExhausted(
                                    if terminal_constraint_rejected {
                                        ScopeExhaustion::TerminalConstraint
                                    } else {
                                        ScopeExhaustion::NoAdmissibleCandidate
                                    },
                                ),
                            ))
                        } else {
                            Err(FitError::FeedbackCycle)
                        };
                    };
                    let purpose = match input.available {
                        Available::Minimum => Some(ConditionalPurpose::MinContent),
                        Available::Definite(_) if !candidate.fits => {
                            Some(ConditionalPurpose::Overflow)
                        }
                        Available::Definite(_) | Available::Maximum => None,
                    };
                    let mut minimal_progress = None;
                    if let Some(purpose) = purpose
                        && let Some(ceiling) = first_ceiling
                        && ceiling.view().boundary().view().participant_offset() > start
                    {
                        let search = input.prepared.conditional_search(
                            &start_boundary,
                            incoming.as_ref(),
                            &ceiling,
                            purpose,
                            &skipped_ordinary,
                            (purpose == ConditionalPurpose::Overflow)
                                .then_some(&candidate.finalized),
                        )?;
                        let query = EndpointContext {
                            accepted_prefix: result,
                            start: &start_boundary,
                            line_box_context,
                            incoming: incoming.as_ref(),
                            position: &position,
                            indent_state,
                            hyphenation_state,
                            rejected_choices,
                            conditional: true,
                        };
                        let mut decisions = Vec::new();
                        for portion in search.phrase_portions() {
                            let (decision, basis) =
                                environment.measure_phrase(input, &query, &candidate, portion)?;
                            decisions.push(decision);
                            phrase_basis.push(basis);
                        }
                        let mut batch = search.begin(&decisions)?;
                        let mut restored_fallback = false;
                        loop {
                            let stage = batch.stage();
                            let mut dispositions = Vec::new();
                            let mut stage_choice: Option<EvaluatedCandidate>;
                            let mut blocked_none;
                            let mut stage_progress;
                            let progress_already_present = minimal_progress.is_some();
                            let zone_start = zone_basis.references.len();
                            let mut comparison_basis: Option<ComparisonBasis> = None;
                            loop {
                                dispositions.clear();
                                zone_basis.references.truncate(zone_start);
                                stage_choice = None;
                                blocked_none = false;
                                stage_progress = None;
                                let mut ends = FittingEnds::default();
                                let mut scoped = ScopedChoices {
                                    goal,
                                    ..ScopedChoices::default()
                                };
                                let mut from = 0;
                                while from < batch.opportunities().len() {
                                    let boundary = batch.opportunities()[from].boundary();
                                    let mut to = from + 1;
                                    while to < batch.opportunities().len()
                                        && batch.opportunities()[to].boundary() == boundary
                                    {
                                        to += 1;
                                    }
                                    let endpoint = environment.choose_endpoint(
                                        input,
                                        &query,
                                        batch.opportunities()[from..to].iter().cloned().map(
                                            |opportunity| {
                                                let alternative = opportunity.alternative();
                                                LineEndpoint {
                                                    boundary: opportunity.boundary(),
                                                    termination: PreparedTermination::Conditional(
                                                        opportunity,
                                                    ),
                                                    alternative,
                                                    ending: LineEnd::Soft,
                                                }
                                            },
                                        ),
                                        &mut zone_basis,
                                    )?;
                                    blocked_none |=
                                        endpoint.fitting_none && endpoint.candidate.is_none();
                                    dispositions.extend(endpoint.dispositions);
                                    if let Some(trial) = endpoint.candidate {
                                        if wrapping.is_some() {
                                            ends.observe(&trial, goal.is_some());
                                        }
                                        if comparison_basis.is_some()
                                            && trial.fits
                                            && trial.disposition.is_formatted()
                                        {
                                            scoped.consider(
                                                wrapping.expect("comparison has a scope loan"),
                                                trial.clone(),
                                            );
                                        }
                                        if purpose == ConditionalPurpose::MinContent {
                                            stage_choice = Some(trial);
                                            break;
                                        }
                                        if stage == ConditionalStage::Arbitrary
                                            && !trial.fits
                                            && !progress_already_present
                                            && stage_progress.is_none()
                                        {
                                            stage_progress = Some(trial);
                                        } else if (stage_choice.is_none() || trial.fits)
                                            && prefer_candidate(
                                                goal,
                                                &trial,
                                                stage_choice.as_ref(),
                                            )?
                                        {
                                            stage_choice = Some(trial);
                                        }
                                    }
                                    from = to;
                                }
                                if wrapping.is_some() {
                                    let observed = ComparisonBasis {
                                        position: position.clone(),
                                        normal: stage_choice
                                            .as_ref()
                                            .map(|candidate| candidate.observed(&position)),
                                        terminal: None,
                                        progress: stage_progress
                                            .as_ref()
                                            .map(|candidate| candidate.observed(&position)),
                                        ceiling: None,
                                        fitting_none: blocked_none,
                                        ends,
                                    };
                                    if let Some(prior) = &comparison_basis {
                                        if prior != &observed {
                                            return Err(FitError::FeedbackCycle);
                                        }
                                        if let Some(preferred) = scoped.preferred()? {
                                            stage_choice = Some(preferred);
                                        }
                                    } else if observed.ends.multiple {
                                        comparison_basis = Some(observed);
                                        continue;
                                    }
                                }
                                break;
                            }
                            if !progress_already_present {
                                minimal_progress = stage_progress;
                            }
                            if blocked_none
                                && stage_choice.as_ref().is_none_or(|choice| !choice.fits)
                            {
                                if constrained
                                    && result.len() > entry.prefix_len
                                    && let Some(parent) = result.pop()
                                {
                                    let depth = result.len() - entry.prefix_len;
                                    visited.truncate(depth + 1);
                                    visited[depth].push(parent.key());
                                    continue 'sequence;
                                }
                                return if scope.stop_at_forced {
                                    Ok(SequenceOutcome::Inadmissible(
                                        SequenceRejection::ScopeExhausted(
                                            if terminal_constraint_rejected {
                                                ScopeExhaustion::TerminalConstraint
                                            } else {
                                                ScopeExhaustion::NoAdmissibleCandidate
                                            },
                                        ),
                                    ))
                                } else {
                                    Err(FitError::FeedbackCycle)
                                };
                            }
                            if let Some(choice) = stage_choice {
                                if choice.fits || purpose == ConditionalPurpose::MinContent {
                                    candidate = choice;
                                    break;
                                }
                                if stage != ConditionalStage::Arbitrary && !restored_fallback {
                                    candidate = choice;
                                    restored_fallback = true;
                                }
                            }
                            let Some(next) = batch.advance(&dispositions)? else {
                                break;
                            };
                            batch = next;
                        }
                    }
                    environment.restore(&candidate, &position)?;
                    let basis = (
                        candidate.request.clone(),
                        candidate.boxes.clone(),
                        candidate.amounts.clone(),
                        candidate.encounters.clone(),
                        candidate.bands,
                        candidate.placed_start,
                        position.logical_start,
                        position.minimum_block,
                        position.deferred_from.clone(),
                        zone_basis.clone(),
                    );
                    if transitions.contains(&basis) {
                        return Err(FitError::FeedbackCycle);
                    }
                    transitions.push(basis);
                    if !candidate.fits
                        && position.deferred_from.is_none()
                        && candidate.encounters.iter().any(|event| event.preceding)
                    {
                        let deferred = (environment.replay)(
                            &candidate.finalized,
                            EncounterReplay {
                                preceding: None,
                                boxes: candidate.boxes.clone(),
                                defer_preceding: true,
                                ..position.replay(candidate.placed_start, None)
                            },
                        )?;
                        if deferred != candidate.encounters {
                            position.deferred_from = Some(candidate.request.clone());
                            continue;
                        }
                    }
                    if !candidate.fits
                        && let Some(progress) = minimal_progress
                    {
                        candidate = progress;
                        environment.restore(&candidate, &position)?;
                    }
                    if !candidate.fits
                        && let Some(next) = candidate
                            .bands
                            .next_transition(position.logical_start, candidate.placed_start)
                    {
                        position.logical_start = position.logical_start.max(next);
                        position.minimum_block = Some(
                            position
                                .minimum_block
                                .map_or(next, |minimum| minimum.max(next)),
                        );
                        continue;
                    }
                    // Deferred replay above may have been a no-op. Restore the exact
                    // candidate once more before any clear/accept operation observes it.
                    environment.restore(&candidate, &position)?;
                    let logical_end = finite(
                        position.logical_start + candidate.disposition.height(&candidate.finalized),
                    )?;
                    let cleared = if candidate.disposition.is_formatted() {
                        clear_source(logical_end, &candidate.request)?
                            .map(finite)
                            .transpose()?
                            .map_or(logical_end, |minimum| minimum.max(logical_end))
                    } else {
                        logical_end
                    };
                    minimum_block = position.minimum_block;
                    if cleared > logical_end {
                        minimum_block =
                            Some(minimum_block.map_or(cleared, |minimum| minimum.max(cleared)));
                    }
                    let ending = candidate.request.context.end;
                    let end = candidate.request.end.view().participant_offset();
                    current_boundary = candidate.request.end.clone();
                    let first_formatted = indent_state.first_formatted();
                    if candidate.disposition.is_formatted() {
                        indent_state.commit(ending == LineEnd::Forced);
                        line_box_context.commit(candidate.request.context.line_box_direction);
                    }
                    accept_encounters()?;
                    if matches!(input.available, Available::Definite(_))
                        && candidate.disposition.is_formatted()
                    {
                        hyphenation_state.commit(candidate.request.alternative.is_some());
                    }
                    // Trials and hypothetical manual portions do not mutate this
                    // state; only the accepted outgoing ending replaces it.
                    incoming = candidate.finalized.hyphenation_continuation();
                    let available = candidate.finalized.metric_bounds().size.width;
                    result.push(FittedLine {
                        disposition: candidate.disposition,
                        boxes: candidate.boxes,
                        box_uses: environment.box_uses.split_off(box_use_start),
                        first_formatted,
                        role_uses: environment.roles.split_off(role_start),
                        line_box_end: line_box_context,
                        zone_basis,
                        phrase_basis,
                        hyphenation_end: hyphenation_state,
                        encounters: candidate.encounters,
                        finalized: candidate.finalized,
                        amounts: candidate.amounts,
                        natural_advance: candidate.natural_advance,
                        program: None,
                        inline_start: candidate.bands.combined().start,
                        placed_start: candidate.placed_start,
                        available,
                        bands: candidate.bands,
                        flow: LineFlow {
                            start: position.logical_start,
                            end: logical_end,
                            next: cleared,
                        },
                    });
                    logical_start = cleared;
                    start = end;
                    completed_end = ending == LineEnd::Paragraph;
                    if result
                        .last()
                        .is_some_and(|line| line.disposition.is_formatted())
                    {
                        formatted_count += 1;
                    }
                    if scope.max_formatted.is_some_and(|max| formatted_count > max) {
                        return Ok(SequenceOutcome::Inadmissible(
                            SequenceRejection::CountExceeded,
                        ));
                    }
                    break;
                }
                if scope.stop_at_forced
                    && result.last().is_some_and(|line| {
                        line.finalized.selected_line().request().context.end == LineEnd::Forced
                    })
                {
                    break;
                }
            }
            if completed_end
                && constrained
                && let Some(last) = result
                    .iter()
                    .rposition(|line| line.finalized.has_in_flow_content())
                && last > 0
                && result[last - 1]
                    .finalized
                    .selected_line()
                    .request()
                    .context
                    .end
                    == LineEnd::Soft
                && result[last - 1]
                    .finalized
                    .selected_line()
                    .request()
                    .alternative
                    .is_some()
            {
                terminal_constraint_rejected = true;
                // Reject the completed suffix, then visit the next actual choice at
                // its deepest decision. Exhausted descendants backtrack above.
                if result.len() <= entry.prefix_len {
                    return Ok(SequenceOutcome::Inadmissible(
                        SequenceRejection::ScopeExhausted(if terminal_constraint_rejected {
                            ScopeExhaustion::TerminalConstraint
                        } else {
                            ScopeExhaustion::NoAdmissibleCandidate
                        }),
                    ));
                }
                let index = result.len() - 1;
                let depth = index - entry.prefix_len;
                visited.truncate(depth + 1);
                visited[depth].push(result[index].key());
                continue 'sequence;
            }
            return Ok(SequenceOutcome::Complete(SequenceEntry {
                boundary: current_boundary,
                logical_start,
                minimum_block,
                indent_state,
                line_box_context,
                hyphenation_state,
                incoming,
                prefix_len: result.len(),
            }));
        }
    }
}

fn query_bands(
    source: &mut impl FnMut(f64, f64) -> Result<Band, FitError>,
    logical_start: f64,
    placed_start: f64,
    height: f64,
) -> Result<LineBands, FitError> {
    let allocation = source(logical_start, finite(logical_start + height)?)?;
    let metrics = if logical_start == placed_start {
        allocation
    } else {
        source(placed_start, finite(placed_start + height)?)?
    };
    for band in [allocation, metrics] {
        finite(band.start)?;
        finite(band.end)?;
        if let Some(occupation) = band.intrinsic_end_occupation {
            finite(occupation)?;
        }
    }
    Ok(LineBands {
        allocation,
        metrics,
    })
}

fn finite(value: f64) -> Result<f64, FitError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FitError::NonFinite(value))
    }
}

/// The actual reference context participates in pass/cycle identity, independently
/// of a candidate's possibly different font-height/float allocation frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZoneBasis {
    policy: crate::LengthPercentageOf<f64>,
    references: Vec<ZoneReference>,
}

/// The full numerical/replay truth of an otherwise-empty phrase trial remains
/// Layout-owned and participates in accepted-pass convergence.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhraseBasis {
    ordinary: CandidateBasis,
    position: CandidatePosition,
    indent_state: crate::text_indent::IndentLineState,
    trial: CandidateBasis,
    fits: bool,
}
#[derive(Clone, Debug, PartialEq)]
struct ZoneReference {
    alternative: PreparedAlternative,
    request: LineSelectionRequest,
    zone: f64,
    measure: surgeist_text::InlineMeasure,
    bands: LineBands,
    encounters: Vec<EncounterBasis>,
    placed_start: f64,
}
fn used_zone(policy: crate::LengthPercentageOf<f64>, span: f64) -> Result<f64, FitError> {
    let basis = crate::PercentageBasisOf::definite(span).map_err(|_| FitError::InvalidGeometry)?;
    match policy.resolve_against(basis) {
        crate::NumericResolutionOf::Resolved(value) => Ok(if value == 0.0 { 0.0 } else { value }),
        crate::NumericResolutionOf::MissingBasis { .. } => Err(FitError::MissingZoneBasis),
        crate::NumericResolutionOf::InvalidNumeric { resolved, .. } => {
            Err(FitError::NonFinite(resolved))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        text::{PreparedParagraph, ahem_style},
        text_fragment::fragment_for,
    };

    fn selected(style: surgeist_text::Style, source: &str, width: f64) -> SelectedLine {
        let paragraph = PreparedParagraph::plain(1, surgeist_text::Source::new(source), style);
        let original = fragment_for(&paragraph.prepared).finalized_line();
        let mut request = original.selected_line().request().clone();
        request.context.available_inline = Some(width);
        request.context.alignment = Alignment::Justify;
        request.context.spacing = SpacingPolicy::Spaces;
        paragraph.prepared.select(request).unwrap()
    }

    fn retained(line: &FinalizedLine) -> Vec<usize> {
        line.spacing_slots()
            .enumerate()
            .filter_map(|(index, row)| {
                matches!(row.disposition, SpacingSlotDisposition::Retained { .. }).then_some(index)
            })
            .collect()
    }

    #[test]
    fn native_spacing_budget_refuses_unresolved_actual_advance() {
        let selected = selected(ahem_style(10.0, 10.0), "X X X", 60.0);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(natural.feedback().advance, 50.0);
        let retained = retained(&natural);
        assert_eq!(retained, [0, 1]);
        // Real complete amounts produce151. The final1 cannot absorb the91
        // overshoot; the completed correction [100,0] still produces150.
        // Refusal preserves these actual facts instead of dropping more slots.
        let mut amounts = vec![100.0, 1.0];
        let proposed = finalize_spacing(&selected, &amounts)
            .unwrap_or_else(|_| panic!("real selected spacing finalizes"));
        assert_eq!(proposed.feedback().advance, 151.0);
        match constrain_spacing_budget(&selected, &retained, 60.0, &mut amounts, proposed) {
            Err(FitError::SpacingBudget { available, advance }) => {
                assert_eq!(available, 60.0);
                assert_eq!(advance, 150.0);
            }
            _ => panic!("unresolved actual advance must be a Layout spacing-budget refusal"),
        }
        assert_eq!(amounts, [100.0, 0.0]);
        let actual = finalize_spacing(&selected, &amounts)
            .unwrap_or_else(|_| panic!("actual corrected complete table finalizes"));
        assert_eq!(actual.feedback().advance, 150.0);
        assert_eq!(actual.selected_line(), selected);
    }

    #[test]
    fn native_spacing_budget_correction_preserves_a_signed_slot_base() {
        let mut style = ahem_style(10.0, 10.0);
        style.word_spacing = -15.0;
        let selected = selected(style, "X X", 25.0);
        let natural = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(natural.feedback().advance, 15.0);
        assert_eq!(natural.spacing_slots().next().unwrap().advance, -5.0);
        let mut amounts = vec![20.0];
        let proposed = finalize_spacing(&selected, &amounts)
            .unwrap_or_else(|_| panic!("real signed-base spacing finalizes"));
        assert_eq!(proposed.feedback().advance, 35.0);
        let actual =
            constrain_spacing_budget(&selected, &retained(&natural), 25.0, &mut amounts, proposed)
                .unwrap_or_else(|_| panic!("one actual overshoot correction fits"));
        assert_eq!(amounts, [10.0]);
        assert_eq!(actual.feedback().advance, 25.0);
        assert_eq!(actual.spacing_slots().next().unwrap().advance, 5.0);
        assert_eq!(actual.selected_line(), selected);
    }

    #[test]
    fn native_spacing_budget_keeps_actual_text_lost_contribution_refusal() {
        let selected = selected(ahem_style(10.0, 10.0), "X X", 40.0);
        let amount = 1.0628135541960295e-153;
        match finalize_spacing(&selected, &[amount]) {
            Err(FitError::Text(error)) => assert!(matches!(error.detail(),
                Some(surgeist_text::ErrorDetail::InvalidTextNumeric {
                    field:"selected spacing amount",phase:surgeist_text::TextNumericPhase::Finalization,
                    category:surgeist_text::TextNumericError::NonZeroLost,value,
                }) if *value==amount)),
            _ => panic!("actual Text numeric refusal must pass through unchanged"),
        }
    }
}
