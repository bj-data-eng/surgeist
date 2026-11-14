//! Centrally admitted native image-space translation parameters.
use crate::accounting::{
    Account, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork,
};
use crate::{Error, FilterDomains, ImageRendering, Result};

/// Offset parameter whose resolved value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OffsetParameter {
    /// Horizontal displacement in layer-local primitive units.
    Dx,
    /// Vertical displacement in layer-local primitive units.
    Dy,
}

/// Actual frame sampling arithmetic that could not be represented safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OffsetNumericPhase {
    /// Representability of the inverse source/result grid mapping.
    GridMapping,
    /// Checked reachable source indices and reconstruction neighborhoods.
    GridIndex,
}

/// Typed offset refusal during preparation or actual frame qualification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OffsetPreparationError {
    /// A reached resolved displacement is not finite.
    InvalidParameter {
        /// Parameter being validated.
        parameter: OffsetParameter,
        /// Supplied resolved value.
        value: f64,
    },
    /// Actual frame sampling cannot be represented under the native float32 policy.
    Numeric {
        /// Numeric boundary at which representation was refused.
        phase: OffsetNumericPhase,
        /// Supplied or intermediate value at that boundary.
        value: f64,
    },
}

/// Untrusted resolved displacement and captured-input reconstruction intent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffsetInput {
    dx: f64,
    dy: f64,
    rendering: ImageRendering,
}

impl OffsetInput {
    /// Requests zero displacement and automatic reconstruction.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dx: 0.0,
            dy: 0.0,
            rendering: ImageRendering::Auto,
        }
    }

    /// Sets horizontal displacement in layer-local primitive units.
    #[must_use]
    pub const fn with_dx(mut self, dx: f64) -> Self {
        self.dx = dx;
        self
    }

    /// Sets vertical displacement in layer-local primitive units.
    #[must_use]
    pub const fn with_dy(mut self, dy: f64) -> Self {
        self.dy = dy;
        self
    }

    /// Selects reconstruction of the captured input; child image hints are independent.
    #[must_use]
    pub const fn with_rendering(mut self, rendering: ImageRendering) -> Self {
        self.rendering = rendering;
        self
    }
}

impl Default for OffsetInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable inline offset admitted independently of any device or capture grid.
///
/// Finite displacement does not guarantee that every future frame's sampling
/// arithmetic is representable. Actual capture/result grids are qualified when
/// planning the frame. This value owns no heap payload or native GPU resource.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedOffset {
    displacement: OffsetDisplacement,
    domains: FilterDomains,
}

impl PreparedOffset {
    pub(crate) const fn domains(self) -> FilterDomains {
        self.domains
    }

    pub(crate) const fn dx(self) -> f64 {
        self.displacement.input.dx
    }

    pub(crate) const fn dy(self) -> f64 {
        self.displacement.input.dy
    }

    pub(crate) const fn rendering(self) -> ImageRendering {
        self.displacement.input.rendering
    }
}

/// Finite displacement and reconstruction intent shared by native effects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OffsetDisplacement {
    input: OffsetInput,
}
impl OffsetDisplacement {
    pub(crate) const fn with_domains(self, domains: FilterDomains) -> PreparedOffset {
        PreparedOffset {
            displacement: self,
            domains,
        }
    }
}

/// Admits scalar displacement inside an already charged native operation.
pub(crate) fn prepare_offset_displacement_in(
    input: OffsetInput,
    account: &Account,
) -> Result<OffsetDisplacement> {
    for (parameter, value) in [
        (OffsetParameter::Dx, input.dx),
        (OffsetParameter::Dy, input.dy),
    ] {
        account.work(RenderWork::CoordinateVisits, 1)?;
        if !value.is_finite() {
            return Err(OffsetPreparationError::InvalidParameter { parameter, value }.into());
        }
    }
    Ok(OffsetDisplacement { input })
}

