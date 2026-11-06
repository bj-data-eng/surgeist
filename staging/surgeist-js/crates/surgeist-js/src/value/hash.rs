use super::{JsValue, StoredValue, inner::InnerValue};
use crate::{JsBigInt, builtins::Number, object::ObjectEdge};
use std::hash::{Hash, Hasher};

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        Self::same_value_zero(self, other)
    }
}

impl Eq for JsValue {}

impl PartialEq for StoredValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Undefined, Self::Undefined) | (Self::Null, Self::Null) => true,
            (Self::Boolean(x), Self::Boolean(y)) => x == y,
            (Self::String(x), Self::String(y)) => x == y,
            (Self::Symbol(x), Self::Symbol(y)) => x == y,
            (Self::Object(x), Self::Object(y)) => x == y,
            (Self::BigInt(x), Self::BigInt(y)) => JsBigInt::same_value_zero(x, y),
            (Self::Integer32(x), Self::Integer32(y)) => x == y,
            (Self::Float64(x), Self::Float64(y)) => Number::same_value_zero(*x, *y),
            (Self::Integer32(x), Self::Float64(y)) | (Self::Float64(y), Self::Integer32(x)) => {
                Number::same_value_zero(f64::from(*x), *y)
            }
            _ => false,
        }
    }
}

impl Eq for StoredValue {}

#[derive(Hash)]
struct UndefinedHashable;

#[derive(Hash)]
struct NullHashable;

fn hash_inner<O: Hash, H: Hasher>(value: &InnerValue<O>, state: &mut H) {
    match value {
        InnerValue::Undefined => UndefinedHashable.hash(state),
        InnerValue::Null => NullHashable.hash(state),
        InnerValue::String(string) => string.hash(state),
        InnerValue::Boolean(boolean) => boolean.hash(state),
        InnerValue::Integer32(integer) => f64::from(*integer).to_bits().hash(state),
        InnerValue::BigInt(bigint) => bigint.hash(state),
        InnerValue::Float64(rational) => rational.to_bits().hash(state),
        InnerValue::Symbol(symbol) => Hash::hash(symbol, state),
        InnerValue::Object(object) => object.hash(state),
    }
}

impl Hash for ObjectEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity().hash(state);
    }
}

impl Hash for StoredValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // SameValueZero equates signed zeros and all NaN payloads.
        match self {
            Self::Float64(value) if *value == 0.0 => 0.0f64.to_bits().hash(state),
            Self::Float64(value) if value.is_nan() => f64::NAN.to_bits().hash(state),
            _ => hash_inner(self, state),
        }
    }
}

impl Hash for JsValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_inner(&self.0, state);
    }
}
