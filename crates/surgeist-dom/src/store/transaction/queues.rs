//! Lazy owner queues retain canonical backing until the reached queue changes.
//! Each inverse retains only a moved backing or the affected sequence span.

use super::super::*;
use super::PublicationCommitted;
#[cfg(test)]
pub(in crate::store) mod admission_observation;
mod associations;
mod targets;
use targets::Targets;
pub(in crate::store) use targets::{CanonicalTargets, FinishedTargets, TargetsCustody};

#[cfg(test)]
#[inline]
fn metric(event: scale_metrics::Event) {
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::Entered(
            scale_metrics::Domain::Transaction,
        ));
        scale_metrics::record(event);
    }
}

// A failed materialization still copied its completed prefix. This guard
// reports initialized/copied bytes, independently of reserved spare capacity.
#[cfg(test)]
#[derive(Default)]
struct CopiedBytes(usize);
#[cfg(test)]
impl Drop for CopiedBytes {
    fn drop(&mut self) {
        if self.0 != 0 {
            metric(scale_metrics::Event::SequenceMaterialized { bytes: self.0 });
        }
    }
}

#[cfg(test)]
fn completed<T>(records: usize) {
    let bytes = records
        .checked_mul(size_of::<T>())
        .expect("actual queue element extent");
    if bytes != 0 {
        metric(scale_metrics::Event::SequenceMaterialized { bytes });
    }
}

enum DocumentSource {
    CurrentTarget,
    Fixed(Option<DocumentId>),
}
fn no_document<S>(_: S) -> DocumentSource {
    DocumentSource::Fixed(None)
}
fn current_document<S>(_: S) -> DocumentSource {
    DocumentSource::CurrentTarget
}
struct Model<C, S> {
    map: fn(&C) -> S,
    measure: fn(S) -> QueueCounts,
    target: fn(S) -> Option<Key>,
    document: fn(S) -> DocumentSource,
}
fn no_target<S>(_: S) -> Option<Key> {
    None
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::store) struct QueueCounts {
    pub selected: usize,
    pub unissued: u64,
}
impl QueueCounts {
    fn add(self, other: Self) -> Result<Self, DomError> {
        Ok(Self {
            selected: self
                .selected
                .checked_add(other.selected)
                .ok_or(DomError::RepresentationExhausted)?,
            unissued: self
                .unissued
                .checked_add(other.unissued)
                .ok_or(DomError::RepresentationExhausted)?,
        })
    }
    fn sub(self, other: Self) -> Result<Self, DomError> {
        Ok(Self {
            selected: self
                .selected
                .checked_sub(other.selected)
                .ok_or(DomError::RepresentationExhausted)?,
            unissued: self
                .unissued
                .checked_sub(other.unissued)
                .ok_or(DomError::RepresentationExhausted)?,
        })
    }
}

enum Backing<C, S> {
    Canonical(custody::Held<SharedVec<C>>),
    Staged(custody::Held<Vec<S>>),
}
struct Published<C> {
    values: custody::Held<SharedVec<C>>,
    selected: usize,
}
pub(in crate::store) struct LazyQueue<C, S> {
    backing: Backing<C, S>,
    map: fn(&C) -> S,
    counts: QueueCounts,
    measure: fn(S) -> QueueCounts,
    target: fn(S) -> Option<Key>,
    document: fn(S) -> DocumentSource,
    targets: Option<Targets>,
    published: Option<Published<C>>,
}
enum Inverse<C, S> {
    Projection,
    Backing(Backing<C, S>),
    Inserted {
        index: usize,
        count: usize,
    },
    Removed {
        index: usize,
        values: custody::Held<Vec<S>>,
    },
    Retained {
        removed: custody::Held<Vec<(usize, S)>>,
    },
}
pub(in crate::store) struct QueueInverse<C, S> {
    published: Option<Published<C>>,
    targets: Option<Targets>,
    inverse: Inverse<C, S>,
    counts: QueueCounts,
}

impl<C, S> QueueInverse<C, S> {
    #[cfg(test)]
    fn journal_bytes(&self) -> usize {
        match &self.inverse {
            Inverse::Backing(Backing::Staged(values)) => values.debit().usage().bytes(),
            Inverse::Removed { values, .. } => values.debit().usage().bytes(),
            Inverse::Retained { removed } => removed.debit().usage().bytes(),
            Inverse::Projection
            | Inverse::Backing(Backing::Canonical(_))
            | Inverse::Inserted { .. } => 0,
        }
    }
}

enum IncomingTargets {
    None,
    One(Option<Key>),
    Many(custody::Held<Vec<Option<Key>>>),
}
struct PreparedInsert<C, S> {
    index: usize,
    count: usize,
    counts: QueueCounts,
    previous: QueueCounts,
    length: usize,
    replacement: Option<Backing<C, S>>,
    targets: Option<Targets>,
    incoming_targets: IncomingTargets,
}
struct PreparedRemoval<C, S> {
    index: usize,
    end: usize,
    counts: QueueCounts,
    previous: QueueCounts,
    length: usize,
    replacement: Option<Backing<C, S>>,
    removed: Option<custody::Held<Vec<S>>>,
    targets: Option<Targets>,
}
/// One contiguous queue removal, including its rollback-retention requirement.
struct Removal {
    index: usize,
    count: usize,
    retain_undo: bool,
}
struct PreparedRetention<C, S> {
    counts: QueueCounts,
    previous: QueueCounts,
    length: usize,
    replacement: Option<Backing<C, S>>,
    removed: custody::Held<Vec<(usize, S)>>,
    targets: Option<Targets>,
}
pub(in crate::store) struct PreparedNativeInsert {
    prepared: PreparedInsert<select::Task<NodeId>, select::Pending>,
    undo: bool,
}
pub(in crate::store) struct PreparedDetailsInsert {
    prepared: PreparedInsert<crate::DomDetailsToggleIntention, details::Pending<Key>>,
    undo: bool,
}

/// Prepared before publication; its replacements only change fixed-size
/// sequence values. The owning ledger retains actual acknowledgement capacity.
pub(in crate::store) struct QueueAck<S, C = ()> {
    published: Option<Published<C>>,
    rows: custody::Held<Vec<(usize, S)>>,
}
pub(in crate::store) struct QueueOutput<C> {
    pub values: SharedVec<C>,
    pub output_debit: Option<custody::Debit>,
}
impl<C> QueueOutput<C> {
    pub fn canonical(values: SharedVec<C>) -> Self {
        Self {
            values,
            output_debit: None,
        }
    }
}
impl<S, C> QueueAck<S, C> {
    pub fn prepare(
        capacity: usize,
        ledger: &custody::Ledger,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        allocate(capacity, ledger, admit).map(|rows| Self {
            rows,
            published: None,
        })
    }
    pub fn publish_values(
        &mut self,
        values: Vec<C>,
        selected: usize,
        budget: Option<&source::formation::Formation>,
        ledger: &custody::Ledger,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<QueueOutput<C>, DomError> {
        if selected > values.len() {
            return Err(DomError::RepresentationExhausted);
        }
        let bytes = if values.capacity() == 0 {
            0
        } else {
            values
                .capacity()
                .checked_mul(size_of::<C>())
                .and_then(|n| n.checked_add(size_of::<Vec<C>>()))
                .ok_or(DomError::RepresentationExhausted)?
        };
        let debit = ledger.retain(custody::Usage::new(bytes, 0), admit)?;
        let output_debit = budget.and_then(|budget| {
            budget
                .split_payload(source::storage::Payload {
                    bytes: values.capacity() * size_of::<C>(),
                    units: 0,
                })
                .into_debit()
        });
        let published = SharedVec::from(values);
        #[cfg(test)]
        if published.has_backing() {
            metric(scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<Vec<C>>(),
            });
        }
        self.published = Some(Published {
            values: custody::Held::new(published.clone(), debit),
            selected,
        });
        Ok(QueueOutput {
            values: published,
            output_debit,
        })
    }
    pub fn push_prepared(&mut self, index: usize, value: S) {
        assert!(self.rows.value().len() < self.rows.value().capacity());
        self.rows.value_mut().push((index, value));
        #[cfg(test)]
        completed::<(usize, S)>(1);
    }
}

