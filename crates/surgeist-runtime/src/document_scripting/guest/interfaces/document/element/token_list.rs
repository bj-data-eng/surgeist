//! Live ordered-set views of the canonical null-namespace attribute.

use super::super::{
    node_list::install_indexed_value_iteration, tag_collection::Keys, text::TextBuffer,
};
use super::*;
use surgeist_bindings_js::{
    IntegerValue, ValueKind,
    conversion::{ArgumentValue, declarations, types::DOM_STRING},
};
use surgeist_dom::{
    source::{DomSourceEdit, DomTokenListEdit},
    tree::{DomNode, DomTokenList},
};
use surgeist_html::{
    attributes::sandbox::HtmlSandboxKeyword,
    elements::token_list::{HtmlTokenListReflection, HtmlTokenListSupport},
    values::Namespace,
};

const INTERFACE: &[u16] = &ascii(b"DOMTokenList");
const VALUE: &[u16] = &ascii(b"value");
const ELEMENT_REFERENCE: u32 = 0;

#[derive(Clone, Copy)]
pub(super) enum Reflection {
    Class,
    Html(HtmlTokenListReflection),
}

struct Member {
    property: &'static [u16],
    getter: &'static [u16],
    setter: &'static [u16],
    reference: u32,
}

impl Reflection {
    fn member(self) -> Member {
        use HtmlTokenListReflection::{
            AnchorRel, AreaRel, FormRel, IframeSandbox, LinkBlocking, LinkRel, LinkSizes,
            OutputHtmlFor, ScriptBlocking, StyleBlocking,
        };

        match self {
            Self::Class => Member {
                property: &const { ascii(b"classList") },
                getter: &const { ascii(b"get classList") },
                setter: &const { ascii(b"set classList") },
                reference: 12,
            },
            Self::Html(ScriptBlocking | LinkBlocking | StyleBlocking) => Member {
                property: &const { ascii(b"blocking") },
                getter: &const { ascii(b"get blocking") },
                setter: &const { ascii(b"set blocking") },
                reference: 13,
            },
            Self::Html(AnchorRel | AreaRel | FormRel | LinkRel) => Member {
                property: &const { ascii(b"relList") },
                getter: &const { ascii(b"get relList") },
                setter: &const { ascii(b"set relList") },
                reference: 14,
            },
            Self::Html(LinkSizes) => Member {
                property: &const { ascii(b"sizes") },
                getter: &const { ascii(b"get sizes") },
                setter: &const { ascii(b"set sizes") },
                reference: 15,
            },
            Self::Html(IframeSandbox) => Member {
                property: &const { ascii(b"sandbox") },
                getter: &const { ascii(b"get sandbox") },
                setter: &const { ascii(b"set sandbox") },
                reference: 16,
            },
            Self::Html(OutputHtmlFor) => Member {
                property: &const { ascii(b"htmlFor") },
                getter: &const { ascii(b"get htmlFor") },
                setter: &const { ascii(b"set htmlFor") },
                reference: 17,
            },
        }
    }

    fn read(
        self,
        read: &NativeRead<'_>,
        node: DomNode<'_>,
    ) -> Result<Option<DomTokenList>, DocumentGuestEventError> {
        match self {
            Self::Class => Ok(Some(DomTokenList::class(node.id()))),
            Self::Html(reflection) => {
                read.step()?;
                Ok(DomTokenList::reflect_html(node, reflection))
            }
        }
    }
}

pub(super) struct Projection {
    prototype: RetainedCallbackObject,
    index: RetainedCallbackObject,
    keys: RetainedCallbackObject,
}

