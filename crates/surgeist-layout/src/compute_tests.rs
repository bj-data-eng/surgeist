use crate::*;
use crate::{
    Available, ComputeInput, Edges, Length, LengthPercentageOf, NodeInput, PreferredSize,
    RequestedAxis,
};

mod leaf_box_arithmetic {
    use super::*;

    fn input<S: LayoutScalar>(known: Size<Option<S>>, content_only: bool) -> ComputeInputOf<S> {
        let constructor = if content_only {
            ComputeInputOf::leaf_content_size
        } else {
            ComputeInputOf::leaf_layout
        };
        constructor(
            known,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            Size::splat(AvailableOf::MAX_CONTENT),
        )
        .unwrap()
    }

    fn assert_invalid_box_edges<S: LayoutScalar>(largest: S) {
        let largest_length = LengthOf::value(LengthPercentageOf::px(largest).unwrap());
        let negative_length = LengthOf::value(LengthPercentageOf::px(-S::ONE).unwrap());
        let cases = [
            (
                Edges::all(largest_length),
                Edges::all(LengthOf::ZERO),
                S::INFINITY,
            ),
            (
                Edges::all(LengthOf::ZERO),
                Edges::all(largest_length),
                S::INFINITY,
            ),
            (
                Edges::new(
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    largest_length,
                ),
                Edges::new(
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    LengthOf::ZERO,
                    largest_length,
                ),
                S::INFINITY,
            ),
            (
                Edges::all(negative_length),
                Edges::all(LengthOf::ZERO),
                -S::ONE,
            ),
            (
                Edges::all(LengthOf::ZERO),
                Edges::all(negative_length),
                -S::ONE,
            ),
        ];
        for content_only in [true, false] {
            for (padding, border, expected) in cases {
                let style = NodeInputOf {
                    padding,
                    border,
                    ..NodeInputOf::default()
                };
                let mut calls = 0;
                let error = compute_leaf(
                    input(Size::splat(Some(S::from_f64(10.0))), content_only),
                    &style,
                    |_| {
                        calls += 1;
                        Ok::<_, ()>(Size::ZERO)
                    },
                )
                .expect_err("invalid resolved box edges cannot produce a successful size");
                assert_eq!(calls, 0);
                assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
                assert_eq!(error.operation(), LayoutOperation::ValueResolution);
                crate::test_support::assert_error_kind(
                    error.kind(),
                    &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                        value: expected,
                    }),
                );
            }
        }
    }

    fn assert_output_overflow<S: LayoutScalar>(largest: S) {
        let half = largest / S::from_f64(2.0);
        let mut padding = Edges::all(LengthOf::ZERO);
        padding.left = LengthOf::value(LengthPercentageOf::px(half).unwrap());
        for content_only in [true, false] {
            for style in [
                NodeInputOf {
                    padding,
                    ..NodeInputOf::default()
                },
                NodeInputOf {
                    aspect_ratio: Some(AspectRatioOf::new(S::from_f64(0.5)).unwrap()),
                    ..NodeInputOf::default()
                },
            ] {
                if content_only && style.aspect_ratio.is_some() {
                    continue;
                }
                // The provider's finite output is valid; overflow belongs to layout arithmetic.
                let error = compute_leaf(input(Size::NONE, content_only), &style, |_| {
                    Ok::<_, ()>(Size::new(largest, S::ONE))
                })
                .expect_err("derived box dimensions must remain finite");
                assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
                assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
                assert!(matches!(error.kind(), LayoutErrorKindOf::InvalidInput(
                    LayoutInvalidInputOf::InvalidNumeric { value }
                ) if !value.is_finite()));
            }
        }
    }

    fn assert_large_valid_boxes<S: LayoutScalar>(largest: S) {
        let edge = largest / S::from_f64(16.0);
        let known = largest / S::from_f64(4.0);
        let style = NodeInputOf {
            padding: Edges::all(LengthOf::value(LengthPercentageOf::px(edge).unwrap())),
            // Signed margins are not padding or border and remain valid.
            margin: Edges::all(LengthAutoOf::value(LengthPercentageOf::px(-edge).unwrap())),
            ..NodeInputOf::default()
        };
        for content_only in [true, false] {
            let mut calls = 0;
            let output = compute_leaf(
                input(Size::splat(Some(known)), content_only),
                &style,
                |_| {
                    calls += 1;
                    Ok::<_, ()>(Size::splat(edge))
                },
            )
            .expect("large representable box arithmetic remains valid");
            assert_eq!(output.size, Size::splat(known));
            assert!(
                output.content_size.width.is_finite() && output.content_size.height.is_finite()
            );
            assert_eq!(calls, usize::from(!content_only));
        }
    }

    fn assert_resolved_size_overflow<S: LayoutScalar>(largest: S) {
        let large = LengthPercentageOf::px(largest).unwrap();
        let mut padding = Edges::all(LengthOf::ZERO);
        padding.left = LengthOf::value(LengthPercentageOf::px(largest / S::from_f64(2.0)).unwrap());
        for style in [
            NodeInputOf {
                padding,
                box_sizing: BoxSizing::ContentBox,
                size: Size::new(PreferredSizeOf::value(large), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                padding,
                box_sizing: BoxSizing::ContentBox,
                min_size: Size::new(MinSizeOf::value(large), MinSizeOf::AUTO),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                padding,
                box_sizing: BoxSizing::ContentBox,
                max_size: Size::new(MaxSizeOf::value(large), MaxSizeOf::NONE),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                size: Size::new(PreferredSizeOf::value(large), PreferredSizeOf::AUTO),
                aspect_ratio: AspectRatioOf::new(S::from_f64(0.5)),
                ..NodeInputOf::default()
            },
        ] {
            let mut calls = 0;
            let error = compute_leaf(input(Size::NONE, false), &style, |_| {
                calls += 1;
                Ok::<_, ()>(Size::ZERO)
            })
            .expect_err("resolved size arithmetic must be checked before measurement");
            assert_eq!(calls, 0);
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            crate::test_support::assert_error_kind(
                error.kind(),
                &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                    value: S::INFINITY,
                }),
            );
        }
    }

    #[test]
    fn leaf_box_arithmetic_rejects_invalid_edges_before_measurement() {
        assert_invalid_box_edges(f32::MAX);
        assert_invalid_box_edges(f64::MAX);
    }

    #[test]
    fn leaf_box_arithmetic_rejects_derived_output_overflow() {
        assert_output_overflow(f32::MAX);
        assert_output_overflow(f64::MAX);
        assert_resolved_size_overflow(f32::MAX);
        assert_resolved_size_overflow(f64::MAX);
    }

    #[test]
    fn leaf_box_arithmetic_preserves_large_valid_boxes_and_known_size_shortcut() {
        assert_large_valid_boxes(f32::MAX);
        assert_large_valid_boxes(f64::MAX);
    }
}

