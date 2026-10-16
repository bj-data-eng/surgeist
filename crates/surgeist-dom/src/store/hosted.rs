//! Canonical native hosted associations and the original creation continuation.

use super::*;
use crate::hosted::*;

// Backing fields precede their debit, including on every failed formation.
struct ConstructorFormation {
    units: Vec<u16>,
    serialization: Option<DocumentString>,
    record: Option<surgeist_html_semantics::HtmlBaseUrlRecord>,
    retained: Retained,
}
impl crate::document::HostedDocumentInputs {
    pub(crate) fn form(
        source: &DomHostedSource,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHostedError> {
        use crate::document::HostedDocumentInputData;
        use surgeist_html_semantics::{
            HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsRevision,
            HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme,
        };
        // The original constructor's intrinsic URL, not an external URL parse.
        const URL: &[u16] = &[97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107];
        work.step(cancel)?;
        let requested_string = size_of::<Vec<u16>>() + size_of_val(URL);
        let header = size_of::<HostedDocumentInputData>();
        crate::hosted::admit(
            size_of::<ConstructorFormation>() + header + requested_string,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let retained = source.retain(
            header
                .checked_add(requested_string)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            cancel,
        )?;
        let mut candidate = ConstructorFormation {
            units: Vec::new(),
            serialization: None,
            record: None,
            retained,
        };
        candidate
            .units
            .try_reserve_exact(URL.len())
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let actual_string = candidate
            .units
            .capacity()
            .checked_mul(size_of::<u16>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<u16>>()))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            size_of::<ConstructorFormation>()
                .checked_add(header)
                .and_then(|n| n.checked_add(actual_string))
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        candidate.retained.append(
            source.retain(
                actual_string
                    .checked_sub(requested_string)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                cancel,
            )?,
        )?;
        for unit in URL {
            work.step(cancel)?;
            candidate.units.push(*unit);
        }
        candidate.serialization = Some(DocumentString::from_utf16(std::mem::take(
            &mut candidate.units,
        )));
        let serialization = candidate
            .serialization
            .as_ref()
            .expect("formed serialization");
        let usage = HtmlBaseUrlRecord::observed_storage_usage(serialization)
            .map_err(|_| DomHostedError::RepresentationExhausted)?;
        // Transfer the already admitted serialization occurrence to its actual
        // producer record: only the additional private record extent is new.
        let additional = usage
            .retained_bytes()
            .checked_sub(actual_string)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            size_of::<ConstructorFormation>()
                .checked_add(header)
                .and_then(|n| n.checked_add(usage.retained_bytes()))
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        candidate
            .retained
            .append(source.retain(additional, cancel)?)?;
        work.step(cancel)?;
        // This producer's initial immutable constructor configuration has no
        // URL/CSP changes. It is distinct from any external parse producer.
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 0, 0, 0);
        candidate.record = Some(
            HtmlBaseUrlRecord::observed(
                revision.clone(),
                candidate
                    .serialization
                    .take()
                    .expect("admitted same serialization"),
                HtmlBaseUrlScheme::Other,
                true,
            )
            .expect("same prepriced serialization and intrinsic about:blank predicates"),
        );
        let facts = HtmlBaseDocumentFacts::new(
            revision,
            candidate.record.take(),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(HtmlBaseEncoding::fresh()),
        )
        .expect("constructor record has its exact original revision and is not srcdoc");
        // All fallible admission and cancellation precede the immutable owner.
        Ok(Self(Arc::new(HostedDocumentInputData {
            facts,
            opener: surgeist_html::HtmlOpenerPolicy::initial(),
            _retained: candidate.retained,
        })))
    }
}

