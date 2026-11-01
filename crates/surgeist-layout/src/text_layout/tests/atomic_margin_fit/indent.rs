//! Actual signed indentation composes before the final CSS fit width floor.

use super::*;

fn two_atomics() -> PreparedParagraph {
    let style = text::Style {
        wrap: text::Wrap::Word,
        white_space: text::WhiteSpace::Preserve,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        ..ahem_style(10.0, 10.0)
    };
    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::Atomic {
                node: 1,
                size: text::Size::new(10.0, 10.0),
            },
            ParagraphPart::Atomic {
                node: 2,
                size: text::Size::new(10.0, 10.0),
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let prepared = &p.prepared;
    assert!(prepared.analyzed_source().source().text().is_empty());
    assert_eq!(prepared.origin_views().len(), 0);
    let views = prepared.atomic_views().collect::<Vec<_>>();
    assert_eq!(views.len(), 2);
    assert_ne!(views[0].handle(), views[1].handle());
    let binding = p.binding(0);
    for (index, view) in views.iter().enumerate() {
        assert_eq!(view.source_position().get(), 0);
        assert_eq!(view.original_box().kind(), text::InlineBoxKind::InFlow);
        assert_eq!(view.original_box().size(), text::Size::new(10.0, 10.0));
        assert_eq!(
            binding.atomics[index].node,
            u32::try_from(index + 1).unwrap()
        );
        assert_eq!(binding.atomics[index].occurrence, view.handle());
    }
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        views
            .iter()
            .map(|view| text::PreparedParticipant::Atomic(view.handle()))
            .collect::<Vec<_>>()
    );
    let middle = prepared
        .opportunity_views()
        .filter(|offer| offer.boundary().view().participant_offset() == 1)
        .collect::<Vec<_>>();
    let [offer] = middle.as_slice() else {
        panic!("one real middle atomic opportunity");
    };
    assert_eq!(offer.kind(), text::PreparedBreakKind::Allowed);
    assert_eq!(offer.boundary().view().source_before().get(), 0);
    assert_eq!(offer.boundary().view().source_after().get(), 0);
    p
}

fn placed<S: LayoutScalar>(
    p: &PreparedParagraph,
    band: f64,
    indent: f64,
    ends: [f64; 2],
) -> CompletedLayoutBatchOf<u32, S> {
    let n = S::from_f64;
    let mut tree = PublicLayoutTreeOf::<S>::new().children(0, [1, 2]).style(
        0,
        NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            size: Size::new(PreferredSizeOf::px(n(band)), PreferredSizeOf::AUTO),
            text_indent: TextIndentOf {
                amount: LengthPercentageOf::px(n(indent)).unwrap(),
                each_line: false,
                hanging: false,
            },
            text_align: TextAlign::Start,
            text_wrap_style: TextWrapStyle::Auto,
            ..NodeInputOf::default()
        },
    );
    for (index, end) in ends.into_iter().enumerate() {
        let node = u32::try_from(index + 1).unwrap();
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                margin: Edges {
                    right: LengthAutoOf::px(n(end)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );
    }
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: vec![],
            child_order: vec![],
        },
        vec![],
    )
    .unwrap();
    compute_layout_with_flow(
        &flow,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(n(band)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn one_line<S: LayoutScalar>(
    p: &PreparedParagraph,
    band: f64,
    indent: f64,
    ends: [f64; 2],
    xs: [f64; 2],
) {
    let batch = placed::<S>(p, band, indent, ends);
    assert_eq!(batch.placed_text().len(), 1);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.analyzed_source().identity(),
        p.prepared.analyzed_source().identity()
    );
    assert!(layout.source().text().is_empty());
    assert_eq!(
        layout.lines().len(),
        1,
        "the whole two-atomic candidate must fit after combining signed indent+flow BEFORE final floor"
    );
    assert!(
        layout
            .glyph_runs()
            .iter()
            .all(|run| run.glyphs().is_empty())
    );
    let boxes = layout.inline_boxes();
    assert_eq!(boxes.len(), 2);
    for (index, (item, x)) in boxes.iter().zip(xs).enumerate() {
        assert_eq!(
            item.id(),
            text::Id::from_u64(u64::try_from(index + 1).unwrap())
        );
        assert_eq!(item.kind(), text::InlineBoxKind::InFlow);
        assert_eq!(
            item.line(),
            0,
            "both actual issued boxes, not a one-unit overflow fallback"
        );
        assert_eq!(item.rect(), text::Rect::new(x, 0.0, 10.0, 10.0));
    }
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size.width, S::from_f64(band));
        for (index, (x, end)) in xs.into_iter().zip(ends).enumerate() {
            let node = u32::try_from(index + 1).unwrap();
            let atomic = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output();
            assert_eq!(atomic.location, Point::new(S::from_f64(x), S::ZERO));
            assert_eq!(atomic.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
            assert_eq!(
                atomic.margin,
                Edges {
                    right: S::from_f64(end),
                    ..Edges::ZERO
                }
            );
        }
    }
}

