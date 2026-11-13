//! DOMImplementation identity associated with a canonical Document.

use super::*;
use surgeist_bindings_js::NativeProjectionPrototype;

const INTERFACE: &[u16] = &ascii(b"DOMImplementation");
const IMPLEMENTATION_REFERENCE: u32 = 0;

struct Implementation;

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
                    Implementation,
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
}
