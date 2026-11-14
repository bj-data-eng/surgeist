use super::*;
use crate::{Layer, Rect, RenderAccountingOptions, RenderLimits, RenderProfile, RenderQuantity};

fn domains() -> FilterDomains {
    let rect = Rect::try_new(0.0, 0.0, 4.0, 3.0).unwrap();
    FilterDomains::try_new(rect, rect, rect).unwrap()
}
fn prepare(input: ConvolutionInput<'_>) -> Result<PreparedConvolution> {
    prepare_convolution(
        input,
        domains(),
        FilterColorSpace::LinearSrgb,
        &mut RenderAccountingContext::default(),
    )
}

#[test]
fn truncation_rectangular_defaults_and_original_row_major_weights() {
    let output =
        prepare(ConvolutionInput::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).with_order(2.9, Some(3.8)))
            .unwrap();
    assert_eq!(output.order(), [2, 3]);
    assert_eq!(output.target(), [1, 1]);
    assert_eq!(
        output.weights(),
        &[
            1.0f32 / 21.0,
            2.0 / 21.0,
            3.0 / 21.0,
            4.0 / 21.0,
            5.0 / 21.0,
            6.0 / 21.0
        ]
    );
    assert_eq!(output.color_space(), FilterColorSpace::LinearSrgb);
    assert!(output.same_allocation(&output.clone()));
}

#[test]
fn mismatch_ignores_unused_invalid_executable_parameters() {
    let output = prepare(
        ConvolutionInput::new(&[f64::NAN])
            .with_target(Some(-4.0), Some(f64::NAN))
            .with_bias(f64::INFINITY)
            .with_divisor(f64::NAN),
    )
    .unwrap();
    assert!(output.is_passthrough());
    assert!(output.weights().is_empty());
    let error = prepare(ConvolutionInput::new(&[]).with_order(0.9, None)).unwrap_err();
    assert!(matches!(
        error.convolution_diagnostic(),
        Some(ConvolutionPreparationError::InvalidParameter {
            parameter: ConvolutionParameter::OrderX,
            ..
        })
    ));
}

#[test]
fn scale_invariant_identity_and_explicit_divisor_skip_unused_sum() {
    for scale in [2.0f64.powi(-130), 2.0f64.powi(130)] {
        let output = prepare(
            ConvolutionInput::new(&[scale])
                .with_order(1.0, None)
                .with_divisor(scale),
        )
        .unwrap();
        assert_eq!(output.weights(), &[1.0]);
    }
    let output = prepare(
        ConvolutionInput::new(&[f64::MAX, f64::MAX])
            .with_order(2.0, Some(1.0))
            .with_divisor(f64::MAX),
    )
    .unwrap();
    assert_eq!(output.weights(), &[1.0, 1.0]);
}

#[test]
fn divisor_zero_default_zero_sum_signed_bias_and_units() {
    let output = prepare(
        ConvolutionInput::new(&[-2.0, 2.0])
            .with_order(2.0, Some(1.0))
            .with_divisor(0.0)
            .with_target(Some(0.0), None)
            .with_bias(-0.25)
            .with_preserve_alpha(true)
            .with_unit_length(0.5, Some(2.0))
            .with_edge_mode(FilterEdgeMode::Mirror),
    )
    .unwrap();
    assert_eq!(output.weights(), &[-2.0, 2.0]);
    assert_eq!(output.target(), [0, 0]);
    assert_eq!(output.bias(), -0.25);
    assert!(output.preserve_alpha());
    assert_eq!(output.unit_length(), Some([0.5, 2.0]));
    assert_eq!(output.edge_mode(), FilterEdgeMode::Mirror);
    let signed = prepare(
        ConvolutionInput::new(&[2.0])
            .with_order(1.0, None)
            .with_divisor(-2.0),
    )
    .unwrap();
    assert_eq!(signed.weights(), &[-1.0]);
    for value in [-1.0, 0.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            prepare(
                ConvolutionInput::new(&[1.0])
                    .with_order(1.0, None)
                    .with_unit_length(value, None)
            )
            .unwrap()
            .unit_length(),
            None
        );
    }
}

