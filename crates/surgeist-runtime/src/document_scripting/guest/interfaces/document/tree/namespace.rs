//! Node namespace methods over the canonical native snapshot.

use super::*;
use surgeist_bindings_js::conversion::declarations::{
    node_is_default_namespace_0_namespace, node_lookup_namespace_uri_0_prefix,
    node_lookup_prefix_0_namespace,
};

#[derive(Clone, Copy)]
enum Lookup {
    Prefix,
    Uri,
    Default,
}

enum Observation {
    Prefix(Option<DocumentString>),
    Uri(Option<surgeist_dom::DomNamespaceUri>),
    IsDefault(bool),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, lookup) in [
        (&ascii(b"lookupPrefix")[..], Lookup::Prefix),
        (&ascii(b"lookupNamespaceURI")[..], Lookup::Uri),
        (&ascii(b"isDefaultNamespace")[..], Lookup::Default),
    ] {
        let handler = capture(size_of::<(Capture, Lookup)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.lookup_namespace(scope, arguments, lookup)
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
    fn lookup_namespace<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        lookup: Lookup,
    ) -> Result<Completion<'scope>, Error> {
        let Some(node) = projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let descriptor = match lookup {
                Lookup::Prefix => node_lookup_prefix_0_namespace(),
                Lookup::Uri => node_lookup_namespace_uri_0_prefix(),
                Lookup::Default => node_is_default_namespace_0_namespace(),
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(value) = converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let units = value.as_ref().map(|value| value.as_code_units());
            events.step()?;
            let observation = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(node.node())?;
                Ok(match lookup {
                    Lookup::Prefix => {
                        Observation::Prefix(node.lookup_prefix_with(units, || read.step())?)
                    }
                    Lookup::Uri => {
                        Observation::Uri(node.lookup_namespace_uri_with(units, || read.step())?)
                    }
                    Lookup::Default => Observation::IsDefault(
                        node.is_default_namespace_with(units, || read.step())?,
                    ),
                })
            })?;
            match &observation {
                Observation::Prefix(Some(prefix)) => {
                    self.copy_dom_units(scope, prefix.code_units())
                }
                Observation::Uri(Some(uri)) => self.copy_dom_units(scope, uri.code_units()),
                Observation::Prefix(None) | Observation::Uri(None) => {
                    Ok(scope.primitive(Primitive::Null)?)
                }
                Observation::IsDefault(value) => Ok(scope.primitive(Primitive::Boolean(*value))?),
            }
        })();
        complete(scope, &events, result)
    }
}
