//! Native straight-byte displacement admission and authentic first-input ownership.
use crate::accounting::{
    Account, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork,
};
use crate::filter_source::FilterSourceDescriptor;
use crate::image::RawImageUploadDescriptor;
use crate::{
    Color, FilterColorSpace, FilterDomains, GaussianPreparationError, GaussianSource, Rect, Result,
    Scene, Size,
};
use std::sync::Arc;

/// Straight byte channel selected from the displacement map.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DisplacementChannel {
    /// Red, with map-only color conversion.
    R,
    /// Green, with map-only color conversion.
    G,
    /// Blue, with map-only color conversion.
    B,
    /// Covered alpha, without color conversion.
    #[default]
    A,
}
impl DisplacementChannel {
    pub(crate) const fn index(self) -> u32 {
        match self {
            Self::R => 0,
            Self::G => 1,
            Self::B => 2,
            Self::A => 3,
        }
    }
}

/// Displacement parameter whose supplied resolved value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplacementParameter {
    /// Signed scale in layer-local primitive units.
    Scale,
}

/// Reached native displacement arithmetic that cannot be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplacementNumericPhase {
    /// Source, period, density or common-lattice mapping.
    GridMapping,
    /// Native map records, addressing or positive coverage fractions.
    MapMapping,
    /// Ordered coefficient arithmetic, address tables or their storage layout.
    TableLayout,
    /// Reachable integer source addresses.
    GridIndex,
    /// Uploaded arithmetic and coverage products.
    Sampling,
}

/// Typed native displacement admission or reached numeric refusal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplacementPreparationError {
    /// The supplied scale is not finite.
    InvalidParameter {
        /// Inspected parameter.
        parameter: DisplacementParameter,
        /// Supplied value.
        value: f64,
    },
    /// Native pixel dimensions are not finite nonnegative integer u32 values.
    InvalidMapDimensions {
        /// Supplied native pixel dimensions.
        size: Size,
    },
    /// The exact nonzero native payload length is not representable.
    MapPayloadOverflow {
        /// Supplied native pixel dimensions.
        size: Size,
    },
    /// The native RGBA8 payload does not have the exact required length.
    InvalidMapPayload {
        /// Required byte count.
        expected: u64,
        /// Supplied byte count.
        actual: u64,
    },
    /// The map rectangle has an invalid extent, origin or derived maximum.
    InvalidMapDomain {
        /// Supplied layer-local map rectangle.
        map: Rect,
    },
    /// Reached arithmetic cannot be represented safely.
    Numeric {
        /// Boundary that refused representation.
        phase: DisplacementNumericPhase,
        /// Supplied or intermediate value at that boundary.
        value: f64,
    },
    /// An authentic periodic source requires positive width and height.
    EmptyPeriod,
    /// Same-layer backdrop is incompatible with a declared infinite input.
    InfiniteSourceBackdrop,
}

impl DisplacementParameter {
    const fn label(self) -> &'static str {
        match self {
            Self::Scale => "scale",
        }
    }
}

impl DisplacementNumericPhase {
    const fn label(self) -> &'static str {
        match self {
            Self::GridMapping => "grid mapping",
            Self::MapMapping => "map mapping",
            Self::TableLayout => "table layout",
            Self::GridIndex => "grid index",
            Self::Sampling => "sampling",
        }
    }
}

impl core::fmt::Display for DisplacementPreparationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidParameter { parameter, value } => write!(
                f,
                "displacement parameter {} has the invalid value {value}",
                parameter.label()
            ),
            Self::InvalidMapDimensions { size } => write!(
                f,
                "displacement map size {} by {} is not a pair of finite nonnegative integers that fit in u32",
                size.width(),
                size.height()
            ),
            Self::MapPayloadOverflow { size } => write!(
                f,
                "the byte length of a displacement map of {} by {} pixels cannot be represented",
                size.width(),
                size.height()
            ),
            Self::InvalidMapPayload { expected, actual } => write!(
                f,
                "displacement map has {actual} bytes but its dimensions require {expected}"
            ),
            Self::InvalidMapDomain { .. } => {
                f.write_str("displacement map rectangle has an invalid extent, origin or maximum")
            }
            Self::Numeric { phase, .. } => {
                write!(f, "displacement numeric check failed at {}", phase.label())
            }
            Self::EmptyPeriod => {
                f.write_str("a periodic source must have positive width and height")
            }
            Self::InfiniteSourceBackdrop => f.write_str(
                "a same-layer backdrop cannot be combined with a declared infinite source",
            ),
        }
    }
}

impl core::error::Error for DisplacementPreparationError {}

