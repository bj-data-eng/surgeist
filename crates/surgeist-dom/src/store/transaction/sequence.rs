//! One selected transaction sequence. Canonical operands are mapped only while
//! read, and an owned sequence reuses its admitted capacity between edits.

use std::{iter::FusedIterator, mem::size_of};

use super::super::{
    DomError, SharedVec,
    custody::{Debit, Held, Ledger, Usage},
};

#[cfg(test)]
#[derive(Default)]
pub(crate) struct CopyObservation {
    bytes: usize,
    relation: bool,
}
#[cfg(test)]
impl CopyObservation {
    pub(crate) fn relation() -> Self {
        Self {
            relation: true,
            ..Self::default()
        }
    }
    pub(crate) fn pushed<T>(&mut self) {
        self.extended::<T>(1);
    }
    pub(crate) fn extended<T>(&mut self, count: usize) {
        self.bytes = count
            .checked_mul(size_of::<T>())
            .and_then(|bytes| self.bytes.checked_add(bytes))
            .expect("actual sequence measurement representation");
    }
}
#[cfg(test)]
impl Drop for CopyObservation {
    fn drop(&mut self) {
        if self.relation {
            record_relation_copied_bytes(self.bytes);
        } else {
            record_copied_bytes(self.bytes);
        }
    }
}
#[cfg(test)]
pub(super) fn record_copied_bytes(bytes: usize) {
    if super::super::scale_metrics::ENABLED && bytes != 0 {
        super::super::scale_metrics::record(
            super::super::scale_metrics::Event::SequenceMaterialized { bytes },
        );
    }
}

#[cfg(test)]
fn record_relation_copied_bytes(bytes: usize) {
    record_copied_bytes(bytes);
    if super::super::scale_metrics::ENABLED && bytes != 0 {
        super::super::scale_metrics::record(
            super::super::scale_metrics::Event::RelationSequenceMaterialized { bytes },
        );
    }
}

enum Backing<C, S> {
    Captured {
        values: SharedVec<C>,
        _custody: super::super::bank::ObjectCustody,
    },
    Staged(Held<Vec<S>>),
}

pub(in crate::store) struct Sequence<C, S> {
    backing: Backing<C, S>,
    map: fn(C) -> S,
}

pub(in crate::store) enum SequenceUndo<C, S> {
    Backing(Sequence<C, S>),
    Inserted { index: usize, count: usize },
    Removed { index: usize, values: Held<Vec<S>> },
}

