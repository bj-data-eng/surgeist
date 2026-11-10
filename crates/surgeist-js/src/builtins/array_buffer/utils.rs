use std::{
    ops::{Range, RangeBounds},
    slice::SliceIndex,
    sync::atomic::Ordering,
};

use crate::{
    Context, JsResult,
    builtins::typed_array::{ClampedU8, Element, TypedArrayElement, TypedArrayKind},
    object::JsObject,
};

use super::{ArrayBuffer, shared_bytes::SharedBytes};

#[derive(Debug, Clone, Copy)]
pub(crate) enum SliceRef<'a> {
    Slice(&'a [u8]),
    AtomicSlice(SharedBytes<'a>),
}

impl SliceRef<'_> {
    /// Gets the byte length of this `SliceRef`.
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Slice(buf) => buf.len(),
            Self::AtomicSlice(buf) => buf.len(),
        }
    }

    /// Gets a subslice of this `SliceRef`.
    pub(crate) fn subslice<I>(&self, index: I) -> SliceRef<'_>
    where
        I: SliceIndex<[u8], Output = [u8]> + RangeBounds<usize>,
    {
        match self {
            Self::Slice(buffer) => SliceRef::Slice(buffer.get(index).expect("index out of bounds")),
            Self::AtomicSlice(buffer) => SliceRef::AtomicSlice(buffer.subslice(index)),
        }
    }

    /// Copies the slice into a vector, loading shared bytes with relaxed ordering.
    #[must_use]
    pub(crate) fn to_vec(self) -> Vec<u8> {
        match self {
            Self::Slice(bytes) => bytes.to_vec(),
            Self::AtomicSlice(bytes) => (0..bytes.len())
                .map(|index| bytes.load_byte(index, Ordering::Relaxed))
                .collect(),
        }
    }

    /// [`GetValueFromBuffer ( arrayBuffer, byteIndex, type, isTypedArray, order [ , isLittleEndian ] )`][spec]
    ///
    /// The start offset is determined by the input buffer instead of a `byteIndex` parameter.
    ///
    /// The caller supplies a checked range containing one aligned element.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getvaluefrombuffer
    pub(crate) fn get_value(&self, kind: TypedArrayKind, order: Ordering) -> TypedArrayElement {
        let buffer = *self;
        match kind {
            TypedArrayKind::Int8 => i8::read(buffer, order).into(),
            TypedArrayKind::Uint8 => u8::read(buffer, order).into(),
            TypedArrayKind::Uint8Clamped => ClampedU8::read(buffer, order).into(),
            TypedArrayKind::Int16 => i16::read(buffer, order).into(),
            TypedArrayKind::Uint16 => u16::read(buffer, order).into(),
            TypedArrayKind::Int32 => i32::read(buffer, order).into(),
            TypedArrayKind::Uint32 => u32::read(buffer, order).into(),
            TypedArrayKind::BigInt64 => i64::read(buffer, order).into(),
            TypedArrayKind::BigUint64 => u64::read(buffer, order).into(),
            #[cfg(feature = "float16")]
            TypedArrayKind::Float16 => {
                crate::builtins::typed_array::Float16::read(buffer, order).into()
            }
            TypedArrayKind::Float32 => f32::read(buffer, order).into(),
            TypedArrayKind::Float64 => f64::read(buffer, order).into(),
        }
    }

    /// [`CloneArrayBuffer ( srcBuffer, srcByteOffset, srcLength )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-clonearraybuffer
    pub(crate) fn clone(&self, context: &mut Context) -> JsResult<JsObject<ArrayBuffer>> {
        // 1. Assert: IsDetachedBuffer(srcBuffer) is false.

        // 2. Let targetBuffer be ? AllocateArrayBuffer(%ArrayBuffer%, srcLength).
        let target_buffer = ArrayBuffer::allocate(
            &context
                .realm()
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()?
                .into(),
            self.len() as u64,
            None,
            context,
        )?;

        // 3. Let srcBlock be srcBuffer.[[ArrayBufferData]].

        // 4. Let targetBlock be targetBuffer.[[ArrayBufferData]].
        {
            let mut target_buffer = target_buffer.borrow_mut()?;
            let target_block = target_buffer
                .data_mut()
                .bytes_mut()
                .expect("ArrayBuffer cannot be detached here");

            // 5. Perform CopyDataBlockBytes(targetBlock, 0, srcBlock, srcByteOffset, srcLength).
            SliceRefMut::Slice(target_block).copy_from(*self);
        }

        // 6. Return targetBuffer.
        Ok(target_buffer)
    }
}

