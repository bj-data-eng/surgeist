//! A Rust API wrapper for Boa's `ArrayBuffer` Builtin ECMAScript Object
use crate::ContextAccess;
use crate::{
    JsResult, JsValue,
    builtins::array_buffer::ArrayBuffer,
    context::intrinsics::StandardConstructors,
    error::{JsNativeError, ObjectAccessError},
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    value::TryFromJs,
};
use std::{
    cell::{Ref, RefMut},
    ops::Deref,
};

/// `JsArrayBuffer` provides a wrapper for Boa's implementation of the ECMAScript `ArrayBuffer` object
#[derive(Debug, Clone)]
pub struct JsArrayBuffer {
    inner: JsObject<ArrayBuffer>,
}

impl From<JsArrayBuffer> for JsObject<ArrayBuffer> {
    #[inline]
    fn from(value: JsArrayBuffer) -> Self {
        value.inner
    }
}

impl From<JsObject<ArrayBuffer>> for JsArrayBuffer {
    #[inline]
    fn from(value: JsObject<ArrayBuffer>) -> Self {
        Self { inner: value }
    }
}

// TODO: Add constructors that also take the `detach_key` as argument.
impl JsArrayBuffer {
    /// Create a new array buffer with byte length.
    ///
    /// ```
    /// # use surgeist_js::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult, JsValue
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::builder().build().unwrap();
    /// // Creates a blank array buffer of n bytes
    /// let array_buffer = JsArrayBuffer::new(4, context.access())?;
    ///
    /// assert_eq!(
    ///     array_buffer.detach(&JsValue::undefined())?.as_slice(),
    ///     &[0u8, 0, 0, 0]
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(byte_length: usize, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let inner = ArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()?
                .into(),
            byte_length as u64,
            None,
            context,
        )?;

