//! `Display`, `Error` and cause retention of layout failures. The expected
//! text and sources come from the documented contract of each error kind.
use super::*;
use std::error::Error as _;
use surgeist_text as text;

#[derive(Debug, PartialEq)]
struct HostFailure(&'static str);

impl fmt::Display for HostFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "host failure: {}", self.0)
    }
}

impl std::error::Error for HostFailure {}

fn error_of<M>(kind: LayoutErrorKindOf<f64, M>) -> LayoutErrorOf<u32, f64, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(3),
        LayoutOperation::ChildLayout,
        kind,
    )
}

fn limit_error() -> text::Error {
    text::Error::new(text::ErrorCode::ScopeAlignmentRefused, "budget spent").with_detail(
        text::ErrorDetail::ScopeAlignment {
            reason: text::scope_alignment::ScopeAlignmentError::Limit {
                kind: text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork,
                required: 11,
                limit: 10,
            },
        },
    )
}

#[test]
fn display_names_operation_site_and_kind() {
    let error = LayoutErrorOf::<u32, f64>::new(
        LayoutErrorSiteOf::Node(3),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis),
    );
    assert_eq!(
        error.to_string(),
        "root layout failed at node 3: missing context: RequiredBasis"
    );
}

#[test]
fn display_of_each_site_names_every_node_involved() {
    let site = |site| {
        LayoutErrorOf::<u32, f64>::new(
            site,
            LayoutOperation::HiddenLayout,
            LayoutErrorKindOf::Allocation,
        )
        .to_string()
    };
    assert_eq!(
        site(LayoutErrorSiteOf::ContainerSubject {
            container: 7,
            subject: 9,
        }),
        "hidden layout failed at subject 9 in container 7: \
         layout could not allocate its working storage"
    );
    assert_eq!(
        site(LayoutErrorSiteOf::Standalone),
        "hidden layout failed at a standalone request: \
         layout could not allocate its working storage"
    );
    let fragment = site(LayoutErrorSiteOf::FragmentContainingBlock {
        container: 7,
        subject: 9,
        fragmentainer_index: 2,
        axis: PhysicalAxis::Vertical,
    });
    assert!(
        fragment.contains("subject 9") && fragment.contains("container 7"),
        "{fragment}"
    );
    assert!(fragment.contains("fragmentainer 2"), "{fragment}");
}

#[test]
fn measurement_failure_displays_and_sources_the_host_error() {
    let error = error_of(LayoutErrorKindOf::Measurement(HostFailure("no metrics")));
    assert_eq!(
        error.to_string(),
        "child layout failed at node 3: measurement failed"
    );
    let source = error.source().expect("a measurement failure has a cause");
    assert_eq!(
        source.downcast_ref::<HostFailure>(),
        Some(&HostFailure("no metrics"))
    );
}

#[test]
fn text_failure_sources_the_text_error() {
    let refusal = text::Error::new(text::ErrorCode::InvalidTextProtocol, "bad request");
    let error = error_of::<core::convert::Infallible>(LayoutErrorKindOf::Text(refusal));
    assert_eq!(
        error.to_string(),
        "child layout failed at node 3: text operation refused"
    );
    let source = error.source().expect("a Text refusal has a cause");
    assert!(source.to_string().contains("bad request"), "{source}");
    assert_eq!(
        source.downcast_ref::<text::Error>().map(|error| error.code),
        Some(text::ErrorCode::InvalidTextProtocol)
    );
}

#[test]
fn resource_limit_is_distinct_from_text_and_sources_the_exhausted_budget() {
    let error =
        error_of::<core::convert::Infallible>(LayoutErrorKindOf::ResourceLimit(limit_error()));
    assert!(!matches!(error.kind(), LayoutErrorKindOf::Text(_)));
    assert_eq!(
        error.to_string(),
        "child layout failed at node 3: layout resource limit reached"
    );
    let cause = error
        .source()
        .and_then(|source| source.downcast_ref::<text::Error>())
        .expect("the exhausted budget is the cause");
    assert!(matches!(
        cause.detail(),
        Some(text::ErrorDetail::ScopeAlignment {
            reason: text::scope_alignment::ScopeAlignmentError::Limit {
                kind: text::scope_alignment::ScopeAlignmentLimitKind::StructuralWork,
                required: 11,
                limit: 10,
            },
        })
    ));
}

#[test]
fn staging_refusal_is_a_resource_limit_only_for_exhausted_budgets() {
    let classify = |error| {
        staging_text_error::<u32, f64, core::convert::Infallible>(
            LayoutErrorSiteOf::Node(1),
            LayoutOperation::ChildLayout,
            error,
        )
    };
    let scope = |reason| {
        text::Error::new(text::ErrorCode::ScopeAlignmentRefused, "refused")
            .with_detail(text::ErrorDetail::ScopeAlignment { reason })
    };
    for budget in [
        limit_error(),
        scope(text::scope_alignment::ScopeAlignmentError::Overflow {
            kind: text::scope_alignment::ScopeAlignmentLimitKind::Records,
        }),
        scope(text::scope_alignment::ScopeAlignmentError::Allocation {
            kind: text::scope_alignment::ScopeAlignmentLimitKind::PayloadBytes,
        }),
    ] {
        assert!(
            matches!(classify(budget).kind(), LayoutErrorKindOf::ResourceLimit(_)),
            "an exhausted budget is a resource limit"
        );
    }
    for other in [
        scope(text::scope_alignment::ScopeAlignmentError::MissingCapture),
        scope(text::scope_alignment::ScopeAlignmentError::LimitsRequired),
        text::Error::new(text::ErrorCode::InvalidTextProtocol, "bad request"),
    ] {
        assert!(
            matches!(classify(other).kind(), LayoutErrorKindOf::Text(_)),
            "a refusal that is not an exhausted budget stays a Text refusal"
        );
    }
}

