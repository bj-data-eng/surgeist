//! Test observations preserve the distinction between actual boxes and non-box metadata.
use super::layout as l;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Box,
    NonBox(l::NonBoxKind),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Observation<S: l::LayoutScalar> {
    pub kind: Kind,
    pub box_output: Option<l::NodeOutputOf<S>>,
    pub named_baselines: l::NamedBaselinesOf<S>,
    pub source_index: l::SourceIndex,
    pub location: l::Point<S>,
    pub size: l::Size<S>,
    pub content_size: l::Size<S>,
    pub scroll_geometry: Option<l::ScrollGeometryOf<S>>,
    pub border: l::Edges<S>,
    pub padding: l::Edges<S>,
    pub margin: l::Edges<S>,
}

impl<S: l::LayoutScalar> From<l::NodeOutputOf<S>> for Observation<S> {
    fn from(output: l::NodeOutputOf<S>) -> Self {
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

pub(crate) fn observe<Node: Copy + Eq, S: l::LayoutScalar>(
    phase: (&l::CompletedLayoutBatchOf<Node, S>, bool),
    node: Node,
    expected_kind: Kind,
) -> Observation<S> {
    let view = phase
        .0
        .node_layout(node)
        .expect("fixture node is published");
    let actual_kind = match view {
        l::NodeLayoutRefOf::SingleBox(_) => Kind::Box,
        l::NodeLayoutRefOf::NonBox(view) => Kind::NonBox(view.kind()),
        l::NodeLayoutRefOf::FragmentedBox(_) => {
            panic!("continuous fixture unexpectedly fragmented")
        }
    };
    assert_eq!(
        actual_kind, expected_kind,
        "fixture publication classification"
    );
    match view {
        l::NodeLayoutRefOf::SingleBox(view) => if phase.1 {
            view.final_output()
        } else {
            view.unrounded_output()
        }
        .into(),
        l::NodeLayoutRefOf::NonBox(view) => {
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
        l::NodeLayoutRefOf::FragmentedBox(_) => {
            panic!("continuous fixture unexpectedly fragmented")
        }
    }
}

/// Select a readback phase without erasing node classification.
pub(crate) fn phase<Node, S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<Node, S>,
    final_phase: bool,
) -> (&l::CompletedLayoutBatchOf<Node, S>, bool) {
    (batch, final_phase)
}

/// Authored normalized fixture input supplies the expected publication role.
/// Ancestor suppression is supplied by the owning fixture traversal.
pub(crate) fn expected_kind<S: l::LayoutScalar>(
    input: l::LayoutInputViewOf<'_, S>,
    suppressed: bool,
) -> Kind {
    if suppressed {
        return Kind::NonBox(l::NonBoxKind::Hidden);
    }
    match input {
        l::LayoutInputViewOf::Box(input) if input.display == l::Display::None => {
            Kind::NonBox(l::NonBoxKind::Hidden)
        }
        l::LayoutInputViewOf::Box(_) => Kind::Box,
        l::LayoutInputViewOf::InlineText(_) => Kind::NonBox(l::NonBoxKind::InlineText),
        l::LayoutInputViewOf::TextControl(_) => Kind::NonBox(l::NonBoxKind::TextControl),
        l::LayoutInputViewOf::LineBreak(_) => Kind::NonBox(l::NonBoxKind::LineBreak),
        l::LayoutInputViewOf::InlineBoundary(_) => Kind::NonBox(l::NonBoxKind::InlineBoundary),
    }
}
