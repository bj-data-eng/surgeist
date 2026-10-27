//! Checked authored counter-changing values from CSS Lists 3.

use crate::numeric::{CssInteger, CssNumeric};
use crate::values::content::CssContentCounterName;

/// One named counter change with an optional authored integer operand.
///
/// The consuming property supplies the meaning of an omitted integer.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterChangeValue {
    name: CssContentCounterName,
    value: Option<CssNumeric<CssInteger>>,
}

impl CssCounterChangeValue {
    /// Retains the checked name and authored presence of the integer operand.
    #[must_use]
    pub const fn new(name: CssContentCounterName, value: Option<CssNumeric<CssInteger>>) -> Self {
        Self { name, value }
    }

    #[must_use]
    pub const fn name(&self) -> &CssContentCounterName {
        &self.name
    }

    #[must_use]
    pub const fn value(&self) -> Option<&CssNumeric<CssInteger>> {
        self.value.as_ref()
    }
}

/// A checked `counter-reset`, `counter-increment`, or `counter-set` value.
///
/// `None` is distinct from a nonempty ordered list, including repeated names.
///
/// # Specified serialization
///
/// Emits every counter name and effective integer for the selected longhand. Omitted operands
/// remain absent in this authored model. Serialization inserts `0` for reset/set or `1` for
/// increment, without executing any counter operations or resolving symbolic integer calculations.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCounterChangesValue {
    changes: Option<Vec<CssCounterChangeValue>>,
}

impl CssCounterChangesValue {
    /// The `none` initial, with no counter operations to interpret downstream.
    #[must_use]
    pub const fn none() -> Self {
        Self { changes: None }
    }

    /// Rejects an empty list; retains duplicates and authored order.
    #[must_use]
    pub fn try_changes(changes: Vec<CssCounterChangeValue>) -> Option<Self> {
        (!changes.is_empty()).then_some(Self {
            changes: Some(changes),
        })
    }

    /// Returns `None` for the keyword, or a nonempty borrowed change list.
    #[must_use]
    pub fn changes(&self) -> Option<&[CssCounterChangeValue]> {
        self.changes.as_deref()
    }
}

/// The consuming longhand for property-specific counter-value serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCounterProperty {
    Reset,
    Increment,
    Set,
}
