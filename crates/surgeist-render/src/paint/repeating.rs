use super::{Color, Error, Point, Result};
#[cfg(feature = "render-paint")]
use crate::accounting::{AccountedVec, BackedValue};
use crate::{
    accounting::{Account, Hold, RenderWork},
    validation::{
        validate_color, validate_finite_f64, validate_non_negative_f64, validate_point,
        validate_positive_f64,
    },
};

#[cfg(feature = "render-paint")]
fn validation_coordinates(account: Option<&Account>, amount: usize) -> Result<()> {
    // The new consumer counts every scalar-domain inspection in its canonical
    // validation walk. Existing frame/glyph units and their oracles stay intact.
    if let Some(account) = account
        && account.is_paint_gradient()
    {
        account.work(RenderWork::CoordinateVisits, amount)?;
    }
    Ok(())
}

/// A resolved repeating-gradient stop, positioned in units of its supplied field.
///
/// Positions are finite `f64` values, including negative and outside-unit values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RepeatingGradientStop {
    position: f64,
    color: Color,
}

impl RepeatingGradientStop {
    #[cfg(feature = "render-paint")]
    pub(crate) const fn from_qualified_paint(position: f64, color: Color) -> Self {
        // The consumer proved the position; Color already owns its invariant.
        // The complete canonical constructor still validates the owned records.
        Self { position, color }
    }
    /// Creates a stop without clamping its position.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for a nonfinite position.
    pub fn try_new(position: f64, color: Color) -> Result<Self> {
        validate_finite_f64(position, "repeating gradient stop position")?;
        validate_color(color, "repeating gradient stop color")?;
        Ok(Self { position, color })
    }

    /// Returns the supplied position in field units.
    #[must_use]
    pub const fn position(self) -> f64 {
        self.position
    }

    /// Returns the supplied numeric-sRGB color.
    #[must_use]
    pub const fn color(self) -> Color {
        self.color
    }
}

/// A resolved logical-space repeating linear or elliptical radial gradient.
///
/// Stops are nonempty, finite, and nondecreasing, with a finite last-minus-first
/// span. Equal positions remain distinct; one stop represents a solid color.
/// Repetition is realized with the actual output transform and surface scale.
#[derive(Clone, Debug, PartialEq)]
pub struct RepeatingGradient {
    geometry: RepeatingGradientGeometry,
    stops: Vec<RepeatingGradientStop>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RepeatingGradientGeometry {
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
    pub fn try_linear(start: Point, end: Point, stops: Vec<RepeatingGradientStop>) -> Result<Self> {
        Self::try_new(RepeatingGradientGeometry::Linear { start, end }, stops)
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
        stops: Vec<RepeatingGradientStop>,
    ) -> Result<Self> {
        Self::try_new(
            RepeatingGradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            },
            stops,
        )
    }

    fn try_new(
        geometry: RepeatingGradientGeometry,
        stops: Vec<RepeatingGradientStop>,
    ) -> Result<Self> {
        let gradient = Self { geometry, stops };
        gradient.validate_in(None)?;
        Ok(gradient)
    }

    /// Returns all stops in their supplied order, including equal positions.
    #[must_use]
    pub fn stops(&self) -> &[RepeatingGradientStop] {
        &self.stops
    }

    /// Returns the supplied linear start and end points, when linear.
    #[must_use]
    pub const fn linear_points(&self) -> Option<(Point, Point)> {
        match self.geometry {
            RepeatingGradientGeometry::Linear { start, end } => Some((start, end)),
            RepeatingGradientGeometry::Radial { .. } => None,
        }
    }

    /// Returns the supplied center and horizontal and vertical radii, when radial.
    #[must_use]
    pub const fn radial_geometry(&self) -> Option<(Point, f64, f64)> {
        match self.geometry {
            RepeatingGradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            } => Some((center, radius_x, radius_y)),
            RepeatingGradientGeometry::Linear { .. } => None,
        }
    }

    pub(crate) const fn geometry(&self) -> RepeatingGradientGeometry {
        self.geometry
    }

    pub(crate) fn validate_in(&self, account: Option<&Account>) -> Result<()> {
        self.validate_with_span(account, |error| error)
    }

    fn validate_with_span(
        &self,
        account: Option<&Account>,
        span_error: impl FnOnce(Error) -> Error,
    ) -> Result<()> {
        match self.geometry {
            RepeatingGradientGeometry::Linear { start, end } => {
                #[cfg(feature = "render-paint")]
                validation_coordinates(account, 4)?;
                validate_point(start, "repeating linear gradient start")?;
                validate_point(end, "repeating linear gradient end")?;
            }
            RepeatingGradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            } => {
                #[cfg(feature = "render-paint")]
                validation_coordinates(account, 4)?;
                validate_point(center, "repeating radial gradient center")?;
                validate_positive_f64(radius_x, "repeating radial gradient horizontal radius")?;
                validate_non_negative_f64(radius_y, "repeating radial gradient vertical radius")?;
            }
        }
        if self.stops.is_empty() {
            return Err(Error::invalid_value(
                "repeating gradient stops",
                "[]",
                "must not be empty",
            ));
        }
        let mut first_position = 0.0;
        let mut previous = 0.0;
        for (index, stop) in self.stops.iter().enumerate() {
            if let Some(account) = account {
                account.work(RenderWork::GradientStopVisits, 1)?;
            }
            #[cfg(feature = "render-paint")]
            validation_coordinates(account, 5)?;
            validate_finite_f64(stop.position, "repeating gradient stop position")?;
            validate_color(stop.color, "repeating gradient stop color")?;
            if index == 0 {
                first_position = stop.position;
            } else {
                #[cfg(feature = "render-paint")]
                validation_coordinates(account, 1)?;
                if stop.position < previous {
                    return Err(Error::invalid_value(
                        "repeating gradient stop position",
                        stop.position,
                        "must be nondecreasing",
                    ));
                }
            }
            previous = stop.position;
        }
        #[cfg(feature = "render-paint")]
        validation_coordinates(account, 2)?;
        validate_non_negative_f64(previous - first_position, "repeating gradient span")
            .map_err(span_error)
    }

    #[cfg(feature = "render-paint")]
    pub(crate) fn from_accounted_paint(
        geometry: RepeatingGradientGeometry,
        stops: AccountedVec<RepeatingGradientStop>,
        fixed_owner: Hold,
        first_source_index: usize,
        last_source_index: usize,
        account: &Account,
    ) -> Result<BackedValue<Self>> {
        let (stops, backing) = stops.into_parts();
        let gradient =
            BackedValue::new(Self { geometry, stops }, [Some(backing), Some(fixed_owner)]);
        account.work(RenderWork::PaintVisits, 1)?;
        gradient.validate_with_span(Some(account), |_| {
            crate::PaintGradientError::UnrepresentableSpan {
                first_source_index,
                last_source_index,
            }
            .into()
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
