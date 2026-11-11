//! Guest execution of an already-selected native Source listener.

use super::*;
use surgeist_bindings_js::{Arguments, Exception, Root};
use surgeist_dom::{
    DomNodeKind, DomNodeRetirementWatch, DomSourceEdit, DomSourceEventPayloadRetention,
    DomSourceEventRequest, DomSourceEventRetention, DomSourceLimits, NodeId,
};

use crate::{
    DomEventCallbackError, DomEventDispatchError, DomEventDispatchOutcome, DomEventDispatcher,
    DomEventHostError, DomEventHostInvocation, DomEventRegistrationId, DomSourceDriveProgress,
    DomSourceEventServiceLimits, DomSourceEventServiceProgress, DomSourceRuntimeError,
    DomSourceServices,
};

#[derive(Clone)]
pub(super) struct ActiveSource {
    request: DomSourceEventRequest,
    delegated: NodeId,
    creation_global: DomHostedGlobalId,
    limits: DomSourceLimits,
    argument: Root,
}

impl ActiveSource {
    pub(super) fn request(&self) -> &DomSourceEventRequest {
        &self.request
    }
}

struct ActiveGuard<'a>(&'a Cell<Option<ActiveSource>>);

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.set(None);
    }
}

/// Failure before a guest listener is selected.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentScriptEventError {
    /// The script owner or its Source capability refused entry.
    Access(DocumentScriptError),
    /// The original native event adapter refused servicing.
    Service(DomSourceRuntimeError),
}

impl fmt::Display for DocumentScriptEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Access(error) => error.fmt(f),
            Self::Service(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for DocumentScriptEventError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Access(error) => Some(error),
            Self::Service(error) => Some(error),
        }
    }
}

impl From<DocumentScriptNativeError> for DocumentScriptEventError {
    fn from(error: DocumentScriptNativeError) -> Self {
        Self::Access(error.into())
    }
}

impl From<Error> for DocumentScriptEventError {
    fn from(error: Error) -> Self {
        Self::Access(error.into())
    }
}

/// Progress from the original native Source event dispatcher.
pub enum DocumentScriptEventProgress<'host> {
    /// The original dispatcher selected an actual host listener.
    Selected(SelectedDocumentScriptListener<'host>),
    /// The existing dispatcher returned its native progress.
    Native(DomSourceEventServiceProgress),
}

fn event_report_bytes() -> usize {
    size_of::<SelectedDocumentScriptListener<'_>>().max(size_of::<DocumentScriptEventReport<'_>>())
}

impl DocumentScriptHost {
    /// Service the exact original event request through the supplied native
    /// registry. A selected listener contains no dispatcher or context loan.
    pub fn select_source_event<C>(
        &mut self,
        permit: &DocumentScriptPermit,
        request: &DomSourceEventRequest,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomSourceEventServiceLimits,
    ) -> Result<DocumentScriptEventProgress<'_>, DocumentScriptEventError> {
        self.check_permit(permit)?;
        self.attachment()?.realm.check_owner_active()?;
        let engine = self
            .engine
            .as_ref()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let bytes = event_report_bytes();
        if engine
            .retained_bytes_with(bytes)
            .and_then(|total| total.checked_add(self.native.stylesheet_bytes.get()))
            .is_none_or(|total| total > self.native.storage_limit)
        {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }

