//! Hosted effects retain this exact original Source root and its Draft keys.
use super::*;
#[derive(Clone, Copy)]
pub(in crate::store) enum ChildHook {
    Create {
        key: Key,
    },
    Remove {
        key: Key,
        navigable: crate::DomHostedNavigableId,
    },
}
impl ChildHook {
    pub(in crate::store) fn key(self) -> Key {
        match self {
            Self::Create { key } | Self::Remove { key, .. } => key,
        }
    }
}

use crate::hosted::*;
use crate::store::hosted as canonical;
use crate::{DomDocumentStamp, DomOpaqueOrigin};
use surgeist_html::{HtmlSandboxFlag, KnownHtmlAttribute};
use surgeist_html_semantics::{HtmlBaseFactsRevision, HtmlBaseParseOutcome};

pub(in crate::store) struct Child {
    pub(in crate::store) request: DomHostedChildRequest,
    hook: ChildHook,
    phase: Phase,
    document: Option<Key>,
    declared_sandbox: Option<DomOpaqueOrigin>,
    url: Option<crate::DomSourceUrlRequest>,
    observation: Option<AcceptedUrl>,
    prefix: Option<ChildPrefix>,
    url_charge: Option<checkpoint::Charge>,
    work_charge: Option<checkpoint::Charge>,
    phase_charge: Option<checkpoint::Charge>,
    handoff_charge: Option<checkpoint::Charge>,
}
struct ChildPrefix {
    host: DomHostedHostId,
    context: DomHostedBrowsingContextId,
    work: Arc<Work>,
}
struct AcceptedUrl {
    value: crate::DomSourceUrlObservation,
    _charge: checkpoint::Charge,
}
enum Phase {
    Prepare,
    PrepareRemoval,
    Removing(RemovingChild),
    RemovalPublished(RemovalPublished),
    Creating(canonical::Operation),
    Queue(QueuedChild),
    Attributes { work: Arc<Work> },
    Returned,
}
struct RemovingChild {
    packet: DomHostedChildRemovalRequest,
    origin: DomOpaqueOrigin,
    retained: Retained,
}
struct RemovalPublished {
    packet: DomHostedChildRemovalRequest,
    key: RequestKey,
    history: DomHostedQueuedHistory,
    creation: DomHostedCreation,
    work: Arc<Work>,
}
struct QueuedChild {
    packet: DomHostedChildHistoryRequest,
    origin: DomOpaqueOrigin,
    policy: Option<DomInitialPermissionsPolicy>,
    retained: Retained,
}
impl Child {
    pub(super) fn returned(&self) -> bool {
        matches!(self.phase, Phase::Returned)
    }
    pub(super) fn form(
        operation: &super::Operation,
        state: &State,
        ids: &[Option<NodeId>],
        basis: DomStamp,
        hook: ChildHook,
        budget: Option<&formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        operation.draft.source_step(cancel)?;
        let container = match hook.key() {
            Key::Existing(slot) => NodeId {
                owner: basis.owner,
                slot,
                generation: operation.draft.generations[slot],
            },
            Key::Created(i) => ids[i].ok_or(DomError::UnavailableCreation)?,
        };
        let node = state.node(basis.owner, container)?;
        let DomNodeKind::Document(metadata) = &state.node(basis.owner, node.document.node())?.kind
        else {
            return Err(DomError::WrongDocument);
        };
        let environment = metadata.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        association
            .validate_live()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let DocumentPhase::InitialEntry {
            navigable: parent, ..
        } = association.phase
        else {
            return Err(DomError::HostedSourceUnavailable);
        };
        let graph = state
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let parent_row = graph
            .navigable_with(parent, || {
                operation.draft.source_step(cancel).map_err(Into::into)
            })
            .map_err(|error| match error {
                DomHostedError::Dom(error) => error,
                _ => DomError::HostedSourceUnavailable,
            })?;
        if parent_row.retired || (matches!(hook, ChildHook::Create { .. }) && parent_row.retiring) {
            return Err(DomError::HostedSourceUnavailable);
        }
        formation::step(budget, cancel)?;
        let occurrence = operation
            .draft
            .source_invocation
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(Self {
            request: DomHostedChildRequest {
                operation: operation.id,
                occurrence,
                basis,
                container,
                document: DomDocumentStamp {
                    store: basis,
                    document: node.document,
                },
                parent,
                parent_global: association.global,
                traversable: parent_row.traversable,
                host: association.host,
            },
            hook,
            phase: if matches!(hook, ChildHook::Remove { .. }) {
                Phase::PrepareRemoval
            } else {
                Phase::Prepare
            },
            document: None,
            declared_sandbox: None,
            url: None,
            url_charge: None,
            observation: None,
            prefix: None,
            work_charge: None,
            phase_charge: None,
            handoff_charge: None,
        })
    }
    fn check(&self, request: &DomHostedChildRequest) -> Result<(), DomHostedError> {
        if &self.request != request {
            return Err(DomHostedError::WrongPhase);
        }
        Ok(())
    }
    fn creating(&self) -> Result<&canonical::Operation, DomHostedError> {
        match &self.phase {
            Phase::Creating(op) => Ok(op),
            _ => Err(DomHostedError::WrongPhase),
        }
    }
    fn work(&self) -> Result<&Arc<Work>, DomHostedError> {
        match &self.phase {
            Phase::Creating(op) => Ok(&op.request().work),
            Phase::Queue(request) => Ok(&request.packet.request.work),
            Phase::Attributes { work, .. } => Ok(work),
            Phase::Removing(row) => Ok(&row.packet.request.work),
            Phase::RemovalPublished(row) => Ok(&row.work),
            _ => Err(DomHostedError::WrongPhase),
        }
    }
    fn step(&self) -> Result<DomHostedChildStep<'_>, DomHostedError> {
        use canonical::CreationPhase as P;
        Ok(match &self.phase {
            Phase::Prepare => DomHostedChildStep::Prepare(&self.request),
            Phase::PrepareRemoval => DomHostedChildStep::PrepareRemoval(&self.request),
            Phase::Removing(row) => DomHostedChildStep::Removal(&row.packet),
            Phase::RemovalPublished(row) => DomHostedChildStep::Removal(&row.packet),
            Phase::Creating(op) => match &op.phase {
                P::Policy(request) => DomHostedChildStep::PermissionsPolicy(request),
                P::Execution(request) => DomHostedChildStep::NativeExecution(request),
                P::MakeActive(request) => DomHostedChildStep::MakeActive(request),
                P::FinishLoading(request) => DomHostedChildStep::FinishLoading(request),
                P::Entry(request) => DomHostedChildStep::InitialEntry(request),
                _ => return Err(DomHostedError::WrongPhase),
            },
            Phase::Queue(request) => DomHostedChildStep::QueueHistory(&request.packet),
            Phase::Attributes { .. } => {
                DomHostedChildStep::NeedsUrl(crate::DomSourceUrlStage::FrameIframeSrc)
            }
            Phase::Returned => DomHostedChildStep::Returned,
        })
    }
}
impl DomSourceLease<'_> {
    /// Run only the actual synchronous attribute tail of this creator. Other
    /// phases are observations of the same retained cursor and are untouched.
    pub fn process_hosted_child_attributes(
        &mut self,
        request: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let operation = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomHostedError::WrongPhase)?;
        let child = operation.child.as_ref().ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        let Phase::Attributes { .. } = &child.phase else {
            return Ok(());
        };
        operation.draft.source_step(cancel)?;
        let key = child.hook.key();
        let iframe = operation
            .draft
            .element(key)?
            .iframe_initial_state()
            .is_some();
        if iframe
            && surgeist_html::iframe_attribute_branch(
                operation
                    .draft
                    .control_attribute(key, KnownHtmlAttribute::Srcdoc, cancel)?
                    .is_some(),
            ) == surgeist_html::HtmlIframeAttributeBranch::Srcdoc
        {
            // The actual srcdoc navigation branch requires its own execution
            // owner. It is not the shared-src initial ancestor-return path.
            return Err(DomHostedError::UnsupportedProfile);
        }
        let document = operation.draft.node(key)?.document;
        let DomNodeKind::Document(metadata) = &operation.draft.node(document)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        association.validate_live()?;
        let DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let raw = operation
            .draft
            .control_attribute(key, KnownHtmlAttribute::Src, cancel)?;
        let candidate = match surgeist_html::bounded::frame_iframe_src_with(raw, || {
            operation.draft.source_step(cancel)
        })? {
            surgeist_html::HtmlFrameIframeSrcDecision::AboutBlank => None,
            surgeist_html::HtmlFrameIframeSrcDecision::ParseSrc(_) => {
                let Some(response) = child.observation.as_ref().filter(|value| {
                    value.value.request().stage() == crate::DomSourceUrlStage::FrameIframeSrc
                }) else {
                    return Ok(());
                };
                match response.value.observation().outcome() {
                    HtmlBaseParseOutcome::Failure => None,
                    HtmlBaseParseOutcome::Parsed { record, .. } => Some(record),
                }
            }
        };
        let child_key = child.document.ok_or(DomHostedError::WrongPhase)?;
        let DomNodeKind::Document(child_metadata) = &operation.draft.node(child_key)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let initial = child_metadata
            .hosted_inputs
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        let candidate = candidate.unwrap_or_else(|| initial.url());
        let graph = operation
            .draft
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        let mut current = Some(navigable);
        let mut matched = false;
        while let Some(id) = current {
            operation.draft.source_step(cancel)?;
            let nav = graph.navigable_with(id, || {
                operation.draft.source_step(cancel).map_err(Into::into)
            })?;
            if nav.retired {
                return Err(DomHostedError::StaleIdentity);
            }
            let active = graph
                .entry_with(nav.active, || {
                    operation.draft.source_step(cancel).map_err(Into::into)
                })?
                .document
                .ok_or(DomHostedError::StaleIdentity)?;
            let active_key = Key::Existing(active.node().slot);
            let DomNodeKind::Document(active_metadata) = &operation.draft.node(active_key)?.kind
            else {
                return Err(DomError::WrongDocument.into());
            };
            let active_url = active_metadata
                .hosted_inputs
                .as_ref()
                .ok_or(DomHostedError::UnsupportedProfile)?
                .url();
            if candidate.equals_excluding_fragments_with(active_url, || {
                operation.draft.source_step(cancel)
            })? {
                matched = true;
                break;
            }
            current = nav.parent;
        }
        let action =
            surgeist_html::bounded::frame_iframe_after_ancestors_with(matched, true, || {
                operation.draft.source_step(cancel)?;
                Ok::<_, DomError>(candidate.matches_about_blank())
            })?;
        match action {
            surgeist_html::HtmlFrameIframeAfterAncestors::ReturnNull => {
                // No URL/history update, initial load, lazy step or navigation
                // runs after the actual inclusive-ancestor early return.
                let child = operation.child.as_mut().expect("validated original child");
                child.phase = Phase::Returned;
                child.url = None;
                child.url_charge = None;
                child.observation = None;
                Ok(())
            }
            // This bounded initial profile has no navigation execution owner;
            // retain the actual reached tail and its original response/Work.
            surgeist_html::HtmlFrameIframeAfterAncestors::UpdateUrlAndHistory
            | surgeist_html::HtmlFrameIframeAfterAncestors::ReturnUrl => {
                Err(DomHostedError::UnsupportedProfile)
            }
        }
    }
    pub fn hosted_child_step(
        &self,
        request: &DomHostedChildRequest,
    ) -> Result<DomHostedChildStep<'_>, DomHostedError> {
        let op = self
            .dom
            .source_operation
            .as_ref()
            .ok_or(DomHostedError::WrongPhase)?;
        let child = op.child.as_ref().ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        child.step()
    }
    pub fn prepare_hosted_child(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedChildPreparation, DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let operation = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomHostedError::WrongPhase)?;
        let child = operation.child.as_ref().ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        if !matches!(child.phase, Phase::Prepare) {
            return Err(DomHostedError::WrongPhase);
        }
        if let Some(prefix) = &child.prefix {
            prefix.work.step(cancel)?;
            return Ok(DomHostedChildPreparation {
                request: request.clone(),
                host: prefix.host,
                context: prefix.context,
                work: prefix.work.clone(),
            });
        }
        let formation = formation::Formation::new(&operation.draft, cancel)?;
        if let Some(formation) = &formation {
            formation.retain_bytes(size_of::<Work>() + 2 * size_of::<AtomicU64>(), cancel)?;
        }
        let prepared = source.prepare_child_prefix(request, cancel)?;
        let child = operation
            .child
            .as_mut()
            .expect("validated original prefix owner");
        child.prefix = Some(ChildPrefix {
            host: prepared.host,
            context: prepared.context,
            work: prepared.work.clone(),
        });
        child.work_charge = formation.map(formation::Formation::into_charge);
        Ok(prepared)
    }
    pub fn accept_hosted_child_preparation(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        prepared: &mut Option<DomHostedPreparedChild>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomHostedError::WrongPhase)?;
        let child = op.child.as_ref().ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        if !matches!(child.phase, Phase::Prepare) {
            return Err(DomHostedError::WrongPhase);
        }
        let prefix = &prepared
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?
            .preparation;
        if &prefix.request != request {
            return Err(DomHostedError::WrongPhase);
        }
        prefix.work.step(cancel)?;
        let original = child.prefix.as_ref().ok_or(DomHostedError::WrongPhase)?;
        if original.context != prefix.context || !Arc::ptr_eq(&original.work, &prefix.work) {
            return Err(DomHostedError::WrongPhase);
        }
        let key = child.hook.key();
        let document = op.draft.node(key)?.document;
        let DomNodeKind::Document(parent) = &op.draft.node(document)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let parent_inputs = parent
            .hosted_inputs
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        let parent_sandbox = parent_inputs.0.active_sandbox;
        let parent_environment = parent.document_environment();
        let parent_association = parent_environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        parent_association.validate_live()?;
        let kind = if op.draft.element(key)?.iframe_initial_state().is_some() {
            DomHostedEmbedderKind::Iframe
        } else {
            DomHostedEmbedderKind::Frame
        };
        let own_sandbox = if kind == DomHostedEmbedderKind::Frame {
            surgeist_html::HtmlSandboxFlags::empty()
        } else {
            op.draft
                .element(key)?
                .iframe_sandbox_flags()
                .expect("actual Iframe flag set")
        };
        let sandbox = op
            .draft
            .control_attribute(key, KnownHtmlAttribute::Sandbox, cancel)?;
        let srcdoc = op
            .draft
            .control_attribute(key, KnownHtmlAttribute::Srcdoc, cancel)?;
        let src = op
            .draft
            .control_attribute(key, KnownHtmlAttribute::Src, cancel)?;
        let allow = op
            .draft
            .control_attribute(key, KnownHtmlAttribute::Allow, cancel)?;
        let allow_fullscreen = op
            .draft
            .control_attribute(key, KnownHtmlAttribute::Allowfullscreen, cancel)?
            .is_some();
        let mut payload = storage::Payload::default();
        payload.add_bytes(size_of::<DomHostedEmbedderPolicyInputs>())?;
        for value in [sandbox, srcdoc, src, allow].into_iter().flatten() {
            prefix.work.step(cancel)?;
            payload.string(value)?;
        }
        let phase_formation = formation::Formation::new(&op.draft, cancel)?;
        if let Some(formation) = &phase_formation {
            formation.retain_payload(payload, cancel)?;
        }
        let retained = source.retain(payload.bytes, cancel)?;
        let inputs = Arc::new(DomHostedEmbedderPolicyInputs {
            kind,
            parent_document: request.document,
            parent_origin: parent.origin().clone(),
            parent_fullscreen: parent_association.policy.inherited,
            parent_sandbox,
            embedder_sandbox: own_sandbox,
            sandbox: sandbox.cloned(),
            srcdoc: srcdoc.cloned(),
            src: src.cloned(),
            allow: allow.cloned(),
            allow_fullscreen,
            _retained: retained,
        });
        let origin = if own_sandbox
            .union(parent_sandbox)
            .contains(HtmlSandboxFlag::Origin)
        {
            DomDocumentMetadata::fresh_opaque_origin()
        } else {
            parent.origin().clone()
        };
        let retained = source.retain(size_of::<canonical::Operation>(), cancel)?;
        let id = source.begin_operation(&prefix.work, 10, 0, cancel)?;
        let prefix = prepared
            .take()
            .expect("validated original child prefix")
            .preparation;
        let inner = Request {
            key: RequestKey {
                operation: id,
                phase: 0,
            },
            host: prefix.host,
            context: prefix.context,
            work: prefix.work,
            parent: Some(ParentContext {
                document: request.document,
                navigable: request.parent,
                global: request.parent_global,
                container: request.container,
                traversable: request.traversable,
            }),
        };
        op.child.as_mut().expect("retained exact child").phase =
            Phase::Creating(canonical::Operation {
                origin: origin.clone(),
                policy: None,
                phase: canonical::CreationPhase::Policy(DomHostedPolicyRequest {
                    request: inner,
                    origin,
                    embedder_policy: Some(inputs),
                }),
                _retained: retained,
            });
        let child = op.child.as_mut().expect("accepted same original prefix");
        child.prefix = None;
        child.phase_charge = phase_formation.map(formation::Formation::into_charge);
        Ok(())
    }
    pub fn declared_sandbox_origin(
        &mut self,
        request: &DomHostedChildRequest,
        source: &mut DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomOpaqueOrigin, DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let child = self
            .dom
            .source_operation
            .as_mut()
            .and_then(|op| op.child.as_mut())
            .ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        let op = child.creating()?;
        if !matches!(op.phase, canonical::CreationPhase::Policy(_)) {
            return Err(DomHostedError::WrongPhase);
        }
        op.request().work.step(cancel)?;
        if child.declared_sandbox.is_none() {
            child.declared_sandbox = Some(DomDocumentMetadata::fresh_opaque_origin());
        }
        Ok(child
            .declared_sandbox
            .as_ref()
            .expect("reached source branch retained")
            .clone())
    }
    pub fn source_url_observation(
        &self,
        request: &DomHostedChildRequest,
        stage: crate::DomSourceUrlStage,
    ) -> Result<Option<&crate::DomSourceUrlObservation>, DomError> {
        let child = self
            .dom
            .source_operation
            .as_ref()
            .and_then(|op| op.child.as_ref())
            .ok_or(DomError::SourceUrlInputsMismatch)?;
        if &child.request != request {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        Ok(child
            .observation
            .as_ref()
            .map(|accepted| &accepted.value)
            .filter(|value| value.request().stage() == stage))
    }
}

