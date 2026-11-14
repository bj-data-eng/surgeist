//! Resolved native alpha-blur, displacement, flood and source-over shadow.
use crate::accounting::{RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork};
use crate::offset::{OffsetDisplacement, prepare_offset_displacement_in};
use crate::{
    Color, FilterColorSpace, FilterDomains, GaussianInput, GaussianPreparationError,
    GaussianSource, ImageRendering, OffsetInput, PreparedGaussian, PreparedOffset, Rect, Result,
    Scene,
};
use crate::{filter_source::FilterSourceDescriptor, gaussian::prepare_gaussian_in};

/// Actual native shadow merge arithmetic that could not be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DropShadowNumericPhase {
    /// Mapping the final output into the retained original and shifted alpha.
    MergeMapping,
    /// Checked original or shifted-alpha sample neighborhoods.
    GridIndex,
    /// Uploaded merge arithmetic and intermediate values.
    Sampling,
}

/// Native shadow refusal; Gaussian and offset retain their own typed causes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropShadowPreparationError {
    /// Resolved flood opacity must be finite and in the inclusive unit interval.
    InvalidFloodOpacity {
        /// Supplied resolved opacity.
        value: f64,
    },
    /// A periodic source requires positive width and height.
    EmptyPeriod,
    /// Same-layer backdrop is incompatible with a declared infinite input.
    InfiniteSourceBackdrop,
    /// Actual reachable merge arithmetic is unrepresentable.
    Numeric {
        /// Boundary at which representation was refused.
        phase: DropShadowNumericPhase,
        /// Supplied or intermediate value.
        value: f64,
    },
}

/// Untrusted resolved native shadow request, in layer-local primitive units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropShadowInput {
    offset: OffsetInput,
    gaussian: GaussianInput,
    flood_color: Color,
    flood_opacity: f64,
}
impl DropShadowInput {
    /// Requests dx=dy=2, deviations `[2,2]`, opaque black and automatic reconstruction.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            offset: OffsetInput::new().with_dx(2.0).with_dy(2.0),
            gaussian: GaussianInput::new().with_standard_deviation(2.0),
            flood_color: Color::BLACK,
            flood_opacity: 1.0,
        }
    }
    /// Sets horizontal displacement.
    #[must_use]
    pub const fn with_dx(mut self, dx: f64) -> Self {
        self.offset = self.offset.with_dx(dx);
        self
    }
    /// Sets vertical displacement.
    #[must_use]
    pub const fn with_dy(mut self, dy: f64) -> Self {
        self.offset = self.offset.with_dy(dy);
        self
    }
    /// Sets both standard deviations.
    #[must_use]
    pub const fn with_standard_deviation(mut self, sigma: f64) -> Self {
        self.gaussian = self.gaussian.with_standard_deviation(sigma);
        self
    }
    /// Sets independent horizontal and vertical standard deviations.
    #[must_use]
    pub const fn with_standard_deviations(mut self, x: f64, y: f64) -> Self {
        self.gaussian = self.gaussian.with_standard_deviations(x, y);
        self
    }
    /// Sets straight numeric-sRGB flood color, including its independent alpha.
    #[must_use]
    pub const fn with_flood_color(mut self, color: Color) -> Self {
        self.flood_color = color;
        self
    }
    /// Sets resolved flood opacity; admission requires finite `[0,1]`.
    #[must_use]
    pub const fn with_flood_opacity(mut self, opacity: f64) -> Self {
        self.flood_opacity = opacity;
        self
    }
    /// Selects reconstruction of displaced alpha.
    #[must_use]
    pub const fn with_rendering(mut self, rendering: ImageRendering) -> Self {
        self.offset = self.offset.with_rendering(rendering);
        self
    }
}
impl Default for DropShadowInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable inline native shadow admitted independently of actual frame grids.
///
/// Negative deviations disable only blur. Offset, flood and original-source merge
/// remain active. This value owns no coefficient allocation or native resource.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedDropShadow {
    displacement: OffsetDisplacement,
    gaussian: PreparedGaussian,
    flood_color: Color,
    flood_opacity: f64,
    color_space: FilterColorSpace,
}
impl PreparedDropShadow {
    pub(crate) const fn gaussian(self) -> PreparedGaussian {
        self.gaussian
    }
    pub(crate) const fn flood_color(self) -> Color {
        self.flood_color
    }
    pub(crate) const fn flood_opacity(self) -> f64 {
        self.flood_opacity
    }
    pub(crate) const fn color_space(self) -> FilterColorSpace {
        self.color_space
    }
    pub(crate) const fn offset(self, domains: FilterDomains) -> PreparedOffset {
        self.displacement.with_domains(domains)
    }
}

