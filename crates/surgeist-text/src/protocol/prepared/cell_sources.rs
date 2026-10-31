//! Selected-local source correspondence for an actual opaque Cell endpoint.
use super::{PresentationUnitKind, SelectedPresentation, SelectedRelationEvent};
use crate::engine::{
    PreparedData,
    scope_alignment::accounting::{self as account, Ledger},
};
use crate::protocol::paragraph::{
    FragmentAssociation, FragmentOwner, OriginIntersection, SemanticAnchorItem,
};
use crate::protocol::selection::SpacingSlotPieceData;
use crate::protocol::{PreparedSourceSegment, PresentationRange, ProtocolError, protocol_error};
use crate::transform::combined::{CellBinding, FixedGroup, Part};
use crate::transform::relation::{ComponentOwner, GeneratedSeparatorOwner, RelationEvent};
use crate::{Range, Result, SpacingPiece};

pub(crate) fn visit_original_part(
    prepared: &PreparedData,
    part: &Part,
    ledger: &mut Ledger,
    mut emit: impl FnMut(FragmentAssociation, &mut Ledger) -> Result<()>,
) -> Result<()> {
    for piece in &part.pieces {
        account::work(ledger, 1)?;
        let origin = prepared
            .analyzed
            .paragraph
            .origin_map
            .visit_view_origin(piece.origin, || account::work(ledger, 1))?
            .ok_or_else(|| {
                crate::processing::contextual::invalid(
                    crate::SourceProcessingError::OriginCorrespondence,
                )
            })?
            .get();
        emit(
            FragmentAssociation {
                owner: FragmentOwner::Origin(origin),
                source: Some(OriginIntersection {
                    origin,
                    aggregate: piece.aggregate,
                    local: piece.local,
                    processed: part.processed,
                }),
            },
            ledger,
        )?;
    }
    Ok(())
}

