#![forbid(unsafe_code)]

//! Semantic `Display` text and `source()` chains of the layout error types.
//!
//! Every expected string is written from the documented meaning of its
//! variant, not from the current output. A wrapper prints only its own
//! context and returns its payload from `source()`, so a full chain is the
//! `Display` of every level.

#[path = "support/native_request.rs"]
mod native_request;

use core::convert::Infallible;
use std::error::Error;
use surgeist_layout as l;
use surgeist_text as text;

fn sizing_request() -> l::SizingRequest {
    native_request::sizing(
        l::Size::new(Some(100.0), Some(100.0)),
        l::Size::new(l::SizingOperation::Normal, l::SizingOperation::Normal),
    )
}

fn root_request() -> l::LayoutRootRequest {
    let sizing = sizing_request();
    l::LayoutRootRequest::standalone(
        sizing,
        l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr),
        sizing.orthogonal_fallback().initial_containing_block(),
    )
}

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

/// Checks a table with one row per variant: the exact message, plain text that
/// does not name its own variant, a message no other row shares, and no
/// `source()` for a payload-free row.
fn assert_messages<E>(table: &[(E, &str)], variants: usize)
where
    E: Error + core::fmt::Debug,
{
    assert_eq!(table.len(), variants, "one row per variant");
    let mut seen = std::collections::BTreeSet::new();
    for (value, expected) in table {
        assert_eq!(value.to_string(), *expected, "message of {value:?}");
        assert_text_is_plain(expected, &format!("{value:?}"));
        let debug = format!("{value:?}");
        let variant = debug
            .split(['(', ' ', '{'])
            .next()
            .expect("a Debug form names its variant");
        assert!(
            !expected.contains(variant),
            "message {expected:?} names its variant {variant}"
        );
        assert!(seen.insert(*expected), "two variants share {expected:?}");
    }
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

// ---------------------------------------------------------------- group D: scroll

#[test]
fn scroll_coordinate_error_names_the_space_axis_and_value() {
    use l::ScrollCoordinateErrorOf as E;
    is_error::<E<f32>>();
    let table: Vec<(E<f32>, &str)> = vec![
        (
            E::NonFinitePhysicalOffset {
                axis: l::PhysicalAxis::Horizontal,
                value: f32::INFINITY,
            },
            "the horizontal physical scroll offset inf is not finite",
        ),
        (
            E::NonFiniteFlowRelativeOffset {
                axis: l::LogicalAxis::Block,
                value: f32::NAN,
            },
            "the block flow-relative scroll offset NaN is not finite",
        ),
        (
            E::NonFinitePhysicalRangeMinimum {
                axis: l::PhysicalAxis::Vertical,
                value: f32::NEG_INFINITY,
            },
            "the vertical physical scroll range minimum -inf is not finite",
        ),
        (
            E::NonFinitePhysicalRangeMaximum {
                axis: l::PhysicalAxis::Horizontal,
                value: f32::INFINITY,
            },
            "the horizontal physical scroll range maximum inf is not finite",
        ),
        (
            E::NonFiniteFlowRelativeRangeMinimum {
                axis: l::LogicalAxis::Inline,
                value: f32::NAN,
            },
            "the inline flow-relative scroll range minimum NaN is not finite",
        ),
        (
            E::NonFiniteFlowRelativeRangeMaximum {
                axis: l::LogicalAxis::Block,
                value: f32::INFINITY,
            },
            "the block flow-relative scroll range maximum inf is not finite",
        ),
        (
            E::InvertedPhysicalRange {
                axis: l::PhysicalAxis::Vertical,
                minimum: 5.0,
                maximum: 2.5,
            },
            "the vertical physical scroll range minimum 5 exceeds its maximum 2.5",
        ),
        (
            E::InvertedFlowRelativeRange {
                axis: l::LogicalAxis::Inline,
                minimum: 1.0,
                maximum: -1.0,
            },
            "the inline flow-relative scroll range minimum 1 exceeds its maximum -1",
        ),
    ];
    let mut seen = [false; 8];
    for (error, expected) in &table {
        assert_text_is_plain(expected, "the expectation");
        assert_eq!(&error.to_string(), expected);
        assert!(error.source().is_none());
        seen[match error {
            E::NonFinitePhysicalOffset { .. } => 0,
            E::NonFiniteFlowRelativeOffset { .. } => 1,
            E::NonFinitePhysicalRangeMinimum { .. } => 2,
            E::NonFinitePhysicalRangeMaximum { .. } => 3,
            E::NonFiniteFlowRelativeRangeMinimum { .. } => 4,
            E::NonFiniteFlowRelativeRangeMaximum { .. } => 5,
            E::InvertedPhysicalRange { .. } => 6,
            E::InvertedFlowRelativeRange { .. } => 7,
        }] = true;
    }
    assert!(seen.iter().all(|&covered| covered));
}

#[test]
fn scroll_coordinate_constructors_return_errors_that_print_their_message() {
    let offset = l::PhysicalScrollOffsetOf::<f32>::try_new(f32::INFINITY, 0.0)
        .expect_err("a non-finite offset is rejected");
    assert_eq!(
        offset.to_string(),
        "the horizontal physical scroll offset inf is not finite"
    );
    let offset = l::FlowRelativeScrollOffsetOf::<f32>::try_new(0.0, f32::NAN)
        .expect_err("a non-finite offset is rejected");
    assert_eq!(
        offset.to_string(),
        "the block flow-relative scroll offset NaN is not finite"
    );
    let range = l::PhysicalScrollRangeOf::<f32>::try_new(2.0, 1.0, 0.0, 1.0)
        .expect_err("an inverted range is rejected");
    assert_eq!(
        range.to_string(),
        "the horizontal physical scroll range minimum 2 exceeds its maximum 1"
    );
    let range = l::FlowRelativeScrollRangeOf::<f32>::try_new(0.0, 1.0, 3.0, 2.0)
        .expect_err("an inverted range is rejected");
    assert_eq!(
        range.to_string(),
        "the block flow-relative scroll range minimum 3 exceeds its maximum 2"
    );
}

// ---------------------------------------------------------------- group D: baselines

#[test]
fn inline_baseline_error_names_the_rejected_fact() {
    use l::InlineBaselineError as E;
    is_error::<E>();
    let table = [
        (
            E::UnrepresentableCoordinate,
            "a named baseline coordinate cannot be represented in its output frame",
        ),
        (
            E::AxisMismatch {
                expected: l::PhysicalAxis::Horizontal,
                actual: l::PhysicalAxis::Vertical,
            },
            "an inline baseline on the vertical axis was supplied where the horizontal axis was \
             expected",
        ),
        (
            E::ReferenceMismatch {
                expected: l::InlineBaselineContext::Physical,
                actual: l::InlineBaselineContext::Named(
                    surgeist_text::font::FontBaseline::Alphabetic,
                ),
            },
            "an inline baseline reference differs from the reference the request expects",
        ),
    ];
    assert_messages(&table, 3);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn grid_baseline_error_names_the_rejected_fact() {
    use l::GridBaselineError as E;
    is_error::<E>();
    let table = [
        (
            E::AxisMismatch {
                expected: l::PhysicalAxis::Vertical,
                actual: l::PhysicalAxis::Horizontal,
            },
            "a grid baseline on the horizontal axis was supplied where the vertical axis was \
             expected",
        ),
        (
            E::UnrepresentableCoordinate,
            "a grid baseline coordinate cannot be represented in its output frame",
        ),
    ];
    assert_messages(&table, 2);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn inline_metrics_constructors_return_errors_that_print_the_rejected_metric() {
    is_error::<l::InlineMetricsError<f32>>();
    let physical = l::InlineBaselineContext::Physical;
    let rejected = |baseline, extent| {
        l::InlineMetricsOf::<f32>::try_new(physical, baseline, extent)
            .expect_err("the metrics are rejected")
    };
    assert_eq!(
        rejected(f32::NAN, 1.0).to_string(),
        "inline metric NaN is not finite"
    );
    assert_eq!(
        rejected(-1.0, 1.0).to_string(),
        "inline metric -1 is negative"
    );
    assert_eq!(
        rejected(5.0, 2.5).to_string(),
        "baseline 5 exceeds the line extent 2.5"
    );
    let error = l::InlineMetricsOf::<f32>::from_line_height_and_baseline(physical, 4.0, 6.0)
        .expect_err("a baseline below the line height is required");
    assert_eq!(error.to_string(), "baseline 6 exceeds the line height 4");
    assert!(error.source().is_none());
}

#[test]
fn text_binding_error_prints_its_message_and_has_no_source() {
    use l::TextBindingError as E;
    is_error::<E>();
    let table = [
        (E::EmptyRun, "a text run binds no nodes where it needs some"),
        (
            E::RunMembership,
            "the nodes bound to a text run do not match the run's membership in the tree",
        ),
        (
            E::ParticipantOrder,
            "the participants of a text run are not in source order",
        ),
        (
            E::DuplicateRunNode,
            "a node appears more than once in a text run",
        ),
        (E::OverlappingParagraphs, "two text paragraphs overlap"),
        (
            E::OriginCoverage,
            "the origin bindings do not cover the paragraph's origins exactly once",
        ),
        (
            E::AtomicCoverage,
            "the atomic bindings do not cover the paragraph's atomic boxes exactly once",
        ),
        (
            E::ControlCoverage,
            "the control bindings do not cover the paragraph's controls exactly once",
        ),
        (
            E::ScopeCoverage,
            "the scope bindings do not cover the paragraph's scopes exactly once",
        ),
        (
            E::ScopeAssociation,
            "a scope is not associated with the tree consistently with its text preparation",
        ),
        (
            E::ScopeBoundaryCapture,
            "a scope boundary is not captured by the text preparation",
        ),
        (
            E::ScopeBoundaryOrder,
            "scope boundaries are not in the order of the text preparation",
        ),
        (
            E::ScopeOccurrenceMembership,
            "a scope occurrence is not a member of its scope",
        ),
        (
            E::ScopeTreeAmbiguity,
            "a scope's association with the tree is ambiguous",
        ),
        (
            E::ParticipantRole,
            "a participant's role differs from the role the text preparation requires",
        ),
        (
            E::MissingParagraph,
            "an inline text or text control node is bound to no paragraph",
        ),
        (
            E::RootMismatch,
            "the text binding names a different root than the layout request",
        ),
        (
            E::TabReferenceDirection,
            "the direction of a tab reference differs from the paragraph's base direction",
        ),
        (
            E::TerminalBlockEdge,
            "a paragraph that ends at a block end is followed by in-flow content",
        ),
        (
            E::ComputedBoundaryAssociation,
            "a computed line boundary is not associated with its text preparation",
        ),
        (
            E::InlineBoxFragmentsRequireFlow,
            "complete inline box fragments require a run that binds a flow",
        ),
    ];
    assert_messages(&table, 21);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn wrapping_scope_error_prints_its_message_and_has_no_source() {
    use l::WrappingScopeError as E;
    is_error::<E>();
    let table = [
        (
            E::Allocation,
            "wrapping could not allocate its working storage",
        ),
        (
            E::NodeNotReachable,
            "a node that the wrapping data names is not reachable in the flow's tree",
        ),
        (
            E::AmbiguousNode,
            "a node that the wrapping data names is reachable by more than one path",
        ),
        (
            E::EndpointRole,
            "a scope endpoint is not a node that can bound an inline scope",
        ),
        (
            E::DifferentFlow,
            "a scope, edge or node does not belong to the inline flow it is declared for",
        ),
        (E::EndpointOrder, "scope endpoints are not in source order"),
        (
            E::DuplicateScope,
            "the same scope is declared more than once",
        ),
        (
            E::ConflictingScope,
            "two declarations of the same scope conflict",
        ),
        (E::CrossingScopes, "two scopes cross without nesting"),
        (
            E::MissingCapture,
            "a text capture or wrapping context that a scope requires is missing",
        ),
        (
            E::ForeignCapture,
            "a scope names a text capture that belongs to another flow",
        ),
        (
            E::NonInlineScope,
            "a scope that is not an inline scope was supplied where an inline scope is required",
        ),
        (
            E::CaptureAssociation,
            "the association between a scope and its text capture is inconsistent",
        ),
        (
            E::UnsupportedIncidence,
            "a scope has an incidence that cannot be established or that this wrapping does not support",
        ),
        (
            E::InconsistentIncidence,
            "the incidences of the wrapping scopes are inconsistent with the flow",
        ),
        (
            E::ChildOrder,
            "the child order of a box is inconsistent with the tree",
        ),
        (
            E::MissingChildOrder,
            "a box that requires a child order has none",
        ),
        (
            E::SourceProcessingRequired,
            "the source must be processed before this wrapping input can be admitted",
        ),
        (
            E::SourceAssociation,
            "a source association is inconsistent with the processed source",
        ),
        (
            E::PlanMismatch,
            "the wrapping input does not match the processed wrapping plan",
        ),
        (E::MissingBoxMetrics, "a box has no metric contribution"),
        (
            E::InvalidBoxMetrics,
            "a box's metric contribution is invalid",
        ),
        (
            E::ConflictingBoxMetrics,
            "a box's metric contributions conflict",
        ),
        (
            E::MissingGap,
            "a gap that the declarations require is missing",
        ),
        (E::AmbiguousGap, "a gap is declared ambiguously"),
        (E::InvalidGapAnchor, "a gap anchor is invalid"),
        (
            E::ConflictingGap,
            "two declarations of the same gap conflict",
        ),
        (E::CyclicGap, "gap anchors form a cycle"),
        (
            E::GapOutsideSelection,
            "a gap lies outside the selected scope",
        ),
        (
            E::DuplicateFamily,
            "the same formatter was requested more than once",
        ),
        (
            E::IncompleteFamily,
            "the supplied proof does not establish the requested complete generated closure",
        ),
        (
            E::IncompatibleContribution,
            "a requested normal family's contribution endpoint has a nonzero flow advance",
        ),
        (
            E::DifferentFamily,
            "the fragment identities belong to different admitted families",
        ),
        (
            E::DifferentBody,
            "the fragments belong to distinct bodies of the same family",
        ),
    ];
    assert_messages(&table, 34);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn named_grid_error_names_the_rejected_line_name_template_or_placement() {
    use l::GridAxisKind as Axis;
    use l::NamedGridError as E;
    is_error::<E>();
    let table = [
        (
            E::ReservedLineName {
                name: "span".to_string(),
            },
            "the grid line name \"span\" is reserved",
        ),
        (
            E::UnresolvedAutoRepeatNames { axis: Axis::Column },
            "the line names of the column axis cannot be resolved against its auto repeat or \
             subgrid span",
        ),
        (E::EmptyTemplateAreas, "the grid template areas are empty"),
        (
            E::TemplateAreaRowLengthMismatch {
                row: 2,
                expected: 3,
                actual: 1,
            },
            "template area row 2 has 1 cells where 3 are expected",
        ),
        (
            E::NonRectangularTemplateArea {
                name: "main".to_string(),
            },
            "the template area \"main\" is not rectangular",
        ),
        (
            E::ZeroRepeat { axis: Axis::Row },
            "a line-name repeat on the row axis has a count of zero",
        ),
        (
            E::MultipleAutoFillRepeats { axis: Axis::Column },
            "the column axis has more than one auto-fill line-name repeat",
        ),
        (E::ZeroLine, "a grid line number cannot be zero"),
        (E::ZeroSpan, "a grid span cannot be zero"),
        (
            E::AutoWithoutCursor,
            "an auto placement has no placement cursor",
        ),
        (
            E::LineBeforeFirst {
                axis: Axis::Row,
                line: -4,
            },
            "grid line -4 lies before the first line of the row axis",
        ),
    ];
    assert_messages(&table, 11);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn layout_handle_error_prints_its_message_and_has_no_source() {
    use l::LayoutHandleError as E;
    is_error::<E>();
    let table = [
        (
            E::WrongResult,
            "the handle belongs to a different layout result",
        ),
        (
            E::MissingFragment,
            "the result has no fragment for the handle",
        ),
        (
            E::MissingFragmentainer,
            "the result has no fragmentainer for the handle",
        ),
    ];
    assert_messages(&table, 3);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn fragmentation_input_error_prints_its_message_and_has_no_source() {
    use l::FragmentationInputError as E;
    is_error::<E>();
    let table = [
        (
            E::ContextReference,
            "a fragmentation context refers to a context that does not exist or does not match",
        ),
        (
            E::ContextCycle,
            "fragmentation contexts refer to each other in a cycle",
        ),
        (
            E::DuplicateOccurrence,
            "an occurrence appears more than once",
        ),
        (E::InvalidPosition, "a fragmentation position is invalid"),
        (
            E::InvalidGeometry,
            "a fragmentation geometry value is not finite or is negative",
        ),
        (
            E::InvalidLineCounts,
            "the orphan or widow count of a window is zero",
        ),
        (
            E::InvalidContentGap,
            "the content gap of a window is negative or not finite",
        ),
        (E::InvalidBoundary, "a fragmentation boundary is invalid"),
        (
            E::InvalidParentFragmentainer,
            "a parent fragmentainer does not match its child",
        ),
        (E::InvalidWindow, "a fragmentation window is invalid"),
        (E::InvalidSlice, "a fragmentation slice is invalid"),
    ];
    assert_messages(&table, 11);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn root_context_mismatch_prints_its_message_and_has_no_source() {
    use l::RootContextMismatch as E;
    is_error::<E>();
    let table = [
        (
            E::Position,
            "the root context's position differs from the node's input",
        ),
        (
            E::WritingMode,
            "the root context's writing mode differs from the node's input",
        ),
        (
            E::Display,
            "the root context's display differs from the node's input",
        ),
        (
            E::BoxEdges,
            "the root context's box edges differ from the node's input",
        ),
        (
            E::Insets,
            "the root context's insets differ from the node's input",
        ),
        (
            E::AspectRatio,
            "the root context's aspect ratio differs from the node's input",
        ),
        (
            E::Replaced,
            "the root context's replaced-element fact differs from the node's input",
        ),
        (
            E::Float,
            "the root context's float differs from the node's input",
        ),
        (
            E::Clearance,
            "the root context's clear differs from the node's input",
        ),
        (
            E::Alignment,
            "the root context's alignment differs from the node's input",
        ),
    ];
    assert_messages(&table, 10);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn block_fragmentation_failure_names_the_fragmentainer_or_the_inconsistent_input() {
    use l::BlockFragmentationFailure as E;
    is_error::<E>();
    let table = [
        (
            E::MissingFragmentainer { index: 3 },
            "fragmentainer 3 is missing",
        ),
        (
            E::MissingPageSide { index: 2 },
            "the page side of fragmentainer 2 is missing",
        ),
        (
            E::NoProgress { occurrence: 5 },
            "fragmentation of occurrence 5 made no progress",
        ),
        (
            E::InvalidGeometry,
            "a fragmentainer frame or fragment geometry is not finite, non-negative or \
             consistent",
        ),
        (
            E::InvalidSource,
            "the supplied fragmentation source is inconsistent with the tree or the request",
        ),
    ];
    assert_messages(&table, 5);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn layout_batch_apply_error_names_the_refusal_and_sources_the_preparation_error() {
    use l::LayoutBatchApplyError as E;
    is_error::<E<HostFailure>>();
    let table: [(E<HostFailure>, &str); 7] = [
        (
            E::MissingTarget,
            "the request names a publication target that the sink does not have",
        ),
        (
            E::TrackedRootRequiresBoundRequest,
            "the sink tracks this root and the request carries no bound target",
        ),
        (
            E::StaleSource,
            "the request's source basis differs from the sink's current basis",
        ),
        (
            E::StaleConstraints,
            "the request's constraints differ from the sink's current constraints",
        ),
        (
            E::Preparation(HostFailure),
            "the sink failed to prepare the replacement",
        ),
        (
            E::OverlappingRoot,
            "the nodes of the batch overlap those of another published root",
        ),
        (
            E::Allocation,
            "layout could not allocate the storage to apply the batch",
        ),
    ];
    assert_messages(&table, 7);
    for (error, _) in &table {
        match error {
            E::Preparation(_) => assert_eq!(
                chain(error),
                [
                    "the sink failed to prepare the replacement",
                    "the host could not measure the node"
                ]
            ),
            _ => assert!(error.source().is_none()),
        }
    }
}

#[test]
fn leaf_size_input_error_names_the_rejected_size_and_sources_the_scalar_error() {
    is_error::<l::LayoutSizeInputErrorOf<f32>>();
    let context = l::ContainingLayoutContext::new(
        l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr),
        l::ParentFormattingContext::NoParent,
    );
    let known = l::ComputeInput::leaf_layout(
        l::Size::new(Some(-1.0), None),
        l::Size::NONE,
        context,
        sizing_request(),
    )
    .expect_err("a negative known size is rejected");
    assert_eq!(
        chain(&known),
        [
            "the known size on the horizontal axis is invalid",
            "scalar must be non-negative"
        ]
    );
    let parent = l::ComputeInput::leaf_layout(
        l::Size::NONE,
        l::Size::new(None, Some(f32::NAN)),
        context,
        sizing_request(),
    )
    .expect_err("a non-finite percentage parent size is rejected");
    assert_eq!(
        chain(&parent),
        [
            "the percentage-parent size on the vertical axis is invalid",
            "scalar must be finite"
        ]
    );
}

#[test]
fn block_fragmentation_request_rejects_frames_with_a_fragmentation_input_error() {
    let frame = |width: f32, breaks| l::BlockFragmentainerOf::<f32> {
        origin: l::Point::new(0.0, 0.0),
        size: l::Size::new(width, 50.0),
        breaks,
        page_side: None,
    };
    let request = |frames| {
        l::LayoutBoundRequestOf::block_fragmentation(
            l::LayoutSourceBasis::fresh(),
            root_request(),
            l::FragmentationKind::Column,
            l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr),
            frames,
        )
        .expect_err("the frames are rejected")
    };
    assert_eq!(
        request(vec![]).to_string(),
        "a fragmentation window is invalid"
    );
    assert_eq!(
        request(vec![frame(-1.0, l::FragmentationBreakKinds::COLUMN)]).to_string(),
        "a fragmentation geometry value is not finite or is negative"
    );
    assert_eq!(
        request(vec![frame(10.0, l::FragmentationBreakKinds::PAGE)]).to_string(),
        "a fragmentation boundary is invalid"
    );
}

#[test]
fn layout_missing_context_prints_its_message_and_has_no_source() {
    use l::LayoutMissingContext as E;
    is_error::<E>();
    let table = [
        (
            E::NamedInlineBaseline,
            "the named inline baseline that the request needs is missing",
        ),
        (
            E::NamedGridBaseline,
            "an eligible grid participant did not establish the requested named baseline",
        ),
        (
            E::RequiredBasis,
            "a definite basis that the operation requires is missing",
        ),
        (
            E::FloatExclusionProvider,
            "the tree supplied no float exclusion interval for the requested query",
        ),
        (
            E::TextCapture,
            "the text capture that the flow requires is missing",
        ),
        (
            E::TextBlockTail,
            "the selected hyphenation needs a completed-block continuation that is missing",
        ),
        (
            E::TextOutOfFlowGeometry,
            "the geometry of an out-of-flow box that text placement requires is missing",
        ),
        (
            E::ComputedLineBoundaryIncidence,
            "a computed line boundary has no proved side in the native flow",
        ),
        (
            E::AnonymousGeneratingBlock,
            "an anonymous block has no generating block context",
        ),
    ];
    assert_messages(&table, 9);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn layout_internal_invariant_prints_its_message_and_has_no_source() {
    use l::LayoutInternalInvariant as E;
    is_error::<E>();
    let table = [
        (
            E::ResolvedLengthOnFailurePath,
            "a length that resolved was reported on a failure path",
        ),
        (
            E::InvalidRoundedScrollGeometry,
            "rounding produced invalid scroll geometry",
        ),
        (
            E::InvalidRoundedInlineFragmentGeometry,
            "inline fitting produced invalid inline fragment geometry",
        ),
        (
            E::InvalidInlineBalance,
            "a balanced-line basis violated its finite, count or replay invariant",
        ),
        (
            E::InvalidInlineTailSelection,
            "a paragraph-tail program did not reproduce its retained basis",
        ),
        (
            E::InvalidInlinePrettySelection,
            "a pretty-wrapping program did not reproduce its retained phase or fallback basis",
        ),
        (
            E::InvalidAtomicReceiverBasis,
            "a deferred receiving metric no longer matched its retained atomic source",
        ),
        (
            E::MissingLeafMeasurementProvider,
            "the tree supplied no leaf measurement for a node that requires one",
        ),
        (
            E::MissingStagedUnroundedOutput,
            "a staged unrounded output is missing",
        ),
        (
            E::InvalidOutputGeometryPhase,
            "a staged output was consumed in a different geometry or readback phase",
        ),
        (
            E::InvalidGridTrackSizing,
            "grid contribution incidence or resolved track state violated its native contract",
        ),
        (
            E::SubgridTrackInheritance,
            "subgrid track inheritance violated its native contract",
        ),
        (
            E::SubgridBaselineInheritance,
            "subgrid baseline inheritance violated its native contract",
        ),
    ];
    assert_messages(&table, 13);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn layout_text_numeric_error_prints_its_message_and_has_no_source() {
    use l::LayoutTextNumericError as E;
    is_error::<E>();
    let table = [
        (E::NonFinite, "the value is not finite"),
        (
            E::OutOfRange,
            "the value is outside the representable range",
        ),
        (
            E::NonZeroLost,
            "a nonzero contribution was lost to rounding or arithmetic",
        ),
    ];
    assert_messages(&table, 3);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn atomic_inline_participation_role_error_prints_its_message_and_has_no_source() {
    use l::AtomicInlineParticipationRoleError as E;
    is_error::<E>();
    let table = [
        (
            E::MissingForAtomicInline,
            "an atomic inline requires atomic inline participation facts and has none",
        ),
        (
            E::UnexpectedForNonAtomic,
            "a node that is not an atomic inline carries atomic inline participation facts",
        ),
        (
            E::UnexpectedForPreparedAtomic,
            "a prepared atomic inline carries atomic inline participation facts that text already supplies",
        ),
    ];
    assert_messages(&table, 3);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn non_box_node_role_error_prints_its_message_and_has_no_source() {
    use l::NonBoxNodeRoleError as E;
    is_error::<E>();
    let table = [
        (
            E::NonCanonicalNodeInput,
            "a non-box node's node input is not the canonical non-box input",
        ),
        (E::HasChildren, "a non-box node has children"),
        (
            E::HasLeafMeasurement,
            "a non-box node has a leaf measurement",
        ),
    ];
    assert_messages(&table, 3);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn float_exclusion_role_error_prints_its_message_and_has_no_source() {
    use l::FloatExclusionRoleError as E;
    is_error::<E>();
    let table = [
        (E::Hidden, "a hidden node cannot supply a float exclusion"),
        (
            E::NonFloating,
            "a node that does not float cannot supply a float exclusion",
        ),
        (
            E::Absolute,
            "an absolutely positioned node cannot supply a float exclusion",
        ),
    ];
    assert_messages(&table, 3);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

// ---------------------------------------------------------------- end to end

/// A tree of hand-built nodes; node 0 is the root.
struct Tree {
    nodes: Vec<(l::NodeInput, Vec<usize>)>,
}

impl l::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        self.nodes[node].1.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.nodes[node].1.len()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.nodes[node].1[index]
    }
}

impl l::LayoutTree for Tree {
    type MeasureError = Infallible;
    fn node_input(&self, node: usize) -> &l::NodeInput {
        &self.nodes[node].0
    }
    fn layout_input(&self, node: usize) -> l::LayoutInputView<'_> {
        l::LayoutInputView::Box(&self.nodes[node].0)
    }
}

fn px(value: f32) -> l::PreferredSize {
    l::PreferredSize::value(l::LengthPercentageOf::px(value).expect("a finite length"))
}

fn block(size: l::Size<l::PreferredSize>) -> l::NodeInput {
    l::NodeInput {
        display: l::Display::Block,
        size,
        ..l::NodeInput::default()
    }
}

fn leaf_context() -> l::ContainingLayoutContext {
    l::ContainingLayoutContext::new(
        l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr),
        l::ParentFormattingContext::NoParent,
    )
}

fn leaf_input(parent: l::Size<Option<f32>>) -> l::ComputeInput {
    l::ComputeInput::leaf_layout(l::Size::NONE, parent, leaf_context(), sizing_request())
        .expect("a finite leaf input is valid")
}

#[test]
fn compute_leaf_reports_an_unresolvable_numeric_width() {
    let style = l::NodeInput {
        size: l::Size::new(
            l::PreferredSize::value(
                l::LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite"),
            ),
            l::PreferredSize::AUTO,
        ),
        ..l::NodeInput::default()
    };
    let error = l::compute_leaf(
        leaf_input(l::Size::new(Some(f32::MAX), None)),
        &style,
        |_| Ok::<_, HostFailure>(l::Size::new(12.0, 8.0)),
    )
    .expect_err("the width overflows");
    assert_eq!(
        chain(&error),
        [
            "value resolution failed at a standalone request: layout input is invalid",
            "numeric value inf is invalid"
        ]
    );
}

#[test]
fn compute_leaf_reports_a_percentage_without_a_basis() {
    let style = l::NodeInput {
        padding: l::Edges::all(l::Length::value(
            l::LengthPercentageOf::from_percent_fraction(0.5).expect("finite"),
        )),
        ..l::NodeInput::default()
    };
    let error = l::compute_leaf(leaf_input(l::Size::NONE), &style, |_| {
        Ok::<_, HostFailure>(l::Size::new(12.0, 8.0))
    })
    .expect_err("a percentage padding needs a basis");
    assert_eq!(
        chain(&error),
        [
            "value resolution failed at a standalone request: context that layout requires is \
             missing",
            "a definite basis that the operation requires is missing"
        ]
    );
}

#[test]
fn compute_leaf_reports_the_host_measurement_failure() {
    let error = l::compute_leaf(leaf_input(l::Size::NONE), &l::NodeInput::default(), |_| {
        Err::<l::Size<f32>, _>(HostFailure)
    })
    .expect_err("the host cannot measure");
    assert_eq!(
        chain(&error),
        [
            "leaf measurement failed at a standalone request: measurement failed",
            "the host could not measure the node"
        ]
    );
}

#[test]
fn compute_leaf_reports_a_negative_measurement_output() {
    let error = l::compute_leaf(leaf_input(l::Size::NONE), &l::NodeInput::default(), |_| {
        Ok::<_, HostFailure>(l::Size::new(-1.0, 5.0))
    })
    .expect_err("a negative measured width is rejected");
    assert_eq!(
        chain(&error),
        [
            "leaf measurement failed at a standalone request: layout input is invalid",
            "a measurement output is invalid",
            "measurement output rejected on the horizontal axis",
            "scalar must be non-negative"
        ]
    );
    is_error::<l::LayoutInvalidInputOf<f32>>();
    is_error::<l::InvalidMeasurementOutputOf<f32>>();
}

#[test]
fn compute_layout_reports_a_stretch_width_on_an_absolute_box() {
    let mut child = block(l::Size::new(
        l::PreferredSize::STRETCH,
        l::PreferredSize::AUTO,
    ));
    child.position = l::Position::Absolute;
    let tree = Tree {
        nodes: vec![
            (block(l::Size::new(px(200.0), px(200.0))), vec![1]),
            (child, vec![]),
        ],
    };
    let error = l::compute_layout(&tree, 0, root_request())
        .expect_err("later-owned absolute sizing is not supported");
    assert_eq!(
        chain(&error),
        [
            "value resolution failed at node 1: layout does not support a capability its input \
             requires",
            "the stretch sizing behavior of the preferred size is not supported by the \
             positioned algorithm on the horizontal axis"
        ]
    );
    is_error::<l::LayoutUnsupportedCapability>();
}

#[test]
fn compute_layout_reports_an_anonymous_block_that_is_not_a_block() {
    let mut anonymous = l::NodeInput {
        display: l::Display::Flex,
        ..l::NodeInput::default()
    };
    anonymous.block_generation = l::BlockGeneration::Anonymous;
    let tree = Tree {
        nodes: vec![
            (block(l::Size::new(px(100.0), px(100.0))), vec![1]),
            (anonymous, vec![]),
        ],
    };
    let error = l::compute_layout(&tree, 0, root_request())
        .expect_err("an anonymous block must be a block");
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 1: layout input is invalid",
            "an anonymous block has a role an anonymous block cannot have"
        ]
    );
}

#[test]
fn compute_layout_reports_a_cycle_in_the_tree() {
    let tree = Tree {
        nodes: vec![
            (
                block(l::Size::new(l::PreferredSize::AUTO, l::PreferredSize::AUTO)),
                vec![1],
            ),
            (
                block(l::Size::new(l::PreferredSize::AUTO, l::PreferredSize::AUTO)),
                vec![0],
            ),
        ],
    };
    let error = l::compute_layout(&tree, 0, root_request()).expect_err("the tree has a cycle");
    let levels = chain(&error);
    assert_eq!(levels[1], "the tree contains a cycle");
    assert!(
        levels[0].ends_with(": layout input is invalid"),
        "{levels:?}"
    );
}

#[test]
fn compute_layout_invalidated_reports_a_changed_node_outside_the_tree() {
    let tree = Tree {
        nodes: vec![(block(l::Size::new(px(50.0), px(50.0))), vec![])],
    };
    let error = l::compute_layout_invalidated(&tree, 0, root_request(), &[5])
        .expect_err("node 5 is not in the tree");
    assert_eq!(
        chain(&error),
        [
            "cache invalidation failed at node 5: layout input is invalid",
            "an invalidated node is not reachable from the layout root"
        ]
    );
}

#[test]
fn flow_input_rejects_a_root_that_the_request_does_not_name() {
    let tree = Tree {
        nodes: vec![
            (
                block(l::Size::new(l::PreferredSize::AUTO, l::PreferredSize::AUTO)),
                vec![1],
            ),
            (
                block(l::Size::new(l::PreferredSize::AUTO, l::PreferredSize::AUTO)),
                vec![],
            ),
        ],
    };
    let flow = l::LayoutFlowInput::try_new(&tree, 0, vec![], l::WrappingInput::default(), vec![])
        .expect("an empty flow is valid");
    let error = l::compute_layout_with_flow(&flow, 1, root_request())
        .expect_err("the flow is bound to node 0");
    assert_eq!(
        chain(&error),
        [
            "text capture failed at node 1: layout input is invalid",
            "the text binding is invalid",
            "the text binding names a different root than the layout request"
        ]
    );
}

#[test]
fn scroll_anchor_selection_error_names_the_unavailable_anchor() {
    is_error::<l::ScrollAnchorSelectionError<u32>>();
    let region = l::ScrollRect::try_new(l::Point::new(0.0, 0.0), l::Size::new(10.0, 10.0))
        .expect("a finite rectangle");
    let request = l::ScrollAnchorRequestOf::<u32, f32> {
        scroller: 1,
        optimal_viewing_region: region,
        focused: l::ScrollAnchorPriority::Absent,
        find_match: l::ScrollAnchorPriority::Absent,
    };
    let error = l::select_scroll_anchor(request, |_| None)
        .expect_err("a scroller that cannot be read is an error");
    assert_eq!(error.to_string(), "scroll anchor node 1 is unavailable");
    use l::ScrollAnchorSelectionError as E;
    let table = [
        (E::FocusUnavailable, "focused-area anchor is unavailable"),
        (
            E::FindMatchUnavailable,
            "active find-match element is unavailable",
        ),
        (E::MissingNode(4), "scroll anchor node 4 is unavailable"),
        (
            E::InconsistentDomChild {
                parent: 2,
                child: 3,
            },
            "scroll anchor node 3 is not a DOM child of 2",
        ),
        (
            E::InconsistentPositionedChild { owner: 2, child: 5 },
            "scroll anchor node 5 has no positioned edge from 2",
        ),
        (E::Cycle(6), "scroll anchor traversal repeats ancestor 6"),
    ];
    assert_messages(&table, 6);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

// ---------------------------------------------------------------- payload tables

#[test]
fn layout_invalid_input_names_the_rejected_input_and_sources_the_detail() {
    use l::LayoutInvalidInputOf as E;
    let axes = l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr);
    let vertical = l::FlowAxes::new(l::WritingMode::VerticalRl, l::Direction::Ltr);
    let table: Vec<(E<f32>, &str, Option<&str>)> = vec![
        (
            E::BlockFragmentationRequest,
            "the block fragmentation request does not match this root layout",
            None,
        ),
        (
            E::InlineBaseline(l::InlineBaselineError::UnrepresentableCoordinate),
            "an inline baseline fact is invalid",
            Some("a named baseline coordinate cannot be represented in its output frame"),
        ),
        (
            E::GridBaseline(l::GridBaselineError::UnrepresentableCoordinate),
            "a grid baseline fact is invalid",
            Some("a grid baseline coordinate cannot be represented in its output frame"),
        ),
        (
            E::RootContext(l::RootContextMismatch::Display),
            "the root context disagrees with the node's input",
            Some("the root context's display differs from the node's input"),
        ),
        (
            E::InvalidNumeric { value: f32::NAN },
            "numeric value NaN is invalid",
            None,
        ),
        (
            E::InlineFlowMismatch {
                expected: axes,
                actual: vertical,
            },
            "a visible inline control's flow axes differ from its containing block's",
            None,
        ),
        (
            E::InlineBoundaryParentMismatch {
                expected: Some(axes),
                actual: None,
            },
            "an inline box edge's parent flow axes differ from the expected parent flow axes",
            None,
        ),
        (
            E::InlineBoundaryParentAxis {
                parent: vertical,
                formatter: axes,
            },
            "an inline box edge's parent runs on a physical axis incompatible with its formatter",
            None,
        ),
        (
            E::AtomicInlineParticipation {
                reason: l::AtomicInlineParticipationRoleError::MissingForAtomicInline,
            },
            "an atomic inline's participation facts are invalid",
            Some("an atomic inline requires atomic inline participation facts and has none"),
        ),
        (
            E::NonBoxNodeRole {
                reason: l::NonBoxNodeRoleError::HasChildren,
            },
            "a non-box node's role is invalid",
            Some("a non-box node has children"),
        ),
        (
            E::FloatExclusionRole {
                reason: l::FloatExclusionRoleError::Hidden,
            },
            "a float exclusion's node role is invalid",
            Some("a hidden node cannot supply a float exclusion"),
        ),
        (
            E::FloatExclusionProviderOutput {
                error: l::FloatExclusionIntervalErrorOf::NonFiniteBandMinimum { value: f32::NAN },
            },
            "float exclusion provider output is invalid",
            Some("band minimum must be finite"),
        ),
        (
            E::AnonymousBlockRole,
            "an anonymous block has a role an anonymous block cannot have",
            None,
        ),
        (
            E::AnonymousIndentMismatch,
            "an anonymous block's text indent differs from that of its generating block",
            None,
        ),
        (
            E::InvalidationNodeNotReachable,
            "an invalidated node is not reachable from the layout root",
            None,
        ),
        (
            E::InlineAtomicNotDirectChild,
            "the atomic inline is not exactly one direct child of its container",
            None,
        ),
        (
            E::InlineAtomicAmbiguousOccurrence,
            "the direct child is also reachable along another path from its container",
            None,
        ),
        (E::TreeTopologyCycle, "the tree contains a cycle", None),
        (
            E::GridNames(l::NamedGridError::ZeroSpan),
            "the grid container's named lines or template areas are invalid",
            Some("a grid span cannot be zero"),
        ),
        (
            E::TextBinding(l::TextBindingError::EmptyRun),
            "the text binding is invalid",
            Some("a text run binds no nodes where it needs some"),
        ),
        (
            E::WrappingScope(l::WrappingScopeError::CyclicGap),
            "the wrapping scopes are invalid",
            Some("gap anchors form a cycle"),
        ),
    ];
    let rows: Vec<(&E<f32>, &str)> = table.iter().map(|(e, m, _)| (e, *m)).collect();
    let mut seen = std::collections::BTreeSet::new();
    assert_eq!(table.len(), 21, "one row per variant but MeasurementOutput");
    for (error, message, source) in &table {
        assert_eq!(&error.to_string(), message);
        assert_text_is_plain(message, "an invalid input");
        assert!(seen.insert(*message));
        assert_eq!(error.source().map(|s| s.to_string()).as_deref(), *source);
    }
    assert_eq!(rows.len(), 21);
}

#[test]
fn layout_unsupported_capability_names_the_missing_capability() {
    use l::LayoutUnsupportedCapability as E;
    let table = [
        (
            E::BlockFragmentationProfile,
            "the block fragmentation profile is not supported",
        ),
        (
            E::NonBoxNodeOutsideInlineFlow,
            "an inline text or text control node has a parent that does not accept inline text",
        ),
        (
            E::NonNumericLength,
            "a non-numeric length value cannot be used where a number is required",
        ),
        (
            E::InlineAtomicSizePreparation,
            "size preparation requires a definite root-local block and an empty, in-flow, \
             unmeasured inline-block",
        ),
        (
            E::NamedGridLanesBaseline {
                axis: l::GridAxisKind::Row,
            },
            "named grid baseline placement on the row axis is not supplied for grid lanes",
        ),
        (
            E::TextWritingMode(l::WritingMode::SidewaysLr),
            "the sideways-lr writing mode needs a physical adapter that text-backed runs do not \
             support",
        ),
    ];
    assert_messages(&table, 6);
    for (error, _) in &table {
        assert!(error.source().is_none());
    }
}

#[test]
fn existing_complete_errors_keep_their_messages_and_report_no_unwanted_source() {
    let baseline = [
        (
            l::BaselineOffsetResolutionError::MissingOwnLineHeight,
            "baseline offset requires the child's own used line height",
        ),
        (
            l::BaselineOffsetResolutionError::NonFinite,
            "baseline offset arithmetic must remain finite",
        ),
        (
            l::BaselineOffsetResolutionError::NonZeroLost,
            "baseline offset arithmetic lost a nonzero contribution",
        ),
    ];
    assert_messages(&baseline, 3);
    let track = [
        (
            l::TrackRepetitionError::ZeroCount,
            "track repeat count must be greater than zero",
        ),
        (
            l::TrackRepetitionError::EmptyComponents,
            "track repeat components must not be empty",
        ),
    ];
    assert_messages(&track, 2);
    let scalar = [
        (
            l::NonNegativeFiniteScalarErrorOf::<f32>::NonFinite { value: f32::NAN },
            "scalar must be finite",
        ),
        (
            l::NonNegativeFiniteScalarErrorOf::<f32>::Negative { value: -1.0 },
            "scalar must be non-negative",
        ),
    ];
    assert_messages(&scalar, 2);
    let finite = [(
        l::FiniteScalarErrorOf::<f32>::NonFinite { value: f32::NAN },
        "scalar must be finite",
    )];
    assert_messages(&finite, 1);
    is_error::<l::BaselineOffsetResolutionError>();
    is_error::<l::CalcSizeConstructionError>();
    is_error::<l::ComputedOverflowError>();
    is_error::<l::FiniteScalarErrorOf<f32>>();
    is_error::<l::FloatExclusionIntervalErrorOf<f32>>();
    is_error::<l::NonNegativeFiniteScalarErrorOf<f32>>();
    is_error::<l::OrthogonalFallbackErrorOf<f32>>();
    is_error::<l::ScrollRectErrorOf<f32>>();
    is_error::<l::SizingCalculationError>();
    is_error::<l::TrackRepetitionError>();
    is_error::<l::ScrollMarginErrorOf<f32>>();
}

struct FailingSink {
    targets: l::LayoutPublicationTargets<usize>,
}

impl l::LayoutBatchSink<usize, f32> for FailingSink {
    type Error = HostFailure;
    type Prepared = ();
    fn publication_targets(&self) -> &l::LayoutPublicationTargets<usize> {
        &self.targets
    }
    fn publication_targets_mut(&mut self) -> &mut l::LayoutPublicationTargets<usize> {
        &mut self.targets
    }
    fn prepare_layout_batch<'call>(
        &self,
        _preparation: l::LayoutBatchPreparation<'call, usize, f32>,
    ) -> Result<Self::Prepared, Self::Error> {
        Err(HostFailure)
    }
    fn commit_layout_batch<'call>(
        &mut self,
        _commit: l::LayoutBatchCommit<'call, usize, f32, Self::Prepared>,
    ) {
    }
}

#[test]
fn applying_a_batch_to_a_sink_that_cannot_prepare_it_reports_the_sink_error() {
    let tree = Tree {
        nodes: vec![(block(l::Size::new(px(50.0), px(50.0))), vec![])],
    };
    let batch = l::compute_layout(&tree, 0, root_request()).expect("a single box lays out");
    let mut sink = FailingSink {
        targets: l::LayoutPublicationTargets::new(),
    };
    let error = batch
        .apply_to(&mut sink)
        .expect_err("the sink cannot prepare the replacement");
    assert_eq!(
        chain(&error),
        [
            "the sink failed to prepare the replacement",
            "the host could not measure the node"
        ]
    );
}

#[test]
fn compute_layout_reports_scroll_geometry_that_overflows_the_scalar() {
    // A root block whose border box ends beyond the largest finite scalar: a
    // left border and padding of f32::MAX on a box of that width.
    let overflowing = l::Edges {
        left: l::Length::value(l::LengthPercentageOf::px(f32::MAX).expect("a finite length")),
        ..l::Edges::all(l::Length::ZERO)
    };
    let root = l::NodeInput {
        padding: overflowing,
        border: overflowing,
        ..block(l::Size::new(px(f32::MAX), px(1.0)))
    };
    let tree = Tree {
        nodes: vec![(root, vec![])],
    };
    let sizing = native_request::sizing(
        l::Size::new(Some(f32::MAX), Some(1.0)),
        l::Size::new(l::SizingOperation::Normal, l::SizingOperation::Normal),
    );
    let request = l::LayoutRootRequest::standalone(
        sizing,
        l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr),
        sizing.orthogonal_fallback().initial_containing_block(),
    );
    let error = l::compute_layout(&tree, 0, request)
        .expect_err("the overflowing scroll geometry is rejected");
    assert_eq!(
        chain(&error),
        [
            "root layout failed at node 0: scroll geometry rejected",
            "canonical scroll geometry is invalid",
            "the scroll box clip or gutter is invalid",
            "a scroll box rectangle is invalid",
            "scroll rectangle horizontal size must be finite"
        ]
    );
}

#[test]
fn a_handle_from_another_result_is_rejected_with_a_handle_error() {
    is_error::<l::LayoutHandleError>();
    let tree = Tree {
        nodes: vec![(block(l::Size::new(px(50.0), px(50.0))), vec![])],
    };
    let first = l::compute_layout(&tree, 0, root_request()).expect("a single box lays out");
    let second = l::compute_layout(&tree, 0, root_request()).expect("a single box lays out");
    let id = first
        .node_layout(0)
        .expect("node 0 has a layout")
        .single_box()
        .expect("a block root produces one box")
        .id()
        .clone();
    assert!(first.fragment(&id).is_ok());
    let error = second
        .fragment(&id)
        .expect_err("the handle belongs to the first result");
    assert_eq!(
        chain(&error),
        ["the handle belongs to a different layout result"]
    );
    let error = match second.child_fragments(&id) {
        Ok(_) => panic!("the handle belongs to the first result"),
        Err(error) => error,
    };
    assert_eq!(error, l::LayoutHandleError::WrongResult);
}
