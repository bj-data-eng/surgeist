//! Real CW and whole-upright preparation consumed by physical Layout placement.
use super::*;

fn capture(orientation: text::LineOrientation) -> TextParagraphBinding<u32> {
    let style = own(4.0, Wrap::None, text::OverflowWrap::Normal, false);
    let source = Source::new("ABC");
    let typography = if orientation == text::LineOrientation::Vertical {
        text::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::LeftToRight,
        }
    } else {
        text::AnalysisTypography::HorizontalComposition
    };
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            tab_reference: None,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, 3),
                aggregate: text::Range::new(0, 3),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        typography,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::None);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert_eq!(prepared.orientation(), orientation);
    assert_eq!(prepared.analyzed_source().typography(), typography);
    let origin = prepared.origin_views().next().unwrap().handle();
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![1],
        origins: vec![TextOriginBinding { origin, node: 1 }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    }
}

fn allocated<S: LayoutScalar>(orientation: text::LineOrientation, mode: WritingMode) {
    qualify_metrics(AHEM, 1000);
    let binding = capture(orientation);
    let origin = binding.origins[0].origin.clone();
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: mode,
                text_align: TextAlign::Justify,
                text_align_last: TextAlignLast::Justify,
                text_justification: TextJustification::InterCharacter,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(42.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
        );
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &flow,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(10.0)),
            AvailableOf::definite(S::from_f64(42.0)),
        ))
        .unwrap(),
    )
    .unwrap();
    let line = lines(&batch).remove(0);
    let selected = line.selected_line();
    assert_eq!(selected.request().context.orientation, orientation);
    assert_eq!(
        selected.request().context.spacing,
        text::SpacingPolicy::InterCharacter
    );
    assert_eq!(selected.request().context.available_inline, Some(42.0));
    amounts(&line, &[2.0, 2.0]);
    assert_eq!(line.measurements().alignment().advance(), 42.0);
    assert_eq!(line.feedback().advance, 42.0);
    for (index, row) in line.spacing_slots().enumerate() {
        assert_eq!(row.stretchability, text::SpacingStretchability::Direct);
        assert_eq!(row.authored_letter_spacing, 4.0);
        assert_eq!(row.advance, 6.0);
        let text::SpacingSlotTarget::Boundary {
            visual_before,
            visual_after,
        } = row.slot.target()
        else {
            panic!("actual character boundary required")
        };
        for (unit, start) in [(visual_before, index), (visual_after, index + 1)] {
            let range = text::Range::new(start, start + 1);
            assert_eq!(
                unit.pieces().next().unwrap().owner(),
                text::SpacingSlotOwner::Source {
                    origin: origin.clone(),
                    local: range,
                    aggregate: range,
                }
            );
        }
    }
    let upright = orientation == text::LineOrientation::Vertical;
    let expected_offsets = if upright { (-5.0, 8.0) } else { (0.0, 0.0) };
    let mut facts = Vec::new();
    for run in batch.placed_text()[0].layout().glyph_runs() {
        assert_eq!(run.orientation(), orientation);
        assert_eq!(
            run.glyph_basis(),
            if upright {
                text::GlyphBasis::Unrotated
            } else {
                text::GlyphBasis::Clockwise
            }
        );
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y()), expected_offsets);
            assert_eq!(glyph.advance(), 10.0);
            facts.push((glyph.range(), glyph.id(), run.origin().y + glyph.pen()));
        }
    }
    facts.sort_by_key(|fact| fact.0.start);
    assert_eq!(
        facts,
        [
            (text::Range::new(0, 1), 35, 0.0),
            (text::Range::new(1, 2), 36, 16.0),
            (text::Range::new(2, 3), 37, 32.0),
        ]
    );
    // Gap ownership is on the preceding cell; physical inline is y in both
    // orientations, independently of the raw whole-upright outline offsets.
    let layout = batch.placed_text()[0].layout();
    for (index, y) in [0.0, 16.0, 32.0, 42.0].into_iter().enumerate() {
        assert_eq!(
            layout
                .cursor(text::Cursor::new(index, text::Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .y,
            y
        );
    }
    copy(&batch, "ABC");
}

#[test]
fn clockwise_and_upright_character_fill_preserve_raw_packets_and_physical_cell_edges() {
    for (orientation, mode) in [
        (
            text::LineOrientation::SidewaysClockwise,
            WritingMode::SidewaysRl,
        ),
        (text::LineOrientation::Vertical, WritingMode::VerticalRl),
    ] {
        allocated::<f32>(orientation, mode);
        allocated::<f64>(orientation, mode);
    }
}
