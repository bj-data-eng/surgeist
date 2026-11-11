mod repeating;

pub(crate) use repeating::GradientGeometry;
pub use repeating::RepeatingGradient;

use super::{
    Error, GradientHintError, Image, Point, Result,
    validation::{
        validate_color, validate_finite_f64, validate_paint, validate_point, validate_positive_f64,
    },
};

/// A validated numeric-sRGB color with straight red, green, blue, and alpha channels.
///
/// Every channel is finite and in the inclusive range `0..=1`. The default is
/// [`Self::TRANSPARENT`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    /// Opaque numeric-sRGB black.
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    /// Fully transparent numeric-sRGB black.
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Constructs channel storage without repeating intrinsic validation.
    ///
    /// Before returning this value across a model boundary, callers must prove
    /// every channel finite and within `0..=1`. Constants and the qualified
    /// native repeating average establish that proof directly; `try_rgba`
    /// validates the stored channels before returning to its public caller.
    #[must_use]
    pub(crate) const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a numeric-sRGB color from straight normalized channels.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] when any channel is non-finite
    /// or outside the inclusive range `0..=1`.
    pub fn try_rgba(r: f32, g: f32, b: f32, a: f32) -> Result<Self> {
        let color = Self::rgba(r, g, b, a);
        validate_color(color, "color")?;
        Ok(color)
    }

    #[must_use]
    /// Returns the normalized red channel.
    pub const fn r(self) -> f32 {
        self.r
    }

    #[must_use]
    /// Returns the normalized green channel.
    pub const fn g(self) -> f32 {
        self.g
    }

    #[must_use]
    /// Returns the normalized blue channel.
    pub const fn b(self) -> f32 {
        self.b
    }

    #[must_use]
    /// Returns the normalized alpha channel.
    pub const fn a(self) -> f32 {
        self.a
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

/// Converts the numeric-sRGB channels to the equivalent Peniko color.
impl From<Color> for peniko::Color {
    fn from(color: Color) -> Self {
        Self::new([color.r, color.g, color.b, color.a])
    }
}

/// The authored color representation carried by [`PaintColor`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintColorSpace {
    /// Straight numeric-sRGB red, green, blue, and alpha channels.
    Srgb,
    /// Hue in degrees followed by normalized saturation, lightness, and alpha.
    Hsl,
}

/// A validated authored color that preserves its sRGB or HSL representation.
///
/// Call [`Self::to_color`] to resolve it to the numeric-sRGB [`Color`] consumed
/// by rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintColor {
    space: PaintColorSpace,
    channels: [f32; 4],
}

impl PaintColor {
    /// Creates an authored numeric-sRGB color from straight normalized channels.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for non-finite or out-of-range channels.
    pub fn try_srgb(r: f32, g: f32, b: f32, a: f32) -> Result<Self> {
        Color::try_rgba(r, g, b, a)?;
        Ok(Self {
            space: PaintColorSpace::Srgb,
            channels: [r, g, b, a],
        })
    }

    /// Creates an authored HSL color.
    ///
    /// Hue is a finite angle in degrees and may be outside one turn; resolution
    /// wraps it modulo 360. Saturation, lightness, and alpha must be finite and
    /// in `0..=1`, otherwise construction returns [`crate::ErrorCode::InvalidInput`].
    pub fn try_hsl(hue_degrees: f32, saturation: f32, lightness: f32, alpha: f32) -> Result<Self> {
        validate_finite_channel(hue_degrees, "hsl hue")?;
        validate_unit_channel(saturation, "hsl saturation")?;
        validate_unit_channel(lightness, "hsl lightness")?;
        validate_unit_channel(alpha, "hsl alpha")?;
        Ok(Self {
            space: PaintColorSpace::Hsl,
            channels: [hue_degrees, saturation, lightness, alpha],
        })
    }

    #[must_use]
    /// Returns the preserved authored color space.
    pub const fn space(self) -> PaintColorSpace {
        self.space
    }

    #[must_use]
    /// Returns the four preserved channels in the order defined by [`Self::space`].
    pub const fn channels(self) -> [f32; 4] {
        self.channels
    }

    /// Resolves this authored value to a validated numeric-sRGB [`Color`].
    ///
    /// HSL hue is wrapped modulo 360 degrees. A typed input diagnostic is
    /// returned if the resulting channels cannot satisfy [`Color`]'s invariant.
    pub fn to_color(self) -> Result<Color> {
        match self.space {
            PaintColorSpace::Srgb => {
                let [r, g, b, a] = self.channels;
                Color::try_rgba(r, g, b, a)
            }
            PaintColorSpace::Hsl => {
                let [hue_degrees, saturation, lightness, alpha] = self.channels;
                let [r, g, b] = hsl_to_srgb(hue_degrees, saturation, lightness);
                Color::try_rgba(r, g, b, alpha)
            }
        }
    }
}

fn validate_finite_channel(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(Error::invalid_value(name, value, "must be finite"));
    }
    Ok(())
}

