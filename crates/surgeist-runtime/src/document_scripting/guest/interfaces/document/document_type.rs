//! Standard DocumentType readback over the original native node.

use super::*;

#[derive(Clone, Copy)]
enum Field {
    Name,
    PublicId,
    SystemId,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    node: &Value<'scope>,
    node_constructor: &Value<'scope>,
    document: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let (_, prototype) = install_derived_interface(
        scope,
        node,
        node_constructor,
        FunctionMetadata {
            name: &ascii(b"DocumentType"),
            length: 0,
        },
        &illegal_constructor(),
    )?;
    for (field, key, name) in [
        (Field::Name, &ascii(b"name")[..], &ascii(b"get name")[..]),
        (
            Field::PublicId,
            &ascii(b"publicId")[..],
            &ascii(b"get publicId")[..],
        ),
        (
            Field::SystemId,
            &ascii(b"systemId")[..],
            &ascii(b"get systemId")[..],
        ),
    ] {
        let capture = (funding.capture)(size_of::<(Capture, Field)>())?;
        install_getter(
            scope,
            &prototype,
            key,
            name,
            true,
            Callback::new(move |scope, arguments| capture.doctype_field(scope, arguments, field)),
        )?;
    }
    let capture = (funding.capture)(size_of::<Capture>())?;
    install_getter(
        scope,
        document,
        &ascii(b"doctype"),
        &ascii(b"get doctype"),
        true,
        Callback::new(move |scope, arguments| capture.document_doctype(scope, arguments)),
    )?;
    let observed = scope.callback_object(&prototype)?;
    scope.retain_callback_object(observed, funding.retain)
}

impl Capture {
    fn doctype_field<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        field: Field,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::DocumentType(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"DocumentType required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let DomNodeKind::DocumentType(doctype) = read.node(id)?.kind() else {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                };
                Ok(match field {
                    Field::Name => doctype.name(),
                    Field::PublicId => doctype.public_id(),
                    Field::SystemId => doctype.system_id(),
                }
                .clone())
            })?;
            self.copy_dom_units(scope, value.code_units())
        })();
        complete(scope, &events, result)
    }

    fn document_doctype<'scope>(
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
        let result = (|| {
            let doctype = self.callbacks.with_read(&owner, &events, |read, _| {
                for &child in read.node(document.node())?.children() {
                    read.step()?;
                    if matches!(read.node(child)?.kind(), DomNodeKind::DocumentType(_)) {
                        return Ok(Some(child));
                    }
                }
                Ok(None)
            })?;
            self.project_observation(scope, &owner, &events, Observation::Node(doctype))
        })();
        complete(scope, &events, result)
    }
}
