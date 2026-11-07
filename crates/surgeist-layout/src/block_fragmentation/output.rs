use std::sync::Arc;

use crate::{
    BlockGeneration, DefaultScalar, Edges, FragmentationGraphicalCoverageOf, LayoutOutputEntryOf,
    LayoutScalar, NamedBaselinesOf, NodeOutputOf, Point, ScrollGeometryOf, Size, SourceIndex,
};

#[derive(Debug)]
pub(crate) struct ResultIdentity;

/// Identity of an actual box instance in exactly one completed result.
#[derive(Clone, Debug)]
pub struct LayoutBoxId {
    pub(crate) result: Arc<ResultIdentity>,
    pub(crate) index: usize,
}
impl PartialEq for LayoutBoxId {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.result, &other.result)
    }
}
impl Eq for LayoutBoxId {}

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
pub enum BoxCoordinateSpace {
    Root,
    Box(LayoutBoxId),
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

/// Actual fragment geometry and accepted source children, not a singular NodeOutput.
#[derive(Clone, Debug)]
pub struct BoxFragmentOutputOf<Node, S: LayoutScalar = DefaultScalar> {
    pub(crate) id: LayoutBoxId,
    pub(crate) node: Node,
    pub(crate) role: BlockGeneration,
    pub(crate) frame: LayoutFragmentainerId,
    pub(crate) parent: BoxCoordinateSpace,
    pub(crate) order: usize,
    pub(crate) previous: Option<LayoutBoxId>,
    pub(crate) next: Option<LayoutBoxId>,
    pub(crate) children: Vec<(SourceIndex, Node)>,
    pub(crate) graphical: Option<FragmentationGraphicalCoverageOf<S>>,
    pub(crate) unrounded: BoxFragmentGeometryOf<S>,
    pub(crate) final_geometry: BoxFragmentGeometryOf<S>,
}
impl<Node: Copy, S: LayoutScalar> BoxFragmentOutputOf<Node, S> {
    #[must_use]
    pub fn id(&self) -> &LayoutBoxId {
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
    pub fn fragmentainer(&self) -> &LayoutFragmentainerId {
        &self.frame
    }
    #[must_use]
    pub fn coordinate_parent(&self) -> &BoxCoordinateSpace {
        &self.parent
    }
    #[must_use]
    pub fn continuation_order(&self) -> usize {
        self.order
    }
    #[must_use]
    pub fn previous(&self) -> Option<&LayoutBoxId> {
        self.previous.as_ref()
    }
    #[must_use]
    pub fn next(&self) -> Option<&LayoutBoxId> {
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
    pub fn unrounded(&self) -> &BoxFragmentGeometryOf<S> {
        &self.unrounded
    }
    #[must_use]
    pub fn final_geometry(&self) -> &BoxFragmentGeometryOf<S> {
        &self.final_geometry
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SingleBox<Node, S: LayoutScalar> {
    pub(crate) role: BlockGeneration,
    pub(crate) id: LayoutBoxId,
    pub(crate) parent: Option<BoxCoordinateSpace>,
    pub(crate) unrounded: LayoutOutputEntryOf<Node, S>,
    pub(crate) final_entry: LayoutOutputEntryOf<Node, S>,
}
#[derive(Clone, Debug)]
pub(crate) enum NodePublication<Node, S: LayoutScalar> {
    Single(SingleBox<Node, S>),
    Fragments {
        node: Node,
        fragments: Vec<BoxFragmentOutputOf<Node, S>>,
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
            Self::Single(value) => value.final_entry.node(),
            Self::Fragments { node, .. } => *node,
            Self::NonBox { final_entry, .. } => final_entry.node(),
        }
    }
    pub(crate) fn view(&self) -> NodeLayoutRefOf<'_, Node, S> {
        match self {
            Self::Single(value) => NodeLayoutRefOf::SingleBox(SingleBoxOutputRefOf(value)),
            Self::Fragments { fragments, .. } => NodeLayoutRefOf::FragmentedBox(fragments),
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
pub struct SingleBoxOutputRefOf<'a, Node, S: LayoutScalar = DefaultScalar>(&'a SingleBox<Node, S>);
impl<Node: Copy, S: LayoutScalar> SingleBoxOutputRefOf<'_, Node, S> {
    #[must_use]
    pub fn generated_role(&self) -> BlockGeneration {
        self.0.role
    }
    #[must_use]
    pub fn node(&self) -> Node {
        self.0.final_entry.node()
    }

    #[must_use]
    pub fn id(&self) -> &LayoutBoxId {
        &self.0.id
    }
    #[must_use]
    pub fn coordinate_parent(&self) -> Option<&BoxCoordinateSpace> {
        self.0.parent.as_ref()
    }
    #[must_use]
    pub fn unrounded_entry(&self) -> LayoutOutputEntryOf<Node, S> {
        self.0.unrounded
    }
    #[must_use]
    pub fn final_entry(&self) -> LayoutOutputEntryOf<Node, S> {
        self.0.final_entry
    }
    #[must_use]
    pub fn unrounded_output(&self) -> NodeOutputOf<S> {
        self.0.unrounded.output()
    }
    #[must_use]
    pub fn final_output(&self) -> NodeOutputOf<S> {
        self.0.final_entry.output()
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
    FragmentedBox(&'a [BoxFragmentOutputOf<Node, S>]),
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
    pub fn fragments(self) -> Option<&'a [BoxFragmentOutputOf<Node, S>]> {
        if let Self::FragmentedBox(value) = self {
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
    MissingBox,
    NotFragmented,
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
        self.frame.page_side
    }
}

/// Checked view of an actual box instance, resolving either coordinate-parent kind.
#[derive(Clone, Copy, Debug)]
pub enum BoxInstanceRefOf<'a, Node, S: LayoutScalar = DefaultScalar> {
    SingleBox(SingleBoxOutputRefOf<'a, Node, S>),
    Fragment(&'a BoxFragmentOutputOf<Node, S>),
}
