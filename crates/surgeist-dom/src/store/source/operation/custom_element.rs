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
    name: Option<payload::Sealed<ElementName>>,
    registration: Option<Arc<ScriptRegistration>>,
    outcome: Option<DomSourceCustomElementOutcome>,
    disposition: Option<DomSourceCustomElementDisposition>,
    receiver: Option<NodeId>,
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
            name: Some(name),
            registration: None,
            outcome: None,
            disposition: None,
            receiver: None,
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
        if let Some(receiver) = self.receiver {
            use surgeist_html::elements::native::HtmlCustomElementState;

            let successful = outcome == DomSourceCustomElementOutcome::Returned(receiver);
            let key = match draft.resolve(receiver.into()) {
                Ok(key) => key,
                // Native reentry can retire the original receiver. Retain its
                // historical ticket and close without editing a later generation.
                Err(DomError::StaleNode) => Key::Existing(receiver),
                Err(error) => return Err(error),
            };
            match draft.node(key) {
                Ok(node) => {
                    let element = node.element().ok_or(DomError::WrongNodeKind)?;
                    let registry = element.custom_element_registry();
                    let saved_is = element.saved_is().cloned();
                    draft.replace_element_custom_context(
                        key,
                        if successful {
                            HtmlCustomElementState::Custom
                        } else {
                            HtmlCustomElementState::Failed
                        },
                        crate::element::CustomAssociation {
                            registry,
                            definition: if successful {
                                crate::DomElementDefinition::Definition(self.request.definition)
                            } else {
                                crate::DomElementDefinition::Null
                            },
                            saved_is,
                        },
                        cancel,
                    )?;
                }
                Err(DomError::StaleNode) => (),
                Err(error) => return Err(error),
            }
            draft.register_ticket(key, cancel)?;
            self.disposition = Some(if successful {
                DomSourceCustomElementDisposition::Constructed
            } else if outcome == DomSourceCustomElementOutcome::Failed {
                DomSourceCustomElementDisposition::FailedReceiverAbrupt
            } else {
                DomSourceCustomElementDisposition::FailedReceiverMismatch
            });
            return Ok(());
        }
        let name = self.name.as_ref().ok_or(DomError::InvalidSourceOperation)?;
        let document = draft.resolve(self.request.document.into())?;
        let registry = crate::DomRegistryBinding::Registry(self.request.definition.registry());
        let returned = match outcome {
            DomSourceCustomElementOutcome::Returned(node) => {
                self.qualify_returned(draft, node, cancel)?
            }
            DomSourceCustomElementOutcome::Failed => Err(DomSourceCustomElementDisposition::Abrupt),
            DomSourceCustomElementOutcome::InvalidInterface => {
                Err(DomSourceCustomElementDisposition::InvalidInterface)
            }
        };
        let (key, disposition) = match returned {
            Ok(key) => {
                draft.finalize_autonomous_element(key, name.value.clone(), registry, cancel)?;
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
                    name.value.clone(),
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

    fn creation_name(&self) -> Result<&crate::DomQualifiedName, DomError> {
        self.name
            .as_ref()
            .map(|name| &name.value.value.value)
            .ok_or(DomError::InvalidSourceOperation)
    }

    pub(super) fn disposition(&self) -> Option<DomSourceCustomElementDisposition> {
        self.disposition
    }

    pub(super) fn accepted_autonomous(&self, request: DomSourceCustomElementRequest) -> bool {
        self.request == request
            && self.name.is_some()
            && self.receiver.is_none()
            && self.outcome.is_some()
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
        let requested = self.creation_name()?;
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
    InvalidInterface,
}

/// Disposition of a custom element factory's completed native publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceCustomElementDisposition {
    Constructed,
    /// The original guest exception remains in Runtime custody.
    Abrupt,
    /// The returned value requires a TypeError and a distinct fallback.
    InvalidInterface,
    /// The returned HTMLElement requires a NotSupportedError and a distinct fallback.
    InvalidElement,
    /// The customized receiver is preserved after an abrupt completion.
    FailedReceiverAbrupt,
    /// The customized receiver is preserved and a TypeError is reported.
    FailedReceiverMismatch,
}

/// Admitted UTF-16 scratch retained during an original constructor callback.
pub struct DomSourceCustomElementRetention {
    _debit: custody::Debit,
}

/// The exact native reentry authority of a running constructor.
#[must_use = "keep the scope live during the constructor invocation"]
pub struct DomSourceCustomElementCallbackScope<'registration> {
    _scope: Arc<DispatchScope>,
    _registration: &'registration DomDocumentScriptRegistration,
}

