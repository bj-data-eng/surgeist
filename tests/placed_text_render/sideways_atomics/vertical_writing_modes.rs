//! Authentic horizontal CW composition follows either vertical block direction.
//! Actual child orientation remains independent of the containing writing mode.
use super::*;

fn composition(mode: layout::WritingMode, kind: text::InlineBoxKind) {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let mut source = text::Source::identified("\u{3a7}\u{3a7}", text::Id::from_u64(1558001), 13);
    source.inline_box(text::InlineBox::new(
        text::Id::from_u64(2),
        kind,
        1,
        text::Size::new(6.0, 14.0),
    ));
    let child_source = text::Source::identified("\u{3a7}", text::Id::from_u64(1558002), 17);
    let parent = prepare(
        &mut system,
        &source,
        true,
        text::LineOrientation::SidewaysClockwise,
    );
    let child = prepare(
        &mut system,
        &child_source,
        false,
        text::LineOrientation::Horizontal,
    );
    assert_eq!(
        parent.analyzed_source().typography(),
        text::AnalysisTypography::HorizontalComposition
    );
    let origins = parent
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let atomic = parent.atomic_views().next().unwrap().handle();
    let child_origin = child.origin_views().next().unwrap().handle();
    let tree = AtomicTree {
        inputs: vec![
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                writing_mode: mode,
                inline_baseline_context: layout::InlineBaselineContext::Physical,
                size: layout::Size::new(px(40.0), px(40.0)),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origins[0].clone())),
            layout::LayoutInput::box_input(layout::NodeInput {
                display: if kind == text::InlineBoxKind::InFlow {
                    layout::Display::InlineBlock
                } else {
                    layout::Display::Block
                },
                position: if kind == text::InlineBoxKind::InFlow {
                    layout::Position::Relative
                } else {
                    layout::Position::Absolute
                },
                writing_mode: layout::WritingMode::HorizontalTb,
                inline_baseline_context: layout::InlineBaselineContext::Physical,
                size: layout::Size::new(px(6.0), px(14.0)),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origins[1].clone())),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(child_origin.clone())),
        ],
        children: vec![vec![1, 2, 3], vec![], vec![4], vec![], vec![]],
        non_box: layout::NodeInput::non_box(),
    };
    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![
            layout::TextParagraphBinding {
                inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
                prepared: parent.clone(),
                container: 0,
                nodes: vec![1, 2, 3],
                origins: origins
                    .iter()
                    .cloned()
                    .zip([1, 3])
                    .map(|(origin, node)| layout::TextOriginBinding { origin, node })
                    .collect(),
                atomics: vec![layout::TextAtomicBinding {
                    occurrence: atomic.clone(),
                    node: ATOMIC_NODE,
                }],
                controls: vec![],
                line_strut: None,
            },
            layout::TextParagraphBinding {
                inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
                prepared: child.clone(),
                container: ATOMIC_NODE,
                nodes: vec![4],
                origins: vec![layout::TextOriginBinding {
                    origin: child_origin,
                    node: 4,
                }],
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            },
        ],
        layout::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &flow,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(40.0),
            layout::Available::Definite(40.0),
        ))
        .unwrap(),
    )
    .unwrap();
    // Ahem's horizontal em metrics give B8/H10. The same CW cell maps
    // line-over to the right in both modes, but block-start changes sides:
    // Vrl C=40-8=32; Vlr C=10-8=2. A physical fallback child baseline is its
    // left line-under border, so in-flow x=C. Absolute all-auto placement
    // instead uses the static flow-start edge: right40 or left0.
    let (cross, child_x) = match (mode, kind) {
        (layout::WritingMode::VerticalRl, text::InlineBoxKind::InFlow) => (32.0, 32.0),
        (layout::WritingMode::VerticalLr, text::InlineBoxKind::InFlow) => (2.0, 2.0),
        (layout::WritingMode::VerticalRl, text::InlineBoxKind::OutOfFlow) => (32.0, 34.0),
        (layout::WritingMode::VerticalLr, text::InlineBoxKind::OutOfFlow) => (2.0, 0.0),
        _ => unreachable!("this witness covers precisely the two vertical writing modes"),
    };
    let (following, advance) = match kind {
        text::InlineBoxKind::InFlow => (24.0, 34.0),
        text::InlineBoxKind::OutOfFlow => (10.0, 20.0),
    };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = entries
            .iter()
            .find(|entry| entry.node() == ATOMIC_NODE)
            .unwrap()
            .output();
        assert_eq!(output.location, layout::Point::new(child_x as f32, 10.0));
        assert_eq!(output.size, layout::Size::new(6.0, 14.0));
        assert_eq!(output.content_size.width, 10.0);
    }
    let parent_layout = placed(&batch, 0);
    let child_layout = placed(&batch, ATOMIC_NODE);
    assert_eq!(batch.placed_text().len(), 2);
    assert_eq!(parent_layout.source(), &source);
    assert_eq!(child_layout.source(), &child_source);
    assert_eq!(
        parent_layout.orientation(),
        text::LineOrientation::SidewaysClockwise
    );
    assert_eq!(
        child_layout.orientation(),
        text::LineOrientation::Horizontal
    );
    assert_eq!(parent_layout.lines().len(), 1);
    assert_eq!(child_layout.lines().len(), 1);
    assert_eq!(parent_layout.lines()[0].baseline(), cross);
    assert_eq!(child_layout.lines()[0].baseline(), 18.0);
    let receipt = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| matches!(entry.owner(), layout::InlineFragmentOwner::Node(1)))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(receipt.prepared_text(), parent);
    assert_eq!(receipt.feedback().advance, advance);
    assert_eq!(receipt.atomic_slots().next().unwrap().occurrence, atomic);
    let parent_glyphs = parent_layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(
                run.face().catalogue_identity(),
                parent.font_catalogue_identity()
            );
            assert_eq!(run.orientation(), text::LineOrientation::SidewaysClockwise);
            assert_eq!(run.glyph_basis(), text::GlyphBasis::Clockwise);
            assert_eq!(run.origin().x, cross);
            run.glyphs()
                .map(|glyph| {
                    assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (0.0, 0.0, 10.0));
                    (glyph.id(), run.origin().y + glyph.pen())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(parent_glyphs, [(274, 0.0), (274, following)]);
    let child_runs = child_layout.run_views().collect::<Vec<_>>();
    assert_eq!(child_runs.len(), 1);
    let child_run = &child_runs[0];
    assert_eq!(child_run.face().bytes(), AHEM);
    assert_eq!(
        child_run.face().catalogue_identity(),
        child.font_catalogue_identity()
    );
    assert_eq!(child_run.orientation(), text::LineOrientation::Horizontal);
    assert_eq!(child_run.glyph_basis(), text::GlyphBasis::Unrotated);
    assert_eq!(child_run.origin(), text::Point::new(child_x, 18.0));
    assert_eq!(
        child_run
            .glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(274, 0.0, 0.0, 0.0, 10.0)]
    );
    let boxes = parent_layout.inline_boxes();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].kind(), kind);
    assert_eq!(boxes[0].rect(), text::Rect::new(child_x, 10.0, 6.0, 14.0));
    assert_eq!(
        parent_layout.hit(text::Point::new(child_x + 1.0, 15.0)),
        text::Hit::InlineBox(text::Id::from_u64(2))
    );
    for (origin, local, y) in [
        (&origins[0], text::Range::new(0, 1), 0.0),
        (&origins[1], text::Range::new(1, 2), following),
    ] {
        assert_eq!(
            parent_layout
                .source_range_fragments(origin, local)
                .unwrap()
                .units()
                .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [text::Rect::new(cross - 2.0, y, 10.0, 10.0)]
        );
    }
    assert_eq!(
        parent_layout
            .cursor(text::Cursor::new(0, text::Affinity::After))
            .unwrap()
            .rect(),
        text::Rect::new(cross - 2.0, 0.0, 10.0, 1.0)
    );
    assert_eq!(
        child_layout
            .cursor(text::Cursor::new(0, text::Affinity::After))
            .unwrap()
            .rect(),
        text::Rect::new(child_x, 10.0, 1.0, 10.0)
    );
    let before_paint = system.stats();
    // The outer scale and translation do not commute with the parent's CW
    // basis. Both exact Layout snapshots enter the Render owner unchanged;
    // Render's private command tests qualify execution of that composition.
    for outer in [
        render::Transform::identity(),
        render::Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
    ] {
        for actual in [parent_layout, child_layout] {
            let paint = render::prepare_text_layout(actual, outer, limits()).unwrap();
            assert!(std::ptr::eq(paint.layout(), actual));
            assert_eq!(paint.layout().source(), actual.source());
            assert_eq!(paint.layout().lines(), actual.lines());
            assert!(!paint.scene().is_empty());
        }
    }
    assert_eq!(system.stats(), before_paint);
    assert_eq!(system.stats().layout_misses(), 0);
}

#[test]
fn native_vertical_writing_modes_keep_clockwise_parent_and_horizontal_atomic_child() {
    // Independently admit the same licensed asymmetric glyph through Render.
    // Its horizontal stripe distinguishes the parent's CW basis from the
    // child's unrotated basis without a font derivative or placeholder box.
    let font = render::FontData::try_from_bytes(AHEM.to_vec(), 0).unwrap();
    let instance =
        render::TextInstance::try_new(10.0, vec![], render::GlyphSynthesis::none()).unwrap();
    let glyphs = [render::TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()];
    let admitted = font
        .prepare_glyph_run(
            &instance,
            &glyphs,
            render::GlyphRunLimits::new(1, 64, 1 << 20, 1 << 20),
        )
        .unwrap();
    assert_eq!(admitted.instance(), &instance);
    assert_eq!(admitted.glyphs(), glyphs);
    assert_eq!(
        admitted.bounds().ink_rect(),
        Some(render::Rect::try_new(0.0, -6.0, 10.0, 2.0).unwrap())
    );
    for mode in [
        layout::WritingMode::VerticalRl,
        layout::WritingMode::VerticalLr,
    ] {
        for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
            composition(mode, kind);
        }
    }
}
