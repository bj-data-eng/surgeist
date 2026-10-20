//! Actual complete-empty Global registry formation and immutable bank custody.

#[cfg(test)]
mod tests;

use super::*;
use crate::hosted::{
    DomHostedError, DomHostedResource, DomHostedSource, RegistryQuota, Retained, Work,
};
use crate::registry::{DomRegistry, DomRegistryId};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureRegistryId, HtmlEmptyGlobalRegistryError, HtmlHostFactsRevision,
};

#[derive(Debug)]
pub(crate) struct Record {
    origin: custody::MaterializationId,
    pub(crate) capture: HtmlCapture,
    pub(crate) local: HtmlCaptureRegistryId,
    pub(crate) payload_bytes: usize,
    _payload: Retained,
    _header: Retained,
    _quota: RegistryQuota,
}

#[derive(Debug)]
pub(crate) struct Bank {
    identity: custody::MaterializationId,
    owner: DomOwnerId,
    records: bank::Bank<Record>,
    _retained: Retained,
}
impl PartialEq for Bank {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.records.shares_root(&other.records)
    }
}
impl Eq for Bank {}
impl Bank {
    pub(in crate::store) fn custody_identity(&self) -> custody::MaterializationId {
        self.identity
    }
    pub(in crate::store) fn custody_usage(&self) -> Result<custody::Usage, DomError> {
        Ok(custody::Usage::new(size_of::<Self>(), 0))
    }
    pub(in crate::store) fn custody_bank(&self) -> &bank::Bank<Record> {
        &self.records
    }
    pub(crate) fn get(&self, id: DomRegistryId) -> Result<&Record, DomError> {
        if id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if id.generation != 1 {
            return Err(DomError::StaleNode);
        }
        self.records.get(id.slot).ok_or(DomError::StaleNode)
    }
    pub(crate) fn form(
        old: Option<&Self>,
        owner: DomOwnerId,
        source: &DomHostedSource,
        work: &Work,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(Arc<Self>, DomRegistryId), DomHostedError> {
        work.step(cancel)?;
        let identity = custody::MaterializationId::fresh()?;
        let record_identity = bank::Record::<Record>::reserve()?;
        let count = old.map_or(0, |bank| bank.records.len());
        let required = count
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            required,
            source.limits().records().registries(),
            DomHostedResource::Registries,
        )?;
        if let Some(formation) = formation {
            formation.retain_bytes(size_of::<Record>(), cancel)?;
        }
        let header = source.retain(size_of::<Record>(), cancel)?;
        let quota = source.retain_registry(cancel)?;
        let hosted_allowance = source
            .remaining_bytes()?
            .min(source.limits().scratch_bytes());
        let parent_allowance = formation
            .map(|formation| formation.remaining_bytes())
            .transpose()?;
        let allowance =
            parent_allowance.map_or(hosted_allowance, |bytes| bytes.min(hosted_allowance));
        // The producer owns its private peak measurement. Reserve the supplied
        // finite envelope before it forms anything, then transfer only the real
        // returned graph extent when its temporary workspace has been dropped.
        if let Some(formation) = formation {
            formation.retain_bytes(allowance, cancel)?;
        }
        let formed = HtmlCapture::complete_empty_global_registry_with(
            HtmlHostFactsRevision::fresh(),
            allowance,
            |amount| {
                work.spend(amount, cancel)?;
                if let Some(formation) = formation {
                    for _ in 0..amount {
                        formation.step(cancel)?;
                    }
                }
                Ok::<_, DomHostedError>(())
            },
        )
        .map_err(|error| match error {
            HtmlEmptyGlobalRegistryError::Interrupted(error) => error,
            HtmlEmptyGlobalRegistryError::FormationBytes { limit, .. } => {
                if parent_allowance.is_some_and(|bytes| bytes <= hosted_allowance) {
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: formation
                            .expect("actual parent envelope")
                            .retained_byte_limit(),
                    }
                    .into()
                } else {
                    DomHostedError::Limit {
                        resource: DomHostedResource::RetainedBytes,
                        limit,
                    }
                }
            }
            HtmlEmptyGlobalRegistryError::RepresentationExhausted => {
                DomHostedError::RepresentationExhausted
            }
            HtmlEmptyGlobalRegistryError::AllocationFailed => DomHostedError::AllocationFailed,
            HtmlEmptyGlobalRegistryError::Capture(_) => DomHostedError::UnsupportedProfile,
        })?;
        let payload_bytes = formed.retained_storage_usage().retained_bytes();
        let payload = source.retain(payload_bytes, cancel)?;
        let (capture, local) = formed.into_parts();
        if let Some(formation) = formation {
            formation.release_payload(source::storage::Payload {
                bytes: allowance
                    .checked_sub(payload_bytes)
                    .ok_or(DomError::RepresentationExhausted)?,
                units: 0,
            });
        }
        let record = record_identity.materialize(Record {
            origin: identity,
            capture,
            local,
            payload_bytes,
            _payload: payload,
            _header: header,
            _quota: quota,
        });
        let scratch = std::cell::Cell::new(0);
        let producer = transaction::auxiliary::index::Producer {
            source,
            work,
            formation,
            cancel,
            origin: identity,
            scratch: &scratch,
        };
        let retained = producer.retain(custody::Usage::new(size_of::<Self>(), 0))?;
        if old.is_some_and(|old| old.owner != owner) {
            return Err(DomError::ForeignOwner.into());
        }
        let mut candidate = Self {
            identity,
            owner,
            records: old.map_or_else(bank::Bank::default, |old| old.records.clone()),
            _retained: retained,
        };
        producer.set(
            &mut candidate.records,
            count,
            record,
            custody::Usage::new(
                size_of::<Record>()
                    .checked_add(payload_bytes)
                    .ok_or(DomError::RepresentationExhausted)?,
                0,
            ),
        )?;
        let candidate = Arc::new(candidate);
        #[cfg(test)]
        transaction::auxiliary::allocated(size_of::<Self>());
        Ok((
            candidate,
            DomRegistryId {
                owner,
                slot: count,
                generation: 1,
            },
        ))
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_materializations(
        &self,
    ) -> Result<Vec<(custody::MaterializationId, custody::Usage)>, DomError> {
        let mut objects = vec![(self.identity, custody::Usage::new(size_of::<Self>(), 0))];
        let graph = self.records.inspect_graph(|record| {
            let capture = record
                .capture
                .retained_storage_usage_with(|| Ok::<_, ()>(()))
                .map_err(|_| DomError::RepresentationExhausted)?;
            Ok(bank::Totals {
                bytes: size_of::<Record>() + capture.retained_bytes(),
                units: 0,
                occurrences: 0,
            })
        })?;
        objects.extend(graph.materializations());
        Ok(objects)
    }
}

