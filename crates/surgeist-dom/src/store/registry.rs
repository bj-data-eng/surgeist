//! Controlled registry formation and immutable bank custody.

#[cfg(test)]
mod tests;

use super::*;
use crate::hosted::{
    DomHostedError, DomHostedGlobalId, DomHostedResource, DomHostedSource, RegistryQuota, Retained,
    Work,
};
use crate::registry::{DomRegistry, DomRegistryDefinitionId, DomRegistryId};
use surgeist_html::accounting::HtmlAccountingFault;

use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureError, HtmlCaptureRegistryId, HtmlCustomElementDefinition,
    HtmlHostFactsRevision, HtmlRegistryFormationError, HtmlRegistryFormationLimits,
};

#[derive(Debug)]
pub(crate) struct Record {
    origin: custody::MaterializationId,
    pub(crate) capture: HtmlCapture,
    pub(crate) local: HtmlCaptureRegistryId,
    pub(crate) scoped_origin: Option<DomHostedGlobalId>,
    pub(crate) payload_bytes: usize,
    payload_units: usize,
    _payload: Retained,
    _header: Retained,
    quota: Arc<RegistryQuota>,
}

impl Record {
    pub(crate) fn definition(
        &self,
        ordinal: usize,
    ) -> Result<&HtmlCustomElementDefinition, HtmlCaptureError> {
        match self.capture.definitions(&self.local)? {
            surgeist_html_semantics::HtmlCapturedDefinitions::Complete(definitions) => definitions
                .get(ordinal)
                .ok_or(HtmlCaptureError::UnknownDefinition),
            surgeist_html_semantics::HtmlCapturedDefinitions::Unavailable => {
                Err(HtmlCaptureError::UnavailableDefinitions(self.local.clone()))
            }
        }
    }

    pub(crate) fn lookup_with(
        &self,
        local: &surgeist_html::values::DocumentString,
        is: Option<&surgeist_html::values::DocumentString>,
        step: &mut impl FnMut(usize) -> Result<(), DomError>,
    ) -> Result<Option<usize>, DomError> {
        let registry = surgeist_html_semantics::HtmlCapturedRegistry::Registry(self.local.clone());
        let mut cursor = self
            .capture
            .definition_lookup_cursor(&registry, local, is)
            .map_err(DomError::SelectCapture)?;
        loop {
            step(cursor.next_cost())?;
            match cursor.step() {
                surgeist_html_semantics::HtmlDefinitionLookupStep::Searching => {}
                surgeist_html_semantics::HtmlDefinitionLookupStep::Ready(result) => {
                    return Ok(result.map(|definition| definition.ordinal()));
                }
            }
        }
    }

    // Each row conservatively charges the shared quota payload it keeps alive.
    // The quota releases one registry count when its last row drops.
    const HEADER_BYTES: usize = size_of::<Self>() + size_of::<RegistryQuota>();
}

pub(in crate::store) enum RegistryEdit<'a> {
    CreateGlobal,
    CreateScoped(DomHostedGlobalId),
    Define(DomRegistryId, &'a HtmlCustomElementDefinition),
}

#[derive(Debug)]
pub(crate) struct Bank {
    identity: custody::MaterializationId,
    owner: DomOwnerId,
    records: bank::Bank<Record>,
    scoped_documents: bank::Bank<ScopedDocuments>,
    has_scoped: bool,
    _retained: Option<Retained>,
}

#[derive(Debug)]
pub(in crate::store) struct ScopedDocuments {
    pub origin: custody::MaterializationId,
    pub documents: Vec<DocumentId>,
}

