//! Unavailable source relationships in the canonical lexical folds.

use crate::{
    Error, ErrorCode, ErrorDetail, Result, SourceContinuityOperation, SourceProcessingError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Edge {
    pub(super) before_origin: usize,
    pub(super) after_origin: usize,
}
impl Edge {
    pub(super) fn unavailable(self, operation: SourceContinuityOperation) -> Error {
        Error::new(
            ErrorCode::UnsupportedFeature,
            "lexical decision needs source continuity",
        )
        .with_detail(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: self.before_origin,
                after_origin: self.after_origin,
                operation,
            },
        })
    }
}

/// Temporary metadata for an absent edge, not an emitted event or character.
pub(super) struct Frontier {
    pub(super) edge: Edge,
    pub(super) domain: usize,
    pub(super) at: usize,
    pub(super) before_order: usize,
    pub(super) after_order: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Adjacency {
    No,
    Yes,
    Unknown(Edge),
}
impl Adjacency {
    pub(super) fn cross(&mut self, edge: Edge) {
        if matches!(self, Self::Yes) {
            *self = Self::Unknown(edge);
        }
    }
    pub(super) fn resolve(self, operation: SourceContinuityOperation) -> Result<bool> {
        match self {
            Self::No => Ok(false),
            Self::Yes => Ok(true),
            Self::Unknown(edge) => Err(edge.unavailable(operation)),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Neighbor {
    Absent,
    Known(char),
    Unknown { continued: char, edge: Edge },
}
impl Neighbor {
    pub(super) fn context(value: Option<char>) -> Self {
        value.map_or(Self::Absent, Self::Known)
    }
    pub(super) fn cross(&mut self, edge: Edge) {
        if let Self::Known(continued) = *self {
            *self = Self::Unknown { continued, edge };
        }
    }
    fn continued(self) -> Option<char> {
        match self {
            Self::Absent => None,
            Self::Known(value)
            | Self::Unknown {
                continued: value, ..
            } => Some(value),
        }
    }
    fn uncertain(self) -> Option<Edge> {
        match self {
            Self::Unknown { edge, .. } => Some(edge),
            _ => None,
        }
    }

    /// Lift the one canonical predicate over its at-most-two real context
    /// values. No source execution is repeated and no subsets are searched.
    pub(super) fn segment_break(
        self,
        after: Self,
        decision: impl Fn(Option<char>, Option<char>) -> bool,
    ) -> Result<bool> {
        let before_value = self.continued();
        let after_value = after.continued();
        let result = decision(before_value, after_value);
        if let Some(edge) = self.uncertain()
            && decision(None, after_value) != result
        {
            return Err(edge.unavailable(SourceContinuityOperation::WhiteSpace));
        }
        if let Some(edge) = after.uncertain()
            && decision(before_value, None) != result
        {
            return Err(edge.unavailable(SourceContinuityOperation::WhiteSpace));
        }
        if let (Some(edge), Some(_)) = (self.uncertain(), after.uncertain())
            && decision(None, None) != result
        {
            return Err(edge.unavailable(SourceContinuityOperation::WhiteSpace));
        }
        Ok(result)
    }
}

/// A scope marker can lie inside an absent origin relationship. In reverse
/// trim traversal its frontier token lies outside the scanned range, so retain
/// that genuine event-interval dependency at entry too.
pub(super) fn at_boundary(frontiers: &[Frontier], domain: usize, order: usize) -> Option<Edge> {
    frontiers
        .iter()
        .find(|frontier| {
            frontier.domain == domain
                && frontier.before_order < order
                && order < frontier.after_order
        })
        .map(|frontier| frontier.edge)
}

/// The genuine domain edge locates its immediate exterior independently of
/// the source relationship between the parent originals on opposite sides.
/// This certifies only entry to that exterior, never a later source seam.
pub(super) struct ExteriorContinuity {
    missing: Option<Edge>,
    enclosing: Option<Edge>,
}
impl ExteriorContinuity {
    pub(super) fn at_boundary(
        frontiers: &[Frontier],
        boundary: &super::super::contextual::TrimBoundary,
    ) -> Self {
        let edge = at_boundary(frontiers, boundary.exterior_domain, boundary.order);
        if boundary.domain != boundary.exterior_domain {
            Self {
                missing: None,
                enclosing: edge,
            }
        } else {
            Self {
                missing: edge,
                enclosing: None,
            }
        }
    }

    pub(super) fn cross(&mut self, edge: Edge) {
        // In forward traversal the enclosing frontier is emitted at the next
        // parent original, after the actual child-domain exit. Crossing it here
        // would spuriously demand continuation through the child just exited.
        if self.enclosing != Some(edge) {
            self.missing = self.missing.or(Some(edge));
        }
    }

    pub(super) fn missing(&self) -> Option<Edge> {
        self.missing
    }
}
