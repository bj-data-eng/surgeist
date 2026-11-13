//! Standard attribute setters over the original native Source operation.

mod node;

use super::{super::character_data::copy_native_data, *};
use crate::{
    DomSourceDriveError, DomSourceDriveProgress, DomSourceRuntimeError,
    document_scripting::guest::{
        AttributeReply, PendingAttribute,
        reports::{ViolationBacking, ViolationOrigin, ViolationQueue, violation_error},
    },
    trusted_types_csp::{AttributeMismatchProgress, AttributePolicies, AttributePolicyError},
};
use surgeist_bindings_js::{
    TrustedAttributeValue, TrustedTypeKind,
    conversion::{ArgumentValue, declarations},
    dom_exception::DomExceptionName,
};
use surgeist_dom::{
    DomAttrId, DomAttributeAttachment, DomSourceAttributeAttachmentOutput, DomSourceAttributeSink,
    DomSourceAttributeVerificationOutcome, DomSourceAttributeVerificationRequest, DomSourceEdit,
    DomSourceLease, DomSourceLimits, DomSourceOutput, DomSourceRequest, DomSourceStatus, Namespace,
};

pub(in crate::document_scripting::guest::interfaces::document) enum AttributeResult<'scope> {
    Value,
    Attached(DomAttrId, Value<'scope>),
    Removed(DomAttrId, Value<'scope>),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, namespaced, length) in [
        (&ascii(b"setAttribute")[..], false, 2),
        (&ascii(b"setAttributeNS")[..], true, 3),
    ] {
        let handler = capture(size_of::<(Capture, bool)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.write_attribute(scope, arguments, namespaced)
            }),
            FunctionMetadata { name, length },
        )?;
        define_data(
            scope,
            element,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    node::install(scope, element, capture)
}

