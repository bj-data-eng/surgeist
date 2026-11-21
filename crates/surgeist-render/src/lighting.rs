//! Native diffuse and specular lighting for finite layer sources.
//!
//! Admitted lighting shades the source alpha as a height field in local
//! primitive units. Supply one light with [`prepare_lighting`] for diffuse
//! reflectivity or [`prepare_specular_lighting`] for specular highlights.
//! Attach the admitted operation and a finite [`LightingSource`] through
//! [`Scene::lighting_layer`]. The source, filter and primitive-result rectangles
//! stay independent, and even an empty source can illuminate a flat plane.

use crate::{
    Error, Result,
    accounting::{Account, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork},
    effect::{FilterColorSpace, FilterDomains},
    error::PrimitiveOperation,
    number_text::Number,
    paint::Color,
    scene::Scene,
};

/// Resolved diffuse lighting input awaiting intrinsic validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffuseLightingInput {
    surface: LightingSurfaceInput,
    constant: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LightingSurfaceInput {
    surface_scale: f64,
    kernel: Option<[f64; 2]>,
    lighting_color: Color,
}

impl LightingSurfaceInput {
    const fn new() -> Self {
        Self {
            surface_scale: 1.0,
            kernel: None,
            lighting_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl DiffuseLightingInput {
    /// Sets unit height scale and diffuse reflectivity, white light, and default grid spacing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            surface: LightingSurfaceInput::new(),
            constant: 1.0,
        }
    }

    /// Sets the signed height associated with unit source alpha.
    #[must_use]
    pub const fn with_surface_scale(mut self, scale: f64) -> Self {
        self.surface.surface_scale = scale;
        self
    }

    /// Sets the nonnegative diffuse reflectivity.
    #[must_use]
    pub const fn with_diffuse_constant(mut self, constant: f64) -> Self {
        self.constant = constant;
        self
    }

    /// Sets the same kernel spacing in local primitive units on both axes.
    #[must_use]
    pub const fn with_kernel_unit_length(self, length: f64) -> Self {
        self.with_kernel_unit_lengths(length, length)
    }

    /// Sets independent horizontal and vertical kernel spacings in local primitive units.
    ///
    /// An explicit pair containing a nonpositive value selects the default
    /// spacing on both axes after validation.
    #[must_use]
    pub const fn with_kernel_unit_lengths(mut self, x: f64, y: f64) -> Self {
        self.surface.kernel = Some([x, y]);
        self
    }

    /// Sets the straight numeric-sRGB light color. Lighting uses its RGB channels.
    #[must_use]
    pub const fn with_lighting_color(mut self, color: Color) -> Self {
        self.surface.lighting_color = color;
        self
    }
}

impl Default for DiffuseLightingInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolved specular lighting input awaiting intrinsic validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecularLightingInput {
    surface: LightingSurfaceInput,
    constant: f64,
    exponent: f64,
}

impl SpecularLightingInput {
    /// Sets unit height scale, specular reflectivity and exponent, with white light and default spacing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            surface: LightingSurfaceInput::new(),
            constant: 1.0,
            exponent: 1.0,
        }
    }

    /// Sets the signed height associated with unit source alpha.
    #[must_use]
    pub const fn with_surface_scale(mut self, scale: f64) -> Self {
        self.surface.surface_scale = scale;
        self
    }

    /// Sets the nonnegative specular reflectivity.
    #[must_use]
    pub const fn with_specular_constant(mut self, constant: f64) -> Self {
        self.constant = constant;
        self
    }

    /// Sets the finite material exponent, which preparation clamps to `1..=128`.
    ///
    /// This exponent controls the highlight and is independent of spot focus.
    #[must_use]
    pub const fn with_specular_exponent(mut self, exponent: f64) -> Self {
        self.exponent = exponent;
        self
    }

    /// Sets the same kernel spacing in local primitive units on both axes.
    #[must_use]
    pub const fn with_kernel_unit_length(self, length: f64) -> Self {
        self.with_kernel_unit_lengths(length, length)
    }

    /// Sets independent horizontal and vertical kernel spacings in local primitive units.
    ///
    /// An explicit pair containing a nonpositive value selects the default
    /// spacing on both axes after validation.
    #[must_use]
    pub const fn with_kernel_unit_lengths(mut self, x: f64, y: f64) -> Self {
        self.surface.kernel = Some([x, y]);
        self
    }

    /// Sets the straight numeric-sRGB light color. Lighting uses its RGB channels.
    #[must_use]
    pub const fn with_lighting_color(mut self, color: Color) -> Self {
        self.surface.lighting_color = color;
        self
    }
}

