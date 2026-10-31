//! Ordinary conditional fallback retains its method across canonical placement.
use super::*;

fn singleton_fallback<S: LayoutScalar>(line_height: f64) {
    qualify_metrics(AHEM, 1000);
    let style = text::Style {
        line_height: text::LineHeight::Absolute(line_height),
        ..own(0.0, Wrap::Word, text::OverflowWrap::Anywhere, false)
    };
    let paragraph = paragraph(AHEM, "ABC", style);
    assert_eq!(
        paragraph
            .prepared
            .opportunity_views()
            .map(|view| view.kind())
            .collect::<Vec<_>>(),
        [text::PreparedBreakKind::ParagraphEnd]
    );

    // Each raw Ahem glyph advances10: two cannot fit15. Each singleton has no
    // character boundary, so its genuine unexpandable fallback is None/Center.
    // The overflowing first two cuts must come from Text's conditional search.
    let actual = finite::<S>(&paragraph, 15.0, TextAlignLast::Justify);
    let layout = actual.placed_text()[0].layout();
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [
            text::Range::new(0, 1),
            text::Range::new(1, 2),
            text::Range::new(2, 3),
        ]
    );
    let finalized = lines(&actual);
    assert_eq!(finalized.len(), 3);
    // Ahem10 has ascent8/descent2; absolute line height contributes equal
    // half-leading. This is the natural metric, before physical snapping.
    let natural_baseline = 8.0 + (line_height - 10.0) / 2.0;
    for (index, line) in finalized.iter().enumerate() {
        assert_eq!(
            raw(line, AHEM),
            [(
                text::Range::new(index, index + 1),
                [35, 36, 37][index],
                10.0
            )]
        );
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.feedback().line_height, line_height);
        assert_eq!(line.feedback().baseline, natural_baseline);
        assert_eq!(line.measurements().fit().advance(), 10.0);
        assert_eq!(line.measurements().alignment().advance(), 10.0);
        amounts(line, &[]);
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(request.context.available_inline, Some(15.0));
        assert_eq!(request.context.spacing, text::SpacingPolicy::None);
        assert_eq!(request.context.alignment, text::Alignment::Center);
        assert_eq!(
            (
                request.start.view().source_after().get(),
                request.end.view().source_before().get(),
            ),
            (index, index + 1)
        );
        if index < 2 {
            assert_eq!(request.context.end, text::LineEnd::Soft);
            let text::PreparedTermination::Conditional(cut) = &request.termination else {
                panic!("singleton overflow retains its actual conditional authority")
            };
            assert_eq!(cut.prepared_text(), paragraph.prepared);
            assert_eq!(cut.purpose(), text::ConditionalPurpose::Overflow);
            assert_eq!(cut.start(), request.start);
            assert_eq!(cut.boundary(), request.end);
            assert_eq!(
                cut.ceiling().view().kind(),
                text::PreparedBreakKind::ParagraphEnd
            );
        } else {
            assert_eq!(request.context.end, text::LineEnd::Paragraph);
            let text::PreparedTermination::Ordinary(end) = &request.termination else {
                panic!("the last source glyph consumes the genuine paragraph end")
            };
            assert_eq!(end.view().kind(), text::PreparedBreakKind::ParagraphEnd);
        }
    }
    assert_eq!(
        paints(&actual),
        [
            (0, text::Range::new(0, 1), 35, 2.5),
            (1, text::Range::new(1, 2), 36, 2.5),
            (2, text::Range::new(2, 3), 37, 2.5),
        ]
    );
    // Canonical placement rounds the actual first baseline to8. LH10 is the
    // integral control; LH10.5 has baseline8.25 and therefore start−.25. The
    // latter must replay the same ordinary method after its canonical retry.
    assert_eq!(layout.lines()[0].baseline(), 8.0);
    assert_eq!(layout.lines()[0].bounds().origin.y, 8.0 - natural_baseline);
    assert_eq!(layout.lines()[0].bounds().size.height, line_height);
    copy(&actual, "ABC");
}

#[test]
fn conditional_fallback_preserves_source_through_fractional_canonical_retry_f32() {
    singleton_fallback::<f32>(10.0);
    singleton_fallback::<f32>(10.5);
}

#[test]
fn conditional_fallback_preserves_source_through_fractional_canonical_retry_f64() {
    singleton_fallback::<f64>(10.0);
    singleton_fallback::<f64>(10.5);
}