impl Capture {
    fn write_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        namespaced: bool,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Element(element)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let required = if namespaced { 3 } else { 2 };
        if arguments.len() < required {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing attribute argument"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let (namespace, name, value) = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    let (namespace, name, value) = if namespaced {
                        let namespace = declarations::element_set_attribute_ns_0_namespace();
                        (
                            Some(cx.argument(&namespace, arguments.get(namespace.index()))?),
                            declarations::element_set_attribute_ns_0_qualified_name(),
                            declarations::element_set_attribute_ns_0_value(),
                        )
                    } else {
                        (
                            None,
                            declarations::element_set_attribute_0_qualified_name(),
                            declarations::element_set_attribute_0_value(),
                        )
                    };
                    Ok((
                        namespace,
                        cx.argument(&name, arguments.get(name.index()))?,
                        cx.argument(&value, arguments.get(value.index()))?,
                    ))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let (ArgumentValue::Present(name), ArgumentValue::Present(value)) =
                (name.get(), value.get())
            else {
                return Err(Error::BackendUnavailable.into());
            };
            let (name, _name_backing) = copy_native_data(&owner, &events, name.as_code_units())?;
            let (text, _value_backing) = copy_native_data(&owner, &events, value.as_code_units())?;
            let uri = match namespace.as_ref().map(|namespace| namespace.get()) {
                Some(ArgumentValue::Present(Some(uri))) => {
                    Some(copy_native_data(&owner, &events, uri.as_code_units())?)
                }
                Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
                _ => None,
            };
            let edit = if namespaced {
                let namespace = uri
                    .as_ref()
                    .map_or_else(Namespace::none, |(uri, _)| Namespace::from_uri(uri.clone()));
                DomSourceEdit::set_attribute_ns(element, namespace, name, text)
            } else {
                DomSourceEdit::set_attribute(element, name, text)
            };
            let brand = match value {
                TrustedAttributeValue::Trusted(brand) => Some(brand.kind()),
                TrustedAttributeValue::String(_) => None,
            };
            self.apply_attribute(scope, &owner, &events, edit, brand, AttributeResult::Value)
        })();
        match result {
            Ok(completion) => Ok(completion),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    pub(in crate::document_scripting::guest::interfaces::document) fn apply_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        edit: DomSourceEdit,
        brand: Option<TrustedTypeKind>,
        result_kind: AttributeResult<'scope>,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let limits = events.limits.source_edits;
        let cancel = &owner.cancellation;
        let event = owner.active_event_request();
        let (mut result, parent) =
            self.callbacks
                .with_qualified_session(owner, event.as_ref(), |session, _| {
                    let mut source = session.source_execution();
                    let mut dom = session.dom();
                    let parent = dom.pending_source_edit().map(DomSourceStatus::operation);
                    let result = source.apply(
                        &mut dom,
                        edit,
                        limits,
                        &mut guest_source_services(&mut |_| None),
                        cancel,
                    );
                    Ok((result, parent))
                })?;
        let realm = scope.realm_authority()?.identity();
        owner.finish_guest_style(&mut result, limits, Some(realm));
        let mut pending = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .read_dom()
            .pending_source_edit();
        let mut abrupt = None;
        let mut output = None;
        if let Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::AttributeVerification(request),
            ..
        }) = result
        {
            let retained = match self.verify_attribute(scope, owner, events, request, brand) {
                Ok(retained) => retained,
                Err(error) => {
                    let cleanup = AtomicBool::new(false);
                    if let Ok(mut session) = owner.session.try_borrow_mut() {
                        let mut dom = session.dom();
                        if owner
                            .pending_attribute
                            .try_borrow()
                            .is_ok_and(|pending| pending.is_none())
                            && let Some(registration) = attachment.registration.upgrade()
                        {
                            let _ = dom
                                .raw_mut()
                                .cancel_interrupted_source_attribute(request, &registration);
                        }
                        if dom
                            .pending_source_edit()
                            .is_some_and(|status| status.operation() == request.operation())
                            && let Ok(mut lease) =
                                dom.resume_source_edit(request.operation(), limits, &cleanup)
                        {
                            lease.request_cancel();
                            let _ = lease.advance_for_runtime(&cleanup);
                        }
                    }
                    return Err(error);
                }
            };
            ((result, pending), abrupt) = continue_attribute(owner, retained, limits, true)?;
        }
        if let Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::AttributeAttachmentOutput(request),
            ..
        }) = result
        {
            let staged = self.stage_attachment_output(scope, owner, events, request);
            if staged.is_err() {
                discard_attachment_output(owner, request);
            }
            output = Some((request.replaced(), staged?));
            let retained = owner
                .pending_attribute
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?
                .take();
            if let Some(retained) = retained {
                ((result, pending), abrupt) = continue_attribute(owner, retained, limits, true)?;
            } else {
                (result, pending) =
                    resume_attribute_source(owner, request.operation(), limits, Some(realm))?;
                if result.is_err() {
                    discard_attachment_output(owner, request);
                }
            }
        }
        match result.map(guest_source_terminal) {
            Ok(Some(Ok(done))) => {
                let reactions = self.drain_native_reactions(scope, owner, events, done.operation());
                if done.cancellation_requested() {
                    return Err(DomError::Cancelled.into());
                }
                reactions?;
                match (result_kind, done.output()) {
                    (AttributeResult::Value, None) => Ok(Completion::Undefined),
                    (
                        AttributeResult::Attached(incoming, _),
                        Some(DomSourceOutput::AttributeAttachment(
                            DomAttributeAttachment::Appended(id),
                        )),
                    ) if id == incoming => {
                        Ok(Completion::Return(scope.primitive(Primitive::Null)?))
                    }
                    (
                        AttributeResult::Attached(incoming, value),
                        Some(DomSourceOutput::AttributeAttachment(
                            DomAttributeAttachment::Unchanged(id),
                        )),
                    )
                    | (
                        AttributeResult::Removed(incoming, value),
                        Some(DomSourceOutput::RemovedAttribute(id)),
                    ) if id == incoming => Ok(Completion::Return(value)),
                    (
                        AttributeResult::Attached(incoming, _),
                        Some(DomSourceOutput::AttributeAttachment(
                            DomAttributeAttachment::Replaced(id),
                        )),
                    ) if id != incoming => match output {
                        Some((staged, value)) if staged == id => Ok(Completion::Return(value)),
                        _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
                    },
                    _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
                }
            }
            Ok(Some(Err(failure))) => {
                let (operation, error, _) = failure.into_parts();
                let reactions = self.drain_native_reactions(scope, owner, events, operation);
                if let (DomError::SourceAttributeVerificationAbrupt, Some(original)) =
                    (&error, abrupt)
                {
                    return Err(original.into());
                }
                if reactions.is_err() {
                    return Err(error.into());
                }
                attribute_failure(scope, error)
            }
            Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)))
                if pending.map(DomSourceStatus::operation) == parent
                    && attribute_exception(&error).is_some() =>
            {
                attribute_failure(scope, error)
            }
            Err(error) => refuse(scope, &attachment.source_failures, error).map_err(Into::into),
            Ok(None) => Err(pending
                .map_or(DocumentGuestEventError::HostRouteUnavailable, |status| {
                    DocumentGuestEventError::SourcePending(status.operation())
                })),
        }
    }

    fn stage_attachment_output<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        request: DomSourceAttributeAttachmentOutput,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let registration = attachment
            .registration
            .upgrade()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let admission =
            ordinary::Admission::attachment_output(owner, events, request, &registration);
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let bindings = &public.document;
        if let Some(value) =
            bindings
                .ordinary
                .open_attachment_output(scope, request.replaced().node(), admission)?
        {
            return Ok(value);
        }
        let global = request
            .creation_global()
            .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?;
        let project = |scope: &mut Scope<'scope>, documents: &DocumentPrototypes| {
            bindings
                .ordinary
                .project_attachment_output(scope, admission, &documents.attr)
        };
        if global == attachment.target.global() {
            project(scope, &bindings.prototypes)
        } else {
            public.with_origin_with(
                global,
                || admission.step(),
                |origin| project(scope, &origin.documents),
            )
        }
    }
}

