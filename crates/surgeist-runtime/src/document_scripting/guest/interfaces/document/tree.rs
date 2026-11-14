//! Structural Node methods over the original canonical Source operation.

use super::*;
use crate::{DomSourceDriveError, DomSourceDriveProgress, DomSourceRuntimeError};
use surgeist_bindings_js::{ValueKind, dom_exception::DomExceptionName};
use surgeist_dom::{DomSourceEdit, DomSourceStatus};

mod clone;
mod comparison;
mod namespace;
pub(super) use clone::install_import;
pub(super) use comparison::install_position_constants;

#[derive(Clone, Copy)]
enum Method {
    Append,
    Insert,
    Replace,
    Remove,
}

pub(super) struct GuestTreeDrive {
    pub(super) progress: Result<DomSourceDriveProgress, DomSourceDriveError>,
    pub(super) parent: Option<surgeist_dom::DomSourceOperationId>,
    pub(super) pending: Option<DomSourceStatus>,
}

impl GuestTreeDrive {
    pub(super) fn terminal_output(&self) -> Option<surgeist_dom::DomSourceOutput> {
        match &self.progress {
            Ok(DomSourceDriveProgress::Complete(done)) => done.output(),
            _ => None,
        }
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    clone::install(scope, prototype, capture)?;
    comparison::install(scope, prototype, capture)?;
    namespace::install(scope, prototype, capture)?;
    for (key, method, length) in [
        (&ascii(b"appendChild")[..], Method::Append, 1),
        (&ascii(b"insertBefore")[..], Method::Insert, 2),
        (&ascii(b"replaceChild")[..], Method::Replace, 2),
        (&ascii(b"removeChild")[..], Method::Remove, 1),
    ] {
        let handler = capture(size_of::<(Capture, Method)>())?;
        let callback = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| handler.edit_tree(scope, arguments, method)),
            FunctionMetadata { name: key, length },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(key),
            &callback,
            true,
            true,
            true,
        )?;
    }
    let handler = capture(size_of::<Capture>())?;
    let name = &ascii(b"normalize");
    let callback = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.normalize_node(scope, arguments)),
        FunctionMetadata { name, length: 0 },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(name),
        &callback,
        true,
        true,
        true,
    )?;
    Ok(())
}

pub(in crate::document_scripting::guest::interfaces) fn projected_node<'scope>(
    scope: &mut Scope<'scope>,
    value: &Value<'scope>,
) -> Result<Option<GuestNode>, Error> {
    Ok(scope
        .native_object::<GuestNodeProjection>(value)?
        .and_then(|projection| projection.payload().id()))
}

