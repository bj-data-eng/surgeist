//! Encoding and decoding helpers for opcode operands.
//!
//! The VM only decodes bytecode emitted and patched by Boa. Because of that, these helpers rely on
//! internal invariants instead of returning recoverable errors for malformed operand layouts. If
//! decoding fails, it points to a compiler or bytecode management bug.

use thin_vec::ThinVec;

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
    fn encode(self, bytes: &mut Vec<u8>);

    /// Decode the argument from a byte slice
    /// Returns the decoded argument and the new position after reading
    fn decode(bytes: &[u8], pos: usize) -> (Self, usize);
}

#[inline(always)]
fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_i8(bytes: &mut Vec<u8>, value: i8) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[inline(always)]
fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn write_f64(bytes: &mut Vec<u8>, value: f64) {
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
}

impl<T: Argument> Argument for ThinVec<T> {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.len() as u32);
        for arg in self {
            arg.encode(bytes);
        }
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
    fn encode(self, _: &mut Vec<u8>) {}

    fn decode(_: &[u8], pos: usize) -> (Self, usize) {
        ((), pos)
    }
}

impl Argument for IndexOperand {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (arg1, pos) = u32::decode(bytes, pos);
        (arg1.into(), pos)
    }
}

impl Argument for RegisterOperand {
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0);
    }

    fn decode(bytes: &[u8], pos: usize) -> (Self, usize) {
        let (arg1, pos) = u32::decode(bytes, pos);
        (Self::new(arg1), pos)
    }
}

impl Argument for Address {
    #[inline(always)]
    fn encode(self, bytes: &mut Vec<u8>) {
        write_u32(bytes, self.0);
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
            fn encode(self, bytes: &mut Vec<u8>) {
                let ($($i, )*) = self;
                $( $i.encode(bytes); )*
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
            fn encode(self, bytes: &mut Vec<u8>) {
                pastey::paste! {
                    [<write_ $t>](bytes, self);
                }
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
    use std::mem::size_of;
    use thin_vec::ThinVec;

    fn round_trip<T: Argument + PartialEq + Clone>(value: &T) {
        let mut bytes = Vec::new();
        value.clone().encode(&mut bytes);
        let (decoded, pos) = T::decode(&bytes, 0);
        assert_eq!(decoded, *value);
        assert_eq!(pos, bytes.len());
    }

    fn round_trip_eq<T: Argument + Clone, F: Fn(&T, &T) -> bool>(value: &T, eq: F) {
        let mut bytes = Vec::new();
        value.clone().encode(&mut bytes);
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
        Address::new(0xDEAD_BEEF).encode(&mut bytes);
        assert_eq!(bytes.len(), size_of::<u32>());

        bytes.clear();
        (0u64).encode(&mut bytes);
        assert_eq!(bytes.len(), size_of::<u64>());

        bytes.clear();
        (0.0f64).encode(&mut bytes);
        assert_eq!(bytes.len(), size_of::<f64>());
    }

    #[test]
    #[should_panic(expected = "buffer too small")]
    fn decode_truncated_buffer_panics() {
        let bytes = [0u8; 2];
        let _ = u32::decode(&bytes, 0);
    }
}