fn discard_attachment_output(owner: &NativeOwner, request: DomSourceAttributeAttachmentOutput) {
    if let Some(registration) = owner
        .attachment
        .get()
        .and_then(|attachment| attachment.registration.upgrade())
        && let Ok(mut session) = owner.session.try_borrow_mut()
    {
        let _ = session
            .dom()
            .raw_mut()
            .discard_source_attribute_attachment_output(request, &registration);
    }
}

impl Capture {
    fn verify_attribute(
        &self,
        scope: &mut Scope<'_>,
        owner: &NativeOwner,
        events: &GuestEvents,
        request: DomSourceAttributeVerificationRequest,
        brand: Option<TrustedTypeKind>,
    ) -> Result<Box<PendingAttribute>, DocumentGuestEventError> {
        let registration = owner
            .attachment
            .get()
            .and_then(|attachment| attachment.registration.upgrade())
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let processor = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .trusted_types_csp
            .clone();
        let cancel = &owner.cancellation;
        let kind = match request.sink() {
            DomSourceAttributeSink::EventHandler(_) => TrustedTypeKind::Script,
            DomSourceAttributeSink::HtmlIframeSrcdoc => TrustedTypeKind::Html,
            DomSourceAttributeSink::HtmlScriptSrc | DomSourceAttributeSink::SvgScriptHref => {
                TrustedTypeKind::ScriptUrl
            }
        };
        let requires_callback = with_attribute_lease(
            owner,
            events.limits.source_edits,
            request,
            cancel,
            |lease| {
                if brand != Some(kind) {
                    let processor = processor
                        .as_ref()
                        .map(|owner| owner.processor.try_borrow())
                        .transpose()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    let policies = AttributePolicies::read(
                        lease,
                        request,
                        &registration,
                        processor.as_deref(),
                        cancel,
                    )
                    .map_err(attribute_policy_error)?;
                    if policies.requires_default_policy(cancel)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
        )?;
        let header = events.reserve(owner, size_of::<PendingAttribute>())?;
        let mut retained = Box::new(PendingAttribute {
            request,
            realm: scope.realm_authority()?.identity(),
            reply: AttributeReply::Abandon,
            parent: owner.active_attribute.get(),
            interrupted_parent: None,
            _header: header,
        });
        if !requires_callback {
            retained.reply = AttributeReply::OriginalValue;
            return Ok(retained);
        }
        let policy = trusted_policy::current_default_policy(scope, events)?;
        let Some(callback) = policy.as_ref().and_then(|policy| policy.callback(kind)) else {
            retained.reply = AttributeReply::Mismatch(AttributeMismatchProgress::default());
            return Ok(retained);
        };
        let (sink, _sink_backing) = callback_sink(owner, events, request.sink())?;
        let (input, callback_scope) = with_attribute_lease(
            owner,
            events.limits.source_edits,
            request,
            cancel,
            |lease| {
                let input = lease
                    .attribute_verification_input(request, &registration, cancel)?
                    .clone();
                let callback =
                    lease.open_attribute_verification_callback(request, &registration, cancel)?;
                Ok((input, callback))
            },
        )?;
        let active = ActiveAttribute {
            slot: &owner.active_attribute,
            previous: owner.active_attribute.replace(Some(request)),
        };
        let reactions = super::super::super::custom_elements::SynchronousReactionScope::enter(
            owner,
            request.operation(),
        );
        let result = self.callbacks.conversion.run(scope, |conversion| {
            conversion.invoke_default_policy(callback, kind, input.code_units(), &sink)
        });
        drop(reactions);
        drop(active);
        drop(callback_scope);
        retained.reply = match result {
            Ok(result) if result.get().is_some() => AttributeReply::Converted {
                result,
                _account: self.callbacks.clone(),
            },
            Ok(_) => AttributeReply::Mismatch(AttributeMismatchProgress::default()),
            Err(failure) => match Error::from(failure) {
                error @ Error::Exception(_) => AttributeReply::Abrupt(error),
                error => {
                    owner.retain_interrupted_attribute(retained);
                    return Err(error.into());
                }
            },
        };
        Ok(retained)
    }
}

type NativeAttributeProgress = (
    Result<DomSourceDriveProgress, DomSourceDriveError>,
    Option<DomSourceStatus>,
);

impl DocumentScriptHost {
    #[cfg(test)]
    pub(in crate::document_scripting) fn park_live_attribute_write_fixture(
        &mut self,
        permit: &DocumentScriptPermit,
        edit: DomSourceEdit,
    ) -> surgeist_dom::DomSourceOperationId {
        let owner = self.native.clone();
        let events = self.guest_events().unwrap();
        let capture = Capture {
            callbacks: events.public.as_ref().unwrap().callbacks.clone(),
            origin: None,
            _backing: events.reserve(&owner, size_of::<Capture>()).unwrap(),
        };
        self.in_document(permit, |scope| {
            let progress = {
                let mut session = owner.session.borrow_mut();
                session.source_execution().apply(
                    &mut session.dom(),
                    edit,
                    events.limits.source_edits,
                    &mut guest_source_services(&mut |_| None),
                    &owner.cancellation,
                )
            };
            let Ok(DomSourceDriveProgress::Pending {
                request: DomSourceRequest::AttributeVerification(request),
                ..
            }) = progress
            else {
                panic!("expected original attribute verification: {progress:?}");
            };
            let retained = capture
                .verify_attribute(scope, &owner, &events, request, None)
                .unwrap();
            assert!(
                owner
                    .pending_attribute
                    .borrow_mut()
                    .replace(retained)
                    .is_none()
            );
            Ok(request.operation())
        })
        .unwrap()
    }

    /// Resume the original attribute write using its retained policy reply.
    /// The outer error carries admission, the original guest exception, or a
    /// reaction delivery failure. The inner result is the original native
    /// Source progress. A settled write delivers committed reactions when
    /// the original heap is live and the native Source is idle. Microtasks
    /// still run at an explicit checkpoint.
    /// Nested recovery applies the supplied limits monotonically to interrupted
    /// attribute ancestors. Each successful raise survives a later refusal.
    /// Resume again to settle each restored ancestor.
    pub fn resume_pending_attribute_write(
        &mut self,
        permit: &DocumentScriptPermit,
        limits: DomSourceLimits,
    ) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DocumentGuestEventError> {
        self.check_permit(permit)?;
        // A native retry remains possible after a terminal bindings refusal.
        // The permit still authenticates the original native registration.
        let progress = resume_retained_attribute(&self.native, limits, false)?;
        if let Ok(result) = &progress
            && let Some(terminal) = guest_terminal_result(result)
        {
            self.deliver_resumed_source_reactions(permit, terminal)?;
        }
        Ok(progress)
    }
}

impl StoppedDocumentScriptHost {
    /// Settle one original pending attribute write after stopping the engine.
    /// An unaccepted reply is abandoned. An accepted reply retains its native
    /// effects and completes under the supplied Source limits.
    /// Nested recovery applies those limits monotonically to interrupted
    /// attribute ancestors. Each successful raise survives a later refusal.
    /// Settle each before recovering the Session.
    pub fn abandon_pending_attribute_write(
        &mut self,
        limits: DomSourceLimits,
    ) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DocumentGuestEventError> {
        resume_retained_attribute(&self.native, limits, true)
    }
}

fn resume_retained_attribute(
    owner: &NativeOwner,
    limits: DomSourceLimits,
    abandon: bool,
) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DocumentGuestEventError> {
    owner.reconcile_pending_attribute()?;
    let mut pending = owner
        .pending_attribute
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let frame = pending
        .as_ref()
        .ok_or(DocumentScriptNativeError::SourceAccess)?;
    if let Some(mut parent) = frame.interrupted_parent.as_deref() {
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut dom = session.dom();
        loop {
            dom.raw_mut().raise_source_attribute_limits(
                parent.request,
                limits,
                &owner.cancellation,
            )?;
            let Some(next) = parent.interrupted_parent.as_deref() else {
                break;
            };
            parent = next;
        }
    }
    let mut retained = pending
        .take()
        .ok_or(DocumentScriptNativeError::SourceAccess)?;
    drop(pending);
    if abandon && !matches!(retained.reply, AttributeReply::Accepted { .. }) {
        retained.reply = AttributeReply::Abandon;
    }
    let ((progress, _), abrupt) = continue_attribute(owner, retained, limits, !abandon)?;
    if abandon
        && let Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::AttributeAttachmentOutput(request),
            ..
        }) = progress
    {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .dom()
                .raw_mut()
                .abandon_source_attribute_attachment_output(
                    request,
                    &attachment.canonical,
                    &owner.cancellation,
                )?;
        }
        let retained = owner
            .pending_attribute
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .take()
            .ok_or(DocumentScriptNativeError::SourceAccess)?;
        let ((progress, _), _) = continue_attribute(owner, retained, limits, !abandon)?;
        return Ok(progress);
    }
    if let (Ok(DomSourceDriveProgress::Failed(failure)), Some(error)) = (&progress, abrupt)
        && failure.error() == &DomError::SourceAttributeVerificationAbrupt
    {
        return Err(error.into());
    }
    Ok(progress)
}

