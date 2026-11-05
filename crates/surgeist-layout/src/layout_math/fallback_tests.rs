//! New-API functional qualification of original-source fallback authority.
use super::*;
use crate::node_projection::CommonBoxProjection;
use crate::scroll::{MeasuredLeafContentBoxInsetSourceOf, SettledAutoScrollbarState};
use crate::sizing::resolve::{
    FixedFallbackSource, SizingResolutionError, fixed_fallback_maximum, fixed_fallback_minimum,
    fixed_fallback_preferred, resolve_minimum_optional,
};
use crate::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn nn<S: LayoutScalar>(value: f64) -> NonNegativeFiniteOf<S> {
    NonNegativeFiniteOf::new(n(value)).unwrap()
}
fn axis<S: LayoutScalar>(operation: SizingOperation) -> AxisSizingRequestOf<S> {
    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operation)
}
fn inherited<S: LayoutScalar>() -> OrthogonalFallbackOf<S> {
    OrthogonalFallbackOf::new(
        Size::new(Some(nn(800.0)), Some(nn(900.0))),
        NearestScrollportOf::Absent,
        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
    )
}
fn query<S: LayoutScalar>(
    mode: SizingMode,
    operations: Size<SizingOperation>,
    known: Size<Option<S>>,
    parent: Size<Option<S>>,
) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        mode,
        RequestedAxis::Both,
        known,
        parent,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        SizingRequestOf::new(operations.map(axis), inherited()),
    )
}
fn hidden() -> ComputedOverflow {
    ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap()
}
fn source<S: LayoutScalar>(style: &NodeInputOf<S>) -> MeasuredLeafContentBoxInsetSourceOf<S> {
    MeasuredLeafContentBoxInsetSourceOf {
        flow_axes: FlowAxes::new(style.writing_mode, style.direction),
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars: SettledAutoScrollbarState::INITIAL,
        padding: style.padding.map(|value| {
            crate::sizing::resolve::resolve_length_or_zero_fallible(value, None).unwrap()
        }),
        border: style.border.map(|value| {
            crate::sizing::resolve::resolve_length_or_zero_fallible(value, None).unwrap()
        }),
    }
}
fn adjustment<S: LayoutScalar>(style: &NodeInputOf<S>) -> Size<S> {
    match style.box_sizing {
        BoxSizing::ContentBox => {
            let edges = source(style);
            (edges.padding + edges.border).sum_axes()
        }
        BoxSizing::BorderBox => Size::ZERO,
    }
}
fn qualify<S: LayoutScalar>(
    input: ComputeInputOf<S>,
    style: &NodeInputOf<S>,
    algorithm: SizingAlgorithm,
) -> Result<FixedFallbackSources<S>, SizingResolutionError<S>> {
    qualify_fixed_fallback_sources(
        input,
        algorithm,
        &CommonBoxProjection::from_node(style),
        input.known(),
        adjustment(style),
        crate::scroll::UsedOverflow::from_computed(style.overflow, style.item_is_replaced)
            .is_scroll_container(),
    )
}
fn assert_unsupported<S: LayoutScalar>(
    result: Result<FixedFallbackSources<S>, SizingResolutionError<S>>,
    property: SizingProperty,
    behavior: SizingBehavior,
    physical_axis: PhysicalAxis,
) {
    let SizingResolutionError::Unsupported(error) = result.unwrap_err() else {
        panic!("expected actual contextual sizing refusal");
    };
    assert_eq!(
        (
            error.property(),
            error.behavior(),
            error.algorithm(),
            error.axis()
        ),
        (property, behavior, SizingAlgorithm::Block, physical_axis)
    );
}

