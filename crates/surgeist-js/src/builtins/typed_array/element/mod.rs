#![forbid(unsafe_code)]

use std::{
    marker::PhantomData,
    ops::{BitAnd, BitOr, BitXor},
    sync::atomic::Ordering,
};

use bytemuck::{AnyBitPattern, NoUninit, bytes_of, pod_read_unaligned};
use num_traits::{WrappingAdd, WrappingSub};

use crate::{
    Context, JsResult, JsValue,
    builtins::{
        array_buffer::{
            shared_bytes::AtomicLane,
            utils::{SliceRef, SliceRefMut},
        },
        typed_array::TypedArrayElement,
    },
    value::Numeric,
};

enum ElementTarget<'a> {
    Shared(AtomicLane<'a>),
    Plain(&'a mut [u8]),
}

/// A checked view of a single native-endian element.
pub(crate) struct ElementRefMut<'a, E> {
    target: ElementTarget<'a>,
    element: PhantomData<E>,
}

impl<E: Element> ElementRefMut<'_, E> {
    pub(crate) fn store(&mut self, value: E, order: Ordering) {
        match &mut self.target {
            ElementTarget::Shared(lane) => lane.store(encode(value), order),
            ElementTarget::Plain(bytes) => bytes.copy_from_slice(bytes_of(&value)),
        }
    }

    fn update(&mut self, order: Ordering, update: impl Fn(E) -> Option<E>) -> E {
        match &mut self.target {
            ElementTarget::Shared(lane) => {
                decode(lane.update(order, |bits| update(decode(bits)).map(encode)))
            }
            ElementTarget::Plain(bytes) => {
                let old = pod_read_unaligned(bytes);
                if let Some(value) = update(old) {
                    bytes.copy_from_slice(bytes_of(&value));
                }
                old
            }
        }
    }

    pub(crate) fn add(&mut self, value: E, order: Ordering) -> E
    where
        E: WrappingAdd,
    {
        self.update(order, |old| Some(old.wrapping_add(&value)))
    }

    pub(crate) fn sub(&mut self, value: E, order: Ordering) -> E
    where
        E: WrappingSub,
    {
        self.update(order, |old| Some(old.wrapping_sub(&value)))
    }

    pub(crate) fn bit_and(&mut self, value: E, order: Ordering) -> E
    where
        E: BitAnd<Output = E>,
    {
        self.update(order, |old| Some(old & value))
    }

    pub(crate) fn bit_or(&mut self, value: E, order: Ordering) -> E
    where
        E: BitOr<Output = E>,
    {
        self.update(order, |old| Some(old | value))
    }

    pub(crate) fn bit_xor(&mut self, value: E, order: Ordering) -> E
    where
        E: BitXor<Output = E>,
    {
        self.update(order, |old| Some(old ^ value))
    }

    pub(crate) fn swap(&mut self, value: E, order: Ordering) -> E {
        self.update(order, |_| Some(value))
    }

    pub(crate) fn compare_exchange(&mut self, expected: E, replacement: E, order: Ordering) -> E
    where
        E: Eq,
    {
        self.update(order, |old| (old == expected).then_some(replacement))
    }
}

// The word representation is independent of the host's native byte order.
// Copy the native representation of E to or from the low-address word bytes.
fn encode<E: Element>(value: E) -> u64 {
    let mut word = [0; size_of::<u64>()];
    word[..size_of::<E>()].copy_from_slice(bytes_of(&value));
    u64::from_le_bytes(word)
}

fn decode<E: Element>(word: u64) -> E {
    pod_read_unaligned(&word.to_le_bytes()[..size_of::<E>()])
}

/// An `u8` that clamps instead of overflowing when converting from a `JsValue`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub(crate) struct ClampedU8(pub(crate) u8);

impl ClampedU8 {
    /// Converts this `ClampedU8` to its big endian representation.
    pub(crate) fn to_be(self) -> Self {
        Self(self.0.to_be())
    }

    /// Converts this `ClampedU8` to its little endian representation.
    pub(crate) fn to_le(self) -> Self {
        Self(self.0.to_le())
    }
}

impl From<ClampedU8> for Numeric {
    fn from(value: ClampedU8) -> Self {
        Numeric::Number(value.0.into())
    }
}

/// Native-endian bits of a 16-bit float.
#[cfg(feature = "float16")]
#[derive(Debug, Clone, Copy, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub(crate) struct Float16(u16);