fn validate_unit_channel(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(Error::invalid_value(
            name,
            value,
            "must be finite and between 0 and 1",
        ));
    }
    Ok(())
}

fn hsl_to_srgb(hue_degrees: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    if saturation == 0.0 {
        return [lightness, lightness, lightness];
    }

    let hue = hue_degrees.rem_euclid(360.0);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_sector = hue / 60.0;
    let x = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
    let m = lightness - chroma / 2.0;

    let [r, g, b] = match hue_sector {
        sector if sector < 1.0 => [chroma, x, 0.0],
        sector if sector < 2.0 => [x, chroma, 0.0],
        sector if sector < 3.0 => [0.0, chroma, x],
        sector if sector < 4.0 => [0.0, x, chroma],
        sector if sector < 5.0 => [x, 0.0, chroma],
        _ => [chroma, 0.0, x],
    };

    [
        (r + m).clamp(0.0, 1.0),
        (g + m).clamp(0.0, 1.0),
        (b + m).clamp(0.0, 1.0),
    ]
}

/// A renderer-facing paint source: a solid color, gradient, or image.
///
/// Each public constructor preserves the validation already encoded by its input type.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    kind: PaintKind,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaintKind {
    Color(Color),
    Gradient(Gradient),
    RepeatingGradient(RepeatingGradient),
    Image(Image),
}

impl Paint {
    #[must_use]
    /// Creates a solid-color paint without changing its numeric-sRGB channels.
    pub const fn color(color: Color) -> Self {
        Self {
            kind: PaintKind::Color(color),
        }
    }

    #[must_use]
    /// Creates a gradient paint from a validated gradient.
    pub fn gradient(gradient: Gradient) -> Self {
        Self {
            kind: PaintKind::Gradient(gradient),
        }
    }

    #[must_use]
    /// Creates a repeating paint from a validated resolved gradient.
    pub fn repeating_gradient(gradient: RepeatingGradient) -> Self {
        Self {
            kind: PaintKind::RepeatingGradient(gradient),
        }
    }

    #[must_use]
    /// Creates an image paint retaining the image's exact content and sampling policy.
    pub fn image(image: Image) -> Self {
        Self {
            kind: PaintKind::Image(image),
        }
    }

    pub(crate) const fn kind(&self) -> &PaintKind {
        &self.kind
    }
}

/// Converts a color to its solid [`Paint`] representation without loss.
impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self::color(color)
    }
}

/// Converts a gradient to its [`Paint`] representation without loss.
impl From<Gradient> for Paint {
    fn from(gradient: Gradient) -> Self {
        Self::gradient(gradient)
    }
}

/// Converts a repeating gradient to its [`Paint`] representation without loss.
impl From<RepeatingGradient> for Paint {
    fn from(gradient: RepeatingGradient) -> Self {
        Self::repeating_gradient(gradient)
    }
}

/// Converts an image to its [`Paint`] representation without loss.
impl From<Image> for Paint {
    fn from(image: Image) -> Self {
        Self::image(image)
    }
}

/// A normalized paint layer whose source has passed canonical paint validation.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedPaintLayer {
    paint: Paint,
}

impl NormalizedPaintLayer {
    /// Canonically validates a renderer-facing paint source.
    ///
    /// Returns a typed input diagnostic when any nested color, gradient,
    /// geometry, stop, or image-size invariant is invalid.
    pub fn try_new(paint: Paint) -> Result<Self> {
        validate_paint(&paint)?;
        Ok(Self { paint })
    }

    #[must_use]
    /// Returns the validated paint source.
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }
}

/// A validated logical-space linear, elliptical radial, horizontal-distance, or sweep gradient.
///
/// Linear, radial, and distance fields preserve nonempty nondecreasing finite positions,
/// including negative and outside-unit positions. Radial axes are positive.
/// Sweep positions remain normalized to `0..=1` in their supplied order, without hints.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    kind: GradientKind,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GradientKind {
    Linear {
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
    },
    Radial {
        center: Point,
        radius_x: f64,
        radius_y: f64,
        stops: Vec<GradientStop>,
    },
    HorizontalDistance {
        origin_x: f64,
        stops: Vec<GradientStop>,
    },
    Sweep {
        center: Point,
        stops: Vec<GradientStop>,
    },
}

/// Returns the accepted endpoints of an ordinary or repeating field.
/// Individual finite positions and colors are already proved by GradientStop.
fn validate_field_stops(stops: &[GradientStop]) -> Result<(&GradientStop, &GradientStop)> {
    let (first, tail) = validate_nonempty_stops(stops)?;
    for (stop_index, pair) in stops.windows(2).enumerate() {
        if pair[1].position < pair[0].position {
            return Err(Error::invalid_value(
                "gradient stop position",
                pair[1].position,
                "must be nondecreasing",
            ));
        }
        if let Some(hint) = pair[0].hint_after
            && !(pair[0].position..=pair[1].position).contains(&hint.position)
        {
            return Err(GradientHintError::HintOutsideInterval { stop_index }.into());
        }
    }
    let last = tail.last().unwrap_or(first);
    if last.hint_after.is_some() {
        return Err(GradientHintError::TrailingHint {
            stop_index: tail.len(),
        }
        .into());
    }
    Ok((first, last))
}

