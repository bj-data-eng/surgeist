//! Admitted native Gaussian parameters and authentic source declarations.
use crate::accounting::{
    Account, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork,
};
use crate::filter_source::FilterSourceDescriptor;
use crate::{Color, FilterColorSpace, FilterDomains, FilterEdgeMode, Rect, Result, Scene};

/// Gaussian parameter whose resolved value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GaussianParameter {
    /// Horizontal standard deviation in layer-local primitive units.
    StandardDeviationX,
    /// Vertical standard deviation in layer-local primitive units.
    StandardDeviationY,
}

/// Actual Gaussian arithmetic that could not be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GaussianNumericPhase {
    /// Device-axis deviation, support, coefficient count or serialized extent.
    KernelExtent,
    /// Checked reachable source indices and reconstruction neighborhoods.
    GridIndex,
    /// Representability of the source/result grid mapping.
    GridMapping,
    /// Uploaded sampling or accumulation arithmetic.
    Sampling,
}

/// Typed Gaussian refusal during admission, source construction or frame qualification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GaussianPreparationError {
    /// A reached resolved standard deviation is not finite.
    InvalidParameter {
        /// Parameter being inspected.
        parameter: GaussianParameter,
        /// Supplied resolved value.
        value: f64,
    },
    /// Actual reachable arithmetic cannot be represented safely.
    Numeric {
        /// Numeric boundary at which representation was refused.
        phase: GaussianNumericPhase,
        /// Supplied or intermediate value at that boundary.
        value: f64,
    },
    /// A periodic source requires positive width and height.
    EmptyPeriod,
    /// A same-layer backdrop could invalidate a declared infinite source.
    InfiniteSourceBackdrop,
}

/// Untrusted resolved independent standard deviations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaussianInput {
    sigma: [f64; 2],
}
impl GaussianInput {
    /// Requests zero standard deviation on both axes.
    #[must_use]
    pub const fn new() -> Self {
        Self { sigma: [0.0; 2] }
    }
    /// Sets the same standard deviation on both axes.
    #[must_use]
    pub const fn with_standard_deviation(mut self, sigma: f64) -> Self {
        self.sigma = [sigma; 2];
        self
    }
    /// Sets independent horizontal and vertical standard deviations.
    #[must_use]
    pub const fn with_standard_deviations(mut self, x: f64, y: f64) -> Self {
        self.sigma = [x, y];
        self
    }
}
impl Default for GaussianInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable inline Gaussian admitted independently of a device or capture grid.
///
/// Either negative deviation disables filtering; both zero select identity.
/// Finite positive values are not subject to the CSS blur limit. Actual kernel,
/// grid and uploaded arithmetic are qualified when the frame is planned. This
/// value owns no coefficient allocation or GPU resource.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedGaussian {
    operation: PreparedOperation,
    color_space: FilterColorSpace,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum PreparedOperation {
    Disabled,
    Identity,
    Active([f64; 2]),
}
impl PreparedGaussian {
    pub(crate) const fn sigma(self) -> Option<[f64; 2]> {
        match self.operation {
            PreparedOperation::Disabled => None,
            PreparedOperation::Identity => Some([0.0; 2]),
            PreparedOperation::Active(sigma) => Some(sigma),
        }
    }
    pub(crate) const fn color_space(self) -> FilterColorSpace {
        self.color_space
    }
    pub(crate) const fn is_passthrough(self) -> bool {
        !matches!(self.operation, PreparedOperation::Active(_))
    }
}

/// Admits standard deviations in X/Y order through central accounting.
///
/// Both coordinates are inspected for finiteness before negative values select
/// disabled input. Each reached coordinate spends one `CoordinateVisits`; the
/// request spends one `FilterOperationVisits`. Preparation allocates no payload.
pub fn prepare_gaussian(
    input: GaussianInput,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedGaussian> {
    accounting.run(
        RenderAttemptKind::Gaussian,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                prepare_gaussian_in(input, color_space, account)
            })
        },
    )
}

