//! Boa's implementation of ECMAScript's global `ArrayBuffer` and `SharedArrayBuffer` objects
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-arraybuffer-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub(crate) mod shared;
pub(crate) mod shared_bytes;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

pub use shared::SharedArrayBuffer;
use std::sync::atomic::Ordering;

use crate::{
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{JsObject, ObjectEdge, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::StoredValue,
};
use surgeist_js_gc::heap::Trace as HeapTrace;

use self::utils::{SliceRef, SliceRefMut};

use super::{
    Array, BuiltInBuilder, BuiltInConstructor, DataView, IntrinsicObject, typed_array::TypedArray,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum BufferRef<B, S> {
    Buffer(B),
    SharedBuffer(S),
}

impl<B, S> BufferRef<B, S>
where
    B: Deref<Target = ArrayBuffer>,
    S: Deref<Target = SharedArrayBuffer>,
{
    /// Gets the inner data of the buffer.
    pub(crate) fn bytes(&self, ordering: Ordering) -> Option<SliceRef<'_>> {
        match self {
            Self::Buffer(buf) => buf.deref().bytes().map(SliceRef::Slice),
            Self::SharedBuffer(buf) => Some(SliceRef::AtomicSlice(buf.deref().bytes(ordering))),
        }
    }

    /// Gets the inner data of the buffer without accessing the current atomic length.
    ///
    /// Returns `None` if the buffer is detached or if the provided `len` is bigger than
    /// the allocated buffer.
    #[track_caller]
    pub(crate) fn bytes_with_len(&self, len: usize) -> Option<SliceRef<'_>> {
        match self {
            Self::Buffer(buf) => buf.deref().bytes_with_len(len).map(SliceRef::Slice),
            Self::SharedBuffer(buf) => Some(SliceRef::AtomicSlice(buf.deref().bytes_with_len(len))),
        }
    }

    pub(crate) fn is_fixed_len(&self) -> bool {
        match self {
            Self::Buffer(buf) => buf.is_fixed_len(),
            Self::SharedBuffer(buf) => buf.is_fixed_len(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum BufferRefMut<B, S> {
    Buffer(B),
    SharedBuffer(S),
}

impl<B, S> BufferRefMut<B, S>
where
    B: DerefMut<Target = ArrayBuffer>,
    S: DerefMut<Target = SharedArrayBuffer>,
{
    pub(crate) fn bytes(&mut self, ordering: Ordering) -> Option<SliceRefMut<'_>> {
        match self {
            Self::Buffer(buf) => buf.deref_mut().bytes_mut().map(SliceRefMut::Slice),
            Self::SharedBuffer(buf) => {
                Some(SliceRefMut::AtomicSlice(buf.deref_mut().bytes(ordering)))
            }
        }
    }

    /// Gets the mutable inner data of the buffer without accessing the current atomic length.
    ///
    /// Returns `None` if the buffer is detached or if the provided `len` is bigger than
    /// the allocated buffer.
    pub(crate) fn bytes_with_len(&mut self, len: usize) -> Option<SliceRefMut<'_>> {
        match self {
            Self::Buffer(buf) => buf
                .deref_mut()
                .bytes_with_len_mut(len)
                .map(SliceRefMut::Slice),
            Self::SharedBuffer(buf) => Some(SliceRefMut::AtomicSlice(
                buf.deref_mut().bytes_with_len(len),
            )),
        }
    }
}

/// A `JsObject` containing a bytes buffer as its inner data.
#[derive(Debug, Clone)]
pub(crate) enum BufferObject {
    Buffer(JsObject<ArrayBuffer>),
    SharedBuffer(JsObject<SharedArrayBuffer>),
}

/// A buffer reference stored by a view in the object graph.
#[derive(Debug, Clone, HeapTrace)]
pub(crate) struct BufferEdge(ObjectEdge);

impl BufferEdge {
    pub(crate) fn to_buffer(&self) -> BufferObject {
        JsObject::from_inner(self.0.clone())
            .into_buffer_object()
            .expect("buffer edge retains its ArrayBuffer or SharedArrayBuffer brand")
    }
}

impl From<BufferObject> for JsObject {
    fn from(value: BufferObject) -> Self {
        match value {
            BufferObject::Buffer(buf) => buf.upcast(),
            BufferObject::SharedBuffer(buf) => buf.upcast(),
        }
    }
}

impl From<BufferObject> for JsValue {
    fn from(value: BufferObject) -> Self {
        JsValue::from(JsObject::from(value))
    }
}

impl BufferObject {
    pub(crate) fn edge(&self) -> BufferEdge {
        BufferEdge(match self {
            Self::Buffer(buffer) => buffer.edge(),
            Self::SharedBuffer(buffer) => buffer.edge(),
        })
    }

    /// Gets the buffer data of the object.
    #[inline]
    pub(crate) fn as_buffer(
        &self,
    ) -> Result<
        BufferRef<Ref<'_, ArrayBuffer>, Ref<'_, SharedArrayBuffer>>,
        crate::error::ObjectAccessError,
    > {
        Ok(match self {
            Self::Buffer(buf) => BufferRef::Buffer(buf.borrow_data()?),
            Self::SharedBuffer(buf) => BufferRef::SharedBuffer(buf.borrow_data()?),
        })
    }

    /// Gets the mutable buffer data of the object
    #[inline]
    #[track_caller]
    pub(crate) fn as_buffer_mut(
        &self,
    ) -> Result<
        BufferRefMut<RefMut<'_, ArrayBuffer>, RefMut<'_, SharedArrayBuffer>>,
        crate::error::ObjectAccessError,
    > {
        Ok(match self {
            Self::Buffer(buf) => BufferRefMut::Buffer(buf.borrow_data_mut()?),
            Self::SharedBuffer(buf) => BufferRefMut::SharedBuffer(buf.borrow_data_mut()?),
        })
    }

    /// Returns `true` if the buffer objects point to the same buffer.
    #[inline]
    #[track_caller]
    pub(crate) fn equals(lhs: &Self, rhs: &Self) -> Result<bool, crate::error::ObjectAccessError> {
        Ok(match (lhs, rhs) {
            (BufferObject::Buffer(lhs), BufferObject::Buffer(rhs)) => JsObject::equals(lhs, rhs),
            (BufferObject::SharedBuffer(lhs), BufferObject::SharedBuffer(rhs)) => {
                if JsObject::equals(lhs, rhs) {
                    return Ok(true);
                }

                let lhs = lhs.borrow()?;
                let rhs = rhs.borrow()?;

                lhs.data().shares_backing_with(rhs.data())
            }
            _ => false,
        })
    }
}

/// The internal representation of an `ArrayBuffer` object.
#[derive(Debug, Clone, HeapTrace, JsData)]
pub struct ArrayBuffer {
    /// The `[[ArrayBufferData]]` internal slot.
    #[heap_trace(skip)]
    data: Option<Vec<u8>>,

    /// The `[[ArrayBufferMaxByteLength]]` internal slot.
    max_byte_len: Option<u64>,

    /// The `[[ArrayBufferDetachKey]]` internal slot.
    detach_key: StoredValue,
}

impl ArrayBuffer {
    pub(crate) fn from_data(data: Vec<u8>, detach_key: &JsValue) -> Self {
        Self {
            data: Some(data),
            max_byte_len: None,
            detach_key: detach_key.clone().into_stored(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.data.as_ref().map_or(0, Vec::len)
    }

    pub(crate) fn bytes(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    pub(crate) fn bytes_mut(&mut self) -> Option<&mut [u8]> {
        self.data.as_deref_mut()
    }

    /// Sets the maximum byte length of the buffer, returning the previous value if present.
    pub(crate) fn set_max_byte_length(&mut self, max_byte_len: u64) -> Option<u64> {
        self.max_byte_len.replace(max_byte_len)
    }

    /// Gets the inner bytes of the buffer without accessing the current atomic length.
    #[track_caller]
    pub(crate) fn bytes_with_len(&self, len: usize) -> Option<&[u8]> {
        if let Some(s) = self.data.as_deref() {
            Some(&s[..len])
        } else {
            None
        }
    }

    /// Gets the mutable inner bytes of the buffer without accessing the current atomic length.
    #[track_caller]
    pub(crate) fn bytes_with_len_mut(&mut self, len: usize) -> Option<&mut [u8]> {
        if let Some(s) = self.data.as_deref_mut() {
            Some(&mut s[..len])
        } else {
            None
        }
    }

    /// Gets the underlying vector for this buffer.
    #[must_use]
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    /// Resizes the buffer to the new size, clamped to the maximum byte length if present.
    pub fn resize(&mut self, new_byte_length: u64) -> JsResult<()> {
        let bytes = self.resizable_bytes(new_byte_length)?;
        resize_byte_block(bytes, new_byte_length)
    }

    fn resizable_limit(&self) -> JsResult<u64> {
        self.max_byte_len.ok_or_else(|| {
            JsNativeError::typ()
                .with_message("ArrayBuffer.resize: cannot resize a fixed-length buffer")
                .into()
        })
    }

    fn resizable_bytes(&mut self, new_byte_length: u64) -> JsResult<&mut Vec<u8>> {
        let max_byte_len = self.resizable_limit()?;
        let Some(buf) = self.data.as_mut() else {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer.resize: cannot resize a detached buffer")
                .into());
        };

        if new_byte_length > max_byte_len {
            return Err(JsNativeError::range()
                .with_message(
                    "ArrayBuffer.resize: new byte length exceeds buffer's maximum byte length",
                )
                .into());
        }

        Ok(buf)
    }

    /// Detaches the inner data of this `ArrayBuffer`, returning the original buffer if still
    /// present.
    ///
    /// # Errors
    ///
    /// Throws an error if the provided detach key is invalid.
    pub fn detach(&mut self, key: &JsValue) -> JsResult<Option<Vec<u8>>> {
        if !JsValue::same_value(&self.detach_key.to_value(), key) {
            return Err(JsNativeError::typ()
                .with_message("Cannot detach array buffer with different key")
                .into());
        }

        Ok(self.data.take())
    }

    /// `IsDetachedBuffer ( arrayBuffer )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdetachedbuffer
    pub(crate) const fn is_detached(&self) -> bool {
        // 1. If arrayBuffer.[[ArrayBufferData]] is null, return true.
        // 2. Return false.
        self.data.is_none()
    }

    pub(crate) fn is_fixed_len(&self) -> bool {
        self.max_byte_len.is_none()
    }
}

impl IntrinsicObject for ArrayBuffer {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build()?;

        let get_byte_length = BuiltInBuilder::callable(realm, Self::get_byte_length)
            .name(js_string!("get byteLength"))
            .build()?;

        let get_resizable = BuiltInBuilder::callable(realm, Self::get_resizable)
            .name(js_string!("get resizable"))
            .build()?;

        let get_max_byte_length = BuiltInBuilder::callable(realm, Self::get_max_byte_length)
            .name(js_string!("get maxByteLength"))
            .build()?;

        #[cfg(feature = "experimental")]
        let get_detached = BuiltInBuilder::callable(realm, Self::get_detached)
            .name(js_string!("get detached"))
            .build()?;

        let builder = BuiltInBuilder::from_standard_constructor::<Self>(realm)?
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )?
            .static_method(Self::is_view, js_string!("isView"), 1)?
            .accessor(
                js_string!("byteLength"),
                Some(get_byte_length),
                None,
                flag_attributes,
            )?
            .accessor(
                js_string!("resizable"),
                Some(get_resizable),
                None,
                flag_attributes,
            )?
            .accessor(
                js_string!("maxByteLength"),
                Some(get_max_byte_length),
                None,
                flag_attributes,
            )?
            .method(Self::js_resize, js_string!("resize"), 1)?
            .method(Self::slice, js_string!("slice"), 2)?
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )?;

        #[cfg(feature = "experimental")]
        let builder = builder
            .accessor(
                js_string!("detached"),
                Some(get_detached),
                None,
                flag_attributes,
            )?
            .method(Self::transfer::<false>, js_string!("transfer"), 0)?
            .method(
                Self::transfer::<true>,
                js_string!("transferToFixedLength"),
                0,
            )?;

        builder.build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> crate::JsResult<JsObject> {
        Ok(Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()?)
    }
}

impl BuiltInObject for ArrayBuffer {
    const NAME: JsString = StaticJsStrings::ARRAY_BUFFER;
}

impl BuiltInConstructor for ArrayBuffer {
    const PROTOTYPE_STORAGE_SLOTS: usize = 13;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 3;
    const CONSTRUCTOR_ARGUMENTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::array_buffer;

    /// `ArrayBuffer ( length )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-length
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer.constructor called with undefined new target")
                .into());
        }

        // 2. Let byteLength be ? ToIndex(length).
        let byte_len = args.get_or_undefined(0).to_index(context.access())?;

        // 3. Let requestedMaxByteLength be ? GetArrayBufferMaxByteLengthOption(options).
        let max_byte_len = get_max_byte_len(args.get_or_undefined(1), context)?;

        // 4. Return ? AllocateArrayBuffer(NewTarget, byteLength, requestedMaxByteLength).
        Ok(Self::allocate(new_target, byte_len, max_byte_len, context)?
            .upcast()
            .into())
    }
}

