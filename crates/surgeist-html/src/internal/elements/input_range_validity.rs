//! HTML Input range validity over source-selected limits and converted values.

use std::cmp::Ordering;

/// The independent underflow and overflow results of an Input range check.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlInputRangeValidity {
    underflow: bool,
    overflow: bool,
}

impl HtmlInputRangeValidity {
    /// Whether the current value suffers from an underflow.
    pub fn underflow(self) -> bool {
        self.underflow
    }

    /// Whether the current value suffers from an overflow.
    pub fn overflow(self) -> bool {
        self.overflow
    }

    pub(crate) fn from_comparisons(
        minimum: Option<Ordering>,
        maximum: Option<Ordering>,
        reversed: bool,
    ) -> Self {
        let underflow = minimum == Some(Ordering::Less);
        let overflow = maximum == Some(Ordering::Greater);
        if reversed {
            let in_gap = underflow && overflow;
            return Self {
                underflow: in_gap,
                overflow: in_gap,
            };
        }
        Self {
            underflow,
            overflow,
        }
    }
}
