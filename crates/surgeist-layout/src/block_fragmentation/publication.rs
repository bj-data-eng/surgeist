use super::LayoutTargetBasis;
use crate::{output::CompletedLayoutBatchOf, scalar::LayoutScalar};

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

impl<E> core::fmt::Display for LayoutBatchApplyError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::MissingTarget => {
                "the request names a publication target that the sink does not have"
            }
            Self::TrackedRootRequiresBoundRequest => {
                "the sink tracks this root and the request carries no bound target"
            }
            Self::StaleSource => "the request's source basis differs from the sink's current basis",
            Self::StaleConstraints => {
                "the request's constraints differ from the sink's current constraints"
            }
            Self::Preparation(_) => "the sink failed to prepare the replacement",
            Self::OverlappingRoot => {
                "the nodes of the batch overlap those of another published root"
            }
            Self::Allocation => "layout could not allocate the storage to apply the batch",
        })
    }
}

/// `source()` returns the sink's preparation error.
impl<E: std::error::Error + 'static> std::error::Error for LayoutBatchApplyError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Preparation(error) => Some(error),
            Self::MissingTarget
            | Self::TrackedRootRequiresBoundRequest
            | Self::StaleSource
            | Self::StaleConstraints
            | Self::OverlappingRoot
            | Self::Allocation => None,
        }
    }
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