fn validate_nonempty_stops(stops: &[GradientStop]) -> Result<(&GradientStop, &[GradientStop])> {
    stops.split_first().ok_or_else(|| {
        Error::invalid_value("gradient stops", "[]", "must not be empty")
            .replace_message("gradient stops must not be empty")
    })
}

impl Gradient {
    /// Creates an ordinary linear field between finite logical points.
    ///
    /// Empty or descending stops and nonfinite geometry return a typed input
    /// diagnostic. Invalid outgoing hint associations return [`GradientHintError`].
    /// Positions are preserved without a finite-span restriction.
    pub fn try_linear(start: Point, end: Point, stops: Vec<GradientStop>) -> Result<Self> {
        validate_point(start, "linear gradient start")?;
        validate_point(end, "linear gradient end")?;
        validate_field_stops(&stops)?;
        Ok(Self {
            kind: GradientKind::Linear { start, end, stops },
        })
    }

    /// Creates an ordinary elliptical radial field around a finite center.
    ///
    /// Both axes must be finite and positive. Empty or descending stops return
    /// a typed input diagnostic; invalid outgoing hints return [`GradientHintError`].
    /// No finite stop-span restriction is imposed.
    pub fn try_radial(
        center: Point,
        radius_x: f64,
        radius_y: f64,
        stops: Vec<GradientStop>,
    ) -> Result<Self> {
        validate_point(center, "radial gradient center")?;
        validate_positive_f64(radius_x, "radial gradient horizontal radius")?;
        validate_positive_f64(radius_y, "radial gradient vertical radius")?;
        validate_field_stops(&stops)?;
        Ok(Self {
            kind: GradientKind::Radial {
                center,
                radius_x,
                radius_y,
                stops,
            },
        })
    }

    /// Creates an ordinary field whose scalar is `abs(local_x - origin_x)`.
    ///
    /// Stops and hints use physical logical-distance units. A nonfinite origin,
    /// empty or descending list, or invalid hint association returns a typed
    /// diagnostic. No finite stop-span restriction is imposed.
    pub fn try_horizontal_distance(origin_x: f64, stops: Vec<GradientStop>) -> Result<Self> {
        validate_finite_f64(origin_x, "horizontal-distance gradient origin")?;
        validate_field_stops(&stops)?;
        Ok(Self {
            kind: GradientKind::HorizontalDistance { origin_x, stops },
        })
    }

    /// Creates a full-turn sweep around a finite center.
    ///
    /// Stops must be nonempty and within `0..=1`. Their supplied order is kept,
    /// including descending positions; invalid input returns a typed diagnostic.
    /// The first outgoing hint returns [`GradientHintError::UnsupportedSweep`].
    pub fn try_sweep(center: Point, stops: Vec<GradientStop>) -> Result<Self> {
        validate_point(center, "sweep gradient center")?;
        validate_nonempty_stops(&stops)?;
        let mut invalid_position = None;
        for (stop_index, stop) in stops.iter().enumerate() {
            if stop.hint_after.is_some() {
                return Err(GradientHintError::UnsupportedSweep { stop_index }.into());
            }
            if !(0.0..=1.0).contains(&stop.position) && invalid_position.is_none() {
                invalid_position = Some(stop.position);
            }
        }
        if let Some(position) = invalid_position {
            return Err(Error::invalid_value(
                "sweep gradient stop position",
                position,
                "must be between 0 and 1",
            ));
        }
        Ok(Self {
            kind: GradientKind::Sweep { center, stops },
        })
    }

    /// Returns all supplied stops, including coincident records.
    #[must_use]
    pub fn stops(&self) -> &[GradientStop] {
        match &self.kind {
            GradientKind::Linear { stops, .. }
            | GradientKind::Radial { stops, .. }
            | GradientKind::HorizontalDistance { stops, .. }
            | GradientKind::Sweep { stops, .. } => stops,
        }
    }

    /// Returns the supplied linear start and end points, when linear.
    #[must_use]
    pub const fn linear_points(&self) -> Option<(Point, Point)> {
        match &self.kind {
            GradientKind::Linear { start, end, .. } => Some((*start, *end)),
            GradientKind::Radial { .. }
            | GradientKind::HorizontalDistance { .. }
            | GradientKind::Sweep { .. } => None,
        }
    }

    /// Returns the supplied center and horizontal and vertical radii, when radial.
    #[must_use]
    pub const fn radial_geometry(&self) -> Option<(Point, f64, f64)> {
        match &self.kind {
            GradientKind::Radial {
                center,
                radius_x,
                radius_y,
                ..
            } => Some((*center, *radius_x, *radius_y)),
            GradientKind::Linear { .. }
            | GradientKind::HorizontalDistance { .. }
            | GradientKind::Sweep { .. } => None,
        }
    }

