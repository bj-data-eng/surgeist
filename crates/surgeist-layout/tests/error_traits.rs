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
