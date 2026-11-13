//! Text and Comment creation and CharacterData access over the canonical DOM.

use super::*;
mod mutations;
mod split_text;
use surgeist_bindings_js::{
    DomString, Root,
    conversion::{
        Argument, Converted, Type,
        declarations::{
            comment_constructor_0_data, document_create_comment_0_data,
            document_create_text_node_0_data, text_constructor_0_data,
        },
    },
};
use surgeist_dom::{DomDetachedNodeCreation, DomSourceEventRequest};

const CHARACTER_DATA: &[u16] = &ascii(b"CharacterData");
const TEXT: &[u16] = &ascii(b"Text");
const COMMENT: &[u16] = &ascii(b"Comment");
const CREATE_TEXT: &[u16] = &ascii(b"createTextNode");
const CREATE_COMMENT: &[u16] = &ascii(b"createComment");

#[derive(Clone, Copy)]
pub(super) enum CharacterKind {
    Text,
    Comment,
}

impl CharacterKind {
    fn name(self) -> &'static [u16] {
        match self {
            Self::Text => TEXT,
            Self::Comment => COMMENT,
        }
    }

    fn factory(self) -> (&'static [u16], Argument<Type<DomString>>) {
        match self {
            Self::Text => (CREATE_TEXT, document_create_text_node_0_data()),
            Self::Comment => (CREATE_COMMENT, document_create_comment_0_data()),
        }
    }

    fn constructor_argument(self) -> Argument<Type<DomString>> {
        match self {
            Self::Text => text_constructor_0_data(),
            Self::Comment => comment_constructor_0_data(),
        }
    }

    fn creation(self, data: DocumentString) -> DomDetachedNodeCreation {
        match self {
            Self::Text => DomDetachedNodeCreation::Text(data),
            Self::Comment => DomDetachedNodeCreation::Comment(data),
        }
    }

    pub(super) fn project(self, node: NodeId) -> GuestNode {
        match self {
            Self::Text => GuestNode::Text(node),
            Self::Comment => GuestNode::Comment(node),
        }
    }
}

pub(super) struct CharacterBindings {
    text: RetainedCallbackObject,
    comment: RetainedCallbackObject,
    pub(super) processing_instruction: RetainedCallbackObject,
}

impl CharacterBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        node_constructor: &Value<'scope>,
        document: &Value<'scope>,
        funding: &OriginInterfaceFunding<'_>,
        constructors: Option<CaptureFunding<'_>>,
    ) -> Result<Self, DocumentGuestEventError> {
        let (interface, prototype) = install_derived_interface(
            scope,
            node,
            node_constructor,
            FunctionMetadata {
                name: CHARACTER_DATA,
                length: 0,
            },
            &illegal_constructor(),
        )?;
        for (length, key, name) in [
            (false, &ascii(b"data")[..], &ascii(b"get data")[..]),
            (true, &ascii(b"length")[..], &ascii(b"get length")[..]),
        ] {
            let getter = (funding.capture)(size_of::<(Capture, bool)>())?;
            let get = scope.fixed_arity_function(
                &Callback::new(move |scope, arguments| {
                    getter.character_member(scope, arguments, length)
                }),
                FunctionMetadata { name, length: 0 },
            )?;
            let set = if length {
                None
            } else {
                Some(mutations::data_setter(scope, &funding.capture)?)
            };
            scope.define_new_property(
                &prototype,
                PropertyKey::String(key),
                PropertyDescriptor::Accessor {
                    get: Some(&get),
                    set: set.as_ref(),
                    enumerable: true,
                    configurable: true,
                },
            )?;
        }
        mutations::install_methods(scope, &prototype, &funding.capture)?;
        let names = tree_mutations::unscopables(scope, &prototype)?;
        tree_mutations::install_child(
            scope,
            &prototype,
            tree_mutations::Receiver::CharacterData,
            &names,
            funding.capture,
        )?;
        let text = Self::install_kind(
            scope,
            &interface,
            &prototype,
            document,
            CharacterKind::Text,
            funding,
            constructors,
        )?;
        let comment = Self::install_kind(
            scope,
            &interface,
            &prototype,
            document,
            CharacterKind::Comment,
            funding,
            constructors,
        )?;
        let constructor = processing_instruction::constructor(funding.capture)?;
        let (_, instruction) = install_derived_interface(
            scope,
            &prototype,
            &interface,
            FunctionMetadata {
                name: &ascii(b"ProcessingInstruction"),
                length: 1,
            },
            &constructor,
        )?;
        processing_instruction::install_factory(scope, document, funding.capture)?;
        processing_instruction::install_pseudo_attributes(scope, &instruction, funding.capture)?;
        let capture = (funding.capture)(size_of::<Capture>())?;
        install_getter(
            scope,
            &instruction,
            &ascii(b"target"),
            &ascii(b"get target"),
            true,
            Callback::new(move |scope, arguments| {
                capture.processing_instruction_target(scope, arguments)
            }),
        )?;
        let observed = scope.callback_object(&instruction)?;
        let processing_instruction = scope.retain_callback_object(observed, funding.retain)?;
        Ok(Self {
            text,
            comment,
            processing_instruction,
        })
    }

    fn install_kind<'scope>(
        scope: &mut Scope<'scope>,
        parent_constructor: &Value<'scope>,
        parent_prototype: &Value<'scope>,
        document: &Value<'scope>,
        kind: CharacterKind,
        funding: &OriginInterfaceFunding<'_>,
        constructors: Option<CaptureFunding<'_>>,
    ) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
        let callback = if let Some(capture) = constructors {
            let fallback = capture(size_of::<(Capture, CharacterKind)>())?;
            let construct = capture(size_of::<(Capture, CharacterKind)>())?;
            ConstructorCallback::bounded(
                1,
                move |scope| {
                    let (_, events) = fallback.active(scope)?;
                    let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                    scope.open_callback_object(
                        public.document.prototypes.character_data.prototype(kind),
                    )
                },
                move |scope, arguments| construct.construct_character(scope, arguments, kind),
            )
        } else {
            illegal_constructor()
        };
        let (_, prototype) = install_derived_interface(
            scope,
            parent_prototype,
            parent_constructor,
            FunctionMetadata {
                name: kind.name(),
                length: 0,
            },
            &callback,
        )?;
        if matches!(kind, CharacterKind::Text) {
            split_text::install(scope, &prototype, funding.capture)?;
            text::install_whole_text(scope, &prototype, funding.capture)?;
        }
        let (name, _) = kind.factory();
        let factory = (funding.capture)(size_of::<(Capture, CharacterKind)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                factory.create_character(scope, arguments, kind)
            }),
            FunctionMetadata { name, length: 1 },
        )?;
        define_data(
            scope,
            document,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
        let observed = scope.callback_object(&prototype)?;
        scope.retain_callback_object(observed, funding.retain)
    }

    pub(super) fn prototype(&self, kind: CharacterKind) -> &RetainedCallbackObject {
        match kind {
            CharacterKind::Text => &self.text,
            CharacterKind::Comment => &self.comment,
        }
    }
}