    /// Returns the supplied physical horizontal-distance origin, when applicable.
    #[must_use]
    pub const fn horizontal_distance_origin(&self) -> Option<f64> {
        match &self.kind {
            GradientKind::HorizontalDistance { origin_x, .. } => Some(*origin_x),
            GradientKind::Linear { .. }
            | GradientKind::Radial { .. }
            | GradientKind::Sweep { .. } => None,
        }
    }

    /// Returns the supplied sweep center, when a sweep.
    #[must_use]
    pub const fn sweep_center(&self) -> Option<Point> {
        match &self.kind {
            GradientKind::Sweep { center, .. } => Some(*center),
            GradientKind::Linear { .. }
            | GradientKind::Radial { .. }
            | GradientKind::HorizontalDistance { .. } => None,
        }
    }

    pub(crate) const fn kind(&self) -> &GradientKind {
        &self.kind
    }

    #[cfg(feature = "render-paint")]
    pub(crate) fn from_accounted_paint(
        geometry: GradientGeometry,
        stops: crate::accounting::AccountedVec<GradientStop>,
        fixed_owner: crate::accounting::Hold,
        account: &crate::accounting::Account,
    ) -> Result<crate::accounting::PreparationValue<Self>> {
        // Authentic Paint geometry proves finite fields and positive ellipse
        // axes. Used records preserve their proof through either qualified
        // quotients or physical identity into this exact backing, without a
        // duplicate canonical validation walk.
        account.work(crate::accounting::RenderWork::PaintVisits, 1)?;
        let fixed_owner = fixed_owner.into_preparation()?;
        let (stops, backing) = stops.into_preparation()?.into_parts();
        let kind = match geometry {
            GradientGeometry::Linear { start, end } => GradientKind::Linear { start, end, stops },
            GradientGeometry::HorizontalDistance { origin_x } => {
                GradientKind::HorizontalDistance { origin_x, stops }
            }
            GradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            } => GradientKind::Radial {
                center,
                radius_x,
                radius_y,
                stops,
            },
        };
        let gradient = crate::accounting::PreparationValue::new(
            Self { kind },
            [Some(backing), Some(fixed_owner)],
        );
        Ok(gradient)
    }
}

/// A finite transition-hint position in its owning gradient field units.
///
/// A hint belongs to one color stop's outgoing interval. Its neighboring colors
/// establish inclusive containment when the complete gradient is constructed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientHint {
    position: f64,
}

impl GradientHint {
    /// Creates a hint while preserving its finite position exactly.
    ///
    /// A nonfinite position returns [`crate::ErrorCode::InvalidInput`].
    pub fn try_new(position: f64) -> Result<Self> {
        validate_finite_f64(position, "gradient hint position")?;
        Ok(Self { position })
    }

    /// Returns the supplied position in field units.
    #[must_use]
    pub const fn position(self) -> f64 {
        self.position
    }

    #[cfg(feature = "render-paint")]
    pub(crate) const fn from_qualified_paint(position: f64) -> Self {
        // The native Paint consumer proved the finite quotient before storage.
        Self { position }
    }
}

/// A resolved gradient stop positioned in its owning field units.
///
/// The finite position is preserved exactly, including negative and outside-unit
/// values. Linear zero/one identify its start/end; radial zero/one identify its
/// center/horizontal radius. Horizontal-distance fields use physical logical
/// distances. Sweep construction requires normalized positions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    position: f64,
    color: Color,
    hint_after: Option<GradientHint>,
}

impl GradientStop {
    /// Creates a stop without clamping its finite position.
    ///
    /// A nonfinite position returns [`crate::ErrorCode::InvalidInput`]. Color
    /// already owns its finite normalized channel invariant.
    pub fn try_new(position: f64, color: Color) -> Result<Self> {
        validate_finite_f64(position, "gradient stop position")?;
        Ok(Self {
            position,
            color,
            hint_after: None,
        })
    }

    #[cfg(feature = "render-paint")]
    pub(crate) const fn from_qualified_paint(
        position: f64,
        color: Color,
        hint_after: Option<GradientHint>,
    ) -> Self {
        // The consumer proved finite ordered quotients and outgoing containment
        // while consuming authentic native Paint records into this backing.
        Self {
            position,
            color,
            hint_after,
        }
    }

    /// Attaches a hint to this stop's outgoing color interval.
    ///
    /// Complete Gradient/RepeatingGradient construction checks that a following
    /// color exists and the hint lies within their inclusive position interval.
    #[must_use]
    pub const fn with_hint_after(mut self, hint: GradientHint) -> Self {
        self.hint_after = Some(hint);
        self
    }

