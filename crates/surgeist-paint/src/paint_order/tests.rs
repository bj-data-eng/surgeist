use super::*;
use crate::ResourceKind;
use crate::{
    BoxFacts, BoxId, ChildOrder, ContextEffect, PaintLimits, PaintReport, Participation, Position,
    ProfileMode, ZIndex,
};
use surgeist_layout::SourceIndex;

fn root() -> BoxFacts {
    BoxFacts::new(
        None,
        SourceIndex::new(0),
        ChildOrder::Source,
        Participation::Ordinary,
        Position::Static,
        ZIndex::Auto,
        ContextEffect::None,
    )
}

fn options() -> PaintOptions {
    PaintOptions::new(
        PaintLimits::unlimited(),
        ProfileMode::Summary { timing: false },
    )
}

fn child(parent: usize, source: usize, position: Position, z: ZIndex) -> BoxFacts {
    BoxFacts::new(
        Some(BoxId::new(parent)),
        SourceIndex::new(source),
        ChildOrder::Source,
        Participation::Ordinary,
        position,
        z,
        ContextEffect::None,
    )
}

fn paint(kind: BoxPaintKind) -> BoxPaint {
    BoxPaint::new(kind, FloatParticipation::None)
}
fn fragment(owner: usize, parent: usize, line: Option<usize>) -> PaintFragment {
    PaintFragment::box_fragment(
        BoxId::new(owner),
        Some(FragmentId::new(parent)),
        line.map(LineId::new),
        0,
    )
}
fn color(fragment: usize) -> PaintPrimitive {
    PaintPrimitive::fragment(
        FragmentId::new(fragment),
        FragmentPaintKind::BackgroundColor,
        0,
    )
}
fn text(fragment: usize) -> PaintPrimitive {
    PaintPrimitive::fragment(FragmentId::new(fragment), FragmentPaintKind::Text, 0)
}

fn events(plan: &PaintOrder<'_>) -> Vec<(char, usize)> {
    plan.events()
        .map(|event| match event {
            PaintOrderEvent::BeginContext(owner) => ('[', owner.id().get()),
            PaintOrderEvent::Draw(primitive) => ('d', primitive.id().get()),
            PaintOrderEvent::EndContext(owner) => (']', owner.id().get()),
        })
        .collect()
}

#[test]
fn same_length_requests_retain_their_distinct_root_order_and_payloads() {
    let first = [
        root(),
        child(0, 0, Position::Relative, ZIndex::Integer(-2)),
        child(0, 1, Position::Relative, ZIndex::Integer(3)),
    ];
    let second = [
        child(2, 1, Position::Relative, ZIndex::Integer(4)),
        child(2, 0, Position::Relative, ZIndex::Integer(-3)),
        root(),
    ];
    let paints = [paint(BoxPaintKind::Block); 3];
    for (boxes, root_id, owners, expected) in [
        (
            &first,
            0,
            [0, 1, 2],
            [
                ('[', 0),
                ('d', 0),
                ('[', 1),
                ('d', 1),
                (']', 1),
                ('[', 2),
                ('d', 2),
                (']', 2),
                (']', 0),
            ],
        ),
        (
            &second,
            2,
            [2, 0, 1],
            [
                ('[', 2),
                ('d', 0),
                ('[', 1),
                ('d', 2),
                (']', 1),
                ('[', 0),
                ('d', 1),
                (']', 0),
                (']', 2),
            ],
        ),
    ] {
        let fragments = [
            PaintFragment::box_fragment(BoxId::new(owners[0]), None, None, 0),
            fragment(owners[1], 0, None),
            fragment(owners[2], 0, None),
        ];
        let primitives = [
            PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
            color(1),
            color(2),
        ];
        let request = PaintRequest::new(
            boxes,
            &paints,
            &[],
            &[],
            &fragments,
            &primitives,
            BoxId::new(root_id),
        );
        let plan = plan_paint_order(request, options()).result.unwrap();
        assert_eq!(events(&plan), expected);
        assert_eq!(plan.request().root(), BoxId::new(root_id));
        assert_eq!(plan.stacking().root_context().box_id(), BoxId::new(root_id));
        assert!(std::ptr::eq(plan.request().boxes(), boxes.as_slice()));
        assert!(std::ptr::eq(plan.stacking().facts(), boxes.as_slice()));
        assert!(std::ptr::eq(
            plan.request().fragments(),
            fragments.as_slice()
        ));
        assert!(std::ptr::eq(
            plan.request().primitives(),
            primitives.as_slice()
        ));
        for event in plan.events() {
            if let PaintOrderEvent::Draw(draw) = event {
                assert!(std::ptr::eq(draw.record(), &primitives[draw.id().get()]));
                let owner = owners[draw.id().get()];
                assert_eq!(draw.owner().id(), BoxId::new(owner));
                assert!(std::ptr::eq(draw.owner().facts(), &boxes[owner]));
            }
        }
    }
}

