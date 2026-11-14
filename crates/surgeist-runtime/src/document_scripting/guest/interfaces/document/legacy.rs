//! Document's legacy methods over authenticated native receivers.
//!
//! Their [pinned defining algorithm](https://github.com/whatwg/html/blob/877d6146487f15b15bc8dd0e3c625063a37b456a/source#L156302)
//! prescribes no effects after receiver qualification.

use super::*;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for name in [
        &ascii(b"clear")[..],
        &ascii(b"captureEvents")[..],
        &ascii(b"releaseEvents")[..],
    ] {
        let handler = capture(size_of::<Capture>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.legacy_document_operation(scope, arguments)
            }),
            FunctionMetadata { name, length: 0 },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &method,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn legacy_document_operation<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = self.callbacks.with_read(&owner, &events, |read, _| {
            read.node(document.node())?
                .as_document()
                .ok_or(DomError::WrongNodeKind)?;
            Ok(())
        });
        tree::tree_completion(scope, &events, result.map(|()| Completion::Undefined))
    }
}