/// Admits scalar deviations inside an already charged native operation.
pub(crate) fn prepare_gaussian_in(
    input: GaussianInput,
    color_space: FilterColorSpace,
    account: &Account,
) -> Result<PreparedGaussian> {
    for (parameter, value) in [
        (GaussianParameter::StandardDeviationX, input.sigma[0]),
        (GaussianParameter::StandardDeviationY, input.sigma[1]),
    ] {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if !value.is_finite() {
            return Err(GaussianPreparationError::InvalidParameter { parameter, value }.into());
        }
    }
    let operation = if input.sigma.iter().any(|s| *s < 0.0) {
        PreparedOperation::Disabled
    } else if input.sigma == [0.0; 2] {
        PreparedOperation::Identity
    } else {
        PreparedOperation::Active(input.sigma.map(|s| if s == 0.0 { 0.0 } else { s }))
    };
    Ok(PreparedGaussian {
        operation,
        color_space,
    })
}

/// An owned finite scene, constant image or fundamental-period scene.
///
/// Constant and periodic declarations denote unbounded input. Their specific
/// extension takes precedence over finite input clipping; filter and result
/// regions still clip output. Outer layer semantics apply after evaluation.
/// Scene value operations and disposal use the existing iterative tree owner.
#[derive(Clone, Debug, PartialEq)]
pub struct GaussianSource {
    scene: Scene,
    descriptor: FilterSourceDescriptor,
}
impl GaussianSource {
    /// Owns a finite scene with transparent extension outside its input domain.
    #[must_use]
    pub fn finite(scene: Scene, domains: FilterDomains) -> Self {
        Self::finite_with_edge(scene, domains, FilterEdgeMode::None)
    }
    /// Owns a finite scene with the selected original-domain edge extension.
    #[must_use]
    pub fn finite_with_edge(scene: Scene, domains: FilterDomains, edge: FilterEdgeMode) -> Self {
        Self {
            scene,
            descriptor: FilterSourceDescriptor::Finite { domains, edge },
        }
    }
    /// Declares a constant unbounded image with independent finite output clips.
    pub fn constant(color: Color, filter: Rect, result: Rect) -> Result<Self> {
        FilterDomains::try_new(Rect::default(), filter, result)?;
        Ok(Self {
            scene: Scene::new(),
            descriptor: FilterSourceDescriptor::Constant {
                color,
                filter,
                result,
            },
        })
    }
    /// Owns one actual fundamental tile and declares its periodic continuation.
    ///
    /// The finite period must have positive width and height. Output filter and
    /// result rectangles may be empty. Tile children are clipped to the period.
    pub fn periodic(scene: Scene, period: Rect, filter: Rect, result: Rect) -> Result<Self> {
        FilterDomains::try_new(period, filter, result)?;
        if period.width() == 0.0 || period.height() == 0.0 {
            return Err(GaussianPreparationError::EmptyPeriod.into());
        }
        Ok(Self {
            scene,
            descriptor: FilterSourceDescriptor::Periodic {
                period,
                filter,
                result,
            },
        })
    }
    pub(crate) fn into_parts(self) -> (Scene, FilterSourceDescriptor) {
        (self.scene, self.descriptor)
    }
    pub(crate) const fn descriptor(&self) -> FilterSourceDescriptor {
        self.descriptor
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GaussianForeground {
    pub(crate) operation: PreparedGaussian,
    pub(crate) source: FilterSourceDescriptor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, Capabilities, Error, ErrorCode, Filter,
        FilterList, FilterOp, Layer, LayerComposition, PorterDuffOperator, RenderAccountingOptions,
        RenderAttemptStatus, RenderLimits, RenderQuantity, RenderResource, Transform,
        command::NormalizedForegroundEffect, layer::LayerFilter, scene::Command,
    };

    fn domains() -> FilterDomains {
        FilterDomains::try_new(
            Rect::new(-1.5, 2.5, 3.0, 4.0),
            Rect::new(-3.0, 1.0, 8.0, 9.0),
            Rect::new(1.25, -2.5, 7.0, 6.0),
        )
        .unwrap()
    }
    fn prepare(input: GaussianInput) -> PreparedGaussian {
        prepare_gaussian(
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
    fn default_shared_and_independent_deviations_preserve_native_semantics() {
        assert_eq!(GaussianInput::new(), GaussianInput::default());
        for input in [
            GaussianInput::default(),
            GaussianInput::new().with_standard_deviation(-0.0),
        ] {
            let value = prepare(input);
            assert_eq!(value.sigma(), Some([0.0, 0.0]));
            assert!(value.is_passthrough());
            assert_eq!(value.color_space(), FilterColorSpace::LinearSrgb);
        }
        assert_eq!(
            prepare(GaussianInput::new().with_standard_deviation(300.0)).sigma(),
            Some([300.0; 2])
        );
        for sigma in [[0.0, 2.5], [3.5, 0.0], [f64::MAX, f64::MIN_POSITIVE]] {
            let value = prepare(GaussianInput::new().with_standard_deviations(sigma[0], sigma[1]));
            assert_eq!(value.sigma(), Some(sigma));
            assert!(!value.is_passthrough());
        }
        for sigma in [[-1.0, f64::MAX], [2.5, -f64::MIN_POSITIVE], [-1.0, -2.0]] {
            let value = prepare(GaussianInput::new().with_standard_deviations(sigma[0], sigma[1]));
            assert_eq!(value.sigma(), None);
            assert!(value.is_passthrough());
        }
    }

    #[test]
    fn nonfinite_deviations_fail_in_axis_order_even_when_other_axis_disables() {
        for (parameter, sigma, visits) in [
            (
                GaussianParameter::StandardDeviationX,
                [f64::NAN, f64::INFINITY],
                1,
            ),
            (
                GaussianParameter::StandardDeviationX,
                [f64::NEG_INFINITY, -1.0],
                1,
            ),
            (
                GaussianParameter::StandardDeviationY,
                [-1.0, f64::INFINITY],
                2,
            ),
            (GaussianParameter::StandardDeviationY, [0.0, f64::NAN], 2),
        ] {
            let mut accounting = RenderAccountingContext::default();
            let error = prepare_gaussian(
                GaussianInput::new().with_standard_deviations(sigma[0], sigma[1]),
                FilterColorSpace::Srgb,
                &mut accounting,
            )
            .unwrap_err();
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            let Some(GaussianPreparationError::InvalidParameter {
                parameter: actual,
                value,
            }) = error.gaussian_diagnostic()
            else {
                panic!("missing Gaussian diagnostic")
            };
            assert_eq!(*actual, parameter);
            assert_eq!(value.to_bits(), sigma[(visits - 1) as usize].to_bits());
            assert!(error.offset_diagnostic().is_none());
            assert!(error.convolution_diagnostic().is_none());
            let report = accounting.last_attempt().unwrap();
            assert_eq!(report.kind(), RenderAttemptKind::Gaussian);
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
                visits
            );
        }
    }

    #[test]
    fn exact_preparation_budget_has_no_payload_and_refusal_retry_is_identical() {
        let exact = RenderLimits::new()
            .with_work_limit(RenderWork::FilterOperationVisits, 1)
            .unwrap()
            .with_work_limit(RenderWork::CoordinateVisits, 2)
            .unwrap()
            .with_retained_bytes(0)
            .with_scratch_bytes(0);
        let options = RenderAccountingOptions::new().with_limits(exact);
        let mut accounting = RenderAccountingContext::new(options);
        let value = prepare_gaussian(
            GaussianInput::default(),
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
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 2);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
        for (work, limit) in [
            (RenderWork::FilterOperationVisits, 0),
            (RenderWork::CoordinateVisits, 1),
        ] {
            accounting
                .set_options(options.with_limits(exact.with_work_limit(work, limit).unwrap()));
            let error = prepare_gaussian(
                GaussianInput::default(),
                FilterColorSpace::Srgb,
                &mut accounting,
            )
            .unwrap_err();
            assert!(
                matches!(error.accounting_diagnostic(), Some(crate::RenderAccountingError::Refused(refusal)) if refusal.quantity() == RenderQuantity::Work(work))
            );
            assert_eq!(
                accounting.last_attempt().unwrap().status(),
                RenderAttemptStatus::Refused
            );
            assert_eq!(
                accounting
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(work),
                limit
            );
            accounting.set_options(options);
            assert_eq!(
                prepare_gaussian(
                    GaussianInput::default(),
                    FilterColorSpace::Srgb,
                    &mut accounting
                )
                .unwrap(),
                value
            );
        }
    }

    #[test]
    fn source_declarations_keep_independent_domains_and_operation_owned_default() {
        let finite = GaussianSource::finite(ink(), domains());
        assert_eq!(
            finite.descriptor(),
            FilterSourceDescriptor::Finite {
                domains: domains(),
                edge: FilterEdgeMode::None
            }
        );
        for edge in [
            FilterEdgeMode::Duplicate,
            FilterEdgeMode::Wrap,
            FilterEdgeMode::Mirror,
            FilterEdgeMode::None,
        ] {
            let source = GaussianSource::finite_with_edge(ink(), domains(), edge);
            assert_eq!(
                source.descriptor(),
                FilterSourceDescriptor::Finite {
                    domains: domains(),
                    edge
                }
            );
        }
        let filter = domains().filter();
        let result = domains().result();
        let constant = GaussianSource::constant(Color::BLACK, filter, result).unwrap();
        assert!(constant.scene.is_empty());
        assert_eq!(
            constant.descriptor(),
            FilterSourceDescriptor::Constant {
                color: Color::BLACK,
                filter,
                result
            }
        );
        let period = Rect::new(-3.25, 2.5, 1.5, 2.25);
        let periodic = GaussianSource::periodic(ink(), period, filter, result).unwrap();
        assert_eq!(periodic.scene, ink());
        assert_eq!(
            periodic.descriptor(),
            FilterSourceDescriptor::Periodic {
                period,
                filter,
                result
            }
        );
        for empty in [Rect::new(0.0, 0.0, 0.0, 1.0), Rect::new(0.0, 0.0, 1.0, 0.0)] {
            assert_eq!(
                GaussianSource::periodic(ink(), empty, filter, result)
                    .unwrap_err()
                    .gaussian_diagnostic(),
                Some(&GaussianPreparationError::EmptyPeriod)
            );
        }
        for invalid in [
            Rect::new(f64::NAN, 0.0, 1.0, 1.0),
            Rect::new(0.0, 0.0, -1.0, 1.0),
            Rect::new(f64::MAX, 0.0, f64::MAX, 1.0),
        ] {
            assert!(GaussianSource::constant(Color::BLACK, invalid, result).is_err());
            assert!(GaussianSource::constant(Color::BLACK, filter, invalid).is_err());
            assert!(GaussianSource::periodic(ink(), invalid, filter, result).is_err());
        }
        assert!(GaussianSource::constant(Color::BLACK, Rect::default(), Rect::default()).is_ok());
    }

    #[test]
    fn receiver_replaces_foreground_and_preserves_outer_semantics_and_child_order() {
        let operation = prepare(GaussianInput::new().with_standard_deviations(1.0, 2.0));
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
        let expected = layer.clone().with_gaussian(GaussianForeground {
            operation,
            source: GaussianSource::finite(Scene::new(), domains()).descriptor(),
        });
        let mut children = ink();
        children.fill(Rect::new(4.0, 5.0, 1.0, 1.0), Color::BLACK);
        let authored = children.clone();
        let mut scene = Scene::new();
        scene
            .gaussian_layer(
                layer,
                operation,
                GaussianSource::finite(children, domains()),
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
            Some(LayerFilter::Gaussian(_))
        ));
        assert!(
            layer
                .clone()
                .with_filter_list(FilterList::none())
                .foreground_filter()
                .is_none()
        );
        let mut accounting = RenderAccountingContext::default();
        accounting.run(RenderAttemptKind::Frame, RenderPhase::Normalize, |account| {
            let normalized = scene.normalize(Capabilities::CURRENT, account)?;
            let crate::command::RenderCommand::Layer { layer, .. } = &normalized.commands[0] else { panic!("expected normalized layer") };
            assert!(matches!(layer.foreground_filters.as_ref(), Some(NormalizedForegroundEffect::Gaussian(value)) if value.operation == operation));
            Ok(())
        }).unwrap();
    }

    #[test]
    fn infinite_backdrop_refusal_appends_nothing_and_finite_backdrop_is_retained() {
        let operation = prepare(GaussianInput::default());
        let mut scene = ink();
        let original = scene.clone();
        for source in [
            GaussianSource::constant(Color::BLACK, domains().filter(), domains().result()).unwrap(),
            GaussianSource::periodic(
                ink(),
                domains().source(),
                domains().filter(),
                domains().result(),
            )
            .unwrap(),
        ] {
            let error = scene
                .gaussian_layer(backdrop_layer(), operation, source)
                .unwrap_err();
            assert_eq!(
                error.gaussian_diagnostic(),
                Some(&GaussianPreparationError::InfiniteSourceBackdrop)
            );
            assert_eq!(scene, original);
        }
        scene
            .gaussian_layer(
                backdrop_layer(),
                operation,
                GaussianSource::finite(ink(), domains()),
            )
            .unwrap();
        let Command::Layer { layer, .. } = &scene.commands[1] else {
            panic!("expected layer")
        };
        assert!(layer.backdrop_filter().is_some());
    }

    #[test]
    fn deep_source_value_operations_inspection_and_disposal_use_existing_tree_owner() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let operation = prepare(GaussianInput::default());
                let mut scene = ink();
                for _ in 0..2048 {
                    let mut parent = Scene::new();
                    parent
                        .gaussian_layer(
                            Layer::new(),
                            operation,
                            GaussianSource::finite(scene, domains()),
                        )
                        .unwrap();
                    scene = parent;
                }
                let source = GaussianSource::periodic(
                    scene,
                    Rect::new(0.0, 0.0, 2.0, 3.0),
                    domains().filter(),
                    domains().result(),
                )
                .unwrap();
                let copy = source.clone();
                assert_eq!(copy, source);
                assert!(!format!("{source:?}").is_empty());
                let summary = source
                    .scene
                    .inspect(&mut RenderAccountingContext::default())
                    .unwrap();
                assert_eq!(summary.layers, 2048);
                assert_eq!(summary.commands, 2049);
                assert_eq!(summary.fills, 1);
                drop(source);
                drop(copy);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn numeric_diagnostics_preserve_gaussian_operation_and_phase() {
        for phase in [
            GaussianNumericPhase::KernelExtent,
            GaussianNumericPhase::GridIndex,
            GaussianNumericPhase::GridMapping,
            GaussianNumericPhase::Sampling,
        ] {
            let diagnostic = GaussianPreparationError::Numeric {
                phase,
                value: f64::MAX,
            };
            let error: Error = diagnostic.into();
            assert_eq!(error.gaussian_diagnostic(), Some(&diagnostic));
            assert!(error.convolution_diagnostic().is_none());
            assert!(error.offset_diagnostic().is_none());
        }
        assert!(
            Capabilities::CURRENT
                .filters()
                .supports_gpu_gaussian_filter_execution()
        );
        let unavailable = crate::RuntimeCapabilityUnavailable::filter_float32_unavailable(
            crate::RuntimeOperation::GaussianRendering,
        );
        assert_eq!(
            unavailable.operation(),
            crate::RuntimeOperation::GaussianRendering
        );
    }
}
