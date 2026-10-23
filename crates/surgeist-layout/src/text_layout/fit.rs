//! Candidate fitting over the actual immutable Text operations.

use crate::{TextAlign, TextAlignLast, TextJustification};
use surgeist_text::{
    Alignment, AtomicAllocation, ConditionalPurpose, ControlAllocation, FinalizedLine, LineContext,
    LineEnd, LineOrientation, LineSelectionRequest, LineStart, ParagraphBaseDirection,
    PreparedAlternative, PreparedBoundary, PreparedBreakKind, PreparedParticipant,
    PreparedTermination, PreparedText, SelectedLine, SpacingAllocation, SpacingAllocations,
    SpacingPolicy, SpacingSlotDisposition, TabContext, TabPreparation,
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
#[derive(Clone, Copy)]
pub(crate) struct EncounterReplay {
    pub source_floor: f64,
    pub logical: f64,
    pub placed: f64,
    pub defer_preceding: bool,
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

/// Actual source-event ordinal and accepted scalar geometry for cycle identity.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EncounterBasis {
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
    encounters: Vec<EncounterBasis>,
    request: LineSelectionRequest,
    amounts: Vec<f64>,
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
}

/// Accepted logical allocation and any genuine following clearance movement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineFlow {
    pub start: f64,
    pub end: f64,
    pub next: f64,
}

pub(crate) struct FittedLine {
    pub encounters: Vec<EncounterBasis>,
    pub finalized: FinalizedLine,
    pub amounts: Vec<f64>,
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
    FeedbackCycle,
    SpacingBudget { available: f64, advance: f64 },
    NonFinite(f64),
    Numeric(f64, crate::LayoutTextNumericError),
    InvalidGeometry,
    Dependency,
}

impl From<surgeist_text::Error> for FitError {
    fn from(error: surgeist_text::Error) -> Self {
        Self::Text(error)
    }
}

pub(crate) struct FitInput<'a> {
    pub prepared: &'a PreparedText,
    pub available: Available,
    pub base_direction: ParagraphBaseDirection,
    pub block_direction: ParagraphBaseDirection,
    pub alignment: TextAlign,
    pub last_alignment: TextAlignLast,
    pub justification: TextJustification,
    pub indent: crate::text_indent::ResolvedTextIndent<f64>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub orientation: LineOrientation,
    pub atomics: &'a [AtomicAllocation],
    pub controls: &'a [ControlAllocation],
    pub line_strut: Option<surgeist_text::LineStrut>,
    pub reusable: &'a [ReusableLine],
}

pub(crate) struct LineAllocations {
    pub atomics: Vec<AtomicAllocation>,
    pub controls: Vec<ControlAllocation>,
    pub line_strut: Option<surgeist_text::LineStrut>,
}

struct FinalizedCandidate {
    request: LineSelectionRequest,
    finalized: FinalizedLine,
    amounts: Vec<f64>,
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

fn finalize_candidate(
    prepared: &PreparedText,
    reusable: &[ReusableLine],
    mut request: LineSelectionRequest,
    normal: TextAlign,
    last: TextAlignLast,
    justification: TextJustification,
) -> Result<FinalizedCandidate, FitError> {
    let effective = normal.for_line(last, request.context.end != LineEnd::Soft);
    request.context.alignment = text_alignment(effective);
    let natural = natural_line(prepared, reusable, &request)?;
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
                finite(width - request.context.indent).and_then(|capacity| {
                    finite(capacity - natural.measurements().alignment().advance())
                        .map(|extra| (capacity, extra))
                })
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
                });
            }
        } else {
            // Pinned CSS Text3 §6.4.3; a new alignment is a new actual request,
            // so no slot capabilities are transferred from the zero pass.
            request.context.alignment = text_alignment(normal.unexpandable(last));
            let finalized = natural_line(prepared, reusable, &request)?;
            amounts = vec![0.0; finalized.selected_line().spacing_slots().len()];
            return Ok(FinalizedCandidate {
                request,
                finalized,
                amounts,
            });
        }
    }
    Ok(FinalizedCandidate {
        request,
        finalized: natural,
        amounts,
    })
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
#[derive(Clone)]
struct CandidatePosition {
    source_floor: f64,
    logical_start: f64,
    minimum_block: Option<f64>,
    deferred_from: Option<LineSelectionRequest>,
}