#[test]
fn normalization_underflow_subnormal_bias_and_overflow_are_typed_refusals() {
    let cases = [
        (f64::MIN_POSITIVE, f64::MAX, 0.0),
        (2.0f64.powi(-130), 1.0, 0.0),
        (1.0, 1.0, 2.0f64.powi(-130)),
        (f64::from(f32::MAX) / 4.0, 1.0, 0.0),
    ];
    for (coefficient, divisor, bias) in cases {
        assert!(matches!(
            prepare(
                ConvolutionInput::new(&[coefficient])
                    .with_order(1.0, None)
                    .with_divisor(divisor)
                    .with_bias(bias)
            )
            .unwrap_err()
            .convolution_diagnostic(),
            Some(ConvolutionPreparationError::Numeric { .. })
        ));
    }
    assert!(
        prepare(ConvolutionInput::new(&[f64::MAX, f64::MAX]).with_order(2.0, Some(1.0))).is_err()
    );
    assert!(
        prepare(
            ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_target(Some(0.5), None)
        )
        .is_err()
    );
}

#[test]
fn multi_batch_signed_witness_is_admitted_without_clamping() {
    let mut kernel = [0.0; 33];
    kernel[0] = 65536.0;
    kernel[32] = -65536.0;
    let output = prepare(
        ConvolutionInput::new(&kernel)
            .with_order(33.0, Some(1.0))
            .with_divisor(1.0),
    )
    .unwrap();
    assert_eq!(output.weights()[0], 65536.0);
    assert_eq!(output.weights()[32], -65536.0);
}

#[test]
fn foreground_effect_setters_replace_one_exclusive_slot() {
    let output = prepare(ConvolutionInput::new(&[1.0]).with_order(1.0, None)).unwrap();
    let layer = Layer::new().with_convolution(output.clone());
    assert_eq!(layer.convolution(), Some(&output));
    assert!(layer.filter_list().is_none());
    let layer = layer.with_filter_list(crate::FilterList::none());
    assert!(layer.convolution().is_none());
    let layer = layer
        .with_convolution(output)
        .try_filter(crate::Filter::try_blur(1.0).unwrap())
        .unwrap();
    assert!(layer.convolution().is_none());
}

#[test]
fn preparation_counts_profiles_exact_cap_refusal_and_retry() {
    let mut counts = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: crate::RenderPhaseFilter::All,
        },
    ] {
        let options = RenderAccountingOptions::default().with_profile(profile);
        let mut accounting = RenderAccountingContext::new(options);
        prepare_convolution(
            ConvolutionInput::new(&[1.0]).with_order(1.0, None),
            domains(),
            FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap();
        let report = accounting.last_attempt().unwrap();
        assert_eq!(report.kind(), RenderAttemptKind::Convolution);
        let work = report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::KernelSampleVisits);
        assert_eq!(work, 3); // validation/default sum, normalization, then independent absolute-weight bound
        if let Some(expected) = counts {
            assert_eq!(work, expected);
        } else {
            counts = Some(work);
        }
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::KernelSampleVisits, work - 1)
            .unwrap();
        accounting.set_options(options.with_limits(limits));
        let error = prepare_convolution(
            ConvolutionInput::new(&[1.0]).with_order(1.0, None),
            domains(),
            FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap_err();
        assert!(
            matches!(error.accounting_diagnostic(),Some(crate::RenderAccountingError::Refused(refusal)) if refusal.quantity() == RenderQuantity::Work(RenderWork::KernelSampleVisits))
        );
        accounting.set_options(
            options.with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::KernelSampleVisits, work)
                    .unwrap(),
            ),
        );
        prepare_convolution(
            ConvolutionInput::new(&[1.0]).with_order(1.0, None),
            domains(),
            FilterColorSpace::Srgb,
            &mut accounting,
        )
        .unwrap();
    }
}