#[cfg(feature = "float16")]
impl Float16 {
    pub(crate) const fn to_bits(self) -> u16 {
        self.0
    }

    pub(crate) fn to_f64(self) -> f64 {
        float16::f16::from_bits(self.0).to_f64()
    }
}

#[cfg(feature = "float16")]
impl From<float16::f16> for Float16 {
    fn from(value: float16::f16) -> Self {
        Self(value.to_bits())
    }
}

#[cfg(feature = "float16")]
impl PartialEq for Float16 {
    fn eq(&self, other: &Self) -> bool {
        float16::f16::from_bits(self.0) == float16::f16::from_bits(other.0)
    }
}

#[cfg(feature = "float16")]
impl PartialOrd for Float16 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        float16::f16::from_bits(self.0).partial_cmp(&float16::f16::from_bits(other.0))
    }
}

#[cfg(feature = "float16")]
impl From<Float16> for Numeric {
    fn from(value: Float16) -> Self {
        Numeric::Number(value.to_f64())
    }
}

#[cfg(feature = "float16")]
impl std::hash::Hash for Float16 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.0.to_le_bytes());
    }
}

/// A native element that can be stored in a TypedArray.
pub(crate) trait Element: Into<TypedArrayElement> + NoUninit + AnyBitPattern {
    fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self>;

    fn to_little_endian(self) -> Self;

    fn to_big_endian(self) -> Self;

    fn read(buffer: SliceRef<'_>, order: Ordering) -> Self {
        match buffer {
            SliceRef::Slice(bytes) => pod_read_unaligned(&bytes[..size_of::<Self>()]),
            SliceRef::AtomicSlice(bytes) => decode(bytes.lane(size_of::<Self>()).load(order)),
        }
    }

    fn read_mut(buffer: SliceRefMut<'_>) -> ElementRefMut<'_, Self> {
        let target = match buffer {
            SliceRefMut::Slice(bytes) => ElementTarget::Plain(&mut bytes[..size_of::<Self>()]),
            SliceRefMut::AtomicSlice(bytes) => ElementTarget::Shared(bytes.lane(size_of::<Self>())),
        };
        ElementRefMut {
            target,
            element: PhantomData,
        }
    }
}

macro_rules! element {
    ($element:ty, from_js: $from_js:path $(,)?) => {
        element!(
            $element,
            from_js: $from_js,
            to_be: |this: $element| this.to_be(),
            to_le: |this: $element| this.to_le()
        );
    };
    (
        $element:ty,
        from_js: $from_js:expr,
        to_be: $to_be:expr,
        to_le: $to_le:expr $(,)?
    ) => {
        #[allow(clippy::redundant_closure_call)]
        impl Element for $element {
            fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
                $from_js(value, context.access())
            }

            fn to_big_endian(self) -> Self {
                $to_be(self)
            }

            fn to_little_endian(self) -> Self {
                $to_le(self)
            }
        }
    };
}

element!(u8, from_js: JsValue::to_uint8);
element!(i8, from_js: JsValue::to_int8);
element!(u16, from_js: JsValue::to_uint16);
element!(i16, from_js: JsValue::to_int16);
element!(u32, from_js: JsValue::to_u32);
element!(i32, from_js: JsValue::to_i32);
element!(u64, from_js: JsValue::to_big_uint64);
element!(i64, from_js: JsValue::to_big_int64);

element!(
    ClampedU8,
    from_js: |value: &JsValue, context| value.to_uint8_clamp(context).map(ClampedU8),
    to_be: |this: ClampedU8| this.to_be(),
    to_le: |this: ClampedU8| this.to_le(),
);

#[cfg(feature = "float16")]
element!(
    Float16,
    from_js: |value: &JsValue, context| value.to_f16(context).map(Float16::from),
    to_be: |this: Float16| Float16(this.0.to_be()),
    to_le: |this: Float16| Float16(this.0.to_le()),
);

element!(
    f32,
    from_js: |value: &JsValue, context| value.to_number(context).map(|f| f as f32),
    to_be: |this: f32| f32::from_bits(this.to_bits().to_be()),
    to_le: |this: f32| f32::from_bits(this.to_bits().to_le()),
);

element!(
    f64,
    from_js: |value: &JsValue, context| value.to_number(context),
    to_be: |this: f64| f64::from_bits(this.to_bits().to_be()),
    to_le: |this: f64| f64::from_bits(this.to_bits().to_le()),
);
