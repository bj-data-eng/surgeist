//! Canonical native hosted associations and the original creation continuation.

pub(in crate::store) mod containers;
mod destruction;
pub(crate) mod history;
pub(in crate::store) mod operations;
pub(in crate::store) mod queue;
use super::*;
use crate::DomOpaqueOrigin;
use crate::hosted::*;
use operations::OperationResult;

// Backing fields precede their debit, including on every failed formation.
struct ConstructorFormation {
    units: Vec<u16>,
    serialization: Option<DocumentString>,
    record: Option<surgeist_html_semantics::HtmlBaseUrlRecord>,
    retained: Retained,
}
pub(in crate::store) struct ChildConstructor<'a> {
    pub(in crate::store) parent: &'a DomDocumentMetadata,
    pub(in crate::store) creator_base: &'a surgeist_html_semantics::HtmlBaseUrlRecord,
    pub(in crate::store) top_policy: &'a surgeist_html::HtmlOpenerPolicy,
    pub(in crate::store) sandbox: surgeist_html::HtmlSandboxFlags,
}
impl crate::document::HostedDocumentInputs {
    pub(crate) fn form(
        source: &DomHostedSource,
        work: &Work,
        origin: &DomOpaqueOrigin,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHostedError> {
        Self::form_profile(source, work, origin, None, None, cancel)
    }
    pub(in crate::store) fn form_child(
        source: &DomHostedSource,
        work: &Work,
        profile: ChildConstructor<'_>,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHostedError> {
        Self::form_profile(
            source,
            work,
            profile.parent.origin(),
            Some((
                profile.parent,
                profile.creator_base,
                profile.top_policy,
                profile.sandbox,
            )),
            formation,
            cancel,
        )
    }
    fn form_profile(
        source: &DomHostedSource,
        work: &Work,
        origin: &DomOpaqueOrigin,
        parent: Option<(
            &DomDocumentMetadata,
            &surgeist_html_semantics::HtmlBaseUrlRecord,
            &surgeist_html::HtmlOpenerPolicy,
            surgeist_html::HtmlSandboxFlags,
        )>,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHostedError> {
        use crate::document::HostedDocumentInputData;
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        use surgeist_html_semantics::{
            HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsRevision,
            HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme,
        };
        // The original constructor's intrinsic URL, not an external URL parse.
        const URL: &[u16] = &[97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107];
        work.step(cancel)?;
        let workspace = if let Some(formation) = formation {
            let payload = source::storage::Payload {
                bytes: size_of::<ConstructorFormation>(),
                units: 0,
            };
            formation.retain_payload(payload, cancel)?;
            Some(formation.split_payload(payload))
        } else {
            None
        };
        let requested_string = size_of::<Vec<u16>>() + size_of_val(URL);
        let header = size_of::<HostedDocumentInputData>();
        crate::hosted::admit(
            size_of::<ConstructorFormation>() + header + requested_string,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        if let Some(formation) = formation {
            formation.retain_payload(
                source::storage::Payload {
                    bytes: header
                        .checked_add(requested_string)
                        .ok_or(DomError::RepresentationExhausted)?,
                    units: URL.len(),
                },
                cancel,
            )?;
        }
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
        if let Some(formation) = formation {
            formation.retain_payload(
                source::storage::Payload {
                    bytes: actual_string
                        .checked_sub(requested_string)
                        .ok_or(DomError::RepresentationExhausted)?,
                    units: candidate
                        .units
                        .capacity()
                        .checked_sub(URL.len())
                        .ok_or(DomError::RepresentationExhausted)?,
                },
                cancel,
            )?;
        }
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
            #[cfg(test)]
            written.pushed::<u16>();
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
        if let Some(formation) = formation {
            formation.retain_bytes(additional, cancel)?;
        }
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
                None,
            )
            .expect("same prepriced serialization and intrinsic about:blank predicates"),
        );
        // Capture all actual creator operands before cloning them into the
        // once-issued request/Document owner. The parent's current base and its
        // document URL/referrer are separate source inputs.
        let (about, opener, top_creation_url, top_origin, active_sandbox, referrer) =
            if let Some((parent, base, opener, sandbox)) = parent {
                work.step(cancel)?;
                let parent_inputs = parent
                    .hosted_inputs
                    .as_ref()
                    .ok_or(DomHostedError::UnsupportedProfile)?;
                let top = parent_inputs
                    .0
                    .top_creation_url
                    .as_ref()
                    .unwrap_or_else(|| parent_inputs.url());
                let mut referrer_payload = source::storage::Payload::default();
                referrer_payload.string(parent.url())?;
                let bytes = base
                    .retained_storage_usage()
                    .retained_bytes()
                    .checked_add(top.retained_storage_usage().retained_bytes())
                    .and_then(|bytes| bytes.checked_add(referrer_payload.bytes))
                    .ok_or(DomHostedError::RepresentationExhausted)?;
                crate::hosted::admit(
                    size_of::<ConstructorFormation>()
                        .checked_add(header)
                        .and_then(|n| n.checked_add(usage.retained_bytes()))
                        .and_then(|n| n.checked_add(bytes))
                        .ok_or(DomHostedError::RepresentationExhausted)?,
                    source.limits().scratch_bytes(),
                    DomHostedResource::ScratchBytes,
                )?;
                if let Some(formation) = formation {
                    let units = base
                        .retained_storage_usage()
                        .utf16_extent()
                        .checked_add(top.retained_storage_usage().utf16_extent())
                        .and_then(|n| n.checked_add(referrer_payload.units))
                        .ok_or(DomError::RepresentationExhausted)?;
                    formation.retain_payload(source::storage::Payload { bytes, units }, cancel)?;
                }
                candidate.retained.append(source.retain(bytes, cancel)?)?;
                let top_origin = &parent_inputs.0.top_origin;
                let selected_opener = if parent.origin() == top_origin {
                    opener.clone()
                } else {
                    surgeist_html::HtmlOpenerPolicy::initial()
                };
                (
                    HtmlAboutBaseUrl::KnownRecord(base.clone()),
                    selected_opener,
                    Some(top.clone()),
                    top_origin.clone(),
                    sandbox,
                    Some(parent.url().clone()),
                )
            } else {
                (
                    HtmlAboutBaseUrl::KnownNull,
                    surgeist_html::HtmlOpenerPolicy::initial(),
                    None,
                    origin.clone(),
                    surgeist_html::HtmlSandboxFlags::empty(),
                    None,
                )
            };
        let facts = HtmlBaseDocumentFacts::new(
            revision,
            candidate.record.take(),
            about,
            false,
            Some(HtmlBaseEncoding::fresh()),
        )
        .expect("constructor record has its exact original revision and is not srcdoc");
        // The workspace guard outlives every temporary backing on refusal;
        // only the result occurrence remains in the caller's Formation.
        let _workspace = workspace;
        // All fallible admission and cancellation precede the immutable owner.
        Ok(Self(Arc::new(HostedDocumentInputData {
            facts,
            opener,
            top_creation_url,
            top_origin,
            active_sandbox,
            referrer,
            _retained: candidate.retained,
        })))
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "Keep the sole continuation inline in the producer-admitted Operation; boxing adds separately fallible storage to the original receipt"
)]
pub(in crate::store) enum CreationPhase {
    Policy(DomHostedPolicyRequest),
    Execution(DomHostedExecutionRequest),
    MakeActive(DomHostedMakeActiveRequest),
    FinishLoading(DomHostedFinishLoadingRequest),
    Entry(DomHostedEntryRequest),
    Complete(Request, DomHostedCreation),
    Queued(queue::Continuation),
    Invocation(destruction::Invocation),
    DestroyEnqueue(DomHostedDestructionRequest),
    DestroyCleanup(DomHostedDestructionRequest),
    Destroyed(Request, DomHostedCreation),
}
pub(in crate::store) struct Operation {
    pub(in crate::store) origin: crate::DomOpaqueOrigin,
    pub(in crate::store) policy: Option<DomInitialPermissionsPolicy>,
    pub(in crate::store) phase: CreationPhase,
    pub(in crate::store) _retained: Retained,
}
impl Operation {
    pub(in crate::store) fn request(&self) -> &Request {
        match &self.phase {
            CreationPhase::Policy(request) => &request.request,
            CreationPhase::Execution(request) => &request.request,
            CreationPhase::MakeActive(request) => &request.request,
            CreationPhase::FinishLoading(request) => &request.request,
            CreationPhase::Entry(request) => &request.request,
            CreationPhase::Complete(request, _) | CreationPhase::Destroyed(request, _) => request,
            CreationPhase::Queued(continuation) => continuation.request(),
            CreationPhase::Invocation(invocation) => &invocation.request.request,
            CreationPhase::DestroyEnqueue(request) | CreationPhase::DestroyCleanup(request) => {
                &request.request
            }
        }
    }
    pub(in crate::store) fn next_request(&self) -> Result<Request, DomHostedError> {
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
            parent: old.parent,
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
    pub(in crate::store) fn validate_hosted_source(
        &self,
        source: &DomHostedSource,
    ) -> Result<(), DomHostedError> {
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
        if self.hosted_operations.synchronous().is_some() {
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
            .checked_add(self.hosted_revisions)
            .and_then(|revision| revision.checked_add(5))
            .ok_or(DomError::RevisionExhausted)?;
        let retained = source.retain(size_of::<Operation>(), cancel)?;
        let origin = DomDocumentMetadata::fresh_opaque_origin();
        let bank = self.hosted_operations.prepare(
            source.next_operation_slot(&prefix.work, cancel)?,
            &prefix.work,
            source,
            0,
            None,
            operations::BodyCredit::Owned(retained),
            operations::Projection::default(),
            cancel,
        )?;
        crate::hosted::admit(
            bank.operation_scratch_bytes()?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let operation = source.begin_operation(&prefix.work, 10, bank.scratch_bytes()?, cancel)?;
        let (bank, retained) = bank.into_owned();
        let prepared = prepared.take().expect("validated original preparation");
        let request = Request {
            key: RequestKey {
                operation,
                phase: 0,
            },
            host: prepared.preparation.host,
            context: prepared.preparation.context,
            work: prepared.preparation.work,
            parent: None,
        };
        drop(self.hosted_operations.insert_prepared(
            operations::Transfer::new(
                bank,
                Operation {
                    origin: origin.clone(),
                    policy: None,
                    phase: CreationPhase::Policy(DomHostedPolicyRequest {
                        request,
                        origin,
                        embedder_policy: None,
                    }),
                    _retained: retained,
                },
            ),
            true,
        ));
        self.hosted_revisions += 5;
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
        self.hosted_operations.get(operation)
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
        let inputs = crate::document::HostedDocumentInputs::form(
            source,
            &operation.request().work,
            &operation.origin,
            cancel,
        )?;
        let next = CreationPhase::Execution(DomHostedExecutionRequest {
            request,
            origin: operation.origin.clone(),
            inputs,
        });
        let permit = source.permit(operation.request(), current.revision, false, cancel)?;
        let mut pending = dom
            .hosted_operations
            .get_mut(self.operation)
            .expect("validated original operation");
        pending.policy = Some(current.policy);
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
                PublicationValue::InitialEntry {
                    entry,
                    navigable,
                    initialization,
                    ..
                },
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
                let mut prepared = dom.prepare_hosted_phase(
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
                let creation = DomHostedCreation {
                    document: request.document,
                    host: operation.request().host,
                    context: operation.request().context,
                    global: request.global,
                    entry: *entry,
                    navigable: *navigable,
                };
                for _ in 0..72 {
                    request.request.work.step(cancel)?;
                }
                let scratch = prepared
                    .formation
                    .as_ref()
                    .map_or(0, |formation| formation.owned_payload().bytes);
                let fresh = prepared
                    .formation
                    .as_ref()
                    .map(source::formation::Formation::fork);
                let context = history::Context::new(
                    source,
                    &request.request.work,
                    cancel,
                    scratch
                        .checked_add(size_of::<Vec<u16>>())
                        .ok_or(DomError::RepresentationExhausted)?,
                );
                let context = if let Some(fresh) = &fresh {
                    context.with_formation(fresh)
                } else {
                    context
                };
                // This fixed top-level front supplies the source's actual empty
                // string name operand. It is distinct from a child's null name.
                let _name_retained = source.retain(size_of::<Vec<u16>>(), cancel)?;
                let _name_source = if let Some(fresh) = &fresh {
                    let payload = source::storage::Payload {
                        bytes: size_of::<Vec<u16>>(),
                        units: 0,
                    };
                    fresh.retain_payload(payload, cancel)?;
                    Some(fresh.split_payload(payload))
                } else {
                    None
                };
                let target_name = DocumentString::from("");
                let DomNodeKind::Document(metadata) = &prepared
                    .state
                    .node(dom.owner, request.document.node())?
                    .kind
                else {
                    return Err(DomError::WrongDocument.into());
                };
                let inputs = metadata
                    .hosted_inputs
                    .as_ref()
                    .ok_or(DomHostedError::UnsupportedProfile)?;
                let graph = history::Graph::initial(
                    dom.state.hosted_history.as_deref(),
                    history::Initial {
                        creation,
                        initialization: DomHostedEntryInitialization::new(
                            initialization.navigation_key,
                            initialization.navigation_id,
                        ),
                        origin: operation.origin.clone(),
                        parent: None,
                        target_name: Some(&target_name),
                        about_base_url: inputs.0.facts.about_base_url(),
                        initiator_origin: None,
                    },
                    &context,
                )?;
                prepared.set_hosted_history(Some(graph), fresh, &dom.custody_registry, cancel)?;
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
        dom.hosted_operations
            .get_mut(self.operation)
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
        let mut draft = dom.hosted_draft(operation, source, cancel)?;
        let mut prepared = draft.finish(dom, cancel)?;
        set_pending(&mut prepared, None, cancel)?;
        let permit = source.permit(request, source.revision()?, true, cancel)?;
        dom.publish_hosted(prepared, cancel)?;
        permit.commit();
        dom.hosted_operations
            .remove(self.operation)
            .expect("completed original operation");
        self.completed = Some(result);
        Ok(result)
    }
}

fn set_pending(
    prepared: &mut Prepared,
    operation: Option<DomHostedOperationId>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    #[cfg(test)]
    let mut written = crate::store::CopyObservation::default();
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
        #[cfg(test)]
        written.pushed::<DomChange>();
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
        self.hosted_request_draft(operation.request(), source, cancel)
    }
    fn hosted_request_draft(
        &self,
        request: &Request,
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
            1,
        );
        Ok(Draft::new_hosted_source(
            self,
            DomBatch::new().owner,
            limits,
            request.work.spent.clone(),
            request.work.spent_id,
            request.key.operation,
            cancel,
        )?)
    }
    /// Hold checked revision headroom for an actual queued original operation.
    /// Every fallible check precedes either owner's reservation change.
    fn reserve_hosted_operation(
        &mut self,
        source: &DomHostedSource,
        request: &Request,
        source_needed: u64,
        dom_revisions: u64,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        check_cancel(cancel)?;
        let reserved = self
            .hosted_revisions
            .checked_add(dom_revisions)
            .ok_or(DomError::RevisionExhausted)?;
        self.revision
            .0
            .checked_add(reserved)
            .ok_or(DomError::RevisionExhausted)?;
        let owned = source.operation_reserved_revisions(request, cancel)?;
        let additional = source_needed.saturating_sub(owned);
        if additional != 0 {
            source.reserve_operation_revisions(request, additional, cancel)?;
        }
        self.hosted_revisions = reserved;
        Ok(())
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
            None,
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
        draft.set_registries(Some(registries), None, cancel)?;
        let formation = source::formation::Formation::new(&draft, cancel)?;
        if let Some(formation) = &formation {
            let mut payload = source::storage::Payload::default();
            payload.add_bytes(inputs.payload_bytes()?)?;
            payload.add_units(inputs.payload_units()?)?;
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
        let document = draft.allocate_formed(
            DomNodeKind::Document(metadata),
            document,
            self.limits,
            cancel,
            formation,
        )?;
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
        let fresh = prepared
            .formation
            .as_ref()
            .map(source::formation::Formation::fork);
        let globals = GlobalDocumentIndex::establish(
            self.state.hosted_globals.as_deref(),
            global,
            document,
            source,
            &request.work,
            fresh.as_ref(),
            cancel,
        )?;
        prepared.set_hosted_globals(Some(globals), fresh, &self.custody_registry, cancel)?;
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
        draft.with_node_fields(Key::Existing(document.node()), cancel, |fields| {
            let DomNodeKind::Document(metadata) = &mut fields.kind else {
                return Err(DomError::WrongDocument);
            };
            metadata.set_hosted_association(association);
            Ok(())
        })??;
        Ok(draft.finish(self, cancel)?)
    }
}

/// Immutable canonical identity index; issuer/slot lookup never fills holes.
#[derive(Debug)]
pub(crate) struct GlobalDocumentIndex {
    identity: custody::MaterializationId,
    entries: transaction::auxiliary::index::Index<GlobalMapping>,
    _retained: Retained,
}
#[derive(Debug)]
pub(in crate::store) struct GlobalMapping {
    pub global: DomHostedGlobalId,
    pub document: DocumentId,
    pub origin: custody::MaterializationId,
    _retained: Retained,
}
impl PartialEq for GlobalDocumentIndex {
    fn eq(&self, other: &Self) -> bool {
        self.entries.same_values(&other.entries, &|a, b| {
            a.global == b.global && a.document == b.document
        })
    }
}
impl Eq for GlobalDocumentIndex {}
impl GlobalDocumentIndex {
    pub(in crate::store) fn custody_identity(&self) -> custody::MaterializationId {
        self.identity
    }
    pub(in crate::store) fn custody_usage(&self) -> custody::Usage {
        custody::Usage::new(size_of::<Self>(), 0)
    }
    pub(in crate::store) fn custody_bank(
        &self,
    ) -> &transaction::auxiliary::index::Index<GlobalMapping> {
        &self.entries
    }
    pub(in crate::store) fn document(
        &self,
        global: DomHostedGlobalId,
    ) -> Result<DocumentId, DomHostedError> {
        let key =
            transaction::auxiliary::index::Key::new(global.source, global.slot, global.generation);
        self.entries
            .get(key)
            .map_err(|error| match error {
                DomError::ForeignOwner => DomHostedError::StaleIdentity,
                other => other.into(),
            })?
            .map(|entry| entry.document)
            .ok_or(DomHostedError::StaleIdentity)
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_materializations(
        &self,
    ) -> Result<Vec<(custody::MaterializationId, custody::Usage)>, DomError> {
        let mut output = vec![(self.identity, custody::Usage::new(size_of::<Self>(), 0))];
        output.extend(self.entries.inspect_materializations(|_| {
            Ok(custody::Usage::new(size_of::<GlobalMapping>(), 0))
        })?);
        Ok(output)
    }
    pub(in crate::store) fn establish(
        old: Option<&Self>,
        global: DomHostedGlobalId,
        document: DocumentId,
        source: &DomHostedSource,
        work: &Work,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Arc<Self>, DomHostedError> {
        work.step(cancel)?;
        let identity = custody::MaterializationId::fresh()?;
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
        let key =
            transaction::auxiliary::index::Key::new(global.source, global.slot, global.generation);
        let duplicate = old
            .map(|old| old.entries.get(key))
            .transpose()?
            .flatten()
            .is_some();
        let mut candidate = Self {
            identity,
            entries: old.map_or_else(Default::default, |old| old.entries.clone()),
            _retained: retained,
        };
        let value = producer.record(|retained| GlobalMapping {
            global,
            document,
            origin: identity,
            _retained: retained,
        })?;
        candidate.entries.insert(
            key,
            value,
            custody::Usage::new(size_of::<GlobalMapping>(), 0),
            true,
            &producer,
        )?;
        if duplicate {
            return Err(DomHostedError::StaleBasis);
        }
        let candidate = Arc::new(candidate);
        #[cfg(test)]
        transaction::auxiliary::allocated(size_of::<Self>());
        Ok(candidate)
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

impl Dom {
    /// Current Global association for an actual native dispatcher. An unfinished
    /// Source requires its exact active dispatch scope; snapshots stay incomplete.
    pub fn dispatch_relevant_global(
        &self,
        node: NodeId,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedGlobalId>, DomHostedError> {
        let _work = self.dispatch_hosted_read_work(cancel)?;
        let global = self.snapshot().relevant_global(node)?;
        if let Some(global) = global {
            self.dispatch_global_document_current(global)?;
        }
        Ok(global)
    }
    /// Current issuer/role-qualified Document of a dispatcher Global endpoint.
    pub fn dispatch_global_document(
        &self,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<DocumentId, DomHostedError> {
        let _work = self.dispatch_hosted_read_work(cancel)?;
        self.dispatch_global_document_current(global)
    }
    fn dispatch_global_document_current(
        &self,
        global: DomHostedGlobalId,
    ) -> Result<DocumentId, DomHostedError> {
        let snapshot = self.snapshot();
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
    /// Fresh activity for an actual dispatcher endpoint. Only canonical closed
    /// or retired state proves false; positive state validates its real issuer.
    pub fn dispatch_document_fully_active(
        &self,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedError> {
        let work = self.dispatch_hosted_read_work(cancel)?;
        let DomNodeKind::Document(metadata) = &self.state.node(self.owner, document.node())?.kind
        else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(false);
        };
        if matches!(association.phase, DocumentPhase::Retired { .. }) {
            return Ok(false);
        }
        association.validate_live()?;
        let DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
            return Err(DomHostedError::IncompleteOperation);
        };
        let graph = self
            .state
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        captured_fully_active(graph, document, navigable, || match &work {
            Some(work) => work.step(cancel).map_err(Into::into),
            None => check_cancel(cancel).map_err(Into::into),
        })
    }
}

impl Draft {
    pub(super) fn document_fully_active(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        match &self
            .document_metadata(document)?
            .document_environment()
            .hosted
        {
            None => Ok(false),
            Some(association) if matches!(association.phase, DocumentPhase::Retired { .. }) => {
                Ok(false)
            }
            Some(association) => {
                association
                    .validate_live()
                    .map_err(|_| DomError::HostedSourceUnavailable)?;
                let DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
                    return Err(DomError::HostedSourceUnavailable);
                };
                let document = self
                    .published_id(document)
                    .map(DocumentId)
                    .ok_or(DomError::HostedSourceUnavailable)?;
                let graph = self
                    .hosted_history
                    .as_deref()
                    .ok_or(DomError::HostedSourceUnavailable)?;
                captured_fully_active(graph, document, navigable, || {
                    self.source_step(cancel).map_err(Into::into)
                })
                .map_err(|error| match error {
                    DomHostedError::Dom(error) => error,
                    _ => DomError::HostedSourceUnavailable,
                })
            }
        }
    }
}

impl Dom {
    fn require_hosted_cleanup_ready(
        &self,
        operation: &Operation,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.require_source_idle()?;
        let work = &operation.request().work;
        // HTML's destroy-Document path runs unloading cleanup even when event
        // unloading is suppressed for an initial child. Canonical fullscreen
        // membership cannot be certified settled by retiring the hosted graph.
        let captured = self.state.node(self.owner, document.node())?;
        let Some(canonical::Interactive::Known { top_layer, .. }) = &captured.interactive_document
        else {
            return Err(DomError::FullscreenModelUnavailable {
                step: crate::DomFullscreenModelStep::ReadTopLayer,
            }
            .into());
        };
        for &id in top_layer.iter().rev() {
            work.step(cancel)?;
            let DomNodeKind::Element(element) = &self.state.node(self.owner, id)?.kind else {
                return Err(DomError::WrongNodeKind.into());
            };
            if element.fullscreen_flag() {
                return Err(DomError::FullscreenModelUnavailable {
                    step: crate::DomFullscreenModelStep::FullyExitFullscreen,
                }
                .into());
            }
        }
        // Mutable CSSOM custody is reconciled by its actual owner, never by a
        // lifecycle metadata transition. Unrelated Documents remain usable.
        work.step(cancel)?;
        if self.style_cssom_roots.has_document(document) {
            return Err(DomHostedError::PendingDocumentObligations);
        }
        work.step(cancel)?;
        if self.state.signal_targets.has_document_obligations(document) {
            return Err(DomHostedError::PendingDocumentObligations);
        }
        work.step(cancel)?;
        if self.state.native_targets.has_document_obligations(document) {
            return Err(DomHostedError::PendingDocumentObligations);
        }
        work.step(cancel)?;
        if self
            .state
            .details_targets
            .has_document_obligations(document)
        {
            return Err(DomHostedError::PendingDocumentObligations);
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
                dom.require_hosted_invocation_children(
                    request.invocation,
                    &request.request.work,
                    cancel,
                )?;
                let next = CreationPhase::DestroyCleanup(DomHostedDestructionRequest {
                    request: next,
                    creation: request.creation,
                    children: request.children.clone(),
                    invocation: request.invocation,
                    traversable: request.traversable,
                });
                let permit = source.permit(operation.request(), current.revision, false, cancel)?;
                dom.hosted_operations
                    .get_mut(self.operation)
                    .expect("retained operation")
                    .phase = next;
                permit.commit();
            }
            (CreationPhase::DestroyCleanup(request), true) => {
                dom.require_hosted_invocation_children(
                    request.invocation,
                    &request.request.work,
                    cancel,
                )?;
                dom.require_hosted_cleanup_ready(operation, request.creation.document, cancel)?;
                let creation = request.creation;
                let mut prepared = dom.prepare_hosted_phase(
                    operation,
                    source,
                    creation.document,
                    DocumentPhase::Retired {
                        operation: self.operation,
                    },
                    cancel,
                )?;
                let fresh = prepared
                    .formation
                    .as_ref()
                    .map(source::formation::Formation::fork);
                let scratch = prepared
                    .formation
                    .as_ref()
                    .map_or(0, |formation| formation.owned_payload().bytes);
                let context = history::Context::new(source, &request.request.work, cancel, scratch);
                let context = if let Some(fresh) = &fresh {
                    context.with_formation(fresh)
                } else {
                    context
                };
                let graph = prepared
                    .state
                    .hosted_history
                    .as_deref()
                    .ok_or(DomHostedError::UnsupportedProfile)?
                    .clear_document(creation.document, &context)?;
                prepared.set_hosted_history(Some(graph), fresh, &dom.custody_registry, cancel)?;
                let permit = source
                    .permit(operation.request(), current.revision, false, cancel)?
                    .retire_navigable(creation)?;
                dom.publish_hosted(prepared, cancel)?;
                dom.hosted_operations
                    .get_mut(self.operation)
                    .expect("retained operation")
                    .phase = CreationPhase::Destroyed(next, creation);
                permit.commit();
                self.proof = Some(DomHostedDocumentDestructionProof {
                    source: source.id(),
                    document: creation.document,
                    global: creation.global,
                    operation: self.operation,
                });
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
        if let Some(creation) = self.completed {
            self.proof
                .as_ref()
                .ok_or(DomHostedError::WrongPhase)?
                .validate(source, creation.document, creation.global)?;
            return Ok(creation);
        }
        let operation = dom.hosted_operation(self.operation)?;
        let CreationPhase::Destroyed(request, creation) = &operation.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let creation = *creation;
        let permit = source.permit(request, source.revision()?, true, cancel)?;
        permit.commit();
        dom.hosted_operations
            .remove(self.operation)
            .expect("completed original operation");
        self.completed = Some(creation);
        Ok(creation)
    }
}

#[cfg(test)]
pub(in crate::store) mod index_tests;

#[cfg(test)]
mod constructor_tests;
#[cfg(test)]
mod container_locality_tests;
#[cfg(test)]
mod source_formation_tests;

impl Dom {
    /// Visits actual content-navigable edges in canonical shadow-including tree
    /// order. No issuer-slot order or later Runtime tree mirror participates.
    fn visit_hosted_children(
        &self,
        graph: &history::Graph,
        parent: DomHostedNavigableId,
        work: &Work,
        cancel: &AtomicBool,
        mut visit: impl FnMut(DomHostedNavigableId) -> Result<(), DomHostedError>,
    ) -> Result<(), DomHostedError> {
        work.step(cancel)?;
        let parent = graph.navigable_with(parent, || work.step(cancel))?;
        let Some(document) = graph
            .entry_with(parent.active, || work.step(cancel))?
            .document
        else {
            return Ok(());
        };
        let root = Key::Existing(document.node());
        let order = transaction::ordered_members::OrderView::state(&self.state, self.owner);
        let index = &self.state.container_index;
        let mut current = index.first_hosted(root, work, cancel)?;
        while let Some(key) = current {
            work.step(cancel)?;
            let Key::Existing(id) = key else {
                return Err(DomError::UnavailableCreation.into());
            };
            let node = self.state.node(self.owner, id)?;
            if let DomNodeKind::Element(element) = &node.kind
                && let Some(Some(child)) = element.child_navigable()
            {
                work.step(cancel)?;
                if graph.navigable_with(child, || work.step(cancel))?.parent == Some(parent.id) {
                    visit(child)?;
                }
            }
            current = index.after_hosted(root, key, order, work, cancel)?;
        }
        Ok(())
    }

    fn capture_hosted_children(
        &self,
        source: &DomHostedSource,
        creation: DomHostedCreation,
        work: &Work,
        scratch_base: usize,
        cancel: &AtomicBool,
    ) -> Result<Arc<CapturedChildren>, DomHostedError> {
        let graph = self
            .state
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        let mut count = 0usize;
        self.visit_hosted_children(graph, creation.navigable, work, cancel, |_| {
            count = count
                .checked_add(1)
                .ok_or(DomHostedError::RepresentationExhausted)?;
            Ok(())
        })?;
        let extent = |capacity: usize| {
            capacity
                .checked_mul(size_of::<DomHostedCreation>())
                .and_then(|bytes| bytes.checked_add(size_of::<CapturedChildren>()))
                .ok_or(DomHostedError::RepresentationExhausted)
        };
        let requested = extent(count)?;
        crate::hosted::admit(
            scratch_base
                .checked_add(requested)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let retained = source.retain(requested, cancel)?;
        let mut children = CapturedChildren {
            values: Vec::new(),
            _retained: retained,
        };
        children
            .values
            .try_reserve_exact(count)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        #[cfg(test)]
        if scale_metrics::ENABLED && children.values.capacity() != 0 {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: children.values.capacity() * size_of::<DomHostedCreation>(),
            });
        }
        let actual = extent(children.values.capacity())?;
        crate::hosted::admit(
            scratch_base
                .checked_add(actual)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        children._retained.append(
            source.retain(
                actual
                    .checked_sub(requested)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                cancel,
            )?,
        )?;
        self.visit_hosted_children(graph, creation.navigable, work, cancel, |child| {
            work.step(cancel)?;
            let child = graph.navigable_with(child, || work.step(cancel))?;
            // The original created Document identity remains cleanup custody
            // even after another invocation clears current DocumentState.
            children.values.push(DomHostedCreation {
                document: child.initial_document,
                host: child.host,
                context: child.context,
                global: child.global,
                entry: child.active,
                navigable: child.id,
            });
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::record(scale_metrics::Event::SequenceMaterialized {
                    bytes: size_of::<DomHostedCreation>(),
                });
            }
            Ok(())
        })?;
        let children = Arc::new(children);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<CapturedChildren>(),
            });
        }
        Ok(children)
    }
}

/// One canonical relation predicate, shared by historical observation and
/// current execution. It owns no activity table and performs no allocation.
pub(crate) fn captured_fully_active(
    graph: &history::Graph,
    document: DocumentId,
    navigable: DomHostedNavigableId,
    mut step: impl FnMut() -> Result<(), DomHostedError>,
) -> Result<bool, DomHostedError> {
    let mut id = navigable;
    let mut expected = Some(document);
    loop {
        let row = graph.navigable_with(id, &mut step)?;
        if row.retired {
            return Ok(false);
        }
        let active = graph.entry_with(row.active, &mut step)?.document;
        if active.is_none() || expected.is_some_and(|expected| active != Some(expected)) {
            return Ok(false);
        }
        match row.parent {
            Some(parent) => {
                id = parent;
                expected = None;
            }
            None => return Ok(row.id == row.traversable),
        }
    }
}
impl Dom {
    pub(in crate::store) fn hosted_document_fully_active(
        &self,
        document: DocumentId,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedError> {
        work.step(cancel)?;
        let DomNodeKind::Document(metadata) = &self.state.node(self.owner, document.node())?.kind
        else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let Some(association) = &environment.hosted else {
            return Ok(false);
        };
        let DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
            return if matches!(association.phase, DocumentPhase::Retired { .. }) {
                Ok(false)
            } else {
                Err(DomHostedError::IncompleteOperation)
            };
        };
        association.validate_current()?;
        let graph = self
            .state
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        captured_fully_active(graph, document, navigable, || work.step(cancel))
    }
}

impl DomHostedSource {
    /// Admit actual native Window-manager backing through the original current
    /// Source and every observing ancestor, before allocation or replacement.
    pub fn retain_runtime_payload(
        &self,
        dom: &mut Dom,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<crate::DomHostedRuntimeRetention, DomError> {
        dom.validate_hosted_source(self)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        self.shared
            .runtime_bytes
            .load(Ordering::Relaxed)
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        self.shared
            .runtime_bytes
            .fetch_add(bytes, Ordering::Relaxed);
        let retained = crate::DomHostedRuntimeRetention {
            shared: Arc::downgrade(&self.shared),
            bytes,
        };
        dom.admit_source_owner_payload(cancel)?;
        Ok(retained)
    }
}

impl crate::DomHostedSource {
    /// Admit actual source-issued Runtime Host work on the original counters.
    /// Independent calls with no active Source have no Source work domain.
    pub fn admit_runtime_work(
        &self,
        dom: &Dom,
        steps: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        dom.validate_hosted_source(self)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        dom.admit_hosted_runtime_work(steps, cancel)
    }
}