#[derive(Clone, Copy)]
enum Method {
    Length,
    Raw,
    SetValue,
    Item,
    Index,
    Contains,
    Add,
    Remove,
    Toggle,
    Replace,
    Supports,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    capture: CaptureFunding<'_>,
    retain: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
) -> Result<Projection, DocumentGuestEventError> {
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(capture)?,
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
    install_getter(
        scope,
        &prototype,
        &ascii(b"length"),
        &ascii(b"get length"),
        true,
        token_callback(capture, Method::Length)?,
    )?;
    install_accessor(
        scope,
        &prototype,
        VALUE,
        &ascii(b"get value"),
        &ascii(b"set value"),
        &token_callback(capture, Method::Raw)?,
        &token_callback(capture, Method::SetValue)?,
    )?;

    for (name, method, length, maximum) in [
        (&ascii(b"item")[..], Method::Item, 1, 1),
        (&ascii(b"contains")[..], Method::Contains, 1, 1),
        (&ascii(b"add")[..], Method::Add, 0, u32::MAX),
        (&ascii(b"remove")[..], Method::Remove, 0, u32::MAX),
        (
            &ascii(b"toggle")[..],
            Method::Toggle,
            1,
            declarations::DOM_TOKEN_LIST_TOGGLE_MAX_ARGUMENTS,
        ),
        (&ascii(b"replace")[..], Method::Replace, 2, 2),
        (&ascii(b"supports")[..], Method::Supports, 1, 1),
        (&ascii(b"toString")[..], Method::Raw, 0, 0),
    ] {
        let function = scope.bounded_function(
            &token_callback(capture, method)?,
            FunctionMetadata { name, length },
            maximum,
        )?;
        define_data(
            scope,
            &prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    install_indexed_value_iteration(scope, &prototype)?;
    install_interface(scope, global, INTERFACE, &constructor, &prototype)?;

    let index = scope.fixed_arity_function(
        &token_callback(capture, Method::Index)?,
        FunctionMetadata {
            name: &[],
            length: 2,
        },
    )?;
    let keys = capture(size_of::<Capture>())?;
    let keys = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| keys.token_list_keys(scope, arguments)),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    let prototype = scope.callback_object(&prototype)?;
    let index = scope.callback_object(&index)?;
    let keys = scope.callback_object(&keys)?;
    Ok(Projection {
        prototype: scope.retain_callback_object(prototype, retain)?,
        index: scope.retain_callback_object(index, retain)?,
        keys: scope.retain_callback_object(keys, retain)?,
    })
}

pub(super) fn install_reflection<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    reflection: Reflection,
) -> Result<(), DocumentGuestEventError> {
    let member = reflection.member();
    let getter = capture(size_of::<(Capture, Reflection)>())?;
    let setter = capture(size_of::<(Capture, Reflection)>())?;
    install_accessor(
        scope,
        prototype,
        member.property,
        member.getter,
        member.setter,
        &Callback::new(move |scope, arguments| {
            getter.element_token_list(scope, arguments, reflection)
        }),
        &Callback::new(move |scope, arguments| {
            setter.forward_token_list(scope, arguments, reflection)
        }),
    )?;
    Ok(())
}

fn install_accessor<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    property: &[u16],
    getter: &[u16],
    setter: &[u16],
    read: &Callback,
    write: &Callback,
) -> Result<(), Error> {
    let get = scope.fixed_arity_function(
        read,
        FunctionMetadata {
            name: getter,
            length: 0,
        },
    )?;
    let set = scope.fixed_arity_function(
        write,
        FunctionMetadata {
            name: setter,
            length: 1,
        },
    )?;
    scope.define_new_property(
        prototype,
        PropertyKey::String(property),
        PropertyDescriptor::Accessor {
            get: Some(&get),
            set: Some(&set),
            enumerable: true,
            configurable: true,
        },
    )
}

fn token_callback(
    capture: CaptureFunding<'_>,
    method: Method,
) -> Result<Callback, DocumentGuestEventError> {
    let capture = capture(size_of::<(Capture, Method)>())?;
    Ok(Callback::new(move |scope, arguments| {
        capture.token_list_method(scope, arguments, method)
    }))
}

impl Projection {
    fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        receiver: &Value<'scope>,
        association: (DomTokenList, u32),
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let prototype = scope.open_callback_object(&self.prototype)?;
        let callback = scope.callback_object(&prototype)?;
        if callback.associated_realm().identity() == scope.realm_authority()?.identity() {
            return self.project_in_realm(
                scope,
                (owner, events),
                &prototype,
                receiver,
                association,
            );
        }

