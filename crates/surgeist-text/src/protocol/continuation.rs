//! Exact after-bearing choice; no receiving geometry or committed-history claim.
use super::{FinalizedLine, PreparedAlternative, PreparedBoundary};

/// An immutable nonempty After emission attached to its authentic cut.
/// Text issues this choice from a selected Soft ending. It authenticates the
/// preparation/alternative/boundary, not that a predecessor was committed.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HyphenationContinuation {
    pub(super) alternative: PreparedAlternative,
    pub(super) boundary: PreparedBoundary,
}
impl HyphenationContinuation {
    pub(crate) fn from_alternative(alternative: PreparedAlternative) -> Option<Self> {
        let prepared = &alternative.prepared;
        let row = &prepared.tables.alternatives[alternative.index];
        if row.text.is_empty() || !row.placement.includes_after() {
            return None;
        }
        let boundary = PreparedBoundary {
            prepared: std::sync::Arc::clone(prepared),
            index: prepared.tables.opportunities[row.opportunity].boundary,
        };
        Some(Self {
            alternative,
            boundary,
        })
    }
    #[must_use]
    pub fn alternative(&self) -> PreparedAlternative {
        self.alternative.clone()
    }
    #[must_use]
    pub fn boundary(&self) -> PreparedBoundary {
        self.boundary.clone()
    }
}
impl FinalizedLine {
    /// Pending nonempty prefix for the exact receiving continuation boundary.
    /// Empty indications retain their ending identity without an emission.
    #[must_use]
    pub fn hyphenation_continuation(&self) -> Option<HyphenationContinuation> {
        let request = &self.inner.selected.request;
        if request.context.end != super::LineEnd::Soft {
            return None;
        }
        HyphenationContinuation::from_alternative(request.alternative.clone()?)
    }
}

pub(super) fn validate(
    prepared: &super::PreparedText,
    start: &PreparedBoundary,
    incoming: Option<&HyphenationContinuation>,
) -> crate::Result<()> {
    if let Some(value) = incoming
        && (!std::sync::Arc::ptr_eq(&prepared.inner, &value.alternative.prepared)
            || value.boundary != *start
            || HyphenationContinuation::from_alternative(value.alternative.clone()).as_ref()
                != Some(value))
    {
        return Err(crate::hyphenation::error(
            crate::HyphenationError::InvalidContinuation,
        ));
    }
    Ok(())
}
