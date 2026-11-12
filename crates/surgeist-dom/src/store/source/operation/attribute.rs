//! The original standard setter's registered verification and retained reply.
use super::dialog::{CallbackObservation, OperationDraft, Usage};
use super::*;
use crate::hosted::ScriptRegistration;
use crate::source::callback::CallbackWork;
use crate::store::attributes::PreparedAttributeName;
use crate::{DomDocumentScriptRegistration, DomHostedError};
use surgeist_html::security::csp::{
    HtmlCspArtifactLayout, HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage,
};

impl crate::DomSourceAttributePolicyRead<'_> {
    /// Admit work for a borrowed CSP inspection on the original Source meter.
    /// Layouts that introduce or transfer native storage require a different owner.
    pub fn admit_csp_read(
        &self,
        layout: HtmlCspLayout,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if layout.retained() != HtmlCspArtifactLayout::default()
            || layout.transferred() != HtmlCspArtifactLayout::default()
            || layout.peak() != HtmlCspStorage::default()
            || layout.closing_work() != HtmlCspDisposalWork::default()
        {
            return Err(DomError::HtmlChargeProtocol);
        }
        layout
            .charge_work(&mut crate::store::html_credits::Credits(|credits| {
                for _ in 0..credits {
                    self.step(cancel)?;
                }
                Ok(())
            }))
            .map_err(crate::store::html_credits::charged)
    }
}

pub(in crate::store) struct Pending {
    request: DomSourceAttributeVerificationRequest,
    registration: Arc<ScriptRegistration>,
    operands: Operands,
    outcome: Option<RetainedOutcome>,
    callback_custody: Option<crate::DomDispatchCustody>,
}

enum Target {
    Named,
    Value(NodeId),
    Attachment(NodeId),
}

struct Operands {
    target: Target,
    name: Option<PreparedAttributeName>,
    input: DocumentString,
    _input_backing: Option<checkpoint::Charge>,
}

struct RetainedOutcome {
    outcome: DomSourceAttributeVerificationOutcome,
    _backing: Option<formation::Formation>,
}

impl Pending {
    pub(super) fn registration_is_active(&self) -> bool {
        self.registration.is_active()
    }

    pub(super) fn has_outcome(&self) -> bool {
        self.outcome.is_some()
    }

    pub(in crate::store) fn apply(
        &self,
        draft: &mut Draft,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use DomSourceAttributeVerificationOutcome::{Abrupt, Rejected, Value};
        let retained = self
            .outcome
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let error = match &retained.outcome {
            Value(value) => {
                let result = (|| match self.operands.target {
                    Target::Named => {
                        let element = draft.resolve(self.request.element.into())?;
                        let name = self
                            .operands
                            .name
                            .as_ref()
                            .ok_or(DomError::InvalidSourceOperation)?;
                        draft.apply_verified_attribute_name(element, name, value, limits, cancel)
                    }
                    Target::Value(attribute) => {
                        let attribute = draft.resolve(attribute.into())?;
                        draft.write_existing_attr_value(attribute, value, limits, cancel)
                    }
                    Target::Attachment(attribute) => {
                        let element = draft.resolve(self.request.element.into())?;
                        let attribute = draft.resolve(attribute.into())?;
                        draft.write_attribute_attachment(
                            element,
                            attribute,
                            Some(value),
                            limits,
                            cancel,
                        )
                    }
                })();
                match result {
                    Err(error @ (DomError::InUseAttribute | DomError::StaleNode)) => error,
                    other => return other,
                }
            }
            Rejected => DomError::SourceAttributeVerificationRejected,
            Abrupt => DomError::SourceAttributeVerificationAbrupt,
        };
        draft.source_set_method(
            Some(crate::store::method::Frame::CompoundFailure(error)),
            cancel,
        )
    }
}

