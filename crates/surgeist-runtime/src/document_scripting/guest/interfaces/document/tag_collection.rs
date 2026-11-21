//! Live Document tag-name reads over the canonical DOM collection.

use super::*;
use surgeist_bindings_js::IntegerValue;
use surgeist_bindings_js::conversion::declarations::{
    document_get_elements_by_tag_name_0_qualified_name, html_collection_item_0_index,
};
use surgeist_dom::tree::{DomHtmlCollection, DomHtmlCollectionLimits};

const COLLECTION: &[u16] = &ascii(b"HTMLCollection");
const GET_BY_TAG: &[u16] = &ascii(b"getElementsByTagName");
const ITEM: &[u16] = &ascii(b"item");

struct Collection {
    native: DomHtmlCollection,
    _query: GuestBacking,
}

#[derive(Clone, Copy)]
enum Read {
    Length,
    Item,
    Index,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
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
    let item = (funding.capture)(size_of::<Capture>())?;
    let item = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            item.read_tag_collection(scope, arguments, Read::Item)
        }),
        FunctionMetadata {
            name: ITEM,
            length: 1,
        },
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(ITEM),
        &item,
        true,
        true,
        true,
    )?;

    let index = (funding.capture)(size_of::<Capture>())?;
    let index = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            index.read_tag_collection(scope, arguments, Read::Index)
        }),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    let prototype = scope.callback_object(&prototype)?;
    let prototype = scope.retain_callback_object(prototype, funding.retain)?;
    let index = scope.callback_object(&index)?;
    let index = scope.retain_callback_object(index, funding.retain)?;
    let query = (funding.capture)(size_of::<(
        Capture,
        RetainedCallbackObject,
        RetainedCallbackObject,
    )>())?;
    let query = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            query.get_elements_by_tag_name(scope, arguments, &prototype, &index)
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
    Ok(())
}

impl Capture {
    fn get_elements_by_tag_name<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        prototype: &RetainedCallbackObject,
        getter: &RetainedCallbackObject,
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
            let prototype = scope.open_callback_object(prototype)?;
            let getter = scope.open_callback_object(getter)?;
            let value = scope.project_native_object_with_indexed_reads(
                ObjectPrototype::Object(&prototype),
                Collection {
                    native,
                    _query: backing,
                },
                &getter,
                |bytes| events.reserve(&owner, bytes),
            )?;
            Ok(Completion::Return(value))
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
            let index = match operation {
                Read::Length => None,
                Read::Item | Read::Index => {
                    let declaration = html_collection_item_0_index();
                    let index = self
                        .callbacks
                        .conversion
                        .run(scope, |conversion| {
                            conversion.argument(&declaration, arguments.get(0))
                        })
                        .map_err(Error::from)?;
                    let ArgumentValue::Present(IntegerValue::UnsignedLong(index)) = index.get()
                    else {
                        return Err(Error::BackendUnavailable.into());
                    };
                    Some(usize::try_from(*index).map_err(|_| Error::BackendUnavailable)?)
                }
            };
            scope.check_active()?;
            let observed = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let view = dom.raw().read_html_collection(
                    &collection.payload().native,
                    DomHtmlCollectionLimits::new(events.limits.work, 0, 0),
                    || events.budget.charge_work(read.account_scope, 1),
                    read.cancel,
                )?;
                match index {
                    Some(index) => Ok(Observation::Node(view.item(index)?)),
                    None => {
                        let count =
                            u32::try_from(view.length()?).map_err(|_| Error::ArrayTooLong)?;
                        Ok(Observation::Number(f64::from(count)))
                    }
                }
            })?;
            if matches!(
                (&observed, operation),
                (Observation::Node(None), Read::Index)
            ) {
                return Ok(Completion::Undefined);
            }
            Ok(Completion::Return(
                self.project_observation(scope, &owner, &events, observed)?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }
}
