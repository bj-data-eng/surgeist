//! Controlled sparse auxiliary frontiers. Semantic roots contain identity only;
//! original hosted producer leases and owner DOM custody remain distinct.
pub(in crate::store) mod index;
use super::super::{DomError, State, bank, custody, hosted, registry};
use custody::{Debit, MaterializationId, SharedDebit, Usage};
use std::{cell::RefCell, mem::size_of, sync::Arc};
#[cfg(test)]
pub(in crate::store) fn allocated(bytes: usize) {
    if crate::store::scale_metrics::ENABLED {
        crate::store::scale_metrics::record(
            crate::store::scale_metrics::Event::StructuralAllocation { bytes },
        );
    }
}
#[cfg(test)]
fn materialized(bytes: usize) {
    if crate::store::scale_metrics::ENABLED {
        crate::store::scale_metrics::record(
            crate::store::scale_metrics::Event::SequenceMaterialized { bytes },
        );
    }
}
type Admit<'a> = dyn FnMut(Usage) -> Result<(), DomError> + 'a;
type Step<'a> = dyn FnMut() -> Result<(), DomError> + 'a;
type DescribeRow<'a, T> = dyn Fn(&T) -> Result<(MaterializationId, Usage), DomError> + 'a;
struct HistoryChildren {
    _navigables: bank::BankCustody<hosted::history::Row<hosted::history::Navigable>>,
    _entries: bank::BankCustody<hosted::history::Row<hosted::history::Entry>>,
    _navigable_ids: index::IndexCustody<index::Ordinal>,
    _entry_ids: index::IndexCustody<index::Ordinal>,
    _documents: DocumentsCustody,
}
struct DocumentsCustody {
    _slots: bank::BankCustody<index::Generations<hosted::history::documents::Members>>,
}
struct DocumentMembersCustody {
    _rows: bank::BankCustody<hosted::history::documents::Member>,
}
struct RegistryChildren {
    _records: bank::BankCustody<registry::Record>,
}
struct GlobalsChildren {
    _entries: index::IndexCustody<hosted::GlobalMapping>,
}
pub(in crate::store) struct HistoryCustody(bank::Attached<HistoryChildren>);
pub(in crate::store) struct RegistryCustody(bank::Attached<RegistryChildren>);
pub(in crate::store) struct GlobalsCustody(bank::Attached<GlobalsChildren>);
impl Clone for HistoryCustody {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Clone for RegistryCustody {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Clone for GlobalsCustody {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
#[derive(Clone, Default)]
pub(in crate::store) struct Auxiliary {
    pub registries: Option<RegistryCustody>,
    pub globals: Option<GlobalsCustody>,
    pub history: Option<HistoryCustody>,
}
pub(in crate::store) struct Prepared<T, C> {
    semantic: Arc<T>,
    custody: C,
}
impl<T, C> Prepared<T, C> {
    pub fn into_parts(self) -> (Arc<T>, C) {
        (self.semantic, self.custody)
    }
}
/// The complete semantic candidate precedes every receipt/staging allocation.
struct Candidate<'a, 'p, 's, T> {
    semantic: Arc<T>,
    adopter: Adopter<'a, 'p, 's>,
}
struct Pool {
    aliases: Vec<SharedDebit>,
    root: Option<bank::ObjectCustody>,
    pending: Option<Debit>,
    shared: Option<SharedDebit>,
    staging: Option<Debit>,
    incoming: Option<Debit>,
}
struct Adopter<'a, 'p, 's> {
    pool: RefCell<Pool>,
    registry: &'a bank::CustodyRegistry,
    origin: MaterializationId,
    admit: RefCell<&'a mut Admit<'p>>,
    step: RefCell<&'a mut Step<'s>>,
}
impl<'a, 'p, 's> Adopter<'a, 'p, 's> {
    fn new(
        incoming: Option<Debit>,
        origin: MaterializationId,
        registry: &'a bank::CustodyRegistry,
        admit: &'a mut Admit<'p>,
        step: &'a mut Step<'s>,
    ) -> Self {
        Self {
            pool: RefCell::new(Pool {
                aliases: Vec::new(),
                root: None,
                pending: None,
                shared: None,
                staging: None,
                incoming,
            }),
            registry,
            origin,
            admit: RefCell::new(admit),
            step: RefCell::new(step),
        }
    }
    fn admit(&self, usage: Usage) -> Result<(), DomError> {
        (self.admit.borrow_mut())(usage)
    }
    fn step(&self) -> Result<(), DomError> {
        (self.step.borrow_mut())()
    }
    /// Install the exact split in the guard BEFORE any fallible metadata work.
    /// Every returned alias remains guard-owned until the complete transfer.
    fn retained(
        &self,
        origin: Option<MaterializationId>,
        usage: Usage,
    ) -> Result<Option<SharedDebit>, DomError> {
        let mut pool = self.pool.borrow_mut();
        if origin != Some(self.origin) || pool.incoming.is_none() {
            return Ok(None);
        }
        if pool.aliases.len() == pool.aliases.capacity() {
            let target = pool
                .aliases
                .capacity()
                .max(2)
                .checked_mul(2)
                .ok_or(DomError::RepresentationExhausted)?;
            let bytes = target
                .checked_mul(size_of::<SharedDebit>())
                .ok_or(DomError::RepresentationExhausted)?;
            // Both real backings survive until the infallible move completes.
            // The replacement guard destroys unused/partial data before debit
            // release on allocation or actual-excess refusal.
            let debit = self
                .registry
                .ledger()
                .retain(Usage::new(bytes, 0), |usage| self.admit(usage))?;
            let mut replacement = custody::Held::new(Vec::new(), debit);
            replacement
                .value_mut()
                .try_reserve_exact(target)
                .map_err(|_| DomError::AllocationFailed)?;
            #[cfg(test)]
            allocated(replacement.value().capacity() * size_of::<SharedDebit>());
            let extra = replacement
                .value()
                .capacity()
                .checked_sub(target)
                .and_then(|extra| extra.checked_mul(size_of::<SharedDebit>()))
                .ok_or(DomError::RepresentationExhausted)?;
            replacement
                .debit_mut()
                .grow(Usage::new(extra, 0), |usage| self.admit(usage))?;
            // Prepay every initialized alias move before changing the old pool.
            // Work refusal leaves its backing and receipts intact; Held releases
            // the unused replacement backing before its admission receipt.
            for _ in 0..pool.aliases.len() {
                self.step()?;
            }
            for alias in pool.aliases.drain(..) {
                replacement.value_mut().push(alias);
                #[cfg(test)]
                materialized(size_of::<SharedDebit>());
            }
            let (aliases, debit) = replacement.into_parts();
            let previous = (
                std::mem::replace(&mut pool.aliases, aliases),
                pool.staging.replace(debit),
            );
            // Tuple destruction frees the old allocation before its receipt.
            drop(previous);
        }
        pool.pending = Some(
            pool.incoming
                .as_mut()
                .expect("present producer pool")
                .split(usage)?,
        );
        pool.shared = Some(
            pool.pending
                .as_mut()
                .expect("guard-owned split")
                .share(|usage| self.admit(usage))?,
        );
        let shared = pool
            .shared
            .as_ref()
            .expect("guard-owned shared occurrence")
            .clone();
        pool.aliases.push(shared.clone());
        #[cfg(test)]
        materialized(size_of::<SharedDebit>());
        pool.shared = None;
        pool.pending = None;
        Ok(Some(shared))
    }
    fn description(
        &self,
        origin: MaterializationId,
        usage: Usage,
    ) -> Result<bank::RecordDescription, DomError> {
        let description = bank::RecordDescription::from(usage);
        Ok(match self.retained(Some(origin), usage)? {
            Some(retained) => description.with_retained(retained),
            None => description,
        })
    }
    fn root(&self, id: MaterializationId, usage: Usage) -> Result<bank::ObjectCustody, DomError> {
        let root = match self.retained(Some(self.origin), usage)? {
            Some(retained) => self
                .registry
                .adopt_object_retained(id, &retained, |usage| self.admit(usage))?,
            None => self
                .registry
                .adopt_object(id, usage, |usage| self.admit(usage))?,
        };
        self.pool.borrow_mut().root = Some(root.clone());
        Ok(root)
    }
    fn attach<T: std::any::Any + Send + Sync>(
        &self,
        root: bank::ObjectCustody,
        value: T,
    ) -> Result<bank::Attached<T>, DomError> {
        let debit = self
            .registry
            .ledger()
            .retain(Usage::new(size_of::<T>(), 0), |usage| self.admit(usage))?;
        let value = Arc::new(value);
        #[cfg(test)]
        allocated(size_of::<T>());
        Ok(root.attach(custody::Held::new(value, debit)))
    }
    fn finish(&self) -> Result<(), DomError> {
        if self
            .pool
            .borrow()
            .incoming
            .as_ref()
            .is_some_and(|incoming| incoming.usage() != Usage::ZERO)
        {
            return Err(DomError::RepresentationExhausted);
        }
        Ok(())
    }
    fn pages(
        &self,
        _id: MaterializationId,
        origin: Option<MaterializationId>,
        usage: Usage,
    ) -> Result<Option<SharedDebit>, DomError> {
        self.retained(origin, usage)
    }
    fn rows<T>(
        &self,
        bank: &bank::Bank<T>,
        describe: &DescribeRow<'_, T>,
    ) -> Result<bank::BankCustody<T>, DomError> {
        bank.adopt_controlled_from(
            self.registry,
            |row| {
                self.step()?;
                let (origin, usage) = describe(row)?;
                self.description(origin, usage)
            },
            |id, origin, usage| self.pages(id, origin, usage),
            |usage| self.admit(usage),
        )
    }
    fn wide<T: Send + Sync + 'static>(
        &self,
        wide: &index::U64Bank<T>,
        describe: &dyn Fn(&T) -> Result<bank::RecordDescription, DomError>,
    ) -> Result<index::U64Custody<T>, DomError> {
        let branches = wide.branches.adopt_controlled_from(
            self.registry,
            |branch| {
                self.step()?;
                let usage = Usage::new(size_of::<index::U64Branch<T>>(), 0);
                let retained = self.retained(Some(branch.origin), usage)?;
                let next = self.wide(&branch.next, describe)?;
                let dependency = bank::Dependency::new(
                    index::U64Children { _next: next },
                    self.registry.ledger(),
                    |usage| self.admit(usage),
                )?;
                let description = bank::RecordDescription::with_dependency(usage, dependency);
                Ok::<_, DomError>(match retained {
                    Some(retained) => description.with_retained(retained),
                    None => description,
                })
            },
            |id, origin, usage| self.pages(id, origin, usage),
            |usage| self.admit(usage),
        )?;
        let values = wide.values.adopt_controlled_from(
            self.registry,
            describe,
            |id, origin, usage| self.pages(id, origin, usage),
            |usage| self.admit(usage),
        )?;
        Ok(index::U64Custody { branches, values })
    }
    fn index<T: Send + Sync + 'static>(
        &self,
        index: &index::Index<T>,
        describe: &DescribeRow<'_, T>,
    ) -> Result<index::IndexCustody<T>, DomError> {
        let sources = self.wide(&index.sources, &|source| {
            self.step()?;
            let retained = self.retained(
                Some(source.origin),
                Usage::new(size_of::<index::Issuer<T>>(), 0),
            )?;
            let slots = source.slots.adopt_controlled_from(
                self.registry,
                |slot| {
                    self.step()?;
                    let usage = Usage::new(size_of::<index::Generations<T>>(), 0);
                    let retained = self.retained(Some(slot.origin), usage)?;
                    let values = self.wide(&slot.values, &|row| {
                        self.step()?;
                        let (origin, usage) = describe(row)?;
                        self.description(origin, usage)
                    })?;
                    let dependency = bank::Dependency::new(
                        index::GenerationsCustody { _values: values },
                        self.registry.ledger(),
                        |usage| self.admit(usage),
                    )?;
                    let description = bank::RecordDescription::with_dependency(usage, dependency);
                    Ok::<_, DomError>(match retained {
                        Some(retained) => description.with_retained(retained),
                        None => description,
                    })
                },
                |id, origin, usage| self.pages(id, origin, usage),
                |usage| self.admit(usage),
            )?;
            let dependency = bank::Dependency::new(
                index::IssuerCustody { _slots: slots },
                self.registry.ledger(),
                |usage| self.admit(usage),
            )?;
            let description = bank::RecordDescription::with_dependency(
                Usage::new(size_of::<index::Issuer<T>>(), 0),
                dependency,
            );
            Ok::<_, DomError>(match retained {
                Some(retained) => description.with_retained(retained),
                None => description,
            })
        })?;
        Ok(index::IndexCustody { sources })
    }
    fn history(&self, graph: &hosted::history::Graph) -> Result<HistoryCustody, DomError> {
        let root = self.root(graph.custody_identity(), graph.custody_usage()?)?;
        if let Some(existing) = root.attachment::<HistoryChildren>() {
            return Ok(HistoryCustody(existing));
        }
        let navigables = self.rows(graph.navigable_bank(), &|row| {
            Ok((row.custody_origin(), row.custody_payload()?))
        })?;
        let entries = self.rows(graph.entry_bank(), &|row| {
            Ok((row.custody_origin(), row.custody_payload()?))
        })?;
        let ordinal =
            |row: &index::Ordinal| Ok((row.origin, Usage::new(size_of::<index::Ordinal>(), 0)));
        let navigable_ids = self.index(graph.navigable_index(), &ordinal)?;
        let entry_ids = self.index(graph.entry_index(), &ordinal)?;
        let documents = self.documents(graph.document_index())?;
        Ok(HistoryCustody(self.attach(
            root,
            HistoryChildren {
                _navigables: navigables,
                _entries: entries,
                _navigable_ids: navigable_ids,
                _entry_ids: entry_ids,
                _documents: documents,
            },
        )?))
    }
    fn documents(
        &self,
        documents: &hosted::history::documents::Index,
    ) -> Result<DocumentsCustody, DomError> {
        use hosted::history::documents::{Member, Members};
        let slots = documents.slots.adopt_controlled_from(
            self.registry,
            |slot| {
                self.step()?;
                let usage = Usage::new(size_of::<index::Generations<Members>>(), 0);
                let retained = self.retained(Some(slot.origin), usage)?;
                let values = self.wide(&slot.values, &|members| {
                    self.step()?;
                    let usage = Usage::new(size_of::<Members>(), 0);
                    let retained = self.retained(Some(members.origin), usage)?;
                    let rows = self.rows(&members.rows, &|row| {
                        Ok((row.origin, Usage::new(size_of::<Member>(), 0)))
                    })?;
                    let dependency = bank::Dependency::new(
                        DocumentMembersCustody { _rows: rows },
                        self.registry.ledger(),
                        |usage| self.admit(usage),
                    )?;
                    let description = bank::RecordDescription::with_dependency(usage, dependency);
                    Ok(match retained {
                        Some(retained) => description.with_retained(retained),
                        None => description,
                    })
                })?;
                let dependency = bank::Dependency::new(
                    index::GenerationsCustody { _values: values },
                    self.registry.ledger(),
                    |usage| self.admit(usage),
                )?;
                let description = bank::RecordDescription::with_dependency(usage, dependency);
                Ok::<_, DomError>(match retained {
                    Some(retained) => description.with_retained(retained),
                    None => description,
                })
            },
            |id, origin, usage| self.pages(id, origin, usage),
            |usage| self.admit(usage),
        )?;
        Ok(DocumentsCustody { _slots: slots })
    }
    fn registries(&self, graph: &registry::Bank) -> Result<RegistryCustody, DomError> {
        let root = self.root(graph.custody_identity(), graph.custody_usage()?)?;
        if let Some(existing) = root.attachment::<RegistryChildren>() {
            return Ok(RegistryCustody(existing));
        }
        let records = self.rows(graph.custody_bank(), &|row| {
            Ok((row.custody_origin(), row.custody_payload()))
        })?;
        Ok(RegistryCustody(
            self.attach(root, RegistryChildren { _records: records })?,
        ))
    }
    fn globals(&self, graph: &hosted::GlobalDocumentIndex) -> Result<GlobalsCustody, DomError> {
        let root = self.root(graph.custody_identity(), graph.custody_usage())?;
        if let Some(existing) = root.attachment::<GlobalsChildren>() {
            return Ok(GlobalsCustody(existing));
        }
        let entries = self.index(graph.custody_bank(), &|row| {
            Ok((
                row.origin,
                Usage::new(size_of::<hosted::GlobalMapping>(), 0),
            ))
        })?;
        Ok(GlobalsCustody(
            self.attach(root, GlobalsChildren { _entries: entries })?,
        ))
    }
}

