//! This module implements the `MapIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-map-iterator-objects

use super::ordered_map::OrderedMap;
use crate::{
    Context, JsData, JsResult,
    builtins::{
        Array, BuiltInBuilder, IntrinsicObject, JsValue,
        iterable::create_iter_result_object_trusted,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_error, js_string,
    object::{JsObject, ObjectEdge},
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    symbol::JsSymbol,
};
use std::rc::Rc;
use surgeist_js_gc::heap::Trace as HeapTrace;

#[derive(Debug, HeapTrace)]
struct MapIteration {
    collection: ObjectEdge,
    #[heap_trace(skip)]
    _position_lock: Rc<()>,
}

impl MapIteration {
    fn new(collection: &JsObject<OrderedMap>) -> JsResult<Self> {
        Ok(Self {
            collection: collection.edge(),
            _position_lock: collection.borrow_mut()?.data_mut().lock(),
        })
    }
}

/// The Map Iterator object represents an iteration over a map. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-map-iterator-objects
#[derive(Debug, HeapTrace, JsData)]
pub(crate) struct MapIterator {
    iterated_map: Option<MapIteration>,
    next_index: usize,
    #[heap_trace(skip)]
    iteration_kind: PropertyNameKind,
}

impl IntrinsicObject for MapIterator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)?
            .prototype(realm.intrinsics().constructors().iterator().prototype()?)?
            .static_method(Self::next, js_string!("next"), 0)?
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Map Iterator"),
                Attribute::CONFIGURABLE,
            )?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> crate::JsResult<JsObject> {
        Ok(intrinsics.objects().iterator_prototypes().map()?)
    }
}

impl MapIterator {
    /// Abstract operation `CreateMapIterator( map, kind )`
    ///
    /// Creates a new iterator over the given map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createmapiterator
    pub(crate) fn create_map_iterator(
        map: JsObject<OrderedMap>,
        kind: PropertyNameKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if !context.runtime().same_heap(&map.runtime()) {
            return Err(js_error!(TypeError: "collection iterator crosses JavaScript runtimes"));
        }
        let iter = Self {
            iterated_map: Some(MapIteration::new(&map)?),
            next_index: 0,
            iteration_kind: kind,
        };
        let map_iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().objects().iterator_prototypes().map()?,
            iter,
        )?;
        Ok(map_iterator.into())
    }

    /// %MapIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the map.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let invalid_receiver = || JsNativeError::typ().with_message("`this` is not a MapIterator");
        let object = this.as_object().ok_or_else(invalid_receiver)?;
        let (element, kind) = {
            let mut iterator = object
                .downcast_mut::<Self>()?
                .ok_or_else(invalid_receiver)?;
            if !context.runtime().same_heap(&object.runtime()) {
                return Err(
                    js_error!(TypeError: "collection iterator crosses JavaScript runtimes"),
                );
            }

            let collection = iterator.iterated_map.as_ref().map(|iteration| {
                iteration
                    .collection
                    .upgrade()
                    .expect("iterator's heap is available")
                    .expect("active iterator retains its collection")
                    .downcast::<OrderedMap>()
                    .expect("iterator retains a Map")
            });
            let element = if let Some(collection) = collection {
                let collection = collection.borrow()?;
                let entries = collection.data();

                loop {
                    if iterator.next_index >= entries.full_len() {
                        break None;
                    }
                    let index = iterator.next_index;
                    iterator.next_index += 1;
                    if let Some(element) = entries.get_index(index) {
                        break Some(element);
                    }
                }
            } else {
                None
            };
            if element.is_none() {
                iterator.iterated_map = None;
            }
            (element, iterator.iteration_kind)
        };

        let Some((key, value)) = element else {
            return create_iter_result_object_trusted(JsValue::undefined(), true, context);
        };
        let item = match kind {
            PropertyNameKind::Key => key,
            PropertyNameKind::Value => value,
            PropertyNameKind::KeyAndValue => {
                Array::create_array_from_list([key, value], context)?.into()
            }
        };
        create_iter_result_object_trusted(item, false, context)
    }
}