impl ScopedDocuments {
    pub fn custody_usage(&self) -> Result<custody::Usage, DomError> {
        let bytes = self
            .documents
            .capacity()
            .checked_mul(size_of::<DocumentId>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(custody::Usage::new(bytes, 0))
    }
}
impl PartialEq for Bank {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
            && self.records.shares_root(&other.records)
            && self.scoped_documents.shares_root(&other.scoped_documents)
    }
}
impl Eq for Bank {}
impl Bank {
    pub(in crate::store) const fn has_scoped_registry(&self) -> bool {
        self.has_scoped
    }
    pub(in crate::store) fn custody_identity(&self) -> custody::MaterializationId {
        self.identity
    }
    pub(in crate::store) fn custody_usage(&self) -> Result<custody::Usage, DomError> {
        Ok(custody::Usage::new(size_of::<Self>(), 0))
    }
    pub(in crate::store) fn custody_bank(&self) -> &bank::Bank<Record> {
        &self.records
    }
    pub(in crate::store) fn custody_scoped_documents(&self) -> &bank::Bank<ScopedDocuments> {
        &self.scoped_documents
    }
    pub(crate) fn scoped_documents<E: From<DomError>>(
        &self,
        registry: DomRegistryId,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<&[DocumentId], E> {
        self.get_with(registry, step)?;
        Ok(self
            .scoped_documents
            .get_with(registry.slot, step)?
            .map_or(&[], |row| row.documents.as_slice()))
    }
    pub(crate) fn get(&self, id: DomRegistryId) -> Result<&Record, DomError> {
        self.get_with(id, &mut || Ok(()))
    }
    pub(crate) fn get_with<E: From<DomError>>(
        &self,
        id: DomRegistryId,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<&Record, E> {
        if id.owner != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        if id.generation != 1 {
            return Err(DomError::StaleNode.into());
        }
        self.records
            .get_with(id.slot, step)?
            .ok_or_else(|| DomError::StaleNode.into())
    }
    pub(crate) fn form(
        old: Option<&Self>,
        owner: DomOwnerId,
        source: &DomHostedSource,
        work: &Work,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(Arc<Self>, DomRegistryId), DomHostedError> {
        Self::edit(
            old,
            owner,
            RegistryEdit::CreateGlobal,
            source,
            work,
            formation,
            cancel,
        )
    }

    pub(in crate::store) fn edit(
        old: Option<&Self>,
        owner: DomOwnerId,
        edit: RegistryEdit<'_>,
        source: &DomHostedSource,
        work: &Work,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(Arc<Self>, DomRegistryId), DomHostedError> {
        work.step(cancel)?;
        if old.is_some_and(|old| old.owner != owner) {
            return Err(DomError::ForeignOwner.into());
        }
        let identity = custody::MaterializationId::fresh()?;
        let record_identity = bank::Record::<Record>::reserve()?;
        let (id, scoped_origin, definition) = match edit {
            RegistryEdit::CreateGlobal | RegistryEdit::CreateScoped(_) => {
                let slot = old.map_or(0, |bank| bank.records.len());
                let required = slot
                    .checked_add(1)
                    .ok_or(DomHostedError::RepresentationExhausted)?;
                crate::hosted::admit(
                    required,
                    source.limits().records().registries(),
                    DomHostedResource::Registries,
                )?;
                (
                    DomRegistryId {
                        owner,
                        slot,
                        generation: 1,
                    },
                    match edit {
                        RegistryEdit::CreateScoped(global) => Some(global),
                        _ => None,
                    },
                    None,
                )
            }
            RegistryEdit::Define(id, definition) => {
                let record = old.ok_or(DomError::StaleNode)?.get_with(id, &mut || {
                    work.step(cancel)?;
                    if let Some(formation) = formation {
                        formation.step(cancel)?;
                    }
                    Ok::<_, DomHostedError>(())
                })?;
                (id, record.scoped_origin, Some((record, definition)))
            }
        };
        if let Some(formation) = formation {
            formation.retain_bytes(Record::HEADER_BYTES, cancel)?;
        }
        let header = source.retain(Record::HEADER_BYTES, cancel)?;
        let quota = match definition {
            Some((record, _)) => record.quota.clone(),
            None => Arc::new(source.retain_registry(cancel)?),
        };
        let hosted_remaining = source.remaining_bytes()?;
        let scratch_limit = source.limits().scratch_bytes();
        let hosted_allowance = hosted_remaining.min(scratch_limit);
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
        let step = |amount| {
            work.spend(amount, cancel)?;
            if let Some(formation) = formation {
                for _ in 0..amount {
                    formation.step(cancel)?;
                }
            }
            Ok::<_, DomHostedError>(())
        };
        let revision = HtmlHostFactsRevision::fresh();
        let formed = match definition {
            Some((record, definition)) => record.capture.append_registry_definition_with(
                &record.local,
                definition,
                revision,
                HtmlRegistryFormationLimits::new(
                    allowance,
                    formation
                        .map(source::formation::Formation::remaining_units)
                        .transpose()?
                        .unwrap_or(usize::MAX),
                ),
                step,
            ),
            None if scoped_origin.is_some() => {
                HtmlCapture::complete_empty_scoped_registry_with(revision, allowance, step)
            }
            None => HtmlCapture::complete_empty_global_registry_with(revision, allowance, step),
        }
        .map_err(|error| match error {
            HtmlRegistryFormationError::Interrupted(error) => error,
            HtmlRegistryFormationError::FormationBytes { .. } => {
                if let Some(formation) = formation
                    .filter(|_| parent_allowance.is_some_and(|bytes| bytes <= hosted_allowance))
                {
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: formation.retained_byte_limit(),
                    }
                    .into()
                } else if scratch_limit < hosted_remaining {
                    DomHostedError::Limit {
                        resource: DomHostedResource::ScratchBytes,
                        limit: scratch_limit,
                    }
                } else {
                    DomHostedError::Limit {
                        resource: DomHostedResource::RetainedBytes,
                        limit: source.limits().retained_bytes(),
                    }
                }
            }
            HtmlRegistryFormationError::RepresentationExhausted => {
                DomHostedError::RepresentationExhausted
            }
            HtmlRegistryFormationError::FormationUnits { limit, .. } => DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                limit: formation.map_or(limit, source::formation::Formation::retained_unit_limit),
            }
            .into(),
            HtmlRegistryFormationError::AllocationFailed => DomHostedError::AllocationFailed,
            HtmlRegistryFormationError::Accounting(fault) => hosted_accounting_error(fault),
            HtmlRegistryFormationError::Capture(error) => match definition {
                Some(_) if error.accounting_fault().is_none() => {
                    DomError::SelectCapture(error).into()
                }
                _ => hosted_capture_error(&error),
            },
        })?;
        let retained = formed.retained_storage_usage();
        let payload_bytes = retained.retained_bytes();
        let payload_units = retained.utf16_extent();
        if let Some(formation) = formation {
            formation.retain_payload(
                source::storage::Payload {
                    bytes: 0,
                    units: payload_units,
                },
                cancel,
            )?;
            formation.release_payload(source::storage::Payload {
                bytes: allowance
                    .checked_sub(payload_bytes)
                    .ok_or(DomError::RepresentationExhausted)?,
                units: 0,
            });
        }
        let payload = source.retain(payload_bytes, cancel)?;
        let (capture, local) = formed.into_parts();
        let record = record_identity.materialize(Record {
            origin: identity,
            capture,
            local,
            scoped_origin,
            payload_bytes,
            payload_units,
            _payload: payload,
            _header: header,
            quota,
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
        let mut candidate = Self {
            identity,
            owner,
            records: old.map_or_else(bank::Bank::default, |old| old.records.clone()),
            scoped_documents: old
                .map_or_else(bank::Bank::default, |bank| bank.scoped_documents.clone()),
            has_scoped: scoped_origin.is_some() || old.is_some_and(Self::has_scoped_registry),
            _retained: Some(retained),
        };
        producer.set(
            &mut candidate.records,
            id.slot,
            record,
            custody::Usage::new(
                Record::HEADER_BYTES
                    .checked_add(payload_bytes)
                    .ok_or(DomError::RepresentationExhausted)?,
                payload_units,
            ),
        )?;
        let candidate = Arc::new(candidate);
        #[cfg(test)]
        transaction::auxiliary::allocated(size_of::<Self>());
        Ok((candidate, id))
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
                bytes: Record::HEADER_BYTES + capture.retained_bytes(),
                units: capture.utf16_extent(),
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
        custody::Usage::new(Self::HEADER_BYTES + self.payload_bytes, self.payload_units)
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

    /// Reads the scoped registry's ordered Document set, preserving historical
    /// identities and admitting the canonical lookup before returning the view.
    pub fn scoped_registry_documents_with<E: From<DomError>>(
        &self,
        id: DomRegistryId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<&[DocumentId], E> {
        let bank = self.state.registries.as_ref().ok_or(DomError::StaleNode)?;
        bank.scoped_documents(id, &mut step)
    }
}

/// A checked canonical registry publication. It borrows the original DOM
/// exclusively until the definition is published or this preparation is dropped.
pub struct DomRegistryDefinitionPreparation<'a> {
    dom: &'a mut Dom,
    _source: &'a DomHostedSource,
    publication: PreparedPublication,
    definition: DomRegistryDefinitionId,
}

impl DomRegistryDefinitionPreparation<'_> {
    /// Publishes the staged definition after the consumer has prepared its
    /// corresponding constructor and callback custody.
    pub fn commit(self) -> (DomCommit, DomRegistryDefinitionId) {
        (self.dom.publish_prepared(self.publication), self.definition)
    }
}

impl Dom {
    /// Creates an empty canonical scoped registry owned by a live Global.
    /// Refusal leaves the original registry bank and snapshot unchanged.
    pub fn create_scoped_registry(
        &mut self,
        source: &DomHostedSource,
        global: DomHostedGlobalId,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(DomCommit, DomRegistryId), DomHostedError> {
        self.require_source_entry(cancel)?;
        self.validate_hosted_source(source)?;
        check_cancel(cancel)?;
        let work = source.work(cancel)?;
        work.step(cancel)?;
        self.dispatch_global_document_with::<DomHostedError>(global, cancel, &mut || {
            work.step(cancel)
        })?;
        self.source_child_limits(limits, cancel)?;
        let mut draft = self.registry_publication_draft(limits, cancel)?;
        let formation = source::formation::Formation::new(&draft, cancel)?
            .ok_or(DomError::HostedSourceUnavailable)?;
        let (bank, id) = Bank::edit(
            draft.registries.as_deref(),
            self.owner,
            RegistryEdit::CreateScoped(global),
            source,
            &work,
            Some(&formation),
            cancel,
        )?;
        draft.set_registries(Some(bank), Some(formation), cancel)?;
        let publication =
            self.prepare_registry_publication(draft, DomChange::RegistryCreated(id), cancel)?;
        Ok((self.publish_prepared(publication), id))
    }

    /// Stage one definition in the current Global's canonical registry or in
    /// a scoped registry issued to that Global.
    /// Every refusal and dropping the preparation leave the registry unchanged.
    pub fn prepare_registry_definition<'a>(
        &'a mut self,
        source: &'a DomHostedSource,
        global: crate::hosted::DomHostedGlobalId,
        registry: DomRegistryId,
        definition: &HtmlCustomElementDefinition,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomRegistryDefinitionPreparation<'a>, DomHostedError> {
        self.require_source_entry(cancel)?;
        self.validate_hosted_source(source)?;
        check_cancel(cancel)?;
        let work = source.work(cancel)?;
        work.step(cancel)?;
        if registry.owner != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        let record = self
            .state
            .registries
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?
            .get_with(registry, &mut || work.step(cancel))?;
        if let Some(origin) = record.scoped_origin {
            if origin != global {
                return Err(DomHostedError::StaleIdentity);
            }
            if !crate::names::same_code_units(
                definition.name().code_units(),
                definition.local_name().code_units(),
                &mut || work.step(cancel),
            )? {
                return Err(DomError::NotSupported.into());
            }
        } else {
            let document =
                self.dispatch_global_document_with::<DomHostedError>(global, cancel, &mut || {
                    work.step(cancel)
                })?;
            work.step(cancel)?;
            let crate::DomNodeKind::Document(metadata) =
                self.state.node(self.owner, document.node())?.kind()
            else {
                return Err(DomError::WrongNodeKind.into());
            };
            if metadata.custom_element_registry() != crate::DomRegistryBinding::Registry(registry) {
                return Err(DomHostedError::StaleIdentity);
            }
        }
        let mut draft = self.registry_publication_draft(limits, cancel)?;
        let formation = source::formation::Formation::new(&draft, cancel)?;
        let formation = formation.ok_or(DomError::HostedSourceUnavailable)?;
        let mut input = source::storage::Payload::default();
        input.add_bytes(size_of::<HtmlCustomElementDefinition>())?;
        formation.step(cancel)?;
        input.string(definition.name())?;
        formation.step(cancel)?;
        input.string(definition.local_name())?;
        let operands = formation.fork();
        operands.retain_payload(input, cancel)?;
        let (bank, registry) = Bank::edit(
            draft.registries.as_deref(),
            self.owner,
            RegistryEdit::Define(registry, definition),
            source,
            &work,
            Some(&formation),
            cancel,
        )?;
        let record = bank.get_with(registry, &mut || draft.source_step(cancel))?;
        let ordinal = record
            .capture
            .usage()
            .definitions()
            .checked_sub(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let definition = DomRegistryDefinitionId { registry, ordinal };
        drop(operands);
        draft.set_registries(Some(bank), Some(formation), cancel)?;
        let publication = self.prepare_registry_publication(
            draft,
            DomChange::RegistryDefinitionAdded(definition),
            cancel,
        )?;
        Ok(DomRegistryDefinitionPreparation {
            dom: self,
            _source: source,
            publication,
            definition,
        })
    }

    fn registry_publication_draft(
        &self,
        limits: crate::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<Draft, DomHostedError> {
        if limits.nested_operations() == 0 {
            return Err(DomError::SourceNestedOperationLimit { limit: 0 }.into());
        }
        if limits.phase_publications() == 0 {
            return Err(DomError::SourcePublicationLimit.into());
        }
        if limits.output_records() == 0 {
            return Err(DomError::SourceOutputLimit.into());
        }
        if limits.published_native_identities() == 0 {
            return Err(DomError::SourceIdentityLimit.into());
        }
        let owner = DomBatch::new().owner;
        Ok(if self.source_operation.is_some() {
            self.source_observed_batch(owner, &[], 0, cancel)?
        } else {
            Draft::new_source_root(self, owner, &[], 0, limits, cancel)?.0
        })
    }

    fn prepare_registry_publication(
        &mut self,
        mut draft: Draft,
        change: DomChange,
        cancel: &AtomicBool,
    ) -> Result<PreparedPublication, DomHostedError> {
        let mut prepared = draft.finish(self, cancel)?;
        prepared.state.incomplete_source = self.state.incomplete_source;
        source::formation::reserve(
            &mut prepared.changes,
            1,
            prepared.formation.as_ref(),
            cancel,
        )?;
        prepared.changes.push(change);
        Ok(self.prepare_publication(prepared, cancel)?)
    }
}

impl Draft {
    pub(in crate::store) fn associate_scoped_registry_document(
        &mut self,
        id: DomRegistryId,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.pending_scoped_documents.is_none()
            && let Some(document_id) = self.published_id(document).map(DocumentId)
        {
            return self.record_scoped_registry_document(id, document_id, cancel);
        }
        let mut pending = self
            .pending_scoped_documents
            .take()
            .unwrap_or_else(|| self.effect_vec());
        let reserved = self.select_reserve(&mut pending, 1, cancel);
        if reserved.is_ok() {
            pending.push((id, document));
        }
        self.pending_scoped_documents = Some(pending);
        reserved
    }

    pub(in crate::store) fn resolve_scoped_registry_documents(
        &mut self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(pending) = self.pending_scoped_documents.take() else {
            return Ok(());
        };
        let result = (|| {
            let allocation_ids = self.allocation_ids(dom, None, cancel)?;
            for &(registry, key) in pending.iter() {
                self.source_step(cancel)?;
                let resolved = match key {
                    Key::Existing(id) => self.node(key).is_ok().then_some(id),
                    Key::Created(index) => allocation_ids.get(index),
                };
                if let Some(document) = resolved {
                    self.record_scoped_registry_document(registry, DocumentId(document), cancel)?;
                }
            }
            Ok(())
        })();
        self.pending_scoped_documents = Some(pending);
        result
    }

    fn record_scoped_registry_document(
        &mut self,
        id: DomRegistryId,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self.registries.as_deref().ok_or(DomError::StaleNode)?;
        let prior = old.scoped_documents(id, &mut || self.source_step(cancel))?;
        for &known in prior {
            self.source_step(cancel)?;
            if known == document {
                return Ok(());
            }
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut ordinary = self
            .custody_ledger()
            .retain(custody::Usage::ZERO, |usage| {
                self.admit_custody_usage(usage, cancel)
            })?;
        let mut admit = |bytes| match &formation {
            Some(formation) => formation.retain_bytes(bytes, cancel),
            None => ordinary.grow(custody::Usage::new(bytes, 0), |usage| {
                self.admit_custody_usage(usage, cancel)
            }),
        };
        let identity = custody::MaterializationId::fresh()?;
        admit(size_of::<Bank>() + size_of::<ScopedDocuments>())?;
        let capacity = prior
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let mut documents = Vec::new();
        let bytes = capacity
            .checked_mul(size_of::<DocumentId>())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(bytes)?;
        documents
            .try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        let extra = documents
            .capacity()
            .checked_sub(capacity)
            .and_then(|count| count.checked_mul(size_of::<DocumentId>()))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(extra)?;
        for &document in prior {
            self.effect_step(cancel)?;
            documents.push(document);
        }
        documents.push(document);
        let row = ScopedDocuments {
            origin: identity,
            documents,
        };
        let usage = row.custody_usage()?;
        let record = bank::Record::reserve()?.materialize(row);
        let mut scoped_documents = old.scoped_documents.clone();
        scoped_documents.set_source_admitted(
            id.slot,
            record,
            bank::Totals {
                bytes: usage.bytes(),
                units: 0,
                occurrences: 0,
            },
            identity,
            |charge| admit(charge.bytes),
        )?;
        let next = Arc::new(Bank {
            identity,
            owner: old.owner,
            records: old.records.clone(),
            scoped_documents,
            has_scoped: old.has_scoped,
            _retained: None,
        });
        if let Some(formation) = formation {
            self.set_registries(Some(next), Some(formation), cancel)
        } else {
            self.set_registries_with_debit(Some(next), Some(ordinary), cancel)
        }
    }

    pub(in crate::store) fn apply_registry_subtree(
        &mut self,
        root: Key,
        id: DomRegistryId,
        initialize: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use crate::DomRegistryBinding::{Null, Registry};
        use transaction::KindRead;

        let scoped = self.creation_registry(id, cancel)?.scoped_origin.is_some();
        let binding = Registry(id);
        self.source_step(cancel)?;
        let node = self.node(root)?;
        if initialize {
            if !scoped
                && (matches!(node.kind(), KindRead::Document(_))
                    || self
                        .document_metadata(node.document())?
                        .custom_element_registry()
                        != binding)
            {
                return Err(DomError::NotSupported);
            }
            match node.kind() {
                KindRead::Document(metadata) if metadata.custom_element_registry() == Null => {
                    let mut metadata = self.document_metadata_mut(root, cancel)?;
                    metadata.establish_registry(id);
                    metadata.commit(cancel)?;
                }
                KindRead::ShadowRoot(state) if state.custom_element_registry() == Null => {
                    let mut state = self.shadow_state_mut(root, cancel)?;
                    state.establish_registry(id);
                    state.commit(cancel)?;
                }
                _ => {}
            }
        }
        let mut pending = self.effect_vec();
        self.select_reserve(&mut pending, 1, cancel)?;
        pending.push(root);
        while let Some(key) = pending.pop() {
            self.source_step(cancel)?;
            let node = self.node(key)?;
            let current = node
                .element()
                .map(|element| element.language().custom_element_registry());
            let document = node.document();
            if current == Some(Null) && initialize {
                self.establish_element_registry(key, binding, cancel)?;
                if scoped {
                    self.associate_scoped_registry_document(id, document, cancel)?;
                }
            }
            if current == Some(binding) || (current == Some(Null) && initialize) {
                self.record_custom_element_registry_upgrade(key, binding, cancel)?;
            }
            let node = self.node(key)?;
            let children = node.children();
            self.select_reserve(&mut pending, children.len(), cancel)?;
            for child in children.iter().rev() {
                self.source_step(cancel)?;
                pending.push(child);
            }
            if !initialize && let Some(shadow) = node.shadow_root() {
                self.select_reserve(&mut pending, 1, cancel)?;
                pending.push(shadow);
            }
        }
        Ok(())
    }

    pub(in crate::store) fn effective_global_registry(
        &self,
        binding: crate::DomRegistryBinding,
        cancel: &AtomicBool,
    ) -> Result<crate::DomRegistryBinding, DomError> {
        if let crate::DomRegistryBinding::Registry(id) = binding
            && self.creation_registry(id, cancel)?.scoped_origin.is_some()
        {
            return Ok(crate::DomRegistryBinding::Null);
        }
        Ok(binding)
    }

    pub(super) fn lookup_element_definition(
        &self,
        key: Key,
        binding: crate::DomRegistryBinding,
        cancel: &AtomicBool,
    ) -> Result<Option<DomRegistryDefinitionId>, DomError> {
        let crate::DomRegistryBinding::Registry(registry) = binding else {
            return Ok(None);
        };
        let record = self.creation_registry(registry, cancel)?;
        let element = self.node(key)?.element().ok_or(DomError::WrongNodeKind)?;
        if element.common_state().html().is_none() {
            return Ok(None);
        }
        self.lookup_definition(registry, record, element.name(), element.saved_is(), cancel)
    }

    pub(super) fn lookup_creation_definition(
        &self,
        binding: crate::DomRegistryBinding,
        name: &crate::DomQualifiedName,
        saved_is: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<Option<DomRegistryDefinitionId>, DomError> {
        let crate::DomRegistryBinding::Registry(registry) = binding else {
            return Ok(None);
        };
        let record = self.creation_registry(registry, cancel)?;
        if !crate::names::in_html_namespace_with(name, || self.source_step(cancel))? {
            return Ok(None);
        }
        self.lookup_definition(registry, record, name, saved_is, cancel)
    }

    pub(super) fn creation_registry(
        &self,
        registry: DomRegistryId,
        cancel: &AtomicBool,
    ) -> Result<&Record, DomError> {
        self.registries
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?
            .get_with(registry, &mut || self.source_step(cancel))
    }

    fn lookup_definition(
        &self,
        registry: DomRegistryId,
        record: &Record,
        name: &crate::DomQualifiedName,
        saved_is: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<Option<DomRegistryDefinitionId>, DomError> {
        let ordinal = record.lookup_with(name.local_name(), saved_is, &mut |amount| {
            for _ in 0..amount {
                self.source_step(cancel)?;
            }
            Ok(())
        })?;
        Ok(ordinal.map(|ordinal| DomRegistryDefinitionId { registry, ordinal }))
    }

    pub(super) fn select_element_registry(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.select_element_registry_with(key, None, cancel)
    }

    pub(super) fn select_element_registry_with(
        &mut self,
        key: Key,
        selected: Option<crate::DomRegistryBinding>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let node = self.node(key)?;
        if node.element().is_none() {
            return Ok(());
        }
        let metadata = self
            .document_metadata(node.document())
            .map_err(|error| match error {
                DomError::WrongNodeKind => DomError::WrongDocument,
                error => error,
            })?;
        let binding = selected.unwrap_or_else(|| metadata.custom_element_registry());
        if self
            .lookup_element_definition(key, binding, cancel)?
            .is_some()
        {
            return Err(DomError::NotSupported);
        }
        self.establish_element_registry(key, binding, cancel)?;
        Ok(())
    }
}

/// The hosted outcome of an accounting fault of the registry formation: a ledger that cannot
/// allocate is the hosted allocation failure, and every other fault breaks the HTML operation's
/// accounting contract.
fn hosted_accounting_error(fault: HtmlAccountingFault) -> DomHostedError {
    match fault {
        HtmlAccountingFault::Allocation => DomHostedError::AllocationFailed,
        _ => DomError::HtmlChargeProtocol.into(),
    }
}

/// The hosted outcome of a capture error of the registry formation: a ledger fault of the capture
/// is the accounting outcome, and every other capture error is an unsupported profile.
fn hosted_capture_error(error: &HtmlCaptureError) -> DomHostedError {
    match error.accounting_fault() {
        Some(fault) => hosted_accounting_error(fault),
        None => DomHostedError::UnsupportedProfile,
    }
}

#[cfg(test)]
mod accounting_fault_tests {
    use super::*;

    #[test]
    fn a_capture_ledger_fault_of_the_formation_follows_the_accounting_outcome() {
        assert_eq!(
            hosted_capture_error(&HtmlCaptureError::Accounting(
                HtmlAccountingFault::Allocation
            )),
            DomHostedError::AllocationFailed
        );
        assert_eq!(
            hosted_capture_error(&HtmlCaptureError::Accounting(
                HtmlAccountingFault::ReleaseUnderflow
            )),
            DomHostedError::Dom(DomError::HtmlChargeProtocol)
        );
        assert_eq!(
            hosted_capture_error(&HtmlCaptureError::ForeignBuilder),
            DomHostedError::UnsupportedProfile
        );
    }

    #[test]
    fn a_registry_formation_ledger_fault_is_an_allocation_failure_or_a_charge_protocol_fault() {
        assert_eq!(
            hosted_accounting_error(HtmlAccountingFault::Allocation),
            DomHostedError::AllocationFailed
        );
        for fault in [
            HtmlAccountingFault::ReleaseUnderflow,
            HtmlAccountingFault::ForeignScope,
            HtmlAccountingFault::ScopeLimit { limit: 1 },
            HtmlAccountingFault::UnexpectedChange,
        ] {
            assert_eq!(
                hosted_accounting_error(fault),
                DomHostedError::Dom(DomError::HtmlChargeProtocol),
                "{fault:?}"
            );
        }
    }
}
