//! A Rust API wrapper for Boa's `Array` Builtin ECMAScript Object
#[cfg(test)]
use crate::Context;
use crate::ContextAccess;
use crate::{
    JsExpect, JsResult, JsString, JsValue,
    builtins::Array,
    error::JsNativeError,
    object::{JsFunction, JsObject},
    value::{IntoOrUndefined, TryFromJs},
};
use std::ops::Deref;
use thin_vec::ThinVec;

/// `JsArray` provides a wrapper for Boa's implementation of the JavaScript `Array` object.
#[derive(Debug, Clone)]
pub struct JsArray {
    inner: JsObject,
}

impl JsArray {
    /// Create a new empty array.
    ///
    /// # Errors
    ///
    /// Returns the exact construction failure if the array cannot be created.
    #[inline]
    pub fn new(context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let inner = Array::array_create(0, None, context)?;

        Ok(Self { inner })
    }

    /// Create an array from an iterator of JavaScript values.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if an element is an object from a different runtime.
    /// No array is created until every element has been checked.
    pub fn from_iter<I>(elements: I, context: ContextAccess<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = JsValue>,
    {
        let context = context.into_owner();
        let elements = elements
            .into_iter()
            .map(|value| {
                context.runtime().check_value(&value)?;
                Ok(value)
            })
            .collect::<JsResult<ThinVec<_>>>()?;
        Ok(Self {
            inner: Array::create_array_with_dense_elements(elements, context)?,
        })
    }

    /// Create a [`JsArray`] from a [`JsObject`], if the object is not an array throw a `TypeError`.
    ///
    /// This does not clone the fields of the array, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_array() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not an Array")
                .into())
        }
    }

    /// Get the length of the array.
    ///
    /// Same as `array.length` in JavaScript.
    #[inline]
    pub fn length(&self, context: ContextAccess<'_>) -> JsResult<u64> {
        let context = context.into_owner();
        self.inner.length_of_array_like(context)
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        self.inner.length_of_array_like(context).map(|len| len == 0)
    }

    /// Push an element to the array.
    pub fn push<T>(&self, value: T, context: ContextAccess<'_>) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        self.push_items(&[value.into()], context.access())
    }

    /// Pushes a slice of elements to the array.
    #[inline]
    pub fn push_items(&self, items: &[JsValue], context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::push(&self.inner.clone().into(), items, context)
    }

    /// Pops an element from the array.
    #[inline]
    pub fn pop(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::pop(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.at()`.
    pub fn at<T>(&self, index: T, context: ContextAccess<'_>) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        let context = context.into_owner();
        Array::at(&self.inner.clone().into(), &[index.into().into()], context)
    }

    /// Calls `Array.prototype.shift()`.
    #[inline]
    pub fn shift(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::shift(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.unshift()`.
    #[inline]
    pub fn unshift(&self, items: &[JsValue], context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::unshift(&self.inner.clone().into(), items, context)
    }

    /// Calls `Array.prototype.reverse()`.
    #[inline]
    pub fn reverse(&self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        Array::reverse(&self.inner.clone().into(), &[], context)?;
        Ok(self.clone())
    }

    /// Calls `Array.prototype.concat()`.
    #[inline]
    pub fn concat(&self, items: &[JsValue], context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let object = Array::concat(&self.inner.clone().into(), items, context)?
            .as_object()
            .js_expect("Array.prototype.concat should always return object")?;

        Self::from_object(object)
    }

    /// Calls `Array.prototype.join()`.
    #[inline]
    pub fn join(
        &self,
        separator: Option<JsString>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsString> {
        let context = context.into_owner();
        let result = Array::join(
            &self.inner.clone().into(),
            &[separator.into_or_undefined()],
            context,
        )?;
        result
            .as_string()
            .js_expect("Array.prototype.join always returns string")
            .map_err(Into::into)
    }

    /// Calls `Array.prototype.fill()`.
    pub fn fill<T>(
        &self,
        value: T,
        start: Option<u32>,
        end: Option<u32>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        Array::fill(
            &self.inner.clone().into(),
            &[
                value.into(),
                start.into_or_undefined(),
                end.into_or_undefined(),
            ],
            context,
        )?;
        Ok(self.clone())
    }

    /// Calls `Array.prototype.indexOf()`.
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<u32>,
        context: ContextAccess<'_>,
    ) -> JsResult<Option<u32>>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        let index = Array::index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .js_expect("Array.prototype.indexOf should always return number")?;

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as u32))
        }
    }

    /// Calls `Array.prototype.lastIndexOf()`.
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<u32>,
        context: ContextAccess<'_>,
    ) -> JsResult<Option<u32>>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        let index = Array::last_index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .js_expect("Array.prototype.lastIndexOf should always return number")?;

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as u32))
        }
    }

    /// Calls `Array.prototype.find()`.
    #[inline]
    pub fn find(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::find(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.filter()`.
    #[inline]
    pub fn filter(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let object = Array::filter(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .js_expect("Array.prototype.filter should always return object")?;

        Self::from_object(object)
    }

    /// Calls `Array.prototype.map()`.
    #[inline]
    pub fn map(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let object = Array::map(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .js_expect("Array.prototype.map should always return object")?;

        Self::from_object(object)
    }

    /// Calls `Array.prototype.every()`.
    #[inline]
    pub fn every(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<bool> {
        let context = context.into_owner();
        let result = Array::every(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .js_expect("Array.prototype.every should always return boolean")?;

        Ok(result)
    }

    /// Calls `Array.prototype.some()`.
    #[inline]
    pub fn some(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<bool> {
        let context = context.into_owner();
        let result = Array::some(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .js_expect("Array.prototype.some should always return boolean")?;

        Ok(result)
    }

    /// Calls `Array.prototype.sort()`.
    #[inline]
    pub fn sort(
        &self,
        compare_fn: Option<JsFunction>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        Array::sort(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(self.clone())
    }

    /// Calls `Array.prototype.slice()`.
    #[inline]
    pub fn slice(
        &self,
        start: Option<u32>,
        end: Option<u32>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let object = Array::slice(
            &self.inner.clone().into(),
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?
        .as_object()
        .js_expect("Array.prototype.slice should always return object")?;

        Self::from_object(object)
    }

    /// Calls `Array.prototype.values()`.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the array belongs to a different runtime.
    #[inline]
    pub fn values(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::values(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.splice()`.
    ///
    /// Removes and/or inserts elements from the array, returning the removed elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsValue};
    /// # use surgeist_js::object::builtins::JsArray;
    /// let context = &mut Context::builder().build().unwrap();
    /// let array = JsArray::from_iter([1, 2, 3].map(JsValue::from), context.access())?;
    ///
    /// // Insert elements at index 1 without removing
    /// let removed = array.splice(
    ///     1,
    ///     Some(0),
    ///     &[JsValue::from(10), JsValue::from(20)],
    ///     context.access(),
    /// ).unwrap();
    ///
    /// assert_eq!(array.length(context.access()).unwrap(), 5);
    /// assert_eq!(removed.length(context.access()).unwrap(), 0);
    /// # Ok::<_, surgeist_js::JsError>(())
    /// ```
    #[inline]
    pub fn splice(
        &self,
        start: u32,
        delete_count: Option<u32>,
        items: &[JsValue],
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let start = JsValue::from(start);
        let delete_count = delete_count.map(JsValue::from);
        let object = Array::splice_internal(
            &self.inner.clone().into(),
            Some(&start),
            delete_count.as_ref(),
            items,
            context,
        )?
        .as_object()
        .js_expect("Array.prototype.splice should always return object")?;

        Self::from_object(object)
    }

    /// Calls `Array.prototype.reduce()`.
    #[inline]
    pub fn reduce(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::reduce(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.reduceRight()`.
    #[inline]
    pub fn reduce_right(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        Array::reduce_right(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.toReversed`.
    #[inline]
    pub fn to_reversed(&self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let array = Array::to_reversed(&self.inner.clone().into(), &[], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .js_expect("`to_reversed` must always return an `Array` on success")?,
        })
    }

    /// Calls `Array.prototype.toSorted`.
    #[inline]
    pub fn to_sorted(
        &self,
        compare_fn: Option<JsFunction>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let array = Array::to_sorted(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: array
                .as_object()
                .js_expect("`to_sorted` must always return an `Array` on success")?,
        })
    }

    /// Calls `Array.prototype.with`.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the array or replacement object belongs to a different runtime.
    #[inline]
    pub fn with(&self, index: u64, value: JsValue, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        context.runtime().check_value(&value)?;
        let array = Array::with(&self.inner.clone().into(), &[index.into(), value], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .js_expect("`with` must always return an `Array` on success")?,
        })
    }
}