impl<'a> From<&'a [u8]> for SliceRef<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::Slice(value)
    }
}

#[derive(Debug)]
pub(crate) enum SliceRefMut<'a> {
    Slice(&'a mut [u8]),
    AtomicSlice(SharedBytes<'a>),
}

impl SliceRefMut<'_> {
    /// Gets the byte length of this `SliceRefMut`.
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Slice(buf) => buf.len(),
            Self::AtomicSlice(buf) => buf.len(),
        }
    }

    /// Gets a subslice of this `SliceRefMut`.
    #[expect(unused, reason = "could still be useful in the future")]
    pub(crate) fn subslice<I>(&self, index: I) -> SliceRef<'_>
    where
        I: SliceIndex<[u8], Output = [u8]> + RangeBounds<usize>,
    {
        match self {
            Self::Slice(buffer) => SliceRef::Slice(buffer.get(index).expect("index out of bounds")),
            Self::AtomicSlice(buffer) => SliceRef::AtomicSlice(buffer.subslice(index)),
        }
    }

    /// Gets a mutable subslice of this `SliceRefMut`.
    pub(crate) fn subslice_mut<I>(&mut self, index: I) -> SliceRefMut<'_>
    where
        I: SliceIndex<[u8], Output = [u8]> + RangeBounds<usize>,
    {
        match self {
            Self::Slice(buffer) => {
                SliceRefMut::Slice(buffer.get_mut(index).expect("index out of bounds"))
            }
            Self::AtomicSlice(buffer) => SliceRefMut::AtomicSlice(buffer.subslice(index)),
        }
    }

    /// Copies equally sized byte slices in increasing address order.
    pub(crate) fn copy_from(&mut self, source: SliceRef<'_>) {
        assert_eq!(self.len(), source.len());
        match (self, source) {
            (Self::Slice(target), SliceRef::Slice(source)) => target.copy_from_slice(source),
            (Self::Slice(target), SliceRef::AtomicSlice(source)) => {
                for (index, target) in target.iter_mut().enumerate() {
                    *target = source.load_byte(index, Ordering::Relaxed);
                }
            }
            (Self::AtomicSlice(target), SliceRef::Slice(source)) => {
                for (index, source) in source.iter().enumerate() {
                    target.store_byte(index, *source, Ordering::Relaxed);
                }
            }
            (Self::AtomicSlice(target), SliceRef::AtomicSlice(source)) => {
                for index in 0..source.len() {
                    target.store_byte(
                        index,
                        source.load_byte(index, Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                }
            }
        }
    }

    /// Moves bytes within one buffer, preserving source bytes during overlap.
    pub(crate) fn copy_within(&mut self, source: Range<usize>, target: usize) {
        match self {
            Self::Slice(bytes) => bytes.copy_within(source, target),
            Self::AtomicSlice(bytes) => {
                let targets = bytes.subslice(target..target + source.len());
                let sources = bytes.subslice(source.clone());
                let copy_byte = |index| {
                    targets.store_byte(
                        index,
                        sources.load_byte(index, Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                };
                if source.start < target && target < source.end {
                    (0..sources.len()).rev().for_each(copy_byte);
                } else {
                    (0..sources.len()).for_each(copy_byte);
                }
            }
        }
    }

    /// Copies one byte at a time from left to right. An earlier write can
    /// affect a later read when a TypedArray slice aliases its source.
    pub(crate) fn copy_forward_within(&mut self, source: Range<usize>, target: usize) {
        match self {
            Self::Slice(bytes) => {
                let count = source.len();
                assert!(source.start <= source.end && source.end <= bytes.len());
                assert!(target <= bytes.len() - count);
                for index in 0..count {
                    bytes[target + index] = bytes[source.start + index];
                }
            }
            Self::AtomicSlice(bytes) => {
                let target = bytes.subslice(target..target + source.len());
                Self::AtomicSlice(target).copy_from(SliceRef::AtomicSlice(bytes.subslice(source)));
            }
        }
    }

    /// `25.1.2.12 SetValueInBuffer ( arrayBuffer, byteIndex, type, value, isTypedArray, order [ , isLittleEndian ] )`
    ///
    /// The start offset is determined by the input buffer instead of a `byteIndex` parameter.
    ///
    /// The caller supplies a checked range containing one aligned element.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setvalueinbuffer
    pub(crate) fn set_value(&mut self, value: TypedArrayElement, order: Ordering) {
        fn write_elem<T: Element>(buffer: SliceRefMut<'_>, value: T, order: Ordering) {
            T::read_mut(buffer).store(value, order);
        }
        let buffer = match self {
            SliceRefMut::Slice(buf) => SliceRefMut::Slice(buf),
            SliceRefMut::AtomicSlice(buf) => SliceRefMut::AtomicSlice(*buf),
        };
        match value {
            TypedArrayElement::Int8(e) => write_elem(buffer, e, order),
            TypedArrayElement::Uint8(e) => write_elem(buffer, e, order),
            TypedArrayElement::Uint8Clamped(e) => write_elem(buffer, e, order),
            TypedArrayElement::Int16(e) => write_elem(buffer, e, order),
            TypedArrayElement::Uint16(e) => write_elem(buffer, e, order),
            TypedArrayElement::Int32(e) => write_elem(buffer, e, order),
            TypedArrayElement::Uint32(e) => write_elem(buffer, e, order),
            TypedArrayElement::BigInt64(e) => write_elem(buffer, e, order),
            TypedArrayElement::BigUint64(e) => write_elem(buffer, e, order),
            #[cfg(feature = "float16")]
            TypedArrayElement::Float16(e) => write_elem(buffer, e, order),
            TypedArrayElement::Float32(e) => write_elem(buffer, e, order),
            TypedArrayElement::Float64(e) => write_elem(buffer, e, order),
        }
    }
}

impl<'a> From<&'a mut [u8]> for SliceRefMut<'a> {
    fn from(value: &'a mut [u8]) -> Self {
        Self::Slice(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Ordering, SliceRef, SliceRefMut};
    use crate::builtins::array_buffer::shared_bytes::SharedByteBlock;

    fn shared_pattern() -> SharedByteBlock {
        let block = SharedByteBlock::zeroed(32).unwrap();
        for index in 0..32 {
            block
                .bytes(32)
                .store_byte(index, index as u8, Ordering::Relaxed);
        }
        block
    }

    #[test]
    fn copy_shared_between_distinct_offsets() {
        let source = shared_pattern();
        let target = SharedByteBlock::zeroed(32).unwrap();
        SliceRefMut::AtomicSlice(target.bytes(32).subslice(2..22))
            .copy_from(SliceRef::AtomicSlice(source.bytes(32).subslice(1..21)));
        assert_eq!(
            SliceRef::AtomicSlice(target.bytes(32).subslice(2..22)).to_vec(),
            (1..21).collect::<Vec<_>>()
        );
    }

    #[test]
    fn move_shared_backward_during_overlap() {
        let bytes = shared_pattern();
        SliceRefMut::AtomicSlice(bytes.bytes(32)).copy_within(1..21, 2);
        assert_eq!(
            SliceRef::AtomicSlice(bytes.bytes(32).subslice(2..22)).to_vec(),
            (1..21).collect::<Vec<_>>()
        );
    }

    #[test]
    fn copy_ordinary_to_shared_between_distinct_offsets() {
        let source: Vec<_> = (0..32).collect();
        let target = SharedByteBlock::zeroed(32).unwrap();
        SliceRefMut::AtomicSlice(target.bytes(32).subslice(2..22))
            .copy_from(SliceRef::Slice(&source[1..21]));
        assert_eq!(
            SliceRef::AtomicSlice(target.bytes(32).subslice(2..22)).to_vec(),
            (1..21).collect::<Vec<_>>()
        );
    }

    #[test]
    fn copy_shared_to_ordinary_between_distinct_offsets() {
        let source = shared_pattern();
        let mut target = [0; 32];
        SliceRefMut::Slice(&mut target[2..22])
            .copy_from(SliceRef::AtomicSlice(source.bytes(32).subslice(1..21)));
        assert_eq!(&target[2..22], &(1..21).collect::<Vec<_>>());
    }
}
