use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

fn input<S: LayoutScalar>(parent: f64) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(60.0)), None),
        Size::new(Some(S::from_f64(parent)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(parent)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}

fn tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .style(0, NodeInputOf::default())
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                ..NodeInputOf::default()
            },
        )
        .style(2, NodeInputOf::default())
}

fn reservations<S: LayoutScalar>() {
    let tree = tree::<S>();
    let (graph, _) =
        crate::engine::validation::validate_layout_request_with_text_graph(&tree, 0, &[], &[])
            .unwrap();
    let sources = Arc::new(Mutex::new(SourceMemo::new(None)));
    let lookup = |request| {
        SourceMemo::float_observation(&sources, &tree, &graph, 1, request, CacheKeyContext::new())
            .unwrap()
            .unwrap()
    };
    let FloatObservation::Miss(outer) = lookup(input(100.0)) else {
        panic!("first request must run")
    };
    let FloatObservation::Miss(inner) = lookup(input(100.0)) else {
        panic!("pending request cannot be reused")
    };
    drop(outer);
    let expected = ComputeOutputOf::from_sizes(
        Size::new(S::from_f64(60.0), S::from_f64(7.0)),
        Size::new(S::from_f64(60.0), S::from_f64(9.0)),
    );
    inner.complete(expected);
    let FloatObservation::Hit(output) = lookup(input(100.0)) else {
        panic!("completed nested request remains reusable")
    };
    assert_eq!(output.size, expected.size);
    assert_eq!(output.content_size, expected.content_size);
    let FloatObservation::Miss(changed) = lookup(input(80.0)) else {
        panic!("different parent and offer require their producer")
    };
    drop(changed);
    assert!(matches!(lookup(input(100.0)), FloatObservation::Hit(_)));
    let FloatObservation::Miss(zero) = lookup(input(0.0)) else {
        panic!("new raw basis")
    };
    zero.complete(expected);
    assert!(
        matches!(lookup(input(-0.0)), FloatObservation::Miss(_)),
        "raw negative zero in the parent basis must remain distinct"
    );
    let fresh = Arc::new(Mutex::new(SourceMemo::new(None)));
    assert!(matches!(
        SourceMemo::float_observation(
            &fresh,
            &tree,
            &graph,
            1,
            input(100.0),
            CacheKeyContext::new()
        )
        .unwrap(),
        Some(FloatObservation::Miss(_))
    ));
}

#[test]
fn entry_source_pending_drop_and_exact_namespace_f32() {
    reservations::<f32>();
}
#[test]
fn entry_source_pending_drop_and_exact_namespace_f64() {
    reservations::<f64>();
}
