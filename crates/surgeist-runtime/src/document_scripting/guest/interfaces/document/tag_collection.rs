//! Live collection wrappers for Document queries and Window named access.

use super::super::window_children::{self, WindowSite};
use super::*;
use surgeist_bindings_js::conversion::declarations::{
    document_get_elements_by_tag_name_0_qualified_name, html_collection_item_0_index,
    html_collection_named_item_0_name,
};
use surgeist_bindings_js::{IntegerValue, Intrinsic, ValueKind};
use surgeist_dom::hosted::DomWindowNamedValue;
use surgeist_dom::tree::{
    DomHtmlCollection, DomHtmlCollectionLimits, DomHtmlCollectionNames, DomHtmlCollectionRead,
};

const COLLECTION: &[u16] = &ascii(b"HTMLCollection");
const GET_BY_TAG: &[u16] = &ascii(b"getElementsByTagName");
const ITEM: &[u16] = &ascii(b"item");
const NAMED_ITEM: &[u16] = &ascii(b"namedItem");

struct Collection {
    native: DomHtmlCollection,
    _query: GuestBacking,
}

#[derive(Clone)]
pub(super) struct CollectionProjection {
    prototype: RetainedCallbackObject,
    read: RetainedCallbackObject,
    keys: RetainedCallbackObject,
}

impl CollectionProjection {
    fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        collection: Collection,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let prototype = scope.open_callback_object(&self.prototype)?;
        let read = scope.open_callback_object(&self.read)?;
        let keys = scope.open_callback_object(&self.keys)?;
        let value = scope.project_native_object_with_collection_properties(
            ObjectPrototype::Object(&prototype),
            collection,
            &read,
            &keys,
            |bytes| events.reserve(owner, bytes),
        )?;
        Ok(Completion::Return(value))
    }
}

#[derive(Clone, Copy)]
enum Read {
    Length,
    Item,
    NamedItem,
    Property,
}

enum Lookup {
    Length,
    Index(usize),
    Name {
        name: DocumentString,
        _backing: GuestBacking,
    },
}

enum Keys {
    Indices(u32),
    Names(DomHtmlCollectionNames),
}

type CollectionView<'dom> =
    DomHtmlCollectionRead<'dom, &'dom dyn Fn() -> Result<(), DocumentGuestEventError>>;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<CollectionProjection, DocumentGuestEventError> {
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(funding.capture)?,
        FunctionMetadata {
            name: COLLECTION,
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
    let global = scope.global()?;
    install_interface(scope, &global, COLLECTION, &constructor, &prototype)?;
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
    let length = (funding.capture)(size_of::<Capture>())?;
    install_getter(
        scope,
        &prototype,
        &ascii(b"length"),
        &ascii(b"get length"),
        true,
        Callback::new(move |scope, arguments| {
            length.read_tag_collection(scope, arguments, Read::Length)
        }),
    )?;
    for (name, operation) in [(ITEM, Read::Item), (NAMED_ITEM, Read::NamedItem)] {
        let method = (funding.capture)(size_of::<(Capture, Read)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                method.read_tag_collection(scope, arguments, operation)
            }),
            FunctionMetadata { name, length: 1 },
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

    let read = (funding.capture)(size_of::<Capture>())?;
    let read = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            read.read_tag_collection(scope, arguments, Read::Property)
        }),
        FunctionMetadata {
            name: &[],
            length: 2,
        },
    )?;
    let keys = (funding.capture)(size_of::<Capture>())?;
    let keys = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| keys.tag_collection_keys(scope, arguments)),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    let prototype = scope.callback_object(&prototype)?;
    let prototype = scope.retain_callback_object(prototype, funding.retain)?;
    let read = scope.callback_object(&read)?;
    let read = scope.retain_callback_object(read, funding.retain)?;
    let keys = scope.callback_object(&keys)?;
    let keys = scope.retain_callback_object(keys, funding.retain)?;
    let projection = CollectionProjection {
        prototype,
        read,
        keys,
    };
    let query_projection = projection.clone();
    let query = (funding.capture)(size_of::<(Capture, CollectionProjection)>())?;
    let query = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            query.get_elements_by_tag_name(scope, arguments, &query_projection)
        }),
        FunctionMetadata {
            name: GET_BY_TAG,
            length: 1,
        },
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(GET_BY_TAG),
        &query,
        true,
        true,
        true,
    )?;
    Ok(projection)
}

