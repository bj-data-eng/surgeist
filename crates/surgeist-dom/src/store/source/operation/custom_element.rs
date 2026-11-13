//! Retained operands for a constructor reached by detached element creation.
use super::dialog::OperationDraft;
use super::*;
use crate::{
    DomDocumentScriptRegistration, DomHostedError,
    element::ElementName,
    hosted::ScriptRegistration,
    source::callback::{CallbackWork, DispatchScope},
};

pub(in crate::store) struct Creation {
    pub(in crate::store) document: DocumentId,
    pub(in crate::store) name: payload::Sealed<ElementName>,
    pub(in crate::store) definition: crate::DomRegistryDefinitionId,
}

pub(super) struct Pending {
    request: DomSourceCustomElementRequest,
    name: payload::Sealed<ElementName>,
    registration: Option<Arc<ScriptRegistration>>,
    outcome: Option<DomSourceCustomElementOutcome>,
    disposition: Option<DomSourceCustomElementDisposition>,
}

impl Pending {
    pub(super) fn new(
        operation: DomSourceOperationId,
        command: usize,
        draft: &Draft,
        creation: Creation,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let Creation {
            document,
            name,
            definition,
        } = creation;
        draft.source_step(cancel)?;
        let global = draft
            .node(Key::Existing(document.node()))?
            .relevant_global()
            .ok_or(DomError::HostedSourceUnavailable)?;
        Ok(Self {
            request: DomSourceCustomElementRequest {
                operation,
                command,
                document,
                global,
                definition,
            },
            name,
            registration: None,
            outcome: None,
            disposition: None,
        })
    }

    pub(super) fn registration_is_active(&self) -> bool {
        self.registration
            .as_ref()
            .is_some_and(|registration| registration.is_active())
    }

    pub(super) fn callback_opened(&self) -> bool {
        self.registration.is_some()
    }

    pub(super) fn finish(
        &mut self,
        draft: &mut Draft,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let outcome = self.outcome.ok_or(DomError::InvalidSourceOperation)?;
        let document = draft.resolve(self.request.document.into())?;
        let registry = crate::DomRegistryBinding::Registry(self.request.definition.registry());
        let returned = match outcome {
            DomSourceCustomElementOutcome::Returned(node) => {
                self.qualify_returned(draft, node, cancel)?
            }
            DomSourceCustomElementOutcome::Failed => Err(DomSourceCustomElementDisposition::Abrupt),
        };
        let (key, disposition) = match returned {
            Ok(key) => {
                draft.finalize_autonomous_element(
                    key,
                    self.name.value.clone(),
                    registry,
                    cancel,
                )?;
                (key, DomSourceCustomElementDisposition::Constructed)
            }
            Err(disposition) => {
                let budget = formation::Formation::new(draft, cancel)?;
                let mut admit = |next| draft.admit_custody_usage(next, cancel);
                let element_formation = crate::element::ElementFormation::new(
                    draft.custody_registry(),
                    &draft.blocks,
                    budget.as_ref(),
                    cancel,
                    &mut admit,
                );
                let formed = DomElement::failed_autonomous_construction(
                    self.name.value.clone(),
                    registry,
                    element_formation,
                )?;
                (
                    draft.allocate_element_formed(formed, document, limits, cancel, budget)?,
                    disposition,
                )
            }
        };
        draft.register_ticket(key, cancel)?;
        self.disposition = Some(disposition);
        Ok(())
    }

    pub(super) fn disposition(&self) -> Option<DomSourceCustomElementDisposition> {
        self.disposition
    }