impl Default for SpecularLightingInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Exactly one resolved light awaiting intrinsic validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightingLightInput {
    /// A direction shared by all surface points.
    Distant {
        /// Horizontal angle in degrees measured from the positive x-axis toward the y-axis.
        azimuth: f64,
        /// Angle in degrees above the local xy plane.
        elevation: f64,
    },
    /// A local position illuminating the surface without distance attenuation.
    Point {
        /// Light position in local primitive units, in x, y, z order.
        position: [f64; 3],
    },
    /// A positioned light with an aim, focus and optional cone.
    Spot(SpotLightInput),
}

impl LightingLightInput {
    /// Creates a distant light with both angles zero.
    #[must_use]
    pub const fn distant() -> Self {
        Self::Distant {
            azimuth: 0.0,
            elevation: 0.0,
        }
    }

    /// Creates a point light at the layer's local origin.
    #[must_use]
    pub const fn point() -> Self {
        Self::Point { position: [0.0; 3] }
    }
}

/// Resolved spot position, aim and falloff awaiting intrinsic validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpotLightInput {
    position: [f64; 3],
    points_at: [f64; 3],
    exponent: f64,
    cone_angle: Option<f64>,
}

impl SpotLightInput {
    /// Creates a unit-focus light at the origin, aimed at the origin, with an open cone.
    ///
    /// A spot whose position and target coincide contributes no incident light.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            position: [0.0; 3],
            points_at: [0.0; 3],
            exponent: 1.0,
            cone_angle: None,
        }
    }

    /// Sets the local x, y and z light position.
    #[must_use]
    pub const fn with_position(mut self, position: [f64; 3]) -> Self {
        self.position = position;
        self
    }

    /// Sets the local x, y and z coordinates the light points at.
    #[must_use]
    pub const fn with_points_at(mut self, points_at: [f64; 3]) -> Self {
        self.points_at = points_at;
        self
    }

    /// Sets the spot focus exponent, which preparation clamps to `1..=128`.
    #[must_use]
    pub const fn with_specular_exponent(mut self, exponent: f64) -> Self {
        self.exponent = exponent;
        self
    }

    /// Sets a cone angle in degrees, folded to its magnitude and capped at 90 during preparation.
    ///
    /// An explicit zero restricts the light to its axis. The default absent
    /// angle allows the focus function over the forward hemisphere.
    #[must_use]
    pub const fn with_limiting_cone_angle(mut self, angle: f64) -> Self {
        self.cone_angle = Some(angle);
        self
    }
}

impl Default for SpotLightInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Finite scene content and the source, filter and output domains for lighting.
#[derive(Clone, Debug, PartialEq)]
pub struct LightingSource {
    scene: Scene,
    domains: FilterDomains,
}

impl LightingSource {
    /// Takes ownership of the scene with transparent extension outside the source domain.
    #[must_use]
    pub const fn finite(scene: Scene, domains: FilterDomains) -> Self {
        Self { scene, domains }
    }

    pub(crate) fn into_parts(self) -> (Scene, FilterDomains) {
        (self.scene, self.domains)
    }
}