    /// Returns this stop's outgoing transition hint, when supplied.
    #[must_use]
    pub const fn hint_after(self) -> Option<GradientHint> {
        self.hint_after
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

/// A borrowed sweep with its validated, nonempty stop list.
pub(crate) struct SweepGradient<'a> {
    center: Point,
    stops: &'a [GradientStop],
}

#[derive(Clone, Copy)]
pub(crate) enum OrdinaryGeometry<'a> {
    Linear {
        start: &'a Point,
        end: &'a Point,
    },
    Radial {
        center: &'a Point,
        radius_x: &'a f64,
        radius_y: &'a f64,
    },
    HorizontalDistance {
        origin_x: &'a f64,
    },
}

/// An ordinary field with its exact validated, nonempty stop list.
pub(crate) struct OrdinaryGradient<'a> {
    geometry: OrdinaryGeometry<'a>,
    stops: &'a [GradientStop],
}

impl<'a> OrdinaryGradient<'a> {
    pub(crate) const fn geometry(&self) -> OrdinaryGeometry<'a> {
        self.geometry
    }

    pub(crate) const fn stops(&self) -> &'a [GradientStop] {
        self.stops
    }
}

pub(crate) enum GradientPreparation<'a> {
    Ordinary(OrdinaryGradient<'a>),
    Sweep(SweepGradient<'a>),
}

impl Gradient {
    pub(crate) fn preparation(&self) -> GradientPreparation<'_> {
        match &self.kind {
            GradientKind::Linear { start, end, stops } => {
                GradientPreparation::Ordinary(OrdinaryGradient {
                    geometry: OrdinaryGeometry::Linear { start, end },
                    stops,
                })
            }
            GradientKind::Radial {
                center,
                radius_x,
                radius_y,
                stops,
            } => GradientPreparation::Ordinary(OrdinaryGradient {
                geometry: OrdinaryGeometry::Radial {
                    center,
                    radius_x,
                    radius_y,
                },
                stops,
            }),
            GradientKind::HorizontalDistance { origin_x, stops } => {
                GradientPreparation::Ordinary(OrdinaryGradient {
                    geometry: OrdinaryGeometry::HorizontalDistance { origin_x },
                    stops,
                })
            }
            GradientKind::Sweep { center, stops } => GradientPreparation::Sweep(SweepGradient {
                center: *center,
                stops,
            }),
        }
    }
}

impl SweepGradient<'_> {
    pub(crate) const fn center(&self) -> Point {
        self.center
    }

    pub(crate) const fn stops(&self) -> &[GradientStop] {
        self.stops
    }

    /// The native ramp starts its first color at zero. Anchor that same color
    /// there when a multistop Pad field has a positive first supplied offset.
    /// A single stop already selects the native solid path, so its offset is
    /// deliberately not inspected here.
    pub(crate) fn native_leading_stop(
        &self,
        account: &crate::accounting::Account,
    ) -> Result<Option<GradientStop>> {
        if self.stops().len() < 2 {
            return Ok(None);
        }
        account.work(crate::accounting::RenderWork::GradientStopVisits, 1)?;
        let first = self.stops()[0];
        account.work(crate::accounting::RenderWork::CoordinateVisits, 1)?;
        let position = first.position;
        account.work(crate::accounting::RenderWork::CoordinateVisits, 1)?;
        Ok((position > 0.0).then_some(GradientStop {
            position: 0.0,
            color: first.color,
            hint_after: None,
        }))
    }
}

impl Gradient {
    pub(crate) fn try_clone_in(
        &self,
        account: &crate::accounting::Account,
    ) -> Result<(Self, crate::accounting::Hold)> {
        let mut stops = account.retained(self.stops().len())?;
        for stop in self.stops() {
            account.work(crate::accounting::RenderWork::GradientStopVisits, 1)?;
            stops.push(*stop)?;
        }
        let (stops, hold) = stops.into_parts()?;
        let kind = match self.kind {
            GradientKind::Linear { start, end, .. } => GradientKind::Linear { start, end, stops },
            GradientKind::HorizontalDistance { origin_x, .. } => {
                GradientKind::HorizontalDistance { origin_x, stops }
            }
            GradientKind::Radial {
                center,
                radius_x,
                radius_y,
                ..
            } => GradientKind::Radial {
                center,
                radius_x,
                radius_y,
                stops,
            },
            GradientKind::Sweep { center, .. } => GradientKind::Sweep { center, stops },
        };
        Ok((Self { kind }, hold))
    }
}

impl Paint {
    pub(crate) fn try_clone_in(
        &self,
        account: &crate::accounting::Account,
    ) -> Result<(Self, Option<crate::accounting::Hold>)> {
        account.work(crate::accounting::RenderWork::PaintVisits, 1)?;
        let (kind, hold) = match &self.kind {
            PaintKind::Color(color) => (PaintKind::Color(*color), None),
            PaintKind::Image(image) => {
                account.work(crate::accounting::RenderWork::ImageIdentityCopies, 1)?;
                (PaintKind::Image(image.clone()), None)
            }
            PaintKind::RepeatingGradient(gradient) => {
                let (gradient, hold) = gradient.try_clone_in(account)?;
                (PaintKind::RepeatingGradient(gradient), Some(hold))
            }
            PaintKind::Gradient(gradient) => {
                let (gradient, hold) = gradient.try_clone_in(account)?;
                (PaintKind::Gradient(gradient), Some(hold))
            }
        };
        Ok((Self { kind }, hold))
    }
}

