use super::{
    ComputedOverflow, DefaultScalar, Edges, FlowAxes, LayoutScalar, LengthPercentageOf,
    LogicalAxis, NumericResolutionOf, Overflow, OverflowClipBox, PercentageBasisOf, PhysicalAxis,
    PhysicalSide, Point, ScrollbarGutter, ScrollbarWidthOf, Size,
    scalar::{canonical_zero, round_layout_coordinate},
};
use crate::geometry::LogicalEdgesOf;
pub(crate) use construction::include_scope_overflow;

mod anchor_selection;
mod box_geometry;
mod construction;
mod contribution;
mod input;
mod model;
pub(crate) mod rounding;
mod snap;
mod snap_distance;

#[cfg(test)]
mod snap_tests;

use box_geometry::{AutoScrollbarOverflowObservation, scroll_rect_axis_interval, used_overflow_at};
pub(crate) use box_geometry::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, ClipMarginSourceOf,
    MeasuredLeafContentBoxInsetSourceOf, OptimalRegionInsetOf, OptimalRegionInsetsOf,
    SettledAutoScrollbarState, UsedOverflow, measured_leaf_content_box_inset,
    scrollbar_size_from_overflow,
};
#[cfg(test)]
pub(crate) use box_geometry::{UsedOverflowAxis, UsedOverflowGutter};

use contribution::{FinalInFlowEndOf, PhysicalContributionBoundsOf, PhysicalFinalInFlowEndsOf};
pub(crate) use contribution::{
    OptionalPhysicalContributionIntervalsOf, PhysicalContributionIntervalOf,
    ScrollContributionAccumulatorOf, ScrollContributionErrorOf, ScrollOriginAxes,
    ScrollOriginProgression,
};

pub(crate) use input::{ScrollBoxProjection, ScrollTargetProjection};

use construction::CanonicalScrollRectFact;
pub(crate) use construction::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollGeometryErrorOf,
    CanonicalScrollGeometrySourceOf, CanonicalScrollRangeSeedPolicy,
    CanonicalScrollSourceBuilderOf, MeasuredLeafScrollGeometrySourceOf,
    canonical_measured_leaf_scroll_geometry, canonical_scroll_box_from_source,
    canonical_scroll_geometry_from_source, rebuild_canonical_scroll_geometry_for_border_box,
    settled_auto_scrollbars_change_available_geometry,
};

pub use anchor_selection::{
    ScrollAnchorBox, ScrollAnchorBoxOf, ScrollAnchorContainingBlock, ScrollAnchorNode,
    ScrollAnchorNodeOf, ScrollAnchorPolicy, ScrollAnchorPosition, ScrollAnchorPriority,
    ScrollAnchorRequest, ScrollAnchorRequestOf, ScrollAnchorSelectionError, ScrollAnchorSubject,
    ScrollAnchorSubjectOf, select_scroll_anchor,
};
use model::validate_physical_scroll_range;
pub use model::{
    FlowRelativeScrollAxisRange, FlowRelativeScrollAxisRangeOf, FlowRelativeScrollOffset,
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRange, FlowRelativeScrollRangeOf, OverflowClip,
    OverflowClipOf, PhysicalClipAxis, PhysicalClipAxisOf, PhysicalScrollAxisRange,
    PhysicalScrollAxisRangeOf, PhysicalScrollOffset, PhysicalScrollOffsetOf, PhysicalScrollRange,
    PhysicalScrollRangeOf, ScrollCoordinateError, ScrollCoordinateErrorOf, ScrollGeometry,
    ScrollGeometryOf, ScrollRect, ScrollRectError, ScrollRectErrorOf, ScrollRectOf,
    ScrollTargetGeometry, ScrollTargetGeometryOf, ScrollbarGutterRects, ScrollbarGutterRectsOf,
};

pub use snap::{ScrollSnapContext, ScrollSnapContextOf, ScrollSnapGeometry, ScrollSnapGeometryOf};