impl Draft {
    fn hosted_base_record(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<&surgeist_html_semantics::HtmlBaseUrlRecord, DomError> {
        self.source_step(cancel)?;
        let node = self.node(document)?;
        let DomNodeKind::Document(metadata) = &node.kind else {
            return Err(DomError::WrongDocument);
        };
        let facts = metadata
            .hosted_inputs
            .as_ref()
            .map(|inputs| &inputs.0.facts);
        let first = match node
            .base_document
            .as_ref()
            .ok_or(DomError::WrongDocument)?
            .first
        {
            None => surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
            Some(key) => {
                self.source_step(cancel)?;
                surgeist_html_semantics::HtmlDocumentFirstBase::Present(
                    self.node(key)?
                        .base
                        .as_ref()
                        .and_then(|state| state.frozen.as_ref())
                        .map(|frozen| &frozen.value),
                )
            }
        };
        surgeist_html_semantics::select_document_base(facts, first)
            .map_err(|missing| DomError::BaseFacts(crate::DomBaseFactsError::Missing(missing)))
    }
}
impl DomSourceLease<'_> {
    /// Qualify retained, unpublished URL evidence against the current service.
    /// A returned stage needs service before it may advance. Without a service,
    /// retain its request and admitted response; with a different configuration,
    /// retire only that stage's request, response and their storage charges.
    /// The caller must discard any unpublished result derived from retired
    /// evidence before resuming the same stage. Completed phases are unchanged.
    pub fn qualify_source_url_configuration(
        &mut self,
        request: &DomHostedChildRequest,
        expected: Option<&HtmlBaseFactsRevision>,
        cancel: &AtomicBool,
    ) -> Result<Option<crate::DomSourceUrlStage>, DomError> {
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomError::SourceUrlInputsMismatch)?;
        let child = op.child.as_ref().ok_or(DomError::SourceUrlInputsMismatch)?;
        if &child.request != request {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        let Some(issued) = child.url.as_ref() else {
            return Ok(None);
        };
        let stage = issued.stage();
        match (&child.phase, stage) {
            (Phase::Creating(inner), crate::DomSourceUrlStage::IframeDeclaredOrigin)
                if matches!(inner.phase, canonical::CreationPhase::Policy(_)) => {}
            (Phase::Attributes { .. }, crate::DomSourceUrlStage::FrameIframeSrc) => {}
            _ => return Err(DomError::SourceUrlInputsMismatch),
        }
        op.draft.source_step(cancel)?;
        let Some(expected) = expected else {
            return Ok(Some(stage));
        };
        if issued.revision() == expected
            && issued.document().store()
                == (DomStamp {
                    owner: self.dom.owner,
                    revision: self.dom.revision,
                })
        {
            return Ok(None);
        }
        // This operation, phase, Work and every accepted construction prefix
        // remain the original values. Drop values before releasing their debit.
        let child = op.child.as_mut().expect("validated original child");
        child.observation = None;
        child.url = None;
        child.url_charge = None;
        Ok(Some(stage))
    }

