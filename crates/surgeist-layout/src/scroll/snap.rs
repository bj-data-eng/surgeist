use super::{
    DefaultScalar, FlowAxes, LayoutScalar, PhysicalAxis, PhysicalScrollAxisRangeOf,
    PhysicalScrollOffsetOf, PhysicalScrollRangeOf, Point, ScrollCoordinateErrorOf,
    ScrollRectErrorOf, ScrollRectOf, ScrollTargetGeometryOf, Size, canonical_zero,
    scroll_rect_axis_interval, validate_physical_scroll_range,
};
use crate::{ScrollSnapAlignValue, ScrollSnapAxis};

impl<S: LayoutScalar> ScrollTargetGeometryOf<S> {
    /// Applies this target's signed scroll margins to its transformed border
    /// bounds in the scroller's physical coordinate frame.
    ///
    /// An inverted interval is empty. Equal edges retain a zero extent.
    /// This operation also supplies the area for scroll-into-view when
    /// snapping is disabled.
    pub fn snap_area_in_scroller(
        self,
        transformed_border: ScrollRectOf<S>,
    ) -> Result<Option<ScrollRectOf<S>>, ScrollRectErrorOf<S>> {
        let margin = self.scroll_margin();
        let origin = transformed_border.origin();
        let size = transformed_border.size();
        let area = ScrollRectOf::try_new(
            Point::new(origin.x - margin.left(), origin.y - margin.top()),
            Size::new(
                expanded_extent(size.width, margin.left(), margin.right()),
                expanded_extent(size.height, margin.top(), margin.bottom()),
            ),
        );
        match area {
            Err(ScrollRectErrorOf::NegativeSize { .. }) => Ok(None),
            result => result.map(Some),
        }
    }
}

fn expanded_extent<S: LayoutScalar>(extent: S, start: S, end: S) -> S {
    let outsets = start + end;
    if outsets.is_finite() {
        extent + outsets
    } else {
        // A positive extent can offset two negative outsets whose sum overflows.
        (extent + start) + end
    }
}

/// Snapport, signed reachable offsets, and active axes in one scroller frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollSnapContextOf<S: LayoutScalar = DefaultScalar> {
    snapport: ScrollRectOf<S>,
    reachable: PhysicalScrollRangeOf<S>,
    flow_axes: FlowAxes,
    snap_axis: ScrollSnapAxis,
}

pub type ScrollSnapContext = ScrollSnapContextOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollSnapContextOf<S> {
    /// The snapport is expressed at zero offset in the common scroller frame.
    #[must_use]
    pub const fn new(
        snapport: ScrollRectOf<S>,
        reachable: PhysicalScrollRangeOf<S>,
        flow_axes: FlowAxes,
        snap_axis: ScrollSnapAxis,
    ) -> Self {
        Self {
            snapport,
            reachable,
            flow_axes,
            snap_axis,
        }
    }

    fn includes_axis(self, axis: PhysicalAxis) -> bool {
        match self.snap_axis {
            ScrollSnapAxis::X => axis == PhysicalAxis::Horizontal,
            ScrollSnapAxis::Y => axis == PhysicalAxis::Vertical,
            ScrollSnapAxis::Block => axis == self.flow_axes.block_axis(),
            ScrollSnapAxis::Inline => axis == self.flow_axes.inline_axis(),
            ScrollSnapAxis::Both => true,
        }
    }

    fn reachable_axis(self, axis: PhysicalAxis) -> PhysicalScrollAxisRangeOf<S> {
        match axis {
            PhysicalAxis::Horizontal => self.reachable.x(),
            PhysicalAxis::Vertical => self.reachable.y(),
        }
    }
}

/// A target's snap geometry with independent candidate offsets for both axes.
/// Visibility is evaluated at the combined destination.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollSnapGeometryOf<S: LayoutScalar = DefaultScalar> {
    area: ScrollRectOf<S>,
    container: ScrollSnapContextOf<S>,
    aligned: Size<Option<S>>,
}

pub type ScrollSnapGeometry = ScrollSnapGeometryOf<DefaultScalar>;