#[derive(Clone, Copy)]
pub(in crate::store) enum SequenceView<'a, C, S> {
    Empty,
    Canonical(&'a SharedVec<C>, fn(C) -> S),
    Staged(&'a Vec<S>),
    AliasedCanonical(
        &'a SharedVec<C>,
        fn(C) -> S,
        &'a super::super::source::AllocationIds,
        fn(S, &super::super::source::AllocationIds) -> S,
    ),
    AliasedStaged(
        &'a Vec<S>,
        &'a super::super::source::AllocationIds,
        fn(S, &super::super::source::AllocationIds) -> S,
    ),
}

impl<C: Copy, S: Copy> Sequence<C, S> {
    /// Alias one already controlled group. The original materialization
    /// control, rather than a new occurrence debit, keeps its backing admitted
    /// for this selected sequence and for any inverse that receives it.
    pub fn captured(
        values: &SharedVec<C>,
        custody: super::super::bank::ObjectCustody,
        map: fn(C) -> S,
    ) -> Self {
        Self {
            backing: Backing::Captured {
                values: values.clone(),
                _custody: custody,
            },
            map,
        }
    }
    pub fn staged(values: Vec<S>, debit: Debit, map: fn(C) -> S) -> Self {
        Self {
            backing: Backing::Staged(Held::new(values, debit)),
            map,
        }
    }

    pub fn empty(
        map: fn(C) -> S,
        ledger: &Ledger,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        Ok(Self::staged(
            Vec::new(),
            ledger.retain(Usage::ZERO, admit)?,
            map,
        ))
    }

    pub fn view(&self) -> SequenceView<'_, C, S> {
        match &self.backing {
            Backing::Captured { values, .. } => SequenceView::Canonical(values, self.map),
            Backing::Staged(values) => SequenceView::Staged(values.value()),
        }
    }

    pub fn len(&self) -> usize {
        self.view().len()
    }

    #[cfg(test)]
    pub fn capacity(&self) -> usize {
        match &self.backing {
            Backing::Captured { values, .. } => values.capacity(),
            Backing::Staged(values) => values.value().capacity(),
        }
    }

    #[cfg(test)]
    pub fn canonical_backing(&self) -> Option<&SharedVec<C>> {
        match &self.backing {
            Backing::Captured { values, .. } => Some(values),
            Backing::Staged(_) => None,
        }
    }

    #[cfg(test)]
    pub fn staged_values(&self) -> Option<&[S]> {
        match &self.backing {
            Backing::Captured { .. } => None,
            Backing::Staged(values) => Some(values.value()),
        }
    }

    #[cfg(test)]
    pub fn receipt(&self) -> Usage {
        match &self.backing {
            Backing::Captured { .. } => Usage::ZERO,
            Backing::Staged(values) => values.debit().usage(),
        }
    }

    /// The caller reserves its journal slot and dependency-index replacements
    /// before entering this operation. Refusal preserves the selected field.
    pub fn insert(
        &mut self,
        index: usize,
        incoming: &[S],
        retain_undo: bool,
        ledger: &Ledger,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<SequenceUndo<C, S>>, DomError> {
        if index > self.len() {
            return Err(DomError::IndexSize);
        }
        let needed = self
            .len()
            .checked_add(incoming.len())
            .ok_or(DomError::RepresentationExhausted)?;
        // All fallible work, including actual copied/shifted participants,
        // precedes the in-place sequence mutation.
        for _ in incoming {
            step()?;
        }
        for _ in index..self.len() {
            step()?;
        }
        let replacement = self.replacement(needed, ledger, &mut admit, &mut step)?;
        let old = replacement.map(|new| std::mem::replace(self, new));
        let Backing::Staged(values) = &mut self.backing else {
            unreachable!("selected sequence was materialized before mutation")
        };
        #[cfg(test)]
        let copied = if incoming.is_empty() {
            0
        } else {
            (values.value().len() - index + incoming.len()) * size_of::<S>()
        };
        values
            .value_mut()
            .splice(index..index, incoming.iter().copied());
        #[cfg(test)]
        record_relation_copied_bytes(copied);
        if !retain_undo {
            drop(old);
            return Ok(None);
        }
        Ok(Some(match old {
            Some(old) => SequenceUndo::Backing(old),
            None => SequenceUndo::Inserted {
                index,
                count: incoming.len(),
            },
        }))
    }

    pub fn remove(
        &mut self,
        index: usize,
        count: usize,
        retain_undo: bool,
        ledger: &Ledger,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<SequenceUndo<C, S>>, DomError> {
        let end = index
            .checked_add(count)
            .filter(|end| *end <= self.len())
            .ok_or(DomError::IndexSize)?;
        let replacement = self.replacement(self.len(), ledger, &mut admit, &mut step)?;
        let removed = if retain_undo && replacement.is_none() {
            let bytes = capacity_bytes::<S>(count)?;
            let mut debit = ledger.retain(Usage::new(bytes, 0), &mut admit)?;
            let mut removed = Vec::new();
            removed
                .try_reserve_exact(count)
                .map_err(|_| DomError::AllocationFailed)?;
            let actual = capacity_bytes::<S>(removed.capacity())?;
            #[cfg(test)]
            if super::super::scale_metrics::ENABLED && actual != 0 {
                super::super::scale_metrics::record(
                    super::super::scale_metrics::Event::StructuralAllocation { bytes: actual },
                );
            }
            if actual > bytes {
                debit.grow(Usage::new(actual - bytes, 0), &mut admit)?;
            }
            #[cfg(test)]
            let mut copied = CopyObservation::relation();
            for value in self.view().iter().skip(index).take(count) {
                step()?;
                removed.push(value);
                #[cfg(test)]
                copied.pushed::<S>();
            }
            Some(Held::new(removed, debit))
        } else {
            None
        };
        for _ in index..self.len() {
            step()?;
        }
        let old = replacement.map(|new| std::mem::replace(self, new));
        let Backing::Staged(values) = &mut self.backing else {
            unreachable!("selected sequence was materialized before mutation")
        };
        #[cfg(test)]
        let copied = if count == 0 {
            0
        } else {
            (values.value().len() - end) * size_of::<S>()
        };
        values.value_mut().drain(index..end);
        #[cfg(test)]
        record_relation_copied_bytes(copied);
        if !retain_undo {
            drop(old);
            return Ok(None);
        }
        Ok(Some(match old {
            Some(old) => SequenceUndo::Backing(old),
            None => SequenceUndo::Removed {
                index,
                values: removed.expect("removed values were admitted before mutation"),
            },
        }))
    }

    /// Move both old backing and debit into the inverse. New backing has
    /// already been admitted while the old field remains retained.
    pub fn replace(&mut self, replacement: Self, retain_undo: bool) -> Option<SequenceUndo<C, S>> {
        let old = std::mem::replace(self, replacement);
        retain_undo.then_some(SequenceUndo::Backing(old))
    }

    pub fn restore(&mut self, undo: SequenceUndo<C, S>) {
        match undo {
            SequenceUndo::Backing(old) => *self = old,
            SequenceUndo::Inserted { index, count } => {
                let Backing::Staged(values) = &mut self.backing else {
                    unreachable!("an in-place inverse retains staged backing")
                };
                #[cfg(test)]
                let copied = if count == 0 {
                    0
                } else {
                    (values.value().len() - index - count) * size_of::<S>()
                };
                values.value_mut().drain(index..index + count);
                #[cfg(test)]
                record_relation_copied_bytes(copied);
            }
            SequenceUndo::Removed { index, values } => {
                let Backing::Staged(current) = &mut self.backing else {
                    unreachable!("an in-place inverse retains staged backing")
                };
                assert!(
                    current.value().len() + values.value().len() <= current.value().capacity(),
                    "inverse insertion fits the original retained capacity"
                );
                #[cfg(test)]
                let copied = if values.value().is_empty() {
                    0
                } else {
                    (current.value().len() - index + values.value().len()) * size_of::<S>()
                };
                current
                    .value_mut()
                    .splice(index..index, values.value().iter().copied());
                #[cfg(test)]
                record_relation_copied_bytes(copied);
                drop(values);
            }
        }
    }

    fn replacement(
        &self,
        needed: usize,
        ledger: &Ledger,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Self>, DomError> {
        if matches!(&self.backing, Backing::Staged(values) if needed <= values.value().capacity()) {
            return Ok(None);
        }
        let capacity = match &self.backing {
            Backing::Captured { .. } => needed.max(self.len()),
            Backing::Staged(values) => needed.max(values.value().capacity().saturating_mul(2)),
        };
        let bytes = capacity_bytes::<S>(capacity)?;
        let mut debit = ledger.retain(Usage::new(bytes, 0), &mut *admit)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = capacity_bytes::<S>(values.capacity())?;
        #[cfg(test)]
        if super::super::scale_metrics::ENABLED && actual != 0 {
            super::super::scale_metrics::record(
                super::super::scale_metrics::Event::StructuralAllocation { bytes: actual },
            );
        }
        if actual > bytes {
            debit.grow(Usage::new(actual - bytes, 0), &mut *admit)?;
        }
        #[cfg(test)]
        let mut copied = CopyObservation::relation();
        for value in self.view().iter() {
            step()?;
            values.push(value);
            #[cfg(test)]
            copied.pushed::<S>();
        }
        Ok(Some(Self::staged(values, debit, self.map)))
    }
}

impl<'a, C: Copy, S: Copy> SequenceView<'a, C, S> {
    pub fn len(self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Canonical(v, _) | Self::AliasedCanonical(v, _, _, _) => v.len(),
            Self::Staged(v) | Self::AliasedStaged(v, _, _) => v.len(),
        }
    }
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
    #[cfg(test)]
    pub fn capacity(self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Canonical(v, _) | Self::AliasedCanonical(v, _, _, _) => v.capacity(),
            Self::Staged(v) | Self::AliasedStaged(v, _, _) => v.capacity(),
        }
    }
    pub fn get(self, index: usize) -> Option<S> {
        match self {
            Self::Empty => None,
            Self::Canonical(v, map) => v.get(index).copied().map(map),
            Self::Staged(v) => v.get(index).copied(),
            Self::AliasedCanonical(v, map, ctx, alias) => v
                .get(index)
                .copied()
                .map(map)
                .map(|value| alias(value, ctx)),
            Self::AliasedStaged(v, ctx, alias) => {
                v.get(index).copied().map(|value| alias(value, ctx))
            }
        }
    }
    pub fn first(self) -> Option<S> {
        self.get(0)
    }
    pub fn aliases(
        self,
        ctx: &'a super::super::source::AllocationIds,
        alias: fn(S, &super::super::source::AllocationIds) -> S,
    ) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Canonical(v, map) | Self::AliasedCanonical(v, map, _, _) => {
                Self::AliasedCanonical(v, map, ctx, alias)
            }
            Self::Staged(v) | Self::AliasedStaged(v, _, _) => Self::AliasedStaged(v, ctx, alias),
        }
    }
    pub fn iter(self) -> Iter<'a, C, S> {
        Iter {
            view: self,
            front: 0,
            back: self.len(),
        }
    }
}
pub(in crate::store) struct Iter<'a, C, S> {
    view: SequenceView<'a, C, S>,
    front: usize,
    back: usize,
}
impl<C: Copy, S: Copy> Iterator for Iter<'_, C, S> {
    type Item = S;
    fn next(&mut self) -> Option<S> {
        if self.front == self.back {
            return None;
        }
        let index = self.front;
        self.front += 1;
        self.view.get(index)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}