/// Intrinsically admitted lighting parameters independent of any frame or device.
///
/// The frame resolves default grid spacing and qualifies the actual grid,
/// spatial mappings, and shader arithmetic against the selected output region.
/// This value owns no source content or native GPU objects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedLighting {
    surface_scale: f64,
    material: LightingMaterial,
    kernel: KernelSpacing,
    lighting_color: Color,
    color_space: FilterColorSpace,
    light: PreparedLight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LightingMaterial {
    Diffuse { constant: f64 },
    Specular { constant: f64, exponent: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum KernelSpacing {
    Default,
    PositivePair([f64; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PreparedLight {
    Distant {
        direction: [f64; 3],
    },
    Point {
        position: [f64; 3],
    },
    Spot {
        position: [f64; 3],
        direction: Option<[f64; 3]>,
        exponent: f64,
        cone_angle: Option<f64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LightingForeground {
    pub(crate) operation: PreparedLighting,
    pub(crate) source: FilterDomains,
}

impl PreparedLighting {
    pub(crate) const fn surface_scale(self) -> f64 {
        self.surface_scale
    }

    pub(crate) const fn constant(self) -> f64 {
        match self.material {
            LightingMaterial::Diffuse { constant }
            | LightingMaterial::Specular { constant, .. } => constant,
        }
    }

    pub(crate) const fn material_exponent(self) -> Option<f64> {
        match self.material {
            LightingMaterial::Diffuse { .. } => None,
            LightingMaterial::Specular { exponent, .. } => Some(exponent),
        }
    }

    pub(crate) const fn primitive_operation(self) -> PrimitiveOperation {
        match self.material {
            LightingMaterial::Diffuse { .. } => {
                PrimitiveOperation::GpuDiffuseLightingFilterExecution
            }
            LightingMaterial::Specular { .. } => {
                PrimitiveOperation::GpuSpecularLightingFilterExecution
            }
        }
    }

    pub(crate) const fn kernel_unit_lengths(self) -> Option<[f64; 2]> {
        match self.kernel {
            KernelSpacing::Default => None,
            KernelSpacing::PositivePair(pair) => Some(pair),
        }
    }

    pub(crate) const fn lighting_color(self) -> Color {
        self.lighting_color
    }

    pub(crate) const fn color_space(self) -> FilterColorSpace {
        self.color_space
    }

    pub(crate) const fn light(self) -> PreparedLight {
        self.light
    }
}

/// Resolved lighting parameter identified by a preparation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LightingParameter {
    /// Signed surface height scale.
    SurfaceScale,
    /// Nonnegative diffuse reflectivity.
    DiffuseConstant,
    /// Nonnegative specular reflectivity.
    SpecularConstant,
    /// Specular material exponent.
    MaterialExponent,
    /// Horizontal kernel spacing.
    KernelUnitLengthX,
    /// Vertical kernel spacing.
    KernelUnitLengthY,
    /// Distant light azimuth in degrees.
    Azimuth,
    /// Distant light elevation in degrees.
    Elevation,
    /// Local light x-coordinate.
    LightX,
    /// Local light y-coordinate.
    LightY,
    /// Local light z-coordinate.
    LightZ,
    /// Local spot target x-coordinate.
    PointsAtX,
    /// Local spot target y-coordinate.
    PointsAtY,
    /// Local spot target z-coordinate.
    PointsAtZ,
    /// Spot focus exponent.
    SpotExponent,
    /// Optional spot cone angle.
    Cone,
}

/// Lighting arithmetic boundary that could not be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LightingNumericPhase {
    /// Full-canvas grid extent.
    GridExtent,
    /// Mapping between lighting grids.
    GridMapping,
    /// Reached grid indices.
    GridIndex,
    /// Reached sampling neighborhoods.
    Sampling,
    /// Surface gradient and height.
    HeightGradient,
    /// Light position, direction or attenuation.
    LightVector,
    /// Reflectivity and working-space color arithmetic.
    ColorProduct,
}

/// Typed lighting refusal during intrinsic preparation or frame qualification.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum LightingPreparationError {
    /// A reached value is nonfinite or the material reflectivity is negative.
    InvalidParameter {
        /// Parameter being validated.
        parameter: LightingParameter,
        /// Supplied resolved value.
        value: f64,
    },
    /// Required lighting arithmetic could not be represented safely.
    Numeric {
        /// Numeric boundary that refused the value.
        phase: LightingNumericPhase,
        /// Supplied or intermediate value at that boundary.
        value: f64,
    },
}

impl LightingParameter {
    const fn label(self) -> &'static str {
        match self {
            Self::SurfaceScale => "surface scale",
            Self::DiffuseConstant => "diffuse constant",
            Self::SpecularConstant => "specular constant",
            Self::MaterialExponent => "material exponent",
            Self::KernelUnitLengthX => "horizontal kernel spacing",
            Self::KernelUnitLengthY => "vertical kernel spacing",
            Self::Azimuth => "azimuth",
            Self::Elevation => "elevation",
            Self::LightX => "light x-coordinate",
            Self::LightY => "light y-coordinate",
            Self::LightZ => "light z-coordinate",
            Self::PointsAtX => "spot target x-coordinate",
            Self::PointsAtY => "spot target y-coordinate",
            Self::PointsAtZ => "spot target z-coordinate",
            Self::SpotExponent => "spot focus exponent",
            Self::Cone => "limiting cone angle",
        }
    }
}

impl LightingNumericPhase {
    const fn label(self) -> &'static str {
        match self {
            Self::GridExtent => "grid extent",
            Self::GridMapping => "grid mapping",
            Self::GridIndex => "grid index",
            Self::Sampling => "sampling",
            Self::HeightGradient => "height and gradient",
            Self::LightVector => "light vector",
            Self::ColorProduct => "color product",
        }
    }
}

impl core::fmt::Display for LightingPreparationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidParameter { parameter, value } => write!(
                f,
                "lighting parameter {} has the invalid value {}",
                parameter.label(),
                Number(*value)
            ),
            Self::Numeric { phase, value } => write!(
                f,
                "lighting numeric check failed at {} for {}",
                phase.label(),
                Number(*value)
            ),
        }
    }
}

