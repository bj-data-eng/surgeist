//! Rust API wrappers for the `TypedArray` Builtin ECMAScript Objects
#[cfg(test)]
use crate::Context;
use crate::ContextAccess;
use crate::{
    JsExpect, JsResult, JsString, JsValue,
    builtins::{
        BuiltInConstructor,
        typed_array::{BuiltinTypedArray, TypedArray, TypedArrayKind},
    },
    error::{JsNativeError, ObjectAccessError},
    object::{JsArrayBuffer, JsFunction, JsObject, JsSharedArrayBuffer},
    value::{IntoOrUndefined, TryFromJs},
};
use std::ops::Deref;
use std::sync::atomic::Ordering;

/// `JsTypedArray` provides a wrapper for Boa's implementation of the ECMAScript `TypedArray`
/// builtin object.
#[derive(Debug, Clone)]
pub struct JsTypedArray {
    inner: JsObject,
}

impl JsTypedArray {
    /// Create a [`JsTypedArray`] from a [`JsObject`], if the object is not a typed array throw a
    /// `TypeError`.
    ///
    /// This does not clone the fields of the typed array, it only does a shallow clone of the
    /// object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is::<TypedArray>() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a TypedArray")
                .into())
        }
    }

    /// Return the kind of typed array this is. This can be used in conjunction with
    /// [`js_typed_array_from_kind`] to create a typed array of the same kind.
    #[inline]
    pub fn kind(&self) -> Result<TypedArrayKind, ObjectAccessError> {
        Ok(self
            .inner
            .downcast_ref::<TypedArray>()?
            .ok_or(ObjectAccessError::WrongNativeBrand)?
            .kind())
    }

    /// Get the length of the array.
    ///
    /// Same as `array.length` in JavaScript.
    ///
    /// # Errors
    ///
    /// Returns an error if the length value is not a number.
    #[inline]
    pub fn length(&self, context: ContextAccess<'_>) -> JsResult<usize> {
        let context = context.into_owner();
        Ok(
            BuiltinTypedArray::length(&self.inner.clone().into(), &[], context)?
                .as_number()
                .js_expect("length should return a number")? as usize,
        )
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: ContextAccess<'_>) -> JsResult<bool> {
        let context = context.into_owner();
        Ok(self.length(context.access())? == 0)
    }

    /// Calls `TypedArray.prototype.at()`.
    pub fn at<T>(&self, index: T, context: ContextAccess<'_>) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        let context = context.into_owner();
        BuiltinTypedArray::at(&self.inner.clone().into(), &[index.into().into()], context)
    }

    /// Returns the `ArrayBuffer` referenced by this typed array at construction time.
    ///
    /// Calls `TypedArray.prototype.buffer()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{js_string, JsResult, object::{builtins::{JsUint8Array, JsArrayBuffer}}, property::{PropertyKey}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let array_buffer8 = JsArrayBuffer::new(8, context.access())?;
    /// let array = JsUint8Array::from_array_buffer(array_buffer8, context.access())?;
    /// assert_eq!(
    ///     array.buffer(context.access())?.as_object().unwrap().get(PropertyKey::String(js_string!("byteLength")), context.access()).unwrap(),
    ///     JsValue::new(8)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn buffer(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::buffer(&self.inner.clone().into(), &[], context)
    }

    /// Returns `TypedArray.prototype.byteLength`.
    ///
    /// # Errors
    ///
    /// Returns an error if the byteLength value is not a number.
    #[inline]
    pub fn byte_length(&self, context: ContextAccess<'_>) -> JsResult<usize> {
        let context = context.into_owner();
        Ok(
            BuiltinTypedArray::byte_length(&self.inner.clone().into(), &[], context)?
                .as_number()
                .js_expect("byteLength should return a number")? as usize,
        )
    }

    /// Returns `TypedArray.prototype.byteOffset`.
    ///
    /// # Errors
    ///
    /// Returns an error if the byteOffset value is not a number.
    #[inline]
    pub fn byte_offset(&self, context: ContextAccess<'_>) -> JsResult<usize> {
        let context = context.into_owner();
        Ok(
            BuiltinTypedArray::byte_offset(&self.inner.clone().into(), &[], context)?
                .as_number()
                .js_expect("byteLength should return a number")? as usize,
        )
    }

    /// Function that created the instance object. It is the hidden `TypedArray` constructor function,
    /// but each typed array subclass also defines its own constructor property.
    ///
    /// Returns `TypedArray.prototype.constructor`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array}, JsNativeError, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let array = JsUint8Array::from_iter(vec![1, 2, 3, 4, 5], context.access())?;
    /// assert_eq!(
    ///     Err(JsNativeError::typ()
    ///         .with_message("the TypedArray constructor should never be called directly")
    ///         .into()),
    ///     array.constructor(context.access())
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn constructor(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::constructor(&self.inner.clone().into(), &[], context)
    }

    /// Shallow copies part of this typed array to another location in the same typed
    /// array and returns this typed array without modifying its length.
    ///
    /// Returns `TypedArray.prototype.copyWithin()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, JsValue, object::{builtins::{JsUint8Array}}, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context.access())?;
    /// array.copy_within(3, 1, Some(3), context.access())?;
    /// assert_eq!(array.get(0, context.access())?, JsValue::new(1.0));
    /// assert_eq!(array.get(1, context.access())?, JsValue::new(2.0));
    /// assert_eq!(array.get(2, context.access())?, JsValue::new(3.0));
    /// assert_eq!(array.get(3, context.access())?, JsValue::new(2.0));
    /// assert_eq!(array.get(4, context.access())?, JsValue::new(3.0));
    /// assert_eq!(array.get(5, context.access())?, JsValue::new(6.0));
    /// assert_eq!(array.get(6, context.access())?, JsValue::new(7.0));
    /// assert_eq!(array.get(7, context.access())?, JsValue::new(8.0));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn copy_within<T>(
        &self,
        target: T,
        start: u64,
        end: Option<u64>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        let object = BuiltinTypedArray::copy_within(
            &self.inner.clone().into(),
            &[target.into(), start.into(), end.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .js_expect("`copyWithin` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.fill()`.
    pub fn fill<T>(
        &self,
        value: T,
        start: Option<usize>,
        end: Option<usize>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        BuiltinTypedArray::fill(
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

    /// Calls `TypedArray.prototype.every()`.
    pub fn every(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<bool> {
        let context = context.into_owner();
        let result = BuiltinTypedArray::every(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .js_expect("TypedArray.prototype.every should always return boolean")?;

        Ok(result)
    }

    /// Calls `TypedArray.prototype.some()`.
    #[inline]
    pub fn some(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<bool> {
        let context = context.into_owner();
        let result = BuiltinTypedArray::some(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .js_expect("TypedArray.prototype.some should always return boolean")?;

        Ok(result)
    }

    /// Calls `TypedArray.prototype.sort()`.
    #[inline]
    pub fn sort(
        &self,
        compare_fn: Option<JsFunction>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        BuiltinTypedArray::sort(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(self.clone())
    }

    /// Returns a new typed array on the same `ArrayBuffer` store and with the same element
    /// types as for this typed array.
    /// The begin offset is inclusive and the end offset is exclusive.
    ///
    /// Calls `TypedArray.prototype.subarray()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context.access())?;
    /// let subarray2_6 = array.subarray(2, 6, context.access())?;
    /// assert_eq!(subarray2_6.length(context.access())?, 4);
    /// assert_eq!(subarray2_6.get(0, context.access())?, JsValue::new(3.0));
    /// assert_eq!(subarray2_6.get(1, context.access())?, JsValue::new(4.0));
    /// assert_eq!(subarray2_6.get(2, context.access())?, JsValue::new(5.0));
    /// assert_eq!(subarray2_6.get(3, context.access())?, JsValue::new(6.0));
    /// let subarray4_6 = array.subarray(-4, 6, context.access())?;
    /// assert_eq!(subarray4_6.length(context.access())?, 2);
    /// assert_eq!(subarray4_6.get(0, context.access())?, JsValue::new(5.0));
    /// assert_eq!(subarray4_6.get(1, context.access())?, JsValue::new(6.0));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn subarray(&self, begin: i64, end: i64, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let subarray = BuiltinTypedArray::subarray(
            &self.inner.clone().into(),
            &[begin.into(), end.into()],
            context,
        )?;

        Ok(Self {
            inner: subarray
                .as_object()
                .js_expect("`subarray` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.toLocaleString()`
    #[inline]
    pub fn to_locale_string(
        &self,
        reserved1: Option<JsValue>,
        reserved2: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::to_locale_string(
            &self.inner.clone().into(),
            &[reserved1.into_or_undefined(), reserved2.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.filter()`.
    #[inline]
    pub fn filter(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let object = BuiltinTypedArray::filter(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .js_expect("`filter` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.map()`.
    #[inline]
    pub fn map(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let object = BuiltinTypedArray::map(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .js_expect("`map` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.reduce()`.
    #[inline]
    pub fn reduce(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::reduce(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.reduceRight()`.
    #[inline]
    pub fn reduce_right(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::reduceright(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.reverse()`.
    #[inline]
    pub fn reverse(&self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        BuiltinTypedArray::reverse(&self.inner.clone().into(), &[], context)?;
        Ok(self.clone())
    }

    /// Stores multiple values in the typed array, reading input values from a specified array.
    ///
    /// Returns `TypedArray.prototype.set()`.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::{JsUint8Array, JsArray, JsArrayBuffer}}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let array_buffer8 = JsArrayBuffer::new(8, context.access())?;
    /// let initialized8_array = JsUint8Array::from_array_buffer(array_buffer8, context.access())?;
    /// initialized8_array.set_values(
    ///   JsArray::from_iter(vec![JsValue::new(1), JsValue::new(2)], context.access())?.into(),
    ///   Some(3),
    ///   context.access(),
    /// )?;
    /// assert_eq!(initialized8_array.get(0, context.access())?, JsValue::new(0));
    /// assert_eq!(initialized8_array.get(1, context.access())?, JsValue::new(0));
    /// assert_eq!(initialized8_array.get(2, context.access())?, JsValue::new(0));
    /// assert_eq!(initialized8_array.get(3, context.access())?, JsValue::new(1.0));
    /// assert_eq!(initialized8_array.get(4, context.access())?, JsValue::new(2.0));
    /// assert_eq!(initialized8_array.get(5, context.access())?, JsValue::new(0));
    /// assert_eq!(initialized8_array.get(6, context.access())?, JsValue::new(0));
    /// assert_eq!(initialized8_array.get(7, context.access())?, JsValue::new(0));
    /// assert_eq!(initialized8_array.get(8, context.access())?, JsValue::undefined());
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_values(
        &self,
        source: JsValue,
        offset: Option<u64>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::set(
            &self.inner.clone().into(),
            &[source, offset.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.slice()`.
    #[inline]
    pub fn slice(
        &self,
        start: Option<usize>,
        end: Option<usize>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let object = BuiltinTypedArray::slice(
            &self.inner.clone().into(),
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .js_expect("`slice` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.find()`.
    #[inline]
    pub fn find(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::find(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Returns the index of the first element in an array that satisfies the
    /// provided testing function.
    /// If no elements satisfy the testing function, `JsResult::Ok(None)` is returned.
    ///
    /// Calls `TypedArray.prototype.findIndex()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::builder().build().unwrap();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context.access())?;
    ///
    /// let greater_than_10_predicate = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_ptr(|_this, args, _context| {
    ///         let element = args
    ///             .first()
    ///             .cloned()
    ///             .unwrap_or_default()
    ///             .as_number()
    ///             .expect("error at number conversion");
    ///         Ok(JsValue::from(element > 10.0))
    ///     }),
    /// )
    /// .build().unwrap();
    /// assert_eq!(
    ///     array.find_index(greater_than_10_predicate, None, context.access()),
    ///     Ok(Some(11))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn find_index(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<Option<u64>> {
        let context = context.into_owner();
        let index = BuiltinTypedArray::find_index(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_number()
        .js_expect("TypedArray.prototype.findIndex() should always return number")?;

        if index >= 0.0 {
            Ok(Some(index as u64))
        } else {
            Ok(None)
        }
    }

    /// Iterates the typed array in reverse order and returns the value of
    /// the first element that satisfies the provided testing function.
    /// If no elements satisfy the testing function, `JsResult::Ok(None)` is returned.
    ///
    /// Calls `TypedArray.prototype.findLast()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::builder().build().unwrap();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context.access())?;
    ///
    /// let lower_than_200_predicate = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_ptr(|_this, args, _context| {
    ///         let element = args
    ///             .first()
    ///             .cloned()
    ///             .unwrap_or_default()
    ///             .as_number()
    ///             .expect("error at number conversion");
    ///         Ok(JsValue::from(element < 200.0))
    ///     }),
    /// )
    /// .build().unwrap();
    /// assert_eq!(
    ///     array.find_last(lower_than_200_predicate.clone(), None, context.access()),
    ///     Ok(JsValue::new(199))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn find_last(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::find_last(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Iterates the typed array in reverse order and returns the index of
    /// the first element that satisfies the provided testing function.
    /// If no elements satisfy the testing function, `JsResult::OK(None)` is returned.
    ///
    /// Calls `TypedArray.prototype.findLastIndex()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::builder().build().unwrap();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context.access())?;
    ///
    /// let lower_than_200_predicate = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_ptr(|_this, args, _context| {
    ///         let element = args
    ///             .first()
    ///             .cloned()
    ///             .unwrap_or_default()
    ///             .as_number()
    ///             .expect("error at number conversion");
    ///         Ok(JsValue::from(element < 200.0))
    ///     }),
    /// )
    /// .build().unwrap();
    /// assert_eq!(
    ///     array.find_last(lower_than_200_predicate.clone(), None, context.access()),
    ///     Ok(JsValue::new(199))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn find_last_index(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<Option<u64>> {
        let context = context.into_owner();
        let index = BuiltinTypedArray::find_last_index(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_number()
        .js_expect("TypedArray.prototype.findLastIndex() should always return number")?;

        if index >= 0.0 {
            Ok(Some(index as u64))
        } else {
            Ok(None)
        }
    }

    /// Executes a provided function once for each typed array element.
    ///
    /// Calls `TypedArray.prototype.forEach()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::{cell::Cell, rc::Rc};
    /// # use surgeist_js::{gc::heap::Trace, native_function::NativeCapture};
    /// # #[derive(Clone, Trace)]
    /// # struct Counter(#[heap_trace(skip)] Rc<Cell<u8>>);
    /// # impl NativeCapture for Counter {
    /// #     type Stored = Self;
    /// #     fn store(&self) -> Self { self.clone() }
    /// #     fn restore(stored: &Self) -> Result<Self, surgeist_js::error::RootingError> {
    /// #         Ok(stored.clone())
    /// #     }
    /// # }
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::builder().build().unwrap();
    /// let array = JsUint8Array::from_iter(vec![1, 2, 3, 4, 5], context.access())?;
    /// let num_to_modify = Counter(Rc::new(Cell::new(0)));
    ///
    /// let js_function = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_with_captures(
    ///         |_, args, captures, mut inner_context| {
    ///             let element = args
    ///                 .first()
    ///                 .cloned()
    ///                 .unwrap_or_default()
    ///                 .to_uint8(inner_context.reborrow())
    ///                 .expect("error at number conversion");
    ///             captures.0.set(captures.0.get() + element);
    ///             Ok(JsValue::undefined())
    ///         },
    ///         num_to_modify.clone(),
    ///     ),
    /// )
    /// .build().unwrap();
    ///
    /// array.for_each(js_function, None, context.access())?;
    /// assert_eq!(num_to_modify.0.get(), 15u8);
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn for_each(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::for_each(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Determines whether a typed array includes a certain value among its entries,
    /// returning true or false as appropriate.
    ///
    /// Calls `TypedArray.prototype.includes()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, object::{builtins::JsUint8Array}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context.access())?;
    ///
    /// assert_eq!(array.includes(JsValue::new(2), None, context.access()), Ok(true));
    /// let empty_array = JsUint8Array::from_iter(vec![], context.access())?;
    /// assert_eq!(
    ///     empty_array.includes(JsValue::new(2), None, context.access()),
    ///     Ok(false)
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn includes<T>(
        &self,
        search_element: T,
        from_index: Option<u64>,
        context: ContextAccess<'_>,
    ) -> JsResult<bool>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        let result = BuiltinTypedArray::includes(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .js_expect("TypedArray.prototype.includes should always return boolean")?;

        Ok(result)
    }

    /// Calls `TypedArray.prototype.indexOf()`.
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: ContextAccess<'_>,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        let index = BuiltinTypedArray::index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .js_expect("TypedArray.prototype.indexOf should always return number")?;

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as usize))
        }
    }

    /// Calls `TypedArray.prototype.lastIndexOf()`.
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: ContextAccess<'_>,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        let index = BuiltinTypedArray::last_index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .js_expect("TypedArray.prototype.lastIndexOf should always return number")?;

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as usize))
        }
    }

    /// Calls `TypedArray.prototype.join()`.
    #[inline]
    pub fn join(
        &self,
        separator: Option<JsString>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsString> {
        let context = context.into_owner();
        BuiltinTypedArray::join(
            &self.inner.clone().into(),
            &[separator.into_or_undefined()],
            context,
        )
        .and_then(|x| {
            x.as_string()
                .js_expect("TypedArray.prototype.join always returns string")
                .map_err(Into::into)
        })
    }

    /// Calls `TypedArray.prototype.toReversed ( )`.
    #[inline]
    pub fn to_reversed(&self, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let array = BuiltinTypedArray::to_reversed(&self.inner.clone().into(), &[], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .js_expect("`to_reversed` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.toSorted ( comparefn )`.
    #[inline]
    pub fn to_sorted(
        &self,
        compare_fn: Option<JsFunction>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let array = BuiltinTypedArray::to_sorted(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: array
                .as_object()
                .js_expect("`to_sorted` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.with ( index, value )`.
    #[inline]
    pub fn with(&self, index: u64, value: JsValue, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let array =
            BuiltinTypedArray::with(&self.inner.clone().into(), &[index.into(), value], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .js_expect("`with` must always return a `TypedArray` on success")?,
        })
    }

    /// Calls `TypedArray.prototype.entries()`.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the array belongs to a different runtime or is out of bounds.
    #[inline]
    pub fn entries(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        BuiltinTypedArray::entries(&self.inner.clone().into(), &[], context)
    }

    /// Calls `TypedArray.prototype.keys()`.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the array belongs to a different runtime or is out of bounds.
    #[inline]
    pub fn keys(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        BuiltinTypedArray::keys(&self.inner.clone().into(), &[], context)
    }

    /// Calls `TypedArray.prototype.values()`.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the array belongs to a different runtime or is out of bounds.
    #[inline]
    pub fn values(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        BuiltinTypedArray::values(&self.inner.clone().into(), &[], context)
    }

    /// Calls `TypedArray.prototype[@@iterator]()`.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the array belongs to a different runtime or is out of bounds.
    #[inline]
    pub fn iterator(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        self.values(context.access())
    }

    /// Calls `TypedArray.prototype.toString()`.
    #[inline]
    pub fn to_string(&self, context: ContextAccess<'_>) -> JsResult<JsString> {
        let context = context.into_owner();
        // TypedArray.prototype.toString is the same as Array.prototype.toString
        let result = crate::builtins::Array::to_string(&self.inner.clone().into(), &[], context)?;
        result
            .as_string()
            .js_expect("Array.prototype.toString always returns string")
            .map_err(Into::into)
    }

    /// It is a getter that returns the same string as the typed array constructor's name.
    /// It returns `Ok(JsValue::Undefined)` if the this value is not one of the typed array subclasses.
    ///
    /// Returns `TypedArray.prototype.toStringTag()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsResult, js_string, object::{builtins::{JsUint8Array}}, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context.access())?;
    /// let tag = array.to_string_tag(context.access())?.to_string(context.access())?;
    /// assert_eq!(tag, js_string!("Uint8Array"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn to_string_tag(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        BuiltinTypedArray::to_string_tag(&self.inner.clone().into(), &[], context)
    }
}

impl From<JsTypedArray> for JsObject {
    #[inline]
    fn from(o: JsTypedArray) -> Self {
        o.inner.clone()
    }
}

impl From<JsTypedArray> for JsValue {
    #[inline]
    fn from(o: JsTypedArray) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsTypedArray {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsTypedArray {
    fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not a TypedArray object")
                .into())
        }
    }
}

macro_rules! JsTypedArrayType {
    (
        $name:ident,
        $constructor_function:ident,
        $kind:expr,
        $constructor_object:ident,
        $value_to_elem:ident,
        $element:ty
    ) => {

        #[doc = concat!(
            "`", stringify!($name),
            "` provides a wrapper for Boa's implementation of the ECMAScript `",
            stringify!($constructor_function) ,"` builtin object."
        )]
        #[derive(Debug, Clone)]
        pub struct $name {
            inner: JsTypedArray,
        }

        impl $name {
            #[doc = concat!("Creates a `", stringify!($name),
                "` using a [`JsObject`]. It will make sure that the object is of the correct kind."
            )]
            #[inline]
            pub fn from_object(object: JsObject) -> JsResult<Self> {
                let is_kind = if let Some(int) = object.downcast_ref::<TypedArray>()? {
                    int.kind() == $kind
                } else {
                    false
                };
                if is_kind {
                    Ok(Self {
                        inner: JsTypedArray {
                            inner: object.into(),
                        },
                    })
                } else {
                    Err(JsNativeError::typ()
                        .with_message("object is not a TypedArray")
                        .into())
                }
            }

            /// Create the typed array from a [`JsArrayBuffer`].
            ///
            /// # Errors
            ///
            /// Returns a `TypeError` if the buffer belongs to a different runtime.
            pub fn from_array_buffer(
                array_buffer: JsArrayBuffer,
                context: ContextAccess<'_>,
            ) -> JsResult<Self> {
                let context = context.into_owner();
                let new_target = context
                    .intrinsics()
                    .constructors()
                    .$constructor_object()
                    .constructor()?
                    .into();
                let object = crate::builtins::typed_array::$constructor_function::constructor(
                    &new_target,
                    &[array_buffer.into()],
                    context,
                )?
                .as_object()
                .js_expect("object")?
                .clone();

                Ok(Self {
                    inner: JsTypedArray {
                        inner: object.into(),
                    },
                })
            }

            /// Create the typed array from a [`JsSharedArrayBuffer`].
            ///
            /// # Errors
            ///
            /// Returns a `TypeError` if the buffer belongs to a different runtime.
            pub fn from_shared_array_buffer(
                buffer: JsSharedArrayBuffer,
                context: ContextAccess<'_>,
            ) -> JsResult<Self> {
                let context = context.into_owner();
                let new_target = context
                    .intrinsics()
                    .constructors()
                    .$constructor_object()
                    .constructor()?
                    .into();
                let object = crate::builtins::typed_array::$constructor_function::constructor(
                    &new_target,
                    &[buffer.into()],
                    context,
                )?
                .as_object()
                .js_expect("object")?
                .clone();

                Ok(Self {
                    inner: JsTypedArray {
                        inner: object.into(),
                    },
                })
            }

            /// Create the typed array from an iterator.
            pub fn from_iter<I>(elements: I, context: ContextAccess<'_>) -> JsResult<Self>
            where
                I: IntoIterator<Item = $element>,
            {
                let context = context.into_owner();
                let bytes = elements
                    .into_iter()
                    .flat_map(<$element>::to_ne_bytes)
                    .collect();
                let array_buffer = JsArrayBuffer::from_byte_block(bytes, context.access())?;
                let new_target = context
                    .intrinsics()
                    .constructors()
                    .$constructor_object()
                    .constructor()?
                    .into();
                let object = crate::builtins::typed_array::$constructor_function::constructor(
                    &new_target,
                    &[array_buffer.into()],
                    context,
                )?
                .as_object()
                .js_expect("object")?
                .clone();

                Ok(Self {
                    inner: JsTypedArray {
                        inner: object.into(),
                    },
                })
            }

            /// Create an iterator over the typed array's elements.
            ///
            /// The initial length is captured when this method succeeds. An initially
            /// detached or out-of-bounds view has length zero. Each item preserves
            /// the indexed read and scalar conversion result. The first item error
            /// is yielded once, after which the iterator remains exhausted.
            ///
            /// # Errors
            ///
            /// Returns an error if the array belongs to another runtime or its
            /// initial length cannot be read.
            ///
            /// The iterator reserves its context access until it is dropped.
            /// A shorter reborrow cannot escape its access capability:
            ///
            /// ```compile_fail
            /// use surgeist_js::{Context, object::builtins::JsUint8Array};
            ///
            /// let mut owner = Context::builder().build().unwrap();
            /// let array = JsUint8Array::from_iter([1u8], owner.access()).unwrap();
            /// let escaped = {
            ///     let mut access = owner.access();
            ///     array.iter(access.reborrow()).unwrap()
            /// };
            /// let _ = escaped.count();
            /// ```
            pub fn iter<'a>(
                &'a self,
                context: ContextAccess<'a>,
            ) -> JsResult<impl std::iter::FusedIterator<Item = JsResult<$element>> + 'a> {
                let context = context.into_owner();
                self.check_runtime(context.runtime())?;
                let length = self.length(context.access())?;
                let mut index = 0;
                let mut done = false;
                Ok(std::iter::from_fn(move || {
                    if done || index == length {
                        done = true;
                        return None;
                    }
                    let result = self.get(index, context.access())
                        .and_then(|value| value.$value_to_elem(context.access()));
                    match &result {
                        Ok(_) => index += 1,
                        Err(_) => done = true,
                    }
                    Some(result)
                }).fuse())
            }
        }

        impl From<$name> for JsObject {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner
                    .inner
                    .clone()
            }
        }

        impl From<$name> for JsValue {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner.inner.clone().into()
            }
        }

        impl Deref for $name {
            type Target = JsTypedArray;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl TryFromJs for $name {
            fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
                if let Some(o) = value.as_object() {
                    Self::from_object(o.clone())
                } else {
                    Err(JsNativeError::typ()
                        .with_message(concat!(
                            "value is not a ",
                            stringify!($constructor_function),
                            " object"
                        ))
                        .into())
                }
            }
        }
    };
}

JsTypedArrayType!(
    JsUint8Array,
    Uint8Array,
    TypedArrayKind::Uint8,
    typed_uint8_array,
    to_uint8,
    u8
);

impl JsUint8Array {
    /// Copies the viewed byte range of this `Uint8Array` into a new `Vec<u8>`.
    ///
    /// Works with both `ArrayBuffer` and `SharedArrayBuffer` backing storage. The buffer must not be
    /// detached; otherwise a `TypeError` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use surgeist_js::{Context, JsResult, object::builtins::JsUint8Array};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::builder().build().unwrap();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data.clone(), context.access())?;
    /// let bytes = array.to_vec(context.access())?;
    /// assert_eq!(bytes, data);
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn to_vec(&self, _context: ContextAccess<'_>) -> JsResult<Vec<u8>> {
        let this_val = self.inner.inner.clone().into();
        let (obj, buf_byte_len) = TypedArray::validate(&this_val, Ordering::SeqCst)?;
        let vec = {
            let array = obj.borrow()?;
            let ta = array.data();
            let buffer = ta.viewed_array_buffer();
            let buffer = buffer.as_buffer()?;
            let Some(slice) = buffer.bytes(Ordering::SeqCst) else {
                return Err(JsNativeError::typ()
                    .with_message("typed array buffer is detached or out of bounds")
                    .into());
            };
            if ta.is_out_of_bounds(slice.len()) {
                return Err(JsNativeError::typ()
                    .with_message("typed array is outside the bounds of its inner buffer")
                    .into());
            }
            let byte_offset = ta.byte_offset() as usize;
            let byte_len = ta.byte_length(buf_byte_len) as usize;
            slice.subslice(byte_offset..byte_offset + byte_len).to_vec()
        };
        Ok(vec)
    }
}

JsTypedArrayType!(
    JsUint8ClampedArray,
    Uint8ClampedArray,
    TypedArrayKind::Uint8Clamped,
    typed_uint8clamped_array,
    to_uint8_clamp,
    u8
);
JsTypedArrayType!(
    JsInt8Array,
    Int8Array,
    TypedArrayKind::Int8,
    typed_int8_array,
    to_int8,
    i8
);
JsTypedArrayType!(
    JsUint16Array,
    Uint16Array,
    TypedArrayKind::Uint16,
    typed_uint16_array,
    to_uint16,
    u16
);
JsTypedArrayType!(
    JsInt16Array,
    Int16Array,
    TypedArrayKind::Int16,
    typed_int16_array,
    to_int16,
    i16
);
JsTypedArrayType!(
    JsUint32Array,
    Uint32Array,
    TypedArrayKind::Uint32,
    typed_uint32_array,
    to_u32,
    u32
);
JsTypedArrayType!(
    JsInt32Array,
    Int32Array,
    TypedArrayKind::Int32,
    typed_int32_array,
    to_i32,
    i32
);
#[cfg(feature = "float16")]
JsTypedArrayType!(
    JsFloat16Array,
    Float16Array,
    TypedArrayKind::Float16,
    typed_float16_array,
    to_f16,
    float16::f16
);
JsTypedArrayType!(
    JsBigInt64Array,
    BigInt64Array,
    TypedArrayKind::BigInt64,
    typed_bigint64_array,
    to_big_int64,
    i64
);
JsTypedArrayType!(
    JsBigUint64Array,
    BigUint64Array,
    TypedArrayKind::BigUint64,
    typed_biguint64_array,
    to_big_uint64,
    u64
);
JsTypedArrayType!(
    JsFloat32Array,
    Float32Array,
    TypedArrayKind::Float32,
    typed_float32_array,
    to_f32,
    f32
);
JsTypedArrayType!(
    JsFloat64Array,
    Float64Array,
    TypedArrayKind::Float64,
    typed_float64_array,
    to_number,
    f64
);

/// Create a [`JsTypedArray`] from a [`TypedArrayKind`] and a backing [`JsArrayBuffer`].
#[inline]
pub fn js_typed_array_from_kind(
    kind: TypedArrayKind,
    inner: JsArrayBuffer,
    context: ContextAccess<'_>,
) -> JsResult<JsValue> {
    let context = context.into_owner();
    match kind {
        TypedArrayKind::Int8 => {
            JsInt8Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Uint8 => {
            JsUint8Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Uint8Clamped => {
            JsUint8ClampedArray::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Int16 => {
            JsInt16Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Uint16 => {
            JsUint16Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Int32 => {
            JsInt32Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Uint32 => {
            JsUint32Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::BigInt64 => {
            JsBigInt64Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::BigUint64 => {
            JsBigUint64Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        #[cfg(feature = "float16")]
        TypedArrayKind::Float16 => {
            JsFloat16Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Float32 => {
            JsFloat32Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
        TypedArrayKind::Float64 => {
            JsFloat64Array::from_array_buffer(inner, context.access()).map(Into::into)
        }
    }
}

#[test]
fn typed_iterators_uint8() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![1u8, 2, 3, 4, 5, 6, 7, 8];

    let array = JsUint8Array::from_iter(vec.clone(), context.access()).unwrap();
    let vec2 = array
        .iter(context.access())
        .unwrap()
        .collect::<JsResult<Vec<_>>>()
        .unwrap();
    assert_eq!(vec, vec2);
}

#[test]
fn uint8_array_to_vec_roundtrip() {
    let context = &mut Context::builder().build().unwrap();
    let data: Vec<u8> = (0..=255).collect();
    let array = JsUint8Array::from_iter(data.clone(), context.access()).unwrap();
    let bytes = array.to_vec(context.access()).unwrap();
    assert_eq!(bytes, data);
}

#[test]
fn typed_iterators_uint32() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![1u32, 2, 0xFFFF, 4, 0xFF12_3456, 6, 7, 8];

    let array = JsUint32Array::from_iter(vec.clone(), context.access()).unwrap();
    let vec2 = array
        .iter(context.access())
        .unwrap()
        .collect::<JsResult<Vec<_>>>()
        .unwrap();
    assert_eq!(vec, vec2);
}

#[test]
fn typed_iterators_f32() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![0.1f32, 0.2, 0.3, 0.4, 1.1, 9.99999];

    let array = JsFloat32Array::from_iter(vec.clone(), context.access()).unwrap();
    let vec2 = array
        .iter(context.access())
        .unwrap()
        .collect::<JsResult<Vec<_>>>()
        .unwrap();
    assert_eq!(vec, vec2);
}

#[cfg(test)]
mod iterator_access_tests {
    use super::*;
    use crate::{JsError, JsNativeErrorKind, error::EngineError};

    fn assert_borrowed(error: &JsError) {
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
        );
    }

    #[test]
    fn typed_iterator_preparation_preserves_array_and_buffer_access_errors() {
        for borrow_array in [true, false] {
            let context = &mut Context::builder().build().unwrap();
            let buffer = JsArrayBuffer::from_byte_block(vec![7, 11], context.access()).unwrap();
            let array = JsUint8Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
            let alias: JsObject = if borrow_array {
                array.clone().into()
            } else {
                buffer.into()
            };
            let view = alias.borrow_mut().unwrap();

            assert_borrowed(&array.length(context.access()).unwrap_err());
            assert_borrowed(&array.iter(context.access()).err().unwrap());
            drop(view);

            let values = array
                .iter(context.access())
                .unwrap()
                .collect::<JsResult<Vec<_>>>()
                .unwrap();
            assert_eq!(values, vec![7, 11]);
        }
    }

    #[test]
    fn typed_iterator_item_access_error_is_yielded_once_without_consuming_the_array() {
        for borrow_array in [true, false] {
            let context = &mut Context::builder().build().unwrap();
            let buffer = JsArrayBuffer::from_byte_block(vec![7, 11, 13], context.access()).unwrap();
            let array = JsUint8Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
            let alias: JsObject = if borrow_array {
                array.clone().into()
            } else {
                buffer.into()
            };
            let mut iterator = array.iter(context.access()).unwrap();
            assert_eq!(iterator.next().unwrap().unwrap(), 7);

            let view = alias.borrow_mut().unwrap();
            assert_borrowed(&iterator.next().unwrap().unwrap_err());
            assert!(iterator.next().is_none());
            drop(view);
            assert!(iterator.next().is_none());
            assert!(iterator.next().is_none());
            drop(iterator);

            let values = array
                .iter(context.access())
                .unwrap()
                .collect::<JsResult<Vec<_>>>()
                .unwrap();
            assert_eq!(values, vec![7, 11, 13]);
        }
    }

    #[test]
    fn typed_iterators_keep_empty_and_initially_detached_views_empty() {
        let context = &mut Context::builder().build().unwrap();
        let empty = JsUint8Array::from_iter([], context.access()).unwrap();
        assert!(empty.iter(context.access()).unwrap().next().is_none());

        let buffer = JsArrayBuffer::from_byte_block(vec![0; 8], context.access()).unwrap();
        let numbers = JsUint8Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
        let bigints = JsBigInt64Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
        buffer.detach(&JsValue::undefined()).unwrap();
        assert!(numbers.iter(context.access()).unwrap().next().is_none());
        assert!(bigints.iter(context.access()).unwrap().next().is_none());
    }

    #[test]
    fn typed_iterators_preserve_numeric_conversion_after_detachment() {
        let context = &mut Context::builder().build().unwrap();
        let buffer = JsArrayBuffer::from_byte_block(vec![7, 11], context.access()).unwrap();
        let array = JsUint8Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
        let mut iterator = array.iter(context.access()).unwrap();
        assert_eq!(iterator.next().unwrap().unwrap(), 7);
        buffer.detach(&JsValue::undefined()).unwrap();
        assert_eq!(iterator.next().unwrap().unwrap(), 0);
        assert!(iterator.next().is_none());
        drop(iterator);

        let bytes = [1.25_f32, 2.5]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        let buffer = JsArrayBuffer::from_byte_block(bytes, context.access()).unwrap();
        let array = JsFloat32Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
        let mut iterator = array.iter(context.access()).unwrap();
        assert_eq!(iterator.next().unwrap().unwrap(), 1.25);
        buffer.detach(&JsValue::undefined()).unwrap();
        assert!(iterator.next().unwrap().unwrap().is_nan());
        assert!(iterator.next().is_none());
    }

    #[test]
    fn typed_iterator_yields_the_bigint_conversion_error_once_after_detachment() {
        let context = &mut Context::builder().build().unwrap();
        let bytes = [7_i64, 11].into_iter().flat_map(i64::to_ne_bytes).collect();
        let buffer = JsArrayBuffer::from_byte_block(bytes, context.access()).unwrap();
        let array = JsBigInt64Array::from_array_buffer(buffer.clone(), context.access()).unwrap();
        let mut iterator = array.iter(context.access()).unwrap();
        assert_eq!(iterator.next().unwrap().unwrap(), 7);
        buffer.detach(&JsValue::undefined()).unwrap();

        let error = iterator.next().unwrap().unwrap_err();
        assert_eq!(error.as_native().unwrap().kind(), &JsNativeErrorKind::Type);
        assert!(error.as_engine().is_none());
        assert!(iterator.next().is_none());
        assert!(iterator.next().is_none());
    }

    #[test]
    fn typed_iterator_preparation_rejects_even_an_empty_foreign_view() {
        let owner = &mut Context::builder().build().unwrap();
        let foreign = &mut Context::builder().build().unwrap();
        let array = JsUint8Array::from_iter([], owner.access()).unwrap();
        let error = array.iter(foreign.access()).err().unwrap();
        assert_eq!(error.as_native().unwrap().kind(), &JsNativeErrorKind::Type);
        assert!(error.as_engine().is_none());
        assert!(array.iter(owner.access()).unwrap().next().is_none());
    }
}

#[test]
fn typed_array_to_string() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![1u8, 2, 3];
    let array = JsUint8Array::from_iter(vec, context.access()).unwrap();
    assert_eq!(
        array.to_string(context.access()).unwrap(),
        crate::js_string!("1,2,3")
    );
}

#[test]
fn typed_array_entries() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![1u8, 2];
    let array = JsUint8Array::from_iter(vec, context.access()).unwrap();
    let entries = array.entries(context.access()).unwrap();
    let mut entries_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = entries
            .as_object()
            .unwrap()
            .get(next_str.clone(), context.access())
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&entries, &[], context.access())
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context.access())
            .unwrap()
            .to_boolean()
        {
            break;
        }
        entries_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context.access())
                .unwrap(),
        );
    }
    assert_eq!(entries_vec.len(), 2);
}

#[test]
fn typed_array_keys() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![1u8, 2];
    let array = JsUint8Array::from_iter(vec, context.access()).unwrap();
    let keys = array.keys(context.access()).unwrap();
    let mut keys_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = keys
            .as_object()
            .unwrap()
            .get(next_str.clone(), context.access())
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&keys, &[], context.access())
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context.access())
            .unwrap()
            .to_boolean()
        {
            break;
        }
        keys_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context.access())
                .unwrap(),
        );
    }
    assert_eq!(keys_vec, vec![JsValue::new(0), JsValue::new(1)]);
}

#[test]
fn typed_array_values() {
    let context = &mut Context::builder().build().unwrap();
    let vec = vec![1u8, 2];
    let array = JsUint8Array::from_iter(vec, context.access()).unwrap();
    let values = array.values(context.access()).unwrap();
    let mut values_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = values
            .as_object()
            .unwrap()
            .get(next_str.clone(), context.access())
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&values, &[], context.access())
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context.access())
            .unwrap()
            .to_boolean()
        {
            break;
        }
        values_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context.access())
                .unwrap(),
        );
    }
    assert_eq!(values_vec, vec![JsValue::new(1), JsValue::new(2)]);
}

#[test]
fn typed_array_iterator() {
    let context = &mut Context::builder().build().unwrap();
    let array = JsUint8Array::from_iter(vec![1u8, 2], context.access()).unwrap();
    let values = array.iterator(context.access()).unwrap();
    let mut values_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = values
            .as_object()
            .unwrap()
            .get(next_str.clone(), context.access())
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&values, &[], context.access())
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context.access())
            .unwrap()
            .to_boolean()
        {
            break;
        }
        values_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context.access())
                .unwrap(),
        );
    }
    assert_eq!(values_vec, vec![JsValue::new(1), JsValue::new(2)]);
}

