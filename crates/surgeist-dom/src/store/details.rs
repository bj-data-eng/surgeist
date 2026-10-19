//! HTML-owned Details decisions over one canonical staged ordinary tree.
use super::*;
#[cfg(test)]
#[path = "details/discovery_tests.rs"]
mod discovery_tests;
use crate::details::{
    DomDetailsToggleIntention, DomDetailsToggleIntentionId, DomDetailsToggleTransfer,
};
use surgeist_html::{
    HtmlDetailsAttribute, HtmlDetailsAttributeEffect, HtmlDetailsExclusivity, HtmlDetailsState,
    KnownHtmlAttribute,
};

#[derive(Clone, Copy)]
pub(super) struct Pending<K> {
    serial: Option<u64>,
    target: K,
    old: HtmlDetailsState,
    new: HtmlDetailsState,
    queue_association: Option<crate::details::DomElementTaskAssociation>,
}
impl<K> Pending<K> {
    pub(in crate::store) fn is_unissued(&self) -> bool {
        self.serial.is_none()
    }
}
impl<K: Copy> Pending<K> {
    pub(in crate::store) fn target(&self) -> K {
        self.target
    }
    pub(in crate::store) fn queue_document(&self) -> Option<DocumentId> {
        self.queue_association
            .map(|association| association.document())
    }
}
impl Pending<Key> {
    pub(in crate::store) fn from_published(item: &DomDetailsToggleIntention) -> Self {
        Self {
            serial: Some(item.operation.serial),
            target: Key::Existing(item.target),
            old: item.old,
            new: item.new,
            queue_association: item.queue_association,
        }
    }
}
// The value drops before its non-Clone Source occurrence debit.
struct DetailsValue {
    value: DocumentString,
    _payload: Option<source::checkpoint::Charge>,
}
pub(super) struct Finished {
    pub intentions: SharedVec<DomDetailsToggleIntention>,
    pub output_debit: Option<custody::Debit>,
    pub ack: Option<transaction::queues::QueueAck<Pending<Key>, DomDetailsToggleIntention>>,
    pub cancellations: custody::Held<Vec<DomDetailsToggleIntention>>,
    pub next: u64,
}
impl Draft {
    pub(in crate::store) fn element_task_association(
        &self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<crate::details::DomElementTaskAssociation>, DomError> {
        let Some(global) = self.node(target)?.relevant_global() else {
            return Ok(None);
        };
        self.source_step(cancel)?;
        let document = self
            .hosted_globals
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?
            .document(global)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        self.source_step(cancel)?;
        let transaction::KindRead::Document(metadata) =
            &self.node(Key::Existing(document.node()))?.kind()
        else {
            return Err(DomError::HostedSourceUnavailable);
        };
        let environment = metadata.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        if association.global != global {
            return Err(DomError::HostedSourceUnavailable);
        }
        association
            .validate_live()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        Ok(Some(crate::details::DomElementTaskAssociation {
            global,
            document,
        }))
    }

    fn is_details(&self, key: Key) -> Result<bool, DomError> {
        Ok(matches!(&self.node(key)?.kind(), transaction::KindRead::Element(e) if e.is_details()))
    }
    fn details_attribute_value(
        &self,
        key: Key,
        name: KnownHtmlAttribute,
        cancel: &AtomicBool,
    ) -> Result<Option<DetailsValue>, DomError> {
        self.control_attribute(key, name, cancel)?
            .map(|value| {
                let (value, payload) = source::formation::shared_string(self, value, cancel)?;
                Ok(DetailsValue {
                    value,
                    _payload: payload,
                })
            })
            .transpose()
    }
    fn details_units_equal(
        &self,
        a: &DocumentString,
        b: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            self.source_step(cancel)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn append_details_intention(
        &mut self,
        target: Key,
        old: HtmlDetailsState,
        new: HtmlDetailsState,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let target = self.live_key(target);
        self.source_step(cancel)?;
        let count = self
            .details_intentions
            .len()
            .checked_add(self.cancelled_details.len())
            .and_then(|n| n.checked_add(1))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            count,
            limits.details_intentions,
            DomResource::DetailsToggleIntentions,
        )?;
        let mut values = [Pending {
            serial: None,
            target,
            old,
            new,
            queue_association: None,
        }];
        let mut prepared =
            self.queue_details_prepare_insert(self.details_intentions.len(), &values, cancel)?;
        self.headroom_details_issue(cancel)?;
        values[0].queue_association = self.element_task_association(target, cancel)?;
        self.queue_details_finalize_insert_document(&mut prepared, &values, cancel)?;
        self.queue_details_commit_insert(prepared, &values);
        Ok(())
    }
    pub(super) fn details_exclusivity(
        &mut self,
        target: Key,
        request: HtmlDetailsExclusivity,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !self.is_details(target)?
            || self
                .control_attribute(target, KnownHtmlAttribute::Open, cancel)?
                .is_none()
        {
            return Ok(());
        }
        let Some(name) = self.details_attribute_value(target, KnownHtmlAttribute::Name, cancel)?
        else {
            return Ok(());
        };
        if name.value.is_empty() {
            return Ok(());
        }
        #[cfg(test)]
        if super::scale_metrics::ENABLED {
            discovery_tests::entered();
        }
        let root = crate::slot::root_admitted(self, target, &mut || self.source_step(cancel))?;
        let group = self
            .families
            .details(self.live_key(root), &name.value, &mut || {
                self.source_step(cancel)
            })?;
        let mut next = self
            .families
            .first(group, &mut || self.source_step(cancel))?;
        while let Some(peer) = next {
            self.source_step(cancel)?;
            #[cfg(test)]
            if super::scale_metrics::ENABLED {
                discovery_tests::visited();
            }
            if peer != target
                && self.is_details(peer)?
                && let Some(other) =
                    self.details_attribute_value(peer, KnownHtmlAttribute::Name, cancel)?
                && self.details_units_equal(&name.value, &other.value, cancel)?
                && self
                    .control_attribute(peer, KnownHtmlAttribute::Open, cancel)?
                    .is_some()
            {
                let close = match request {
                    HtmlDetailsExclusivity::CloseGiven => target,
                    HtmlDetailsExclusivity::CloseOther => peer,
                };
                // Re-enter the actual attribute owner. Its Open->Closed hook
                // appends its own transition and requests no further closing.
                let operand = source::formation::Formation::new(self, cancel)?;
                let name = source::formation::string("open", operand.as_ref(), cancel)?;
                return self.remove_attribute(close, None, &name, limits, cancel);
            }
            next = self
                .families
                .after(group, peer, self.order_view(), &mut || {
                    self.source_step(cancel)
                })?;
        }
        Ok(())
    }
    pub(super) fn details_attribute_change(
        &mut self,
        target: Key,
        name: &crate::DomQualifiedName,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !self.is_details(target)? {
            return Ok(());
        }
        let attribute = if self.source_ascii(name.local_name(), "name", cancel)? {
            HtmlDetailsAttribute::Name
        } else if self.source_ascii(name.local_name(), "open", cancel)? {
            HtmlDetailsAttribute::Open
        } else {
            HtmlDetailsAttribute::Other
        };
        let namespace = name
            .namespace_uri()
            .cloned()
            .map_or_else(crate::Namespace::none, crate::Namespace::from_uri);
        match surgeist_html::details_attribute_effect(&namespace, attribute, old, new) {
            HtmlDetailsAttributeEffect::None => Ok(()),
            HtmlDetailsAttributeEffect::CloseGiven => {
                self.details_exclusivity(target, HtmlDetailsExclusivity::CloseGiven, limits, cancel)
            }
            HtmlDetailsAttributeEffect::Notify {
                old,
                new,
                close_other,
            } => {
                self.append_details_intention(target, old, new, limits, cancel)?;
                if close_other {
                    self.details_exclusivity(
                        target,
                        HtmlDetailsExclusivity::CloseOther,
                        limits,
                        cancel,
                    )?;
                }
                Ok(())
            }
        }
    }
    pub(super) fn cancel_details_intentions(
        &mut self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let target = self.live_key(target);
        self.source_step(cancel)?;
        if self.details_intentions.target_count(target) == 0 {
            return Ok(());
        }
        for index in 0..self.details_intentions.len() {
            self.source_step(cancel)?;
            let item = self
                .details_intentions
                .get(index)
                .expect("bounded Details queue index");
            if item.target == target
                && let Some(id) = self.published_id(target)
            {
                #[cfg(test)]
                let growth = source_observation::before(
                    self,
                    self.cancelled_details.len() == self.cancelled_details.capacity(),
                    cancel,
                )?;
                self.queue_cancelled_details_insert(
                    self.cancelled_details.len(),
                    &[Pending {
                        serial: item.serial,
                        target: id,
                        old: item.old,
                        new: item.new,
                        queue_association: item.queue_association,
                    }],
                    cancel,
                )?;
                #[cfg(test)]
                source_observation::grew(
                    growth,
                    self.cancelled_details.capacity() * size_of::<Pending<NodeId>>(),
                );
            }
        }
        self.queue_details_retain(|item| item.target != target, cancel)?;
        Ok(())
    }
    pub(super) fn finish_details(
        &self,
        dom: &Dom,
        id: &impl Fn(Key) -> NodeId,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        let mut next = dom.state.next_details_intention;
        let mut serial = |old: Option<u64>| -> Result<DomDetailsToggleIntentionId, DomError> {
            let value = if let Some(old) = old {
                old
            } else {
                let current = next;
                next = next
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                current
            };
            Ok(DomDetailsToggleIntentionId {
                owner: dom.owner,
                serial: value,
            })
        };
        let mut ack = None;
        let intentions = if let Some(canonical) = self.details_intentions.canonical_backing() {
            transaction::queues::QueueOutput::canonical(canonical.clone())
        } else {
            let mut intentions = Vec::new();
            source::formation::reserve(
                &mut intentions,
                self.details_intentions.len(),
                budget,
                cancel,
            )?;

            ack = Some(transaction::queues::QueueAck::prepare(
                usize::try_from(self.details_intentions.counts().unissued)
                    .map_err(|_| DomError::RepresentationExhausted)?,
                self.custody_ledger(),
                |usage| match budget {
                    Some(budget) => budget.admit_custody_usage(usage, cancel),
                    None => check_cancel(cancel),
                },
            )?);

            for (index, item) in self.details_intentions.iter().enumerate() {
                source::formation::step(budget, cancel)?;
                if self.is_private(item.target) {
                    continue;
                }
                let operation = serial(item.serial)?;
                intentions.push(DomDetailsToggleIntention {
                    operation,
                    target: id(item.target),
                    old: item.old,
                    new: item.new,
                    queue_association: item.queue_association,
                });
                #[cfg(test)]
                written.pushed::<DomDetailsToggleIntention>();
                if item.serial.is_none() {
                    ack.as_mut()
                        .expect("preadmitted unissued acknowledgement")
                        .push_prepared(
                            index,
                            Pending {
                                serial: Some(operation.serial),
                                ..item
                            },
                        );
                }
            }
            let selected = intentions.len();
            ack.as_mut()
                .expect("staged publication plan")
                .publish_values(
                    intentions,
                    selected,
                    budget,
                    self.custody_ledger(),
                    |usage| match budget {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                )?
        };

        let mut cancellations = Vec::new();
        source::formation::reserve(
            &mut cancellations,
            self.cancelled_details.len(),
            budget,
            cancel,
        )?;
        for item in self.cancelled_details.iter() {
            source::formation::step(budget, cancel)?;
            cancellations.push(DomDetailsToggleIntention {
                operation: serial(item.serial)?,
                target: item.target,
                old: item.old,
                new: item.new,
                queue_association: item.queue_association,
            });
            #[cfg(test)]
            written.pushed::<DomDetailsToggleIntention>();
        }
        let cancellations = transaction::queues::finish_vec(
            cancellations,
            budget,
            self.custody_ledger(),
            |usage| match budget {
                Some(budget) => budget.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
        )?;
        Ok(Finished {
            intentions: intentions.values,
            output_debit: intentions.output_debit,
            ack,
            cancellations,
            next,
        })
    }
}
impl Dom {
    /// Actual prior standalone transfer; it makes an unknown external future-task
    /// owner unsuitable for adoption by a fresh bound session.
    pub fn has_transferred_details_toggle_intentions(&self) -> bool {
        self.details_transferred
    }
    pub fn pending_details_toggle_intentions(&self) -> usize {
        self.state.details_intentions.len()
    }
    /// Atomically transfer all raw intentions. Output storage is admitted before
    /// any pending ownership is relinquished; this does not queue/fire events.
    pub fn take_details_toggle_intentions(
        &mut self,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomDetailsToggleTransfer, DomError> {
        self.take_details_toggle_intentions_inner(max_output, cancel, true)
    }
    /// Bound intake uses the exact registered original source after complete
    /// Runtime preadmission; no public skip flag can substitute for that owner.
    pub fn take_hosted_details_toggle_intentions(
        &mut self,
        source: &crate::hosted::DomHostedSource,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomDetailsToggleTransfer, DomError> {
        if self.owner != source.id().owner || self.hosted_source.as_ref() != Some(&source.observe())
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        source
            .revision()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        self.take_details_toggle_intentions_inner(max_output, cancel, false)
    }
    fn take_details_toggle_intentions_inner(
        &mut self,
        max_output: usize,
        cancel: &AtomicBool,
        standalone: bool,
    ) -> Result<DomDetailsToggleTransfer, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if max_output > isize::MAX as usize / size_of::<DomDetailsToggleIntention>() {
            return Err(DomError::InvalidLimit(DomResource::DetailsToggleOutput));
        }
        admit(
            self.state.details_intentions.len(),
            max_output,
            DomResource::DetailsToggleOutput,
        )?;
        let mut out = Vec::new();
        out.try_reserve_exact(self.state.details_intentions.len())
            .map_err(|_| DomError::AllocationFailed)?;
        for item in &self.state.details_intentions {
            check_cancel(cancel)?;
            out.push(DomDetailsToggleIntention {
                operation: item.operation,
                target: item.target,
                old: item.old,
                new: item.new,
                queue_association: item.queue_association,
            });
        }
        let mut draft = Draft::new(self, DomBatch::new().owner, cancel)?;
        draft.queue_details_clear(cancel)?;
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        if standalone && !out.is_empty() {
            self.details_transferred = true;
        }
        Ok(DomDetailsToggleTransfer {
            source: self.snapshot(),
            changes: commit.changes,
            intentions: out,
        })
    }
}

#[cfg(test)]
mod source_observation {
    use super::*;
    use std::{cell::Cell, sync::atomic::Ordering};
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Growth {
        pub required: usize,
        pub limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Growth>> = const { Cell::new(None) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(super) fn finish() -> Option<Growth> {
        ACTIVE.set(false);
        LAST.take()
    }
    pub(super) fn before(
        draft: &Draft,
        growing: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Growth>, DomError> {
        if !ACTIVE.get() || !growing {
            return Ok(None);
        }
        let Some(limits) = draft.source_limits else {
            return Ok(None);
        };
        Ok(Some(Growth {
            required: draft.source_storage_bytes(cancel)?
                + draft.source_other_bytes
                + draft.source_formation_bytes.load(Ordering::Relaxed),
            limit: limits.retained_bytes,
        }))
    }
    pub(super) fn grew(before: Option<Growth>, new: usize) {
        if let Some(mut before) = before {
            // Before includes the actual old bank. The fresh actual capacity
            // coexists with it during replacement; add that new bank once.
            before.required += new;
            if LAST.get().is_none_or(|old| old.required < before.required) {
                LAST.set(Some(before));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Namespace;
    fn policy(pending: usize) -> DomLimits {
        DomLimits::new(
            32,
            4096,
            64,
            0,
            0,
            8,
            65536,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_details_toggle_intentions(pending)
        .unwrap()
    }
    fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
        let mut dom = Dom::new(policy(8)).unwrap();
        let mut b = DomBatch::new();
        let doc = b.create_document();
        let a = b.create_element_ns(&doc, Namespace::html(), "details");
        let z = b.create_element_ns(&doc, Namespace::html(), "details");
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        (
            dom,
            c.document(&doc).unwrap(),
            c.created(&a).unwrap().unwrap(),
            c.created(&z).unwrap().unwrap(),
        )
    }
    fn actual_source_retirement_growth(bytes: usize) -> Option<source_observation::Growth> {
        let (mut dom, _, a, _) = fixture();
        let cancel = AtomicBool::new(false);
        dom.set_limits(policy(1024)).unwrap();
        for _ in 0..16 {
            let mut setup = DomBatch::new();
            for _ in 0..16 {
                setup.set_attribute(a, "open", "");
                setup.remove_attribute(a, "open");
            }
            dom.apply(setup, &cancel).unwrap();
        }
        assert_eq!(dom.pending_details_toggle_intentions(), 512);
        let before = dom.snapshot();
        let mut edit = DomBatch::new();
        edit.destroy(a);
        let limits = crate::DomSourceLimits::new(bytes, 4_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(edit, limits, &cancel) else {
            return None;
        };
        source_observation::start();
        let result = lease.advance(&cancel);
        let growth = source_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_details_retirement_admits_actual_old_and_new_banks_before_allocation() {
        let high =
            actual_source_retirement_growth(16_000_000).expect("actual cancellation bank growth");
        if let Some(actual) = actual_source_retirement_growth(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "Details cancellation banks retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
    fn source_budget(draft: &mut Draft, bytes: usize, units: usize) {
        let cancel = AtomicBool::new(false);
        draft.source_limits = Some(crate::DomSourceLimits::new(
            usize::MAX,
            usize::MAX,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
        let base = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
        let base_units = draft.source_storage_units(&cancel).unwrap() + draft.source_other_units;
        draft.source_limits = Some(crate::DomSourceLimits::new(
            base + bytes,
            base_units + units,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
    }
    struct Fixture {
        draft: Draft,
        _dom: Dom,
    }
    impl std::ops::Deref for Fixture {
        type Target = Draft;
        fn deref(&self) -> &Draft {
            &self.draft
        }
    }
    impl std::ops::DerefMut for Fixture {
        fn deref_mut(&mut self) -> &mut Draft {
            &mut self.draft
        }
    }
    fn named_draft() -> (Fixture, Key, usize) {
        let (mut dom, _, a, _) = fixture();
        dom.set_limits(
            DomLimits::new(
                32,
                100_000,
                64,
                0,
                0,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut units = Vec::with_capacity(4096);
        units.push(u16::from(b'g'));
        let capacity = units.capacity();
        let mut setup = DomBatch::new();
        setup.set_attribute(a, "name", DocumentString::from_utf16(units));
        dom.apply(setup, &AtomicBool::new(false)).unwrap();
        let draft = Draft::new(&dom, Arc::new(BatchIdentity), &AtomicBool::new(false)).unwrap();
        (Fixture { draft, _dom: dom }, Key::Existing(a), capacity)
    }
    #[test]
    fn source_details_name_has_exact_capacity_custody_and_releases_on_first_stop() {
        use std::sync::atomic::Ordering;
        let cancel = AtomicBool::new(false);
        let (mut draft, key, capacity) = named_draft();
        let bytes = size_of::<Vec<u16>>() + capacity * size_of::<u16>();
        source_budget(&mut draft, bytes * 2, capacity * 2);
        let first = draft
            .details_attribute_value(key, KnownHtmlAttribute::Name, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(first.value.code_units(), [u16::from(b'g')]);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), bytes);
        assert_eq!(
            draft.source_temporary_units.load(Ordering::Relaxed),
            capacity
        );
        let second = draft
            .details_attribute_value(key, KnownHtmlAttribute::Name, &cancel)
            .unwrap()
            .unwrap();
        assert!(Arc::ptr_eq(
            first.value.shared_code_units(),
            second.value.shared_code_units()
        ));
        assert_eq!(
            draft.source_formation_bytes.load(Ordering::Relaxed),
            bytes * 2
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            draft.details_attribute_value(key, KnownHtmlAttribute::Name, &cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(
            draft.source_formation_bytes.load(Ordering::Relaxed),
            bytes * 2
        );
        drop(second);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), bytes);
        drop(first);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        cancel.store(false, Ordering::Release);
        for short_units in [false, true] {
            source_budget(
                &mut draft,
                bytes - usize::from(!short_units),
                capacity - usize::from(short_units),
            );
            assert!(matches!(
                draft.details_attribute_value(key, KnownHtmlAttribute::Name, &cancel),
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes | DomResource::SourceRetainedUnits,
                    ..
                })
            ));
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        }
        draft.source_limits = None;
        let ordinary = draft
            .details_attribute_value(key, KnownHtmlAttribute::Name, &cancel)
            .unwrap()
            .unwrap();
        assert!(ordinary._payload.is_none());
        assert_eq!(ordinary.value.code_units(), [u16::from(b'g')]);
    }
    #[test]
    fn source_details_append_without_index_headroom_refuses_and_retry_preserves_raw_order() {
        let (dom, _, a, _) = fixture();
        let cancel = AtomicBool::new(false);
        let key = Key::Existing(a);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        // The full owner includes persistent target/document index preparation;
        // its no-headroom refusal is independent of the raw backing-only test.
        source_budget(&mut draft, 0, 0);
        assert!(matches!(
            draft.append_details_intention(
                key,
                HtmlDetailsState::Closed,
                HtmlDetailsState::Open,
                policy(8),
                &cancel
            ),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert!(draft.details_intentions.is_empty());
        source_budget(&mut draft, 1_000_000, 1_000_000);
        draft
            .append_details_intention(
                key,
                HtmlDetailsState::Closed,
                HtmlDetailsState::Open,
                policy(8),
                &cancel,
            )
            .unwrap();
        assert_eq!(draft.details_intentions.capacity(), 1);
        let pointer = draft.details_intentions.staged_pointer().unwrap();
        source_budget(&mut draft, 0, 0);
        assert!(matches!(
            draft.append_details_intention(
                key,
                HtmlDetailsState::Open,
                HtmlDetailsState::Closed,
                policy(8),
                &cancel
            ),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert_eq!(draft.details_intentions.len(), 1);
        assert_eq!(draft.details_intentions.capacity(), 1);
        assert_eq!(draft.details_intentions.staged_pointer().unwrap(), pointer);
        source_budget(&mut draft, 1_000_000, 1_000_000);
        draft
            .append_details_intention(
                key,
                HtmlDetailsState::Open,
                HtmlDetailsState::Closed,
                policy(8),
                &cancel,
            )
            .unwrap();
        transaction::queues::assert_details_backing_boundary([
            draft.details_intentions.get(0).unwrap(),
            draft.details_intentions.get(1).unwrap(),
        ]);
        assert_eq!(draft.details_intentions.len(), 2);
        assert_eq!(
            (
                draft.details_intentions.get(0).unwrap().old,
                draft.details_intentions.get(0).unwrap().new
            ),
            (HtmlDetailsState::Closed, HtmlDetailsState::Open)
        );
        assert_eq!(
            (
                draft.details_intentions.get(1).unwrap().old,
                draft.details_intentions.get(1).unwrap().new
            ),
            (HtmlDetailsState::Open, HtmlDetailsState::Closed)
        );
        assert!(
            draft
                .details_intentions
                .iter()
                .all(|item| item.is_unissued())
        );
        cancel.store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            draft.cancel_details_intentions(key, &cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(draft.details_intentions.len(), 2);
        assert!(draft.cancelled_details.is_empty());
    }
    #[test]
    fn source_details_raw_intention_survives_actual_csp_parking_and_cancelled_reentry() {
        use std::sync::atomic::Ordering;
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(policy(16)).unwrap();
        let mut setup = DomBatch::new();
        let doc = setup.create_document();
        let parent = setup.create_element_ns(&doc, Namespace::html(), "div");
        let details = setup.create_element_ns(&doc, Namespace::html(), "details");
        setup.insert_before(&parent, &doc, None);
        setup.insert_before(&details, &parent, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let doc = created.document(&doc).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let details = created.created(&details).unwrap().unwrap();
        let before = dom.snapshot();
        let mut edit = DomBatch::new();
        edit.set_attribute(details, "open", "");
        let style = edit.create_element_ns(doc, Namespace::html(), "style");
        let text = edit.create_text(doc, "a {}");
        edit.insert_before(&text, &style, None);
        edit.insert_before(&style, parent, None);
        let limits =
            crate::DomSourceLimits::new(8_000_000, 4_000_000, 100_000_000, 64, 4096, 4096, 1);
        let mut lease = dom.begin_source_commands(edit, limits, &cancel).unwrap();
        let request = match lease.advance(&cancel).unwrap() {
            crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(request)) => request,
            other => panic!("actual connected Style CSP boundary, got {other:?}"),
        };
        assert_eq!(
            request.action(),
            crate::DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp)
        );
        assert!(lease.status().has_published());
        let operation = lease.operation();
        let published = lease.snapshot().stamp();
        drop(lease);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            dom.resume_source_edit(operation, limits, &cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(dom.snapshot().stamp(), published);
        cancel.store(false, Ordering::Release);
        let mut lease = dom.resume_source_edit(operation, limits, &cancel).unwrap();
        let resumed = match lease.advance(&cancel).unwrap() {
            crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(request)) => request,
            other => panic!("same retained CSP request, got {other:?}"),
        };
        assert!(Arc::ptr_eq(&request.source, &resumed.source));
        lease
            .accept_inline_csp(
                &resumed,
                &crate::DomStyleInlineCspObservation::evaluated(
                    resumed.source(),
                    Arc::new(Vec::new()),
                    1,
                    false,
                ),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            crate::DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert!(
            before
                .node(details)
                .unwrap()
                .attribute(&DocumentString::from("open"))
                .is_none()
        );
        let transferred = dom.take_details_toggle_intentions(16, &cancel).unwrap();
        let records: Vec<_> = transferred.into_iter().collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].target(), details);
        assert_eq!(
            (records[0].old_state(), records[0].new_state()),
            (HtmlDetailsState::Closed, HtmlDetailsState::Open)
        );
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
    }
    #[test]
    fn canceled_prepared_subtree_retirement_preserves_intentions_and_retries_exactly_once() {
        let (mut dom, doc, a, survivor) = fixture();
        let cancel = AtomicBool::new(false);
        let history_start = dom.snapshot().stamp();
        let mut setup = DomBatch::new();
        let root = setup.create_element_ns(doc, Namespace::html(), "div");
        let victim = setup.create_element_ns(doc, Namespace::html(), "section");
        let nested = setup.create_element_ns(doc, Namespace::html(), "details");
        setup.insert_before(&root, doc, None);
        setup.insert_before(&victim, &root, None);
        setup.insert_before(a, &victim, None);
        setup.insert_before(&nested, a, None);
        setup.insert_before(survivor, &root, None);
        setup.set_attribute(a, "open", "");
        setup.set_attribute(survivor, "open", "");
        setup.set_attribute(&nested, "open", "");
        setup.remove_attribute(a, "open");
        setup.remove_attribute(survivor, "open");
        let commit = dom.apply(setup, &cancel).unwrap();
        let root = commit.created(&root).unwrap().unwrap();
        let victim = commit.created(&victim).unwrap().unwrap();
        let nested = commit.created(&nested).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let history = dom.changes_since(history_start).unwrap();
        let next = dom.state.next_details_intention;
        assert_eq!(next, 5);
        let original = &before.state.details_intentions;
        assert_eq!(original.len(), 5);
        // Literal source transition order; no Runtime coalescing or delivery is
        // supplied by this owning prepare/publication characterization.
        for (index, (target, old, new)) in [
            (a, HtmlDetailsState::Closed, HtmlDetailsState::Open),
            (survivor, HtmlDetailsState::Closed, HtmlDetailsState::Open),
            (nested, HtmlDetailsState::Closed, HtmlDetailsState::Open),
            (a, HtmlDetailsState::Open, HtmlDetailsState::Closed),
            (survivor, HtmlDetailsState::Open, HtmlDetailsState::Closed),
        ]
        .into_iter()
        .enumerate()
        {
            let row = &original[index];
            assert_eq!(
                (row.operation.owner, row.operation.serial),
                (dom.owner, u64::try_from(index).unwrap())
            );
            assert_eq!((row.target, row.old, row.new), (target, old, new));
            assert_eq!(row.queue_association, None);
        }
        let destroy = || {
            let mut edit = DomBatch::new();
            edit.detach(victim);
            edit.destroy(victim);
            edit
        };
        let prepared = dom.prepare(destroy(), &cancel).unwrap();
        for target in [victim, a, nested] {
            assert!(matches!(
                prepared.state.node(dom.owner, target),
                Err(DomError::StaleNode)
            ));
            assert!(prepared.changes.contains(&DomChange::Destroyed(target)));
        }
        assert_eq!(
            prepared.state.details_intentions.iter().collect::<Vec<_>>(),
            vec![&original[1], &original[4]]
        );
        let expected_cancellations: Vec<_> = [0, 2, 3]
            .into_iter()
            .map(|index| {
                let row = &original[index];
                DomChange::DetailsToggleIntentionCancelled {
                    operation: row.operation,
                    target: row.target,
                    old: row.old,
                    new: row.new,
                    reason: crate::DomDetailsToggleCancellationReason::TargetDestroyed,
                }
            })
            .collect();
        for change in &expected_cancellations {
            assert!(prepared.changes.contains(change));
        }
        assert_eq!(prepared.state.next_details_intention, next);
        // #1240 makes retirement and pending-intention cancellation one atomic
        // transaction, including refusal at the final real publication gate.
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert!(Arc::ptr_eq(&dom.state, &before.state));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.revision, before.revision);
        assert_eq!(dom.usage(), usage);
        let unchanged_history = dom.changes_since(history_start).unwrap();
        assert_eq!(unchanged_history.from(), history.from());
        assert_eq!(unchanged_history.to(), history.to());
        assert_eq!(unchanged_history.changes(), history.changes());
        assert_eq!(dom.state.details_intentions, *original);
        assert_eq!(dom.state.next_details_intention, next);
        for id in [doc.node(), root, victim, a, nested, survivor] {
            assert!(dom.snapshot().node(id).is_ok());
        }
        assert_eq!(
            dom.snapshot().node(root).unwrap().children(),
            &[victim, survivor]
        );
        assert_eq!(dom.snapshot().node(victim).unwrap().children(), &[a]);
        assert_eq!(dom.snapshot().node(a).unwrap().children(), &[nested]);
        assert_eq!(dom.snapshot().node(victim).unwrap().parent(), Some(root));
        assert_eq!(dom.snapshot().node(a).unwrap().parent(), Some(victim));
        assert_eq!(dom.snapshot().node(nested).unwrap().parent(), Some(a));

        cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        let retry = dom.prepare(destroy(), &cancel).unwrap();
        let commit = dom.publish(retry, &cancel).unwrap();
        for change in &expected_cancellations {
            assert_eq!(
                commit
                    .changes()
                    .changes()
                    .iter()
                    .filter(|actual| *actual == change)
                    .count(),
                1
            );
        }
        assert_eq!(
            commit
                .changes()
                .changes()
                .iter()
                .filter(|change| matches!(
                    change,
                    DomChange::DetailsToggleIntentionCancelled { .. }
                ))
                .count(),
            3
        );
        for id in [victim, a, nested] {
            assert_eq!(dom.snapshot().node(id).unwrap_err(), DomError::StaleNode);
            assert!(before.node(id).is_ok());
        }
        assert_eq!(dom.snapshot().node(root).unwrap().children(), &[survivor]);
        assert_eq!(dom.state.details_intentions.len(), 2);
        assert_eq!(dom.state.details_intentions[0], original[1]);
        assert_eq!(dom.state.details_intentions[1], original[4]);
        assert_eq!(dom.state.next_details_intention, next);
        // A later genuine transition receives the next unused operation once,
        // rather than skipping identities for canceled or retried retirement.
        let mut edit = DomBatch::new();
        edit.set_attribute(survivor, "open", "");
        dom.apply(edit, &cancel).unwrap();
        assert_eq!(dom.state.details_intentions[2].operation.serial, next);
        assert_eq!(dom.state.next_details_intention, next + 1);
        assert_eq!(before.state.details_intentions, *original);
    }
    #[test]
    fn late_publication_cancellation_preserves_primary_secondary_and_operation_counter() {
        let (mut dom, doc, a, b) = fixture();
        let cancel = AtomicBool::new(false);
        let mut edit = DomBatch::new();
        let root = edit.create_element_ns(doc, Namespace::html(), "div");
        for n in [a, b] {
            edit.set_attribute(n, "name", "g");
            edit.insert_before(n, &root, None);
        }
        edit.set_attribute(a, "open", "");
        dom.apply(edit, &cancel).unwrap();
        let before = dom.snapshot();
        let next = dom.state.next_details_intention;
        let mut edit = DomBatch::new();
        edit.set_attribute(b, "open", "");
        let prepared = dom.prepare(edit, &cancel).unwrap();
        assert_eq!(prepared.state.details_intentions.len(), 3);
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.state.next_details_intention, next);
        assert_eq!(dom.pending_details_toggle_intentions(), 1);
        assert_eq!(
            dom.snapshot()
                .node(a)
                .unwrap()
                .as_details()
                .unwrap()
                .state(),
            HtmlDetailsState::Open
        );
        assert_eq!(
            dom.snapshot()
                .node(b)
                .unwrap()
                .as_details()
                .unwrap()
                .state(),
            HtmlDetailsState::Closed
        );
    }
    #[test]
    fn checked_operation_exhaustion_and_transfer_revision_refusal_are_atomic() {
        let (mut dom, _, a, _) = fixture();
        let cancel = AtomicBool::new(false);
        Arc::get_mut(&mut dom.state).unwrap().next_details_intention = u64::MAX;
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.set_attribute(a, "open", "");
        assert_eq!(
            dom.apply(b, &cancel).unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
        drop(before);
        Arc::get_mut(&mut dom.state).unwrap().next_details_intention = 0;
        let mut b = DomBatch::new();
        b.set_attribute(a, "open", "");
        dom.apply(b, &cancel).unwrap();
        let id = dom.state.details_intentions[0].operation;
        let revision = dom.revision;
        dom.revision = DomRevision(u64::MAX);
        assert_eq!(
            dom.take_details_toggle_intentions(1, &cancel).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(dom.pending_details_toggle_intentions(), 1);
        assert_eq!(dom.state.details_intentions[0].operation, id);
        dom.revision = revision;
        assert_eq!(
            dom.take_details_toggle_intentions(1, &cancel)
                .unwrap()
                .intentions()[0]
                .operation(),
            id
        );
    }
    #[test]
    fn unpublished_create_destroy_is_exact_noop_without_fabricated_operation() {
        let (mut dom, doc, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        Arc::get_mut(&mut dom.state).unwrap().next_details_intention = u64::MAX;
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        let n = b.create_element_ns(doc, Namespace::html(), "details");
        b.set_attribute(&n, "open", "");
        b.remove_attribute(&n, "open");
        b.destroy(&n);
        let c = dom.apply(b, &cancel).unwrap();
        assert_eq!(c.created(&n).unwrap(), None);
        assert!(c.changes().changes().is_empty());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.state.next_details_intention, u64::MAX);
    }
    #[test]
    fn retirement_evidence_keeps_the_batch_capacity_and_existing_new_operation_identity() {
        let (mut dom, _, a, b) = fixture();
        let cancel = AtomicBool::new(false);
        dom.set_limits(policy(2)).unwrap();
        let before = dom.snapshot();
        let mut edit = DomBatch::new();
        edit.set_attribute(a, "open", "");
        edit.destroy(a);
        edit.set_attribute(b, "open", "");
        edit.remove_attribute(b, "open");
        assert!(matches!(
            dom.apply(edit, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::DetailsToggleIntentions,
                ..
            })
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        let mut edit = DomBatch::new();
        edit.set_attribute(a, "open", "");
        edit.destroy(a);
        edit.set_attribute(b, "open", "");
        let c = dom.apply(edit, &cancel).unwrap();
        let operation = c
            .changes()
            .changes()
            .iter()
            .find_map(|c| {
                if let DomChange::DetailsToggleIntentionCancelled {
                    operation, target, ..
                } = c
                {
                    assert_eq!(*target, a);
                    Some(*operation)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(dom.state.next_details_intention, 2);
        let t = dom.take_details_toggle_intentions(1, &cancel).unwrap();
        assert_ne!(t.intentions()[0].operation(), operation);
        assert_eq!(t.intentions()[0].target(), b);
    }
    #[test]
    fn retirement_retains_an_already_published_id_without_allocating_another() {
        let (mut dom, _, a, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut edit = DomBatch::new();
        edit.set_attribute(a, "open", "");
        dom.apply(edit, &cancel).unwrap();
        let operation = dom.state.details_intentions[0].operation;
        Arc::get_mut(&mut dom.state).unwrap().next_details_intention = u64::MAX;
        let mut edit = DomBatch::new();
        edit.destroy(a);
        let c = dom.apply(edit, &cancel).unwrap();
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::DetailsToggleIntentionCancelled {
                    operation,
                    target: a,
                    old: HtmlDetailsState::Closed,
                    new: HtmlDetailsState::Open,
                    reason: crate::DomDetailsToggleCancellationReason::TargetDestroyed,
                })
        );
        assert_eq!(dom.state.next_details_intention, u64::MAX);
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
    }
}
