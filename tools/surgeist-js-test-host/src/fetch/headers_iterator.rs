//! This module implements the `HeadersIterator` object.
//!
//! More information:
//!  - [Fetch specification][spec]
//!
//! [spec]: https://fetch.spec.whatwg.org/#headers-class

use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::native_function::{CapturedObject, NativeCapture};
use surgeist_js::{
    ContextAccess, JsData, JsResult, JsString, JsValue, boa_class,
    builtins::iterable::create_iter_result_object, error::JsNativeError, interop::JsClass,
    object::JsObject, object::builtins::JsArray,
};

use super::headers::JsHeaders;

/// The `IterationKind` enum represents the different kinds of iteration that can be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IterationKind {
    /// Iterates over the keys.
    Key,
    /// Iterates over the values.
    Value,
    /// Iterates over the keys and values.
    KeyAndValue,
}

/// The Headers Iterator object represents an iteration over a Headers object.
/// It implements the iterator protocol.
///
/// More information:
///  - [Fetch specification][spec]
///
/// [spec]: https://fetch.spec.whatwg.org/#headers-class
#[derive(Debug, JsData, HeapTrace)]
pub(crate) struct HeadersIterator {
    iterated_headers: CapturedObject,
    next_index: usize,
    #[heap_trace(skip)]
    iteration_kind: IterationKind,
}

#[boa_class(rename = "Headers Iterator")]
impl HeadersIterator {
    /// Prevent direct construction — `HeadersIterator` instances are only
    /// created internally via [`HeadersIterator::create_headers_iterator`].
    #[boa(constructor)]
    fn constructor() -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }

    /// `%HeadersIteratorPrototype%.next()`
    ///
    /// Advances the iterator and returns the next `{ value, done }` result.
    #[boa(method)]
    fn next(&mut self, mut context: ContextAccess<'_>) -> JsResult<JsValue> {
        let item_kind = self.iteration_kind;

        let headers = JsObject::<JsHeaders>::restore(&self.iterated_headers)?;
        let element = headers
            .borrow()?
            .data()
            .headers
            .borrow()
            .iter()
            .nth(self.next_index)
            .map(|(k, v)| {
                (
                    JsValue::from(JsString::from(k.as_str())),
                    JsValue::from(JsString::from(v.to_str().unwrap_or(""))),
                )
            });

        if let Some((key, value)) = element {
            let value = match item_kind {
                IterationKind::Key => key,
                IterationKind::Value => value,
                IterationKind::KeyAndValue => {
                    JsArray::from_iter([key, value], context.reborrow())?.into()
                }
            };
            self.next_index += 1;
            return create_iter_result_object(value, false, context.reborrow());
        }

        create_iter_result_object(JsValue::undefined(), true, context.reborrow())
    }

    /// Returns `this`, making the iterator itself iterable (`for...of` support).
    #[boa(method)]
    #[boa(symbol = "iterator")]
    fn symbol_iterator(this: JsClass<Self>) -> JsValue {
        this.inner().into()
    }
}

impl HeadersIterator {
    /// Creates a new iterator over the given `Headers` object.
    pub(crate) fn create_headers_iterator(
        headers: JsObject<JsHeaders>,
        kind: IterationKind,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let iter = Self {
            iterated_headers: headers.store(),
            next_index: 0,
            iteration_kind: kind,
        };

        let proto = context
            .realm()
            .get_class::<Self>()?
            .ok_or_else(|| surgeist_js::js_error!(Error: "Headers Iterator not registered"))?
            .prototype();

        let headers_iterator = JsObject::from_proto_and_data(context.runtime(), proto, iter)?;
        Ok(headers_iterator.into())
    }
}
