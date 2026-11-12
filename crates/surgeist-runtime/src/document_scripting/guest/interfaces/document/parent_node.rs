//! ParentNode.moveBefore over the canonical native Source operation.

use super::*;
use crate::{
    DomSourceDriveError, DomSourceDriveProgress, DomSourceRuntimeError, DomSourceServices,
};
use surgeist_bindings_js::{ValueKind, dom_exception::DomExceptionName};
use surgeist_dom::DomSourceEdit;

const MOVE_BEFORE: &[u16] = &ascii(b"moveBefore");

#[derive(Clone, Copy)]
pub(super) enum Receiver {
    Document,
    Fragment,
    Element,
}

impl Receiver {
    fn accepts(self, node: GuestNode) -> bool {
        matches!(
            (self, node),
            (Self::Document, GuestNode::Document(_))
                | (
                    Self::Fragment,
                    GuestNode::Fragment(_) | GuestNode::ShadowRoot(_)
                )
                | (Self::Element, GuestNode::Element(_))
        )
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, Receiver)>())?;
    let function = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.move_before(scope, arguments, receiver)),
        FunctionMetadata {
            name: MOVE_BEFORE,
            length: 2,
        },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(MOVE_BEFORE),
        &function,
        true,
        true,
        true,
    )?;
    Ok(())
}

fn projected_node<'scope>(
    scope: &mut Scope<'scope>,
    value: &Value<'scope>,
) -> Result<Option<GuestNode>, Error> {
    Ok(scope
        .native_object::<GuestNodeProjection>(value)?
        .and_then(|projection| projection.payload().id()))
}

fn move_exception(error: &DomError) -> Option<DomExceptionName> {
    match error {
        DomError::HierarchyRequest => Some(DomExceptionName::HierarchyRequestError),
        DomError::NotFound => Some(DomExceptionName::NotFoundError),
        _ => None,
    }
}

impl Capture {
    fn move_before<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let Some(parent) =
            projected_node(scope, arguments.receiver())?.filter(|node| receiver.accepts(*node))
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ParentNode receiver required"))?,
            ));
        };
        let (Some(node), Some(reference)) = (arguments.get(0), arguments.get(1)) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"moveBefore requires two arguments"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            events.step()?;
            let Some(node) = projected_node(scope, node)? else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Node required"))?,
                ));
            };
            events.step()?;
            let before = if matches!(
                scope.kind(reference)?,
                ValueKind::Null | ValueKind::Undefined
            ) {
                None
            } else {
                let Some(reference) = projected_node(scope, reference)? else {
                    return Ok(Completion::Throw(
                        scope.type_error(&ascii(b"Node or null required"))?,
                    ));
                };
                Some(reference.node())
            };
            self.apply_move(
                scope,
                &owner,
                &events,
                DomSourceEdit::move_before(node.node(), parent.node(), before),
            )
        })();
        match result {
            Ok(completion) => Ok(completion),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn apply_move<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        edit: DomSourceEdit,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let (result, pending) = self.with_native(owner, |session, _| {
            let mut source = session.source_execution();
            let mut dom = session.dom();
            let result = source.apply(
                &mut dom,
                edit,
                events.limits.source_edits,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &owner.cancellation,
            );
            Ok((result, dom.pending_source_edit()))
        })?;
        match result {
            Ok(DomSourceDriveProgress::Complete(done)) if done.cancellation_requested() => {
                Err(DocumentScriptNativeError::from(DomError::Cancelled).into())
            }
            Ok(DomSourceDriveProgress::Complete(_)) => Ok(Completion::Undefined),
            Ok(DomSourceDriveProgress::Failed(failure)) => {
                let error = failure.into_parts().1;
                match move_exception(&error) {
                    Some(name) => Ok(Completion::Throw(scope.dom_exception(name, &[])?)),
                    None => Err(DocumentScriptNativeError::from(error).into()),
                }
            }
            Err(error) => {
                if pending.is_none()
                    && let DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)) = &error
                    && let Some(name) = move_exception(error)
                {
                    return Ok(Completion::Throw(scope.dom_exception(name, &[])?));
                }
                refuse(scope, &attachment.source_failures, error).map_err(Into::into)
            }
            Ok(_) => Err(
                pending.map_or(DocumentGuestEventError::HostRouteUnavailable, |status| {
                    DocumentGuestEventError::SourcePending(status.operation())
                }),
            ),
        }
    }
}