    fn qualify_returned(
        &self,
        draft: &Draft,
        node: NodeId,
        cancel: &AtomicBool,
    ) -> Result<Result<Key, DomSourceCustomElementDisposition>, DomError> {
        draft.source_step(cancel)?;
        let key = match draft.resolve(node.into()) {
            Ok(key) => key,
            Err(DomError::StaleNode | DomError::ForeignOwner) => {
                return Ok(Err(DomSourceCustomElementDisposition::InvalidInterface));
            }
            Err(error) => return Err(error),
        };
        let node = draft.node(key)?;
        let Some(element) = node.element() else {
            return Ok(Err(DomSourceCustomElementDisposition::InvalidInterface));
        };
        if element.common_state().html().is_none() {
            return Ok(Err(DomSourceCustomElementDisposition::InvalidInterface));
        }
        if !node.attribute_ids().is_empty()
            || !node.children().is_empty()
            || node.parent().is_some()
            || node.document() != Key::Existing(self.request.document.node())
        {
            return Ok(Err(DomSourceCustomElementDisposition::InvalidElement));
        }
        let requested = &self.name.value.value.value;
        draft.source_step(cancel)?;
        if !crate::names::expanded_equals_with(
            element.name(),
            requested.namespace_uri(),
            requested.local_name(),
            || draft.source_step(cancel),
        )? {
            return Ok(Err(DomSourceCustomElementDisposition::InvalidElement));
        }
        Ok(Ok(key))
    }
}

/// The registered constructor's completed return or abrupt completion.
/// The script host retains the original thrown value for reporting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceCustomElementOutcome {
    Returned(NodeId),
    Failed,
}

/// Disposition of an autonomous factory's completed native publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceCustomElementDisposition {
    Constructed,
    /// The original guest exception remains in Runtime custody.
    Abrupt,
    /// The returned value requires a TypeError and a distinct fallback.
    InvalidInterface,
    /// The returned HTMLElement requires a NotSupportedError and a distinct fallback.
    InvalidElement,
}

/// The exact native reentry authority of a running autonomous constructor.
#[must_use = "keep the scope live during the constructor invocation"]
pub struct DomSourceCustomElementCallbackScope<'registration> {
    _scope: Arc<DispatchScope>,
    _registration: &'registration DomDocumentScriptRegistration,
}

impl DomSourceLease<'_> {
    pub(super) fn check_custom_element_scope_closed(&self) -> Result<(), DomError> {
        if self
            .dom
            .source_operation
            .as_ref()
            .and_then(|operation| operation.callback_observation())
            .is_some_and(|observation| !observation.work.scope_closed())
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(())
    }

    /// Accept the original completion after its scope has closed. A refusal
    /// leaves the completion with the caller for retry.
    pub fn accept_custom_element_callback(
        &mut self,
        request: DomSourceCustomElementRequest,
        registration: &DomDocumentScriptRegistration,
        outcome: &mut Option<DomSourceCustomElementOutcome>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.check_custom_element_scope_closed()?;
        let operation = self
            .dom
            .source_operation
            .as_deref_mut()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let OperationDraft::CustomElementObserved(observation) = &operation.draft else {
            return Err(DomError::InvalidSourceOperation);
        };
        let pending = operation
            .custom_element
            .as_mut()
            .filter(|pending| {
                pending.request == request
                    && pending.outcome.is_none()
                    && pending
                        .registration
                        .as_ref()
                        .is_some_and(|original| Arc::ptr_eq(original, &registration.0))
            })
            .ok_or(DomError::InvalidSourceOperation)?;
        if observation.work.stage.load(Ordering::Relaxed) != 1 {
            return Err(DomError::InvalidSourceOperation);
        }
        observation.work.step(cancel)?;
        let accepted = outcome.take().ok_or(DomError::InvalidSourceOperation)?;
        pending.outcome = Some(accepted);
        Ok(())
    }

    /// Open the registered executor's single autonomous constructor scope.
    /// Release the lease before executing guest code.
    /// Customized built-ins require a receiver publication before execution.
    pub fn open_custom_element_callback<'registration>(
        &mut self,
        request: DomSourceCustomElementRequest,
        registration: &'registration DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<DomSourceCustomElementCallbackScope<'registration>, DomError> {
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if !matches!(operation.draft, OperationDraft::Segment(_))
            || operation.waiting_custom_element() != Some(request)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        operation.qualify_custom_element_registration(self.dom, &registration.0, cancel)?;
        let definition = request.definition;
        let facts = operation
            .draft
            .creation_registry(definition.registry(), cancel)?
            .definition(definition.ordinal())
            .map_err(DomError::SelectCapture)?;
        if !crate::names::same_code_units(
            facts.name().code_units(),
            facts.local_name().code_units(),
            || operation.draft.source_step(cancel),
        )? {
            return Err(DomError::NotSupported);
        }
        let remaining = headroom::FINAL_PUBLICATION;
        if operation.limits.phases < remaining {
            return Err(DomError::SourcePublicationLimit);
        }
        if operation.limits.identities == 0 {
            return Err(DomError::SourceIdentityLimit);
        }
        if operation.limits.outputs == 0 {
            return Err(DomError::SourceOutputLimit);
        }
        let protected = self.dom.source_reserved_closing(cancel)?;
        self.dom
            .revision
            .0
            .checked_add(self.dom.hosted_revisions)
            .and_then(|revision| revision.checked_add(protected))
            .and_then(|revision| revision.checked_add(remaining))
            .ok_or(DomError::RevisionExhausted)?;
        let prepared = operation.prepare_callback_observation(cancel)?;
        let scope = prepared.open_scope()?;
        let operation = self
            .dom
            .source_operation
            .as_deref_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        let pending = operation
            .custom_element
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        pending.registration = Some(registration.0.clone());
        operation.headroom = Some(headroom::Reservation {
            remaining_phases: remaining,
        });
        operation.draft =
            OperationDraft::CustomElementObserved(operation.begin_callback_observation(prepared));
        Ok(DomSourceCustomElementCallbackScope {
            _scope: scope,
            _registration: registration,
        })
    }

    /// Borrow the normalized name from the original factory while the
    /// constructor request is pending.
    pub fn custom_element_creation_name(
        &self,
        request: DomSourceCustomElementRequest,
        cancel: &AtomicBool,
    ) -> Result<&crate::DomQualifiedName, DomError> {
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let pending = operation
            .custom_element
            .as_ref()
            .filter(|pending| pending.request == request)
            .ok_or(DomError::InvalidSourceOperation)?;
        operation.custom_element_step(cancel)?;
        Ok(&pending.name.value.value.value)
    }
}

