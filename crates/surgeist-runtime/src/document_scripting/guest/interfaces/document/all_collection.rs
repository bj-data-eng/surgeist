//! Callable HTMLAllCollection over the canonical native Document.

use super::tag_collection::{Keys, key_limits};
use super::*;
use surgeist_bindings_js::conversion::declarations::{
    html_all_collection_item_0_name_or_index, html_all_collection_named_item_0_name,
};
use surgeist_bindings_js::{
    DomString, HtmlAllCollectionCallbacks, IntegerKind, IntegerMode, IntegerValue, Intrinsic,
    ValueKind,
};
use surgeist_dom::tree::{
    DomHtmlAllCollection, DomHtmlAllCollectionRead, DomHtmlAllNamedValue, DomHtmlCollection,
    DomHtmlCollectionLimits,
};

const INTERFACE: &[u16] = &ascii(b"HTMLAllCollection");
const ALL_REFERENCE: u32 = 2;
const DOCUMENT_REFERENCE: u32 = 0;

struct Collection {
    document: DocumentId,
    native: DomHtmlAllCollection,
}

pub(super) struct Projection {
    prototype: RetainedCallbackObject,
    read: RetainedCallbackObject,
    keys: RetainedCallbackObject,
    named_getter: RetainedCallbackObject,
    call: RetainedCallbackObject,
}

#[derive(Clone, Copy)]
enum Read {
    Length,
    Item,
    NamedItem,
    Property,
    Call,
}

enum Lookup {
    Missing,
    Length,
    Index(u32),
    Name(DocumentString, GuestBacking),
}

type CurrentRead<'dom> =
    DomHtmlAllCollectionRead<'dom, &'dom dyn Fn() -> Result<(), DocumentGuestEventError>>;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    document: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<Projection, DocumentGuestEventError> {
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(funding.capture)?,
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
    let length = (funding.capture)(size_of::<Capture>())?;
    install_getter(
        scope,
        &prototype,
        &ascii(b"length"),
        &ascii(b"get length"),
        true,
        Callback::new(move |scope, arguments| {
            length.read_all_collection(scope, arguments, Read::Length)
        }),
    )?;
    let named_getter =
        collection_function(scope, funding, &ascii(b"namedItem"), 1, Read::NamedItem)?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(&ascii(b"namedItem")),
        &named_getter,
        true,
        true,
        true,
    )?;
    let item = collection_function(scope, funding, &ascii(b"item"), 0, Read::Item)?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(&ascii(b"item")),
        &item,
        true,
        true,
        true,
    )?;
    install_interface(scope, global, INTERFACE, &constructor, &prototype)?;
    let iterator = scope.well_known_symbol(WellKnownSymbol::Iterator)?;
    let values = scope.intrinsic(Intrinsic::ArrayValues)?;
    define_data(
        scope,
        &prototype,
        PropertyKey::Value(&iterator),
        &values,
        true,
        false,
        true,
    )?;

    let getter = (funding.capture)(size_of::<Capture>())?;
    install_getter(
        scope,
        document,
        &ascii(b"all"),
        &ascii(b"get all"),
        true,
        Callback::new(move |scope, arguments| getter.document_all(scope, arguments)),
    )?;
    let read = collection_function(scope, funding, &[], 2, Read::Property)?;
    let call = collection_function(scope, funding, &[], 0, Read::Call)?;
    let keys = (funding.capture)(size_of::<Capture>())?;
    let keys = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| keys.all_collection_keys(scope, arguments)),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    let prototype = scope.callback_object(&prototype)?;
    let read = scope.callback_object(&read)?;
    let keys = scope.callback_object(&keys)?;
    let named_getter = scope.callback_object(&named_getter)?;
    let call = scope.callback_object(&call)?;
    Ok(Projection {
        prototype: scope.retain_callback_object(prototype, funding.retain)?,
        read: scope.retain_callback_object(read, funding.retain)?,
        keys: scope.retain_callback_object(keys, funding.retain)?,
        named_getter: scope.retain_callback_object(named_getter, funding.retain)?,
        call: scope.retain_callback_object(call, funding.retain)?,
    })
}

fn collection_function<'scope>(
    scope: &mut Scope<'scope>,
    funding: &OriginInterfaceFunding<'_>,
    name: &[u16],
    length: u32,
    operation: Read,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let handler = (funding.capture)(size_of::<(Capture, Read)>())?;
    let maximum_arguments = match operation {
        Read::Item | Read::Call => 1,
        _ => length,
    };
    Ok(scope.bounded_function(
        &Callback::new(move |scope, arguments| {
            handler.read_all_collection(scope, arguments, operation)
        }),
        FunctionMetadata { name, length },
        maximum_arguments,
    )?)
}

