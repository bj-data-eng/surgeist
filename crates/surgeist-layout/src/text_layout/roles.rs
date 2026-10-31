//! Candidate-local formatted roles and nonplaceable measurement flow evidence.

use super::fit::{EncounterBasis, FitError, FitInput, FittedLine, LineBands};
use super::formatted::FormattedTailAssignment;
use surgeist_text as text;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RoleUse {
    pub(crate) disposition: super::fit::LineDisposition,
    pub(crate) request: text::LineSelectionRequest,
    pub(crate) boxes: Option<super::boxes::BoxAssignment>,
    pub(crate) amounts: Vec<f64>,
    pub(crate) usage: text::FormattedLineUsage,
    pub(crate) outgoing: Option<text::HyphenationContinuation>,
    pub(crate) basis: RoleBasis,
    pub(crate) bands: LineBands,
    pub(crate) encounters: Vec<EncounterBasis>,
    pub(crate) logical_start: f64,
    pub(crate) source_floor: f64,
    pub(crate) minimum_block: Option<f64>,
    pub(crate) deferred_from: Option<text::LineSelectionRequest>,
    pub(crate) placed_start: f64,
}

impl RoleUse {
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RoleBasis {
    Actual {
        first: bool,
        assignment: Option<FormattedTailAssignment>,
    },
    Prospective(Box<ProspectiveRoleBasis>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProspectiveRoleBasis {
    recipe: text::PreparedMeasurementFlow,
    observations: Vec<text::FlowObservation>,
    boxes: Vec<Option<super::boxes::BoxAssignment>>,
    query_start: text::PreparedBoundary,
    query_incoming: Option<text::HyphenationContinuation>,
    checkpoint: EntryCheckpoint,
    prior: bool,
    following: bool,
    external: Option<FormattedTailAssignment>,
}

#[derive(Clone, Debug, PartialEq)]
struct EntryCheckpoint {
    entry: text::PreparedBoundary,
    first: bool,
    outside: bool,
    box_consumed: bool,
    receipts: Vec<CheckpointReceipt>,
}

#[derive(Clone, Debug)]
struct CheckpointReceipt {
    line: text::FinalizedLine,
    amounts: Vec<f64>,
    boxes: Option<super::boxes::BoxAssignment>,
}

impl PartialEq for CheckpointReceipt {
    fn eq(&self, other: &Self) -> bool {
        // Finalization is deterministic from these admitted inputs. Retain the
        // authentic receipt for observation without comparing its allocation.
        self.line.selected_line().request() == other.line.selected_line().request()
            && self.amounts == other.amounts
            && self.line.scope_allocations() == other.line.scope_allocations()
            && self.boxes == other.boxes
    }
}

pub(super) struct RoleEntry<'a> {
    pub current_first: bool,
    pub query_start: &'a text::PreparedBoundary,
    pub query_incoming: Option<&'a text::HyphenationContinuation>,
    pub boxes: Option<&'a super::boxes::BoxAssignment>,
    pub accepted: &'a [FittedLine],
    pub preceding: Option<(
        &'a text::FinalizedLine,
        &'a [f64],
        Option<&'a super::boxes::BoxAssignment>,
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
            *basis = Some(RoleBasis::Prospective(Box::new(prospective(
                input, request, entry,
            )?)));
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
    )?;
    let mut observations = Vec::new();
    let mut box_assignments: Vec<Option<super::boxes::BoxAssignment>> = Vec::new();
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
        let boxes = if segment.role() == text::FlowSegmentRole::Target {
            entry.boxes.cloned()
        } else if let Some(resolver) = input.wrapping {
            let controls: Vec<_> = input.controls.iter().filter(|allocation| segment.participants().any(|participant| matches!(participant, text::PreparedParticipant::Control(control) if control == allocation.occurrence))).cloned().collect();
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
        observations.push(observation);
    }
    recipe.validate_observations(&observations)?;
    let checkpoint = if phrase {
        EntryCheckpoint {
            entry: recipe.entry(),
            first: entry.current_first,
            outside: false,
            box_consumed: false,
            receipts: Vec::new(),
        }
    } else {
        checkpoint(input, &recipe.entry(), entry, outside)?
    };
    Ok(ProspectiveRoleBasis {
        recipe,
        observations,
        boxes: box_assignments,
        query_start: entry.query_start.clone(),
        query_incoming: entry.query_incoming.cloned(),
        checkpoint,
        prior,
        following,
        external: None,
    })
}

fn checkpoint(
    input: &FitInput<'_>,
    entry_boundary: &text::PreparedBoundary,
    entry: &RoleEntry<'_>,
    observed_outside: bool,
) -> Result<EntryCheckpoint, FitError> {
    let at = entry_boundary.view().participant_offset();
    let mut receipts = Vec::new();
    for (line, amounts, first, boxes) in entry
        .accepted
        .iter()
        .map(|line| {
            (
                &line.finalized,
                line.amounts.as_slice(),
                line.first_formatted,
                line.boxes.as_ref(),
            )
        })
        .chain(
            entry
                .preceding
                .map(|(line, amounts, boxes)| (line, amounts, entry.current_first, boxes)),
        )
    {
        let selected = line.selected_line();
        let request = selected.request();
        let from = request.start.view().participant_offset();
        let to = request.end.view().participant_offset();
        if to <= at {
            receipts.push(CheckpointReceipt {
                line: line.clone(),
                amounts: amounts.to_vec(),
                boxes: boxes.cloned(),
            });
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
            receipts.push(CheckpointReceipt {
                line: line.clone(),
                amounts: amounts.to_vec(),
                boxes: boxes.cloned(),
            });
            return Ok(EntryCheckpoint {
                entry: entry_boundary.clone(),
                first,
                outside: outside || observed_outside,
                box_consumed,
                receipts,
            });
        }
    }
    // The recipe's EntryPrefix observations supply an unaccepted entry. Any
    // real accepted line before that entry is outside the replaced word branch.
    let has_formatted = receipts.iter().try_fold(false, |seen, receipt| {
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
    Ok(EntryCheckpoint {
        entry: entry_boundary.clone(),
        first,
        outside: observed_outside,
        box_consumed,
        receipts,
    })
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
    let participants: Vec<_> = segment.participants().collect();
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
        atomics: input.atomics.iter().filter(|row| participants.iter().any(|p| matches!(p,text::PreparedParticipant::Atomic(value) if value==&row.occurrence))).map(|row| target.atomics.iter().find(|actual|actual.occurrence==row.occurrence).unwrap_or(row).clone()).collect(),
        controls: input.controls.iter().filter(|row| participants.iter().any(|p| matches!(p,text::PreparedParticipant::Control(value) if value==&row.occurrence))).map(|row| target.controls.iter().find(|actual|actual.occurrence==row.occurrence).unwrap_or(row).clone()).collect(),
        line_strut: target.line_strut,
        line_box_struts: if segment.role() == text::FlowSegmentRole::Target { target.line_box_struts.clone() } else { boxes.map_or_else(Vec::new, super::boxes::BoxAssignment::struts) },
        boundary_context: Vec::new(),
    };
    while let Some(occurrence) = input
        .prepared
        .flow_boundary_context_demand(segment, &request)?
    {
        let boundary = input
            .controls
            .iter()
            .find(|row| row.occurrence == occurrence)
            .and_then(|row| row.boundary)
            .ok_or_else(invalid_flow)?;
        if request
            .boundary_context
            .iter()
            .any(|row| row.occurrence == occurrence)
        {
            return Err(invalid_flow());
        }
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