fn invalid_numeric_affine_value() -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients")
}

fn invalid_numeric_affine_input() -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(f32::MAX), None),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    )
}

#[test]
fn completed_batch_exposes_read_only_layout_and_cache_entries() {
    let unrounded = NodeOutput {
        size: Size::new(10.25, 20.5),
        ..NodeOutput::default()
    };
    let final_layout = NodeOutput {
        size: Size::new(10.0, 21.0),
        ..NodeOutput::default()
    };
    let cache_output = ComputeOutput::from_sizes(Size::new(10.25, 20.5), Size::new(8.0, 16.0));
    let cache_input = ComputeInput::leaf_layout(
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(40.0), Available::definite(30.0)),
    )
    .expect("test availability is valid");
    let cache_context = CacheKeyContext::new();

    let batch = CompletedLayoutBatch::from_entries(
        vec![LayoutOutputEntry::new(7, unrounded)],
        vec![LayoutOutputEntry::new(7, final_layout)],
        Vec::new(),
        Vec::new(),
        vec![LayoutCacheStoreEntry::new(
            7,
            cache_input,
            cache_context,
            cache_output,
        )],
        vec![LayoutCacheClearEntry::new(11)],
        Vec::new(),
    );

    assert_eq!(batch.unrounded_entries()[0].node(), 7);
    assert_eq!(batch.unrounded_entries()[0].output(), unrounded);
    assert_eq!(batch.final_entries()[0].node(), 7);
    assert_eq!(batch.final_entries()[0].output(), final_layout);
    assert_eq!(batch.cache_store_entries()[0].node(), 7);
    assert_eq!(batch.cache_store_entries()[0].input(), &cache_input);
    assert_eq!(batch.cache_store_entries()[0].context(), cache_context);
    assert_eq!(batch.cache_store_entries()[0].output(), cache_output);
    assert_eq!(batch.cache_clear_entries()[0].node(), 11);
}

