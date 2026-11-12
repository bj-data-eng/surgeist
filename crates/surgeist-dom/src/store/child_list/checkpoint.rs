//! One native notification turn, sharing the Slot FIFO marker. Observer
//! record queues are drained when reached, so earlier callbacks may edit them.

use super::*;
use crate::{DomNativeCheckpoint, DomSlotNotifications, DomSlotQueryLimits};
use std::cell::Cell;

pub(in crate::store) struct CheckpointWork {
    limit: u64,
    spent: Cell<u64>,
}

impl CheckpointWork {
    pub(in crate::store) fn new(limit: u64) -> Self {
        Self {
            limit,
            spent: Cell::new(0),
        }
    }

    pub(super) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .spent
            .get()
            .checked_add(1)
            .filter(|next| *next <= self.limit)
            .ok_or(DomError::ChildListWorkLimit { limit: self.limit })?;
        self.spent.set(next);
        Ok(())
    }
}

struct Delivery {
    active: AtomicBool,
    _storage: custody::Debit,
}

/// One reached native observer delivery. Release this value after handling
/// the records to advance the notification turn to the next observer.
#[must_use]
pub struct DomChildListNotification {
    observer: DomChildListObserverId,
    records: custody::Held<Vec<DomChildListRecord>>,
    delivery: Arc<Delivery>,
}

impl DomChildListNotification {
    pub fn observer(&self) -> DomChildListObserverId {
        self.observer
    }

    pub fn records(&self) -> &[DomChildListRecord] {
        self.records.value()
    }
}

impl std::fmt::Debug for DomChildListNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomChildListNotification")
            .field("observer", &self.observer)
            .field("records", self.records.value())
            .finish()
    }
}

impl Drop for DomChildListNotification {
    fn drop(&mut self) {
        self.delivery.active.store(false, Ordering::Release);
    }
}

pub(in crate::store) struct Turn {
    slots: DomSlotNotifications,
    order: custody::Held<Vec<DomChildListObserverId>>,
    cursor: usize,
    delivery: Option<Arc<Delivery>>,
}

impl Working {
    fn take_pending(
        &mut self,
        order: &mut custody::Held<Vec<DomChildListObserverId>>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let mut current = self.tables.pending.first;
        while let Some(index) = current {
            edit.step()?;
            let slot = self
                .tables
                .observers
                .get(index)
                .copied()
                .ok_or(DomError::StaleChildListObserver)?;
            let mut observer = slot.value.ok_or(DomError::StaleChildListObserver)?;
            current = observer
                .pending
                .take()
                .ok_or(DomError::ChildListPublicationUnavailable)?
                .next;
            let id = DomChildListObserverId {
                owner: edit.draft.store,
                slot: index,
                generation: slot.generation,
            };
            records::reserve_output(order, edit)?;
            order.value_mut().push(id);
            self.put_observer(id, Some(observer), edit)?;
        }
        self.tables.pending = List::default();
        Ok(())
    }
}

impl Dom {
    pub(in crate::store) fn has_pending_child_lists(&self) -> bool {
        self.state
            .child_lists
            .as_ref()
            .is_some_and(|state| state.tables.pending.first.is_some())
    }

    pub(in crate::store) fn begin_child_list_turn(
        &mut self,
        limits: DomSlotQueryLimits,
        work: &CheckpointWork,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        work.step(cancel)?;
        let bytes = self
            .state
            .signal_slots
            .len()
            .checked_mul(size_of::<NodeId>())
            .and_then(|bytes| bytes.checked_add(size_of::<Turn>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let debit = self
            .custody_registry
            .ledger()
            .retain(custody::Usage::new(bytes, 0), |_| check_cancel(cancel))?;
        let mut order = custody::Held::new(Vec::new(), debit);
        let (mut draft, targets) = self.stage_mutation_notification(limits, cancel)?;
        draft.edit_child_lists_with_work(cancel, Some(work), |working, edit| {
            working.take_pending(&mut order, edit)
        })?;
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        let slots = DomSlotNotifications::new(self.snapshot(), targets, commit.changes);
        self.child_list_turn = Some(Box::new(Turn {
            order,
            slots,
            cursor: 0,
            delivery: None,
        }));
        Ok(())
    }

    pub(in crate::store) fn advance_child_list_turn(
        &mut self,
        work: &CheckpointWork,
        cancel: &AtomicBool,
    ) -> Result<Option<DomNativeCheckpoint>, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if let Some(turn) = &mut self.child_list_turn {
            if turn
                .delivery
                .as_ref()
                .is_some_and(|delivery| delivery.active.load(Ordering::Acquire))
            {
                return Err(DomError::ChildListDeliveryActive);
            }
            turn.delivery = None;
        }
        loop {
            let turn = self
                .child_list_turn
                .as_ref()
                .ok_or(DomError::ChildListPublicationUnavailable)?;
            let Some(&id) = turn.order.value().get(turn.cursor) else {
                let turn = self
                    .child_list_turn
                    .take()
                    .ok_or(DomError::ChildListPublicationUnavailable)?;
                return if turn.slots.targets().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(DomNativeCheckpoint::Slots(turn.slots)))
                };
            };
            work.step(cancel)?;
            let next = turn
                .cursor
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            let result = self.prepare_child_list_delivery(id, work, cancel)?;
            let turn = self
                .child_list_turn
                .as_mut()
                .ok_or(DomError::ChildListPublicationUnavailable)?;
            turn.cursor = next;
            if let Some(delivery) = result {
                turn.delivery = Some(delivery.delivery.clone());
                return Ok(Some(DomNativeCheckpoint::ChildList(delivery)));
            }
        }
    }

    fn prepare_child_list_delivery(
        &mut self,
        id: DomChildListObserverId,
        work: &CheckpointWork,
        cancel: &AtomicBool,
    ) -> Result<Option<DomChildListNotification>, DomError> {
        let Some(state) = &self.state.child_lists else {
            return Ok(None);
        };
        let observer = match state.tables.observer(self.owner, id) {
            Ok(observer) => observer,
            Err(DomError::StaleChildListObserver) => return Ok(None),
            Err(error) => return Err(error),
        };
        if observer.records.first.is_none() && observer.registrations.is_none() {
            return Ok(None);
        }
        self.edit_native_state(cancel, |draft| {
            let mut records = custody::Held::new(
                Vec::new(),
                draft
                    .custody_ledger()
                    .retain(custody::Usage::ZERO, |usage| {
                        draft.admit_custody_usage(usage, cancel)
                    })?,
            );
            draft.edit_child_lists_with_work(cancel, Some(work), |working, edit| {
                working.drain_records(id, Some(&mut records), edit)?;
                working.clear_transients(id, None, edit)
            })?;
            if records.value().is_empty() {
                return Ok(None);
            }
            let debit = draft
                .custody_ledger()
                .retain(custody::Usage::new(size_of::<Delivery>(), 0), |usage| {
                    draft.admit_custody_usage(usage, cancel)
                })?;
            let delivery = Arc::new(Delivery {
                active: AtomicBool::new(true),
                _storage: debit,
            });
            Ok(Some(DomChildListNotification {
                observer: id,
                records,
                delivery,
            }))
        })
    }
}
