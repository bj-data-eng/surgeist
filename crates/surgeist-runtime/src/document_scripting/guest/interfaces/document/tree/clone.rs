use super::*;
use surgeist_bindings_js::{
    conversion::declarations::{document_import_node_0_options, node_clone_node_0_subtree},
    declarations::ImportNodeArgument,
};
use surgeist_dom::DomSourceOutput;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let name = &ascii(b"cloneNode");
    let handler = capture(size_of::<Capture>())?;
    let callback = scope.bounded_function(
        &Callback::new(move |scope, arguments| handler.clone_node(scope, arguments)),
        FunctionMetadata { name, length: 0 },
        1,
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(name),
        &callback,
        true,
        true,
        true,
    )?;
    Ok(())
}

enum CloneKind {
    Document(DomDocumentInterface),
    Element(DomQualifiedName),
    Ordinary(fn(NodeId) -> GuestNode),
}

pub(in crate::document_scripting::guest::interfaces::document) fn install_import<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let name = &ascii(b"importNode");
    let handler = capture(size_of::<Capture>())?;
    let method = scope.bounded_function(
        &Callback::new(move |scope, arguments| handler.import_node(scope, arguments)),
        FunctionMetadata { name, length: 1 },
        2,
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn import_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(destination)) = projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let Some(input) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let Some(node) = projected_node(scope, input)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let descriptor = document_import_node_0_options();
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(options) = converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            let (subtree, registry) = match options {
                ImportNodeArgument::Subtree(subtree) => (*subtree, None),
                ImportNodeArgument::Options(options) => {
                    (!options.self_only, options.custom_element_registry)
                }
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let facts = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let receiving = read.node(destination.node())?;
                    let source = read.node(node.node())?;
                    node.node_type(source.kind())?;
                    if matches!(node, GuestNode::Document(_) | GuestNode::ShadowRoot(_)) {
                        return Ok(None);
                    }
                    let registry = registry
                        .map(|identity| public.native_registry(read, identity))
                        .transpose()?;
                    let origin = receiving
                        .relevant_global()
                        .map_err(DomHostedRuntimeError::Dom)
                        .map_err(DocumentScriptNativeError::from)?;
                    let kind = ordinary_clone_kind(source.kind())?;
                    Ok(Some((
                        origin.filter(|origin| *origin != attached),
                        registry,
                        kind,
                    )))
                })?;
            let Some((origin, registry, kind)) = facts else {
                return Ok(Completion::Throw(
                    scope.dom_exception(DomExceptionName::NotSupportedError, &[])?,
                ));
            };
            let edit = DomSourceEdit::import_live_node(
                node.node(),
                destination,
                subtree,
                registry,
                creation_realm_fallback(&owner)?,
            );
            self.publish_copied_node(scope, (&owner, &events), (node, origin, kind), edit)
        })();
        tree_completion(scope, &events, result)
    }

    fn clone_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(node) = projected_node(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let descriptor = node_clone_node_0_subtree();
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(subtree) = converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            if matches!(node, GuestNode::ShadowRoot(_)) {
                self.callbacks.with_read(&owner, &events, |read, _| {
                    node.node_type(read.node(node.node())?.kind())?;
                    Ok(())
                })?;
                return Ok(Completion::Throw(
                    scope.dom_exception(DomExceptionName::NotSupportedError, &[])?,
                ));
            }
            let (origin, kind) = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let native = read.node(node.node())?;
                    node.node_type(native.kind())?;
                    let (origin_document, kind) = match native.kind() {
                        DomNodeKind::Document(metadata) => {
                            (native, CloneKind::Document(metadata.interface()))
                        }
                        kind => {
                            let document = native
                                .owner_document()
                                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                            let document = read.node(document.node())?;
                            let kind = ordinary_clone_kind(kind)?;
                            (document, kind)
                        }
                    };
                    let origin = origin_document
                        .relevant_global()
                        .map_err(DomHostedRuntimeError::Dom)
                        .map_err(DocumentScriptNativeError::from)?;
                    Ok((origin.filter(|origin| *origin != attached), kind))
                })?;
            let edit = DomSourceEdit::clone_node_in_attachment(
                node.node(),
                *subtree,
                creation_realm_fallback(&owner)?,
            );
            self.publish_copied_node(scope, (&owner, &events), (node, origin, kind), edit)
        })();
        tree_completion(scope, &events, result)
    }

    fn publish_copied_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        (node, origin, kind): (GuestNode, Option<DomHostedGlobalId>, CloneKind),
        edit: DomSourceEdit,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let realm = scope.realm_authority()?.identity();
        let mut driven = None;
        let project =
            |documents: &DocumentPrototypes, elements: &element::ElementBindings| match kind {
                CloneKind::Document(interface) => {
                    let event = owner.active_event_request();
                    let attribute = owner.active_attribute_registration()?;
                    let constructor = owner.current_constructor_registration()?;
                    let admission = match (&constructor, &attribute, &event) {
                        (Some(active), _, _) => {
                            ordinary::Admission::constructor(owner, events, active)
                        }
                        (None, Some((request, registration)), _) => {
                            ordinary::Admission::attribute(owner, events, *request, registration)
                        }
                        (None, None, Some(request)) => {
                            ordinary::Admission::source(owner, events, request)
                        }
                        (None, None, None) => ordinary::Admission::ordinary(owner, events),
                    };
                    public.document.publish_document(
                        scope,
                        admission,
                        ordinary::NodePrototype::Creation(documents.for_document(interface)),
                        || {
                            let drive = self.drive_guest_tree_edit(owner, events, realm, edit)?;
                            let document = match drive.terminal_output() {
                                Some(DomSourceOutput::Document(document)) => Some(document),
                                _ => None,
                            };
                            driven = Some(drive);
                            document
                                .ok_or_else(|| DocumentScriptNativeError::ProjectionCustody.into())
                        },
                    )
                }
                kind => {
                    let (prototype, project) = match kind {
                        CloneKind::Element(name) => (
                            elements.prototype_for_name(&name, || events.step())?,
                            GuestNode::Element as fn(NodeId) -> GuestNode,
                        ),
                        CloneKind::Ordinary(project) => {
                            let prototype = match node {
                                GuestNode::Attr(_) => &documents.attr,
                                GuestNode::DocumentType(_) => &documents.document_type,
                                GuestNode::Fragment(_) => &documents.fragment,
                                GuestNode::Text(_) => documents
                                    .character_data
                                    .prototype(character_data::CharacterKind::Text),
                                GuestNode::Comment(_) => documents
                                    .character_data
                                    .prototype(character_data::CharacterKind::Comment),
                                GuestNode::ProcessingInstruction(_) => {
                                    &documents.character_data.processing_instruction
                                }
                                _ => {
                                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                                }
                            };
                            (prototype, project)
                        }
                        CloneKind::Document(_) => {
                            return Err(DocumentScriptNativeError::ProjectionCustody.into());
                        }
                    };
                    public.document.ordinary.project_source_node(
                        scope,
                        (owner, events),
                        prototype,
                        |output| {
                            let drive = self.drive_guest_watched_node(
                                (owner, events),
                                realm,
                                edit,
                                output,
                            )?;
                            let node = match drive.terminal_output() {
                                Some(DomSourceOutput::Node(node)) => Some(project(node)),
                                _ => None,
                            };
                            driven = Some(drive);
                            node.ok_or_else(|| DocumentScriptNativeError::ProjectionCustody.into())
                        },
                    )
                }
            };
        let projection = if let Some(global) = origin {
            public.with_origin(events, global, |origin| {
                project(&origin.documents, &origin.elements)
            })
        } else {
            project(&public.document.prototypes, &public.document.element)
        };
        let Some(drive) = driven else {
            return projection.map(Completion::Return);
        };
        self.finish_guest_tree_edit(
            scope,
            owner,
            events,
            drive.progress,
            (drive.parent, drive.pending),
            |_, _| projection.map(Completion::Return),
        )
    }
}

fn ordinary_clone_kind(kind: DomNodeKind<'_>) -> Result<CloneKind, DocumentGuestEventError> {
    let project = match kind {
        DomNodeKind::Element(value) => return Ok(CloneKind::Element(value.name().clone())),
        DomNodeKind::Attr(_) => GuestNode::Attr,
        DomNodeKind::DocumentType(_) => GuestNode::DocumentType,
        DomNodeKind::DocumentFragment => GuestNode::Fragment,
        DomNodeKind::Text(_) => GuestNode::Text,
        DomNodeKind::Comment(_) => GuestNode::Comment,
        DomNodeKind::ProcessingInstruction(_) => GuestNode::ProcessingInstruction,
        _ => return Err(DocumentScriptNativeError::ProjectionCustody.into()),
    };
    Ok(CloneKind::Ordinary(project))
}
