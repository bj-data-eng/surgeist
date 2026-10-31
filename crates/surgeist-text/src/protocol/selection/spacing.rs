//! Canonical selected spacing targets and their actual mixed-arena custody.
use super::*;
use crate::engine::layout::data::ShapedData;
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::engine::scope_alignment::census::{AllocationVisitor, AllocationWitness as W};
use crate::engine::spacing::character::{Graph, Subject};
use crate::processing::contextual::alignment::operations;

pub(crate) fn funded(shaped: &ShapedData, policy: SpacingPolicy) -> bool {
    !shaped.cells.is_empty() && policy != SpacingPolicy::None
}
fn append_unique(rows: &mut Vec<usize>, value: usize, ledger: &mut Ledger) -> crate::Result<()> {
    for &old in rows.iter() {
        account::work(ledger, 1)?;
        if old == value {
            return Ok(());
        }
    }
    account::push(ledger, rows, value)
}
fn lookup(rows: &[(usize, usize)], key: usize, ledger: &mut Ledger) -> crate::Result<usize> {
    operations::binary_search_by(rows, ledger.as_mut(), |row| row.0.cmp(&key))?
        .ok()
        .map(|i| rows[i].1)
        .ok_or_else(|| {
            crate::Error::new(
                crate::ErrorCode::LayoutFailed,
                "selected spacing endpoint lacks its actual occurrence",
            )
        })
}

