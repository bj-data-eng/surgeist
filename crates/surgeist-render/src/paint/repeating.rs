use super::{GradientStop, Point, Result, validate_field_stops};
#[cfg(feature = "render-paint")]
use crate::accounting::{AccountedVec, BackedValue};
use crate::{
    accounting::{Account, Hold, RenderWork},
    validation::{validate_non_negative_f64, validate_point, validate_positive_f64},
};

/// A resolved logical-space repeating linear or elliptical radial gradient.
///
/// Stops are nonempty, finite, and nondecreasing, with a finite last-minus-first
/// span. Equal positions remain distinct; one stop represents a solid color.
/// Repetition is realized with the actual output transform and surface scale.
#[derive(Clone, Debug, PartialEq)]
pub struct RepeatingGradient {
    geometry: GradientGeometry,
    stops: Vec<GradientStop>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GradientGeometry {
    Linear {
        start: Point,
        end: Point,
    },
    Radial {
        center: Point,
        radius_x: f64,
        radius_y: f64,
    },
}

impl RepeatingGradient {
    /// Creates a repeating linear field between finite logical points.
    ///
    /// Coincident points are allowed. Returns [`crate::ErrorCode::InvalidInput`]
    /// for invalid geometry, empty or descending stops, or an overflowing span.
    pub fn try_linear(start: Point, end: Point, stops: Vec<GradientStop>) -> Result<Self> {
        Self::try_new(GradientGeometry::Linear { start, end }, stops)
    }

    /// Creates a repeating elliptical radial field at a finite logical center.
    ///
    /// The finite horizontal radius must be positive; the finite vertical radius
    /// may be zero, explicitly selecting a solid average. Returns
    /// [`crate::ErrorCode::InvalidInput`] for invalid geometry or stops.
    pub fn try_radial(
        center: Point,
        radius_x: f64,
        radius_y: f64,
        stops: Vec<GradientStop>,
    ) -> Result<Self> {
        Self::try_new(
            GradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            },
            stops,
        )
    }

    fn try_new(geometry: GradientGeometry, stops: Vec<GradientStop>) -> Result<Self> {
        let gradient = Self { geometry, stops };
        gradient.validate()?;
        Ok(gradient)
    }

    /// Returns all stops in their supplied order, including equal positions.
    #[must_use]
    pub fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    /// Returns the supplied linear start and end points, when linear.
    #[must_use]
    pub const fn linear_points(&self) -> Option<(Point, Point)> {
        match self.geometry {
            GradientGeometry::Linear { start, end } => Some((start, end)),
            GradientGeometry::Radial { .. } => None,
        }
    }

    /// Returns the supplied center and horizontal and vertical radii, when radial.
    #[must_use]
    pub const fn radial_geometry(&self) -> Option<(Point, f64, f64)> {
        match self.geometry {
            GradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            } => Some((center, radius_x, radius_y)),
            GradientGeometry::Linear { .. } => None,
        }
    }

    pub(crate) const fn geometry(&self) -> GradientGeometry {
        self.geometry
    }

    fn validate(&self) -> Result<()> {
        match self.geometry {
            GradientGeometry::Linear { start, end } => {
                validate_point(start, "repeating linear gradient start")?;
                validate_point(end, "repeating linear gradient end")?;
            }
            GradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            } => {
                validate_point(center, "repeating radial gradient center")?;
                validate_positive_f64(radius_x, "repeating radial gradient horizontal radius")?;
                validate_non_negative_f64(radius_y, "repeating radial gradient vertical radius")?;
            }
        }
        validate_field_stops(&self.stops)?;
        validate_non_negative_f64(
            self.stops
                .last()
                .expect("nonempty validated list")
                .position()
                - self.stops[0].position(),
            "repeating gradient span",
        )
    }

    #[cfg(feature = "render-paint")]
    pub(crate) fn from_accounted_paint(
        geometry: GradientGeometry,
        stops: AccountedVec<GradientStop>,
        fixed_owner: Hold,
        source_indices: (usize, usize),
        positions: (f64, f64),
        account: &Account,
    ) -> Result<BackedValue<Self>> {
        // Paint's immutable geometry and the consumer's actual quotient walk
        // prove the geometry/list domains. Only repetition's span remains.
        account.work(RenderWork::PaintVisits, 1)?;
        let (stops, backing) = stops.into_parts();
        let gradient =
            BackedValue::new(Self { geometry, stops }, [Some(backing), Some(fixed_owner)]);
        account.work(RenderWork::CoordinateVisits, 1)?;
        let span = positions.1 - positions.0;
        account.work(RenderWork::CoordinateVisits, 1)?;
        validate_non_negative_f64(span, "repeating gradient span").map_err(|_| {
            crate::Error::from(crate::PaintGradientError::UnrepresentableSpan {
                first_source_index: source_indices.0,
                last_source_index: source_indices.1,
            })
        })?;
        Ok(gradient)
    }

    pub(crate) fn try_clone_in(&self, account: &Account) -> Result<(Self, Hold)> {
        let mut stops = account.retained(self.stops.len())?;
        for stop in &self.stops {
            account.work(RenderWork::GradientStopVisits, 1)?;
            stops.push(*stop)?;
        }
        let (stops, hold) = stops.into_parts();
        Ok((
            Self {
                geometry: self.geometry,
                stops,
            },
            hold,
        ))
    }
}