impl ArrayBuffer {
    /// `ArrayBuffer.isView ( arg )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.isview
    #[allow(clippy::unnecessary_wraps)]
    fn is_view(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. If Type(arg) is not Object, return false.
        // 2. If arg has a [[ViewedArrayBuffer]] internal slot, return true.
        // 3. Return false.
        Ok(args
            .get_or_undefined(0)
            .as_object()
            .is_some_and(|obj| obj.is::<TypedArray>() || obj.is::<DataView>())
            .into())
    }

    /// `get ArrayBuffer [ @@species ]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer-@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `get ArrayBuffer.prototype.byteLength`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer.prototype.bytelength
    pub(crate) fn get_byte_length(
        this: &JsValue,
        _args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let object = this.as_object();
        let buf = object
            .as_ref()
            .map(JsObject::downcast_ref::<Self>)
            .transpose()?
            .flatten()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("get ArrayBuffer.prototype.byteLength called with invalid `this`")
            })?;

        // 4. If IsDetachedBuffer(O) is true, return +0𝔽.
        // 5. Let length be O.[[ArrayBufferByteLength]].
        // 6. Return 𝔽(length).
        Ok(buf.len().into())
    }

    /// [`get ArrayBuffer.prototype.maxByteLength`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer.prototype.maxbytelength
    pub(crate) fn get_max_byte_length(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let object = this.as_object();
        let buf = object
            .as_ref()
            .map(JsObject::downcast_ref::<Self>)
            .transpose()?
            .flatten()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "get ArrayBuffer.prototype.maxByteLength called with invalid `this`",
                )
            })?;

        // 4. If IsDetachedBuffer(O) is true, return +0𝔽.
        let Some(data) = buf.bytes() else {
            return Ok(JsValue::from(0));
        };

        // 5. If IsFixedLengthArrayBuffer(O) is true, then
        //     a. Let length be O.[[ArrayBufferByteLength]].
        // 6. Else,
        //     a. Let length be O.[[ArrayBufferMaxByteLength]].
        // 7. Return 𝔽(length).
        Ok(buf.max_byte_len.unwrap_or(data.len() as u64).into())
    }

    /// [`get ArrayBuffer.prototype.resizable`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer.prototype.resizable
    pub(crate) fn get_resizable(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let object = this.as_object();
        let buf = object
            .as_ref()
            .map(JsObject::downcast_ref::<Self>)
            .transpose()?
            .flatten()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("get ArrayBuffer.prototype.resizable called with invalid `this`")
            })?;

        // 4. If IsFixedLengthArrayBuffer(O) is false, return true; otherwise return false.
        Ok(JsValue::from(!buf.is_fixed_len()))
    }

    /// [`get ArrayBuffer.prototype.detached`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-arraybuffer-transfer/#sec-get-arraybuffer.prototype.detached
    #[cfg(feature = "experimental")]
    fn get_detached(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let object = this.as_object();
        let buf = object
            .as_ref()
            .map(JsObject::downcast_ref::<Self>)
            .transpose()?
            .flatten()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("get ArrayBuffer.prototype.detached called with invalid `this`")
            })?;

        // 4. Return IsDetachedBuffer(O).
        Ok(buf.is_detached().into())
    }

    /// [`ArrayBuffer.prototype.resize ( newLength )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.prototype.resize
    pub(crate) fn js_resize(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let buf = this
            .as_object()
            .and_then(|o| o.clone().downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("ArrayBuffer.prototype.resize called with invalid `this`")
            })?;

        buf.borrow()?.data().resizable_limit()?;

        // 4. Let newByteLength be ? ToIndex(newLength).
        let new_byte_length = args.get_or_undefined(0).to_index(context.access())?;

        // 5–6. Validate the source before asking the host to admit an allocation.
        let capacity = buf
            .borrow_mut()?
            .data_mut()
            .resizable_bytes(new_byte_length)?
            .capacity();
        let limit = (new_byte_length > capacity as u64)
            .then(|| context.host_hooks().max_buffer_size(context.access()));

        // TODO: 7. Let hostHandled be ? HostResizeArrayBuffer(O, newByteLength).
        // 8. If hostHandled is handled, return undefined.
        // Used in engines to handle Wasm buffers in a special way, but we don't
        // have a Wasm interpreter in place yet.

        // 9. Let oldBlock be O.[[ArrayBufferData]].
        // 10. Let newBlock be ? CreateByteDataBlock(newByteLength).
        // 11. Let copyLength be min(newByteLength, O.[[ArrayBufferByteLength]]).
        // 12. Perform CopyDataBlockBytes(newBlock, 0, oldBlock, 0, copyLength).
        // 13. NOTE: Neither creation of the new Data Block nor copying from the old Data Block are observable.
        //     Implementations may implement this method as in-place growth or shrinkage.
        // 14. Set O.[[ArrayBufferData]] to newBlock.
        // 15. Set O.[[ArrayBufferByteLength]] to newByteLength.
        // The host hook can reenter. Validate the current source before any edit.
        let mut buf = buf.borrow_mut()?;
        let bytes = buf.data_mut().resizable_bytes(new_byte_length)?;
        if let Some(limit) = limit
            && new_byte_length > bytes.capacity() as u64
        {
            check_buffer_allocation_limit(new_byte_length, limit)?;
        }
        resize_byte_block(bytes, new_byte_length)?;

        // 16. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `ArrayBuffer.prototype.slice ( start, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.prototype.slice
    fn slice(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let buf = this
            .as_object()
            .and_then(|o| o.clone().downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("ArrayBuffer.slice called with invalid `this` value")
            })?;

        let len = {
            let buf = buf.borrow()?;
            // 4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
            if buf.data().is_detached() {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer.slice called with detached buffer")
                    .into());
            }
            // 5. Let len be O.[[ArrayBufferByteLength]].
            buf.data().len() as u64
        };

        // 6. Let relativeStart be ? ToIntegerOrInfinity(start).
        // 7. If relativeStart = -∞, let first be 0.
        // 8. Else if relativeStart < 0, let first be max(len + relativeStart, 0).
        // 9. Else, let first be min(relativeStart, len).
        let first = Array::get_relative_start(context, args.get_or_undefined(0), len)?;

        // 10. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        // 11. If relativeEnd = -∞, let final be 0.
        // 12. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
        // 13. Else, let final be min(relativeEnd, len).
        let final_ = Array::get_relative_end(context, args.get_or_undefined(1), len)?;

        // 14. Let newLen be max(final - first, 0).
        let new_len = final_.saturating_sub(first);

        // 15. Let ctor be ? SpeciesConstructor(O, %ArrayBuffer%).
        let ctor = buf
            .clone()
            .upcast()
            .species_constructor(StandardConstructors::array_buffer, context)?;

        // 16. Let new be ? Construct(ctor, « 𝔽(newLen) »).
        let new = ctor.construct(&[new_len.into()], Some(&ctor), context.access())?;

        // 17. Perform ? RequireInternalSlot(new, [[ArrayBufferData]]).
        // 18. If IsSharedArrayBuffer(new) is true, throw a TypeError exception.
        let Ok(new) = new.downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer constructor returned invalid object")
                .into());
        };

        // 20. If SameValue(new, O) is true, throw a TypeError exception.
        if JsObject::equals(&buf, &new) {
            return Err(JsNativeError::typ()
                .with_message("new ArrayBuffer is the same as this ArrayBuffer")
                .into());
        }

        {
            // 19. If IsDetachedBuffer(new) is true, throw a TypeError exception.
            // 25. Let toBuf be new.[[ArrayBufferData]].
            let mut new = new.borrow_mut()?;
            let Some(to_buf) = new.data_mut().bytes_mut() else {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer constructor returned detached ArrayBuffer")
                    .into());
            };

            // 21. If new.[[ArrayBufferByteLength]] < newLen, throw a TypeError exception.
            if (to_buf.len() as u64) < new_len {
                return Err(JsNativeError::typ()
                    .with_message("new ArrayBuffer length too small")
                    .into());
            }

            // 22. NOTE: Side-effects of the above steps may have detached O.
            // 23. If IsDetachedBuffer(O) is true, throw a TypeError exception.
            // 24. Let fromBuf be O.[[ArrayBufferData]].
            let buf = buf.borrow()?;
            let Some(from_buf) = buf.data().bytes() else {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer detached while ArrayBuffer.slice was running")
                    .into());
            };

            // 26. Perform CopyDataBlockBytes(toBuf, 0, fromBuf, first, newLen).
            let first = first as usize;
            let new_len = new_len as usize;
            to_buf[..new_len].copy_from_slice(&from_buf[first..first + new_len]);
        }

        // 27. Return new.
        Ok(new.upcast().into())
    }

    /// [`ArrayBuffer.prototype.transfer ( [ newLength ] )`][transfer] and
    /// [`ArrayBuffer.prototype.transferToFixedLength ( [ newLength ] )`][transferFL]
    ///
    /// [transfer]: https://tc39.es/proposal-arraybuffer-transfer/#sec-arraybuffer.prototype.transfer
    /// [transferFL]: https://tc39.es/proposal-arraybuffer-transfer/#sec-arraybuffer.prototype.transfertofixedlength
    #[cfg(feature = "experimental")]
    fn transfer<const TO_FIXED_LENGTH: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Return ? ArrayBufferCopyAndDetach(O, newLength, preserve-resizability).

        // Abstract operation `ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )`
        // https://tc39.es/proposal-arraybuffer-transfer/#sec-arraybuffercopyanddetach

        let new_length = args.get_or_undefined(0);

        // 1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
        // 2. If IsSharedArrayBuffer(arrayBuffer) is true, throw a TypeError exception.
        let buf = this
            .as_object()
            .and_then(|o| o.clone().downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ().with_message(if TO_FIXED_LENGTH {
                    "ArrayBuffer.prototype.transferToFixedLength called with invalid `this`"
                } else {
                    "ArrayBuffer.prototype.transfer called with invalid `this`"
                })
            })?;

        // 3. If newLength is undefined, then
        let new_len = if new_length.is_undefined() {
            // a. Let newByteLength be arrayBuffer.[[ArrayBufferByteLength]].
            buf.borrow()?.data().len() as u64
        } else {
            // 4. Else,
            //     a. Let newByteLength be ? ToIndex(newLength).
            new_length.to_index(context.access())?
        };

        let needs_allocation = {
            let mut source = buf.borrow_mut()?;
            let (bytes, new_max_len) = source
                .data_mut()
                .transfer_source(new_len, TO_FIXED_LENGTH)?;
            transfer_allocates(bytes.capacity(), new_len, new_max_len)
        };
        let limit =
            needs_allocation.then(|| context.host_hooks().max_buffer_size(context.access()));

        let (bytes, new_max_len) = {
            let mut source = buf.borrow_mut()?;
            let source = source.data_mut();
            let (bytes, new_max_len) = source.transfer_source(new_len, TO_FIXED_LENGTH)?;
            if let Some(limit) = limit
                && transfer_allocates(bytes.capacity(), new_len, new_max_len)
            {
                check_buffer_allocation_limit(new_len, limit)?;
            }

            // 9–15. Prepare the destination bytes before detaching the source.
            let destination = if new_max_len.is_none() && new_len < bytes.capacity() as u64 {
                let mut destination = Vec::new();
                resize_byte_block(&mut destination, new_len)?;
                let copied = destination.len().min(bytes.len());
                destination[..copied].copy_from_slice(&bytes[..copied]);
                destination
            } else {
                resize_byte_block(bytes, new_len)?;
                std::mem::take(bytes)
            };
            source.data = None;
            (destination, new_max_len)
        };

        let prototype = context
            .intrinsics()
            .constructors()
            .array_buffer()
            .prototype()?;

        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ArrayBuffer {
                data: Some(bytes),
                max_byte_len: new_max_len,
                detach_key: StoredValue::Undefined,
            },
        )?
        .into())
    }

    #[cfg(feature = "experimental")]
    fn transfer_source(
        &mut self,
        new_len: u64,
        to_fixed_length: bool,
    ) -> JsResult<(&mut Vec<u8>, Option<u64>)> {
        // 5–8. Validate detachment, result resizability and the detach key.
        let Some(bytes) = self.data.as_mut() else {
            return Err(JsNativeError::typ()
                .with_message("cannot transfer a detached buffer")
                .into());
        };

        let new_max_len = self.max_byte_len.filter(|_| !to_fixed_length);
        if !matches!(self.detach_key, StoredValue::Undefined) {
            return Err(JsNativeError::typ()
                .with_message("cannot transfer a buffer with a detach key")
                .into());
        }
        if new_max_len.is_some_and(|max_len| new_len > max_len) {
            return Err(JsNativeError::range()
                .with_message("`length` cannot be bigger than `maxByteLength`")
                .into());
        }
        Ok((bytes, new_max_len))
    }

    /// `AllocateArrayBuffer ( constructor, byteLength )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-allocatearraybuffer
    pub(crate) fn allocate(
        constructor: &JsValue,
        byte_len: u64,
        max_byte_len: Option<u64>,
        context: &mut Context,
    ) -> JsResult<JsObject<ArrayBuffer>> {
        // 1. Let slots be « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] ».
        // 2. If maxByteLength is present and maxByteLength is not empty, let allocatingResizableBuffer be true; otherwise let allocatingResizableBuffer be false.
        // 3. If allocatingResizableBuffer is true, then
        //     a. If byteLength > maxByteLength, throw a RangeError exception.
        //     b. Append [[ArrayBufferMaxByteLength]] to slots.
        if let Some(max_byte_len) = max_byte_len
            && byte_len > max_byte_len
        {
            return Err(JsNativeError::range()
                .with_message("`length` cannot be bigger than `maxByteLength`")
                .into());
        }

        // 4. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", slots).
        let prototype = get_prototype_from_constructor(
            constructor,
            StandardConstructors::array_buffer,
            context,
        )?;

        // 5. Let block be ? CreateByteDataBlock(byteLength).
        // Preemptively allocate for `max_byte_len` if possible.
        //     a. If it is not possible to create a Data Block block consisting of maxByteLength bytes, throw a RangeError exception.
        //     b. NOTE: Resizable ArrayBuffers are designed to be implementable with in-place growth. Implementations may
        //        throw if, for example, virtual memory cannot be reserved up front.
        let block = create_byte_data_block(byte_len, max_byte_len, context)?;

        let obj = JsObject::new(
            context.root_shape(),
            prototype,
            Self {
                // 6. Set obj.[[ArrayBufferData]] to block.
                // 7. Set obj.[[ArrayBufferByteLength]] to byteLength.
                data: Some(block),
                // 8. If allocatingResizableBuffer is true, then
                //    c. Set obj.[[ArrayBufferMaxByteLength]] to maxByteLength.
                max_byte_len,
                detach_key: StoredValue::Undefined,
            },
        )?;

        // 9. Return obj.
        Ok(obj)
    }
}

