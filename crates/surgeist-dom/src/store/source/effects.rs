//! Actual algorithm backing retained beside private Source continuations.
use super::*;

#[cfg(test)]
pub(in crate::store) fn record_frame_writes(frames: usize) {
    if scale_metrics::ENABLED && frames != 0 {
        scale_metrics::record(scale_metrics::Event::SequenceMaterialized {
            bytes: frames * size_of::<select::EffectFrame>(),
        });
    }
}

/// An inline owner receipt for live algorithm storage in the independent Select
/// scratch domain. Its containing owner prices this header; the global custody
/// ledger separately owns the represented record or backing allocation.
pub(in crate::store) struct ScratchRetained {
    mirrors: formation::SourceMirrors,
    bytes: usize,
}
impl ScratchRetained {
    pub(in crate::store) fn empty(mirrors: formation::SourceMirrors) -> Self {
        Self { mirrors, bytes: 0 }
    }
    pub(in crate::store) fn new(
        draft: &Draft,
        bytes: usize,
        enforce: bool,
    ) -> Result<Self, DomError> {
        if enforce {
            draft.select_scratch(bytes)?;
        }
        let mirrors = draft.source_mirrors();
        let target = mirrors.temporary();
        let next = target
            .load(Ordering::Relaxed)
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        target.store(next, Ordering::Relaxed);
        Ok(Self { mirrors, bytes })
    }
    /// Reclassify this owner's exact surviving storage after its allocation
    /// gate already admitted every prospective overlap. Other scratch owners
    /// may change independently while the selected owner is executing.
    pub(in crate::store) fn replace_admitted(&mut self, bytes: usize) {
        let target = self.mirrors.temporary();
        let growth = bytes.saturating_sub(self.bytes);
        let shrink = self.bytes.saturating_sub(bytes);
        let before = target.load(Ordering::Relaxed);
        before
            .checked_add(growth)
            .and_then(|value| value.checked_sub(shrink))
            .expect("preadmitted owned Select scratch extent");
        target.fetch_add(growth, Ordering::Relaxed);
        let before = target.fetch_sub(shrink, Ordering::Relaxed);
        assert!(before >= shrink, "owned Select scratch backing");
        self.bytes = bytes;
    }
}
impl Drop for ScratchRetained {
    fn drop(&mut self) {
        let before = self
            .mirrors
            .temporary()
            .fetch_sub(self.bytes, Ordering::Relaxed);
        assert!(before >= self.bytes, "owned Select scratch backing");
    }
}

pub(in crate::store) struct EffectsCapacity {
    // Preserve the original empty backing while the current values live in its
    // replacement. Reverse earlier value inverses before restoring this bank.
    pub old: Vec<select::EffectFrame>,
    pub debit: custody::Debit,
    pub scratch: ScratchRetained,
}
impl Draft {
    pub(in crate::store) fn reserve_source_effects(
        &mut self,
        additional: usize,
        scoped: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let required = self
            .effects
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= self.effects.capacity() {
            return Ok(());
        }
        let maximum = isize::MAX as usize / size_of::<select::EffectFrame>();
        let wanted = self
            .effects
            .capacity()
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let requested = wanted
            .checked_mul(size_of::<select::EffectFrame>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        let undo = self.prepare_undo(cancel)?;
        #[cfg(test)]
        let before_growth = self.effect_bytes + self.select_temporary_bytes.load(Ordering::Relaxed);
        if scoped {
            self.select_scratch(requested)?;
        }
        let ledger = self.custody_ledger().clone();
        let mut debit = ledger.retain(custody::Usage::new(requested, 0), |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let mut next = Vec::new();
        next.try_reserve_exact(wanted)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = next
            .capacity()
            .checked_mul(size_of::<select::EffectFrame>())
            .ok_or(DomError::RepresentationExhausted)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes: actual });
        }
        if actual > requested {
            if scoped {
                self.select_scratch(actual)?;
            }
            debit.grow(custody::Usage::new(actual - requested, 0), |usage| {
                self.admit_custody_usage(usage, cancel)
            })?;
        }
        for _ in &self.effects {
            self.source_step(cancel)?;
        }
        // Retaining the old backing in an inverse is prepared before any value
        // moves. Scratch admits the old/new overlap through the checks above.
        let old_bytes = self
            .effects
            .capacity()
            .checked_mul(size_of::<select::EffectFrame>())
            .ok_or(DomError::RepresentationExhausted)?;
        let scratch = if undo {
            Some(ScratchRetained::new(self, old_bytes, false)?)
        } else {
            None
        };
        next.append(&mut self.effects);
        #[cfg(test)]
        record_frame_writes(next.len());
        let old = std::mem::replace(&mut self.effects, next);
        let old_debit = std::mem::replace(&mut self.effects_charge, debit);
        self.effect_bytes = actual;
        #[cfg(test)]
        if scoped {
            super::super::tree::scratch_observation::observe_growth(
                before_growth,
                actual,
                self.effect_bytes + self.select_temporary_bytes.load(Ordering::Relaxed),
                self.select_scratch_limit,
            );
        }
        if let Some(scratch) = scratch {
            self.record_source_undo_prepared(ControlUndo::EffectsCapacity(EffectsCapacity {
                old,
                debit: old_debit,
                scratch,
            }));
        } else {
            drop(old);
            drop(old_debit);
        }
        Ok(())
    }
}

