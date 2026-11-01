use super::*;

const AHEM: &[u8] = include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn capture(
    system: &mut text::System,
    uppercase: bool,
    available: layout::Available,
) -> (
    layout::CompletedLayoutBatch<usize>,
    text::PreparedOrigin,
    text::Source,
) {
    let source = text::Source::identified("ß", text::Id::from_u64(6303), 13);
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        word_break: text::WordBreak::BreakAll,
        text_transform: text::TextTransform::Set(text::TextTransformSet::none().with_case(
            if uppercase {
                text::CaseTransform::Uppercase
            } else {
                text::CaseTransform::None
            },
        )),
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(0, 1),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        [0xdf]
    );
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                size: layout::Size::new(
                    match available {
                        layout::Available::Definite(width) => layout::PreferredSizeOf::value(
                            layout::LengthPercentageOf::px(width).unwrap(),
                        ),
                        _ => layout::PreferredSizeOf::AUTO,
                    },
                    layout::PreferredSizeOf::AUTO,
                ),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &capture,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            available,
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    (batch, origin, source)
}

#[test]
fn casing_expansion_changes_actual_fit_and_intrinsic_widths() {
    // Original Ahem has ß134 and S53, each1000units at UPEM1000.
    // Full Unicode uppercase is SS. Break-all supplies the legal S|S cut;
    // fitting operates on the two actual10px glyphs, not source length1.
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    for uppercase in [false, true] {
        let (batch, _, source) = capture(&mut system, uppercase, layout::Available::Definite(12.0));
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.source(), &source);
        assert_eq!(source.text().code_units(), [0xdf]);
        assert_eq!(source.id(), Some(text::Id::from_u64(6303)));
        assert_eq!(source.revision(), 13);
        let glyphs = placed
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), AHEM);
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let expected_glyph = (if uppercase { 53 } else { 134 }, 10.0);
        let expected_count = if uppercase { 2 } else { 1 };
        assert_eq!(glyphs, vec![expected_glyph; expected_count]);
        assert_eq!(
            placed
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
                .collect::<Vec<_>>(),
            vec![text::Range::new(0, 1); expected_count]
        );
        let lines = placed.lines();
        assert_eq!(lines.len(), if uppercase { 2 } else { 1 });
        assert!(
            lines
                .iter()
                .all(|line| line.range() == text::Range::new(0, 1))
        );
        let advances = batch
            .final_inline_fragments()
            .iter()
            .map(|entry| {
                let fragment = entry.fragment();
                (
                    fragment.line_index(),
                    fragment
                        .finalized_fragment()
                        .finalized_line()
                        .feedback()
                        .advance,
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(advances.len(), lines.len());
        assert!(advances.values().all(|advance| *advance == 10.0));
        for (available, expected) in [
            (layout::Available::MinContent, 10.0),
            (
                layout::Available::MaxContent,
                if uppercase { 20.0 } else { 10.0 },
            ),
        ] {
            let (intrinsic, _, _) = capture(&mut system, uppercase, available);
            let root = intrinsic
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap();
            assert_eq!(root.output().size.width, expected);
        }
    }
}

#[test]
fn receiving_expansion_selection_and_paint_retain_exact_original_occurrence() {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let (batch, origin, source) = capture(&mut system, true, layout::Available::Definite(12.0));
    let placed = batch.placed_text()[0].layout();
    let position_at = |x, y| {
        placed
            .positions()
            .find(|position| {
                placed.position_geometry(position).unwrap().rect().origin == text::Point::new(x, y)
            })
            .expect("actual issued position at the literal Ahem line edge")
    };
    let first = position_at(0.0, 0.0);
    let receiving = position_at(0.0, 10.0);
    let end = position_at(10.0, 10.0);
    for (a, b) in [(&receiving, &end), (&end, &receiving)] {
        let selected = placed.position_selection(a, b).unwrap();
        assert!(!selected.is_collapsed());
        assert_eq!(selected.source_text().code_units(), [0xdf]);
        assert_eq!(selected.plaintext().code_units(), [0xdf]);
        assert_eq!(selected.source_pieces().len(), 1);
        let piece = &selected.source_pieces()[0];
        assert_eq!(piece.origin(), &origin);
        assert_eq!(piece.origin().view().source(), &source);
        assert_eq!(piece.local_range(), text::Range::new(0, 1));
        assert_eq!(piece.aggregate_range(), text::Range::new(0, 1));
        assert_eq!(piece.code_units(), [0xdf]);
        assert_eq!(
            selected
                .geometry()
                .rects()
                .iter()
                .map(|rect| (rect.line(), rect.rect()))
                .collect::<Vec<_>>(),
            [(1, text::Rect::new(0.0, 10.0, 10.0, 10.0))]
        );
    }
    let both = placed.position_selection(&first, &end).unwrap();
    assert_eq!(both.source_text().code_units(), [0xdf]);
    assert_eq!(both.source_pieces().len(), 1);
    let collapsed = placed.position_selection(&receiving, &receiving).unwrap();
    assert!(collapsed.is_collapsed());
    assert!(collapsed.source_text().code_units().is_empty());
    assert!(collapsed.source_pieces().is_empty());
    assert!(collapsed.geometry().rects().is_empty());
    let fragments = placed
        .source_range_fragments(&origin, text::Range::new(0, 1))
        .unwrap();
    let mut rects = fragments
        .units()
        .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    rects.sort_by(|a, b| a.origin.y.total_cmp(&b.origin.y));
    assert_eq!(
        rects,
        [
            text::Rect::new(0.0, 0.0, 10.0, 10.0),
            text::Rect::new(0.0, 10.0, 10.0, 10.0)
        ]
    );
    assert_eq!(system.stats().layout_misses(), 0);
    drop(system);
    let paint = render::prepare_text_layout(
        placed,
        render::Transform::identity(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            8,
        ),
    )
    .unwrap();
    assert!(std::ptr::eq(paint.layout(), placed));
    assert_eq!(paint.layout().source(), &source);
    assert!(!paint.scene().is_empty());
    assert_eq!(paint.layout().lines(), placed.lines());
    assert!(
        paint
            .layout()
            .run_views()
            .all(|run| run.face().bytes() == AHEM)
    );
}

#[cfg(feature = "text-accessibility")]
#[test]
fn casing_expansion_enriches_original_accessibility_with_actual_geometry() {
    for uppercase in [false, true] {
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let (batch, _, _) = capture(&mut system, uppercase, layout::Available::Definite(30.0));
        let placed = batch.placed_text()[0].layout();
        let analyzed = placed.analyzed_source();
        drop(system);
        let mut accessibility = text::Accessibility::new(20);
        let parent = 1000_u64.into();
        let cursor = text::Cursor::new(0, text::Affinity::After);
        let source_update = analyzed
            .accessibility_update(&mut accessibility, parent, parent)
            .unwrap();
        let source_position = analyzed
            .access_position(&accessibility, cursor)
            .unwrap()
            .unwrap();
        let source_node = &source_update
            .nodes
            .iter()
            .find(|(id, _)| *id == source_position.node)
            .unwrap()
            .1;
        assert_eq!(source_node.value(), Some("ß"));
        // AccessKit lengths count original UTF8 bytes, not presentation scalars.
        assert_eq!(source_node.character_lengths(), [2]);
        assert!(source_node.bounds().is_none());
        let update = placed
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::Point::new(100.0, 200.0),
            )
            .unwrap();
        let position = placed
            .access_position(&accessibility, cursor)
            .unwrap()
            .unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == position.node)
            .unwrap()
            .1;
        assert_eq!(node.value(), Some("ß"));
        assert_eq!(node.character_lengths(), [2]);
        assert_eq!(position.character_index, 0);
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (100.0, 200.0, if uppercase { 120.0 } else { 110.0 }, 210.0)
        );
    }
}
