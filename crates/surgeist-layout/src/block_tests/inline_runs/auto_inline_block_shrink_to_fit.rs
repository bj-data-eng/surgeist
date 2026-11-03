//! Auto inline-block width uses shrink-to-fit, independently of fitted line advance.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{PreparedParagraph, ahem_style};
use surgeist_text as text;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn prepared_words() -> PreparedParagraph {
    let style = text::Style {
        white_space: text::WhiteSpace::Collapse,
        wrap: text::Wrap::Word,
        ..ahem_style(10.0, 10.0)
    };
    PreparedParagraph::plain(2, text::Source::new("AAA BBB"), style)
}

fn qualify_intrinsics<S: LayoutScalar>(paragraph: &PreparedParagraph) {
    for (operation, expected) in [
        (AvailableOf::MIN_CONTENT, 30.0),
        (AvailableOf::MAX_CONTENT, 70.0),
    ] {
        let tree = paragraph.flat_text_tree::<S>(
            1,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
        let flow = LayoutFlowInput::try_new(
            &tree,
            1,
            vec![paragraph.binding(1)],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &flow,
            1,
            LayoutRootRequestOf::viewport(Size::new(operation, AvailableOf::MAX_CONTENT)).unwrap(),
        )
        .unwrap();
        for (phase, entries) in [
            ("unrounded", batch.unrounded_entries()),
            ("final", batch.final_entries()),
        ] {
            let actual = entries
                .iter()
                .find(|entry| entry.node() == 1)
                .unwrap()
                .output();
            assert_eq!(
                actual.size.width,
                n(expected),
                "{phase}: genuine {operation:?} Text/native width"
            );
        }
        assert_eq!(
            batch.placed_text()[0].layout().source().text(),
            &text::Text::from("AAA BBB")
        );
    }
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    available: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(n(available)), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .input(2, paragraph.text_input(2));
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(1)],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &flow,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::Definite(n(available)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn assert_lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    wrapped: bool,
) {
    let placed = batch
        .placed_text()
        .iter()
        .find(|placed| placed.container() == 1)
        .unwrap();
    assert_eq!(placed.nodes(), &[2]);
    let layout = placed.layout();
    assert_eq!(layout.source().text(), &text::Text::from("AAA BBB"));
    let origin = paragraph.origin(2);
    assert_eq!(origin.view().source().text(), &text::Text::from("AAA BBB"));
    assert_eq!(origin.view().local_range(), text::Range::new(0, 7));
    let expected_lines = if wrapped { 2 } else { 1 };
    assert_eq!(
        layout.lines().len(),
        expected_lines,
        "actual word-wrapped Text line count"
    );
    let mut finalized: Vec<(usize, text::FinalizedLine)> = Vec::new();
    for entry in batch.final_inline_fragments() {
        if entry.owner() != &InlineFragmentOwner::Node(2) {
            continue;
        }
        let fragment = entry.fragment();
        let issued = fragment.finalized_fragment();
        assert_eq!(issued.view().origin(), Some(origin.clone()));
        let line = issued.finalized_line();
        if let Some((_, previous)) = finalized
            .iter()
            .find(|(index, _)| *index == fragment.line_index())
        {
            assert_eq!(
                previous, &line,
                "one actual finalized owner per selected line"
            );
        } else {
            finalized.push((fragment.line_index(), line));
        }
    }
    finalized.sort_by_key(|(index, _)| *index);
    assert_eq!(
        finalized.len(),
        expected_lines,
        "issued source-owned finalized lines"
    );
    let mut original_units = Vec::new();
    for (index, (ordinal, line)) in finalized.iter().enumerate() {
        assert_eq!(*ordinal, index);
        assert_eq!(line.prepared_text(), paragraph.prepared);
        let advance = if wrapped { 30.0 } else { 70.0 };
        assert_eq!(
            line.feedback().advance,
            advance,
            "line{index}: actual flow advance"
        );
        assert_eq!(
            line.measurements().fit().advance(),
            advance,
            "line{index}: canonical fit advance"
        );
        assert_eq!(
            line.feedback().line_height,
            10.0,
            "line{index}: registered-font line height"
        );
        let raw: f64 = line.run_views().map(|run| run.advance()).sum();
        assert_eq!(raw, advance, "line{index}: actual native run advances");
        assert_eq!(layout.lines()[index].baseline(), 8.0 + index as f64 * 10.0);
        let mut word_glyphs = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            for glyph in run.glyphs() {
                if glyph.advance() == 0.0 {
                    continue;
                }
                assert_eq!(
                    glyph.advance(),
                    10.0,
                    "line{index}: actual Ahem scalar advance"
                );
                let range = glyph.range();
                // A blank space need not publish a glyph; its original source
                // is retained independently in the actual selected mapping.
                if range != text::Range::new(3, 4) {
                    word_glyphs.push(range);
                }
            }
        }
        let expected = if wrapped && index == 0 {
            vec![0, 1, 2]
        } else if wrapped {
            vec![4, 5, 6]
        } else {
            vec![0, 1, 2, 4, 5, 6]
        };
        assert_eq!(
            word_glyphs,
            expected
                .into_iter()
                .map(|at| text::Range::new(at, at + 1))
                .collect::<Vec<_>>(),
            "line{index}: genuine AAA/BBB source-to-glyph correspondence"
        );
        for mapping in line.mapping_views() {
            let range = mapping.source_range();
            original_units.extend(range.start..range.end);
        }
    }
    original_units.sort_unstable();
    original_units.dedup();
    assert_eq!(
        original_units,
        (0..7).collect::<Vec<_>>(),
        "original separator custody survives wrapping"
    );
}

fn assert_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    available: f64,
    width: f64,
    height: f64,
) {
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            root.size.width,
            n(available),
            "{phase}: actual containing Block width"
        );
        let child = entries
            .iter()
            .find(|entry| entry.node() == 1)
            .unwrap()
            .output();
        assert_eq!(
            child.size,
            Size::new(n(width), n(height)),
            "{phase}: auto InlineBlock shrink-to-fit box at available{available}"
        );
        assert_eq!(
            (child.margin, child.padding, child.border),
            (Edges::ZERO, Edges::ZERO, Edges::ZERO)
        );
        assert_eq!(child.location.x, S::ZERO);
    }
}

fn shrink_to_fit_keeps_available_width<S: LayoutScalar>() {
    let paragraph = prepared_words();
    qualify_intrinsics::<S>(&paragraph);
    let wide = compute::<S>(&paragraph, 100.0);
    assert_lines(&wide, &paragraph, false);
    assert_box(&wide, 100.0, 70.0, 10.0);
    let narrow = compute::<S>(&paragraph, 20.0);
    assert_lines(&narrow, &paragraph, true);
    assert_box(&narrow, 20.0, 30.0, 20.0);
    let intermediate = compute::<S>(&paragraph, 50.0);
    assert_lines(&intermediate, &paragraph, true);
    // min(max(30, 50), 70)=50 even though both fitted lines advance30.
    assert_box(&intermediate, 50.0, 50.0, 20.0);
}

#[test]
fn auto_inline_block_shrink_to_fit_keeps_available_width_f32() {
    shrink_to_fit_keeps_available_width::<f32>();
}
#[test]
fn auto_inline_block_shrink_to_fit_keeps_available_width_f64() {
    shrink_to_fit_keeps_available_width::<f64>();
}