impl GuestEvents {
    pub(in crate::document_scripting) fn has_public_node_projection(&self) -> bool {
        self.public.is_some()
    }

    pub(in crate::document_scripting) fn project_source_comment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, attachment): (&NativeOwner, &Attachment),
        request: &DomSourceEventRequest,
        node: NodeId,
        origin: DomHostedGlobalId,
        handler: &Root,
    ) -> Result<Value<'scope>, Error> {
        let admission = ordinary::Admission::source(owner, self, request);
        let result = (|| {
            let public = self.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let ordinary = &public.document.ordinary;
            if let Some(value) = ordinary.open_with(scope, node, admission)? {
                return Ok(value);
            }
            let mut watch = |prototypes: &DocumentPrototypes| {
                // Root import only validates the owner and opens a local
                // reference. Defer its result until native watch admission so
                // the same first refusal wins in either Source route.
                let handler = scope.import(handler).map(drop);
                ordinary.watch_source(
                    scope,
                    (owner, self),
                    request,
                    GuestNode::Comment(node),
                    prototypes.character_data.prototype(CharacterKind::Comment),
                    handler,
                )
            };
            if origin == attachment.target.global() {
                watch(&public.document.prototypes)
            } else {
                public.with_origin_with(
                    origin,
                    || admission.step(),
                    |record| watch(&record.documents),
                )
            }
        })();
        match result {
            Ok(value) => Ok(value),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(error))) => {
                refuse(scope, &attachment.failures, error)
            }
            Err(error) => refuse(scope, &self.refusal, error),
        }
    }
}