fn original_roles<S: LayoutScalar>() {
    let percent = LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap();
    for algorithm in [
        SizingAlgorithm::Block,
        SizingAlgorithm::Flex,
        SizingAlgorithm::Grid,
        SizingAlgorithm::GridLanes,
    ] {
        for physical_axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
            assert_eq!(
                fixed_fallback_preferred::<S>(
                    &PreferredSizeOf::AUTO,
                    algorithm,
                    physical_axis,
                    None
                ),
                Ok(FixedFallbackSource::NoFixedBound)
            );
            assert_eq!(
                fixed_fallback_minimum::<S>(&MinSizeOf::AUTO, algorithm, physical_axis, None),
                Ok(FixedFallbackSource::NoFixedBound)
            );
            assert_eq!(
                fixed_fallback_maximum::<S>(&MaxSizeOf::NONE, algorithm, physical_axis, None),
                Ok(FixedFallbackSource::NoFixedBound)
            );
            for value in [PreferredSizeOf::MIN_CONTENT, PreferredSizeOf::MAX_CONTENT] {
                assert_eq!(
                    fixed_fallback_preferred::<S>(&value, algorithm, physical_axis, None),
                    Ok(FixedFallbackSource::NoFixedBound)
                );
            }
            for value in [MinSizeOf::MIN_CONTENT, MinSizeOf::MAX_CONTENT] {
                assert_eq!(
                    fixed_fallback_minimum::<S>(&value, algorithm, physical_axis, None),
                    Ok(FixedFallbackSource::NoFixedBound)
                );
            }
            for value in [MaxSizeOf::MIN_CONTENT, MaxSizeOf::MAX_CONTENT] {
                assert_eq!(
                    fixed_fallback_maximum::<S>(&value, algorithm, physical_axis, None),
                    Ok(FixedFallbackSource::NoFixedBound)
                );
            }
            // The admitted None declares an indefinite basis; no numeric zero
            // or unavailable-browser-authority substitute is manufactured.
            for (basis, expected) in [
                (None, FixedFallbackSource::NoFixedBound),
                (Some(n(200.0)), FixedFallbackSource::Fixed(n(100.0))),
            ] {
                assert_eq!(
                    fixed_fallback_preferred::<S>(
                        &PreferredSizeOf::value(percent),
                        algorithm,
                        physical_axis,
                        basis
                    ),
                    Ok(expected)
                );
                assert_eq!(
                    fixed_fallback_minimum::<S>(
                        &MinSizeOf::value(percent),
                        algorithm,
                        physical_axis,
                        basis
                    ),
                    Ok(expected)
                );
                assert_eq!(
                    fixed_fallback_maximum::<S>(
                        &MaxSizeOf::value(percent),
                        algorithm,
                        physical_axis,
                        basis
                    ),
                    Ok(expected)
                );
            }
            assert_eq!(
                resolve_minimum_optional::<S>(
                    &MinSizeOf::AUTO,
                    algorithm,
                    physical_axis,
                    None,
                    true
                ),
                Ok(None)
            );
            assert_eq!(
                resolve_minimum_optional::<S>(
                    &MinSizeOf::ZERO,
                    algorithm,
                    physical_axis,
                    None,
                    true
                ),
                Ok(Some(S::ZERO))
            );
            assert_eq!(
                fixed_fallback_minimum::<S>(&MinSizeOf::ZERO, algorithm, physical_axis, None),
                Ok(FixedFallbackSource::Fixed(S::ZERO))
            );
        }
    }
    let style: NodeInputOf<S> = NodeInputOf {
        min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::px(n(40.0)), MaxSizeOf::px(n(40.0))),
        box_sizing: BoxSizing::BorderBox,
        padding: Edges::new(
            LengthOf::px(n(2.0)),
            LengthOf::px(n(1.0)),
            LengthOf::px(n(2.0)),
            LengthOf::px(n(1.0)),
        ),
        border: Edges::new(
            LengthOf::px(n(1.0)),
            LengthOf::px(n(2.0)),
            LengthOf::px(n(1.0)),
            LengthOf::px(n(2.0)),
        ),
        ..NodeInputOf::default()
    };
    let input = query::<S>(
        SizingMode::ContentSize,
        Size::new(SizingOperation::MinContent, SizingOperation::Normal),
        Size::NONE,
        Size::NONE,
    );
    let facts = qualify(input, &style, SizingAlgorithm::Grid).unwrap();
    assert_eq!(
        facts.minimum_outer,
        Size::splat(FixedFallbackSource::NoFixedBound)
    );
    // A body/track's resolved or measured automatic minimum is not an input to
    // this owner. Only the actual original Auto is qualified here.
    let insets = source(&style);
    assert_eq!(
        (insets.padding + insets.border).sum_axes(),
        Size::new(n(6.0), n(6.0))
    );
    assert_eq!(
        containing_orthogonal_fallback_limits(
            facts.minimum_outer,
            facts.maximum_outer,
            (insets.padding + insets.border).sum_axes()
        )
        .unwrap(),
        Size::splat(OrthogonalFallbackLimitOf::Finite(nn(34.0)))
    );
}