pub(super) fn install_adoption<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let name = &ascii(b"adoptNode");
    let handler = capture(size_of::<Capture>())?;
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.adopt_node(scope, arguments)),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    pub(super) fn drive_guest_watched_node(
        &self,
        (owner, events): (&NativeOwner, &GuestEvents),
        realm: RealmId,
        edit: DomSourceEdit,
        output: &mut surgeist_dom::DomSourceNodeOutputWatch<'_>,
    ) -> Result<GuestTreeDrive, DocumentGuestEventError> {
        let limits = events.limits.source_edits;
        let event = owner.active_event_request();
        let (mut progress, parent) =
            self.callbacks
                .with_qualified_session(owner, event.as_ref(), |session, _| {
                    let source = session.source_execution();
                    Ok(source.drive_watched_node(
                        &mut session.dom(),
                        Ok(edit),
                        limits,
                        output,
                        &owner.cancellation,
                    ))
                })?;
        owner.finish_watched_node_style(&mut progress, limits, realm, output);
        let pending = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .read_dom()
            .pending_source_edit();
        Ok(GuestTreeDrive {
            progress,
            parent,
            pending,
        })
    }

    fn normalize_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(node) = projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::normalize(node.node()),
                Completion::Undefined,
            )
        })();
        tree_completion(scope, &events, result)
    }

    fn adopt_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) = projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (_, values) = arguments.into_parts();
        let Some(value) = values.into_iter().next() else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let Some(node) = projected_node(scope, &value)? else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Node required"))?,
                ));
            };
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::adopt(node.node(), document.node()),
                Completion::Return(value),
            )
        })();
        tree_completion(scope, &events, result)
    }

    fn edit_tree<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, Error> {
        let Some(parent) = projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let required = if matches!(method, Method::Insert | Method::Replace) {
            2
        } else {
            1
        };
        if arguments.len() < required {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Not enough arguments"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let (_, values) = arguments.into_parts();
            let mut values = values.into_iter();
            let first = values.next().ok_or(Error::BackendUnavailable)?;
            events.step()?;
            let Some(node) = projected_node(scope, &first)? else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Node required"))?,
                ));
            };
            let (edit, returned) = match method {
                Method::Append => (
                    DomSourceEdit::insert_before(node.node(), parent.node(), None),
                    first,
                ),
                Method::Remove => (
                    DomSourceEdit::remove_child(parent.node(), node.node()),
                    first,
                ),
                Method::Insert | Method::Replace => {
                    let second = values.next().ok_or(Error::BackendUnavailable)?;
                    events.step()?;
                    if matches!(method, Method::Insert)
                        && matches!(scope.kind(&second)?, ValueKind::Null | ValueKind::Undefined)
                    {
                        (
                            DomSourceEdit::insert_before(node.node(), parent.node(), None),
                            first,
                        )
                    } else {
                        let Some(child) = projected_node(scope, &second)? else {
                            return Ok(Completion::Throw(
                                scope.type_error(&ascii(b"Node required"))?,
                            ));
                        };
                        match method {
                            Method::Insert => (
                                DomSourceEdit::insert_before(
                                    node.node(),
                                    parent.node(),
                                    Some(child.node()),
                                ),
                                first,
                            ),
                            _ => (
                                DomSourceEdit::replace_child(
                                    node.node(),
                                    parent.node(),
                                    child.node(),
                                ),
                                second,
                            ),
                        }
                    }
                }
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Return(returned))
        })();
        tree_completion(scope, &events, result)
    }

    pub(in crate::document_scripting::guest::interfaces) fn apply_tree_edit<'scope, Backing>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        edit: impl Into<surgeist_dom::DomSourceInput<Backing>>,
        completion: Completion<'scope>,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let drive =
            self.drive_guest_tree_edit(owner, events, scope.realm_authority()?.identity(), edit)?;
        self.finish_guest_tree_edit(
            scope,
            owner,
            events,
            drive.progress,
            (drive.parent, drive.pending),
            |_, _| Ok(completion),
        )
    }

    pub(super) fn drive_guest_tree_edit<Backing>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        realm: RealmId,
        edit: impl Into<surgeist_dom::DomSourceInput<Backing>>,
    ) -> Result<GuestTreeDrive, DocumentGuestEventError> {
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
        owner.finish_guest_tree_policies(&mut result, limits, realm)?;
        let pending = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .read_dom()
            .pending_source_edit();
        Ok(GuestTreeDrive {
            progress: result,
            parent,
            pending,
        })
    }

    pub(super) fn finish_guest_tree_edit<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        result: Result<DomSourceDriveProgress, DomSourceDriveError>,
        (parent, pending): (
            Option<surgeist_dom::DomSourceOperationId>,
            Option<DomSourceStatus>,
        ),
        complete: impl FnOnce(
            &mut Scope<'scope>,
            &surgeist_dom::DomSourceCompletion,
        ) -> Result<Completion<'scope>, DocumentGuestEventError>,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        match result.map(guest_source_terminal) {
            Ok(Some(Ok(done))) => {
                let reactions = self.drain_native_reactions(scope, owner, events, done.operation());
                if done.cancellation_requested() {
                    return Err(DomError::Cancelled.into());
                }
                reactions?;
                complete(scope, &done)
            }
            Ok(Some(Err(failure))) => self.finish_failed_tree_edit(scope, owner, events, failure),
            Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)))
                if pending.map(DomSourceStatus::operation) == parent
                    && tree_exception(&error).is_some() =>
            {
                let name =
                    tree_exception(&error).ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                Ok(Completion::Throw(scope.dom_exception(name, &[])?))
            }
            Err(error) => refuse(scope, &attachment.source_failures, error).map_err(Into::into),
            Ok(None) => Err(pending
                .map_or(DocumentGuestEventError::HostRouteUnavailable, |status| {
                    DocumentGuestEventError::SourcePending(status.operation())
                })),
        }
    }

    fn finish_failed_tree_edit<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        failure: surgeist_dom::DomSourceFailure,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let (operation, error, _) = failure.into_parts();
        let Some(name) = tree_exception(&error) else {
            return Err(error.into());
        };
        let original = scope.dom_exception(name, &[])?;
        type PriorFailure = HostFailureSlot<DocumentGuestEventError, GuestBacking>;
        let prior = PriorFailure::new(
            &scope.realm_authority()?,
            events
                .reserve(
                    owner,
                    PriorFailure::storage_bytes() + 2 * size_of::<usize>(),
                )
                .map_err(|_| DocumentGuestEventError::from(error.clone()))?,
        )?;
        scope.complete_after_host_failure(&prior, error.into(), |scope| {
            self.drain_native_reactions(scope, owner, events, operation)
        })?;
        Ok(Completion::Throw(original))
    }
}

fn tree_exception(error: &DomError) -> Option<DomExceptionName> {
    match error {
        DomError::HierarchyRequest => Some(DomExceptionName::HierarchyRequestError),
        DomError::NotFound => Some(DomExceptionName::NotFoundError),
        DomError::NotSupported => Some(DomExceptionName::NotSupportedError),
        DomError::InvalidCharacter => Some(DomExceptionName::InvalidCharacterError),
        DomError::InvalidAdjacentPosition => Some(DomExceptionName::SyntaxError),
        DomError::IndexSize => Some(DomExceptionName::IndexSizeError),
        _ => None,
    }
}