#[test]
fn empty_root_has_one_balanced_context_and_no_fabricated_draw() {
    let boxes = [root()];
    let paint = [BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None)];
    let outcome = plan_paint_order(
        PaintRequest::new(&boxes, &paint, &[], &[], &[], &[], BoxId::new(0)),
        options(),
    );
    assert_eq!(events(&outcome.result.unwrap()), [('[', 0), (']', 0)]);
    assert_eq!(outcome.report.unwrap().live(ResourceKind::ScratchBytes), 0);
}

#[test]
fn canvas_appearance_and_root_border_precede_the_late_outline() {
    let boxes = [root()];
    let paint = [BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None)];
    let fragments = [PaintFragment::box_fragment(BoxId::new(0), None, None, 0)];
    let primitives = [
        PaintPrimitive::fragment(FragmentId::new(0), FragmentPaintKind::Outline, 0),
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundImages, 0),
        PaintPrimitive::fragment(FragmentId::new(0), FragmentPaintKind::Border, 0),
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
    ];
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paint,
            &[],
            &[],
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    assert_eq!(
        events(&outcome.result.unwrap()),
        [('[', 0), ('d', 3), ('d', 1), ('d', 2), ('d', 0), (']', 0)]
    );
}

#[test]
fn every_paint_band_uses_its_semantic_order_and_only_actual_context_boundaries() {
    let boxes = [
        root(),
        child(0, 0, Position::Relative, ZIndex::Integer(-2)),
        child(0, 1, Position::Static, ZIndex::Auto),
        child(0, 2, Position::Static, ZIndex::Auto),
        child(0, 3, Position::Static, ZIndex::Auto),
        child(0, 4, Position::Relative, ZIndex::Auto),
        child(0, 5, Position::Relative, ZIndex::Integer(0)),
        child(0, 6, Position::Relative, ZIndex::Integer(3)),
    ];
    let mut paints = [paint(BoxPaintKind::Block); 8];
    paints[3] = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating);
    paints[4] = paint(BoxPaintKind::Inline);
    let fragments = [
        PaintFragment::box_fragment(BoxId::new(0), None, None, 0),
        fragment(1, 0, None),
        fragment(2, 0, None),
        fragment(3, 0, None),
        fragment(4, 0, Some(0)),
        fragment(5, 0, None),
        fragment(6, 0, None),
        fragment(7, 0, None),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(4), LineId::new(0), 0),
    ];
    let lines = [PaintLine::new(FragmentId::new(0), 0)];
    let texts = [TextSource::new(BoxId::new(4), SourceIndex::new(0))];
    let primitives = [
        PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
        color(1),
        color(2),
        color(3),
        color(4),
        color(5),
        color(6),
        color(7),
        text(8),
        PaintPrimitive::fragment(FragmentId::new(0), FragmentPaintKind::Outline, 0),
    ];
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paints,
            &texts,
            &lines,
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    assert_eq!(
        events(&outcome.result.unwrap()),
        [
            ('[', 0),
            ('d', 0),
            ('[', 1),
            ('d', 1),
            (']', 1),
            ('d', 2),
            ('d', 3),
            ('d', 4),
            ('d', 8),
            ('d', 5),
            ('[', 6),
            ('d', 6),
            (']', 6),
            ('[', 7),
            ('d', 7),
            (']', 7),
            ('d', 9),
            (']', 0)
        ]
    );
}

