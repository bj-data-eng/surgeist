use super::*;
use crate::{
    ChildOrder, ContextEffect, PaintLimits, PaintOptions, Participation, Position, ProfileMode,
    ZIndex,
};

fn facts(parent: Option<BoxId>, source: usize) -> BoxFacts {
    BoxFacts::new(
        parent,
        SourceIndex::new(source),
        ChildOrder::Source,
        Participation::Ordinary,
        Position::Static,
        ZIndex::Auto,
        ContextEffect::None,
    )
}

#[test]
fn draw_views_retain_exact_records_and_original_text_owner_across_lines() {
    let root = BoxId::new(0);
    let inline = BoxId::new(1);
    let boxes = [facts(None, 0), facts(Some(root), 1)];
    let box_paint = [
        BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None),
        BoxPaint::new(BoxPaintKind::Inline, FloatParticipation::None),
    ];
    let texts = [TextSource::new(inline, SourceIndex::new(0))];
    let lines = [
        PaintLine::new(FragmentId::new(0), 0),
        PaintLine::new(FragmentId::new(0), 1),
    ];
    // A native adapter can associate these distinct slots with run A/line 0 and
    // run B/line 0. Local producer line indices alone would alias these sources.
    let line_associations = [("run A", 0usize), ("run B", 0usize)];
    let fragments = [
        PaintFragment::box_fragment(root, None, None, 0),
        PaintFragment::box_fragment(inline, Some(FragmentId::new(0)), Some(LineId::new(0)), 0),
        PaintFragment::box_fragment(inline, Some(FragmentId::new(0)), Some(LineId::new(1)), 1),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(1), LineId::new(0), 0),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(2), LineId::new(1), 1),
    ];
    let primitives = [
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
        PaintPrimitive::fragment(FragmentId::new(3), FragmentPaintKind::Text, 0),
        PaintPrimitive::decoration(FragmentId::new(4), root, DecorationKind::Underline, 0),
        PaintPrimitive::fragment(FragmentId::new(4), FragmentPaintKind::Text, 0),
    ];
    let request = PaintRequest::new(
        &boxes,
        &box_paint,
        &texts,
        &lines,
        &fragments,
        &primitives,
        root,
    );
    let plan = crate::plan_paint_order(
        request,
        PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off),
    )
    .result
    .unwrap();

    assert!(std::ptr::eq(plan.request().boxes(), boxes.as_slice()));
    assert!(std::ptr::eq(
        plan.request().primitives(),
        primitives.as_slice()
    ));
    assert!(std::ptr::eq(plan.stacking().facts(), boxes.as_slice()));
    let mut events = plan.events();
    assert_eq!(events.len(), 6);
    let PaintOrderEvent::BeginContext(view) = events.next().unwrap() else {
        panic!("context begins");
    };
    assert_eq!(view.id(), root);
    assert!(std::ptr::eq(view.facts(), &boxes[0]));
    assert!(std::ptr::eq(view.paint(), &box_paint[0]));
    let PaintOrderEvent::Draw(canvas) = events.next().unwrap() else {
        panic!("canvas draw");
    };
    assert_eq!(canvas.owner().id(), root);
    assert!(canvas.fragment().is_none());
    assert!(std::ptr::eq(canvas.record(), &primitives[0]));

    let mut associations = Vec::new();
    for expected in [1, 2, 3] {
        let PaintOrderEvent::Draw(draw) = events.next().unwrap() else {
            panic!("fragment draw");
        };
        assert_eq!(draw.id(), PrimitiveId::new(expected));
        assert!(std::ptr::eq(draw.record(), &primitives[expected]));
        assert_eq!(draw.owner().id(), inline);
        assert_eq!(draw.owner().parent().unwrap().id(), root);
        let fragment = draw.fragment().unwrap();
        let text = fragment.text_source().unwrap();
        assert!(std::ptr::eq(text, &texts[0]));
        assert_eq!(fragment.owner().id(), inline);
        let line = fragment.line().unwrap();
        assert!(std::ptr::eq(line.record(), &lines[line.id().get()]));
        assert_eq!(line.container().id(), FragmentId::new(0));
        assert_eq!(fragment.parent().unwrap().owner().id(), inline);
        assert!(std::ptr::eq(
            fragment.record(),
            &fragments[fragment.id().get()]
        ));
        associations.push(line_associations[line.id().get()]);
        if expected == 2 {
            assert_eq!(draw.applying_box().unwrap().id(), root);
        } else {
            assert!(draw.applying_box().is_none());
        }
    }
    assert_eq!(associations, [("run A", 0), ("run B", 0), ("run B", 0)]);
    let PaintOrderEvent::EndContext(view) = events.next().unwrap() else {
        panic!("context ends");
    };
    assert_eq!(view.id(), root);
    assert_eq!(events.len(), 0);
    assert!(events.next().is_none());
    assert!(plan.box_view(BoxId::new(2)).is_none());
    assert!(plan.fragment_view(FragmentId::new(5)).is_none());
    assert!(plan.line_view(LineId::new(2)).is_none());
    assert!(plan.primitive_view(PrimitiveId::new(4)).is_none());
}