/// The original upgrade boundary for an existing element.
pub enum DomSourceCustomElementUpgrade<'registration> {
    /// Invoke the constructor with the existing receiver.
    Callback {
        request: DomSourceCustomElementRequest,
        receiver_registry_matches: bool,
        scope: DomSourceCustomElementCallbackScope<'registration>,
    },
    /// The definition forbids this receiver's existing shadow root. The
    /// failed state is canonical and no constructor may run.
    ShadowForbidden,
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

    /// Publish the native receiver and retirement watch before invoking a
    /// customized built-in constructor. The wrapper preparation runs before
    /// the publication fence and must not execute guest code. A refusal
    /// leaves the original factory unstarted.
    pub fn open_customized_element_callback_with_watch<'registration, T, E: From<DomError>>(
        &mut self,
        request: DomSourceCustomElementRequest,
        registration: &'registration DomDocumentScriptRegistration,
        (inbox, key): (&crate::DomNodeRetirementInbox, usize),
        watch_limits: crate::DomNodeRetirementLimits,
        cancel: &AtomicBool,
        prepare: impl FnOnce(&crate::DomQualifiedName) -> Result<T, E>,
    ) -> Result<
        (
            DomSourceCustomElementCallbackScope<'registration>,
            crate::DomWatchedNode,
            T,
        ),
        E,
    > {
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            operation.open_customized_element(
                self.dom,
                (self.id, request),
                registration,
                (inbox, key, watch_limits),
                cancel,
                prepare,
            )
        }));
        self.dom.source_operation = Some(operation);
        #[cfg(test)]
        self.dom.observe_source_lifecycle();
        match result {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
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
        pending.creation_name()
    }
}