fn positive_indent_retraction<S: LayoutScalar>() {
    let p = two_atomics();
    // No intermediate width floor: actual CSS flow-10 and-10, I10 yields
    // final-10 and required0<=5. Border geometry remains uncut10..20/0..10.
    one_line::<S>(&p, 5.0, 10.0, [-20.0, -20.0], [10.0, 0.0]);
}
fn zero_total_does_not_cancel_positive_indent<S: LayoutScalar>() {
    let p = two_atomics();
    // F1-10 permits the genuine middle cut, while F0 of the WHOLE candidate
    // still requires I10>band5. First prefix is genuinely fitting under AC;
    // the second indivisible border10 may use its own unavoidable overflow.
    let batch = placed::<S>(&p, 5.0, 10.0, [-20.0, 0.0]);
    assert_eq!(batch.placed_text().len(), 1);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.analyzed_source().identity(),
        p.prepared.analyzed_source().identity()
    );
    assert_eq!(
        layout.lines().len(),
        2,
        "all-CSS F0 does not admit a full candidate with I10 in band5"
    );
    let boxes = layout.inline_boxes();
    assert_eq!(boxes.len(), 2);
    assert_eq!((boxes[0].id(), boxes[0].line()), (text::Id::from_u64(1), 0));
    assert_eq!((boxes[1].id(), boxes[1].line()), (text::Id::from_u64(2), 1));
    assert_eq!(boxes[0].rect(), text::Rect::new(10.0, 0.0, 10.0, 10.0));
    assert_eq!(boxes[1].rect().origin.x, 0.0); // ordinary second line has no first-line indent.
    assert_eq!(boxes[1].rect().size, text::Size::new(10.0, 10.0));
    assert!(boxes[1].rect().origin.y > 0.0);
    assert!(
        layout
            .glyph_runs()
            .iter()
            .all(|run| run.glyphs().is_empty())
    );
}
fn negative_indent_preserves_positive_flow_fit<S: LayoutScalar>() {
    let p = two_atomics();
    // Both real zero-margin CSS boxes contribute10. I-10 plus F20 requires10
    // in band10, preserving negative first border rather than relocating it.
    one_line::<S>(&p, 10.0, -10.0, [0.0, 0.0], [-10.0, 0.0]);
}

#[test]
fn positive_first_indent_combines_with_later_negative_css_flow_before_final_floor_both_scalars() {
    positive_indent_retraction::<f32>();
    positive_indent_retraction::<f64>();
}
#[test]
fn zero_all_css_flow_still_requires_the_actual_positive_first_indent_both_scalars() {
    zero_total_does_not_cancel_positive_indent::<f32>();
    zero_total_does_not_cancel_positive_indent::<f64>();
}
#[test]
fn negative_first_indent_preserves_twenty_px_css_flow_in_a_ten_px_band_both_scalars() {
    negative_indent_preserves_positive_flow_fit::<f32>();
    negative_indent_preserves_positive_flow_fit::<f64>();
}