pub(in crate::store) fn prepare_history(
    semantic: Arc<hosted::history::Graph>,
    incoming: Option<Debit>,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<Prepared<hosted::history::Graph, HistoryCustody>, DomError> {
    let origin = semantic.custody_origin();
    let candidate = Candidate {
        semantic,
        adopter: Adopter::new(incoming, origin, registry, admit, step),
    };
    let custody = candidate.adopter.history(&candidate.semantic)?;
    candidate.adopter.finish()?;
    Ok(Prepared {
        semantic: candidate.semantic,
        custody,
    })
}
pub(in crate::store) fn prepare_registry(
    semantic: Arc<registry::Bank>,
    incoming: Option<Debit>,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<Prepared<registry::Bank, RegistryCustody>, DomError> {
    let origin = semantic.custody_identity();
    let candidate = Candidate {
        semantic,
        adopter: Adopter::new(incoming, origin, registry, admit, step),
    };
    let custody = candidate.adopter.registries(&candidate.semantic)?;
    candidate.adopter.finish()?;
    Ok(Prepared {
        semantic: candidate.semantic,
        custody,
    })
}
pub(in crate::store) fn prepare_globals(
    semantic: Arc<hosted::GlobalDocumentIndex>,
    incoming: Option<Debit>,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<Prepared<hosted::GlobalDocumentIndex, GlobalsCustody>, DomError> {
    let origin = semantic.custody_identity();
    let candidate = Candidate {
        semantic,
        adopter: Adopter::new(incoming, origin, registry, admit, step),
    };
    let custody = candidate.adopter.globals(&candidate.semantic)?;
    candidate.adopter.finish()?;
    Ok(Prepared {
        semantic: candidate.semantic,
        custody,
    })
}
pub(in crate::store) fn adopt_history(
    semantic: &hosted::history::Graph,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<HistoryCustody, DomError> {
    Adopter::new(None, semantic.custody_origin(), registry, admit, step).history(semantic)
}
pub(in crate::store) fn adopt_registry(
    semantic: &registry::Bank,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<RegistryCustody, DomError> {
    Adopter::new(None, semantic.custody_identity(), registry, admit, step).registries(semantic)
}
pub(in crate::store) fn adopt_globals(
    semantic: &hosted::GlobalDocumentIndex,
    registry: &bank::CustodyRegistry,
    admit: &mut Admit<'_>,
    step: &mut Step<'_>,
) -> Result<GlobalsCustody, DomError> {
    Adopter::new(None, semantic.custody_identity(), registry, admit, step).globals(semantic)
}
impl Auxiliary {
    pub fn adopt(
        state: &State,
        registry: &bank::CustodyRegistry,
        admit: &mut Admit<'_>,
        step: &mut Step<'_>,
    ) -> Result<Self, DomError> {
        Ok(Self {
            registries: state
                .registries
                .as_deref()
                .map(|value| adopt_registry(value, registry, admit, step))
                .transpose()?,
            globals: state
                .hosted_globals
                .as_deref()
                .map(|value| adopt_globals(value, registry, admit, step))
                .transpose()?,
            history: state
                .hosted_history
                .as_deref()
                .map(|value| adopt_history(value, registry, admit, step))
                .transpose()?,
        })
    }
}

#[cfg(test)]
impl HistoryCustody {
    pub(in crate::store) fn inspect_claimed_materializations(
        &self,
    ) -> Vec<(MaterializationId, Usage)> {
        let mut output = vec![self.0.inspect_owner_payload()];
        output.extend(self.0._navigables.inspect_materializations());
        output.extend(self.0._entries.inspect_materializations());
        output.extend(self.0._navigable_ids.inspect_materializations());
        output.extend(self.0._entry_ids.inspect_materializations());
        output.extend(self.0._documents._slots.inspect_materializations());
        for slot in self
            .0
            ._documents
            ._slots
            .inspect_dependencies::<index::GenerationsCustody<hosted::history::documents::Members>>(
            )
        {
            output.extend(slot._values.inspect_materializations());
            for members in slot
                ._values
                .inspect_dependencies::<DocumentMembersCustody>()
            {
                output.extend(members._rows.inspect_materializations());
            }
        }
        output
    }
}
#[cfg(test)]
impl RegistryCustody {
    pub(in crate::store) fn inspect_claimed_materializations(
        &self,
    ) -> Vec<(MaterializationId, Usage)> {
        let mut output = vec![self.0.inspect_owner_payload()];
        output.extend(self.0._records.inspect_materializations());
        output
    }
}
#[cfg(test)]
impl Auxiliary {
    pub(in crate::store) fn inspect_claimed_materializations(
        &self,
    ) -> Vec<(MaterializationId, Usage)> {
        let mut output = Vec::new();
        if let Some(value) = &self.history {
            output.extend(value.inspect_claimed_materializations());
        }
        if let Some(value) = &self.registries {
            output.extend(value.inspect_claimed_materializations());
        }
        if let Some(value) = &self.globals {
            output.push(value.0.inspect_owner_payload());
            output.extend(value.0._entries.inspect_materializations());
        }
        output
    }
}

#[cfg(test)]
mod tests;
