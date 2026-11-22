//! Live dataset projections over the canonical Element attributes.

use super::{
    character_data::{build_native_data, copy_native_data},
    element::AttributeResult,
    reflection::Receiver,
    text::TextBuffer,
    *,
};
use surgeist_bindings_js::{
    NativeMutableNamedPropertyCallbacks, NativeNamedPropertyCallbacks,
    conversion::{ArgumentValue, declarations},
    dom_exception::DomExceptionName,
};
use surgeist_dom::{source::DomSourceEdit, tree::DomDataset};
use surgeist_html::microsyntax::dataset::{HtmlDatasetAttributeName, HtmlDatasetPropertyName};

const INTERFACE: &[u16] = &ascii(b"DOMStringMap");
const DATASET_REFERENCE: u32 = 18;
const ELEMENT_REFERENCE: u32 = 0;

pub(super) struct Projection {
    prototype: RetainedCallbackObject,
    read: RetainedCallbackObject,
    keys: RetainedCallbackObject,
    set: RetainedCallbackObject,
    delete: RetainedCallbackObject,
}

#[derive(Clone, Copy)]
enum Member {
    Read,
    Keys,
    Set,
    Delete,
}

enum NamedRead {
    Presence(bool),
    Value(Option<DocumentString>),
}

#[derive(Default)]
struct DatasetNames {
    names: Vec<TextBuffer>,
    _backing: Option<GuestBacking>,
}

impl DatasetNames {
    fn push(
        &mut self,
        read: &NativeRead<'_>,
        name: HtmlDatasetPropertyName<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        if self.names.len() == self.names.capacity() {
            let capacity = self.names.capacity().saturating_mul(2).max(1);
            let bytes = capacity
                .checked_mul(size_of::<TextBuffer>())
                .ok_or(Error::Allocation)?;
            let backing = read.reserve(bytes)?;
            let mut enlarged = Vec::new();
            enlarged
                .try_reserve_exact(capacity)
                .map_err(|_| Error::Allocation)?;
            if enlarged.capacity() != capacity {
                return Err(Error::Allocation.into());
            }
            for name in self.names.drain(..) {
                read.step()?;
                enlarged.push(name);
            }
            self.names = enlarged;
            self._backing = Some(backing);
        }
        let text = TextBuffer::build_exact(read, name.len(), |write| {
            name.visit_with(|| read.step(), write)
        })?;
        read.step()?;
        self.names.push(text);
        Ok(())
    }
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
    install_interface(scope, global, INTERFACE, &constructor, &prototype)?;
    let prototype = scope.callback_object(&prototype)?;
    Ok(Projection {
        prototype: scope.retain_callback_object(prototype, retain)?,
        read: install_callback(scope, capture, retain, Member::Read, 2)?,
        keys: install_callback(scope, capture, retain, Member::Keys, 0)?,
        set: install_callback(scope, capture, retain, Member::Set, 2)?,
        delete: install_callback(scope, capture, retain, Member::Delete, 1)?,
    })
}

fn install_callback(
    scope: &mut Scope<'_>,
    capture: CaptureFunding<'_>,
    retain: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    member: Member,
    length: u32,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, Member)>())?;
    let callback = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.dataset_member(scope, arguments, member)),
        FunctionMetadata { name: &[], length },
    )?;
    let callback = scope.callback_object(&callback)?;
    scope.retain_callback_object(callback, retain)
}

pub(super) fn install_accessor<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, Receiver)>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"dataset"),
        &ascii(b"get dataset"),
        true,
        Callback::new(move |scope, arguments| handler.element_dataset(scope, arguments, receiver)),
    )?;
    Ok(())
}

impl Projection {
    fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        receiver: &Value<'scope>,
        dataset: DomDataset,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let prototype = scope.open_callback_object(&self.prototype)?;
        let callback = scope.callback_object(&prototype)?;
        if callback.associated_realm().identity() == scope.realm_authority()?.identity() {
            return self.project_in_realm(scope, (owner, events), &prototype, receiver, dataset);
        }
        let receiver = scope.retain(receiver);
        let root = scope.in_callback_object_realm(&callback, |scope, prototype| {
            let receiver = scope.import(&receiver)?;
            let map =
                self.project_in_realm(scope, (owner, events), prototype, &receiver, dataset)?;
            Ok::<_, DocumentGuestEventError>(scope.retain(&map))
        })?;
        Ok(scope.import(&root)?)
    }

    fn project_in_realm<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: &Value<'scope>,
        receiver: &Value<'scope>,
        dataset: DomDataset,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let read = scope.open_callback_object(&self.read)?;
        let keys = scope.open_callback_object(&self.keys)?;
        let set = scope.open_callback_object(&self.set)?;
        let delete = scope.open_callback_object(&self.delete)?;
        let admit = cached_projection_admission(scope, owner, events, 2)?;
        let value = scope.project_native_object_with_mutable_named_properties(
            ObjectPrototype::Object(prototype),
            dataset,
            NativeMutableNamedPropertyCallbacks {
                properties: NativeNamedPropertyCallbacks {
                    read: &read,
                    keys: &keys,
                    unforgeable_names: &[],
                },
                set: &set,
                delete: &delete,
            },
            admit,
        )?;
        scope.set_native_object_reference::<DomDataset>(
            &value,
            ELEMENT_REFERENCE,
            Some(receiver),
        )?;
        scope.set_native_object_reference::<GuestNodeProjection>(
            receiver,
            DATASET_REFERENCE,
            Some(&value),
        )?;
        Ok(value)
    }
}

