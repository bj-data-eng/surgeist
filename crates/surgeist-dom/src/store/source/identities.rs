//! Original allocation identities are monotonic sparse persistent maps. Captures
//! share roots and admission follows only a newly assigned identity's paths.
use super::*;
#[derive(Clone, Copy, PartialEq)]
struct Binding {
    id: NodeId,
    index: usize,
}
#[derive(Clone, Default)]
pub(crate) struct AllocationIds {
    ids: bank::Bank<NodeId>,
    ids_custody: bank::BankCustody<NodeId>,
    inverse: bank::Bank<Binding>,
    inverse_custody: bank::BankCustody<Binding>,
}
impl AllocationIds {
    pub fn get(&self, index: usize) -> Option<NodeId> {
        self.ids.get(index).copied()
    }
    pub fn len(&self) -> usize {
        self.ids.totals().occurrences
    }
    pub fn original(&self, id: NodeId) -> Option<usize> {
        self.inverse
            .get(id.slot)
            .filter(|v| v.id == id)
            .map(|v| v.index)
    }
    pub fn assign(
        &mut self,
        index: usize,
        id: NodeId,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if let Some(before) = self.get(index) {
            assert_eq!(before, id, "original allocation generation is immutable");
            return Ok(());
        }
        let record = bank::ControlledRecord::new(
            id,
            custody::Usage::new(size_of::<NodeId>(), 0),
            registry,
            &mut admit,
        )?;
        // Keep semantic storage before its custody in one guard until every
        // fallible inverse preparation has completed.
        let ids = self.ids.set_controlled(
            &self.ids_custody,
            index,
            record,
            false,
            bank::Totals {
                bytes: size_of::<NodeId>(),
                units: 0,
                occurrences: 1,
            },
            registry,
            &mut admit,
        )?;
        let binding = Binding { id, index };
        let record = bank::ControlledRecord::new(
            binding,
            custody::Usage::new(size_of::<Binding>(), 0),
            registry,
            &mut admit,
        )?;
        let inverse = self.inverse.set_controlled(
            &self.inverse_custody,
            id.slot,
            record,
            false,
            bank::Totals {
                bytes: size_of::<Binding>(),
                units: 0,
                occurrences: 1,
            },
            registry,
            &mut admit,
        )?;
        *self = Self {
            ids: ids.0,
            ids_custody: ids.1,
            inverse: inverse.0,
            inverse_custody: inverse.1,
        };
        Ok(())
    }
}
#[derive(Clone, Default)]
pub(in crate::store) struct Privacy {
    values: bank::Bank<bool>,
    custody: bank::BankCustody<bool>,
}
impl Privacy {
    pub fn get(&self, index: usize) -> bool {
        self.values.get(index).copied().unwrap_or(false)
    }
    pub fn set(
        &mut self,
        index: usize,
        value: bool,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if self.get(index) == value {
            return Ok(());
        }
        let record = bank::ControlledRecord::new(
            value,
            custody::Usage::new(size_of::<bool>(), 0),
            registry,
            &mut admit,
        )?;
        let (values, custody) = self.values.set_controlled(
            &self.custody,
            index,
            record,
            false,
            bank::Totals {
                bytes: size_of::<bool>(),
                units: 0,
                occurrences: 1,
            },
            registry,
            &mut admit,
        )?;
        *self = Self { values, custody };
        Ok(())
    }
}

pub(crate) fn alias_key(key: Key, aliases: &AllocationIds) -> Key {
    match key {
        Key::Existing(id) => aliases.original(id).map_or(key, Key::Created),
        Key::Created(_) => key,
    }
}
pub(crate) fn alias_id(id: NodeId, aliases: &AllocationIds) -> Key {
    alias_key(Key::Existing(id), aliases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_inverse_assignment_keeps_original_maps_and_releases_every_candidate() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                10_000,
                32,
                0,
                0,
                8,
                10_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let first = batch.create_text(&document, "a");
        let second = batch.create_text(&document, "b");
        let commit = dom.apply(batch, &cancel).unwrap();
        let first = commit.created(&first).unwrap().unwrap();
        let second = commit.created(&second).unwrap().unwrap();
        let registry = &dom.custody_registry;
        let mut original = AllocationIds::default();
        original.assign(0, first, registry, |_| Ok(())).unwrap();
        let mut refused = 0;
        for cutoff in 1..128 {
            registry.reconcile_releases();
            let baseline = registry.ledger().total();
            let mut candidate = original.clone();
            let mut admissions = 0;
            let result = candidate.assign(1, second, registry, |_| {
                admissions += 1;
                if admissions == cutoff {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            });
            match result {
                Err(DomError::Cancelled) => {
                    refused += 1;
                    assert_eq!(candidate.get(0), Some(first));
                    assert_eq!(candidate.get(1), None);
                    assert_eq!(candidate.original(first), Some(0));
                    assert_eq!(candidate.original(second), None);
                }
                Ok(()) => {
                    assert_eq!(candidate.get(1), Some(second));
                    assert_eq!(candidate.original(second), Some(1));
                    drop(candidate);
                    registry.reconcile_releases();
                    assert_eq!(registry.ledger().total(), baseline);
                    assert!(
                        refused > 0,
                        "fallible admission was exercised before the successful assignment"
                    );
                    return;
                }
                Err(error) => panic!("original admission error: {error:?}"),
            }
            drop(candidate);
            registry.reconcile_releases();
            assert_eq!(registry.ledger().total(), baseline);
            assert_eq!(original.get(1), None);
        }
        panic!("bounded two-map assignment never completed");
    }
}
