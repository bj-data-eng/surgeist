//! Parameter admission and lattice construction for native Filter Effects turbulence.
//!
//! The pinned Filter Effects 1 § 9.21 algorithm supplies the random stream,
//! gradient rejection, frequency adjustment and stitch recurrence. Native
//! frame planning also qualifies the precision of the reached GPU samples.

use crate::accounting::{Account, RenderWork, SharedValue};
use crate::filter_domains::FilterDomains;
use crate::filter_model::FilterColorSpace;
use crate::geometry::Rect;
use crate::number_text::Number;
use crate::{Error, Result};

/// Selects how the Perlin samples are accumulated and mapped to straight RGBA.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TurbulenceKind {
    /// Accumulate absolute samples, then clamp each channel.
    #[default]
    Turbulence,
    /// Accumulate signed samples, map from −1..1 to 0..1, then clamp.
    FractalNoise,
}

/// Selects whether turbulence lattice edges are adjusted for repeating tiles.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TurbulenceStitch {
    /// Use the requested frequencies without tile adjustment.
    #[default]
    NoStitch,
    /// Adjust frequencies and wrap lattice neighbors using the primitive subregion.
    Stitch,
}

/// Immutable, resolved parameters for generating a source-free Perlin noise image.
///
/// The default requests zero base frequency, one octave, turbulence without stitching,
/// and linear sRGB. The renderer uses up to 16 octaves for its supported color depth.
/// Frame planning qualifies the actual sampling lattice before any GPU work.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turbulence {
    base_frequency: [f64; 2],
    seed: u32,
    octaves: u32,
    kind: TurbulenceKind,
    stitch: TurbulenceStitch,
    color_space: FilterColorSpace,
}

impl Turbulence {
    /// Admits X and Y base frequencies in layer-local units and a unitless seed.
    ///
    /// Returns [`crate::ErrorCode::InvalidInput`] for a negative or non-finite frequency
    /// or a non-finite seed. Truncates the seed toward zero and normalizes it to the
    /// generator interval defined by Filter Effects 1.
    pub fn try_new(base_frequency: [f64; 2], seed: f64) -> Result<Self> {
        for (axis, value) in ["x", "y"].into_iter().zip(base_frequency) {
            if !value.is_finite() || value < 0.0 {
                return Err(Error::invalid_value(
                    format!("turbulence base frequency {axis}"),
                    Number(value),
                    "must be finite and non-negative",
                ));
            }
        }
        if !seed.is_finite() {
            return Err(Error::invalid_value(
                "turbulence seed",
                Number(seed),
                "must be finite",
            ));
        }
        let seed = seed.trunc();
        let maximum = f64::from(RANDOM_MODULUS - 1);
        let normalized = if seed <= 0.0 {
            -(seed % maximum) + 1.0
        } else {
            seed.min(maximum)
        };
        Ok(Self {
            base_frequency,
            seed: normalized as u32,
            ..Self::default()
        })
    }

    /// Sets the requested octave count. Zero generates a constant image.
    #[must_use]
    pub const fn with_octaves(mut self, octaves: u32) -> Self {
        self.octaves = octaves;
        self
    }