fn fragment_value<S: LayoutScalar>(
    fragment: surgeist_text::FinalizedFragment,
    origin: Point<S>,
    size: Size<S>,
    baseline: Point<S>,
    line_index: usize,
) -> InlineFragmentOutputOf<S> {
    InlineFragmentOutputOf::new(
        fragment,
        Some(ScrollRectOf::try_new(origin, size).unwrap()),
        baseline,
        line_index,
    )
}

fn assert_fragment_output_contract<S: LayoutScalar>() {
    fn assert_immutable_carrier<T: Clone + core::fmt::Debug + PartialEq>() {}
    assert_immutable_carrier::<InlineFragmentOutputOf<S>>();
    assert_immutable_carrier::<InlineFragmentOutputEntryOf<u32, S>>();
    let handle = crate::test_support::text_fragment::fragment();
    let rect = ScrollRectOf::try_new(
        Point::new(S::from_f64(1.25), S::from_f64(2.5)),
        Size::new(S::from_f64(3.75), S::from_f64(4.5)),
    )
    .unwrap();
    let fragment = fragment_value(
        handle.clone(),
        rect.origin(),
        rect.size(),
        Point::new(S::from_f64(1.25), S::from_f64(5.5)),
        2,
    );
    assert_eq!(fragment.finalized_fragment(), &handle);
    assert_eq!(fragment.rect(), Some(rect));
    assert_eq!(
        fragment.baseline(),
        Point::new(S::from_f64(1.25), S::from_f64(5.5))
    );
    assert_eq!(fragment.line_index(), 2);
    let entry = InlineFragmentOutputEntryOf::new(7_u32, fragment.clone());
    assert_eq!(entry.node(), 7);
    assert_eq!(entry.fragment(), &fragment);
}

#[test]
fn finalized_fragment_carriers_retain_actual_owner_in_both_scalar_lanes() {
    assert_fragment_output_contract::<f32>();
    assert_fragment_output_contract::<f64>();
}

fn assert_fragment_batch_phases<S: LayoutScalar>() {
    let first = crate::test_support::text_fragment::fragment();
    let second = crate::test_support::text_fragment::fragment();
    let output = |handle, x: f64, y: f64, width: f64, height: f64, baseline: f64| {
        fragment_value(
            handle,
            Point::new(S::from_f64(x), S::from_f64(y)),
            Size::new(S::from_f64(width), S::from_f64(height)),
            Point::new(S::from_f64(x), S::from_f64(baseline)),
            0,
        )
    };
    let first_unrounded = output(first.clone(), 0.25, 0.5, 4.5, 2.25, 2.0);
    let second_unrounded = output(second.clone(), 5.25, 0.5, 3.5, 2.25, 2.0);
    let first_final = output(first.clone(), 0.0, 1.0, 5.0, 2.0, 2.0);
    let second_final = output(second.clone(), 5.0, 1.0, 4.0, 2.0, 2.0);
    let batch = CompletedLayoutBatchOf::from_entries(
        Vec::new(),
        Vec::new(),
        vec![
            InlineFragmentOutputEntryOf::new(10_u32, first_unrounded.clone()),
            InlineFragmentOutputEntryOf::new(10_u32, second_unrounded),
        ],
        vec![
            InlineFragmentOutputEntryOf::new(10_u32, first_final.clone()),
            InlineFragmentOutputEntryOf::new(10_u32, second_final.clone()),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(
        batch
            .unrounded_inline_fragments()
            .iter()
            .map(|entry| (entry.node(), entry.fragment().finalized_fragment().clone()))
            .collect::<Vec<_>>(),
        vec![(10, first), (10, second)]
    );
    assert_eq!(
        batch
            .final_inline_fragments()
            .iter()
            .map(|entry| entry.fragment().clone())
            .collect::<Vec<_>>(),
        vec![first_final.clone(), second_final]
    );
    assert_ne!(first_unrounded.rect(), first_final.rect());
    assert!(batch.unrounded_entries().is_empty());
    assert!(batch.final_entries().is_empty());
    assert!(batch.cache_store_entries().is_empty());
    assert!(batch.cache_clear_entries().is_empty());
}

#[test]
fn fragment_batch_preserves_actual_associations_and_separate_phase_geometry() {
    assert_fragment_batch_phases::<f32>();
    assert_fragment_batch_phases::<f64>();
}

#[test]
fn layout_error_accessors_preserve_typed_site_operation_kind_and_source() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct ProviderSource(&'static str);

    let error = LayoutError::new(
        LayoutErrorSite::ContainerSubject {
            container: 1,
            subject: 2,
        },
        LayoutOperation::LeafMeasurement,
        LayoutErrorKind::Measurement(ProviderSource("measure failed")),
    );

    assert_eq!(
        error.site(),
        LayoutErrorSite::ContainerSubject {
            container: 1,
            subject: 2,
        }
    );
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::Measurement(ProviderSource("measure failed")),
    );

    let scalar_error = LayoutError::<u32, ProviderSource>::new(
        LayoutErrorSite::Node(3),
        LayoutOperation::RootLayout,
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::RootAvailability {
            axis: PhysicalAxis::Horizontal,
            error: NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 },
        }),
    );

    crate::test_support::assert_error_kind(
        scalar_error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::RootAvailability {
            axis: PhysicalAxis::Horizontal,
            error: NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 },
        }),
    );
}