impl Projection {
    fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        native: (&NativeOwner, &GuestEvents, Collection),
        document: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let prototype = scope.open_callback_object(&self.prototype)?;
        let callback = scope.callback_object(&prototype)?;
        if callback.associated_realm().identity() == scope.realm_authority()?.identity() {
            return self.project_in_realm(scope, &prototype, native, document);
        }
        let document = scope.retain(document);
        let result = scope.in_callback_object_realm(&callback, |scope, prototype| {
            let document = scope.import(&document)?;
            let result = self.project_in_realm(scope, prototype, native, &document)?;
            Ok::<_, DocumentGuestEventError>(scope.retain(&result))
        })?;
        Ok(scope.import(&result)?)
    }

    fn project_in_realm<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        (owner, events, collection): (&NativeOwner, &GuestEvents, Collection),
        document: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let read = scope.open_callback_object(&self.read)?;
        let keys = scope.open_callback_object(&self.keys)?;
        let named_getter = scope.open_callback_object(&self.named_getter)?;
        let call = scope.open_callback_object(&self.call)?;
        let admit = cached_projection_admission(scope, owner, events, 2)?;
        let all = scope.project_html_all_collection(
            ObjectPrototype::Object(prototype),
            collection,
            HtmlAllCollectionCallbacks {
                read: &read,
                keys: &keys,
                named_getter: &named_getter,
                call: &call,
            },
            admit,
        )?;
        scope.set_native_object_reference::<Collection>(
            &all,
            DOCUMENT_REFERENCE,
            Some(document),
        )?;
        scope.set_native_object_reference::<GuestNodeProjection>(
            document,
            ALL_REFERENCE,
            Some(&all),
        )?;
        Ok(all)
    }
}