impl Operation {
    pub(super) fn waiting_custom_element(&self) -> Option<DomSourceCustomElementRequest> {
        self.custom_element
            .as_ref()
            .filter(|pending| pending.outcome.is_none())
            .map(|pending| pending.request)
    }

    fn custom_element_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if let Some(observation) = self.callback_observation() {
            observation.work.step(cancel)
        } else {
            self.draft.source_step(cancel)
        }
    }

    fn qualify_custom_element_registration(
        &self,
        dom: &Dom,
        registration: &Arc<ScriptRegistration>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let pending = self
            .custom_element
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let original = match &self.draft {
            OperationDraft::Segment(draft) => draft.script_registration.as_ref(),
            OperationDraft::CustomElementObserved(_) => pending.registration.as_ref(),
            _ => None,
        };
        if dom.active_script_registration() != Some(registration.key)
            || !original.is_some_and(|original| Arc::ptr_eq(original, registration))
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        let mut step = || {
            self.custom_element_step(cancel)
                .map_err(DomHostedError::from)
        };
        let unavailable = |error| match error {
            DomHostedError::Dom(error) => error,
            _ => DomError::HostedSourceUnavailable,
        };
        if !dom
            .dispatch_script_registration_current_with(registration, &mut step)
            .map_err(unavailable)?
            || dom
                .dispatch_global_document_current_with::<DomHostedError>(
                    pending.request.global,
                    &mut step,
                )
                .map_err(unavailable)?
                != pending.request.document
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        Ok(())
    }

    pub(super) fn qualify_active_custom_element_callback(
        &self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<&CallbackWork, DomError> {
        let registration = self
            .custom_element
            .as_ref()
            .and_then(|pending| pending.registration.as_ref())
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = self
            .callback_observation()
            .filter(|observation| observation.work.active())
            .ok_or(DomError::InvalidSourceOperation)?;
        self.qualify_custom_element_registration(dom, registration, cancel)?;
        Ok(&observation.work)
    }
}

