//! Natural optional-quality measurements and per-line goal domains.

use super::balance::{BalanceGoal, NaturalResidual};
use super::body::BodyGoal;
use super::tail::TailGoal;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NaturalLine {
    advance: f64,
    fullness: f64,
}

impl NaturalLine {
    pub(crate) fn advance(self) -> f64 {
        self.advance
    }
    pub(crate) fn fullness(self) -> f64 {
        self.fullness
    }

    pub(crate) fn from_components(available: f64, indent: f64, advance: f64) -> Option<Self> {
        if ![available, indent, advance].into_iter().all(f64::is_finite) {
            return None;
        }
        let capacity = available - indent;
        if !capacity.is_finite() || capacity <= 0.0 || advance < 0.0 || advance > capacity {
            return None;
        }
        // Both operands are finite and 0 <= advance <= capacity. Division is
        // bounded by one; it never creates a geometry width or overflowing
        // product. Rounded ties merely miss an optional improvement.
        Some(Self {
            advance,
            fullness: advance / capacity,
        })
    }
}

/// The goal and its key belong to one actual endpoint walk. Tail's invalid
/// optional domain remains an ordinary candidate, never a geometry error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LineQualityGoal {
    Balance(BalanceGoal),
    Tail(TailGoal),
    Body(BodyGoal),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LineQualityKey {
    Balance(NaturalResidual),
    Tail(Option<NaturalLine>),
    Body(Option<NaturalLine>),
}

impl LineQualityGoal {
    pub(crate) fn key(self, available: f64, indent: f64, advance: f64) -> Option<LineQualityKey> {
        match self {
            Self::Balance(_) => NaturalResidual::from_components(available, indent, advance)
                .map(LineQualityKey::Balance),
            Self::Tail(_) => Some(LineQualityKey::Tail(NaturalLine::from_components(
                available, indent, advance,
            ))),
            Self::Body(_) => Some(LineQualityKey::Body(NaturalLine::from_components(
                available, indent, advance,
            ))),
        }
    }

    pub(crate) fn compare(self, left: LineQualityKey, right: LineQualityKey) -> Ordering {
        match (self, left, right) {
            (
                Self::Balance(goal),
                LineQualityKey::Balance(left),
                LineQualityKey::Balance(right),
            ) => goal.compare_distance(left, right),
            (Self::Tail(goal), LineQualityKey::Tail(left), LineQualityKey::Tail(right)) => {
                goal.compare_distance(left, right)
            }
            (Self::Body(goal), LineQualityKey::Body(left), LineQualityKey::Body(right)) => {
                goal.compare_distance(left, right)
            }
            _ => unreachable!("one endpoint walk retains its actual quality goal"),
        }
    }
}