enum CreationPhase {
    Policy(DomHostedPolicyRequest),
    Execution(DomHostedExecutionRequest),
    MakeActive(DomHostedMakeActiveRequest),
    FinishLoading(DomHostedFinishLoadingRequest),
    Entry(DomHostedEntryRequest),
    Complete(Request, DomHostedCreation),
    DestroyEnqueue(DomHostedDestructionRequest),
    DestroyCleanup(DomHostedDestructionRequest),
    Destroyed(Request, DomHostedCreation),
}
pub(super) struct Operation {
    origin: crate::DomOpaqueOrigin,
    policy: Option<DomInitialPermissionsPolicy>,
    phase: CreationPhase,
    _retained: Retained,
}
impl Operation {
    fn request(&self) -> &Request {
        match &self.phase {
            CreationPhase::Policy(request) => &request.request,
            CreationPhase::Execution(request) => &request.request,
            CreationPhase::MakeActive(request) => &request.request,
            CreationPhase::FinishLoading(request) => &request.request,
            CreationPhase::Entry(request) => &request.request,
            CreationPhase::Complete(request, _) | CreationPhase::Destroyed(request, _) => request,
            CreationPhase::DestroyEnqueue(request) | CreationPhase::DestroyCleanup(request) => {
                &request.request
            }
        }
    }
    fn next_request(&self) -> Result<Request, DomHostedError> {
        let old = self.request();
        Ok(Request {
            key: RequestKey {
                operation: old.key.operation,
                phase: old
                    .key
                    .phase
                    .checked_add(1)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
            },
            host: old.host,
            context: old.context,
            work: old.work.clone(),
        })
    }
    fn step(&self) -> Result<DomHostedCreationStep<'_>, DomHostedError> {
        Ok(match &self.phase {
            CreationPhase::Policy(request) => DomHostedCreationStep::PermissionsPolicy(request),
            CreationPhase::Execution(request) => DomHostedCreationStep::NativeExecution(request),
            CreationPhase::MakeActive(request) => DomHostedCreationStep::MakeActive(request),
            CreationPhase::FinishLoading(request) => DomHostedCreationStep::FinishLoading(request),
            CreationPhase::Entry(request) => DomHostedCreationStep::InitialEntry(request),
            CreationPhase::Complete(_, result) => DomHostedCreationStep::Complete(result.document),
            _ => return Err(DomHostedError::WrongPhase),
        })
    }
}

impl Dom {
    fn validate_hosted_source(&self, source: &DomHostedSource) -> Result<(), DomHostedError> {
        if source.id().owner != self.owner || self.hosted_source.as_ref() != Some(&source.observe())
        {
            return Err(DomHostedError::ForeignSource);
        }
        source.revision()?;
        Ok(())
    }
    /// Begin the actual no-opener constructor with its original BC preparation.
    /// Refusal leaves the entire preparation and actual Runtime prefix owned.
    pub fn begin_hosted_top_level(
        &mut self,
        source: &mut DomHostedSource,
        prepared: &mut Option<DomHostedPreparedTopLevel>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreationLease, DomHostedError> {
        self.require_source_idle()?;
        self.validate_hosted_source(source)?;
        check_cancel(cancel)?;
        if self.hosted_operation.is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        let current = prepared
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        let prefix = &current.preparation;
        prefix.work.step(cancel)?;
        source.observe().validate(
            prefix.host.source,
            Role::Host,
            prefix.host.slot,
            prefix.host.generation,
        )?;
        source.observe().validate(
            prefix.context.source,
            Role::BrowsingContext,
            prefix.context.slot,
            prefix.context.generation,
        )?;
        // Five real canonical phase publications are mandatory after the policy
        // result. Other edits must leave these reserved increments available.
        self.revision
            .0
            .checked_add(5)
            .ok_or(DomError::RevisionExhausted)?;
        let retained = source.retain(size_of::<Operation>(), cancel)?;
        let origin = DomDocumentMetadata::fresh_opaque_origin();
        let operation = source.begin_operation(&prefix.work, 10, cancel)?;
        let prepared = prepared.take().expect("validated original preparation");
        let request = Request {
            key: RequestKey {
                operation,
                phase: 0,
            },
            host: prepared.preparation.host,
            context: prepared.preparation.context,
            work: prepared.preparation.work,
        };
        self.hosted_operation = Some(Box::new(Operation {
            origin: origin.clone(),
            policy: None,
            phase: CreationPhase::Policy(DomHostedPolicyRequest { request, origin }),
            _retained: retained,
        }));
        self.hosted_revisions = 5;
        Ok(DomHostedCreationLease {
            operation,
            completed: None,
        })
    }
    fn hosted_operation(
        &self,
        operation: DomHostedOperationId,
    ) -> Result<&Operation, DomHostedError> {
        if operation.source.owner != self.owner {
            return Err(DomHostedError::ForeignSource);
        }
        self.hosted_operation
            .as_deref()
            .filter(|pending| pending.request().key.operation == operation)
            .ok_or(DomHostedError::StaleIdentity)
    }
    fn publish_hosted(
        &mut self,
        prepared: Prepared,
        cancel: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        self.hosted_revisions = self
            .hosted_revisions
            .checked_sub(1)
            .ok_or(DomError::HostedSourceUnavailable)?;
        match self.publish(prepared, cancel) {
            Ok(commit) => Ok(commit),
            Err(error) => {
                self.hosted_revisions += 1;
                Err(error)
            }
        }
    }
    /// Register the unique issuer before any externally bound session exists.
    /// No external callbacks run and no hosted Document is created by this step.
    pub fn register_hosted_source(
        &mut self,
        limits: DomHostedLimits,
        cancel: &AtomicBool,
    ) -> Result<DomHostedSource, DomHostedError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if self.hosted_source.is_some() {
            return Err(DomHostedError::AlreadyRegistered);
        }
        if self.details_transferred {
            return Err(DomHostedError::PriorDetailsTransfer);
        }
        let next = self
            .next_hosted_source
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let source = DomHostedSource::new(
            DomHostedSourceId {
                owner: self.owner,
                serial: self.next_hosted_source,
            },
            limits,
            cancel,
        )?;
        self.hosted_source = Some(source.observe());
        self.next_hosted_source = next;
        Ok(source)
    }
}