#[cfg(test)]
mod field_model_tests {
    use super::*;

    fn stop(position: f64) -> GradientStop {
        GradientStop::try_new(position, Color::BLACK).unwrap()
    }

    #[test]
    fn validated_field_endpoints_borrow_the_actual_first_and_last_records() {
        let records = [
            stop(3.0),
            stop(3.0),
            GradientStop::try_new(3.0, Color::TRANSPARENT).unwrap(),
        ];
        let (first, last) = validate_field_stops(&records[..1]).unwrap();
        assert!(std::ptr::eq(first, &records[0]));
        assert!(std::ptr::eq(last, first));

        let (first, last) = validate_field_stops(&records).unwrap();
        assert!(std::ptr::eq(first, &records[0]));
        assert!(std::ptr::eq(last, &records[2]));
    }

    #[test]
    fn gradient_projection_borrows_validated_stops_for_each_geometry() {
        fn ordinary(gradient: &Gradient) -> OrdinaryGradient<'_> {
            let GradientPreparation::Ordinary(field) = gradient.preparation() else {
                panic!("expected ordinary field preparation");
            };
            assert!(std::ptr::eq(field.stops(), gradient.stops()));
            assert_eq!(
                field.stops(),
                [
                    stop(1.0),
                    GradientStop::try_new(5.0, Color::TRANSPARENT).unwrap()
                ]
            );
            field
        }

        let stops = [
            stop(1.0),
            GradientStop::try_new(5.0, Color::TRANSPARENT).unwrap(),
        ];
        let start = Point::new(1.0, -2.0);
        let end = Point::new(3.0, 4.0);
        let linear = Gradient::try_linear(start, end, stops.to_vec()).unwrap();
        assert!(matches!(
            ordinary(&linear).geometry(),
            OrdinaryGeometry::Linear { start: actual_start, end: actual_end }
                if *actual_start == start && *actual_end == end
        ));
        let radial = Gradient::try_radial(end, 2.0, 5.0, stops.to_vec()).unwrap();
        assert!(matches!(
            ordinary(&radial).geometry(),
            OrdinaryGeometry::Radial { center, radius_x, radius_y }
                if *center == end && *radius_x == 2.0 && *radius_y == 5.0
        ));
        let distance = Gradient::try_horizontal_distance(-7.0, stops.to_vec()).unwrap();
        assert!(matches!(
            ordinary(&distance).geometry(),
            OrdinaryGeometry::HorizontalDistance { origin_x } if *origin_x == -7.0
        ));

        let sweep = Gradient::try_sweep(start, vec![stop(0.75), stop(0.25)]).unwrap();
        let GradientPreparation::Sweep(field) = sweep.preparation() else {
            panic!("expected sweep preparation");
        };
        assert_eq!(field.center(), start);
        assert!(std::ptr::eq(field.stops(), sweep.stops()));
        assert_eq!(field.stops(), [stop(0.75), stop(0.25)]);
    }

    #[test]
    fn ordinary_fields_preserve_finite_records_with_an_overflowing_span() {
        let center = Point::new(3.0, -4.0);
        let stops = vec![stop(-f64::MAX), stop(-1.0), stop(-1.0), stop(f64::MAX)];
        let linear = Gradient::try_linear(center, center, stops.clone()).unwrap();
        let radial = Gradient::try_radial(center, 2.0, 5.0, stops.clone()).unwrap();
        assert_eq!(linear.stops(), stops);
        assert_eq!(linear.linear_points(), Some((center, center)));
        assert_eq!(radial.stops(), stops);
        assert_eq!(radial.radial_geometry(), Some((center, 2.0, 5.0)));
        assert!(RepeatingGradient::try_linear(center, center, stops).is_err());
    }

    #[test]
    fn ordinary_fields_require_order_but_sweep_keeps_supplied_normalized_order() {
        let p = Point::new(0.0, 0.0);
        let descending = vec![stop(0.75), stop(0.25)];
        assert!(Gradient::try_linear(p, p, descending.clone()).is_err());
        assert!(Gradient::try_radial(p, 1.0, 2.0, descending.clone()).is_err());
        assert_eq!(
            Gradient::try_sweep(p, descending.clone()).unwrap().stops(),
            descending
        );
        for position in [-f64::MIN_POSITIVE, 1.0 + f64::EPSILON] {
            assert!(Gradient::try_sweep(p, vec![stop(position)]).is_err());
        }
    }

    #[test]
    fn canonical_gradient_boundaries_refuse_empty_nonfinite_and_nonpositive_inputs() {
        let p = Point::new(0.0, 0.0);
        assert!(Gradient::try_linear(p, p, vec![]).is_err());
        assert!(Gradient::try_radial(p, 1.0, 1.0, vec![]).is_err());
        assert!(Gradient::try_sweep(p, vec![]).is_err());
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(GradientStop::try_new(value, Color::BLACK).is_err());
            // Internal geometry arithmetic can produce an invalid raw Point.
            assert!(Gradient::try_linear(Point::new(value, 0.0), p, vec![stop(0.0)]).is_err());
            assert!(
                Gradient::try_radial(Point::new(0.0, value), 1.0, 1.0, vec![stop(0.0)]).is_err()
            );
        }
        for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(Gradient::try_radial(p, value, 1.0, vec![stop(0.0)]).is_err());
            assert!(Gradient::try_radial(p, 1.0, value, vec![stop(0.0)]).is_err());
        }
        assert!(RepeatingGradient::try_radial(p, 1.0, 0.0, vec![stop(0.0)]).is_ok());
    }
}