/// Persistent top-only algorithm frontier. Its snapshots retain original nodes;
/// a push or top update materializes one admitted node and a pop aliases parent.
pub(in crate::store) struct ScratchStack<T: Copy> {
    top: Option<Arc<StackNode<T>>>,
    len: usize,
    scoped: bool,
}
struct StackNode<T: Copy> {
    value: T,
    previous: Option<Arc<StackNode<T>>>,
    _debit: custody::Debit,
    _scratch: ScratchRetained,
}
impl<T: Copy> Clone for ScratchStack<T> {
    fn clone(&self) -> Self {
        Self {
            top: self.top.clone(),
            len: self.len,
            scoped: self.scoped,
        }
    }
}
fn release_stack<T: Copy>(mut top: Option<Arc<StackNode<T>>>) {
    while let Some(node) = top {
        match Arc::try_unwrap(node) {
            Ok(mut node) => {
                top = node.previous.take();
            }
            Err(shared) => {
                drop(shared);
                break;
            }
        }
    }
}
impl<T: Copy> Drop for ScratchStack<T> {
    fn drop(&mut self) {
        release_stack(self.top.take());
    }
}
impl<T: Copy> ScratchStack<T> {
    pub fn new(draft: &Draft) -> Self {
        Self {
            top: None,
            len: 0,
            scoped: draft.effect_scoped,
        }
    }
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.len
    }
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn last(&self) -> Option<T> {
        self.top.as_ref().map(|node| node.value)
    }
    fn node(
        &self,
        value: T,
        previous: Option<Arc<StackNode<T>>>,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Arc<StackNode<T>>, DomError> {
        let bytes = size_of::<StackNode<T>>();
        if self.scoped || draft.effect_scoped {
            draft.select_scratch(bytes)?;
        }
        let debit = draft
            .custody_ledger()
            .retain(custody::Usage::new(bytes, 0), |usage| {
                draft.admit_custody_usage(usage, cancel)
            })?;
        let scratch = ScratchRetained::new(draft, bytes, false)?;
        Ok(formation::allocated_arc(StackNode {
            value,
            previous,
            _debit: debit,
            _scratch: scratch,
        }))
    }
    pub fn push(&mut self, value: T, draft: &Draft, cancel: &AtomicBool) -> Result<(), DomError> {
        let len = self
            .len
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let top = self.node(value, self.top.clone(), draft, cancel)?;
        release_stack(self.top.replace(top));
        self.len = len;
        Ok(())
    }
    pub fn replace_top(
        &mut self,
        value: T,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let previous = self
            .top
            .as_ref()
            .expect("replacing an actual frontier node")
            .previous
            .clone();
        let top = self.node(value, previous, draft, cancel)?;
        release_stack(self.top.replace(top));
        Ok(())
    }
    pub fn pop(&mut self) -> Option<T> {
        let node = self.top.as_ref()?;
        let value = node.value;
        let previous = node.previous.clone();
        release_stack(std::mem::replace(&mut self.top, previous));
        self.len -= 1;
        Some(value)
    }
    #[cfg(test)]
    pub fn top_identity(&self) -> Option<usize> {
        self.top.as_ref().map(|node| Arc::as_ptr(node) as usize)
    }
    #[cfg(test)]
    pub fn node_bytes() -> usize {
        size_of::<StackNode<T>>()
    }
}