impl core::error::Error for LightingPreparationError {}

pub(crate) fn numeric_error(phase: LightingNumericPhase, value: f64) -> Error {
    LightingPreparationError::Numeric { phase, value }.into()
}

/// Admits diffuse lighting and its selected light through central accounting.
///
/// Preparation inspects height scale, reflectivity, horizontal and vertical
/// spacing when supplied, then the selected light. Distant inputs are inspected
/// in azimuth/elevation order. Point position and spot position/target are
/// inspected in x/y/z order, followed by the spot focus and optional cone.
/// An explicit finite spacing pair with any nonpositive value selects the
/// frame's default grid on both axes. That default places one kernel step per
/// local raster cell. Light RGB is interpreted in numeric sRGB and later
/// converted to the selected filter working space.
///
/// # Errors
///
/// Returns an accounting error when admission is refused, or a
/// [`LightingPreparationError`] diagnostic if a reached parameter is invalid
/// or the spot axis is unrepresentable.
pub fn prepare_lighting(
    input: DiffuseLightingInput,
    light: LightingLightInput,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedLighting> {
    prepare_lighting_with(input.surface, light, color_space, accounting, |account| {
        Ok(LightingMaterial::Diffuse {
            constant: inspect_reflectivity(
                input.constant,
                LightingParameter::DiffuseConstant,
                account,
            )?,
        })
    })
}

/// Admits specular lighting and its selected light through central accounting.
///
/// Preparation checks height scale, reflectivity and material exponent before
/// any explicit kernel spacing and light parameters. A finite material
/// exponent is clamped to `1..=128`, independently of spot focus. Grid spacing
/// and light color follow the same rules as [`prepare_lighting`].
///
/// # Errors
///
/// Returns an accounting error when admission is refused, or a
/// [`LightingPreparationError`] diagnostic if a reached parameter is invalid
/// or the spot axis is unrepresentable.
pub fn prepare_specular_lighting(
    input: SpecularLightingInput,
    light: LightingLightInput,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedLighting> {
    prepare_lighting_with(input.surface, light, color_space, accounting, |account| {
        Ok(LightingMaterial::Specular {
            constant: inspect_reflectivity(
                input.constant,
                LightingParameter::SpecularConstant,
                account,
            )?,
            exponent: inspect_exponent(
                input.exponent,
                LightingParameter::MaterialExponent,
                account,
            )?,
        })
    })
}

fn prepare_lighting_with(
    surface: LightingSurfaceInput,
    light: LightingLightInput,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
    admit_material: impl FnOnce(&Account) -> Result<LightingMaterial>,
) -> Result<PreparedLighting> {
    accounting.run(
        RenderAttemptKind::Lighting,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                prepare_lighting_in(surface, light, color_space, admit_material, account)
            })
        },
    )
}