#[test]
fn rejected_measurement_output_sources_the_scalar_error() {
    let scalar = NonNegativeFiniteScalarErrorOf::<f64>::Negative { value: -1.0 };
    let error = error_of::<core::convert::Infallible>(LayoutErrorKindOf::InvalidInput(
        LayoutInvalidInputOf::MeasurementOutput(InvalidMeasurementOutputOf::new(
            PhysicalAxis::Horizontal,
            scalar,
        )),
    ));
    assert_eq!(
        error.to_string(),
        "child layout failed at node 3: measurement output rejected on the Horizontal axis"
    );
    let source = error.source().expect("the scalar error is the cause");
    assert_eq!(
        source.downcast_ref::<NonNegativeFiniteScalarErrorOf<f64>>(),
        Some(&scalar)
    );
}

#[test]
fn failures_without_a_deeper_cause_have_no_source() {
    let error = error_of::<core::convert::Infallible>(LayoutErrorKindOf::Allocation);
    assert!(error.source().is_none());
    let error = error_of::<core::convert::Infallible>(LayoutErrorKindOf::TextFeedbackCycle);
    assert!(error.source().is_none());
}

#[test]
fn scroll_geometry_failure_keeps_the_rejected_coordinate() {
    let rejected = crate::ScrollCoordinateErrorOf::<f64>::NonFinitePhysicalOffset {
        axis: PhysicalAxis::Horizontal,
        value: f64::INFINITY,
    };
    let error: LayoutErrorOf<u32, f64> =
        layout_own_geometry_error(5, RunMode::PerformLayout, rejected);
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(5));
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    let LayoutErrorKindOf::ScrollGeometry(cause) = error.kind() else {
        panic!("a rejected scroll coordinate is a scroll geometry failure: {error:?}");
    };
    let cause = format!("{cause:?}");
    assert!(
        cause.contains("NonFinitePhysicalOffset") && cause.contains("inf"),
        "{cause}"
    );
    assert_eq!(
        error.to_string(),
        "child layout failed at node 5: scroll geometry rejected"
    );
    let source = error.source().expect("the geometry failure is the cause");
    assert_eq!(source.to_string(), "a scroll coordinate is invalid");
}

#[test]
fn root_and_child_geometry_failures_report_their_own_operation() {
    let rejected = crate::ScrollCoordinateErrorOf::<f64>::NonFinitePhysicalOffset {
        axis: PhysicalAxis::Vertical,
        value: f64::NAN,
    };
    let root: LayoutErrorOf<u32, f64> =
        layout_own_geometry_error(1, RunMode::PerformRootLayout, rejected);
    assert_eq!(root.operation(), LayoutOperation::RootLayout);
    let child: LayoutErrorOf<u32, f64> = layout_child_geometry_error(1, 2, rejected);
    assert_eq!(child.operation(), LayoutOperation::ChildLayout);
    assert_eq!(
        child.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 1,
            subject: 2,
        }
    );
}

#[test]
fn resolved_length_on_a_failure_path_is_its_own_invariant() {
    let error: LayoutErrorOf<u32, f64> =
        value_resolution_error(4, LengthResolutionStatus::<f64>::Resolved);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::ResolvedLengthOnFailurePath)
    ));
}

#[test]
fn rejected_scroll_rectangle_is_reported_through_source_only() {
    let rejected = crate::ScrollRectErrorOf::<f64>::NegativeSize {
        axis: PhysicalAxis::Vertical,
        value: -2.0,
    };
    let error: LayoutErrorOf<u32, f64> =
        layout_own_geometry_error(5, RunMode::PerformLayout, rejected);
    let LayoutErrorKindOf::ScrollGeometry(geometry) = error.kind() else {
        panic!("a rejected scroll rectangle is a scroll geometry failure: {error:?}");
    };
    assert_eq!(geometry.to_string(), "a scroll rectangle is invalid");
    let cause = geometry
        .source()
        .and_then(|source| source.downcast_ref::<crate::ScrollRectErrorOf<f64>>())
        .expect("the rejected rectangle is the cause");
    assert_eq!(*cause, rejected);
    assert!(cause.to_string().contains("vertical"), "{cause}");
}

#[test]
fn rejected_float_exclusion_provider_output_displays_without_its_cause_and_sources_it() {
    let interval = FloatExclusionIntervalErrorOf::<f64>::NonFiniteBandMinimum { value: f64::NAN };
    let error = error_of::<core::convert::Infallible>(LayoutErrorKindOf::InvalidInput(
        LayoutInvalidInputOf::FloatExclusionProviderOutput { error: interval },
    ));
    assert_eq!(
        error.to_string(),
        "child layout failed at node 3: float exclusion provider output rejected"
    );
    let source = error.source().expect("the interval error is the cause");
    assert!(
        source
            .downcast_ref::<FloatExclusionIntervalErrorOf<f64>>()
            .is_some()
    );
}