        Ok(Self { inner })
    }

    /// Create a new array buffer from byte block.
    ///
    /// This uses the passed byte block as the internal storage, it does not clone it!
    ///
    /// The `byte_length` will be set to `byte_block.len()`.
    ///
    /// ```
    /// # use surgeist_js::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult, JsValue,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::builder().build().unwrap();
    ///
    /// // Create a buffer from a chunk of data
    /// let data_block = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context.access())?;
    ///
    /// assert_eq!(
    ///     array_buffer.detach(&JsValue::undefined())?.as_slice(),
    ///     &[0u8, 1, 2, 3, 4]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_byte_block(byte_block: Vec<u8>, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let constructor = context
            .intrinsics()
            .constructors()
            .array_buffer()
            .constructor()?
            .into();

        // 1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] »).
        let prototype = get_prototype_from_constructor(
            &constructor,
            StandardConstructors::array_buffer,
            context,
        )?;

        // 2. Let block be ? CreateByteDataBlock(byteLength).
        //
        // NOTE: We skip step 2. because we already have the block
        // that is passed to us as a function argument.
        let block = byte_block;

        // 3. Set obj.[[ArrayBufferData]] to block.
        // 4. Set obj.[[ArrayBufferByteLength]] to byteLength.
        let obj = JsObject::new(
            context.root_shape(),
            prototype,
            ArrayBuffer::from_data(block, &JsValue::undefined()),
        )?;

        Ok(Self { inner: obj })
    }

    /// Set a maximum length for the underlying array buffer.
    #[inline]
    pub fn with_max_byte_length(self, max_byte_len: u64) -> Result<Self, ObjectAccessError> {
        self.inner
            .borrow_mut()?
            .data_mut()
            .set_max_byte_length(max_byte_len);
        Ok(self)
    }

    /// Create a [`JsArrayBuffer`] from a [`JsObject`], if the object is not an array buffer throw a `TypeError`.
    ///
    /// This does not clone the fields of the array buffer, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        object
            .downcast::<ArrayBuffer>()
            .map(|inner| Self { inner })
            .map_err(|_| {
                JsNativeError::typ()
                    .with_message("object is not an ArrayBuffer")
                    .into()
            })
    }

    /// Returns the byte length of the array buffer.
    ///
    /// ```
    /// # use surgeist_js::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::builder().build().unwrap();
    /// // Create a buffer from a chunk of data
    /// let data_block = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context.access())?;
    ///
    /// // Take the inner buffer
    /// let buffer_length = array_buffer.byte_length()?;
    ///
    /// assert_eq!(buffer_length, 5);
    /// # Ok(())
    /// # }
    ///  ```
    #[inline]
    pub fn byte_length(&self) -> Result<usize, ObjectAccessError> {
        Ok(self.inner.borrow()?.data().len())
    }

    /// Take the inner `ArrayBuffer`'s `array_buffer_data` field and replace it with `None`
    ///
    /// # Note
    ///
    /// This tries to detach the pre-existing `JsArrayBuffer`, meaning the original detach
    /// key is required. By default, the key is set to `undefined`.
    ///
    /// ```
    /// # use surgeist_js::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult, JsValue
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::builder().build().unwrap();
    /// // Create a buffer from a chunk of data
    /// let data_block = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context.access())?;
    ///
    /// // Take the inner buffer
    /// let internal_buffer = array_buffer.detach(&JsValue::undefined())?;
    ///
    /// assert_eq!(internal_buffer.as_slice(), &[0u8, 1, 2, 3, 4]);
    ///
    /// // Anymore interaction with the buffer will return an error
    /// let detached_err = array_buffer.detach(&JsValue::undefined());
    /// assert!(detached_err.is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn detach(&self, detach_key: &JsValue) -> JsResult<Vec<u8>> {
        self.detach_if_attached(detach_key)?.ok_or_else(|| {
            JsNativeError::typ()
                .with_message("ArrayBuffer was already detached")
                .into()
        })
    }

    /// Detaches the buffer and returns its bytes if it was attached.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the detach key does not match.
    #[inline]
    pub fn detach_if_attached(&self, detach_key: &JsValue) -> JsResult<Option<Vec<u8>>> {
        self.inner.borrow_mut()?.data_mut().detach(detach_key)
    }

    /// Get an immutable reference to the [`JsArrayBuffer`]'s data.
    ///
    /// Returns `None` if detached.
    ///
    /// ```
    /// # use surgeist_js::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult, JsValue,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::builder().build().unwrap();
    /// // Create a buffer from a chunk of data
    /// let data_block = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context.access())?;
    ///
    /// // Get a reference to the data.
    /// let internal_buffer = array_buffer.data()?;
    ///
    /// assert_eq!(internal_buffer.as_deref(), Some(&[0u8, 1, 2, 3, 4][..]));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn data(&self) -> Result<Option<Ref<'_, [u8]>>, ObjectAccessError> {
        Ok(Ref::filter_map(self.inner.borrow_data()?, ArrayBuffer::bytes).ok())
    }

    /// Copies the contents of this [`JsArrayBuffer`] into a new [`Vec<u8>`].
    ///
    /// Returns `None` if the buffer has been detached.
    ///
    /// See also [`crate::object::builtins::JsUint8Array::to_vec`] and
    /// [`crate::object::builtins::JsSharedArrayBuffer::to_vec`].
    ///
    /// # Example
    ///
    /// ```
    /// # use surgeist_js::object::builtins::JsArrayBuffer;
    /// # use surgeist_js::{Context, JsResult, JsValue};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::builder().build().unwrap();
    /// let data = vec![1u8, 2, 3, 4];
    /// let buffer = JsArrayBuffer::from_byte_block(data, context.access())?;
    /// assert_eq!(buffer.to_vec()?, Some(vec![1u8, 2, 3, 4]));
    ///
    /// buffer.detach(&JsValue::undefined())?;
    /// assert_eq!(buffer.to_vec()?, None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn to_vec(&self) -> Result<Option<Vec<u8>>, ObjectAccessError> {
        Ok(self.data()?.map(|data| data.to_vec()))
    }

    /// Get a mutable reference to the [`JsArrayBuffer`]'s data.
    ///
    /// Returns `None` if detached.
    ///
    /// ```
    /// # use surgeist_js::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult, JsValue
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::builder().build().unwrap();
    /// // Create a buffer from a chunk of data
    /// let data_block = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context.access())?;
    ///
    /// // Get a reference to the data.
    /// let mut internal_buffer = array_buffer
    ///     .data_mut()?
    ///     .expect("the buffer should not be detached");
    ///
    /// internal_buffer.fill(10);
    ///
    /// assert_eq!(&*internal_buffer, &[10u8, 10, 10, 10, 10]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn data_mut(&self) -> Result<Option<RefMut<'_, [u8]>>, ObjectAccessError> {
        Ok(RefMut::filter_map(self.inner.borrow_data_mut()?, ArrayBuffer::bytes_mut).ok())
    }
}

