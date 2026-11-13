//! Read-only Node comparisons over the current canonical snapshot.

use super::*;
use surgeist_dom::DomDocumentPosition;

#[derive(Clone, Copy)]
enum Comparison {
    Containment,
    Identity,
    Equality,
    Position,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, comparison) in [
        (&ascii(b"contains")[..], Comparison::Containment),
        (&ascii(b"isSameNode")[..], Comparison::Identity),
        (&ascii(b"isEqualNode")[..], Comparison::Equality),
        (&ascii(b"compareDocumentPosition")[..], Comparison::Position),
    ] {
        let handler = capture(size_of::<(Capture, Comparison)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.compare_nodes(scope, arguments, comparison)
            }),
            FunctionMetadata { name, length: 1 },
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

pub(in crate::document_scripting::guest::interfaces::document) fn install_position_constants<
    'scope,
>(
    scope: &mut Scope<'scope>,
    constructor: &Value<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    for (name, bits) in [
        (
            &ascii(b"DOCUMENT_POSITION_DISCONNECTED")[..],
            DomDocumentPosition::DISCONNECTED,
        ),
        (
            &ascii(b"DOCUMENT_POSITION_PRECEDING")[..],
            DomDocumentPosition::PRECEDING,
        ),
        (
            &ascii(b"DOCUMENT_POSITION_FOLLOWING")[..],
            DomDocumentPosition::FOLLOWING,
        ),
        (
            &ascii(b"DOCUMENT_POSITION_CONTAINS")[..],
            DomDocumentPosition::CONTAINS,
        ),
        (
            &ascii(b"DOCUMENT_POSITION_CONTAINED_BY")[..],
            DomDocumentPosition::CONTAINED_BY,
        ),
        (
            &ascii(b"DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC")[..],
            DomDocumentPosition::IMPLEMENTATION_SPECIFIC,
        ),
    ] {
        let value = scope.primitive(Primitive::Number(f64::from(bits)))?;
        for target in [constructor, prototype] {
            define_data(
                scope,
                target,
                PropertyKey::String(name),
                &value,
                false,
                true,
                false,
            )?;
        }
    }
    Ok(())
}

impl Capture {
    fn compare_nodes<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        comparison: Comparison,
    ) -> Result<Completion<'scope>, Error> {
        let Some(receiver) = projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let Some(other) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Not enough arguments"))?,
            ));
        };
        let other = if !matches!(comparison, Comparison::Position)
            && matches!(scope.kind(other)?, ValueKind::Null | ValueKind::Undefined)
        {
            None
        } else if let Some(node) = projected_node(scope, other)? {
            Some(node)
        } else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let result = self.callbacks.with_read(&owner, &events, |read, _| {
                let receiver = read.node(receiver.node())?;
                let Some(other) = other else {
                    return Ok(Primitive::Boolean(false));
                };
                let matches = match comparison {
                    Comparison::Containment => {
                        receiver.contains_with(other.node(), || read.step())?
                    }
                    Comparison::Identity => {
                        let other = read.node(other.node())?;
                        receiver.id() == other.id()
                    }
                    Comparison::Equality => receiver.is_equal_node_with(
                        other.node(),
                        || read.step(),
                        |bytes| read.reserve(bytes),
                    )?,
                    Comparison::Position => {
                        let position = receiver
                            .compare_document_position_with(other.node(), || read.step())?;
                        return Ok(Primitive::Number(f64::from(position.bits())));
                    }
                };
                Ok(Primitive::Boolean(matches))
            })?;
            Ok(scope.primitive(result)?)
        })();
        complete(scope, &events, result)
    }
}