#[test]
fn value_resolution_diagnostics_classify_represented_non_numeric_values_as_unsupported() {
    let error: LayoutErrorOf<u32, f32> = crate::error::value_resolution_error_at_site(
        LayoutErrorSite::Node(3),
        LengthResolutionStatus::NonNumeric,
    );

    assert_eq!(error.site(), LayoutErrorSite::Node(3));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior),
    );
}

#[derive(Clone, Copy, Debug)]
enum InvalidLeafInputScalar {
    Negative,
    Nan,
    PositiveInfinity,
    NegativeInfinity,
}

type LeafInputConstructor<S> = fn(
    Size<Option<S>>,
    Size<Option<S>>,
    crate::ContainingLayoutContext,
    Size<AvailableOf<S>>,
) -> Result<ComputeInputOf<S>, RootAvailabilityErrorOf<S>>;

fn invalid_leaf_input_scalar<S: LayoutScalar>(case: InvalidLeafInputScalar) -> S {
    match case {
        InvalidLeafInputScalar::Negative => S::from_f64(-1.0),
        InvalidLeafInputScalar::Nan => S::NAN,
        InvalidLeafInputScalar::PositiveInfinity => S::INFINITY,
        InvalidLeafInputScalar::NegativeInfinity => -S::INFINITY,
    }
}

fn assert_leaf_input_error<S: LayoutScalar>(
    constructor: LeafInputConstructor<S>,
    known: Size<Option<S>>,
    parent: Size<Option<S>>,
    axis: PhysicalAxis,
    case: InvalidLeafInputScalar,
) {
    let error = constructor(
        known,
        parent,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MinContent, AvailableOf::MaxContent),
    )
    .unwrap_err();

    assert_eq!(error.axis(), axis);
    match (case, error.scalar()) {
        (InvalidLeafInputScalar::Negative, NonNegativeFiniteScalarErrorOf::Negative { value }) => {
            assert_eq!(value, S::from_f64(-1.0))
        }
        (InvalidLeafInputScalar::Nan, NonNegativeFiniteScalarErrorOf::NonFinite { value }) => {
            assert!(value.to_f64().is_nan());
        }
        (
            InvalidLeafInputScalar::PositiveInfinity,
            NonNegativeFiniteScalarErrorOf::NonFinite { value },
        ) => assert_eq!(value, S::INFINITY),
        (
            InvalidLeafInputScalar::NegativeInfinity,
            NonNegativeFiniteScalarErrorOf::NonFinite { value },
        ) => assert_eq!(value, -S::INFINITY),
        (case, actual) => panic!("expected {case:?} scalar error, got {actual:?}"),
    }
}

