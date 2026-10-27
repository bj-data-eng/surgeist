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
    /// `auto`: the column count is determined by the other column properties.
    Auto,
    /// A positive `<integer>` column count.
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
    /// `auto`: columns are filled sequentially.
    Auto,
    /// `balance`: content is balanced between columns, as far as possible; in fragmented contexts only the last fragment is balanced.
    Balance,
    /// `balance-all`: content is balanced between columns, as far as possible; in fragmented contexts all fragments are balanced.
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
    /// `none`: the element does not span multiple columns.
    None,
    /// `all`: the element spans all columns of the multicol container.
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

    /// Returns the column width, which may be `auto`.
    #[must_use]
    pub const fn width(&self) -> &CssSizeValue {
        &self.width
    }

    /// Returns the column count, which may be `auto`.
    #[must_use]
    pub const fn count(&self) -> &CssColumnCount {
        &self.count
    }
}
