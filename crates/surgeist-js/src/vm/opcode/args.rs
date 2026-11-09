//! Encoding and decoding helpers for opcode operands.
//!
//! The VM only decodes bytecode emitted and patched by Boa. Because of that, these helpers rely on
//! internal invariants instead of returning recoverable errors for malformed operand layouts. If
//! decoding fails, it points to a compiler or bytecode management bug.

use thin_vec::ThinVec;

use crate::{
    JsResult,
    bytecompiler::control::{CompilerControl, CompilerWork, reserve, size_overflow},
};

use super::{Address, IndexOperand, RegisterOperand};

#[inline(always)]
#[track_caller]
fn read_operand_bytes<const N: usize>(bytes: &[u8], offset: usize) -> ([u8; N], usize) {
    let operand = bytes
        .get(offset..)
        .and_then(<[u8]>::first_chunk::<N>)
        .expect("buffer too small to read operand");
    (*operand, offset + N)
}

pub(crate) trait Argument: Sized + std::fmt::Debug {
    /// Encode the argument into a byte slice
    fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()>;

    /// Decode the argument from a byte slice
    /// Returns the decoded argument and the new position after reading
    fn decode(bytes: &[u8], pos: usize) -> (Self, usize);
}

#[inline(always)]
fn write_operand_bytes<const N: usize>(
    bytes: &mut Vec<u8>,
    value: [u8; N],
    control: &mut CompilerControl<'_>,
) -> JsResult<()> {
    let end = bytes.len().checked_add(N).ok_or_else(size_overflow)?;
    u32::try_from(end).map_err(|_| size_overflow())?;
    control.admit(CompilerWork::EmitBytes { bytes: N })?;
    reserve(bytes, N, control)?;
    bytes.extend_from_slice(&value);
    Ok(())
}

impl<T: Argument> Argument for ThinVec<T> {
    fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()> {
        let len = u32::try_from(self.len()).map_err(|_| size_overflow())?;
        len.encode(bytes, control)?;
        let mut arguments = self.into_iter();
        for _ in 0..len {
            control.admit(CompilerWork::MetadataVisit)?;
            arguments
                .next()
                .expect("the encoded operand count matches its owned vector")
                .encode(bytes, control)?;
        }
        Ok(())
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (len, mut pos) = u32::decode(bytes, pos);
        let total_len = len as usize;
        let mut result = ThinVec::with_capacity(total_len);
        for _ in 0..total_len {
            let (arg, new_pos) = T::decode(bytes, pos);
            result.push(arg);
            pos = new_pos;
        }
        (result, pos)
    }
}

impl Argument for () {
    fn encode(self, _: &mut Vec<u8>, _: &mut CompilerControl<'_>) -> JsResult<()> {
        Ok(())
    }

    fn decode(_: &[u8], pos: usize) -> (Self, usize) {
        ((), pos)
    }
}

impl Argument for IndexOperand {
    fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()> {
        self.0.encode(bytes, control)
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (arg1, pos) = u32::decode(bytes, pos);
        (arg1.into(), pos)
    }
}

impl Argument for RegisterOperand {
    fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()> {
        self.0.encode(bytes, control)
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (arg1, pos) = u32::decode(bytes, pos);
        (Self::new(arg1), pos)
    }
}

impl Argument for Address {
    #[inline(always)]
    fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()> {
        self.0.encode(bytes, control)
    }

    #[inline(always)]
    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (value, pos) = u32::decode(bytes, pos);
        (Self::new(value), pos)
    }
}

macro_rules! impl_argument_for_tuple {
    ($( $i: ident  $t: ident ),*) => {
        impl<$( $t: Argument, )*> Argument for ($( $t, )*) {
            #[inline(always)]
            fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()> {
                let ($($i, )*) = self;
                $( $i.encode(bytes, control)?; )*
                Ok(())
            }

            #[inline(always)]
            fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
                $( let ($i, pos) = $t::decode(bytes, pos); )*
                (($($i,)*), pos)
            }
        }
    };
}

impl_argument_for_tuple!(a A);
impl_argument_for_tuple!(a A, b B);
impl_argument_for_tuple!(a A, b B, c C);
impl_argument_for_tuple!(a A, b B, c C, d D);
impl_argument_for_tuple!(a A, b B, c C, d D, e E);

macro_rules! impl_argument_for_int {
    ($( $t: ty )*) => {
        $(
        impl Argument for $t {
            #[inline(always)]
            fn encode(self, bytes: &mut Vec<u8>, control: &mut CompilerControl<'_>) -> JsResult<()> {
                write_operand_bytes(bytes, self.to_le_bytes(), control)
            }

            #[inline(always)]
            fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
                let (operand, next) = read_operand_bytes(bytes, pos);
                (Self::from_le_bytes(operand), next)
            }
        }
        )*
    };
}

