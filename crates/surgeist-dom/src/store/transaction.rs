//! Sparse transaction machinery. Canonical storage remains immutable; journal
//! holders retain only values needed by live rollback marks.

pub(in crate::store) mod auxiliary;
mod capture;
pub(in crate::store) mod controls;
pub(in crate::store) mod counted_relations;
mod counts;
mod fields;
pub(in crate::store) mod inbound;
mod journal;
mod names;
pub(in crate::store) mod nodes;
mod publication;
pub(in crate::store) mod queues;
pub(in crate::store) mod ranges;
mod records;
mod sequence;
pub(in crate::store) mod slot_index;

pub(in crate::store) use capture::{StateCustody, StateFrontiers};
pub(in crate::store) use counts::Counts;
pub(in crate::store) use fields::{ElementFieldsMut, NodeFields, NodeFieldsMut};
pub(in crate::store) use journal::{Journal, Mark};
pub(in crate::store) use names::Names;
pub(in crate::store) use records::{AutofocusRead, InteractiveRead, NodeRead};
#[cfg(test)]
pub(crate) use sequence::CopyObservation;
pub(in crate::store) use sequence::{Sequence, SequenceUndo, SequenceView};

use std::sync::Arc;

use super::{DomStamp, State, bank, custody};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NodeReceipts {
    pub fields: custody::Usage,
    pub payload: custody::Usage,
    pub names: custody::Usage,
    pub name_units: usize,
}

/// Proof of the actual canonical publication fence. It contains no root,
/// snapshot, debit or execution authority beyond that completed publication.
#[derive(Debug)]
pub(in crate::store) struct PublicationCommitted {
    _private: (),
}

impl PublicationCommitted {
    pub(super) fn at_fence() -> Self {
        Self { _private: () }
    }
}

pub(in crate::store) struct PreparedRebase {
    pub(super) state: Arc<State>,
    pub(super) custody: StateCustody,
    pub(super) stamp: DomStamp,
    pub(super) journal: journal::Commit<Undo>,
    pub(super) ids: super::source::AllocationIds,
    pub(super) signal_ack: Option<queues::QueueAck<super::Key, super::NodeId>>,
    pub(super) native_ack:
        Option<queues::QueueAck<super::select::Pending, super::select::Task<super::NodeId>>>,
    pub(super) details_ack: Option<
        queues::QueueAck<super::details::Pending<super::Key>, crate::DomDetailsToggleIntention>,
    >,
    pub(super) transition_ack: super::view_transition::TransitionAck,
}

pub(in crate::store) type JournalMark = Mark<Undo>;

#[expect(
    clippy::large_enum_variant,
    reason = "The admitted journal backing owns inverse headers inline; boxing adds a fallible allocation and separate custody to each mandatory inverse"
)]
pub(in crate::store) enum Undo {
    Source(super::source::ControlUndo),
    Image(super::image::Undo),
    Controls(controls::ControlUndo),
    Slots(slot_index::SlotUndo),
    TicketsLength(usize),
    TicketsCapacity(custody::Held<Vec<super::Key>>),
    RetirementsLength(usize),
    RetirementsCapacity(custody::Held<Vec<(super::NodeId, super::Key)>>),
    Range(ranges::RangeUndo),
    Transition(super::view_transition::TransitionUndo),
    Inbound(inbound::InboundUndo),
    CreatedMaterialized {
        index: usize,
    },
    CreatedDirty {
        before: bank::Bank<bool>,
        custody: bank::BankCustody<bool>,
    },
    Queue(queues::QueueUndo),
    Privacy {
        before: super::source::Privacy,
        nodes: usize,
        units: usize,
        counts: Counts,
    },
    Retired {
        key: super::Key,
        old: super::DraftNode,
        counts: Counts,
        nodes: usize,
        units: usize,
        private_nodes: usize,
        private_units: usize,
        private_counts: Counts,
    },
    Created {
        len: usize,
        counts: Counts,
        nodes: usize,
        units: usize,
    },
    Fields {
        key: super::Key,
        old: custody::Held<Box<NodeFields>>,
        counts: Counts,
        units: usize,
        private_units: usize,
        private_counts: Counts,
    },
    Relations {
        key: super::Key,
        field: RelationField,
        inverse: SequenceUndo<super::NodeId, super::Key>,
        counts: Counts,
        private_counts: Counts,
    },
    FullscreenEvents {
        key: super::Key,
        inverse: SequenceUndo<crate::DomFullscreenEvent, crate::DomFullscreenEvent>,
        counts: Counts,
        private_counts: Counts,
    },
    Materialized {
        before: bank::Bank<usize>,
        custody: bank::BankCustody<usize>,
        records: usize,
    },
    ArenaCapacity {
        old: Vec<Option<super::DraftNode>>,
        debit: custody::Debit,
    },
    CreatedCapacity {
        old: Vec<Option<super::DraftNode>>,
        debit: custody::Debit,
    },
}

