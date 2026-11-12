mod pixel_edge;

use crate::output::GeometryStateOf;
use crate::{
    DefaultScalar, Direction, FlowAxes, LayoutScalar, PhysicalAxis, Point, SingleBoxOutputRefOf,
    Size, WritingMode,
};

/// The pre-transform device-pixel grid for an accepted CSS content box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeObserverPixelGrid {
    content_origin: Point<f64>,
    scale: f64,
}

impl ResizeObserverPixelGrid {
    /// Creates the mapping from CSS units to device pixels.
    ///
    /// Supply the physical content origin relative to the pixel grid and the
    /// effective pixels per CSS unit from the same accepted geometry revision.
    /// Include zoom and host scale once. Exclude CSS transforms.
    pub fn try_new(
        content_origin: Point<f64>,
        scale: f64,
    ) -> Result<Self, ResizeObserverMeasurementError> {
        use ResizeObserverMeasurementError as Error;
        for (axis, coordinate) in [
            (PhysicalAxis::Horizontal, content_origin.x),
            (PhysicalAxis::Vertical, content_origin.y),
        ] {
            if !coordinate.is_finite() {
                return Err(Error::NonFiniteContentOrigin { axis, coordinate });
            }
        }
        if !scale.is_finite() || scale <= 0.0 {
            return Err(Error::InvalidScale { scale });
        }
        Ok(Self {
            content_origin,
            scale,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeObserverMeasurementError {
    NonFiniteContentOrigin { axis: PhysicalAxis, coordinate: f64 },
    InvalidScale { scale: f64 },
    DevicePixelExtentOutOfRange { axis: PhysicalAxis },
}

impl core::fmt::Display for ResizeObserverMeasurementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::NonFiniteContentOrigin { axis, coordinate } => {
                write!(
                    f,
                    "nonfinite content origin {coordinate} on the {axis:?} axis"
                )
            }
            Self::InvalidScale { scale } => write!(f, "invalid device-pixel scale {scale}"),
            Self::DevicePixelExtentOutOfRange { axis } => {
                write!(f, "device-pixel extent exceeds u64 on the {axis:?} axis")
            }
        }
    }
}

impl std::error::Error for ResizeObserverMeasurementError {}

/// One logical box size. The unit is set by the measured box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResizeObserverBoxSize<T> {
    pub inline: T,
    pub block: T,
}

impl<T> ResizeObserverBoxSize<T> {
    fn from_physical(axes: FlowAxes, size: Size<T>) -> Self {
        let size = axes.logical_size(size);
        Self {
            inline: size.inline,
            block: size.block,
        }
    }
}

/// Physical content rectangle in CSS units, with a padding-relative origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeObserverContentRectOf<S: LayoutScalar = DefaultScalar> {
    pub origin: Point<S>,
    pub size: Size<S>,
}

/// The measurements of one reported CSS box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeObserverMeasurementOf<S: LayoutScalar = DefaultScalar> {
    border_box: ResizeObserverBoxSize<S>,
    content_box: ResizeObserverBoxSize<S>,
    device_pixel_content_box: ResizeObserverBoxSize<u64>,
    content_rect: ResizeObserverContentRectOf<S>,
}

impl<S: LayoutScalar> ResizeObserverMeasurementOf<S> {
    /// Accepted border-box size in logical CSS units.
    #[must_use]
    pub const fn border_box(self) -> ResizeObserverBoxSize<S> {
        self.border_box
    }

    /// Accepted content-box size in logical CSS units.
    #[must_use]
    pub const fn content_box(self) -> ResizeObserverBoxSize<S> {
        self.content_box
    }

    /// Content-box size in integral logical device pixels.
    #[must_use]
    pub const fn device_pixel_content_box(self) -> ResizeObserverBoxSize<u64> {
        self.device_pixel_content_box
    }

    /// Accepted content rectangle in physical CSS units.
    #[must_use]
    pub const fn content_rect(self) -> ResizeObserverContentRectOf<S> {
        self.content_rect
    }

    /// Returns the observation for a target known to have no observable box,
    /// such as a non-replaced inline. A missing measurement is represented by
    /// the absence of this value.
    #[must_use]
    pub const fn empty() -> Self {
        let css = ResizeObserverBoxSize {
            inline: S::ZERO,
            block: S::ZERO,
        };
        Self {
            border_box: css,
            content_box: css,
            device_pixel_content_box: ResizeObserverBoxSize {
                inline: 0,
                block: 0,
            },
            content_rect: ResizeObserverContentRectOf {
                origin: Point::ZERO,
                size: Size::ZERO,
            },
        }
    }
}

impl<Node: Copy, S: LayoutScalar> SingleBoxOutputRefOf<'_, Node, S> {
    /// Measures the accepted CSS box using the target writing mode and grid.
    ///
    /// The consumer determines whether this is an observable CSS box.
    /// `Ok(None)` reports unavailable measured geometry. Each reported box
    /// size describes one box. Device pixels use nearest-edge rounding with
    /// ties toward positive infinity.
    pub fn resize_observer_measurement(
        &self,
        writing_mode: WritingMode,
        pixel_grid: ResizeObserverPixelGrid,
    ) -> Result<Option<ResizeObserverMeasurementOf<S>>, ResizeObserverMeasurementError> {
        let entry = self.unrounded_entry();
        let (measured, padding_origin) = match entry.geometry_state() {
            GeometryStateOf::OrdinaryUnrounded { measured_normal } => {
                let padding = entry.output().padding;
                (measured_normal, Point::new(padding.left, padding.top))
            }
            GeometryStateOf::CanonicalAtomic {
                measured_normal,
                measured_padding_origin,
                ..
            } => (measured_normal, measured_padding_origin),
            GeometryStateOf::OrdinaryCanonical { .. } => return Ok(None),
        };
        let Some(content) = measured.content_box() else {
            return Ok(None);
        };
        let content_size = content.size();
        let extent = |axis, origin, size: S| {
            pixel_edge::pixel_extent(origin, size.to_f64(), pixel_grid.scale)
                .ok_or(ResizeObserverMeasurementError::DevicePixelExtentOutOfRange { axis })
        };
        let pixels = Size::new(
            extent(
                PhysicalAxis::Horizontal,
                pixel_grid.content_origin.x,
                content_size.width,
            )?,
            extent(
                PhysicalAxis::Vertical,
                pixel_grid.content_origin.y,
                content_size.height,
            )?,
        );
        let axes = FlowAxes::new(writing_mode, Direction::Ltr);
        Ok(Some(ResizeObserverMeasurementOf {
            border_box: ResizeObserverBoxSize::from_physical(axes, measured.size),
            content_box: ResizeObserverBoxSize::from_physical(axes, content_size),
            device_pixel_content_box: ResizeObserverBoxSize::from_physical(axes, pixels),
            content_rect: ResizeObserverContentRectOf {
                origin: padding_origin,
                size: content_size,
            },
        }))
    }
}

#[cfg(test)]
mod tests;
