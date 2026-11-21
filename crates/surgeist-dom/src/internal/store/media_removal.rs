use super::{transaction::KindRead, *};
use std::collections::VecDeque;

#[derive(Clone, Copy)]
pub(super) struct Pending {
    target: Key,
    removed_root: Key,
    former_parent: Key,
    former_document: Key,
}

#[derive(Debug)]
pub(super) struct Binding {
    owner: DomOwnerId,
    max_pending: usize,
    pending: AtomicUsize,
    _custody: custody::Debit,
}

/// The single checkpoint owner bound to this canonical DOM.
#[derive(Debug)]
pub struct DomMediaStableState {
    binding: Arc<Binding>,
}

/// One actual removal occurrence, qualified by its original DOM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomMediaRemoval {
    target: NodeId,
    former_document: DocumentId,
    serial: u64,
}

impl DomMediaRemoval {
    /// The exact original media node generation.
    pub const fn target(self) -> NodeId {
        self.target
    }

    /// The ordinary Document from which this occurrence removed the node.
    pub const fn former_document(self) -> DocumentId {
        self.former_document
    }
}

#[derive(Default)]
pub(super) struct Bank {
    binding: std::sync::Weak<Binding>,
    queue: VecDeque<DomMediaRemoval>,
    queue_charge: Option<custody::Debit>,
    serial: u64,
    scheduled: usize,
    work_limit: u64,
}

#[derive(Default)]
pub(super) struct Prepared {
    append: Option<custody::Held<Vec<DomMediaRemoval>>>,
    replacement: Option<custody::Held<VecDeque<DomMediaRemoval>>>,
    serial: u64,
}

impl Bank {
    pub(super) fn scheduler(&self) -> std::sync::Weak<Binding> {
        self.binding.clone()
    }

    pub(super) fn authorize_limits(&mut self, limits: DomLimits) {
        let nodes = u64::try_from(limits.managed_nodes()).unwrap_or(u64::MAX);
        // The caller may enlarge the managed forest after a removal. This
        // authorization grows its stable-state allowance at the same limits
        // fence, before any longer current parent chain can be published.
        self.work_limit = self.work_limit.max(nodes.saturating_add(2));
    }

    fn observe_count(&self) {
        if let Some(binding) = self.binding.upgrade() {
            binding.pending.store(self.queue.len(), Ordering::Relaxed);
        }
    }

    pub(super) fn prepare(
        &self,
        append: Option<custody::Held<Vec<DomMediaRemoval>>>,
        ledger: &custody::Ledger,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Prepared, DomError> {
        let Some(mut append) = append else {
            return Ok(Prepared::default());
        };
        if append.value().is_empty() {
            return Ok(Prepared::default());
        }
        let binding = self
            .binding
            .upgrade()
            .ok_or(DomError::MediaRemovalStableStateUnavailable)?;
        let len = self
            .queue
            .len()
            .checked_add(append.value().len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            len,
            binding.max_pending,
            DomResource::MediaRemovalContinuations,
        )?;
        let mut serial = self.serial;
        for removal in append.value_mut() {
            source::formation::step(budget, cancel)?;
            serial = serial
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            removal.serial = serial;
        }
        let replacement = if len > self.queue.capacity() {
            let mut queue = VecDeque::new();
            let wanted = len.max(
                self.queue
                    .capacity()
                    .saturating_mul(2)
                    .min(binding.max_pending),
            );
            let usage = |capacity| {
                usize::checked_mul(capacity, size_of::<DomMediaRemoval>())
                    .map(|bytes| custody::Usage::new(bytes, 0))
                    .ok_or(DomError::RepresentationExhausted)
            };
            let admit = |usage| match budget {
                Some(budget) => budget.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            };
            let mut debit = ledger.retain(usage(wanted)?, admit)?;
            queue
                .try_reserve_exact(wanted)
                .map_err(|_| DomError::AllocationFailed)?;
            transaction::nodes::resize_debit(&mut debit, usage(queue.capacity())?, admit)?;
            for removal in self.queue.iter().copied() {
                source::formation::step(budget, cancel)?;
                queue.push_back(removal);
            }
            Some(custody::Held::new(queue, debit))
        } else {
            None
        };
        Ok(Prepared {
            append: Some(append),
            replacement,
            serial,
        })
    }

    pub(super) fn publish(&mut self, prepared: Prepared) {
        let Some(append) = prepared.append else {
            return;
        };
        if let Some(replacement) = prepared.replacement {
            let (queue, debit) = replacement.into_parts();
            self.queue = queue;
            self.queue_charge = Some(debit);
        }
        let (append, debit) = append.into_parts();
        self.queue.extend(append);
        drop(debit);
        self.serial = prepared.serial;
        self.observe_count();
    }

    fn pop(&mut self) {
        self.queue.pop_front();
        self.scheduled = self.scheduled.saturating_sub(1);
        if self.queue.is_empty() {
            self.queue = VecDeque::new();
            self.queue_charge = None;
        }
        self.observe_count();
    }

    fn discard(&mut self) {
        self.queue = VecDeque::new();
        self.queue_charge = None;
        self.scheduled = 0;
        self.observe_count();
    }

    fn qualify(&self, authority: &DomMediaStableState, owner: DomOwnerId) -> Result<(), DomError> {
        if authority.binding.owner != owner
            || !self.binding.ptr_eq(&Arc::downgrade(&authority.binding))
        {
            return Err(DomError::MediaRemovalStableStateUnavailable);
        }
        Ok(())
    }
}

impl Dom {
    /// Binds the sole native scheduler for actual media removals in this DOM.
    pub fn bind_media_stable_state(
        &mut self,
        max_pending: usize,
        cancel: &AtomicBool,
    ) -> Result<DomMediaStableState, DomError> {
        self.require_source_entry(cancel)?;
        check_cancel(cancel)?;
        if self.media_removals.binding.strong_count() != 0 {
            return Err(DomError::MediaRemovalStableStateUnavailable);
        }
        let debit = self
            .custody_registry
            .ledger()
            .retain(custody::Usage::new(size_of::<Binding>(), 0), |_| {
                check_cancel(cancel)
            })?;
        self.media_removals.discard();
        self.media_removals.authorize_limits(self.limits);
        let binding = Arc::new(Binding {
            owner: self.owner,
            max_pending,
            pending: AtomicUsize::new(0),
            _custody: debit,
        });
        self.media_removals.binding = Arc::downgrade(&binding);
        Ok(DomMediaStableState { binding })
    }

