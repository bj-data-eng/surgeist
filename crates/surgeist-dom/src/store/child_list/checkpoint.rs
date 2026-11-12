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

    fn require(&self, steps: usize) -> Result<(), DomError> {
        let refused = DomError::ChildListWorkLimit { limit: self.limit };
        let steps = u64::try_from(steps).map_err(|_| refused.clone())?;
        self.spent
            .get()
            .checked_add(steps)
            .filter(|spent| *spent <= self.limit)
            .map(|_| ())
            .ok_or(refused)
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
    slots: Option<CapturedSlots>,
    order: custody::Held<Vec<DomChildListObserverId>>,
    cursor: usize,
    delivery: Option<Arc<Delivery>>,
}

struct CapturedSlots {
    notifications: DomSlotNotifications,
    _state: transaction::StateCustody,
    _backing: custody::Debit,
}

impl Working {
    fn take_pending(
        &mut self,
        order: &mut custody::Held<Vec<DomChildListObserverId>>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        for (_, observer) in self.tables.pending.iter() {
            edit.step()?;
            records::reserve_output(order, edit)?;
            order.value_mut().push(*observer);
        }
        self.tables.pending = bank::Bank::default();
        self.controls.pending = bank::BankCustody::default();
        Ok(())
    }
}

impl Dom {
    pub(in crate::store) fn has_pending_child_lists(&self) -> bool {
        self.state
            .child_lists
            .as_ref()
            .is_some_and(|state| state.tables.pending.record_count() != 0)
    }

    pub(in crate::store) fn begin_child_list_turn(
        &mut self,
        limits: DomSlotQueryLimits,
        work: &CheckpointWork,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.require_source_idle()?;
        work.step(cancel)?;
        let debit = self
            .custody_registry
            .ledger()
            .retain(custody::Usage::new(size_of::<Turn>(), 0), |_| {
                check_cancel(cancel)
            })?;
        let mut order = custody::Held::new(Vec::new(), debit);
        let mut draft = Draft::new(self, DomBatch::new().owner, cancel)?;
        draft.edit_child_lists_with_work(cancel, Some(work), |working, edit| {
            working.take_pending(&mut order, edit)
        })?;
        let count = self.state.signal_slots.len();
        admit(count, limits.output_nodes(), DomResource::SlotQueryOutput)?;
        work.require(count)?;
        let targets = self.capture_child_list_slot_targets(&draft, work, cancel)?;
        draft.clear_mutation_notification(cancel)?;
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        let slots = targets.map(|targets| {
            let (targets, backing) = targets.into_parts();
            CapturedSlots {
                notifications: DomSlotNotifications::new(self.snapshot(), targets, commit.changes),
                _state: self.state_custody.clone(),
                _backing: backing,
            }
        });
        self.child_list_turn = Some(Box::new(Turn {
            order,
            slots,
            cursor: 0,
            delivery: None,
        }));
        Ok(())
    }