#[test]
fn backing_exact_cap_one_under_and_count_recovery_have_truthful_work() {
    let input = ConvolutionInput::new(&[1.0]).with_order(1.0, None);
    let mut accounting = RenderAccountingContext::default();
    let output =
        prepare_convolution(input, domains(), FilterColorSpace::Srgb, &mut accounting).unwrap();
    let bytes = output.own_backing_bytes().unwrap() as u64;
    let report = accounting.last_attempt().unwrap();
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .peak(crate::RenderResource::RetainedCpuBytes),
        bytes
    );
    accounting.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_retained_bytes(bytes - 1)),
    );
    let error =
        prepare_convolution(input, domains(), FilterColorSpace::Srgb, &mut accounting).unwrap_err();
    assert!(
        matches!(error.accounting_diagnostic(),Some(crate::RenderAccountingError::Refused(refusal))
        if refusal.quantity() == RenderQuantity::Resource(crate::RenderResource::RetainedCpuBytes))
    );
    accounting.set_options(
        RenderAccountingOptions::new().with_limits(RenderLimits::new().with_retained_bytes(bytes)),
    );
    prepare_convolution(input, domains(), FilterColorSpace::Srgb, &mut accounting).unwrap();
    accounting.set_options(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::KernelSampleVisits, 0)
                .unwrap(),
        ),
    );
    let passthrough = prepare_convolution(
        ConvolutionInput::new(&[f64::NAN]),
        domains(),
        FilterColorSpace::Srgb,
        &mut accounting,
    )
    .unwrap();
    assert!(passthrough.is_passthrough());
    assert_eq!(
        accounting
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::KernelSampleVisits),
        0
    );
}

#[test]
fn domain_failures_use_filter_invalid_value_diagnostics() {
    let valid = Rect::default();
    let error =
        FilterDomains::try_new(Rect::new(f64::INFINITY, 0.0, 1.0, 1.0), valid, valid).unwrap_err();
    assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
    let diagnostic = error
        .invalid_value_diagnostic()
        .expect("shared filter domains must use InvalidValue diagnostics");
    assert!(diagnostic.field().starts_with("filter domain"));
    assert!(error.convolution_diagnostic().is_none());
}

#[test]
fn independent_domains_remain_resolved_and_empty_regions_are_valid() {
    let source = Rect::try_new(-2.0, 1.0, 7.0, 4.0).unwrap();
    let filter = Rect::try_new(-1.5, 1.25, 4.0, 2.0).unwrap();
    let result = Rect::try_new(0.5, 1.5, 0.0, 1.0).unwrap();
    let domains = FilterDomains::try_new(source, filter, result).unwrap();
    assert_eq!(domains.source(), source);
    assert_eq!(domains.filter(), filter);
    assert_eq!(domains.result(), result);
    let output = prepare_convolution(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        domains,
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    assert_eq!(output.domains(), domains);
}

#[test]
fn native_convolution_capability_accepts_all_resolved_edge_modes() {
    assert!(
        crate::Capabilities::CURRENT
            .filters()
            .supports_gpu_convolution_filter_execution()
    );
    let primitive = crate::UnsupportedPrimitive::new(
        crate::PrimitiveFamily::Filters,
        crate::PrimitiveOperation::GpuConvolutionFilterExecution,
    );
    crate::Capabilities::CURRENT
        .ensure_supported(primitive)
        .unwrap();
    assert_eq!(
        primitive.operation().label(),
        "GPU convolution filter execution"
    );
    for edge in [
        FilterEdgeMode::Duplicate,
        FilterEdgeMode::Wrap,
        FilterEdgeMode::Mirror,
        FilterEdgeMode::None,
    ] {
        let output = prepare(
            ConvolutionInput::new(&[1.0])
                .with_order(1.0, None)
                .with_edge_mode(edge),
        )
        .unwrap();
        assert_eq!(output.edge_mode(), edge);
    }
}
