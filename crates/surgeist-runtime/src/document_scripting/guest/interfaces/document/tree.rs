//! Structural Node methods over the original canonical Source operation.

use super::*;
use crate::{DomSourceDriveError, DomSourceDriveProgress, DomSourceRuntimeError};
use surgeist_bindings_js::{ValueKind, dom_exception::DomExceptionName};
use surgeist_dom::{DomSourceEdit, DomSourceStatus};

#[derive(Clone, Copy)]
enum Method {
    Append,
    Insert,
    Replace,
    Remove,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
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
    Ok(())
}

pub(super) fn projected_node<'scope>(
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

    pub(super) fn apply_tree_edit<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        edit: DomSourceEdit,
        completion: Completion<'scope>,
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
        owner.finish_guest_tree_policies(
            &mut result,
            limits,
            scope.realm_authority()?.identity(),
        )?;
        let pending = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .read_dom()
            .pending_source_edit();
        match result {
            Ok(DomSourceDriveProgress::Complete(done)) if done.cancellation_requested() => {
                Err(DomError::Cancelled.into())
            }
            Ok(DomSourceDriveProgress::Complete(done)) => {
                self.drain_native_reactions(scope, owner, events, done.operation())?;
                Ok(completion)
            }
            Ok(DomSourceDriveProgress::Failed(failure)) => {
                let error = failure.into_parts().1;
                match tree_exception(&error) {
                    Some(name) => Ok(Completion::Throw(scope.dom_exception(name, &[])?)),
                    None => Err(error.into()),
                }
            }
            Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)))
                if pending.map(DomSourceStatus::operation) == parent
                    && tree_exception(&error).is_some() =>
            {
                let name =
                    tree_exception(&error).ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                Ok(Completion::Throw(scope.dom_exception(name, &[])?))
            }
            Err(error) => refuse(scope, &attachment.source_failures, error).map_err(Into::into),
            Ok(_) => Err(
                pending.map_or(DocumentGuestEventError::HostRouteUnavailable, |status| {
                    DocumentGuestEventError::SourcePending(status.operation())
                }),
            ),
        }
    }
}

fn tree_exception(error: &DomError) -> Option<DomExceptionName> {
    match error {
        DomError::HierarchyRequest => Some(DomExceptionName::HierarchyRequestError),
        DomError::NotFound => Some(DomExceptionName::NotFoundError),
        DomError::NotSupported => Some(DomExceptionName::NotSupportedError),
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
}

pub(super) fn tree_completion<'scope>(
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
