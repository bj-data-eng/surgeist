//! Grid Lanes placement read back from `compute_layout` for independent
//! expectations.
//!
//! The container has equal fixed-size tracks along the grid axis, so an item's
//! grid-axis start line and span follow from its physical location and size.

use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{self, Phase};
use crate::{
    engine::compute_layout,
    geometry::{FlowAxes, Size},
    grid::GridAxisKind,
    node_input::{
        AlignContent, Direction, Display, GridFlowToleranceOf, GridLanesFlow, GridPlacement,
        NodeInputOf, WritingMode,
    },
    output::LayoutRootRequestOf,
    scalar::LayoutScalar,
    sizing::PreferredSizeOf,
    value::{
        AvailableSpaceOf, AxisSizingRequestOf, LengthOf, NearestScrollportOf, NonNegativeFiniteOf,
        OrthogonalFallbackLimitOf, OrthogonalFallbackOf, SizingOperation, SizingRequestOf,
        TrackComponentOf,
    },
};

/// Size of every grid-axis track.
const TRACK: f64 = 10.0;
/// Definite space offered along the lane axis.
const LANE_SPACE: f64 = 10_000.0;

/// One in-flow item of the fixture.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LaneFixtureItem {
    /// One-based grid-axis start line, or `None` for automatic placement.
    pub(crate) definite_start: Option<usize>,
    pub(crate) span: usize,
    /// Lane-axis margin-box size; the item has no margin.
    pub(crate) lane_size: f64,
}

impl LaneFixtureItem {
    pub(crate) const fn auto(span: usize, lane_size: f64) -> Self {
        Self {
            definite_start: None,
            span,
            lane_size,
        }
    }

    pub(crate) const fn definite(start: usize, span: usize, lane_size: f64) -> Self {
        Self {
            definite_start: Some(start),
            span,
            lane_size,
        }
    }
}

/// A Grid Lanes container over `tracks` equal fixed tracks.
#[derive(Clone, Debug)]
pub(crate) struct LaneFixture<S: LayoutScalar> {
    pub(crate) grid_axis: GridAxisKind,
    pub(crate) reversed: bool,
    pub(crate) tracks: usize,
    pub(crate) lane_gap: f64,
    pub(crate) tolerance: GridFlowToleranceOf<S>,
    pub(crate) items: Vec<LaneFixtureItem>,
}

/// What `compute_layout` published for the fixture, in item order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LaneReadback<S: LayoutScalar> {
    pub(crate) starts: Vec<usize>,
    pub(crate) spans: Vec<usize>,
    pub(crate) offsets: Vec<S>,
    /// The far edge of the last-ending item along the lane axis.
    pub(crate) content_extent: S,
}

fn physical<T>(grid_axis: GridAxisKind, grid: T, lane: T) -> Size<T> {
    match grid_axis {
        GridAxisKind::Column => Size::new(grid, lane),
        GridAxisKind::Row => Size::new(lane, grid),
    }
}

fn lane_request<S: LayoutScalar>(viewport: Size<S>) -> LayoutRootRequestOf<S> {
    let axes = viewport.map(|value| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
            SizingOperation::Normal,
        )
    });
    let definite = axes.map(|axis| axis.space().definite_size());
    LayoutRootRequestOf::standalone(
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
    )
}

impl<S: LayoutScalar> LaneFixture<S> {
    /// Lays the fixture out through `compute_layout` and reads the placement
    /// of each item back from its final physical geometry.
    pub(crate) fn layout(&self) -> LaneReadback<S> {
        let px = S::from_f64;
        let columns = self.grid_axis == GridAxisKind::Column;
        let tracks = vec![TrackComponentOf::px(px(TRACK)); self.tracks];
        let grid_extent = px(TRACK * self.tracks as f64);
        let root = NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: GridLanesFlow::new(self.grid_axis, self.reversed, false),
            size: physical(
                self.grid_axis,
                PreferredSizeOf::px(grid_extent),
                PreferredSizeOf::AUTO,
            ),
            grid_template_columns: if columns { tracks.clone() } else { Vec::new() },
            grid_template_rows: if columns { Vec::new() } else { tracks },
            gap: physical(
                self.grid_axis,
                LengthOf::ZERO,
                LengthOf::px(px(self.lane_gap)),
            ),
            grid_flow_tolerance: self.tolerance,
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        };
        let mut tree = PublicLayoutTreeOf::new()
            .children(1, (0..self.items.len() as u32).map(|item| item + 2))
            .style(1, root);
        for (node, item) in (2..).zip(&self.items) {
            let placement = match item.definite_start {
                Some(start) => {
                    GridPlacement::try_line_span(isize::try_from(start).unwrap(), item.span)
                }
                None => GridPlacement::try_span(item.span),
            }
            .expect("fixture placement is valid");
            tree = tree.style(
                node,
                NodeInputOf {
                    grid_column: if columns {
                        placement
                    } else {
                        GridPlacement::AUTO
                    },
                    grid_row: if columns {
                        GridPlacement::AUTO
                    } else {
                        placement
                    },
                    size: physical(
                        self.grid_axis,
                        PreferredSizeOf::AUTO,
                        PreferredSizeOf::px(px(item.lane_size)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
        let batch = compute_layout(
            &tree,
            1,
            lane_request(physical(self.grid_axis, grid_extent, px(LANE_SPACE))),
        )
        .expect("valid Grid Lanes fixture");
        let mut readback = LaneReadback {
            starts: Vec::new(),
            spans: Vec::new(),
            offsets: Vec::new(),
            content_extent: S::ZERO,
        };
        for node in (0..self.items.len() as u32).map(|item| item + 2) {
            let output = readback::single_box_output(&batch, node, Phase::Final);
            let (grid_start, grid_size, lane_start) = match self.grid_axis {
                GridAxisKind::Column => (output.location.x, output.size.width, output.location.y),
                GridAxisKind::Row => (output.location.y, output.size.height, output.location.x),
            };
            readback
                .starts
                .push((grid_start.to_f64() / TRACK).round() as usize + 1);
            readback
                .spans
                .push((grid_size.to_f64() / TRACK).round() as usize);
            readback.offsets.push(lane_start);
        }
        // The far edge of the last-ending item along the lane axis. The
        // container's own content size also counts the trailing lane gap.
        readback.content_extent = readback
            .offsets
            .iter()
            .zip(&self.items)
            .map(|(offset, item)| *offset + px(item.lane_size))
            .fold(S::ZERO, S::max);
        readback
    }
}
