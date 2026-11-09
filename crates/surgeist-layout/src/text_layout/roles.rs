//! Candidate-local formatted roles and nonplaceable measurement flow evidence.

use super::fit::{EncounterBasis, FitError, FitInput, FittedLine, LineBands};
use super::formatted::{EntryCopies, FormattedTailAssignment, SharedPayload};
use super::scope_alignment::{EntryBacking, EntryFundingRef};
use std::ops::Deref;
use surgeist_text as text;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RoleUse(SharedPayload<RoleUseData>);
pub(crate) type RoleUseRows = SharedPayload<Vec<RoleUse>>;

pub(super) fn retained_rows(
    rows: Vec<RoleUse>,
    copies: EntryCopies,
    funding: Option<EntryFundingRef<'_>>,
) -> text::Result<RoleUseRows> {
    let funding = if rows.is_empty() { None } else { funding };
    SharedPayload::with_copies_ref(rows, copies, funding)
}

pub(super) fn contains(
    rows: &[RoleUse],
    used: &RoleUse,
    funding: Option<EntryFundingRef<'_>>,
) -> text::Result<bool> {
    for row in rows {
        visit(funding, 1)?;
        if row == used {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn append_rows(
    rows: &mut RoleUseRows,
    extra: Vec<RoleUse>,
    funding: Option<EntryFundingRef<'_>>,
) -> Result<(), FitError> {
    if extra.is_empty() {
        return Ok(());
    }
    let count = rows
        .len()
        .checked_add(extra.len())
        .ok_or(FitError::Allocation)?;
    let previous = std::mem::replace(rows, SharedPayload::new(Vec::new(), None)?);
    *rows = match previous.into_native() {
        Ok(mut native) if funding.is_none() => {
            native.extend(extra);
            retained_rows(native, EntryCopies::default(), None)?
        }
        Ok(native) => merged_rows(native.into_iter(), extra, count, funding)?,
        Err(shared) => merged_rows(shared.into_rows(), extra, count, funding)?,
    };
    Ok(())
}

fn merged_rows(
    previous: impl Iterator<Item = RoleUse>,
    extra: Vec<RoleUse>,
    count: usize,
    funding: Option<EntryFundingRef<'_>>,
) -> Result<RoleUseRows, FitError> {
    let mut copies = EntryCopies::default();
    let mut rows = copies.rows_ref(funding, count)?;
    for used in previous.chain(extra) {
        visit(funding, 1)?;
        rows.push(used);
    }
    Ok(retained_rows(rows, copies, funding)?)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RoleUseData {
    pub(crate) disposition: super::fit::LineDisposition,
    pub(crate) request: text::LineSelectionRequest,
    pub(crate) boxes: Option<super::boxes::BoxAssignment>,
    pub(crate) amounts: Vec<f64>,
    pub(crate) usage: text::FormattedLineUsage,
    pub(crate) outgoing: Option<text::HyphenationContinuation>,
    pub(crate) basis: RoleBasis,
    pub(crate) bands: LineBands,
    pub(crate) encounters: Vec<EncounterBasis>,
    pub(crate) frame: super::fit::CandidateFrame,
    pub(crate) placed_start: f64,
}

impl RoleUse {
    pub(crate) fn new(
        value: RoleUseData,
        copies: EntryCopies,
        funding: Option<EntryFundingRef<'_>>,
    ) -> text::Result<Self> {
        SharedPayload::with_copies_ref(value, copies, funding).map(Self)
    }
    pub(crate) fn last_assignment(&self) -> Option<&FormattedTailAssignment> {
        if self.usage.last() {
            self.demanded_assignment()
        } else {
            None
        }
    }

    pub(crate) fn demanded_assignment(&self) -> Option<&FormattedTailAssignment> {
        match &self.basis {
            RoleBasis::Actual { assignment, .. } => assignment.as_ref(),
            RoleBasis::Prospective(basis) => basis.external.as_ref(),
        }
    }
}

impl Deref for RoleUse {
    type Target = RoleUseData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RoleBasis {
    Actual {
        first: bool,
        assignment: Option<FormattedTailAssignment>,
    },
    Prospective(ProspectiveRoleBasis),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProspectiveRoleBasis {
    // Entry clones share the enclosing Arc and this original Box/lease. Native
    // values retain the existing Box deep-copy behavior without a large variant.
    data: SharedPayload<Box<ProspectiveRoleData>>,
    external: Option<FormattedTailAssignment>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProspectiveRoleData {
    recipe: text::PreparedMeasurementFlow,
    observations: Vec<text::FlowObservation>,
    boxes: Vec<Option<super::boxes::BoxAssignment>>,
    query_start: text::PreparedBoundary,
    query_incoming: Option<text::HyphenationContinuation>,
    checkpoint: EntryCheckpoint,
    prior: bool,
    following: bool,
}

impl Deref for ProspectiveRoleBasis {
    type Target = ProspectiveRoleData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Clone, Debug, PartialEq)]
struct EntryCheckpoint(SharedPayload<EntryCheckpointData>);

#[derive(Clone, Debug, PartialEq)]
struct EntryCheckpointData {
    phase: super::fit::EntryPhase,
    entry: text::PreparedBoundary,
    first: bool,
    outside: bool,
    box_consumed: bool,
    receipts: Vec<CheckpointReceipt>,
}

impl Deref for EntryCheckpoint {
    type Target = EntryCheckpointData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(super) fn visit(funding: Option<EntryFundingRef<'_>>, count: usize) -> text::Result<()> {
    funding.map_or(Ok(()), |funding| funding.visit(count))
}

pub(super) fn copied_rows<T: Clone>(
    copies: &mut EntryCopies,
    funding: Option<EntryFundingRef<'_>>,
    values: &[T],
) -> text::Result<Vec<T>> {
    let mut rows = copies.rows_ref(funding, values.len())?;
    visit(funding, values.len())?;
    rows.extend_from_slice(values);
    Ok(rows)
}

#[derive(Clone, Debug)]
struct CheckpointReceipt {
    frame: super::fit::CandidateFrame,
    line: text::FinalizedLine,
    amounts: Vec<f64>,
    boxes: Option<super::boxes::BoxAssignment>,
}

impl PartialEq for CheckpointReceipt {
    fn eq(&self, other: &Self) -> bool {
        // Finalization is deterministic from these admitted inputs. Retain the
        // authentic receipt for observation without comparing its allocation.
        self.frame == other.frame
            && self.line.selected_line().request() == other.line.selected_line().request()
            && self.amounts == other.amounts
            && self.line.scope_allocations() == other.line.scope_allocations()
            && self.boxes == other.boxes
    }
}

pub(super) struct RoleEntry<'a> {
    pub current_first: bool,
    pub phase: super::fit::EntryPhase,
    pub query_start: &'a text::PreparedBoundary,
    pub query_incoming: Option<&'a text::HyphenationContinuation>,
    pub boxes: Option<&'a super::boxes::BoxAssignment>,
    pub accepted: &'a [FittedLine],
    pub preceding: Option<(
        &'a text::FinalizedLine,
        &'a [f64],
        Option<&'a super::boxes::BoxAssignment>,
        &'a super::fit::CandidateFrame,
    )>,
}

pub(super) fn measurement(request: &text::LineSelectionRequest) -> bool {
    matches!(
        &request.termination,
        text::PreparedTermination::Measurement(_) | text::PreparedTermination::PhraseMeasurement(_)
    )
}

pub(super) fn initial_basis(
    request: &text::LineSelectionRequest,
    first: bool,
) -> Option<RoleBasis> {
    (!measurement(request)).then_some(RoleBasis::Actual {
        first,
        assignment: None,
    })
}

pub(super) fn resolve(
    input: &FitInput<'_>,
    request: &text::LineSelectionRequest,
    entry: &RoleEntry<'_>,
    role: text::PhaseTwoContext,
    basis: &mut Option<RoleBasis>,
) -> Result<Option<bool>, FitError> {
    if measurement(request) {
        if basis.is_none() {
            *basis = Some(RoleBasis::Prospective(prospective(input, request, entry)?));
        }
        let Some(RoleBasis::Prospective(value)) = basis else {
            return Err(invalid_flow());
        };
        return match role {
            text::PhaseTwoContext::FirstFormattedLine => Ok(Some(
                value.checkpoint.first && !value.checkpoint.outside && !value.prior,
            )),
            text::PhaseTwoContext::LastFormattedLine if value.following => Ok(Some(false)),
            text::PhaseTwoContext::LastFormattedLine => {
                let completion = value.recipe.completion().view().boundary();
                value.external =
                    input
                        .formatted
                        .resolve_external(&input.segment, &completion, None);
                Ok(value.external.as_ref().map(FormattedTailAssignment::last))
            }
            _ => Ok(None),
        };
    }
    let Some(RoleBasis::Actual { assignment, .. }) = basis else {
        return Err(invalid_flow());
    };
    match role {
        text::PhaseTwoContext::FirstFormattedLine => Ok(Some(entry.current_first)),
        text::PhaseTwoContext::LastFormattedLine => {
            *assignment =
                input
                    .formatted
                    .resolve(&input.segment, &request.end, request.alternative.as_ref());
            Ok(assignment.as_ref().map(FormattedTailAssignment::last))
        }
        _ => Ok(None),
    }
}

fn prospective(
    input: &FitInput<'_>,
    request: &text::LineSelectionRequest,
    entry: &RoleEntry<'_>,
) -> Result<ProspectiveRoleBasis, FitError> {
    let recipe = input.prepared.measurement_flow(
        &input.segment,
        entry.query_start,
        entry.query_incoming,
        &request.termination,
        input.inline_boxes,
    )?;
    let mut copies = EntryCopies::default();
    let mut observations = Vec::new();
    let mut box_assignments = Vec::new();
    let box_checkpoint = if input.wrapping.is_some() {
        Some(checkpoint(input, &recipe.entry(), entry, false)?)
    } else {
        None
    };
    let mut unconsumed_prefix = box_checkpoint
        .as_ref()
        .is_some_and(|checkpoint| !checkpoint.box_consumed);
    let mut box_frontier = box_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.receipts.last())
        .and_then(|receipt| {
            let selected = receipt.line.selected_line();
            receipt.boxes.as_ref().map(|boxes| {
                if selected.request().end == recipe.entry() {
                    boxes.outgoing.clone()
                } else {
                    boxes.incoming.clone()
                }
            })
        });
    let phrase = matches!(
        &request.termination,
        text::PreparedTermination::PhraseMeasurement(_)
    );
    let mut prior = false;
    let mut following = false;
    let mut outside = false;
    for segment in recipe.segments() {
        visit(input.entry_funding, 1)?;
        let boxes = if segment.role() == text::FlowSegmentRole::Target {
            entry.boxes.cloned()
        } else if let Some(resolver) = input.wrapping {
            let mut controls_copies = EntryCopies::default();
            let controls = if input.entry_funding.is_some() {
                let mut count = 0;
                for allocation in input.controls {
                    visit(input.entry_funding, 1)?;
                    if contains_control(input, &segment, &allocation.occurrence)? {
                        count += 1;
                    }
                }
                let mut controls = controls_copies.rows_ref(input.entry_funding, count)?;
                for allocation in input.controls {
                    visit(input.entry_funding, 1)?;
                    if contains_control(input, &segment, &allocation.occurrence)? {
                        controls.push(allocation.clone());
                    }
                }
                controls
            } else {
                input.controls.iter().filter(|allocation| segment.participants().any(|participant| matches!(participant, text::PreparedParticipant::Control(control) if control == allocation.occurrence))).cloned().collect()
            };
            let choices = resolver.choices(super::boxes::BoxChoice {
                start: &segment.start(),
                end: &segment.end(),
                incoming: box_frontier.as_ref().map(|frontier| {
                    if unconsumed_prefix {
                        super::boxes::BoxEntry::UnconsumedFrom(frontier)
                    } else {
                        super::boxes::BoxEntry::ConsumedTo(frontier)
                    }
                }),
                controls: &controls,
                ending: flow_ending(&segment),
                authority: super::boxes::BoxAuthority::Observed {
                    recipe: &recipe,
                    segment: &segment,
                    observations: &observations,
                    preceding: box_assignments.last().and_then(Option::as_ref),
                },
            })?;
            if entry
                .boxes
                .is_some_and(|target| target.incoming.boundary == segment.end())
            {
                choices
                    .into_iter()
                    .find(|choice| {
                        entry
                            .boxes
                            .is_some_and(|target| choice.outgoing == target.incoming)
                    })
                    .ok_or_else(invalid_flow)
                    .map(Some)?
            } else {
                choices.into_iter().next()
            }
        } else {
            None
        };
        let observation = observe(input, request, &segment, boxes.as_ref())?;
        let retained = observation.has_retained_flow_participation();
        let forced = segment
            .mandatory_end()
            .is_some_and(|offer| offer.view().kind() == text::PreparedBreakKind::Forced);
        let emitted = observation.has_in_flow_content() || forced;
        if let Some(boxes) = &boxes {
            if retained || forced {
                box_frontier = Some(boxes.outgoing.clone());
                unconsumed_prefix = false;
            } else {
                if box_frontier.is_none() {
                    box_frontier = Some(boxes.incoming.clone());
                }
                unconsumed_prefix = true;
            }
        }
        if input.entry_funding.is_some() && box_assignments.capacity() == 0 {
            box_assignments = copies.rows_ref(input.entry_funding, recipe.segments().len())?;
        }
        box_assignments.push(boxes);
        match segment.role() {
            text::FlowSegmentRole::EntryPrefix => {
                if phrase && (emitted || segment.outgoing().is_some()) {
                    return Err(invalid_flow());
                }
                outside |= emitted;
            }
            text::FlowSegmentRole::PriorPartition => prior |= emitted,
            text::FlowSegmentRole::Target => {
                // The target's real cut/continuation remains in its receipt.
                // Only actual later emitted content/Forced segments establish
                // a following formatted line; a strut-only receiver does not.
            }
            text::FlowSegmentRole::Remaining => following |= emitted,
        }
        if input.entry_funding.is_some() && observations.capacity() == 0 {
            observations = copies.rows_ref(input.entry_funding, recipe.segments().len())?;
        }
        observations.push(observation);
    }
    recipe.validate_observations(&observations)?;
    let checkpoint = if phrase {
        EntryCheckpoint(SharedPayload::new(
            EntryCheckpointData {
                phase: entry.phase,
                entry: recipe.entry(),
                first: entry.current_first,
                outside: false,
                box_consumed: false,
                receipts: Vec::new(),
            },
            input.entry_funding,
        )?)
    } else {
        checkpoint(input, &recipe.entry(), entry, outside)?
    };
    let data = ProspectiveRoleData {
        recipe,
        observations,
        boxes: box_assignments,
        query_start: entry.query_start.clone(),
        query_incoming: entry.query_incoming.cloned(),
        checkpoint,
        prior,
        following,
    };
    copies.header_ref::<ProspectiveRoleData>(input.entry_funding)?;
    Ok(ProspectiveRoleBasis {
        data: SharedPayload::with_copies_ref(Box::new(data), copies, input.entry_funding)?,
        external: None,
    })
}

fn contains_control(
    input: &FitInput<'_>,
    segment: &text::PreparedFlowSegment,
    control: &text::PreparedControl,
) -> Result<bool, FitError> {
    for participant in segment.participants() {
        visit(input.entry_funding, 1)?;
        if matches!(participant, text::PreparedParticipant::Control(value) if &value == control) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn checkpoint(
    input: &FitInput<'_>,
    entry_boundary: &text::PreparedBoundary,
    entry: &RoleEntry<'_>,
    observed_outside: bool,
) -> Result<EntryCheckpoint, FitError> {
    let at = entry_boundary.view().participant_offset();
    let mut copies = EntryCopies::default();
    let lines = || {
        entry
            .accepted
            .iter()
            .map(|line| &line.finalized)
            .chain(entry.preceding.map(|(line, _, _, _)| line))
    };
    let mut count = 0;
    if input.entry_funding.is_some() {
        for line in lines() {
            visit(input.entry_funding, 1)?;
            let selected = line.selected_line();
            let request = selected.request();
            let from = request.start.view().participant_offset();
            let to = request.end.view().participant_offset();
            if to <= at {
                count += 1;
            } else if from <= at {
                count += 1;
                break;
            }
        }
    }
    let mut receipts = Vec::new();
    for (line, amounts, first, boxes, frame) in entry
        .accepted
        .iter()
        .map(|line| {
            (
                &line.finalized,
                line.amounts.as_slice(),
                line.first_formatted,
                line.boxes.as_ref(),
                &line.flow.frame,
            )
        })
        .chain(entry.preceding.map(|(line, amounts, boxes, frame)| {
            (line, amounts, entry.current_first, boxes, frame)
        }))
    {
        visit(input.entry_funding, 1)?;
        let selected = line.selected_line();
        let request = selected.request();
        let from = request.start.view().participant_offset();
        let to = request.end.view().participant_offset();
        if to <= at {
            let receipt = CheckpointReceipt {
                frame: frame.clone(),
                line: line.clone(),
                amounts: copied_rows(&mut copies, input.entry_funding, amounts)?,
                boxes: boxes.cloned(),
            };
            if input.entry_funding.is_some() && receipts.capacity() == 0 {
                receipts = copies.rows_ref(input.entry_funding, count)?;
            }
            receipts.push(receipt);
            continue;
        }
        if from <= at {
            let (outside, box_consumed) = if at == from {
                (false, false)
            } else {
                let participant = input
                    .prepared
                    .participants()
                    .nth(at)
                    .ok_or_else(invalid_flow)?;
                let prefix = line.prefix_at(&text::TextSeam::BeforeParticipant(participant))?;
                (
                    prefix.has_in_flow_content(),
                    prefix.has_retained_flow_participation(),
                )
            };
            let receipt = CheckpointReceipt {
                frame: frame.clone(),
                line: line.clone(),
                amounts: copied_rows(&mut copies, input.entry_funding, amounts)?,
                boxes: boxes.cloned(),
            };
            if input.entry_funding.is_some() && receipts.capacity() == 0 {
                receipts = copies.rows_ref(input.entry_funding, count)?;
            }
            receipts.push(receipt);
            let data = EntryCheckpointData {
                phase: entry.phase,
                entry: entry_boundary.clone(),
                first,
                outside: outside || observed_outside,
                box_consumed,
                receipts,
            };
            return Ok(EntryCheckpoint(SharedPayload::with_copies_ref(
                data,
                copies,
                input.entry_funding,
            )?));
        }
    }
    // The recipe's EntryPrefix observations supply an unaccepted entry. Any
    // real accepted line before that entry is outside the replaced word branch.
    let has_formatted = receipts.iter().try_fold(false, |seen, receipt| {
        visit(input.entry_funding, 1)?;
        Ok::<_, FitError>(
            seen || super::fit::LineDisposition::resolve(&receipt.line)?.is_formatted(),
        )
    })?;
    let first = input.indent_state.first_formatted() && !has_formatted;
    let box_consumed = if let Some(receipt) = receipts.last() {
        let selected = receipt.line.selected_line();
        let request = selected.request();
        if request.context.end != text::LineEnd::Soft {
            true
        } else if let Some(last) = request.participants.last() {
            receipt
                .line
                .prefix_at(&text::TextSeam::AfterParticipant(last.clone()))?
                .has_retained_flow_participation()
        } else {
            false
        }
    } else {
        false
    };
    let data = EntryCheckpointData {
        phase: entry.phase,
        entry: entry_boundary.clone(),
        first,
        outside: observed_outside,
        box_consumed,
        receipts,
    };
    Ok(EntryCheckpoint(SharedPayload::with_copies_ref(
        data,
        copies,
        input.entry_funding,
    )?))
}

fn flow_ending(segment: &text::PreparedFlowSegment) -> text::LineEnd {
    if segment.role() == text::FlowSegmentRole::Target {
        text::LineEnd::Soft
    } else {
        match segment.mandatory_end().map(|offer| offer.view().kind()) {
            Some(text::PreparedBreakKind::Forced) => text::LineEnd::Forced,
            Some(text::PreparedBreakKind::ParagraphEnd) => text::LineEnd::Paragraph,
            _ => text::LineEnd::Soft,
        }
    }
}

fn observe(
    input: &FitInput<'_>,
    target: &text::LineSelectionRequest,
    segment: &text::PreparedFlowSegment,
    boxes: Option<&super::boxes::BoxAssignment>,
) -> Result<text::FlowObservation, FitError> {
    let mut copies = EntryCopies::default();
    let mut atomics_backing = EntryBacking::default();
    let mut controls_backing = EntryBacking::default();
    let mut boundary_backing = EntryBacking::default();
    let mut participants = copies.rows_ref(input.entry_funding, segment.participants().len())?;
    visit(input.entry_funding, segment.participants().len())?;
    participants.extend(segment.participants());
    let mut atomics = Vec::new();
    for row in input.atomics {
        visit(input.entry_funding, 1)?;
        let mut included = false;
        for participant in &participants {
            visit(input.entry_funding, 1)?;
            if matches!(participant, text::PreparedParticipant::Atomic(value) if value == &row.occurrence)
            {
                included = true;
                break;
            }
        }
        if included {
            let mut actual = row;
            for candidate in &target.atomics {
                visit(input.entry_funding, 1)?;
                if candidate.occurrence == row.occurrence {
                    actual = candidate;
                    break;
                }
            }
            atomics_backing.reserve_push_ref(&mut atomics, input.entry_funding)?;
            atomics.push(actual.clone());
        }
    }
    let mut controls = Vec::new();
    for row in input.controls {
        visit(input.entry_funding, 1)?;
        let mut included = false;
        for participant in &participants {
            visit(input.entry_funding, 1)?;
            if matches!(participant, text::PreparedParticipant::Control(value) if value == &row.occurrence)
            {
                included = true;
                break;
            }
        }
        if included {
            let mut actual = row;
            for candidate in &target.controls {
                visit(input.entry_funding, 1)?;
                if candidate.occurrence == row.occurrence {
                    actual = candidate;
                    break;
                }
            }
            controls_backing.reserve_push_ref(&mut controls, input.entry_funding)?;
            controls.push(actual.clone());
        }
    }
    let line_box_struts = if segment.role() == text::FlowSegmentRole::Target {
        copied_rows(&mut copies, input.entry_funding, &target.line_box_struts)?
    } else if let Some(boxes) = boxes {
        let mut rows = copies.rows_ref(input.entry_funding, boxes.contributions.len())?;
        for contribution in &boxes.contributions {
            visit(input.entry_funding, 1)?;
            rows.push(contribution.original.clone());
        }
        rows
    } else {
        Vec::new()
    };
    let mut context = target.context.clone();
    context.formatted_line = text::FormattedLineContext::default();
    context.start = if segment.start().view().participant_offset() == 0 {
        text::LineStart::Paragraph
    } else {
        text::LineStart::Continuation
    };
    context.end = flow_ending(segment);
    let mut request = text::FlowObservationRequest {
        context,
        atomics,
        controls,
        line_strut: target.line_strut,
        line_box_struts,
        boundary_context: Vec::new(),
    };
    while let Some(occurrence) = input
        .prepared
        .flow_boundary_context_demand(segment, &request)?
    {
        let mut boundary = None;
        for row in input.controls {
            visit(input.entry_funding, 1)?;
            if row.occurrence == occurrence {
                boundary = row.boundary.clone();
                break;
            }
        }
        let boundary = boundary.ok_or_else(invalid_flow)?;
        for row in &request.boundary_context {
            visit(input.entry_funding, 1)?;
            if row.occurrence == occurrence {
                return Err(invalid_flow());
            }
        }
        boundary_backing.reserve_push_ref(&mut request.boundary_context, input.entry_funding)?;
        request
            .boundary_context
            .push(text::InlineBoundaryContextAllocation {
                occurrence,
                boundary,
            });
    }
    let operation = input.scope_alignments.begin(
        text::ScopeAlignmentCensusInput::Flow {
            prepared: input.prepared,
            segment,
            request: &request,
        },
        0,
    )?;
    Ok(match &operation {
        Some(operation) => input.prepared.observe_flow_with_scope_allowance(
            segment,
            &request,
            operation.allowance()?,
        )?,
        None => input.prepared.observe_flow(segment, &request)?,
    })
}

fn invalid_flow() -> FitError {
    FitError::Text(
        text::Error::new(
            text::ErrorCode::InvalidTextProtocol,
            "invalid prospective formatted flow",
        )
        .with_detail(text::ErrorDetail::InvalidTextProtocol {
            reason: text::ProtocolError::InvalidMeasurementFlow,
        }),
    )
}

#[cfg(test)]
mod funding_tests {
    use super::*;
    use crate::text_layout::scope_alignment::{
        EntryFunding, ScopeAlignmentFacts, ScopeAlignmentResources, ScopeAlignmentSnapshot,
    };

    const LIMITS: text::ScopeAlignmentLimits =
        text::ScopeAlignmentLimits::new(100_000, 100, 1_000_000, 16 << 20);

    fn sizing_request() -> crate::SizingRequestOf<f64> {
        crate::SizingRequestOf::new(
            crate::Size::new(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(100.0).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
            ),
            crate::OrthogonalFallbackOf::new(
                crate::Size::NONE,
                crate::NearestScrollportOf::Absent,
                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
    }

    fn source() -> (
        crate::test_support::inline_endpoint::Fixture,
        text::FinalizedLine,
    ) {
        let source =
            crate::test_support::inline_endpoint::Fixture::with_scope_limits("X X", LIMITS);
        let request: crate::SizingRequestOf<f64> = crate::SizingRequestOf::new(
            crate::Size::new(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(100.0).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
            ),
            crate::OrthogonalFallbackOf::new(
                crate::Size::NONE,
                crate::NearestScrollportOf::Absent,
                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        let batch = source.compute(
            crate::InlineBoundaryEdgeOf::ZERO,
            crate::InlineBoundaryEdgeOf::ZERO,
            false,
            Some(100.0),
            false,
            request,
        );
        let line = batch
            .final_inline_fragments()
            .first()
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line()
            .clone();
        (source, line)
    }

    fn owner(
        source: &crate::test_support::inline_endpoint::Fixture,
    ) -> ScopeAlignmentResources<f64> {
        ScopeAlignmentResources::new(
            ScopeAlignmentSnapshot::admit(vec![], std::slice::from_ref(&source.binding), false)
                .unwrap()
                .freeze(),
            crate::text_layout::fragmentation::InlineBoxFamilies::default(),
        )
    }

    fn input<'a>(
        prepared: &'a text::PreparedText,
        request: &'a text::LineSelectionRequest,
        resources: &'a ScopeAlignmentResources<f64>,
        funding: EntryFundingRef<'a>,
        formatted: &'a super::super::formatted::FormattedTailSnapshot,
    ) -> FitInput<'a> {
        FitInput {
            entry_funding: Some(funding),
            adjoining_displacement: 0.0,
            inline_boxes: None,
            groups: crate::group_alignment::Snapshot::default(),
            wrapping: None,
            formatted,
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                crate::Direction::Ltr,
            ),
            prepared,
            segment: request.segment.clone(),
            scope_alignments: resources,
            available: crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(100.0).unwrap()),
                crate::SizingOperation::Normal,
            ),
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            block_direction: text::ParagraphBaseDirection::LeftToRight,
            alignment: crate::TextAlign::Start,
            wrap_style: crate::TextWrapStyle::Auto,
            last_alignment: crate::TextAlignLast::Auto,
            justification: crate::TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: super::super::fit::TailContext::Unrestricted,
            orientation: text::LineOrientation::Horizontal,
            atomics: &request.atomics,
            controls: &request.controls,
            line_strut: request.line_strut,
            reusable: &[],
        }
    }

    fn exhaust(funding: &EntryFunding<f64>) {
        for _ in 0..=LIMITS.structural_work() {
            if let Err(error) = funding.visit(1) {
                assert!(matches!(
                    error.detail(),
                    Some(text::ErrorDetail::ScopeAlignment {
                        reason: text::ScopeAlignmentError::Limit {
                            kind: text::ScopeAlignmentLimitKind::StructuralWork,
                            ..
                        }
                    })
                ));
                return;
            }
        }
        panic!("real one-unit work admissions must exhaust the captured ceiling");
    }

    #[test]
    fn prospective_flow_clones_keep_actual_backing_without_refunding_work() {
        let (source, line) = source();
        let resources = owner(&source);
        let funding = EntryFunding::new(resources.clone(), source.binding.prepared.clone())
            .unwrap()
            .unwrap();
        let initial = resources.retention();
        let selected = line.selected_line();
        let original = selected.request();
        let formatted = super::super::formatted::FormattedTailSnapshot::default();
        let input = input(
            &source.binding.prepared,
            original,
            &resources,
            funding.as_ref(),
            &formatted,
        );
        let portion = input
            .prepared
            .word_views()
            .next()
            .unwrap()
            .portions()
            .unwrap()
            .remove(0);
        let mut request = original.clone();
        request.start = portion.start();
        request.end = portion.end();
        request.termination = text::PreparedTermination::Measurement(portion);
        request.context.end = text::LineEnd::Soft;
        let entry = RoleEntry {
            current_first: true,
            phase: super::super::fit::EntryPhase::Pending,
            query_start: &original.start,
            query_incoming: None,
            boxes: None,
            accepted: &[],
            preceding: None,
        };
        let basis = prospective(&input, &request, &entry)
            .unwrap_or_else(|_| panic!("actual first-word measurement flow must be observed"));
        assert!(
            basis.following,
            "the second authored X is actual remaining content"
        );
        assert_eq!(basis.query_start, original.start);
        basis
            .recipe
            .validate_observations(&basis.observations)
            .unwrap();
        let live = resources.retention();
        assert!(
            live.payload_bytes() > initial.payload_bytes(),
            "actual retained observation buffers must be leased"
        );
        exhaust(&funding);
        let retained = basis.clone();
        assert_eq!(retained, basis);
        assert_eq!(
            resources.retention(),
            live,
            "immutable clone shares actual backing"
        );
        drop(basis);
        assert_eq!(
            resources.retention(),
            live,
            "retained clone keeps the original lease"
        );
        drop(retained);
        assert_eq!(
            resources.retention(),
            initial,
            "last owner releases actual live backing"
        );
        assert!(
            funding.visit(1).is_err(),
            "dropping proof storage cannot refund completed work"
        );
        let error = prospective(&input, &request, &entry)
            .expect_err("another prospective producer must refuse the exhausted original owner");
        assert!(
            matches!(error, FitError::Text(error) if matches!(error.detail(),
            Some(text::ErrorDetail::ScopeAlignment { reason: text::ScopeAlignmentError::Limit {
                kind: text::ScopeAlignmentLimitKind::StructuralWork, ..
            } })))
        );
        assert_eq!(
            resources.retention(),
            initial,
            "refused construction exports no live scratch"
        );
    }

    #[test]
    fn mixed_native_first_roles_retain_the_admitted_operation_backing() {
        use crate::test_support::text::{PreparedParagraph, ahem_style};
        let captured =
            crate::test_support::inline_endpoint::Fixture::with_scope_limits("X", LIMITS);
        let native = PreparedParagraph::plain(
            101,
            text::Source::new("(A A"),
            text::Style {
                hanging_punctuation: text::HangingPunctuation::new(true, None, false),
                ..ahem_style(10.0, 10.0)
            },
        );
        let tree = captured
            .tree::<f64>(
                crate::InlineBoundaryEdgeOf::ZERO,
                crate::InlineBoundaryEdgeOf::ZERO,
                false,
                Some(100.0),
                false,
            )
            .children(200, [0, 100])
            .children(100, [101])
            .style(
                200,
                crate::NodeInputOf {
                    display: crate::Display::Block,
                    size: crate::Size::new(
                        crate::PreferredSizeOf::px(100.0),
                        crate::PreferredSizeOf::AUTO,
                    ),
                    ..Default::default()
                },
            )
            .style(
                100,
                crate::NodeInputOf {
                    display: crate::Display::Block,
                    size: crate::Size::new(
                        crate::PreferredSizeOf::px(100.0),
                        crate::PreferredSizeOf::AUTO,
                    ),
                    text_align: crate::TextAlign::Justify,
                    text_align_last: crate::TextAlignLast::Justify,
                    text_justification: crate::TextJustification::InterWord,
                    text_wrap_style: crate::TextWrapStyle::Balance,
                    ..Default::default()
                },
            )
            .input(101, native.text_input(101));
        let capture = crate::LayoutFlowInput::try_new(
            &tree,
            200,
            vec![captured.binding.clone(), native.binding(100)],
            crate::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = crate::compute_layout_with_flow(
            &capture,
            200,
            crate::LayoutRootRequestOf::standalone(
                sizing_request(),
                crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
                crate::Size::NONE,
            ),
        )
        .unwrap();
        let original = batch
            .final_inline_fragments()
            .iter()
            .find_map(|row| {
                let line = row.fragment().finalized_fragment().finalized_line();
                (line.prepared_text() == native.prepared).then(|| line.clone())
            })
            .expect("the mixed operation must produce the actual Native line");
        let selected = original.selected_line();
        let resources = ScopeAlignmentResources::new(
            ScopeAlignmentSnapshot::admit(vec![], capture.paragraphs(), false)
                .unwrap()
                .freeze(),
            crate::text_layout::fragmentation::InlineBoxFamilies::default(),
        );
        let funding = EntryFunding::new(resources.clone(), captured.binding.prepared.clone())
            .unwrap()
            .unwrap();
        let initial = resources.retention();
        let formatted = super::super::formatted::FormattedTailSnapshot::default();
        let mut input = input(
            &native.prepared,
            selected.request(),
            &resources,
            funding.as_ref(),
            &formatted,
        );
        input.alignment = crate::TextAlign::Justify;
        input.last_alignment = crate::TextAlignLast::Justify;
        input.justification = crate::TextJustification::InterWord;
        input.wrap_style = crate::TextWrapStyle::Balance;
        // Both real bindings were admitted above. This Native run has no float,
        // so its actual local inline interval is the supplied100px container.
        // It borrows the same mixed operation's captured owner, not a budget token.
        let lines = super::super::fit::fit(
            input,
            |_, _| {
                Ok(super::super::fit::Band {
                    start: 0.0,
                    end: 100.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            |position, _| {
                Ok(position
                    .minimum
                    .map_or(position.proposed, |minimum| minimum.max(position.proposed)))
            },
            |_, _, _, struts| {
                Ok(super::super::fit::LineAllocations {
                    line_box_struts: struts.to_vec(),
                    atomics: Vec::new(),
                    controls: Vec::new(),
                    line_strut: None,
                })
            },
            |_, _| Ok(None),
            super::super::fit::EncounterCallbacks {
                replay: |_: &text::FinalizedLine, _: super::super::fit::EncounterReplay| {
                    Ok(Vec::new())
                },
                accept: || Ok(()),
                reset: |_| Ok(()),
                checkpoint_group: || Ok(()),
            },
        )
        .unwrap_or_else(|_| panic!("the admitted mixed Native role producer must fit"));
        let roles = &lines.last().unwrap().role_uses;
        let used = roles
            .iter()
            .find(|used| used.usage.first())
            .expect("actual leading punctuation consumes its authentic First role");
        assert_eq!(used.request.context.formatted_line.first(), Some(true));
        assert!(
            !used.amounts.is_empty(),
            "actual inter-word justification retains its issued amounts"
        );
        assert_eq!(
            lines.last().unwrap().finalized.feedback(),
            original.feedback()
        );
        let live = resources.retention();
        assert!(live.payload_bytes() > initial.payload_bytes());
        exhaust(&funding);
        let retained = RoleUseRows::clone(roles);
        assert_eq!(
            resources.retention(),
            live,
            "funded role-row clone shares original storage"
        );
        drop(lines);
        assert!(
            resources.retention().payload_bytes() > initial.payload_bytes(),
            "retained role rows keep real nested backing"
        );
        assert!(
            retained.iter().any(|used| used.usage.first()
                && used.request.context.formatted_line.first() == Some(true))
        );
        drop(retained);
        assert_eq!(resources.retention(), initial);
        assert!(
            funding.visit(1).is_err(),
            "role disposal cannot refund attempted producer work"
        );
    }
}
