//! Attached read of native constraint-validation candidacy.

use super::{reflection::Receiver, *};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, Receiver)>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"willValidate"),
        &ascii(b"get willValidate"),
        true,
        Callback::new(move |scope, arguments| handler.will_validate(scope, arguments, receiver)),
    )?;
    Ok(())
}

impl Capture {
    fn will_validate<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let candidate = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                Ok(Some(node.html_will_validate_with(|| read.step())?))
            })?;
            let Some(candidate) = candidate else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(
                scope.primitive(Primitive::Boolean(candidate))?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }
}