impl Undo {
    pub(super) fn restore(self, draft: &mut super::Draft) {
        match self {
            Self::Source(undo) => undo.restore(draft),
            Self::Image(undo) => undo.restore(draft),
            Self::Controls(undo) => undo.restore(draft),
            Self::Slots(undo) => undo.restore(draft),
            Self::TicketsLength(len) => draft.tickets.truncate(len),
            Self::TicketsCapacity(before) => {
                let (values, debit) = before.into_parts();
                drop(std::mem::replace(&mut draft.tickets, values));
                draft.tickets_charge = debit;
            }
            Self::RetirementsLength(len) => draft.dispatch_retirements.truncate(len),
            Self::RetirementsCapacity(before) => {
                let (values, debit) = before.into_parts();
                drop(std::mem::replace(&mut draft.dispatch_retirements, values));
                draft.dispatch_retirements_charge = debit;
            }
            Self::Range(undo) => undo.restore(draft),
            Self::Transition(undo) => undo.restore(draft),
            Self::Inbound(old) => {
                draft.inbound = old;
            }
            Self::CreatedMaterialized { index } => {
                draft.created[index] = None;
            }
            Self::CreatedDirty { before, custody } => {
                draft.created_dirty = before;
                draft.created_dirty_custody = custody;
            }
            Self::Queue(undo) => undo.restore(draft),
            Self::Privacy {
                before,
                nodes,
                units,
                counts,
            } => {
                draft
                    .phase_map
                    .as_mut()
                    .expect("Source privacy inverse")
                    .private = before;
                draft.private_nodes = nodes;
                draft.private_units = units;
                draft.private_counts = counts;
            }
            Self::Retired {
                key,
                old,
                counts,
                nodes,
                units,
                private_nodes,
                private_units,
                private_counts,
            } => {
                match key {
                    super::Key::Existing(id) => {
                        draft.records[*draft
                            .existing
                            .get(id.slot)
                            .expect("retirement inverse index")] = Some(old)
                    }
                    super::Key::Created(index) => draft.created[index] = Some(old),
                };
                draft.counts = counts;
                draft.nodes = nodes;
                draft.units = units;
                draft.private_nodes = private_nodes;
                draft.private_units = private_units;
                draft.private_counts = private_counts;
            }
            Self::Created {
                len,
                counts,
                nodes,
                units,
            } => {
                draft.created.truncate(len);
                draft.counts = counts;
                draft.nodes = nodes;
                draft.units = units;
            }
            Self::Fields {
                key,
                old,
                counts,
                units,
                private_units,
                private_counts,
            } => {
                draft.private_node_mut(key).fields = old;
                draft.counts = counts;
                draft.units = units;
                draft.private_units = private_units;
                draft.private_counts = private_counts;
            }
            Self::Relations {
                key,
                field,
                inverse,
                counts,
                private_counts,
            } => {
                draft
                    .private_node_mut(key)
                    .relation_mut(field)
                    .restore(inverse);
                draft.counts = counts;
                draft.private_counts = private_counts;
            }
            Self::FullscreenEvents {
                key,
                inverse,
                counts,
                private_counts,
            } => {
                draft
                    .private_node_mut(key)
                    .fullscreen_events
                    .restore(inverse);
                draft.counts = counts;
                draft.private_counts = private_counts;
            }
            Self::Materialized {
                before,
                custody,
                records,
            } => {
                draft.records.truncate(records);
                draft.existing = before;
                draft.existing_custody = custody;
            }
            Self::ArenaCapacity { mut old, debit } => {
                old.append(&mut draft.records);
                draft.records = old;
                draft.records_charge = debit;
            }
            Self::CreatedCapacity { mut old, debit } => {
                old.append(&mut draft.created);
                draft.created = old;
                draft.created_charge = debit;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::store) enum RelationField {
    Children,
    ManualNodes,
    AssignedNodes,
    RenderBlocking,
    AutofocusCandidates,
    OpenDialogs,
    TopLayer,
    PendingTopLayerRemovals,
}

#[cfg(test)]
impl Undo {
    pub(super) fn journal_bytes(&self) -> usize {
        match self {
            Self::Source(undo) => undo.journal_bytes(),
            Self::Image(undo) => undo.journal_bytes(),
            Self::Controls(undo) => undo.journal_bytes(),
            Self::Slots(undo) => undo.journal_bytes(),
            Self::Range(undo) => undo.journal_bytes(),
            Self::Transition(undo) => undo.journal_bytes(),
            Self::Queue(undo) => undo.journal_bytes(),
            Self::TicketsCapacity(old) => old.debit().usage().bytes(),
            Self::RetirementsCapacity(old) => old.debit().usage().bytes(),
            Self::Fields { old, .. } => old.debit().usage().bytes(),
            Self::ArenaCapacity { debit, .. } | Self::CreatedCapacity { debit, .. } => {
                debit.usage().bytes()
            }
            Self::Relations { inverse, .. } => inverse.journal_bytes(),
            Self::FullscreenEvents { inverse, .. } => inverse.journal_bytes(),
            Self::Retired { old, .. } => old.journal_bytes(),
            _ => 0,
        }
    }
}
