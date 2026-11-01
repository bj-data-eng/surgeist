//! Selection-local endpoint incidence, consumed context and numeric authority.
use super::paragraph::{ParagraphControlKind, SemanticAnchor, SemanticAnchorItem};
use super::prepared::{
    ParticipantData, PresentationUnitKind, RetainedAssignment, SelectedPresentation,
    presentation::Attachment,
};
use super::{PreparedControl, PreparedText, ProtocolError, ProtocolHandleKind, protocol_error};
use crate::{
    Result,
    engine::{PreparedData, analysis::wrapping::Event},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    sync::Arc,
};

pub(crate) fn is_endpoint(kind: &ParagraphControlKind) -> bool {
    matches!(
        kind,
        ParagraphControlKind::InlineBoundaryStart | ParagraphControlKind::InlineBoundaryEnd
    )
}

pub(crate) struct Resolved {
    pub(crate) original: BTreeSet<usize>,
    pub(crate) presentation: BTreeSet<usize>,
    separating: Vec<Incidence>,
    stops: Vec<ContextStop>,
}

#[derive(Clone, Copy)]
struct ContextStop {
    before: usize,
    after: usize,
    atom: usize,
}

#[derive(Clone, Copy)]
struct Incidence {
    control: usize,
    atom: usize,
    participant: usize,
    /// Exact transform event, retaining order around raw-empty cells.
    relation: usize,
}

fn incidences(prepared: &PreparedData) -> Vec<Incidence> {
    let projection = &prepared.analyzed.projection;
    prepared
        .analyzed
        .logical
        .events
        .iter()
        .enumerate()
        .filter_map(|(participant, event)| {
            let Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::Control(occurrence),
                ..
            }) = event.kind
            else {
                return None;
            };
            if !is_endpoint(&prepared.analyzed.paragraph.controls[occurrence].kind) {
                return None;
            }
            let control = prepared.tables.control_rows[occurrence];
            let order = prepared.tables.controls[control].anchor;
            let relation = prepared.analyzed.transform.events().iter().position(|event| {
                matches!(*event, crate::transform::relation::RelationEvent::Anchor { order: actual } if actual == order)
            })?;
            let position = projection
                .atoms
                .get(event.atom)
                .map_or(projection.presentation_end(), |a| a.presentation.start);
            projection
                .graphemes
                .binary_search(&position)
                .is_ok()
                .then_some(Incidence {
                    control: prepared.tables.control_rows[occurrence],
                    atom: event.atom,
                    participant,
                    relation,
                })
        })
        .collect()
}

fn control_allocation<'a>(
    selected: super::input::PresentationInput<'a>,
    unit: usize,
) -> Option<&'a super::ControlAllocation> {
    let PresentationUnitKind::Control { occurrence } = selected.presentation.units[unit].kind
    else {
        return None;
    };
    Some(
        selected
            .controls
            .iter()
            .find(|row| {
                selected.prepared.tables.controls[row.occurrence.index].occurrence == occurrence
            })
            .expect("complete admitted control inventory"),
    )
}
pub(crate) fn allocation<'a>(
    selected: super::input::PresentationInput<'a>,
    unit: usize,
) -> Option<&'a super::InlineBoundaryAllocation> {
    control_allocation(selected, unit)?.boundary.as_ref()
}
pub(crate) fn advance(selected: super::input::PresentationInput<'_>, unit: usize) -> f64 {
    allocation(selected, unit).map_or(0.0, super::InlineBoundaryAllocation::advance)
}
pub(crate) fn component_side(
    selected: super::input::PresentationInput<'_>,
    unit: usize,
) -> Result<Option<super::LineEdgeSide>> {
    let Some(context) =
        allocation(selected, unit).and_then(super::InlineBoundaryAllocation::box_context)
    else {
        return Ok(None);
    };
    let PresentationUnitKind::Control { occurrence } = selected.presentation.units[unit].kind
    else {
        return Err(super::inline_box::invalid(
            super::InlineBoxEdgeError::InvalidOwner,
        ));
    };
    match selected.prepared.analyzed.paragraph.controls[occurrence].kind {
        ParagraphControlKind::InlineBoundaryStart => Ok(Some(context.start_side())),
        ParagraphControlKind::InlineBoundaryEnd => Ok(Some(match context.start_side() {
            super::LineEdgeSide::Start => super::LineEdgeSide::End,
            super::LineEdgeSide::End => super::LineEdgeSide::Start,
        })),
        _ => Err(super::inline_box::invalid(
            super::InlineBoxEdgeError::InvalidOwner,
        )),
    }
}
pub(crate) fn execute(
    selected: super::input::PresentationInput<'_>,
    unit: usize,
    pen: f64,
    phase: crate::TextNumericPhase,
) -> Result<super::inline_box::EdgeExecution> {
    let Some(control) = control_allocation(selected, unit) else {
        return Ok(super::inline_box::EdgeExecution {
            flow: (pen, pen),
            strip: (pen, pen),
        });
    };
    let Some(edge) = control.boundary.as_ref() else {
        return Ok(super::inline_box::EdgeExecution {
            flow: (pen, pen),
            strip: (pen, pen),
        });
    };
    if let Some(components) = edge.box_components() {
        let side =
            component_side(selected, unit)?.expect("a component edge retains its checked context");
        components.evaluate(pen, side, phase)
    } else {
        let end = crate::engine::layout::add("inline boundary end", phase, pen, edge.advance())?;
        Ok(super::inline_box::EdgeExecution {
            flow: (pen, end),
            strip: (pen, end),
        })
    }
}