fn prepare_attr_value(
    draft: &mut Draft,
    id: NodeId,
    attr: Key,
    input: &DocumentString,
    limits: DomLimits,
    cancel: &AtomicBool,
) -> Result<Option<(Key, DomSourceAttributeSink, Operands)>, DomError> {
    let Some((owner, sink)) = draft.existing_attr_sink(attr, cancel)? else {
        draft.write_existing_attr_value(attr, input, limits, cancel)?;
        return Ok(None);
    };
    Ok(Some((
        owner,
        sink,
        Operands {
            target: Target::Value(id),
            name: None,
            input: input.clone(),
            _input_backing: None,
        },
    )))
}

impl Operation {
    pub(super) fn waiting_attribute_verification(
        &self,
    ) -> Option<DomSourceAttributeVerificationRequest> {
        self.attribute_verification
            .as_ref()
            .filter(|pending| !pending.has_outcome())
            .map(|pending| pending.request)
    }

    pub(super) fn apply_reached_command(
        &mut self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(pending) = &self.attribute_verification {
            self.draft.source_step(cancel)?;
            return pending.apply(&mut self.draft, dom.limits, cancel);
        }
        let command = self
            .commands
            .get(self.index)
            .ok_or(DomError::InvalidSourceOperation)?;
        let (element, sink, operands) = match command {
            Command::SetAttribute {
                node,
                namespace,
                name,
                value,
                mode: AttributeSetterMode::Standard,
            } => {
                self.draft.source_step(cancel)?;
                let element = self.draft.resolve(node.clone())?;
                let (name, sink) = self.draft.prepare_standard_attribute_name(
                    element,
                    namespace.as_ref(),
                    name,
                    cancel,
                )?;
                let Some(sink) = sink else {
                    return self
                        .draft
                        .apply_attribute_name(element, name, value, dom.limits, cancel);
                };
                (
                    element,
                    sink,
                    Operands {
                        target: Target::Named,
                        name: Some(name),
                        input: value.clone(),
                        _input_backing: None,
                    },
                )
            }
            Command::SetAttributeValue { attribute, value } => {
                self.draft.source_step(cancel)?;
                let id = attribute.node();
                let attr = self.draft.resolve(id.into())?;
                let Some(pending) =
                    prepare_attr_value(&mut self.draft, id, attr, value, dom.limits, cancel)?
                else {
                    return Ok(());
                };
                pending
            }
            Command::Compound(crate::compound::Edit::TextContent { receiver, value }) => {
                self.draft.source_step(cancel)?;
                self.draft.source_step(cancel)?;
                let attr = self.draft.resolve((*receiver).into())?;
                if !matches!(
                    self.draft.node(attr)?.kind(),
                    transaction::KindRead::Attr(_)
                ) {
                    return self
                        .draft
                        .compound_text_content(attr, value, dom.limits, cancel);
                }
                let Some(pending) = prepare_attr_value(
                    &mut self.draft,
                    *receiver,
                    attr,
                    value,
                    dom.limits,
                    cancel,
                )?
                else {
                    return Ok(());
                };
                pending
            }
            Command::AttachAttribute {
                node,
                attribute,
                mode: AttributeSetterMode::Standard,
            } => {
                self.draft.source_step(cancel)?;
                let element = self.draft.resolve(node.clone())?;
                let id = attribute.node();
                let attr = self.draft.resolve(id.into())?;
                let Some((sink, value)) = self.draft.attachment_sink(element, attr, cancel)? else {
                    return self
                        .draft
                        .write_attribute_attachment(element, attr, None, dom.limits, cancel);
                };
                let (input, backing) = formation::shared_string(&self.draft, value, cancel)?;
                (
                    element,
                    sink,
                    Operands {
                        target: Target::Attachment(id),
                        name: None,
                        input,
                        _input_backing: backing,
                    },
                )
            }
            _ => return self.apply_command(dom.limits, cancel),
        };
        let document = self.draft.node(element)?.document();
        let global = self
            .draft
            .node(document)?
            .relevant_global()
            .ok_or(DomError::SourceAttributeGlobalUnavailable)?;
        let registration = self
            .draft
            .script_registration
            .as_ref()
            .ok_or(DomError::SourceAttributeExecutorUnavailable)?;
        self.qualify_attribute_registration(dom, global, registration, cancel)?;
        let element = self
            .draft
            .published_id(element)
            .ok_or(DomError::InvalidSourceOperation)?;
        self.attribute_verification = Some(Pending {
            request: DomSourceAttributeVerificationRequest {
                operation: self.id,
                element,
                global,
                sink,
            },
            registration: registration.clone(),
            operands,
            outcome: None,
            callback_custody: None,
        });
        Ok(())
    }