#[cfg(test)]
mod hint_model_tests {
    use super::*;

    fn stop(position: f64, hint: Option<f64>) -> GradientStop {
        let stop = GradientStop::try_new(position, Color::BLACK).unwrap();
        hint.map_or(stop, |position| {
            stop.with_hint_after(GradientHint::try_new(position).unwrap())
        })
    }

    #[test]
    fn finite_hints_and_inclusive_endpoint_associations_preserve_exact_values() {
        let p = Point::new(0.0, 0.0);
        for (a, h, b) in [
            (-f64::MAX, -1.0, f64::MAX),
            (-2.0, -2.0, 3.0),
            (-2.0, 3.0, 3.0),
            (7.0, 7.0, 7.0),
        ] {
            let stops = vec![stop(a, Some(h)), stop(b, None)];
            let gradient = Gradient::try_linear(p, p, stops.clone()).unwrap();
            assert_eq!(gradient.stops(), stops);
            assert_eq!(
                gradient.stops()[0]
                    .hint_after()
                    .unwrap()
                    .position()
                    .to_bits(),
                h.to_bits()
            );
            assert!(gradient.stops()[1].hint_after().is_none());
            if a != -f64::MAX {
                assert_eq!(
                    RepeatingGradient::try_linear(p, p, stops.clone())
                        .unwrap()
                        .stops(),
                    stops
                );
            }
        }
        for position in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = GradientHint::try_new(position).unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
            assert!(error.gradient_hint_diagnostic().is_none());
        }
    }

    #[test]
    fn canonical_fields_report_the_outgoing_interval_for_bad_hint_associations() {
        let p = Point::new(0.0, 0.0);
        for (stops, expected) in [
            (
                vec![stop(0.0, Some(0.0))],
                GradientHintError::TrailingHint { stop_index: 0 },
            ),
            (
                vec![stop(0.0, None), stop(1.0, Some(1.0))],
                GradientHintError::TrailingHint { stop_index: 1 },
            ),
            (
                vec![stop(0.0, Some(-1.0)), stop(1.0, None)],
                GradientHintError::HintOutsideInterval { stop_index: 0 },
            ),
            (
                vec![stop(0.0, None), stop(1.0, Some(3.0)), stop(2.0, None)],
                GradientHintError::HintOutsideInterval { stop_index: 1 },
            ),
        ] {
            for error in [
                Gradient::try_linear(p, p, stops.clone()).unwrap_err(),
                Gradient::try_radial(p, 1.0, 2.0, stops.clone()).unwrap_err(),
                RepeatingGradient::try_linear(p, p, stops.clone()).unwrap_err(),
                RepeatingGradient::try_radial(p, 1.0, 2.0, stops.clone()).unwrap_err(),
                Gradient::try_horizontal_distance(0.0, stops.clone()).unwrap_err(),
                RepeatingGradient::try_horizontal_distance(0.0, stops.clone()).unwrap_err(),
            ] {
                assert_eq!(error.code(), crate::ErrorCode::GradientHint);
                assert_eq!(error.gradient_hint_diagnostic(), Some(&expected));
                assert!(error.invalid_value_diagnostic().is_none());
                assert!(error.gradient_realization_diagnostic().is_none());
            }
        }
    }

    #[test]
    fn field_stop_validation_preserves_diagnostic_priority() {
        let center = Point::new(0.0, 0.0);
        let descending = vec![stop(1.0, Some(2.0)), stop(0.0, None)];
        for error in [
            Gradient::try_linear(center, center, descending.clone()).unwrap_err(),
            RepeatingGradient::try_linear(center, center, descending).unwrap_err(),
        ] {
            assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
            let diagnostic = error.invalid_value_diagnostic().unwrap();
            assert_eq!(diagnostic.field(), "gradient stop position");
            assert_eq!(diagnostic.value(), "0");
            assert_eq!(diagnostic.invariant(), "must be nondecreasing");
        }

        let early_hint = vec![stop(0.0, Some(2.0)), stop(1.0, None), stop(0.0, None)];
        let error = RepeatingGradient::try_linear(center, center, early_hint).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::GradientHint);
        assert_eq!(
            error.gradient_hint_diagnostic(),
            Some(&GradientHintError::HintOutsideInterval { stop_index: 0 })
        );

        let trailing = vec![stop(-f64::MAX, None), stop(f64::MAX, Some(f64::MAX))];
        let error = RepeatingGradient::try_linear(center, center, trailing).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::GradientHint);
        assert_eq!(
            error.gradient_hint_diagnostic(),
            Some(&GradientHintError::TrailingHint { stop_index: 1 })
        );
        let overflow = vec![stop(-f64::MAX, None), stop(f64::MAX, None)];
        let error = RepeatingGradient::try_linear(center, center, overflow).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
        let diagnostic = error.invalid_value_diagnostic().unwrap();
        assert_eq!(diagnostic.field(), "repeating gradient span");
        assert_eq!(diagnostic.value(), "inf");
        assert_eq!(diagnostic.invariant(), "must be finite and non-negative");

        for (radius, field) in [
            (0.0, "repeating radial gradient horizontal radius"),
            (1.0, "gradient stops"),
        ] {
            let error = RepeatingGradient::try_radial(center, radius, 1.0, vec![]).unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
            assert_eq!(error.invalid_value_diagnostic().unwrap().field(), field);
        }
    }

    #[test]
    fn sweep_reports_its_first_hint_before_an_unrelated_range_error() {
        let p = Point::new(0.0, 0.0);
        let error = Gradient::try_sweep(
            p,
            vec![
                stop(-1.0, None),
                stop(0.25, Some(0.5)),
                stop(0.75, Some(0.75)),
            ],
        )
        .unwrap_err();
        assert_eq!(
            error.gradient_hint_diagnostic(),
            Some(&GradientHintError::UnsupportedSweep { stop_index: 1 })
        );
        let error = Gradient::try_sweep(p, vec![stop(-1.0, None), stop(0.75, None)]).unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
    }
}