fn original_calc_dependency<S: LayoutScalar>() {
    let calculation = CalcSizeCalculationOf::from_coefficients(n(10.0), n(0.5), S::ZERO).unwrap();
    let preferred =
        PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Any, calculation.clone()).unwrap();
    let minimum = MinSizeOf::calc_size(MinSizeCalcBasis::Any, calculation.clone()).unwrap();
    let maximum = MaxSizeOf::calc_size(MaxSizeCalcBasis::Any, calculation).unwrap();
    for (basis, expected) in [
        (None, FixedFallbackSource::NoFixedBound),
        (Some(n(200.0)), FixedFallbackSource::Fixed(n(110.0))),
    ] {
        assert_eq!(
            fixed_fallback_preferred::<S>(
                &preferred,
                SizingAlgorithm::Block,
                PhysicalAxis::Horizontal,
                basis
            ),
            Ok(expected)
        );
        assert_eq!(
            fixed_fallback_minimum::<S>(
                &minimum,
                SizingAlgorithm::Block,
                PhysicalAxis::Horizontal,
                basis
            ),
            Ok(expected)
        );
        assert_eq!(
            fixed_fallback_maximum::<S>(
                &maximum,
                SizingAlgorithm::Block,
                PhysicalAxis::Horizontal,
                basis
            ),
            Ok(expected)
        );
    }
    // FullPercentage remains a dependency even with a constant expression;
    // classification must inspect the original basis before dispatch erases it.
    let constant = CalcSizeCalculationOf::from_coefficients(n(7.0), S::ZERO, S::ZERO).unwrap();
    let preferred =
        PreferredSizeOf::calc_size(PreferredSizeCalcBasis::FullPercentage, constant.clone())
            .unwrap();
    let minimum = MinSizeOf::calc_size(MinSizeCalcBasis::FullPercentage, constant.clone()).unwrap();
    let maximum = MaxSizeOf::calc_size(MaxSizeCalcBasis::FullPercentage, constant).unwrap();
    for (basis, expected) in [
        (None, FixedFallbackSource::NoFixedBound),
        (Some(n(200.0)), FixedFallbackSource::Fixed(n(7.0))),
    ] {
        assert_eq!(
            fixed_fallback_preferred::<S>(
                &preferred,
                SizingAlgorithm::Block,
                PhysicalAxis::Vertical,
                basis
            ),
            Ok(expected)
        );
        assert_eq!(
            fixed_fallback_minimum::<S>(
                &minimum,
                SizingAlgorithm::Block,
                PhysicalAxis::Vertical,
                basis
            ),
            Ok(expected)
        );
        assert_eq!(
            fixed_fallback_maximum::<S>(
                &maximum,
                SizingAlgorithm::Block,
                PhysicalAxis::Vertical,
                basis
            ),
            Ok(expected)
        );
    }
    let dependent = CalcSizeCalculationOf::from_coefficients(S::ZERO, S::ZERO, n(1.0)).unwrap();
    let keyword = PreferredSizeOf::calc_size(PreferredSizeCalcBasis::Auto, dependent).unwrap();
    let SizingResolutionError::Unsupported(error) = fixed_fallback_preferred::<S>(
        &keyword,
        SizingAlgorithm::Block,
        PhysicalAxis::Horizontal,
        None,
    )
    .unwrap_err() else {
        panic!("a demanded contextual basis must retain its real refusal");
    };
    assert_eq!(
        (
            error.property(),
            error.behavior(),
            error.algorithm(),
            error.axis()
        ),
        (
            SizingProperty::Preferred,
            SizingBehavior::CalcSize(CalcSizeBehaviorBasis::Auto),
            SizingAlgorithm::Block,
            PhysicalAxis::Horizontal
        )
    );
}