/// End temporary formation classification while retaining the actual returned
/// buffer until its consumer has moved the values and released the allocation.
pub(in crate::store) fn finish_vec<T>(
    values: Vec<T>,
    budget: Option<&source::formation::Formation>,
    ledger: &custody::Ledger,
    admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<custody::Held<Vec<T>>, DomError> {
    let bytes = values
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    let debit = match budget.and_then(|budget| {
        budget
            .split_payload(source::storage::Payload { bytes, units: 0 })
            .into_debit()
    }) {
        Some(debit) => debit,
        None => ledger.retain(custody::Usage::new(bytes, 0), admit)?,
    };
    Ok(custody::Held::new(values, debit))
}

fn allocate<T>(
    capacity: usize,
    ledger: &custody::Ledger,
    mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<custody::Held<Vec<T>>, DomError> {
    let bytes = capacity
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    let mut debit = ledger.retain(custody::Usage::new(bytes, 0), &mut admit)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if values.capacity() != 0 && size_of::<T>() != 0 {
        // Every allocation through this factory is an actual queue, inverse,
        // acknowledgement or scratch Vec backing, never a retained alias.
        metric(scale_metrics::Event::StructuralAllocation {
            bytes: values.capacity() * size_of::<T>(),
        });
    }
    let actual = values
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    if actual > bytes {
        debit.grow(custody::Usage::new(actual - bytes, 0), admit)?;
    }
    Ok(custody::Held::new(values, debit))
}

/// Backing-only characterization: target/document indexes are covered by the
/// real Draft append test. This exercises the actual prepared buffer producer.
#[cfg(test)]
pub(in crate::store) fn assert_details_backing_boundary(values: [details::Pending<Key>; 2]) {
    let accept = |_: custody::Usage| Ok(());
    for short in [true, false] {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let mut queue = LazyQueue::empty(
            Model {
                map: details,
                measure: detail_count,
                target: details_target,
                document: details_document,
            },
            &registry,
            accept,
        )
        .unwrap();
        let first = queue
            .prepare_insert(
                0,
                &values[..1],
                |_| Ok(None),
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap();
        queue.commit_insert(first, &values[..1], false);
        let Backing::Staged(backing) = &queue.backing else {
            unreachable!()
        };
        assert_eq!(backing.value().capacity(), 1);
        let pointer = backing.value().as_ptr();
        let baseline = registry.ledger().total();
        let limit = baseline.bytes() + 2 * size_of::<details::Pending<Key>>() - usize::from(short);
        let prepared = queue.prepare_insert(
            1,
            &values[1..],
            |_| Ok(None),
            registry.ledger(),
            |usage| admit(usage.bytes(), limit, DomResource::SourceRetainedBytes),
            || Ok(()),
        );
        if short {
            assert!(
                matches!(prepared, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit: actual }) if actual == limit)
            );
            assert_eq!(queue.len(), 1);
            let Backing::Staged(backing) = &queue.backing else {
                unreachable!()
            };
            assert_eq!(backing.value().as_ptr(), pointer);
            assert_eq!(registry.ledger().total(), baseline);
        } else {
            queue.commit_insert(prepared.unwrap(), &values[1..], false);
            assert_eq!(queue.len(), 2);
            assert_eq!(queue.capacity(), 2);
            assert_eq!(queue.get(0).unwrap().target(), values[0].target());
            assert_eq!(queue.get(1).unwrap().target(), values[1].target());
        }
    }
}
impl<C, S: Copy> LazyQueue<C, S> {
    fn capture(
        values: &SharedVec<C>,
        model: Model<C, S>,
        selected: usize,
        targets: Option<Targets>,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let bytes = values.retained_bytes()?;
        let debit = registry
            .ledger()
            .retain(custody::Usage::new(bytes, 0), admit)?;
        Ok(Self {
            backing: Backing::Canonical(custody::Held::new(values.clone(), debit)),
            map: model.map,
            target: model.target,
            document: model.document,
            targets,
            published: None,
            counts: QueueCounts {
                selected,
                unissued: 0,
            },
            measure: model.measure,
        })
    }
    fn empty(
        model: Model<C, S>,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        Ok(Self {
            backing: Backing::Staged(allocate(0, registry.ledger(), admit)?),
            map: model.map,
            counts: QueueCounts::default(),
            measure: model.measure,
            target: model.target,
            document: model.document,
            targets: None,
            published: None,
        })
    }
    pub fn len(&self) -> usize {
        match &self.backing {
            Backing::Canonical(values) => values.value().len(),
            Backing::Staged(values) => values.value().len(),
        }
    }
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn capacity(&self) -> usize {
        match &self.backing {
            Backing::Canonical(values) => values.value().capacity(),
            Backing::Staged(values) => values.value().capacity(),
        }
    }
    #[cfg(test)]
    pub fn staged_pointer(&self) -> Option<*const S> {
        match &self.backing {
            Backing::Staged(values) => Some(values.value().as_ptr()),
            Backing::Canonical(_) => None,
        }
    }
    pub fn get(&self, index: usize) -> Option<S> {
        let value = match &self.backing {
            Backing::Canonical(values) => values.value().get(index).map(self.map),
            Backing::Staged(values) => values.value().get(index).copied(),
        };
        #[cfg(test)]
        if value.is_some() {
            metric(scale_metrics::Event::QueueRecordVisited);
        }
        value
    }
    pub fn first(&self) -> Option<S> {
        self.get(0)
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = S> + DoubleEndedIterator + '_ {
        (0..self.len()).map(|index| self.get(index).expect("bounded queue index"))
    }
    pub fn target_count(&self, key: Key) -> usize {
        self.targets
            .as_ref()
            .map_or(0, |targets| targets.count(key))
    }
    fn commit_projection(&mut self, targets: Targets, undo: bool) -> Option<QueueInverse<C, S>> {
        let previous_targets = self.targets.replace(targets);
        let published = self.published.take();
        undo.then_some(QueueInverse {
            targets: previous_targets,
            published,
            counts: self.counts,
            inverse: Inverse::Projection,
        })
    }
    fn finish_targets(
        &self,
        id: &impl Fn(Key) -> NodeId,
        private: impl Fn(Key) -> bool,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<FinishedTargets, DomError> {
        self.targets
            .as_ref()
            .expect("live semantic queue target index")
            .finish(id, private, admit, step)
    }
    pub fn published_selected(&self) -> usize {
        self.published
            .as_ref()
            .map_or(self.counts.selected, |published| published.selected)
    }
    pub fn counts(&self) -> QueueCounts {
        self.counts
    }
    pub fn canonical_backing(&self) -> Option<&SharedVec<C>> {
        if let Some(published) = &self.published {
            return Some(published.values.value());
        }
        match &self.backing {
            Backing::Canonical(values) => Some(values.value()),
            Backing::Staged(_) => None,
        }
    }
    #[cfg(test)]
    pub fn staged_values(&self) -> Option<&[S]> {
        match &self.backing {
            Backing::Canonical(_) => None,
            Backing::Staged(values) => Some(values.value()),
        }
    }
    fn replacement(
        &self,
        needed: usize,
        ledger: &custody::Ledger,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Backing<C, S>>, DomError> {
        if matches!(self.backing, Backing::Staged(_)) && needed <= self.capacity() {
            return Ok(None);
        }
        let capacity = needed.max(
            self.capacity()
                .checked_mul(2)
                .ok_or(DomError::RepresentationExhausted)?,
        );
        let mut next = allocate(capacity, ledger, &mut admit)?;
        #[cfg(test)]
        let mut copied = CopiedBytes::default();
        for value in self.iter() {
            step()?;
            next.value_mut().push(value);
            #[cfg(test)]
            {
                copied.0 += size_of::<S>();
            }
        }
        Ok(Some(Backing::Staged(next)))
    }
    fn prepare_insert(
        &self,
        index: usize,
        values: &[S],
        mut document: impl FnMut(S) -> Result<Option<Key>, DomError>,
        ledger: &custody::Ledger,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<PreparedInsert<C, S>, DomError> {
        if index > self.len() {
            return Err(DomError::IndexSize);
        }
        let needed = self
            .len()
            .checked_add(values.len())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut counts = self.counts;
        for value in values {
            #[cfg(test)]
            metric(scale_metrics::Event::QueueRecordVisited);
            step()?;
            counts = counts.add((self.measure)(*value))?;
        }
        for _ in index..self.len() {
            step()?;
        }
        let incoming_targets = if self.targets.is_none() || values.is_empty() {
            IncomingTargets::None
        } else if values.len() == 1 {
            IncomingTargets::One((self.target)(values[0]))
        } else {
            let mut targets = allocate(values.len(), ledger, &mut admit)?;
            #[cfg(test)]
            let mut copied = CopiedBytes::default();
            for value in values {
                #[cfg(test)]
                metric(scale_metrics::Event::QueueRecordVisited);
                step()?;
                targets.value_mut().push((self.target)(*value));
                #[cfg(test)]
                {
                    copied.0 += size_of::<Option<Key>>();
                }
            }
            IncomingTargets::Many(targets)
        };
        let targets = match &self.targets {
            Some(targets) => Some(targets.updated(
                values.iter().copied().filter_map(|value| {
                    (self.target)(value).map(|key| document(value).map(|document| (key, document)))
                }),
                true,
                &mut admit,
                &mut step,
            )?),
            None => None,
        };
        let replacement = if values.is_empty() {
            None
        } else {
            self.replacement(needed, ledger, &mut admit, &mut step)?
        };
        Ok(PreparedInsert {
            index,
            count: values.len(),
            counts,
            previous: self.counts,
            length: self.len(),
            replacement,
            targets,
            incoming_targets,
        })
    }
    fn commit_insert(
        &mut self,
        prepared: PreparedInsert<C, S>,
        values: &[S],
        retain_undo: bool,
    ) -> Option<QueueInverse<C, S>> {
        assert_eq!(self.len(), prepared.length);
        assert_eq!(self.counts, prepared.previous);
        assert_eq!(values.len(), prepared.count);
        match &prepared.incoming_targets {
            IncomingTargets::None => {}
            IncomingTargets::One(expected) => assert_eq!(
                *expected,
                (self.target)(values[0]),
                "prepared queue target identity"
            ),
            IncomingTargets::Many(targets) => {
                for (expected, value) in targets.value().iter().zip(values) {
                    #[cfg(test)]
                    metric(scale_metrics::Event::QueueRecordVisited);
                    assert_eq!(
                        *expected,
                        (self.target)(*value),
                        "prepared queue target identities"
                    );
                }
            }
        }
        let mut counts = self.counts;
        for value in values {
            #[cfg(test)]
            metric(scale_metrics::Event::QueueRecordVisited);
            counts = counts
                .add((self.measure)(*value))
                .expect("prepared queue value counts");
        }
        assert_eq!(counts, prepared.counts);
        if values.is_empty() {
            return None;
        }
        let old = prepared
            .replacement
            .map(|next| std::mem::replace(&mut self.backing, next));
        let Backing::Staged(staged) = &mut self.backing else {
            unreachable!("materialized queue")
        };
        staged
            .value_mut()
            .splice(prepared.index..prepared.index, values.iter().copied());
        #[cfg(test)]
        metric(scale_metrics::Event::QueueRecordsMoved {
            records: prepared.length - prepared.index,
        });
        #[cfg(test)]
        completed::<S>(values.len() + prepared.length - prepared.index);
        let previous = std::mem::replace(&mut self.counts, counts);
        let targets = std::mem::replace(&mut self.targets, prepared.targets);
        let published = self.published.take();
        if retain_undo {
            Some(QueueInverse {
                published,
                targets,
                inverse: match old {
                    Some(old) => Inverse::Backing(old),
                    None => Inverse::Inserted {
                        index: prepared.index,
                        count: values.len(),
                    },
                },
                counts: previous,
            })
        } else {
            None
        }
    }
    #[cfg(test)]
    fn insert(
        &mut self,
        index: usize,
        values: &[S],
        retain_undo: bool,
        ledger: &custody::Ledger,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<QueueInverse<C, S>>, DomError> {
        let prepared = self.prepare_insert(index, values, |_| Ok(None), ledger, admit, step)?;
        Ok(self.commit_insert(prepared, values, retain_undo))
    }
    fn prepare_remove(
        &self,
        removal: Removal,
        mut document: impl FnMut(S) -> Result<Option<Key>, DomError>,
        mut association: impl FnMut(S) -> Result<Option<(Key, Key)>, DomError>,
        ledger: &custody::Ledger,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<PreparedRemoval<C, S>>, DomError> {
        let Removal {
            index,
            count,
            retain_undo,
        } = removal;
        let end = index
            .checked_add(count)
            .filter(|end| *end <= self.len())
            .ok_or(DomError::IndexSize)?;
        if count == 0 {
            return Ok(None);
        }
        let mut counts = self.counts;
        for value in self.iter().skip(index).take(count) {
            step()?;
            counts = counts.sub((self.measure)(value))?;
        }
        let targets = match &self.targets {
            Some(targets) => Some(targets.updated(
                self.iter().skip(index).take(count).filter_map(|value| {
                    (self.target)(value).map(|key| document(value).map(|document| (key, document)))
                }),
                false,
                &mut admit,
                &mut step,
            )?),
            None => None,
        };
        let targets =
            match targets {
                Some(targets) => Some(targets.change_associations(
                    self.iter().skip(index).take(count).filter_map(|value| {
                        match association(value) {
                            Ok(Some((target, document))) => Some(Ok((target, document, 1))),
                            Ok(None) => None,
                            Err(error) => Some(Err(error)),
                        }
                    }),
                    false,
                    &mut admit,
                    &mut step,
                )?),
                None => None,
            };
        let replacement = self.replacement(self.len(), ledger, &mut admit, &mut step)?;
        let removed = if retain_undo && replacement.is_none() {
            let mut removed = allocate(count, ledger, &mut admit)?;
            #[cfg(test)]
            let mut copied = CopiedBytes::default();
            for value in self.iter().skip(index).take(count) {
                step()?;
                removed.value_mut().push(value);
                #[cfg(test)]
                {
                    copied.0 += size_of::<S>();
                }
            }
            Some(removed)
        } else {
            None
        };
        for _ in index..self.len() {
            step()?;
        }
        Ok(Some(PreparedRemoval {
            index,
            end,
            counts,
            previous: self.counts,
            length: self.len(),
            replacement,
            removed,
            targets,
        }))
    }
    fn commit_remove(
        &mut self,
        prepared: PreparedRemoval<C, S>,
        retain_undo: bool,
    ) -> Option<QueueInverse<C, S>> {
        assert_eq!(self.len(), prepared.length);
        assert_eq!(self.counts, prepared.previous);
        let PreparedRemoval {
            index,
            end,
            counts,
            replacement,
            removed,
            targets,
            ..
        } = prepared;
        let old = replacement.map(|next| std::mem::replace(&mut self.backing, next));
        let Backing::Staged(staged) = &mut self.backing else {
            unreachable!("materialized queue")
        };
        staged.value_mut().drain(index..end);
        #[cfg(test)]
        metric(scale_metrics::Event::QueueRecordsMoved {
            records: self.len() - index,
        });
        #[cfg(test)]
        completed::<S>(self.len() - index);
        let previous = std::mem::replace(&mut self.counts, counts);
        let targets = std::mem::replace(&mut self.targets, targets);
        let published = self.published.take();
        if retain_undo {
            Some(QueueInverse {
                published,
                targets,
                inverse: match old {
                    Some(old) => Inverse::Backing(old),
                    None => Inverse::Removed {
                        index,
                        values: removed.expect("preadmitted removed span"),
                    },
                },
                counts: previous,
            })
        } else {
            None
        }
    }
    fn restore(&mut self, inverse: QueueInverse<C, S>) {
        match inverse.inverse {
            Inverse::Projection => {}
            Inverse::Backing(old) => self.backing = old,
            Inverse::Inserted { index, count } => {
                let Backing::Staged(values) = &mut self.backing else {
                    unreachable!("staged inverse")
                };
                values.value_mut().drain(index..index + count);
                #[cfg(test)]
                metric(scale_metrics::Event::QueueRecordsMoved {
                    records: values.value().len() - index,
                });
                #[cfg(test)]
                completed::<S>(values.value().len() - index);
            }
            Inverse::Removed { index, values } => {
                let Backing::Staged(staged) = &mut self.backing else {
                    unreachable!("staged inverse")
                };
                staged
                    .value_mut()
                    .splice(index..index, values.value().iter().copied());
                #[cfg(test)]
                metric(scale_metrics::Event::QueueRecordsMoved {
                    records: staged.value().len() - values.value().len() - index,
                });
                #[cfg(test)]
                completed::<S>(staged.value().len() - index);
            }
            Inverse::Retained { removed } => {
                let Backing::Staged(staged) = &mut self.backing else {
                    unreachable!("staged inverse")
                };
                let values = staged.value_mut();
                let mut kept = values.len();
                let mut removed_index = removed.value().len();
                let original = kept + removed_index;
                values.resize(original, removed.value()[0].1);
                #[cfg(test)]
                let mut copied = CopiedBytes(removed_index * size_of::<S>());
                for destination in (0..original).rev() {
                    #[cfg(test)]
                    metric(scale_metrics::Event::QueueRecordVisited);
                    if removed_index != 0 && removed.value()[removed_index - 1].0 == destination {
                        removed_index -= 1;
                        values[destination] = removed.value()[removed_index].1;
                    } else {
                        kept -= 1;
                        values[destination] = values[kept];
                    }
                    #[cfg(test)]
                    {
                        copied.0 += size_of::<S>();
                    }
                }
            }
        }
        self.counts = inverse.counts;
        self.targets = inverse.targets;
        self.published = inverse.published;
    }
    fn prepare_retain(
        &self,
        keep: impl Fn(S) -> bool,
        mut document: impl FnMut(S) -> Result<Option<Key>, DomError>,
        mut association: impl FnMut(S) -> Result<Option<(Key, Key)>, DomError>,
        ledger: &custody::Ledger,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<PreparedRetention<C, S>>, DomError> {
        let mut removed_count = 0usize;
        let mut counts = self.counts;
        for value in self.iter() {
            step()?;
            if !keep(value) {
                removed_count += 1;
                counts = counts.sub((self.measure)(value))?;
            }
        }
        if removed_count == 0 {
            return Ok(None);
        }
        let mut removed = allocate(removed_count, ledger, &mut admit)?;
        #[cfg(test)]
        let mut copied = CopiedBytes::default();
        for (index, value) in self.iter().enumerate() {
            step()?;
            if !keep(value) {
                removed.value_mut().push((index, value));
                #[cfg(test)]
                {
                    copied.0 += size_of::<(usize, S)>();
                }
            }
        }
        let targets = match &self.targets {
            Some(targets) => Some(targets.updated(
                removed.value().iter().filter_map(|(_, value)| {
                    #[cfg(test)]
                    metric(scale_metrics::Event::QueueRecordVisited);
                    (self.target)(*value)
                        .map(|key| document(*value).map(|document| (key, document)))
                }),
                false,
                &mut admit,
                &mut step,
            )?),
            None => None,
        };
        let targets = match targets {
            Some(targets) => Some(targets.change_associations(
                removed.value().iter().filter_map(|(_, value)| {
                    #[cfg(test)]
                    metric(scale_metrics::Event::QueueRecordVisited);
                    match association(*value) {
                        Ok(Some((target, document))) => Some(Ok((target, document, 1))),
                        Ok(None) => None,
                        Err(error) => Some(Err(error)),
                    }
                }),
                false,
                &mut admit,
                &mut step,
            )?),
            None => None,
        };
        let replacement = self.replacement(self.len(), ledger, &mut admit, &mut step)?;
        Ok(Some(PreparedRetention {
            counts,
            previous: self.counts,
            length: self.len(),
            replacement,
            removed,
            targets,
        }))
    }
    fn commit_retain(
        &mut self,
        prepared: PreparedRetention<C, S>,
        retain_undo: bool,
    ) -> Option<QueueInverse<C, S>> {
        assert_eq!(self.len(), prepared.length);
        assert_eq!(self.counts, prepared.previous);
        let PreparedRetention {
            counts,
            replacement,
            removed,
            targets,
            ..
        } = prepared;
        let old = replacement.map(|next| std::mem::replace(&mut self.backing, next));
        let Backing::Staged(staged) = &mut self.backing else {
            unreachable!("materialized queue")
        };
        let mut current = 0usize;
        let mut removed_index = 0usize;
        #[cfg(test)]
        let mut moved = 0usize;
        staged.value_mut().retain(|_| {
            #[cfg(test)]
            metric(scale_metrics::Event::QueueRecordVisited);
            let remove = removed_index < removed.value().len()
                && removed.value()[removed_index].0 == current;
            current += 1;
            if remove {
                removed_index += 1;
            }
            #[cfg(test)]
            if !remove && removed_index != 0 {
                moved += 1;
            }
            !remove
        });
        #[cfg(test)]
        metric(scale_metrics::Event::QueueRecordsMoved { records: moved });
        #[cfg(test)]
        completed::<S>(moved);
        let previous = std::mem::replace(&mut self.counts, counts);
        let targets = std::mem::replace(&mut self.targets, targets);
        let published = self.published.take();
        if retain_undo {
            Some(QueueInverse {
                published,
                targets,
                inverse: match old {
                    Some(old) => Inverse::Backing(old),
                    None => Inverse::Retained { removed },
                },
                counts: previous,
            })
        } else {
            None
        }
    }
    #[cfg(test)]
    fn retain(
        &mut self,
        keep: impl Fn(S) -> bool,
        retain_undo: bool,
        ledger: &custody::Ledger,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<QueueInverse<C, S>>, DomError> {
        let Some(prepared) =
            self.prepare_retain(keep, |_| Ok(None), |_| Ok(None), ledger, admit, step)?
        else {
            return Ok(None);
        };
        Ok(self.commit_retain(prepared, retain_undo))
    }
    fn apply_ack(&mut self, ack: QueueAck<S, C>, _committed: &PublicationCommitted) {
        for (index, value) in ack.rows.value() {
            let old = self
                .get(*index)
                .expect("prepared queue acknowledgement index");
            assert_eq!(
                (self.target)(old),
                (self.target)(*value),
                "acknowledgement preserves queue target"
            );
            self.counts = self
                .counts
                .sub((self.measure)(old))
                .and_then(|counts| counts.add((self.measure)(*value)))
                .expect("prepared fixed-size acknowledgement counts");
        }
        if let Backing::Staged(staged) = &mut self.backing {
            #[cfg(test)]
            let mut copied = CopiedBytes::default();
            for (index, value) in ack.rows.value() {
                #[cfg(test)]
                metric(scale_metrics::Event::QueueRecordVisited);
                staged.value_mut()[*index] = *value;
                #[cfg(test)]
                {
                    copied.0 += size_of::<S>();
                }
            }
        } else {
            assert!(ack.rows.value().is_empty());
        }
        self.published = ack.published;
    }
    fn clear_committed(&mut self, _committed: &PublicationCommitted) {
        let Backing::Staged(values) = &mut self.backing else {
            unreachable!("cancellation queues are private staged queues")
        };
        values.value_mut().clear();
        self.counts = QueueCounts::default();
    }
}

fn signal_target(key: Key) -> Option<Key> {
    Some(key)
}
fn native_target(value: select::Pending) -> Option<Key> {
    match value {
        select::Pending::Update { target, .. } => Some(target),
        select::Pending::Slots => None,
    }
}
fn details_document(value: details::Pending<Key>) -> DocumentSource {
    DocumentSource::Fixed(value.queue_document())
}
fn details_target(value: details::Pending<Key>) -> Option<Key> {
    Some(value.target())
}
fn signal(value: &NodeId) -> Key {
    Key::Existing(*value)
}
fn native(value: &select::Task<NodeId>) -> select::Pending {
    match *value {
        select::Task::Slots => select::Pending::Slots,
        select::Task::Update { serial, target } => select::Pending::Update {
            serial: Some(serial),
            target: Key::Existing(target),
        },
    }
}
fn details(value: &crate::DomDetailsToggleIntention) -> details::Pending<Key> {
    details::Pending::from_published(value)
}
fn native_count(value: select::Pending) -> QueueCounts {
    match value {
        select::Pending::Slots => QueueCounts::default(),
        select::Pending::Update { serial, .. } => QueueCounts {
            selected: 1,
            unissued: u64::from(serial.is_none()),
        },
    }
}
fn copy<T: Copy>(value: &T) -> T {
    *value
}
fn one<T>(_: T) -> QueueCounts {
    QueueCounts {
        selected: 1,
        unissued: 0,
    }
}
fn detail_count<K>(value: details::Pending<K>) -> QueueCounts {
    QueueCounts {
        selected: 1,
        unissued: u64::from(value.is_unissued()),
    }
}
fn cancelled_select_count(value: (Option<u64>, NodeId)) -> QueueCounts {
    QueueCounts {
        selected: 1,
        unissued: u64::from(value.0.is_none()),
    }
}

macro_rules! wrapper {
    ($name:ident, $canonical:ty, $staged:ty, $map:expr, $measure:expr) => {
        pub(in crate::store) struct $name(LazyQueue<$canonical, $staged>);
        impl std::ops::Deref for $name {
            type Target = LazyQueue<$canonical, $staged>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}
wrapper!(Signals, NodeId, Key, signal, one::<Key>);
wrapper!(
    NativeTasks,
    select::Task<NodeId>,
    select::Pending,
    native,
    native_count
);
wrapper!(
    Details,
    crate::DomDetailsToggleIntention,
    details::Pending<Key>,
    details,
    detail_count::<Key>
);
wrapper!(
    CancelledSignals,
    NodeId,
    NodeId,
    copy::<NodeId>,
    one::<NodeId>
);
wrapper!(
    CancelledSelects,
    (Option<u64>, NodeId),
    (Option<u64>, NodeId),
    copy::<(Option<u64>, NodeId)>,
    cancelled_select_count
);
wrapper!(
    CancelledDetails,
    details::Pending<NodeId>,
    details::Pending<NodeId>,
    copy::<details::Pending<NodeId>>,
    detail_count::<NodeId>
);

macro_rules! capture_wrapper {
    ($name:ident,$canonical:ty,$map:expr,$measure:expr,$target:expr,$document:expr) => {
        impl $name {
            pub fn capture(
                values: &SharedVec<$canonical>,
                selected: usize,
                targets: &CanonicalTargets,
                targets_custody: &TargetsCustody,
                registry: &bank::CustodyRegistry,
                admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
            ) -> Result<Self, DomError> {
                LazyQueue::capture(
                    values,
                    Model {
                        map: $map,
                        measure: $measure,
                        target: $target,
                        document: $document,
                    },
                    selected,
                    Some(Targets::capture(targets, targets_custody, registry)),
                    registry,
                    admit,
                )
                .map(Self)
            }
        }
    };
}
macro_rules! empty_wrapper {
    ($name:ident,$map:expr,$measure:expr) => {
        impl $name {
            pub fn empty(
                registry: &bank::CustodyRegistry,
                admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
            ) -> Result<Self, DomError> {
                LazyQueue::empty(
                    Model {
                        map: $map,
                        measure: $measure,
                        target: no_target,
                        document: no_document,
                    },
                    registry,
                    admit,
                )
                .map(Self)
            }
        }
    };
}
capture_wrapper!(
    Signals,
    NodeId,
    signal,
    one::<Key>,
    signal_target,
    current_document::<Key>
);
capture_wrapper!(
    NativeTasks,
    select::Task<NodeId>,
    native,
    native_count,
    native_target,
    current_document::<select::Pending>
);
capture_wrapper!(
    Details,
    crate::DomDetailsToggleIntention,
    details,
    detail_count::<Key>,
    details_target,
    details_document
);
empty_wrapper!(CancelledSignals, copy::<NodeId>, one::<NodeId>);
empty_wrapper!(
    CancelledSelects,
    copy::<(Option<u64>, NodeId)>,
    cancelled_select_count
);
empty_wrapper!(
    CancelledDetails,
    copy::<details::Pending<NodeId>>,
    detail_count::<NodeId>
);

pub(in crate::store) struct FinishedSignals {
    pub values: SharedVec<NodeId>,
    pub output_debit: Option<custody::Debit>,
    pub ack: Option<QueueAck<Key, NodeId>>,
}

pub(in crate::store) struct QueueDocumentPlan {
    signals: Option<Targets>,
    native: Option<Targets>,
    undo: bool,
}
pub(in crate::store) struct QueueDocumentUndo {
    signals: Option<Targets>,
    native: Option<Targets>,
}
pub(in crate::store) struct QueueVisibilityPlan {
    signals: Option<Targets>,
    native: Option<Targets>,
    details: Option<Targets>,
    undo: bool,
}
pub(in crate::store) struct QueueVisibilityUndo {
    signals: Option<QueueInverse<NodeId, Key>>,
    native: Option<QueueInverse<select::Task<NodeId>, select::Pending>>,
    details: Option<QueueInverse<crate::DomDetailsToggleIntention, details::Pending<Key>>>,
}
#[expect(
    clippy::large_enum_variant,
    reason = "inline inverses retain admitted queue roots without an additional fallible boxed allocation"
)]
pub(in crate::store) enum QueueUndo {
    Visibility(QueueVisibilityUndo),
    Documents(QueueDocumentUndo),
    Signals(QueueInverse<NodeId, Key>),
    Native(QueueInverse<select::Task<NodeId>, select::Pending>),
    Details(QueueInverse<crate::DomDetailsToggleIntention, details::Pending<Key>>),
    CancelledSignals(QueueInverse<NodeId, NodeId>),
    CancelledSelects(QueueInverse<(Option<u64>, NodeId), (Option<u64>, NodeId)>),
    CancelledDetails(QueueInverse<details::Pending<NodeId>, details::Pending<NodeId>>),
}
impl QueueUndo {
    /// Private inverse buffers only. Shared semantic/index graph custody is
    /// reported by SourceUnion; fixed inverse headers live in journal capacity.
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        match self {
            Self::Documents(_) | Self::Visibility(_) => 0,
            Self::Signals(inverse) => inverse.journal_bytes(),
            Self::Native(inverse) => inverse.journal_bytes(),
            Self::Details(inverse) => inverse.journal_bytes(),
            Self::CancelledSignals(inverse) => inverse.journal_bytes(),
            Self::CancelledSelects(inverse) => inverse.journal_bytes(),
            Self::CancelledDetails(inverse) => inverse.journal_bytes(),
        }
    }
    pub fn restore(self, draft: &mut Draft) {
        match self {
            Self::Visibility(undo) => {
                if let Some(inverse) = undo.signals {
                    draft.signal_slots.restore(inverse);
                }
                if let Some(inverse) = undo.native {
                    draft.native_tasks.restore(inverse);
                }
                if let Some(inverse) = undo.details {
                    draft.details_intentions.restore(inverse);
                }
            }
            Self::Documents(undo) => {
                if let Some(signals) = undo.signals {
                    draft.signal_slots.targets = Some(signals);
                }
                if let Some(native) = undo.native {
                    draft.native_tasks.targets = Some(native);
                }
            }
            Self::Signals(inverse) => draft.signal_slots.restore(inverse),
            Self::Native(inverse) => draft.native_tasks.restore(inverse),
            Self::Details(inverse) => draft.details_intentions.restore(inverse),
            Self::CancelledSignals(inverse) => draft.cancelled_signals.restore(inverse),
            Self::CancelledSelects(inverse) => draft.cancelled_selects.restore(inverse),
            Self::CancelledDetails(inverse) => draft.cancelled_details.restore(inverse),
        }
        draft.counts.select_tasks = draft.native_tasks.counts().selected;
        draft.counts.details_intentions = draft.details_intentions.len();
    }
}

macro_rules! insert_operation {
    ($insert:ident,$field:ident,$item:ty,$variant:ident) => {
        impl Draft {
            pub(in crate::store) fn $insert(
                &mut self,
                index: usize,
                values: &[$item],
                cancel: &AtomicBool,
            ) -> Result<(), DomError> {
                if values.is_empty() {
                    return Ok(());
                }
                let undo = self.prepare_undo(cancel)?;
                let formation = source::formation::Formation::new(self, cancel)?;
                let ledger = self.custody_ledger().clone();
                let prepared = self.$field.prepare_insert(
                    index,
                    values,
                    |value| self.queue_document(&self.$field, value, cancel),
                    &ledger,
                    |usage| match &formation {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    || source::formation::step(formation.as_ref(), cancel),
                )?;
                let inverse = self.$field.commit_insert(prepared, values, undo);
                if let Some(inverse) = inverse {
                    self.record_queue_undo_prepared(QueueUndo::$variant(inverse));
                }
                self.counts.select_tasks = self.native_tasks.counts().selected;
                self.counts.details_intentions = self.details_intentions.len();
                Ok(())
            }
        }
    };
}
macro_rules! remove_operation {
    ($remove:ident,$field:ident,$item:ty,$variant:ident) => {
        impl Draft {
            pub(in crate::store) fn $remove(
                &mut self,
                index: usize,
                count: usize,
                cancel: &AtomicBool,
            ) -> Result<(), DomError> {
                if count == 0 {
                    return Ok(());
                }
                let undo = self.prepare_undo(cancel)?;
                let formation = source::formation::Formation::new(self, cancel)?;
                let ledger = self.custody_ledger().clone();
                let prepared = self.$field.prepare_remove(
                    Removal {
                        index,
                        count,
                        retain_undo: undo,
                    },
                    |value| self.queue_document(&self.$field, value, cancel),
                    |value| self.queue_association(&self.$field, value, cancel),
                    &ledger,
                    |usage| match &formation {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    || source::formation::step(formation.as_ref(), cancel),
                )?;
                let inverse =
                    prepared.and_then(|prepared| self.$field.commit_remove(prepared, undo));
                if let Some(inverse) = inverse {
                    self.record_queue_undo_prepared(QueueUndo::$variant(inverse));
                }
                self.counts.select_tasks = self.native_tasks.counts().selected;
                self.counts.details_intentions = self.details_intentions.len();
                Ok(())
            }
        }
    };
}
macro_rules! retain_operation {
    ($retain:ident,$field:ident,$item:ty,$variant:ident) => {
        impl Draft {
            pub(in crate::store) fn $retain(
                &mut self,
                keep: impl Fn($item) -> bool,
                cancel: &AtomicBool,
            ) -> Result<(), DomError> {
                let undo = self.prepare_undo(cancel)?;
                let formation = source::formation::Formation::new(self, cancel)?;
                let ledger = self.custody_ledger().clone();
                let prepared = self.$field.prepare_retain(
                    keep,
                    |value| self.queue_document(&self.$field, value, cancel),
                    |value| self.queue_association(&self.$field, value, cancel),
                    &ledger,
                    |usage| match &formation {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    || source::formation::step(formation.as_ref(), cancel),
                )?;
                let inverse =
                    prepared.and_then(|prepared| self.$field.commit_retain(prepared, undo));
                if let Some(inverse) = inverse {
                    self.record_queue_undo_prepared(QueueUndo::$variant(inverse));
                }
                self.counts.select_tasks = self.native_tasks.counts().selected;
                self.counts.details_intentions = self.details_intentions.len();
                Ok(())
            }
        }
    };
}
insert_operation!(queue_signals_insert, signal_slots, Key, Signals);
remove_operation!(queue_signals_remove, signal_slots, Key, Signals);
insert_operation!(queue_native_insert, native_tasks, select::Pending, Native);
remove_operation!(queue_native_remove, native_tasks, select::Pending, Native);
retain_operation!(queue_native_retain, native_tasks, select::Pending, Native);

remove_operation!(
    queue_details_remove,
    details_intentions,
    details::Pending<Key>,
    Details
);
retain_operation!(
    queue_details_retain,
    details_intentions,
    details::Pending<Key>,
    Details
);
insert_operation!(
    queue_cancelled_signals_insert,
    cancelled_signals,
    NodeId,
    CancelledSignals
);
insert_operation!(
    queue_cancelled_selects_insert,
    cancelled_selects,
    (Option<u64>, NodeId),
    CancelledSelects
);
insert_operation!(
    queue_cancelled_details_insert,
    cancelled_details,
    details::Pending<NodeId>,
    CancelledDetails
);
macro_rules! prepared_operation {
    ($prepare:ident,$commit:ident,$field:ident,$item:ty,$token:ident,$variant:ident) => {
        impl Draft {
            pub(in crate::store) fn $prepare(
                &mut self,
                index: usize,
                values: &[$item],
                cancel: &AtomicBool,
            ) -> Result<$token, DomError> {
                let undo = self.prepare_undo(cancel)?;
                let formation = source::formation::Formation::new(self, cancel)?;
                let ledger = self.custody_ledger().clone();
                let prepared = self.$field.prepare_insert(
                    index,
                    values,
                    |value| self.queue_document(&self.$field, value, cancel),
                    &ledger,
                    |usage| match &formation {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    || source::formation::step(formation.as_ref(), cancel),
                )?;
                Ok($token { prepared, undo })
            }
            pub(in crate::store) fn $commit(&mut self, prepared: $token, values: &[$item]) {
                if let Some(inverse) =
                    self.$field
                        .commit_insert(prepared.prepared, values, prepared.undo)
                {
                    self.record_queue_undo_prepared(QueueUndo::$variant(inverse));
                }
                self.counts.select_tasks = self.native_tasks.counts().selected;
                self.counts.details_intentions = self.details_intentions.len();
            }
        }
    };
}
prepared_operation!(
    queue_native_prepare_insert,
    queue_native_commit_insert,
    native_tasks,
    select::Pending,
    PreparedNativeInsert,
    Native
);
prepared_operation!(
    queue_details_prepare_insert,
    queue_details_commit_insert,
    details_intentions,
    details::Pending<Key>,
    PreparedDetailsInsert,
    Details
);

impl Draft {
    pub(in crate::store) fn prepare_queue_visibility_change(
        &self,
        target: Key,
        was_private: bool,
        is_private: bool,
        cancel: &AtomicBool,
    ) -> Result<QueueVisibilityPlan, DomError> {
        let target = self.live_key(target);
        self.source_step(cancel)?;
        let signal_count = self.signal_slots.target_count(target);
        let native_count = self.native_tasks.target_count(target);
        let details_count = self.details_intentions.target_count(target);
        if was_private == is_private
            || (signal_count == 0 && native_count == 0 && details_count == 0)
        {
            return Ok(QueueVisibilityPlan {
                signals: None,
                native: None,
                details: None,
                undo: false,
            });
        }
        let undo = self.prepare_undo(cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut admit = |usage| match &formation {
            Some(budget) => budget.admit_custody_usage(usage, cancel),
            None => check_cancel(cancel),
        };
        let mut step = || source::formation::step(formation.as_ref(), cancel);
        let document = if signal_count != 0 || native_count != 0 {
            Some(self.live_key(self.node(target)?.document))
        } else {
            None
        };
        let mut current = |targets: &Targets, count: usize| {
            if count == 0 {
                Ok(None)
            } else {
                let mut candidate = targets.change_documents(
                    [Ok((document.expect("selected current document"), count))],
                    !is_private,
                    &mut admit,
                    &mut step,
                )?;
                candidate.touch_target(target, &mut admit)?;
                Ok(Some(candidate))
            }
        };
        let signals = current(
            self.signal_slots
                .targets
                .as_ref()
                .expect("signal target owner"),
            signal_count,
        )?;
        let native = current(
            self.native_tasks
                .targets
                .as_ref()
                .expect("native target owner"),
            native_count,
        )?;
        let details = if details_count == 0 {
            None
        } else {
            let mut candidate = self
                .details_intentions
                .targets
                .as_ref()
                .expect("Details target owner")
                .change_associated_documents(
                    target,
                    &|document| self.live_key(document),
                    !is_private,
                    &mut admit,
                    &mut step,
                )?;
            candidate.touch_target(target, &mut admit)?;
            Some(candidate)
        };
        Ok(QueueVisibilityPlan {
            signals,
            native,
            details,
            undo,
        })
    }
    pub(in crate::store) fn commit_queue_visibility_change(&mut self, plan: QueueVisibilityPlan) {
        let signals = plan
            .signals
            .and_then(|targets| self.signal_slots.commit_projection(targets, plan.undo));
        let native = plan
            .native
            .and_then(|targets| self.native_tasks.commit_projection(targets, plan.undo));
        let details = plan.details.and_then(|targets| {
            self.details_intentions
                .commit_projection(targets, plan.undo)
        });
        if plan.undo {
            self.record_queue_undo_prepared(QueueUndo::Visibility(QueueVisibilityUndo {
                signals,
                native,
                details,
            }));
        }
    }
    pub(in crate::store) fn prepare_queue_document_change(
        &self,
        target: Key,
        old: Key,
        new: Key,
        cancel: &AtomicBool,
    ) -> Result<QueueDocumentPlan, DomError> {
        let target = self.live_key(target);
        let old = self.live_key(old);
        let new = self.live_key(new);
        self.source_step(cancel)?;
        if old == new
            || self.is_private(target)
            || (self.signal_slots.target_count(target) == 0
                && self.native_tasks.target_count(target) == 0)
        {
            return Ok(QueueDocumentPlan {
                signals: None,
                native: None,
                undo: false,
            });
        }
        let undo = self.prepare_undo(cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut admit = |usage| match &formation {
            Some(budget) => budget.admit_custody_usage(usage, cancel),
            None => check_cancel(cancel),
        };
        let mut step = || source::formation::step(formation.as_ref(), cancel);
        let signals = self
            .signal_slots
            .targets
            .as_ref()
            .expect("signal target owner")
            .move_document(target, old, new, &mut admit, &mut step)?;
        let native = self
            .native_tasks
            .targets
            .as_ref()
            .expect("native target owner")
            .move_document(target, old, new, &mut admit, &mut step)?;
        Ok(QueueDocumentPlan {
            signals,
            native,
            undo,
        })
    }
    pub(in crate::store) fn commit_queue_document_change(&mut self, plan: QueueDocumentPlan) {
        let signals = plan.signals.map(|signals| {
            self.signal_slots
                .targets
                .replace(signals)
                .expect("signal target owner")
        });
        let native = plan.native.map(|native| {
            self.native_tasks
                .targets
                .replace(native)
                .expect("native target owner")
        });
        if plan.undo {
            self.record_queue_undo_prepared(QueueUndo::Documents(QueueDocumentUndo {
                signals,
                native,
            }));
        }
    }
}
impl Draft {
    fn queue_association<C, S: Copy>(
        &self,
        queue: &LazyQueue<C, S>,
        value: S,
        cancel: &AtomicBool,
    ) -> Result<Option<(Key, Key)>, DomError> {
        self.source_step(cancel)?;
        let Some(target) = (queue.target)(value) else {
            return Ok(None);
        };
        Ok(match (queue.document)(value) {
            DocumentSource::Fixed(Some(document)) => Some((
                self.live_key(target),
                self.live_key(Key::Existing(document.node())),
            )),
            DocumentSource::CurrentTarget | DocumentSource::Fixed(None) => None,
        })
    }
    fn queue_document<C, S: Copy>(
        &self,
        queue: &LazyQueue<C, S>,
        value: S,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        let Some(target) = (queue.target)(value) else {
            return Ok(None);
        };
        self.source_step(cancel)?;
        if self.is_private(self.live_key(target)) {
            return Ok(None);
        }
        match (queue.document)(value) {
            DocumentSource::CurrentTarget => Ok(Some(
                self.live_key(self.node(self.live_key(target))?.document),
            )),
            DocumentSource::Fixed(document) => {
                Ok(document.map(|document| self.live_key(Key::Existing(document.node()))))
            }
        }
    }
    pub(in crate::store) fn queue_details_finalize_insert_document(
        &self,
        prepared: &mut PreparedDetailsInsert,
        values: &[details::Pending<Key>],
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let formation = source::formation::Formation::new(self, cancel)?;
        let documents = values.iter().filter_map(|value| {
            #[cfg(test)]
            metric(scale_metrics::Event::QueueRecordVisited);
            if self.is_private(self.live_key(value.target())) {
                return None;
            }
            value
                .queue_document()
                .map(|document| self.live_key(Key::Existing(document.node())))
        });
        let targets = prepared
            .prepared
            .targets
            .as_ref()
            .expect("prepared Details target owner")
            .change_documents(
                documents.map(|document| Ok((document, 1))),
                true,
                |usage| match &formation {
                    Some(budget) => budget.admit_custody_usage(usage, cancel),
                    None => check_cancel(cancel),
                },
                || source::formation::step(formation.as_ref(), cancel),
            )?;
        let targets = targets.change_associations(
            values.iter().filter_map(|value| {
                #[cfg(test)]
                metric(scale_metrics::Event::QueueRecordVisited);
                match self.queue_association(&self.details_intentions, *value, cancel) {
                    Ok(Some((target, document))) => Some(Ok((target, document, 1))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            }),
            true,
            |usage| match &formation {
                Some(budget) => budget.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
            || source::formation::step(formation.as_ref(), cancel),
        )?;
        prepared.prepared.targets = Some(targets);
        Ok(())
    }
}
impl Draft {
    pub(in crate::store) fn queue_retired_signal_remove(
        &mut self,
        index: usize,
        target: Key,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let target = self.live_key(target);
        assert_eq!(self.signal_slots.get(index), Some(target));
        let document = (!self.is_private(target)).then(|| self.live_key(document));
        let undo = self.prepare_undo(cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let ledger = self.custody_ledger().clone();
        let prepared = self.signal_slots.prepare_remove(
            Removal {
                index,
                count: 1,
                retain_undo: undo,
            },
            |value| {
                assert_eq!(value, target);
                self.source_step(cancel)?;
                Ok(document)
            },
            |_| Ok(None),
            &ledger,
            |usage| match &formation {
                Some(budget) => budget.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
            || source::formation::step(formation.as_ref(), cancel),
        )?;
        if let Some(inverse) =
            prepared.and_then(|prepared| self.signal_slots.commit_remove(prepared, undo))
        {
            self.record_queue_undo_prepared(QueueUndo::Signals(inverse));
        }
        Ok(())
    }
    pub(in crate::store) fn queue_signals_clear(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.queue_signals_remove(0, self.signal_slots.len(), cancel)
    }
    pub(in crate::store) fn queue_details_clear(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.queue_details_remove(0, self.details_intentions.len(), cancel)
    }
}

macro_rules! finish_targets {
    ($method:ident,$field:ident) => {
        impl Draft {
            pub(in crate::store) fn $method(
                &self,
                id: &impl Fn(Key) -> NodeId,
                budget: Option<&source::formation::Formation>,
                cancel: &AtomicBool,
            ) -> Result<FinishedTargets, DomError> {
                let qualified = |key| self.published_id(key).unwrap_or_else(|| id(key));
                self.$field.finish_targets(
                    &qualified,
                    |key| {
                        self.is_private(key)
                            || (self.published_id(key).is_none() && self.node(key).is_err())
                    },
                    |usage| match budget {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    || source::formation::step(budget, cancel),
                )
            }
        }
    };
}
finish_targets!(finish_signal_targets, signal_slots);
finish_targets!(finish_native_targets, native_tasks);
finish_targets!(finish_details_targets, details_intentions);

impl Draft {
    pub(in crate::store) fn finish_signals(
        &self,
        id: &impl Fn(Key) -> NodeId,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<FinishedSignals, DomError> {
        if let Some(canonical) = self.signal_slots.canonical_backing() {
            return Ok(FinishedSignals {
                values: canonical.clone(),
                output_debit: None,
                ack: None,
            });
        }
        let mut output = Vec::new();
        source::formation::reserve(&mut output, self.signal_slots.len(), budget, cancel)?;
        for key in self.signal_slots.iter() {
            source::formation::step(budget, cancel)?;
            if !self.is_private(key) {
                output.push(id(key));
            }
        }
        let mut ack = QueueAck::prepare(0, self.custody_ledger(), |usage| match budget {
            Some(budget) => budget.admit_custody_usage(usage, cancel),
            None => check_cancel(cancel),
        })?;
        let selected = output.len();
        let values = ack.publish_values(
            output,
            selected,
            budget,
            self.custody_ledger(),
            |usage| match budget {
                Some(budget) => budget.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
        )?;
        Ok(FinishedSignals {
            values: values.values,
            output_debit: values.output_debit,
            ack: Some(ack),
        })
    }
    pub(in crate::store) fn acknowledge_signals_publication(
        &mut self,
        ack: QueueAck<Key, NodeId>,
        committed: &PublicationCommitted,
    ) {
        self.signal_slots.apply_ack(ack, committed);
    }
    pub(in crate::store) fn acknowledge_queue_targets_publication(
        &mut self,
        signal: (&CanonicalTargets, &TargetsCustody),
        native: (&CanonicalTargets, &TargetsCustody),
        details: (&CanonicalTargets, &TargetsCustody),
    ) {
        self.signal_slots
            .targets
            .as_mut()
            .expect("signal target owner")
            .mark_published(signal.0, signal.1);
        self.native_tasks
            .targets
            .as_mut()
            .expect("native target owner")
            .mark_published(native.0, native.1);
        self.details_intentions
            .targets
            .as_mut()
            .expect("details target owner")
            .mark_published(details.0, details.1);
    }
    pub(in crate::store) fn unissued_source_identities(&self) -> (u64, u64) {
        (
            self.native_tasks
                .counts()
                .unissued
                .checked_add(self.cancelled_selects.counts().unissued)
                .expect("admitted Select identity count"),
            self.details_intentions
                .counts()
                .unissued
                .checked_add(self.cancelled_details.counts().unissued)
                .expect("admitted Details identity count"),
        )
    }
    pub(in crate::store) fn acknowledge_native_publication(
        &mut self,
        ack: QueueAck<select::Pending, select::Task<NodeId>>,
        committed: &PublicationCommitted,
    ) {
        self.native_tasks.apply_ack(ack, committed);
    }
    pub(in crate::store) fn acknowledge_details_publication(
        &mut self,
        ack: QueueAck<details::Pending<Key>, crate::DomDetailsToggleIntention>,
        committed: &PublicationCommitted,
    ) {
        self.details_intentions.apply_ack(ack, committed);
    }
    pub(in crate::store) fn acknowledge_cancelled_queues(
        &mut self,
        committed: &PublicationCommitted,
    ) {
        self.cancelled_signals.clear_committed(committed);
        self.cancelled_selects.clear_committed(committed);
        self.cancelled_details.clear_committed(committed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Affine(u64);
    thread_local! { static MAPPED:Cell<usize>=const{Cell::new(0)}; }
    fn map(value: &Affine) -> u64 {
        MAPPED.set(MAPPED.get() + 1);
        value.0
    }
    fn accept(_: custody::Usage) -> Result<(), DomError> {
        Ok(())
    }
    fn fixture(
        values: Vec<Affine>,
    ) -> (
        bank::CustodyRegistry,
        SharedVec<Affine>,
        LazyQueue<Affine, u64>,
    ) {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let values = SharedVec::from(values);
        let queue = LazyQueue::capture(
            &values,
            Model {
                map,
                measure: one::<u64>,
                target: no_target,
                document: no_document,
            },
            values.len(),
            None,
            &registry,
            accept,
        )
        .unwrap();
        (registry, values, queue)
    }

    #[test]
    fn empty_queue_capture_and_publication_charge_only_retained_backings() {
        for capacity in [0, 11] {
            let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
            registry.reconcile_releases();
            let baseline = registry.ledger().total();
            let raw = Vec::<Affine>::with_capacity(capacity);
            let actual_capacity = raw.capacity();
            let values = SharedVec::from(raw);
            let expected = custody::Usage::new(
                if actual_capacity == 0 {
                    0
                } else {
                    size_of::<Vec<Affine>>() + actual_capacity * size_of::<Affine>()
                },
                0,
            );
            let queue = LazyQueue::capture(
                &values,
                Model {
                    map,
                    measure: one::<u64>,
                    target: no_target,
                    document: no_document,
                },
                0,
                None,
                &registry,
                accept,
            )
            .unwrap();
            assert!(queue.canonical_backing().unwrap().same_storage(&values));
            assert_eq!(
                registry.ledger().total(),
                baseline.checked_add(expected).unwrap()
            );
            drop(queue);
            assert_eq!(registry.ledger().total(), baseline);

            let mut ack = QueueAck::<u64, Affine>::prepare(0, registry.ledger(), accept).unwrap();
            let raw = Vec::<Affine>::with_capacity(capacity);
            let published_capacity = raw.capacity();
            let published_usage = custody::Usage::new(
                if published_capacity == 0 {
                    0
                } else {
                    size_of::<Vec<Affine>>() + published_capacity * size_of::<Affine>()
                },
                0,
            );
            let output = ack
                .publish_values(raw, 0, None, registry.ledger(), accept)
                .unwrap();
            assert!(output.values.is_empty());
            assert_eq!(output.values.capacity(), published_capacity);
            assert_eq!(output.values.has_backing(), capacity != 0);
            assert_eq!(
                registry.ledger().total(),
                baseline.checked_add(published_usage).unwrap()
            );
            drop(output);
            drop(ack);
            assert_eq!(registry.ledger().total(), baseline);
        }
    }
    #[test]
    fn counters_distinguish_retained_backing_actual_reads_and_materialized_queue_capacity() {
        if !scale_metrics::ENABLED {
            return;
        }
        use scale_metrics::{Domain, Hooks, Measurement};
        let (registry, original, mut queue) = fixture(vec![Affine(1), Affine(2), Affine(3)]);
        let observation = Measurement::begin().unwrap();
        let retained = queue.canonical_backing().unwrap().clone();
        assert!(retained.same_storage(&original));
        assert_eq!(queue.counts().selected, 3);
        let counters = observation.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 0);
        assert_eq!(counters.queue_record_visits, 0);

        let observation = Measurement::begin().unwrap();
        assert_eq!(queue.get(1), Some(2));
        assert_eq!(queue.get(8), None);
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(counters.queue_record_visits, 1);
        assert_eq!(counters.sequence_materializations, 0);
        assert_eq!(counters.structural_allocations, 0);

        let observation = Measurement::begin().unwrap();
        let replacement = queue
            .replacement(4, registry.ledger(), accept, || Ok(()))
            .unwrap()
            .unwrap();
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        let Backing::Staged(ref values) = replacement else {
            panic!("materialized backing")
        };
        assert_eq!(values.value().as_slice(), [1, 2, 3]);
        let bytes = values.value().capacity() * size_of::<u64>();
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(counters.structural_bytes, bytes);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(counters.sequence_materialized_bytes, 3 * size_of::<u64>());
        assert_eq!(counters.queue_record_visits, 3);
        queue.backing = replacement;

        let observation = Measurement::begin().unwrap();
        assert!(
            queue
                .replacement(4, registry.ledger(), accept, || Ok(()))
                .unwrap()
                .is_none()
        );
        let counters = observation.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 0);
        assert_eq!(counters.queue_record_visits, 0);
    }

    #[test]
    fn counters_record_only_completed_queue_suffix_moves_and_real_allocations() {
        if !scale_metrics::ENABLED {
            return;
        }
        use scale_metrics::{Domain, Hooks, Measurement};
        let (registry, _, mut queue) = fixture(vec![Affine(1), Affine(2), Affine(3)]);
        let observation = Measurement::begin().unwrap();
        assert!(matches!(
            queue.replacement(
                4,
                registry.ledger(),
                |_| Err(DomError::Cancelled),
                || Ok(())
            ),
            Err(DomError::Cancelled)
        ));
        let counters = observation.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 0);
        assert_eq!(counters.queue_record_visits, 0);
        assert_eq!(counters.queue_records_moved, 0);

        let observation = Measurement::begin().unwrap();
        let mut steps = 0;
        let refused = queue.replacement(4, registry.ledger(), accept, || {
            steps += 1;
            if steps == 2 {
                Err(DomError::Cancelled)
            } else {
                Ok(())
            }
        });
        assert!(matches!(refused, Err(DomError::Cancelled)));
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(steps, 2);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<u64>(),
            "only the first completed push initialized a replacement row"
        );
        assert_eq!(
            counters.queue_record_visits, 2,
            "the second row was read before its copy step refused"
        );
        assert_eq!(queue.iter().collect::<Vec<_>>(), [1, 2, 3]);

        queue.backing = queue
            .replacement(4, registry.ledger(), accept, || Ok(()))
            .unwrap()
            .unwrap();
        let prepared = queue
            .prepare_insert(1, &[9], |_| Ok(None), registry.ledger(), accept, || Ok(()))
            .unwrap();
        let observation = Measurement::begin().unwrap();
        queue.commit_insert(prepared, &[9], false);
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(counters.queue_records_moved, 2);
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(counters.sequence_materialized_bytes, 3 * size_of::<u64>());
        assert_eq!(queue.staged_values().unwrap(), [1, 9, 2, 3]);

        let prepared = queue
            .prepare_remove(
                Removal {
                    index: 1,
                    count: 1,
                    retain_undo: false,
                },
                |_| Ok(None),
                |_| Ok(None),
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap()
            .unwrap();
        let observation = Measurement::begin().unwrap();
        queue.commit_remove(prepared, false);
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(counters.queue_records_moved, 2);
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(counters.sequence_materialized_bytes, 2 * size_of::<u64>());
        assert_eq!(queue.staged_values().unwrap(), [1, 2, 3]);
    }

    #[test]
    fn counters_cover_completed_target_scratch_inverse_and_acknowledgement_writes() {
        if !scale_metrics::ENABLED {
            return;
        }
        use scale_metrics::{Domain, Hooks, Measurement};
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let canonical = SharedVec::from(Vec::<NodeId>::new());
        let signals = Signals::capture(
            &canonical,
            0,
            &CanonicalTargets::default(),
            &TargetsCustody::default(),
            &registry,
            accept,
        )
        .unwrap();
        let incoming = [Key::Created(4), Key::Created(9)];
        let observation = Measurement::begin().unwrap();
        let mut steps = 0;
        let refused = signals.prepare_insert(
            0,
            &incoming,
            |_| Ok(None),
            registry.ledger(),
            accept,
            || {
                steps += 1;
                if steps == 4 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<Option<Key>>(),
            "one target scratch row was completed before refusal"
        );
        assert_eq!(signals.len(), 0);
        let observation = Measurement::begin().unwrap();
        let prepared = signals
            .prepare_insert(
                0,
                &incoming,
                |_| Ok(None),
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap();
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            2 * size_of::<Option<Key>>(),
            "scratch target rows are initialized; reserved empty queue slots are not"
        );
        drop(prepared);

        let (registry, _, mut queue) = fixture(vec![Affine(1), Affine(2), Affine(3)]);
        queue.backing = queue
            .replacement(6, registry.ledger(), accept, || Ok(()))
            .unwrap()
            .unwrap();
        let original_pointer = queue.staged_values().unwrap().as_ptr();
        let observation = Measurement::begin().unwrap();
        let mut steps = 0;
        let refused = queue.prepare_remove(
            Removal {
                index: 0,
                count: 2,
                retain_undo: true,
            },
            |_| Ok(None),
            |_| Ok(None),
            registry.ledger(),
            accept,
            || {
                steps += 1;
                if steps == 4 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<u64>(),
            "only the completed removed-span prefix was initialized"
        );
        assert_eq!(queue.staged_values().unwrap(), [1, 2, 3]);
        let observation = Measurement::begin().unwrap();
        let prepared = queue
            .prepare_remove(
                Removal {
                    index: 1,
                    count: 1,
                    retain_undo: true,
                },
                |_| Ok(None),
                |_| Ok(None),
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap()
            .unwrap();
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(counters.sequence_materialized_bytes, size_of::<u64>());
        let observation = Measurement::begin().unwrap();
        let inverse = queue.commit_remove(prepared, true).unwrap();
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(counters.sequence_materialized_bytes, size_of::<u64>());
        assert_eq!(counters.queue_records_moved, 1);
        let observation = Measurement::begin().unwrap();
        queue.restore(inverse);
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            2 * size_of::<u64>(),
            "restore writes one incoming inverse row and moves one suffix row"
        );
        assert_eq!(queue.staged_values().unwrap(), [1, 2, 3]);
        assert_eq!(queue.staged_values().unwrap().as_ptr(), original_pointer);

        let observation = Measurement::begin().unwrap();
        let prepared = queue
            .prepare_retain(
                |value| value != 2,
                |_| Ok(None),
                |_| Ok(None),
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap()
            .unwrap();
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<(usize, u64)>()
        );
        let observation = Measurement::begin().unwrap();
        let inverse = queue.commit_retain(prepared, true).unwrap();
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(counters.sequence_materialized_bytes, size_of::<u64>());
        let observation = Measurement::begin().unwrap();
        queue.restore(inverse);
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            4 * size_of::<u64>(),
            "resize initializes one slot, then the actual restore loop writes all three rows"
        );
        assert_eq!(queue.staged_values().unwrap(), [1, 2, 3]);
        assert_eq!(queue.staged_values().unwrap().as_ptr(), original_pointer);

        let mut ack = QueueAck::<u64>::prepare(2, registry.ledger(), accept).unwrap();
        let observation = Measurement::begin().unwrap();
        ack.push_prepared(0, 11);
        ack.push_prepared(1, 13);
        let counters = observation.finish(Hooks::one(Domain::Transaction)).unwrap();
        assert_eq!(
            counters.sequence_materialized_bytes,
            2 * size_of::<(usize, u64)>()
        );
        assert_eq!(ack.rows.value().as_slice(), [(0, 11), (1, 13)]);
    }

    #[test]
    fn details_visibility_metadata_visits_only_the_selected_targets_rows() {
        if !scale_metrics::ENABLED {
            return;
        }
        for unrelated in [0, 64] {
            let cancel = AtomicBool::new(false);
            let limits = DomLimits::new(
                16,
                4096,
                64,
                0,
                0,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_details_toggle_intentions(128)
            .unwrap();
            let mut dom = Dom::new(limits).unwrap();
            let mut setup = DomBatch::new();
            let document = setup.create_document();
            let details = setup.create_element_ns(&document, crate::Namespace::html(), "details");
            let commit = dom.apply(setup, &cancel).unwrap();
            let document = commit.document(&document).unwrap();
            let details = commit.created(&details).unwrap().unwrap();
            let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
            draft.phase_map = Some(source::PhaseMap {
                ids: source::AllocationIds::default(),
                private: source::Privacy::default(),
            });
            let kind = draft.node(Key::Existing(details)).unwrap().kind.clone();
            let target = draft
                .allocate(kind, Key::Existing(document.node()), limits, &cancel)
                .unwrap();
            draft.mark_source_private(target, &cancel).unwrap();
            draft
                .set_attribute(target, None, &"open".into(), &"".into(), limits, &cancel)
                .unwrap();
            for index in 0..unrelated {
                if index % 2 == 0 {
                    draft
                        .set_attribute(
                            Key::Existing(details),
                            None,
                            &"open".into(),
                            &"".into(),
                            limits,
                            &cancel,
                        )
                        .unwrap();
                } else {
                    draft
                        .remove_attribute(
                            Key::Existing(details),
                            None,
                            &"open".into(),
                            limits,
                            &cancel,
                        )
                        .unwrap();
                }
            }
            assert_eq!(draft.details_intentions.target_count(target), 1);
            assert_eq!(draft.details_intentions.len(), unrelated + 1);
            let observation = scale_metrics::Measurement::begin().unwrap();
            let plan = draft
                .prepare_queue_visibility_change(target, true, false, &cancel)
                .unwrap();
            let counters = observation.finish(scale_metrics::Hooks::default()).unwrap();
            drop(plan);
            assert!(
                counters.queue_record_visits <= 1,
                "metadata lookup visited {} rows with {unrelated} unrelated intentions",
                counters.queue_record_visits
            );
            assert_eq!(
                counters.sequence_materializations, 0,
                "metadata does not publish the changed ordered queue"
            );
        }
    }
    #[test]
    fn actual_signal_and_select_obligations_follow_adoption_and_exact_retirement() {
        let cancel = AtomicBool::new(false);
        let limits = DomLimits::new(
            32,
            4096,
            64,
            32,
            32,
            8,
            65536,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_select_processing(16, 1_000_000, 1_000_000)
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut setup = DomBatch::new();
        let first = setup.create_document();
        let second = setup.create_document();
        let host = setup.create_element_ns(&first, crate::Namespace::html(), "div");
        setup.insert_before(&host, &first, None);
        let shadow = setup.attach_shadow(
            &host,
            crate::HtmlShadowRootProperties::new(
                crate::HtmlShadowRootMode::Open,
                crate::HtmlSlotAssignment::Manual,
                false,
                false,
                false,
            ),
        );
        let slot = setup.create_element_ns(&first, crate::Namespace::html(), "slot");
        let text = setup.create_text(&first, "light");
        setup.insert_before(&slot, &shadow, None);
        setup.insert_before(&text, &host, None);
        let select = setup.create_element_ns(&first, crate::Namespace::html(), "select");
        let option = setup.create_element_ns(&first, crate::Namespace::html(), "option");
        setup.insert_before(&option, &select, None);
        let commit = dom.apply(setup, &cancel).unwrap();
        let first = commit.document(&first).unwrap();
        let second = commit.document(&second).unwrap();
        let host = commit.created(&host).unwrap().unwrap();
        let slot = commit.created(&slot).unwrap().unwrap();
        let text = commit.created(&text).unwrap().unwrap();
        let select = commit.created(&select).unwrap().unwrap();
        let option = commit.created(&option).unwrap().unwrap();
        let mut batch = DomBatch::new();
        batch.assign_slot(slot, vec![text.into()]);
        batch.detach(option);
        dom.apply(batch, &cancel).unwrap();
        assert_eq!(dom.snapshot().pending_slot_signals(), &[slot]);
        assert!(dom.state.signal_targets.has_document_obligations(first));
        assert!(dom.state.native_targets.has_document_obligations(first));
        assert!(!dom.state.signal_targets.has_document_obligations(second));
        assert!(!dom.state.native_targets.has_document_obligations(second));
        let historical = dom.snapshot();
        let mut adopt = DomBatch::new();
        adopt.adopt(host, second.node());
        adopt.adopt(select, second.node());
        dom.apply(adopt, &cancel).unwrap();
        assert_eq!(
            dom.snapshot().node(slot).unwrap().owner_document(),
            Some(second)
        );
        assert_eq!(
            dom.snapshot().node(select).unwrap().owner_document(),
            Some(second)
        );
        assert!(!dom.state.signal_targets.has_document_obligations(first));
        assert!(!dom.state.native_targets.has_document_obligations(first));
        assert!(dom.state.signal_targets.has_document_obligations(second));
        assert!(dom.state.native_targets.has_document_obligations(second));
        assert!(
            historical
                .state
                .signal_targets
                .has_document_obligations(first)
        );
        assert!(
            historical
                .state
                .native_targets
                .has_document_obligations(first)
        );
        let mut retire = DomBatch::new();
        retire.destroy(host);
        retire.destroy(select);
        dom.apply(retire, &cancel).unwrap();
        assert!(!dom.state.signal_targets.has_document_obligations(second));
        assert!(!dom.state.native_targets.has_document_obligations(second));
    }
    #[test]
    fn private_target_exposure_updates_obligations_and_invalidates_clean_projection() {
        let cancel = AtomicBool::new(false);
        let limits = DomLimits::new(
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
        .with_select_processing(16, 1_000_000, 1_000_000)
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let select = setup.create_element_ns(&document, crate::Namespace::html(), "select");
        let commit = dom.apply(setup, &cancel).unwrap();
        let document = commit.document(&document).unwrap();
        let select = commit.created(&select).unwrap().unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.phase_map = Some(source::PhaseMap {
            ids: source::AllocationIds::default(),
            private: source::Privacy::default(),
        });
        let kind = draft.node(Key::Existing(select)).unwrap().kind.clone();
        let target = draft
            .allocate(kind, Key::Existing(document.node()), limits, &cancel)
            .unwrap();
        draft.mark_source_private(target, &cancel).unwrap();
        draft
            .queue_native_insert(
                0,
                &[select::Pending::Update {
                    serial: None,
                    target,
                }],
                &cancel,
            )
            .unwrap();
        let (hidden, _hidden_custody) = draft
            .finish_native_targets(
                &|_| panic!("private target must not publish"),
                None,
                &cancel,
            )
            .unwrap()
            .into_parts();
        assert!(!hidden.has_document_obligations(document));
        draft
            .native_tasks
            .0
            .targets
            .as_mut()
            .unwrap()
            .mark_published(&hidden, &_hidden_custody);
        // An earlier Source phase retained a clean empty public projection;
        // the private row remains in the actual local staged sequence.
        let debit = draft
            .custody_ledger()
            .retain(custody::Usage::ZERO, accept)
            .unwrap();
        draft.native_tasks.0.published = Some(Published {
            values: custody::Held::new(SharedVec::from(Vec::new()), debit),
            selected: 0,
        });
        assert!(draft.native_tasks.canonical_backing().unwrap().is_empty());
        draft.expose_source_subtree(target, &cancel).unwrap();
        assert!(!draft.is_private(target));
        assert!(draft.native_tasks.canonical_backing().is_none());
        let mapped = NodeId {
            owner: dom.owner,
            slot: 27,
            generation: 9,
        };
        let (visible, _visible_custody) = draft
            .finish_native_targets(
                &|key| match key {
                    Key::Existing(id) => id,
                    key if key == target => mapped,
                    _ => panic!("unrelated identity"),
                },
                None,
                &cancel,
            )
            .unwrap()
            .into_parts();
        assert!(visible.has_document_obligations(document));
        let visible_owner = Targets::capture(&visible, &_visible_custody, draft.custody_registry());
        assert_eq!(visible_owner.count(Key::Existing(mapped)), 1);
    }
    #[test]
    fn document_counts_preserve_distinct_original_associations_for_one_target() {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let owner = crate::DomOwnerId::allocate().unwrap();
        let target = Key::Existing(NodeId {
            owner,
            slot: 7,
            generation: 2,
        });
        let first = DocumentId(NodeId {
            owner,
            slot: 1,
            generation: 1,
        });
        let second = DocumentId(NodeId {
            owner,
            slot: 2,
            generation: 3,
        });
        let index = Targets::capture(
            &CanonicalTargets::default(),
            &TargetsCustody::default(),
            &registry,
        )
        .updated(
            [
                Ok((target, Some(Key::Existing(first.node())))),
                Ok((target, Some(Key::Existing(second.node())))),
            ],
            true,
            accept,
            || Ok(()),
        )
        .unwrap()
        .change_associations(
            [
                Ok((target, Key::Existing(first.node()), 1)),
                Ok((target, Key::Existing(second.node()), 1)),
            ],
            true,
            accept,
            || Ok(()),
        )
        .unwrap();
        let hidden = index
            .change_associated_documents(target, &|key| key, false, &mut accept, &mut || Ok(()))
            .unwrap();
        let (hidden_canonical, _hidden_custody) = hidden
            .finish(
                &|_| panic!("Existing associations"),
                |_| false,
                accept,
                || Ok(()),
            )
            .unwrap()
            .into_parts();
        assert!(!hidden_canonical.has_document_obligations(first));
        assert!(!hidden_canonical.has_document_obligations(second));
        let shown = hidden
            .change_associated_documents(target, &|key| key, true, &mut accept, &mut || Ok(()))
            .unwrap();
        let (shown_canonical, _shown_custody) = shown
            .finish(
                &|_| panic!("Existing associations"),
                |_| false,
                accept,
                || Ok(()),
            )
            .unwrap()
            .into_parts();
        assert!(shown_canonical.has_document_obligations(first));
        assert!(shown_canonical.has_document_obligations(second));
        let finished = index
            .finish(
                &|_| panic!("existing association identities need no mapping"),
                |_| false,
                accept,
                || Ok(()),
            )
            .unwrap();
        let (canonical, custody) = finished.into_parts();
        assert!(canonical.has_document_obligations(first));
        assert!(canonical.has_document_obligations(second));
        let index = Targets::capture(&canonical, &custody, &registry)
            .updated(
                [Ok((target, Some(Key::Existing(first.node()))))],
                false,
                accept,
                || Ok(()),
            )
            .unwrap()
            .change_associations(
                [Ok((target, Key::Existing(first.node()), 1))],
                false,
                accept,
                || Ok(()),
            )
            .unwrap();
        let (canonical, _custody) = index
            .finish(
                &|_| panic!("no Created associations"),
                |_| false,
                accept,
                || Ok(()),
            )
            .unwrap()
            .into_parts();
        assert_eq!(
            canonical.inspect_associations(),
            vec![(target, Key::Existing(second.node()), 1)]
        );
        assert!(!canonical.has_document_obligations(first));
        assert!(canonical.has_document_obligations(second));
        assert!(!canonical.has_document_obligations(DocumentId(NodeId {
            generation: 4,
            ..second.node()
        })));
    }
    #[test]
    fn document_only_finish_visits_dirty_documents_without_remapping_published_created_backlog() {
        for size in [1usize, 128] {
            let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
            let owner = crate::DomOwnerId::allocate().unwrap();
            let first = DocumentId(NodeId {
                owner,
                slot: 1,
                generation: 1,
            });
            let second = DocumentId(NodeId {
                owner,
                slot: 2,
                generation: 1,
            });
            let first_key = Key::Existing(first.node());
            let second_key = Key::Existing(second.node());
            let mapped = |key| match key {
                Key::Existing(id) => id,
                Key::Created(index) => NodeId {
                    owner,
                    slot: index + 100,
                    generation: 3,
                },
            };
            let mut targets = Targets::capture(
                &CanonicalTargets::default(),
                &TargetsCustody::default(),
                &registry,
            )
            .updated(
                (0..size).map(|index| Ok((Key::Created(index), Some(first_key)))),
                true,
                accept,
                || Ok(()),
            )
            .unwrap();
            let (canonical, custody) = targets
                .finish(&mapped, |_| false, accept, || Ok(()))
                .unwrap()
                .into_parts();
            targets.mark_published(&canonical, &custody);
            let mut moved = targets
                .move_document(Key::Created(0), first_key, second_key, accept, || Ok(()))
                .unwrap()
                .unwrap();
            let visits = Cell::new(0);
            let (canonical, custody) = moved
                .finish(
                    &|key| match key {
                        Key::Existing(id) => id,
                        Key::Created(_) => {
                            panic!("document move must not remap prior Created targets")
                        }
                    },
                    |_| false,
                    accept,
                    || {
                        visits.set(visits.get() + 1);
                        Ok(())
                    },
                )
                .unwrap()
                .into_parts();
            assert_eq!(visits.get(), 2, "backlog={size}");
            assert!(canonical.has_document_obligations(second));
            assert_eq!(canonical.has_document_obligations(first), size > 1);
            moved.mark_published(&canonical, &custody);
            let removed = moved
                .updated(
                    [Ok((Key::Created(0), Some(second_key)))],
                    false,
                    accept,
                    || Ok(()),
                )
                .unwrap();
            let (canonical, custody) = removed
                .finish(&mapped, |_| false, accept, || Ok(()))
                .unwrap()
                .into_parts();
            assert!(!canonical.has_document_obligations(second));
            let published = Targets::capture(&canonical, &custody, &registry);
            assert_eq!(published.count(Key::Existing(mapped(Key::Created(0)))), 0);
            if size > 1 {
                assert_eq!(
                    published.count(Key::Existing(mapped(Key::Created(size - 1)))),
                    1
                );
            }
        }
    }
    #[test]
    fn public_document_index_excludes_private_rows_without_losing_shared_obligations() {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let owner = crate::DomOwnerId::allocate().unwrap();
        let document = DocumentId(NodeId {
            owner,
            slot: 1,
            generation: 3,
        });
        let public = Key::Existing(NodeId {
            owner,
            slot: 2,
            generation: 5,
        });
        let private = Key::Created(7);
        let doc = Key::Existing(document.node());
        let index = Targets::capture(
            &CanonicalTargets::default(),
            &TargetsCustody::default(),
            &registry,
        )
        .updated(
            [Ok((public, Some(doc))), Ok((private, None))],
            true,
            accept,
            || Ok(()),
        )
        .unwrap();
        let (canonical, custody) = index
            .finish(
                &|key| match key {
                    Key::Existing(id) => id,
                    _ => panic!("private identity must not publish"),
                },
                |key| key == private,
                accept,
                || Ok(()),
            )
            .unwrap()
            .into_parts();
        assert!(canonical.has_document_obligations(document));
        let remaining = Targets::capture(&canonical, &custody, &registry)
            .updated([Ok((public, Some(doc)))], false, accept, || Ok(()))
            .unwrap();
        let (canonical, _custody) = remaining
            .finish(
                &|_| panic!("existing identity requires no mapping"),
                |_| false,
                accept,
                || Ok(()),
            )
            .unwrap()
            .into_parts();
        assert!(!canonical.has_document_obligations(document));
    }
    #[test]
    fn affine_canonical_capture_and_unchanged_publication_reuse_the_actual_backing() {
        MAPPED.set(0);
        let mut values = Vec::with_capacity(4096);
        values.extend([Affine(3), Affine(5)]);
        let (_registry, canonical, queue) = fixture(values);
        assert_eq!(MAPPED.get(), 0);
        assert!(queue.canonical_backing().unwrap().same_storage(&canonical));
        assert_eq!(queue.capacity(), 4096);
        assert_eq!(queue.get(1), Some(5));
        assert_eq!(MAPPED.get(), 1);
    }
    #[test]
    fn exact_target_presence_does_not_map_unrelated_queue_rows_and_qualifies_generation() {
        fn mapped(value: &NodeId) -> Key {
            MAPPED.set(MAPPED.get() + 1);
            Key::Existing(*value)
        }
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let owner = crate::DomOwnerId::allocate().unwrap();
        let target = NodeId {
            owner,
            slot: 7,
            generation: 3,
        };
        let other = NodeId {
            owner,
            slot: 8000,
            generation: 1,
        };
        let targets = Targets::capture(
            &CanonicalTargets::default(),
            &TargetsCustody::default(),
            &registry,
        )
        .updated(
            [
                Key::Existing(target),
                Key::Existing(target),
                Key::Existing(other),
            ]
            .into_iter()
            .map(|key| Ok((key, None))),
            true,
            accept,
            || Ok(()),
        )
        .unwrap();
        let canonical = SharedVec::from(vec![target, target, other]);
        MAPPED.set(0);
        let queue = LazyQueue::capture(
            &canonical,
            Model {
                map: mapped,
                measure: one::<Key>,
                target: signal_target,
                document: no_document,
            },
            3,
            Some(targets),
            &registry,
            accept,
        )
        .unwrap();
        assert_eq!(queue.target_count(Key::Existing(target)), 2);
        assert_eq!(
            queue.target_count(Key::Existing(NodeId {
                generation: 4,
                ..target
            })),
            0
        );
        assert_eq!(queue.target_count(Key::Created(target.slot)), 0);
        assert_eq!(
            queue.target_count(Key::Existing(NodeId {
                slot: 9000,
                ..target
            })),
            0
        );
        assert_eq!(MAPPED.get(), 0);
    }
    #[test]
    fn target_index_rollback_and_created_publication_update_only_actual_target_records() {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let canonical = SharedVec::from(Vec::<NodeId>::new());
        let mut queue = Signals::capture(
            &canonical,
            0,
            &CanonicalTargets::default(),
            &TargetsCustody::default(),
            &registry,
            accept,
        )
        .unwrap();
        let first = Key::Created(4);
        let private = Key::Created(9);
        let outer = queue
            .insert(0, &[first, first], true, registry.ledger(), accept, || {
                Ok(())
            })
            .unwrap()
            .unwrap();
        let inner = queue
            .insert(2, &[private], true, registry.ledger(), accept, || Ok(()))
            .unwrap()
            .unwrap();
        assert_eq!(queue.target_count(first), 2);
        assert_eq!(queue.target_count(private), 1);
        queue.restore(inner);
        assert_eq!(queue.target_count(first), 2);
        assert_eq!(queue.target_count(private), 0);
        let owner = crate::DomOwnerId::allocate().unwrap();
        let node = NodeId {
            owner,
            slot: 34,
            generation: 5,
        };
        let steps = Cell::new(0usize);
        let finished = queue
            .finish_targets(
                &|key| {
                    assert_eq!(key, first);
                    node
                },
                |key| key == private,
                accept,
                || {
                    steps.set(steps.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(steps.get(), 1);
        let (records, custody) = finished.into_parts();
        let adopted = Targets::capture(&records, &custody, &registry);
        assert_eq!(adopted.count(Key::Existing(node)), 2);
        assert_eq!(adopted.count(first), 0);
        queue
            .targets
            .as_mut()
            .unwrap()
            .mark_published(&records, &custody);
        let clean = queue
            .finish_targets(
                &|_| panic!("unchanged target publication must reuse the canonical root"),
                |_| false,
                accept,
                || panic!("unchanged target publication must not visit prior Created targets"),
            )
            .unwrap();
        let (clean_records, _clean_custody) = clean.into_parts();
        assert!(clean_records.shares_root(&records));
        queue.restore(outer);
        assert_eq!(queue.target_count(first), 0);
        assert!(queue.canonical_backing().unwrap().same_storage(&canonical));
    }
    #[test]
    fn staged_appends_touch_only_incoming_values_and_nested_inverses_restore_capacity() {
        let (registry, canonical, mut queue) = fixture(vec![Affine(3), Affine(5)]);
        let outer = queue
            .insert(2, &[8], true, registry.ledger(), accept, || Ok(()))
            .unwrap()
            .unwrap();
        let capacity = queue.capacity();
        let steps = Cell::new(0);
        let inner = queue
            .insert(3, &[13], true, registry.ledger(), accept, || {
                steps.set(steps.get() + 1);
                Ok(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(steps.get(), 1);
        assert_eq!(queue.staged_values().unwrap(), [3, 5, 8, 13]);
        queue.restore(inner);
        assert_eq!(queue.capacity(), capacity);
        assert_eq!(queue.staged_values().unwrap(), [3, 5, 8]);
        queue.restore(outer);
        assert!(queue.canonical_backing().unwrap().same_storage(&canonical));
        assert_eq!(queue.counts().selected, 2);
    }
    #[test]
    fn refused_materialization_preserves_canonical_identity_counts_and_ledger() {
        let (registry, canonical, mut queue) = fixture(vec![Affine(3), Affine(5)]);
        let before = registry.total();
        assert!(matches!(
            queue.insert(
                2,
                &[8],
                true,
                registry.ledger(),
                |_| Err(DomError::Cancelled),
                || Ok(())
            ),
            Err(DomError::Cancelled)
        ));
        assert!(queue.canonical_backing().unwrap().same_storage(&canonical));
        assert_eq!(
            queue.counts(),
            QueueCounts {
                selected: 2,
                unissued: 0
            }
        );
        assert_eq!(registry.total(), before);
    }
    #[test]
    fn dropping_prepared_capacity_preserves_queue_and_releases_provisional_custody() {
        let (registry, canonical, mut queue) = fixture(vec![Affine(3), Affine(5)]);
        let before = registry.total();
        let prepared = queue
            .prepare_insert(2, &[8], |_| Ok(None), registry.ledger(), accept, || Ok(()))
            .unwrap();
        assert!(registry.total().bytes() > before.bytes());
        assert!(queue.canonical_backing().unwrap().same_storage(&canonical));
        // Dropping the provisional token does not commit a queue write.
        drop(prepared);
        assert_eq!(registry.total(), before);
        assert_eq!(
            queue.counts(),
            QueueCounts {
                selected: 2,
                unissued: 0
            }
        );
        let prepared = queue
            .prepare_insert(2, &[8], |_| Ok(None), registry.ledger(), accept, || Ok(()))
            .unwrap();
        queue.commit_insert(prepared, &[8], false);
        assert_eq!(queue.staged_values().unwrap(), [3, 5, 8]);
    }
    #[test]
    fn sparse_retention_inverse_restores_original_order_without_reallocating_staged_capacity() {
        let (registry, _canonical, mut queue) =
            fixture(vec![Affine(1), Affine(2), Affine(3), Affine(4)]);
        queue
            .insert(4, &[5], false, registry.ledger(), accept, || Ok(()))
            .unwrap();
        let capacity = queue.capacity();
        let pointer = queue.staged_values().unwrap().as_ptr();
        let inverse = queue
            .retain(
                |value| value % 2 != 0,
                true,
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap()
            .unwrap();
        assert_eq!(queue.staged_values().unwrap(), [1, 3, 5]);
        assert_eq!(queue.staged_values().unwrap().as_ptr(), pointer);
        queue.restore(inverse);
        assert_eq!(queue.staged_values().unwrap(), [1, 2, 3, 4, 5]);
        assert_eq!(queue.capacity(), capacity);
        assert_eq!(queue.staged_values().unwrap().as_ptr(), pointer);
    }
    #[test]
    fn produced_capacity_receipt_moves_without_releasing_or_readmitting_the_state_occurrence() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(
            DomLimits::new(
                16,
                4096,
                32,
                0,
                0,
                8,
                65536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        draft.source_limits = Some(crate::DomSourceLimits::new(
            usize::MAX,
            usize::MAX,
            1_000_000,
            64,
            4096,
            4096,
            1,
        ));
        let formation = source::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let mut values = Vec::new();
        source::formation::reserve(&mut values, 3, Some(&formation), &cancel).unwrap();
        values.extend([2u64, 5, 8]);
        let bytes = values.capacity() * size_of::<u64>();
        assert_eq!(formation.owned_payload().bytes, bytes);
        let mut ack = QueueAck::<u64, u64>::prepare(0, draft.custody_ledger(), accept).unwrap();
        let output = ack
            .publish_values(values, 3, Some(&formation), draft.custody_ledger(), accept)
            .unwrap();
        assert_eq!(formation.owned_payload().bytes, 0);
        assert_eq!(output.output_debit.as_ref().unwrap().usage().bytes(), bytes);
        assert_eq!(&*output.values, &[2, 5, 8]);
        let retained = draft.custody_ledger().total();
        drop(formation);
        assert_eq!(draft.custody_ledger().total(), retained);
        drop(output);
        assert_eq!(
            draft.custody_ledger().total().bytes(),
            retained.bytes() - bytes
        );
        assert_eq!(
            &**ack.published.as_ref().unwrap().values.value(),
            &[2, 5, 8]
        );
    }
    #[test]
    fn acknowledged_queue_reuses_its_published_backing_and_rollback_restores_clean_alias() {
        let (registry, _canonical, mut queue) = fixture(vec![Affine(3), Affine(5)]);
        queue
            .insert(2, &[8], false, registry.ledger(), accept, || Ok(()))
            .unwrap();
        let mut ack = QueueAck::prepare(0, registry.ledger(), accept).unwrap();
        let published = ack
            .publish_values(
                vec![Affine(3), Affine(5), Affine(8)],
                3,
                None,
                registry.ledger(),
                accept,
            )
            .unwrap();
        queue.apply_ack(ack, &PublicationCommitted::at_fence());
        MAPPED.set(0);
        assert!(
            queue
                .canonical_backing()
                .unwrap()
                .same_storage(&published.values)
        );
        assert_eq!(MAPPED.get(), 0);
        let inverse = queue
            .insert(3, &[13], true, registry.ledger(), accept, || Ok(()))
            .unwrap()
            .unwrap();
        assert!(queue.canonical_backing().is_none());
        queue.restore(inverse);
        assert!(
            queue
                .canonical_backing()
                .unwrap()
                .same_storage(&published.values)
        );
        assert_eq!(queue.staged_values().unwrap(), [3, 5, 8]);
    }
    #[test]
    fn prepared_acknowledgement_changes_only_its_exact_row_and_unissued_cache() {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let mut queue = LazyQueue::empty(
            Model {
                map: native,
                measure: native_count,
                target: no_target,
                document: no_document,
            },
            &registry,
            accept,
        )
        .unwrap();
        let target = Key::Created(7);
        queue
            .insert(
                0,
                &[
                    select::Pending::Slots,
                    select::Pending::Update {
                        serial: None,
                        target,
                    },
                ],
                false,
                registry.ledger(),
                accept,
                || Ok(()),
            )
            .unwrap();
        let pointer = queue.staged_values().unwrap().as_ptr();
        let mut ack = QueueAck::prepare(1, registry.ledger(), accept).unwrap();
        ack.push_prepared(
            1,
            select::Pending::Update {
                serial: Some(23),
                target,
            },
        );
        queue.apply_ack(ack, &PublicationCommitted::at_fence());
        assert!(matches!(queue.first(), Some(select::Pending::Slots)));
        assert!(matches!(
            queue.get(1),
            Some(select::Pending::Update {
                serial: Some(23),
                target: Key::Created(7)
            })
        ));
        assert_eq!(
            queue.counts(),
            QueueCounts {
                selected: 1,
                unissued: 0
            }
        );
        assert_eq!(queue.staged_values().unwrap().as_ptr(), pointer);
    }
}
