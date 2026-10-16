//! HTML-owned Details decisions over one canonical staged ordinary tree.
use super::*;
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
    queue_association: Option<crate::details::DomDetailsQueueAssociation>,
}
impl<K> Pending<K> {
    pub(in crate::store) fn is_unissued(&self) -> bool {
        self.serial.is_none()
    }
}
// The value drops before its non-Clone Source occurrence debit.
struct DetailsValue {
    value: DocumentString,
    _payload: Option<source::checkpoint::Charge>,
}
pub(super) struct Finished {
    pub intentions: Vec<DomDetailsToggleIntention>,
    pub cancellations: Vec<DomDetailsToggleIntention>,
    pub next: u64,
}
pub(super) fn draft_formed(
    source: &[DomDetailsToggleIntention],
    initial: Option<&source::initial::Initial>,
    cancel: &AtomicBool,
) -> Result<Vec<Pending<Key>>, DomError> {
    let mut out = Vec::new();
    if let Some(initial) = initial {
        initial.reserve(&mut out, source.len(), cancel)?;
    } else {
        out.try_reserve_exact(source.len())
            .map_err(|_| DomError::AllocationFailed)?;
    }
    for item in source {
        source::initial::step(initial, cancel)?;
        out.push(Pending {
            serial: Some(item.operation.serial),
            target: Key::Existing(item.target.slot),
            old: item.old,
            new: item.new,
            queue_association: item.queue_association,
        });
    }
    Ok(out)
}
impl Draft {
    fn details_queue_association(
        &self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<crate::details::DomDetailsQueueAssociation>, DomError> {
        let Some(global) = self.node(target)?.relevant_global else {
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
        let DomNodeKind::Document(metadata) = &self.node(Key::Existing(document.node().slot))?.kind
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
        Ok(Some(crate::details::DomDetailsQueueAssociation {
            global,
            document,
        }))
    }

    fn is_details(&self, key: Key) -> Result<bool, DomError> {
        Ok(matches!(&self.node(key)?.kind, DomNodeKind::Element(e) if e.is_details()))
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
        let additional = self.source_reserve_amount::<Pending<Key>>(
            self.details_intentions.len(),
            self.details_intentions.capacity(),
            1,
            cancel,
        )?;
        self.details_intentions
            .try_reserve_exact(additional.unwrap_or(1))
            .map_err(|_| DomError::AllocationFailed)?;
        self.source_admit(0, cancel)?;
        self.headroom_details_issue(cancel)?;
        let queue_association = self.details_queue_association(target, cancel)?;
        self.details_intentions.push(Pending {
            serial: None,
            target,
            old,
            new,
            queue_association,
        });
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
        let root = crate::slot::root_admitted(self, target, &mut || self.source_step(cancel))?;
        let mut next = Some(root);
        while let Some(peer) = next {
            self.source_step(cancel)?;
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
            next = crate::slot::next_admitted(self, root, peer, &mut || self.source_step(cancel))?;
        }
        Ok(())
    }
    pub(super) fn details_attribute_change(
        &mut self,
        target: Key,
        name: &crate::QualifiedName,
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
        match surgeist_html::details_attribute_effect(name.namespace(), attribute, old, new) {
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
        let mut kept = 0;
        for i in 0..self.details_intentions.len() {
            self.source_step(cancel)?;
            let item = self.details_intentions[i];
            if item.target == target {
                if let Some(id) = self.published_id(target) {
                    #[cfg(test)]
                    let growth = source_observation::before(
                        self,
                        self.cancelled_details.len() == self.cancelled_details.capacity(),
                        cancel,
                    )?;
                    let additional = self.source_reserve_amount::<Pending<NodeId>>(
                        self.cancelled_details.len(),
                        self.cancelled_details.capacity(),
                        1,
                        cancel,
                    )?;
                    self.cancelled_details
                        .try_reserve_exact(additional.unwrap_or(1))
                        .map_err(|_| DomError::AllocationFailed)?;
                    #[cfg(test)]
                    source_observation::grew(
                        growth,
                        self.cancelled_details.capacity() * size_of::<Pending<NodeId>>(),
                    );
                    self.source_admit(0, cancel)?;
                    self.cancelled_details.push(Pending {
                        serial: item.serial,
                        target: id,
                        old: item.old,
                        new: item.new,
                        queue_association: item.queue_association,
                    });
                }
            } else {
                self.details_intentions[kept] = item;
                kept += 1;
            }
        }
        self.details_intentions.truncate(kept);
        Ok(())
    }
    pub(super) fn acknowledge_details_publication(
        &mut self,
        published: &[DomDetailsToggleIntention],
    ) {
        let mut published = published.iter();
        let private = self.phase_map.as_ref().map(|m| &m.private);
        for pending in &mut self.details_intentions {
            if matches!(pending.target,Key::Created(i) if private.is_some_and(|m|m[i])) {
                continue;
            }
            pending.serial = Some(
                published
                    .next()
                    .expect("visible intention")
                    .operation
                    .serial,
            );
        }
    }
    pub(super) fn finish_details(
        &self,
        dom: &Dom,
        id: &impl Fn(Key) -> NodeId,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
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
        let mut intentions = Vec::new();
        let mut cancellations = Vec::new();
        source::formation::reserve(
            &mut intentions,
            self.details_intentions.len(),
            budget,
            cancel,
        )?;
        source::formation::reserve(
            &mut cancellations,
            self.cancelled_details.len(),
            budget,
            cancel,
        )?;
        for item in &self.details_intentions {
            source::formation::step(budget, cancel)?;
            if self.is_private(item.target) {
                continue;
            }
            intentions.push(DomDetailsToggleIntention {
                operation: serial(item.serial)?,
                target: id(item.target),
                old: item.old,
                new: item.new,
                queue_association: item.queue_association,
            });
        }
        for item in &self.cancelled_details {
            source::formation::step(budget, cancel)?;
            cancellations.push(DomDetailsToggleIntention {
                operation: serial(item.serial)?,
                target: item.target,
                old: item.old,
                new: item.new,
                queue_association: item.queue_association,
            });
        }
        Ok(Finished {
            intentions,
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
        draft.details_intentions.clear();
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
        DomLimits::new(32, 4096, 64, 0, 0, 8, 65536)
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
        let limits = crate::DomSourceLimits::new(bytes, 4_000_000, 100_000_000, 64, 4096, 4096);
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
        ));
    }
    fn named_draft() -> (Draft, Key, usize) {
        let (mut dom, _, a, _) = fixture();
        dom.set_limits(DomLimits::new(32, 100_000, 64, 0, 0, 8, 65536).unwrap())
            .unwrap();
        let mut units = Vec::with_capacity(4096);
        units.push(u16::from(b'g'));
        let capacity = units.capacity();
        let mut setup = DomBatch::new();
        setup.set_attribute(a, "name", DocumentString::from_utf16(units));
        dom.apply(setup, &AtomicBool::new(false)).unwrap();
        (
            Draft::new(&dom, Arc::new(BatchIdentity), &AtomicBool::new(false)).unwrap(),
            Key::Existing(a.slot),
            capacity,
        )
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
    fn source_details_append_checks_replacement_peak_and_preserves_raw_order() {
        let (dom, _, a, _) = fixture();
        let cancel = AtomicBool::new(false);
        let key = Key::Existing(a.slot);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let bytes = size_of::<Pending<Key>>();
        source_budget(&mut draft, bytes - 1, 0);
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
        source_budget(&mut draft, bytes, 0);
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
        source_budget(&mut draft, bytes * 2 - 1, 0);
        assert!(
            draft
                .append_details_intention(
                    key,
                    HtmlDetailsState::Open,
                    HtmlDetailsState::Closed,
                    policy(8),
                    &cancel
                )
                .is_err()
        );
        assert_eq!(draft.details_intentions.len(), 1);
        source_budget(&mut draft, bytes * 2, 0);
        draft
            .append_details_intention(
                key,
                HtmlDetailsState::Open,
                HtmlDetailsState::Closed,
                policy(8),
                &cancel,
            )
            .unwrap();
        assert_eq!(draft.details_intentions.len(), 2);
        assert_eq!(
            (
                draft.details_intentions[0].old,
                draft.details_intentions[0].new
            ),
            (HtmlDetailsState::Closed, HtmlDetailsState::Open)
        );
        assert_eq!(
            (
                draft.details_intentions[1].old,
                draft.details_intentions[1].new
            ),
            (HtmlDetailsState::Open, HtmlDetailsState::Closed)
        );
        assert!(draft.details_intentions.iter().all(Pending::is_unissued));
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
        let limits = crate::DomSourceLimits::new(8_000_000, 4_000_000, 100_000_000, 64, 4096, 4096);
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
