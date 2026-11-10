//! Floats retain child margins while ordinary shared blocks may collapse them.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MarginGeometry<S: LayoutScalar> {
    container_top: S,
    container_height: S,
    child_top: S,
    root_height: S,
}

fn expected<S: LayoutScalar>(values: (f64, f64, f64, f64)) -> MarginGeometry<S> {
    MarginGeometry {
        container_top: S::from_f64(values.0),
        container_height: S::from_f64(values.1),
        child_top: S::from_f64(values.2),
        root_height: S::from_f64(values.3),
    }
}

fn block<S: LayoutScalar>(width: f64, height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            height.map_or(PreferredSizeOf::AUTO, |height| {
                PreferredSizeOf::px(S::from_f64(height))
            }),
        ),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    }
}

fn observe<S: LayoutScalar>(
    top: f64,
    height: f64,
    bottom: f64,
    float: Float,
    context: BlockFlowContext,
) -> MarginGeometry<S> {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(0, block::<S>(100.0, None))
        .style(
            1,
            NodeInputOf {
                float,
                block_flow_context: context,
                ..block(40.0, None)
            },
        )
        .style(
            2,
            NodeInputOf {
                margin: Edges {
                    top: LengthAutoOf::px(S::from_f64(top)),
                    bottom: LengthAutoOf::px(S::from_f64(bottom)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..block(20.0, Some(height))
            },
        );
    let width = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    let batch = compute_layout(&tree, 0, request).unwrap();
    let root = single_box_output(&batch, 0, Phase::Unrounded);
    let container = single_box_output(&batch, 1, Phase::Unrounded);
    let child = single_box_output(&batch, 2, Phase::Unrounded);
    assert_eq!(root.size.width, S::from_f64(100.0));
    assert_eq!(container.size.width, S::from_f64(40.0));
    assert_eq!(
        child.size,
        Size::new(S::from_f64(20.0), S::from_f64(height))
    );
    assert_eq!(child.location.x, S::ZERO);
    assert_eq!(
        container.location.x,
        S::from_f64(if float == Float::Right { 60.0 } else { 0.0 })
    );
    MarginGeometry {
        container_top: container.location.y,
        container_height: container.size.height,
        child_top: child.location.y,
        root_height: root.size.height,
    }
}

fn qualify<S: LayoutScalar>(
    top: f64,
    height: f64,
    bottom: f64,
    shared: (f64, f64, f64, f64),
    contained: (f64, f64, f64, f64),
) {
    // These actual public computations establish the eligible shared collapse
    // and the separate explicit-context barrier before testing Float/Normal.
    assert_eq!(
        observe::<S>(top, height, bottom, Float::None, BlockFlowContext::Normal),
        expected(shared),
        "ordinary Normal control"
    );
    for side in [Float::None, Float::Left, Float::Right] {
        assert_eq!(
            observe::<S>(top, height, bottom, side, BlockFlowContext::Independent),
            expected(contained),
            "explicit Independent control: {side:?}"
        );
    }
    // Read both sides before asserting the paired result. A failing left
    // result must not prevent execution or observation of the right float.
    let actual = [Float::Left, Float::Right].map(|side| {
        (
            side,
            observe::<S>(top, height, bottom, side, BlockFlowContext::Normal),
        )
    });
    assert_eq!(
        actual,
        [
            (Float::Left, expected(contained)),
            (Float::Right, expected(contained)),
        ],
        "Float itself supplies the child-margin barrier"
    );
}

fn positive<S: LayoutScalar>() {
    // With no parent/child collapse, the child's border starts10 inside the
    // float and its bottom margin ends at10+10+7=27. Ordinary shared context
    // instead exports10/7 around its10px box, preserving the same root extent.
    qualify::<S>(
        10.0,
        10.0,
        7.0,
        (10.0, 10.0, 0.0, 27.0),
        (0.0, 27.0, 10.0, 27.0),
    );
}

fn signed<S: LayoutScalar>() {
    // The negative top margin moves only the child's border to-4. It is not
    // clamped: the contained automatic extent is-4+10+7=13.
    qualify::<S>(
        -4.0,
        10.0,
        7.0,
        (-4.0, 10.0, 0.0, 13.0),
        (0.0, 13.0, -4.0, 13.0),
    );
}

fn empty<S: LayoutScalar>() {
    // CSS2 box §8.3.1 permits this empty child's OWN adjoining10/7 margins
    // to collapse to10. The float remains outside that collapse. Under the
    // collapsed-through position rule, a nonparticipating parent places the
    // child's border as if its bottom border were nonzero: child localy10.
    // That border position does not add an extra bottom7 to the group extent.
    qualify::<S>(
        10.0,
        0.0,
        7.0,
        (10.0, 0.0, 0.0, 10.0),
        (0.0, 10.0, 10.0, 10.0),
    );
}

#[test]
fn floated_normal_contains_positive_child_margins_f32() {
    positive::<f32>();
}
#[test]
fn floated_normal_contains_positive_child_margins_f64() {
    positive::<f64>();
}
#[test]
fn floated_normal_retains_signed_child_margins_f32() {
    signed::<f32>();
}
#[test]
fn floated_normal_retains_signed_child_margins_f64() {
    signed::<f64>();
}
#[test]
fn floated_normal_preserves_empty_child_own_margin_collapse_f32() {
    empty::<f32>();
}
#[test]
fn floated_normal_preserves_empty_child_own_margin_collapse_f64() {
    empty::<f64>();
}