/// Untrusted resolved displacement scale and independent map channels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplacementInput {
    scale: f64,
    channels: [DisplacementChannel; 2],
}
impl DisplacementInput {
    /// Requests zero scale and alpha for both axes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scale: 0.0,
            channels: [DisplacementChannel::A; 2],
        }
    }
    /// Sets the signed scale in layer-local primitive units.
    #[must_use]
    pub const fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
    /// Sets independent horizontal and vertical map channels.
    #[must_use]
    pub const fn with_channels(mut self, x: DisplacementChannel, y: DisplacementChannel) -> Self {
        self.channels = [x, y];
        self
    }
}
impl Default for DisplacementInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable finite displacement parameters, independent of actual frame grids.
///
/// Signed finite scales have no arbitrary magnitude cap. Reached grid and
/// ordered binary64 address arithmetic are qualified during frame planning.
/// The selected color space applies only to map RGB, never first-input pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedDisplacement {
    scale: f64,
    channels: [DisplacementChannel; 2],
    color_space: FilterColorSpace,
}
impl PreparedDisplacement {
    pub(crate) const fn scale(self) -> f64 {
        self.scale
    }
    pub(crate) const fn channels(self) -> [DisplacementChannel; 2] {
        self.channels
    }
    pub(crate) const fn color_space(self) -> FilterColorSpace {
        self.color_space
    }
    pub(crate) fn is_passthrough(self) -> bool {
        self.scale == 0.0
    }
}

/// Admits a finite signed scale through central accounting.
///
/// Observes one `FilterOperationVisits` and one reached `CoordinateVisits`.
/// Zero scale selects bounded pass-through without map execution.
pub fn prepare_displacement(
    input: DisplacementInput,
    map_color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedDisplacement> {
    accounting.run(
        RenderAttemptKind::Displacement,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                account.work(RenderWork::CoordinateVisits, 1)?;
                if !input.scale.is_finite() {
                    return Err(DisplacementPreparationError::InvalidParameter {
                        parameter: DisplacementParameter::Scale,
                        value: input.scale,
                    }
                    .into());
                }
                Ok(PreparedDisplacement {
                    scale: input.scale,
                    channels: input.channels,
                    color_space: map_color_space,
                })
            })
        },
    )
}

/// Immutable native-authored straight RGBA8 map, or a closed pass-through request.
///
/// Native bytes declare a value, not a document-origin or filter-taint
/// certificate. There is no implicit Image or Scene conversion. Native exterior
/// and zero-sized maps supply zero channel data, which can displace the source;
/// they do not select semantic pass-through. Clones share accounted backing.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedDisplacementMap {
    value: MapValue,
}
#[derive(Clone, Debug, PartialEq)]
enum MapValue {
    Native {
        upload: RawImageUploadDescriptor,
        map: Rect,
    },
    SourceGraphic,
    SourceAlpha,
    Suppressed,
}
impl PreparedDisplacementMap {
    /// Refers to this receiver's actual first input; the tainted map is bypassed.
    #[must_use]
    pub const fn source_graphic() -> Self {
        Self {
            value: MapValue::SourceGraphic,
        }
    }
    /// Refers to this receiver's first-input alpha; the tainted map is bypassed.
    #[must_use]
    pub const fn source_alpha() -> Self {
        Self {
            value: MapValue::SourceAlpha,
        }
    }
    /// Conservatively requests pass-through without claiming document provenance.
    #[must_use]
    pub const fn suppressed() -> Self {
        Self {
            value: MapValue::Suppressed,
        }
    }
    pub(crate) fn is_passthrough(&self) -> bool {
        !matches!(self.value, MapValue::Native { .. })
    }
    pub(crate) fn native_parts(&self) -> Option<(&RawImageUploadDescriptor, Rect)> {
        match &self.value {
            MapValue::Native { upload, map } => Some((upload, *map)),
            _ => None,
        }
    }
    fn is_zero_map(&self) -> bool {
        self.native_parts().is_some_and(|(upload, map)| {
            let size = upload.physical_size();
            size.width() == 0 || size.height() == 0 || rect_empty(map)
        })
    }
}

/// Admits immutable native-authored straight numeric-sRGB RGBA8 bytes and M.
///
/// Dimensions are exact nonnegative integer pixels; payload length must equal
/// width × height × 4. Either zero dimension requires empty bytes. An empty M
/// supplies zero map data while retaining the independently admitted payload.
/// Fixed dimensions, payload length and domain coordinates are validated first.
/// Actual byte custody is then admitted before the metered identity scan;
/// refusal disposes the incoming payload before releasing that custody.
pub fn prepare_native_displacement_map(
    size: Size,
    bytes: Arc<[u8]>,
    map: Rect,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedDisplacementMap> {
    accounting.run(
        RenderAttemptKind::DisplacementMap,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                for (value, extent) in [
                    (map.x(), false),
                    (map.y(), false),
                    (map.width(), true),
                    (map.height(), true),
                    (map.max().x(), false),
                    (map.max().y(), false),
                ] {
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    if !value.is_finite() || (extent && value < 0.0) {
                        return Err(DisplacementPreparationError::InvalidMapDomain { map }.into());
                    }
                }
                let upload = RawImageUploadDescriptor::prepare_native_in(size, bytes, account)?;
                Ok(PreparedDisplacementMap {
                    value: MapValue::Native { upload, map },
                })
            })
        },
    )
}

