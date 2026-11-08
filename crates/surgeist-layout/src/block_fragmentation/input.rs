use std::sync::Arc;

use crate::{
    DefaultScalar, FlowAxes, FragmentationBreakKinds, FragmentationInputError, FragmentationKind,
    FragmentationPhysicalPageSide, LayoutRootRequestOf, LayoutScalar, Point, Size,
};

#[derive(Debug)]
struct SourceIdentity;

/// Host promise identifying immutable topology, normalized inputs and measurement
/// environment. Clone while unchanged; issue a fresh basis when any of them change.
#[derive(Clone, Debug)]
pub struct LayoutSourceBasis(Arc<SourceIdentity>);

impl LayoutSourceBasis {
    #[must_use]
    pub fn fresh() -> Self {
        Self(Arc::new(SourceIdentity))
    }
}
impl PartialEq for LayoutSourceBasis {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for LayoutSourceBasis {}

#[derive(Debug)]
struct ConstraintIdentity;

/// Authorization for one source-bound root request, independent of visible output.
#[derive(Clone, Debug)]
pub struct LayoutTargetBasis {
    source: LayoutSourceBasis,
    constraints: Arc<ConstraintIdentity>,
}
impl PartialEq for LayoutTargetBasis {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && Arc::ptr_eq(&self.constraints, &other.constraints)
    }
}
impl Eq for LayoutTargetBasis {}
impl LayoutTargetBasis {
    #[must_use]
    pub fn source_basis(&self) -> &LayoutSourceBasis {
        &self.source
    }
    pub(crate) fn same_constraints(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.constraints, &other.constraints)
    }
}

/// An external content rectangle in the bound request's physical coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockFragmentainerOf<S: LayoutScalar = DefaultScalar> {
    pub origin: Point<S>,
    pub size: Size<S>,
    pub breaks: FragmentationBreakKinds,
    /// Actual physical side; Page progression is supplied by the bound kind.
    pub page_side: Option<FragmentationPhysicalPageSide>,
}

#[derive(Clone, Debug)]
pub(crate) enum BoundMode<S: LayoutScalar> {
    Continuous,
    Block {
        kind: FragmentationKind,
        axes: FlowAxes,
        frames: Vec<BlockFragmentainerOf<S>>,
    },
}
#[derive(Clone, Debug)]
struct BoundRequest<S: LayoutScalar> {
    target: LayoutTargetBasis,
    root: LayoutRootRequestOf<S>,
    mode: BoundMode<S>,
}

/// Immutable source-bound request. Continuous mode supports checked replacement
/// of fragmented output with a single result using the same publication protocol.
#[derive(Clone, Debug)]
pub struct LayoutBoundRequestOf<S: LayoutScalar = DefaultScalar>(Arc<BoundRequest<S>>);
impl<S: LayoutScalar> LayoutBoundRequestOf<S> {
    #[must_use]
    pub fn continuous(source: LayoutSourceBasis, request: LayoutRootRequestOf<S>) -> Self {
        Self::new(source, request, BoundMode::Continuous)
    }
    pub fn block_fragmentation(
        source: LayoutSourceBasis,
        request: LayoutRootRequestOf<S>,
        kind: FragmentationKind,
        flow_axes: FlowAxes,
        frames: Vec<BlockFragmentainerOf<S>>,
    ) -> Result<Self, FragmentationInputError> {
        if frames.is_empty() {
            return Err(FragmentationInputError::InvalidWindow);
        }
        for (index, frame) in frames.iter().enumerate() {
            if !frame.origin.x.is_finite()
                || !frame.origin.y.is_finite()
                || ![frame.size.width, frame.size.height]
                    .into_iter()
                    .all(|value| value.is_finite() && value >= S::ZERO)
            {
                return Err(FragmentationInputError::InvalidGeometry);
            }
            let allowed = if matches!(kind, FragmentationKind::Page { .. }) {
                FragmentationBreakKinds::SPREAD
            } else {
                kind.breaks()
            };
            if !allowed.contains(frame.breaks)
                || (index != 0 && frame.breaks == FragmentationBreakKinds::NONE)
            {
                return Err(FragmentationInputError::InvalidBoundary);
            }
        }
        Ok(Self::new(
            source,
            request,
            BoundMode::Block {
                kind,
                axes: flow_axes,
                frames,
            },
        ))
    }
    fn new(source: LayoutSourceBasis, root: LayoutRootRequestOf<S>, mode: BoundMode<S>) -> Self {
        Self(Arc::new(BoundRequest {
            target: LayoutTargetBasis {
                source,
                constraints: Arc::new(ConstraintIdentity),
            },
            root,
            mode,
        }))
    }
    #[must_use]
    pub fn target_basis(&self) -> LayoutTargetBasis {
        self.0.target.clone()
    }
    #[must_use]
    pub fn root_request(&self) -> LayoutRootRequestOf<S> {
        self.0.root
    }
    pub(crate) fn mode(&self) -> &BoundMode<S> {
        &self.0.mode
    }
}

