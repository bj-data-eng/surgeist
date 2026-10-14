//! Actual typed ancestry retained once in admitted Algorithm records.

use super::CssColorPathStep;
use super::input::Operand;
use crate::CssValueOrigin;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SiteRecord<'css> {
    pub(super) operand: Operand<'css>,
    pub(super) parent: Option<usize>,
    pub(super) edge: Option<CssColorPathStep>,
    pub(super) depth: usize,
    pub(super) origin: Option<&'css CssValueOrigin>,
}
impl SiteRecord<'_> {
    pub(super) fn identical(self, other: Self) -> bool {
        self.parent == other.parent
            && self.edge == other.edge
            && self.depth == other.depth
            && match (self.origin, other.origin) {
                (Some(a), Some(b)) => std::ptr::eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && super::input::same_operand(self.operand, other.operand)
    }
}
/// Actual arithmetic traversal records participate in both the admitted stream
/// and its one-to-one evaluation-index translation. Their evaluated value is
/// Empty; the existing numerical evaluator remains authoritative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TraversalRecord {
    Enter { site: usize },
    Return { site: usize },
}
impl TraversalRecord {
    pub(crate) const fn site(self) -> usize {
        match self {
            Self::Enter { site } | Self::Return { site } => site,
        }
    }
}
