//! ParentNode.moveBefore over the canonical native Source operation.

use super::{
    tree::{projected_node, tree_completion},
    *,
};
use surgeist_bindings_js::ValueKind;
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
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::move_before(node.node(), parent.node(), before),
                Completion::Undefined,
            )
        })();
        tree_completion(scope, &events, result)
    }
}