    pub(super) fn observe_attribute_verification(
        &mut self,
        dom: &mut Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let protected = dom.source_reserved_closing(cancel)?;
        if self.limits.phases < headroom::FINAL_PUBLICATION {
            return Err(DomError::SourcePublicationLimit);
        }
        dom.revision
            .0
            .checked_add(dom.hosted_revisions)
            .and_then(|next| next.checked_add(protected))
            .and_then(|next| next.checked_add(headroom::FINAL_PUBLICATION))
            .ok_or(DomError::RevisionExhausted)?;
        let custody = formation::CallbackWorkCustody::prepare(
            self.draft.custody_ledger(),
            self.draft.source_work_custody(),
            self.draft.source_ancestor_custody(),
            |usage| self.draft.admit_custody_usage(usage, cancel),
        )?;
        let work = formation::allocated_arc(CallbackWork::new(
            self.draft.source_work.source_arc().clone(),
            self.limits.work,
            self.draft.source_ancestors.clone(),
            formation::SourceStorage::new(self.draft.custody_ledger().clone()),
        ));
        let autofocus = self.draft.autofocus_count(cancel)?;
        let observation = CallbackObservation {
            owner: self.draft.owner.clone(),
            usage: Usage::capture(&mut self.draft, autofocus),
            limits: self.limits,
            inputs: self.command_storage,
            work: work.clone(),
        };
        self.event_custody = Some(custody.finish(work));
        self.headroom = Some(headroom::Reservation {
            remaining_phases: headroom::FINAL_PUBLICATION,
        });
        self.draft = OperationDraft::AttributeObserved(observation);
        self.command_storage = storage::Payload::default();
        self.custody_bytes = 0;
        self.custody_units = 0;
        Ok(())
    }

    pub(super) fn qualify_active_attribute_callback(
        &self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let pending = self
            .attribute_verification
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        self.qualify_attribute_registration(
            dom,
            pending.request.global,
            &pending.registration,
            cancel,
        )
    }

    fn qualify_attribute_registration(
        &self,
        dom: &Dom,
        global: crate::DomHostedGlobalId,
        registration: &Arc<ScriptRegistration>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let original = match &self.draft {
            OperationDraft::Segment(draft) => draft.script_registration.as_ref(),
            OperationDraft::AttributeObserved(_) => self
                .attribute_verification
                .as_ref()
                .map(|pending| &pending.registration),
            OperationDraft::Observed(_) => None,
        };
        if registration.global != global
            || dom.active_script_registration() != Some(registration.key)
            || !original.is_some_and(|original| Arc::ptr_eq(original, registration))
        {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        match dom.dispatch_script_registration_current_with(registration, || {
            match self.callback_observation() {
                Some(observation) => observation.work.step(cancel),
                None => self.draft.source_step(cancel),
            }
            .map_err(DomHostedError::from)
        }) {
            Ok(true) => Ok(()),
            Err(DomHostedError::Dom(error)) => Err(error),
            Err(DomHostedError::SourceExpired | DomHostedError::SourcePoisoned) => {
                Err(DomError::HostedSourceUnavailable)
            }
            Ok(false) | Err(_) => Err(DomError::SourceAttributeExecutorUnavailable),
        }
    }
}

impl DomSourceLease<'_> {
    pub(super) fn parked_attribute_request(
        &self,
    ) -> Result<Option<DomSourceAttributeVerificationRequest>, DomError> {
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation
            .callback_observation()
            .is_some_and(|observation| !observation.work.scope_closed())
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok((!operation.cancelled)
            .then(|| operation.waiting_attribute_verification())
            .flatten())
    }

