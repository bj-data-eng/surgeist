//! Native fixture viewport facts, separate from an authored root size or operation.
use surgeist::layout;

#[derive(Clone, Copy)]
pub(super) struct NativeViewport {
    physical_extent: layout::Size<Option<layout::NonNegativeFiniteOf<f32>>>,
    containing_axes: layout::FlowAxes,
}

impl NativeViewport {
    pub(super) fn new(
        physical_extent: layout::Size<Option<f32>>,
        containing_axes: layout::FlowAxes,
    ) -> Self {
        Self {
            physical_extent: physical_extent.map(|extent| {
                extent.map(|value| {
                    layout::NonNegativeFiniteOf::new(value).expect(
                        "a declared native fixture viewport extent is finite and nonnegative",
                    )
                })
            }),
            containing_axes,
        }
    }

    pub(super) fn request(
        self,
        operations: layout::Size<layout::SizingOperation>,
    ) -> layout::LayoutRootRequest {
        let spaces = self.physical_extent.map(|extent| match extent {
            Some(value) => layout::AvailableSpaceOf::Definite(value),
            None => layout::AvailableSpaceOf::Indefinite,
        });
        // This value declares the actual native ICB. Its known inner dimensions
        // supply percentage bases and eligible orthogonal fallback, independently
        // of sizing operations. There is no ancestor scrollport or constrained
        // generated containing box. No outside formatting parent allocates the root.
        layout::LayoutRootRequest::standalone(
            layout::SizingRequest::new(
                layout::Size::new(
                    layout::AxisSizingRequest::new(spaces.width, operations.width),
                    layout::AxisSizingRequest::new(spaces.height, operations.height),
                ),
                layout::OrthogonalFallbackOf::new(
                    self.physical_extent,
                    layout::NearestScrollportOf::Absent,
                    layout::Size::splat(layout::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            self.containing_axes,
            self.physical_extent,
        )
    }
}
