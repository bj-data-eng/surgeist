mod floating;
mod integer;
pub(crate) mod string;

pub use integer::{IntegerKind, IntegerMode, IntegerValue};
pub use string::{ByteString, DomString, UsvString};

pub(crate) use floating::{double, float, unrestricted_double, unrestricted_float};
pub(crate) use integer::convert;