    pub(super) fn rebase_attribute_observation(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let OperationDraft::AttributeObserved(observation) = &operation.draft else {
            return Err(DomError::InvalidSourceOperation);
        };
        let limits = self.dom.source_rebase_transferred_limits(
            observation.limits,
            observation.inputs,
            cancel,
        )?;
        let (mut draft, command) = Draft::new_source_root_with_work(
            self.dom,
            observation.owner.clone(),
            super::super::initial::CommandStorage {
                commands: &operation.commands,
                capacity: operation.commands.capacity(),
            },
            limits,
            super::super::initial::Replay {
                work: observation.work.work.clone(),
                identity: operation
                    .event_custody
                    .as_ref()
                    .ok_or(DomError::InvalidSourceOperation)?
                    .source()
                    .identity(),
                names: Some(operation.names.clone()),
            },
            cancel,
        )?;
        let custody_bytes = self.dom.source_owner_custody_bytes()?;
        let custody_units = self.dom.fullscreen_custody_units()?;
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        operation.prepare_rebased_inline(self.dom, &mut draft);
        let old = std::mem::replace(&mut operation.draft, OperationDraft::Segment(draft));
        let OperationDraft::AttributeObserved(observation) = old else {
            self.dom.source_operation = Some(operation);
            return Err(DomError::InvalidSourceOperation);
        };
        observation.work.stage.store(4, Ordering::Relaxed);
        observation.usage.restore(&mut operation.draft);
        if let Some(pending) = &mut operation.attribute_verification {
            pending.callback_custody = None;
        }
        operation.command_storage = command;
        operation.custody_bytes = custody_bytes;
        operation.custody_units = custody_units;
        operation.limits = observation.limits;
        let refreshed = (|| {
            operation.draft.source_limits =
                Some(self.dom.source_rebase_limits(operation.limits, cancel)?);
            operation.canonical_aux_bytes = self
                .dom
                .source_retained_stack_bytes()?
                .checked_add(
                    self.dom
                        .source_dispatch_custody_bytes(|| operation.draft.source_step(cancel))?,
                )
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(())
        })();
        self.dom.source_operation = Some(operation);
        refreshed
    }