pub(super) fn install_window(
    scope: &mut Scope<'_>,
    capture: CaptureFunding<'_>,
    projection: &CollectionProjection,
    site: WindowSite,
) -> Result<(), DocumentGuestEventError> {
    let names = capture(size_of::<(Capture, CollectionProjection, WindowSite)>())?;
    let projection = projection.clone();
    let name_site = site.clone();
    scope.install_window_name_reader(&Callback::new(move |scope, arguments| {
        names.read_window_name(scope, arguments, &projection, &name_site)
    }))?;
    window_children::install(scope, capture, site)
}

impl Capture {
    fn get_elements_by_tag_name<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        projection: &CollectionProjection,
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
            let declaration = document_get_elements_by_tag_name_0_qualified_name();
            let name = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&declaration, arguments.get(0))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(name) = name.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let units = name.as_code_units();
            let (name, backing) = character_data::copy_native_data(&owner, &events, units)?;
            scope.check_active()?;
            let native = self.callbacks.with_read(&owner, &events, |read, _| {
                read.step()?;
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom
                    .raw()
                    .get_elements_by_tag_name(document, name, units.len(), read.cancel)?)
            })?;
            projection.project(
                scope,
                &owner,
                &events,
                Collection {
                    native,
                    _query: backing,
                },
            )
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn read_window_name<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        projection: &CollectionProjection,
        site: &WindowSite,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let argument = arguments.get(0).ok_or(Error::BackendUnavailable)?;
            let units = scope.dom_string(argument)?;
            let units = units.as_code_units();
            let (name, backing) = character_data::copy_native_data(&owner, &events, units)?;
            let named = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let Some(target) = site.qualify(self, read, attached)? else {
                        return Ok(DomWindowNamedValue::Unsupported);
                    };
                    let dom = read
                        .dom
                        .try_borrow()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    dom.raw().window_named_value(
                        target.global,
                        name,
                        units.len(),
                        events.limits.work,
                        || events.budget.charge_work(read.account_scope, 1),
                        read.cancel,
                    )
                })?;
            match named {
                DomWindowNamedValue::Unsupported => Ok(Completion::Undefined),
                DomWindowNamedValue::Element(node) => {
                    Ok(Completion::Return(self.project_observation(
                        scope,
                        &owner,
                        &events,
                        Observation::Node(Some(node)),
                    )?))
                }
                DomWindowNamedValue::Collection(native) => projection.project(
                    scope,
                    &owner,
                    &events,
                    Collection {
                        native,
                        _query: backing,
                    },
                ),
                DomWindowNamedValue::Child(context) => {
                    self.project_window_child(scope, &owner, &events, site, context)
                }
            }
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn read_tag_collection<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        operation: Read,
    ) -> Result<Completion<'scope>, Error> {
        let Some(collection) = scope.native_object::<Collection>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLCollection required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let argument = arguments.get(0);
            let lookup = match operation {
                Read::Length => Lookup::Length,
                Read::Item => self.tag_collection_index(scope, argument)?,
                Read::NamedItem => self.tag_collection_name(scope, &owner, &events, argument)?,
                Read::Property => match scope.kind(argument.ok_or(Error::BackendUnavailable)?)? {
                    ValueKind::Number => self.tag_collection_index(scope, argument)?,
                    ValueKind::String => {
                        self.tag_collection_name(scope, &owner, &events, argument)?
                    }
                    _ => return Err(Error::BackendUnavailable.into()),
                },
            };
            scope.check_active()?;
            let observed = self.with_tag_collection(
                &owner,
                &events,
                collection.payload(),
                DomHtmlCollectionLimits::new(events.limits.work, 0, 0),
                |view| {
                    Ok(match lookup {
                        Lookup::Length => Observation::Number(f64::from(collection_length(&view)?)),
                        Lookup::Index(index) => Observation::Node(view.item(index)?),
                        Lookup::Name { ref name, .. } => Observation::Node(view.named_item(name)?),
                    })
                },
            )?;
            if matches!(operation, Read::Property) {
                if arguments.get(1).is_some() {
                    return Ok(Completion::Return(scope.primitive(Primitive::Boolean(
                        matches!(observed, Observation::Node(Some(_))),
                    ))?));
                }
                if matches!(observed, Observation::Node(None)) {
                    return Ok(Completion::Undefined);
                }
            }
            Ok(Completion::Return(
                self.project_observation(scope, &owner, &events, observed)?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn tag_collection_index<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        argument: Option<&Value<'scope>>,
    ) -> Result<Lookup, Error> {
        let declaration = html_collection_item_0_index();
        let index = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                conversion.argument(&declaration, argument)
            })
            .map_err(Error::from)?;
        let ArgumentValue::Present(IntegerValue::UnsignedLong(index)) = index.get() else {
            return Err(Error::BackendUnavailable);
        };
        Ok(Lookup::Index(
            usize::try_from(*index).map_err(|_| Error::BackendUnavailable)?,
        ))
    }

    fn tag_collection_name<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        argument: Option<&Value<'scope>>,
    ) -> Result<Lookup, DocumentGuestEventError> {
        let declaration = html_collection_named_item_0_name();
        let name = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                conversion.argument(&declaration, argument)
            })
            .map_err(Error::from)?;
        let ArgumentValue::Present(name) = name.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let (name, backing) =
            character_data::copy_native_data(owner, events, name.as_code_units())?;
        Ok(Lookup::Name {
            name,
            _backing: backing,
        })
    }

    fn tag_collection_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(collection) = scope.native_object::<Collection>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLCollection required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let named = !arguments.is_empty();
            let limits = if named {
                let bytes = events.limits.storage_bytes;
                DomHtmlCollectionLimits::new(
                    events.limits.work,
                    bytes / (size_of::<DocumentString>() + size_of::<Value<'scope>>()),
                    bytes / size_of::<u16>(),
                )
            } else {
                DomHtmlCollectionLimits::new(events.limits.work, 0, 0)
            };
            scope.check_active()?;
            let keys =
                self.with_tag_collection(&owner, &events, collection.payload(), limits, |view| {
                    if named {
                        Ok(Keys::Names(view.supported_names()?))
                    } else {
                        Ok(Keys::Indices(collection_length(&view)?))
                    }
                })?;
            let count = match &keys {
                Keys::Indices(count) => usize::try_from(*count).map_err(|_| Error::ArrayTooLong)?,
                Keys::Names(names) => names.as_slice().len(),
            };
            let bytes = count
                .checked_mul(size_of::<Value<'scope>>())
                .ok_or(Error::Allocation)?;
            let _backing = events.reserve(&owner, bytes)?;
            let mut projected = Vec::new();
            projected
                .try_reserve_exact(count)
                .map_err(|_| Error::Allocation)?;
            if projected.capacity() != count {
                return Err(Error::Allocation.into());
            }
            match keys {
                Keys::Indices(count) => {
                    for index in 0..count {
                        events.step()?;
                        projected.push(scope.primitive(Primitive::Number(f64::from(index)))?);
                    }
                }
                Keys::Names(names) => {
                    for name in names.as_slice() {
                        events.step()?;
                        projected.push(self.copy_dom_units(scope, name.code_units())?);
                    }
                }
            }
            Ok(Completion::Return(scope.array(&projected)?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn with_tag_collection<T>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        collection: &Collection,
        limits: DomHtmlCollectionLimits,
        action: impl FnOnce(CollectionView<'_>) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, _| {
            let dom = read
                .dom
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let step = || events.budget.charge_work(read.account_scope, 1);
            let view = dom.raw().read_html_collection(
                &collection.native,
                limits,
                &step as &dyn Fn() -> Result<(), DocumentGuestEventError>,
                read.cancel,
            )?;
            action(view)
        })
    }
}

fn collection_length(view: &CollectionView<'_>) -> Result<u32, DocumentGuestEventError> {
    Ok(u32::try_from(view.length()?).map_err(|_| Error::ArrayTooLong)?)
}
