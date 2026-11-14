//! Admitted native rectangular component extrema and authentic source declarations.
use crate::accounting::{RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork};
use crate::filter_source::FilterSourceDescriptor;
use crate::{
    Color, FilterColorSpace, FilterDomains, GaussianPreparationError, GaussianSource, Rect, Result,
    Scene,
};

/// Component-wise rectangular extrema selected for native morphology.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MorphologyOperator {
    /// Selects the minimum of every premultiplied RGBA component.
    #[default]
    Erode,
    /// Selects the maximum of every premultiplied RGBA component.
    Dilate,
}

/// Morphology parameter whose supplied resolved value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MorphologyParameter {
    /// Horizontal radius in layer-local primitive units.
    RadiusX,
    /// Vertical radius in layer-local primitive units.
    RadiusY,
}

/// Reached morphology arithmetic that could not be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MorphologyNumericPhase {
    /// Resolving a positive radius against its actual axis density.
    RadiusGrid,
    /// Mapping bounded query endpoints between input and output grids.
    EndpointMapping,
    /// Qualifying extrema-table levels, spans and storage extents.
    TableLayout,
    /// Qualifying reachable source indices and reconstruction neighborhoods.
    GridIndex,
    /// Qualifying uploaded sampling arithmetic and intermediate values.
    Sampling,
}

/// Typed morphology refusal during admission, source construction or frame qualification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MorphologyPreparationError {
    /// A reached resolved radius is not finite.
    InvalidParameter {
        /// Parameter being inspected.
        parameter: MorphologyParameter,
        /// Supplied resolved value.
        value: f64,
    },
    /// Actual reachable arithmetic cannot be represented safely.
    Numeric {
        /// Boundary at which representation was refused.
        phase: MorphologyNumericPhase,
        /// Supplied or intermediate value at that boundary.
        value: f64,
    },
    /// A periodic source requires positive width and height.
    EmptyPeriod,
    /// Same-layer backdrop is incompatible with a declared infinite input.
    InfiniteSourceBackdrop,
}

/// Untrusted resolved independent morphology radii and component operator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorphologyInput {
    radii: [f64; 2],
    operator: MorphologyOperator,
}
impl MorphologyInput {
    /// Requests erosion with zero radii on both axes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            radii: [0.0; 2],
            operator: MorphologyOperator::Erode,
        }
    }
    /// Sets the same resolved radius on both axes.
    #[must_use]
    pub const fn with_radius(mut self, radius: f64) -> Self {
        self.radii = [radius; 2];
        self
    }
    /// Sets independent resolved horizontal and vertical radii.
    #[must_use]
    pub const fn with_radii(mut self, x: f64, y: f64) -> Self {
        self.radii = [x, y];
        self
    }
    /// Selects component-wise minimum or maximum.
    #[must_use]
    pub const fn with_operator(mut self, operator: MorphologyOperator) -> Self {
        self.operator = operator;
        self
    }
}
impl Default for MorphologyInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable inline morphology admitted independently of actual frame grids.
///
/// Any nonpositive supplied radius disables the whole primitive. Strictly
/// positive radii remain distinct even when frame resolution floors an axis to
/// zero. No coefficient allocation or native resource belongs to this value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedMorphology {
    radii: PreparedRadii,
    operator: MorphologyOperator,
    color_space: FilterColorSpace,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum PreparedRadii {
    Disabled,
    PositivePair([f64; 2]),
}
impl PreparedMorphology {
    pub(crate) const fn radii(self) -> Option<[f64; 2]> {
        match self.radii {
            PreparedRadii::Disabled => None,
            PreparedRadii::PositivePair(radii) => Some(radii),
        }
    }
    pub(crate) const fn operator(self) -> MorphologyOperator {
        self.operator
    }
    pub(crate) const fn color_space(self) -> FilterColorSpace {
        self.color_space
    }
    pub(crate) const fn is_disabled(self) -> bool {
        matches!(self.radii, PreparedRadii::Disabled)
    }
}

/// Admits radii in X/Y order through central accounting.
///
/// Both coordinates are inspected for finiteness before a nonpositive radius
/// disables the request. Each reached coordinate spends one `CoordinateVisits`;
/// the request spends one `FilterOperationVisits`. No payload is allocated.
pub fn prepare_morphology(
    input: MorphologyInput,
    color_space: FilterColorSpace,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedMorphology> {
    accounting.run(
        RenderAttemptKind::Morphology,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                for (parameter, value) in [
                    (MorphologyParameter::RadiusX, input.radii[0]),
                    (MorphologyParameter::RadiusY, input.radii[1]),
                ] {
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    if !value.is_finite() {
                        return Err(MorphologyPreparationError::InvalidParameter {
                            parameter,
                            value,
                        }
                        .into());
                    }
                }
                Ok(PreparedMorphology {
                    radii: if input.radii[0] <= 0.0 || input.radii[1] <= 0.0 {
                        PreparedRadii::Disabled
                    } else {
                        PreparedRadii::PositivePair(input.radii)
                    },
                    operator: input.operator,
                    color_space,
                })
            })
        },
    )
}