    /// Sets the accumulation and channel mapping mode.
    #[must_use]
    pub const fn with_kind(mut self, kind: TurbulenceKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the tile stitching mode.
    #[must_use]
    pub const fn with_stitch(mut self, stitch: TurbulenceStitch) -> Self {
        self.stitch = stitch;
        self
    }

    /// Sets the space in which the straight RGBA channels are generated.
    #[must_use]
    pub const fn with_color_space(mut self, color_space: FilterColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    /// Returns the X and Y base frequencies before any stitching adjustment.
    #[must_use]
    pub const fn base_frequency(self) -> [f64; 2] {
        self.base_frequency
    }

    /// Returns the normalized pseudo-random generator seed.
    #[must_use]
    pub const fn seed(self) -> u32 {
        self.seed
    }

    /// Returns the requested octave count before the renderer's depth limit.
    #[must_use]
    pub const fn octaves(self) -> u32 {
        self.octaves
    }

    /// Returns the accumulation and channel mapping mode.
    #[must_use]
    pub const fn kind(self) -> TurbulenceKind {
        self.kind
    }

    /// Returns the tile stitching mode.
    #[must_use]
    pub const fn stitch(self) -> TurbulenceStitch {
        self.stitch
    }

    /// Returns the space in which the channels are generated.
    #[must_use]
    pub const fn color_space(self) -> FilterColorSpace {
        self.color_space
    }

    pub(crate) fn parameters(
        self,
        region: TurbulenceRegion,
        clip: [f64; 4],
        raster_edge: [f64; 2],
        raster_step: f64,
        extent: [u32; 2],
        account: &Account,
    ) -> Result<TurbulenceParameters> {
        let raster = |axis: usize| {
            AxisRaster::try_new(
                raster_edge[axis],
                raster_step,
                extent[axis],
                [clip[axis], clip[axis + 2]],
            )
        };
        let x_raster = raster(0)?;
        let y_raster = raster(1)?;
        let octaves = if self.base_frequency == [0.0; 2] {
            0
        } else {
            self.octaves.min(MAX_OCTAVES)
        };
        let (x, y, lattice) = if octaves == 0 {
            (NoiseAxis::default(), NoiseAxis::default(), None)
        } else {
            let primitive = region.primitive();
            let stitch = self.stitch == TurbulenceStitch::Stitch;
            let (x, x_error) = NoiseAxis::resolve(
                self.base_frequency[0],
                stitch.then_some([primitive.x(), primitive.width()]),
                x_raster,
                octaves,
            )?;
            let (y, y_error) = NoiseAxis::resolve(
                self.base_frequency[1],
                stitch.then_some([primitive.y(), primitive.height()]),
                y_raster,
                octaves,
            )?;
            self.qualify_phase(x_error, y_error, octaves)?;
            (x, y, Some(NoiseLattice::generate(self.seed, account)?))
        };

        account.work(RenderWork::SerializedBytes, TurbulenceParameters::SIZE)?;
        let mut bytes = [0; TurbulenceParameters::SIZE];
        write_words(&mut bytes, 0, clip.map(|edge| (edge as f32).to_bits()));
        write_words(
            &mut bytes,
            16,
            [x.phase, y.phase, x.step, y.step].map(f32::to_bits),
        );
        write_words(
            &mut bytes,
            32,
            [x.wrap, y.wrap, x.width, y.width].map(|value| value as u32),
        );
        write_words(
            &mut bytes,
            48,
            [
                octaves,
                u32::from(self.kind == TurbulenceKind::FractalNoise),
                u32::from(self.color_space == FilterColorSpace::LinearSrgb),
                0,
            ],
        );
        if let Some(lattice) = lattice {
            write_words(&mut bytes, PERMUTATION_OFFSET, lattice.permutation);
            for (index, channels) in lattice.gradients.into_iter().enumerate() {
                write_words(
                    &mut bytes,
                    GRADIENT_OFFSET + index * 16,
                    channels.map(f32::to_bits),
                );
            }
        }
        Ok(TurbulenceParameters {
            octaves,
            bytes: SharedValue::new(bytes, &account.storage())?,
        })
    }

    fn qualify_phase(self, x_error: f64, y_error: f64, octaves: u32) -> Result<()> {
        let mut weighted_displacement = 0.0;
        for octave in 0..octaves {
            let weight = f64::from(1_u32 << octave).recip();
            // A coordinate error amplified by the octave is attenuated by its
            // weight. Beyond one lattice cell, bound the full sample range.
            weighted_displacement += x_error.min(weight) + y_error.min(weight);
        }
        // Along either axis the noise derivative has a unit gradient term and
        // a cubic-fade term bounded by 1.5 * (1 + sqrt(2)). Five bounds their
        // sum as well as a full raw sample range (2 * sqrt(2)).
        let accumulation_gain = if self.kind == TurbulenceKind::FractalNoise {
            0.5
        } else {
            1.0
        };
        // The greatest slope of the linear-to-sRGB transfer is 12.92. Bound
        // phase error to one normalized 8-bit step in a straight output channel.
        let transfer_gain = if self.color_space == FilterColorSpace::LinearSrgb {
            12.92
        } else {
            1.0
        };
        let channel_error = 5.0 * weighted_displacement * accumulation_gain * transfer_gain;
        if channel_error > 1.0 / 255.0 {
            return Err(Error::invalid_value(
                "turbulence sampling precision",
                Number(channel_error),
                "phase rounding may exceed one normalized 8-bit channel step",
            ));
        }
        Ok(())
    }
}

impl Default for Turbulence {
    fn default() -> Self {
        Self {
            base_frequency: [0.0; 2],
            seed: 1,
            octaves: 1,
            kind: TurbulenceKind::Turbulence,
            stitch: TurbulenceStitch::NoStitch,
            color_space: FilterColorSpace::LinearSrgb,
        }
    }
}

/// Independent hard filter region and turbulence primitive subregion.
///
/// Their intersection clips the output. Stitching uses the original primitive subregion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurbulenceRegion(FilterDomains);

impl TurbulenceRegion {
    /// Admits two finite rectangles with nonnegative extents and finite maxima.
    ///
    /// Empty rectangles are valid and produce no output.
    pub fn try_new(filter: Rect, primitive: Rect) -> Result<Self> {
        FilterDomains::try_new(Rect::default(), filter, primitive).map(Self)
    }

    /// Returns the hard filter region.
    #[must_use]
    pub const fn filter(self) -> Rect {
        self.0.filter()
    }

    /// Returns the original primitive subregion.
    #[must_use]
    pub const fn primitive(self) -> Rect {
        self.0.result()
    }
}

// Filter Effects 1 § 9.21 permits limiting octaves by supported color depth.
const MAX_OCTAVES: u32 = 16;
const RANDOM_MODULUS: u32 = 2_147_483_647;
const LATTICE_SIZE: usize = 256;
const PERMUTATION_OFFSET: usize = 80;
const GRADIENT_OFFSET: usize = PERMUTATION_OFFSET + LATTICE_SIZE * 4;

#[derive(Clone, PartialEq)]
pub(crate) struct TurbulenceParameters {
    octaves: u32,
    bytes: SharedValue<[u8; Self::SIZE]>,
}

impl TurbulenceParameters {
    // Five header vec4s, 256 permutation words, and an X/Y vec4 pair per slot.
    pub(crate) const SIZE: usize = GRADIENT_OFFSET + LATTICE_SIZE * 2 * 16;

    pub(crate) fn as_bytes(&self) -> &[u8; Self::SIZE] {
        &self.bytes
    }

    pub(crate) const fn octaves(&self) -> u32 {
        self.octaves
    }
}

impl core::fmt::Debug for TurbulenceParameters {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TurbulenceParameters")
            .field("octaves", &self.octaves)
            .finish_non_exhaustive()
    }
}

fn write_words<const N: usize>(bytes: &mut [u8], offset: usize, values: [u32; N]) {
    for (word, value) in bytes[offset..offset + N * 4]
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(values)
    {
        word.copy_from_slice(&value.to_le_bytes());
    }
}

struct NoiseLattice {
    permutation: [u32; LATTICE_SIZE],
    gradients: [[f32; 4]; 2 * LATTICE_SIZE],
}

impl NoiseLattice {
    fn generate(mut seed: u32, account: &Account) -> Result<Self> {
        let mut gradients = [[0.0; 4]; 2 * LATTICE_SIZE];
        // The pinned algorithm consumes the same generator stream across R, G, B and A.
        for channel in 0..4 {
            for pair in gradients.as_chunks_mut::<2>().0 {
                loop {
                    account.work(RenderWork::TurbulenceLatticeCandidates, 1)?;
                    seed = next_seed(seed);
                    let x = (seed % 512) as i32 - 256;
                    seed = next_seed(seed);
                    let y = (seed % 512) as i32 - 256;
                    let squared = x * x + y * y;
                    if squared != 0 && squared <= 256 * 256 {
                        let magnitude = f64::from(squared).sqrt();
                        pair[0][channel] = (f64::from(x) / magnitude) as f32;
                        pair[1][channel] = (f64::from(y) / magnitude) as f32;
                        break;
                    }
                }
            }
        }
        let mut permutation = core::array::from_fn(|index| index as u32);
        for index in (1..LATTICE_SIZE).rev() {
            seed = next_seed(seed);
            permutation.swap(index, seed as usize % LATTICE_SIZE);
        }
        Ok(Self {
            permutation,
            gradients,
        })
    }
}

fn next_seed(seed: u32) -> u32 {
    // The Park–Miller multiplier and modulus fit in a widened integer product.
    ((u64::from(seed) * 16_807) % u64::from(RANDOM_MODULUS)) as u32
}

#[derive(Clone, Copy, Default)]
struct NoiseAxis {
    phase: f32,
    step: f32,
    wrap: i32,
    width: i32,
}

#[derive(Clone, Copy)]
struct AxisRaster {
    edge: f64,
    step: f64,
    exact_centers: [f64; 2],
    shader_centers: [f32; 2],
    has_interior: bool,
}

impl AxisRaster {
    fn try_new(edge: f64, step: f64, extent: u32, clip: [f64; 2]) -> Result<Self> {
        // This keeps the pixel center and both cell edges distinct in binary32.
        const MAX_RASTER_EXTENT: u32 = 1 << 23;
        let [lo, hi] = clip;
        if extent > MAX_RASTER_EXTENT || !(lo >= 0.0 && lo < hi && hi <= f64::from(extent)) {
            return Err(numeric("raster extent", f64::from(extent)));
        }
        let cells = [lo.floor(), hi.ceil() - 1.0];
        let exact_centers = cells.map(|cell| {
            let minimum = cell.max(lo);
            let maximum = (cell + 1.0).min(hi);
            minimum + (maximum - minimum) * 0.5
        });
        let [lo_f32, hi_f32] = clip.map(|value| value as f32);
        // Narrowing must preserve the reached cells as well as nonempty extent.
        if f64::from(lo_f32.floor()) != cells[0] {
            return Err(numeric("raster clip", lo));
        }
        if lo_f32 >= hi_f32 || f64::from(hi_f32.ceil() - 1.0) != cells[1] {
            return Err(numeric("raster clip", hi));
        }
        let shader_centers = cells.map(|cell| {
            let cell = cell as f32;
            let minimum = cell.max(lo_f32);
            let maximum = (cell + 1.0).min(hi_f32);
            minimum + (maximum - minimum) * 0.5
        });
        Ok(Self {
            edge,
            step,
            exact_centers,
            shader_centers,
            has_interior: cells[1] - cells[0] > 1.0,
        })
    }
}

impl NoiseAxis {
    fn resolve(
        frequency: f64,
        tile: Option<[f64; 2]>,
        raster: AxisRaster,
        octaves: u32,
    ) -> Result<(Self, f64)> {
        // A zero axis has no phase or stitching. The other axis may still generate noise.
        if frequency == 0.0 {
            return Ok((Self::default(), 0.0));
        }
        let (frequency, width) = match tile {
            Some([_, length]) => stitched_frequency(frequency, length)?,
            None => (frequency, 0.0),
        };
        let phase = rebased_phase(raster.edge, frequency)?;
        let step = raster.step * frequency;
        let phase_f32 = phase as f32;
        let step_f32 = step as f32;
        if !raster.step.is_finite() || raster.step <= 0.0 || !step_f32.is_finite() {
            return Err(numeric("sampling step", step));
        }

        let octave_scale = f64::from(1_u32 << (octaves - 1));
        let mut max_phase_error = 0.0_f64;
        let mut max_phase = 0.0_f32;
        for (exact_center, center) in raster.exact_centers.into_iter().zip(raster.shader_centers) {
            // Bound the real affine map and both rounded evaluations of WGSL fma.
            // With positive step the endpoint bounds also bound the interior.
            let exact = exact_center.mul_add(step, phase);
            let fused = center.mul_add(step_f32, phase_f32);
            let separate = center * step_f32 + phase_f32;
            for point in [exact, f64::from(fused), f64::from(separate)] {
                let reached = point * octave_scale;
                // At 2^23, binary32 spacing reaches one whole lattice cell.
                if !reached.is_finite() || reached.abs() >= f64::from(1_u32 << 23) {
                    return Err(numeric("lattice coordinate", reached));
                }
            }
            max_phase = max_phase.max(fused.abs()).max(separate.abs());
            max_phase_error = max_phase_error
                .max((exact - f64::from(fused)).abs())
                .max((exact - f64::from(separate)).abs());
        }
        if raster.has_interior {
            // Coefficient error grows at most linearly with the largest center.
            // Monotonicity then bounds each f32 rounding by its largest reached
            // half-ULP. Count both operations for a non-fused implementation.
            let farthest_center = raster.shader_centers[1];
            let coefficient_error = (phase - f64::from(phase_f32)).abs()
                + f64::from(farthest_center) * (step - f64::from(step_f32)).abs();
            let rounding_error = half_ulp(farthest_center * step_f32) + half_ulp(max_phase);
            max_phase_error = max_phase_error.max(coefficient_error + rounding_error);
        }
        // Binary32 targets may flush subnormal operands and intermediates.
        // Count three minimum normals for phase, product and sum and scale the
        // possible step/center flush by the other factor in the multiplication.
        max_phase_error += (3.0 + f64::from(raster.shader_centers[1]) + f64::from(step_f32))
            * f64::from(f32::MIN_POSITIVE);

        let (wrap, width) = match tile {
            Some([origin, _]) => {
                // PerlinN is a multiple of 256. Omitting it from both the sample
                // and wrap makes the spec's octave update a simple doubling.
                let tile_phase = rebased_phase(origin, frequency)?;
                let period = LATTICE_SIZE as f64;
                // Recover the integral number of periods between the two reductions.
                // Floor the tile phase before adding that integral translation so
                // rounding a translated coordinate cannot change its lattice cell.
                let translation = (origin - raster.edge).mul_add(frequency, phase - tile_phase);
                let first_lattice = tile_phase.floor() + (translation / period).round() * period;
                let wrap = first_lattice + width;
                for value in [first_lattice, wrap, width] {
                    let reached = value * octave_scale;
                    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&reached) {
                        return Err(numeric("stitch lattice", reached));
                    }
                }
                (wrap as i32, width as i32)
            }
            None => (0, 0),
        };
        Ok((
            Self {
                phase: phase_f32,
                step: step_f32,
                wrap,
                width,
            },
            max_phase_error,
        ))
    }
}