        let cancellation = self.native.cancellation.clone();
        let (progress, backing) = self.with_native(|dom, _, source| {
            let cancel = &cancellation;
            let root = dom
                .pending_source_edit()
                .ok_or(DocumentScriptNativeError::SourceAccess)?;
            if root.operation() != request.operation() {
                return Err(DocumentScriptEventError::from(
                    DocumentScriptNativeError::SourceAccess,
                ));
            }
            dom.raw()
                .source_event_step(request, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            dom.snapshot()
                .document(permit.document)
                .and_then(|document| document.node(request.subject()))
                .map_err(DocumentScriptNativeError::from)?;
            let backing = dom
                .raw()
                .retain_source_event_payload(request, bytes, 0, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let progress = source
                .service_event(dom, request, dispatcher, context, limits, cancel)
                .map_err(DocumentScriptEventError::Service)?;
            Ok((progress, backing))
        })??;

        match progress {
            DomSourceEventServiceProgress::Retained {
                dispatch: DomEventDispatchOutcome::HostInvocation(invocation),
            } => Ok(DocumentScriptEventProgress::Selected(
                SelectedDocumentScriptListener {
                    host: self,
                    invocation,
                    request: request.clone(),
                    _backing: backing,
                },
            )),
            progress => Ok(DocumentScriptEventProgress::Native(progress)),
        }
    }
}

/// The actual selected native listener, before its guest invocation. This
/// token borrows the script owner and carries the original event's custody.
pub struct SelectedDocumentScriptListener<'host> {
    host: &'host mut DocumentScriptHost,
    invocation: DomEventHostInvocation,
    request: DomSourceEventRequest,
    _backing: DomSourceEventRetention,
}

impl<'host> SelectedDocumentScriptListener<'host> {
    /// Caller-owned routing key of the selected native registration.
    pub fn key(&self) -> u64 {
        self.invocation.key()
    }

    /// Exact native registration selected by the original dispatcher.
    pub fn registration(&self) -> DomEventRegistrationId {
        self.invocation.registration()
    }

    /// Invoke the rooted guest handler with the projected direct Document
    /// Comment. The nested detach hook accepts that exact argument only while
    /// this original invocation is active.
    pub fn invoke(
        self,
        handler: &Root,
        delegated: NodeId,
        limits: DomSourceLimits,
    ) -> DocumentScriptEventReport<'host> {
        let Self {
            host,
            invocation,
            request,
            _backing,
        } = self;
        let outcome = match call_guest(host, handler, request, delegated, limits) {
            Ok(()) => DocumentScriptGuestResult::Returned,
            Err(Error::Exception(exception)) => DocumentScriptGuestResult::Threw(exception),
            Err(error) => DocumentScriptGuestResult::Refused(error),
        };
        DocumentScriptEventReport {
            _host: host,
            invocation,
            outcome,
            _backing,
        }
    }
}

/// The exact result selected by guest execution.
#[derive(Debug)]
pub enum DocumentScriptGuestResult {
    /// The guest returned normally.
    Returned,
    /// The original guest exception. Native delivery can continue.
    Threw(Exception),
    /// A terminal refusal for this event, including sticky native failures.
    Refused(Error),
}

/// One selected guest result and its exact native reporting authority. The
/// original dispatcher may accept it once. Its report backing was admitted
/// before the dispatcher selected the listener.
pub struct DocumentScriptEventReport<'host> {
    _host: &'host mut DocumentScriptHost,
    invocation: DomEventHostInvocation,
    outcome: DocumentScriptGuestResult,
    _backing: DomSourceEventRetention,
}

impl DocumentScriptEventReport<'_> {
    /// The selected guest or host result, preserved across report attempts.
    pub fn outcome(&self) -> &DocumentScriptGuestResult {
        &self.outcome
    }

    /// Report through the original native dispatcher. After acceptance, use
    /// the original Source request to continue dispatch and then resume its
    /// operation to run the remaining mandatory tail.
    pub fn report<C>(
        &mut self,
        dispatcher: &DomEventDispatcher<C>,
    ) -> Result<(), DomEventDispatchError> {
        let code = self.invocation.key();
        match self.outcome {
            DocumentScriptGuestResult::Returned => self.invocation.complete(dispatcher, Ok(())),
            DocumentScriptGuestResult::Threw(_) => self
                .invocation
                .complete(dispatcher, Err(DomEventCallbackError { code })),
            DocumentScriptGuestResult::Refused(_) => self
                .invocation
                .refuse(dispatcher, DomEventHostError { code }),
        }
    }
}

