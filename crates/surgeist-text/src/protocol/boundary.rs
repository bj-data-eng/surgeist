//! Selection-local endpoint incidence, consumed context and numeric authority.
use super::paragraph::{ParagraphAnchor, ParagraphAnchorItem, ParagraphControlKind};
use super::prepared::{
    PresentationUnitKind, RetainedAssignment, SelectedPresentation, presentation::Attachment,
};
use super::{
    LineSelectionRequest, PreparedControl, PreparedText, ProtocolError, ProtocolHandleKind,
    protocol_error,
};
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
    positive: Vec<Incidence>,
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
            let Event::Anchor(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(occurrence),
                ..
            }) = event.kind
            else {
                return None;
            };
            if !is_endpoint(&prepared.analyzed.paragraph.controls[occurrence].kind) {
                return None;
            }
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
                })
        })
        .collect()
}

pub(crate) fn advance(selected: &super::selection::SelectedData, unit: usize) -> f64 {
    let PresentationUnitKind::Control { occurrence } = selected.presentation.units[unit].kind
    else {
        return 0.0;
    };
    selected
        .request
        .controls
        .iter()
        .find(|row| {
            selected.prepared.tables.controls[row.occurrence.index].occurrence == occurrence
        })
        .expect("complete admitted control inventory")
        .boundary
        .map_or(0.0, |edge| edge.advance())
}

pub(crate) fn groups(
    selected: &super::selection::SelectedData,
    shaped: &crate::engine::layout::data::ShapedData,
) -> Vec<Range<usize>> {
    let count = selected.presentation.projection.atoms.len();
    let mut ends: Vec<_> = (1..=count).collect();
    for range in selected.presentation.analysis.graphemes.iter().chain(
        shaped
            .clusters
            .iter()
            .map(|cluster| &cluster.provider_group),
    ) {
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
            end = end.max(ends[atom]);
            atom += 1;
        }
        result.push(start..end);
    }
    result
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
    let (participant, atom, preceding) = match domain.attachment {
        Attachment::Before { participant, atom } => (participant, atom, true),
        Attachment::After { participant, atom } => (participant, atom, false),
    };
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
            for incidence in &self.positive {
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
    presentation: &SelectedPresentation,
    incidence: Incidence,
    selected_start: usize,
) -> Option<usize> {
    let mut cut = 0;
    for (index, unit) in presentation.units.iter().enumerate() {
        let preceding = if let Some(participant) = unit.participant {
            selected_start + participant < incidence.participant
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
            participant < incidence.participant
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
    request: &LineSelectionRequest,
    presentation: &mut SelectedPresentation,
) -> Result<(Resolved, Option<PreparedControl>)> {
    let data = &prepared.inner;
    let mut known: BTreeMap<usize, f64> = request
        .controls
        .iter()
        .filter_map(|row| {
            row.boundary
                .map(|value| (row.occurrence.index, value.advance()))
        })
        .collect();
    let mut supplied = BTreeSet::new();
    for row in &request.boundary_context {
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
    let mut stops: Vec<_> = crate::transform::separators::connection_seams(
        &data.analyzed.projection,
        &data.analyzed.logical.events,
        &data.analyzed.paragraph,
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
            Event::Anchor(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(control),
                ..
            }) => {
                data.analyzed.paragraph.controls[control].kind == ParagraphControlKind::ForcedBreak
            }
            Event::Anchor(ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(atomic),
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
        positive: Vec::new(),
        stops,
    };
    let selected_start = data.tables.boundaries[request.start.index].participant;
    for incidence in &incidences {
        if known
            .get(&incidence.control)
            .is_some_and(|&value| value > 0.0)
        {
            result.positive.push(*incidence);
            result.original.insert(incidence.atom);
            if let Some(cut) = presentation_cut(presentation, *incidence, selected_start) {
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
                        let value = row.boundary.advance();
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
                if value > 0.0 {
                    if !result
                        .positive
                        .iter()
                        .any(|known| known.control == incidence.control)
                    {
                        result.positive.push(incidence);
                    }
                    result.original.insert(incidence.atom);
                    if let Some(cut) = presentation_cut(presentation, incidence, selected_start) {
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
                || incidence.participant >= data.tables.boundaries[request.end.index].participant
            {
                continue;
            }
            if let Some(cut) = presentation_cut(presentation, *incidence, selected_start)
                && assignment.atoms.start <= cut
                && cut <= assignment.atoms.end
                && known
                    .get(&incidence.control)
                    .is_some_and(|&value| value > 0.0)
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