impl From<JsArrayBuffer> for JsObject {
    #[inline]
    fn from(o: JsArrayBuffer) -> Self {
        o.inner.upcast()
    }
}

impl From<JsArrayBuffer> for JsValue {
    #[inline]
    fn from(o: JsArrayBuffer) -> Self {
        o.inner.upcast().into()
    }
}

impl Deref for JsArrayBuffer {
    type Target = JsObject<ArrayBuffer>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsArrayBuffer {
    fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not an ArrayBuffer object")
                .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, JsValue};

    #[test]
    fn array_buffer_access_distinguishes_alias_refusal_from_detachment() {
        let context = &mut Context::builder().build().unwrap();
        let buffer = JsArrayBuffer::from_byte_block(vec![3, 5], context.access()).unwrap();
        let alias: JsObject<ArrayBuffer> = buffer.clone().into();
        let view = alias.borrow_mut().unwrap();

        assert_eq!(buffer.byte_length(), Err(ObjectAccessError::Borrowed));
        assert_eq!(buffer.data().unwrap_err(), ObjectAccessError::Borrowed);
        assert_eq!(buffer.data_mut().unwrap_err(), ObjectAccessError::Borrowed);
        assert_eq!(buffer.to_vec(), Err(ObjectAccessError::Borrowed));
        assert_eq!(
            buffer
                .detach_if_attached(&JsValue::undefined())
                .unwrap_err()
                .as_engine(),
            Some(&crate::error::EngineError::ObjectAccess(
                ObjectAccessError::Borrowed
            ))
        );
        drop(view);

        assert_eq!(buffer.to_vec().unwrap(), Some(vec![3, 5]));
        assert_eq!(buffer.detach(&JsValue::undefined()).unwrap(), vec![3, 5]);
        assert!(buffer.data().unwrap().is_none());
        assert!(buffer.data_mut().unwrap().is_none());
        assert_eq!(buffer.to_vec().unwrap(), None);
    }

    #[test]
    fn array_buffer_to_vec_roundtrip_and_detach() {
        let context = &mut Context::builder().build().unwrap();

        let data = vec![1u8, 2, 3, 4, 5];
        let buffer = JsArrayBuffer::from_byte_block(data, context.access()).unwrap();

        assert_eq!(buffer.to_vec().unwrap(), Some(vec![1u8, 2, 3, 4, 5]));

        buffer.detach(&JsValue::undefined()).unwrap();
        assert_eq!(buffer.to_vec().unwrap(), None);
    }

    #[test]
    fn array_buffer_to_vec_empty() {
        let context = &mut Context::builder().build().unwrap();

        let data = Vec::new();
        let buffer = JsArrayBuffer::from_byte_block(data, context.access()).unwrap();

        assert_eq!(buffer.to_vec().unwrap(), Some(Vec::new()));
    }
}