impl Operation {
    fn open_customized_element<'registration, T, E: From<DomError>>(
        &mut self,
        dom: &mut Dom,
        (id, request): (DomSourceOperationId, DomSourceCustomElementRequest),
        registration: &'registration DomDocumentScriptRegistration,
        (inbox, key, watch_limits): (
            &crate::DomNodeRetirementInbox,
            usize,
            crate::DomNodeRetirementLimits,
        ),
        cancel: &AtomicBool,
        prepare: impl FnOnce(&crate::DomQualifiedName) -> Result<T, E>,
    ) -> Result<
        (
            DomSourceCustomElementCallbackScope<'registration>,
            crate::DomWatchedNode,
            T,
        ),
        E,
    > {
        use surgeist_html::elements::native::HtmlCustomElementState;

        if self.id != id
            || !matches!(self.draft, OperationDraft::Segment(_))
            || self.waiting_custom_element() != Some(request)
        {
            return Err(DomError::InvalidSourceOperation.into());
        }
        self.qualify_custom_element_registration(dom, &registration.0, cancel)?;
        let definition = request.definition;
        let facts = self
            .draft
            .creation_registry(definition.registry(), cancel)?
            .definition(definition.ordinal())
            .map_err(DomError::SelectCapture)?;
        if crate::names::same_code_units(
            facts.name().code_units(),
            facts.local_name().code_units(),
            || self.draft.source_step(cancel),
        )? {
            return Err(DomError::InvalidSourceOperation.into());
        }
        let saved_is = facts.name().clone();
        let pending = self
            .custom_element
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let name = pending.creation_name()?.clone();
        let input_bytes = self.input_bytes()?;
        let checkpoint = self.draft.source_checkpoint(input_bytes, 0, cancel)?;
        let prior = self.headroom.replace(headroom::Reservation {
            remaining_phases: 1 + headroom::FINAL_PUBLICATION,
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let document = self.draft.resolve(request.document.into())?;
            let budget = formation::Formation::new(&self.draft, cancel)?;
            let bound_name = self
                .draft
                .bind_canonical_name(&name, budget.as_ref(), cancel)?;
            let formed = self.draft.form_native_element(
                bound_name,
                Some(&saved_is),
                self.draft
                    .document_metadata(document)?
                    .document_environment(),
                budget.as_ref(),
                cancel,
            )?;
            let receiver = self
                .draft
                .allocate_element_formed(formed, document, dom.limits, cancel, budget)?;
            self.draft.replace_element_custom_context(
                receiver,
                HtmlCustomElementState::Precustomized,
                crate::element::CustomAssociation {
                    registry: crate::DomRegistryBinding::Registry(definition.registry()),
                    definition: crate::DomElementDefinition::Definition(definition),
                    saved_is: Some(saved_is),
                },
                cancel,
            )?;
            self.draft
                .initialize_native_element_features(receiver, dom.limits, cancel)?;
            let Key::Created(index) = receiver else {
                return Err(DomError::InvalidSourceOperation.into());
            };
            let observed = self.prepare_callback_observation(cancel)?;
            let scope = observed.open_scope()?;
            let segment = self
                .prepare_segment(dom, true, PublicationTarget::None, None, cancel)
                .map_err(|error| match error {
                    DomHostedError::Dom(error) => error,
                    _ => DomError::HostedSourceUnavailable,
                })?;
            if segment
                .prepared
                .inline_recording
                .as_ref()
                .is_some_and(|recording| recording.has_inputs())
            {
                return Err(DomError::IncompleteSourceOperation.into());
            }
            // Closing changes the receiver state and clears this Source's
            // incomplete marker. Both mandatory outputs fit before guest entry.
            if segment
                .outputs
                .checked_add(2)
                .is_none_or(|outputs| outputs > self.limits.outputs)
            {
                return Err(DomError::SourceOutputLimit.into());
            }
            let receiver = segment
                .prepared
                .allocation_ids
                .get(index)
                .ok_or(DomError::UnavailableCreation)?;
            let projection = prepare(&name)?;
            let watch = dom.watch_prepared_source_node(
                &segment.prepared,
                receiver,
                inbox,
                key,
                watch_limits,
                cancel,
            )?;
            Ok((scope, segment, observed, receiver, watch, projection))
        }));
        let (scope, segment, observed, receiver, watch, projection) = match result {
            Ok(Ok(prepared)) => prepared,
            Ok(Err(error)) => {
                checkpoint.restore(&mut self.draft);
                self.headroom = prior;
                return Err(error);
            }
            Err(panic) => {
                checkpoint.restore(&mut self.draft);
                self.headroom = prior;
                std::panic::resume_unwind(panic);
            }
        };
        drop(checkpoint);
        let pending = self
            .custom_element
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        pending.receiver = Some(receiver);
        pending.registration = Some(registration.0.clone());
        let publication = segment.publish(self, dom);
        // The remaining command settles this published receiver using its
        // reserved closing phase. Replaying the original Create would perform
        // a fresh registry lookup and incorrectly plan a second element.
        self.command_admitted = true;
        self.draft =
            OperationDraft::CustomElementObserved(self.begin_callback_observation(observed));
        Ok((
            DomSourceCustomElementCallbackScope {
                _scope: scope,
                _registration: registration,
            },
            crate::DomWatchedNode::new(publication.commit, receiver, watch),
            projection,
        ))
    }

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
    /// Start an upgrade of one existing receiver selected by a registry
    /// definition. The receiver is requalified against the live tree.
    /// An ineligible or retired candidate is skipped.
    pub fn open_custom_element_upgrade<'registration>(
        &mut self,
        receiver: NodeId,
        (definition, global): (crate::DomRegistryDefinitionId, crate::DomHostedGlobalId),
        registration: &'registration DomDocumentScriptRegistration,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<Option<DomSourceCustomElementUpgrade<'registration>>, DomError> {
        use surgeist_html::elements::native::HtmlCustomElementState;

        let lease = self.begin_source_root(
            OperationRoot::CustomElementUpgrade(receiver),
            formation::allocated_arc(BatchIdentity),
            limits,
            cancel,
        )?;
        let dom = &mut *lease.dom;
        let mut operation = dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let result = (|| {
            operation.draft.source_step(cancel)?;
            let key = match operation.draft.resolve(receiver.into()) {
                Ok(key) => key,
                Err(DomError::StaleNode) => return Ok(None),
                Err(error) => return Err(error),
            };
            let node = operation.draft.node(key)?;
            let Some(element) = node.element() else {
                return Ok(None);
            };
            let definition_registry = crate::DomRegistryBinding::Registry(definition.registry());
            let registry = element.custom_element_registry();
            if element.custom_element_state() != HtmlCustomElementState::Undefined
                || operation
                    .draft
                    .lookup_element_definition(key, definition_registry, cancel)?
                    != Some(definition)
            {
                return Ok(None);
            }
            let document = dom
                .dispatch_global_document_current_with::<DomHostedError>(global, &mut || {
                    operation.draft.source_step(cancel).map_err(Into::into)
                })
                .map_err(|error| match error {
                    DomHostedError::Dom(error) => error,
                    _ => DomError::HostedSourceUnavailable,
                })?;
            let registry_metadata = operation
                .draft
                .document_metadata(Key::Existing(document.node()))?;
            if registry_metadata.custom_element_registry() != definition_registry {
                return Err(DomError::InvalidSourceOperation);
            }
            let saved_is = element.saved_is().cloned();
            let shadow_forbidden = node.shadow_root().is_some()
                && operation
                    .draft
                    .creation_registry(definition.registry(), cancel)?
                    .definition(definition.ordinal())
                    .map_err(DomError::SelectCapture)?
                    .disable_shadow();

            let request = DomSourceCustomElementRequest {
                operation: operation.id,
                command: 0,
                document,
                global,
                definition,
            };
            operation.custom_element = Some(Pending {
                request,
                name: None,
                registration: None,
                outcome: None,
                disposition: None,
                receiver: Some(receiver),
            });
            operation.qualify_custom_element_registration(dom, &registration.0, cancel)?;
            operation.headroom = Some(headroom::Reservation {
                remaining_phases: 1 + headroom::FINAL_PUBLICATION,
            });
            if shadow_forbidden {
                operation.draft.replace_element_custom_context(
                    key,
                    HtmlCustomElementState::Failed,
                    crate::element::CustomAssociation {
                        registry,
                        definition: crate::DomElementDefinition::Null,
                        saved_is,
                    },
                    cancel,
                )?;
                let segment = operation
                    .prepare_segment(dom, false, PublicationTarget::None, None, cancel)
                    .map_err(|error| match error {
                        DomHostedError::Dom(error) => error,
                        _ => DomError::HostedSourceUnavailable,
                    })?;
                if segment
                    .prepared
                    .inline_recording
                    .as_ref()
                    .is_some_and(|recording| recording.has_inputs())
                {
                    return Err(DomError::IncompleteSourceOperation);
                }
                segment.publish(&mut operation, dom);
                return Ok(Some(DomSourceCustomElementUpgrade::ShadowForbidden));
            }
            operation.draft.replace_element_custom_context(
                key,
                HtmlCustomElementState::Precustomized,
                crate::element::CustomAssociation {
                    registry,
                    definition: crate::DomElementDefinition::Definition(definition),
                    saved_is,
                },
                cancel,
            )?;
            let observed = operation.prepare_callback_observation(cancel)?;
            let scope = observed.open_scope()?;
            let segment = operation
                .prepare_segment(dom, true, PublicationTarget::None, None, cancel)
                .map_err(|error| match error {
                    DomHostedError::Dom(error) => error,
                    _ => DomError::HostedSourceUnavailable,
                })?;
            if segment
                .prepared
                .inline_recording
                .as_ref()
                .is_some_and(|recording| recording.has_inputs())
            {
                return Err(DomError::IncompleteSourceOperation);
            }
            if segment
                .outputs
                .checked_add(2)
                .is_none_or(|outputs| outputs > limits.outputs)
            {
                return Err(DomError::SourceOutputLimit);
            }
            let pending = operation
                .custom_element
                .as_mut()
                .ok_or(DomError::InvalidSourceOperation)?;
            pending.registration = Some(registration.0.clone());
            segment.publish(&mut operation, dom);
            operation.command_admitted = true;
            operation.draft = OperationDraft::CustomElementObserved(
                operation.begin_callback_observation(observed),
            );
            Ok(Some(DomSourceCustomElementUpgrade::Callback {
                request,
                receiver_registry_matches: registry == definition_registry,
                scope: DomSourceCustomElementCallbackScope {
                    _scope: scope,
                    _registration: registration,
                },
            }))
        })();
        if matches!(
            result,
            Ok(Some(DomSourceCustomElementUpgrade::ShadowForbidden))
        ) {
            dom.source_operation = dom.source_parents.pop();
            return result;
        }
        if !matches!(result, Ok(Some(_))) {
            operation.custom_element = None;
        }
        dom.source_operation = Some(operation);
        result
    }

    /// Raise closed constructor ancestors during stopped-host recovery.
    /// Original callback work and accepted results remain charged.
    pub fn raise_stopped_constructor_ancestors(
        &mut self,
        registration: &DomDocumentScriptRegistration,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if registration.is_active() {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut remaining = limits.work;
        for operation in self.source_parents.iter_mut().rev() {
            check_cancel(cancel)?;
            remaining = remaining
                .checked_sub(1)
                .ok_or(DomError::SourceWorkLimit { limit: limits.work })?;
            let OperationDraft::CustomElementObserved(observation) = &operation.draft else {
                continue;
            };
            if !observation.work.scope_closed()
                || !operation
                    .custom_element
                    .as_ref()
                    .and_then(|pending| pending.registration.as_ref())
                    .is_some_and(|original| Arc::ptr_eq(original, &registration.0))
            {
                continue;
            }
            let old = observation.limits;
            let raised = DomSourceLimits::new(
                limits.retained_bytes.max(old.retained_bytes),
                limits.retained_units.max(old.retained_units),
                limits.work.max(old.work),
                limits.phases.max(old.phases),
                limits.outputs.max(old.outputs),
                limits.identities.max(old.identities),
                limits.nested_operations.max(old.nested_operations),
            );
            operation.admit_observation_limits(raised, cancel)?;
        }
        Ok(())
    }

    /// Whether the exact factory remains in the current Source stack.
    pub fn retains_custom_element(
        &self,
        request: DomSourceCustomElementRequest,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        for operation in self
            .source_operation
            .iter()
            .chain(self.source_parents.iter().rev())
        {
            check_cancel(cancel)?;
            if operation.id == request.operation {
                return Ok(operation
                    .custom_element
                    .as_ref()
                    .is_some_and(|pending| pending.request == request));
            }
        }
        Ok(false)
    }

    /// Admit scratch UTF-16 units held by the exact running constructor.
    pub fn retain_source_custom_element_units(
        &self,
        request: DomSourceCustomElementRequest,
        registration: &DomDocumentScriptRegistration,
        units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomSourceCustomElementRetention, DomError> {
        self.source_custom_element_callback_step(request, registration, cancel)?;
        let debit = self.retain_source_callback_payload(custody::Usage::new(0, units), cancel)?;
        Ok(DomSourceCustomElementRetention { _debit: debit })
    }
    /// Publish the native result of an autonomous constructor's HTMLElement
    /// super call, with its retirement watch. The running callback pays for
    /// the mutation. The selected constructor can be an active ancestor.
    /// The supplied Global identifies the actual HTML interface's realm.
    pub fn construct_autonomous_element_with_watch(
        &mut self,
        (running, selected): (DomSourceCustomElementRequest, DomSourceCustomElementRequest),
        registration: &DomDocumentScriptRegistration,
        callee_global: crate::DomHostedGlobalId,
        (inbox, key): (&crate::DomNodeRetirementInbox, usize),
        limits: crate::DomNodeRetirementLimits,
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        let work = self.active_custom_element_callback_work(running, registration, cancel)?;
        work.step(cancel)?;
        let document = self
            .dispatch_global_document_current_with::<DomHostedError>(callee_global, &mut || {
                work.step(cancel).map_err(Into::into)
            })
            .map_err(|error| match error {
                DomHostedError::Dom(error) => error,
                _ => DomError::HostedSourceUnavailable,
            })?;
        let mut selected_name = None;
        for operation in self
            .source_operation
            .iter()
            .chain(self.source_parents.iter().rev())
        {
            work.step(cancel)?;
            if operation.waiting_custom_element() == Some(selected) {
                operation.qualify_active_custom_element_callback(self, cancel)?;
                selected_name = operation.custom_element.as_ref();
                break;
            }
        }
        let name = selected_name
            .ok_or(DomError::InvalidSourceOperation)?
            .creation_name()?;
        let name = crate::DomQualifiedName::from_atoms(
            name.namespace().cloned(),
            None,
            name.local_name_atom().clone(),
        )?;
        let draft =
            self.source_observed_batch(formation::allocated_arc(BatchIdentity), &[], 0, cancel)?;
        let document = draft.resolve(document.into())?;
        let budget = formation::Formation::new(&draft, cancel)?;
        let name = draft.bind_canonical_name(&name, budget.as_ref(), cancel)?;
        self.publish_constructed_custom_element(
            draft,
            document,
            (name, None, selected.definition),
            (inbox, key, limits),
            budget,
            cancel,
        )
    }

    /// Create the original native receiver for a registered HTML constructor
    /// with an empty construction stack. The Global must own the definition's
    /// registry. The caller supplies the active script registration and watch.
    pub fn construct_registered_element_with_watch(
        &mut self,
        (definition, global): (crate::DomRegistryDefinitionId, crate::DomHostedGlobalId),
        registration: &DomDocumentScriptRegistration,
        (inbox, key, watch_limits): (
            &crate::DomNodeRetirementInbox,
            usize,
            crate::DomNodeRetirementLimits,
        ),
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        self.require_source_entry(cancel)?;
        if limits.nested_operations() == 0 {
            return Err(DomError::SourceNestedOperationLimit { limit: 0 });
        }
        if limits.phase_publications() == 0 {
            return Err(DomError::SourcePublicationLimit);
        }
        if limits.output_records() == 0 {
            return Err(DomError::SourceOutputLimit);
        }
        if limits.published_native_identities() == 0 {
            return Err(DomError::SourceIdentityLimit);
        }
        let limits = self.source_child_limits(limits, cancel)?;
        let owner = formation::allocated_arc(BatchIdentity);
        let (draft, _) = Draft::new_source_root(self, owner, &[], 0, limits, cancel)?;
        let mut step = || draft.source_step(cancel).map_err(DomHostedError::from);
        let unavailable = |error| match error {
            DomHostedError::Dom(error) => error,
            _ => DomError::HostedSourceUnavailable,
        };
        if self.active_script_registration() != Some(registration.0.key)
            || !self
                .dispatch_script_registration_current_with(&registration.0, &mut step)
                .map_err(unavailable)?
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        let document = self
            .dispatch_global_document_current_with(global, &mut step)
            .map_err(unavailable)?;
        let document = draft.resolve(document.into())?;
        let metadata = draft.document_metadata(document)?;
        if metadata.custom_element_registry()
            != crate::DomRegistryBinding::Registry(definition.registry())
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let facts = draft
            .creation_registry(definition.registry(), cancel)?
            .definition(definition.ordinal())
            .map_err(DomError::SelectCapture)?;
        let customized = !crate::names::same_code_units(
            facts.name().code_units(),
            facts.local_name().code_units(),
            &mut || draft.source_step(cancel),
        )?;
        let saved_is = customized.then(|| facts.name().clone());
        let raw = surgeist_html::values::QualifiedName::new(
            surgeist_html::values::Namespace::html(),
            None,
            facts.local_name().clone(),
        );
        let budget = formation::Formation::new(&draft, cancel)?;
        crate::names::validate_element_components(&raw, &mut || draft.name_step(cancel))?;
        let name = draft.bind_name(&raw, budget.as_ref(), cancel)?;
        self.publish_constructed_custom_element(
            draft,
            document,
            (name, saved_is, definition),
            (inbox, key, watch_limits),
            budget,
            cancel,
        )
    }

    fn publish_constructed_custom_element(
        &mut self,
        mut draft: Draft,
        document: Key,
        (name, saved_is, definition): (
            crate::DomQualifiedName,
            Option<crate::DocumentString>,
            crate::DomRegistryDefinitionId,
        ),
        (inbox, key, limits): (
            &crate::DomNodeRetirementInbox,
            usize,
            crate::DomNodeRetirementLimits,
        ),
        budget: Option<formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        let draft_limits = draft
            .source_limits
            .ok_or(DomError::InvalidSourceOperation)?;
        let formed = draft.form_native_element(
            name,
            saved_is.as_ref(),
            draft.document_metadata(document)?.document_environment(),
            budget.as_ref(),
            cancel,
        )?;
        let node = draft.allocate_element_formed(formed, document, self.limits, cancel, budget)?;
        draft.initialize_native_element_features(node, self.limits, cancel)?;
        draft.replace_element_custom_context(
            node,
            surgeist_html::elements::native::HtmlCustomElementState::Custom,
            crate::element::CustomAssociation {
                registry: crate::DomRegistryBinding::Registry(definition.registry()),
                definition: crate::DomElementDefinition::Definition(definition),
                saved_is,
            },
            cancel,
        )?;
        draft.register_ticket(node, cancel)?;
        let mut prepared = draft.finish(self, cancel)?;
        if u64::try_from(prepared.allocation_ids.len())
            .map_err(|_| DomError::RepresentationExhausted)?
            > draft_limits.identities
        {
            return Err(DomError::SourceIdentityLimit);
        }
        if prepared
            .inline_recording
            .as_ref()
            .is_some_and(|recording| recording.has_inputs())
        {
            return Err(DomError::IncompleteSourceOperation);
        }
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
        self.active_custom_element_callback_work(request, registration, cancel)?
            .step(cancel)
    }

    pub(in crate::store) fn active_custom_element_callback_work(
        &self,
        request: DomSourceCustomElementRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&CallbackWork, DomError> {
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
        operation.qualify_active_custom_element_callback(self, cancel)
    }
}