impl<C: Copy, S: Copy> DoubleEndedIterator for Iter<'_, C, S> {
    fn next_back(&mut self) -> Option<S> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.view.get(self.back)
    }
}
impl<C: Copy, S: Copy> ExactSizeIterator for Iter<'_, C, S> {}
impl<C: Copy, S: Copy> FusedIterator for Iter<'_, C, S> {}

fn capacity_bytes<T>(capacity: usize) -> Result<usize, DomError> {
    capacity
        .checked_mul(size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(DomError::RepresentationExhausted)
}

impl<'a, C: Copy, S: Copy> IntoIterator for SequenceView<'a, C, S> {
    type Item = S;
    type IntoIter = Iter<'a, C, S>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, C: Copy, S: Copy> IntoIterator for &SequenceView<'a, C, S> {
    type Item = S;
    type IntoIter = Iter<'a, C, S>;
    fn into_iter(self) -> Self::IntoIter {
        (*self).iter()
    }
}
impl<C: Copy, S: Copy + PartialEq> PartialEq<Vec<S>> for SequenceView<'_, C, S> {
    fn eq(&self, other: &Vec<S>) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter().copied())
    }
}

impl<C: Copy, S: Copy + std::fmt::Debug> std::fmt::Debug for SequenceView<'_, C, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
impl<C, S> SequenceUndo<C, S> {
    pub(super) fn journal_bytes(&self) -> usize {
        match self {
            Self::Backing(old) => match &old.backing {
                Backing::Captured { .. } => 0,
                Backing::Staged(values) => values.debit().usage().bytes(),
            },
            Self::Inserted { .. } => 0,
            Self::Removed { values, .. } => values.debit().usage().bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn accept(_: Usage) -> Result<(), DomError> {
        Ok(())
    }
    fn map(value: u32) -> u64 {
        u64::from(value)
    }

    fn captured_sequence(
        values: &SharedVec<u32>,
        ledger: &Ledger,
    ) -> (
        Sequence<u32, u64>,
        super::super::super::payload::Sealed<
            super::super::super::canonical::ContainerChildren<u32>,
        >,
        super::super::super::bank::CustodyRegistry,
    ) {
        use super::super::super::{bank::CustodyRegistry, canonical::ContainerChildren, payload};
        let registry = CustodyRegistry::new(ledger, accept).unwrap();
        let usage = Usage::new(
            size_of::<payload::Payload<ContainerChildren<u32>>>()
                + size_of::<Vec<u32>>()
                + values.capacity() * size_of::<u32>(),
            0,
        );
        let group = payload::Pending::new(
            Held::new(
                ContainerChildren {
                    values: values.clone(),
                },
                ledger.retain(Usage::ZERO, accept).unwrap(),
            ),
            usage,
            accept,
        )
        .unwrap()
        .seal(&registry, accept)
        .unwrap();
        let sequence = Sequence::captured(&group.value.value.values, group.custody.clone(), map);
        (sequence, group, registry)
    }

    fn assert_relation_copies(
        counters: &super::super::super::scale_metrics::Counters,
        bytes: usize,
    ) {
        assert_eq!(counters.sequence_materialized_bytes, bytes);
        assert_eq!(counters.relation_sequence_materialized_bytes, bytes);
        assert_eq!(
            counters.relation_sequence_materializations,
            counters.sequence_materializations
        );
    }

    #[test]
    fn captured_group_alias_retains_original_backing_without_a_second_occurrence() {
        use super::super::super::{bank::CustodyRegistry, canonical::ContainerChildren, payload};
        let ledger = Ledger::default();
        let registry = CustodyRegistry::new(&ledger, accept).unwrap();
        let mut values = Vec::with_capacity(4096);
        values.extend([3u32, 5, 8]);
        let values = SharedVec::from(values);
        let usage = Usage::new(
            size_of::<payload::Payload<ContainerChildren<u32>>>()
                + size_of::<Vec<u32>>()
                + values.capacity() * size_of::<u32>(),
            0,
        );
        let debit = ledger.retain(Usage::ZERO, accept).unwrap();
        let original = payload::Pending::new(
            super::super::super::custody::Held::new(
                ContainerChildren {
                    values: values.clone(),
                },
                debit,
            ),
            usage,
            accept,
        )
        .unwrap()
        .seal(&registry, accept)
        .unwrap();
        let before = ledger.total();
        let mut sequence =
            Sequence::captured(&original.value.value.values, original.custody.clone(), map);
        assert_eq!(ledger.total(), before);
        assert!(sequence.canonical_backing().unwrap().same_storage(&values));
        assert_eq!(sequence.view().iter().collect::<Vec<_>>(), [3, 5, 8]);
        assert_eq!(sequence.receipt(), Usage::ZERO);
        let inverse = sequence
            .insert(1, &[11], true, &ledger, accept, || Ok(()))
            .unwrap()
            .unwrap();
        assert_eq!(sequence.view().iter().collect::<Vec<_>>(), [3, 11, 5, 8]);
        sequence.restore(inverse);
        assert!(sequence.canonical_backing().unwrap().same_storage(&values));
        assert_eq!(sequence.capacity(), 4096);
        assert_eq!(ledger.total(), before);
        let rejected = sequence.insert(
            1,
            &[13],
            true,
            &ledger,
            |_| Err(DomError::Cancelled),
            || Ok(()),
        );
        assert!(matches!(rejected, Err(DomError::Cancelled)));
        assert!(sequence.canonical_backing().unwrap().same_storage(&values));
        assert_eq!(ledger.total(), before);
        drop(original);
        drop(sequence);
        registry.reconcile_releases();
        drop(registry);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn empty_view_has_no_backing_and_aliases_remain_empty() {
        let view = SequenceView::<u32, u64>::Empty;
        assert_eq!(view.len(), 0);
        assert_eq!(view.capacity(), 0);
        assert_eq!(view.get(0), None);
        assert_eq!(view.iter().next(), None);
        assert_eq!(view.iter().next_back(), None);
    }

    #[test]
    fn scalar_field_changes_can_retain_a_large_sequence_without_mapping_members() {
        let ledger = Ledger::default();
        let mut values = Vec::with_capacity(4096);
        values.extend([3, 5, 8]);
        let values = SharedVec::from(values);
        let (sequence, _group, _registry) = captured_sequence(&values, &ledger);
        assert_eq!(
            sequence.canonical_backing().unwrap().as_ptr(),
            values.as_ptr()
        );
        assert_eq!(sequence.capacity(), 4096);
        assert_eq!(sequence.view().get(1), Some(5));
    }

    #[test]
    fn appending_to_owned_spare_capacity_does_not_copy_the_existing_members() {
        let ledger = Ledger::default();
        let values = Vec::<u64>::with_capacity(8);
        let debit = ledger
            .retain(Usage::new(values.capacity() * size_of::<u64>(), 0), accept)
            .unwrap();
        let mut sequence = Sequence::staged(values, debit, map);
        let copies = Cell::new(0);
        for value in 0..8 {
            sequence
                .insert(sequence.len(), &[value], false, &ledger, accept, || {
                    copies.set(copies.get() + 1);
                    Ok(())
                })
                .unwrap();
        }
        assert_eq!(copies.get(), 8);
        assert_eq!(sequence.staged_values().unwrap(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn nested_same_sequence_edits_restore_capacity_and_each_intermediate_value() {
        let ledger = Ledger::default();
        let values = SharedVec::from(vec![3u32, 5]);
        let (mut sequence, _group, _registry) = captured_sequence(&values, &ledger);
        let outer = sequence
            .insert(2, &[8], true, &ledger, accept, || Ok(()))
            .unwrap()
            .unwrap();
        let inner = sequence
            .remove(0, 1, true, &ledger, accept, || Ok(()))
            .unwrap()
            .unwrap();
        sequence.restore(inner);
        assert_eq!(sequence.staged_values().unwrap(), &[3, 5, 8]);
        sequence.restore(outer);
        assert!(sequence.staged_values().is_none());
        assert_eq!(
            sequence.canonical_backing().unwrap().as_ptr(),
            values.as_ptr()
        );
        assert_eq!(sequence.capacity(), values.capacity());
    }

    #[test]
    fn refused_materialization_preserves_original_backing_and_custody() {
        let ledger = Ledger::default();
        let values = SharedVec::from(vec![3u32, 5]);
        let (mut sequence, _group, _registry) = captured_sequence(&values, &ledger);
        let before = ledger.total();
        assert!(
            sequence
                .insert(
                    2,
                    &[8],
                    true,
                    &ledger,
                    |_| Err(DomError::Cancelled),
                    || Ok(())
                )
                .is_err()
        );
        assert_eq!(ledger.total(), before);
        assert_eq!(
            sequence.canonical_backing().unwrap().as_ptr(),
            values.as_ptr()
        );
    }

    #[test]
    fn replacement_metrics_separate_allocated_capacity_from_initialized_members() {
        use super::super::super::scale_metrics::{self, Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let mut values = Vec::with_capacity(8);
        values.extend([3u64, 5, 8]);
        let debit = ledger
            .retain(Usage::new(values.capacity() * size_of::<u64>(), 0), accept)
            .unwrap();
        let sequence = Sequence::staged(values, debit, map);
        let measurement = Measurement::begin().unwrap();
        let replacement = sequence
            .replacement(9, &ledger, &mut accept, &mut || Ok(()))
            .unwrap()
            .unwrap();
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(replacement.staged_values().unwrap(), &[3, 5, 8]);
        assert!(replacement.capacity() >= 16);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            replacement.capacity() * size_of::<u64>()
        );
        assert_relation_copies(&counters, 3 * size_of::<u64>());

        let measurement = Measurement::begin().unwrap();
        assert!(
            sequence
                .replacement(4, &ledger, &mut accept, &mut || Ok(()))
                .unwrap()
                .is_none()
        );
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, 0);
    }

    #[test]
    fn failed_replacement_metrics_preserve_the_completed_copy_prefix() {
        use super::super::super::scale_metrics::{self, Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let values = SharedVec::from(vec![3u32, 5, 8]);
        let (sequence, _group, _registry) = captured_sequence(&values, &ledger);
        let before = ledger.total();
        let mut steps = 0;
        let measurement = Measurement::begin().unwrap();
        let refused = sequence.replacement(5, &ledger, &mut accept, &mut || {
            steps += 1;
            if steps == 3 {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(sequence.view().iter().collect::<Vec<_>>(), vec![3, 5, 8]);
        assert!(sequence.canonical_backing().unwrap().same_storage(&values));
        assert_eq!(ledger.total(), before);
        assert_eq!(counters.structural_allocations, 1);
        assert!(counters.structural_bytes >= 5 * size_of::<u64>());
        assert_relation_copies(&counters, 2 * size_of::<u64>());

        let measurement = Measurement::begin().unwrap();
        let refused = sequence.replacement(
            5,
            &ledger,
            &mut |_| Err(DomError::Cancelled),
            &mut || Ok(()),
        );
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(ledger.total(), before);
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, 0);
    }

    #[test]
    fn in_place_edit_and_inverse_metrics_count_completed_writes_and_tail_movement() {
        use super::super::super::scale_metrics::{self, Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let mut values = Vec::with_capacity(16);
        values.extend([10u64, 20, 30, 40]);
        let debit = ledger
            .retain(Usage::new(values.capacity() * size_of::<u64>(), 0), accept)
            .unwrap();
        let mut sequence = Sequence::staged(values, debit, map);

        let measurement = Measurement::begin().unwrap();
        let undo = sequence
            .insert(1, &[7, 8], true, &ledger, accept, || Ok(()))
            .unwrap()
            .unwrap();
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(sequence.staged_values().unwrap(), &[10, 7, 8, 20, 30, 40]);
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, 5 * size_of::<u64>());

        let measurement = Measurement::begin().unwrap();
        sequence.restore(undo);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(sequence.staged_values().unwrap(), &[10, 20, 30, 40]);
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, 3 * size_of::<u64>());

        let measurement = Measurement::begin().unwrap();
        let undo = sequence
            .remove(1, 2, true, &ledger, accept, || Ok(()))
            .unwrap()
            .unwrap();
        let allocated = match &undo {
            SequenceUndo::Removed { values, .. } => values.value().capacity() * size_of::<u64>(),
            _ => panic!("retained in-place removal owns its actual inverse backing"),
        };
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(sequence.staged_values().unwrap(), &[10, 40]);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(counters.structural_bytes, allocated);
        assert_relation_copies(&counters, 3 * size_of::<u64>());

        let measurement = Measurement::begin().unwrap();
        sequence.restore(undo);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(sequence.staged_values().unwrap(), &[10, 20, 30, 40]);
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, 3 * size_of::<u64>());

        let measurement = Measurement::begin().unwrap();
        assert!(
            sequence
                .remove(3, 1, false, &ledger, accept, || Ok(()))
                .unwrap()
                .is_none()
        );
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(sequence.staged_values().unwrap(), &[10, 20, 30]);
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, 0);

        let measurement = Measurement::begin().unwrap();
        sequence
            .insert(3, &[50], false, &ledger, accept, || Ok(()))
            .unwrap();
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(sequence.staged_values().unwrap(), &[10, 20, 30, 50]);
        assert_eq!(counters.structural_allocations, 0);
        assert_relation_copies(&counters, size_of::<u64>());
    }

    #[test]
    fn cancelled_inverse_copy_reports_its_prefix_without_mutating_the_original_sequence() {
        use super::super::super::scale_metrics::{self, Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let mut values = Vec::with_capacity(8);
        values.extend([1u64, 2, 3, 4]);
        let debit = ledger
            .retain(Usage::new(values.capacity() * size_of::<u64>(), 0), accept)
            .unwrap();
        let mut sequence = Sequence::staged(values, debit, map);
        let pointer = sequence.staged_values().unwrap().as_ptr();
        let before = ledger.total();
        let mut steps = 0;
        let measurement = Measurement::begin().unwrap();
        let refused = sequence.remove(1, 3, true, &ledger, accept, || {
            steps += 1;
            if steps == 3 {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(sequence.staged_values().unwrap(), &[1, 2, 3, 4]);
        assert_eq!(sequence.staged_values().unwrap().as_ptr(), pointer);
        assert_eq!(ledger.total(), before);
        assert_eq!(counters.structural_allocations, 1);
        assert!(counters.structural_bytes >= 3 * size_of::<u64>());
        assert_relation_copies(&counters, 2 * size_of::<u64>());
    }

    #[test]
    fn relation_copy_metrics_measure_append_growth_separately_from_real_journal_movement() {
        use super::super::super::scale_metrics::{self, Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let mut sequence = Sequence::empty(map, &ledger, accept).unwrap();
        let journal = super::super::Journal::new(&ledger, accept).unwrap();
        let measurement = Measurement::begin().unwrap();
        let mark = journal.mark(&ledger, accept).unwrap();
        for value in 0..16u64 {
            journal.prepare(&ledger, accept).unwrap();
            sequence
                .insert(sequence.len(), &[value], false, &ledger, accept, || Ok(()))
                .unwrap();
            journal.push_prepared(value);
        }
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(
            sequence.view().iter().collect::<Vec<_>>(),
            (0..16).collect::<Vec<_>>()
        );
        assert!(counters.relation_sequence_materializations > 0);
        assert!(counters.relation_sequence_materialized_bytes >= 16 * size_of::<u64>());
        // Append initializes each member once; geometric growth copies fewer
        // than twice the final member count, independently of journal growth.
        assert!(counters.relation_sequence_materialized_bytes < 3 * 16 * size_of::<u64>());
        assert!(
            counters.sequence_materialized_bytes > counters.relation_sequence_materialized_bytes
        );
        journal.rollback(mark, |_| {});
    }
}