impl From<JsArray> for JsObject {
    #[inline]
    fn from(o: JsArray) -> Self {
        o.inner.clone()
    }
}

impl From<JsArray> for JsValue {
    #[inline]
    fn from(o: JsArray) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsArray {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsArray {
    fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not an Array object")
                .into())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_array_factory_preserves_its_heap_refusal_without_panic_wrapping() {
        let context = &mut Context::builder().build().unwrap();
        let runtime = context.runtime().clone();
        runtime
            .heap
            .read(|_| {
                let error = JsArray::new(context.access()).unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&crate::error::EngineError::Rooting(
                        crate::error::RootingError::Access(
                            surgeist_js_gc::heap::AccessError::HeapBorrowed
                        )
                    ))
                );
            })
            .unwrap();

        assert_eq!(
            JsArray::new(context.access())
                .unwrap()
                .length(context.access())
                .unwrap(),
            0
        );
    }

    #[test]
    fn from_iter_rejects_foreign_objects_and_retains_peer_objects() {
        use crate::JsNativeErrorKind;

        let context = &mut Context::builder().build().unwrap();
        let peer = Context::builder()
            .runtime(context.runtime().clone())
            .build()
            .unwrap();
        let foreign = Context::builder().build().unwrap();
        let foreign_object = JsObject::with_null_proto(foreign.runtime()).unwrap();
        let error = JsArray::from_iter([JsValue::from(1), foreign_object.into()], context.access())
            .unwrap_err();
        let error = error.as_native().expect("a native runtime admission error");
        assert_eq!(error.kind(), &JsNativeErrorKind::Type);
        assert_eq!(error.message(), "object belongs to a different runtime");

        let peer_object = JsObject::with_null_proto(peer.runtime()).unwrap();
        let identity = peer_object.edge();
        let array =
            JsArray::from_iter([JsValue::from(1), peer_object.into()], context.access()).unwrap();
        context.runtime().collect().unwrap();
        assert_eq!(array.length(context.access()).unwrap(), 2);
        assert_eq!(array.get(0, context.access()).unwrap(), JsValue::from(1));
        let retained = array.get(1, context.access()).unwrap();
        assert_eq!(
            retained.as_object(),
            Some(identity.upgrade().unwrap().unwrap())
        );
    }

    #[test]
    fn splice_remove() {
        let context = &mut Context::builder().build().unwrap();
        let array = JsArray::from_iter([1, 2, 3].map(JsValue::from), context.access()).unwrap();

        let removed = array.splice(1, Some(1), &[], context.access()).unwrap();

        assert_eq!(array.length(context.access()).unwrap(), 2);
        assert_eq!(removed.length(context.access()).unwrap(), 1);
    }

    #[test]
    fn splice_insert() {
        let context = &mut Context::builder().build().unwrap();
        let array = JsArray::from_iter([1, 2, 3].map(JsValue::from), context.access()).unwrap();

        let removed = array
            .splice(
                1,
                Some(0),
                &[JsValue::from(10), JsValue::from(20)],
                context.access(),
            )
            .unwrap();

        assert_eq!(array.length(context.access()).unwrap(), 5);
        assert_eq!(removed.length(context.access()).unwrap(), 0);
    }

    #[test]
    fn splice_replace() {
        let context = &mut Context::builder().build().unwrap();
        let array = JsArray::from_iter([1, 2, 3].map(JsValue::from), context.access()).unwrap();

        let removed = array
            .splice(1, Some(1), &[JsValue::from(99)], context.access())
            .unwrap();

        assert_eq!(array.length(context.access()).unwrap(), 3);
        assert_eq!(removed.length(context.access()).unwrap(), 1);
    }

    #[test]
    fn splice_from_start() {
        let context = &mut Context::builder().build().unwrap();
        let array = JsArray::from_iter([1, 2, 3].map(JsValue::from), context.access()).unwrap();

        let removed = array.splice(0, Some(1), &[], context.access()).unwrap();

        assert_eq!(array.length(context.access()).unwrap(), 2);
        assert_eq!(removed.length(context.access()).unwrap(), 1);
    }

    #[test]
    fn splice_empty_array() {
        let context = &mut Context::builder().build().unwrap();
        let array = JsArray::new(context.access()).unwrap();

        let removed = array.splice(0, Some(0), &[], context.access()).unwrap();

        assert_eq!(array.length(context.access()).unwrap(), 0);
        assert_eq!(removed.length(context.access()).unwrap(), 0);
    }
}