impl Record {
    pub(in crate::store) fn custody_origin(&self) -> custody::MaterializationId {
        self.origin
    }
    pub(in crate::store) fn custody_payload(&self) -> custody::Usage {
        custody::Usage::new(size_of::<Self>() + self.payload_bytes, 0)
    }
}

impl DomSnapshot {
    pub fn registry(&self, id: DomRegistryId) -> Result<DomRegistry<'_>, DomError> {
        self.require_complete_source()?;
        self.state
            .registries
            .as_ref()
            .ok_or(DomError::StaleNode)?
            .get(id)?;
        Ok(DomRegistry { snapshot: self, id })
    }
}

impl Draft {
    pub(super) fn select_element_registry(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let node = self.node(key)?;
        let Some(element) = node.element() else {
            return Ok(());
        };
        let metadata = self
            .document_metadata(node.document())
            .map_err(|error| match error {
                DomError::WrongNodeKind => DomError::WrongDocument,
                error => error,
            })?;
        let binding = metadata.custom_element_registry();
        if let crate::DomRegistryBinding::Registry(id) = binding {
            let record = self
                .registries
                .as_ref()
                .ok_or(DomError::HostedSourceUnavailable)?
                .get(id)?;
            if element.common_state().html().is_some() {
                let registry =
                    surgeist_html_semantics::HtmlCapturedRegistry::Registry(record.local.clone());
                let mut cursor = record
                    .capture
                    .definition_lookup_cursor(
                        &registry,
                        element.name().local_name(),
                        element.saved_is(),
                    )
                    .map_err(DomError::SelectCapture)?;
                loop {
                    for _ in 0..cursor.next_cost() {
                        self.source_step(cancel)?;
                    }
                    match cursor.step() {
                        surgeist_html_semantics::HtmlDefinitionLookupStep::Searching => {}
                        surgeist_html_semantics::HtmlDefinitionLookupStep::Ready(None) => break,
                        surgeist_html_semantics::HtmlDefinitionLookupStep::Ready(Some(_)) => {
                            return Err(DomError::NotSupported);
                        }
                    }
                }
            }
        }
        self.establish_element_registry(key, binding, cancel)?;
        Ok(())
    }
}