pub(crate) fn generated_owner(
    prepared: &PreparedData,
    event: &RelationEvent,
) -> Option<FragmentOwner> {
    let analyzed = &prepared.analyzed;
    let component_owner = |owner| match owner {
        ComponentOwner::Wbr(control) => Some(FragmentOwner::Control(control)),
        ComponentOwner::Virtual(index) => Some(FragmentOwner::VirtualSeparator(index)),
        ComponentOwner::Original(_) => None,
    };
    match *event {
        RelationEvent::Text { atom } => {
            component_owner(analyzed.transform.component_for_atom(atom).owner)
        }
        RelationEvent::GeneratedSuppression { component, .. } => {
            component_owner(analyzed.transform.components[component].owner)
        }
        RelationEvent::Anchor { order } => match analyzed.paragraph.anchor_order[order].item {
            SemanticAnchorItem::Control(control) => Some(FragmentOwner::Control(control)),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Owner {
    Original,
    Generated(GeneratedSeparatorOwner),
}
#[derive(Clone, Copy)]
struct Occurrence {
    event: usize,
    presentation: Range,
    owner: Owner,
}

pub(crate) struct CellCorrespondence<'a> {
    prepared: &'a PreparedData,
    presentation: &'a SelectedPresentation,
    unit: usize,
    segment: usize,
    binding: &'a CellBinding,
    group: &'a FixedGroup,
    occurrences: Vec<Option<Occurrence>>,
}
impl<'a> CellCorrespondence<'a> {
    fn invalid(&self) -> crate::Error {
        protocol_error(ProtocolError::InvalidSourceSegment {
            segment: self.segment,
        })
    }
    pub(crate) fn new(
        prepared: &'a PreparedData,
        presentation: &'a SelectedPresentation,
        segment: &PreparedSourceSegment,
        unit: usize,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let invalid = || {
            protocol_error(ProtocolError::InvalidSourceSegment {
                segment: segment.index,
            })
        };
        let row = presentation.units.get(unit).ok_or_else(invalid)?;
        let PresentationUnitKind::Cell { cell } = row.kind else {
            return Err(invalid());
        };
        let output = prepared
            .analyzed
            .combination
            .as_deref()
            .ok_or_else(invalid)?;
        let binding = output
            .bindings
            .get(prepared.shaped.cells.get(cell).ok_or_else(invalid)?.binding)
            .ok_or_else(invalid)?;
        let group = output.plan.groups.get(binding.group).ok_or_else(invalid)?;
        let mut result = Self {
            prepared,
            presentation,
            unit,
            segment: segment.index,
            binding,
            group,
            occurrences: account::filled(ledger, group.parts.len(), None)?,
        };
        let mut atom = row.atoms.start;
        let mut cut = row.presentation.start;
        for index in presentation.selected_relation.unit_events[unit].clone() {
            account::work(ledger, 1)?;
            let selected_event = &presentation.selected_relation.events[index];
            let paired =
                atom < row.atoms.end && presentation.selected_relation.atom_events[atom] == index;
            let range = if paired {
                let range = presentation.projection.atoms[atom].presentation;
                cut = range.end;
                atom += 1;
                range
            } else {
                Range::new(cut, cut)
            };
            let SelectedRelationEvent::Base { event } = *selected_event else {
                return Err(invalid());
            };
            if !binding.events.contains(&event) {
                continue;
            }
            let map = &prepared.analyzed.transform;
            match map.events()[event] {
                RelationEvent::Text { atom: original } => {
                    if !paired || presentation.originals[atom - 1] != Some(original) {
                        return Err(invalid());
                    }
                    let part = map
                        .emission_for_atom(original)
                        .selection_part
                        .ok_or_else(invalid)?;
                    let owner = match map.component_for_atom(original).owner {
                        ComponentOwner::Original(_) => Owner::Original,
                        ComponentOwner::Wbr(i) => {
                            Owner::Generated(GeneratedSeparatorOwner::Control(i))
                        }
                        ComponentOwner::Virtual(i) => {
                            Owner::Generated(GeneratedSeparatorOwner::Virtual(i))
                        }
                    };
                    result.record(part, index, range, owner, ledger)?;
                }
                RelationEvent::Removal {
                    selection_part: Some(part),
                    ..
                } => {
                    result.record(part, index, range, Owner::Original, ledger)?;
                }
                RelationEvent::GeneratedSuppression {
                    component,
                    selection_part,
                } => {
                    let owner = match map.components[component].owner {
                        ComponentOwner::Wbr(i) => {
                            Owner::Generated(GeneratedSeparatorOwner::Control(i))
                        }
                        ComponentOwner::Virtual(i) => {
                            Owner::Generated(GeneratedSeparatorOwner::Virtual(i))
                        }
                        ComponentOwner::Original(_) => return Err(invalid()),
                    };
                    result.record(selection_part, index, range, owner, ledger)?;
                }
                RelationEvent::PhaseIDeleted { lexical, .. } => {
                    let mut cursor =
                        lexical.occurrence_cursor(&prepared.analyzed.processed.lexical);
                    while let Some((record, _)) = cursor.next(|| account::work(ledger, 1))? {
                        if let Some(part) = output.plan.lexical_parts[record] {
                            result.record(part, index, range, Owner::Original, ledger)?;
                        }
                    }
                }
                _ => (),
            }
        }
        if atom != row.atoms.end {
            return Err(invalid());
        }
        for occurrence in &result.occurrences {
            account::work(ledger, 1)?;
            if occurrence.is_none() {
                return Err(invalid());
            }
        }
        Ok(result)
    }
    fn record(
        &mut self,
        part: usize,
        event: usize,
        presentation: Range,
        owner: Owner,
        ledger: &mut Ledger,
    ) -> Result<()> {
        account::work(ledger, 1)?;
        if !self.group.parts.contains(&part) {
            return Ok(());
        }
        let output = self
            .prepared
            .analyzed
            .combination
            .as_deref()
            .ok_or_else(|| self.invalid())?;
        let visible = output.plan.visible[part];
        // A retained lexical receipt can precede the same part's real revived emission.
        if visible && presentation.is_empty() {
            return Ok(());
        }
        if !visible && !presentation.is_empty() {
            return Err(self.invalid());
        }
        let index = part - self.group.parts.start;
        if let Some(old) = self.occurrences[index] {
            if visible || old.owner != owner {
                return Err(self.invalid());
            }
            return Ok(());
        }
        self.occurrences[index] = Some(Occurrence {
            event,
            presentation,
            owner,
        });
        Ok(())
    }
    pub(crate) fn visit_slot_pieces(
        &self,
        ledger: &mut Ledger,
        mut emit: impl FnMut(SpacingSlotPieceData, &mut Ledger) -> Result<()>,
    ) -> Result<()> {
        let row = &self.presentation.units[self.unit];
        let output = self
            .prepared
            .analyzed
            .combination
            .as_deref()
            .ok_or_else(|| self.invalid())?;
        let mut atom = row.atoms.start;
        let mut cut = row.presentation.start;
        for index in self.presentation.selected_relation.unit_events[self.unit].clone() {
            account::work(ledger, 1)?;
            if atom < row.atoms.end
                && self.presentation.selected_relation.atom_events[atom] == index
            {
                cut = self.presentation.projection.atoms[atom].presentation.end;
                atom += 1;
            }
            let SelectedRelationEvent::Base { event } =
                self.presentation.selected_relation.events[index]
            else {
                return Err(self.invalid());
            };
            if !self.binding.events.contains(&event) {
                continue;
            }
            let map = &self.prepared.analyzed.transform;
            match map.events()[event] {
                RelationEvent::Text { atom } => {
                    if let Some(part) = map.emission_for_atom(atom).selection_part {
                        self.emit_part(part, index, ledger, &mut emit)?;
                    }
                }
                RelationEvent::Removal {
                    selection_part: Some(part),
                    ..
                }
                | RelationEvent::GeneratedSuppression {
                    selection_part: part,
                    ..
                } => self.emit_part(part, index, ledger, &mut emit)?,
                RelationEvent::PhaseIDeleted { lexical, .. } => {
                    let mut cursor =
                        lexical.occurrence_cursor(&self.prepared.analyzed.processed.lexical);
                    while let Some((record, _)) = cursor.next(|| account::work(ledger, 1))? {
                        if let Some(part) = output.plan.lexical_parts[record] {
                            self.emit_part(part, index, ledger, &mut emit)?;
                        }
                    }
                }
                RelationEvent::Anchor { .. } => {
                    if let Some(FragmentOwner::Control(control)) =
                        generated_owner(self.prepared, &map.events()[event])
                    {
                        let mut represented = false;
                        for occurrence in &self.occurrences {
                            account::work(ledger, 1)?;
                            represented |= occurrence.is_some_and(|value| {
                                value.owner
                                    == Owner::Generated(GeneratedSeparatorOwner::Control(control))
                            });
                        }
                        if !represented {
                            emit(
                                SpacingSlotPieceData {
                                    owner: SpacingPiece::Control {
                                        occurrence: control,
                                    },
                                    presentation: PresentationRange {
                                        start: cut,
                                        end: cut,
                                    },
                                },
                                ledger,
                            )?;
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
    fn emit_part(
        &self,
        part: usize,
        event: usize,
        ledger: &mut Ledger,
        emit: &mut impl FnMut(SpacingSlotPieceData, &mut Ledger) -> Result<()>,
    ) -> Result<()> {
        account::work(ledger, 1)?;
        if !self.group.parts.contains(&part) {
            return Ok(());
        }
        let value =
            self.occurrences[part - self.group.parts.start].ok_or_else(|| self.invalid())?;
        if value.event != event {
            return Ok(());
        }
        let range = PresentationRange {
            start: value.presentation.start,
            end: value.presentation.end,
        };
        match value.owner {
            Owner::Original => {
                let output = self
                    .prepared
                    .analyzed
                    .combination
                    .as_deref()
                    .ok_or_else(|| self.invalid())?;
                visit_original_part(
                    self.prepared,
                    &output.plan.parts[part],
                    ledger,
                    |association, ledger| {
                        let source = association.source.ok_or_else(|| self.invalid())?;
                        emit(
                            SpacingSlotPieceData {
                                owner: SpacingPiece::Source {
                                    origin: source.origin,
                                    local: source.local,
                                    aggregate: source.aggregate,
                                },
                                presentation: range,
                            },
                            ledger,
                        )
                    },
                )?;
            }
            Owner::Generated(owner) => {
                let owner = match owner {
                    GeneratedSeparatorOwner::Control(occurrence) => {
                        SpacingPiece::Control { occurrence }
                    }
                    GeneratedSeparatorOwner::Virtual(occurrence) => {
                        SpacingPiece::VirtualSeparator { occurrence }
                    }
                };
                emit(
                    SpacingSlotPieceData {
                        owner,
                        presentation: range,
                    },
                    ledger,
                )?;
            }
        }
        Ok(())
    }
    pub(crate) fn drop(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.occurrences)
    }
}
