use super::*;

fn components<S: LayoutScalar>() {
    for (margin, border, padding, bp, flow) in [
        (3.0, 2.0, 1.0, 3.0, 6.0),
        (-4.0, 1.0, 2.0, 3.0, -1.0),
        (-5.0, 2.0, 3.0, 5.0, 0.0),
        (-5.0, 0.0, 0.0, 0.0, -5.0),
        (0.0, 0.0, 0.0, 0.0, 0.0),
    ] {
        let edge = InlineBoundaryEdgeOf::<S>::try_new(
            S::from_f64(margin),
            S::from_f64(border),
            S::from_f64(padding),
        )
        .unwrap();
        assert_eq!(edge.margin(), S::from_f64(margin));
        assert_eq!(edge.border(), S::from_f64(border));
        assert_eq!(edge.padding(), S::from_f64(padding));
        assert_eq!(edge.border_padding_advance(), S::from_f64(bp));
        assert_eq!(edge.flow_advance(), S::from_f64(flow));
    }
    let zero =
        InlineBoundaryEdgeOf::<S>::try_new(S::from_f64(-0.0), S::from_f64(-0.0), S::from_f64(-0.0))
            .unwrap();
    assert_eq!(zero, InlineBoundaryEdgeOf::ZERO);
    for value in [
        zero.margin(),
        zero.border(),
        zero.padding(),
        zero.border_padding_advance(),
        zero.flow_advance(),
    ] {
        assert!(!value.to_f64().is_sign_negative());
    }
}

#[test]
fn signed_components_preserve_independent_flow_and_border_padding_in_both_scalar_lanes() {
    components::<f32>();
    components::<f64>();
}

#[test]
fn border_padding_aggregate_keeps_ordinary_native_lane_rounding() {
    let edge32 = InlineBoundaryEdgeOf::<f32>::try_new(0.0, 16_777_216.0, 1.0).unwrap();
    assert_eq!((edge32.border(), edge32.padding()), (16_777_216.0, 1.0));
    assert_eq!(edge32.border_padding_advance(), 16_777_216.0);
    assert_eq!(edge32.flow_advance(), 16_777_216.0);
    let edge64 = InlineBoundaryEdgeOf::<f64>::try_new(0.0, 16_777_216.0, 1.0).unwrap();
    assert_eq!(edge64.padding(), 1.0);
    assert_eq!(edge64.border_padding_advance(), 16_777_217.0);
    assert_eq!(edge64.flow_advance(), 16_777_217.0);
    let large64 = InlineBoundaryEdgeOf::<f64>::try_new(0.0, 9_007_199_254_740_992.0, 1.0).unwrap();
    assert_eq!(large64.padding(), 1.0);
    assert_eq!(large64.border_padding_advance(), 9_007_199_254_740_992.0);
    assert_eq!(large64.flow_advance(), 9_007_199_254_740_992.0);
}

#[test]
fn text_component_identity_preserves_source_lane_and_canonical_zero() {
    use surgeist_text::InlineBoxEdgeComponents;
    let zero32 = InlineBoxEdgeComponents::from_f32(-0.0, -0.0, -0.0).unwrap();
    let plus32 = InlineBoxEdgeComponents::from_f32(0.0, 0.0, 0.0).unwrap();
    let zero64 = InlineBoxEdgeComponents::from_f64(-0.0, -0.0, -0.0).unwrap();
    let plus64 = InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap();
    assert_eq!(zero32, plus32);
    assert_eq!(zero64, plus64);
    assert_ne!(zero32, zero64);
    let rounded32 = InlineBoxEdgeComponents::from_f32(0.0, 16_777_216.0, 1.0).unwrap();
    assert_eq!(rounded32.padding(), 1.0);
    assert_eq!(rounded32.border_padding_advance(), 16_777_216.0);
    assert_eq!(rounded32.flow_advance(), 16_777_216.0);
    let wider64 = InlineBoxEdgeComponents::from_f64(0.0, 16_777_216.0, 1.0).unwrap();
    assert_eq!(wider64.border_padding_advance(), 16_777_217.0);
    assert_eq!(wider64.flow_advance(), 16_777_217.0);
    let rounded64 = InlineBoxEdgeComponents::from_f64(0.0, 9_007_199_254_740_992.0, 1.0).unwrap();
    assert_eq!(rounded64.padding(), 1.0);
    assert_eq!(rounded64.border_padding_advance(), 9_007_199_254_740_992.0);
    assert_eq!(rounded64.flow_advance(), 9_007_199_254_740_992.0);
    assert!(InlineBoxEdgeComponents::from_f32(-5.0, 2.0, 3.0).is_ok());
    assert!(InlineBoxEdgeComponents::from_f64(-5.0, 2.0, 3.0).is_ok());
}

