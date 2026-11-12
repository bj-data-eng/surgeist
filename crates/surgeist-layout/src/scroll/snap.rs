use core::cmp::Ordering;

use super::snap_distance::{SnapDistance, SnapRounding};
use super::{
    DefaultScalar, FlowAxes, LayoutScalar, PhysicalAxis, PhysicalScrollAxisRangeOf,
    PhysicalScrollOffsetOf, PhysicalScrollRangeOf, Point, ScrollCoordinateErrorOf,
    ScrollRectErrorOf, ScrollRectOf, ScrollTargetGeometryOf, Size, canonical_zero,
    validate_physical_scroll_range,
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
        let border_origin = transformed_border.origin();
        let origin = Point::new(
            border_origin.x - margin.left(),
            border_origin.y - margin.top(),
        );
        let size = transformed_border.size();
        let width = SnapDistance::sum(&[size.width, margin.left(), margin.right()], &[]);
        let height = SnapDistance::sum(&[size.height, margin.top(), margin.bottom()], &[]);
        if width.sign() == Ordering::Less || height.sign() == Ordering::Less {
            ScrollRectOf::validate_origin(origin)?;
            return Ok(None);
        }
        ScrollRectOf::try_new(
            origin,
            Size::new(
                width.rounded(SnapRounding::Nearest),
                height.rounded(SnapRounding::Nearest),
            ),
        )
        .map(Some)
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
            let area = RectAxis::new(area, axis);
            let port = RectAxis::new(container.snapport, axis);
            let origins = [area.start, -port.start];
            let extents = [area.extent, -port.extent];
            let distance = match alignment {
                ScrollSnapAlignValue::None => return None,
                ScrollSnapAlignValue::Center => SnapDistance::sum(&origins, &extents),
                ScrollSnapAlignValue::Start | ScrollSnapAlignValue::End => {
                    if (alignment == ScrollSnapAlignValue::End) ^ decreasing {
                        area.end_relative_to(port)
                    } else {
                        SnapDistance::sum(&origins, &[])
                    }
                }
            };
            let offset = distance.rounded(SnapRounding::Nearest);
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
            if SnapDistance::sum(&[subsequent, -previous, -extent], &[]).sign() != Ordering::Greater
            {
                return Ok(None);
            }
        }

        let area = RectAxis::new(self.area, axis);
        let port = RectAxis::new(snapport, axis);
        let reachable = self.container.reachable_axis(axis);
        // Round toward the interior so every returned scalar offset covers the port.
        let start: S = SnapDistance::sum(&[area.start, -port.start], &[]).rounded(SnapRounding::Up);
        let end: S = area.end_relative_to(port).rounded(SnapRounding::Down);
        let minimum = start
            .max(reachable.minimum())
            .max(previous.unwrap_or(reachable.minimum()));
        let maximum = end
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
            let area = RectAxis::new(self.area, axis);
            let port = RectAxis::new(self.container.snapport, axis);
            SnapDistance::sum(&[port.start, port.extent, offset, -area.start], &[]).sign()
                != Ordering::Less
                && SnapDistance::sum(&[area.start, area.extent, -offset, -port.start], &[]).sign()
                    != Ordering::Less
        })
    }
}

fn rect_extent<S: LayoutScalar>(rect: ScrollRectOf<S>, axis: PhysicalAxis) -> S {
    match axis {
        PhysicalAxis::Horizontal => rect.size().width,
        PhysicalAxis::Vertical => rect.size().height,
    }
}

struct RectAxis<S> {
    start: S,
    extent: S,
}

impl<S: LayoutScalar> RectAxis<S> {
    fn new(rect: ScrollRectOf<S>, axis: PhysicalAxis) -> Self {
        let origin = rect.origin();
        Self {
            start: match axis {
                PhysicalAxis::Horizontal => origin.x,
                PhysicalAxis::Vertical => origin.y,
            },
            extent: rect_extent(rect, axis),
        }
    }

    fn end_relative_to(self, other: Self) -> SnapDistance {
        SnapDistance::sum(&[self.start, -other.start, self.extent, -other.extent], &[])
    }
}