    /// Open the exact original verifier's one callback scope. Drop the lease
    /// before nested edits and drop this scope before accepting the outcome.
    pub fn open_attribute_verification_callback<'registration>(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        registration: &'registration DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<DomSourceAttributeCallbackScope<'registration>, DomError> {
        let operation = self.attribute_verification_operation(request, registration, cancel)?;
        operation.qualify_attribute_registration(
            self.dom,
            request.global,
            &registration.0,
            cancel,
        )?;
        let observation = operation
            .callback_observation()
            .ok_or(DomError::InvalidSourceOperation)?;
        let work = observation.work.clone();
        if work.stage.load(Ordering::Relaxed) != 0 {
            return Err(DomError::InvalidSourceOperation);
        }
        let target = &operation
            .attribute_verification
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?
            .operands
            .target;
        let receivers = match target {
            Target::Named => [request.element, request.element],
            Target::Value(attr) | Target::Attachment(attr) => [request.element, *attr],
        };
        let count = if receivers[0] == receivers[1] { 1 } else { 2 };
        let formation = self.dom.source_observation_formation(cancel)?;
        work.step(cancel)?;
        let custody = self.dom.register_dispatch_custody_with_formation(
            &receivers[..count],
            formation.as_ref(),
            cancel,
        )?;
        let pending = self
            .dom
            .source_operation
            .as_mut()
            .and_then(|operation| operation.attribute_verification.as_mut())
            .ok_or(DomError::InvalidSourceOperation)?;
        let scope = work.open_scope().ok_or(DomError::InvalidSourceOperation)?;
        work.stage
            .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| DomError::InvalidSourceOperation)?;
        pending.callback_custody = Some(custody);
        Ok(DomSourceAttributeCallbackScope {
            _scope: scope,
            _registration: registration,
        })
    }

    fn attribute_verification_operation(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&Operation, DomError> {
        check_cancel(cancel)?;
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .filter(|operation| {
                operation.id == self.id
                    && operation.waiting_attribute_verification() == Some(request)
            })
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation.cancelled {
            return Err(DomError::Cancelled);
        }
        if !operation
            .attribute_verification
            .as_ref()
            .is_some_and(|pending| Arc::ptr_eq(&pending.registration, &registration.0))
        {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        Ok(operation)
    }

    /// Read the original, exact UTF-16 input for this registered verifier.
    pub fn attribute_verification_input(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&DocumentString, DomError> {
        let operation = self.attribute_verification_operation(request, registration, cancel)?;
        operation.qualify_attribute_registration(
            self.dom,
            request.global,
            &registration.0,
            cancel,
        )?;
        let pending = operation
            .attribute_verification
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        Ok(&pending.operands.input)
    }

    /// Read the current CSP container of the exact original Global. An
    /// inherited Global uses its associated Document's container. Call this
    /// again after guest reentry when the selected policy algorithm re-reads it.
    pub fn attribute_verification_policy(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<DomSourceAttributePolicyRead<'_>, DomError> {
        let operation = self.attribute_verification_operation(request, registration, cancel)?;
        operation.qualify_attribute_registration(
            self.dom,
            request.global,
            &registration.0,
            cancel,
        )?;
        let observation = operation
            .callback_observation()
            .ok_or(DomError::InvalidSourceOperation)?;
        let work = &observation.work;
        work.step(cancel)?;
        let document = registration.0.key.document;
        let DomNodeKind::Document(metadata) =
            self.dom.state.node(self.dom.owner, document.node())?.kind()
        else {
            return Err(DomError::WrongDocument);
        };
        Ok(DomSourceAttributePolicyRead {
            document,
            state: metadata.policy_container().csp(),
            work,
        })
    }

    /// Admit the registered verifier's reply once. On refusal the caller still
    /// owns the reply, including any rooted abrupt outcome it represents.
    pub fn accept_attribute_verification(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        outcome: &mut Option<DomSourceAttributeVerificationOutcome>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self.attribute_verification_operation(request, registration, cancel)?;
        let observation = operation
            .callback_observation()
            .ok_or(DomError::InvalidSourceOperation)?;
        if !observation.work.scope_closed() {
            return Err(DomError::InvalidSourceOperation);
        }
        match observation.work.stage.load(Ordering::Relaxed) {
            0 => operation.qualify_attribute_registration(
                self.dom,
                request.global,
                &registration.0,
                cancel,
            )?,
            // The original live Global was qualified on entry. Its outcome
            // still belongs to that invocation after nested changes.
            1 => observation.work.step(cancel)?,
            _ => return Err(DomError::InvalidSourceOperation),
        }
        let candidate = outcome.as_ref().ok_or(DomError::InvalidSourceOperation)?;
        let backing = if let DomSourceAttributeVerificationOutcome::Value(value) = candidate {
            let formation = self.dom.source_observation_formation(cancel)?;
            if let Some(formation) = &formation {
                let mut payload = storage::Payload::default();
                payload.string(value)?;
                formation.retain_payload(payload, cancel)?;
            }
            formation
        } else {
            None
        };
        let pending = self
            .dom
            .source_operation
            .as_mut()
            .and_then(|operation| operation.attribute_verification.as_mut())
            .ok_or(DomError::InvalidSourceOperation)?;
        let outcome = outcome.take().ok_or(DomError::InvalidSourceOperation)?;
        pending.outcome = Some(RetainedOutcome {
            outcome,
            _backing: backing,
        });
        Ok(())
    }
}

impl Dom {
    /// Qualify and charge the currently entered verifier's native guest work.
    /// The original callback scope must still be open.
    pub fn source_attribute_callback_step(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let operation = self
            .source_operation
            .as_ref()
            .filter(|op| !op.cancelled && op.waiting_attribute_verification() == Some(request))
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = operation
            .callback_observation()
            .filter(|observation| observation.work.active())
            .ok_or(DomError::InvalidSourceOperation)?;
        operation.qualify_attribute_registration(self, request.global, &registration.0, cancel)?;
        observation.work.step(cancel)
    }
}