/// Admits dx, dy, sigma X, sigma Y, then resolved flood opacity, in that order.
///
/// One operation visit and each reached scalar visit are accounted to the outer
/// DropShadow attempt. Alpha Gaussian has no RGB transfer; the selected color
/// space belongs to the final original-over-shadow merge. No payload is allocated.
pub fn prepare_drop_shadow(
    input: DropShadowInput,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedDropShadow> {
    accounting.run(
        RenderAttemptKind::DropShadow,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                let displacement = prepare_offset_displacement_in(input.offset, account)?;
                let gaussian =
                    prepare_gaussian_in(input.gaussian, FilterColorSpace::Srgb, account)?;
                account.work(RenderWork::CoordinateVisits, 1)?;
                if !input.flood_opacity.is_finite() || !(0.0..=1.0).contains(&input.flood_opacity) {
                    return Err(DropShadowPreparationError::InvalidFloodOpacity {
                        value: input.flood_opacity,
                    }
                    .into());
                }
                Ok(PreparedDropShadow {
                    displacement,
                    gaussian,
                    flood_color: input.flood_color,
                    flood_opacity: input.flood_opacity,
                    color_space,
                })
            })
        },
    )
}

/// Authentic finite, constant or fundamental-period source for a native shadow.
///
/// Finite sources always use transparent extension. The original Scene remains
/// owned by the existing Gaussian source owner until moved into Layer children.
#[derive(Clone, Debug, PartialEq)]
pub struct DropShadowSource(GaussianSource);
impl DropShadowSource {
    /// Owns finite input with transparent extension outside its source domain.
    #[must_use]
    pub fn finite(scene: Scene, domains: FilterDomains) -> Self {
        Self(GaussianSource::finite(scene, domains))
    }
    /// Declares an authentic constant unbounded input and finite output domains.
    pub fn constant(color: Color, filter: Rect, result: Rect) -> Result<Self> {
        GaussianSource::constant(color, filter, result).map(Self)
    }
    /// Owns one actual fundamental tile and declares its periodic continuation.
    pub fn periodic(scene: Scene, period: Rect, filter: Rect, result: Rect) -> Result<Self> {
        GaussianSource::periodic(scene, period, filter, result)
            .map(Self)
            .map_err(|error| {
                if matches!(
                    error.gaussian_diagnostic(),
                    Some(GaussianPreparationError::EmptyPeriod)
                ) {
                    DropShadowPreparationError::EmptyPeriod.into()
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DropShadowForeground {
    pub(crate) operation: PreparedDropShadow,
    pub(crate) source: FilterSourceDescriptor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, Capabilities, Error, ErrorCode, Filter,
        FilterList, FilterOp, GaussianParameter, Layer, LayerComposition, OffsetParameter,
        OffsetPreparationError, PorterDuffOperator, RenderAccountingOptions, RenderAttemptStatus,
        RenderLimits, RenderQuantity, RenderResource, Transform,
        command::{NormalizedForegroundEffect, RenderCommand},
        layer::LayerFilter,
        scene::Command,
    };

    fn domains() -> FilterDomains {
        FilterDomains::try_new(
            Rect::new(-1.5, 2.5, 3.0, 4.0),
            Rect::new(-3.0, 1.0, 8.0, 9.0),
            Rect::new(1.25, -2.5, 7.0, 6.0),
        )
        .unwrap()
    }
    fn prepare(input: DropShadowInput) -> PreparedDropShadow {
        prepare_drop_shadow(
            input,
            FilterColorSpace::LinearSrgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap()
    }
    fn ink() -> Scene {
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 2.0, 3.0), Color::BLACK);
        scene
    }
    fn backdrop_layer() -> Layer {
        Layer::new()
            .try_backdrop_filter(
                BackdropFilterInput::try_new(
                    FilterList::try_ops(vec![FilterOp::blur(
                        crate::FilterBlur::try_new(1.0).unwrap(),
                    )])
                    .unwrap(),
                    BackdropCaptureBounds::try_new(Rect::new(0.0, 0.0, 4.0, 5.0)).unwrap(),
                    None,
                )
                .unwrap(),
            )
            .unwrap()
    }

    #[test]
    fn defaults_shared_axes_and_disabled_blur_preserve_other_shadow_steps() {
        assert_eq!(DropShadowInput::new(), DropShadowInput::default());
        let value = prepare(DropShadowInput::default());
        let offset = value.offset(domains());
        assert_eq!([offset.dx(), offset.dy()], [2.0; 2]);
        assert_eq!(offset.domains(), domains());
        assert_eq!(value.gaussian().sigma(), Some([2.0; 2]));
        assert_eq!(value.gaussian().color_space(), FilterColorSpace::Srgb);
        assert_eq!(value.flood_color(), Color::BLACK);
        assert_eq!(value.flood_opacity(), 1.0);
        assert_eq!(offset.rendering(), ImageRendering::Auto);
        assert_eq!(value.color_space(), FilterColorSpace::LinearSrgb);
        assert_eq!(
            prepare(DropShadowInput::new().with_standard_deviation(300.0))
                .gaussian()
                .sigma(),
            Some([300.0; 2])
        );
        let color = Color::try_rgba(0.2, 0.4, 0.6, 0.5).unwrap();
        for sigma in [
            [-1.0, 2.0],
            [3.0, -1.0],
            [0.0, -0.0],
            [0.0, 2.0],
            [3.0, 0.0],
        ] {
            let value = prepare(
                DropShadowInput::new()
                    .with_standard_deviations(sigma[0], sigma[1])
                    .with_dx(-2.5)
                    .with_dy(3.25)
                    .with_flood_color(color)
                    .with_flood_opacity(0.75)
                    .with_rendering(ImageRendering::HighQuality),
            );
            assert_eq!(
                value.gaussian().sigma(),
                if sigma.iter().any(|s| *s < 0.0) {
                    None
                } else {
                    Some(sigma.map(|s| if s == 0.0 { 0.0 } else { s }))
                }
            );
            assert_eq!(value.flood_color(), color);
            assert_eq!(value.flood_opacity(), 0.75);
            let offset = value.offset(domains());
            assert_eq!([offset.dx(), offset.dy()], [-2.5, 3.25]);
            assert_eq!(offset.rendering(), ImageRendering::HighQuality);
            assert_eq!(offset.domains(), domains());
        }
    }

    #[test]
    fn invalid_scalars_keep_owner_cause_validation_order_and_outer_attempt() {
        let cases = [
            (
                DropShadowInput::new()
                    .with_dx(f64::NAN)
                    .with_dy(f64::INFINITY),
                1,
            ),
            (
                DropShadowInput::new()
                    .with_dy(f64::INFINITY)
                    .with_standard_deviation(f64::NAN),
                2,
            ),
            (
                DropShadowInput::new().with_standard_deviations(f64::NAN, f64::INFINITY),
                3,
            ),
            (
                DropShadowInput::new()
                    .with_standard_deviations(-1.0, f64::INFINITY)
                    .with_flood_opacity(f64::NAN),
                4,
            ),
            (DropShadowInput::new().with_flood_opacity(f64::NAN), 5),
        ];
        for (input, reached) in cases {
            let mut accounting = RenderAccountingContext::default();
            let error =
                prepare_drop_shadow(input, FilterColorSpace::Srgb, &mut accounting).unwrap_err();
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            match reached {
                1 | 2 => {
                    assert!(
                        matches!(error.offset_diagnostic(), Some(OffsetPreparationError::InvalidParameter { parameter, .. }) if *parameter == if reached == 1 { OffsetParameter::Dx } else { OffsetParameter::Dy })
                    );
                    assert!(error.gaussian_diagnostic().is_none());
                    assert!(error.drop_shadow_diagnostic().is_none());
                }
                3 | 4 => {
                    assert!(
                        matches!(error.gaussian_diagnostic(), Some(GaussianPreparationError::InvalidParameter { parameter, .. }) if *parameter == if reached == 3 { GaussianParameter::StandardDeviationX } else { GaussianParameter::StandardDeviationY })
                    );
                    assert!(error.offset_diagnostic().is_none());
                    assert!(error.drop_shadow_diagnostic().is_none());
                }
                _ => {
                    assert!(
                        matches!(error.drop_shadow_diagnostic(), Some(DropShadowPreparationError::InvalidFloodOpacity { value }) if value.is_nan())
                    );
                    assert!(error.offset_diagnostic().is_none());
                    assert!(error.gaussian_diagnostic().is_none());
                }
            }
            let report = accounting.last_attempt().unwrap();
            assert_eq!(report.kind(), RenderAttemptKind::DropShadow);
            assert_eq!(
                report.status(),
                RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
            );
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::CoordinateVisits),
                reached
            );
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::FilterOperationVisits),
                1
            );
        }
        for value in [f64::NEG_INFINITY, -0.001, 1.001, f64::INFINITY] {
            let error = prepare_drop_shadow(
                DropShadowInput::new().with_flood_opacity(value),
                FilterColorSpace::Srgb,
                &mut RenderAccountingContext::default(),
            )
            .unwrap_err();
            assert_eq!(
                error.drop_shadow_diagnostic(),
                Some(&DropShadowPreparationError::InvalidFloodOpacity { value })
            );
        }
        for value in [0.0, -0.0, 1.0] {
            assert_eq!(
                prepare(DropShadowInput::new().with_flood_opacity(value)).flood_opacity(),
                value
            );
        }
    }

    #[test]
    fn exact_admission_budget_has_no_payload_and_refusal_retry_is_identical() {
        let exact = RenderLimits::new()
            .with_work_limit(RenderWork::FilterOperationVisits, 1)
            .unwrap()
            .with_work_limit(RenderWork::CoordinateVisits, 5)
            .unwrap()
            .with_retained_bytes(0)
            .with_scratch_bytes(0);
        let options = RenderAccountingOptions::new().with_limits(exact);
        let mut accounting = RenderAccountingContext::new(options);
        let value = prepare_drop_shadow(
            DropShadowInput::new(),
            FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap();
        let usage = accounting
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 5);
        assert_eq!(usage.work(RenderWork::FilterOperationVisits), 1);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        for (work, limit) in [
            (RenderWork::FilterOperationVisits, 0),
            (RenderWork::CoordinateVisits, 4),
        ] {
            accounting
                .set_options(options.with_limits(exact.with_work_limit(work, limit).unwrap()));
            let error = prepare_drop_shadow(
                DropShadowInput::new(),
                FilterColorSpace::Srgb,
                &mut accounting,
            )
            .unwrap_err();
            assert!(
                matches!(error.accounting_diagnostic(), Some(crate::RenderAccountingError::Refused(refusal)) if refusal.quantity() == RenderQuantity::Work(work))
            );
            let report = accounting.last_attempt().unwrap();
            assert_eq!(report.kind(), RenderAttemptKind::DropShadow);
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.attempt().unwrap().usage().work(work), limit);
            accounting.set_options(options);
            assert_eq!(
                prepare_drop_shadow(
                    DropShadowInput::new(),
                    FilterColorSpace::Srgb,
                    &mut accounting
                )
                .unwrap(),
                value
            );
        }
    }

    #[test]
    fn sources_delegate_validation_and_internal_gaussian_result_is_filter() {
        let finite = DropShadowSource::finite(ink(), domains());
        assert_eq!(
            finite.descriptor(),
            FilterSourceDescriptor::Finite {
                domains: domains(),
                edge: crate::FilterEdgeMode::None
            }
        );
        let filter = domains().filter();
        let result = domains().result();
        let constant = DropShadowSource::constant(Color::BLACK, filter, result).unwrap();
        let periodic =
            DropShadowSource::periodic(ink(), domains().source(), filter, result).unwrap();
        for source in [finite, constant, periodic] {
            let descriptor = source.descriptor();
            let internal = descriptor.for_filter_output();
            assert_eq!(descriptor.result(), result);
            assert_eq!(internal.filter(), filter);
            assert_eq!(internal.result(), filter);
            assert_eq!(internal.is_infinite(), descriptor.is_infinite());
        }
        for empty in [Rect::new(0.0, 0.0, 0.0, 1.0), Rect::new(0.0, 0.0, 1.0, 0.0)] {
            assert_eq!(
                DropShadowSource::periodic(ink(), empty, filter, result)
                    .unwrap_err()
                    .drop_shadow_diagnostic(),
                Some(&DropShadowPreparationError::EmptyPeriod)
            );
        }
        for invalid in [
            Rect::new(f64::NAN, 0.0, 1.0, 1.0),
            Rect::new(0.0, 0.0, -1.0, 1.0),
            Rect::new(f64::MAX, 0.0, f64::MAX, 1.0),
        ] {
            assert!(DropShadowSource::constant(Color::BLACK, invalid, result).is_err());
            assert!(DropShadowSource::constant(Color::BLACK, filter, invalid).is_err());
            assert!(DropShadowSource::periodic(ink(), invalid, filter, result).is_err());
        }
    }

    #[test]
    fn receiver_replaces_foreground_preserves_outer_state_and_normalizes() {
        let operation = prepare(DropShadowInput::new());
        let layer = Layer::new()
            .try_filter(Filter::try_blur(2.0).unwrap())
            .unwrap()
            .with_transform(Transform::try_new([1.0, 0.5, -0.25, 2.0, 3.0, -4.0]).unwrap())
            .isolated()
            .try_opacity(0.5)
            .unwrap()
            .with_composition(LayerComposition::normal_porter_duff(
                PorterDuffOperator::Copy,
            ));
        let expected = layer.clone().with_native_shadow(DropShadowForeground {
            operation,
            source: DropShadowSource::finite(Scene::new(), domains()).descriptor(),
        });
        let mut children = ink();
        children.fill(Rect::new(4.0, 5.0, 1.0, 1.0), Color::BLACK);
        let authored = children.clone();
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                layer,
                operation,
                DropShadowSource::finite(children, domains()),
            )
            .unwrap();
        let Command::Layer { layer, children } = &scene.commands[0] else {
            panic!("expected layer")
        };
        assert_eq!(**layer, expected);
        assert!(layer.filter().is_none());
        assert!(layer.filter_list().is_none());
        assert_eq!(children.as_slice(), authored.commands.as_slice());
        assert!(matches!(
            layer.foreground_filter(),
            Some(LayerFilter::NativeShadow(_))
        ));
        assert!(
            layer
                .clone()
                .with_filter_list(FilterList::none())
                .foreground_filter()
                .is_none()
        );
        assert!(matches!(
            layer
                .clone()
                .with_offset(operation.offset(domains()))
                .foreground_filter(),
            Some(LayerFilter::Offset(_))
        ));
        let mut accounting = RenderAccountingContext::default();
        accounting.run(RenderAttemptKind::Frame, RenderPhase::Normalize, |account| {
            let normalized = scene.normalize(Capabilities::CURRENT, account)?;
            let RenderCommand::Layer { layer, .. } = &normalized.commands[0] else { panic!("expected normalized layer") };
            assert!(matches!(layer.foreground_filters.as_ref(), Some(NormalizedForegroundEffect::NativeShadow(value)) if value.operation == operation));
            Ok(())
        }).unwrap();
    }

    #[test]
    fn infinite_backdrop_refusal_appends_nothing_and_finite_backdrop_is_retained() {
        let operation = prepare(DropShadowInput::new());
        let mut scene = ink();
        let original = scene.clone();
        for source in [
            DropShadowSource::constant(Color::BLACK, domains().filter(), domains().result())
                .unwrap(),
            DropShadowSource::periodic(
                ink(),
                domains().source(),
                domains().filter(),
                domains().result(),
            )
            .unwrap(),
        ] {
            let error = scene
                .drop_shadow_layer(backdrop_layer(), operation, source)
                .unwrap_err();
            assert_eq!(
                error.drop_shadow_diagnostic(),
                Some(&DropShadowPreparationError::InfiniteSourceBackdrop)
            );
            assert_eq!(scene, original);
        }
        scene
            .drop_shadow_layer(
                backdrop_layer(),
                operation,
                DropShadowSource::finite(ink(), domains()),
            )
            .unwrap();
        let Command::Layer { layer, .. } = &scene.commands[1] else {
            panic!("expected layer")
        };
        assert!(layer.backdrop_filter().is_some());
    }

    #[test]
    fn deep_source_value_operations_and_inspection_use_existing_tree_owner() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let operation = prepare(DropShadowInput::new());
                let mut scene = ink();
                for _ in 0..2048 {
                    let mut parent = Scene::new();
                    parent
                        .drop_shadow_layer(
                            Layer::new(),
                            operation,
                            DropShadowSource::finite(scene, domains()),
                        )
                        .unwrap();
                    scene = parent;
                }
                let source = DropShadowSource::periodic(
                    scene,
                    Rect::new(0.0, 0.0, 2.0, 3.0),
                    domains().filter(),
                    domains().result(),
                )
                .unwrap();
                let copy = source.clone();
                assert_eq!(copy, source);
                assert!(!format!("{source:?}").is_empty());
                let (scene, _) = source.into_parts();
                let summary = scene
                    .inspect(&mut RenderAccountingContext::default())
                    .unwrap();
                assert_eq!(summary.layers, 2048);
                assert_eq!(summary.commands, 2049);
                assert_eq!(summary.fills, 1);
                drop(scene);
                drop(copy);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn merge_numeric_diagnostics_and_capability_are_distinct_from_css_shadow() {
        for phase in [
            DropShadowNumericPhase::MergeMapping,
            DropShadowNumericPhase::GridIndex,
            DropShadowNumericPhase::Sampling,
        ] {
            let diagnostic = DropShadowPreparationError::Numeric {
                phase,
                value: f64::MAX,
            };
            let error: Error = diagnostic.into();
            assert_eq!(error.drop_shadow_diagnostic(), Some(&diagnostic));
            assert!(error.gaussian_diagnostic().is_none());
            assert!(error.offset_diagnostic().is_none());
            assert!(error.convolution_diagnostic().is_none());
        }
        assert!(
            Capabilities::CURRENT
                .filters()
                .supports_gpu_native_drop_shadow_filter_execution()
        );
        let unavailable = crate::RuntimeCapabilityUnavailable::filter_float32_unavailable(
            crate::RuntimeOperation::NativeDropShadowRendering,
        );
        assert_eq!(
            unavailable.operation(),
            crate::RuntimeOperation::NativeDropShadowRendering
        );
    }
}