fn call_guest(
    host: &mut DocumentScriptHost,
    handler: &Root,
    request: DomSourceEventRequest,
    delegated: NodeId,
    limits: DomSourceLimits,
) -> Result<(), Error> {
    enum PreparedPin {
        Existing(usize),
        Funded {
            plan: projection::SlotPlan,
            watch: DomNodeRetirementWatch,
            backing: DomSourceEventPayloadRetention,
            slots_backing: Option<DomSourceEventPayloadRetention>,
        },
    }

    let native = &host.native;
    let engine = host.engine.as_mut().ok_or(Error::ExpiredIdentity)?;
    let attachment = native.attachment.get().ok_or(Error::ExpiredIdentity)?;
    if native.active_source().is_some() {
        return Err(Error::BackendUnavailable);
    }
    let active = ActiveGuard(&native.active_source);
    let pin_bytes = projection::ProjectedComments::pin_bytes(&engine.process);
    let plan = engine.comments.plan_slot();
    let slot_bytes = plan.and_then(projection::SlotPlan::growth_bytes);
    let miss_bytes = pin_bytes
        .zip(slot_bytes)
        .and_then(|(pin, slots)| pin.checked_add(slots))
        .and_then(|bytes| bytes.checked_add(event_report_bytes()))
        .and_then(|additional| engine.retained_bytes_with(additional))
        .and_then(|total| total.checked_add(native.stylesheet_bytes.get()));
    let comments = &mut engine.comments;
    let receiving_realm = &engine.realm;
    let result = (|| {
        let mut execution = engine.process.execution();
        let (creation_global, creation_realm, prepared) =
            execution.in_realm(receiving_realm, |scope| {
                let prepare = (|| {
                    let mut session = native
                        .session
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    native.qualify_event(attachment, &mut session, &request)?;
                    let creation_global =
                        delegated_comment_global(&session, attachment, delegated)?;
                    let mut dom = session.dom();
                    let step = || {
                        dom.raw()
                            .source_event_step(&request, &native.cancellation)
                            .map_err(DocumentScriptNativeError::from)
                    };
                    let creation_realm = if creation_global == attachment.target.global() {
                        receiving_realm
                    } else {
                        engine
                            .creations
                            .find(creation_global, step)?
                            .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?
                    };
                    step()?;
                    if let Some(existing) = comments.find(delegated, step)? {
                        return Ok((
                            creation_global,
                            creation_realm,
                            PreparedPin::Existing(existing),
                        ));
                    }
                    if miss_bytes.is_none_or(|total| total > native.storage_limit) {
                        return Err(DocumentScriptNativeError::StorageLimit);
                    }
                    let pin_bytes =
                        pin_bytes.ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                    let slot_bytes =
                        slot_bytes.ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                    let plan = plan.ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                    let bytes = pin_bytes
                        .checked_add(slot_bytes)
                        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                    let mut retained = dom.raw().retain_source_event_payload(
                        &request,
                        bytes,
                        0,
                        &native.cancellation,
                    )?;
                    let backing = retained.split_payload(pin_bytes, 0)?;
                    let slots_backing = (slot_bytes != 0)
                        .then(|| retained.split_payload(slot_bytes, 0))
                        .transpose()?;
                    let watch = comments.arm(
                        dom.raw_mut(),
                        &request,
                        delegated,
                        plan,
                        &native.cancellation,
                    )?;
                    Ok((
                        creation_global,
                        creation_realm,
                        PreparedPin::Funded {
                            plan,
                            watch,
                            backing,
                            slots_backing,
                        },
                    ))
                })();
                let prepared = match prepare {
                    Ok(prepared) => prepared,
                    Err(error) => return refuse(scope, &attachment.failures, error),
                };
                scope.import(handler)?;
                Ok(prepared)
            })?;

        let key = match prepared {
            PreparedPin::Existing(key) => key,
            PreparedPin::Funded {
                plan,
                watch,
                backing,
                slots_backing,
            } => {
                if let Err(error) = comments.grow(plan, slots_backing) {
                    return execution.in_realm(receiving_realm, |scope| {
                        refuse(scope, &attachment.failures, error.into())
                    });
                }
                let Some(vacant) = comments.vacant(plan.key) else {
                    return execution.in_realm(receiving_realm, |scope| {
                        refuse(
                            scope,
                            &attachment.failures,
                            DocumentScriptNativeError::ProjectionCustody,
                        )
                    });
                };
                let object = execution.in_realm(creation_realm, |scope| scope.managed_object())?;
                vacant.install(delegated, object, watch, backing);
                plan.key
            }
        };
        execution.in_realm(receiving_realm, |scope| {
            let handler = scope.import(handler)?;
            let Some(object) = comments.object(key) else {
                return refuse(
                    scope,
                    &attachment.failures,
                    DocumentScriptNativeError::ProjectionCustody,
                );
            };
            let argument = scope.open_managed_object(object)?;
            native.active_source.set(Some(ActiveSource {
                request,
                delegated,
                creation_global,
                limits,
                argument: scope.retain(&argument),
            }));
            let receiver = scope.primitive(Primitive::Undefined)?;
            scope.call(&handler, &receiver, &[argument])?;
            Ok(())
        })
    })();
    drop(active);
    engine.finish_projection_cleanup(&attachment.realm);
    result
}