    /// Whether actual removals still belong to this session's checkpoint owner.
    pub fn has_pending_media_removals(&self) -> bool {
        !self.media_removals.queue.is_empty()
    }
}

impl DomMediaStableState {
    /// Observe the next occurrence awaiting a callback in owning queue order.
    pub fn next_to_schedule(&self, dom: &Dom) -> Result<Option<DomMediaRemoval>, DomError> {
        let bank = &dom.media_removals;
        bank.qualify(self, dom.owner)?;
        Ok(bank.queue.get(bank.scheduled).copied())
    }

    /// Acknowledge the exact occurrence whose callback the owner just queued.
    pub fn mark_scheduled(&self, dom: &mut Dom, removal: DomMediaRemoval) -> Result<(), DomError> {
        if self.next_to_schedule(dom)? != Some(removal) {
            return Err(DomError::MediaRemovalStableStateUnavailable);
        }
        dom.media_removals.scheduled += 1;
        Ok(())
    }

    /// Run one queued callback at its position in the owning microtask queue.
    pub fn run_scheduled(
        &self,
        dom: &mut Dom,
        cancel: &AtomicBool,
    ) -> Result<Option<DomMediaRemoval>, DomError> {
        dom.media_removals.qualify(self, dom.owner)?;
        if dom.media_removals.scheduled == 0 {
            return Ok(None);
        }
        complete_front(dom, cancel)
    }

    /// Run one native occurrence at a checkpoint with no attached guest queue.
    pub fn run_native(
        &self,
        dom: &mut Dom,
        cancel: &AtomicBool,
    ) -> Result<Option<DomMediaRemoval>, DomError> {
        dom.media_removals.qualify(self, dom.owner)?;
        if dom.media_removals.scheduled != 0 {
            return Err(DomError::MediaRemovalStableStateUnavailable);
        }
        complete_front(dom, cancel)
    }

