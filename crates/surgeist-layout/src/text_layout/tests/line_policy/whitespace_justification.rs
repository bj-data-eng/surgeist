//! Existing-callable consumer budget, independently of new measurement APIs.
use super::*;

fn preserved_soft_budget<S: LayoutScalar>() {
    let paragraph = prepare("X YYY", WhiteSpace::Preserve, false);
    let batch = batch::<S>(
        &paragraph,
        40.0,
        TextAlign::Justify,
        TextAlignLast::Auto,
        TextJustification::Spaces,
        false,
    );
    let lines = lines(&batch);
    assert_eq!(lines.len(), 2);
    let first = &lines[0];
    assert_eq!(
        first.selected_line().request().context.end,
        surgeist_text::LineEnd::Soft
    );
    assert_eq!(
        first
            .selected_line()
            .request()
            .end
            .view()
            .source_after()
            .get(),
        2
    );
    assert_eq!(
        first.selected_line().request().context.available_inline,
        Some(40.0)
    );
    assert_eq!(lines[1].feedback().advance, 30.0);
    let mut glyphs = Vec::new();
    for run in first.run_views() {
        assert_eq!(
            run.face().bytes(),
            include_bytes!("../../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
        );
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
        glyphs.extend(
            run.glyphs()
                .map(|g| (g.range(), g.id(), g.advance(), run.origin().x + g.pen())),
        );
    }
    assert_eq!(
        glyphs,
        [
            (surgeist_text::Range::new(0, 1), 58, 10.0, 0.0),
            (surgeist_text::Range::new(1, 2), 3, 10.0, 10.0)
        ]
    );
    let slots: Vec<_> = first.spacing_slots().collect();
    assert_eq!(slots.len(), 1);
    assert_eq!(
        slots[0].disposition,
        SpacingSlotDisposition::Retained { trailing: true }
    );
    let unit = paragraph
        .prepared
        .unit_views()
        .find(|unit| unit.source_range() == surgeist_text::Range::new(1, 2))
        .unwrap()
        .handle();
    assert_eq!(
        slots[0].slot.participants().cloned().collect::<Vec<_>>(),
        [surgeist_text::PreparedParticipant::Text(unit.clone())]
    );
    assert_eq!(
        slots[0]
            .slot
            .pieces()
            .map(|piece| piece.owner())
            .collect::<Vec<_>>(),
        [SpacingSlotOwner::Source {
            origin: paragraph.origin(1),
            local: surgeist_text::Range::new(1, 2),
            aggregate: surgeist_text::Range::new(1, 2),
        }]
    );
    assert_eq!(
        unit.view()
            .origins()
            .map(|o| o.origin())
            .collect::<Vec<_>>(),
        [paragraph.origin(1)]
    );
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), &[88, 32, 89, 89, 89]);
    assert_eq!(
        layout
            .cursor(surgeist_text::Cursor::new(
                1,
                surgeist_text::Affinity::Before
            ))
            .unwrap()
            .rect()
            .origin
            .x,
        10.0
    );
    let end = layout
        .cursor(surgeist_text::Cursor::new(
            2,
            surgeist_text::Affinity::Before,
        ))
        .unwrap()
        .rect()
        .origin
        .x;
    // Natural Soft hanging omits the final SPACE from alignment, leaving10.
    // The explicit one-slot operation distributes40-10=30 to its real10px
    // contribution. Raw flow/caret geometry is therefore50; it is not a claim
    // that the allocation can fill the non-hanging core or a second formatter.
    assert_eq!(
        (first.feedback().advance, slots[0].advance, end),
        (50.0, 40.0, 50.0)
    );
}

#[test]
fn preserved_soft_justification_budgets_alignment_without_rewriting_raw_geometry_f32() {
    preserved_soft_budget::<f32>();
}
#[test]
fn preserved_soft_justification_budgets_alignment_without_rewriting_raw_geometry_f64() {
    preserved_soft_budget::<f64>();
}
