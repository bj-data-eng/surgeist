//! Two genuine Direct opportunities share the positive residual at one priority.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    last: TextAlignLast,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                text_align: TextAlign::Justify,
                text_align_last: last,
                text_justification: TextJustification::InterWord,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(70.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, request(70.0)).unwrap()
}

fn finalized<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> text::FinalizedLine {
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 1);
    batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line()
}

fn glyphs<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<(text::Range, u32, f32, f64)> {
    let mut result = Vec::new();
    for run in batch.placed_text()[0].layout().glyph_runs() {
        assert_eq!(run.orientation(), text::LineOrientation::Horizontal);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
            result.push((
                glyph.range(),
                glyph.id(),
                glyph.advance(),
                run.origin().x + glyph.pen(),
            ));
        }
    }
    result.sort_by_key(|fact| fact.0.start);
    result
}

fn lane<S: LayoutScalar>() {
    let own = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: Wrap::None,
        word_spacing: 4.0,
        ..ahem_style(10.0, 10.0)
    };
    let paragraph = PreparedParagraph::plain(1, Source::new("A B C"), own);
    let natural = compute::<S>(&paragraph, TextAlignLast::Auto);
    let old = finalized(&natural);
    let old_glyphs = glyphs(&natural);
    // Five intrinsic10 glyphs plus two authored4 terms:50+8=58.
    assert_eq!(old.measurements().alignment().advance(), 58.0);
    assert_eq!(
        old.spacing_slots()
            .map(|row| row.allocation)
            .collect::<Vec<_>>(),
        [0.0, 0.0]
    );
    assert_eq!(
        old_glyphs.iter().map(|fact| fact.3).collect::<Vec<_>>(),
        [0.0, 12.0, 24.0, 36.0, 48.0]
    );

    let actual = compute::<S>(&paragraph, TextAlignLast::Justify);
    let line = finalized(&actual);
    let selected = line.selected_line();
    let context = &selected.request().context;
    assert_eq!(context.spacing, text::SpacingPolicy::InterWord);
    assert_eq!(context.alignment, text::Alignment::Justify);
    assert_eq!(context.end, text::LineEnd::Paragraph);
    assert_eq!(context.available_inline, Some(70.0));
    assert_eq!(context.indent, 0.0);
    assert_eq!(context.alignment_insets, text::LineAlignmentInsets::ZERO);
    assert_eq!(line.prepared_text(), paragraph.prepared);
    assert!(line.run_views().all(|run| run.face().bytes() == AHEM));
    // Capacity70 minus natural58 is12; two equal-priority Direct owners get6.
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].slot, rows[1].slot);
    assert_eq!(
        selected.spacing_slots().collect::<Vec<_>>(),
        rows.iter().map(|row| row.slot.clone()).collect::<Vec<_>>()
    );
    for (row, range) in rows
        .iter()
        .zip([text::Range::new(1, 2), text::Range::new(3, 4)])
    {
        assert_eq!(row.slot.selected_line(), selected);
        assert_eq!(
            row.geometry,
            text::SpacingSlotGeometry::Independent {
                intrinsic_advance: 10.0
            }
        );
        assert_eq!(row.stretchability, text::SpacingStretchability::Direct);
        assert!(row.can_expand());
        assert_eq!(
            (row.authored_word_spacing, row.allocation, row.advance),
            (4.0, 6.0, 20.0)
        );
        assert_eq!(
            row.slot
                .pieces()
                .map(|piece| piece.owner())
                .collect::<Vec<_>>(),
            [text::SpacingSlotOwner::Source {
                origin: paragraph.origin(1),
                local: range,
                aggregate: range,
            }]
        );
        let unit = paragraph
            .prepared
            .unit_views()
            .find(|unit| unit.source_range() == range)
            .unwrap()
            .handle();
        assert_eq!(
            row.slot.participants().cloned().collect::<Vec<_>>(),
            [text::PreparedParticipant::Text(unit)]
        );
    }
    assert_eq!(line.measurements().alignment().advance(), 70.0);
    assert_eq!(line.feedback().advance, 70.0);
    assert_eq!(
        glyphs(&actual),
        [
            (text::Range::new(0, 1), 35, 10.0, 0.0),
            (text::Range::new(1, 2), 3, 10.0, 15.0),
            (text::Range::new(2, 3), 36, 10.0, 30.0),
            (text::Range::new(3, 4), 3, 10.0, 45.0),
            (text::Range::new(4, 5), 37, 10.0, 60.0),
        ]
    );
    // Logical cell starts differ from the two half-adjusted SPACE paint pens.
    let layout = actual.placed_text()[0].layout();
    assert_eq!(layout.source().text(), &text::Text::from("A B C"));
    for (at, x) in [0.0, 10.0, 30.0, 40.0, 60.0, 70.0].into_iter().enumerate() {
        assert_eq!(
            layout
                .cursor(text::Cursor::new(at, text::Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            x
        );
    }
    assert_eq!(glyphs(&natural), old_glyphs);
    assert_eq!(old.measurements().alignment().advance(), 58.0);
    assert_eq!(
        old.spacing_slots()
            .map(|row| row.allocation)
            .collect::<Vec<_>>(),
        [0.0, 0.0]
    );
}

#[test]
fn two_actual_direct_separators_receive_equal_complete_allocations_and_fill_seventy() {
    // Independently read licensed Ahem's UPEM and actual horizontal metrics.
    let u16_at = |at| u16::from_be_bytes(AHEM[at..at + 2].try_into().unwrap());
    let u32_at = |at| u32::from_be_bytes(AHEM[at..at + 4].try_into().unwrap()) as usize;
    let table = |tag: &[u8; 4]| {
        let record = (0..usize::from(u16_at(4)))
            .map(|index| 12 + index * 16)
            .find(|&record| &AHEM[record..record + 4] == tag)
            .unwrap();
        u32_at(record + 8)
    };
    assert_eq!(u16_at(table(b"head") + 18), 1000);
    let hmtx = table(b"hmtx");
    for glyph in [3, 35, 36, 37] {
        assert_eq!(u16_at(hmtx + glyph * 4), 1000);
    }
    lane::<f32>();
    lane::<f64>();
}