fn continue_attribute(
    owner: &NativeOwner,
    mut retained: Box<PendingAttribute>,
    limits: DomSourceLimits,
    live: bool,
) -> Result<(NativeAttributeProgress, Option<Error>), DocumentGuestEventError> {
    let request = retained.request;
    let realm = live.then_some(retained.realm);
    let attempted = retained
        .acknowledge(owner, limits)
        .and_then(|()| resume_attribute_source(owner, request.operation(), limits, realm));
    let terminal = matches!(
        &attempted,
        Ok((
            Ok(DomSourceDriveProgress::Complete(_) | DomSourceDriveProgress::Failed(_)),
            _
        ))
    );
    let abrupt = if terminal {
        match &mut retained.reply {
            AttributeReply::Accepted { abrupt } => abrupt.take(),
            _ => None,
        }
    } else {
        None
    };
    let mut pending = owner
        .pending_attribute
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    if pending.is_some() {
        return Err(DocumentScriptNativeError::Busy.into());
    }
    *pending = if terminal {
        retained.interrupted_parent.take()
    } else {
        Some(retained)
    };
    drop(pending);
    owner.reconcile_pending_attribute()?;
    Ok((attempted?, abrupt))
}

impl PendingAttribute {
    fn acknowledge(
        &mut self,
        owner: &NativeOwner,
        limits: DomSourceLimits,
    ) -> Result<(), DocumentGuestEventError> {
        let cancel = &owner.cancellation;
        match &mut self.reply {
            AttributeReply::Accepted { .. } => return Ok(()),
            AttributeReply::Abandon => {
                with_attribute_lease(owner, limits, self.request, cancel, |lease| {
                    lease.request_cancel();
                    Ok(())
                })?;
            }
            AttributeReply::Mismatch(progress) => {
                accept_mismatch(owner, self.request, limits, progress)?;
            }
            reply => {
                let registration = owner
                    .attachment
                    .get()
                    .and_then(|attachment| attachment.registration.upgrade())
                    .ok_or(DocumentScriptNativeError::Revoked)?;
                with_attribute_lease(owner, limits, self.request, cancel, |lease| {
                    match reply {
                        AttributeReply::OriginalValue => {
                            let input = lease
                                .attribute_verification_input(self.request, &registration, cancel)?
                                .clone();
                            let mut outcome =
                                Some(DomSourceAttributeVerificationOutcome::Value(input));
                            lease.accept_attribute_verification(
                                self.request,
                                &registration,
                                &mut outcome,
                                cancel,
                            )?;
                        }
                        AttributeReply::Converted { result, .. } => {
                            let output = result
                                .get()
                                .as_ref()
                                .ok_or(DocumentScriptNativeError::SourceAccess)?;
                            lease.accept_attribute_verification_utf16(
                                self.request,
                                &registration,
                                output.as_code_units(),
                                cancel,
                            )?;
                        }
                        AttributeReply::Abrupt(_) => {
                            lease.accept_attribute_verification(
                                self.request,
                                &registration,
                                &mut Some(DomSourceAttributeVerificationOutcome::Abrupt),
                                cancel,
                            )?;
                        }
                        _ => return Err(DocumentScriptNativeError::SourceAccess.into()),
                    }
                    Ok(())
                })?;
            }
        }
        let previous =
            std::mem::replace(&mut self.reply, AttributeReply::Accepted { abrupt: None });
        if let AttributeReply::Abrupt(error) = previous {
            self.reply = AttributeReply::Accepted {
                abrupt: Some(error),
            };
        }
        Ok(())
    }
}