fn raw_purpose_and_known<S: LayoutScalar>() {
    let style: NodeInputOf<S> = NodeInputOf {
        size: Size::new(PreferredSizeOf::px(n(80.0)), PreferredSizeOf::px(n(60.0))),
        min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
        overflow: hidden(),
        box_sizing: BoxSizing::BorderBox,
        ..NodeInputOf::default()
    };
    for (operations, expected) in [
        (
            Size::new(SizingOperation::MinContent, SizingOperation::Normal),
            Size::new(false, true),
        ),
        (
            Size::new(SizingOperation::Normal, SizingOperation::MaxContent),
            Size::new(true, false),
        ),
        (
            Size::new(SizingOperation::Normal, SizingOperation::Normal),
            Size::new(false, false),
        ),
        (
            Size::new(SizingOperation::MinContent, SizingOperation::MaxContent),
            Size::new(false, false),
        ),
    ] {
        let input = query::<S>(
            SizingMode::ContentSize,
            operations,
            Size::NONE,
            Size::new(Some(n(200.0)), Some(n(300.0))),
        );
        assert_eq!(input.requested_axis(), RequestedAxis::Both);
        assert_eq!(
            Size::new(
                fallback_preferred_is_available(input, PhysicalAxis::Horizontal),
                fallback_preferred_is_available(input, PhysicalAxis::Vertical)
            ),
            expected
        );
        let facts = qualify(input, &style, SizingAlgorithm::Block).unwrap();
        assert_eq!(
            facts.definite_outer,
            Size::new(
                if expected.width {
                    FixedFallbackSource::Fixed(n(80.0))
                } else {
                    FixedFallbackSource::Unqualified
                },
                if expected.height {
                    FixedFallbackSource::Fixed(n(60.0))
                } else {
                    FixedFallbackSource::Unqualified
                }
            )
        );
        assert_eq!(input.known(), Size::NONE);
        assert_eq!(input.parent(), Size::new(Some(n(200.0)), Some(n(300.0))));
        assert_eq!(
            input
                .sizing_request()
                .axes()
                .map(AxisSizingRequestOf::operation),
            operations
        );
    }
    let known = query::<S>(
        SizingMode::ContentSize,
        Size::new(SizingOperation::MinContent, SizingOperation::Normal),
        Size::new(Some(n(13.0)), None),
        Size::NONE,
    );
    assert!(!fallback_preferred_is_available(
        known,
        PhysicalAxis::Horizontal
    ));
    let facts = qualify(known, &style, SizingAlgorithm::Block).unwrap();
    assert_eq!(
        facts.definite_outer.width,
        FixedFallbackSource::Fixed(n(13.0))
    );
    assert_eq!(
        facts.definite_outer.height,
        FixedFallbackSource::Fixed(n(60.0))
    );
    assert_eq!(known.known(), Size::new(Some(n(13.0)), None));
    assert_eq!(known.parent(), Size::NONE);
    assert_eq!(
        known.sizing_request().axes().width.operation(),
        SizingOperation::MinContent
    );
}

fn needed_only_refusals<S: LayoutScalar>(largest: S) {
    let input = query::<S>(
        SizingMode::ContentSize,
        Size::new(SizingOperation::MinContent, SizingOperation::Normal),
        Size::NONE,
        Size::NONE,
    );
    let mut style: NodeInputOf<S> = NodeInputOf {
        size: Size::new(PreferredSizeOf::STRETCH, PreferredSizeOf::STRETCH),
        min_size: Size::new(MinSizeOf::STRETCH, MinSizeOf::STRETCH),
        max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
        box_sizing: BoxSizing::BorderBox,
        ..NodeInputOf::default()
    };
    // No scrollport and no finite maximum need neither preferred nor minimum.
    let ignored = qualify(input, &style, SizingAlgorithm::Block).unwrap();
    assert_eq!(
        ignored.maximum_outer,
        Size::splat(FixedFallbackSource::NoFixedBound)
    );
    assert_eq!(
        ignored.minimum_outer,
        Size::splat(FixedFallbackSource::Unqualified)
    );
    assert_eq!(
        ignored.definite_outer,
        Size::splat(FixedFallbackSource::Unqualified)
    );
    style.max_size.height = MaxSizeOf::px(n(40.0));
    assert_unsupported(
        qualify(input, &style, SizingAlgorithm::Block),
        SizingProperty::Minimum,
        SizingBehavior::Stretch,
        PhysicalAxis::Vertical,
    );
    style.min_size.height = MinSizeOf::AUTO;
    style.overflow = hidden();
    assert_unsupported(
        qualify(input, &style, SizingAlgorithm::Block),
        SizingProperty::Preferred,
        SizingBehavior::Stretch,
        PhysicalAxis::Vertical,
    );
    // A contextual field on the queried/suppressed axis stays unqualified;
    // formal output Both does not make that source necessary or fixed.
    style.max_size.width = MaxSizeOf::STRETCH;
    style.size.height = PreferredSizeOf::AUTO;
    let facts = qualify(input, &style, SizingAlgorithm::Block).unwrap();
    assert_eq!(facts.maximum_outer.width, FixedFallbackSource::Unqualified);
    assert_eq!(
        facts.maximum_outer.height,
        FixedFallbackSource::Fixed(n(40.0))
    );
    let missing = child_orthogonal_fallback(
        inherited(),
        facts.definite_outer,
        facts.minimum_outer,
        facts.maximum_outer,
        source(&style),
    )
    .unwrap();
    assert_eq!(
        missing.containing_limits().width,
        OrthogonalFallbackLimitOf::Unknown
    );
    assert_eq!(
        missing.resolve(PhysicalAxis::Horizontal),
        Err(OrthogonalFallbackErrorOf::MissingContainingBounds {
            axis: PhysicalAxis::Horizontal
        })
    );
    // All authored coefficients and the admitted basis are finite. Their actual
    // product overflows at the demanded original-property resolution boundary.
    style.max_size.height =
        MaxSizeOf::value(LengthPercentageOf::from_percent_fraction(n(2.0)).unwrap());
    let input = query::<S>(
        SizingMode::ContentSize,
        Size::new(SizingOperation::MinContent, SizingOperation::Normal),
        Size::NONE,
        Size::new(None, Some(largest)),
    );
    let SizingResolutionError::Status(LengthResolutionStatus::InvalidNumeric { value }) =
        qualify(input, &style, SizingAlgorithm::Block).unwrap_err()
    else {
        panic!("overflow must preserve the real numeric error");
    };
    assert!(!value.is_finite());
    let raw = fixed_fallback_preferred::<S>(
        &PreferredSizeOf::px(largest),
        SizingAlgorithm::Block,
        PhysicalAxis::Horizontal,
        None,
    )
    .unwrap();
    assert!(matches!(outer_fallback_source(raw, largest),
        Err(LengthResolutionStatus::InvalidNumeric { value }) if !value.is_finite()));
    assert_eq!(remaining_available_extent(S::ZERO, n(10.0)), Ok(S::ZERO));
}

