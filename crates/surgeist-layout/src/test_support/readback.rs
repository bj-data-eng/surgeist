//! Typed readback for existing continuous layout fixtures.
use crate::{
    CompletedLayoutBatchOf, LayoutOutputEntryOf, LayoutScalar, NodeLayoutRefOf, NodeOutputOf,
};

#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Unrounded,
    Final,
}

/// A box geometry oracle must observe one actual single box.
pub(crate) fn single_box_output<Node: Copy + Eq, S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<Node, S>,
    node: Node,
    phase: Phase,
) -> NodeOutputOf<S> {
    assert_eq!(
        batch
            .node_layouts()
            .filter(|(published, _)| *published == node)
            .count(),
        1,
        "one actual publication for the requested fixture node"
    );
    let single = batch
        .node_layout(node)
        .expect("fixture publishes the requested node")
        .single_box()
        .expect("box geometry oracle requires SingleBox");
    match phase {
        Phase::Unrounded => single.unrounded_output(),
        Phase::Final => single.final_output(),
    }
}

/// Preserve actual opaque entries when a continuous host snapshots custody.
/// This collector deliberately accepts non-box custody without projecting it
/// into a box and refuses fragmented output unsupported by these old fixtures.
pub(crate) fn continuous_custody<Node: Copy + Eq, S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<Node, S>,
    phase: Phase,
) -> Vec<LayoutOutputEntryOf<Node, S>> {
    batch
        .node_layouts()
        .map(|(_, layout)| match (phase, layout) {
            (Phase::Unrounded, NodeLayoutRefOf::SingleBox(value)) => value.unrounded_entry(),
            (Phase::Final, NodeLayoutRefOf::SingleBox(value)) => value.final_entry(),
            (Phase::Unrounded, NodeLayoutRefOf::NonBox(value)) => value.unrounded_entry(),
            (Phase::Final, NodeLayoutRefOf::NonBox(value)) => value.final_entry(),
            (_, NodeLayoutRefOf::FragmentedBox(_)) => {
                panic!("continuous custody fixture unexpectedly fragmented")
            }
        })
        .collect()
}

/// Independently expected classification from an authored fixture role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    SingleBox,
    NonBox(crate::NonBoxKind),
}

/// Numeric observation of one explicitly classified fixture participant.
/// Non-box fields come directly from dedicated metadata geometry, never from
/// a reconstructed actual NodeOutput.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Observation<S: LayoutScalar> {
    pub kind: Kind,
    pub named_baselines: crate::NamedBaselinesOf<S>,
    pub source_index: crate::SourceIndex,
    pub location: crate::Point<S>,
    pub size: crate::Size<S>,
    pub content_size: crate::Size<S>,
    pub scroll_geometry: Option<crate::ScrollGeometryOf<S>>,
    pub border: crate::Edges<S>,
    pub padding: crate::Edges<S>,
    pub margin: crate::Edges<S>,
}

pub(crate) fn observe<Node: Copy + Eq, S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<Node, S>,
    node: Node,
    phase: Phase,
    expected: Kind,
) -> Observation<S> {
    let observed = match batch
        .node_layout(node)
        .expect("fixture publishes the requested participant")
    {
        NodeLayoutRefOf::SingleBox(value) => {
            let output = match phase {
                Phase::Unrounded => value.unrounded_output(),
                Phase::Final => value.final_output(),
            };
            Observation {
                kind: Kind::SingleBox,
                source_index: output.source_index,
                named_baselines: output.named_baselines,
                location: output.location,
                size: output.size,
                content_size: output.content_size,
                scroll_geometry: output.scroll_geometry,
                border: output.border,
                padding: output.padding,
                margin: output.margin,
            }
        }
        NodeLayoutRefOf::NonBox(value) => {
            let geometry = match phase {
                Phase::Unrounded => value.unrounded_geometry(),
                Phase::Final => value.final_geometry(),
            };
            Observation {
                kind: Kind::NonBox(value.kind()),
                source_index: value.source_index(),
                named_baselines: geometry.named_baselines,
                location: geometry.location,
                size: geometry.size,
                content_size: geometry.content_size,
                scroll_geometry: geometry.scroll_geometry,
                border: geometry.border,
                padding: geometry.padding,
                margin: geometry.margin,
            }
        }
        NodeLayoutRefOf::FragmentedBox(_) => {
            panic!("continuous observation fixture unexpectedly fragmented")
        }
    };
    assert_eq!(
        observed.kind, expected,
        "authored participant classification"
    );
    observed
}

impl<S: LayoutScalar> Observation<S> {
    /// Compare every field of a pre-existing numeric oracle after the factory
    /// has separately asserted its independently expected classification.
    pub(crate) fn assert_numeric(&self, expected: NodeOutputOf<S>) {
        assert_eq!(self.named_baselines, expected.named_baselines);
        assert_eq!(self.source_index, expected.source_index);
        assert_eq!(self.location, expected.location);
        assert_eq!(self.size, expected.size);
        assert_eq!(self.content_size, expected.content_size);
        assert_eq!(self.scroll_geometry, expected.scroll_geometry);
        assert_eq!(self.border, expected.border);
        assert_eq!(self.padding, expected.padding);
        assert_eq!(self.margin, expected.margin);
    }
}
