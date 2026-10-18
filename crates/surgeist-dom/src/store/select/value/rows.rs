//! Call-scoped identity lookup into the original capture entries. The opaque
//! builder subjects stay in their original ordered buffer exactly once.
use super::*;
use source::effects::ScratchRetained;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
struct Row {
    key: Key,
    ordinal: usize,
}
struct Pair {
    values: bank::Bank<Row>,
    custody: bank::BankCustody<Row>,
}

/// All local materializations belong exclusively to this synchronous capture.
/// One global affine bridge and one Select scratch receipt retain exactly its
/// local ledger plus this inline header. No producer/input graph enters here.
pub(super) struct Rows {
    existing: bank::Bank<Row>,
    created: bank::Bank<Row>,
    existing_custody: bank::BankCustody<Row>,
    created_custody: bank::BankCustody<Row>,
    registry: Option<bank::CustodyRegistry>,
    ledger_admin: Option<custody::Debit>,
    ledger: Option<custody::Ledger>,
    bridge: custody::Debit,
    scratch: ScratchRetained,
    mirrored: usize,
}
impl Rows {
    fn header() -> custody::Usage {
        custody::Usage::new(size_of::<Self>(), 0)
    }
    pub fn new(draft: &Draft, cancel: &AtomicBool) -> Result<Self, DomError> {
        let initial = Self::header().checked_add(custody::Ledger::administration_usage())?;
        draft.select_scratch(initial.bytes())?;
        let bridge = draft
            .custody_ledger()
            .retain(initial, |usage| draft.admit_custody_usage(usage, cancel))?;
        // The actual Ledger target is covered before allocation. Its local
        // admin debit classifies it once; it does not add a second global charge.
        let ledger = custody::Ledger::default();
        let ledger_admin = ledger.retain(custody::Ledger::administration_usage(), |_| Ok(()))?;
        let scratch = ScratchRetained::new(draft, initial.bytes(), false)?;
        let mut rows = Self {
            existing: Default::default(),
            created: Default::default(),
            existing_custody: Default::default(),
            created_custody: Default::default(),
            registry: None,
            ledger_admin: Some(ledger_admin),
            ledger: Some(ledger.clone()),
            bridge,
            scratch,
            mirrored: initial.bytes(),
        };
        let result = bank::CustodyRegistry::new(&ledger, |usage| rows.admit(draft, usage, cancel));
        let error = match result {
            Ok(registry) => {
                rows.registry = Some(registry);
                None
            }
            Err(error) => Some(error),
        };
        rows.sync();
        drop(ledger);
        match error {
            Some(error) => Err(error),
            None => Ok(rows),
        }
    }
    fn admit(
        &mut self,
        draft: &Draft,
        local: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let owned = Self::header().checked_add(local)?;
        // The steady old map is already mirrored. The local prospective total
        // also contains every provisional old/new path retained by this call.
        draft.select_scratch(owned.bytes().saturating_sub(self.mirrored))?;
        transaction::nodes::resize_debit(&mut self.bridge, owned, |usage| {
            draft.admit_custody_usage(usage, cancel)
        })
    }
    fn sync(&mut self) {
        if let Some(registry) = &self.registry {
            registry.reconcile_releases();
        }
        let actual = Self::header()
            .checked_add(self.ledger.as_ref().expect("live map ledger").total())
            .expect("already admitted local graph");
        let excess = self
            .bridge
            .usage()
            .checked_sub(actual)
            .expect("every local allocation was bridged before allocation");
        if excess != custody::Usage::ZERO {
            drop(self.bridge.split(excess).expect("owned bridge excess"));
        }
        self.scratch.replace_admitted(actual.bytes());
        self.mirrored = actual.bytes();
    }
    pub fn insert(
        &mut self,
        draft: &Draft,
        key: Key,
        ordinal: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        draft.select_step(cancel)?;
        let key = draft.live_key(key);
        if let Key::Existing(id) = key
            && id.owner != draft.store
        {
            return Err(DomError::ForeignOwner);
        }
        let registry = self
            .registry
            .as_ref()
            .expect("initialized map registry")
            .clone();
        let result = (|| {
            // These O(1) paired aliases keep the original path intact through
            // every refusal. All provisional aliases drop before sync below.
            let (old, slot) = match key {
                Key::Existing(id) => (
                    Pair {
                        values: self.existing.clone(),
                        custody: self.existing_custody.clone(),
                    },
                    id.slot,
                ),
                Key::Created(slot) => (
                    Pair {
                        values: self.created.clone(),
                        custody: self.created_custody.clone(),
                    },
                    slot,
                ),
            };
            if old.values.get(slot).is_some() {
                return Err(DomError::SelectProtocol);
            }
            let row = bank::ControlledRecord::new(
                Row { key, ordinal },
                custody::Usage::new(size_of::<Row>(), 0),
                &registry,
                |usage| self.admit(draft, usage, cancel),
            )?;
            let (next, control) = old.values.set_controlled(
                &old.custody,
                slot,
                row,
                false,
                bank::Totals {
                    bytes: size_of::<Row>(),
                    units: 0,
                    occurrences: 1,
                },
                &registry,
                |usage| self.admit(draft, usage, cancel),
            )?;
            match key {
                Key::Existing(_) => {
                    self.existing = next;
                    self.existing_custody = control;
                }
                Key::Created(_) => {
                    self.created = next;
                    self.created_custody = control;
                }
            }
            Ok(())
        })();
        self.sync();
        result
    }
    pub fn get(&self, draft: &Draft, key: Key, cancel: &AtomicBool) -> Result<usize, DomError> {
        draft.select_step(cancel)?;
        let key = draft.live_key(key);
        let row = match key {
            Key::Existing(id) => self.existing.get(id.slot),
            Key::Created(slot) => self.created.get(slot),
        }
        .ok_or(DomError::SelectProtocol)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            super::super::discovery_observation::record(
                super::super::discovery_observation::Event::MapCompared,
            );
        }
        if row.key != key {
            return Err(DomError::SelectProtocol);
        }
        Ok(row.ordinal)
    }
}
impl Drop for Rows {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.existing));
        drop(std::mem::take(&mut self.created));
        drop(std::mem::take(&mut self.existing_custody));
        drop(std::mem::take(&mut self.created_custody));
        if let Some(registry) = &self.registry {
            registry.reconcile_releases();
        }
        drop(self.registry.take());
        drop(self.ledger_admin.take());
        drop(self.ledger.take());
        // The global bridge and scratch receipt drop only after all represented
        // local semantic, custody, registry and ledger objects above are gone.
    }
}