    fn capture_child_list_slot_targets(
        &self,
        draft: &Draft,
        work: &CheckpointWork,
        cancel: &AtomicBool,
    ) -> Result<Option<custody::Held<Arc<[NodeId]>>>, DomError> {
        if self.state.signal_slots.is_empty() {
            return Ok(None);
        }
        let edit = Edit {
            draft,
            cancel,
            checkpoint: Some(work),
        };
        let ledger = draft.custody_ledger();
        let mut temporary = ledger.retain(custody::Usage::ZERO, |usage| edit.admit(usage))?;
        let copied = records::copy_vec(
            self.state.signal_slots.iter().copied().map(Ok),
            &mut temporary,
            &edit,
        )?;
        let bytes = copied
            .len()
            .checked_mul(size_of::<NodeId>())
            .ok_or(DomError::RepresentationExhausted)?;
        let backing = ledger.retain(custody::Usage::new(bytes, 0), |usage| edit.admit(usage))?;
        let targets = copied.into();
        drop(temporary);
        Ok(Some(custody::Held::new(targets, backing)))
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
                return Ok(turn
                    .slots
                    .map(|slots| DomNativeCheckpoint::Slots(slots.notifications)));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(observer_count: usize, slotted: bool) -> Dom {
        let cancel = AtomicBool::new(false);
        let names = crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
        let observations = DomChildListObservationLimits::new(observer_count, 8, 8, 32).unwrap();
        let limits = DomLimits::new(32, 4096, 64, 64, 0, 8, 1_000_000, names)
            .unwrap()
            .with_child_list_observation(observations);
        let mut dom = Dom::new(limits).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let root = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&root, &document, None);
        if slotted {
            let shadow = setup.attach_shadow(
                &root,
                crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Open),
            );
            let slot = setup.create_element_ns(&document, crate::Namespace::html(), "slot");
            setup.insert_before(&slot, &shadow, None);
        }
        let commit = dom.apply(setup, &cancel).unwrap();
        let root = commit.created(&root).unwrap().unwrap();
        let document = commit.document(&document).unwrap();
        for index in 0..observer_count {
            let observer = dom.create_child_list_observer(&cancel).unwrap();
            if index + 1 == observer_count {
                dom.observe_child_list(observer, root, false, &cancel)
                    .unwrap();
            }
        }
        let mut append = DomBatch::new();
        let text = append.create_text(document, "pending");
        append.insert_before(&text, root, None);
        dom.apply(append, &cancel).unwrap();
        dom
    }

    fn begin(dom: &mut Dom, work: &CheckpointWork, cancel: &AtomicBool) {
        dom.begin_child_list_turn(DomSlotQueryLimits::new(64, 64).unwrap(), work, cancel)
            .unwrap();
    }

    #[test]
    fn observer_only_turn_does_not_pin_the_old_dom_state() {
        let cancel = AtomicBool::new(false);
        let work = CheckpointWork::new(1_000_000);
        let mut dom = fixture(1, false);
        begin(&mut dom, &work, &cancel);
        let old = Arc::downgrade(&dom.state);
        let delivery = dom.advance_child_list_turn(&work, &cancel).unwrap();
        assert!(matches!(delivery, Some(DomNativeCheckpoint::ChildList(_))));
        assert!(old.upgrade().is_none());
    }

    #[test]
    fn slot_turn_retains_the_capture_with_its_exact_existing_custody() {
        let cancel = AtomicBool::new(false);
        let work = CheckpointWork::new(1_000_000);
        let mut dom = fixture(1, true);
        assert_eq!(dom.snapshot().pending_slot_signals().len(), 1);
        begin(&mut dom, &work, &cancel);
        let capture = dom.state.clone();
        let delivery = dom.advance_child_list_turn(&work, &cancel).unwrap();
        assert!(matches!(delivery, Some(DomNativeCheckpoint::ChildList(_))));
        assert!(!Arc::ptr_eq(&capture, &dom.state));
        let mut visits = 0;
        let _retained = transaction::StateCustody::adopt(
            &capture,
            &dom.custody_registry,
            None,
            |_| Ok(()),
            || {
                visits += 1;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            visits, 1,
            "the in-flight turn keeps the original controlled capture"
        );
    }

    #[test]
    #[ignore = "requires SURGEIST_DOM_STORAGE_COUNTERS at compilation"]
    fn high_slot_observer_publication_skips_unrelated_observer_records() {
        let cancel = AtomicBool::new(false);
        let measured = |count| {
            let mut dom = fixture(count, false);
            let work = CheckpointWork::new(1_000_000);
            let measure = scale_metrics::Measurement::begin().unwrap();
            begin(&mut dom, &work, &cancel);
            measure
                .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Bank))
                .unwrap()
                .canonical_record_reads
        };
        let low = measured(8);
        let high = measured(512);
        assert!(
            high < low + 128,
            "bank record reads for 8 and 512 observers were {low} and {high}"
        );
    }
}
