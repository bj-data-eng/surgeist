//! Grid-axis track sizes of a Grid Lanes or Grid container read back from
//! `compute_layout`, for independent expectations about intrinsic track sizing.
//!
//! One zero-size observer item per track is placed at that track; its stretched
//! width is the track size. Items under test are measured leaves with a
//! min-content and a max-content width.

use crate::test_support::readback::{self, Phase};
use crate::{
    AvailableSpaceOf, AxisSizingRequestOf, Direction, Display, FlowAxes, GridAxisKind,
    GridLanesFlow, GridPlacement, LayoutInputOf, LayoutInputViewOf, LayoutRootRequestOf,
    LayoutScalar, LayoutTree, LengthOf, MinSizeOf, NearestScrollportOf, NodeInputOf,
    NonNegativeFiniteOf, OrthogonalFallbackLimitOf, OrthogonalFallbackOf, PreferredSizeOf, Size,
    SizingOperation, SizingRequestOf, TrackSizingOf, Traverse, WritingMode, compute_layout,
};

/// An item with pre-resolved min-content and max-content widths.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TrackItem {
    /// Zero-based first track and span for a definite item, `None` for an
    /// automatically placed item of `span` tracks.
    pub(crate) definite_start: Option<usize>,
    pub(crate) span: usize,
    /// Specified `min-width` in px, or the automatic minimum size.
    pub(crate) min_size: Option<f64>,
    pub(crate) min_content: f64,
    pub(crate) max_content: f64,
}

impl TrackItem {
    pub(crate) const fn definite(
        start: usize,
        span: usize,
        min_content: f64,
        max_content: f64,
    ) -> Self {
        Self {
            definite_start: Some(start),
            span,
            min_size: Some(0.0),
            min_content,
            max_content,
        }
    }

    pub(crate) const fn auto(span: usize, min_content: f64, max_content: f64) -> Self {
        Self {
            definite_start: None,
            span,
            min_size: Some(0.0),
            min_content,
            max_content,
        }
    }

    pub(crate) const fn with_min_size(mut self, min_size: f64) -> Self {
        self.min_size = Some(min_size);
        self
    }

    /// The item has the automatic minimum size (`min-width: auto`).
    pub(crate) const fn with_automatic_minimum(mut self) -> Self {
        self.min_size = None;
        self
    }
}

/// Width of the container.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ContainerWidth {
    /// A definite px width; tracks keep their base sizes when it is zero.
    Definite(f64),
    /// The container is sized to its max-content width.
    MaxContent,
}

struct SizesTree<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
    children: Vec<usize>,
    measures: Vec<Option<(S, S)>>,
}

impl<S: LayoutScalar> Traverse for SizesTree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a> = core::iter::Copied<core::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &self.children } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.children.len() } else { 0 }
    }

    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl<S: LayoutScalar> LayoutTree for SizesTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }

    fn has_leaf_measurement(&self, node: usize) -> bool {
        self.measures[node].is_some()
    }

    fn measure_leaf(
        &self,
        node: usize,
        input: crate::LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        let (min, max) = self.measures[node]?;
        let request = input.content_sizing_request().axes().width;
        let width = match request.operation() {
            SizingOperation::MinContent => min,
            SizingOperation::MaxContent => max,
            SizingOperation::Normal => match request.space() {
                AvailableSpaceOf::Definite(space) => max.min(space.get()).max(min),
                AvailableSpaceOf::Indefinite => max,
            },
        };
        Some(Ok(Size::new(width, S::from_f64(10.0))))
    }
}

/// Lays out a container whose column tracks are `tracks` and whose children
/// are `items` plus one observer per track, and returns the used column track
/// sizes. `display` is `Display::Grid` or `Display::GridLanes`.
pub(crate) fn column_track_sizes<S: LayoutScalar>(
    display: Display,
    width: ContainerWidth,
    tracks: Vec<TrackSizingOf<S>>,
    gap: f64,
    items: &[TrackItem],
) -> Vec<S> {
    try_column_track_sizes(display, width, tracks, gap, items).expect("valid track sizing fixture")
}

/// As `column_track_sizes`, returning the layout error of an invalid input.
pub(crate) fn try_column_track_sizes<S: LayoutScalar>(
    display: Display,
    width: ContainerWidth,
    tracks: Vec<TrackSizingOf<S>>,
    gap: f64,
    items: &[TrackItem],
) -> Result<Vec<S>, crate::LayoutErrorOf<usize, S>> {
    let px = S::from_f64;
    let track_count = tracks.len();
    let root_width = match width {
        ContainerWidth::Definite(value) => PreferredSizeOf::px(px(value)),
        ContainerWidth::MaxContent => PreferredSizeOf::MAX_CONTENT,
    };
    let mut inputs = vec![LayoutInputOf::box_input(NodeInputOf {
        display,
        grid_lanes_flow: GridLanesFlow::new(GridAxisKind::Column, false, false),
        size: Size::new(root_width, PreferredSizeOf::AUTO),
        grid_template_columns: tracks.into_iter().map(Into::into).collect(),
        gap: Size::new(LengthOf::px(px(gap)), LengthOf::ZERO),
        ..NodeInputOf::default()
    })];
    let mut measures = vec![None];
    let mut children = Vec::new();
    for item in items {
        let placement = match item.definite_start {
            Some(start) => {
                GridPlacement::try_line_span(isize::try_from(start + 1).unwrap(), item.span)
            }
            None => GridPlacement::try_span(item.span),
        }
        .expect("item placement is valid");
        inputs.push(LayoutInputOf::box_input(NodeInputOf {
            grid_column: placement,
            min_size: Size::new(
                item.min_size
                    .map_or(MinSizeOf::AUTO, |value| MinSizeOf::px(px(value))),
                MinSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        }));
        measures.push(Some((px(item.min_content), px(item.max_content))));
        children.push(inputs.len() - 1);
    }
    let first_observer = inputs.len();
    for track in 0..track_count {
        inputs.push(LayoutInputOf::box_input(NodeInputOf {
            grid_column: GridPlacement::try_line_span(isize::try_from(track + 1).unwrap(), 1)
                .unwrap(),
            ..NodeInputOf::default()
        }));
        measures.push(None);
        children.push(inputs.len() - 1);
    }
    let tree = SizesTree {
        inputs,
        children,
        measures,
    };
    let viewport = Size::new(px(10_000.0), px(10_000.0));
    let axes = viewport.map(|value| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
            SizingOperation::Normal,
        )
    });
    let definite = axes.map(|axis| axis.space().definite_size());
    let request = LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            axes,
            OrthogonalFallbackOf::new(
                definite,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        definite,
    );
    let batch = compute_layout(&tree, 0, request)?;
    Ok((0..track_count)
        .map(|track| {
            readback::single_box_output(&batch, first_observer + track, Phase::Unrounded)
                .size
                .width
        })
        .collect())
}