fn edge_debits_and_minimum_floor<S: LayoutScalar>() {
    for (box_sizing, local, scrollport) in [
        (
            BoxSizing::BorderBox,
            Size::new(32.0, 34.0),
            Size::new(34.0, 38.0),
        ),
        (
            BoxSizing::ContentBox,
            Size::new(38.0, 40.0),
            Size::new(40.0, 44.0),
        ),
    ] {
        let style: NodeInputOf<S> = NodeInputOf {
            min_size: Size::new(MinSizeOf::px(n(10.0)), MinSizeOf::px(n(10.0))),
            max_size: Size::new(MaxSizeOf::px(n(40.0)), MaxSizeOf::px(n(40.0))),
            box_sizing,
            overflow: hidden(),
            scrollbar_gutter: ScrollbarGutter::Stable,
            padding: Edges::new(
                LengthOf::px(n(2.0)),
                LengthOf::px(n(1.0)),
                LengthOf::px(n(2.0)),
                LengthOf::px(n(1.0)),
            ),
            border: Edges::new(
                LengthOf::px(n(1.0)),
                LengthOf::px(n(2.0)),
                LengthOf::px(n(1.0)),
                LengthOf::px(n(2.0)),
            ),
            scrollbar_width: ScrollbarWidthOf::try_new(n(2.0)).unwrap(),
            ..NodeInputOf::default()
        };
        let input = query::<S>(
            SizingMode::InherentSize,
            Size::splat(SizingOperation::Normal),
            Size::NONE,
            Size::NONE,
        );
        let facts = qualify(input, &style, SizingAlgorithm::Block).unwrap();
        let edges = source(&style);
        assert_eq!(edges.padding.sum_axes(), Size::new(n(2.0), n(4.0)));
        assert_eq!(edges.border.sum_axes(), Size::new(n(4.0), n(2.0)));
        let frame = child_orthogonal_fallback(
            inherited(),
            facts.definite_outer,
            facts.minimum_outer,
            facts.maximum_outer,
            source(&style),
        )
        .unwrap();
        assert_eq!(
            frame.containing_limits(),
            local.map(|v| OrthogonalFallbackLimitOf::Finite(nn(v)))
        );
        assert_eq!(
            frame.nearest_scrollport(),
            NearestScrollportOf::Present(
                scrollport.map(|v| OrthogonalFallbackLimitOf::Finite(nn(v)))
            )
        );
        assert_eq!(frame.resolve(PhysicalAxis::Horizontal), Ok(nn(local.width)));
        assert_eq!(frame.resolve(PhysicalAxis::Vertical), Ok(nn(local.height)));
        assert_eq!(
            frame.initial_containing_block(),
            inherited::<S>().initial_containing_block()
        );
    }
    let style: NodeInputOf<S> = NodeInputOf {
        min_size: Size::new(MinSizeOf::px(n(50.0)), MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::px(n(40.0)), MaxSizeOf::px(n(3.0))),
        box_sizing: BoxSizing::BorderBox,
        overflow: hidden(),
        scrollbar_gutter: ScrollbarGutter::Stable,
        padding: Edges::new(
            LengthOf::px(n(2.0)),
            LengthOf::px(n(1.0)),
            LengthOf::px(n(2.0)),
            LengthOf::px(n(1.0)),
        ),
        border: Edges::new(
            LengthOf::px(n(1.0)),
            LengthOf::px(n(2.0)),
            LengthOf::px(n(1.0)),
            LengthOf::px(n(2.0)),
        ),
        scrollbar_width: ScrollbarWidthOf::try_new(n(2.0)).unwrap(),
        ..NodeInputOf::default()
    };
    let facts = qualify(
        query::<S>(
            SizingMode::InherentSize,
            Size::splat(SizingOperation::Normal),
            Size::NONE,
            Size::NONE,
        ),
        &style,
        SizingAlgorithm::Block,
    )
    .unwrap();
    let frame = child_orthogonal_fallback(
        inherited(),
        facts.definite_outer,
        facts.minimum_outer,
        facts.maximum_outer,
        source(&style),
    )
    .unwrap();
    // X: border min50/max40 minus BP6+gutter2 =>42. Y: max3 minusBP6 exhausts to0.
    assert_eq!(
        frame.containing_limits(),
        Size::new(
            OrthogonalFallbackLimitOf::Finite(nn(42.0)),
            OrthogonalFallbackLimitOf::Finite(nn(0.0))
        )
    );
    assert_eq!(
        frame.nearest_scrollport(),
        NearestScrollportOf::Present(Size::new(
            OrthogonalFallbackLimitOf::Finite(nn(44.0)),
            OrthogonalFallbackLimitOf::Finite(nn(1.0))
        ))
    );
    let no_icb = OrthogonalFallbackOf::new(
        Size::NONE,
        NearestScrollportOf::Unknown,
        Size::splat(OrthogonalFallbackLimitOf::Unknown),
    );
    let exhausted = child_orthogonal_fallback(
        no_icb,
        facts.definite_outer,
        facts.minimum_outer,
        facts.maximum_outer,
        source(&style),
    )
    .unwrap();
    assert_eq!(
        exhausted.resolve(PhysicalAxis::Vertical),
        Ok(nn(0.0)),
        "a post-debit zero dominates unavailable external bounds"
    );
}