pub(crate) fn groups_accounted(
    selected: &super::selection::SelectedData,
    shaped: &crate::engine::layout::data::ShapedData,
    ledger: &mut crate::engine::scope_alignment::accounting::Ledger,
) -> Result<Vec<Range<usize>>> {
    use crate::engine::scope_alignment::accounting as account;
    let count = selected.presentation.projection.atoms.len();
    let mut ends = account::collect_fallible(ledger, 1..=count)?;
    for unit in &selected.presentation.units {
        account::work(ledger, 1)?;
        if matches!(unit.kind, PresentationUnitKind::Cell { .. }) && !unit.atoms.is_empty() {
            ends[unit.atoms.start] = ends[unit.atoms.start].max(unit.atoms.end);
        }
    }
    for range in selected.presentation.analysis.graphemes.iter().chain(
        shaped
            .clusters
            .iter()
            .map(|cluster| &cluster.provider_group),
    ) {
        account::work(ledger, 1)?;
        if !range.is_empty() {
            ends[range.start] = ends[range.start].max(range.end);
        }
    }
    let mut result = Vec::new();
    let mut atom = 0;
    while atom < count {
        let start = atom;
        let mut end = ends[atom];
        while atom < end {
            account::work(ledger, 1)?;
            end = end.max(ends[atom]);
            atom += 1;
        }
        account::push_fallible(ledger, &mut result, start..end)?;
    }
    account::drop_vec(ledger, ends)?;
    Ok(result)
}

pub(crate) struct Seed {
    pub(crate) atoms: Range<usize>,
    pub(crate) before: usize,
    pub(crate) after: usize,
    pub(crate) preceding: bool,
}

pub(crate) fn original_seed(
    prepared: &PreparedData,
    presentation: &SelectedPresentation,
    assignment: &RetainedAssignment,
) -> Result<Seed> {
    let originals: Vec<_> = presentation.originals[assignment.atoms.clone()]
        .iter()
        .flatten()
        .copied()
        .collect();
    if let (Some(&first), Some(&last)) = (originals.first(), originals.last()) {
        let before = prepared
            .analyzed
            .logical
            .events
            .iter()
            .position(|event| matches!(event.kind, Event::Text(atom) if atom == first))
            .expect("mapped original event");
        let after = prepared
            .analyzed
            .logical
            .events
            .iter()
            .rposition(|event| matches!(event.kind, Event::Text(atom) if atom == last))
            .expect("mapped original event")
            + 1;
        return Ok(Seed {
            atoms: first..last + 1,
            before,
            after,
            preceding: false,
        });
    }
    let domain = presentation
        .domains
        .iter()
        .find(|domain| {
            presentation.units[domain.units.clone()].iter().any(|unit| {
                unit.atoms.start <= assignment.atoms.start
                    && assignment.atoms.start < unit.atoms.end
            })
        })
        .ok_or_else(|| protocol_error(ProtocolError::InvalidWordPortion))?;
    if let Attachment::After {
        participant,
        atom: None,
    } = domain.attachment
        && let ParticipantData::Atomic(atomic) = prepared.tables.participants[participant]
    {
        // Prepared participants and original logical events are distinct index
        // spaces. This actual occurrence, not an adjacent scalar or equal source
        // offset, supplies the atomic's zero-width joining seed and barrier.
        let occurrence = prepared.tables.atomics[atomic].occurrence;
        let event = prepared
            .analyzed
            .logical
            .events
            .iter()
            .position(|event| {
                matches!(event.kind, Event::Anchor(SemanticAnchor {
                    item: SemanticAnchorItem::Atomic(actual),
                    ..
                }) if actual == occurrence)
            })
            .ok_or_else(|| protocol_error(ProtocolError::InvalidWordPortion))?;
        let atom = prepared.analyzed.logical.events[event].atom;
        return Ok(Seed {
            atoms: atom..atom,
            before: event,
            after: event + 1,
            preceding: false,
        });
    }
    let (participant, atom, preceding) = match domain.attachment {
        Attachment::Before { participant, atom } => (participant, atom, true),
        Attachment::After { participant, atom } => (participant, atom, false),
    };
    if let ParticipantData::Cell(cell) = prepared.tables.participants[participant] {
        let output = prepared
            .analyzed
            .combination
            .as_deref()
            .ok_or_else(|| protocol_error(ProtocolError::InvalidWordPortion))?;
        let binding = &output.bindings[prepared.tables.cells[cell].binding];
        return Ok(Seed {
            atoms: binding.atoms.end..binding.atoms.end,
            before: binding.participants.end,
            after: binding.participants.end,
            preceding,
        });
    }
    let participant = prepared
        .tables
        .source_prefix(participant)
        .ok_or_else(|| protocol_error(ProtocolError::InvalidWordPortion))?;
    let atoms = atom.map_or_else(
        || {
            let atom = prepared.analyzed.logical.events[participant].atom;
            atom..atom
        },
        |atom| atom..atom + 1,
    );
    Ok(Seed {
        atoms,
        before: participant,
        after: participant + 1,
        preceding,
    })
}