#[cfg(test)]
mod horizontal_distance_model_tests {
    use super::*;

    fn stop(position: f64) -> GradientStop {
        GradientStop::try_new(position, Color::BLACK).unwrap()
    }

    #[test]
    fn physical_distance_records_hints_and_origin_survive_model_and_accounted_clone() {
        let stops = vec![
            stop(-4.0).with_hint_after(GradientHint::try_new(-2.0).unwrap()),
            stop(4.0),
            stop(12.0),
        ];
        let ordinary = Gradient::try_horizontal_distance(8.0, stops.clone()).unwrap();
        let repeated = RepeatingGradient::try_horizontal_distance(8.0, stops.clone()).unwrap();
        assert_eq!(ordinary.horizontal_distance_origin(), Some(8.0));
        assert_eq!(repeated.horizontal_distance_origin(), Some(8.0));
        assert_eq!(ordinary.stops(), stops);
        assert_eq!(repeated.stops(), stops);
        assert_eq!(ordinary.linear_points(), None);
        assert_eq!(ordinary.radial_geometry(), None);
        assert_eq!(ordinary.sweep_center(), None);
        assert_eq!(repeated.linear_points(), None);
        assert_eq!(repeated.radial_geometry(), None);
        let account = crate::accounting::Account::for_test();
        let (copy, hold) = ordinary.try_clone_in(&account).unwrap();
        assert_eq!(copy, ordinary);
        drop((copy, hold));
        let (copy, hold) = repeated.try_clone_in(&account).unwrap();
        assert_eq!(copy, repeated);
        drop((copy, hold));
        let p = Point::new(0.0, 0.0);
        assert_eq!(
            Gradient::try_linear(p, p, vec![stop(0.0)])
                .unwrap()
                .horizontal_distance_origin(),
            None
        );
        assert_eq!(
            RepeatingGradient::try_linear(p, p, vec![stop(0.0)])
                .unwrap()
                .horizontal_distance_origin(),
            None
        );
    }

    #[test]
    fn finite_origin_and_existing_list_rules_are_the_horizontal_distance_boundary() {
        for origin in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Gradient::try_horizontal_distance(origin, vec![stop(0.0)]).is_err());
            assert!(RepeatingGradient::try_horizontal_distance(origin, vec![stop(0.0)]).is_err());
        }
        for stops in [vec![], vec![stop(1.0), stop(0.0)]] {
            assert!(Gradient::try_horizontal_distance(0.0, stops.clone()).is_err());
            assert!(RepeatingGradient::try_horizontal_distance(0.0, stops).is_err());
        }
        // Public construction does not form an unused origin+1 endpoint, and
        // ordinary physical positions have no finite-span restriction.
        let extremes = vec![stop(-f64::MAX), stop(f64::MAX)];
        assert_eq!(
            Gradient::try_horizontal_distance(f64::MAX, extremes.clone())
                .unwrap()
                .stops(),
            extremes
        );
        assert!(RepeatingGradient::try_horizontal_distance(0.0, extremes).is_err());
        let zeros = vec![stop(-0.0), stop(0.0)];
        assert!(RepeatingGradient::try_horizontal_distance(0.0, zeros).is_ok());
    }
}
