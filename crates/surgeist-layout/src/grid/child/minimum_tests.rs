use super::*;
use crate::{
    AspectRatioOf, ComputedOverflow, Direction, NodeInputOf, SizingCalculationOf, WritingMode,
};

// These qualify the private pure planner on authored normalized box properties.
// They do not claim a completed child measurement or fabricate a provider output.
fn prepared<S: LayoutScalar>(known: Size<Option<S>>, parent: Size<Option<S>>) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        crate::RequestedAxis::Both,
        known,
        parent,
        crate::ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            crate::ParentFormattingContext::Grid,
        ),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MinContent,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}

fn plan<S: LayoutScalar>(
    input: &NodeInputOf<S>,
    prepared: ComputeInputOf<S>,
    tracks: &[TrackSizingOf<S>],
) -> Result<GridMinimumPlan<S>, SizingResolutionError<S>> {
    let item = GridItemProjection::from_node(input);
    plan_grid_minimum(GridMinimumContext {
        item: &item,
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        axis: GridAxisKind::Column,
        prepared,
        margin_basis: prepared.parent(),
        span: GridMinimumTrackFunctions::new(tracks, None).span_facts(S::ZERO)?,
        opposite_fixed_max_span: None,
        subgrid_requests: Size::new(false, false),
        algorithm: SizingAlgorithm::Grid,
    })
}

fn known<S: LayoutScalar>(plan: GridMinimumPlan<S>) -> S {
    match plan {
        GridMinimumPlan::KnownBorder(value) => value,
        GridMinimumPlan::ReuseMinContent | GridMinimumPlan::ContentSuggestion(_) => {
            panic!("expected an independently resolved border minimum")
        }
    }
}

fn assert_minimum_properties<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let mut input = NodeInputOf::<S> {
        box_sizing: BoxSizing::ContentBox,
        overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
        min_size: Size::new(crate::MinSizeOf::px(scalar(15.0)), crate::MinSizeOf::AUTO),
        padding: Edges {
            top: LengthOf::ZERO,
            bottom: LengthOf::ZERO,
            left: LengthOf::px(scalar(3.0)),
            right: LengthOf::px(scalar(4.0)),
        },
        border: Edges {
            top: LengthOf::ZERO,
            bottom: LengthOf::ZERO,
            left: LengthOf::px(scalar(1.0)),
            right: LengthOf::px(scalar(2.0)),
        },
        ..NodeInputOf::default()
    };
    let tracks = [TrackSizingOf::AUTO];
    let request = prepared(Size::NONE, Size::NONE);
    assert_eq!(known(plan(&input, request, &tracks).unwrap()), scalar(25.0));
    input.box_sizing = BoxSizing::BorderBox;
    assert_eq!(known(plan(&input, request, &tracks).unwrap()), scalar(15.0));
    input.min_size.width = crate::MinSizeOf::px(scalar(5.0));
    assert_eq!(known(plan(&input, request, &tracks).unwrap()), scalar(10.0));
    input.min_size.width = crate::MinSizeOf::calculation(SizingCalculationOf::value(
        crate::LengthPercentageOf::from_coefficients(scalar(15.0), scalar(0.2)).unwrap(),
    ));
    assert_eq!(known(plan(&input, request, &tracks).unwrap()), scalar(15.0));

    let input = NodeInputOf {
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::AUTO),
        ..NodeInputOf::default()
    };
    assert!(matches!(
        plan(&input, request, &tracks).unwrap(),
        GridMinimumPlan::ReuseMinContent
    ));

    let input = NodeInputOf {
        size: Size::new(PreferredSizeOf::percent(scalar(0.5)), PreferredSizeOf::AUTO),
        overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
        ..NodeInputOf::default()
    };
    let request = prepared(Size::NONE, Size::new(Some(scalar(100.0)), None));
    assert_eq!(known(plan(&input, request, &tracks).unwrap()), S::ZERO);
}

