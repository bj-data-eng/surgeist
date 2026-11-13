use super::*;
use crate::{
    AtomicInlineParticipation, BaselineName, InlineBaselineContext, InlineBreakOpportunity,
    LayoutErrorKindOf, LayoutErrorSiteOf, LayoutMissingContext, LayoutOperation, MinSizeOf,
    PreferredSizeOf, VerticalAlignOf,
};

fn named_receiver_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let preferred = |value| PreferredSizeOf::px(S::from_f64(value));
    let named = InlineBaselineContext::Named(BaselineName::Alphabetic);
    let atomic = |alignment, following_break| NodeInputOf {
        display: Display::InlineBlock,
        atomic_inline_participation: Some(AtomicInlineParticipation::new(
            surgeist_text::BidiLevel::try_new(0).unwrap(),
            following_break,
        )),
        inline_baseline_context: named,
        vertical_align: alignment,
        size: Size::new(preferred(13.0), preferred(7.0)),
        ..NodeInputOf::default()
    };
    let first_sibling = 3 * depth;
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, first_sibling])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                inline_baseline_context: named,
                size: Size::new(preferred(13.0), preferred(80.0)),
                ..NodeInputOf::default()
            },
        )
        .style(
            first_sibling,
            atomic(VerticalAlignOf::Baseline, InlineBreakOpportunity::Allowed),
        );
    for layer in 0..depth {
        let child = 3 * layer + 1;
        tree = tree.style(
            child,
            atomic(VerticalAlignOf::Top, InlineBreakOpportunity::Mandatory),
        );
        if layer + 1 < depth {
            let sibling = first_sibling + layer + 1;
            tree = tree
                .children(child, [child + 1])
                .children(child + 1, [child + 2])
                .children(child + 2, [child + 3, sibling])
                .style(
                    sibling,
                    atomic(VerticalAlignOf::Baseline, InlineBreakOpportunity::Allowed),
                )
                .style(
                    child + 1,
                    NodeInputOf {
                        display: Display::Grid,
                        size: Size::new(preferred(13.0), PreferredSizeOf::AUTO),
                        min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                        align_items: Some(crate::AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    child + 2,
                    NodeInputOf {
                        display: Display::Block,
                        inline_baseline_context: named,
                        size: Size::new(preferred(13.0), preferred(7.0)),
                        ..NodeInputOf::default()
                    },
                );
        }
    }
    tree
}

fn assert_terminal_receiver<S: LayoutScalar>() {
    let tree = named_receiver_chain::<S>(1);
    let batch =
        compute_layout(&tree, 0, standalone()).expect("empty atomic has an absent baseline");
    assert_eq!(batch.node_layouts().len(), 3);
    for (node, height, top, source) in [(0, 80.0, 0.0, 0), (1, 7.0, 0.0, 0), (3, 7.0, 7.0, 1)] {
        let output = batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .unwrap()
            .final_output();
        assert_eq!(
            output.size,
            Size::new(S::from_f64(13.0), S::from_f64(height))
        );
        assert_eq!(
            output.location,
            crate::Point::new(S::ZERO, S::from_f64(top))
        );
        assert_eq!(output.source_index, crate::SourceIndex::new(source));
    }
}

fn assert_named_refusal<S: LayoutScalar>(depth: u32) {
    let tree = named_receiver_chain::<S>(depth);
    let error =
        compute_layout(&tree, 0, standalone()).expect_err("Grid supplies no named baseline");
    // The terminal empty atom supplies Absent. The nearest enclosing atom
    // contains a Grid whose exported named fact remains Unknown, so this
    // actual receiving line requires a typed refusal at that source box.
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(3 * (depth - 2) + 1));
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::NamedInlineBaseline)
    ));
}

#[test]
fn shallow_named_receiver_preserves_geometry_and_refusal() {
    on_bounded_stack(
        "engine::depth_tests::named_inline::shallow_named_receiver_preserves_geometry_and_refusal",
        || {
            assert_terminal_receiver::<f32>();
            assert_terminal_receiver::<f64>();
            assert_named_refusal::<f32>(3);
            assert_named_refusal::<f64>(3);
        },
    );
}

#[test]
fn deep_named_receiver_returns_typed_refusal_f32() {
    on_bounded_stack(
        "engine::depth_tests::named_inline::deep_named_receiver_returns_typed_refusal_f32",
        || assert_named_refusal::<f32>(24),
    );
}

#[test]
fn deep_named_receiver_returns_typed_refusal_f64() {
    on_bounded_stack(
        "engine::depth_tests::named_inline::deep_named_receiver_returns_typed_refusal_f64",
        || assert_named_refusal::<f64>(24),
    );
}