fn half_ulp(magnitude: f32) -> f64 {
    f64::from(magnitude.next_up() - magnitude) * 0.5
}

fn stitched_frequency(frequency: f64, length: f64) -> Result<(f64, f64)> {
    let tiles = length * frequency;
    if !tiles.is_finite() || length <= 0.0 {
        return Err(numeric("stitch period", tiles));
    }
    if tiles < 1.0 {
        return Ok((length.recip(), 1.0));
    }
    let low = tiles.floor();
    let high = tiles.ceil();
    let width = if frequency / (low / length) < (high / length) / frequency {
        low
    } else {
        high
    };
    Ok((width / length, width))
}

fn rebased_phase(edge: f64, frequency: f64) -> Result<f64> {
    let origin = edge * frequency;
    if !origin.is_finite() {
        return Err(numeric("sampling origin", origin));
    }
    // Reduce the product and its rounding residual separately. At distant origins
    // the product alone can round away the lattice index or the fractional phase.
    let period = LATTICE_SIZE as f64;
    let residual = edge.mul_add(frequency, -origin);
    let phase = (origin % period + residual % period) % period;
    Ok(if phase >= period * 0.5 {
        phase - period
    } else if phase < -period * 0.5 {
        phase + period
    } else {
        phase
    })
}

fn numeric(field: &str, value: f64) -> Error {
    Error::invalid_value(
        format!("turbulence {field}"),
        Number(value),
        "cannot represent the reached sampling lattice",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
        RenderPhase,
    };

    fn parameters(
        operation: Turbulence,
        accounting: &mut RenderAccountingContext,
    ) -> Result<TurbulenceParameters> {
        let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
        accounting.run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
            operation.parameters(
                TurbulenceRegion::try_new(rect, rect).unwrap(),
                [0.0, 0.0, 1.0, 1.0],
                [0.0; 2],
                1.0,
                [1; 2],
                account,
            )
        })
    }

    fn words<const N: usize>(parameters: &TurbulenceParameters, offset: usize) -> [u32; N] {
        let bytes = parameters.as_bytes();
        core::array::from_fn(|index| {
            let start = offset + index * 4;
            u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap())
        })
    }

    #[test]
    fn seed_truncates_then_normalizes_and_invalid_fields_are_typed() {
        for (authored, normalized) in [
            (-0.99, 1),
            (-1.99, 2),
            (-2_147_483_647.9, 2),
            (3.9, 3),
            (f64::MAX, 2_147_483_646),
        ] {
            assert_eq!(
                Turbulence::try_new([0.0; 2], authored).unwrap().seed(),
                normalized
            );
        }
        assert_eq!(
            Turbulence::try_new([0.0; 2], 0.0).unwrap(),
            Turbulence::default()
        );
        for (frequency, seed, field) in [
            ([-1.0, 0.0], 0.0, "turbulence base frequency x"),
            ([0.0, f64::INFINITY], 0.0, "turbulence base frequency y"),
            ([0.0; 2], f64::NAN, "turbulence seed"),
        ] {
            let error = Turbulence::try_new(frequency, seed).unwrap_err();
            assert_eq!(error.invalid_value_diagnostic().unwrap().field(), field);
        }
    }

    #[test]
    fn park_miller_matches_pinned_ten_thousandth_value() {
        let mut seed = 1;
        for _ in 0..10_000 {
            seed = next_seed(seed);
        }
        assert_eq!(seed, 1_043_618_065);
    }

    #[test]
    fn four_consecutive_channel_streams_and_rejected_candidates_match_pinned_algorithm() {
        // The pinned four-channel loop, evaluated independently with widened integer
        // products, accepts 1024 of the first 1321 candidates for normalized seed 1.
        let operation = Turbulence::try_new([0.25, 0.0], 1.0).unwrap();
        let options = |cap| {
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::TurbulenceLatticeCandidates, cap)
                    .unwrap(),
            )
        };
        let mut accounting = RenderAccountingContext::new(options(1320));
        let error = parameters(operation, &mut accounting).unwrap_err();
        assert!(error.accounting_diagnostic().is_some());

        accounting.set_options(options(1321));
        let prepared = parameters(operation, &mut accounting).unwrap();
        assert_eq!(
            words::<8>(&prepared, PERMUTATION_OFFSET),
            [0, 209, 23, 69, 96, 172, 177, 198]
        );
        let x = words::<4>(&prepared, GRADIENT_OFFSET).map(f32::from_bits);
        let y = words::<4>(&prepared, GRADIENT_OFFSET + 16).map(f32::from_bits);
        let expected = [
            (167.0_f64, -15.0),
            (208.0, 57.0),
            (106.0, 69.0),
            (-207.0, 21.0),
        ];
        for (channel, (gx, gy)) in expected.into_iter().enumerate() {
            let length = gx.hypot(gy);
            assert_eq!(x[channel], (gx / length) as f32);
            assert_eq!(y[channel], (gy / length) as f32);
        }
        let usage = accounting
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage();
        assert_eq!(usage.work(RenderWork::TurbulenceLatticeCandidates), 1321);
        assert_eq!(usage.work(RenderWork::SerializedBytes), 9296);
    }

    #[test]
    fn constant_cases_skip_lattice_even_with_zero_candidate_budget() {
        let options = RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::TurbulenceLatticeCandidates, 0)
                .unwrap(),
        );
        let mut accounting = RenderAccountingContext::new(options);
        for operation in [
            Turbulence::default().with_octaves(u32::MAX),
            Turbulence::try_new([f64::MAX, 0.0], 1.0)
                .unwrap()
                .with_octaves(0)
                .with_kind(TurbulenceKind::FractalNoise),
        ] {
            let prepared = parameters(operation, &mut accounting).unwrap();
            assert_eq!(prepared.octaves(), 0);
            assert_eq!(
                words::<4>(&prepared, 48),
                [
                    0,
                    u32::from(operation.kind() == TurbulenceKind::FractalNoise),
                    1,
                    0
                ]
            );
        }
    }

    #[test]
    fn stitch_uses_negative_primitive_origin_and_ignores_cropped_filter_for_period() {
        let operation = Turbulence::try_new([0.4, 0.61], 0.0)
            .unwrap()
            .with_stitch(TurbulenceStitch::Stitch);
        let filter = Rect::new(0.25, -0.875, 1.75, 2.875);
        let primitive = Rect::new(-1.25, -2.5, 4.0, 5.0);
        let region = TurbulenceRegion::try_new(filter, primitive).unwrap();
        assert_eq!(region.filter(), filter);
        assert_eq!(region.primitive(), primitive);
        let prepared = RenderAccountingContext::default()
            .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                operation.parameters(
                    region,
                    [0.25, 0.125, 2.0, 3.0],
                    [0.0, -1.0],
                    1.0,
                    [2, 3],
                    account,
                )
            })
            .unwrap();
        // A 4-unit tile selects 2 cells at 0.5, a 5-unit tile selects 3 at 0.6.
        // The first lattice indices are floor(-1.25 * 0.5) and floor(-2.5 * 0.6).
        assert_eq!(
            words::<4>(&prepared, 16).map(f32::from_bits),
            [0.0, -0.6, 0.5, 0.6]
        );
        assert_eq!(words::<4>(&prepared, 32), [1, 1, 2, 3]);
    }

    #[test]
    fn distant_phase_preserves_fraction_before_narrowing() {
        // Multiplying the binary64 representation of 0.3 by (2^52 + 1) gives
        // a fraction of 0.05 in the modulo-256 lattice. Ordinary binary64
        // multiplication loses that fraction before a modulo operation.
        let edge = ((1_u64 << 52) + 1) as f64;
        assert!((rebased_phase(edge, 0.3).unwrap() - -50.95).abs() < 1e-12);
        assert!((rebased_phase(-edge, 0.3).unwrap() - 50.95).abs() < 1e-12);
    }

    #[test]
    fn precision_refuses_rounding_off_a_visible_fractional_lattice_sample() {
        let mut accounting = RenderAccountingContext::default();
        let exact = Turbulence::try_new([8_388_608.0, 0.0], 1.0)
            .unwrap()
            .with_kind(TurbulenceKind::FractalNoise)
            .with_color_space(FilterColorSpace::Srgb);
        assert!(parameters(exact, &mut accounting).is_ok());

        // At center 0.5 the narrowed step erases 0.1875 of a lattice cell.
        // The sample is inside the numeric range, but the false lattice vertex
        // would replace Perlin variation with all four channels at 0.5.
        let fractional = Turbulence::try_new([8_388_608.375, 0.0], 1.0)
            .unwrap()
            .with_kind(TurbulenceKind::FractalNoise)
            .with_color_space(FilterColorSpace::Srgb);
        let error = parameters(fractional, &mut accounting).unwrap_err();
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "turbulence sampling precision"
        );
    }

    #[test]
    fn precision_includes_the_narrowing_of_a_fractional_cell_centroid() {
        // Narrowing the high edge to 0.5 shifts the true centroid by 2^-28.
        // At this frequency its 2^-8 lattice displacement exceeds a color step.
        let raster = AxisRaster::try_new(0.0, 1.0, 1, [0.0, 0.5 + 2.0_f64.powi(-27)]).unwrap();
        let (_, error_bound) = NoiseAxis::resolve(f64::from(1_u32 << 20), None, raster, 1).unwrap();
        let operation = Turbulence::default().with_color_space(FilterColorSpace::Srgb);
        let error = operation.qualify_phase(error_bound, 0.0, 1).unwrap_err();
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "turbulence sampling precision"
        );
    }

    #[test]
    fn octave_limit_qualifies_coordinate_and_stitch_at_last_reached_octave() {
        let mut accounting = RenderAccountingContext::default();
        let operation = Turbulence::try_new([256.0, 0.0], 0.0)
            .unwrap()
            .with_octaves(u32::MAX);
        assert_eq!(
            parameters(operation, &mut accounting).unwrap().octaves(),
            16
        );
        let operation = Turbulence::try_new([512.0, 0.0], 0.0)
            .unwrap()
            .with_octaves(16);
        let error = parameters(operation, &mut accounting).unwrap_err();
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "turbulence lattice coordinate"
        );

        let operation = Turbulence::try_new([1.0, 0.0], 0.0)
            .unwrap()
            .with_octaves(16)
            .with_stitch(TurbulenceStitch::Stitch);
        let filter = Rect::new(0.0, 0.0, 1.0, 1.0);
        let primitive = Rect::new(0.0, 0.0, 65_536.0, 1.0);
        let region = TurbulenceRegion::try_new(filter, primitive).unwrap();
        let error = accounting
            .run(RenderAttemptKind::Frame, RenderPhase::Plan, |account| {
                operation.parameters(region, [0.0, 0.0, 1.0, 1.0], [0.0; 2], 1.0, [1; 2], account)
            })
            .unwrap_err();
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "turbulence stitch lattice"
        );
    }
}