#[test]
fn positioned_descendants_escape_nested_float_and_atomic_inline_pseudo_scopes() {
    let boxes = [
        root(),
        child(0, 0, Position::Static, ZIndex::Auto),
        child(1, 0, Position::Static, ZIndex::Auto),
        child(2, 1, Position::Relative, ZIndex::Auto),
        child(3, 0, Position::Relative, ZIndex::Integer(-1)),
        child(3, 1, Position::Absolute, ZIndex::Auto),
        child(3, 2, Position::Relative, ZIndex::Integer(0)),
        child(6, 0, Position::Relative, ZIndex::Integer(99)),
    ];
    let mut paints = [paint(BoxPaintKind::Block); 8];
    paints[1] = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating);
    paints[2] = paint(BoxPaintKind::InlineBlock);
    let fragments = [
        PaintFragment::box_fragment(BoxId::new(0), None, None, 0),
        fragment(1, 0, None),
        fragment(2, 1, Some(0)),
        fragment(3, 2, None),
        fragment(4, 3, None),
        fragment(5, 3, None),
        fragment(6, 3, None),
        fragment(7, 6, None),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(2), LineId::new(1), 0),
    ];
    let lines = [
        PaintLine::new(FragmentId::new(1), 0),
        PaintLine::new(FragmentId::new(2), 0),
    ];
    let texts = [TextSource::new(BoxId::new(2), SourceIndex::new(0))];
    let primitives = [
        color(1),
        color(2),
        color(3),
        color(4),
        color(5),
        color(6),
        color(7),
        text(8),
    ];
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paints,
            &texts,
            &lines,
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    assert_eq!(
        events(&outcome.result.unwrap()),
        [
            ('[', 0),
            ('[', 4),
            ('d', 3),
            (']', 4),
            ('d', 0),
            ('d', 1),
            ('d', 7),
            ('d', 2),
            ('d', 4),
            ('[', 6),
            ('d', 5),
            ('[', 7),
            ('d', 6),
            (']', 7),
            (']', 6),
            (']', 0)
        ]
    );
}

#[test]
fn split_inline_context_keeps_promoted_block_background_and_foreground_between_lines() {
    let boxes = [
        root(),
        child(0, 1, Position::Relative, ZIndex::Integer(2)),
        child(1, 1, Position::Static, ZIndex::Auto),
        child(0, 0, Position::Relative, ZIndex::Integer(1)),
    ];
    let paints = [
        paint(BoxPaintKind::Block),
        paint(BoxPaintKind::Inline),
        paint(BoxPaintKind::Block),
        paint(BoxPaintKind::Block),
    ];
    let fragments = [
        PaintFragment::box_fragment(BoxId::new(0), None, None, 0),
        fragment(1, 0, Some(0)),
        PaintFragment::box_fragment(
            BoxId::new(1),
            Some(FragmentId::new(0)),
            Some(LineId::new(2)),
            1,
        ),
        PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(0), 1, 0),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(1), LineId::new(0), 0),
        PaintFragment::text_fragment(TextId::new(1), FragmentId::new(2), LineId::new(2), 0),
        PaintFragment::text_fragment(TextId::new(2), FragmentId::new(3), LineId::new(1), 0),
        fragment(3, 0, None),
    ];
    let lines = [
        PaintLine::new(FragmentId::new(0), 0),
        PaintLine::new(FragmentId::new(3), 0),
        PaintLine::new(FragmentId::new(0), 2),
    ];
    let texts = [
        TextSource::new(BoxId::new(1), SourceIndex::new(0)),
        TextSource::new(BoxId::new(1), SourceIndex::new(2)),
        TextSource::new(BoxId::new(2), SourceIndex::new(0)),
    ];
    let primitives = [
        color(7),
        color(3),
        color(1),
        text(4),
        color(2),
        text(5),
        text(6),
        PaintPrimitive::fragment(FragmentId::new(1), FragmentPaintKind::Outline, 0),
        PaintPrimitive::fragment(FragmentId::new(2), FragmentPaintKind::Outline, 0),
        PaintPrimitive::fragment(FragmentId::new(3), FragmentPaintKind::Outline, 0),
    ];
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paints,
            &texts,
            &lines,
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    let plan = outcome.result.unwrap();
    assert_eq!(
        events(&plan),
        [
            ('[', 0),
            ('[', 3),
            ('d', 0),
            (']', 3),
            ('[', 1),
            ('d', 1),
            ('d', 2),
            ('d', 3),
            ('d', 6),
            ('d', 4),
            ('d', 5),
            ('d', 7),
            ('d', 8),
            ('d', 9),
            (']', 1),
            (']', 0)
        ]
    );
    assert_eq!(plan.request().boxes()[2].parent(), Some(BoxId::new(1)));
    assert_eq!(
        plan.fragment_view(FragmentId::new(3))
            .unwrap()
            .record()
            .parent(),
        Some(FragmentId::new(0))
    );
}