impl Resolved {
    pub(crate) fn original_extent(&self, prepared: &PreparedData, seed: &Seed) -> Range<usize> {
        let mut result = extent(
            seed,
            prepared.analyzed.projection.atoms.len(),
            &self.original,
        );
        if seed.atoms.is_empty() {
            let cut = seed.atoms.start;
            result.start = self.original.range(..cut).next_back().copied().unwrap_or(0);
            result.end = self
                .original
                .range((std::ops::Bound::Excluded(cut), std::ops::Bound::Unbounded))
                .next()
                .copied()
                .unwrap_or(prepared.analyzed.projection.atoms.len());
            for incidence in &self.separating {
                if incidence.atom == cut {
                    if incidence.participant < seed.before {
                        result.start = cut;
                    }
                    if incidence.participant >= seed.after {
                        result.end = cut;
                    }
                }
            }
            for stop in &self.stops {
                if stop.atom == cut {
                    if stop.after <= seed.before {
                        result.start = cut;
                    }
                    if stop.before >= seed.after {
                        result.end = cut;
                    }
                    // An unresolved connection interval does not prove either
                    // incident original domain for an interior control anchor.
                    if stop.before < seed.after && seed.before < stop.after {
                        result.start = cut;
                        result.end = cut;
                    }
                }
            }
        }
        result
    }
}

pub(crate) fn extent(seed: &Seed, length: usize, barriers: &BTreeSet<usize>) -> Range<usize> {
    let mut start = barriers
        .range(..=seed.atoms.start)
        .next_back()
        .copied()
        .unwrap_or(0);
    let mut end = barriers
        .range(seed.atoms.end..)
        .next()
        .copied()
        .unwrap_or(length);
    if seed.atoms.is_empty() && barriers.contains(&seed.atoms.start) {
        if seed.preceding {
            start = barriers
                .range(..seed.atoms.start)
                .next_back()
                .copied()
                .unwrap_or(0);
        } else {
            end = barriers
                .range((
                    std::ops::Bound::Excluded(seed.atoms.end),
                    std::ops::Bound::Unbounded,
                ))
                .next()
                .copied()
                .unwrap_or(length);
        }
    }
    start..end
}

fn presentation_cut(
    prepared: &PreparedData,
    presentation: &SelectedPresentation,
    incidence: Incidence,
    selected_start: usize,
) -> Option<usize> {
    let mut cut = 0;
    for (index, unit) in presentation.units.iter().enumerate() {
        let preceding = if let Some(participant) = unit.participant {
            let outer = selected_start + participant;
            if let PresentationUnitKind::Cell { cell } = unit.kind {
                let output = prepared.analyzed.combination.as_deref()?;
                let binding = &output.bindings[prepared.shaped.cells[cell].binding];
                if incidence.relation < binding.events.start {
                    false
                } else if incidence.relation >= binding.events.end {
                    true
                } else {
                    return None;
                }
            } else {
                prepared.tables.source_prefix(outer)? < incidence.participant
            }
        } else {
            let attachment = presentation
                .domains
                .iter()
                .find(|d| d.units.contains(&index))?
                .attachment;
            let participant = match attachment {
                Attachment::Before { participant, .. } | Attachment::After { participant, .. } => {
                    participant
                }
            };
            if let ParticipantData::Cell(cell) = prepared.tables.participants[participant] {
                let output = prepared.analyzed.combination.as_deref()?;
                output.bindings[prepared.tables.cells[cell].binding]
                    .events
                    .end
                    <= incidence.relation
            } else {
                prepared.tables.source_prefix(participant)? < incidence.participant
            }
        };
        if !preceding {
            break;
        }
        cut = unit.atoms.end;
    }
    (cut == 0
        || cut == presentation.projection.atoms.len()
        || presentation
            .analysis
            .graphemes
            .iter()
            .any(|g| g.start == cut))
    .then_some(cut)
}

