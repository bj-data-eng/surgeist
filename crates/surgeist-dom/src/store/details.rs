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
}
pub(super) struct Finished {
    pub intentions: Vec<DomDetailsToggleIntention>,
    pub cancellations: Vec<DomDetailsToggleIntention>,
    pub next: u64,
}
pub(super) fn draft(
    source: &[DomDetailsToggleIntention],
    cancel: &AtomicBool,
) -> Result<Vec<Pending<Key>>, DomError> {
    let mut out = Vec::new();
    out.try_reserve_exact(source.len())
        .map_err(|_| DomError::AllocationFailed)?;
    for item in source {
        check_cancel(cancel)?;
        out.push(Pending {
            serial: Some(item.operation.serial),
            target: Key::Existing(item.target.slot),
            old: item.old,
            new: item.new,
        });
    }
    Ok(out)
}
impl Draft {
    fn is_details(&self, key: Key) -> Result<bool, DomError> {
        Ok(matches!(&self.node(key)?.kind, DomNodeKind::Element(e) if e.is_details()))
    }
    fn details_attribute_value(
        &self,
        key: Key,
        name: KnownHtmlAttribute,
        cancel: &AtomicBool,
    ) -> Result<Option<DocumentString>, DomError> {
        let DomNodeKind::Element(e) = &self.node(key)?.kind else {
            return Ok(None);
        };
        for attr in e.attributes().iter() {
            check_cancel(cancel)?;
            if attr.name().namespace().uri().is_none() {
                let mut expected = name.local_name().encode_utf16();
                let mut matches = true;
                for &unit in attr.name().local_name().code_units() {
                    check_cancel(cancel)?;
                    if Some(unit) != expected.next() {
                        matches = false;
                        break;
                    }
                }
                if matches && expected.next().is_none() {
                    return Ok(Some(attr.value().clone()));
                }
            }
        }
        Ok(None)
    }
    fn details_units_equal(
        a: &DocumentString,
        b: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            check_cancel(cancel)?;
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
        check_cancel(cancel)?;
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
        self.details_intentions
            .try_reserve_exact(1)
            .map_err(|_| DomError::AllocationFailed)?;
        self.details_intentions.push(Pending {
            serial: None,
            target,
            old,
            new,
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
        check_cancel(cancel)?;
        if !self.is_details(target)?
            || self
                .details_attribute_value(target, KnownHtmlAttribute::Open, cancel)?
                .is_none()
        {
            return Ok(());
        }
        let Some(name) = self.details_attribute_value(target, KnownHtmlAttribute::Name, cancel)?
        else {
            return Ok(());
        };
        if name.is_empty() {
            return Ok(());
        }
        let root = crate::slot::root(self, target, cancel)?;
        let mut next = Some(root);
        while let Some(peer) = next {
            check_cancel(cancel)?;
            if peer != target
                && self.is_details(peer)?
                && let Some(other) =
                    self.details_attribute_value(peer, KnownHtmlAttribute::Name, cancel)?
                && Self::details_units_equal(&name, &other, cancel)?
                && self
                    .details_attribute_value(peer, KnownHtmlAttribute::Open, cancel)?
                    .is_some()
            {
                let close = match request {
                    HtmlDetailsExclusivity::CloseGiven => target,
                    HtmlDetailsExclusivity::CloseOther => peer,
                };
                // Re-enter the actual attribute owner. Its Open->Closed hook
                // appends its own transition and requests no further closing.
                return self.remove_attribute(close, None, "open".into(), limits, cancel);
            }
            next = crate::slot::next(self, root, peer, cancel)?;
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
        if !self.is_details(target)? {
            return Ok(());
        }
        let attribute = if name.local_name().code_units() == [0x6e, 0x61, 0x6d, 0x65] {
            HtmlDetailsAttribute::Name
        } else if name.local_name().code_units() == [0x6f, 0x70, 0x65, 0x6e] {
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
            check_cancel(cancel)?;
            let item = self.details_intentions[i];
            if item.target == target {
                if let Key::Existing(slot) = target {
                    self.cancelled_details
                        .try_reserve_exact(1)
                        .map_err(|_| DomError::AllocationFailed)?;
                    self.cancelled_details.push(Pending {
                        serial: item.serial,
                        target: NodeId {
                            owner: self.store,
                            slot,
                            generation: self.generations[slot],
                        },
                        old: item.old,
                        new: item.new,
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
    pub(super) fn finish_details(
        &self,
        dom: &Dom,
        id: &impl Fn(Key) -> NodeId,
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
        intentions
            .try_reserve_exact(self.details_intentions.len())
            .map_err(|_| DomError::AllocationFailed)?;
        cancellations
            .try_reserve_exact(self.cancelled_details.len())
            .map_err(|_| DomError::AllocationFailed)?;
        for item in &self.details_intentions {
            check_cancel(cancel)?;
            intentions.push(DomDetailsToggleIntention {
                operation: serial(item.serial)?,
                target: id(item.target),
                old: item.old,
                new: item.new,
            });
        }
        for item in &self.cancelled_details {
            check_cancel(cancel)?;
            cancellations.push(DomDetailsToggleIntention {
                operation: serial(item.serial)?,
                target: item.target,
                old: item.old,
                new: item.new,
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
            });
        }
        let mut draft = Draft::new(self, DomBatch::new().owner, cancel)?;
        draft.details_intentions.clear();
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        Ok(DomDetailsToggleTransfer {
            source: self.snapshot(),
            changes: commit.changes,
            intentions: out,
        })
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