fn same_scalar<S: LayoutScalar>(actual: S, expected: S) {
    if expected.to_f64().is_nan() {
        assert!(actual.to_f64().is_nan());
    } else {
        assert_eq!(actual, expected);
    }
}

fn constructor_errors<S: LayoutScalar>(max: S, precision_boundary: S) {
    use InlineBoundaryEdgeErrorOf as E;
    assert!(max.is_finite());
    for value in [S::NAN, S::INFINITY, -S::INFINITY] {
        match InlineBoundaryEdgeOf::<S>::try_new(value, S::ZERO, S::ZERO).unwrap_err() {
            E::InvalidMargin(crate::FiniteScalarErrorOf::NonFinite { value: actual }) => {
                same_scalar(actual, value)
            }
            other => panic!("{other:?}"),
        }
        match InlineBoundaryEdgeOf::<S>::try_new(S::ZERO, value, S::ZERO).unwrap_err() {
            E::InvalidBorder(crate::NonNegativeFiniteScalarErrorOf::NonFinite {
                value: actual,
            }) => same_scalar(actual, value),
            other => panic!("{other:?}"),
        }
        match InlineBoundaryEdgeOf::<S>::try_new(S::ZERO, S::ZERO, value).unwrap_err() {
            E::InvalidPadding(crate::NonNegativeFiniteScalarErrorOf::NonFinite {
                value: actual,
            }) => same_scalar(actual, value),
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(
        InlineBoundaryEdgeOf::<S>::try_new(S::ZERO, -S::ONE, S::ZERO).unwrap_err(),
        E::InvalidBorder(crate::NonNegativeFiniteScalarErrorOf::Negative { value: -S::ONE })
    );
    assert_eq!(
        InlineBoundaryEdgeOf::<S>::try_new(S::ZERO, S::ZERO, -S::ONE).unwrap_err(),
        E::InvalidPadding(crate::NonNegativeFiniteScalarErrorOf::Negative { value: -S::ONE })
    );
    assert!(
        matches!(InlineBoundaryEdgeOf::<S>::try_new(S::ZERO, max, max),
        Err(E::InvalidBorderPaddingAdvance(crate::FiniteScalarErrorOf::NonFinite { value })) if !value.is_finite())
    );
    assert!(
        matches!(InlineBoundaryEdgeOf::<S>::try_new(max, max, S::ZERO),
        Err(E::InvalidFlowAdvance(crate::FiniteScalarErrorOf::NonFinite { value })) if !value.is_finite())
    );
    // Losing a NEW margin/strip term in signed flow is distinct from ordinary
    // rounding of the pre-existing BP sum, which remains accepted above.
    assert_eq!(
        InlineBoundaryEdgeOf::<S>::try_new(S::ONE, precision_boundary, S::ZERO).unwrap_err(),
        E::LostMargin { value: S::ONE }
    );
    assert_eq!(
        InlineBoundaryEdgeOf::<S>::try_new(precision_boundary, S::ONE, S::ZERO).unwrap_err(),
        E::LostBorderPaddingAdvance { value: S::ONE }
    );
    let valid =
        InlineBoundaryEdgeOf::<S>::try_new(S::from_f64(-5.0), S::from_f64(2.0), S::from_f64(3.0))
            .unwrap();
    assert_eq!(valid.flow_advance(), S::ZERO);
}

#[test]
fn component_errors_and_new_signed_flow_losses_keep_their_typed_scalar_owner() {
    constructor_errors::<f32>(f32::MAX, 16_777_216.0);
    constructor_errors::<f64>(f64::MAX, 9_007_199_254_740_992.0);
}

fn contribution_role<S: LayoutScalar>() {
    let metrics = InlineMetricsOf::try_new(
        crate::InlineBaselineContext::Physical,
        S::from_f64(8.0),
        S::from_f64(10.0),
    )
    .unwrap();
    for (margin, border, padding) in [(5.0, 0.0, 0.0), (-5.0, 0.0, 0.0), (-5.0, 2.0, 3.0)] {
        let edge = InlineBoundaryEdgeOf::<S>::try_new(
            S::from_f64(margin),
            S::from_f64(border),
            S::from_f64(padding),
        )
        .unwrap();
        assert_eq!(
            InlineBoundaryInputOf::contribution(InlineBoundaryKind::Start, metrics, edge)
                .unwrap_err(),
            InlineBoundaryEdgeErrorOf::MarginRequiresBoxEdge {
                margin: S::from_f64(margin)
            }
        );
    }
    let edge =
        InlineBoundaryEdgeOf::<S>::try_new(S::ZERO, S::from_f64(2.0), S::from_f64(3.0)).unwrap();
    let accepted =
        InlineBoundaryInputOf::contribution(InlineBoundaryKind::Start, metrics, edge).unwrap();
    assert_eq!(accepted.edge(), edge);
    assert_eq!(accepted.metrics(), metrics);
    assert_eq!(accepted.parent_edge_axes(), None);
    let zero = InlineBoundaryInputOf::contribution(
        InlineBoundaryKind::End,
        metrics,
        InlineBoundaryEdgeOf::try_new(S::from_f64(-0.0), S::ZERO, S::ZERO).unwrap(),
    )
    .unwrap();
    assert_eq!(zero.edge(), InlineBoundaryEdgeOf::ZERO);
}

#[test]
fn a_nonzero_margin_cannot_become_a_scalar_contribution_even_when_its_total_cancels() {
    contribution_role::<f32>();
    contribution_role::<f64>();
}

fn text_error(
    result: std::result::Result<surgeist_text::InlineBoxEdgeComponents, surgeist_text::Error>,
    field: &'static str,
    category: surgeist_text::TextNumericError,
) {
    use surgeist_text::{ErrorCode, ErrorDetail, TextNumericPhase};
    let error = result.unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
    let Some(ErrorDetail::InvalidTextNumeric {
        field: actual_field,
        phase,
        category: actual,
        ..
    }) = error.detail()
    else {
        panic!("{error:?}")
    };
    assert_eq!(*actual_field, field);
    assert_eq!(*phase, TextNumericPhase::Selection);
    assert_eq!(*actual, category);
}

#[test]
fn text_component_construction_reports_the_actual_component_and_native_lane_failure() {
    use surgeist_text::{InlineBoxEdgeComponents as C, TextNumericError as N};
    for result in [
        C::from_f32(f32::NAN, 0.0, 0.0),
        C::from_f64(f64::INFINITY, 0.0, 0.0),
    ] {
        text_error(result, "inline edge margin", N::NonFinite);
    }
    text_error(
        C::from_f32(0.0, -1.0, 0.0),
        "inline edge border",
        N::OutOfRange,
    );
    text_error(
        C::from_f64(0.0, 0.0, -1.0),
        "inline edge padding",
        N::OutOfRange,
    );
    text_error(
        C::from_f32(0.0, f32::MAX, f32::MAX),
        "inline edge border padding advance",
        N::NonFinite,
    );
    text_error(
        C::from_f64(f64::MAX, f64::MAX, 0.0),
        "inline edge flow advance",
        N::NonFinite,
    );
    text_error(
        C::from_f32(1.0, 16_777_216.0, 0.0),
        "inline edge margin flow",
        N::NonZeroLost,
    );
    text_error(
        C::from_f32(16_777_216.0, 1.0, 0.0),
        "inline edge strip flow",
        N::NonZeroLost,
    );
    text_error(
        C::from_f64(1.0, 9_007_199_254_740_992.0, 0.0),
        "inline edge margin flow",
        N::NonZeroLost,
    );
    text_error(
        C::from_f64(9_007_199_254_740_992.0, 1.0, 0.0),
        "inline edge strip flow",
        N::NonZeroLost,
    );
}

#[test]
fn finite_subnormal_margin_is_not_an_implicit_zero_or_a_constructor_magnitude_limit() {
    let tiny32 = -f32::from_bits(1);
    let tiny64 = -f64::from_bits(1);
    let a = InlineBoundaryEdgeOf::<f32>::try_new(tiny32, 0.0, 0.0).unwrap();
    let b = InlineBoundaryEdgeOf::<f64>::try_new(tiny64, 0.0, 0.0).unwrap();
    assert_eq!(a.flow_advance(), tiny32);
    assert_eq!(b.flow_advance(), tiny64);
    let a = surgeist_text::InlineBoxEdgeComponents::from_f32(tiny32, 0.0, 0.0).unwrap();
    let b = surgeist_text::InlineBoxEdgeComponents::from_f64(tiny64, 0.0, 0.0).unwrap();
    assert_eq!(a.margin(), f64::from(tiny32));
    assert_eq!(b.margin(), tiny64);
}