pub(crate) fn build_spacing_slots(
    prepared: &crate::engine::PreparedData,
    presentation: &super::super::prepared::SelectedPresentation,
    request: &LineSelectionRequest,
    shaped: &ShapedData,
    characters: &Graph,
    additional: &std::collections::BTreeSet<usize>,
    ledger: &mut Ledger,
) -> crate::Result<Vec<SpacingSlotData>> {
    let active = funded(shaped, request.context.spacing);
    let mut native = None;
    let ledger = if active {
        if ledger.is_none() {
            return Err(crate::engine::scope_alignment::refusal(
                crate::ScopeAlignmentError::LimitsRequired,
            ));
        }
        account::header::<Vec<SpacingSlotData>>(ledger)?;
        ledger
    } else {
        &mut native
    };
    let mut slots = Vec::new();
    let mut targets = Vec::new();
    match request.context.spacing {
        SpacingPolicy::None => return Ok(slots),
        SpacingPolicy::InterCharacter if characters.boundaries.is_empty() => return Ok(slots),
        SpacingPolicy::InterCharacter => (),
        SpacingPolicy::Spaces => {
            for unit in &presentation.units {
                account::work(ledger, 1)?;
                if unit.eligible_space {
                    account::push(
                        ledger,
                        &mut targets,
                        crate::engine::spacing::SeparatorCandidate {
                            atoms: unit.atoms.clone(),
                            base: unit.atoms.start,
                        },
                    )?;
                }
            }
        }
        SpacingPolicy::InterWord => crate::engine::spacing::visit_outer_candidates(
            presentation,
            additional,
            ledger,
            |candidate, ledger| account::push(ledger, &mut targets, candidate),
        )?,
    }
    if targets.is_empty() && request.context.spacing != SpacingPolicy::InterCharacter {
        account::drop_vec(ledger, targets)?;
        return Ok(slots);
    }
    let mut participant_at = account::filled(ledger, presentation.projection.atoms.len(), None)?;
    let mut cells = Vec::new();
    let mut atomics = Vec::new();
    for (index, unit) in presentation.units.iter().enumerate() {
        account::work(ledger, 1)?;
        for atom in unit.atoms.clone() {
            account::work(ledger, 1)?;
            participant_at[atom] = unit.participant;
        }
        match unit.kind {
            super::super::prepared::PresentationUnitKind::Cell { cell } => {
                account::push(
                    ledger,
                    &mut cells,
                    (prepared.shaped.cells[cell].binding, index),
                )?;
            }
            super::super::prepared::PresentationUnitKind::Atomic { occurrence } => {
                account::push(ledger, &mut atomics, (occurrence, index))?;
            }
            _ => (),
        }
    }
    operations::order_by(&mut cells, ledger.as_mut(), Ord::cmp)?;
    operations::order_by(&mut atomics, ledger.as_mut(), Ord::cmp)?;
    for rows in [&cells, &atomics] {
        for pair in rows.windows(2) {
            account::work(ledger, 1)?;
            if pair[0].0 == pair[1].0 {
                return Err(protocol_error(ProtocolError::ParticipantMismatch {
                    position: pair[1].1,
                }));
            }
        }
    }
    for candidate in &targets {
        let mut participants = Vec::new();
        for atom in candidate.atoms.clone() {
            account::work(ledger, 1)?;
            if let Some(index) = participant_at[atom] {
                append_unique(&mut participants, index, ledger)?;
            }
        }
        let pieces = crate::engine::spacing::slot_pieces(
            prepared,
            presentation,
            candidate.atoms.clone(),
            ledger,
        )?;
        let own = account::clone_vec(ledger, &participants)?;
        let tracking = prepared.styles[presentation.analysis.infos[candidate.base].1]
            .style
            .letter_spacing;
        account::push(
            ledger,
            &mut slots,
            SpacingSlotData {
                target: SpacingTargetData::Separator {
                    atoms: candidate.atoms.clone(),
                    base: candidate.base,
                    unit: SpacingUnitData {
                        participants: own,
                        pieces: 0..pieces.len(),
                        left: tracking,
                        right: tracking,
                    },
                },
                participants,
                pieces,
            },
        )?;
    }
    if request.context.spacing == SpacingPolicy::InterCharacter {
        for (edge, boundary) in characters.boundaries.iter().enumerate() {
            account::work(ledger, 1)?;
            let mut pieces = Vec::new();
            let mut participants = Vec::new();
            let mut endpoints = Vec::new();
            for index in [boundary.before, boundary.after] {
                let unit = &characters.units[index];
                let first = pieces.len();
                let mut own = Vec::new();
                match unit.subject() {
                    Subject::Cell(binding) => {
                        let index = lookup(&cells, binding, ledger)?;
                        let row = &presentation.units[index];
                        let participant = row.participant.ok_or_else(|| {
                            protocol_error(ProtocolError::ParticipantMismatch { position: index })
                        })?;
                        let super::super::prepared::PresentationUnitKind::Cell { cell } = row.kind
                        else {
                            return Err(protocol_error(ProtocolError::ParticipantMismatch {
                                position: participant,
                            }));
                        };
                        let Some(PreparedParticipant::Cell(handle)) =
                            request.participants.get(participant)
                        else {
                            return Err(protocol_error(ProtocolError::ParticipantMismatch {
                                position: participant,
                            }));
                        };
                        if !Arc::ptr_eq(&handle.prepared, &request.start.prepared)
                            || prepared.tables.cells[handle.index].cell != cell
                        {
                            return Err(protocol_error(ProtocolError::ParticipantMismatch {
                                position: participant,
                            }));
                        }
                        account::push(ledger, &mut own, participant)?;
                        let source = super::super::prepared::cell_sources::CellCorrespondence::new(
                            prepared,
                            presentation,
                            &request.segment,
                            index,
                            ledger,
                        )?;
                        source.visit_slot_pieces(ledger, |piece, ledger| {
                            account::push(ledger, &mut pieces, piece)
                        })?;
                        source.drop(ledger)?;
                    }
                    Subject::Text => {
                        for atoms in &unit.atoms {
                            crate::engine::spacing::visit_slot_pieces(
                                prepared,
                                presentation,
                                atoms.clone(),
                                ledger,
                                |piece, ledger| account::push(ledger, &mut pieces, piece),
                            )?;
                            for atom in atoms.clone() {
                                account::work(ledger, 1)?;
                                if let Some(i) = participant_at[atom] {
                                    append_unique(&mut own, i, ledger)?;
                                }
                            }
                        }
                    }
                    Subject::Atomics => {
                        for &occurrence in &unit.atomics {
                            let actual = &presentation.units[lookup(&atomics, occurrence, ledger)?];
                            account::push(
                                ledger,
                                &mut pieces,
                                SpacingSlotPieceData {
                                    owner: crate::SpacingPiece::Atomic { occurrence },
                                    presentation: PresentationRange {
                                        start: actual.presentation.start,
                                        end: actual.presentation.end,
                                    },
                                },
                            )?;
                            if let Some(i) = actual.participant {
                                append_unique(&mut own, i, ledger)?;
                            }
                        }
                    }
                }
                for &i in &own {
                    append_unique(&mut participants, i, ledger)?;
                }
                account::push(
                    ledger,
                    &mut endpoints,
                    SpacingUnitData {
                        participants: own,
                        pieces: first..pieces.len(),
                        left: unit.left,
                        right: unit.right,
                    },
                )?;
            }
            let mut drained = endpoints.drain(..);
            let before = drained.next().ok_or_else(|| {
                crate::Error::new(
                    crate::ErrorCode::LayoutFailed,
                    "missing first boundary endpoint",
                )
            })?;
            let after = drained.next().ok_or_else(|| {
                crate::Error::new(
                    crate::ErrorCode::LayoutFailed,
                    "missing second boundary endpoint",
                )
            })?;
            drop(drained);
            account::drop_vec(ledger, endpoints)?;
            account::push(
                ledger,
                &mut slots,
                SpacingSlotData {
                    target: SpacingTargetData::Boundary {
                        edge,
                        units: [before, after],
                    },
                    participants,
                    pieces,
                },
            )?;
        }
    }
    account::drop_vec(ledger, participant_at)?;
    account::drop_vec(ledger, cells)?;
    account::drop_vec(ledger, atomics)?;
    account::drop_vec(ledger, targets)?;
    Ok(slots)
}

pub(crate) fn visit_allocations(
    slots: &Vec<SpacingSlotData>,
    visit: &mut AllocationVisitor<'_>,
) -> crate::Result<()> {
    visit(W::object(slots, 0))?;
    visit(W::buffer(slots, slots.len())?)?;
    for slot in slots {
        visit(W::buffer(&slot.pieces, slot.pieces.len())?)?;
        visit(W::buffer(&slot.participants, slot.participants.len())?)?;
        match &slot.target {
            SpacingTargetData::Separator { unit, .. } => {
                visit(W::buffer(&unit.participants, unit.participants.len())?)?
            }
            SpacingTargetData::Boundary { units, .. } => {
                for unit in units {
                    visit(W::buffer(&unit.participants, unit.participants.len())?)?;
                }
            }
        }
    }
    Ok(())
}

/// Outer word feedback receipts share the same immutable selected ownership rule.
pub(crate) fn visit_word_allocations(
    shaped: &ShapedData,
    visit: &mut AllocationVisitor<'_>,
) -> crate::Result<()> {
    visit(W::object(&shaped.word_spacing, 0))?;
    visit(W::buffer(&shaped.word_spacing, shaped.word_spacing.len())?)?;
    for unit in &shaped.word_spacing {
        visit(W::buffer(&unit.components, unit.components.len())?)?;
    }
    Ok(())
}