/// Abstract operation [`GetArrayBufferMaxByteLengthOption ( options )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-getarraybuffermaxbytelengthoption
fn get_max_byte_len(options: &JsValue, context: &mut Context) -> JsResult<Option<u64>> {
    // 1. If options is not an Object, return empty.
    let Some(options) = options.as_object() else {
        return Ok(None);
    };

    // 2. Let maxByteLength be ? Get(options, "maxByteLength").
    let max_byte_len = options.get(js_string!("maxByteLength"), context.access())?;

    // 3. If maxByteLength is undefined, return empty.
    if max_byte_len.is_undefined() {
        return Ok(None);
    }

    // 4. Return ? ToIndex(maxByteLength).
    max_byte_len.to_index(context.access()).map(Some)
}

/// `CreateByteDataBlock ( size )` abstract operation.
///
/// The abstract operation `CreateByteDataBlock` takes argument `size` (a non-negative
/// integer). For more information, check the [spec][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-createbytedatablock
pub(crate) fn create_byte_data_block(
    size: u64,
    max_buffer_size: Option<u64>,
    context: &mut Context,
) -> JsResult<Vec<u8>> {
    let alloc_size = max_buffer_size.unwrap_or(size);

    assert!(size <= alloc_size);

    check_buffer_allocation_limit(
        alloc_size,
        context.host_hooks().max_buffer_size(context.access()),
    )?;

    // 1. Let db be a new Data Block value consisting of size bytes. If it is impossible to
    //    create such a Data Block, throw a RangeError exception.
    let alloc_size = alloc_size.try_into().map_err(|e| {
        JsNativeError::range().with_message(format!("couldn't allocate the data block: {e}"))
    })?;

    let mut data_block = Vec::new();
    data_block.try_reserve_exact(alloc_size).map_err(|error| {
        JsNativeError::range().with_message(format!("couldn't allocate the data block: {error}"))
    })?;

    // since size <= alloc_size, then `size` must also fit inside a `usize`.
    let size = size as usize;

    // 2. Set all of the bytes of db to 0.
    data_block.resize(size, 0);

    // 3. Return db.
    Ok(data_block)
}

fn check_buffer_allocation_limit(byte_length: u64, limit: u64) -> JsResult<()> {
    if byte_length > limit {
        return Err(JsNativeError::range()
            .with_message("cannot allocate a buffer that exceeds the maximum buffer size")
            .into());
    }
    Ok(())
}

#[cfg(feature = "experimental")]
fn transfer_allocates(capacity: usize, byte_length: u64, maximum: Option<u64>) -> bool {
    let capacity = capacity as u64;
    byte_length > capacity || (maximum.is_none() && byte_length != 0 && byte_length < capacity)
}

fn resize_byte_block(bytes: &mut Vec<u8>, byte_length: u64) -> JsResult<()> {
    let byte_length: usize = byte_length.try_into().map_err(|error| {
        JsNativeError::range().with_message(format!("couldn't resize the data block: {error}"))
    })?;
    if byte_length > bytes.capacity() {
        bytes
            .try_reserve_exact(byte_length - bytes.len())
            .map_err(|error| {
                JsNativeError::range()
                    .with_message(format!("couldn't resize the data block: {error}"))
            })?;
    }
    bytes.resize(byte_length, 0);
    Ok(())
}