/// Authentic finite, constant or fundamental-period source for native morphology.
///
/// Finite sources always use transparent exterior. The existing Scene owner
/// retains commands until they are moved into Layer children; no second live
/// tree or independently selectable edge extension is introduced.
#[derive(Clone, Debug, PartialEq)]
pub struct MorphologySource(GaussianSource);
impl MorphologySource {
    /// Owns finite input with transparent extension outside its source domain.
    #[must_use]
    pub fn finite(scene: Scene, domains: FilterDomains) -> Self {
        Self(GaussianSource::finite(scene, domains))
    }
    /// Declares authentic constant unbounded input and finite output domains.
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
                    MorphologyPreparationError::EmptyPeriod.into()
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
pub(crate) struct MorphologyForeground {
    pub(crate) operation: PreparedMorphology,
    pub(crate) source: FilterSourceDescriptor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, Capabilities, Error, ErrorCode, Filter,
        FilterEdgeMode, FilterList, FilterOp, Layer, LayerComposition, PorterDuffOperator,
        RenderAccountingOptions, RenderAttemptStatus, RenderLimits, RenderQuantity, RenderResource,
        Transform, command::NormalizedForegroundEffect, layer::LayerFilter, scene::Command,
    };

    fn domains() -> FilterDomains {
        FilterDomains::try_new(
            Rect::new(-1.5, 2.5, 3.0, 4.0),
            Rect::new(-3.0, 1.0, 8.0, 9.0),
            Rect::new(1.25, -2.5, 7.0, 6.0),
        )
        .unwrap()
    }
    fn prepare(input: MorphologyInput) -> PreparedMorphology {
        prepare_morphology(
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
    fn defaults_disable_erosion_and_positive_radii_preserve_the_supplied_pair() {
        assert_eq!(MorphologyInput::new(), MorphologyInput::default());
        assert_eq!(MorphologyOperator::default(), MorphologyOperator::Erode);
        let default = prepare(MorphologyInput::default());
        assert!(default.is_disabled());
        assert_eq!(default.radii(), None);
        assert_eq!(default.operator(), MorphologyOperator::Erode);
        assert_eq!(default.color_space(), FilterColorSpace::LinearSrgb);
        let shared = prepare(
            MorphologyInput::new()
                .with_radius(2.5)
                .with_operator(MorphologyOperator::Dilate),
        );
        assert_eq!(shared.radii(), Some([2.5; 2]));
        assert_eq!(shared.operator(), MorphologyOperator::Dilate);
        for radii in [
            [0.25, 2.0],
            [2.0, 0.25],
            [0.25, 0.25],
            [f64::MAX, f64::MIN_POSITIVE],
        ] {
            let value = prepare(MorphologyInput::new().with_radii(radii[0], radii[1]));
            assert_eq!(value.radii(), Some(radii));
            assert!(!value.is_disabled());
        }
        for radii in [
            [0.0, 2.0],
            [2.0, 0.0],
            [-0.0, 2.0],
            [2.0, -0.0],
            [-1.0, f64::MAX],
            [f64::MAX, -1.0],
            [-1.0, -2.0],
        ] {
            let value = prepare(MorphologyInput::new().with_radii(radii[0], radii[1]));
            assert!(value.is_disabled());
            assert_eq!(value.radii(), None);
        }
    }

    #[test]
    fn nonfinite_radii_fail_in_axis_order_before_nonpositive_disabling() {
        for (parameter, radii, visits) in [
            (MorphologyParameter::RadiusX, [f64::NAN, f64::INFINITY], 1),
            (MorphologyParameter::RadiusX, [f64::NEG_INFINITY, -1.0], 1),
            (MorphologyParameter::RadiusY, [-1.0, f64::INFINITY], 2),
            (MorphologyParameter::RadiusY, [0.0, f64::NAN], 2),
        ] {
            let mut accounting = RenderAccountingContext::default();
            let error = prepare_morphology(
                MorphologyInput::new().with_radii(radii[0], radii[1]),
                FilterColorSpace::Srgb,
                &mut accounting,
            )
            .unwrap_err();
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            let Some(MorphologyPreparationError::InvalidParameter {
                parameter: actual,
                value,
            }) = error.morphology_diagnostic()
            else {
                panic!("missing morphology diagnostic")
            };
            assert_eq!(*actual, parameter);
            assert_eq!(value.to_bits(), radii[(visits - 1) as usize].to_bits());
            assert!(error.gaussian_diagnostic().is_none());
            let report = accounting.last_attempt().unwrap();
            assert_eq!(report.kind(), RenderAttemptKind::Morphology);
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
    fn exact_preparation_budget_has_no_payload_and_refusal_preserves_retry() {
        let exact = RenderLimits::new()
            .with_work_limit(RenderWork::FilterOperationVisits, 1)
            .unwrap()
            .with_work_limit(RenderWork::CoordinateVisits, 2)
            .unwrap()
            .with_retained_bytes(0)
            .with_scratch_bytes(0);
        let options = RenderAccountingOptions::new().with_limits(exact);
        let mut accounting = RenderAccountingContext::new(options);
        for input in [
            MorphologyInput::default(),
            MorphologyInput::new().with_radii(0.25, 2.0),
        ] {
            let expected =
                prepare_morphology(input, FilterColorSpace::Srgb, &mut accounting).unwrap();
            let usage = accounting
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage();
            assert_eq!(usage.work(RenderWork::FilterOperationVisits), 1);
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
                let error =
                    prepare_morphology(input, FilterColorSpace::Srgb, &mut accounting).unwrap_err();
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
                    prepare_morphology(input, FilterColorSpace::Srgb, &mut accounting).unwrap(),
                    expected
                );
            }
        }
    }

    #[test]
    fn source_declarations_preserve_domains_and_map_empty_period_to_morphology() {
        assert_eq!(
            MorphologySource::finite(ink(), domains()).descriptor(),
            FilterSourceDescriptor::Finite {
                domains: domains(),
                edge: FilterEdgeMode::None
            }
        );
        let filter = domains().filter();
        let result = domains().result();
        let constant = MorphologySource::constant(Color::BLACK, filter, result).unwrap();
        assert_eq!(
            constant.descriptor(),
            FilterSourceDescriptor::Constant {
                color: Color::BLACK,
                filter,
                result
            }
        );
        let (scene, _) = constant.into_parts();
        assert!(scene.is_empty());
        let period = Rect::new(-3.25, 2.5, 1.5, 2.25);
        let periodic = MorphologySource::periodic(ink(), period, filter, result).unwrap();
        assert_eq!(
            periodic.descriptor(),
            FilterSourceDescriptor::Periodic {
                period,
                filter,
                result
            }
        );
        assert_eq!(periodic.into_parts().0, ink());
        for empty in [Rect::new(0.0, 0.0, 0.0, 1.0), Rect::new(0.0, 0.0, 1.0, 0.0)] {
            let error = MorphologySource::periodic(ink(), empty, filter, result).unwrap_err();
            assert_eq!(
                error.morphology_diagnostic(),
                Some(&MorphologyPreparationError::EmptyPeriod)
            );
            assert!(error.gaussian_diagnostic().is_none());
        }
        for invalid in [
            Rect::new(f64::NAN, 0.0, 1.0, 1.0),
            Rect::new(0.0, 0.0, -1.0, 1.0),
            Rect::new(f64::MAX, 0.0, f64::MAX, 1.0),
        ] {
            assert!(MorphologySource::constant(Color::BLACK, invalid, result).is_err());
            assert!(MorphologySource::constant(Color::BLACK, filter, invalid).is_err());
            assert!(MorphologySource::periodic(ink(), invalid, filter, result).is_err());
        }
    }

    #[test]
    fn receiver_replaces_foreground_preserving_outer_layer_and_child_order() {
        let operation = prepare(MorphologyInput::new().with_radii(1.0, 2.0));
        let mask = crate::ResolvedLayerAlphaMask::try_new(
            crate::Image::from_rgba(crate::Size::new(1.0, 1.0), vec![255, 255, 255, 128]).unwrap(),
            Rect::new(-1.0, 0.0, 5.0, 6.0),
        )
        .unwrap();
        let layer = Layer::new()
            .try_filter(Filter::try_blur(2.0).unwrap())
            .unwrap()
            .try_clip(crate::Shape::rect(Rect::new(-2.0, 0.0, 7.0, 8.0)))
            .unwrap()
            .with_resolved_alpha_mask(mask)
            .with_transform(Transform::try_new([1.0, 0.5, -0.25, 2.0, 3.0, -4.0]).unwrap())
            .isolated()
            .try_opacity(0.5)
            .unwrap()
            .with_composition(LayerComposition::normal_porter_duff(
                PorterDuffOperator::Copy,
            ));
        let expected = layer.clone().with_morphology(MorphologyForeground {
            operation,
            source: MorphologySource::finite(Scene::new(), domains()).descriptor(),
        });
        let mut child = ink();
        child.fill(Rect::new(4.0, 5.0, 1.0, 1.0), Color::BLACK);
        let authored = child.clone();
        let mut scene = Scene::new();
        scene
            .morphology_layer(layer, operation, MorphologySource::finite(child, domains()))
            .unwrap();
        let Command::Layer { layer, children } = &scene.commands[0] else {
            panic!("expected layer")
        };
        assert_eq!(**layer, expected);
        assert_eq!(children.as_slice(), authored.commands.as_slice());
        assert!(layer.filter().is_none());
        assert!(layer.filter_list().is_none());
        assert!(matches!(
            layer.foreground_filter(),
            Some(LayerFilter::Morphology(_))
        ));
        assert!(
            layer
                .clone()
                .with_filter_list(FilterList::none())
                .foreground_filter()
                .is_none()
        );
        RenderAccountingContext::default().run(RenderAttemptKind::Frame, RenderPhase::Normalize, |account| {
            let normalized = scene.normalize(Capabilities::CURRENT, account)?;
            let crate::command::RenderCommand::Layer { layer, .. } = &normalized.commands[0] else { panic!("expected normalized layer") };
            assert!(matches!(layer.foreground_filters.as_ref(), Some(NormalizedForegroundEffect::Morphology(value)) if value.operation == operation));
            Ok(())
        }).unwrap();
    }

    #[test]
    fn infinite_backdrop_refusal_appends_nothing_and_finite_backdrop_is_retained() {
        let operation = prepare(MorphologyInput::default());
        let mut scene = ink();
        let original = scene.clone();
        for source in [
            MorphologySource::constant(Color::BLACK, domains().filter(), domains().result())
                .unwrap(),
            MorphologySource::periodic(
                ink(),
                domains().source(),
                domains().filter(),
                domains().result(),
            )
            .unwrap(),
        ] {
            let error = scene
                .morphology_layer(backdrop_layer(), operation, source)
                .unwrap_err();
            assert_eq!(
                error.morphology_diagnostic(),
                Some(&MorphologyPreparationError::InfiniteSourceBackdrop)
            );
            assert!(error.gaussian_diagnostic().is_none());
            assert_eq!(scene, original);
        }
        scene
            .morphology_layer(
                backdrop_layer(),
                operation,
                MorphologySource::finite(ink(), domains()),
            )
            .unwrap();
        let Command::Layer { layer, .. } = &scene.commands[1] else {
            panic!("expected layer")
        };
        assert!(layer.backdrop_filter().is_some());
    }

    #[test]
    fn deep_source_value_operations_inspection_and_disposal_use_iterative_scene_custody() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let operation = prepare(MorphologyInput::default());
                let mut scene = ink();
                for _ in 0..2048 {
                    let mut parent = Scene::new();
                    parent
                        .morphology_layer(
                            Layer::new(),
                            operation,
                            MorphologySource::finite(scene, domains()),
                        )
                        .unwrap();
                    scene = parent;
                }
                let source = MorphologySource::periodic(
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
    fn numeric_diagnostics_and_capabilities_preserve_morphology_identity() {
        for phase in [
            MorphologyNumericPhase::RadiusGrid,
            MorphologyNumericPhase::EndpointMapping,
            MorphologyNumericPhase::TableLayout,
            MorphologyNumericPhase::GridIndex,
            MorphologyNumericPhase::Sampling,
        ] {
            let diagnostic = MorphologyPreparationError::Numeric {
                phase,
                value: f64::MAX,
            };
            let error: Error = diagnostic.into();
            assert_eq!(error.morphology_diagnostic(), Some(&diagnostic));
            assert!(error.gaussian_diagnostic().is_none());
            assert!(error.convolution_diagnostic().is_none());
            assert!(error.offset_diagnostic().is_none());
        }
        assert!(
            Capabilities::CURRENT
                .filters()
                .supports_gpu_morphology_filter_execution()
        );
        let unavailable = crate::RuntimeCapabilityUnavailable::filter_float32_unavailable(
            crate::RuntimeOperation::MorphologyRendering,
        );
        assert_eq!(
            unavailable.operation(),
            crate::RuntimeOperation::MorphologyRendering
        );
    }
}
