//! DOMImplementation identity associated with a canonical Document.

use super::*;
use crate::{DomSourceDriveError, DomSourceRuntimeError};
use character_data::copy_native_data;
use surgeist_bindings_js::{
    NativeProjectionPrototype, ValueKind, conversion::declarations, dom_exception::DomExceptionName,
};
use surgeist_dom::{DomDetachedNodeCreation, DomSourceEdit, DomSourceOutput, DomSourceStatus};
use surgeist_html::values::Namespace;

const INTERFACE: &[u16] = &ascii(b"DOMImplementation");
const IMPLEMENTATION_REFERENCE: u32 = 0;

struct Implementation(DocumentId);

#[derive(Clone, Copy)]
enum Factory {
    Doctype,
    Xml,
    Html,
}

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

    for (name, length, maximum_arguments, factory) in [
        (&ascii(b"createDocumentType")[..], 3, 3, Factory::Doctype),
        (&ascii(b"createDocument")[..], 2, 3, Factory::Xml),
        (&ascii(b"createHTMLDocument")[..], 0, 1, Factory::Html),
    ] {
        let handler = (funding.capture)(size_of::<(Capture, Factory, u32)>())?;
        let method = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                handler.create(scope, arguments, factory, length)
            }),
            FunctionMetadata { name, length },
            maximum_arguments,
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
    }

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

    fn create<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        factory: Factory,
        length: u32,
    ) -> Result<Completion<'scope>, Error> {
        let Some(receiver) = scope.native_object::<Implementation>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"DOMImplementation required"))?,
            ));
        };
        let document = receiver.payload().0;
        if arguments.len() < length as usize {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing factory arguments"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = match factory {
            Factory::Doctype => {
                self.create_document_type(scope, &arguments, (&owner, &events), document)
            }
            Factory::Xml => {
                self.create_xml_document(scope, &arguments, (&owner, &events), document)
            }
            Factory::Html => {
                self.create_html_document(scope, &arguments, (&owner, &events), document)
            }
        };
        match result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::InvalidCharacter),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::InvalidCharacterError, &[])?,
            )),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::Namespace),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::NamespaceError, &[])?,
            )),
            result => complete(scope, &events, result),
        }
    }

    fn create_document_type<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        document: DocumentId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
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
        let (name, _name_backing) = copy_native_data(owner, events, name.as_code_units())?;
        let (public_id, _public_id_backing) =
            copy_native_data(owner, events, public_id.as_code_units())?;
        let (system_id, _system_id_backing) =
            copy_native_data(owner, events, system_id.as_code_units())?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let create = |scope: &mut Scope<'scope>, prototypes: &DocumentPrototypes| {
            public.document.ordinary.create_detached(
                scope,
                self,
                (owner, events),
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
        if let Some(global) = public.other_document_origin(owner, events, document)? {
            public.with_origin(events, global, |origin| create(scope, &origin.documents))
        } else {
            create(scope, &public.document.prototypes)
        }
    }

    fn create_xml_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        document: DocumentId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let namespace = declarations::dom_implementation_create_document_0_namespace();
        let name = declarations::dom_implementation_create_document_0_qualified_name();
        let (namespace, name) = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                Ok((
                    conversion.argument(&namespace, arguments.get(namespace.index()))?,
                    conversion.argument(&name, arguments.get(name.index()))?,
                ))
            })
            .map_err(Error::from)?;
        let doctype = match arguments.get(2) {
            Some(value)
                if !matches!(scope.kind(value)?, ValueKind::Null | ValueKind::Undefined) =>
            {
                let Some(GuestNode::DocumentType(doctype)) = tree::projected_node(scope, value)?
                else {
                    let error = scope.type_error(&ascii(b"DocumentType required"))?;
                    return Err(Error::Exception(scope.invocation_exception(&error)).into());
                };
                Some(doctype)
            }
            _ => None,
        };
        scope.check_active()?;
        let (ArgumentValue::Present(namespace), ArgumentValue::Present(name)) =
            (namespace.get(), name.get())
        else {
            return Err(Error::BackendUnavailable.into());
        };
        let namespace = namespace
            .as_ref()
            .map(|uri| copy_native_data(owner, events, uri.as_code_units()))
            .transpose()?;
        let uri = namespace
            .as_ref()
            .map_or_else(Namespace::none, |(uri, _)| Namespace::from_uri(uri.clone()));
        let (name, _name_backing) = copy_native_data(owner, events, name.as_code_units())?;
        self.publish_factory_document(
            scope,
            (owner, events),
            (document, DomDocumentInterface::XmlDocument),
            DomSourceEdit::create_xml_document(document.node(), uri, name, doctype),
        )
    }

    fn create_html_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        document: DocumentId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let title = declarations::dom_implementation_create_html_document_0_title();
        let title = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                conversion.argument(&title, arguments.get(title.index()))
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let (title, _title_backing) = match title.get() {
            ArgumentValue::Missing => (None, None),
            ArgumentValue::Present(title) => {
                let (title, backing) = copy_native_data(owner, events, title.as_code_units())?;
                (Some(title), Some(backing))
            }
        };
        self.publish_factory_document(
            scope,
            (owner, events),
            (document, DomDocumentInterface::Document),
            DomSourceEdit::create_html_document(document.node(), title),
        )
    }

    fn publish_factory_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        (document, interface): (DocumentId, DomDocumentInterface),
        edit: DomSourceEdit,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let realm = scope.realm_authority()?.identity();
        let event = owner.active_event_request();
        let attribute = owner.active_attribute_registration()?;
        let constructor = owner.current_constructor_registration()?;
        let admission = match (&constructor, &attribute, &event) {
            (Some(active), _, _) => ordinary::Admission::constructor(owner, events, active),
            (None, Some((request, registration)), _) => {
                ordinary::Admission::attribute(owner, events, *request, registration)
            }
            (None, None, Some(request)) => ordinary::Admission::source(owner, events, request),
            (None, None, None) => ordinary::Admission::ordinary(owner, events),
        };
        let mut completed = None;
        let mut source_failure = None;
        let publish = |scope: &mut Scope<'scope>, prototypes: &DocumentPrototypes| {
            public.document.publish_document(
                scope,
                admission,
                ordinary::NodePrototype::Creation(prototypes.for_document(interface)),
                || {
                    let drive = self.drive_guest_tree_edit(owner, events, realm, edit)?;
                    let pending = drive.pending.map(DomSourceStatus::operation);
                    match drive.progress.map(guest_source_terminal) {
                        Ok(Some(Ok(done))) => {
                            completed = Some((done.operation(), done.cancellation_requested()));
                            match done.output() {
                                Some(DomSourceOutput::Document(document)) => Ok(document),
                                _ if done.cancellation_requested() => {
                                    Err(DomError::Cancelled.into())
                                }
                                _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
                            }
                        }
                        Ok(Some(Err(failure))) => {
                            let (operation, error, _) = failure.into_parts();
                            completed = Some((operation, false));
                            Err(error.into())
                        }
                        Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error)))
                            if pending == drive.parent =>
                        {
                            Err(error.into())
                        }
                        Err(error) => {
                            source_failure = Some(error);
                            Err(DocumentGuestEventError::HostRouteUnavailable)
                        }
                        Ok(None) => Err(pending.map_or(
                            DocumentGuestEventError::HostRouteUnavailable,
                            DocumentGuestEventError::SourcePending,
                        )),
                    }
                },
            )
        };
        let result = if let Some(global) = public.other_document_origin(owner, events, document)? {
            public.with_origin(events, global, |origin| publish(scope, &origin.documents))
        } else {
            publish(scope, &public.document.prototypes)
        };
        if let Some(failure) = source_failure {
            return refuse(scope, &attachment.source_failures, failure).map_err(Into::into);
        }
        if let Some((operation, cancelled)) = completed {
            let reactions = self.drain_native_reactions(scope, owner, events, operation);
            if cancelled {
                return Err(DomError::Cancelled.into());
            }
            if result.is_ok() {
                reactions?;
            }
        }
        result
    }
}
