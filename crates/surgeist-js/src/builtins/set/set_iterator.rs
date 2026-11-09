//! This module implements the `SetIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects

use super::ordered_set::OrderedSet;
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
struct SetIteration {
    collection: ObjectEdge,
    #[heap_trace(skip)]
    _position_lock: Rc<()>,
}

impl SetIteration {
    fn new(collection: &JsObject<OrderedSet>) -> JsResult<Self> {
        Ok(Self {
            collection: collection.edge(),
            _position_lock: collection.borrow_mut()?.data_mut().lock(),
        })
    }
}

/// The Set Iterator object represents an iteration over a set. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-set-iterator-objects
#[derive(Debug, HeapTrace, JsData)]
pub(crate) struct SetIterator {
    iterated_set: Option<SetIteration>,
    next_index: usize,
    #[heap_trace(skip)]
    iteration_kind: PropertyNameKind,
}

impl IntrinsicObject for SetIterator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)?
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Set Iterator"),
                Attribute::CONFIGURABLE,
            )?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().set()
    }
}

impl SetIterator {
    /// Abstract operation `CreateSetIterator( set, kind )`
    ///
    /// Creates a new iterator over the given set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createsetiterator
    pub(crate) fn create_set_iterator(
        set: JsObject<OrderedSet>,
        kind: PropertyNameKind,
        context: &Context,
    ) -> JsResult<JsValue> {
        if !context.runtime().same_heap(&set.runtime()) {
            return Err(js_error!(TypeError: "collection iterator crosses JavaScript runtimes"));
        }
        let set_iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().objects().iterator_prototypes().set(),
            Self {
                iterated_set: Some(SetIteration::new(&set)?),
                next_index: 0,
                iteration_kind: kind,
            },
        );
        Ok(set_iterator.into())
    }

    /// %SetIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the set.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%setiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let invalid_receiver = || JsNativeError::typ().with_message("`this` is not a SetIterator");
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

            let collection = iterator.iterated_set.as_ref().map(|iteration| {
                iteration
                    .collection
                    .upgrade()
                    .expect("iterator's heap is available")
                    .expect("active iterator retains its collection")
                    .downcast::<OrderedSet>()
                    .expect("iterator retains a Set")
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
                iterator.iterated_set = None;
            }
            (element, iterator.iteration_kind)
        };

        let Some(element) = element else {
            return Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            ));
        };
        let item = match kind {
            PropertyNameKind::KeyAndValue => {
                Array::create_array_from_list([element.clone(), element], context).into()
            }
            _ => element,
        };
        Ok(create_iter_result_object_trusted(item, false, context))
    }
}