    pub fn source_url_request(
        &mut self,
        request: &DomHostedChildRequest,
        stage: crate::DomSourceUrlStage,
        expected: &HtmlBaseFactsRevision,
        cancel: &AtomicBool,
    ) -> Result<crate::DomSourceUrlRequest, DomError> {
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomError::SourceUrlInputsMismatch)?;
        let child = op.child.as_ref().ok_or(DomError::SourceUrlInputsMismatch)?;
        if &child.request != request {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        child
            .work()
            .map_err(|_| DomError::SourceUrlInputsMismatch)?;
        op.draft.source_step(cancel)?;
        let target = match (&child.phase, stage) {
            (Phase::Creating(inner), crate::DomSourceUrlStage::IframeDeclaredOrigin) => {
                let canonical::CreationPhase::Policy(policy) = &inner.phase else {
                    return Err(DomError::SourceUrlInputsMismatch);
                };
                if policy
                    .embedder_policy
                    .as_ref()
                    .is_none_or(|inputs| inputs.kind != DomHostedEmbedderKind::Iframe)
                {
                    return Err(DomError::SourceUrlInputsMismatch);
                }
                Some(&policy.origin)
            }
            (Phase::Attributes { .. }, crate::DomSourceUrlStage::FrameIframeSrc) => None,
            _ => return Err(DomError::SourceUrlInputsMismatch),
        };
        if let Some(issued) = &child.url {
            if issued.stage() == stage
                && issued.revision() == expected
                && issued.document().store()
                    == (DomStamp {
                        owner: self.dom.owner,
                        revision: self.dom.revision,
                    })
            {
                return Ok(issued.clone());
            }
            // Retained admitted evidence must first cross the explicit current
            // configuration qualifier so its consumer can retire derived state.
            // A same-stage failed request formation keeps its prior custody.
            if child.observation.is_some() {
                return Err(DomError::SourceUrlInputsMismatch);
            }
        }
        let key = child.hook.key();
        let document = op.draft.node(key)?.document;
        let DomNodeKind::Document(metadata) = &op.draft.node(document)?.kind else {
            return Err(DomError::WrongDocument);
        };
        let inputs = metadata
            .hosted_inputs
            .as_ref()
            .ok_or(DomError::SourceUrlUnavailable)?;
        let encoding = inputs
            .0
            .facts
            .encoding()
            .ok_or(DomError::SourceUrlUnavailable)?;
        let base = op.draft.hosted_base_record(document, cancel)?;
        let raw = op
            .draft
            .control_attribute(key, KnownHtmlAttribute::Src, cancel)?
            .ok_or(DomError::SourceUrlInputsMismatch)?;
        let formation =
            formation::Formation::new(&op.draft, cancel)?.ok_or(DomError::SourceUrlUnavailable)?;
        let mut payload = storage::Payload::default();
        payload.add_bytes(size_of::<crate::source::UrlBasis>())?;
        payload.string(raw)?;
        payload.add_bytes(base.retained_storage_usage().retained_bytes())?;
        payload.add_units(base.retained_storage_usage().utf16_extent())?;
        formation.retain_payload(payload, cancel)?;
        let occurrence = op
            .draft
            .source_invocation
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let issued = crate::DomSourceUrlRequest {
            basis: Arc::new(crate::source::UrlBasis {
                child: request.clone(),
                occurrence,
                document: DomDocumentStamp {
                    store: DomStamp {
                        owner: self.dom.owner,
                        revision: self.dom.revision,
                    },
                    document: request.document.document(),
                },
                stage,
                raw: raw.clone(),
                base: base.clone(),
                encoding: encoding.clone(),
                revision: expected.clone(),
                target: target.cloned(),
            }),
        };
        let charge = formation.into_charge();
        let child = op.child.as_mut().expect("validated same original child");
        child.url = Some(issued.clone());
        child.url_charge = Some(charge);
        op.draft.source_invocation = occurrence;
        Ok(issued)
    }
    pub fn accept_source_url_observation(
        &mut self,
        request: &DomHostedChildRequest,
        issued: &crate::DomSourceUrlRequest,
        observation: &mut Option<crate::DomSourceUrlObservation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomError::SourceUrlInputsMismatch)?;
        let child = op.child.as_ref().ok_or(DomError::SourceUrlInputsMismatch)?;
        if &child.request != request
            || !child
                .url
                .as_ref()
                .is_some_and(|old| Arc::ptr_eq(&old.basis, &issued.basis))
        {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        if issued.document().store()
            != (DomStamp {
                owner: self.dom.owner,
                revision: self.dom.revision,
            })
        {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        let candidate = observation.as_ref().ok_or(DomError::SourceUrlUnavailable)?;
        if !Arc::ptr_eq(&candidate.request.basis, &issued.basis) || child.observation.is_some() {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        op.draft.source_step(cancel)?;
        if !candidate.observation.inputs_match(
            issued.base_url(),
            issued.encoding(),
            issued.revision(),
        ) {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        op.draft.source_step(cancel)?;
        if candidate.observation.raw().len() != issued.raw().len() {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        for (left, right) in candidate
            .observation
            .raw()
            .code_units()
            .iter()
            .zip(issued.raw().code_units())
        {
            op.draft.source_step(cancel)?;
            if left != right {
                return Err(DomError::SourceUrlInputsMismatch);
            }
        }
        // Hidden fallback extent is qualified by inputs_match; the actual raw
        // response capacity and Parsed record remain independently measured.
        let mut payload = storage::Payload::default();
        payload.add_bytes(size_of::<crate::DomSourceUrlObservation>())?;
        payload.string(candidate.observation.raw())?;
        payload.add_bytes(issued.base_url().retained_storage_usage().retained_bytes())?;
        payload.add_units(issued.base_url().retained_storage_usage().utf16_extent())?;
        if let HtmlBaseParseOutcome::Parsed { record, .. } = candidate.observation.outcome() {
            payload.add_bytes(record.retained_storage_usage().retained_bytes())?;
            payload.add_units(record.retained_storage_usage().utf16_extent())?;
        }
        let formation =
            formation::Formation::new(&op.draft, cancel)?.ok_or(DomError::SourceUrlUnavailable)?;
        formation.retain_payload(payload, cancel)?;
        check_cancel(cancel)?;
        let value = observation
            .take()
            .expect("validated original response remains owned until admission");
        op.child
            .as_mut()
            .expect("validated original child")
            .observation = Some(AcceptedUrl {
            value,
            _charge: formation.into_charge(),
        });
        Ok(())
    }
}

impl DomSourceLease<'_> {
    pub fn accept_hosted_child_policy(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        publication: &mut Option<DomHostedPolicyPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomHostedError::WrongPhase)?;
        let child = op.child.as_ref().ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        let inner = child.creating()?;
        let canonical::CreationPhase::Policy(policy) = &inner.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let current = publication
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        if current.key != inner.request().key {
            return Err(DomHostedError::WrongPhase);
        }
        let key = child.hook.key();
        let document = op.draft.node(key)?.document;
        let DomNodeKind::Document(parent) = &op.draft.node(document)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let base = op.draft.hosted_base_record(document, cancel)?;
        let graph = op
            .draft
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        let top = graph.navigable_with(request.traversable, || {
            op.draft.source_step(cancel).map_err(Into::into)
        })?;
        let top_document = graph
            .entry_with(top.active, || {
                op.draft.source_step(cancel).map_err(Into::into)
            })?
            .document
            .ok_or(DomHostedError::StaleIdentity)?;
        let top_key = op.draft.resolve(top_document.node().into())?;
        let DomNodeKind::Document(top_metadata) = &op.draft.node(top_key)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let top_policy = &top_metadata
            .hosted_inputs
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?
            .0
            .opener;
        let embedder = policy
            .embedder_policy
            .as_ref()
            .ok_or(DomHostedError::WrongPhase)?;
        let phase_formation = formation::Formation::new(&op.draft, cancel)?;
        let inputs = crate::document::HostedDocumentInputs::form_child(
            source,
            &inner.request().work,
            canonical::ChildConstructor {
                parent,
                creator_base: base,
                top_policy,
                sandbox: embedder.embedder_sandbox.union(embedder.parent_sandbox),
            },
            phase_formation.as_ref(),
            cancel,
        )?;
        let next = inner.next_request()?;
        let permit = source.permit(inner.request(), current.revision, false, cancel)?;
        let Phase::Creating(inner) = &mut op.child.as_mut().expect("validated child").phase else {
            unreachable!("exact phase retained")
        };
        inner.policy = Some(DomInitialPermissionsPolicy {
            inherited: current.inherited,
        });
        inner.phase = canonical::CreationPhase::Execution(DomHostedExecutionRequest {
            request: next,
            origin: inner.origin.clone(),
            inputs,
        });
        permit.commit();
        publication.take();
        // A PP observation no longer influences later family src parsing. The
        // later reached stage must issue a new request under its own basis.
        let child = op.child.as_mut().expect("validated child");
        child.phase_charge = phase_formation.map(formation::Formation::into_charge);
        child.observation = None;
        child.url = None;
        child.url_charge = None;
        Ok(())
    }
}
impl Draft {
    fn child_document(
        &mut self,
        source: &DomHostedSource,
        inner: &canonical::Operation,
        global: DomHostedGlobalId,
        navigation_start: DomHostedMoment,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomHostedError> {
        let canonical::CreationPhase::Execution(execution) = &inner.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let inputs = &execution.inputs;
        let request = inner.request();
        let formation = formation::Formation::new(self, cancel)?;
        let (registries, registry) = registry::Bank::form(
            self.registries.as_deref(),
            self.store,
            source,
            &request.work,
            formation.as_ref(),
            cancel,
        )?;
        if let Some(formation) = &formation {
            formation.retain_bytes(size_of::<DocumentAssociation>(), cancel)?;
        }
        let retained = source.retain(size_of::<DocumentAssociation>(), cancel)?;
        let association = Arc::new(DocumentAssociation {
            source: source.observe(),
            host: request.host,
            context: request.context,
            global,
            navigation_start,
            policy: inner.policy.ok_or(DomHostedError::WrongPhase)?,
            phase: DocumentPhase::Constructed,
            _retained: retained,
        });
        if let Some(budget) = &formation {
            let mut payload = storage::Payload::default();
            payload.add_bytes(inputs.payload_bytes()?)?;
            payload.add_units(inputs.payload_units()?)?;
            payload.string(inputs.url().serialization())?;
            budget.retain_payload(payload, cancel)?;
        }
        let parent = request.parent.ok_or(DomHostedError::WrongPhase)?;
        self.source_step(cancel)?;
        let parent_graph = self
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::WrongPhase)?;
        let parent_nav = parent_graph.navigable_with(parent.navigable, || {
            self.source_step(cancel).map_err(Into::into)
        })?;
        if parent_nav.traversable != parent.traversable
            || parent_graph
                .entry_with(parent_nav.active, || {
                    self.source_step(cancel).map_err(Into::into)
                })?
                .document
                != Some(parent.document.document())
            || parent_nav.global != parent.global
        {
            return Err(DomHostedError::StaleBasis);
        }
        let parent_key = self.resolve(parent.document.document().node().into())?;
        let DomNodeKind::Document(parent_metadata) = &self.node(parent_key)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let platform = parent_metadata.document_environment().input_platform();
        let parent_policy = *parent_metadata.policy_container();
        let mut metadata = DomDocumentMetadata::hosted_initial(
            platform,
            inner.origin.clone(),
            registry,
            association,
            request.key.operation,
            inputs.clone(),
            &mut |text| formation::string(text, formation.as_ref(), cancel),
        )?;
        *metadata.policy_container_mut() = parent_policy;
        let document = Key::Created(self.created.len());
        let registry_payload = registries.retained_payload_bytes(|| self.source_step(cancel))?;
        self.registries = Some(registries);
        if let Some(formation) = &formation {
            formation.release_payload(storage::Payload {
                bytes: registry_payload,
                units: 0,
            });
        }
        let old_owner = self.hosted_owner.replace(request.key.operation);
        let result = self.with_effect_scope(|draft| {
            let document =
                draft.allocate(DomNodeKind::Document(metadata), document, limits, cancel)?;
            drop(formation);
            let html = draft.html_skeleton_element(document, "html", limits, cancel)?;
            draft.select_element_registry(html, cancel)?;
            draft.pre_insert(html, document, None, limits, cancel)?;
            let head = draft.html_skeleton_element(document, "head", limits, cancel)?;
            draft.select_element_registry(head, cancel)?;
            draft.pre_insert(head, html, None, limits, cancel)?;
            let body = draft.html_skeleton_element(document, "body", limits, cancel)?;
            draft.select_element_registry(body, cancel)?;
            draft.pre_insert(body, html, None, limits, cancel)?;
            Ok(document)
        });
        self.hosted_owner = old_owner;
        result
    }
    fn child_document_phase(
        &mut self,
        source: &DomHostedSource,
        document: Key,
        phase: DocumentPhase,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let old = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::WrongPhase)?;
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
        let DomNodeKind::Document(metadata) = &mut self.node_mut(document).kind else {
            unreachable!("validated actual Document")
        };
        metadata.set_hosted_association(association);
        Ok(())
    }
}

impl DomSourceLease<'_> {
    pub fn accept_hosted_child_publication(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        publication: &mut Option<DomHostedCreationPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomHostedError::WrongPhase)?;
        let Some(mut child) = operation.child.take() else {
            self.dom.source_operation = Some(operation);
            return Err(DomHostedError::WrongPhase);
        };
        let result = (|| {
            child.check(request)?;
            let inner = child.creating()?;
            let current = publication
                .as_ref()
                .ok_or(DomHostedError::MissingPublication)?;
            if current.key != inner.request().key {
                return Err(DomHostedError::WrongPhase);
            }
            let next = inner.next_request()?;
            let checkpoint =
                operation
                    .draft
                    .source_checkpoint(operation.input_bytes()?, 0, cancel)?;
            enum Next {
                Document {
                    key: Key,
                    global: DomHostedGlobalId,
                },
                Active {
                    document: DocumentId,
                    global: DomHostedGlobalId,
                    start: DomHostedMoment,
                },
                Loaded {
                    document: DocumentId,
                    global: DomHostedGlobalId,
                },
                Queue(DomHostedCreation),
            }
            let attempt = (|| {
                use canonical::CreationPhase as P;
                let next_phase = match (&inner.phase, &current.value) {
                    (
                        P::Execution(_),
                        PublicationValue::NativeExecution {
                            global,
                            navigation_start,
                        },
                    ) => {
                        let graph = operation
                            .draft
                            .hosted_history
                            .as_deref()
                            .ok_or(DomHostedError::UnsupportedProfile)?;
                        let parent = graph.navigable_with(request.parent, || {
                            operation.draft.source_step(cancel).map_err(Into::into)
                        })?;
                        if parent.retiring || parent.retired {
                            return Err(DomHostedError::StaleIdentity);
                        }
                        let key = operation.draft.child_document(
                            source,
                            inner,
                            *global,
                            *navigation_start,
                            self.dom.limits,
                            cancel,
                        )?;
                        operation.hosted_global =
                            Some((key, *global, inner.request().work.clone()));
                        Next::Document {
                            key,
                            global: *global,
                        }
                    }
                    (P::MakeActive(active), PublicationValue::MakeActive) => {
                        let key = child.document.ok_or(DomHostedError::WrongPhase)?;
                        let DomNodeKind::Document(metadata) = &operation.draft.node(key)?.kind
                        else {
                            return Err(DomError::WrongDocument.into());
                        };
                        let start = metadata
                            .document_environment()
                            .hosted
                            .as_ref()
                            .ok_or(DomHostedError::WrongPhase)?
                            .navigation_start;
                        operation.draft.child_document_phase(
                            source,
                            key,
                            DocumentPhase::Active,
                            cancel,
                        )?;
                        Next::Active {
                            document: active.document,
                            global: active.global,
                            start,
                        }
                    }
                    (P::FinishLoading(loading), PublicationValue::FinishLoading(moment)) => {
                        operation.draft.child_document_phase(
                            source,
                            child.document.ok_or(DomHostedError::WrongPhase)?,
                            DocumentPhase::Loaded(*moment),
                            cancel,
                        )?;
                        Next::Loaded {
                            document: loading.document,
                            global: loading.global,
                        }
                    }
                    (
                        P::Entry(entry),
                        PublicationValue::InitialEntry {
                            entry: entry_id,
                            navigable,
                            initialization,
                            ..
                        },
                    ) => {
                        let key = child.document.ok_or(DomHostedError::WrongPhase)?;
                        let DomNodeKind::Document(metadata) = &operation.draft.node(key)?.kind
                        else {
                            return Err(DomError::WrongDocument.into());
                        };
                        let DocumentPhase::Loaded(loaded) = metadata
                            .document_environment()
                            .hosted
                            .as_ref()
                            .ok_or(DomHostedError::WrongPhase)?
                            .phase
                        else {
                            return Err(DomHostedError::WrongPhase);
                        };
                        let creation = DomHostedCreation {
                            document: entry.document,
                            host: inner.request().host,
                            context: inner.request().context,
                            global: entry.global,
                            entry: *entry_id,
                            navigable: *navigable,
                        };
                        for _ in 0..72 {
                            inner.request().work.step(cancel)?;
                        }
                        // The source samples name only after BC and Document
                        // creation, at the reached initialize-navigable step.
                        let target_name = operation.draft.control_attribute(
                            child.hook.key(),
                            KnownHtmlAttribute::Name,
                            cancel,
                        )?;
                        let facts = metadata
                            .hosted_inputs
                            .as_ref()
                            .ok_or(DomHostedError::UnsupportedProfile)?;
                        let fresh = formation::Formation::new(&operation.draft, cancel)?;
                        let context = canonical::history::Context::new(
                            source,
                            &inner.request().work,
                            cancel,
                            0,
                        );
                        let context = if let Some(fresh) = &fresh {
                            context.with_formation(fresh)
                        } else {
                            context
                        };
                        let graph = canonical::history::Graph::initial(
                            operation.draft.hosted_history.as_deref(),
                            canonical::history::Initial {
                                creation,
                                initialization: DomHostedEntryInitialization::new(
                                    initialization.navigation_key,
                                    initialization.navigation_id,
                                ),
                                origin: inner.origin.clone(),
                                parent: Some((request.parent, request.container)),
                                target_name,
                                about_base_url: facts.0.facts.about_base_url(),
                                initiator_origin: Some(&inner.origin),
                            },
                            &context,
                        )?;
                        operation.draft.hosted_history = Some(graph);
                        // Draft storage now owns this exact occurrence; the
                        // following publication recomputes that baseline.
                        drop(fresh);
                        operation.draft.child_document_phase(
                            source,
                            key,
                            DocumentPhase::InitialEntry {
                                entry: *entry_id,
                                navigable: *navigable,
                                loaded,
                            },
                            cancel,
                        )?;
                        operation
                            .draft
                            .element_mut(child.hook.key())
                            .establish_child_navigable(Some(*navigable))?;
                        Next::Queue(creation)
                    }
                    _ => return Err(DomHostedError::WrongPhase),
                };
                operation.publication_with_hosted(
                    self.dom,
                    true,
                    super::PublicationTarget::None,
                    Some((source, inner.request(), current.revision)),
                    cancel,
                )?;
                Ok(next_phase)
            })();
            let next_phase = match attempt {
                Ok(next) => next,
                Err(error) => {
                    checkpoint.restore(&mut operation.draft);
                    operation.hosted_global = None;
                    return Err(error);
                }
            };
            match next_phase {
                Next::Document { key, global } => {
                    let document = DocumentId(
                        operation
                            .draft
                            .published_id(key)
                            .expect("validated source allocation published"),
                    );
                    child.document = Some(key);
                    let Phase::Creating(inner) = &mut child.phase else {
                        unreachable!("validated exact creator")
                    };
                    inner.phase =
                        canonical::CreationPhase::MakeActive(DomHostedMakeActiveRequest {
                            request: next,
                            document,
                            global,
                        });
                    // Drop the private request occurrence before its debit;
                    // published metadata already owns its independent extent.
                    child.phase_charge = None;
                }
                Next::Active {
                    document,
                    global,
                    start,
                } => {
                    let Phase::Creating(inner) = &mut child.phase else {
                        unreachable!("validated exact creator")
                    };
                    inner.phase =
                        canonical::CreationPhase::FinishLoading(DomHostedFinishLoadingRequest {
                            request: next,
                            document,
                            global,
                            navigation_start: start,
                        });
                }
                Next::Loaded { document, global } => {
                    let Phase::Creating(inner) = &mut child.phase else {
                        unreachable!("validated exact creator")
                    };
                    inner.phase = canonical::CreationPhase::Entry(DomHostedEntryRequest {
                        request: next,
                        document,
                        global,
                    });
                }
                Next::Queue(creation) => {
                    let Phase::Creating(inner) =
                        std::mem::replace(&mut child.phase, Phase::Returned)
                    else {
                        unreachable!("validated exact creator")
                    };
                    child.phase = Phase::Queue(QueuedChild {
                        packet: DomHostedChildHistoryRequest {
                            request: next,
                            parent: request.parent,
                            traversable: request.traversable,
                            creation,
                        },
                        origin: inner.origin,
                        policy: inner.policy,
                        retained: inner._retained,
                    });
                }
            }
            publication.take();
            Ok(())
        })();
        operation.child = Some(child);
        self.dom.source_operation = Some(operation);
        result
    }
}

