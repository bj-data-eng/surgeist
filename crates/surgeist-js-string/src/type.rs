//! Code-unit types for typed string builders.

mod sealed {
    /// Seal to prevent others from implementing their own string types.
    pub trait Sealed {}
}

/// Trait that maps the data type to the appropriate internal types and constants.
pub trait StringType: sealed::Sealed {
    /// The unit of a character for this type of string. For example, UTF-16 should
    /// have a 16-bits size, while ASCII should have 8 bits.
    type Byte: Copy + Eq + 'static;
}

// It is good defensive programming to have [`Latin1`] `!Copy`, as it should
// not be used as a value anyway.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Latin1 {}

impl sealed::Sealed for Latin1 {}
impl StringType for Latin1 {
    type Byte = u8;
}

// It is good defensive programming to have [`Utf16`] `!Copy`, as it should
// not be used as a value anyway.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Utf16 {}

impl sealed::Sealed for Utf16 {}
impl StringType for Utf16 {
    type Byte = u16;
}
