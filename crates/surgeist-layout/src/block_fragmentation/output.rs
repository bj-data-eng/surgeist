use std::sync::Arc;

use crate::{
    BlockGeneration, DefaultScalar, Edges, FragmentationGraphicalCoverageOf, LayoutOutputEntryOf,
    LayoutScalar, NamedBaselinesOf, NodeOutputOf, Point, ScrollGeometryOf, Size, SourceIndex,
};

#[derive(Debug)]
pub(crate) struct ResultIdentity;

/// Identity of a fragment in exactly one completed result.
#[derive(Clone, Debug)]
pub struct LayoutFragmentId {
    pub(crate) result: Arc<ResultIdentity>,
    pub(crate) index: usize,
}
impl PartialEq for LayoutFragmentId {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.result, &other.result)
    }
}
impl Eq for LayoutFragmentId {}

#[derive(Clone, Debug)]
pub struct LayoutFragmentainerId {
    pub(crate) result: Arc<ResultIdentity>,
    pub(crate) index: usize,
}
impl PartialEq for LayoutFragmentainerId {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.result, &other.result)
    }
}
impl Eq for LayoutFragmentainerId {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FragmentCoordinateSpace {
    Root,
    Fragment(LayoutFragmentId),
    Fragmentainer(LayoutFragmentainerId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonBoxKind {
    Hidden,
    InlineText,
    TextControl,
    LineBreak,
    InlineBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxFragmentGeometryOf<S: LayoutScalar = DefaultScalar> {
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub margin: Edges<S>,
    pub named_baselines: NamedBaselinesOf<S>,
    pub scroll_geometry: Option<ScrollGeometryOf<S>>,
}
impl<S: LayoutScalar> BoxFragmentGeometryOf<S> {
    pub(crate) fn from_packet(output: NodeOutputOf<S>) -> Self {
        Self {
            location: output.location,
            size: output.size,
            content_size: output.content_size,
            padding: output.padding,
            border: output.border,
            margin: output.margin,
            named_baselines: output.named_baselines,
            scroll_geometry: output.scroll_geometry,
        }
    }
}

/// Geometry metadata of a non-box participant, with no principal-box claim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonBoxGeometryOf<S: LayoutScalar = DefaultScalar> {
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub padding: Edges<S>,
    pub border: Edges<S>,
    pub margin: Edges<S>,
    pub named_baselines: NamedBaselinesOf<S>,
    pub scroll_geometry: Option<ScrollGeometryOf<S>>,
}
impl<S: LayoutScalar> NonBoxGeometryOf<S> {
    fn from_packet(output: NodeOutputOf<S>) -> Self {
        Self {
            location: output.location,
            size: output.size,
            content_size: output.content_size,
            padding: output.padding,
            border: output.border,
            margin: output.margin,
            named_baselines: output.named_baselines,
            scroll_geometry: output.scroll_geometry,
        }
    }
}

/// Paired principal-box geometry retaining canonical rounding custody.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxGeometryPairOf<Node, S: LayoutScalar = DefaultScalar> {
    pub(crate) unrounded: LayoutOutputEntryOf<Node, S>,
    pub(crate) final_entry: LayoutOutputEntryOf<Node, S>,
}
impl<Node: Copy, S: LayoutScalar> BoxGeometryPairOf<Node, S> {
    #[must_use]
    pub fn unrounded(&self) -> BoxFragmentGeometryOf<S> {
        BoxFragmentGeometryOf::from_packet(self.unrounded.output())
    }
    #[must_use]
    pub fn final_geometry(&self) -> BoxFragmentGeometryOf<S> {
        BoxFragmentGeometryOf::from_packet(self.final_entry.output())
    }
}

/// Overflow custody in the coordinate parent's space, without a principal box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverflowFragmentGeometryOf<S: LayoutScalar = DefaultScalar> {
    pub origin: Point<S>,
    pub content_bounds: crate::ScrollRectOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverflowGeometryPairOf<S: LayoutScalar = DefaultScalar> {
    pub(crate) unrounded: OverflowFragmentGeometryOf<S>,
    pub(crate) final_geometry: OverflowFragmentGeometryOf<S>,
}
impl<S: LayoutScalar> OverflowGeometryPairOf<S> {
    #[must_use]
    pub fn unrounded(&self) -> &OverflowFragmentGeometryOf<S> {
        &self.unrounded
    }
    #[must_use]
    pub fn final_geometry(&self) -> &OverflowFragmentGeometryOf<S> {
        &self.final_geometry
    }
}

/// The geometry kind is shared by both rounding phases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FragmentPayloadOf<Node, S: LayoutScalar = DefaultScalar> {
    ActualBox(BoxGeometryPairOf<Node, S>),
    OverflowOnly(OverflowGeometryPairOf<S>),
}

/// A source-associated fragment and its canonical coordinate ancestry.
#[derive(Clone, Debug)]
pub struct FragmentOutputOf<Node, S: LayoutScalar = DefaultScalar> {
    pub(crate) id: LayoutFragmentId,
    pub(crate) node: Node,
    pub(crate) role: BlockGeneration,
    pub(crate) frame: Option<LayoutFragmentainerId>,
    pub(crate) parent: Option<FragmentCoordinateSpace>,
    pub(crate) order: usize,
    pub(crate) previous: Option<LayoutFragmentId>,
    pub(crate) next: Option<LayoutFragmentId>,
    pub(crate) children: Vec<(SourceIndex, Node)>,
    pub(crate) graphical: Option<FragmentationGraphicalCoverageOf<S>>,
    pub(crate) payload: FragmentPayloadOf<Node, S>,
}
impl<Node: Copy, S: LayoutScalar> FragmentOutputOf<Node, S> {
    #[must_use]
    pub fn id(&self) -> &LayoutFragmentId {
        &self.id
    }
    #[must_use]
    pub fn node(&self) -> Node {
        self.node
    }
    #[must_use]
    pub fn generated_role(&self) -> BlockGeneration {
        self.role
    }
    #[must_use]
    pub fn fragmentainer(&self) -> Option<&LayoutFragmentainerId> {
        self.frame.as_ref()
    }
    /// None when a continuous shared source node has no unique coordinate parent.
    #[must_use]
    pub fn coordinate_parent(&self) -> Option<&FragmentCoordinateSpace> {
        self.parent.as_ref()
    }
    #[must_use]
    pub fn continuation_order(&self) -> usize {
        self.order
    }
    #[must_use]
    pub fn previous(&self) -> Option<&LayoutFragmentId> {
        self.previous.as_ref()
    }
    #[must_use]
    pub fn next(&self) -> Option<&LayoutFragmentId> {
        self.next.as_ref()
    }
    #[must_use]
    pub fn source_children(&self) -> &[(SourceIndex, Node)] {
        &self.children
    }
    #[must_use]
    pub fn graphical_coverage(&self) -> Option<FragmentationGraphicalCoverageOf<S>> {
        self.graphical
    }
    #[must_use]
    pub fn payload(&self) -> &FragmentPayloadOf<Node, S> {
        &self.payload
    }
    #[must_use]
    pub fn principal_box(&self) -> Option<&BoxGeometryPairOf<Node, S>> {
        match &self.payload {
            FragmentPayloadOf::ActualBox(value) => Some(value),
            FragmentPayloadOf::OverflowOnly(_) => None,
        }
    }
    #[must_use]
    pub fn overflow_only(&self) -> Option<&OverflowGeometryPairOf<S>> {
        match &self.payload {
            FragmentPayloadOf::OverflowOnly(value) => Some(value),
            FragmentPayloadOf::ActualBox(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum NodePublication<Node, S: LayoutScalar> {
    /// Exactly one occurrence; only an actual-box payload has singular box access.
    One(FragmentOutputOf<Node, S>),
    Fragments {
        node: Node,
        fragments: Vec<FragmentOutputOf<Node, S>>,
    },
    NonBox {
        kind: NonBoxKind,
        unrounded: LayoutOutputEntryOf<Node, S>,
        final_entry: LayoutOutputEntryOf<Node, S>,
    },
}
impl<Node: Copy, S: LayoutScalar> NodePublication<Node, S> {
    pub(crate) fn node(&self) -> Node {
        match self {
            Self::One(value) => value.node(),
            Self::Fragments { node, .. } => *node,
            Self::NonBox { final_entry, .. } => final_entry.node(),
        }
    }
    pub(crate) fn view(&self) -> NodeLayoutRefOf<'_, Node, S> {
        match self {
            Self::One(value) => match &value.payload {
                FragmentPayloadOf::ActualBox(geometry) => {
                    NodeLayoutRefOf::SingleBox(SingleBoxOutputRefOf {
                        fragment: value,
                        geometry,
                    })
                }
                FragmentPayloadOf::OverflowOnly(_) => {
                    NodeLayoutRefOf::Fragments(core::slice::from_ref(value))
                }
            },
            Self::Fragments { fragments, .. } => NodeLayoutRefOf::Fragments(fragments),
            Self::NonBox {
                kind,
                unrounded,
                final_entry,
            } => NodeLayoutRefOf::NonBox(NonBoxOutputRefOf {
                kind: *kind,
                unrounded,
                final_entry,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SingleBoxOutputRefOf<'a, Node, S: LayoutScalar = DefaultScalar> {
    fragment: &'a FragmentOutputOf<Node, S>,
    geometry: &'a BoxGeometryPairOf<Node, S>,
}
impl<Node: Copy, S: LayoutScalar> SingleBoxOutputRefOf<'_, Node, S> {
    #[must_use]
    pub fn generated_role(&self) -> BlockGeneration {
        self.fragment.role
    }
    #[must_use]
    pub fn node(&self) -> Node {
        self.fragment.node
    }

    #[must_use]
    pub fn id(&self) -> &LayoutFragmentId {
        &self.fragment.id
    }
    #[must_use]
    pub fn coordinate_parent(&self) -> Option<&FragmentCoordinateSpace> {
        self.fragment.parent.as_ref()
    }
    #[must_use]
    pub fn unrounded_entry(&self) -> LayoutOutputEntryOf<Node, S> {
        self.geometry.unrounded
    }
    #[must_use]
    pub fn final_entry(&self) -> LayoutOutputEntryOf<Node, S> {
        self.geometry.final_entry
    }
    #[must_use]
    pub fn unrounded_output(&self) -> NodeOutputOf<S> {
        self.geometry.unrounded.output()
    }
    #[must_use]
    pub fn final_output(&self) -> NodeOutputOf<S> {
        self.geometry.final_entry.output()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NonBoxOutputRefOf<'a, Node, S: LayoutScalar = DefaultScalar> {
    kind: NonBoxKind,
    unrounded: &'a LayoutOutputEntryOf<Node, S>,
    final_entry: &'a LayoutOutputEntryOf<Node, S>,
}
impl<Node: Copy, S: LayoutScalar> NonBoxOutputRefOf<'_, Node, S> {
    #[must_use]
    pub fn kind(&self) -> NonBoxKind {
        self.kind
    }
    #[must_use]
    pub fn source_index(&self) -> SourceIndex {
        self.final_entry.output().source_index
    }
    #[must_use]
    pub fn unrounded_geometry(&self) -> NonBoxGeometryOf<S> {
        NonBoxGeometryOf::from_packet(self.unrounded.output())
    }
    #[must_use]
    pub fn final_geometry(&self) -> NonBoxGeometryOf<S> {
        NonBoxGeometryOf::from_packet(self.final_entry.output())
    }
    #[must_use]
    pub fn unrounded_entry(&self) -> LayoutOutputEntryOf<Node, S> {
        *self.unrounded
    }
    #[must_use]
    pub fn final_entry(&self) -> LayoutOutputEntryOf<Node, S> {
        *self.final_entry
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NodeLayoutRefOf<'a, Node, S: LayoutScalar = DefaultScalar> {
    SingleBox(SingleBoxOutputRefOf<'a, Node, S>),
    Fragments(&'a [FragmentOutputOf<Node, S>]),
    NonBox(NonBoxOutputRefOf<'a, Node, S>),
}
impl<'a, Node, S: LayoutScalar> NodeLayoutRefOf<'a, Node, S> {
    #[must_use]
    pub fn single_box(self) -> Option<SingleBoxOutputRefOf<'a, Node, S>> {
        if let Self::SingleBox(value) = self {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub fn fragments(self) -> Option<&'a [FragmentOutputOf<Node, S>]> {
        if let Self::Fragments(value) = self {
            Some(value)
        } else {
            None
        }
    }
    #[must_use]
    pub fn non_box(self) -> Option<NonBoxOutputRefOf<'a, Node, S>> {
        if let Self::NonBox(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutHandleError {
    WrongResult,
    MissingFragment,
    MissingFragmentainer,
}

/// One captured external fragmentainer in this result's root coordinate space.
#[derive(Clone, Debug)]
pub struct LayoutFragmentainerOutputOf<S: LayoutScalar = DefaultScalar> {
    pub(crate) id: LayoutFragmentainerId,
    pub(crate) kind: crate::FragmentationKind,
    pub(crate) axes: crate::FlowAxes,
    pub(crate) frame: super::BlockFragmentainerOf<S>,
}
impl<S: LayoutScalar> LayoutFragmentainerOutputOf<S> {
    #[must_use]
    pub fn id(&self) -> &LayoutFragmentainerId {
        &self.id
    }
    #[must_use]
    pub fn kind(&self) -> crate::FragmentationKind {
        self.kind
    }
    #[must_use]
    pub fn flow_axes(&self) -> crate::FlowAxes {
        self.axes
    }
    #[must_use]
    pub fn origin(&self) -> Point<S> {
        self.frame.origin
    }
    #[must_use]
    pub fn final_origin(&self) -> Point<S> {
        Point::new(
            crate::scalar::round_layout_coordinate(self.frame.origin.x),
            crate::scalar::round_layout_coordinate(self.frame.origin.y),
        )
    }
    #[must_use]
    pub fn size(&self) -> Size<S> {
        self.frame.size
    }
    #[must_use]
    pub fn boundary(&self) -> crate::FragmentationBreakKinds {
        self.frame.breaks
    }
    #[must_use]
    pub fn page_side(&self) -> Option<crate::FragmentationPageSide> {
        match self.kind {
            crate::FragmentationKind::Page { progression } => self
                .frame
                .page_side
                .map(|physical| progression.classify(physical)),
            crate::FragmentationKind::Column | crate::FragmentationKind::Region => None,
        }
    }
}
