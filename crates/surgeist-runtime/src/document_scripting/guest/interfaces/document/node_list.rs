//! Live indexed NodeLists over the canonical DOM.

use super::tag_collection::Keys;
use super::*;
use surgeist_bindings_js::conversion::declarations::{
    document_html_get_elements_by_name_0_element_name, node_list_item_0_index,
};
use surgeist_bindings_js::{IntegerValue, Intrinsic};
use surgeist_dom::tree::{DomNodeList, DomNodeListRead};

const INTERFACE: &[u16] = &ascii(b"NodeList");
const CHILD_LIST_REFERENCE: u32 = 10;
const ROOT_REFERENCE: u32 = 0;

struct List {
    native: DomNodeList,
    custody: ListCustody,
}

enum ListCustody {
    ChildNodes,
    DocumentName { _query: GuestBacking },
}

pub(super) struct Projection {
    prototype: RetainedCallbackObject,
    read: RetainedCallbackObject,
    keys: RetainedCallbackObject,
}

#[derive(Clone, Copy)]
enum Operation {
    Length,
    Item,
    Index,
}

type ListView<'dom> = DomNodeListRead<'dom, &'dom dyn Fn() -> Result<(), DocumentGuestEventError>>;

pub(super) fn install_children<'scope>(
    scope: &mut Scope<'scope>,
    node: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        node,
        &ascii(b"childNodes"),
        &ascii(b"get childNodes"),
        true,
        Callback::new(move |scope, arguments| handler.child_node_list(scope, arguments)),
    )?;
    Ok(())
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
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

    let capture = funding.capture;
    let length = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        &prototype,
        &ascii(b"length"),
        &ascii(b"get length"),
        true,
        Callback::new(move |scope, arguments| {
            length.read_node_list(scope, arguments, Operation::Length)
        }),
    )?;
    let item = capture(size_of::<Capture>())?;
    let item_name = &ascii(b"item");
    let item = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            item.read_node_list(scope, arguments, Operation::Item)
        }),
        FunctionMetadata {
            name: item_name,
            length: 1,
        },
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(item_name),
        &item,
        true,
        true,
        true,
    )?;
    install_indexed_value_iteration(scope, &prototype)?;
    install_interface(scope, global, INTERFACE, &constructor, &prototype)?;

    let read = capture(size_of::<Capture>())?;
    let read = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            read.read_node_list(scope, arguments, Operation::Index)
        }),
        FunctionMetadata {
            name: &[],
            length: 2,
        },
    )?;
    let keys = capture(size_of::<Capture>())?;
    let keys = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| keys.node_list_keys(scope, arguments)),
        FunctionMetadata {
            name: &[],
            length: 0,
        },
    )?;
    let prototype = scope.callback_object(&prototype)?;
    let read = scope.callback_object(&read)?;
    let keys = scope.callback_object(&keys)?;
    Ok(Projection {
        prototype: scope.retain_callback_object(prototype, funding.retain)?,
        read: scope.retain_callback_object(read, funding.retain)?,
        keys: scope.retain_callback_object(keys, funding.retain)?,
    })
}

pub(super) fn install_document_lookup<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let name_handler = capture(size_of::<Capture>())?;
    let name = &ascii(b"getElementsByName");
    let query = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| name_handler.elements_by_name(scope, arguments)),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(name),
        &query,
        true,
        true,
        true,
    )?;
    Ok(())
}

pub(super) fn install_indexed_value_iteration<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    let iterator = scope.well_known_symbol(WellKnownSymbol::Iterator)?;
    let values = scope.intrinsic(Intrinsic::ArrayValues)?;
    define_data(
        scope,
        prototype,
        PropertyKey::Value(&iterator),
        &values,
        true,
        false,
        true,
    )?;
    let entries = scope.intrinsic(Intrinsic::ArrayEntries)?;
    let keys = scope.intrinsic(Intrinsic::ArrayKeys)?;
    let for_each = scope.intrinsic(Intrinsic::ArrayForEach)?;
    for (name, value) in [
        (&ascii(b"entries")[..], &entries),
        (&ascii(b"keys")[..], &keys),
        (&ascii(b"values")[..], &values),
        (&ascii(b"forEach")[..], &for_each),
    ] {
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            value,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Projection {
    fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        receiver: &Value<'scope>,
        list: List,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let prototype = scope.open_callback_object(&self.prototype)?;
        let callback = scope.callback_object(&prototype)?;
        if callback.associated_realm().identity() == scope.realm_authority()?.identity() {
            return self.project_in_realm(scope, (owner, events), &prototype, receiver, list);
        }
        let receiver = scope.retain(receiver);
        let value = scope.in_callback_object_realm(&callback, |scope, prototype| {
            let receiver = scope.import(&receiver)?;
            let value =
                self.project_in_realm(scope, (owner, events), prototype, &receiver, list)?;
            Ok::<_, DocumentGuestEventError>(scope.retain(&value))
        })?;
        Ok(scope.import(&value)?)
    }

    fn project_in_realm<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: &Value<'scope>,
        receiver: &Value<'scope>,
        list: List,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let read = scope.open_callback_object(&self.read)?;
        let keys = scope.open_callback_object(&self.keys)?;
        let same_object = matches!(list.custody, ListCustody::ChildNodes);
        let admit =
            cached_projection_admission(scope, owner, events, 1 + usize::from(same_object))?;
        let list = scope.project_native_object_with_indexed_properties(
            ObjectPrototype::Object(prototype),
            list,
            &read,
            &keys,
            admit,
        )?;
        scope.set_native_object_reference::<List>(&list, ROOT_REFERENCE, Some(receiver))?;
        if same_object {
            scope.set_native_object_reference::<GuestNodeProjection>(
                receiver,
                CHILD_LIST_REFERENCE,
                Some(&list),
            )?;
        }
        Ok(list)
    }
}

