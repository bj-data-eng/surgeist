//! NonElementParentNode lookup over the current ordinary tree.

use super::{tree_mutations::Receiver, *};
use surgeist_bindings_js::conversion::declarations::non_element_parent_node_get_element_by_id_0_element_id;

const GET_ELEMENT_BY_ID: &[u16] = &ascii(b"getElementById");

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    fragment: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (receiver, prototype) in [
        (Receiver::Document, document),
        (Receiver::Fragment, fragment),
    ] {
        let handler = capture(size_of::<(Capture, Receiver)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.get_element_by_id(scope, arguments, receiver)
            }),
            FunctionMetadata {
                name: GET_ELEMENT_BY_ID,
                length: 1,
            },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(GET_ELEMENT_BY_ID),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn get_element_by_id<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let Some(root) = tree::projected_node(scope, arguments.receiver())?
            .filter(|node| receiver.accepts(*node))
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"getElementById receiver required"))?,
            ));
        };
        let declaration = non_element_parent_node_get_element_by_id_0_element_id();
        let Some(id) = arguments.get(declaration.index()) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"getElementById requires an ID"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let id = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&declaration, Some(id))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(id) = id.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let element = self.callbacks.with_read(&owner, &events, |read, _| {
                let root = read.node(root.node())?;
                root.descendant_element_by_id_with(
                    id.as_code_units(),
                    || read.step(),
                    |bytes| read.reserve(bytes),
                )
            })?;
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(element),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }
}
