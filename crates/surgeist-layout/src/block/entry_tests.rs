use super::*;
use crate::engine::ComputeSession;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use std::cell::Cell;

thread_local! {
    static MEASUREMENTS: Cell<usize> = const { Cell::new(0) };
}
fn measured_zero<S: LayoutScalar>(_: crate::LeafMeasureInputOf<S>) -> Size<S> {
    MEASUREMENTS.with(|value| value.set(value.get() + 1));
    Size::ZERO
}
fn input<S: LayoutScalar>() -> ComputeInputOf<S> {
    let axis = crate::AxisSizingRequestOf::new(
        crate::AvailableSpaceOf::Indefinite,
        crate::SizingOperation::Normal,
    );
    ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::from_f64(100.0)), None),
        Size::new(Some(S::from_f64(100.0)), None),
        ContainingLayoutContext::new(
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        crate::SizingRequestOf::new(
            Size::splat(axis),
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}
fn first_margin<S: LayoutScalar>() {
    for (measured, separating, expected) in
        [(true, false, 5.0), (false, false, 5.0), (true, true, 11.0)]
    {
        let mut owner = crate::NodeInputOf::<S> {
            display: crate::Display::Block,
            ..Default::default()
        };
        owner.margin.top = LengthAutoOf::px(S::from_f64(11.0));
        if separating {
            owner.padding.top = crate::LengthOf::px(S::from_f64(10.0));
        }
        let first = crate::NodeInputOf::<S> {
            display: crate::Display::Block,
            margin: Edges::new(
                LengthAutoOf::px(S::from_f64(5.0)),
                LengthAutoOf::ZERO,
                LengthAutoOf::ZERO,
                LengthAutoOf::ZERO,
            ),
            ..Default::default()
        };
        let later = crate::NodeInputOf::<S> {
            display: crate::Display::Block,
            size: Size::new(
                crate::PreferredSizeOf::px(S::from_f64(10.0)),
                crate::PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            margin: Edges::new(
                LengthAutoOf::px(S::from_f64(29.0)),
                LengthAutoOf::ZERO,
                LengthAutoOf::ZERO,
                LengthAutoOf::ZERO,
            ),
            ..Default::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .style(0, owner)
            .style(1, first)
            .style(2, later)
            .children(0, [1, 2]);
        let tree = if measured {
            tree.measure_with(1, measured_zero::<S>)
        } else {
            tree
        };
        let mut session = ComputeSession::new(&tree, Vec::new());
        MEASUREMENTS.with(|value| value.set(0));
        let Qualification::Known(fact) =
            qualify(&mut session, 0, input(), PhysicalSide::Top).unwrap()
        else {
            panic!("same-progression Block entry is independently qualified");
        };
        assert_eq!(fact.margin.resolve(), S::from_f64(expected));
        MEASUREMENTS.with(|value| assert_eq!(value.get(), 0));
        let body = session.compute_child(0, input()).unwrap();
        assert_eq!(
            body.block_margin_collapse.at(crate::PhysicalSide::Top),
            fact.margin
        );
        if measured {
            let Qualification::Known(leaf_start) =
                qualify(&mut session, 1, input(), PhysicalSide::Top).unwrap()
            else {
                panic!("measured leaf reports its start set without measurement")
            };
            assert_eq!(leaf_start.margin, CollapsibleMarginOf::ZERO);
            let leaf = session.compute_child(1, input()).unwrap();
            assert!(
                leaf.block_margin_collapse
                    .can_collapse_through(crate::FlowAxes::new(
                        crate::WritingMode::HorizontalTb,
                        Direction::Ltr
                    ))
            );
        }
    }
    let mut last = crate::NodeInputOf::<S> {
        display: crate::Display::Block,
        size: Size::new(
            crate::PreferredSizeOf::AUTO,
            crate::PreferredSizeOf::px(S::from_f64(10.0)),
        ),
        ..Default::default()
    };
    last.margin.bottom = LengthAutoOf::px(S::from_f64(13.0));
    let tree = PublicLayoutTreeOf::new()
        .style(
            0,
            crate::NodeInputOf::<S> {
                display: crate::Display::Block,
                ..Default::default()
            },
        )
        .style(1, last)
        .children(0, [1]);
    let mut session = ComputeSession::new(&tree, Vec::new());
    assert_eq!(
        qualify(&mut session, 0, input(), PhysicalSide::Bottom).unwrap(),
        Qualification::RequiresBody
    );
    let actual = session.compute_child(0, input()).unwrap();
    assert_eq!(
        actual
            .block_margin_collapse
            .at(PhysicalSide::Bottom)
            .resolve(),
        S::from_f64(13.0)
    );
    let start = reduce_start(
        false,
        false,
        true,
        CollapsibleMarginOf::<S>::from_margin(S::from_f64(4.0)),
        CollapsibleMarginOf::from_margin(S::from_f64(-9.0)),
    );
    assert_eq!(start.offset, S::from_f64(-5.0));
}
#[test]
fn first_start_fact_does_not_measure_or_accumulate_later_children_f32() {
    first_margin::<f32>();
}
#[test]
fn first_start_fact_does_not_measure_or_accumulate_later_children_f64() {
    first_margin::<f64>();
}