impl DomHostedCreationLease {
    pub fn step<'a>(&self, dom: &'a Dom) -> Result<DomHostedCreationStep<'a>, DomHostedError> {
        if let Some(result) = self.completed {
            if dom.owner() != self.operation.source.owner {
                return Err(DomHostedError::ForeignSource);
            }
            return Ok(DomHostedCreationStep::Complete(result.document));
        }
        dom.hosted_operation(self.operation)?.step()
    }
    pub fn accept_permissions_policy(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        publication: &mut Option<DomHostedPolicyPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.validate_hosted_source(source)?;
        let operation = dom.hosted_operation(self.operation)?;
        if !matches!(operation.phase, CreationPhase::Policy(_)) {
            return Err(DomHostedError::WrongPhase);
        }
        let current = publication
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        if current.key != operation.request().key {
            return Err(DomHostedError::WrongPhase);
        }
        let request = operation.next_request()?;
        // Settings consumes these same once-established records during execution.
        // Formation failure leaves the original Policy publication and phase intact.
        let inputs =
            crate::document::HostedDocumentInputs::form(source, &operation.request().work, cancel)?;
        let next = CreationPhase::Execution(DomHostedExecutionRequest {
            request,
            origin: operation.origin.clone(),
            inputs,
        });
        let permit = source.permit(operation.request(), current.revision, false, cancel)?;
        let pending = dom
            .hosted_operation
            .as_mut()
            .expect("validated original operation");
        pending.policy = Some(DomInitialPermissionsPolicy {
            inherited: current.inherited,
        });
        pending.phase = next;
        permit.commit();
        publication.take();
        Ok(())
    }
    pub fn accept_publication(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        publication: &mut Option<DomHostedCreationPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.validate_hosted_source(source)?;
        let operation = dom.hosted_operation(self.operation)?;
        let current = publication
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        if current.key != operation.request().key {
            return Err(DomHostedError::WrongPhase);
        }
        operation.request().work.step(cancel)?;
        let next_request = operation.next_request()?;
        let (mut prepared, next) = match (&operation.phase, &current.value) {
            (
                CreationPhase::Execution(execution),
                PublicationValue::NativeExecution {
                    global,
                    navigation_start,
                },
            ) => {
                source.observe().validate(
                    global.source,
                    Role::Global,
                    global.slot,
                    global.generation,
                )?;
                source.validate_created(
                    Role::Global,
                    global.source,
                    global.slot,
                    global.generation,
                    current.key,
                )?;
                let (prepared, document) = dom.prepare_hosted_document(
                    operation,
                    source,
                    *global,
                    *navigation_start,
                    &execution.inputs,
                    cancel,
                )?;
                (
                    prepared,
                    CreationPhase::MakeActive(DomHostedMakeActiveRequest {
                        request: next_request,
                        document,
                        global: *global,
                    }),
                )
            }
            (CreationPhase::MakeActive(request), PublicationValue::MakeActive) => {
                let prepared = dom.prepare_hosted_phase(
                    operation,
                    source,
                    request.document,
                    DocumentPhase::Active,
                    cancel,
                )?;
                let association = dom
                    .snapshot()
                    .document(request.document)?
                    .metadata()
                    .document_environment()
                    .hosted
                    .clone()
                    .ok_or(DomHostedError::UnsupportedProfile)?;
                (
                    prepared,
                    CreationPhase::FinishLoading(DomHostedFinishLoadingRequest {
                        request: next_request,
                        document: request.document,
                        global: request.global,
                        navigation_start: association.navigation_start,
                    }),
                )
            }
            (CreationPhase::FinishLoading(request), PublicationValue::FinishLoading(moment)) => {
                if moment.source != source.id()
                    || moment.microseconds < request.navigation_start.microseconds
                {
                    return Err(DomHostedError::ClockRegression);
                }
                let prepared = dom.prepare_hosted_phase(
                    operation,
                    source,
                    request.document,
                    DocumentPhase::Loaded(*moment),
                    cancel,
                )?;
                (
                    prepared,
                    CreationPhase::Entry(DomHostedEntryRequest {
                        request: next_request,
                        document: request.document,
                        global: request.global,
                    }),
                )
            }
            (
                CreationPhase::Entry(request),
                PublicationValue::InitialEntry { entry, navigable },
            ) => {
                source.observe().validate(
                    entry.source,
                    Role::Entry,
                    entry.slot,
                    entry.generation,
                )?;
                source.validate_created(
                    Role::Entry,
                    entry.source,
                    entry.slot,
                    entry.generation,
                    current.key,
                )?;
                source.observe().validate(
                    navigable.source,
                    Role::Navigable,
                    navigable.slot,
                    navigable.generation,
                )?;
                source.validate_created(
                    Role::Navigable,
                    navigable.source,
                    navigable.slot,
                    navigable.generation,
                    current.key,
                )?;
                let snapshot = dom.snapshot();
                let environment = snapshot
                    .document(request.document)?
                    .metadata()
                    .document_environment();
                let association = environment
                    .hosted
                    .as_ref()
                    .ok_or(DomHostedError::UnsupportedProfile)?;
                let DocumentPhase::Loaded(loaded) = association.phase else {
                    return Err(DomHostedError::WrongPhase);
                };
                let prepared = dom.prepare_hosted_phase(
                    operation,
                    source,
                    request.document,
                    DocumentPhase::InitialEntry {
                        entry: *entry,
                        navigable: *navigable,
                        loaded,
                    },
                    cancel,
                )?;
                (
                    prepared,
                    CreationPhase::Complete(
                        next_request,
                        DomHostedCreation {
                            document: request.document,
                            host: operation.request().host,
                            context: operation.request().context,
                            global: request.global,
                            entry: *entry,
                            navigable: *navigable,
                        },
                    ),
                )
            }
            _ => return Err(DomHostedError::WrongPhase),
        };
        set_pending(&mut prepared, Some(self.operation), cancel)?;
        let permit = source.permit(operation.request(), current.revision, false, cancel)?;
        dom.publish_hosted(prepared, cancel)?;
        dom.hosted_operation
            .as_mut()
            .expect("original retained operation")
            .phase = next;
        permit.commit();
        publication.take();
        Ok(())
    }
    /// Explicitly retire the completed original cursor. Failure keeps both the
    /// permission and the canonical continuation available for retry.
    pub fn finish(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreation, DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.validate_hosted_source(source)?;
        let operation = dom.hosted_operation(self.operation)?;
        let CreationPhase::Complete(request, result) = &operation.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let result = *result;
        let draft = dom.hosted_draft(operation, source, cancel)?;
        let mut prepared = draft.finish(dom, cancel)?;
        set_pending(&mut prepared, None, cancel)?;
        let permit = source.permit(request, source.revision()?, true, cancel)?;
        dom.publish_hosted(prepared, cancel)?;
        permit.commit();
        dom.hosted_operation.take();
        self.completed = Some(result);
        Ok(result)
    }
}