impl_argument_for_int!(u8 u16 u32 u64 i8 i16 i32 f32 f64);

#[cfg(test)]
mod tests {
    use super::{Address, Argument, IndexOperand, RegisterOperand};
    use crate::{
        JsError, JsValue,
        bytecompiler::control::{CompilerControl, CompilerDepth, CompilerWork},
    };
    use std::mem::size_of;
    use thin_vec::ThinVec;

    fn encode<T: Argument>(value: T, bytes: &mut Vec<u8>) {
        let depth = CompilerDepth::new();
        value
            .encode(bytes, &mut CompilerControl::new(None, &depth))
            .unwrap();
    }

    fn round_trip<T: Argument + PartialEq + Clone>(value: &T) {
        let mut bytes = Vec::new();
        encode(value.clone(), &mut bytes);
        let (decoded, pos) = T::decode(&bytes, 0);
        assert_eq!(decoded, *value);
        assert_eq!(pos, bytes.len());
    }

    fn round_trip_eq<T: Argument + Clone, F: Fn(&T, &T) -> bool>(value: &T, eq: F) {
        let mut bytes = Vec::new();
        encode(value.clone(), &mut bytes);
        let (decoded, pos) = T::decode(&bytes, 0);
        assert!(eq(&decoded, value));
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn test_unit_round_trip() {
        round_trip(&());
    }

    #[test]
    fn test_address_round_trip() {
        round_trip_eq(&Address::new(0), |a, b| u32::from(*a) == u32::from(*b));
        round_trip_eq(&Address::new(0x1234_5678), |a, b| {
            u32::from(*a) == u32::from(*b)
        });
    }

    #[test]
    fn checked_address_addition_accepts_the_exact_maximum_and_rejects_overflow() {
        assert_eq!(
            Address::new(0).checked_add(u32::MAX).unwrap(),
            Address::new(u32::MAX)
        );
        assert_eq!(
            Address::new(u32::MAX - 4).checked_add(4).unwrap(),
            Address::new(u32::MAX)
        );
        assert_eq!(
            Address::new(u32::MAX).checked_add(0).unwrap(),
            Address::new(u32::MAX)
        );
        for (start, offset) in [(u32::MAX, 1), (u32::MAX - 4, 5), (1, u32::MAX)] {
            assert_eq!(
                Address::new(start)
                    .checked_add(offset)
                    .unwrap_err()
                    .as_engine(),
                Some(&crate::error::EngineError::Resource(
                    crate::error::EngineResourceError::SizeOverflow
                )),
            );
        }
    }

    #[test]
    fn test_register_operand_round_trip() {
        round_trip_eq(&RegisterOperand::new(0), |a, b| {
            u32::from(*a) == u32::from(*b)
        });
        round_trip_eq(&RegisterOperand::new(255), |a, b| {
            u32::from(*a) == u32::from(*b)
        });
    }

    #[test]
    fn test_varying_operand_round_trip() {
        round_trip_eq(&IndexOperand::new(0), |a, b| u32::from(*a) == u32::from(*b));
        round_trip_eq(&IndexOperand::new(0xFFFF_FFFF), |a, b| {
            u32::from(*a) == u32::from(*b)
        });
    }

    #[test]
    fn test_primitive_round_trips() {
        round_trip(&0u8);
        round_trip(&255u8);
        round_trip(&0i8);
        round_trip(&(-128i8));
        round_trip(&0u16);
        round_trip(&0xFFFFu16);
        round_trip(&(-32768i16));
        round_trip(&0u32);
        round_trip(&0xFFFF_FFFFu32);
        round_trip(&0i32);
        round_trip(&i32::MIN);
        round_trip(&0u64);
        round_trip(&0xFFFF_FFFF_FFFF_FFFFu64);
        round_trip(&0.0f32);
        round_trip(&1.5f32);
        round_trip(&0.0f64);
        round_trip(&1.5f64);
    }

    #[test]
    fn decodes_little_endian_operand_bytes_at_an_offset() {
        let bytes = [
            0xff, // unrelated preceding byte
            0x34, 0x12, // u16
            0x00, 0x00, 0x00, 0x80, // i32
            0x00, 0x00, 0x00, 0x80, // f32
            0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x7f, // f64
            0xff, // unrelated following byte
        ];
        let ((wide, signed, zero, nan), position) = <(u16, i32, f32, f64)>::decode(&bytes, 1);
        assert_eq!(wide, 0x1234);
        assert_eq!(signed, i32::MIN);
        assert_eq!(zero.to_bits(), 0x8000_0000);
        assert_eq!(nan.to_bits(), 0x7ff8_0000_0000_1234);
        assert_eq!(position, bytes.len() - 1);
    }

    #[test]
    #[should_panic(expected = "buffer too small")]
    fn decode_rejects_overflowing_offset_as_out_of_bounds() {
        let _ = u32::decode(&[0; 4], usize::MAX);
    }

    #[test]
    fn test_tuple_round_trips() {
        round_trip(&(0u8, 1u8));
        round_trip(&(0u32, 1u32));
        let tuple = (Address::new(0), RegisterOperand::new(1));
        round_trip_eq(&tuple, |a, b| {
            u32::from(a.0) == u32::from(b.0) && u32::from(a.1) == u32::from(b.1)
        });
    }

    #[test]
    fn test_thin_vec_round_trip() {
        let v: ThinVec<u32> = ThinVec::new();
        round_trip(&v);
        let v: ThinVec<u32> = [1u32, 2, 3].into_iter().collect();
        round_trip(&v);
        let v: ThinVec<RegisterOperand> = [RegisterOperand::new(0), RegisterOperand::new(1)]
            .into_iter()
            .collect();
        round_trip_eq(&v, |a, b| {
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|(x, y)| u32::from(*x) == u32::from(*y))
        });
    }