#[test]
fn decoration_categories_and_applying_ancestor_order_are_independent_of_table_order() {
    let boxes = [root(), child(0, 0, Position::Static, ZIndex::Auto)];
    let paints = [paint(BoxPaintKind::Block), paint(BoxPaintKind::Inline)];
    let texts = [TextSource::new(BoxId::new(1), SourceIndex::new(0))];
    let lines = [PaintLine::new(FragmentId::new(0), 0)];
    let fragments = [
        PaintFragment::box_fragment(BoxId::new(0), None, None, 0),
        fragment(1, 0, Some(0)),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(1), LineId::new(0), 0),
    ];
    let decoration =
        |owner, kind| PaintPrimitive::decoration(FragmentId::new(2), BoxId::new(owner), kind, 0);
    let primitives = [
        decoration(1, DecorationKind::LineThrough),
        text(2),
        decoration(1, DecorationKind::Underline),
        decoration(0, DecorationKind::Overline),
        decoration(0, DecorationKind::Underline),
        decoration(0, DecorationKind::LineThrough),
        decoration(1, DecorationKind::Overline),
        color(1),
        PaintPrimitive::fragment(FragmentId::new(1), FragmentPaintKind::Outline, 0),
        PaintPrimitive::fragment(FragmentId::new(0), FragmentPaintKind::Outline, 0),
    ];
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paints,
            &texts,
            &lines,
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    assert_eq!(
        events(&outcome.result.unwrap()),
        [
            ('[', 0),
            ('d', 7),
            ('d', 4),
            ('d', 2),
            ('d', 3),
            ('d', 6),
            ('d', 1),
            ('d', 5),
            ('d', 0),
            ('d', 9),
            ('d', 8),
            (']', 0)
        ]
    );
}