impl Capture {
    fn processing_instruction_target<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::ProcessingInstruction(id)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ProcessingInstruction required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let target = self.callbacks.with_read(&owner, &events, |read, _| {
                let DomNodeKind::ProcessingInstruction(instruction) = read.node(id)?.kind() else {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                };
                Ok(instruction.target().clone())
            })?;
            self.copy_dom_units(scope, target.code_units())
        })();
        complete(scope, &events, result)
    }

    fn construct_character<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
        kind: CharacterKind,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let descriptor = kind.constructor_argument();
            let data =
                self.character_argument(scope, &descriptor, arguments.get(descriptor.index()))?;
            let selected = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let document = owner
                .attachment
                .get()
                .ok_or(DocumentScriptNativeError::Revoked)?
                .target
                .document();
            self.publish_character(
                scope,
                &owner,
                &events,
                ordinary::NodePrototype::Selected(selected.prototype()),
                (kind, document),
                &data,
            )
        })();
        match result {
            Ok(value) => Ok(ConstructorCompletion::Return(value)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn create_character<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        kind: CharacterKind,
    ) -> Result<Completion<'scope>, Error> {
        let projection = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Document(document)) = projection.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let (_, descriptor) = kind.factory();
            let data =
                self.character_argument(scope, &descriptor, arguments.get(descriptor.index()))?;
            scope.check_active()?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let mut project = |prototypes: &DocumentPrototypes| {
                self.publish_character(
                    scope,
                    &owner,
                    &events,
                    ordinary::NodePrototype::Creation(prototypes.character_data.prototype(kind)),
                    (kind, document),
                    &data,
                )
            };
            if let Some(global) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, global, |origin| project(&origin.documents))
            } else {
                project(&public.document.prototypes)
            }
        })();
        complete(scope, &events, result)
    }

    fn character_argument<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        descriptor: &Argument<Type<DomString>>,
        input: Option<&Value<'scope>>,
    ) -> Result<Converted<ArgumentValue<DomString>>, Error> {
        self.callbacks
            .conversion
            .run(scope, |conversion| conversion.argument(descriptor, input))
            .map_err(Into::into)
    }

    fn publish_character<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        prototype: ordinary::NodePrototype<'_, 'scope>,
        (kind, document): (CharacterKind, DocumentId),
        data: &Converted<ArgumentValue<DomString>>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let ArgumentValue::Present(data) = data.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let (data, _backing) = copy_native_data(owner, events, data.as_code_units())?;
        let ordinary = &events
            .public
            .as_ref()
            .ok_or(Error::ExpiredIdentity)?
            .document
            .ordinary;
        ordinary.create_detached(
            scope,
            self,
            (owner, events),
            prototype,
            (document, kind.creation(data)),
        )
    }

    fn character_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        length: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(id) = character_receiver(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"CharacterData required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let data = self.character_data(&owner, &events, id)?;
            if length {
                let count = u32::try_from(data.len())
                    .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
                return Ok(scope.primitive(Primitive::Number(f64::from(count)))?);
            }
            Ok(self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.copy_to_js(data.code_units()))
                .map_err(Error::from)?)
        })();
        complete(scope, &events, result)
    }

    fn character_data(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        id: GuestNode,
    ) -> Result<DocumentString, DocumentGuestEventError> {
        events.step()?;
        self.callbacks.with_read(owner, events, |read, _| {
            let node = read.node(id.node())?;
            match (id, node.kind()) {
                (GuestNode::Text(_), DomNodeKind::Text(data))
                | (GuestNode::Comment(_), DomNodeKind::Comment(data)) => Ok(data.clone()),
                (
                    GuestNode::ProcessingInstruction(_),
                    DomNodeKind::ProcessingInstruction(value),
                ) => Ok(value.data().clone()),
                _ => Err(DocumentScriptNativeError::ProjectionCustody.into()),
            }
        })
    }
}

fn character_receiver<'scope>(
    scope: &mut Scope<'scope>,
    receiver: &Value<'scope>,
) -> Result<Option<GuestNode>, Error> {
    Ok(scope
        .native_object::<GuestNodeProjection>(receiver)?
        .and_then(|value| value.payload().id())
        .filter(|id| {
            matches!(
                id,
                GuestNode::Text(_) | GuestNode::Comment(_) | GuestNode::ProcessingInstruction(_)
            )
        }))
}

pub(in crate::document_scripting::guest::interfaces) fn copy_native_data(
    owner: &NativeOwner,
    events: &GuestEvents,
    units: &[u16],
) -> Result<(DocumentString, GuestBacking), DocumentGuestEventError> {
    let overhead = size_of::<Vec<u16>>() + 2 * size_of::<usize>();
    let bytes = units
        .len()
        .checked_mul(size_of::<u16>())
        .and_then(|bytes| bytes.checked_add(overhead))
        .ok_or(DocumentScriptNativeError::StorageLimit)?;
    let backing = events.reserve(owner, bytes)?;
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(units.len())
        .map_err(|_| Error::Allocation)?;
    if copied.capacity() > units.len() {
        return Err(DocumentScriptNativeError::StorageLimit.into());
    }
    let work = u64::try_from(units.len()).map_err(|_| DocumentGuestEventError::WorkLimit)?;
    events.charge_work(work)?;
    copied.extend_from_slice(units);
    Ok((DocumentString::from_utf16(copied), backing))
}