#[test]
fn raw_request_preserves_forward_references_without_claiming_validation() {
    const FRAGMENT: PaintFragment =
        PaintFragment::text_fragment(TextId::new(17), FragmentId::new(31), LineId::new(9), 4);
    const PRIMITIVE: PaintPrimitive = PaintPrimitive::decoration(
        FragmentId::new(23),
        BoxId::new(41),
        DecorationKind::LineThrough,
        7,
    );
    let fragments = [FRAGMENT];
    let primitives = [PRIMITIVE];
    let request = PaintRequest::new(&[], &[], &[], &[], &fragments, &primitives, BoxId::new(8));
    assert_eq!(request.root(), BoxId::new(8));
    assert_eq!(
        request.fragments()[0].owner(),
        FragmentOwner::Text(TextId::new(17))
    );
    assert_eq!(request.fragments()[0].parent(), Some(FragmentId::new(31)));
    assert_eq!(request.fragments()[0].line(), Some(LineId::new(9)));
    assert_eq!(request.fragments()[0].logical_order(), 4);
    assert_eq!(
        request.primitives()[0].fragment_id(),
        Some(FragmentId::new(23))
    );
    assert_eq!(
        request.primitives()[0].kind(),
        PrimitiveKind::Decoration(DecorationKind::LineThrough)
    );
    assert_eq!(request.primitives()[0].applying_box(), Some(BoxId::new(41)));
    assert_eq!(request.primitives()[0].record_order(), 7);
}

#[test]
fn promoted_block_preserves_distinct_container_and_owner_order_domains() {
    const PROMOTED: PaintFragment =
        PaintFragment::block_in_inline(BoxId::new(4), FragmentId::new(7), 5, 23);
    let fragments = [PROMOTED];
    let request = PaintRequest::new(&[], &[], &[], &[], &fragments, &[], BoxId::new(0));
    let retained = &request.fragments()[0];
    assert!(std::ptr::eq(retained, &fragments[0]));
    assert_eq!(retained.owner(), FragmentOwner::Box(BoxId::new(4)));
    assert_eq!(retained.parent(), Some(FragmentId::new(7)));
    assert_eq!(retained.line(), None);
    assert_eq!(retained.logical_order(), 23);
    let placement = retained.block_in_inline_placement().unwrap();
    assert_eq!(placement.container(), FragmentId::new(7));
    assert_eq!(placement.content_order(), 5);
    assert_eq!(
        placement,
        BlockInInlinePlacement::new(FragmentId::new(7), 5)
    );

    let direct = PaintFragment::box_fragment(BoxId::new(4), Some(FragmentId::new(7)), None, 23);
    assert_eq!(direct.block_in_inline_placement(), None);
    let text = PaintFragment::text_fragment(TextId::new(0), FragmentId::new(7), LineId::new(0), 0);
    assert_eq!(text.block_in_inline_placement(), None);
}
