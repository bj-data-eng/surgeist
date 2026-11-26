//! Test observations preserve the distinction between actual boxes and non-box metadata.
use super::layout as l;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Box,
    NonBox(l::fragmentation::NonBoxKind),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Observation<S: l::scalar::LayoutScalar> {
    pub kind: Kind,
    pub box_output: Option<l::node::NodeOutputOf<S>>,
    pub named_baselines: l::baseline::NamedBaselinesOf<S>,
    pub source_index: l::node::SourceIndex,
    pub location: l::geometry::Point<S>,
    pub size: l::geometry::Size<S>,
    pub content_size: l::geometry::Size<S>,
    pub scroll_geometry: Option<l::scroll::ScrollGeometryOf<S>>,
    pub border: l::geometry::Edges<S>,
    pub padding: l::geometry::Edges<S>,
    pub margin: l::geometry::Edges<S>,
}

impl<S: l::scalar::LayoutScalar> From<l::node::NodeOutputOf<S>> for Observation<S> {
    fn from(output: l::node::NodeOutputOf<S>) -> Self {
        Self {
            kind: Kind::Box,
            box_output: Some(output),
            named_baselines: output.named_baselines,
            source_index: output.source_index,
            location: output.location,
            size: output.size,
            content_size: output.content_size,
            scroll_geometry: output.scroll_geometry,
            border: output.border,
            padding: output.padding,
            margin: output.margin,
        }
    }
}

pub(crate) fn observe<Node: Copy + Eq, S: l::scalar::LayoutScalar>(
    phase: (&l::batch::CompletedLayoutBatchOf<Node, S>, bool),
    node: Node,
    expected_kind: Kind,
) -> Observation<S> {
    let view = phase
        .0
        .node_layout(node)
        .expect("fixture node is published");
    let actual_kind = match view {
        l::fragmentation::NodeLayoutRefOf::SingleBox(_) => Kind::Box,
        l::fragmentation::NodeLayoutRefOf::NonBox(view) => Kind::NonBox(view.kind()),
        l::fragmentation::NodeLayoutRefOf::Fragments(_) => {
            panic!("continuous fixture unexpectedly fragmented")
        }
    };
    assert_eq!(
        actual_kind, expected_kind,
        "fixture publication classification"
    );
    match view {
        l::fragmentation::NodeLayoutRefOf::SingleBox(view) => if phase.1 {
            view.final_output()
        } else {
            view.unrounded_output()
        }
        .into(),
        l::fragmentation::NodeLayoutRefOf::NonBox(view) => {
            let geometry = if phase.1 {
                view.final_geometry()
            } else {
                view.unrounded_geometry()
            };
            Observation {
                kind: Kind::NonBox(view.kind()),
                box_output: None,
                source_index: view.source_index(),
                location: geometry.location,
                size: geometry.size,
                content_size: geometry.content_size,
                named_baselines: geometry.named_baselines,
                scroll_geometry: geometry.scroll_geometry,
                border: geometry.border,
                padding: geometry.padding,
                margin: geometry.margin,
            }
        }
        l::fragmentation::NodeLayoutRefOf::Fragments(_) => {
            panic!("continuous fixture unexpectedly fragmented")
        }
    }
}

/// Select a readback phase without erasing node classification.
pub(crate) fn phase<Node, S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<Node, S>,
    final_phase: bool,
) -> (&l::batch::CompletedLayoutBatchOf<Node, S>, bool) {
    (batch, final_phase)
}

/// Authored normalized fixture input supplies the expected publication role.
/// Ancestor suppression is supplied by the owning fixture traversal.
pub(crate) fn expected_kind<S: l::scalar::LayoutScalar>(
    input: l::node::LayoutInputViewOf<'_, S>,
    suppressed: bool,
) -> Kind {
    if suppressed {
        return Kind::NonBox(l::fragmentation::NonBoxKind::Hidden);
    }
    match input {
        l::node::LayoutInputViewOf::Box(input) if input.display == l::box_model::Display::None => {
            Kind::NonBox(l::fragmentation::NonBoxKind::Hidden)
        }
        l::node::LayoutInputViewOf::Box(_) => Kind::Box,
        l::node::LayoutInputViewOf::InlineText(_) => {
            Kind::NonBox(l::fragmentation::NonBoxKind::InlineText)
        }
        l::node::LayoutInputViewOf::TextControl(_) => {
            Kind::NonBox(l::fragmentation::NonBoxKind::TextControl)
        }
        l::node::LayoutInputViewOf::LineBreak(_) => {
            Kind::NonBox(l::fragmentation::NonBoxKind::LineBreak)
        }
        l::node::LayoutInputViewOf::InlineBoundary(_) => {
            Kind::NonBox(l::fragmentation::NonBoxKind::InlineBoundary)
        }
    }
}
