//! CSS tab positioning uses actual stops, with source text preserved.
use super::*;

#[test]
fn preserved_tab_positions_the_next_glyph_at_its_block_stop() {
    let mut system = system();
    let prepared = prepared(&mut system, "A\tB", WhiteSpace::Preserve);
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [65, 9, 66]
    );

    // The independent FontAdvance policy remains a real retained-font control:
    // Ahem at 20px emits its 20px .notdef glyph for U+0009 under that policy.
    let mut control = request(&prepared);
    control.context.tabs = TabContext::FontAdvance;
    let control = prepared
        .select(control)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(control.feedback().advance, 60.0);
    let glyphs = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance(), glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        glyphs(&control),
        [(35, 20.0, 0.0), (0, 20.0, 20.0), (36, 20.0, 40.0)]
    );

    // Four actual 20px spaces give an 80px tab interval. From A's end at
    // 20px, the next stop is 80px: a 60px shift, above the 10px half-ch floor.
    // CSS renders the tab as that shift, without its FontAdvance glyph.
    let mut selection = request(&prepared);
    selection.context.tabs = TabContext::Stops {
        origin: 0.0,
        positions: vec![80.0, 160.0, 240.0],
    };
    let line = prepared
        .select(selection)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 100.0);
    assert_eq!(glyphs(&line), [(35, 20.0, 0.0), (36, 20.0, 80.0)]);
    assert_eq!(
        line.prepared_text()
            .analyzed_source()
            .source()
            .text()
            .code_units(),
        [65, 9, 66]
    );
}