impl NativeOwner {
    pub(super) fn active_source(&self) -> Option<ActiveSource> {
        let active = self.active_source.take()?;
        let observation = active.clone();
        self.active_source.set(Some(active));
        Some(observation)
    }

    pub(super) fn qualify_event(
        &self,
        attachment: &Attachment,
        session: &mut DomSession,
        request: &DomSourceEventRequest,
    ) -> Result<HtmlScriptingStatus, DocumentScriptNativeError> {
        let cancel = &self.cancellation;
        let hosted = session.hosted_execution();
        let mut dom = session.dom();
        let current = dom
            .pending_source_edit()
            .ok_or(DocumentScriptNativeError::SourceAccess)?;
        if current.operation() != request.operation() {
            return Err(DocumentScriptNativeError::SourceAccess);
        }
        let status = hosted.with_script_settings(
            &mut dom,
            attachment.target,
            Some(request),
            cancel,
            |source, dom| {
                source
                    .document_scripting_during_dispatch(
                        dom.raw(),
                        &attachment.canonical,
                        request,
                        DomDocumentScriptingFacts {
                            engine: HtmlScriptingEngine::Supported,
                            policy: attachment.policy,
                        },
                        cancel,
                    )
                    .map_err(DomHostedRuntimeError::Dom)
            },
        )?;
        if status != HtmlScriptingStatus::Enabled {
            return Err(DocumentScriptNativeError::Hosted(
                DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled),
            ));
        }
        let snapshot = dom.snapshot();
        let document = snapshot.document(attachment.target.document())?;
        document.node(request.subject())?;
        Ok(status)
    }
}