impl Capture {
    fn document_all<'scope>(
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
            if let Some(all) =
                scope.native_object_reference::<GuestNodeProjection>(receiver, ALL_REFERENCE)?
            {
                return Ok(Completion::Return(all));
            }
            let native = self.callbacks.with_read(&owner, &events, |read, _| {
                read.step()?;
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom.raw().html_all_collection(document, read.cancel)?)
            })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let project = |scope: &mut Scope<'scope>, projection: &Projection| {
                projection.project(
                    scope,
                    (&owner, &events, Collection { document, native }),
                    receiver,
                )
            };
            let value =
                if let Some(origin) = public.other_document_origin(&owner, &events, document)? {
                    public.with_origin(&events, origin, |origin| {
                        project(scope, &origin.documents.all)
                    })?
                } else {
                    project(scope, &public.document.prototypes.all)?
                };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn read_all_collection<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        operation: Read,
    ) -> Result<Completion<'scope>, Error> {
        let Some(all) = scope.native_object::<Collection>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLAllCollection required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let (lookup, presence) =
                self.all_lookup(scope, &arguments, &owner, &events, operation)?;
            scope.check_active()?;
            let limits = key_limits(&events, false);
            let collection = all.payload();
            let observed = match lookup {
                Lookup::Missing => Observation::Node(None),
                Lookup::Length => {
                    self.with_all_collection(&owner, &events, collection, limits, |view| {
                        let length =
                            u32::try_from(view.length()?).map_err(|_| Error::ArrayTooLong)?;
                        Ok(Observation::Number(f64::from(length)))
                    })?
                }
                Lookup::Index(index) => {
                    let index = usize::try_from(index).map_err(|_| Error::ArrayTooLong)?;
                    let item =
                        self.with_all_collection(&owner, &events, collection, limits, |view| {
                            view.item(index)
                        })?;
                    if presence {
                        Observation::Boolean(item.is_some())
                    } else {
                        Observation::Node(item)
                    }
                }
                Lookup::Name(name, backing) => {
                    if presence {
                        let supported = self.with_all_collection(
                            &owner,
                            &events,
                            collection,
                            limits,
                            |view| view.has_named_item(&name),
                        )?;
                        return Ok(Completion::Return(
                            scope.primitive(Primitive::Boolean(supported))?,
                        ));
                    }
                    let found =
                        self.with_all_collection(&owner, &events, collection, limits, |view| {
                            view.named_item(&name, name.code_units().len())
                        })?;
                    match found {
                        DomHtmlAllNamedValue::Absent => Observation::Node(None),
                        DomHtmlAllNamedValue::Element(node) => Observation::Node(Some(node)),
                        DomHtmlAllNamedValue::Collection(native) => {
                            return self.project_all_named_result(
                                scope,
                                (&owner, &events),
                                collection.document,
                                native,
                                backing,
                            );
                        }
                    }
                }
            };
            if matches!(operation, Read::Property) && matches!(observed, Observation::Node(None)) {
                return Ok(Completion::Undefined);
            }
            Ok(Completion::Return(
                self.project_observation(scope, &owner, &events, observed)?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn all_lookup<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        operation: Read,
    ) -> Result<(Lookup, bool), DocumentGuestEventError> {
        let argument = arguments.get(0);
        let lookup = match operation {
            Read::Length => Lookup::Length,
            Read::Property => {
                let key = argument.ok_or(Error::ExpectedPropertyKey)?;
                let lookup = match scope.kind(key)? {
                    ValueKind::Number => {
                        let IntegerValue::UnsignedLong(index) = scope.integer(
                            key,
                            IntegerKind::UnsignedLong,
                            IntegerMode::EnforceRange,
                        )?
                        else {
                            return Err(Error::ExpectedPropertyKey.into());
                        };
                        Lookup::Index(index)
                    }
                    ValueKind::String => {
                        let name = scope.dom_string(key)?;
                        all_name(owner, events, &name, false)?
                    }
                    _ => return Err(Error::ExpectedPropertyKey.into()),
                };
                return Ok((lookup, arguments.get(1).is_some()));
            }
            Read::Item | Read::NamedItem => {
                let (declaration, numeric) = match operation {
                    Read::Item => (html_all_collection_item_0_name_or_index(), true),
                    _ => (html_all_collection_named_item_0_name(), false),
                };
                let name = self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| {
                        conversion.argument(&declaration, argument)
                    })
                    .map_err(Error::from)?;
                match name.get() {
                    ArgumentValue::Present(name) => all_name(owner, events, name, numeric)?,
                    _ => Lookup::Missing,
                }
            }
            Read::Call => match argument {
                Some(argument) if scope.kind(argument)? != ValueKind::Undefined => {
                    let name = scope.dom_string(argument)?;
                    all_name(owner, events, &name, true)?
                }
                _ => Lookup::Missing,
            },
        };
        Ok((lookup, false))
    }

    fn with_all_collection<T>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        collection: &Collection,
        limits: DomHtmlCollectionLimits,
        action: impl FnOnce(CurrentRead<'_>) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, _| {
            let dom = read
                .dom
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let step = || events.budget.charge_work(read.account_scope, 1);
            let view = dom.raw().read_html_all_collection(
                &collection.native,
                limits,
                &step as &dyn Fn() -> Result<(), DocumentGuestEventError>,
                read.cancel,
            )?;
            action(view)
        })
    }

    fn project_all_named_result<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        document: DocumentId,
        native: DomHtmlCollection,
        backing: GuestBacking,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        if let Some(origin) = public.other_document_origin(owner, events, document)? {
            public.with_origin(events, origin, |origin| {
                origin
                    .documents
                    .collections
                    .project_named_result(scope, owner, events, native, backing)
            })
        } else {
            public
                .document
                .prototypes
                .collections
                .project_named_result(scope, owner, events, native, backing)
        }
    }

    fn all_collection_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let all = scope
            .native_object::<Collection>(arguments.receiver())?
            .ok_or(Error::ExpectedNativeObject)?;
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let named = !arguments.is_empty();
            scope.check_active()?;
            let keys = self.with_all_collection(
                &owner,
                &events,
                all.payload(),
                key_limits(&events, named),
                |view| {
                    if named {
                        Ok(Keys::Names(view.supported_names()?))
                    } else {
                        let count =
                            u32::try_from(view.length()?).map_err(|_| Error::ArrayTooLong)?;
                        Ok(Keys::Indices(count))
                    }
                },
            )?;
            self.project_collection_keys(scope, &owner, &events, keys)
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn all_name(
    owner: &NativeOwner,
    events: &GuestEvents,
    name: &DomString,
    select_index: bool,
) -> Result<Lookup, DocumentGuestEventError> {
    if select_index {
        events.step()?;
        if let Some(index) = name.array_index() {
            return Ok(Lookup::Index(index));
        }
    }
    let (name, backing) = character_data::copy_native_data(owner, events, name.as_code_units())?;
    Ok(Lookup::Name(name, backing))
}