impl Capture {
    fn element_dataset<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        brand: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some(GuestNode::Element(element)) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(scope.type_error(brand.error_message())?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let dataset = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(element)?;
                if !brand.matches(node, || read.step())? {
                    return Ok(None);
                }
                read.step()?;
                Ok(DomDataset::new(node))
            })?;
            let Some(dataset) = dataset else {
                return Ok(Completion::Throw(scope.type_error(brand.error_message())?));
            };
            if let Some(cached) =
                scope.native_object_reference::<GuestNodeProjection>(receiver, DATASET_REFERENCE)?
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
            let value = if origin == attached {
                public
                    .document
                    .element
                    .dataset
                    .project(scope, &owner, &events, receiver, dataset)?
            } else {
                public.with_origin(&events, origin, |origin| {
                    origin
                        .elements
                        .dataset
                        .project(scope, &owner, &events, receiver, dataset)
                })?
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn dataset_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let Some(native) = scope.native_object::<DomDataset>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"DOMStringMap required"))?,
            ));
        };
        let dataset = *native.payload();
        let (owner, events) = self.active(scope)?;
        let result = match member {
            Member::Read => self.dataset_read(scope, &arguments, &owner, &events, dataset),
            Member::Keys => self.dataset_keys(scope, &owner, &events, dataset),
            Member::Set | Member::Delete => self.dataset_edit(
                scope,
                &arguments,
                (&owner, &events),
                dataset,
                matches!(member, Member::Set),
            ),
        };
        tree::tree_completion(scope, &events, result)
    }

    fn dataset_read<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        dataset: DomDataset,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let key = scope.dom_string(arguments.get(0).ok_or(Error::BackendUnavailable)?)?;
        scope.check_active()?;
        let presence = arguments.get(1).is_some();
        let observation = self.callbacks.with_read(owner, events, |read, _| {
            let attribute =
                dataset.get_with(&read.snapshot, key.as_code_units(), || read.step())?;
            Ok(if presence {
                NamedRead::Presence(attribute.is_some())
            } else {
                NamedRead::Value(attribute.map(|attribute| attribute.value().clone()))
            })
        })?;
        match observation {
            NamedRead::Presence(found) => Ok(Completion::Return(
                scope.primitive(Primitive::Boolean(found))?,
            )),
            NamedRead::Value(Some(value)) => Ok(Completion::Return(
                self.copy_dom_units(scope, value.code_units())?,
            )),
            NamedRead::Value(None) => Ok(Completion::Undefined),
        }
    }

    fn dataset_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        dataset: DomDataset,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let keys = self.callbacks.with_read(owner, events, |read, _| {
            let mut names = DatasetNames::default();
            dataset.visit_with(
                &read.snapshot,
                || read.step(),
                |name, _| names.push(read, name),
            )?;
            Ok(names)
        })?;
        // The named platform object suppresses duplicate property keys in the
        // canonical input order, including numeric-looking string names.
        let count = keys.names.len();
        let bytes = count
            .checked_mul(size_of::<Value<'scope>>())
            .ok_or(Error::Allocation)?;
        let _projected_backing = events.reserve(owner, bytes)?;
        let mut projected = Vec::new();
        projected
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        if projected.capacity() != count {
            return Err(Error::Allocation.into());
        }
        for name in &keys.names {
            events.step()?;
            projected.push(self.copy_dom_units(scope, name.code_units())?);
        }
        Ok(Completion::Return(scope.array(&projected)?))
    }

    fn dataset_edit<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        dataset: DomDataset,
        set: bool,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let key = scope.dom_string(arguments.get(0).ok_or(Error::BackendUnavailable)?)?;
        if !set {
            let name =
                HtmlDatasetAttributeName::for_delete_with(key.as_code_units(), || events.step())?;
            let (name, _backing) = build_native_data(owner, events, name.len(), |write| {
                name.visit_with(|| events.step(), write)
            })?;
            return self.apply_tree_edit(
                scope,
                owner,
                events,
                DomSourceEdit::remove_attribute(dataset.element(), name),
                Completion::Undefined,
            );
        }
        let value = declarations::dom_string_map_operation_1_value();
        let converted = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                conversion.argument(&value, arguments.get(value.index()))
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let ArgumentValue::Present(value) = converted.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let name = HtmlDatasetAttributeName::for_set_with(key.as_code_units(), || events.step())?;
        let Some(name) = name else {
            return Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::SyntaxError, &[])?,
            ));
        };
        let (name, _name_backing) = build_native_data(owner, events, name.len(), |write| {
            name.visit_with(|| events.step(), write)
        })?;
        let (value, _value_backing) = copy_native_data(owner, events, value.as_code_units())?;
        self.apply_attribute(
            scope,
            owner,
            events,
            DomSourceEdit::set_attribute_local_name(dataset.element(), name, value),
            None,
            AttributeResult::Value,
        )
    }
}