fn resume_attribute_source(
    owner: &NativeOwner,
    operation: surgeist_dom::DomSourceOperationId,
    limits: DomSourceLimits,
    realm: Option<RealmId>,
) -> Result<NativeAttributeProgress, DocumentGuestEventError> {
    let mut result = {
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        session.source_execution().resume(
            &mut session.dom(),
            operation,
            limits,
            &mut guest_source_services(&mut |_| None),
            &owner.cancellation,
        )
    };
    owner.finish_guest_style(&mut result, limits, realm);
    let session = owner
        .session
        .try_borrow()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    Ok((result, session.read_dom().pending_source_edit()))
}

fn accept_mismatch(
    owner: &NativeOwner,
    request: DomSourceAttributeVerificationRequest,
    limits: DomSourceLimits,
    progress: &mut AttributeMismatchProgress,
) -> Result<(), DocumentGuestEventError> {
    let cancel = &owner.cancellation;
    let registration = owner
        .attachment
        .get()
        .and_then(|attachment| attachment.registration.upgrade())
        .ok_or(DocumentScriptNativeError::Revoked)?;
    let processor = owner
        .session
        .try_borrow()
        .map_err(|_| DocumentScriptNativeError::Busy)?
        .trusted_types_csp
        .clone();
    with_attribute_lease(owner, limits, request, cancel, |lease| {
        let processor = processor
            .as_ref()
            .map(|owner| owner.processor.try_borrow())
            .transpose()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let policies =
            AttributePolicies::read(lease, request, &registration, processor.as_deref(), cancel)
                .map_err(attribute_policy_error)?;
        let mut mismatch = policies.resume_mismatch(progress)?;
        let origin = ViolationOrigin {
            global: request.global(),
            document: policies.document(),
        };
        let mut reports = owner
            .trusted_types_reports
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        loop {
            let next = mismatch.next_report(lease, &registration, cancel, |bytes| {
                ViolationQueue::admit_report(
                    &mut reports,
                    origin,
                    bytes,
                    |bytes, units| {
                        let backing = lease.retain_attribute_verification_payload(
                            request,
                            &registration,
                            bytes,
                            units,
                            cancel,
                        )?;
                        Ok(ViolationBacking::Attribute { _backing: backing })
                    },
                    |work| {
                        if work != 0 {
                            let read = lease.attribute_verification_policy(
                                request,
                                &registration,
                                cancel,
                            )?;
                            for _ in 0..work {
                                read.step(cancel)?;
                            }
                        }
                        Ok(())
                    },
                )
            });
            mismatch.save(progress);
            let Some(report) = next.map_err(violation_error)? else {
                break;
            };
            reports
                .as_mut()
                .ok_or(DocumentScriptNativeError::Busy)?
                .push(report);
        }
        let outcome = match mismatch.blocked() {
            Some(true) => DomSourceAttributeVerificationOutcome::Rejected,
            Some(false) => DomSourceAttributeVerificationOutcome::Value(
                lease
                    .attribute_verification_input(request, &registration, cancel)?
                    .clone(),
            ),
            None => return Err(DomError::InvalidSourceOperation.into()),
        };
        lease.accept_attribute_verification(request, &registration, &mut Some(outcome), cancel)?;
        Ok(())
    })
}