struct EvaluatedCandidate {
    request: LineSelectionRequest,
    finalized: FinalizedLine,
    amounts: Vec<f64>,
    encounters: Vec<EncounterBasis>,
    bands: LineBands,
    placed_start: f64,
    fits: bool,
    formatted: bool,
}

/// These callbacks all operate on the same unpublished invocation. Reset only
/// restores its accepted encounter checkpoint; it is not a child publication.
struct CandidateEnvironment<B, C, A, R, X> {
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
    A: FnMut(&FinalizedLine, Band, f64) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(),
{
    fn evaluate(
        &mut self,
        input: &FitInput<'_>,
        mut request: LineSelectionRequest,
        position: &CandidatePosition,
    ) -> Result<EvaluatedCandidate, FitError> {
        (self.reset)();
        let mut height = 0.0;
        let mut placed_start = position.logical_start;
        let mut rejected = Vec::new();
        loop {
            let bands = query_bands(&mut self.band, position.logical_start, placed_start, height)?;
            let band = bands.combined();
            request.context.available_inline = selection_width(input.available, band)?;
            request.context.tabs = tab_context(input, band, request.context.available_inline)?;
            let mut allocation_bases = Vec::new();
            let (finalized, amounts) = loop {
                let candidate = finalize_candidate(
                    input.prepared,
                    input.reusable,
                    request,
                    input.alignment,
                    input.last_alignment,
                    input.justification,
                )?;
                request = candidate.request;
                (self.replay)(&candidate.finalized, position.replay(placed_start))?;
                let actual = (self.allocate)(&candidate.finalized, band, placed_start)?;
                if actual.atomics == request.atomics
                    && actual.controls == request.controls
                    && actual.line_strut == request.line_strut
                {
                    break (candidate.finalized, candidate.amounts);
                }
                let basis = (request.clone(), candidate.amounts);
                if allocation_bases.contains(&basis) {
                    return Err(FitError::FeedbackCycle);
                }
                allocation_bases.push(basis);
                request.atomics = actual.atomics;
                request.controls = actual.controls;
                request.line_strut = actual.line_strut;
            };
            let encounters = (self.replay)(&finalized, position.replay(placed_start))?;
            let feedback = finalized.feedback();
            let actual_height = finite(feedback.line_height)?;
            let actual_start = (self.canonical)(
                BlockPosition {
                    proposed: position.logical_start,
                    minimum: position.minimum_block,
                },
                feedback.baseline,
            )?;
            let actual_bands = query_bands(
                &mut self.band,
                position.logical_start,
                actual_start,
                actual_height,
            )?;
            let basis = CandidateBasis {
                encounters: encounters.clone(),
                request: request.clone(),
                amounts: amounts.clone(),
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
            // Retain the genuine receipt even for a phantom Soft trial.
            // The conditional query authenticates complete earlier inventory.
            let formatted = if request.context.end == LineEnd::Soft {
                match request.participants.last() {
                    Some(last) => finalized
                        .prefix_at(&surgeist_text::TextSeam::AfterParticipant(last.clone()))?
                        .has_retained_flow_participation(),
                    None => false,
                }
            } else {
                true
            };
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
            let fits = fit_width.is_none_or(|width| {
                measurement.advance() <= width - request.context.indent
                    && occupied <= width - request.context.indent
            });
            return Ok(EvaluatedCandidate {
                request,
                finalized,
                amounts,
                encounters,
                bands,
                placed_start,
                fits,
                formatted,
            });
        }
    }

    /// A later rejected candidate may own the provisional ledger. Restore and
    /// verify the chosen candidate's actual complete receipt before accepting it.
    fn restore(
        &mut self,
        candidate: &EvaluatedCandidate,
        position: &CandidatePosition,
    ) -> Result<(), FitError> {
        (self.reset)();
        let encounters = (self.replay)(
            &candidate.finalized,
            position.replay(candidate.placed_start),
        )?;
        let bands = query_bands(
            &mut self.band,
            position.logical_start,
            candidate.placed_start,
            candidate.finalized.feedback().line_height,
        )?;
        let actual = (self.allocate)(
            &candidate.finalized,
            candidate.bands.combined(),
            candidate.placed_start,
        )?;
        if encounters != candidate.encounters
            || bands != candidate.bands
            || actual.atomics != candidate.request.atomics
            || actual.controls != candidate.request.controls
            || actual.line_strut != candidate.request.line_strut
        {
            return Err(FitError::FeedbackCycle);
        }
        Ok(())
    }
}

impl CandidatePosition {
    fn replay(&self, placed: f64) -> EncounterReplay {
        EncounterReplay {
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

/// Resolve the captured block edge relative to this actual allocation's
/// paragraph progression start. Shared starts cancel without a block width.
/// An intrinsic opposite start depends on the candidate's still-unknown width.
fn tab_context(
    input: &FitInput<'_>,
    band: Band,
    allocation: Option<f64>,
) -> Result<TabContext, FitError> {
    if input.prepared.preparation_options().tabs() == TabPreparation::FontAdvance {
        return Ok(TabContext::FontAdvance);
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
pub(crate) struct EncounterCallbacks<Replay, Accept, Reset> {
    pub replay: Replay,
    pub accept: Accept,
    pub reset: Reset,
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
    endpoint: LineEndpoint,
    indent_state: crate::text_indent::IndentLineState,
) -> LineSelectionRequest {
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
    LineSelectionRequest {
        start: start.clone(),
        end: endpoint.boundary,
        participants,
        termination: endpoint.termination,
        alternative: endpoint.alternative,
        context: LineContext {
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
        line_strut: input.line_strut,
    }
}

/// Fit only preparation-issued opportunities. Every measured candidate executes
/// its own retained finalizer and complete convergence, not preliminary widths.
pub(crate) fn fit(
    input: FitInput<'_>,
    band_source: impl FnMut(f64, f64) -> Result<Band, FitError>,
    canonical_block: impl FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    allocate: impl FnMut(&FinalizedLine, Band, f64) -> Result<LineAllocations, FitError>,
    mut clear_source: impl FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
    encounters: EncounterCallbacks<
        impl FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
        impl FnMut() -> Result<(), FitError>,
        impl FnMut(),
    >,
) -> Result<Vec<FittedLine>, FitError> {
    let EncounterCallbacks {
        replay: replay_encounters,
        accept: mut accept_encounters,
        reset: reset_candidate,
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
    let mut environment = CandidateEnvironment {
        band: band_source,
        canonical: canonical_block,
        allocate,
        replay: replay_encounters,
        reset: reset_candidate,
    };
    let mut result = Vec::new();
    let mut start = 0;
    let mut logical_start = 0.0;
    let mut minimum_block: Option<f64> = None;
    let mut indent_state = input.indent_state;
    let mut completed_end = false;
    while !completed_end {
        let start_boundary = boundaries
            .iter()
            .find(|(offset, _)| *offset == start)
            .expect("a consumed opportunity retains its mixed boundary")
            .1
            .clone();
        let mut position = CandidatePosition {
            source_floor: logical_start,
            logical_start,
            minimum_block,
            deferred_from: None,
        };
        let mut transitions = Vec::new();
        loop {
            let mut choice: Option<EvaluatedCandidate> = None;
            let mut first_ceiling = None;
            let mut skipped_ordinary = Vec::new();
            for (end, opportunity, kind, alternatives) in &opportunities {
                if *end < start || (*end == start && *kind != PreparedBreakKind::ParagraphEnd) {
                    continue;
                }
                let variants = (*kind != PreparedBreakKind::Discretionary)
                    .then_some(None)
                    .into_iter()
                    .chain(alternatives.iter().cloned().map(Some));
                let mut endpoint_choice: Option<EvaluatedCandidate> = None;
                let mut phantom_variants = Vec::new();
                for alternative in variants {
                    let request = selection_request(
                        &input,
                        &start_boundary,
                        LineEndpoint {
                            boundary: opportunity.view().boundary(),
                            termination: PreparedTermination::Ordinary(opportunity.clone()),
                            alternative,
                            ending: match kind {
                                PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => {
                                    LineEnd::Soft
                                }
                                PreparedBreakKind::Forced => LineEnd::Forced,
                                PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                            },
                        },
                        indent_state,
                    );
                    let candidate = environment.evaluate(&input, request, &position)?;
                    if !candidate.formatted {
                        phantom_variants.push(candidate.finalized);
                        continue;
                    }
                    let fits = candidate.fits;
                    if endpoint_choice.is_none() || fits {
                        endpoint_choice = Some(candidate);
                    }
                    // An actual fitting declared variant retains its preference.
                    if fits {
                        break;
                    }
                }
                if let Some(candidate) = endpoint_choice {
                    if first_ceiling.is_none() {
                        first_ceiling = Some(opportunity.clone());
                    }
                    if choice.is_none()
                        || candidate.fits
                        || matches!(input.available, Available::Maximum)
                    {
                        choice = Some(candidate);
                    }
                    if matches!(input.available, Available::Minimum) {
                        break;
                    }
                } else if first_ceiling.is_none() {
                    // Every legal variant was executed and proved phantom.
                    // Keep the original start and every original occurrence.
                    skipped_ordinary.extend(phantom_variants);
                }
                if !matches!(
                    kind,
                    PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                ) {
                    break;
                }
            }
            let Some(mut candidate) = choice else {
                return Err(FitError::FeedbackCycle);
            };
            let purpose = match input.available {
                Available::Minimum => Some(ConditionalPurpose::MinContent),
                Available::Definite(_) if !candidate.fits => Some(ConditionalPurpose::Overflow),
                Available::Definite(_) | Available::Maximum => None,
            };
            let mut minimal_progress = None;
            if let Some(purpose) = purpose
                && let Some(ceiling) = first_ceiling
                && ceiling.view().boundary().view().participant_offset() > start
            {
                let mut fitting = None;
                for opportunity in input.prepared.conditional_opportunities(
                    &start_boundary,
                    &ceiling,
                    purpose,
                    &skipped_ordinary,
                )? {
                    let request = selection_request(
                        &input,
                        &start_boundary,
                        LineEndpoint {
                            boundary: opportunity.boundary(),
                            termination: PreparedTermination::Conditional(opportunity),
                            alternative: None,
                            ending: LineEnd::Soft,
                        },
                        indent_state,
                    );
                    let trial = environment.evaluate(&input, request, &position)?;
                    if !trial.formatted {
                        continue;
                    }
                    if purpose == ConditionalPurpose::MinContent {
                        fitting = Some(trial);
                        break;
                    }
                    if trial.fits {
                        fitting = Some(trial);
                    } else if minimal_progress.is_none() {
                        minimal_progress = Some(trial);
                    }
                    // Do not assume that later shaped prefixes are wider.
                }
                if let Some(fitting) = fitting {
                    candidate = fitting;
                }
            }
            environment.restore(&candidate, &position)?;
            let basis = (
                candidate.request.clone(),
                candidate.amounts.clone(),
                candidate.encounters.clone(),
                candidate.bands,
                candidate.placed_start,
                position.logical_start,
                position.minimum_block,
                position.deferred_from.clone(),
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
                        defer_preceding: true,
                        ..position.replay(candidate.placed_start)
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
            let feedback = candidate.finalized.feedback();
            let logical_end = finite(position.logical_start + feedback.line_height)?;
            let cleared = clear_source(logical_end, &candidate.request)?
                .map(finite)
                .transpose()?
                .map_or(logical_end, |minimum| minimum.max(logical_end));
            minimum_block = position.minimum_block;
            if cleared > logical_end {
                minimum_block = Some(minimum_block.map_or(cleared, |minimum| minimum.max(cleared)));
            }
            let ending = candidate.request.context.end;
            let end = candidate.request.end.view().participant_offset();
            indent_state.commit(ending == LineEnd::Forced);
            accept_encounters()?;
            let available = candidate.finalized.metric_bounds().size.width;
            result.push(FittedLine {
                encounters: candidate.encounters,
                finalized: candidate.finalized,
                amounts: candidate.amounts,
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
            break;
        }
    }
    Ok(result)
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
