//! Explicit source ZWSP replacement keeps original custody through native paint.

use super::*;

fn capture(
    system: &mut text::System,
    value: &str,
    replacement: text::style::WordSpaceTransform,
    mode: text::style::WhiteSpace,
    full_width: bool,
    authored_width: Option<f32>,
    request: layout::request::LayoutRootRequest,
) -> (
    layout::batch::CompletedLayoutBatch<usize>,
    text::prepared::PreparedOrigin,
    text::source::Source,
) {
    let source = text::source::Source::identified(value, text::source::Id::from_u64(6306), 23);
    let source_range = text::source::Range::new(0, source.text().len());
    let style = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        white_space: mode,
        word_space_transform: replacement,
        text_transform: text::style::TextTransform::Set(
            text::style::TextTransformSet::none().with_full_width(full_width),
        ),
        ..text::style::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let analyzed = text::analyzed::AnalyzedSource::from_paragraph(
        text::paragraph::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            terminal_edge: text::paragraph::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: source_range,
                aggregate: source_range,
                kind: text::paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    // These sources contain no Phase-I-discarded units. ZWSP replacement is a
    // later presentation operation; original U+3000 also survives Discard.
    assert_eq!(
        analyzed.processed_source().processed_text().code_units(),
        source.text().code_units()
    );
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::prepared::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                size: layout::geometry::Size::new(
                    match authored_width {
                        Some(width) => layout::sizing::PreferredSizeOf::value(
                            layout::length::LengthPercentageOf::px(width).unwrap(),
                        ),
                        None => layout::sizing::PreferredSizeOf::AUTO,
                    },
                    layout::sizing::PreferredSizeOf::AUTO,
                ),
                ..layout::node::NodeInput::DEFAULT
            }),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                origin.clone(),
            )),
        ],
        non_box: layout::node::NodeInput::non_box(),
    };
    let captured = layout::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::text_layout::TextParagraphBinding {
            inline_box_fragments: layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::text_layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(&captured, 0, request).unwrap();
    (batch, origin, source)
}