fn prepare_lighting_in(
    surface: LightingSurfaceInput,
    light: LightingLightInput,
    color_space: FilterColorSpace,
    admit_material: impl FnOnce(&Account) -> Result<LightingMaterial>,
    account: &Account,
) -> Result<PreparedLighting> {
    let surface_scale = inspect(
        surface.surface_scale,
        LightingParameter::SurfaceScale,
        account,
    )?;
    let material = admit_material(account)?;
    let kernel = if let Some([x, y]) = surface.kernel {
        let x = inspect(x, LightingParameter::KernelUnitLengthX, account)?;
        let y = inspect(y, LightingParameter::KernelUnitLengthY, account)?;
        if x > 0.0 && y > 0.0 {
            KernelSpacing::PositivePair([x, y])
        } else {
            KernelSpacing::Default
        }
    } else {
        KernelSpacing::Default
    };
    let light = prepare_light(light, account)?;
    account.work(RenderWork::CoordinateVisits, 3)?;
    let rgb = [
        surface.lighting_color.r(),
        surface.lighting_color.g(),
        surface.lighting_color.b(),
    ];
    // The validated source channels are finite and in range. Alpha has no
    // lighting meaning. Absolute value only changes signed zeros here.
    let [r, g, b] = rgb.map(f32::abs);
    Ok(PreparedLighting {
        surface_scale,
        material,
        kernel,
        lighting_color: Color::rgba(r, g, b, 1.0),
        color_space,
        light,
    })
}

fn prepare_light(input: LightingLightInput, account: &Account) -> Result<PreparedLight> {
    const POSITION_PARAMETERS: [LightingParameter; 3] = [
        LightingParameter::LightX,
        LightingParameter::LightY,
        LightingParameter::LightZ,
    ];
    const TARGET_PARAMETERS: [LightingParameter; 3] = [
        LightingParameter::PointsAtX,
        LightingParameter::PointsAtY,
        LightingParameter::PointsAtZ,
    ];
    match input {
        LightingLightInput::Distant { azimuth, elevation } => {
            let azimuth = inspect(azimuth, LightingParameter::Azimuth, account)?;
            let elevation = inspect(elevation, LightingParameter::Elevation, account)?;
            let (sin_a, cos_a) = sin_cos_degrees(azimuth);
            let (sin_e, cos_e) = sin_cos_degrees(elevation);
            let direction = [cos_a * cos_e, sin_a * cos_e, sin_e].map(canonical_zero);
            Ok(PreparedLight::Distant { direction })
        }
        LightingLightInput::Point { position } => Ok(PreparedLight::Point {
            position: inspect_xyz(position, POSITION_PARAMETERS, account)?,
        }),
        LightingLightInput::Spot(spot) => {
            let position = inspect_xyz(spot.position, POSITION_PARAMETERS, account)?;
            let target = inspect_xyz(spot.points_at, TARGET_PARAMETERS, account)?;
            let exponent =
                inspect_exponent(spot.exponent, LightingParameter::SpotExponent, account)?;
            let cone_angle = spot
                .cone_angle
                .map(|angle| inspect(angle, LightingParameter::Cone, account))
                .transpose()?
                .map(|angle| angle.abs().min(90.0));
            Ok(PreparedLight::Spot {
                position,
                direction: spot_direction(position, target, cone_angle == Some(0.0), account)?,
                exponent,
                cone_angle,
            })
        }
    }
}