#[test]
fn typed_array_to_reversed() {
    let context = &mut Context::builder().build().unwrap();
    let array = JsUint8Array::from_iter(vec![3u8, 1, 2], context.access()).unwrap();

    let reversed = array.to_reversed(context.access()).unwrap();

    // New array has reversed order
    assert_eq!(
        reversed.at(0i64, context.access()).unwrap(),
        JsValue::new(2)
    );
    assert_eq!(
        reversed.at(1i64, context.access()).unwrap(),
        JsValue::new(1)
    );
    assert_eq!(
        reversed.at(2i64, context.access()).unwrap(),
        JsValue::new(3)
    );

    // Original is unchanged
    assert_eq!(array.at(0i64, context.access()).unwrap(), JsValue::new(3));
    assert_eq!(array.at(1i64, context.access()).unwrap(), JsValue::new(1));
    assert_eq!(array.at(2i64, context.access()).unwrap(), JsValue::new(2));
}

#[test]
fn typed_array_to_sorted() {
    let context = &mut Context::builder().build().unwrap();
    let array = JsUint8Array::from_iter(vec![3u8, 1, 2], context.access()).unwrap();

    let sorted = array.to_sorted(None, context.access()).unwrap();

    // New array is sorted
    assert_eq!(sorted.at(0i64, context.access()).unwrap(), JsValue::new(1));
    assert_eq!(sorted.at(1i64, context.access()).unwrap(), JsValue::new(2));
    assert_eq!(sorted.at(2i64, context.access()).unwrap(), JsValue::new(3));

    // Original is unchanged
    assert_eq!(array.at(0i64, context.access()).unwrap(), JsValue::new(3));
    assert_eq!(array.at(1i64, context.access()).unwrap(), JsValue::new(1));
    assert_eq!(array.at(2i64, context.access()).unwrap(), JsValue::new(2));
}

#[test]
fn typed_array_to_locale_string() {
    let context = &mut Context::builder().build().unwrap();
    let array = JsUint8Array::from_iter(vec![1u8, 2, 3], context.access()).unwrap();

    let result = array
        .to_locale_string(None, None, context.access())
        .unwrap();

    let result_str = result
        .as_string()
        .expect("toLocaleString should return a string");
    assert!(
        result_str.to_std_string_escaped().contains('1'),
        "result should contain element values"
    );
}