impl Dom {
    /// Publish the native result of an autonomous constructor's HTMLElement
    /// super call, with its retirement watch. The supplied Global identifies
    /// the actual HTML constructor interface's realm.
    /// This is a callback mutation, charged to the active observation's work
    /// and retained storage. The factory still owns its final publication.
    pub fn construct_autonomous_element_with_watch(
        &mut self,
        request: DomSourceCustomElementRequest,
        registration: &DomDocumentScriptRegistration,
        callee_global: crate::DomHostedGlobalId,
        (inbox, key): (&crate::DomNodeRetirementInbox, usize),
        limits: crate::DomNodeRetirementLimits,
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        self.source_custom_element_callback_step(request, registration, cancel)?;
        let operation = self
            .source_operation
            .as_deref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let work = operation
            .callback_observation()
            .ok_or(DomError::InvalidSourceOperation)?;
        let document = self
            .dispatch_global_document_current_with::<DomHostedError>(callee_global, &mut || {
                work.work.step(cancel).map_err(Into::into)
            })
            .map_err(|error| match error {
                DomHostedError::Dom(error) => error,
                _ => DomError::HostedSourceUnavailable,
            })?;
        let name = &operation
            .custom_element
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?
            .name
            .value
            .value
            .value;
        let name = crate::DomQualifiedName::from_atoms(
            name.namespace().cloned(),
            None,
            name.local_name_atom().clone(),
        )?;
        let mut draft =
            self.source_observed_batch(formation::allocated_arc(BatchIdentity), &[], 0, cancel)?;
        let document = draft.resolve(document.into())?;
        let budget = formation::Formation::new(&draft, cancel)?;
        let name = draft.bind_canonical_name(&name, budget.as_ref(), cancel)?;
        let formed = draft.form_native_element(
            name,
            None,
            draft.document_metadata(document)?.document_environment(),
            budget.as_ref(),
            cancel,
        )?;
        let node = draft.allocate_element_formed(formed, document, self.limits, cancel, budget)?;
        draft.replace_element_custom_context(
            node,
            surgeist_html::elements::native::HtmlCustomElementState::Custom,
            crate::element::CustomAssociation {
                registry: crate::DomRegistryBinding::Registry(request.definition.registry()),
                definition: crate::DomElementDefinition::Definition(request.definition),
                saved_is: None,
            },
            cancel,
        )?;
        draft.register_ticket(node, cancel)?;
        let mut prepared = draft.finish(self, cancel)?;
        prepared.state.incomplete_source = self.state.incomplete_source;
        let [Some(node)] = prepared.node_outputs.as_slice() else {
            return Err(DomError::UnavailableCreation);
        };
        let node = *node;
        let publication = self.prepare_publication(prepared, cancel)?;
        let watch =
            self.watch_prepared_source_node(&publication, node, inbox, key, limits, cancel)?;
        let commit = self.publish_prepared(publication);
        Ok(crate::DomWatchedNode::new(commit, node, watch))
    }

    /// Release a factory before opening its constructor scope. The exact
    /// request must still be the current, unpublished Source.
    pub fn discard_unstarted_custom_element(
        &mut self,
        request: DomSourceCustomElementRequest,
    ) -> Result<(), DomError> {
        let operation = self
            .source_operation
            .as_deref()
            .filter(|operation| {
                operation.waiting_custom_element() == Some(request) && !operation.published
            })
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation
            .custom_element
            .as_ref()
            .is_some_and(Pending::callback_opened)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        self.source_operation = self.source_parents.pop();
        #[cfg(test)]
        self.observe_source_lifecycle();
        Ok(())
    }

    /// Charge native work performed by the exact running constructor.
    pub fn source_custom_element_callback_step(
        &self,
        request: DomSourceCustomElementRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self
            .source_operation
            .as_deref()
            .filter(|operation| operation.waiting_custom_element() == Some(request))
            .ok_or(DomError::InvalidSourceOperation)?;
        let pending = operation
            .custom_element
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if !pending
            .registration
            .as_ref()
            .is_some_and(|original| Arc::ptr_eq(original, &registration.0))
        {
            return Err(DomError::InvalidSourceOperation);
        }
        operation
            .qualify_active_custom_element_callback(self, cancel)?
            .step(cancel)
    }
}