    /// Abandon callbacks queued on a retired guest owner.
    ///
    /// Occurrences still held solely by the native owner remain available.
    pub fn discard_scheduled(&self, dom: &mut Dom) -> Result<(), DomError> {
        dom.media_removals.qualify(self, dom.owner)?;
        let bank = &mut dom.media_removals;
        if bank.scheduled > 0 {
            bank.queue.drain(..bank.scheduled);
            bank.scheduled = 0;
            if bank.queue.is_empty() {
                bank.discard();
            } else {
                bank.observe_count();
            }
        }
        Ok(())
    }
}

fn complete_front(dom: &mut Dom, cancel: &AtomicBool) -> Result<Option<DomMediaRemoval>, DomError> {
    check_cancel(cancel)?;
    let Some(removal) = dom.media_removals.queue.front().copied() else {
        return Ok(None);
    };
    dom.require_source_entry(cancel)?;
    let work_limit = dom.media_removals.work_limit;
    let mut spent = 0u64;
    let result = (|| {
        let mut step = || {
            check_cancel(cancel)?;
            let next = spent
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            if next > work_limit {
                return Err(DomError::WorkLimit {
                    limit: work_limit,
                    resource: DomResource::MediaRemovalWork,
                });
            }
            spent = next;
            Ok(())
        };
        step()?;
        match dom.state.node(dom.owner, removal.target) {
            Ok(_) => {}
            Err(DomError::StaleNode) => return Ok(()),
            Err(error) => return Err(error),
        }
        dom.edit_native_state(cancel, |draft| {
            let node = Key::Existing(removal.target);
            let root = crate::internal::slot::root_admitted(draft, node, &mut step)?;
            if matches!(draft.node(root)?.kind(), KindRead::Document(_)) {
                return Ok(());
            }
            step()?;
            draft.with_element_fields(node, cancel, |element| element.pause_unloaded_media())?
        })
    })();
    // Once entered, this synchronous section has one terminal attempt. A
    // failed candidate publishes no state, and another callback may not
    // replay this occurrence. Entry cancellation leaves it unentered.
    dom.media_removals.pop();
    result.map(|()| Some(removal))
}

impl Draft {
    pub(super) fn reach_media_removal(
        &mut self,
        target: Key,
        removed_root: Key,
        former_parent: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(next) = self
            .media_removals
            .as_ref()
            .and_then(|rows| rows.value().get(self.media_hook_index))
        else {
            return Ok(());
        };
        // The pre-effect scan and removal walk have the same order. The
        // scan omits shadow trees and non-Document removals. A published
        // Source prefix already admitted its rows and reset this cursor.
        if (next.target, next.removed_root, next.former_parent)
            != (target, removed_root, former_parent)
        {
            return Ok(());
        }
        let binding = self
            .media_scheduler
            .upgrade()
            .ok_or(DomError::MediaRemovalStableStateUnavailable)?;
        let reached = self
            .media_hook_index
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let pending = binding
            .pending
            .load(Ordering::Relaxed)
            .checked_add(reached)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            pending,
            binding.max_pending,
            DomResource::MediaRemovalContinuations,
        )?;
        if self.prepare_undo_slots(1, cancel)? {
            self.journal
                .push_prepared(transaction::Undo::MediaHookIndex(self.media_hook_index));
        }
        self.media_hook_index = reached;
        Ok(())
    }

    pub(super) fn record_media_removal(
        &mut self,
        target: Key,
        removed_root: Key,
        former_parent: Key,
        former_document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let undo = self.prepare_undo_slots(2, cancel)?;
        let empty = Vec::new();
        let current = self
            .media_removals
            .as_ref()
            .map_or(&empty, custody::Held::value);
        if let Some(next) = self.prepare_output_buffer(current, 1, cancel)? {
            let before = self.media_removals.replace(next);
            if undo {
                self.journal
                    .push_prepared(transaction::Undo::MediaRemovalsCapacity(before));
            }
        }
        self.effect_step(cancel)?;
        let pending = self
            .media_removals
            .as_mut()
            .ok_or(DomError::RepresentationExhausted)?
            .value_mut();
        let len = pending.len();
        pending.push(Pending {
            target,
            removed_root,
            former_parent,
            former_document,
        });
        if undo {
            self.journal
                .push_prepared(transaction::Undo::MediaRemovalsLength(len));
        }
        Ok(())
    }

    pub(super) fn finish_media_removals(
        &self,
        ids: &source::AllocationIds,
        cancel: &AtomicBool,
    ) -> Result<Option<custody::Held<Vec<DomMediaRemoval>>>, DomError> {
        let Some(pending) = self.media_removals.as_ref().map(custody::Held::value) else {
            return Ok(None);
        };
        if pending.is_empty() {
            return Ok(None);
        }
        let resolve = |key| match key {
            Key::Existing(id) => Some(id),
            Key::Created(index) => ids.get(index),
        };
        let (mut output, debit) = self
            .prepare_output_buffer(&Vec::new(), pending.len(), cancel)?
            .ok_or(DomError::RepresentationExhausted)?
            .into_parts();
        for removal in pending {
            self.source_step(cancel)?;
            // A destroyed provisional creation has no externally observable
            // generation. An existing target still owns its retirement slot.
            let Some(target) = resolve(removal.target) else {
                continue;
            };
            let former_document = resolve(removal.former_document)
                .map(DocumentId)
                .ok_or(DomError::MediaRemovalStableStateUnavailable)?;
            output.push(DomMediaRemoval {
                target,
                former_document,
                serial: 0,
            });
        }
        Ok(Some(custody::Held::new(output, debit)))
    }
}