fn assert_leaf_input_constructors_validate_all_scalars<S: LayoutScalar>() {
    let constructors = [
        ComputeInputOf::<S>::leaf_layout as LeafInputConstructor<S>,
        ComputeInputOf::<S>::leaf_content_size,
    ];
    let valid_known = Size::new(None, Some(S::ZERO));
    let valid_parent = Size::new(Some(S::ZERO), None);
    let valid_available = Size::new(AvailableOf::MinContent, AvailableOf::MaxContent);
    let containing_flow_axes =
        crate::geometry::FlowAxes::new(crate::WritingMode::VerticalRl, crate::Direction::Rtl);
    let containing_layout_context = crate::ContainingLayoutContext::new(
        containing_flow_axes,
        crate::ParentFormattingContext::NoParent,
    );

    for constructor in constructors {
        let input = constructor(
            valid_known,
            valid_parent,
            containing_layout_context,
            valid_available,
        )
        .expect("zero definite and indefinite leaf inputs are valid");
        assert_eq!(input.known(), valid_known);
        assert_eq!(input.parent(), valid_parent);
        assert_eq!(input.containing_flow_axes(), containing_flow_axes);
        assert_eq!(input.available(), valid_available);

        for case in [
            InvalidLeafInputScalar::Negative,
            InvalidLeafInputScalar::Nan,
            InvalidLeafInputScalar::PositiveInfinity,
            InvalidLeafInputScalar::NegativeInfinity,
        ] {
            let value = invalid_leaf_input_scalar::<S>(case);
            assert_leaf_input_error(
                constructor,
                Size::new(Some(value), Some(S::ZERO)),
                Size::new(Some(S::ZERO), Some(S::ZERO)),
                PhysicalAxis::Horizontal,
                case,
            );
            assert_leaf_input_error(
                constructor,
                Size::new(Some(S::ZERO), Some(value)),
                Size::new(Some(S::ZERO), Some(S::ZERO)),
                PhysicalAxis::Vertical,
                case,
            );
            assert_leaf_input_error(
                constructor,
                Size::new(Some(S::ZERO), Some(S::ZERO)),
                Size::new(Some(value), Some(S::ZERO)),
                PhysicalAxis::Horizontal,
                case,
            );
            assert_leaf_input_error(
                constructor,
                Size::new(Some(S::ZERO), Some(S::ZERO)),
                Size::new(Some(S::ZERO), Some(value)),
                PhysicalAxis::Vertical,
                case,
            );
        }
    }
}

#[test]
fn compute_input_leaf_constructors_retain_non_horizontal_flow_for_f32() {
    assert_leaf_input_constructors_validate_all_scalars::<f32>();
}

#[test]
fn compute_input_leaf_constructors_retain_non_horizontal_flow_for_f64() {
    assert_leaf_input_constructors_validate_all_scalars::<f64>();
}

#[test]
fn leaf_affine_width_resolves_against_parent_basis() {
    let width = LengthPercentageOf::from_coefficients(10.0, 0.5).expect("finite coefficients");
    let style = NodeInput {
        size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(100.0), None),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    );

    let output = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(12.0, 8.0))).unwrap();

    assert_eq!(output.size.width, 60.0);
}

#[test]
fn public_leaf_affine_px_width_needs_no_resolver() {
    let width = LengthPercentageOf::px(10.0).expect("finite px");
    let style = NodeInput {
        size: Size::new(PreferredSize::value(width), PreferredSize::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(100.0), None),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    );

    let output = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(12.0, 8.0))).unwrap();

    assert_eq!(output.size.width, 10.0);
}

#[test]
fn public_leaf_invalid_numeric_affine_width_returns_typed_error() {
    let style = NodeInput {
        size: Size::new(
            PreferredSize::value(invalid_numeric_affine_value()),
            PreferredSize::AUTO,
        ),
        ..NodeInput::default()
    };

    let error = compute_leaf(invalid_numeric_affine_input(), &style, |_input| {
        Ok::<_, ()>(Size::new(12.0, 8.0))
    })
    .expect_err("invalid affine width must not fall back to measurement");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        }),
    );
}