fn system(bytes: &[u8]) -> text::System {
    let mut system = text::System::default();
    system
        .register_fonts(bytes.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    system
}

#[test]
fn explicit_separator_replacement_keeps_original_range_selection_and_blank_paint() {
    let bytes = super::width_transform::width_ahem();
    let mut cases = vec![(
        text::style::WordSpaceTransform::IdeographicSpace,
        text::style::WhiteSpace::Preserve,
        false,
        35,
        10.0,
        1,
        20.0,
        40.0,
    )];
    // All four actual modes keep generated SPACE, and generated SPACE qualifies
    // for subsequent width conversion even in Collapse/Discard. This does not
    // change the separate rules for original authored SPACE.
    for mode in [
        text::style::WhiteSpace::Collapse,
        text::style::WhiteSpace::Discard,
        text::style::WhiteSpace::Preserve,
        text::style::WhiteSpace::PreserveSpaces,
    ] {
        cases.push((
            text::style::WordSpaceTransform::Space,
            mode,
            false,
            35,
            10.0,
            3,
            5.0,
            25.0,
        ));
        cases.push((
            text::style::WordSpaceTransform::Space,
            mode,
            true,
            36,
            20.0,
            1,
            20.0,
            60.0,
        ));
    }
    for (replacement, mode, width, letter, letter_advance, blank, blank_advance, total) in cases {
        let mut system = system(&bytes);
        let (batch, origin, source) = capture(
            &mut system,
            "A\u{200b}A",
            replacement,
            mode,
            width,
            Some(100.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(100.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.source(), &source);
        assert_eq!(source.text().code_units(), [0x41, 0x200b, 0x41]);
        assert_eq!(source.id(), Some(text::source::Id::from_u64(6306)));
        assert_eq!(source.revision(), 23);
        assert_eq!(placed.lines().len(), 1);
        let finalized = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(finalized.feedback().advance, total);
        let actual = placed
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            [
                (letter, letter_advance),
                (blank, blank_advance),
                (letter, letter_advance)
            ]
        );
        assert_eq!(
            placed
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
                .collect::<Vec<_>>(),
            [
                text::source::Range::new(0, 1),
                text::source::Range::new(1, 2),
                text::source::Range::new(2, 3)
            ]
        );
        let at = |x| {
            placed
                .positions()
                .find(|position| {
                    placed.position_geometry(position).unwrap().rect().origin
                        == text::geometry::Point::new(x, 0.0)
                })
                .unwrap()
        };
        let start = at(f64::from(letter_advance));
        let end = at(f64::from(letter_advance + blank_advance));
        let expected = text::geometry::Rect::new(
            f64::from(letter_advance),
            0.0,
            f64::from(blank_advance),
            10.0,
        );
        for (a, b) in [(&start, &end), (&end, &start)] {
            let selected = placed.position_selection(a, b).unwrap();
            assert_eq!(selected.source_text().code_units(), [0x200b]);
            assert_eq!(selected.plaintext().code_units(), [0x200b]);
            assert_eq!(selected.source_pieces().len(), 1);
            assert_eq!(selected.source_pieces()[0].origin(), &origin);
            assert_eq!(
                selected.source_pieces()[0].local_range(),
                text::source::Range::new(1, 2)
            );
            assert_eq!(
                selected.source_pieces()[0].aggregate_range(),
                text::source::Range::new(1, 2)
            );
            assert_eq!(selected.geometry().rects().len(), 1);
            assert_eq!(selected.geometry().rects()[0].rect(), expected);
        }
        let range = placed
            .source_range_fragments(&origin, text::source::Range::new(1, 2))
            .unwrap();
        assert_eq!(range.units().count(), 1);
        assert_eq!(
            range
                .units()
                .next()
                .unwrap()
                .pieces()
                .next()
                .unwrap()
                .rect(),
            expected
        );
        let whole = placed.position_selection(&at(0.0), &at(total)).unwrap();
        assert_eq!(whole.source_text().code_units(), [0x41, 0x200b, 0x41]);
        assert_eq!(whole.plaintext().code_units(), [0x41, 0x200b, 0x41]);
        // Use the same retained actual glyph to distinguish typographic advance
        // from ink; both qualified space glyphs have blank outlines.
        let glyph = placed
            .run_views()
            .flat_map(|run| run.glyphs().collect::<Vec<_>>())
            .find(|glyph| glyph.id() == blank)
            .unwrap();
        let glyph = render::text::TextGlyph::try_new(
            glyph.id(),
            glyph.pen() + f64::from(glyph.x()),
            f64::from(glyph.y()),
            glyph.advance(),
        )
        .unwrap();
        let font = render::text::FontData::try_from_bytes(
            bytes.clone(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let instance =
            render::text::TextInstance::try_new(10.0, vec![], render::text::GlyphSynthesis::none())
                .unwrap();
        let retained = font
            .prepare_glyph_run(
                &instance,
                &[glyph],
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                &mut surgeist_render::accounting::RenderAccountingContext::default(),
            )
            .unwrap();
        assert_eq!(
            retained.bounds().kind(),
            render::text::TextRunBoundsKind::Empty
        );
        drop(system);
        let paint = render::text_layout::prepare_text_layout(
            placed,
            render::geometry::Transform::identity(),
            render::text_layout::TextLayoutLimits::new(
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                8,
                32,
                8,
            ),
            &mut surgeist_render::accounting::RenderAccountingContext::default(),
        )
        .unwrap();
        assert!(std::ptr::eq(paint.layout(), placed));
        assert_eq!(paint.layout().source(), &source);
        assert_eq!(paint.layout().lines(), placed.lines());
        assert!(!paint.scene().is_empty());
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn separator_accessibility_keeps_original_zwsp_with_actual_replacement_geometry() {
    let bytes = super::width_transform::width_ahem();
    for (replacement, width, x, extent) in [
        (text::style::WordSpaceTransform::Space, false, 10.0, 5.0),
        (
            text::style::WordSpaceTransform::IdeographicSpace,
            false,
            10.0,
            20.0,
        ),
        (text::style::WordSpaceTransform::Space, true, 20.0, 20.0),
    ] {
        let mut system = system(&bytes);
        let (batch, _, source) = capture(
            &mut system,
            "A\u{200b}A",
            replacement,
            text::style::WhiteSpace::Preserve,
            width,
            Some(100.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(100.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        let analyzed = placed.analyzed_source();
        drop(system);
        let mut accessibility = text::accessibility::Accessibility::new(20);
        let parent = 1000_u64.into();
        let cursor = text::layout::Cursor::new(1, text::layout::Affinity::After);
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
        assert_eq!(source_node.value(), Some("\u{200b}"));
        assert_eq!(source_node.character_lengths(), [3]);
        assert!(source_node.bounds().is_none());
        let update = placed
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::geometry::Point::new(100.0, 200.0),
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
        assert_eq!(placed.source(), &source);
        assert_eq!(node.value(), Some("\u{200b}"));
        assert_eq!(node.character_lengths(), [3]);
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (100.0 + x, 200.0, 100.0 + x + extent, 210.0)
        );
    }
}

#[test]
fn discard_line_end_retains_ordinary_fit_and_intrinsic_contributions_in_a_narrow_block() {
    let bytes = super::width_transform::width_ahem();
    for (value, replacement, width, expected_glyphs, total, edge_x, edge_width) in [
        (
            "A\u{3000}",
            text::style::WordSpaceTransform::None,
            false,
            [(35, 10.0), (1, 20.0)],
            30.0,
            10.0,
            20.0,
        ),
        (
            "A\u{200b}",
            text::style::WordSpaceTransform::Space,
            false,
            [(35, 10.0), (3, 5.0)],
            15.0,
            10.0,
            5.0,
        ),
        (
            "A\u{200b}",
            text::style::WordSpaceTransform::Space,
            true,
            [(36, 20.0), (1, 20.0)],
            40.0,
            20.0,
            20.0,
        ),
    ] {
        let mut system = system(&bytes);
        let (batch, origin, source) = capture(
            &mut system,
            value,
            replacement,
            text::style::WhiteSpace::Discard,
            width,
            Some(10.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(10.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.source(), &source);
        assert_eq!(
            source.text().code_units(),
            value.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(placed.lines().len(), 1);
        let root = batch
            .node_layout(0)
            .unwrap()
            .single_box()
            .expect("fixture node produces one box");
        assert_eq!(root.final_output().size.width, 10.0);
        let finalized = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(finalized.feedback().advance, total);
        // These are actual Layout-selected receipts at a finite10px width.
        // Discard's surviving characters have no hanging exclusion, even
        // beyond that edge; raw glyph advance alone cannot prove this rule.
        let measurements = finalized.measurements();
        for measure in [
            measurements.fit(),
            measurements.alignment(),
            measurements.min_content_contribution(),
            measurements.max_content_contribution(),
        ] {
            assert_eq!(measure.advance(), total);
            assert_eq!(measure.occupied_start(), 0.0);
            assert_eq!(measure.occupied_end(), total);
        }
        let edge = finalized
            .edge_units()
            .find(|unit| {
                unit.source_pieces()
                    .any(|piece| piece == text::source::Range::new(1, 2))
            })
            .unwrap();
        assert_eq!(
            edge.disposition(),
            text::finalized::LineEdgeDisposition::Ordinary
        );
        assert_eq!(edge.excluded_contribution(), 0.0);
        assert_eq!(edge.outward_overlap(), 0.0);
        assert_eq!(edge.advance(), edge_width);
        let glyphs = placed
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, expected_glyphs);
        let expected_edge = text::geometry::Rect::new(edge_x, 0.0, edge_width, 10.0);
        let fragments = placed
            .source_range_fragments(&origin, text::source::Range::new(1, 2))
            .unwrap();
        assert_eq!(fragments.units().count(), 1);
        assert_eq!(
            fragments
                .units()
                .next()
                .unwrap()
                .pieces()
                .next()
                .unwrap()
                .rect(),
            expected_edge
        );
        let at = |x| {
            placed
                .positions()
                .find(|position| {
                    placed.position_geometry(position).unwrap().rect().origin
                        == text::geometry::Point::new(x, 0.0)
                })
                .unwrap()
        };
        let start = at(edge_x);
        let end = at(total);
        for (a, b) in [(&start, &end), (&end, &start)] {
            let selected = placed.position_selection(a, b).unwrap();
            assert_eq!(
                selected.source_text().code_units(),
                &source.text().code_units()[1..2]
            );
            assert_eq!(
                selected.plaintext().code_units(),
                &source.text().code_units()[1..2]
            );
            assert_eq!(selected.source_pieces().len(), 1);
            assert_eq!(selected.source_pieces()[0].origin(), &origin);
            assert_eq!(
                selected.source_pieces()[0].local_range(),
                text::source::Range::new(1, 2)
            );
            assert_eq!(selected.geometry().rects()[0].rect(), expected_edge);
        }
        // LB7/LB21 prohibit breaking before the trailing SPACE/BA separator;
        // there is no earlier word or emergency opportunity in these inputs.
        // Min/max therefore each consume the entire ordinary15/30/40px line.
        for purpose in [
            layout::constraint::SizingOperation::MinContent,
            layout::constraint::SizingOperation::MaxContent,
        ] {
            let (intrinsic, _, _) = capture(
                &mut system,
                value,
                replacement,
                text::style::WhiteSpace::Discard,
                width,
                None,
                NativeViewport::new(
                    layout::geometry::Size::new(None, None),
                    layout::geometry::FlowAxes::new(
                        layout::geometry::WritingMode::HorizontalTb,
                        layout::geometry::Direction::Ltr,
                    ),
                )
                .request(layout::geometry::Size::new(
                    purpose,
                    layout::constraint::SizingOperation::Normal,
                )),
            );
            let root = intrinsic
                .node_layout(0)
                .unwrap()
                .single_box()
                .expect("fixture node produces one box");
            assert_eq!(root.final_output().size.width, total as f32);
        }
    }
}
