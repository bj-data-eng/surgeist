use std::{ops::Range, slice::SliceIndex, sync::atomic::Ordering};

use portable_atomic::AtomicU8;

use crate::{
    Context, JsResult,
    builtins::typed_array::{ClampedU8, Element, TypedArrayElement, TypedArrayKind},
    object::JsObject,
};

use super::ArrayBuffer;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SliceRef<'a> {
    Slice(&'a [u8]),
    AtomicSlice(&'a [AtomicU8]),
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
        I: SliceIndex<[u8], Output = [u8]> + SliceIndex<[AtomicU8], Output = [AtomicU8]>,
    {
        match self {
            Self::Slice(buffer) => SliceRef::Slice(buffer.get(index).expect("index out of bounds")),
            Self::AtomicSlice(buffer) => {
                SliceRef::AtomicSlice(buffer.get(index).expect("index out of bounds"))
            }
        }
    }

    /// Copies the slice into a vector, loading shared bytes with relaxed ordering.
    #[must_use]
    pub(crate) fn to_vec(self) -> Vec<u8> {
        match self {
            Self::Slice(bytes) => bytes.to_vec(),
            Self::AtomicSlice(bytes) => bytes
                .iter()
                .map(|byte| byte.load(Ordering::Relaxed))
                .collect(),
        }
    }

    /// [`GetValueFromBuffer ( arrayBuffer, byteIndex, type, isTypedArray, order [ , isLittleEndian ] )`][spec]
    ///
    /// The start offset is determined by the input buffer instead of a `byteIndex` parameter.
    ///
    /// # Safety
    ///
    /// - There must be enough bytes in `buffer` to read an element from an array with type `TypedArrayKind`.
    /// - Shared storage must be aligned for the element’s atomic type.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getvaluefrombuffer
    pub(crate) unsafe fn get_value(
        &self,
        kind: TypedArrayKind,
        order: Ordering,
    ) -> TypedArrayElement {
        unsafe fn read_elem<T: Element>(buffer: SliceRef<'_>, order: Ordering) -> T {
            // <https://tc39.es/ecma262/#sec-getvaluefrombuffer>

            // 1. Assert: IsDetachedBuffer(arrayBuffer) is false.
            // 2. Assert: There are sufficient bytes in arrayBuffer starting at byteIndex to represent a value of type.
            debug_assert!(buffer.len() >= size_of::<T>());

            // 3. Let block be arrayBuffer.[[ArrayBufferData]].
            // 4. Let elementSize be the Element Size value specified in Table 70 for Element Type type.
            // 5. If IsSharedArrayBuffer(arrayBuffer) is true, then
            //     a. Let execution be the [[CandidateExecution]] field of the surrounding agent's Agent Record.
            //     b. Let eventsRecord be the Agent Events Record of execution.[[EventsRecords]] whose [[AgentSignifier]] is AgentSignifier().
            //     c. If isTypedArray is true and IsNoTearConfiguration(type, order) is true, let noTear be true; otherwise let noTear be false.
            //     d. Let rawValue be a List of length elementSize whose elements are nondeterministically chosen byte values.
            //     e. NOTE: In implementations, rawValue is the result of a non-atomic or atomic read instruction on the underlying hardware. The nondeterminism is a semantic prescription of the memory model to describe observable behaviour of hardware with weak consistency.
            //     f. Let readEvent be ReadSharedMemory { [[Order]]: order, [[NoTear]]: noTear, [[Block]]: block, [[ByteIndex]]: byteIndex, [[ElementSize]]: elementSize }.
            //     g. Append readEvent to eventsRecord.[[EventList]].
            //     h. Append Chosen Value Record { [[Event]]: readEvent, [[ChosenValue]]: rawValue } to execution.[[ChosenValues]].
            // 6. Else,
            //     a. Let rawValue be a List whose elements are bytes from block at indices in the interval from byteIndex (inclusive) to byteIndex + elementSize (exclusive).
            // 7. Assert: The number of elements in rawValue is elementSize.
            // 8. If isLittleEndian is not present, set isLittleEndian to the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
            // 9. Return RawBytesToNumeric(type, rawValue, isLittleEndian).

            // SAFETY: The invariants of this operation are ensured by the caller.
            unsafe { T::read(buffer).load(order) }
        }

        let buffer = *self;

        // SAFETY: The invariants of this operation are ensured by the caller.
        unsafe {
            match kind {
                TypedArrayKind::Int8 => read_elem::<i8>(buffer, order).into(),
                TypedArrayKind::Uint8 => read_elem::<u8>(buffer, order).into(),
                TypedArrayKind::Uint8Clamped => read_elem::<ClampedU8>(buffer, order).into(),
                TypedArrayKind::Int16 => read_elem::<i16>(buffer, order).into(),
                TypedArrayKind::Uint16 => read_elem::<u16>(buffer, order).into(),
                TypedArrayKind::Int32 => read_elem::<i32>(buffer, order).into(),
                TypedArrayKind::Uint32 => read_elem::<u32>(buffer, order).into(),
                TypedArrayKind::BigInt64 => read_elem::<i64>(buffer, order).into(),
                TypedArrayKind::BigUint64 => read_elem::<u64>(buffer, order).into(),
                #[cfg(feature = "float16")]
                TypedArrayKind::Float16 => {
                    read_elem::<crate::builtins::typed_array::Float16>(buffer, order).into()
                }
                TypedArrayKind::Float32 => read_elem::<f32>(buffer, order).into(),
                TypedArrayKind::Float64 => read_elem::<f64>(buffer, order).into(),
            }
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
                .constructor()
                .into(),
            self.len() as u64,
            None,
            context,
        )?;

        // 3. Let srcBlock be srcBuffer.[[ArrayBufferData]].

        // 4. Let targetBlock be targetBuffer.[[ArrayBufferData]].
        {
            let mut target_buffer = target_buffer.borrow_mut();
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

impl<'a> From<&'a [AtomicU8]> for SliceRef<'a> {
    fn from(value: &'a [AtomicU8]) -> Self {
        Self::AtomicSlice(value)
    }
}

#[derive(Debug)]
pub(crate) enum SliceRefMut<'a> {
    Slice(&'a mut [u8]),
    AtomicSlice(&'a [AtomicU8]),
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
        I: SliceIndex<[u8], Output = [u8]> + SliceIndex<[AtomicU8], Output = [AtomicU8]>,
    {
        match self {
            Self::Slice(buffer) => SliceRef::Slice(buffer.get(index).expect("index out of bounds")),
            Self::AtomicSlice(buffer) => {
                SliceRef::AtomicSlice(buffer.get(index).expect("index out of bounds"))
            }
        }
    }

    /// Gets a mutable subslice of this `SliceRefMut`.
    pub(crate) fn subslice_mut<I>(&mut self, index: I) -> SliceRefMut<'_>
    where
        I: SliceIndex<[u8], Output = [u8]> + SliceIndex<[AtomicU8], Output = [AtomicU8]>,
    {
        match self {
            Self::Slice(buffer) => {
                SliceRefMut::Slice(buffer.get_mut(index).expect("index out of bounds"))
            }
            Self::AtomicSlice(buffer) => {
                SliceRefMut::AtomicSlice(buffer.get(index).expect("index out of bounds"))
            }
        }
    }

    /// Copies equally sized byte slices in increasing address order.
    pub(crate) fn copy_from(&mut self, source: SliceRef<'_>) {
        assert_eq!(self.len(), source.len());
        match (self, source) {
            (Self::Slice(target), SliceRef::Slice(source)) => target.copy_from_slice(source),
            (Self::Slice(target), SliceRef::AtomicSlice(source)) => {
                for (target, source) in target.iter_mut().zip(source) {
                    *target = source.load(Ordering::Relaxed);
                }
            }
            (Self::AtomicSlice(target), SliceRef::Slice(source)) => {
                for (target, source) in target.iter().zip(source) {
                    target.store(*source, Ordering::Relaxed);
                }
            }
            (Self::AtomicSlice(target), SliceRef::AtomicSlice(source)) => {
                for (target, source) in target.iter().zip(source) {
                    target.store(source.load(Ordering::Relaxed), Ordering::Relaxed);
                }
            }
        }
    }

    /// Moves bytes within one buffer, preserving source bytes during overlap.
    pub(crate) fn copy_within(&mut self, source: Range<usize>, target: usize) {
        match self {
            Self::Slice(bytes) => bytes.copy_within(source, target),
            Self::AtomicSlice(bytes) => {
                let targets = &bytes[target..target + source.len()];
                let pairs = bytes[source.clone()].iter().zip(targets);
                let copy_byte = |(source, target): (&AtomicU8, &AtomicU8)| {
                    target.store(source.load(Ordering::Relaxed), Ordering::Relaxed);
                };
                if source.start < target && target < source.end {
                    pairs.rev().for_each(copy_byte);
                } else {
                    pairs.for_each(copy_byte);
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
                let target = &bytes[target..target + source.len()];
                Self::AtomicSlice(target).copy_from(SliceRef::AtomicSlice(&bytes[source]));
            }
        }
    }

    /// `25.1.2.12 SetValueInBuffer ( arrayBuffer, byteIndex, type, value, isTypedArray, order [ , isLittleEndian ] )`
    ///
    /// The start offset is determined by the input buffer instead of a `byteIndex` parameter.
    ///
    /// # Safety
    ///
    /// - There must be enough bytes in `buffer` to write the `TypedArrayElement`.
    /// - Shared storage must be aligned for the element’s atomic type.
    ///
    /// # Panics
    ///
    /// - Panics if the type of `value` is not equal to the content of `kind`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setvalueinbuffer
    pub(crate) unsafe fn set_value(&mut self, value: TypedArrayElement, order: Ordering) {
        unsafe fn write_elem<T: Element>(buffer: SliceRefMut<'_>, value: T, order: Ordering) {
            // <https://tc39.es/ecma262/#sec-setvalueinbuffer>

            // 1. Assert: IsDetachedBuffer(arrayBuffer) is false.
            // 2. Assert: There are sufficient bytes in arrayBuffer starting at byteIndex to represent a value of type.
            // 3. Assert: value is a BigInt if IsBigIntElementType(type) is true; otherwise, value is a Number.
            debug_assert!(buffer.len() >= size_of::<T>());

            // 4. Let block be arrayBuffer.[[ArrayBufferData]].
            // 5. Let elementSize be the Element Size value specified in Table 70 for Element Type type.
            // 6. If isLittleEndian is not present, set isLittleEndian to the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
            // 7. Let rawBytes be NumericToRawBytes(type, value, isLittleEndian).
            // 8. If IsSharedArrayBuffer(arrayBuffer) is true, then
            //     a. Let execution be the [[CandidateExecution]] field of the surrounding agent's Agent Record.
            //     b. Let eventsRecord be the Agent Events Record of execution.[[EventsRecords]] whose [[AgentSignifier]] is AgentSignifier().
            //     c. If isTypedArray is true and IsNoTearConfiguration(type, order) is true, let noTear be true; otherwise let noTear be false.
            //     d. Append WriteSharedMemory { [[Order]]: order, [[NoTear]]: noTear, [[Block]]: block, [[ByteIndex]]: byteIndex, [[ElementSize]]: elementSize, [[Payload]]: rawBytes } to eventsRecord.[[EventList]].
            // 9. Else,
            //     a. Store the individual bytes of rawBytes into block, starting at block[byteIndex].
            // 10. Return unused.

            // SAFETY: The invariants of this operation are ensured by the caller.
            unsafe {
                T::read_mut(buffer).store(value, order);
            }
        }

        // Have to rebind in order to remove the outer `&mut` ref.
        let buffer = match self {
            SliceRefMut::Slice(buf) => SliceRefMut::Slice(buf),
            SliceRefMut::AtomicSlice(buf) => SliceRefMut::AtomicSlice(buf),
        };

        // SAFETY: The invariants of this operation are ensured by the caller.
        unsafe {
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
}

impl<'a> From<&'a mut [u8]> for SliceRefMut<'a> {
    fn from(value: &'a mut [u8]) -> Self {
        Self::Slice(value)
    }
}

impl<'a> From<&'a [AtomicU8]> for SliceRefMut<'a> {
    fn from(value: &'a [AtomicU8]) -> Self {
        Self::AtomicSlice(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{AtomicU8, SliceRef, SliceRefMut};

    #[test]
    fn copy_shared_between_distinct_offsets() {
        let source: Vec<_> = (0..32).map(AtomicU8::new).collect();
        let target: Vec<_> = (0..32).map(|_| AtomicU8::new(0)).collect();
        SliceRefMut::AtomicSlice(&target[2..22]).copy_from(SliceRef::AtomicSlice(&source[1..21]));
        assert_eq!(
            SliceRef::AtomicSlice(&target[2..22]).to_vec(),
            (1..21).collect::<Vec<_>>()
        );
    }

    #[test]
    fn move_shared_backward_during_overlap() {
        let bytes: Vec<_> = (0..32).map(AtomicU8::new).collect();
        SliceRefMut::AtomicSlice(&bytes).copy_within(1..21, 2);
        assert_eq!(
            SliceRef::AtomicSlice(&bytes[2..22]).to_vec(),
            (1..21).collect::<Vec<_>>()
        );
    }

    #[test]
    fn copy_ordinary_to_shared_between_distinct_offsets() {
        let source: Vec<_> = (0..32).collect();
        let target: Vec<_> = (0..32).map(|_| AtomicU8::new(0)).collect();
        SliceRefMut::AtomicSlice(&target[2..22]).copy_from(SliceRef::Slice(&source[1..21]));
        assert_eq!(
            SliceRef::AtomicSlice(&target[2..22]).to_vec(),
            (1..21).collect::<Vec<_>>()
        );
    }

    #[test]
    fn copy_shared_to_ordinary_between_distinct_offsets() {
        let source: Vec<_> = (0..32).map(AtomicU8::new).collect();
        let mut target = [0; 32];
        SliceRefMut::Slice(&mut target[2..22]).copy_from(SliceRef::AtomicSlice(&source[1..21]));
        assert_eq!(&target[2..22], &(1..21).collect::<Vec<_>>());
    }
}