fn nearest_replacement<S: LayoutScalar>() {
    let original = OrthogonalFallbackOf::new(
        inherited::<S>().initial_containing_block(),
        NearestScrollportOf::Present(Size::new(
            OrthogonalFallbackLimitOf::Finite(nn(100.0)),
            OrthogonalFallbackLimitOf::Finite(nn(120.0)),
        )),
        Size::new(
            OrthogonalFallbackLimitOf::Finite(nn(50.0)),
            OrthogonalFallbackLimitOf::Finite(nn(60.0)),
        ),
    );
    let input = query::<S>(
        SizingMode::InherentSize,
        Size::splat(SizingOperation::Normal),
        Size::NONE,
        Size::NONE,
    );
    let mut style: NodeInputOf<S> = NodeInputOf {
        box_sizing: BoxSizing::BorderBox,
        overflow: hidden(),
        ..NodeInputOf::default()
    };
    let facts = qualify(input, &style, SizingAlgorithm::Block).unwrap();
    let uncapped = child_orthogonal_fallback(
        original,
        facts.definite_outer,
        facts.minimum_outer,
        facts.maximum_outer,
        source(&style),
    )
    .unwrap();
    assert_eq!(
        uncapped.nearest_scrollport(),
        NearestScrollportOf::Present(Size::splat(OrthogonalFallbackLimitOf::Unbounded))
    );
    assert_eq!(uncapped.resolve(PhysicalAxis::Horizontal), Ok(nn(800.0)));
    assert_eq!(uncapped.resolve(PhysicalAxis::Vertical), Ok(nn(900.0)));
    style.overflow = ComputedOverflow::VISIBLE;
    let facts = qualify(input, &style, SizingAlgorithm::Block).unwrap();
    let preserved = child_orthogonal_fallback(
        original,
        facts.definite_outer,
        facts.minimum_outer,
        facts.maximum_outer,
        source(&style),
    )
    .unwrap();
    assert_eq!(
        preserved.nearest_scrollport(),
        original.nearest_scrollport()
    );
    style.overflow = hidden();
    style.size = Size::new(PreferredSizeOf::px(n(200.0)), PreferredSizeOf::px(n(300.0)));
    let facts = qualify(input, &style, SizingAlgorithm::Block).unwrap();
    let replaced = child_orthogonal_fallback(
        original,
        facts.definite_outer,
        facts.minimum_outer,
        facts.maximum_outer,
        source(&style),
    )
    .unwrap();
    assert_eq!(replaced.resolve(PhysicalAxis::Horizontal), Ok(nn(200.0)));
    assert_eq!(replaced.resolve(PhysicalAxis::Vertical), Ok(nn(300.0)));
    // No formal axis establishes a ContentSize opposite here. Source values
    // remain unevaluated/unknown, never a fabricated fixed size or no-bound fact.
    let unknown = qualify(
        query::<S>(
            SizingMode::ContentSize,
            Size::splat(SizingOperation::Normal),
            Size::NONE,
            Size::NONE,
        ),
        &style,
        SizingAlgorithm::Block,
    )
    .unwrap();
    let missing = child_orthogonal_fallback(
        original,
        unknown.definite_outer,
        unknown.minimum_outer,
        unknown.maximum_outer,
        source(&style),
    )
    .unwrap();
    assert_eq!(
        missing.resolve(PhysicalAxis::Horizontal),
        Err(OrthogonalFallbackErrorOf::MissingScrollportLimit {
            axis: PhysicalAxis::Horizontal
        })
    );
}