fn set_pending(
    prepared: &mut Prepared,
    operation: Option<DomHostedOperationId>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    if prepared.state.incomplete_hosted != operation {
        source::formation::reserve(
            &mut prepared.changes,
            1,
            prepared.formation.as_ref(),
            cancel,
        )?;
        prepared.changes.push(DomChange::HostedOperationState {
            operation: operation
                .or(prepared.state.incomplete_hosted)
                .expect("one actual published operation"),
            complete: operation.is_none(),
        });
        prepared.state.incomplete_hosted = operation;
    }
    Ok(())
}

impl Dom {
    fn hosted_draft(
        &self,
        operation: &Operation,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<Draft, DomHostedError> {
        let limits = crate::DomSourceLimits::new(
            source
                .remaining_bytes()?
                .min(source.limits().scratch_bytes()),
            self.limits.units,
            source.limits().work(),
            0,
            0,
            0,
        );
        Ok(Draft::new_hosted_source(
            self,
            DomBatch::new().owner,
            limits,
            operation.request().work.spent.clone(),
            operation.request().key.operation,
            cancel,
        )?)
    }
    fn prepare_hosted_document(
        &self,
        operation: &Operation,
        source: &DomHostedSource,
        global: DomHostedGlobalId,
        navigation_start: DomHostedMoment,
        inputs: &crate::document::HostedDocumentInputs,
        cancel: &AtomicBool,
    ) -> Result<(Prepared, DocumentId), DomHostedError> {
        let request = operation.request();
        let (registries, registry) = registry::Bank::form(
            self.state.registries.as_deref(),
            self.owner,
            source,
            &request.work,
            cancel,
        )?;
        let retained = source.retain(size_of::<DocumentAssociation>(), cancel)?;
        let association = Arc::new(DocumentAssociation {
            source: source.observe(),
            host: request.host,
            context: request.context,
            global,
            navigation_start,
            policy: operation.policy.ok_or(DomHostedError::WrongPhase)?,
            phase: DocumentPhase::Constructed,
            _retained: retained,
        });
        let mut draft = self.hosted_draft(operation, source, cancel)?;
        draft.registries = Some(registries);
        let formation = source::formation::Formation::new(&draft, cancel)?;
        if let Some(formation) = &formation {
            let mut payload = source::storage::Payload::default();
            payload.add_bytes(inputs.payload_bytes()?)?;
            payload.add_units(inputs.url().retained_storage_usage().utf16_extent())?;
            // Metadata also retains its URL occurrence, sharing this actual backing.
            payload.string(inputs.url().serialization())?;
            formation.retain_payload(payload, cancel)?;
        }
        let metadata = DomDocumentMetadata::hosted_initial(
            self.document_environment.input_platform(),
            operation.origin.clone(),
            registry,
            association,
            request.key.operation,
            inputs.clone(),
            &mut |text| source::formation::string(text, formation.as_ref(), cancel),
        )?;
        let document = Key::Created(draft.created.len());
        let document = draft.allocate(
            DomNodeKind::Document(metadata),
            document,
            self.limits,
            cancel,
        )?;
        drop(formation);
        let html = draft.html_skeleton_element(document, "html", self.limits, cancel)?;
        draft.select_element_registry(html, cancel)?;
        draft.pre_insert(html, document, None, self.limits, cancel)?;
        let head = draft.html_skeleton_element(document, "head", self.limits, cancel)?;
        draft.select_element_registry(head, cancel)?;
        draft.pre_insert(head, html, None, self.limits, cancel)?;
        let body = draft.html_skeleton_element(document, "body", self.limits, cancel)?;
        draft.select_element_registry(body, cancel)?;
        draft.pre_insert(body, html, None, self.limits, cancel)?;
        let (mut prepared, mapped) =
            draft.finish_with_mapping(self, cancel, [document].into_iter())?;
        let document = DocumentId(mapped[0]);
        prepared.state.hosted_globals = Some(GlobalDocumentIndex::establish(
            self.state.hosted_globals.as_deref(),
            global,
            document,
            source,
            &request.work,
            prepared.formation.as_ref(),
            cancel,
        )?);
        Ok((prepared, document))
    }
    fn prepare_hosted_phase(
        &self,
        operation: &Operation,
        source: &DomHostedSource,
        document: DocumentId,
        phase: DocumentPhase,
        cancel: &AtomicBool,
    ) -> Result<Prepared, DomHostedError> {
        let node = self.state.node(self.owner, document.node())?;
        let DomNodeKind::Document(metadata) = &node.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let old = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        if old.context != operation.request().context || old.host != operation.request().host {
            return Err(DomHostedError::StaleBasis);
        }
        let retained = source.retain(size_of::<DocumentAssociation>(), cancel)?;
        let association = Arc::new(DocumentAssociation {
            source: old.source.clone(),
            host: old.host,
            context: old.context,
            global: old.global,
            navigation_start: old.navigation_start,
            policy: old.policy,
            phase,
            _retained: retained,
        });
        let mut draft = self.hosted_draft(operation, source, cancel)?;
        let DomNodeKind::Document(metadata) =
            &mut draft.node_mut(Key::Existing(document.node().slot)).kind
        else {
            return Err(DomError::WrongDocument.into());
        };
        metadata.set_hosted_association(association);
        Ok(draft.finish(self, cancel)?)
    }
}

/// Immutable canonical identity index; no activity or execution state is mirrored.
#[derive(Debug)]
pub(crate) struct GlobalDocumentIndex {
    entries: Vec<Option<(DomHostedGlobalId, DocumentId)>>,
    _retained: Retained,
}
impl PartialEq for GlobalDocumentIndex {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl Eq for GlobalDocumentIndex {}
impl GlobalDocumentIndex {
    pub(in crate::store) fn document(
        &self,
        global: DomHostedGlobalId,
    ) -> Result<DocumentId, DomHostedError> {
        self.entries
            .get(global.slot)
            .and_then(|entry| *entry)
            .filter(|(current, _)| *current == global)
            .map(|(_, document)| document)
            .ok_or(DomHostedError::StaleIdentity)
    }
    pub(in crate::store) fn retained_payload_bytes(&self) -> Result<usize, DomError> {
        self.entries
            .capacity()
            .checked_mul(size_of::<Option<(DomHostedGlobalId, DocumentId)>>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DomError::RepresentationExhausted)
    }
    fn establish(
        old: Option<&Self>,
        global: DomHostedGlobalId,
        document: DocumentId,
        source: &DomHostedSource,
        work: &Work,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Arc<Self>, DomHostedError> {
        work.step(cancel)?;
        let required = old.map_or(0, |index| index.entries.len()).max(
            global
                .slot
                .checked_add(1)
                .ok_or(DomHostedError::RepresentationExhausted)?,
        );
        let backing = required
            .checked_mul(size_of::<Option<(DomHostedGlobalId, DocumentId)>>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let header = size_of::<Vec<Option<(DomHostedGlobalId, DocumentId)>>>();
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
        let retained = source.retain(requested, cancel)?;
        if let Some(formation) = formation {
            formation.retain_bytes(requested, cancel)?;
        }
        // Field order keeps the debit live until after backing destruction on
        // every refusal, including a late callback during the retained copy.
        let mut candidate = Self {
            entries: Vec::new(),
            _retained: retained,
        };
        candidate
            .entries
            .try_reserve_exact(required)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let actual = candidate
            .entries
            .capacity()
            .checked_mul(size_of::<Option<(DomHostedGlobalId, DocumentId)>>())
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
        candidate._retained.append(source.retain(extra, cancel)?)?;
        if let Some(formation) = formation {
            formation.retain_bytes(extra, cancel)?;
        }
        if let Some(old) = old {
            for entry in &old.entries {
                work.step(cancel)?;
                candidate.entries.push(*entry);
            }
        }
        while candidate.entries.len() < required {
            work.step(cancel)?;
            candidate.entries.push(None);
        }
        work.step(cancel)?;
        if candidate.entries[global.slot].is_some_and(|(current, _)| current == global) {
            return Err(DomHostedError::StaleBasis);
        }
        candidate.entries[global.slot] = Some((global, document));
        Ok(Arc::new(candidate))
    }
}
impl DomSnapshot {
    /// Historical creation identity, independent of later adoption or retirement.
    /// This observation grants no current execution authority.
    pub fn relevant_global(
        &self,
        node: NodeId,
    ) -> Result<Option<DomHostedGlobalId>, DomHostedError> {
        let global = self.state.node(self.owner, node)?.relevant_global;
        if global.is_some_and(|id| id.source.owner != self.owner) {
            return Err(DomHostedError::ForeignSource);
        }
        Ok(global)
    }
    /// The checked association retained by this exact immutable snapshot.
    /// Current execution uses the exclusive Dom owner's qualified method.
    pub fn global_document(&self, global: DomHostedGlobalId) -> Result<DocumentId, DomHostedError> {
        if global.source.owner != self.owner {
            return Err(DomHostedError::ForeignSource);
        }
        let document = self
            .state
            .hosted_globals
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?
            .document(global)?;
        let metadata = self.document(document)?.metadata();
        if metadata
            .document_environment()
            .hosted
            .as_ref()
            .is_none_or(|association| association.global != global)
        {
            return Err(DomHostedError::StaleIdentity);
        }
        Ok(document)
    }
}
impl Dom {
    /// Current association qualified by the original issuer, role generations
    /// and this canonical owner's completed source basis.
    pub fn global_document(&self, global: DomHostedGlobalId) -> Result<DocumentId, DomHostedError> {
        let snapshot = self.snapshot();
        snapshot.require_complete_source()?;
        let document = snapshot.global_document(global)?;
        let environment = snapshot
            .document(document)?
            .metadata()
            .document_environment();
        environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?
            .validate_live()?;
        Ok(document)
    }
    /// Current use of a node's original Global. Adoption does not substitute
    /// the destination Document's Global for this creation identity.
    pub fn relevant_global(
        &self,
        node: NodeId,
    ) -> Result<Option<DomHostedGlobalId>, DomHostedError> {
        let snapshot = self.snapshot();
        snapshot.require_complete_source()?;
        let global = snapshot.relevant_global(node)?;
        if let Some(global) = global {
            self.global_document(global)?;
        }
        Ok(global)
    }
}

impl Draft {
    pub(super) fn document_fully_active(&self, document: Key) -> Result<bool, DomError> {
        match &self.document_environment(document)?.hosted {
            None => Ok(false),
            Some(association) => {
                association
                    .validate_live()
                    .map_err(|_| DomError::HostedSourceUnavailable)?;
                association
                    .historical_fully_active()
                    .map_err(|_| DomError::HostedSourceUnavailable)
            }
        }
    }
}

impl Dom {
    /// Retain the actual top-level teardown. Queuing its task does not make the
    /// current canonical tree incomplete or block an already started dispatch.
    pub fn begin_hosted_top_level_destruction(
        &mut self,
        source: &mut DomHostedSource,
        navigable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionLease, DomHostedError> {
        self.require_source_idle()?;
        self.validate_hosted_source(source)?;
        if self.hosted_operation.is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        source.observe().validate(
            navigable.source,
            Role::Navigable,
            navigable.slot,
            navigable.generation,
        )?;
        check_cancel(cancel)?;
        // A retained pre-publication creation prefix still owns its original
        // work counter and future limit updates, even before a Dom cursor exists.
        if source.active_work.upgrade().is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        let work = source.work(cancel)?;
        let mut found = None;
        for (slot, value) in self.state.slots.iter().enumerate() {
            work.step(cancel)?;
            let Some(Node {
                kind: DomNodeKind::Document(metadata),
                ..
            }) = &value.node
            else {
                continue;
            };
            let environment = metadata.document_environment();
            let Some(association) = &environment.hosted else {
                continue;
            };
            let DocumentPhase::InitialEntry {
                entry,
                navigable: current,
                ..
            } = association.phase
            else {
                continue;
            };
            if current != navigable {
                continue;
            }
            association.validate_live()?;
            found = Some((
                DomHostedCreation {
                    document: DocumentId(NodeId {
                        owner: self.owner,
                        slot,
                        generation: value.generation,
                    }),
                    host: association.host,
                    context: association.context,
                    global: association.global,
                    entry,
                    navigable,
                },
                metadata.origin().clone(),
            ));
            break;
        }
        let (creation, origin) = found.ok_or(DomHostedError::StaleIdentity)?;
        self.revision
            .0
            .checked_add(1)
            .ok_or(DomError::RevisionExhausted)?;
        let retained = source.retain(size_of::<Operation>(), cancel)?;
        let operation = source.begin_operation(&work, 4, cancel)?;
        source.active_work = Arc::downgrade(&work);
        let request = Request {
            key: RequestKey {
                operation,
                phase: 0,
            },
            host: creation.host,
            context: creation.context,
            work,
        };
        self.hosted_operation = Some(Box::new(Operation {
            origin,
            policy: None,
            phase: CreationPhase::DestroyEnqueue(DomHostedDestructionRequest { request, creation }),
            _retained: retained,
        }));
        self.hosted_revisions = 1;
        Ok(DomHostedDestructionLease {
            operation,
            completed: None,
        })
    }
    fn require_hosted_cleanup_ready(
        &self,
        operation: &Operation,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.require_source_idle()?;
        let work = &operation.request().work;
        // Mutable CSSOM custody is reconciled by its actual owner, never by a
        // lifecycle metadata transition. Unrelated Documents remain usable.
        for root in &self.style_cssom_roots {
            work.step(cancel)?;
            if self.state.node(self.owner, root.root())?.document == document {
                return Err(DomHostedError::PendingDocumentObligations);
            }
        }
        for target in &self.state.signal_slots {
            work.step(cancel)?;
            if self.state.node(self.owner, *target)?.document == document {
                return Err(DomHostedError::PendingDocumentObligations);
            }
        }
        for task in &self.state.native_tasks {
            work.step(cancel)?;
            if let select::Task::Update { target, .. } = task
                && self.state.node(self.owner, *target)?.document == document
            {
                return Err(DomHostedError::PendingDocumentObligations);
            }
        }
        for row in &self.state.details_intentions {
            work.step(cancel)?;
            if row
                .queue_association()
                .is_some_and(|queue| queue.document() == document)
            {
                return Err(DomHostedError::PendingDocumentObligations);
            }
        }
        Ok(())
    }
}
impl DomHostedDestructionLease {
    pub fn step<'a>(&self, dom: &'a Dom) -> Result<DomHostedDestructionStep<'a>, DomHostedError> {
        if self.completed.is_some() {
            if self.operation.source.owner != dom.owner {
                return Err(DomHostedError::ForeignSource);
            }
            return Ok(DomHostedDestructionStep::Complete);
        }
        match &dom.hosted_operation(self.operation)?.phase {
            CreationPhase::DestroyEnqueue(request) => {
                Ok(DomHostedDestructionStep::EnqueueParentTask(request))
            }
            CreationPhase::DestroyCleanup(request) => {
                Ok(DomHostedDestructionStep::Cleanup(request))
            }
            CreationPhase::Destroyed(_, _) => Ok(DomHostedDestructionStep::Complete),
            _ => Err(DomHostedError::WrongPhase),
        }
    }
    pub fn accept_publication(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        publication: &mut Option<DomHostedDestructionPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.validate_hosted_source(source)?;
        let operation = dom.hosted_operation(self.operation)?;
        let current = publication
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        if current.key != operation.request().key {
            return Err(DomHostedError::WrongPhase);
        }
        let next = operation.next_request()?;
        match (&operation.phase, current.cleanup) {
            (CreationPhase::DestroyEnqueue(request), false) => {
                let next = CreationPhase::DestroyCleanup(DomHostedDestructionRequest {
                    request: next,
                    creation: request.creation,
                });
                let permit = source.permit(operation.request(), current.revision, false, cancel)?;
                dom.hosted_operation
                    .as_mut()
                    .expect("retained operation")
                    .phase = next;
                permit.commit();
            }
            (CreationPhase::DestroyCleanup(request), true) => {
                dom.require_hosted_cleanup_ready(operation, request.creation.document, cancel)?;
                let creation = request.creation;
                let prepared = dom.prepare_hosted_phase(
                    operation,
                    source,
                    creation.document,
                    DocumentPhase::Retired,
                    cancel,
                )?;
                let permit = source
                    .permit(operation.request(), current.revision, false, cancel)?
                    .retire_top_level(creation)?;
                dom.publish_hosted(prepared, cancel)?;
                dom.hosted_operation
                    .as_mut()
                    .expect("retained operation")
                    .phase = CreationPhase::Destroyed(next, creation);
                permit.commit();
            }
            _ => return Err(DomHostedError::WrongPhase),
        }
        publication.take();
        Ok(())
    }
    /// Acknowledge only the actual completed original cleanup. Host custody is
    /// shared and remains live until a separate proved-empty host cleanup.
    pub fn finish(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreation, DomHostedError> {
        dom.validate_hosted_source(source)?;
        let operation = dom.hosted_operation(self.operation)?;
        let CreationPhase::Destroyed(request, creation) = &operation.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let creation = *creation;
        let permit = source.permit(request, source.revision()?, true, cancel)?;
        permit.commit();
        dom.hosted_operation.take();
        self.completed = Some(creation);
        Ok(creation)
    }
}

#[cfg(test)]
mod index_tests;

#[cfg(test)]
mod constructor_tests;