fn split_assignments(presentation: &mut SelectedPresentation, cuts: &BTreeSet<usize>) {
    let old = std::mem::take(&mut presentation.assignments);
    for assignment in old {
        let mut start = assignment.atoms.start;
        for end in cuts
            .range((
                std::ops::Bound::Excluded(start),
                std::ops::Bound::Excluded(assignment.atoms.end),
            ))
            .copied()
            .chain(std::iter::once(assignment.atoms.end))
        {
            let mut run = assignment.run.clone();
            run.atoms = start..end;
            presentation.assignments.push(RetainedAssignment {
                atoms: start..end,
                run,
            });
            start = end;
        }
    }
}

pub(crate) fn resolve(
    prepared: &PreparedText,
    request: super::input::RequestInput<'_>,
    presentation: &mut SelectedPresentation,
) -> Result<(Resolved, Option<PreparedControl>)> {
    let data = &prepared.inner;
    let mut known: BTreeMap<usize, &super::InlineBoundaryAllocation> = request
        .controls
        .iter()
        .filter_map(|row| {
            row.boundary
                .as_ref()
                .map(|value| (row.occurrence.index, value))
        })
        .collect();
    let mut supplied = BTreeSet::new();
    for row in request.boundary_context {
        super::selection::require_prepared(
            prepared,
            &row.occurrence.prepared,
            ProtocolHandleKind::Control,
            row.occurrence.index,
        )?;
        if request
            .controls
            .iter()
            .any(|selected| selected.occurrence.index == row.occurrence.index)
            || !supplied.insert(row.occurrence.index)
        {
            return Err(protocol_error(ProtocolError::DuplicateBoundaryContext {
                occurrence: row.occurrence.index,
            }));
        }
        if data
            .tables
            .controls
            .get(row.occurrence.index)
            .is_none_or(|c| !is_endpoint(&data.analyzed.paragraph.controls[c.occurrence].kind))
        {
            return Err(protocol_error(ProtocolError::UnexpectedBoundaryContext {
                occurrence: row.occurrence.index,
            }));
        }
    }
    let incidences = incidences(data);
    let source_context = crate::transform::context::SourceContext::exact(
        &data.analyzed.processed,
        &data.analyzed.paragraph,
    )?;
    let mut stops: Vec<_> = crate::transform::separators::connection_seams(
        &data.analyzed.projection,
        &data.analyzed.logical.events,
        &source_context,
    )?
    .into_iter()
    .map(|seam| ContextStop {
        before: seam.lower,
        after: seam.upper,
        atom: seam.atom,
    })
    .collect();
    for (participant, event) in data.analyzed.logical.events.iter().enumerate() {
        let barrier = match event.kind {
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::ComputedLineBoundary(_),
                ..
            }) => true,
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::Control(control),
                ..
            }) => {
                data.analyzed.paragraph.controls[control].kind == ParagraphControlKind::ForcedBreak
            }
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::Atomic(atomic),
                ..
            }) => {
                data.analyzed.processed.source().boxes()[atomic].kind()
                    == crate::InlineBoxKind::InFlow
            }
            _ => false,
        };
        if barrier {
            stops.push(ContextStop {
                before: participant,
                after: participant + 1,
                atom: event.atom,
            });
        }
    }
    let mut result = Resolved {
        original: data
            .analyzed
            .projection
            .barrier_atoms
            .iter()
            .copied()
            .collect(),
        presentation: presentation
            .projection
            .barrier_atoms
            .iter()
            .copied()
            .collect(),
        separating: Vec::new(),
        stops,
    };
    if let Some(output) = data.analyzed.combination.as_deref() {
        for cell in &data.tables.cells {
            let binding = &output.bindings[cell.binding];
            for atom in [binding.atoms.start, binding.atoms.end] {
                result.original.insert(atom);
                result.stops.push(ContextStop {
                    before: binding.participants.start,
                    after: binding.participants.end,
                    atom,
                });
            }
        }
    }
    for unit in &presentation.units {
        if matches!(unit.kind, PresentationUnitKind::Cell { .. }) {
            result.presentation.insert(unit.atoms.start);
            result.presentation.insert(unit.atoms.end);
        }
    }
    let selected_outer_start = data.tables.boundaries[request.start.index].participant;
    let selected_start = data.tables.boundaries[request.start.index].source_participant;
    for incidence in &incidences {
        if known
            .get(&incidence.control)
            .is_some_and(|&value| value.participates())
        {
            result.separating.push(*incidence);
            result.original.insert(incidence.atom);
            if let Some(cut) =
                presentation_cut(data, presentation, *incidence, selected_outer_start)
            {
                result.presentation.insert(cut);
            }
        }
    }
    split_assignments(presentation, &result.presentation);
    let mut next = 0;
    for assignment in &presentation.assignments {
        let seed = original_seed(data, presentation, assignment)?;
        let intrinsic: BTreeSet<_> = data
            .analyzed
            .projection
            .barrier_atoms
            .iter()
            .copied()
            .collect();
        let bounds = extent(&seed, data.analyzed.projection.atoms.len(), &intrinsic);
        let preceding = result
            .stops
            .iter()
            .filter(|stop| stop.before <= seed.before)
            .map(|stop| stop.after.min(seed.before))
            .max()
            .unwrap_or(0);
        let following = result
            .stops
            .iter()
            .filter(|stop| stop.after >= seed.after)
            .map(|stop| stop.before.max(seed.after))
            .min()
            .unwrap_or(data.analyzed.logical.events.len());
        let walks: Vec<Vec<Incidence>> = vec![
            incidences
                .iter()
                .copied()
                .filter(|i| {
                    i.participant >= seed.before
                        && i.participant < seed.after
                        && i.atom >= seed.atoms.start
                        && i.atom <= seed.atoms.end
                })
                .collect(),
            incidences
                .iter()
                .rev()
                .copied()
                .filter(|i| {
                    i.participant < seed.before
                        && i.participant >= preceding
                        && i.atom >= bounds.start
                        && i.atom <= seed.atoms.start
                })
                .collect(),
            incidences
                .iter()
                .copied()
                .filter(|i| {
                    i.participant >= seed.after
                        && i.participant < following
                        && i.atom <= bounds.end
                        && i.atom >= seed.atoms.end
                })
                .collect(),
        ];
        for (side, walk) in walks.into_iter().enumerate() {
            for incidence in walk {
                let value = if let Some(&value) = known.get(&incidence.control) {
                    value
                } else {
                    if let Some(row) = request.boundary_context.get(next) {
                        if row.occurrence.index != incidence.control {
                            return Err(protocol_error(ProtocolError::UnexpectedBoundaryContext {
                                occurrence: row.occurrence.index,
                            }));
                        }
                        let value = &row.boundary;
                        known.insert(incidence.control, value);
                        next += 1;
                        value
                    } else {
                        return Ok((
                            result,
                            Some(PreparedControl {
                                prepared: Arc::clone(data),
                                index: incidence.control,
                            }),
                        ));
                    }
                };
                if value.participates() {
                    if !result
                        .separating
                        .iter()
                        .any(|known| known.control == incidence.control)
                    {
                        result.separating.push(incidence);
                    }
                    result.original.insert(incidence.atom);
                    if let Some(cut) =
                        presentation_cut(data, presentation, incidence, selected_outer_start)
                    {
                        result.presentation.insert(cut);
                    }
                    if side != 0 {
                        break;
                    }
                }
            }
        }
        // Presentation contexts cannot extend beyond the selected projection.
        // Their endpoint values are therefore selected facts, already admitted.
        // Replay them after this assignment's original context, transporting
        // only genuine event cuts, never interior expansion/grapheme cuts.
        for incidence in &incidences {
            if incidence.participant < selected_start
                || incidence.participant
                    >= data.tables.boundaries[request.end.index].source_participant
            {
                continue;
            }
            if let Some(cut) =
                presentation_cut(data, presentation, *incidence, selected_outer_start)
                && assignment.atoms.start <= cut
                && cut <= assignment.atoms.end
                && known
                    .get(&incidence.control)
                    .is_some_and(|&value| value.participates())
            {
                result.presentation.insert(cut);
            }
        }
    }
    if let Some(row) = request.boundary_context.get(next) {
        return Err(protocol_error(ProtocolError::UnexpectedBoundaryContext {
            occurrence: row.occurrence.index,
        }));
    }
    split_assignments(presentation, &result.presentation);
    Ok((result, None))
}

#[cfg(test)]
mod tests;
