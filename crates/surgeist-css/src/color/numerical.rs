//! Fixed numerical colors and kernels, independent of CSS recipe storage.
//!
//! Authored domains, expression execution, reached facts and ordered mix
//! scheduling stay in the CSS resolver. These kernels accept resolved operands.

pub(super) mod interpolation;
pub(super) mod serialization;
pub(super) mod value;