impl<S: LayoutScalar> ScrollTargetGeometryOf<S> {
    /// Derives the target's reachable alignment offsets using bounds and
    /// snapport expressed in the same physical frame.
    pub fn snap_geometry_in_scroller(
        self,
        transformed_border: ScrollRectOf<S>,
        container: ScrollSnapContextOf<S>,
    ) -> Result<Option<ScrollSnapGeometryOf<S>>, ScrollRectErrorOf<S>> {
        let Some(area) = self.snap_area_in_scroller(transformed_border)? else {
            return Ok(None);
        };
        let target_inline = self.flow_axes().inline_axis();
        let edge_axes =
            if rect_extent(area, target_inline) > rect_extent(container.snapport, target_inline) {
                self.flow_axes()
            } else {
                container.flow_axes
            };
        let aligned_offset = |axis| {
            if !container.includes_axis(axis) {
                return None;
            }
            let alignment = if container.flow_axes.inline_axis() == axis {
                self.snap_align().inline()
            } else {
                self.snap_align().block()
            };
            let decreasing = edge_axes.physical_axis_progression(axis).is_decreasing();
            let anchor = |rect| alignment_anchor(rect, axis, alignment, decreasing);
            let offset = anchor(area)? - anchor(container.snapport)?;
            Some(canonical_zero(container.reachable_axis(axis).clamp(offset)))
        };
        Ok(Some(ScrollSnapGeometryOf {
            area,
            container,
            aligned: Size::new(
                aligned_offset(PhysicalAxis::Horizontal),
                aligned_offset(PhysicalAxis::Vertical),
            ),
        }))
    }
}

impl<S: LayoutScalar> ScrollSnapGeometryOf<S> {
    #[must_use]
    pub const fn area(self) -> ScrollRectOf<S> {
        self.area
    }

    /// Returns the alignment offset after clamping to the reachable range.
    #[must_use]
    pub const fn aligned_offset(self, axis: PhysicalAxis) -> Option<S> {
        match axis {
            PhysicalAxis::Horizontal => self.aligned.width,
            PhysicalAxis::Vertical => self.aligned.height,
        }
    }

    /// Returns the reachable portion of an adjacent gap where the target
    /// covers the snapport. An absent neighbor leaves that end of the gap open.
    pub fn covering_offsets(
        self,
        axis: PhysicalAxis,
        previous: Option<S>,
        subsequent: Option<S>,
    ) -> Result<Option<PhysicalScrollAxisRangeOf<S>>, ScrollCoordinateErrorOf<S>> {
        let snapport = self.container.snapport;
        let extent = rect_extent(snapport, axis);
        if self.aligned_offset(axis).is_none() || rect_extent(self.area, axis) <= extent {
            return Ok(None);
        }

        if let Some(value) = previous
            && !value.is_finite()
        {
            return Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMinimum { axis, value });
        }
        if let Some(value) = subsequent
            && !value.is_finite()
        {
            return Err(ScrollCoordinateErrorOf::NonFinitePhysicalRangeMaximum { axis, value });
        }
        if let (Some(previous), Some(subsequent)) = (previous, subsequent) {
            validate_physical_scroll_range(axis, previous, subsequent)?;
            if subsequent - previous <= extent {
                return Ok(None);
            }
        }

        let (area_start, area_end) = scroll_rect_axis_interval(self.area, axis);
        let (port_start, port_end) = scroll_rect_axis_interval(snapport, axis);
        let reachable = self.container.reachable_axis(axis);
        let minimum = (area_start - port_start)
            .max(reachable.minimum())
            .max(previous.unwrap_or(reachable.minimum()));
        let maximum = (area_end - port_end)
            .min(reachable.maximum())
            .min(subsequent.unwrap_or(reachable.maximum()));
        Ok((minimum <= maximum).then(|| {
            PhysicalScrollAxisRangeOf::new(canonical_zero(minimum), canonical_zero(maximum))
        }))
    }

    /// Tests the target against the snapport at the final two-axis offset.
    /// Shared boundary coordinates count as visible.
    #[must_use]
    pub fn is_visible_at(self, offset: PhysicalScrollOffsetOf<S>) -> bool {
        [
            (PhysicalAxis::Horizontal, offset.x()),
            (PhysicalAxis::Vertical, offset.y()),
        ]
        .into_iter()
        .all(|(axis, offset)| {
            let (area_start, area_end) = scroll_rect_axis_interval(self.area, axis);
            let (port_start, port_end) = scroll_rect_axis_interval(self.container.snapport, axis);
            area_start <= port_end + offset && area_end >= port_start + offset
        })
    }
}

fn rect_extent<S: LayoutScalar>(rect: ScrollRectOf<S>, axis: PhysicalAxis) -> S {
    match axis {
        PhysicalAxis::Horizontal => rect.size().width,
        PhysicalAxis::Vertical => rect.size().height,
    }
}

fn alignment_anchor<S: LayoutScalar>(
    rect: ScrollRectOf<S>,
    axis: PhysicalAxis,
    alignment: ScrollSnapAlignValue,
    decreasing: bool,
) -> Option<S> {
    let (minimum, maximum) = scroll_rect_axis_interval(rect, axis);
    match alignment {
        ScrollSnapAlignValue::None => None,
        ScrollSnapAlignValue::Center => Some(minimum + rect_extent(rect, axis) * S::from_f64(0.5)),
        ScrollSnapAlignValue::Start | ScrollSnapAlignValue::End => {
            Some(if (alignment == ScrollSnapAlignValue::End) ^ decreasing {
                maximum
            } else {
                minimum
            })
        }
    }
}