/// Admits one finite native image-space offset through central accounting.
///
/// Translation moves captured input within the independent filter and result
/// clips. It preserves captured premultiplied color and the layer's outer affine
/// transform. Preparation inspects dx, dy, then each source/filter/result domain;
/// the first refusal retains only the work reached. No payload allocation occurs.
pub fn prepare_offset(
    input: OffsetInput,
    domains: FilterDomains,
    accounting: &mut RenderAccountingContext,
) -> Result<PreparedOffset> {
    accounting.run(
        RenderAttemptKind::Offset,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                let displacement = prepare_offset_displacement_in(input, account)?;
                for (role, rect) in [
                    ("source", domains.source()),
                    ("filter", domains.filter()),
                    ("result", domains.result()),
                ] {
                    for (field, value, extent) in [
                        ("x", rect.x(), false),
                        ("y", rect.y(), false),
                        ("width", rect.width(), true),
                        ("height", rect.height(), true),
                        ("max x", rect.max().x(), false),
                        ("max y", rect.max().y(), false),
                    ] {
                        account.work(RenderWork::CoordinateVisits, 1)?;
                        if !value.is_finite() || (extent && value < 0.0) {
                            return Err(Error::invalid_value(
                                format!("filter domain {role} {field}"),
                                value,
                                if extent {
                                    "must be finite and non-negative"
                                } else {
                                    "must be finite"
                                },
                            ));
                        }
                    }
                }
                Ok(displacement.with_domains(domains))
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ErrorCode, Filter, FilterList, Layer, Rect, RenderAccountingOptions, RenderAttemptStatus,
        RenderLimits, RenderQuantity, RenderResource,
    };

    fn domains() -> FilterDomains {
        FilterDomains::try_new(
            Rect::try_new(-2.5, 3.25, 4.0, 5.0).unwrap(),
            Rect::try_new(-4.0, 1.0, 8.0, 9.0).unwrap(),
            Rect::try_new(1.0, 2.0, 7.5, 8.5).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn default_and_finite_requests_preserve_independent_domains_and_hint() {
        let mut accounting = RenderAccountingContext::default();
        let default = prepare_offset(OffsetInput::new(), domains(), &mut accounting).unwrap();
        assert_eq!(default.dx(), 0.0);
        assert_eq!(default.dy(), 0.0);
        assert_eq!(default.rendering(), ImageRendering::Auto);
        assert_eq!(default.domains(), domains());

        for rendering in [
            ImageRendering::Auto,
            ImageRendering::Smooth,
            ImageRendering::Pixelated,
            ImageRendering::CrispEdges,
            ImageRendering::HighQuality,
        ] {
            let prepared = prepare_offset(
                OffsetInput::default()
                    .with_dx(f64::MAX)
                    .with_dy(-f64::MIN_POSITIVE)
                    .with_rendering(rendering),
                domains(),
                &mut accounting,
            )
            .unwrap();
            assert_eq!(prepared.dx(), f64::MAX);
            assert_eq!(prepared.dy(), -f64::MIN_POSITIVE);
            assert_eq!(prepared.rendering(), rendering);
            assert_eq!(prepared.domains(), domains());
        }
    }

    #[test]
    fn nonfinite_displacement_reports_axis_and_only_reached_work() {
        for (parameter, visits) in [(OffsetParameter::Dx, 1), (OffsetParameter::Dy, 2)] {
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let input = match parameter {
                    OffsetParameter::Dx => OffsetInput::new().with_dx(value),
                    OffsetParameter::Dy => OffsetInput::new().with_dy(value),
                };
                let mut accounting = RenderAccountingContext::default();
                let error = prepare_offset(input, domains(), &mut accounting).unwrap_err();
                assert_eq!(error.code(), ErrorCode::InvalidInput);
                let Some(OffsetPreparationError::InvalidParameter {
                    parameter: actual,
                    value: actual_value,
                }) = error.offset_diagnostic()
                else {
                    panic!("expected an offset parameter diagnostic");
                };
                assert_eq!(*actual, parameter);
                assert_eq!(actual_value.to_bits(), value.to_bits());
                assert!(error.convolution_diagnostic().is_none());
                let report = accounting.last_attempt().unwrap();
                assert_eq!(report.kind(), RenderAttemptKind::Offset);
                assert_eq!(
                    report.status(),
                    RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
                );
                let usage = report.attempt().unwrap().usage();
                assert_eq!(usage.work(RenderWork::FilterOperationVisits), 1);
                assert_eq!(usage.work(RenderWork::CoordinateVisits), visits);
                assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
            }
        }
    }

    #[test]
    fn exact_preparation_budget_succeeds_without_payload_backing_and_retry_is_identical() {
        let exact = RenderLimits::new()
            .with_work_limit(RenderWork::FilterOperationVisits, 1)
            .unwrap()
            .with_work_limit(RenderWork::CoordinateVisits, 20)
            .unwrap()
            .with_retained_bytes(0)
            .with_scratch_bytes(0);
        let options = RenderAccountingOptions::new().with_limits(exact);
        let mut accounting = RenderAccountingContext::new(options);
        let prepared = prepare_offset(OffsetInput::new(), domains(), &mut accounting).unwrap();
        let usage = accounting
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage();
        assert_eq!(usage.work(RenderWork::FilterOperationVisits), 1);
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 20);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);

        for (work, limit) in [
            (RenderWork::FilterOperationVisits, 0),
            (RenderWork::CoordinateVisits, 19),
        ] {
            accounting
                .set_options(options.with_limits(exact.with_work_limit(work, limit).unwrap()));
            let error = prepare_offset(OffsetInput::new(), domains(), &mut accounting).unwrap_err();
            assert!(
                matches!(error.accounting_diagnostic(), Some(crate::RenderAccountingError::Refused(refusal))
                if refusal.quantity() == RenderQuantity::Work(work))
            );
            let report = accounting.last_attempt().unwrap();
            assert_eq!(report.kind(), RenderAttemptKind::Offset);
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.attempt().unwrap().usage().work(work), limit);
            accounting.set_options(options);
            assert_eq!(
                prepare_offset(OffsetInput::new(), domains(), &mut accounting).unwrap(),
                prepared
            );
        }
    }

    #[test]
    fn selecting_offset_replaces_and_is_replaced_by_the_foreground_slot() {
        let prepared = prepare_offset(
            OffsetInput::new().with_dx(0.5),
            domains(),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let layer = Layer::new()
            .try_filter(Filter::try_blur(2.0).unwrap())
            .unwrap()
            .with_offset(prepared);
        assert_eq!(layer.offset(), Some(&prepared));
        assert!(layer.filter().is_none());
        assert!(layer.filter_list().is_none());
        assert!(layer.convolution().is_none());
        assert!(
            layer
                .with_filter_list(FilterList::none())
                .offset()
                .is_none()
        );
    }

    #[test]
    fn numeric_frame_refusal_keeps_the_offset_phase_and_scalar() {
        for phase in [
            OffsetNumericPhase::GridMapping,
            OffsetNumericPhase::GridIndex,
        ] {
            let diagnostic = OffsetPreparationError::Numeric {
                phase,
                value: f64::MAX,
            };
            let error: Error = diagnostic.into();
            assert_eq!(error.offset_diagnostic(), Some(&diagnostic));
            assert!(error.convolution_diagnostic().is_none());
            assert!(error.message().contains("offset"));
        }
    }
}