fn inspect(value: f64, parameter: LightingParameter, account: &Account) -> Result<f64> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    if !value.is_finite() {
        return Err(LightingPreparationError::InvalidParameter { parameter, value }.into());
    }
    Ok(canonical_zero(value))
}

fn inspect_reflectivity(
    value: f64,
    parameter: LightingParameter,
    account: &Account,
) -> Result<f64> {
    let value = inspect(value, parameter, account)?;
    if value < 0.0 {
        return Err(LightingPreparationError::InvalidParameter { parameter, value }.into());
    }
    Ok(value)
}

fn inspect_exponent(value: f64, parameter: LightingParameter, account: &Account) -> Result<f64> {
    Ok(inspect(value, parameter, account)?.clamp(1.0, 128.0))
}

fn inspect_xyz(
    [x, y, z]: [f64; 3],
    [px, py, pz]: [LightingParameter; 3],
    account: &Account,
) -> Result<[f64; 3]> {
    Ok([
        inspect(x, px, account)?,
        inspect(y, py, account)?,
        inspect(z, pz, account)?,
    ])
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

fn sin_cos_degrees(angle: f64) -> (f64, f64) {
    let reduced = angle % 360.0;
    let reduced = if reduced > 180.0 {
        reduced - 360.0
    } else if reduced <= -180.0 {
        reduced + 360.0
    } else {
        reduced
    };
    // Exact cardinal directions avoid spurious components at the horizon and poles.
    match reduced {
        0.0 => (0.0, 1.0),
        90.0 => (1.0, 0.0),
        180.0 => (0.0, -1.0),
        -90.0 => (-1.0, 0.0),
        _ => reduced.to_radians().sin_cos(),
    }
}

fn spot_direction(
    [x, y, z]: [f64; 3],
    [target_x, target_y, target_z]: [f64; 3],
    exact_ray: bool,
    account: &Account,
) -> Result<Option<[f64; 3]>> {
    account.work(RenderWork::CoordinateVisits, 3)?;
    let axis = [target_x - x, target_y - y, target_z - z];
    if let Some(value) = axis.into_iter().find(|value| !value.is_finite()) {
        return Err(numeric_error(LightingNumericPhase::LightVector, value));
    }
    let scale = axis.into_iter().map(f64::abs).fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(None);
    }
    if exact_ray {
        // Preserve the vector's component rounding so the shader can compare
        // the ray and the incident vector using the same representation.
        // A power-of-two rescale keeps extremely large or small axes usable.
        let axis_scale = if (f64::from(f32::MIN_POSITIVE)..=f64::from(f32::MAX)).contains(&scale) {
            1.0
        } else {
            f64::from_bits(scale.to_bits() & f64::INFINITY.to_bits()).max(f64::MIN_POSITIVE)
        };
        return Ok(Some(
            axis.map(|component| canonical_zero(component / axis_scale)),
        ));
    }
    let [x, y, z] = axis.map(|component| component / scale);
    let length = (x * x + y * y + z * z).sqrt();
    Ok(Some(
        [x / length, y / length, z / length].map(canonical_zero),
    ))
}

#[cfg(test)]
mod tests;
