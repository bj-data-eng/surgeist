//! Component values, token views, serialized values and their limits and errors.

pub(crate) mod values;

#[doc(inline)]
pub use values::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueLimits,
    CssComponentValueRef, CssComponentValues, CssFunctionValue, CssHashFlag, CssNumericTokenKind,
    CssNumericTokenRef, CssSerializedOrigin, CssSerializedOriginSegment, CssSerializedValue,
    CssSimpleBlock, CssValueOrigin, CssValueTokenRef,
};
