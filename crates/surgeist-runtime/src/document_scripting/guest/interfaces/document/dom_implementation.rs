//! DOMImplementation identity associated with a canonical Document.

use super::*;
use character_data::copy_native_data;
use surgeist_bindings_js::{
    NativeProjectionPrototype, conversion::declarations, dom_exception::DomExceptionName,
};
use surgeist_dom::DomDetachedNodeCreation;

const INTERFACE: &[u16] = &ascii(b"DOMImplementation");
const IMPLEMENTATION_REFERENCE: u32 = 0;

struct Implementation(DocumentId);

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let global = scope.global()?;
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(),
        FunctionMetadata {
            name: INTERFACE,
            length: 0,
        },
    )?;
    scope.define_property(
        &constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: &prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    install_interface(scope, &global, INTERFACE, &constructor, &prototype)?;

    let name = &ascii(b"hasFeature");
    let method = scope.fixed_arity_function(
        &Callback::new(|scope, arguments| {
            if scope
                .native_object::<Implementation>(arguments.receiver())?
                .is_none()
            {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"DOMImplementation required"))?,
                ));
            }
            Ok(Completion::Return(
                scope.primitive(Primitive::Boolean(true))?,
            ))
        }),
        FunctionMetadata { name, length: 0 },
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;

    let factory = (funding.capture)(size_of::<Capture>())?;
    let name = &ascii(b"createDocumentType");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| factory.create_document_type(scope, arguments)),
        FunctionMetadata { name, length: 3 },
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;

    let handler = (funding.capture)(size_of::<Capture>())?;
    install_getter(
        scope,
        document,
        &ascii(b"implementation"),
        &ascii(b"get implementation"),
        true,
        Callback::new(move |scope, arguments| handler.document_implementation(scope, arguments)),
    )?;
    let observed = scope.callback_object(&prototype)?;
    scope.retain_callback_object(observed, funding.retain)
}

impl Capture {
    fn document_implementation<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some(GuestNode::Document(document)) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            if let Some(implementation) = scope.native_object_reference::<GuestNodeProjection>(
                receiver,
                IMPLEMENTATION_REFERENCE,
            )? {
                return Ok(implementation);
            }
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let project = |scope: &mut Scope<'scope>, prototype: &RetainedCallbackObject| {
                let (value, holder, ()) = scope.try_project_retained_native_object(
                    NativeProjectionPrototype::Creation(prototype),
                    Implementation(document),
                    |payload, holder| {
                        Ok::<_, DocumentGuestEventError>((
                            events.reserve(&owner, payload)?,
                            events.reserve(&owner, holder)?,
                        ))
                    },
                    |_| Ok(()),
                )?;
                let installed = scope.set_native_object_reference::<GuestNodeProjection>(
                    receiver,
                    IMPLEMENTATION_REFERENCE,
                    Some(&value),
                );
                let released = scope.release_callback_object(holder);
                installed?;
                released?;
                Ok(value)
            };
            if let Some(origin) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, origin, |origin| {
                    project(scope, &origin.documents.implementation)
                })
            } else {
                project(scope, &public.document.prototypes.implementation)
            }
        })();
        complete(scope, &events, result)
    }

    fn create_document_type<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(receiver) = scope.native_object::<Implementation>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"DOMImplementation required"))?,
            ));
        };
        let document = receiver.payload().0;
        if arguments.len() < 3 {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing document type arguments"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let name = declarations::dom_implementation_create_document_type_0_name();
            let public_id = declarations::dom_implementation_create_document_type_0_public_id();
            let system_id = declarations::dom_implementation_create_document_type_0_system_id();
            let (name, public_id, system_id) = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    Ok((
                        conversion.argument(&name, arguments.get(name.index()))?,
                        conversion.argument(&public_id, arguments.get(public_id.index()))?,
                        conversion.argument(&system_id, arguments.get(system_id.index()))?,
                    ))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let (
                ArgumentValue::Present(name),
                ArgumentValue::Present(public_id),
                ArgumentValue::Present(system_id),
            ) = (name.get(), public_id.get(), system_id.get())
            else {
                return Err(Error::BackendUnavailable.into());
            };
            let (name, _name_backing) = copy_native_data(&owner, &events, name.as_code_units())?;
            let (public_id, _public_id_backing) =
                copy_native_data(&owner, &events, public_id.as_code_units())?;
            let (system_id, _system_id_backing) =
                copy_native_data(&owner, &events, system_id.as_code_units())?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let create = |scope: &mut Scope<'scope>, prototypes: &DocumentPrototypes| {
                public.document.ordinary.create_detached(
                    scope,
                    self,
                    (&owner, &events),
                    ordinary::NodePrototype::Creation(&prototypes.document_type),
                    (
                        document,
                        DomDetachedNodeCreation::DocumentType {
                            name,
                            public_id,
                            system_id,
                        },
                    ),
                )
            };
            if let Some(global) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, global, |origin| create(scope, &origin.documents))
            } else {
                create(scope, &public.document.prototypes)
            }
        })();
        match result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::InvalidCharacter),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::InvalidCharacterError, &[])?,
            )),
            result => complete(scope, &events, result),
        }
    }
}