#[test]
fn public_leaf_invalid_numeric_affine_padding_returns_typed_error() {
    let style = NodeInput {
        padding: Edges::all(Length::value(invalid_numeric_affine_value())),
        ..NodeInput::default()
    };

    let error = compute_leaf(invalid_numeric_affine_input(), &style, |_input| {
        Ok::<_, ()>(Size::new(12.0, 8.0))
    })
    .expect_err("invalid affine padding must not fall back to zero");

    assert_eq!(error.site(), LayoutErrorSite::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::INFINITY,
        }),
    );
}

#[test]
fn public_f64_leaf_invalid_numeric_affine_width_returns_typed_error() {
    let style = NodeInputOf::<f64> {
        size: Size::new(
            PreferredSizeOf::value(
                LengthPercentageOf::from_coefficients(f64::MAX, 1.0).expect("finite coefficients"),
            ),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::new(Some(f64::MAX), None),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::definite(100.0), AvailableOf::MAX_CONTENT),
    )
    .expect("finite leaf input is valid");

    let error = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(12.0, 8.0)))
        .expect_err("invalid affine width must not fall back to measurement");

    assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
            value: f64::INFINITY,
        }),
    );
}

fn assert_public_leaf_missing_basis_returns_typed_error<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        padding: Edges::all(LengthOf::percent(S::from_f64(0.5))),
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    )
    .expect("indefinite leaf input is valid");

    let error = compute_leaf(input, &style, |_input| -> Result<Size<S>, ()> {
        panic!("missing required basis must not invoke the provider")
    })
    .expect_err("missing required basis must not fall back to zero");

    assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis),
    );
}

#[test]
fn public_f32_leaf_missing_basis_returns_typed_error() {
    assert_public_leaf_missing_basis_returns_typed_error::<f32>();
}

#[test]
fn public_f64_leaf_missing_basis_returns_typed_error() {
    assert_public_leaf_missing_basis_returns_typed_error::<f64>();
}

fn assert_public_leaf_intrinsic_percent_padding_is_valid_without_basis<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        padding: Edges::all(LengthOf::percent(S::from_f64(0.5))),
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_content_size(
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    )
    .expect("indefinite intrinsic leaf input is valid");

    let output = compute_leaf(input, &style, |measure_input| {
        assert_eq!(measure_input.known_content_size(), Size::NONE);
        assert_eq!(
            measure_input.available_content_size(),
            Size::new(
                MeasurementAvailableOf::MAX_CONTENT,
                MeasurementAvailableOf::MAX_CONTENT,
            )
        );
        Ok::<_, ()>(Size::new(S::from_f64(12.0), S::from_f64(8.0)))
    })
    .expect("intrinsic percentage padding remains explicitly basis-independent");

    assert_eq!(output.size, Size::new(S::from_f64(12.0), S::from_f64(8.0)));
}

#[test]
fn public_f32_leaf_intrinsic_percent_padding_is_valid_without_basis() {
    assert_public_leaf_intrinsic_percent_padding_is_valid_without_basis::<f32>();
}

#[test]
fn public_f64_leaf_intrinsic_percent_padding_is_valid_without_basis() {
    assert_public_leaf_intrinsic_percent_padding_is_valid_without_basis::<f64>();
}

fn assert_public_leaf_basis_independent_width_is_valid<S: LayoutScalar>() {
    let style = NodeInputOf::<S> {
        size: Size::new(
            PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(10.0)).expect("finite px")),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    )
    .expect("indefinite leaf input is valid");

    let output = compute_leaf(input, &style, |measure_input| {
        assert_eq!(
            measure_input.available_content_size(),
            Size::new(
                MeasurementAvailableOf::definite(S::from_f64(10.0)).expect("finite width is valid"),
                MeasurementAvailableOf::MAX_CONTENT,
            )
        );
        Ok::<_, ()>(Size::new(S::from_f64(12.0), S::from_f64(8.0)))
    })
    .expect("basis-independent leaf width must remain valid without a parent basis");

    assert_eq!(output.size, Size::new(S::from_f64(10.0), S::from_f64(8.0)));
}

#[test]
fn public_f32_leaf_basis_independent_width_is_valid_without_parent_basis() {
    assert_public_leaf_basis_independent_width_is_valid::<f32>();
}

#[test]
fn public_f64_leaf_basis_independent_width_is_valid_without_parent_basis() {
    assert_public_leaf_basis_independent_width_is_valid::<f64>();
}