impl DomSourceLease<'_> {
    pub fn accept_hosted_child_history(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        publication: &mut Option<DomHostedChildHistoryPublication>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedQueuedHistory, DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomHostedError::WrongPhase)?;
        let Some(mut child) = operation.child.take() else {
            self.dom.source_operation = Some(operation);
            return Err(DomHostedError::WrongPhase);
        };
        let result = (|| {
            child.check(request)?;
            let Phase::Queue(queued) = &child.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            let packet = publication
                .as_ref()
                .ok_or(DomHostedError::MissingPublication)?;
            if packet.key != queued.packet.request.key {
                return Err(DomHostedError::WrongPhase);
            }
            let original = &queued.packet.request;
            let next = Request {
                key: RequestKey {
                    operation: original.key.operation,
                    phase: original
                        .key
                        .phase
                        .checked_add(1)
                        .ok_or(DomHostedError::RepresentationExhausted)?,
                },
                host: original.host,
                context: original.context,
                work: original.work.clone(),
                parent: original.parent,
            };
            let handoff_formation = formation::Formation::new(&operation.draft, cancel)?;
            if let Some(formation) = &handoff_formation {
                formation.retain_bytes(size_of::<canonical::Operation>(), cancel)?;
            }
            let bank = self.dom.hosted_operations.prepare(
                original.key.operation.slot,
                &original.work,
                source,
                0,
                handoff_formation.as_ref(),
                cancel,
            )?;
            let checkpoint =
                operation
                    .draft
                    .source_checkpoint(operation.input_bytes()?, 0, cancel)?;
            let attempt = (|| {
                let fresh = formation::Formation::new(&operation.draft, cancel)?;
                let context = canonical::history::Context::new(source, &original.work, cancel, 0);
                let context = if let Some(fresh) = &fresh {
                    context.with_formation(fresh)
                } else {
                    context
                };
                let graph = operation
                    .draft
                    .hosted_history
                    .as_deref()
                    .ok_or(DomHostedError::MissingPublication)?;
                let graph = graph.mark_creation_queued(
                    queued.packet.creation.navigable,
                    original.key.operation,
                    &context,
                )?;
                operation.draft.hosted_history = Some(graph);
                drop(fresh);
                operation.publication_with_hosted(
                    self.dom,
                    true,
                    super::PublicationTarget::None,
                    Some((source, original, packet.revision)),
                    cancel,
                )?;
                Ok(())
            })();
            if let Err(error) = attempt {
                checkpoint.restore(&mut operation.draft);
                return Err(error);
            }
            let Phase::Queue(queued) = std::mem::replace(&mut child.phase, Phase::Returned) else {
                unreachable!("published exact queued child")
            };
            let permission = DomHostedQueuedHistory {
                operation: next.key.operation,
                traversable: queued.packet.traversable,
            };
            let work = next.work.clone();
            let creation = queued.packet.creation;
            let continuation =
                canonical::queue::Continuation::creation(next, creation, queued.packet.traversable);
            self.dom.hosted_operations.insert_prepared(
                bank,
                canonical::Operation {
                    origin: queued.origin,
                    policy: queued.policy,
                    phase: canonical::CreationPhase::Queued(continuation),
                    _retained: queued.retained,
                },
                false,
            );
            if source
                .active_work
                .upgrade()
                .is_some_and(|active| Arc::ptr_eq(&active, &work))
            {
                source.active_work = std::sync::Weak::new();
            }
            child.phase = Phase::Attributes { work };
            child.handoff_charge = handoff_formation.map(formation::Formation::into_charge);
            publication.take();
            Ok(permission)
        })();
        operation.child = Some(child);
        self.dom.source_operation = Some(operation);
        result
    }
}