struct OpaqueProbe<S: LayoutScalar> {
    tree: crate::test_support::layout_tree::PublicLayoutTreeOf<S>,
    calls: std::cell::RefCell<Vec<LeafMeasureInputOf<S>>>,
}
impl<S: LayoutScalar> Traverse for OpaqueProbe<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <crate::test_support::layout_tree::PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        crate::Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for OpaqueProbe<S> {
    type MeasureError = OrthogonalFallbackErrorOf<S>;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == 1
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        self.calls.borrow_mut().push(input);
        // This opaque Native formatter owns a natural physical 10x50 strip.
        // Its Normal orthogonal indefinite inline profile chooses the issued
        // fallback. Physical space and fallback authority remain distinct.
        let own_axis = input.flow_axes().inline_axis();
        let containing_axis = input.containing_layout_context().flow_axes().inline_axis();
        let packet = input.content_sizing_request();
        let request = match own_axis {
            PhysicalAxis::Horizontal => packet.axes().width,
            PhysicalAxis::Vertical => packet.axes().height,
        };
        let known = match own_axis {
            PhysicalAxis::Horizontal => input.known_content_size().width,
            PhysicalAxis::Vertical => input.known_content_size().height,
        };
        let band = if request.operation() != SizingOperation::Normal {
            None
        } else if let Some(space) = request.space().definite_size() {
            Some(space.get())
        } else if known.is_none() && own_axis != containing_axis {
            let required = match own_axis {
                PhysicalAxis::Horizontal => packet.orthogonal_fallback().width,
                PhysicalAxis::Vertical => packet.orthogonal_fallback().height,
            };
            match required {
                Ok(value) => Some(value.get()),
                Err(error) => return Some(Err(error)),
            }
        } else {
            None
        };
        let mut measured = Size::new(n::<S>(10.0), n::<S>(50.0));
        if let Some(band) = band {
            match own_axis {
                PhysicalAxis::Horizontal => measured.width = measured.width.min(band),
                PhysicalAxis::Vertical => measured.height = measured.height.min(band),
            }
        }
        Some(Ok(input
            .known_content_size()
            .zip_map(measured, |known, measured| {
                known.unwrap_or(measured)
            })))
    }
}

