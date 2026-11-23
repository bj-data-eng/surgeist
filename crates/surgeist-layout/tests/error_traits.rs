#![forbid(unsafe_code)]

//! Semantic `Display` text and `source()` chains of the layout error types.
//!
//! Every expected string is written from the documented meaning of its
//! variant, not from the current output. A wrapper prints only its own
//! context and returns its payload from `source()`, so a full chain is the
//! `Display` of every level.

use core::convert::Infallible;
use std::error::Error;
use surgeist_layout as l;
use surgeist_text as text;

fn is_error<E: Error + 'static>() {}

/// The `Display` of an error and of every level of its `source()` chain.
fn chain(error: &(dyn Error + 'static)) -> Vec<String> {
    let mut out = vec![error.to_string()];
    let mut next = error.source();
    while let Some(level) = next {
        out.push(level.to_string());
        next = level.source();
    }
    out
}

fn assert_text_is_plain(text: &str, what: &str) {
    assert!(
        !text.contains(['{', '}']),
        "message of {what} contains struct syntax: {text}"
    );
}

#[derive(Debug, PartialEq)]
struct HostFailure;

impl core::fmt::Display for HostFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("the host could not measure the node")
    }
}

impl Error for HostFailure {}

fn error_of<M>(
    site: l::LayoutErrorSiteOf<u32>,
    operation: l::LayoutOperation,
    kind: l::LayoutErrorKindOf<f32, M>,
) -> l::LayoutErrorOf<u32, f32, M> {
    l::LayoutErrorOf::new(site, operation, kind)
}

fn at_node<M>(kind: l::LayoutErrorKindOf<f32, M>) -> l::LayoutErrorOf<u32, f32, M> {
    error_of(
        l::LayoutErrorSiteOf::Node(7),
        l::LayoutOperation::RootLayout,
        kind,
    )
}

// ---------------------------------------------------------------- group B

#[test]
fn calc_size_calculation_error_names_the_coefficient_and_sources_the_scalar_error() {
    is_error::<l::CalcSizeCalculationErrorOf<f32>>();
    let rejected = |a, p, s| {
        l::CalcSizeCalculationOf::<f32>::from_coefficients(a, p, s)
            .expect_err("a non-finite coefficient is rejected")
    };
    assert_eq!(
        chain(&rejected(f32::NAN, 0.0, 0.0)),
        [
            "absolute length coefficient must be finite",
            "scalar must be finite"
        ]
    );
    assert_eq!(
        chain(&rejected(0.0, f32::INFINITY, 0.0)),
        [
            "percentage coefficient must be finite",
            "scalar must be finite"
        ]
    );
    assert_eq!(
        chain(&rejected(0.0, 0.0, f32::NEG_INFINITY)),
        ["size coefficient must be finite", "scalar must be finite"]
    );
}

#[test]
fn length_percentage_error_names_the_coefficient_and_sources_the_scalar_error() {
    is_error::<l::LengthPercentageErrorOf<f32>>();
    let rejected = |a, p| {
        l::LengthPercentageOf::<f32>::from_coefficients(a, p)
            .expect_err("a non-finite coefficient is rejected")
    };
    assert_eq!(
        chain(&rejected(f32::NAN, 0.0)),
        [
            "absolute length coefficient must be finite",
            "scalar must be finite"
        ]
    );
    assert_eq!(
        chain(&rejected(0.0, f32::NAN)),
        [
            "percentage coefficient must be finite",
            "scalar must be finite"
        ]
    );
}

