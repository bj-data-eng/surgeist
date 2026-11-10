//! Boa's implementation of the `%WrapForValidIteratorPrototype%` object.
//!
//! This object wraps an iterator record to make it a valid `Iterator` instance
//! by inheriting from `%Iterator.prototype%`.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-wrapforvaliditeratorprototype-object

use crate::{
    Context, JsData, JsResult, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject, iterable::create_iter_result_object_trusted},
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    realm::Realm,
};
use surgeist_js_gc::heap::Trace as HeapTrace;

use super::StoredIteratorRecord;

/// The internal representation of a `WrapForValidIterator` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-wrapforvaliditeratorprototype-object
#[derive(Debug, HeapTrace, JsData)]
pub(crate) struct WrapForValidIterator {
    /// `[[Iterated]]` — the iterator record this wrapper delegates to.
    pub(crate) iterated: StoredIteratorRecord,
}

impl IntrinsicObject for WrapForValidIterator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)?
            .prototype(realm.intrinsics().constructors().iterator().prototype()?)?
            .static_method(Self::next, js_string!("next"), 0)?
            .static_method(Self::r#return, js_string!("return"), 0)?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> crate::JsResult<JsObject> {
        Ok(intrinsics
            .objects()
            .iterator_prototypes()
            .wrap_for_valid_iterator()?)
    }
}

impl WrapForValidIterator {
    /// `%WrapForValidIteratorPrototype%.next ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%25wrapforvaliditeratorprototype%25.next
    pub(crate) fn next(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        // 2. Perform ? RequireInternalSlot(O, [[Iterated]]).
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WrapForValidIterator method called on non-object")
        })?;

        let wrapper = object.downcast_ref::<Self>()?.ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WrapForValidIterator method called on incompatible object")
        })?;

        // 3. Let iteratorRecord be O.[[Iterated]].
        // 4. Return ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
        let iterated = wrapper.iterated.to_iterator()?;
        drop(wrapper);

        iterated
            .next_method()
            .call(&iterated.iterator().clone().into(), &[], context)
    }

    /// `%WrapForValidIteratorPrototype%.return ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%25wrapforvaliditeratorprototype%25.return
    pub(crate) fn r#return(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        // 2. Perform ? RequireInternalSlot(O, [[Iterated]]).
        let object = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WrapForValidIterator method called on non-object")
        })?;

        let wrapper = object.downcast_ref::<Self>()?.ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WrapForValidIterator method called on incompatible object")
        })?;

        // 3. Let iterator be O.[[Iterated]].[[Iterator]].
        let iterator = wrapper.iterated.to_iterator()?.iterator().clone();
        drop(wrapper);

        // 4. Assert: iterator is an Object.
        // 5. Let returnMethod be ? GetMethod(iterator, "return").
        let return_method = iterator.get_method(js_string!("return"), context)?;

        match return_method {
            // 6. If returnMethod is undefined, then
            None => {
                // a. Return CreateIterResultObject(undefined, true).
                create_iter_result_object_trusted(JsValue::undefined(), true, context)
            }
            // 7. Return ? Call(returnMethod, iterator).
            Some(return_method) => return_method.call(&iterator.into(), &[], context.access()),
        }
    }
}
