use crate::numeric::{CssInteger, CssNumeric, CssPositive};
use crate::values::box_model::CssSizeValue;

/// The authored `column-count` value.
///
/// # Specified serialization
///
/// Serializes the specified count or `auto`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColumnCount {
    Auto,
    Count(CssNumeric<CssInteger, CssPositive>),
}

/// The authored `column-fill` keyword.
///
/// # Specified serialization
///
/// Serializes the authored fill keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColumnFill {
    Auto,
    Balance,
    BalanceAll,
}

/// The authored `column-span` keyword.
///
/// # Specified serialization
///
/// Serializes the authored span keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColumnSpan {
    None,
    All,
}

/// The authored component values of the `columns` shorthand.
///
/// Omitted components are represented by their specified `auto` initial value. This syntax value
/// does not calculate used column sizes or perform multi-column layout.
///
/// # Specified serialization
///
/// Serializes both effective values in width-then-count order, including omitted `auto`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColumns {
    width: CssSizeValue,
    count: CssColumnCount,
}

impl CssColumns {
    /// Constructs both effective values; omitted shorthand components are supplied as `auto`.
    pub const fn new(width: CssSizeValue, count: CssColumnCount) -> Self {
        Self { width, count }
    }

    #[must_use]
    pub const fn width(&self) -> &CssSizeValue {
        &self.width
    }

    #[must_use]
    pub const fn count(&self) -> &CssColumnCount {
        &self.count
    }
}