#[test]
fn inline_boundary_edge_error_names_the_component_and_sources_the_scalar_error() {
    is_error::<l::InlineBoundaryEdgeErrorOf<f32>>();
    let rejected = |margin, border, padding| {
        l::InlineBoundaryEdgeOf::<f32>::try_new(margin, border, padding)
            .expect_err("the components are rejected")
    };
    assert_eq!(
        chain(&rejected(f32::NAN, 0.0, 0.0)),
        ["inline margin must be finite", "scalar must be finite"]
    );
    assert_eq!(
        chain(&rejected(0.0, -1.0, 0.0)),
        [
            "inline border must be finite and nonnegative",
            "scalar must be non-negative"
        ]
    );
    assert_eq!(
        chain(&rejected(0.0, 0.0, f32::NAN)),
        [
            "inline padding must be finite and nonnegative",
            "scalar must be finite"
        ]
    );
    assert_eq!(
        chain(&rejected(0.0, f32::MAX, f32::MAX)),
        [
            "inline border/padding sum must be finite",
            "scalar must be finite"
        ]
    );
    assert_eq!(
        chain(&rejected(f32::MAX, f32::MAX, 0.0)),
        ["inline signed flow must be finite", "scalar must be finite"]
    );
}

#[test]
fn inline_boundary_edge_errors_without_a_payload_have_no_source() {
    let lost_margin = l::InlineBoundaryEdgeOf::<f32>::try_new(1.0, f32::MAX, 0.0)
        .expect_err("a margin lost against a huge strip is rejected");
    assert_eq!(
        chain(&lost_margin),
        ["inline signed flow lost its nonzero margin"]
    );
    let lost_strip = l::InlineBoundaryEdgeOf::<f32>::try_new(f32::MAX, 1.0, 0.0)
        .expect_err("a strip lost against a huge margin is rejected");
    assert_eq!(
        chain(&lost_strip),
        ["inline signed flow lost its nonzero strip extent"]
    );
}

// ---------------------------------------------------------------- group C

#[test]
fn site_names_the_node_and_the_axis_without_a_debug_form() {
    let kind = || l::LayoutErrorKindOf::<f32, Infallible>::Allocation;
    let site = |site| {
        l::LayoutErrorOf::<u32, f32, Infallible>::new(
            site,
            l::LayoutOperation::HiddenLayout,
            kind(),
        )
        .to_string()
    };
    assert_eq!(
        site(l::LayoutErrorSiteOf::Node(4)),
        "hidden layout failed at node 4: layout could not allocate its working storage"
    );
    assert_eq!(
        site(l::LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 9,
        }),
        "hidden layout failed at subject 9 in container 7: \
         layout could not allocate its working storage"
    );
    assert_eq!(
        site(l::LayoutErrorSiteOf::Standalone),
        "hidden layout failed at a standalone request: \
         layout could not allocate its working storage"
    );
    for (axis, label) in [
        (l::PhysicalAxis::Horizontal, "horizontal"),
        (l::PhysicalAxis::Vertical, "vertical"),
    ] {
        assert_eq!(
            site(l::LayoutErrorSiteOf::FragmentContainingBlock {
                container: 7,
                subject: 9,
                fragmentainer_index: 2,
                axis,
            }),
            format!(
                "hidden layout failed at fragment containing block of subject 9 in container 7 \
                 (fragmentainer 2, {label} axis): layout could not allocate its working storage"
            )
        );
    }
}