#[test]
fn atomic_occurrences_select_their_own_text_while_float_custody_invokes_once() {
    let boxes = [
        root(),
        child(0, 0, Position::Static, ZIndex::Auto),
        child(1, 0, Position::Static, ZIndex::Auto),
    ];
    let paints = [
        paint(BoxPaintKind::Block),
        BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating),
        paint(BoxPaintKind::InlineBlock),
    ];
    let texts = [TextSource::new(BoxId::new(2), SourceIndex::new(0))];
    let fragments = [
        PaintFragment::box_fragment(BoxId::new(0), None, None, 0),
        fragment(1, 0, None),
        PaintFragment::box_fragment(BoxId::new(1), Some(FragmentId::new(5)), None, 1),
        fragment(2, 1, Some(0)),
        PaintFragment::box_fragment(
            BoxId::new(2),
            Some(FragmentId::new(2)),
            Some(LineId::new(1)),
            1,
        ),
        PaintFragment::box_fragment(BoxId::new(0), None, None, 1),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(3), LineId::new(2), 0),
        PaintFragment::text_fragment(TextId::new(0), FragmentId::new(4), LineId::new(3), 1),
    ];
    let lines = [
        PaintLine::new(FragmentId::new(1), 0),
        PaintLine::new(FragmentId::new(2), 0),
        PaintLine::new(FragmentId::new(3), 0),
        PaintLine::new(FragmentId::new(4), 0),
    ];
    let outline = |id| PaintPrimitive::fragment(FragmentId::new(id), FragmentPaintKind::Outline, 0);
    let primitives = [
        color(1),
        color(2),
        color(3),
        color(4),
        text(6),
        text(7),
        outline(3),
        outline(4),
        outline(1),
        outline(2),
        outline(0),
    ];
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paints,
            &texts,
            &lines,
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    assert_eq!(
        events(&outcome.result.unwrap()),
        [
            ('[', 0),
            ('d', 0),
            ('d', 1),
            ('d', 2),
            ('d', 4),
            ('d', 6),
            ('d', 3),
            ('d', 5),
            ('d', 7),
            ('d', 8),
            ('d', 9),
            ('d', 10),
            (']', 0)
        ]
    );
}

#[test]
fn deep_contexts_use_iterative_balanced_output_even_without_fragment_geometry() {
    let n = 2048;
    let mut boxes = vec![root()];
    for parent in 0..n - 1 {
        boxes.push(child(parent, 0, Position::Relative, ZIndex::Integer(0)));
    }
    let paints = vec![paint(BoxPaintKind::Block); n];
    let outcome = plan_paint_order(
        PaintRequest::new(&boxes, &paints, &[], &[], &[], &[], BoxId::new(0)),
        options(),
    );
    let plan = outcome.result.unwrap();
    let mut open = Vec::new();
    for event in plan.events() {
        match event {
            PaintOrderEvent::BeginContext(owner) => open.push(owner.id()),
            PaintOrderEvent::EndContext(owner) => assert_eq!(open.pop(), Some(owner.id())),
            PaintOrderEvent::Draw(_) => panic!("no input primitive exists"),
        }
    }
    assert!(open.is_empty());
    assert_eq!(plan.events().len(), 2 * n);
    assert_eq!(outcome.report.unwrap().live(ResourceKind::ScratchBytes), 0);
}

#[derive(Clone, Copy, Debug)]
enum BlockifiedRole {
    Float,
    Absolute,
    Flex,
    Grid,
}

fn category_case(
    kind: BoxPaintKind,
    role: BlockifiedRole,
) -> (Result<Vec<(char, usize)>, PaintError>, PaintReport) {
    let child_order = match role {
        BlockifiedRole::Flex => ChildOrder::Flex,
        BlockifiedRole::Grid => ChildOrder::Grid,
        _ => ChildOrder::Source,
    };
    let participation = match role {
        BlockifiedRole::Flex => Participation::FlexItem(surgeist_layout::ItemOrder::new(0)),
        BlockifiedRole::Grid => Participation::GridItem(surgeist_layout::ItemOrder::new(0)),
        _ => Participation::Ordinary,
    };
    let boxes = [
        BoxFacts::new(
            None,
            SourceIndex::ZERO,
            child_order,
            Participation::Ordinary,
            Position::Static,
            ZIndex::Auto,
            ContextEffect::None,
        ),
        BoxFacts::new(
            Some(BoxId::new(0)),
            SourceIndex::ZERO,
            ChildOrder::Source,
            participation,
            if matches!(role, BlockifiedRole::Absolute) {
                Position::Absolute
            } else {
                Position::Static
            },
            ZIndex::Auto,
            ContextEffect::None,
        ),
    ];
    let paints = [
        paint(BoxPaintKind::Block),
        BoxPaint::new(
            kind,
            if matches!(role, BlockifiedRole::Float) {
                FloatParticipation::Floating
            } else {
                FloatParticipation::None
            },
        ),
    ];
    let fragments = [
        PaintFragment::box_fragment(BoxId::new(0), None, None, 0),
        fragment(1, 0, None),
    ];
    let mut primitives = vec![color(1)];
    if matches!(
        kind,
        BoxPaintKind::InlineReplaced | BoxPaintKind::BlockReplaced
    ) {
        primitives.push(PaintPrimitive::fragment(
            FragmentId::new(1),
            FragmentPaintKind::ReplacedContent,
            0,
        ));
    }
    let outcome = plan_paint_order(
        PaintRequest::new(
            &boxes,
            &paints,
            &[],
            &[],
            &fragments,
            &primitives,
            BoxId::new(0),
        ),
        options(),
    );
    (
        outcome.result.map(|plan| events(&plan)),
        outcome.report.unwrap(),
    )
}