fn assert_preferred_transfer_authority<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let mut input = NodeInputOf::<S> {
        item_is_replaced: true,
        aspect_ratio: Some(AspectRatioOf::new(scalar(2.0)).unwrap()),
        ..NodeInputOf::default()
    };
    let tracks = [TrackSizingOf::AUTO];
    let request = prepared(Size::new(None, Some(scalar(30.0))), Size::NONE);
    let GridMinimumPlan::ContentSuggestion(suggestion) = plan(&input, request, &tracks).unwrap()
    else {
        panic!("an Auto preferred size does not establish a transferred suggestion");
    };
    assert_eq!(suggestion.query().known().height, Some(scalar(30.0)));
    assert_eq!(suggestion.query().known().width, None);
    assert_eq!(suggestion.query().sizing_mode(), SizingMode::ContentSize);
    assert_eq!(suggestion.finish(scalar(20.0)).unwrap(), scalar(20.0));
    input.size.height = PreferredSizeOf::px(scalar(30.0));
    assert_eq!(known(plan(&input, request, &tracks).unwrap()), scalar(60.0));
}

fn assert_clip_and_fixed_cap<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let input = NodeInputOf::<S> {
        overflow: ComputedOverflow::try_new(Overflow::Clip, Overflow::Clip).unwrap(),
        ..NodeInputOf::default()
    };
    let request = prepared(Size::NONE, Size::NONE);
    let tracks = [TrackSizingOf::fit_content(SizingCalculationOf::value(
        crate::LengthPercentageOf::px(scalar(10.0)).unwrap(),
    ))];
    let GridMinimumPlan::ContentSuggestion(suggestion) = plan(&input, request, &tracks).unwrap()
    else {
        panic!("Clip keeps its content-based automatic minimum");
    };
    assert_eq!(suggestion.finish(scalar(40.0)).unwrap(), scalar(40.0));
    let tracks = [TrackSizingOf::minmax(
        MinTrackSizingOf::Auto,
        MaxTrackSizingOf::Calculation(SizingCalculationOf::value(
            crate::LengthPercentageOf::px(scalar(10.0)).unwrap(),
        )),
    )];
    let GridMinimumPlan::ContentSuggestion(suggestion) = plan(&input, request, &tracks).unwrap()
    else {
        panic!("the fixed maximum clamps a content suggestion");
    };
    assert_eq!(suggestion.finish(scalar(40.0)).unwrap(), scalar(10.0));
    let tracks = [TrackSizingOf::AUTO];
    let GridMinimumPlan::ContentSuggestion(suggestion) = plan(&input, request, &tracks).unwrap()
    else {
        panic!("Clip still demands a content suggestion");
    };
    assert!(matches!(
        suggestion.finish(S::INFINITY),
        Err(SizingResolutionError::Status(
            LengthResolutionStatus::InvalidNumeric { .. }
        ))
    ));
}

#[test]
fn minimum_planner_properties_f32() {
    assert_minimum_properties::<f32>();
    assert_preferred_transfer_authority::<f32>();
    assert_clip_and_fixed_cap::<f32>();
}

#[test]
fn minimum_planner_properties_f64() {
    assert_minimum_properties::<f64>();
    assert_preferred_transfer_authority::<f64>();
    assert_clip_and_fixed_cap::<f64>();
}

