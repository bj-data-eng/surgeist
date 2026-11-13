//! Read-only Node comparisons over the current canonical snapshot.

use super::*;

#[derive(Clone, Copy)]
enum Comparison {
    Containment,
    Identity,
    Equality,
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
        let other = if matches!(scope.kind(other)?, ValueKind::Null | ValueKind::Undefined) {
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
            let matches = self.callbacks.with_read(&owner, &events, |read, _| {
                let receiver = read.node(receiver.node())?;
                let Some(other) = other else { return Ok(false) };
                match comparison {
                    Comparison::Containment => receiver.contains_with(other.node(), || read.step()),
                    Comparison::Identity => {
                        let other = read.node(other.node())?;
                        Ok(receiver.id() == other.id())
                    }
                    Comparison::Equality => receiver.is_equal_node_with(
                        other.node(),
                        || read.step(),
                        |bytes| read.reserve(bytes),
                    ),
                }
            })?;
            Ok(scope.primitive(Primitive::Boolean(matches))?)
        })();
        complete(scope, &events, result)
    }
}