/// Lifecycle of one actual parked frame. Consumed frames created after the
/// newest mark need no header snapshot: every surviving mark predates creation.
/// The inverse keeps only this fixed control atom, rather than cloned inputs.
pub(in crate::store) struct FrameLife {
    live: AtomicBool,
    born: Option<u64>,
    _debit: custody::Debit,
    _scratch: ScratchRetained,
}
impl FrameLife {
    pub fn new(draft: &Draft, scoped: bool, cancel: &AtomicBool) -> Result<Arc<Self>, DomError> {
        let bytes = size_of::<Self>();
        if scoped {
            draft.select_scratch(bytes)?;
        }
        let debit = draft
            .custody_ledger()
            .retain(custody::Usage::new(bytes, 0), |usage| {
                draft.admit_custody_usage(usage, cancel)
            })?;
        let scratch = ScratchRetained::new(draft, bytes, false)?;
        Ok(formation::allocated_arc(Self {
            live: AtomicBool::new(true),
            born: draft.journal.latest_mark_id(),
            _debit: debit,
            _scratch: scratch,
        }))
    }
    pub fn born(&self) -> Option<u64> {
        self.born
    }
    pub fn is_live(&self) -> bool {
        self.live.load(Ordering::Relaxed)
    }
    pub fn consume(&self) {
        self.live.store(false, Ordering::Relaxed);
    }
    pub fn restore(&self) {
        self.live.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> (Dom, Draft) {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(DomLimits::new(8, 4096, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
        let batch = DomBatch::new();
        let draft = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 128, 4096, 4096, 4),
            &cancel,
        )
        .unwrap()
        .0;
        (dom, draft)
    }

    fn push_empty_post(draft: &mut Draft, cancel: &AtomicBool) {
        let list = super::super::scratch::ScratchVec::new(draft, false);
        draft
            .push_effect(
                select::Effect::Post {
                    list,
                    index: 0,
                    stage: 0,
                },
                false,
                cancel,
            )
            .unwrap();
    }

    #[test]
    fn effect_capacity_growth_counts_only_the_frames_actually_moved() {
        use scale_metrics::{Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        let (_dom, mut draft) = draft();
        push_empty_post(&mut draft, &cancel);
        let old_len = draft.effects.len();
        let additional = draft.effects.capacity() + 1;
        let life = draft.effects[0].life.clone().unwrap();
        let measurement = Measurement::begin().unwrap();
        draft
            .reserve_source_effects(additional, false, &cancel)
            .unwrap();
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(draft.effects.len(), old_len);
        assert!(Arc::ptr_eq(draft.effects[0].life.as_ref().unwrap(), &life));
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            draft.effects.capacity() * size_of::<select::EffectFrame>()
        );
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(
            counters.sequence_materialized_bytes,
            old_len * size_of::<select::EffectFrame>()
        );
        let measurement = Measurement::begin().unwrap();
        draft.reserve_source_effects(0, false, &cancel).unwrap();
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 0);
    }

    #[test]
    fn actual_effect_push_and_both_rollback_paths_count_completed_frame_writes() {
        use scale_metrics::{Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        let (dom, mut draft) = draft();
        let measurement = Measurement::begin().unwrap();
        push_empty_post(&mut draft, &cancel);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<select::EffectFrame>()
        );
        let original = draft.effects[0].life.clone().unwrap();
        let pointer = draft.effects.as_ptr();
        let capacity = draft.effects.capacity();

        let mark = draft.source_checkpoint(0, 0, &cancel).unwrap();
        draft
            .reserve_source_effects(capacity + 1, false, &cancel)
            .unwrap();
        let spent = draft.source_work.load(Ordering::Relaxed);
        let measurement = Measurement::begin().unwrap();
        mark.restore(&mut draft);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<select::EffectFrame>()
        );
        assert_eq!(draft.effects.as_ptr(), pointer);
        assert_eq!(draft.effects.capacity(), capacity);
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);

        let mark = draft.source_checkpoint(0, 0, &cancel).unwrap();
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert!(draft.effects.is_empty());
        assert!(!original.is_live());
        let spent = draft.source_work.load(Ordering::Relaxed);
        let measurement = Measurement::begin().unwrap();
        mark.restore(&mut draft);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<select::EffectFrame>()
        );
        assert_eq!(draft.effects.as_ptr(), pointer);
        assert!(Arc::ptr_eq(
            draft.effects[0].life.as_ref().unwrap(),
            &original
        ));
        assert!(original.is_live());
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    }

    #[test]
    fn nested_mark_restores_the_original_parked_frame_before_outer_push_is_removed() {
        let cancel = AtomicBool::new(false);
        let (dom, mut draft) = draft();
        let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
        push_empty_post(&mut draft, &cancel);
        let original = draft.effects[0].life.clone().unwrap();
        let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert!(draft.effects.is_empty());
        assert!(!original.is_live());
        let spent = draft.source_work.load(Ordering::Relaxed);
        cancel.store(true, Ordering::Relaxed);
        inner.restore(&mut draft);
        assert_eq!(draft.effects.len(), 1);
        assert!(Arc::ptr_eq(
            draft.effects[0].life.as_ref().unwrap(),
            &original
        ));
        assert!(original.is_live());
        outer.restore(&mut draft);
        assert!(draft.effects.is_empty());
        assert!(!original.is_live());
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    }

    #[test]
    fn consumed_frame_born_under_a_sealed_inner_mark_is_removed_by_outer_rollback() {
        let cancel = AtomicBool::new(false);
        let (dom, mut draft) = draft();
        let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
        let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
        push_empty_post(&mut draft, &cancel);
        let original = draft.effects[0].life.clone().unwrap();
        drop(inner);
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert!(draft.effects.is_empty());
        assert!(!original.is_live());
        outer.restore(&mut draft);
        assert!(draft.effects.is_empty());
        assert!(!original.is_live());
    }
}
