//! Explicit host facts for unit-test roots. No old availability-tag conversion.
use crate::{
    AxisSizingRequestOf, FlowAxes, LayoutRootRequestOf, LayoutScalar, NonNegativeFiniteOf,
    OrthogonalFallbackOf, Size, SizingRequestOf,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativeRootFrameOf<S: LayoutScalar> {
    containing_flow_axes: FlowAxes,
    percentage_parent: Size<Option<NonNegativeFiniteOf<S>>>,
    orthogonal_fallback: OrthogonalFallbackOf<S>,
}

impl<S: LayoutScalar> NativeRootFrameOf<S> {
    pub(crate) const fn new(
        containing_flow_axes: FlowAxes,
        percentage_parent: Size<Option<NonNegativeFiniteOf<S>>>,
        orthogonal_fallback: OrthogonalFallbackOf<S>,
    ) -> Self {
        Self {
            containing_flow_axes,
            percentage_parent,
            orthogonal_fallback,
        }
    }

    pub(crate) const fn containing_flow_axes(self) -> FlowAxes {
        self.containing_flow_axes
    }
    pub(crate) const fn percentage_parent(self) -> Size<Option<NonNegativeFiniteOf<S>>> {
        self.percentage_parent
    }
    pub(crate) const fn orthogonal_fallback(self) -> OrthogonalFallbackOf<S> {
        self.orthogonal_fallback
    }
    pub(crate) const fn sizing_request(
        self,
        axes: Size<AxisSizingRequestOf<S>>,
    ) -> SizingRequestOf<S> {
        SizingRequestOf::new(axes, self.orthogonal_fallback())
    }
    pub(crate) const fn request(
        self,
        axes: Size<AxisSizingRequestOf<S>>,
    ) -> LayoutRootRequestOf<S> {
        LayoutRootRequestOf::standalone(
            self.sizing_request(axes),
            self.containing_flow_axes,
            self.percentage_parent,
        )
    }
}