        let receiver = scope.retain(receiver);
        let root = scope.in_callback_object_realm(&callback, |scope, prototype| {
            let receiver = scope.import(&receiver)?;
            let value =
                self.project_in_realm(scope, (owner, events), prototype, &receiver, association)?;
            Ok::<_, DocumentGuestEventError>(scope.retain(&value))
        })?;
        Ok(scope.import(&root)?)
    }

    fn project_in_realm<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: &Value<'scope>,
        receiver: &Value<'scope>,
        (list, reference): (DomTokenList, u32),
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let index = scope.open_callback_object(&self.index)?;
        let keys = scope.open_callback_object(&self.keys)?;
        let admit = cached_projection_admission(scope, owner, events, 2)?;
        let value = scope.project_native_object_with_indexed_properties(
            ObjectPrototype::Object(prototype),
            list,
            &index,
            &keys,
            admit,
        )?;
        scope.set_native_object_reference::<DomTokenList>(
            &value,
            ELEMENT_REFERENCE,
            Some(receiver),
        )?;
        scope.set_native_object_reference::<GuestNodeProjection>(
            receiver,
            reference,
            Some(&value),
        )?;
        Ok(value)
    }
}

impl Capture {
    fn element_token_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        reflection: Reflection,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some(GuestNode::Element(element)) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let reference = reflection.member().reference;
        let result = (|| {
            events.step()?;
            let list = self.callbacks.with_read(&owner, &events, |read, _| {
                reflection.read(read, read.node(element)?)
            })?;
            let Some(list) = list else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Incompatible Element"))?,
                ));
            };
            // A scoped root can reach native admission. Authenticate the
            // receiver first, then open the slot outside that native borrow.
            if let Some(cached) =
                scope.native_object_reference::<GuestNodeProjection>(receiver, reference)?
            {
                return Ok(Completion::Return(cached));
            }
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let (origin, attached) =
                self.callbacks
                    .with_read(&owner, &events, |read, attached| {
                        let node = read.node(element)?;
                        read.step()?;
                        let origin = node
                            .relevant_global()?
                            .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?;
                        Ok((origin, attached))
                    })?;
            let association = (list, reference);
            let value = if origin == attached {
                public.document.element.token_list.project(
                    scope,
                    &owner,
                    &events,
                    receiver,
                    association,
                )?
            } else {
                public.with_origin(&events, origin, |origin| {
                    origin.elements.token_list.project(
                        scope,
                        &owner,
                        &events,
                        receiver,
                        association,
                    )
                })?
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn forward_token_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        reflection: Reflection,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some(GuestNode::Element(element)) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            if matches!(reflection, Reflection::Html(_)) {
                let applicable = self.callbacks.with_read(&owner, &events, |read, _| {
                    reflection
                        .read(read, read.node(element)?)
                        .map(|list| list.is_some())
                })?;
                if !applicable {
                    return Ok(Completion::Throw(
                        scope.type_error(&ascii(b"Incompatible Element"))?,
                    ));
                }
            }
            events.step()?;
            let target = scope.get(receiver, PropertyKey::String(reflection.member().property))?;
            scope.check_active()?;
            if scope.kind(&target)? != ValueKind::Object {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Token list must be an object"))?,
                ));
            }
            let undefined;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    undefined = scope.primitive(Primitive::Undefined)?;
                    &undefined
                }
            };
            events.step()?;
            match scope.set(&target, PropertyKey::String(VALUE), input) {
                // Set is non-throwing for an ordinary rejection. Keep any
                // terminal host failure even when it has the same error type.
                Err(Error::PropertyRejected) => scope.check_active()?,
                result => result?,
            }
            Ok(Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn token_list_method<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, Error> {
        let Some(list) = scope.native_object::<DomTokenList>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"DOMTokenList required"))?,
            ));
        };
        let list = *list.payload();
        let (owner, events) = self.active(scope)?;
        let result = match method {
            Method::Length
            | Method::Raw
            | Method::Item
            | Method::Index
            | Method::Contains
            | Method::Supports => {
                self.read_token_list(scope, &owner, &events, list, &arguments, method)
            }
            Method::SetValue | Method::Add | Method::Remove | Method::Toggle | Method::Replace => {
                self.edit_token_list(scope, &owner, &events, list, arguments, method)
            }
        };
        tree::tree_completion(scope, &events, result)
    }

    fn read_token_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        list: DomTokenList,
        arguments: &Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let value = match method {
            Method::Length => {
                let length = self.token_list_length(owner, events, list)?;
                scope.primitive(Primitive::Number(f64::from(length)))?
            }
            Method::Raw => {
                let raw = self.callbacks.with_read(owner, events, |read, _| {
                    Ok(list.raw_with(&read.snapshot, || read.step())?.cloned())
                })?;
                self.copy_dom_units(scope, raw.as_ref().map_or(&[], DocumentString::code_units))?
            }
            Method::Item | Method::Index => {
                return self.token_list_item(scope, owner, events, list, arguments, method);
            }
            Method::Contains | Method::Supports => {
                let declaration = if matches!(method, Method::Contains) {
                    declarations::dom_token_list_contains_0_token()
                } else {
                    declarations::dom_token_list_supports_0_token()
                };
                let token = self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| {
                        conversion.argument(&declaration, arguments.get(0))
                    })
                    .map_err(Error::from)?;
                scope.check_active()?;
                let ArgumentValue::Present(token) = token.get() else {
                    return Err(Error::BackendUnavailable.into());
                };
                if matches!(method, Method::Supports) {
                    return self.token_list_supports(
                        scope,
                        owner,
                        events,
                        list,
                        token.as_code_units(),
                    );
                }
                let contains = self.callbacks.with_read(owner, events, |read, _| {
                    list.contains_with(&read.snapshot, token.as_code_units(), || read.step())
                })?;
                scope.primitive(Primitive::Boolean(contains))?
            }
            _ => return Err(Error::BackendUnavailable.into()),
        };
        Ok(Completion::Return(value))
    }

    fn token_list_supports<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        list: DomTokenList,
        token: &[u16],
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let support = match list.html_reflection() {
            Some(reflection) => self.callbacks.with_read(owner, events, |read, _| {
                read.node(list.element())?;
                reflection.support_with(token, || read.step())
            })?,
            None => HtmlTokenListSupport::Undefined,
        };
        let supported = match support {
            HtmlTokenListSupport::Undefined => {
                return Ok(Completion::Throw(scope.type_error(&ascii(
                    b"The association has no defined supported tokens",
                ))?));
            }
            HtmlTokenListSupport::Supported
            | HtmlTokenListSupport::SandboxCandidate(
                HtmlSandboxKeyword::AllowScripts | HtmlSandboxKeyword::AllowSameOrigin,
            ) => true,
            HtmlTokenListSupport::Unsupported => false,
            HtmlTokenListSupport::RelCandidate(_) | HtmlTokenListSupport::SandboxCandidate(_) => {
                return Err(DocumentGuestEventError::HostRouteUnavailable);
            }
        };
        Ok(Completion::Return(
            scope.primitive(Primitive::Boolean(supported))?,
        ))
    }

    fn token_list_item<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        list: DomTokenList,
        arguments: &Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let declaration = declarations::dom_token_list_item_0_index();
        let index = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                conversion.argument(&declaration, arguments.get(0))
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let ArgumentValue::Present(IntegerValue::UnsignedLong(index)) = index.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let index = usize::try_from(*index).map_err(|_| Error::ArrayTooLong)?;
        let presence_only = matches!(method, Method::Index) && arguments.get(1).is_some();
        let (present, item) = self.callbacks.with_read(owner, events, |read, _| {
            let captured =
                list.capture_with(&read.snapshot, || read.step(), |bytes| read.reserve(bytes))?;
            let Some(units) = captured.item(index) else {
                return Ok((false, None));
            };
            if presence_only {
                return Ok((true, None));
            }
            let item = TextBuffer::build_exact(read, units.len(), |push| {
                for &unit in units {
                    read.step()?;
                    push(unit)?;
                }
                Ok(())
            })?;
            Ok((true, Some(item)))
        })?;
        let value = if presence_only {
            scope.primitive(Primitive::Boolean(present))?
        } else if let Some(item) = item {
            self.copy_dom_units(scope, item.code_units())?
        } else if matches!(method, Method::Index) {
            return Ok(Completion::Undefined);
        } else {
            scope.primitive(Primitive::Null)?
        };
        Ok(Completion::Return(value))
    }

    fn token_list_length(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        list: DomTokenList,
    ) -> Result<u32, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, _| {
            let captured =
                list.capture_with(&read.snapshot, || read.step(), |bytes| read.reserve(bytes))?;
            Ok(u32::try_from(captured.len()).map_err(|_| Error::ArrayTooLong)?)
        })
    }

    fn token_list_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(list) = scope.native_object::<DomTokenList>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"DOMTokenList required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let length = self.token_list_length(&owner, &events, *list.payload())?;
            self.project_collection_keys(scope, &owner, &events, Keys::Indices(length))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn edit_token_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        list: DomTokenList,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        match method {
            Method::SetValue => {
                let undefined;
                let value = match arguments.get(0) {
                    Some(value) => value,
                    None => {
                        undefined = scope.primitive(Primitive::Undefined)?;
                        &undefined
                    }
                };
                let value = self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| conversion.value(&DOM_STRING, value))
                    .map_err(Error::from)?;
                scope.check_active()?;
                let (name, _name) = copy_native_data(owner, events, list.attribute_name())?;
                let (value, _text) = copy_native_data(owner, events, value.get().as_code_units())?;
                let edit =
                    DomSourceEdit::set_attribute_ns(list.element(), Namespace::none(), name, value);
                self.apply_attribute(scope, owner, events, edit, None, AttributeResult::Value)
            }
            Method::Add | Method::Remove => {
                self.edit_variadic_tokens(scope, (owner, events), list, arguments, method)
            }
            Method::Toggle => self.toggle_token(scope, (owner, events), list, &arguments),
            Method::Replace => self.replace_token(scope, (owner, events), list, &arguments),
            _ => Err(Error::BackendUnavailable.into()),
        }
    }

    fn edit_variadic_tokens<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        list: DomTokenList,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let count = arguments.len();
        let bytes = count
            .checked_mul(size_of::<DocumentString>() + size_of::<GuestBacking>())
            .ok_or(Error::Allocation)?;
        let _buffers = events.reserve(owner, bytes)?;
        let mut backing = Vec::new();
        let mut tokens = Vec::new();
        backing
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        tokens
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        if backing.capacity() != count || tokens.capacity() != count {
            return Err(Error::Allocation.into());
        }
        for value in arguments.into_parts().1 {
            events.step()?;
            let value = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&DOM_STRING, &value))
                .map_err(Error::from)?;
            scope.check_active()?;
            let (token, text) = copy_native_data(owner, events, value.get().as_code_units())?;
            tokens.push(token);
            backing.push(text);
        }
        let edit = if matches!(method, Method::Add) {
            DomTokenListEdit::Add(tokens)
        } else {
            DomTokenListEdit::Remove(tokens)
        };
        let edit = DomSourceEdit::edit_token_list(list, edit);
        self.apply_attribute(scope, owner, events, edit, None, AttributeResult::Value)
    }

    fn toggle_token<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        list: DomTokenList,
        arguments: &Arguments<'scope>,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let converted = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                declarations::convert_dom_token_list_toggle(conversion, arguments)
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let force = match converted.force.get() {
            ArgumentValue::Missing => None,
            ArgumentValue::Present(force) => Some(*force),
        };
        let (token, _token_backing) =
            copy_native_data(owner, events, converted.token.get().as_code_units())?;
        self.apply_token_result(
            scope,
            (owner, events),
            list,
            DomTokenListEdit::Toggle { token, force },
        )
    }

    fn replace_token<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        list: DomTokenList,
        arguments: &Arguments<'scope>,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let (token, replacement) = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                let token = conversion.argument(
                    &declarations::dom_token_list_replace_0_token(),
                    arguments.get(0),
                )?;
                let replacement = conversion.argument(
                    &declarations::dom_token_list_replace_0_new_token(),
                    arguments.get(1),
                )?;
                Ok((token, replacement))
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let ArgumentValue::Present(token) = token.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let (token, _token_backing) = copy_native_data(owner, events, token.as_code_units())?;
        let ArgumentValue::Present(replacement) = replacement.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let (replacement, _replacement_backing) =
            copy_native_data(owner, events, replacement.as_code_units())?;
        self.apply_token_result(
            scope,
            (owner, events),
            list,
            DomTokenListEdit::Replace { token, replacement },
        )
    }

    fn apply_token_result<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        list: DomTokenList,
        edit: DomTokenListEdit,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let edit = DomSourceEdit::edit_token_list(list, edit);
        let result = AttributeResult::TokenListResult {
            when_true: scope.primitive(Primitive::Boolean(true))?,
            when_false: scope.primitive(Primitive::Boolean(false))?,
        };
        self.apply_attribute(scope, owner, events, edit, None, result)
    }
}