pub(super) fn detach_callback(
    native: Weak<NativeOwner>,
    permit: DocumentScriptPermit,
) -> impl for<'scope> Fn(&mut Scope<'scope>, Arguments<'scope>) -> Result<Completion<'scope>, Error>
{
    move |scope, arguments| {
        let native = native.upgrade().ok_or(Error::ExpiredIdentity)?;
        let attachment = native.attachment.get().ok_or(Error::ExpiredIdentity)?;
        if scope.realm_authority()?.identity() != permit.realm {
            return refuse(
                scope,
                &attachment.failures,
                DocumentScriptNativeError::ForeignPermit,
            );
        }
        let Some(active) = native.active_source() else {
            return refuse(
                scope,
                &attachment.failures,
                DocumentScriptNativeError::SourceAccess,
            );
        };
        if let Err(error) = admit_detach_work(&native, &active.request, 1) {
            return refuse(scope, &attachment.failures, error);
        }
        let expected = scope.import(&active.argument)?;
        let matches = match arguments.get(1) {
            Some(candidate) => scope.strict_equals(candidate, &expected)?,
            None => false,
        };
        if !matches {
            return detach_type_error(
                scope,
                &native,
                attachment,
                &active.request,
                b"Delegated Comment required",
            );
        }
        let Some(value) = arguments.get(0) else {
            return detach_type_error(
                scope,
                &native,
                attachment,
                &active.request,
                b"Number required",
            );
        };
        let converted = scope.unrestricted_double(value);
        scope.check_active()?;
        let number = match converted {
            Ok(number) => number,
            Err(error @ Error::Exception(_)) => {
                finish_detach_conversion(scope, &native, attachment, active, false)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        finish_detach_conversion(
            scope,
            &native,
            attachment,
            active,
            number != 0.0 && !number.is_nan(),
        )?;
        Ok(Completion::Undefined)
    }
}

fn admit_detach_work(
    native: &NativeOwner,
    request: &DomSourceEventRequest,
    steps: usize,
) -> Result<(), DocumentScriptNativeError> {
    let mut session = native
        .session
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let dom = session.dom();
    for _ in 0..steps {
        dom.raw().source_event_step(request, &native.cancellation)?;
    }
    Ok(())
}

fn detach_type_error<'scope, const N: usize>(
    scope: &mut Scope<'scope>,
    native: &NativeOwner,
    attachment: &Attachment,
    request: &DomSourceEventRequest,
    ascii: &[u8; N],
) -> Result<Completion<'scope>, Error> {
    if let Err(error) = admit_detach_work(native, request, N) {
        return refuse(scope, &attachment.failures, error);
    }
    let message = ascii.map(u16::from);
    Ok(Completion::Throw(scope.type_error(&message)?))
}

fn delegated_comment_global(
    session: &DomSession,
    attachment: &Attachment,
    delegated: NodeId,
) -> Result<DomHostedGlobalId, DocumentScriptNativeError> {
    let snapshot = session.snapshot();
    let node = snapshot.node(delegated)?;
    if !matches!(node.kind(), DomNodeKind::Comment(_))
        || node.parent() != Some(attachment.target.document().node())
    {
        return Err(DocumentScriptNativeError::SourceAccess);
    }
    snapshot
        .relevant_global(delegated)
        .map_err(DomHostedRuntimeError::Dom)?
        .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)
}

fn finish_detach_conversion(
    scope: &mut Scope<'_>,
    native: &NativeOwner,
    attachment: &Attachment,
    active: ActiveSource,
    should_detach: bool,
) -> Result<(), Error> {
    let performed = (|| {
        let cancel = &native.cancellation;
        let mut session = native
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        native.qualify_event(attachment, &mut session, &active.request)?;
        if !should_detach {
            return Ok(None);
        }
        if delegated_comment_global(&session, attachment, active.delegated)?
            != active.creation_global
        {
            return Err(DocumentScriptNativeError::CreationGlobalUnavailable);
        }
        // The parent request authenticates the receiving Document. Native
        // detach captures its own creation provenance from the exact Comment.
        let mut source = session.source_execution();
        Ok(Some(source.apply(
            &mut session.dom(),
            DomSourceEdit::detach(active.delegated),
            active.limits,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            cancel,
        )))
    })();
    match performed {
        Err(error) => refuse(scope, &attachment.failures, error),
        Ok(Some(Err(error))) => refuse(scope, &attachment.source_failures, error),
        Ok(None | Some(Ok(DomSourceDriveProgress::Complete(_)))) => Ok(()),
        Ok(Some(Ok(DomSourceDriveProgress::Failed(failure)))) => refuse(
            scope,
            &attachment.failures,
            DocumentScriptNativeError::Dom(failure.into_parts().1),
        ),
        Ok(Some(Ok(_))) => refuse(
            scope,
            &attachment.failures,
            DocumentScriptNativeError::SourcePending,
        ),
    }
}