/// Sticky target installation for a root's publication lifetime. Installing a
/// target authorizes new inputs without replacing old visible geometry. There
/// is no untracking operation; do not reset this owner while its roots survive.
/// Clone only with the coupled visible geometry and caches to preserve the
/// complete publication snapshot, including committed ownership history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutPublicationTargets<Node> {
    entries: Vec<RootPublication<Node>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RootPublication<Node> {
    root: Node,
    target: Option<LayoutTargetBasis>,
    members: Vec<Node>,
}
impl<Node: Copy + Eq> LayoutPublicationTargets<Node> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn install(
        &mut self,
        root: Node,
        target: LayoutTargetBasis,
    ) -> Result<(), std::collections::TryReserveError> {
        if let Some(current) = self.entries.iter_mut().find(|entry| entry.root == root) {
            current.target = Some(target);
        } else {
            self.entries.try_reserve(1)?;
            self.entries.push(RootPublication {
                root,
                target: Some(target),
                members: Vec::new(),
            });
        }
        Ok(())
    }
    #[must_use]
    pub fn get(&self, root: Node) -> Option<&LayoutTargetBasis> {
        self.entries
            .iter()
            .find(|entry| entry.root == root)
            .and_then(|entry| entry.target.as_ref())
    }
    #[must_use]
    pub fn published_members(&self, root: Node) -> &[Node] {
        self.entries
            .iter()
            .find(|entry| entry.root == root)
            .map_or(&[], |entry| &entry.members)
    }
    pub(crate) fn overlaps(&self, root: Node, members: &[Node], bound: bool) -> bool {
        self.entries.iter().any(|entry| {
            entry.root != root
                && (bound || entry.target.is_some())
                && entry.members.iter().any(|node| members.contains(node))
        })
    }
    pub(crate) fn reserve_root(
        &mut self,
        root: Node,
    ) -> Result<(), std::collections::TryReserveError> {
        if !self.entries.iter().any(|entry| entry.root == root) {
            self.entries.try_reserve(1)?;
        }
        Ok(())
    }
    pub(crate) fn commit_members(&mut self, root: Node, members: Vec<Node>) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.root == root) {
            entry.members = members;
        } else {
            self.entries.push(RootPublication {
                root,
                target: None,
                members,
            });
        }
    }
}
impl<Node: Copy + Eq> Default for LayoutPublicationTargets<Node> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockFragmentationFailure {
    MissingFragmentainer { index: usize },
    MissingPageSide { index: usize },
    NoProgress { occurrence: usize },
    InvalidGeometry,
    InvalidSource,
}

/// Used auto-box allocation produced by an accepted native fragment pass.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FragmentBoxAllocation<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) known: Size<Option<S>>,
    pub(crate) indefinite_axis: crate::PhysicalAxis,
    pub(crate) minimum_content: Option<S>,
}