#[cfg(test)]
impl DocumentScriptHost {
    pub(in crate::document_scripting) fn run_guest_tree_edit_fixture(
        &mut self,
        permit: &DocumentScriptPermit,
        edit: DomSourceEdit,
    ) -> Result<(), DocumentScriptError> {
        let owner = self.native.clone();
        let events = self.guest_events().unwrap();
        let public = events.public.as_ref().unwrap();
        let capture = Capture {
            callbacks: public.callbacks.clone(),
            origin: None,
            _backing: events.reserve(&owner, size_of::<Capture>()).unwrap(),
        };
        self.in_document(permit, |scope| {
            let result =
                capture.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined);
            let Completion::Throw(exception) = tree_completion(scope, &events, result)? else {
                panic!("the original compound edit must throw");
            };
            let name = scope.get(&exception, PropertyKey::String(&ascii(b"name")))?;
            assert_eq!(scope.to_utf16(&name)?, ascii(b"HierarchyRequestError"));
            Ok(())
        })
    }

    pub(in crate::document_scripting) fn run_guest_joined_tree_edit_fixture(
        &mut self,
        permit: &DocumentScriptPermit,
        edit: crate::DomSourceInlineStyleEdit,
    ) -> Result<(), DocumentScriptError> {
        let owner = self.native.clone();
        let events = self.guest_events().unwrap();
        let public = events.public.as_ref().unwrap();
        let capture = Capture {
            callbacks: public.callbacks.clone(),
            origin: None,
            _backing: events.reserve(&owner, size_of::<Capture>()).unwrap(),
        };
        self.in_document(permit, |scope| {
            let mut session = owner.session.borrow_mut();
            let result = session.source_execution().apply_with_inline_styles(
                &mut session.dom(),
                edit,
                events.limits.source_edits,
                &mut guest_source_services(&mut |_| None),
                |_, _| panic!("empty inline declarations require no parser decision"),
                &owner.cancellation,
            );
            drop(session);
            assert!(
                matches!(
                    result,
                    Ok(DomSourceDriveProgress::InlineSource {
                        source: surgeist_dom::DomSourceProgress::Failed(_),
                        ..
                    })
                ),
                "{result:?}"
            );
            let result = capture.finish_guest_tree_edit(
                scope,
                &owner,
                &events,
                result,
                (None, None),
                |_, _| Ok(Completion::Undefined),
            );
            let Completion::Throw(exception) = tree_completion(scope, &events, result)? else {
                panic!("the original compound edit must throw");
            };
            let name = scope.get(&exception, PropertyKey::String(&ascii(b"name")))?;
            assert_eq!(scope.to_utf16(&name)?, ascii(b"HierarchyRequestError"));
            Ok(())
        })
    }

    pub(in crate::document_scripting) fn run_guest_meta_tree_edit_fixture(
        &mut self,
        permit: &DocumentScriptPermit,
        edit: DomSourceEdit,
    ) -> Result<(), DocumentScriptError> {
        let owner = self.native.clone();
        let events = self.guest_events().unwrap();
        let public = events.public.as_ref().unwrap();
        let capture = Capture {
            callbacks: public.callbacks.clone(),
            origin: None,
            _backing: events.reserve(&owner, size_of::<Capture>()).unwrap(),
        };
        self.in_document(permit, |scope| {
            let mut session = owner.session.borrow_mut();
            let result = session.source_execution().apply(
                &mut session.dom(),
                edit,
                events.limits.source_edits,
                &mut crate::DomSourceServices {
                    meta: Some(
                        surgeist_dom::DomMetaAtomicLimits::new(
                            16_000_000, 1_000_000, 1_000_000, 128,
                        )
                        .unwrap(),
                    ),
                    ..guest_source_services(&mut |_| None)
                },
                &owner.cancellation,
            );
            drop(session);
            let Ok(DomSourceDriveProgress::MetaSource(outcome)) = &result else {
                panic!("expected the actual joined Meta result: {result:?}");
            };
            assert!(matches!(
                outcome.source(),
                surgeist_dom::DomSourceProgress::Complete(_)
            ));
            assert_eq!(outcome.preferences().len(), 1);
            assert_eq!(
                outcome.preferences()[0].observation().preferred_name(),
                &surgeist_dom::DocumentString::from("Prefix")
            );
            let result = capture.finish_guest_tree_edit(
                scope,
                &owner,
                &events,
                result,
                (None, None),
                |_, _| Ok(Completion::Undefined),
            );
            assert!(matches!(
                tree_completion(scope, &events, result)?,
                Completion::Undefined
            ));
            Ok(())
        })
    }
}

pub(in crate::document_scripting::guest::interfaces) fn tree_completion<'scope>(
    scope: &mut Scope<'scope>,
    events: &GuestEvents,
    result: Result<Completion<'scope>, DocumentGuestEventError>,
) -> Result<Completion<'scope>, Error> {
    match result {
        Ok(completion) => Ok(completion),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}
