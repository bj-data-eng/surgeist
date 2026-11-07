use super::LayoutTargetBasis;
use crate::{CompletedLayoutBatchOf, LayoutScalar};

/// Call-scoped preparation authority issued only by CompletedLayoutBatch::apply_to.
/// The actual local target borrow prevents retention as replay authority.
pub struct LayoutBatchPreparation<'call, Node, S: LayoutScalar> {
    pub(crate) batch: &'call CompletedLayoutBatchOf<Node, S>,
    pub(crate) target: &'call Option<LayoutTargetBasis>,
    pub(crate) previous: &'call [Node],
}
impl<Node, S: LayoutScalar> LayoutBatchPreparation<'_, Node, S> {
    #[must_use]
    pub fn batch(&self) -> &CompletedLayoutBatchOf<Node, S> {
        self.batch
    }
    #[must_use]
    pub fn target_basis(&self) -> Option<&LayoutTargetBasis> {
        self.target.as_ref()
    }
    #[must_use]
    pub fn previous_members(&self) -> &[Node] {
        self.previous
    }
}

/// Call-scoped commit authority, minted after successful preparation and a
/// second target check. Retained Prepared data cannot construct this carrier.
pub struct LayoutBatchCommit<'call, Node, S: LayoutScalar, Prepared> {
    pub(crate) batch: &'call CompletedLayoutBatchOf<Node, S>,
    pub(crate) target: &'call Option<LayoutTargetBasis>,
    pub(crate) previous: &'call [Node],
    pub(crate) prepared: Prepared,
}
impl<Node, S: LayoutScalar, Prepared> LayoutBatchCommit<'_, Node, S, Prepared> {
    #[must_use]
    pub fn batch(&self) -> &CompletedLayoutBatchOf<Node, S> {
        self.batch
    }
    #[must_use]
    pub fn target_basis(&self) -> Option<&LayoutTargetBasis> {
        self.target.as_ref()
    }
    #[must_use]
    pub fn previous_members(&self) -> &[Node] {
        self.previous
    }
    #[must_use]
    pub fn into_prepared(self) -> Prepared {
        self.prepared
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutBatchApplyError<E> {
    MissingTarget,
    TrackedRootRequiresBoundRequest,
    StaleSource,
    StaleConstraints,
    Preparation(E),
    OverlappingRoot,
    Allocation,
}

pub(crate) fn check_target<E>(
    requested: Option<&LayoutTargetBasis>,
    current: Option<&LayoutTargetBasis>,
) -> Result<(), LayoutBatchApplyError<E>> {
    match (requested, current) {
        (None, None) => Ok(()),
        (Some(_), None) => Err(LayoutBatchApplyError::MissingTarget),
        (None, Some(_)) => Err(LayoutBatchApplyError::TrackedRootRequiresBoundRequest),
        (Some(requested), Some(current)) if requested.source_basis() != current.source_basis() => {
            Err(LayoutBatchApplyError::StaleSource)
        }
        (Some(requested), Some(current)) if !requested.same_constraints(current) => {
            Err(LayoutBatchApplyError::StaleConstraints)
        }
        (Some(_), Some(_)) => Ok(()),
    }
}