fn block_to_opaque_callback<S: LayoutScalar>() {
    for (box_sizing, content_height, border_height) in [
        (BoxSizing::BorderBox, 34.0, 40.0),
        (BoxSizing::ContentBox, 40.0, 46.0),
    ] {
        let root: NodeInputOf<S> = NodeInputOf {
            display: Display::Block,
            box_sizing,
            size: Size::new(PreferredSizeOf::px(n(100.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::px(n(10.0))),
            max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::px(n(40.0))),
            padding: Edges::new(
                LengthOf::px(n(1.0)),
                LengthOf::ZERO,
                LengthOf::px(n(3.0)),
                LengthOf::ZERO,
            ),
            border: Edges::new(
                LengthOf::px(n(1.0)),
                LengthOf::ZERO,
                LengthOf::px(n(1.0)),
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };
        let child: NodeInputOf<S> = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            box_sizing: BoxSizing::BorderBox,
            ..NodeInputOf::default()
        };
        let host: OpaqueProbe<S> = OpaqueProbe {
            tree: crate::test_support::layout_tree::PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, root)
                .style(1, child),
            calls: std::cell::RefCell::new(Vec::new()),
        };
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(nn(100.0)),
                        SizingOperation::Normal,
                    ),
                    axis(SizingOperation::Normal),
                ),
                inherited(),
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        let batch = compute_layout(&host, 0, request).unwrap();
        let calls = host.calls.borrow();
        assert!(!calls.is_empty(), "a real leaf provider must execute");
        for input in calls.iter() {
            assert_eq!(
                input.flow_axes(),
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr)
            );
            assert_eq!(
                input.containing_layout_context().flow_axes(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
            );
            assert_eq!(
                input.containing_layout_context().formatting_context(),
                ParentFormattingContext::BlockFlow
            );
            assert_eq!(
                input.known_content_size(),
                Size::NONE,
                "the parent's maximum is space authority, never a known child size"
            );
            assert_eq!(
                input.content_sizing_request().axes().height.operation(),
                SizingOperation::Normal
            );
            assert_eq!(
                input.content_sizing_request().axes().height.space(),
                AvailableSpaceOf::Indefinite
            );
            assert_eq!(
                input.content_sizing_request().orthogonal_fallback().height,
                Ok(nn(content_height))
            );
        }
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let root = entries.iter().find(|row| row.node() == 0).unwrap().output();
            let child = entries.iter().find(|row| row.node() == 1).unwrap().output();
            assert_eq!(root.location, Point::ZERO);
            assert_eq!(root.size, Size::new(n(100.0), n(border_height)));
            assert_eq!(child.size, Size::new(n(10.0), n(content_height)));
            assert_eq!(child.location, Point::new(S::ZERO, n(2.0)));
            let content = root.scroll_geometry.unwrap().content_box();
            assert_eq!(content.origin(), Point::new(S::ZERO, n(2.0)));
            assert_eq!(content.size(), Size::new(n(100.0), n(content_height)));
        }
        drop(calls);
        // The same parent cannot manufacture missing ICB authority.
        // Only the actual orthogonal Normal/Indefinite provider consumes this
        // required error; it is not a blanket rejection of every missing axis.
        let missing = OrthogonalFallbackOf::new(
            Size::new(Some(nn(800.0)), None),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        );
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(nn(100.0)),
                        SizingOperation::Normal,
                    ),
                    axis(SizingOperation::Normal),
                ),
                missing,
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        let error = match compute_layout(&host, 0, request) {
            Err(error) => error,
            Ok(_) => panic!("the provider must refuse its missing required fallback"),
        };
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement(
                OrthogonalFallbackErrorOf::MissingInitialContainingBlock {
                    axis: PhysicalAxis::Vertical
                }
            )
        ));
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .find(|row| row.node() == 1)
                .unwrap()
                .output()
                .size,
            Size::new(n(10.0), n(content_height)),
            "the prior successful batch remains usable"
        );
    }
}

#[test]
fn original_role_sources_f32() {
    original_roles::<f32>();
}
#[test]
fn original_role_sources_f64() {
    original_roles::<f64>();
}
#[test]
fn original_calc_size_dependency_f32() {
    original_calc_dependency::<f32>();
}
#[test]
fn original_calc_size_dependency_f64() {
    original_calc_dependency::<f64>();
}
#[test]
fn raw_content_purpose_and_known_f32() {
    raw_purpose_and_known::<f32>();
}
#[test]
fn raw_content_purpose_and_known_f64() {
    raw_purpose_and_known::<f64>();
}
#[test]
fn needed_only_typed_refusals_f32() {
    needed_only_refusals::<f32>(f32::MAX);
}
#[test]
fn needed_only_typed_refusals_f64() {
    needed_only_refusals::<f64>(f64::MAX);
}
#[test]
fn box_edges_and_minimum_floor_f32() {
    edge_debits_and_minimum_floor::<f32>();
}
#[test]
fn box_edges_and_minimum_floor_f64() {
    edge_debits_and_minimum_floor::<f64>();
}
#[test]
fn actual_nearest_scrollport_replacement_f32() {
    nearest_replacement::<f32>();
}
#[test]
fn actual_nearest_scrollport_replacement_f64() {
    nearest_replacement::<f64>();
}
#[test]
fn real_block_to_orthogonal_opaque_callback_f32() {
    block_to_opaque_callback::<f32>();
}
#[test]
fn real_block_to_orthogonal_opaque_callback_f64() {
    block_to_opaque_callback::<f64>();
}