fn assert_original_function_retention<S: LayoutScalar>() {
    let fixed = |value| {
        TrackSizingOf::minmax(
            MinTrackSizingOf::Auto,
            MaxTrackSizingOf::Calculation(SizingCalculationOf::value(
                crate::LengthPercentageOf::px(S::from_f64(value)).unwrap(),
            )),
        )
    };
    let tracks = [TrackSizingOf::AUTO, fixed(10.0), fixed(20.0)];
    let original = GridMinimumTrackFunctions::new(&tracks, None);
    let reversed = original.mapped_span(1, 3, true).unwrap();
    let retained = reversed.retain().unwrap();
    let borrowed = GridMinimumTrackFunctions::from_qualified(&retained);
    assert_eq!(
        borrowed
            .span_facts(S::from_f64(3.0))
            .unwrap()
            .fixed_max_span(),
        Some(S::from_f64(33.0))
    );
    assert_eq!(
        borrowed
            .mapped_span(0, 1, false)
            .unwrap()
            .span_facts(S::ZERO)
            .unwrap()
            .fixed_max_span(),
        Some(S::from_f64(20.0))
    );
    assert_eq!(
        borrowed
            .mapped_span(0, 2, true)
            .unwrap()
            .mapped_span(0, 1, false)
            .unwrap()
            .span_facts(S::ZERO)
            .unwrap()
            .fixed_max_span(),
        Some(S::from_f64(10.0))
    );
    assert!(original.mapped_span(1, 1, false).is_none());
    assert!(original.mapped_span(2, 4, false).is_none());

    let tracks = [TrackSizingOf::minmax(
        MinTrackSizingOf::Auto,
        MaxTrackSizingOf::Calculation(SizingCalculationOf::value(
            crate::LengthPercentageOf::from_coefficients(S::ZERO, S::from_f64(0.5)).unwrap(),
        )),
    )];
    assert_eq!(
        GridMinimumTrackFunctions::new(&tracks, None)
            .span_facts(S::ZERO)
            .unwrap()
            .fixed_max_span(),
        None
    );
    assert_eq!(
        GridMinimumTrackFunctions::new(&tracks, Some(S::from_f64(100.0)))
            .span_facts(S::ZERO)
            .unwrap()
            .fixed_max_span(),
        Some(S::from_f64(50.0))
    );
}

#[test]
fn original_function_retention_f32() {
    assert_original_function_retention::<f32>();
}

#[test]
fn original_function_retention_f64() {
    assert_original_function_retention::<f64>();
}

fn assert_cyclic_percentage_function_requalification<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let calculation = SizingCalculationOf::value(
        crate::LengthPercentageOf::from_coefficients(S::ZERO, scalar(0.5)).unwrap(),
    );
    let tracks = [TrackSizingOf::minmax(
        MinTrackSizingOf::Calculation(calculation.clone()),
        MaxTrackSizingOf::Calculation(calculation.clone()),
    )];
    let original = GridMinimumTrackFunctions::new(&tracks, None);
    let initial = original.span_facts(S::ZERO).unwrap();
    assert!(initial.content_minimum_eligible);
    assert_eq!(initial.fixed_max_span(), None);

    // Only the original lender changes its own qualification phase. Its actual
    // final90 basis resolves50% to45; this is not a child's guessed allocation.
    let final_functions = original.requalified(Some(scalar(90.0)));
    let final_facts = final_functions.span_facts(S::ZERO).unwrap();
    assert!(!final_facts.content_minimum_eligible);
    assert_eq!(final_facts.fixed_max_span(), Some(scalar(45.0)));
    let retained = final_functions.retain().unwrap();
    let qualified = GridMinimumTrackFunctions::from_qualified(&retained);
    assert_eq!(
        qualified
            .requalified(Some(scalar(50.0)))
            .span_facts(S::ZERO)
            .unwrap(),
        final_facts,
    );
    let parent = crate::grid::subgrid::GridMinimumParentFunctions {
        columns: Some(original),
        rows: Some(qualified),
    }
    .requalified(LogicalSizeOf::new(Some(scalar(90.0)), Some(scalar(50.0))));
    assert_eq!(
        parent.columns.unwrap().span_facts(S::ZERO).unwrap(),
        final_facts
    );
    assert_eq!(
        parent.rows.unwrap().span_facts(S::ZERO).unwrap(),
        final_facts
    );

    // An invalid definite basis is not the missing-basis Auto state.
    assert!(!crate::grid::tracks::track_calculation_is_cyclic_auto(
        &calculation,
        Some(S::INFINITY)
    ));
    assert!(matches!(
        original.requalified(Some(S::INFINITY)).span_facts(S::ZERO),
        Err(SizingResolutionError::Status(
            LengthResolutionStatus::InvalidNumeric { .. }
        )),
    ));
}

#[test]
fn cyclic_percentage_function_requalification_f32() {
    assert_cyclic_percentage_function_requalification::<f32>();
}

#[test]
fn cyclic_percentage_function_requalification_f64() {
    assert_cyclic_percentage_function_requalification::<f64>();
}