fn attribute_policy_error(error: AttributePolicyError) -> DocumentGuestEventError {
    match error {
        AttributePolicyError::Dom(error) => error.into(),
        _ => DocumentScriptNativeError::TrustedTypesPolicyUnavailable.into(),
    }
}

struct ActiveAttribute<'a> {
    slot: &'a Cell<Option<DomSourceAttributeVerificationRequest>>,
    previous: Option<DomSourceAttributeVerificationRequest>,
}

impl Drop for ActiveAttribute<'_> {
    fn drop(&mut self) {
        self.slot.set(self.previous);
    }
}

fn with_attribute_lease<T>(
    owner: &NativeOwner,
    limits: surgeist_dom::DomSourceLimits,
    request: DomSourceAttributeVerificationRequest,
    cancel: &AtomicBool,
    action: impl FnOnce(&mut DomSourceLease<'_>) -> Result<T, DocumentGuestEventError>,
) -> Result<T, DocumentGuestEventError> {
    let mut session = owner
        .session
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let mut dom = session.dom();
    let mut lease = dom.resume_source_edit(request.operation(), limits, cancel)?;
    action(&mut lease)
}

fn callback_sink(
    owner: &NativeOwner,
    events: &GuestEvents,
    sink: DomSourceAttributeSink,
) -> Result<(Vec<u16>, GuestBacking), DocumentGuestEventError> {
    let (interface, name) = crate::trusted_types_csp::attribute_sink_name(sink);
    let sink = || {
        interface
            .encode_utf16()
            .chain([u16::from(b' ')])
            .chain(name.encode_utf16())
    };
    let count = sink().count();
    let work = u64::try_from(count).map_err(|_| DocumentGuestEventError::WorkLimit)?;
    events.charge_work(work)?;
    let bytes = count
        .checked_mul(size_of::<u16>())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    let backing = events.reserve(owner, bytes)?;
    let mut label = Vec::new();
    label
        .try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    if label.capacity() != count {
        return Err(Error::Allocation.into());
    }
    label.extend(sink());
    Ok((label, backing))
}

fn attribute_exception(error: &DomError) -> Option<DomExceptionName> {
    match error {
        DomError::InvalidCharacter => Some(DomExceptionName::InvalidCharacterError),
        DomError::Namespace => Some(DomExceptionName::NamespaceError),
        DomError::InUseAttribute => Some(DomExceptionName::InUseAttributeError),
        DomError::NotFound => Some(DomExceptionName::NotFoundError),
        _ => None,
    }
}

fn attribute_failure<'scope>(
    scope: &mut Scope<'scope>,
    error: DomError,
) -> Result<Completion<'scope>, DocumentGuestEventError> {
    if error == DomError::SourceAttributeVerificationRejected {
        return Ok(Completion::Throw(
            scope.type_error(&ascii(b"Trusted Types attribute blocked by CSP"))?,
        ));
    }
    match attribute_exception(&error) {
        Some(name) => Ok(Completion::Throw(scope.dom_exception(name, &[])?)),
        None => Err(error.into()),
    }
}
