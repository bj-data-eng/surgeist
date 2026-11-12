use super::{LayoutScalar, Point, ScrollRectErrorOf, ScrollRectOf, ScrollTargetGeometryOf, Size};

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
