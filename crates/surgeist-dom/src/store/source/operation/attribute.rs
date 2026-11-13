//! The original standard setter's registered verification and retained reply.
use super::dialog::OperationDraft;
use super::*;
use crate::hosted::ScriptRegistration;
use crate::source::callback::CallbackWork;
use crate::store::attributes::{AttributeAttachmentPlan, PreparedAttributeName};
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

#[derive(Clone, Copy)]
pub(super) struct AttachmentOutput {
    plan: AttributeAttachmentPlan,
    request: DomSourceAttributeAttachmentOutput,
    admitted: bool,
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
        self.apply_with_attachment_plan(draft, None, limits, cancel)
    }

    fn apply_with_attachment_plan(
        &self,
        draft: &mut Draft,
        plan: Option<AttributeAttachmentPlan>,
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
                        if let Some(plan) = plan {
                            return draft.apply_attribute_attachment(
                                plan,
                                Some(value),
                                limits,
                                cancel,
                            );
                        }
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
    pub(super) fn waiting_attachment_output(&self) -> Option<DomSourceAttributeAttachmentOutput> {
        self.attachment_output
            .filter(|output| !output.admitted && !self.cancelled)
            .map(|output| output.request)
    }

    fn attachment_output_draft(
        &self,
        request: DomSourceAttributeAttachmentOutput,
        registration: &DomDocumentScriptRegistration,
    ) -> Result<&Draft, DomError> {
        if self.waiting_attachment_output() != Some(request) {
            return Err(DomError::InvalidSourceOperation);
        }
        if !self
            .output_registration()
            .is_some_and(|expected| Arc::ptr_eq(expected, &registration.0) && expected.is_active())
        {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        match &self.draft {
            OperationDraft::Segment(draft) => Ok(draft),
            _ => Err(DomError::InvalidSourceOperation),
        }
    }

    fn prepare_attachment_output(
        &mut self,
        dom: &Dom,
        node: Key,
        attr: Key,
        verified: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<AttributeAttachmentPlan>, DomError> {
        if let Some(output) = self.attachment_output {
            return Ok(output.admitted.then_some(output.plan));
        }
        let plan = self
            .draft
            .prepare_attribute_attachment(node, attr, cancel)?;
        if let Some(replaced) = plan.replaced(&self.draft)? {
            let registration = self
                .output_registration()
                .ok_or(DomError::InvalidSourceOperation)?;
            if !verified || !registration.engine_disposed.load(Ordering::Acquire) {
                self.qualify_attribute_registration(
                    dom,
                    registration.global,
                    registration,
                    cancel,
                )?;
            }
            let creation_global = dom
                .state
                .node_with(dom.owner, replaced.node(), &mut || {
                    self.draft.source_step(cancel)
                })?
                .relevant_global();
            self.attachment_output = Some(AttachmentOutput {
                plan,
                request: DomSourceAttributeAttachmentOutput {
                    operation: self.id,
                    replaced,
                    creation_global,
                },
                admitted: false,
            });
            return Ok(None);
        }
        Ok(Some(plan))
    }

    fn output_registration(&self) -> Option<&Arc<ScriptRegistration>> {
        match self.commands.get(self.index) {
            Some(Command::AttachAttribute {
                output_registration,
                ..
            }) => output_registration.as_ref(),
            _ => None,
        }
    }

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
        if let Some(pending) = &mut self.custom_element {
            return pending.finish(&mut self.draft, dom.limits, cancel);
        }
        if let Some(pending) = &self.attribute_verification {
            self.draft.source_step(cancel)?;
            let mut plan = None;
            if self.output_registration().is_some()
                && let Target::Attachment(attr) = pending.operands.target
                && let Some(RetainedOutcome {
                    outcome: DomSourceAttributeVerificationOutcome::Value(_),
                    ..
                }) = &pending.outcome
            {
                let element = pending.request.element;
                let prepared = (|| {
                    let element = self.draft.resolve(element.into())?;
                    let attr = self.draft.resolve(attr.into())?;
                    self.prepare_attachment_output(dom, element, attr, true, cancel)
                })();
                match prepared {
                    Ok(Some(prepared)) => plan = Some(prepared),
                    Ok(None) => return Ok(()),
                    Err(error @ (DomError::InUseAttribute | DomError::StaleNode)) => {
                        return self.draft.source_set_method(
                            Some(crate::store::method::Frame::CompoundFailure(error)),
                            cancel,
                        );
                    }
                    Err(error) => return Err(error),
                }
            }
            let pending = self
                .attribute_verification
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            return pending.apply_with_attachment_plan(&mut self.draft, plan, dom.limits, cancel);
        }
        if let Some(output) = self.attachment_output {
            if !output.admitted {
                return Err(DomError::InvalidSourceOperation);
            }
            return self
                .draft
                .apply_attribute_attachment(output.plan, None, dom.limits, cancel);
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
                output_registration,
            } => {
                if let Some(registration) = output_registration {
                    self.qualify_attribute_registration(
                        dom,
                        registration.global,
                        registration,
                        cancel,
                    )?;
                }
                self.draft.source_step(cancel)?;
                let element = self.draft.resolve(node.clone())?;
                let id = attribute.node();
                let attr = self.draft.resolve(id.into())?;
                let Some((sink, value)) = self.draft.attachment_sink(element, attr, cancel)? else {
                    if output_registration.is_some() {
                        return match self
                            .prepare_attachment_output(dom, element, attr, false, cancel)?
                        {
                            Some(plan) => self
                                .draft
                                .apply_attribute_attachment(plan, None, dom.limits, cancel),
                            None => Ok(()),
                        };
                    }
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
                depth: dom.source_parents.len(),
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
        let prepared = self.prepare_callback_observation(cancel)?;
        self.headroom = Some(headroom::Reservation {
            remaining_phases: headroom::FINAL_PUBLICATION,
        });
        self.draft = OperationDraft::AttributeObserved(self.begin_callback_observation(prepared));
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
            OperationDraft::Observed(_) | OperationDraft::CustomElementObserved(_) => None,
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

impl Dom {
    /// Discard an unpublished guest attachment with no accepted verification.
    /// This cleanup spends no work and restores the exact enclosing Source.
    pub fn discard_source_attribute_attachment_output(
        &mut self,
        request: DomSourceAttributeAttachmentOutput,
        registration: &DomDocumentScriptRegistration,
    ) -> Result<(), DomError> {
        let operation = self
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation.published
            || operation.event_exposed
            || operation.script_parked
            || operation.attribute_verification.is_some()
            || operation.meta_csp.is_some()
            || operation.pending_meta.is_some()
            || operation.meta_policy.is_some()
            || !operation
                .attachment_output
                .is_some_and(|output| output.request == request)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        if !operation
            .output_registration()
            .is_some_and(|expected| Arc::ptr_eq(expected, &registration.0))
        {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        self.source_operation = self.source_parents.pop();
        #[cfg(test)]
        self.observe_source_lifecycle();
        Ok(())
    }

    pub(in crate::store) fn qualified_attribute_attachment_output(
        &self,
        request: DomSourceAttributeAttachmentOutput,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&Draft, DomError> {
        if self.active_script_registration() != Some(registration.0.key) {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        check_cancel(cancel)?;
        self.source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?
            .attachment_output_draft(request, registration)
    }

    /// Charge one wrapper admission step to the original attachment and ancestors.
    pub fn source_attribute_attachment_output_step(
        &self,
        request: DomSourceAttributeAttachmentOutput,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.qualified_attribute_attachment_output(request, registration, cancel)?
            .source_step(cancel)
    }

    /// Accept custody of the exact Attr while the guest can still return it.
    /// The adapter keeps the admitted watch with that wrapper through publication.
    pub fn accept_source_attribute_attachment_output(
        &mut self,
        request: DomSourceAttributeAttachmentOutput,
        registration: &DomDocumentScriptRegistration,
        watch: &crate::DomNodeRetirementWatch,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let draft = self.qualified_attribute_attachment_output(request, registration, cancel)?;
        match self.dispatch_script_registration_current_with(&registration.0, || {
            draft.source_step(cancel).map_err(DomHostedError::from)
        }) {
            Ok(true) => (),
            Err(DomHostedError::Dom(error)) => return Err(error),
            _ => return Err(DomError::SourceAttributeExecutorUnavailable),
        }
        if !watch.qualifies(request.replaced().node(), self, || {
            draft.source_step(cancel)
        })? {
            return Err(DomError::InvalidSourceOperation);
        }
        self.admit_attribute_attachment_output(request)
    }

    /// Settle the native output after disposal of the original guest registration.
    /// This transfers no guest return authority to another attachment.
    pub fn abandon_source_attribute_attachment_output(
        &mut self,
        request: DomSourceAttributeAttachmentOutput,
        binding: &crate::DomDocumentScriptBinding,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let operation = self
            .source_operation
            .as_ref()
            .filter(|operation| {
                operation.waiting_attachment_output() == Some(request)
                    && operation
                        .attribute_verification
                        .as_ref()
                        .is_some_and(Pending::has_outcome)
            })
            .ok_or(DomError::InvalidSourceOperation)?;
        let registration = operation
            .output_registration()
            .ok_or(DomError::InvalidSourceOperation)?;
        if registration.key != binding.key()
            || registration.global != binding.global
            || !registration.engine_disposed.load(Ordering::Acquire)
        {
            return Err(DomError::SourceAttributeExecutorUnavailable);
        }
        operation.draft.source_step(cancel)?;
        self.admit_attribute_attachment_output(request)
    }

    fn admit_attribute_attachment_output(
        &mut self,
        request: DomSourceAttributeAttachmentOutput,
    ) -> Result<(), DomError> {
        let output = self
            .source_operation
            .as_mut()
            .and_then(|operation| operation.attachment_output.as_mut())
            .filter(|output| !output.admitted && output.request == request)
            .ok_or(DomError::InvalidSourceOperation)?;
        output.admitted = true;
        Ok(())
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

    pub(super) fn rebase_registered_callback(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self
            .dom
            .source_operation
            .as_deref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = match &operation.draft {
            OperationDraft::AttributeObserved(observation)
            | OperationDraft::CustomElementObserved(observation) => observation,
            _ => return Err(DomError::InvalidSourceOperation),
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
        let requested_limits = observation.limits;
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let refreshed = (|| {
            draft.source_limits = Some(self.dom.source_rebase_limits(requested_limits, cancel)?);
            self.dom
                .source_retained_stack_bytes()?
                .checked_add(
                    self.dom
                        .source_dispatch_custody_bytes(|| draft.source_step(cancel))?,
                )
                .ok_or(DomError::RepresentationExhausted)
        })();
        let canonical_aux_bytes = match refreshed {
            Ok(bytes) => bytes,
            Err(error) => {
                self.dom.source_operation = Some(operation);
                return Err(error);
            }
        };
        operation.prepare_rebased_inline(self.dom, &mut draft);
        let old = std::mem::replace(&mut operation.draft, OperationDraft::Segment(draft));
        let observation = match old {
            OperationDraft::AttributeObserved(observation)
            | OperationDraft::CustomElementObserved(observation) => observation,
            old => {
                operation.draft = old;
                self.dom.source_operation = Some(operation);
                return Err(DomError::InvalidSourceOperation);
            }
        };
        observation.work.stage.store(4, Ordering::Relaxed);
        observation.usage.restore(&mut operation.draft);
        if let Some(pending) = &mut operation.attribute_verification {
            pending.callback_custody = None;
        }
        operation.command_storage = command;
        operation.custody_bytes = custody_bytes;
        operation.custody_units = custody_units;
        operation.canonical_aux_bytes = canonical_aux_bytes;
        operation.limits = observation.limits;
        self.dom.source_operation = Some(operation);
        Ok(())
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

    /// Admit storage for a retained report from this verified original request.
    /// The debit remains live after Source completion until the report is dropped.
    pub fn retain_attribute_verification_payload(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        bytes: usize,
        units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomSourceAttributePayloadRetention, DomError> {
        let operation = self.attribute_verification_operation(request, registration, cancel)?;
        let observation = operation
            .callback_observation()
            .ok_or(DomError::InvalidSourceOperation)?;
        if !observation.work.scope_closed() {
            return Err(DomError::InvalidSourceOperation);
        }
        if units
            .checked_mul(size_of::<u16>())
            .is_none_or(|extent| extent > bytes)
        {
            return Err(DomError::RepresentationExhausted);
        }
        operation.qualify_attribute_registration(
            self.dom,
            request.global,
            &registration.0,
            cancel,
        )?;
        let debit = self
            .dom
            .retain_source_attribute_observer_payload(custody::Usage::new(bytes, units), cancel)?;
        Ok(DomSourceAttributePayloadRetention { _debit: debit })
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
        self.qualify_attribute_reply(request, registration, cancel)?;
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

    /// Admit and copy the exact UTF-16 result of the completed verifier.
    /// Refusal leaves the borrowed result with the caller for another attempt.
    pub fn accept_attribute_verification_utf16(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        value: &[u16],
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let work = self.qualify_attribute_reply(request, registration, cancel)?;
        if work.stage.load(Ordering::Relaxed) != 1 {
            return Err(DomError::InvalidSourceOperation);
        }
        let backing = self
            .dom
            .source_observation_formation(cancel)?
            .ok_or(DomError::InvalidSourceOperation)?;
        let value = formation::owned_utf16_copy(value, &backing, cancel)?;
        let pending = self
            .dom
            .source_operation
            .as_mut()
            .and_then(|operation| operation.attribute_verification.as_mut())
            .ok_or(DomError::InvalidSourceOperation)?;
        pending.outcome = Some(RetainedOutcome {
            outcome: DomSourceAttributeVerificationOutcome::Value(value),
            _backing: Some(backing),
        });
        Ok(())
    }

    fn qualify_attribute_reply(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&CallbackWork, DomError> {
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
        Ok(&observation.work)
    }
}

impl Dom {
    /// Mark an interrupted verifier whose callback has closed for cancellation
    /// when its original Source next becomes executable. The active child
    /// retains its own phase and accepted effects.
    pub fn cancel_interrupted_source_attribute(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
    ) -> Result<(), DomError> {
        if request.operation.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let operation = if request.depth == self.source_parents.len() {
            self.source_operation.as_mut()
        } else {
            self.source_parents.get_mut(request.depth)
        }
        .filter(|operation| operation.id == request.operation)
        .ok_or(DomError::InvalidSourceOperation)?;
        if !operation
            .attribute_verification
            .as_ref()
            .is_some_and(|pending| {
                pending.request == request
                    && !pending.has_outcome()
                    && Arc::ptr_eq(&pending.registration, &registration.0)
            })
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let OperationDraft::AttributeObserved(observation) = &operation.draft else {
            return Err(DomError::InvalidSourceOperation);
        };
        if observation.work.stage.load(Ordering::Relaxed) != 1 || !observation.work.scope_closed() {
            return Err(DomError::InvalidSourceOperation);
        }
        operation.cancelled = true;
        Ok(())
    }

    // A child retry may raise cancelled enclosing verifiers. An active or
    // retained guest callback still needs its own explicit recovery authority.
    pub(super) fn raise_cancelled_attribute_parent(
        &mut self,
        child: DomSourceOperationId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self
            .source_operation
            .as_ref()
            .is_none_or(|operation| operation.id != child)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let max_inspected = usize::try_from(limits.work).unwrap_or(usize::MAX);
        for (inspected, index) in (0..self.source_parents.len()).rev().enumerate() {
            check_cancel(cancel)?;
            if inspected >= max_inspected {
                return Err(DomError::SourceWorkLimit { limit: limits.work });
            }
            let parent = &self.source_parents[index];
            let (OperationDraft::AttributeObserved(observation), Some(pending)) =
                (&parent.draft, &parent.attribute_verification)
            else {
                continue;
            };
            if !parent.cancelled {
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
            if raised != old {
                self.raise_source_attribute_limits(pending.request, raised, cancel)?;
            }
        }
        Ok(())
    }

    /// Raise the policy of this exact closed attribute callback, including
    /// while a child is topmost. This leaves the original work and stack
    /// intact and grants no verifier or execution authority.
    pub fn raise_source_attribute_limits(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if request.operation.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let mut lookup = 0_u64;
        let mut located = None;
        for (depth, operation) in self
            .source_operation
            .iter_mut()
            .chain(self.source_parents.iter_mut().rev())
            .enumerate()
        {
            check_cancel(cancel)?;
            if lookup == limits.work {
                return Err(DomError::SourceWorkLimit { limit: limits.work });
            }
            lookup += 1;
            if operation
                .attribute_verification
                .as_ref()
                .is_some_and(|pending| pending.request == request)
            {
                located = Some((depth, operation));
                break;
            }
        }
        let (depth, operation) = located.ok_or(DomError::InvalidSourceOperation)?;
        let OperationDraft::AttributeObserved(observation) = &operation.draft else {
            return Err(DomError::InvalidSourceOperation);
        };
        if observation.work.stage.load(Ordering::Relaxed) != 1 || !observation.work.scope_closed() {
            return Err(DomError::InvalidSourceOperation);
        }
        let old = observation.limits;
        if limits.retained_bytes < old.retained_bytes
            || limits.retained_units < old.retained_units
            || limits.work < old.work
            || limits.phases < old.phases
            || limits.outputs < old.outputs
            || limits.identities < old.identities
            || limits.nested_operations < old.nested_operations
        {
            return Err(DomError::SourceLimitReduction);
        }
        if depth >= limits.nested_operations {
            return Err(DomError::SourceNestedOperationLimit {
                limit: limits.nested_operations,
            });
        }
        let work = observation.work.clone();
        let next = work
            .work
            .load(Ordering::Relaxed)
            .checked_add(lookup)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > limits.work {
            return Err(DomError::SourceWorkLimit { limit: limits.work });
        }
        operation.admit_observation_limits(limits, cancel)?;
        // Commit the lookup debit and new policy together. A failed raise
        // cannot leave charged work above the original limit.
        work.work.store(next, Ordering::Relaxed);
        #[cfg(test)]
        for _ in 0..lookup {
            SourceStorage::record_step();
        }
        Ok(())
    }

    /// Qualify and charge the currently entered verifier's native guest work.
    /// The original callback scope must still be open.
    pub fn source_attribute_callback_step(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.active_source_attribute_callback_work(request, registration, cancel)?
            .step(cancel)
    }

    /// Admit scratch UTF-16 capacity owned by an already byte-accounted
    /// caller during the exact active attribute verification callback.
    pub fn retain_source_attribute_callback_units(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        units: usize,
        cancel: &AtomicBool,
    ) -> Result<crate::DomSourceAttributePayloadRetention, DomError> {
        self.active_source_attribute_callback_work(request, registration, cancel)?;
        let debit =
            self.retain_source_attribute_observer_payload(custody::Usage::new(0, units), cancel)?;
        Ok(crate::DomSourceAttributePayloadRetention { _debit: debit })
    }

    pub(in crate::store) fn active_source_attribute_callback_work(
        &self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) -> Result<&CallbackWork, DomError> {
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
        Ok(&observation.work)
    }

    // The caller has qualified the original request in its required scope.
    pub(in crate::store) fn retain_source_attribute_observer_payload(
        &self,
        usage: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<custody::Debit, DomError> {
        let external = self.source_observation_external(cancel)?;
        self.custody_registry.ledger().retain(usage, |next| {
            let total = next.checked_add(custody::Usage::new(external.bytes, external.units))?;
            self.admit_source_observers(
                storage::Payload {
                    bytes: total.bytes(),
                    units: total.units(),
                },
                cancel,
            )
        })
    }
}

#[cfg(test)]
impl Dom {
    pub(in crate::store) fn assert_interrupted_attribute_raises_for_child(
        &mut self,
        element: NodeId,
        registration: &DomDocumentScriptRegistration,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) {
        let mut lease = self
            .begin_source_edit(
                DomSourceEdit::set_attribute(element, "onclick", "parent"),
                limits,
                cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(parent)) =
            lease.advance(cancel).unwrap()
        else {
            panic!("expected a parent verification");
        };
        let scope = lease
            .open_attribute_verification_callback(parent, registration, cancel)
            .unwrap();
        drop(lease);
        let mut lease = self
            .begin_source_edit(
                DomSourceEdit::set_attribute(element, "onclick", "child"),
                limits,
                cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(child)) =
            lease.advance(cancel).unwrap()
        else {
            panic!("expected a parked child");
        };
        drop(lease);
        assert_eq!(
            self.cancel_interrupted_source_attribute(parent, registration),
            Err(DomError::InvalidSourceOperation)
        );
        drop(scope);
        self.cancel_interrupted_source_attribute(parent, registration)
            .unwrap();
        let OperationDraft::AttributeObserved(observation) =
            &mut self.source_parents.last_mut().unwrap().draft
        else {
            panic!("expected the closed parent observer");
        };
        let spent = observation.work.work.load(Ordering::Relaxed);
        observation.limits.work = spent;
        observation.work.work_limit.store(spent, Ordering::Relaxed);
        let short = DomSourceLimits {
            work: spent,
            ..limits
        };
        assert!(
            matches!(self.resume_source_edit(child.operation(), short, cancel), Err(DomError::SourceWorkLimit { limit }) if limit == spent)
        );
        let mut lease = self
            .resume_source_edit(child.operation(), limits, cancel)
            .unwrap();
        lease.request_cancel();
        assert!(
            matches!(lease.advance(cancel).unwrap(), DomSourceProgress::Complete(done) if done.operation() == child.operation() && done.cancellation_requested())
        );
        drop(lease);
        let mut lease = self
            .resume_source_edit(parent.operation(), limits, cancel)
            .unwrap();
        assert!(
            matches!(lease.advance(cancel).unwrap(), DomSourceProgress::Complete(done) if done.operation() == parent.operation() && done.cancellation_requested())
        );
    }

    pub(in crate::store) fn assert_attribute_callback_unit_peak(
        &mut self,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        cancel: &AtomicBool,
    ) {
        fn limit(dom: &mut Dom, units: usize) -> usize {
            let OperationDraft::AttributeObserved(observation) =
                &mut dom.source_operation.as_mut().unwrap().draft
            else {
                panic!("expected observing attribute callback");
            };
            std::mem::replace(&mut observation.limits.retained_units, units)
        }
        let usage = |dom: &Dom| {
            dom.source_observation_total(storage::Payload::default(), cancel)
                .unwrap()
                .units
        };
        let baseline = usage(self);
        let original_units = limit(self, baseline + 8);
        let first = self
            .retain_source_attribute_callback_units(request, registration, 3, cancel)
            .unwrap();
        assert!(matches!(
            self.retain_source_attribute_callback_units(request, registration, 6, cancel),
            Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedUnits, limit }) if limit == baseline + 8
        ));
        assert_eq!(usage(self), baseline + 3);
        limit(self, baseline + 9);
        let replacement = self
            .retain_source_attribute_callback_units(request, registration, 6, cancel)
            .unwrap();
        assert_eq!(usage(self), baseline + 9);
        drop(first);
        drop(replacement);
        assert_eq!(usage(self), baseline);
        limit(self, original_units);
    }
}
