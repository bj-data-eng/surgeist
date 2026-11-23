//! Native fixture viewport facts, separate from an authored root size or operation.
use surgeist::layout;

#[derive(Clone, Copy)]
pub(super) struct NativeViewport {
    physical_extent: layout::geometry::Size<Option<layout::length::NonNegativeFiniteOf<f32>>>,
    containing_axes: layout::geometry::FlowAxes,
}

impl NativeViewport {
    pub(super) fn new(
        physical_extent: layout::geometry::Size<Option<f32>>,
        containing_axes: layout::geometry::FlowAxes,
    ) -> Self {
        Self {
            physical_extent: physical_extent.map(|extent| {
                extent.map(|value| {
                    layout::length::NonNegativeFiniteOf::new(value).expect(
                        "a declared native fixture viewport extent is finite and nonnegative",
                    )
                })
            }),
            containing_axes,
        }
    }

    pub(super) fn request(
        self,
        operations: layout::geometry::Size<layout::constraint::SizingOperation>,
    ) -> layout::request::LayoutRootRequest {
        let spaces = self.physical_extent.map(|extent| match extent {
            Some(value) => layout::constraint::AvailableSpaceOf::Definite(value),
            None => layout::constraint::AvailableSpaceOf::Indefinite,
        });
        // This value declares the actual native ICB. Its known inner dimensions
        // supply percentage bases and eligible orthogonal fallback, independently
        // of sizing operations. There is no ancestor scrollport or constrained
        // generated containing box. No outside formatting parent allocates the root.
        layout::request::LayoutRootRequest::standalone(
            layout::constraint::SizingRequest::new(
                layout::geometry::Size::new(
                    layout::constraint::AxisSizingRequest::new(spaces.width, operations.width),
                    layout::constraint::AxisSizingRequest::new(spaces.height, operations.height),
                ),
                layout::constraint::OrthogonalFallbackOf::new(
                    self.physical_extent,
                    layout::constraint::NearestScrollportOf::Absent,
                    layout::geometry::Size::splat(
                        layout::constraint::OrthogonalFallbackLimitOf::Unbounded,
                    ),
                ),
            ),
            self.containing_axes,
            self.physical_extent,
        )
    }
}