impl Capture {
    fn child_node_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some(node) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            // The traced list can outlive the canonical node.
            let origin = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let node = read.node(node.node())?;
                    Ok(node.relevant_global()?.filter(|origin| *origin != attached))
                })?;
            if let Some(list) = scope
                .native_object_reference::<GuestNodeProjection>(receiver, CHILD_LIST_REFERENCE)?
            {
                return Ok(list);
            }
            let native = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom.raw().child_nodes(node.node(), read.cancel)?)
            })?;
            let list = List {
                native,
                custody: ListCustody::ChildNodes,
            };
            self.project_node_list(scope, (&owner, &events), origin, receiver, list)
        })();
        complete(scope, &events, result)
    }

    fn elements_by_name<'scope>(
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
            let declaration = document_html_get_elements_by_name_0_element_name();
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
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let origin = public.other_document_origin(&owner, &events, document)?;
            let native = self.callbacks.with_read(&owner, &events, |read, _| {
                read.step()?;
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom
                    .raw()
                    .get_elements_by_name(document, name, units.len(), read.cancel)?)
            })?;
            let list = List {
                native,
                custody: ListCustody::DocumentName { _query: backing },
            };
            self.project_node_list(scope, (&owner, &events), origin, receiver, list)
                .map(Completion::Return)
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn project_node_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        origin: Option<DomHostedGlobalId>,
        receiver: &Value<'scope>,
        list: List,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        if let Some(origin) = origin {
            return public.with_origin(events, origin, |origin| {
                origin
                    .documents
                    .node_lists
                    .project(scope, owner, events, receiver, list)
            });
        }
        public
            .document
            .prototypes
            .node_lists
            .project(scope, owner, events, receiver, list)
    }

    fn read_node_list<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        operation: Operation,
    ) -> Result<Completion<'scope>, Error> {
        let Some(list) = scope.native_object::<List>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"NodeList required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            if matches!(operation, Operation::Length) {
                let length = self
                    .with_node_list(&owner, &events, list.payload(), |view| list_length(&view))?;
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Number(f64::from(length)))?,
                ));
            }
            let declaration = node_list_item_0_index();
            let index = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&declaration, arguments.get(0))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(IntegerValue::UnsignedLong(index)) = index.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let index = usize::try_from(*index).map_err(|_| Error::ArrayTooLong)?;
            scope.check_active()?;
            let item =
                self.with_node_list(&owner, &events, list.payload(), |view| view.item(index))?;
            if matches!(operation, Operation::Index) {
                if arguments.get(1).is_some() {
                    return Ok(Completion::Return(
                        scope.primitive(Primitive::Boolean(item.is_some()))?,
                    ));
                }
                if item.is_none() {
                    return Ok(Completion::Undefined);
                }
            }
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(item),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn node_list_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(list) = scope.native_object::<List>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"NodeList required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let length =
                self.with_node_list(&owner, &events, list.payload(), |view| list_length(&view))?;
            self.project_collection_keys(scope, &owner, &events, Keys::Indices(length))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn with_node_list<T>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        list: &List,
        action: impl FnOnce(ListView<'_>) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, _| {
            let dom = read
                .dom
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let step = || events.budget.charge_work(read.account_scope, 1);
            let view = dom.raw().read_node_list(
                &list.native,
                events.limits.work,
                &step as &dyn Fn() -> Result<(), DocumentGuestEventError>,
                read.cancel,
            )?;
            action(view)
        })
    }
}

fn list_length(view: &ListView<'_>) -> Result<u32, DocumentGuestEventError> {
    Ok(u32::try_from(view.length()?).map_err(|_| Error::ArrayTooLong)?)
}