#[test]
fn error_kinds_state_the_kind_of_failure_and_leave_the_detail_to_the_payload() {
    use l::LayoutErrorKindOf as K;
    let table: Vec<(K<f32, HostFailure>, &str)> = vec![
        (
            K::InvalidInput(l::LayoutInvalidInputOf::TreeTopologyCycle),
            "layout input is invalid",
        ),
        (
            K::BlockFragmentation(l::BlockFragmentationFailure::InvalidSource),
            "block fragmentation failed",
        ),
        (
            K::MissingContext(l::LayoutMissingContext::RequiredBasis),
            "context that layout requires is missing",
        ),
        (
            K::UnsupportedCapability(l::LayoutUnsupportedCapability::NonNumericLength),
            "layout does not support a capability its input requires",
        ),
        (K::Measurement(HostFailure), "measurement failed"),
        (
            K::Allocation,
            "layout could not allocate its working storage",
        ),
        (
            K::Text(text::Error::new(
                text::ErrorCode::InvalidTextProtocol,
                "bad request",
            )),
            "text operation refused",
        ),
        (
            K::ResourceLimit(text::Error::new(
                text::ErrorCode::ScopeAlignmentRefused,
                "budget spent",
            )),
            "layout resource limit reached",
        ),
        (
            K::TextFeedbackCycle,
            "text feedback repeated a complete basis without acceptance",
        ),
        (
            K::InlineFeedbackCycle,
            "inline avoidance repeated a basis without convergence",
        ),
        (
            K::TextSpacingBudget {
                available: 5.0,
                advance: 7.5,
            },
            "justification advance 7.5 exceeds the available line budget 5",
        ),
        (
            K::TextNumeric {
                value: 0.25,
                reason: l::LayoutTextNumericError::NonZeroLost,
            },
            "text value 0.25 failed a numeric conversion",
        ),
        (
            K::InternalInvariant(l::LayoutInternalInvariant::MissingStagedUnroundedOutput),
            "an internal layout invariant was violated",
        ),
    ];
    for (kind, expected) in table {
        assert_text_is_plain(expected, "the expectation");
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn error_chain_names_operation_site_and_kind_and_then_the_payload() {
    is_error::<l::LayoutErrorOf<u32, f32, HostFailure>>();
    let error = error_of::<HostFailure>(
        l::LayoutErrorSiteOf::Node(3),
        l::LayoutOperation::RootLayout,
        l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TreeTopologyCycle),
    );
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 3: layout input is invalid",
            "the tree contains a cycle"
        ]
    );
    let error = at_node::<HostFailure>(l::LayoutErrorKindOf::MissingContext(
        l::LayoutMissingContext::RequiredBasis,
    ));
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 7: context that layout requires is missing",
            "a definite basis that the operation requires is missing"
        ]
    );
    let error = at_node::<HostFailure>(l::LayoutErrorKindOf::TextNumeric {
        value: 0.25,
        reason: l::LayoutTextNumericError::NonZeroLost,
    });
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 7: text value 0.25 failed a numeric conversion",
            "a nonzero contribution was lost to rounding or arithmetic"
        ]
    );
    let error = at_node::<HostFailure>(l::LayoutErrorKindOf::Measurement(HostFailure));
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 7: measurement failed",
            "the host could not measure the node"
        ]
    );
    let refusal = text::Error::new(text::ErrorCode::InvalidTextProtocol, "bad request");
    let expected = refusal.to_string();
    let error = at_node::<HostFailure>(l::LayoutErrorKindOf::Text(refusal));
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 7: text operation refused".to_string(),
            expected
        ]
    );
}

#[test]
fn failures_without_a_deeper_cause_have_no_source() {
    for kind in [
        l::LayoutErrorKindOf::<f32, HostFailure>::Allocation,
        l::LayoutErrorKindOf::TextFeedbackCycle,
        l::LayoutErrorKindOf::InlineFeedbackCycle,
        l::LayoutErrorKindOf::TextSpacingBudget {
            available: 1.0,
            advance: 2.0,
        },
    ] {
        assert!(at_node(kind).source().is_none());
    }
}

#[test]
fn resize_observer_measurement_error_names_the_axis_in_words() {
    is_error::<l::ResizeObserverMeasurementError>();
    let rejected = |origin, scale| {
        l::ResizeObserverPixelGrid::try_new(origin, scale).expect_err("the pixel grid is rejected")
    };
    assert_eq!(
        rejected(l::Point::new(f64::NAN, 0.0), 1.0).to_string(),
        "the content origin coordinate NaN on the horizontal axis is not finite"
    );
    assert_eq!(
        rejected(l::Point::new(0.0, f64::INFINITY), 1.0).to_string(),
        "the content origin coordinate inf on the vertical axis is not finite"
    );
    assert_eq!(
        rejected(l::Point::new(0.0, 0.0), 0.0).to_string(),
        "the device-pixel scale 0 is not finite and positive"
    );
    for (axis, label) in [
        (l::PhysicalAxis::Horizontal, "horizontal"),
        (l::PhysicalAxis::Vertical, "vertical"),
    ] {
        let error = l::ResizeObserverMeasurementError::DevicePixelExtentOutOfRange { axis };
        assert_eq!(
            error.to_string(),
            format!("the device-pixel extent on the {label} axis exceeds the range of u64")
        );
        assert!(error.source().is_none());
    }
}
