//! This module implements the `ArrayIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects

use crate::{
    Context, JsData, JsResult,
    builtins::{
        Array, BuiltInBuilder, IntrinsicObject, JsValue,
        iterable::create_iter_result_object_trusted, typed_array::TypedArray,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::{JsObject, ObjectEdge},
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    symbol::JsSymbol,
};
use surgeist_js_gc::heap::Trace as HeapTrace;

/// The Array Iterator object represents an iteration over an array. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, HeapTrace, JsData)]
pub(crate) struct ArrayIterator {
    array: Option<ObjectEdge>,
    next_index: u64,
    #[heap_trace(skip)]
    kind: PropertyNameKind,
}

impl IntrinsicObject for ArrayIterator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)?
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Array Iterator"),
                Attribute::CONFIGURABLE,
            )?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().array()
    }
}

impl ArrayIterator {
    fn new(array: &JsObject, kind: PropertyNameKind) -> Self {
        Self {
            array: Some(array.edge()),
            kind,
            next_index: 0,
        }
    }

    /// `CreateArrayIterator( array, kind )`
    ///
    /// Creates a new iterator over the given array.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createarrayiterator
    pub(crate) fn create_array_iterator(
        array: JsObject,
        kind: PropertyNameKind,
        context: &Context,
    ) -> JsResult<JsValue> {
        array.check_runtime(context.runtime())?;
        let array_iterator = JsObject::new(
            context.root_shape(),
            context.intrinsics().objects().iterator_prototypes().array(),
            Self::new(&array, kind),
        )?;
        Ok(array_iterator.upcast().into())
    }

    /// %ArrayIteratorPrototype%.next( )
    ///
    /// Gets the next result in the array.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let array_iterator = object
            .and_then(|object| object.downcast::<Self>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not an ArrayIterator"))?;
        let state = array_iterator.borrow()?;
        let index = state.data().next_index;
        let kind = state.data().kind;
        let array = state.data().array.clone();
        drop(state);

        let Some(array) = array else {
            return Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            ));
        };
        let array = JsObject::from_inner(array);

        let len = if let Some(f) = array.downcast_ref::<TypedArray>()? {
            let buffer = f.viewed_array_buffer();
            let buf = buffer.as_buffer()?;
            let Some(buf) = buf
                .bytes(std::sync::atomic::Ordering::SeqCst)
                .filter(|buf| !f.is_out_of_bounds(buf.len()))
            else {
                return Err(JsNativeError::typ()
                    .with_message("Cannot get value from out of bounds typed array")
                    .into());
            };

            f.array_length(buf.len())
        } else {
            array.length_of_array_like(context)?
        };

        if index >= len {
            array_iterator.borrow_mut()?.data_mut().array = None;
            return Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            ));
        }
        array_iterator.borrow_mut()?.data_mut().next_index = index + 1;
        match kind {
            PropertyNameKind::Key => Ok(create_iter_result_object_trusted(
                index.into(),
                false,
                context,
            )),
            PropertyNameKind::Value => {
                let element_value = array.get(index, context.access())?;
                Ok(create_iter_result_object_trusted(
                    element_value,
                    false,
                    context,
                ))
            }
            PropertyNameKind::KeyAndValue => {
                let element_value = array.get(index, context.access())?;
                let result = Array::create_array_from_list([index.into(), element_value], context);
                Ok(create_iter_result_object_trusted(
                    result.into(),
                    false,
                    context,
                ))
            }
        }
    }
}