    #[test]
    fn test_encoded_size_matches_type_size() {
        let mut bytes = Vec::new();
        encode(Address::new(0xDEAD_BEEF), &mut bytes);
        assert_eq!(bytes.len(), size_of::<u32>());

        bytes.clear();
        encode(0u64, &mut bytes);
        assert_eq!(bytes.len(), size_of::<u64>());

        bytes.clear();
        encode(0.0f64, &mut bytes);
        assert_eq!(bytes.len(), size_of::<f64>());
    }

    #[test]
    #[should_panic(expected = "buffer too small")]
    fn decode_truncated_buffer_panics() {
        let bytes = [0u8; 2];
        let _ = u32::decode(&bytes, 0);
    }

    #[test]
    fn scalar_encoding_admits_exact_bytes_before_any_write() {
        let marker = JsError::from_opaque(JsValue::new(197)).unwrap();
        for limit in [3, 4] {
            let mut bytes = Vec::with_capacity(4);
            let mut accepted = 0;
            let mut admission = |work| {
                let CompilerWork::EmitBytes { bytes } = work else {
                    panic!("preallocated scalar encoding needs no other work");
                };
                if accepted + bytes > limit {
                    return Err(marker.clone());
                }
                accepted += bytes;
                Ok(())
            };
            let depth = CompilerDepth::new();
            let result = 0x1234_5678_u32.encode(
                &mut bytes,
                &mut CompilerControl::new(Some(&mut admission), &depth),
            );
            if limit == 4 {
                result.unwrap();
                assert_eq!(accepted, 4);
                assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);
            } else {
                assert_eq!(result.unwrap_err(), marker);
                assert_eq!(accepted, 0);
                assert!(bytes.is_empty());
            }
        }
    }

    #[test]
    fn variable_operand_refusal_keeps_only_the_admitted_prefix() {
        // Four length bytes plus two element visits and eight element bytes.
        const EXACT: usize = 14;
        let marker = JsError::from_opaque(JsValue::new(198)).unwrap();
        for limit in [EXACT - 1, EXACT] {
            let values: ThinVec<u32> = [0x1234_5678, 0x90ab_cdef].into_iter().collect();
            let mut bytes = Vec::with_capacity(12);
            let mut accepted = 0;
            let mut probes = 0;
            let mut admission = |work| {
                let units = match work {
                    CompilerWork::EmitBytes { bytes } => bytes,
                    CompilerWork::MetadataVisit => {
                        probes += 1;
                        1
                    }
                    _ => panic!("preallocated operands need only bytes and element visits"),
                };
                if accepted + units > limit {
                    return Err(marker.clone());
                }
                accepted += units;
                Ok(())
            };
            let depth = CompilerDepth::new();
            let result = values.encode(
                &mut bytes,
                &mut CompilerControl::new(Some(&mut admission), &depth),
            );
            assert_eq!(probes, 2);
            if limit == EXACT {
                result.unwrap();
                assert_eq!(accepted, EXACT);
                assert_eq!(
                    bytes,
                    [2, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0xef, 0xcd, 0xab, 0x90]
                );
            } else {
                assert_eq!(result.unwrap_err(), marker);
                assert_eq!(accepted, 10);
                assert_eq!(bytes, [2, 0, 0, 0, 0x78, 0x56, 0x34, 0x12]);
            }
        }
    }
}
