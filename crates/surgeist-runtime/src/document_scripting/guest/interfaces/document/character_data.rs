//! Text and Comment creation and CharacterData access over the canonical DOM.

use super::*;
mod mutations;
use surgeist_bindings_js::{
    DomString,
    conversion::{
        Argument, Converted, Type,
        declarations::{
            comment_constructor_0_data, document_create_comment_0_data,
            document_create_text_node_0_data, text_constructor_0_data,
        },
    },
};

const CHARACTER_DATA: &[u16] = &ascii(b"CharacterData");
const TEXT: &[u16] = &ascii(b"Text");
const COMMENT: &[u16] = &ascii(b"Comment");
const CREATE_TEXT: &[u16] = &ascii(b"createTextNode");
const CREATE_COMMENT: &[u16] = &ascii(b"createComment");

#[derive(Clone, Copy)]
enum CharacterKind {
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

    fn create(
        self,
        batch: &mut DomBatch,
        document: DocumentId,
        data: DocumentString,
    ) -> surgeist_dom::CreatedNode {
        match self {
            Self::Text => batch.create_text(document, data),
            Self::Comment => batch.create_comment(document, data),
        }
    }

    fn project(self, node: NodeId) -> GuestNode {
        match self {
            Self::Text => GuestNode::Text(node),
            Self::Comment => GuestNode::Comment(node),
        }
    }
}

pub(super) struct CharacterBindings {
    text: RetainedCallbackObject,
    comment: RetainedCallbackObject,
}

impl CharacterBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        node_constructor: &Value<'scope>,
        document: &Value<'scope>,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let prototype = scope.object(ObjectPrototype::Object(node))?;
        let interface = scope.constructor(
            &ConstructorCallback::bounded(
                0,
                |_| Err(Error::BackendUnavailable),
                |scope, _| {
                    Ok(ConstructorCompletion::Throw(
                        scope.type_error(&ascii(b"Illegal constructor"))?,
                    ))
                },
            ),
            FunctionMetadata {
                name: CHARACTER_DATA,
                length: 0,
            },
        )?;
        install_derived(
            scope,
            node_constructor,
            CHARACTER_DATA,
            &interface,
            &prototype,
        )?;
        for (length, key, name) in [
            (false, &ascii(b"data")[..], &ascii(b"get data")[..]),
            (true, &ascii(b"length")[..], &ascii(b"get length")[..]),
        ] {
            let getter = capture(size_of::<(Capture, bool)>())?;
            let get = scope.fixed_arity_function(
                &Callback::new(move |scope, arguments| {
                    getter.character_member(scope, arguments, length)
                }),
                FunctionMetadata { name, length: 0 },
            )?;
            let set = if length {
                None
            } else {
                Some(mutations::data_setter(scope, capture)?)
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
        mutations::install_methods(scope, &prototype, capture)?;
        let text = Self::install_kind(
            scope,
            &interface,
            &prototype,
            document,
            CharacterKind::Text,
            capture,
            fund,
        )?;
        let comment = Self::install_kind(
            scope,
            &interface,
            &prototype,
            document,
            CharacterKind::Comment,
            capture,
            fund,
        )?;
        Ok(Self { text, comment })
    }

    fn install_kind<'scope>(
        scope: &mut Scope<'scope>,
        parent_constructor: &Value<'scope>,
        parent_prototype: &Value<'scope>,
        document: &Value<'scope>,
        kind: CharacterKind,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
        let prototype = scope.object(ObjectPrototype::Object(parent_prototype))?;
        let fallback = capture(size_of::<(Capture, CharacterKind)>())?;
        let construct = capture(size_of::<(Capture, CharacterKind)>())?;
        let callback = ConstructorCallback::bounded(
            1,
            move |scope| {
                let (_, events) = fallback.active(scope)?;
                let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                scope.open_callback_object(public.document.character_data.prototype(kind))
            },
            move |scope, arguments| construct.construct_character(scope, arguments, kind),
        );
        let interface = scope.constructor(
            &callback,
            FunctionMetadata {
                name: kind.name(),
                length: 0,
            },
        )?;
        install_derived(
            scope,
            parent_constructor,
            kind.name(),
            &interface,
            &prototype,
        )?;
        let (name, _) = kind.factory();
        let factory = capture(size_of::<(Capture, CharacterKind)>())?;
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
        scope.retain_callback_object(observed, fund)
    }

    fn prototype(&self, kind: CharacterKind) -> &RetainedCallbackObject {
        match kind {
            CharacterKind::Text => &self.text,
            CharacterKind::Comment => &self.comment,
        }
    }
}

fn install_derived<'scope>(
    scope: &mut Scope<'scope>,
    parent: &Value<'scope>,
    name: &[u16],
    interface: &Value<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    scope.set_prototype(interface, ObjectPrototype::Object(parent))?;
    scope.define_property(
        interface,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    let global = scope.global()?;
    install_interface(scope, &global, name, interface, prototype)
}

impl Capture {
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
                selected.prototype(),
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
            let bindings = &events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .document;
            let prototype = scope.open_callback_object(bindings.character_data.prototype(kind))?;
            self.publish_character(scope, &owner, &events, &prototype, (kind, document), &data)
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
        prototype: &Value<'scope>,
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
        ordinary.project(scope, owner, events, prototype, |inbox, key| {
            self.with_native(owner, |session, _| {
                let mut batch = DomBatch::new();
                let ticket = kind.create(&mut batch, document, data);
                let (_, node, watch) = session
                    .dom()
                    .raw_mut()
                    .apply_with_node_retirement_watch(
                        batch,
                        &ticket,
                        inbox,
                        key,
                        events.limits.node_retirement,
                        &owner.cancellation,
                    )
                    .map_err(DocumentScriptNativeError::from)?
                    .into_parts();
                Ok((kind.project(node), watch))
            })
        })
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
        self.with_native(owner, |session, _| {
            let snapshot = session.snapshot();
            let node = snapshot
                .node(id.node())
                .map_err(DocumentScriptNativeError::from)?;
            match (id, node.kind()) {
                (GuestNode::Text(_), DomNodeKind::Text(data))
                | (GuestNode::Comment(_), DomNodeKind::Comment(data)) => Ok(data.clone()),
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
        .filter(|id| matches!(id, GuestNode::Text(_) | GuestNode::Comment(_))))
}

fn copy_native_data(
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