fn assert_blockified_category_refusal(kind: BoxPaintKind, role: BlockifiedRole) {
    let (result, report) = category_case(kind, role);
    assert!(
        matches!(result, Err(PaintError::Input(PaintInputError::InvalidBoxCategory { box_id, .. }))
        if box_id == BoxId::new(1)),
        "expected typed resolved-category refusal for {kind:?}/{role:?}, got {result:?}"
    );
    let reason = match role {
        BlockifiedRole::Float => BoxCategoryError::FloatingInline,
        BlockifiedRole::Absolute => BoxCategoryError::OutOfFlowInline,
        BlockifiedRole::Flex | BlockifiedRole::Grid => BoxCategoryError::InlineItem,
    };
    assert_eq!(
        result,
        Err(PaintError::Input(PaintInputError::InvalidBoxCategory {
            box_id: BoxId::new(1),
            reason,
        }))
    );
    for &resource in ResourceKind::ALL {
        assert_eq!(report.live(resource), 0);
    }
    // CSS blockification changes the outer inline category, while the matched
    // resolved block box keeps the same role, records and occurrences.
    let replaced = kind == BoxPaintKind::InlineReplaced;
    let control_kind = if replaced {
        BoxPaintKind::BlockReplaced
    } else {
        BoxPaintKind::Block
    };
    let (control, control_report) = category_case(control_kind, role);
    let expected = if replaced {
        vec![('[', 0), ('d', 0), ('d', 1), (']', 0)]
    } else {
        vec![('[', 0), ('d', 0), (']', 0)]
    };
    assert_eq!(control.unwrap(), expected);
    assert_eq!(control_report.live(ResourceKind::ScratchBytes), 0);
}

#[test]
fn floating_replaced_inline_requires_a_resolved_block_category() {
    assert_blockified_category_refusal(BoxPaintKind::InlineReplaced, BlockifiedRole::Float);
}
#[test]
fn absolute_replaced_inline_requires_a_resolved_block_category() {
    assert_blockified_category_refusal(BoxPaintKind::InlineReplaced, BlockifiedRole::Absolute);
}
#[test]
fn flex_replaced_inline_requires_a_resolved_block_category() {
    assert_blockified_category_refusal(BoxPaintKind::InlineReplaced, BlockifiedRole::Flex);
}
#[test]
fn grid_inline_requires_a_resolved_block_category() {
    assert_blockified_category_refusal(BoxPaintKind::Inline, BlockifiedRole::Grid);
}

#[test]
fn inline_categories_in_blockifying_roles_require_resolved_block_categories() {
    for role in [
        BlockifiedRole::Float,
        BlockifiedRole::Absolute,
        BlockifiedRole::Flex,
        BlockifiedRole::Grid,
    ] {
        for kind in [
            BoxPaintKind::Inline,
            BoxPaintKind::InlineBlock,
            BoxPaintKind::InlineReplaced,
        ] {
            assert_blockified_category_refusal(kind, role);
        }
    }
}
