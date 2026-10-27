//! Checked authored operands over the canonical CSS numerical resolver.

pub(crate) mod input;
pub(crate) mod lowering;
pub(crate) mod numeric;
pub(crate) mod path;
pub(crate) mod position;
pub(crate) mod scalar;
pub(crate) mod site;
pub(crate) mod spelling;

pub(crate) mod controlled;
pub(crate) mod ordinary;
#[cfg(test)]
pub(crate) mod tests;