/// Authentic finite, constant or fundamental-period first input.
///
/// Finite input has transparent exterior. All variants are hard-clipped to F
/// before displacement; infinite declarations do not bypass that clip. The
/// actual Scene moves into the existing iterative Layer child owner.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplacementSource(GaussianSource);
impl DisplacementSource {
    /// Owns finite input with transparent exterior.
    #[must_use]
    pub fn finite(scene: Scene, domains: FilterDomains) -> Self {
        Self(GaussianSource::finite(scene, domains))
    }
    /// Declares authentic constant input with independent finite F and P.
    pub fn constant(color: Color, filter: Rect, result: Rect) -> Result<Self> {
        GaussianSource::constant(color, filter, result).map(Self)
    }
    /// Owns an actual fundamental tile with periodic continuation before F.
    pub fn periodic(scene: Scene, period: Rect, filter: Rect, result: Rect) -> Result<Self> {
        GaussianSource::periodic(scene, period, filter, result)
            .map(Self)
            .map_err(|error| {
                if matches!(
                    error.gaussian_diagnostic(),
                    Some(GaussianPreparationError::EmptyPeriod)
                ) {
                    DisplacementPreparationError::EmptyPeriod.into()
                } else {
                    error
                }
            })
    }
    pub(crate) const fn descriptor(&self) -> FilterSourceDescriptor {
        self.0.descriptor()
    }
    pub(crate) fn into_parts(self) -> (Scene, FilterSourceDescriptor) {
        self.0.into_parts()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplacementForeground {
    pub(crate) operation: PreparedDisplacement,
    pub(crate) source: FilterSourceDescriptor,
    pub(crate) map: PreparedDisplacementMap,
}
/// The normalized route owns only resources that this operation can consume.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NormalizedDisplacementForeground {
    pub(crate) operation: PreparedDisplacement,
    pub(crate) source: FilterSourceDescriptor,
    pub(crate) map: NormalizedDisplacementMap,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NormalizedDisplacementMap {
    Empty,
    Bypass,
    ZeroNative,
    Native {
        upload: RawImageUploadDescriptor,
        map: Rect,
    },
}
impl NormalizedDisplacementMap {
    pub(crate) const fn is_passthrough(&self) -> bool {
        matches!(self, Self::Bypass)
    }
    pub(crate) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
    pub(crate) fn native_parts(&self) -> Option<(&RawImageUploadDescriptor, Rect)> {
        match self {
            Self::Native { upload, map } => Some((upload, *map)),
            _ => None,
        }
    }
}
impl DisplacementForeground {
    /// Selects the declared execution route before importing or retaining pixels.
    /// Source validation and child normalization remain with their existing owners.
    pub(crate) fn normalized_in(
        &self,
        account: &Account,
        source_available: bool,
    ) -> Result<NormalizedDisplacementForeground> {
        let empty_source = match self.source {
            FilterSourceDescriptor::Finite { domains, .. } => {
                !source_available || !rect_overlap(domains.source(), domains.filter())
            }
            FilterSourceDescriptor::Constant { color, .. } => color.a() == 0.0,
            FilterSourceDescriptor::Periodic { .. } => !source_available,
        };
        let map = if empty_source || !rect_overlap(self.source.filter(), self.source.result()) {
            NormalizedDisplacementMap::Empty
        } else if self.operation.is_passthrough() || self.map.is_passthrough() {
            NormalizedDisplacementMap::Bypass
        } else if self.map.is_zero_map()
            || self
                .map
                .native_parts()
                .is_some_and(|(_, map)| !rect_overlap(map, self.source.filter()))
        {
            NormalizedDisplacementMap::ZeroNative
        } else {
            match &self.map.value {
                MapValue::Native { upload, map } => NormalizedDisplacementMap::Native {
                    upload: upload.imported_in(account)?,
                    map: *map,
                },
                MapValue::SourceGraphic | MapValue::SourceAlpha | MapValue::Suppressed => {
                    NormalizedDisplacementMap::Bypass
                }
            }
        };
        Ok(NormalizedDisplacementForeground {
            operation: self.operation,
            source: self.source,
            map,
        })
    }
}

fn rect_empty(rect: Rect) -> bool {
    rect.width() == 0.0 || rect.height() == 0.0
}
fn rect_overlap(a: Rect, b: Rect) -> bool {
    !rect_empty(a)
        && !rect_empty(b)
        && a.x().max(b.x()) < a.max().x().min(b.max().x())
        && a.y().max(b.y()) < a.max().y().min(b.max().y())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::LayerFilter;
    use crate::scene::Command;
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, Capabilities, ErrorCode, FilterList, Layer,
        PrimitiveFamily, PrimitiveOperation, RenderAccountingOptions, RenderAttemptStatus,
        RenderLimits, RenderResource, UnsupportedPrimitive,
    };

    fn domains() -> FilterDomains {
        FilterDomains::try_new(
            Rect::new(-2.0, -1.0, 8.0, 6.0),
            Rect::new(-1.0, -2.0, 6.0, 7.0),
            Rect::new(0.0, 0.0, 3.0, 2.0),
        )
        .unwrap()
    }
    fn prepare(scale: f64) -> PreparedDisplacement {
        prepare_displacement(
            DisplacementInput::new().with_scale(scale),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap()
    }
    fn native(size: Size, bytes: Arc<[u8]>, map: Rect) -> PreparedDisplacementMap {
        prepare_native_displacement_map(size, bytes, map, &mut RenderAccountingContext::default())
            .unwrap()
    }
    fn ink() -> Scene {
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        scene
    }
    fn normalize(value: &DisplacementForeground) -> NormalizedDisplacementForeground {
        RenderAccountingContext::default()
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::Normalize,
                |account| value.normalized_in(account, true),
            )
            .unwrap()
    }

    #[test]
    fn defaults_are_zero_alpha_and_finite_signed_scales_remain_unbounded() {
        assert_eq!(DisplacementInput::new(), DisplacementInput::default());
        assert_eq!(DisplacementChannel::default(), DisplacementChannel::A);
        let default = prepare(0.0);
        assert!(default.is_passthrough());
        assert_eq!(default.channels(), [DisplacementChannel::A; 2]);
        for scale in [
            -f64::MAX,
            -4.25,
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            4.25,
            f64::MAX,
        ] {
            let value = prepare_displacement(
                DisplacementInput::new()
                    .with_scale(scale)
                    .with_channels(DisplacementChannel::R, DisplacementChannel::B),
                FilterColorSpace::LinearSrgb,
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
            assert_eq!(value.scale().to_bits(), scale.to_bits());
            assert_eq!(
                value.channels(),
                [DisplacementChannel::R, DisplacementChannel::B]
            );
            assert_eq!(value.color_space(), FilterColorSpace::LinearSrgb);
            assert_eq!(value.is_passthrough(), scale == 0.0);
        }
        assert_eq!(
            [
                DisplacementChannel::R.index(),
                DisplacementChannel::G.index(),
                DisplacementChannel::B.index(),
                DisplacementChannel::A.index()
            ],
            [0, 1, 2, 3]
        );
    }

    #[test]
    fn nonfinite_scale_is_typed_and_effect_admission_spends_exactly_two_visits() {
        for scale in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut context = RenderAccountingContext::default();
            let error = prepare_displacement(
                DisplacementInput::new().with_scale(scale),
                FilterColorSpace::Srgb,
                &mut context,
            )
            .unwrap_err();
            let Some(DisplacementPreparationError::InvalidParameter { parameter, value }) =
                error.displacement_diagnostic()
            else {
                panic!("missing displacement diagnostic")
            };
            assert_eq!(*parameter, DisplacementParameter::Scale);
            assert_eq!(value.to_bits(), scale.to_bits());
            assert_eq!(
                context.last_attempt().unwrap().kind(),
                RenderAttemptKind::Displacement
            );
            assert_eq!(
                context.last_attempt().unwrap().status(),
                RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
            );
        }
        let exact = RenderLimits::new()
            .with_work_limit(RenderWork::FilterOperationVisits, 1)
            .unwrap()
            .with_work_limit(RenderWork::CoordinateVisits, 1)
            .unwrap()
            .with_retained_bytes(0)
            .with_scratch_bytes(0);
        let options = RenderAccountingOptions::new().with_limits(exact);
        let mut context = RenderAccountingContext::new(options);
        let expected = prepare_displacement(
            DisplacementInput::new().with_scale(-f64::MAX),
            FilterColorSpace::Srgb,
            &mut context,
        )
        .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::FilterOperationVisits), 1);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 1);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        for work in [
            RenderWork::FilterOperationVisits,
            RenderWork::CoordinateVisits,
        ] {
            context.set_options(options.with_limits(exact.with_work_limit(work, 0).unwrap()));
            assert!(
                prepare_displacement(
                    DisplacementInput::new().with_scale(-f64::MAX),
                    FilterColorSpace::Srgb,
                    &mut context
                )
                .unwrap_err()
                .accounting_diagnostic()
                .is_some()
            );
            context.set_options(options);
            assert_eq!(
                prepare_displacement(
                    DisplacementInput::new().with_scale(-f64::MAX),
                    FilterColorSpace::Srgb,
                    &mut context
                )
                .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn native_map_retains_exact_straight_bytes_and_zero_axes_require_empty_payload() {
        let bytes: Arc<[u8]> = Arc::from([255, 20, 64, 1]);
        let weak = Arc::downgrade(&bytes);
        let map = native(Size::new(1.0, 1.0), bytes, Rect::new(2.5, -4.0, 7.0, 3.0));
        let copy = map.clone();
        assert_eq!(copy.native_parts().unwrap().0.bytes(), &[255, 20, 64, 1]);
        assert!(!map.is_passthrough());
        drop(map);
        assert!(weak.upgrade().is_some());
        drop(copy);
        assert!(weak.upgrade().is_none());
        for size in [
            Size::new(0.0, 1.0),
            Size::new(1.0, 0.0),
            Size::new(0.0, f64::from(u32::MAX)),
            Size::new(f64::from(u32::MAX), 0.0),
        ] {
            let map = native(size, Arc::from([]), domains().source());
            assert!(map.is_zero_map());
            assert!(!map.is_passthrough());
            let error = prepare_native_displacement_map(
                size,
                Arc::from([0]),
                domains().source(),
                &mut RenderAccountingContext::default(),
            )
            .unwrap_err();
            assert_eq!(
                error.displacement_diagnostic(),
                Some(&DisplacementPreparationError::InvalidMapPayload {
                    expected: 0,
                    actual: 1
                })
            );
        }
        let empty_domain = native(
            Size::new(1.0, 1.0),
            Arc::from([1, 2, 3, 4]),
            Rect::default(),
        );
        assert!(empty_domain.is_zero_map());
        assert_eq!(
            empty_domain.native_parts().unwrap().0.bytes(),
            &[1, 2, 3, 4]
        );
    }

    #[test]
    fn map_fixed_input_errors_precede_retention_and_byte_visits() {
        for size in [
            Size::new(-1.0, 1.0),
            Size::new(1.5, 1.0),
            Size::new(f64::INFINITY, 1.0),
            Size::new(f64::from(u32::MAX) + 1.0, 1.0),
        ] {
            let mut context = RenderAccountingContext::default();
            let error = prepare_native_displacement_map(
                size,
                Arc::from([]),
                domains().source(),
                &mut context,
            )
            .unwrap_err();
            assert_eq!(
                error.displacement_diagnostic(),
                Some(&DisplacementPreparationError::InvalidMapDimensions { size })
            );
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::DisplacementMapByteVisits), 0);
            assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        }
        let huge = Size::new(f64::from(u32::MAX), f64::from(u32::MAX));
        let error = prepare_native_displacement_map(
            huge,
            Arc::from([]),
            domains().source(),
            &mut RenderAccountingContext::default(),
        )
        .unwrap_err();
        assert_eq!(
            error.displacement_diagnostic(),
            Some(&DisplacementPreparationError::MapPayloadOverflow { size: huge })
        );
        let bad_domain = Rect::new(0.0, 0.0, -1.0, 2.0);
        let error = prepare_native_displacement_map(
            Size::new(1.0, 1.0),
            Arc::from([0; 4]),
            bad_domain,
            &mut RenderAccountingContext::default(),
        )
        .unwrap_err();
        assert_eq!(
            error.displacement_diagnostic(),
            Some(&DisplacementPreparationError::InvalidMapDomain { map: bad_domain })
        );
    }

    #[test]
    fn native_byte_scan_exact_budget_refuses_one_under_and_identical_retry_succeeds() {
        let exact = RenderLimits::new()
            .with_work_limit(RenderWork::DisplacementMapByteVisits, 4)
            .unwrap();
        let options = RenderAccountingOptions::new().with_limits(exact);
        let mut context = RenderAccountingContext::new(options);
        let bytes: Arc<[u8]> = Arc::from([255, 128, 64, 0]);
        let expected = prepare_native_displacement_map(
            Size::new(1.0, 1.0),
            bytes.clone(),
            domains().source(),
            &mut context,
        )
        .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::DisplacementMapByteVisits), 4);
        assert!(usage.peak(RenderResource::RetainedCpuBytes) >= 4);
        context.set_options(
            options.with_limits(
                exact
                    .with_work_limit(RenderWork::DisplacementMapByteVisits, 3)
                    .unwrap(),
            ),
        );
        assert!(
            prepare_native_displacement_map(
                Size::new(1.0, 1.0),
                bytes.clone(),
                domains().source(),
                &mut context
            )
            .unwrap_err()
            .accounting_diagnostic()
            .is_some()
        );
        assert_eq!(
            context.last_attempt().unwrap().status(),
            RenderAttemptStatus::Refused
        );
        context.set_options(options);
        assert_eq!(
            prepare_native_displacement_map(
                Size::new(1.0, 1.0),
                bytes,
                domains().source(),
                &mut context
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn normalization_discards_unused_native_backing_and_keeps_zero_map_active() {
        for (scale, source, map_domain, size, expected_empty, expected_bypass) in [
            (
                0.0,
                domains(),
                domains().source(),
                Size::new(1.0, 1.0),
                false,
                true,
            ),
            (
                4.0,
                domains(),
                Rect::default(),
                Size::new(1.0, 1.0),
                false,
                false,
            ),
            (
                4.0,
                domains(),
                Rect::new(100.0, 100.0, 2.0, 2.0),
                Size::new(1.0, 1.0),
                false,
                false,
            ),
            (
                4.0,
                domains(),
                domains().source(),
                Size::new(0.0, 1.0),
                false,
                false,
            ),
            (
                4.0,
                FilterDomains::try_new(Rect::default(), domains().filter(), domains().result())
                    .unwrap(),
                domains().source(),
                Size::new(1.0, 1.0),
                true,
                false,
            ),
            (
                0.0,
                FilterDomains::try_new(domains().source(), domains().filter(), Rect::default())
                    .unwrap(),
                domains().source(),
                Size::new(1.0, 1.0),
                true,
                false,
            ),
        ] {
            let bytes: Arc<[u8]> = if size.width() == 0.0 {
                Arc::from([])
            } else {
                Arc::from([255, 255, 255, 255])
            };
            let weak = Arc::downgrade(&bytes);
            let authored = DisplacementForeground {
                operation: prepare(scale),
                source: DisplacementSource::finite(ink(), source).descriptor(),
                map: native(size, bytes, map_domain),
            };
            let value = normalize(&authored);
            assert_eq!(value.map.is_empty(), expected_empty);
            assert_eq!(value.map.is_passthrough(), expected_bypass);
            assert!(value.map.native_parts().is_none());
            drop(authored);
            assert!(weak.upgrade().is_none());
        }
        for map in [
            PreparedDisplacementMap::source_graphic(),
            PreparedDisplacementMap::source_alpha(),
            PreparedDisplacementMap::suppressed(),
        ] {
            let value = normalize(&DisplacementForeground {
                operation: prepare(f64::MAX),
                source: DisplacementSource::finite(ink(), domains()).descriptor(),
                map,
            });
            assert!(value.map.is_passthrough());
        }
    }

    #[test]
    fn touching_map_and_result_domains_still_retain_map_for_shared_raster_cells() {
        // At density one, O occupies half of cell zero and M the other half.
        // Map coverage is evaluated on the whole lattice-cell footprint before
        // the independent O coverage is applied, so continuous P/M intersection
        // cannot prove that native map data is unused.
        let source = FilterDomains::try_new(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Rect::new(0.0, 0.0, 0.5, 1.0),
        )
        .unwrap();
        let authored = DisplacementForeground {
            operation: prepare(4.0),
            source: DisplacementSource::finite(ink(), source).descriptor(),
            map: native(
                Size::new(1.0, 1.0),
                Arc::from([255; 4]),
                Rect::new(0.5, 0.0, 0.5, 1.0),
            ),
        };
        assert!(normalize(&authored).map.native_parts().is_some());
    }

    #[test]
    fn absent_first_input_omits_native_import_but_constant_input_still_retains_map() {
        let finite = DisplacementSource::finite(Scene::new(), domains()).descriptor();
        let periodic = DisplacementSource::periodic(
            Scene::new(),
            domains().source(),
            domains().filter(),
            domains().result(),
        )
        .unwrap()
        .descriptor();
        let transparent = DisplacementSource::constant(
            Color::TRANSPARENT,
            domains().filter(),
            domains().result(),
        )
        .unwrap()
        .descriptor();
        for source in [finite, periodic, transparent] {
            let bytes: Arc<[u8]> = Arc::from([255; 4]);
            let weak = Arc::downgrade(&bytes);
            let authored = DisplacementForeground {
                operation: prepare(f64::MAX),
                source,
                map: native(Size::new(1.0, 1.0), bytes, domains().source()),
            };
            let mut context = RenderAccountingContext::default();
            let normalized = context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::Normalize,
                    |account| authored.normalized_in(account, false),
                )
                .unwrap();
            assert!(normalized.map.is_empty());
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::ResourceProbes), 0);
            assert_eq!(usage.work(RenderWork::ImageIdentityCopies), 0);
            assert_eq!(usage.work(RenderWork::DisplacementMapByteVisits), 0);
            drop(authored);
            assert!(weak.upgrade().is_none());
        }
        let bytes: Arc<[u8]> = Arc::from([255; 4]);
        let weak = Arc::downgrade(&bytes);
        let authored = DisplacementForeground {
            operation: prepare(4.0),
            source: DisplacementSource::constant(
                Color::BLACK,
                domains().filter(),
                domains().result(),
            )
            .unwrap()
            .descriptor(),
            map: native(Size::new(1.0, 1.0), bytes, domains().source()),
        };
        let normalized = RenderAccountingContext::default()
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::Normalize,
                |account| authored.normalized_in(account, false),
            )
            .unwrap();
        assert!(normalized.map.native_parts().is_some());
        drop(authored);
        assert!(weak.upgrade().is_some());
        drop(normalized);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn receiver_moves_ordered_children_and_replaces_only_foreground() {
        let operation = prepare(4.0);
        let map = PreparedDisplacementMap::source_alpha();
        let child = ink();
        let expected_children = child.clone();
        let layer = Layer::new().with_filter_list(
            FilterList::try_ops(vec![crate::FilterOp::opacity(
                crate::UnitFilterAmount::try_new(0.5).unwrap(),
            )])
            .unwrap(),
        );
        let mut scene = Scene::new();
        scene
            .displacement_layer(
                layer,
                operation,
                DisplacementSource::finite(child, domains()),
                map.clone(),
            )
            .unwrap();
        let Command::Layer { layer, children } = &scene.commands[0] else {
            panic!("expected layer")
        };
        assert_eq!(children.as_slice(), expected_children.commands.as_slice());
        assert!(layer.filter_list().is_none());
        assert!(
            matches!(layer.foreground_filter(), Some(LayerFilter::Displacement(value)) if value.operation == operation && value.map == map)
        );
        assert!(
            Capabilities::CURRENT
                .filters()
                .supports_gpu_displacement_filter_execution()
        );
        assert!(
            Capabilities::CURRENT
                .ensure_supported(UnsupportedPrimitive::new(
                    PrimitiveFamily::Filters,
                    PrimitiveOperation::GpuDisplacementFilterExecution
                ))
                .is_ok()
        );
    }

    #[test]
    fn infinite_source_backdrop_is_typed_and_never_appends() {
        let backdrop = Layer::new()
            .try_backdrop_filter(
                BackdropFilterInput::try_new(
                    FilterList::try_ops(vec![crate::FilterOp::blur(
                        crate::FilterBlur::try_new(0.0).unwrap(),
                    )])
                    .unwrap(),
                    BackdropCaptureBounds::try_new(domains().filter()).unwrap(),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        let mut scene = ink();
        let original = scene.clone();
        for source in [
            DisplacementSource::constant(Color::BLACK, domains().filter(), domains().result())
                .unwrap(),
            DisplacementSource::periodic(
                ink(),
                domains().source(),
                domains().filter(),
                domains().result(),
            )
            .unwrap(),
        ] {
            let error = scene
                .displacement_layer(
                    backdrop.clone(),
                    prepare(0.0),
                    source,
                    PreparedDisplacementMap::suppressed(),
                )
                .unwrap_err();
            assert_eq!(
                error.displacement_diagnostic(),
                Some(&DisplacementPreparationError::InfiniteSourceBackdrop)
            );
            assert_eq!(scene, original);
        }
        scene
            .displacement_layer(
                backdrop,
                prepare(0.0),
                DisplacementSource::finite(ink(), domains()),
                PreparedDisplacementMap::suppressed(),
            )
            .unwrap();
        let Command::Layer { layer, .. } = &scene.commands[1] else {
            panic!("expected layer")
        };
        assert!(layer.backdrop_filter().is_some());
        let error = DisplacementSource::periodic(
            ink(),
            Rect::default(),
            domains().filter(),
            domains().result(),
        )
        .unwrap_err();
        assert_eq!(
            error.displacement_diagnostic(),
            Some(&DisplacementPreparationError::EmptyPeriod)
        );
    }

    #[test]
    fn deep_source_value_operations_use_the_existing_iterative_scene_owner() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let bytes: Arc<[u8]> = Arc::from([255, 128, 64, 1]);
                let weak = Arc::downgrade(&bytes);
                let map = native(Size::new(1.0, 1.0), bytes, domains().source());
                let mut scene = ink();
                for _ in 0..2048 {
                    let mut parent = Scene::new();
                    parent
                        .displacement_layer(
                            Layer::new(),
                            prepare(4.0),
                            DisplacementSource::finite(scene, domains()),
                            map.clone(),
                        )
                        .unwrap();
                    scene = parent;
                }
                drop(map);
                let copy = scene.clone();
                assert_eq!(scene, copy);
                assert!(!format!("{scene:?}").is_empty());
                let summary = scene
                    .inspect(&mut RenderAccountingContext::default())
                    .unwrap();
                assert_eq!(summary.layers, 2048);
                assert_eq!(summary.commands, 2049);
                assert_eq!(summary.fills, 1);
                drop(scene);
                assert!(weak.upgrade().is_some());
                drop(copy);
                assert!(weak.upgrade().is_none());
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn two_authored_layers_share_native_backing_and_normalization_preserves_other_clones() {
        let bytes: Arc<[u8]> = Arc::from([255, 128, 64, 1]);
        let weak = Arc::downgrade(&bytes);
        let map = native(Size::new(1.0, 1.0), bytes, domains().source());
        let external = map.clone();
        let mut scene = Scene::new();
        for _ in 0..2 {
            scene
                .displacement_layer(
                    Layer::new(),
                    prepare(4.0),
                    DisplacementSource::finite(ink(), domains()),
                    map.clone(),
                )
                .unwrap();
        }
        let uploads: Vec<_> = scene
            .commands
            .iter()
            .map(|command| {
                let Command::Layer { layer, .. } = command else {
                    panic!("expected layer")
                };
                let Some(LayerFilter::Displacement(value)) = layer.foreground_filter() else {
                    panic!("expected displacement")
                };
                value.map.native_parts().unwrap().0
            })
            .collect();
        assert!(uploads[0].same_allocation(uploads[1]));
        assert!(uploads[0].same_allocation(external.native_parts().unwrap().0));
        let normalized = RenderAccountingContext::default()
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::Normalize,
                |account| scene.normalize(Capabilities::CURRENT, account),
            )
            .unwrap();
        assert_eq!(
            external.native_parts().unwrap().0.bytes(),
            &[255, 128, 64, 1]
        );
        assert_eq!(external.native_parts().unwrap().1, domains().source());
        assert!(
            external
                .native_parts()
                .unwrap()
                .0
                .same_allocation(map.native_parts().unwrap().0)
        );
        drop(uploads);
        drop(scene);
        drop(map);
        drop(external);
        assert!(weak.upgrade().is_some());
        drop(normalized);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn pixel_and_metadata_storage_caps_refuse_without_retaining_the_input_arc() {
        // Four native bytes alone exceed three retained bytes. Even a zero-map
        // value requires a positive-size SharedValue owner, so zero retained
        // bytes cannot admit its metadata. Neither bound depends on a measured
        // aggregate or guessed Rust structure layout.
        for (size, bytes, cap) in [
            (Size::new(1.0, 1.0), Arc::<[u8]>::from([255; 4]), 3),
            (Size::new(0.0, 1.0), Arc::<[u8]>::from([]), 0),
        ] {
            let weak = Arc::downgrade(&bytes);
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(cap));
            let mut context = RenderAccountingContext::new(options);
            let error =
                prepare_native_displacement_map(size, bytes, domains().source(), &mut context)
                    .unwrap_err();
            assert!(error.accounting_diagnostic().is_some());
            assert_eq!(
                context.last_attempt().unwrap().status(),
                RenderAttemptStatus::Refused
            );
            assert!(weak.upgrade().is_none());
            context.set_options(RenderAccountingOptions::new());
            let bytes: Arc<[u8]> = if size.width() == 0.0 {
                Arc::from([])
            } else {
                Arc::from([255; 4])
            };
            assert!(
                prepare_native_displacement_map(size, bytes, domains().source(), &mut context)
                    .is_ok()
            );
        }
    }

    #[test]
    fn raw_map_capability_refusal_identifies_the_native_operation() {
        let diagnostic = crate::RuntimeCapabilityUnavailable::displacement_map_format_unavailable();
        assert_eq!(
            diagnostic.operation(),
            crate::RuntimeOperation::DisplacementRendering
        );
        assert_eq!(
            diagnostic.reason(),
            crate::RuntimeCapabilityUnavailableReason::DisplacementMapFormatUnavailable
        );
        let error = crate::Error::runtime_capability_unavailable(diagnostic);
        assert_eq!(error.code(), ErrorCode::RuntimeCapabilityUnavailable);
    }
}
