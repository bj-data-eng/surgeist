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
    pub(crate) capture: HtmlCapture,
    pub(crate) local: HtmlCaptureRegistryId,
    pub(crate) payload_bytes: usize,
    _payload: Retained,
    _header: Retained,
    _quota: RegistryQuota,
}

#[derive(Debug)]
pub(crate) struct Bank {
    owner: DomOwnerId,
    records: Vec<Arc<Record>>,
    _retained: Retained,
}
impl PartialEq for Bank {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.records.len() == other.records.len()
            && self
                .records
                .iter()
                .zip(&other.records)
                .all(|(a, b)| Arc::ptr_eq(a, b))
    }
}
impl Eq for Bank {}
impl Bank {
    pub(crate) fn get(&self, id: DomRegistryId) -> Result<&Record, DomError> {
        if id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if id.generation != 1 {
            return Err(DomError::StaleNode);
        }
        self.records
            .get(id.slot)
            .map(Arc::as_ref)
            .ok_or(DomError::StaleNode)
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
        let record = Arc::new(Record {
            capture,
            local,
            payload_bytes,
            _payload: payload,
            _header: header,
            _quota: quota,
        });
        let backing = required
            .checked_mul(size_of::<Arc<Record>>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let header = size_of::<Vec<Arc<Record>>>();
        crate::hosted::admit(
            backing
                .checked_add(header)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let requested = backing
            .checked_add(size_of::<Self>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        if let Some(formation) = formation {
            formation.retain_bytes(requested, cancel)?;
        }
        let retained = source.retain(requested, cancel)?;
        let mut candidate = Self {
            owner,
            records: Vec::new(),
            _retained: retained,
        };
        candidate
            .records
            .try_reserve_exact(required)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let actual = candidate
            .records
            .capacity()
            .checked_mul(size_of::<Arc<Record>>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            actual
                .checked_add(header)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let extra = actual
            .checked_sub(backing)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        if let Some(formation) = formation {
            formation.retain_bytes(extra, cancel)?;
        }
        let extra_retained = source.retain(extra, cancel)?;
        // The exposed capacity check precedes every copy. The requested and
        // extra occurrences are combined without dropping either admission.
        candidate._retained.append(extra_retained)?;
        if let Some(old) = old {
            for record in &old.records {
                work.step(cancel)?;
                if let Some(formation) = formation {
                    formation.retain_bytes(
                        size_of::<Record>()
                            .checked_add(record.payload_bytes)
                            .ok_or(DomError::RepresentationExhausted)?,
                        cancel,
                    )?;
                }
                candidate.records.push(record.clone());
            }
        }
        candidate.records.push(record);
        Ok((
            Arc::new(candidate),
            DomRegistryId {
                owner,
                slot: count,
                generation: 1,
            },
        ))
    }
    pub(crate) fn retained_payload_bytes(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        let mut bytes = size_of::<Self>()
            .checked_add(
                self.records
                    .capacity()
                    .checked_mul(size_of::<Arc<Record>>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        for record in &self.records {
            step()?;
            bytes = bytes
                .checked_add(size_of::<Record>())
                .and_then(|n| n.checked_add(record.payload_bytes))
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(bytes)
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
        let DomNodeKind::Element(element) = &node.kind else {
            return Ok(());
        };
        let DomNodeKind::Document(metadata) = &self.node(node.document)?.kind else {
            return Err(DomError::WrongDocument);
        };
        self.document_environment(node.document)?;
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
        let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
            unreachable!("checked Element")
        };
        element.establish_registry(binding);
        Ok(())
    }
}
