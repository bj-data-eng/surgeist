//! Full issuer/slot/generation qualification using the shared persistent Bank.
use super::super::super::*;
use crate::hosted::{DomHostedError, DomHostedSource, DomHostedSourceId, Retained, Work};
use std::cell::Cell;

pub(in crate::store) struct Producer<'a> {
    pub source: &'a DomHostedSource,
    pub work: &'a Work,
    pub formation: Option<&'a source::formation::Formation>,
    pub cancel: &'a AtomicBool,
    pub origin: custody::MaterializationId,
    pub scratch: &'a Cell<usize>,
}
impl Producer<'_> {
    pub fn retain(&self, usage: custody::Usage) -> Result<Retained, DomHostedError> {
        self.work.step(self.cancel)?;
        let scratch = self
            .scratch
            .get()
            .checked_add(usage.bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        crate::hosted::admit(
            scratch,
            self.source.limits().scratch_bytes(),
            crate::hosted::DomHostedResource::ScratchBytes,
        )?;
        let retained = self.source.retain(usage.bytes(), self.cancel)?;
        if let Some(formation) = self.formation {
            formation.retain_payload(
                source::storage::Payload {
                    bytes: usage.bytes(),
                    units: usage.units(),
                },
                self.cancel,
            )?;
        }
        self.scratch.set(scratch);
        Ok(retained)
    }
    pub fn record<T>(
        &self,
        form: impl FnOnce(Retained) -> T,
    ) -> Result<bank::Record<T>, DomHostedError> {
        let reserved = bank::Record::reserve()?;
        let retained = self.retain(custody::Usage::new(size_of::<T>(), 0))?;
        Ok(reserved.materialize(form(retained)))
    }
    pub fn set<T>(
        &self,
        bank: &mut bank::Bank<T>,
        key: usize,
        value: bank::Record<T>,
        usage: custody::Usage,
    ) -> Result<(), DomHostedError> {
        bank.set_hosted_admitted(
            key,
            value,
            false,
            bank::Totals {
                bytes: usage.bytes(),
                units: usage.units(),
                occurrences: 0,
            },
            self.origin,
            |charge| self.retain(custody::Usage::new(charge.bytes, 0)),
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(in crate::store) struct Key {
    pub source: DomHostedSourceId,
    pub slot: usize,
    pub generation: u64,
}
impl Key {
    pub fn new(source: DomHostedSourceId, slot: usize, generation: u64) -> Self {
        Self {
            source,
            slot,
            generation,
        }
    }
}

/// Four bounded native chunks preserve every u64 bit, including terminal
/// generations, without ever indexing Bank at usize::MAX on 32/64-bit hosts.
pub(in crate::store) struct U64Bank<T> {
    pub branches: bank::Bank<U64Branch<T>>,
    pub values: bank::Bank<T>,
}
pub(in crate::store) struct U64Branch<T> {
    pub origin: custody::MaterializationId,
    pub next: U64Bank<T>,
    _retained: Retained,
}
impl<T> Default for U64Bank<T> {
    fn default() -> Self {
        Self {
            branches: bank::Bank::default(),
            values: bank::Bank::default(),
        }
    }
}
impl<T> Clone for U64Bank<T> {
    fn clone(&self) -> Self {
        Self {
            branches: self.branches.clone(),
            values: self.values.clone(),
        }
    }
}
fn chunk(value: u64, level: u32) -> usize {
    usize::from(u16::try_from((value >> (level * 16)) & 0xffff).expect("masked native chunk"))
}
impl<T> U64Bank<T> {
    pub fn get(&self, key: u64) -> Option<&T> {
        self.get_with(key, &mut || Ok::<(), std::convert::Infallible>(()))
            .expect("infallible observation")
    }
    fn get_with<E>(
        &self,
        key: u64,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&T>, E> {
        let mut current = self;
        for level in (1..=3).rev() {
            let Some(branch) = current.branches.get_with(chunk(key, level), step)? else {
                return Ok(None);
            };
            current = &branch.next;
        }
        current.values.get_with(chunk(key, 0), step)
    }
    pub fn usage(&self) -> Result<custody::Usage, DomError> {
        bank_usage(&self.branches)?.checked_add(bank_usage(&self.values)?)
    }
    pub fn path_usage(
        &self,
        key: u64,
        origin: custody::MaterializationId,
    ) -> Result<custody::Usage, DomError> {
        let mut current = self;
        let mut usage = custody::Usage::ZERO;
        for level in (1..=3).rev() {
            let index = chunk(key, level);
            usage = usage.checked_add(custody::Usage::new(
                current
                    .branches
                    .origin_path_index_charge(index, origin)?
                    .bytes,
                0,
            ))?;
            let Some(branch) = current.branches.get(index) else {
                return Ok(usage);
            };
            if branch.origin == origin {
                usage = usage.checked_add(custody::Usage::new(size_of::<U64Branch<T>>(), 0))?;
            }
            current = &branch.next;
        }
        usage.checked_add(custody::Usage::new(
            current
                .values
                .origin_path_index_charge(chunk(key, 0), origin)?
                .bytes,
            0,
        ))
    }
    pub fn insert(
        &mut self,
        key: u64,
        value: bank::Record<T>,
        usage: custody::Usage,
        producer: &Producer<'_>,
    ) -> Result<(), DomHostedError> {
        fn at<T>(
            current: &mut U64Bank<T>,
            key: u64,
            level: u32,
            value: bank::Record<T>,
            usage: custody::Usage,
            producer: &Producer<'_>,
        ) -> Result<(), DomHostedError> {
            let index = chunk(key, level);
            if level == 0 {
                return producer.set(&mut current.values, index, value, usage);
            }
            let mut next = current
                .branches
                .get(index)
                .map_or_else(U64Bank::default, |branch| branch.next.clone());
            at(&mut next, key, level - 1, value, usage, producer)?;
            let usage =
                custody::Usage::new(size_of::<U64Branch<T>>(), 0).checked_add(next.usage()?)?;
            let branch = producer.record(|retained| U64Branch {
                origin: producer.origin,
                next,
                _retained: retained,
            })?;
            producer.set(&mut current.branches, index, branch, usage)
        }
        at(self, key, 3, value, usage, producer)
    }
    pub fn same_values(&self, other: &Self, equal: &impl Fn(&T, &T) -> bool) -> bool {
        self.branches
            .same_records_by(&other.branches, &|a, b| a.next.same_values(&b.next, equal))
            && self.values.same_records_by(&other.values, equal)
    }
    #[cfg(test)]
    pub fn visit(&self, visit: &mut dyn FnMut(&T) -> Result<(), DomError>) -> Result<(), DomError> {
        for (_, branch) in self.branches.iter() {
            branch.next.visit(visit)?;
        }
        for (_, value) in self.values.iter() {
            visit(value)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn inspect_materializations(
        &self,
        describe: &dyn Fn(&T) -> Result<custody::Usage, DomError>,
    ) -> Result<Vec<(custody::MaterializationId, custody::Usage)>, DomError> {
        let mut output = Vec::new();
        let branches = self.branches.inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<U64Branch<T>>(),
                units: 0,
                occurrences: 0,
            })
        })?;
        output.extend(branches.materializations());
        for (_, branch) in self.branches.iter() {
            output.extend(branch.next.inspect_materializations(describe)?);
        }
        let values = self.values.inspect_graph(|value| {
            let own = describe(value)?;
            Ok(bank::Totals {
                bytes: own.bytes(),
                units: own.units(),
                occurrences: 0,
            })
        })?;
        output.extend(values.materializations());
        Ok(output)
    }
}
pub(in crate::store) struct U64Custody<T> {
    pub branches: bank::BankCustody<U64Branch<T>>,
    pub values: bank::BankCustody<T>,
}
pub(in crate::store) struct U64Children<T> {
    pub _next: U64Custody<T>,
}
impl<T> Clone for U64Custody<T> {
    fn clone(&self) -> Self {
        Self {
            branches: self.branches.clone(),
            values: self.values.clone(),
        }
    }
}
#[cfg(test)]
impl<T: Send + Sync + 'static> U64Custody<T> {
    pub fn inspect_materializations(&self) -> Vec<(custody::MaterializationId, custody::Usage)> {
        let mut output = self.branches.inspect_materializations();
        output.extend(self.values.inspect_materializations());
        for branch in self.branches.inspect_dependencies::<U64Children<T>>() {
            output.extend(branch._next.inspect_materializations());
        }
        output
    }
    pub fn inspect_dependencies<D: std::any::Any + Send + Sync>(&self) -> Vec<bank::Retained<D>> {
        let mut output = self.values.inspect_dependencies::<D>();
        for branch in self.branches.inspect_dependencies::<U64Children<T>>() {
            output.extend(branch._next.inspect_dependencies::<D>());
        }
        output
    }
}

pub(in crate::store) struct Generations<T> {
    pub origin: custody::MaterializationId,
    pub values: U64Bank<T>,
    _retained: Retained,
}
impl<T> Generations<T> {
    pub fn form(
        values: U64Bank<T>,
        producer: &Producer<'_>,
    ) -> Result<bank::Record<Self>, DomHostedError> {
        producer.record(|retained| Self {
            origin: producer.origin,
            values,
            _retained: retained,
        })
    }
}
pub(in crate::store) struct Issuer<T> {
    pub origin: custody::MaterializationId,
    pub slots: bank::Bank<Generations<T>>,
    _retained: Retained,
}
pub(in crate::store) struct Index<T> {
    owner: Option<DomOwnerId>,
    pub sources: U64Bank<Issuer<T>>,
}
impl<T> Default for Index<T> {
    fn default() -> Self {
        Self {
            owner: None,
            sources: U64Bank::default(),
        }
    }
}
impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner,
            sources: self.sources.clone(),
        }
    }
}
impl<T> std::fmt::Debug for Index<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}
pub(in crate::store) fn bank_usage<T>(bank: &bank::Bank<T>) -> Result<custody::Usage, DomError> {
    Ok(custody::Usage::new(
        bank.index_charge()
            .bytes
            .checked_add(bank.totals().bytes)
            .ok_or(DomError::RepresentationExhausted)?,
        bank.totals().units,
    ))
}
impl<T> Index<T> {
    pub fn get(&self, key: Key) -> Result<Option<&T>, DomError> {
        self.get_with(key, &mut || Ok(()))
    }
    pub fn get_with<E: From<DomError>>(
        &self,
        key: Key,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&T>, E> {
        if self.owner.is_some_and(|owner| owner != key.source.owner) {
            return Err(DomError::ForeignOwner.into());
        }
        let Some(source) = self.sources.get_with(key.source.serial, step)? else {
            return Ok(None);
        };
        let Some(slot) = source.slots.get_with(key.slot, step)? else {
            return Ok(None);
        };
        slot.values.get_with(key.generation, step)
    }
    pub fn usage(&self) -> Result<custody::Usage, DomError> {
        self.sources.usage()
    }
    /// Return only old, same-call objects on the replaced paths. The caller
    /// releases this receipt after destroying its complete old semantic root.
    pub fn insert(
        &mut self,
        key: Key,
        value: bank::Record<T>,
        usage: custody::Usage,
        replace_generation: bool,
        producer: &Producer<'_>,
    ) -> Result<custody::Usage, DomHostedError> {
        let _ = self.get(key)?;
        let serial = key.source.serial;
        let generation = key.generation;
        let old_source = self.sources.get(serial);
        let old_slot = old_source.and_then(|source| source.slots.get(key.slot));
        let mut discarded = self.sources.path_usage(serial, producer.origin)?;
        if let Some(source) = old_source {
            discarded = discarded.checked_add(custody::Usage::new(
                source
                    .slots
                    .origin_path_index_charge(key.slot, producer.origin)?
                    .bytes,
                0,
            ))?;
            if source.origin == producer.origin {
                discarded =
                    discarded.checked_add(custody::Usage::new(size_of::<Issuer<T>>(), 0))?;
            }
        }
        if let Some(slot) = old_slot {
            if slot.origin == producer.origin {
                discarded =
                    discarded.checked_add(custody::Usage::new(size_of::<Generations<T>>(), 0))?;
            }
            if !replace_generation {
                discarded =
                    discarded.checked_add(slot.values.path_usage(generation, producer.origin)?)?;
            }
        }
        // Global's selected slot retains one current generation. Historical
        // snapshots retain its former root; other issuers/slots remain shared.
        let mut values = if replace_generation {
            U64Bank::default()
        } else {
            old_slot.map_or_else(U64Bank::default, |slot| slot.values.clone())
        };
        values.insert(generation, value, usage, producer)?;
        let slot_usage =
            custody::Usage::new(size_of::<Generations<T>>(), 0).checked_add(values.usage()?)?;
        let slot = producer.record(|retained| Generations {
            origin: producer.origin,
            values,
            _retained: retained,
        })?;
        let mut slots = old_source.map_or_else(bank::Bank::default, |source| source.slots.clone());
        producer.set(&mut slots, key.slot, slot, slot_usage)?;
        let source_usage =
            custody::Usage::new(size_of::<Issuer<T>>(), 0).checked_add(bank_usage(&slots)?)?;
        let source = producer.record(|retained| Issuer {
            origin: producer.origin,
            slots,
            _retained: retained,
        })?;
        let mut sources = self.sources.clone();
        sources.insert(serial, source, source_usage, producer)?;
        self.sources = sources;
        self.owner = Some(key.source.owner);
        Ok(discarded)
    }
}

pub(in crate::store) struct Ordinal {
    pub value: usize,
    pub origin: custody::MaterializationId,
    _retained: Retained,
}
impl Ordinal {
    pub fn form(
        value: usize,
        producer: &Producer<'_>,
    ) -> Result<bank::Record<Self>, DomHostedError> {
        producer.record(|retained| Self {
            value,
            origin: producer.origin,
            _retained: retained,
        })
    }
}

/// Strong child-only capsules live on their original branch record custody.
pub(in crate::store) struct GenerationsCustody<T> {
    pub _values: U64Custody<T>,
}
pub(in crate::store) struct IssuerCustody<T> {
    pub _slots: bank::BankCustody<Generations<T>>,
}
pub(in crate::store) struct IndexCustody<T> {
    pub sources: U64Custody<Issuer<T>>,
}
impl<T> Clone for IndexCustody<T> {
    fn clone(&self) -> Self {
        Self {
            sources: self.sources.clone(),
        }
    }
}

impl<T> Index<T> {
    pub fn same_values(&self, other: &Self, equal: &impl Fn(&T, &T) -> bool) -> bool {
        self.owner == other.owner
            && self.sources.same_values(&other.sources, &|a, b| {
                a.slots
                    .same_records_by(&b.slots, &|a, b| a.values.same_values(&b.values, equal))
            })
    }
    #[cfg(test)]
    pub fn inspect_materializations(
        &self,
        describe: impl Fn(&T) -> Result<custody::Usage, DomError>,
    ) -> Result<Vec<(custody::MaterializationId, custody::Usage)>, DomError> {
        let mut output = self
            .sources
            .inspect_materializations(&|_| Ok(custody::Usage::new(size_of::<Issuer<T>>(), 0)))?;
        self.sources.visit(&mut |source| {
            let own = source.slots.inspect_graph(|_| {
                Ok(bank::Totals {
                    bytes: size_of::<Generations<T>>(),
                    units: 0,
                    occurrences: 0,
                })
            })?;
            output.extend(own.materializations());
            for (_, slot) in source.slots.iter() {
                output.extend(slot.values.inspect_materializations(&describe)?);
            }
            Ok(())
        })?;
        Ok(output)
    }
}
#[cfg(test)]
impl<T: Send + Sync + 'static> IndexCustody<T> {
    pub fn inspect_materializations(&self) -> Vec<(custody::MaterializationId, custody::Usage)> {
        let mut output = self.sources.inspect_materializations();
        for source in self.sources.inspect_dependencies::<IssuerCustody<T>>() {
            output.extend(source._slots.inspect_materializations());
            for slot in source
                ._slots
                .inspect_dependencies::<GenerationsCustody<T>>()
            {
                output.extend(slot._values.inspect_materializations());
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    struct Leaf {
        value: u64,
        _retained: Retained,
    }
    #[test]
    fn native_chunks_preserve_high_source_bits_and_terminal_generations() {
        let owner = DomOwnerId::allocate().unwrap();
        let cancel = AtomicBool::new(false);
        let source = DomHostedSource::new(
            DomHostedSourceId { owner, serial: 0 },
            crate::hosted::DomHostedLimits::new(
                crate::hosted::DomHostedRecordLimits::new(0, 0, 0, 0, 0, 0, 0),
                16_000_000,
                16_000_000,
                10_000_000,
            ),
            &cancel,
        )
        .unwrap();
        let work = source.work(&cancel).unwrap();
        let scratch = Cell::new(0);
        let origin = custody::MaterializationId::fresh().unwrap();
        let producer = Producer {
            source: &source,
            work: &work,
            formation: None,
            cancel: &cancel,
            origin,
            scratch: &scratch,
        };
        let mut index = Index::default();
        let first = Key::new(DomHostedSourceId { owner, serial: 7 }, 0, 9);
        let high_source = Key::new(
            DomHostedSourceId {
                owner,
                serial: (1 << 48) | 7,
            },
            0,
            9,
        );
        let high_generation = Key::new(first.source, 0, (1 << 48) | 9);
        let terminal = Key::new(
            DomHostedSourceId {
                owner,
                serial: u64::MAX,
            },
            0,
            u64::MAX,
        );
        for (key, value) in [
            (first, 11),
            (high_source, 23),
            (high_generation, 37),
            (terminal, 41),
        ] {
            let leaf = producer
                .record(|retained| Leaf {
                    value,
                    _retained: retained,
                })
                .unwrap();
            index
                .insert(
                    key,
                    leaf,
                    custody::Usage::new(size_of::<Leaf>(), 0),
                    false,
                    &producer,
                )
                .unwrap();
        }
        assert_eq!(index.get(first).unwrap().unwrap().value, 11);
        assert_eq!(index.get(high_source).unwrap().unwrap().value, 23);
        assert_eq!(index.get(high_generation).unwrap().unwrap().value, 37);
        assert_eq!(index.get(terminal).unwrap().unwrap().value, 41);
        assert_eq!(
            index
                .get(Key::new(first.source, 0, 10))
                .unwrap()
                .map(|leaf| leaf.value),
            None
        );
        let snapshot = index.clone();
        let leaf = producer
            .record(|retained| Leaf {
                value: 53,
                _retained: retained,
            })
            .unwrap();
        index
            .insert(
                terminal,
                leaf,
                custody::Usage::new(size_of::<Leaf>(), 0),
                true,
                &producer,
            )
            .unwrap();
        assert_eq!(snapshot.get(terminal).unwrap().unwrap().value, 41);
        assert_eq!(index.get(terminal).unwrap().unwrap().value, 53);
        assert_eq!(index.get(first).unwrap().unwrap().value, 11);
        assert!(matches!(
            index.get(Key::new(
                DomHostedSourceId {
                    owner: DomOwnerId::allocate().unwrap(),
                    serial: 7
                },
                0,
                9
            )),
            Err(DomError::ForeignOwner)
        ));
        assert_eq!(
            [
                chunk(u64::MAX, 0),
                chunk(u64::MAX, 1),
                chunk(u64::MAX, 2),
                chunk(u64::MAX, 3)
            ],
            [65535; 4]
        );
    }
}
