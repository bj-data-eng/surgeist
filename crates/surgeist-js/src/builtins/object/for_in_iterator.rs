//! This module implements the `ForInIterator` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-for-in-iterator-objects

use crate::{
    Context, JsData, JsResult, JsString, JsValue, NativeFunction,
    builtins::iterable::create_iter_result_object_trusted,
    error::JsNativeError,
    js_string,
    object::{FunctionObjectBuilder, JsObject, internal_methods::InternalMethodPropertyContext},
    property::PropertyKey,
    value::StoredValue,
};
use rustc_hash::FxHashSet;
use std::collections::VecDeque;
use surgeist_js_gc::heap::Trace as HeapTrace;

/// The `ForInIterator` object represents an iteration over some specific object.
/// It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-for-in-iterator-objects
#[derive(Debug, Clone, HeapTrace, JsData)]
pub(crate) struct ForInIterator {
    object: StoredValue,
    #[heap_trace(skip)]
    visited_keys: FxHashSet<JsString>,
    #[heap_trace(skip)]
    remaining_keys: VecDeque<JsString>,
    object_was_visited: bool,
}

impl ForInIterator {
    fn new(object: &JsValue) -> Self {
        Self {
            object: object.clone().into_stored(),
            visited_keys: FxHashSet::default(),
            remaining_keys: VecDeque::default(),
            object_was_visited: false,
        }
    }

    /// `CreateForInIterator( object )`
    ///
    /// Creates a new iterator over the given object.
    ///
    /// Returns the iterator object and its `next` method as a pair,
    /// avoiding the need to look up `next` through the prototype chain.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createforiniterator
    pub(crate) fn create_for_in_iterator(
        object: JsValue,
        context: &Context,
    ) -> JsResult<(JsObject, JsValue)> {
        let iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().iterator().prototype()?,
            Self::new(&object),
        )?
        .upcast();

        let next_method =
            FunctionObjectBuilder::new(context.realm(), NativeFunction::from_builtin(Self::next))
                .name(js_string!("next"))
                .length(0)
                .build()?;

        Ok((iterator, next_method.into()))
    }

    /// %ForInIteratorPrototype%.next( )
    ///
    /// Gets the next result in the object.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%foriniteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterator = this
            .as_object()
            .and_then(|object| object.downcast::<Self>().ok())
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a ForInIterator"))?;
        let value = iterator.borrow()?.data().object.to_value();
        let mut object = value.to_object(context.access())?;
        loop {
            let object_was_visited = iterator.borrow()?.data().object_was_visited;
            if !object_was_visited {
                let keys = object
                    .__own_property_keys__(&mut InternalMethodPropertyContext::new(context))?;
                let mut state = iterator.borrow_mut()?;
                let state = state.data_mut();
                for k in keys {
                    match k {
                        PropertyKey::String(ref k) => {
                            state.remaining_keys.push_back(k.clone());
                        }
                        PropertyKey::Index(i) => {
                            state.remaining_keys.push_back(i.get().into());
                        }
                        PropertyKey::Symbol(_) => {}
                    }
                }
                state.object_was_visited = true;
            }
            loop {
                let key = iterator.borrow_mut()?.data_mut().remaining_keys.pop_front();
                let Some(key) = key else {
                    break;
                };
                let visited = iterator.borrow()?.data().visited_keys.contains(&key);
                if !visited
                    && let Some(desc) = object.__get_own_property__(
                        &PropertyKey::from(key.clone()),
                        &mut InternalMethodPropertyContext::new(context),
                    )?
                {
                    iterator
                        .borrow_mut()?
                        .data_mut()
                        .visited_keys
                        .insert(key.clone());
                    if desc.expect_enumerable() {
                        return create_iter_result_object_trusted(
                            JsValue::new(key),
                            false,
                            context,
                        );
                    }
                }
            }
            let proto = object.prototype()?.clone();
            match proto {
                Some(o) => {
                    object = o;
                }
                _ => {
                    return create_iter_result_object_trusted(JsValue::undefined(), true, context);
                }
            }
            let mut state = iterator.borrow_mut()?;
            let state = state.data_mut();
            state.object = JsValue::new(object.clone()).into_stored();
            state.object_was_visited = false;
        }
    }
}