impl DomSourceLease<'_> {
    pub fn prepare_hosted_child_removal(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomHostedError::WrongPhase)?;
        let child = op.child.as_ref().ok_or(DomHostedError::WrongPhase)?;
        child.check(request)?;
        if !matches!(child.phase, Phase::PrepareRemoval) {
            return Err(DomHostedError::WrongPhase);
        }
        let ChildHook::Remove { navigable, .. } = child.hook else {
            return Err(DomHostedError::WrongPhase);
        };
        let work_formation = formation::Formation::new(&op.draft, cancel)?;
        if let Some(formation) = &work_formation {
            formation.retain_bytes(size_of::<Work>() + 2 * size_of::<AtomicU64>(), cancel)?;
        }
        let work = source.work(cancel)?;
        work.step(cancel)?;
        let graph = op
            .draft
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        let nav = graph.navigable_with(navigable, || work.step(cancel))?;
        let active = graph.entry_with(nav.active, || work.step(cancel))?;
        let document = active.document.ok_or(DomHostedError::StaleIdentity)?;
        let creation = DomHostedCreation {
            document,
            host: nav.host,
            context: nav.context,
            global: nav.global,
            entry: nav.active,
            navigable,
        };
        let DomNodeKind::Document(metadata) =
            &op.draft.node(Key::Existing(document.node().slot))?.kind
        else {
            return Err(DomError::WrongDocument.into());
        };
        let origin = metadata.origin().clone();
        let retained = source.retain(size_of::<canonical::Operation>(), cancel)?;
        let id = source.begin_operation(&work, 2, 0, cancel)?;
        let packet = DomHostedChildRemovalRequest {
            request: Request {
                key: RequestKey {
                    operation: id,
                    phase: 0,
                },
                host: creation.host,
                context: creation.context,
                work,
                parent: None,
            },
            parent: request.parent,
            traversable: request.traversable,
            creation,
        };
        op.child
            .as_mut()
            .expect("validated actual removal")
            .work_charge = work_formation.map(formation::Formation::into_charge);
        op.child.as_mut().expect("validated actual removal").phase =
            Phase::Removing(RemovingChild {
                packet,
                origin,
                retained,
            });
        Ok(())
    }
    pub fn accept_hosted_child_removal(
        &mut self,
        source: &mut DomHostedSource,
        request: &DomHostedChildRequest,
        publication: &mut Option<DomHostedChildRemovalPublication>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedChildRemovalHandoff, DomHostedError> {
        self.dom.validate_hosted_source(source)?;
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomHostedError::WrongPhase)?;
        let Some(mut child) = operation.child.take() else {
            self.dom.source_operation = Some(operation);
            return Err(DomHostedError::WrongPhase);
        };
        let result = (|| {
            child.check(request)?;
            let packet = publication
                .as_ref()
                .ok_or(DomHostedError::MissingPublication)?;
            if let Phase::Removing(removal) = &child.phase {
                let original = &removal.packet.request;
                if packet.key != original.key {
                    return Err(DomHostedError::WrongPhase);
                }
                let next = Request {
                    key: RequestKey {
                        operation: original.key.operation,
                        phase: original
                            .key
                            .phase
                            .checked_add(1)
                            .ok_or(DomHostedError::RepresentationExhausted)?,
                    },
                    host: original.host,
                    context: original.context,
                    work: original.work.clone(),
                    parent: original.parent,
                };
                let handoff_formation = formation::Formation::new(&operation.draft, cancel)?;
                if let Some(formation) = &handoff_formation {
                    formation.retain_bytes(size_of::<canonical::Operation>(), cancel)?;
                }
                let bank = self.dom.hosted_operations.prepare(
                    original.key.operation.slot,
                    &original.work,
                    source,
                    0,
                    handoff_formation.as_ref(),
                    cancel,
                )?;
                let checkpoint =
                    operation
                        .draft
                        .source_checkpoint(operation.input_bytes()?, 0, cancel)?;
                let attempt = (|| {
                    let fresh = formation::Formation::new(&operation.draft, cancel)?;
                    let context =
                        canonical::history::Context::new(source, &original.work, cancel, 0);
                    let context = if let Some(fresh) = &fresh {
                        context.with_formation(fresh)
                    } else {
                        context
                    };
                    let graph = operation
                        .draft
                        .hosted_history
                        .as_deref()
                        .ok_or(DomHostedError::MissingPublication)?;
                    let graph = graph.mark_retiring(removal.packet.creation.navigable, &context)?;
                    operation.draft.hosted_history = Some(graph);
                    drop(fresh);
                    let fresh = formation::Formation::new(&operation.draft, cancel)?;
                    let context =
                        canonical::history::Context::new(source, &original.work, cancel, 0);
                    let context = if let Some(fresh) = &fresh {
                        context.with_formation(fresh)
                    } else {
                        context
                    };
                    let graph = operation
                        .draft
                        .hosted_history
                        .as_deref()
                        .ok_or(DomHostedError::MissingPublication)?
                        .unlink_child(
                            removal.packet.creation.navigable,
                            removal.packet.parent,
                            &context,
                        )?;
                    operation.draft.hosted_history = Some(graph);
                    drop(fresh);
                    operation.publication_with_hosted(
                        self.dom,
                        true,
                        super::PublicationTarget::None,
                        Some((source, original, packet.revision)),
                        cancel,
                    )?;
                    Ok(())
                })();
                if let Err(error) = attempt {
                    checkpoint.restore(&mut operation.draft);
                    return Err(error);
                }
                let Phase::Removing(removal) = std::mem::replace(&mut child.phase, Phase::Returned)
                else {
                    unreachable!("published exact removal")
                };
                let work = next.work.clone();
                let creation = removal.packet.creation;
                let traversable = removal.packet.traversable;
                let history = DomHostedQueuedHistory {
                    operation: next.key.operation,
                    traversable,
                };
                let continuation =
                    canonical::queue::Continuation::removal(next, creation, traversable);
                self.dom.hosted_operations.insert_prepared(
                    bank,
                    canonical::Operation {
                        origin: removal.origin,
                        policy: None,
                        phase: canonical::CreationPhase::Queued(continuation),
                        _retained: removal.retained,
                    },
                    false,
                );
                child.handoff_charge = handoff_formation.map(formation::Formation::into_charge);
                child.phase = Phase::RemovalPublished(RemovalPublished {
                    packet: removal.packet,
                    key: packet.key,
                    history,
                    creation,
                    work,
                });
            }
            let Phase::RemovalPublished(removal) = &child.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            if packet.key != removal.key {
                return Err(DomHostedError::WrongPhase);
            }
            // The original Source draft already published retiring/unlinking.
            // This prepares only original issuer rows and captured obligations;
            // it creates no competing canonical tree publisher.
            let invocation = self.dom.begin_hosted_invocation(
                source,
                removal.creation,
                removal.history.traversable,
                removal.work.clone(),
                Some(removal.history.operation),
                cancel,
            )?;
            let Phase::RemovalPublished(removal) =
                std::mem::replace(&mut child.phase, Phase::Returned)
            else {
                unreachable!("prepared exact parallel handoff")
            };
            publication.take();
            Ok(DomHostedChildRemovalHandoff {
                invocation,
                history: removal.history,
            })
        })();
        operation.child = Some(child);
        self.dom.source_operation = Some(operation);
        result
    }
}
