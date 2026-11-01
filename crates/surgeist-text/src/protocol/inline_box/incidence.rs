//! One canonical borrowed Source Event/Cut visitor for packets and fragments.
use super::*;
use crate::protocol::prepared::{PresentationUnitKind as K, SelectedRelationEvent};
use crate::transform::relation::{ComponentOwner, RelationEvent};

#[derive(Clone, Copy)]
pub(crate) enum SourceIncidence {
    Event(usize),
    Cut(usize),
}
impl SourceIncidence {
    pub(crate) fn position(self) -> usize {
        match self {
            Self::Event(at) | Self::Cut(at) => at,
        }
    }
    pub(crate) fn before_end(self, end: usize) -> bool {
        match self {
            Self::Event(at) => at < end,
            Self::Cut(at) => at <= end,
        }
    }
}
pub(crate) fn visit_unit_events(
    prepared: &crate::engine::PreparedData,
    presentation: &crate::protocol::prepared::SelectedPresentation,
    unit: usize,
    ledger: &mut Ledger,
    mut emit: impl FnMut(SourceIncidence, &mut Ledger) -> Result<()>,
) -> Result<()> {
    let analyzed = &prepared.analyzed;
    let paragraph = &analyzed.paragraph;
    let topology = paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
    let transform = &analyzed.transform;
    // Components are source-owned; no glyph hull or presentation interval grants
    // ownership. Generated operations retain their actual original site.
    for relation in &presentation.selected_relation.events
        [presentation.selected_relation.unit_events[unit].clone()]
    {
        account::work(ledger, 1)?;
        let SelectedRelationEvent::Base { event } = *relation else {
            continue;
        };
        let event = &transform.events[event];
        visit_base_event(prepared, *event, ledger, &mut emit)?;
    }
    if let K::Replacement { alternative, .. } = presentation.units[unit].kind {
        use crate::protocol::prepared::AlternativeDefinition as D;
        let event = match prepared.tables.alternatives[alternative].definition {
            D::Source { origin, .. } | D::Automatic { origin, .. } => {
                paragraph.captured_origin_event(origin)
            }
            D::Control { control, .. } => Some(topology.control_orders[control]),
        }
        .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?;
        emit(SourceIncidence::Event(event), ledger)?;
    }
    Ok(())
}

pub(crate) fn visit_original_row(
    prepared: &crate::engine::PreparedData,
    row: usize,
    ledger: &mut Ledger,
    emit: &mut impl FnMut(SourceIncidence, &mut Ledger) -> Result<()>,
) -> Result<()> {
    let transform = &prepared.analyzed.transform;
    for piece in &transform.pieces[transform.rows[row].pieces.clone()] {
        account::work(ledger, 1)?;
        emit(
            SourceIncidence::Event(
                piece
                    .origin
                    .captured_event()
                    .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?,
            ),
            ledger,
        )?;
    }
    Ok(())
}

pub(crate) fn visit_base_event(
    prepared: &crate::engine::PreparedData,
    event: RelationEvent,
    ledger: &mut Ledger,
    emit: &mut impl FnMut(SourceIncidence, &mut Ledger) -> Result<()>,
) -> Result<()> {
    let analyzed = &prepared.analyzed;
    let paragraph = &analyzed.paragraph;
    let topology = paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
    let transform = &analyzed.transform;
    match event {
        RelationEvent::Text { atom } => {
            let component = transform.component_for_atom(atom);
            match component.owner {
                ComponentOwner::Original(row) => visit_original_row(prepared, row, ledger, emit)?,
                ComponentOwner::Wbr(control) => emit(
                    SourceIncidence::Event(topology.control_orders[control]),
                    ledger,
                )?,
                ComponentOwner::Virtual(index) => emit(
                    SourceIncidence::Cut(transform.virtuals[index].capture_cut),
                    ledger,
                )?,
            }
        }
        RelationEvent::Removal { row, .. } => visit_original_row(prepared, row, ledger, emit)?,
        RelationEvent::GeneratedSuppression { component, .. } => {
            match transform.components[component].owner {
                ComponentOwner::Original(row) => visit_original_row(prepared, row, ledger, emit)?,
                ComponentOwner::Wbr(control) => emit(
                    SourceIncidence::Event(topology.control_orders[control]),
                    ledger,
                )?,
                ComponentOwner::Virtual(index) => emit(
                    SourceIncidence::Cut(transform.virtuals[index].capture_cut),
                    ledger,
                )?,
            }
        }
        RelationEvent::PhaseIDeleted { source, .. } => {
            paragraph.visit_origin_indices(source, |visit| {
                match visit {
                    crate::protocol::paragraph::OriginVisit::Inspect => account::work(ledger, 1)?,
                    crate::protocol::paragraph::OriginVisit::Intersection { origin, .. } => emit(
                        SourceIncidence::Event(
                            paragraph.captured_origin_event(origin).ok_or_else(|| {
                                invalid(InlineBoxEdgeError::UnrepresentedIncidence)
                            })?,
                        ),
                        ledger,
                    )?,
                }
                Ok(())
            })?
        }
        RelationEvent::Anchor { order } => {
            use crate::protocol::paragraph::SemanticAnchorItem as A;
            match paragraph.anchor_order[order].item {
                A::Control(control) => emit(
                    SourceIncidence::Event(topology.control_orders[control]),
                    ledger,
                )?,
                A::Atomic(atomic) => emit(
                    SourceIncidence::Event(topology.capture.atomic_orders[atomic]),
                    ledger,
                )?,
                A::EmptyOrigin(origin) => emit(
                    SourceIncidence::Event(
                        paragraph
                            .captured_origin_event(origin)
                            .ok_or_else(|| invalid(InlineBoxEdgeError::UnrepresentedIncidence))?,
                    ),
                    ledger,
                )?,
                A::ComputedLineBoundary(boundary) => {
                    for &cause in &paragraph.computed_boundaries[boundary].causes {
                        account::work(ledger, 1)?;
                        emit(
                            SourceIncidence::Cut(topology.capture.computed[cause].capture_cut),
                            ledger,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn visit_bidi_owner(
    prepared: &crate::engine::PreparedData,
    owner: crate::engine::analysis::BidiOwner,
    ledger: &mut Ledger,
    emit: &mut impl FnMut(SourceIncidence, &mut Ledger) -> Result<()>,
) -> Result<()> {
    use crate::engine::analysis::BidiOwner as B;
    let analyzed = &prepared.analyzed;
    let p = &analyzed.paragraph;
    let topology = p
        .scope_topology
        .as_ref()
        .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
    match owner {
        B::Atom(atom) => visit_base_event(prepared, RelationEvent::Text { atom }, ledger, emit),
        B::Atomic(i) => emit(
            SourceIncidence::Event(topology.capture.atomic_orders[i]),
            ledger,
        ),
        B::Control(i) => emit(SourceIncidence::Event(topology.control_orders[i]), ledger),
        B::ComputedLineBoundary(i) => {
            for &cause in &p.computed_boundaries[i].causes {
                account::work(ledger, 1)?;
                emit(
                    SourceIncidence::Cut(topology.capture.computed[cause].capture_cut),
                    ledger,
                )?;
            }
            Ok(())
        }
        B::CombinedCell(i) => {
            let binding = &analyzed
                .combination
                .as_ref()
                .expect("real Cell source")
                .bindings[i];
            for &event in &analyzed.transform.events[binding.events.clone()] {
                account::work(ledger, 1)?;
                visit_base_event(prepared, event, ledger, emit)?;
            }
            Ok(())
        }
    }
}
