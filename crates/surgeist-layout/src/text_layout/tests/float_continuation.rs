//! One native paragraph keeps its forced-line continuation across a float.
//!
//! CSS Text 3 ignored-boxes: an out-of-flow box introduces no break.
//! The actual preserved LF already supplies the one forced break. The float
//! changes a line's available band, not the paragraph's authored membership.

use super::*;

fn one_paragraph() -> PreparedParagraph {
    let style = ahem_style(10.0, 10.0);
    PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("A\n"), style.clone()),
            ParagraphPart::text(2, Source::new("X"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn tree<S: LayoutScalar>(
    first: &PreparedParagraph,
    second: &PreparedParagraph,
    with_float: bool,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(
            0,
            if with_float {
                vec![1, 9, 2]
            } else {
                vec![1, 2]
            },
        )
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, first.text_input(1))
        .input(2, second.text_input(2));
    if with_float {
        tree = tree.children(9, []).style(
            9,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(1.0)),
                    PreferredSizeOf::px(S::from_f64(1.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}

fn root_height<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> S {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output()
        .size
        .height
}

fn assert_one_paragraph<S: LayoutScalar>(with_float: bool) {
    let paragraph = one_paragraph();
    let origins: Vec<_> = paragraph.prepared.origin_views().collect();
    assert_eq!(origins.len(), 2);
    assert_eq!(origins[0].source().text().code_units(), &[65, 10]);
    assert_eq!(origins[1].source().text().code_units(), &[88]);
    assert_eq!(
        origins[0].aggregate_range(),
        surgeist_text::Range::new(0, 2)
    );
    assert_eq!(
        origins[1].aggregate_range(),
        surgeist_text::Range::new(2, 3)
    );
    // The registered real source already issues its mandatory LF boundary.
    // The float is not a Text atomic or a fabricated source-less break.
    assert_eq!(paragraph.prepared.atomic_views().count(), 0);
    assert_eq!(paragraph.prepared.control_views().count(), 0);
    let forced: Vec<_> = paragraph
        .prepared
        .opportunity_views()
        .filter(|view| view.kind() == surgeist_text::PreparedBreakKind::Forced)
        .collect();
    assert_eq!(forced.len(), 1);
    assert_eq!(forced[0].boundary().view().source_after().get(), 2);

    let tree = tree::<S>(&paragraph, &paragraph, with_float);
    let mut binding = paragraph.binding(0);
    if with_float {
        // This actual ignored box occurs between original offsets 2 and 2:
        // after the LF and before X. It has no invented Text participant.
        binding.nodes = vec![1, 9, 2];
    }
    assert_eq!(binding.origins[0].origin, paragraph.origin(1));
    assert_eq!(binding.origins[1].origin, paragraph.origin(2));
    assert_eq!(binding.prepared, paragraph.prepared);
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding]).unwrap_or_else(|error| {
        panic!("one prepared paragraph must remain admissible across an ignored float: {error:?}")
    });
    let batch = compute_layout_with_text(&capture, 0, request(40.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[65, 10, 88]);
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].range(), surgeist_text::Range::new(0, 2));
    assert_eq!(lines[1].range(), surgeist_text::Range::new(2, 3));
    // Independent Ahem10 metrics: ascent8, descent2, absolute line-height10.
    // A nonterminal preserved LF yields two lines, without a terminal empty.
    assert_eq!(lines[0].baseline(), 8.0);
    assert_eq!(lines[1].baseline(), 18.0);
    assert_eq!(lines[0].bounds().size.height, 10.0);
    assert_eq!(lines[1].bounds().size.height, 10.0);
    assert_eq!(root_height(&batch), S::from_f64(20.0));
    assert_eq!(
        placed
            .glyph_runs()
            .iter()
            .map(|run| run.advance())
            .sum::<f64>(),
        20.0
    );
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.finalized_line().prepared_text(),
            paragraph.prepared
        );
        assert!(matches!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(origin)
                if origin == paragraph.origin(1) || origin == paragraph.origin(2)
        ));
    }
    if with_float {
        let float = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 9)
            .unwrap()
            .output();
        assert_eq!(float.size, Size::new(S::from_f64(1.0), S::from_f64(1.0)));
    }
}

#[test]
fn native_one_paragraph_without_float_keeps_one_nonterminal_forced_break_f32() {
    assert_one_paragraph::<f32>(false);
}

#[test]
fn native_one_paragraph_without_float_keeps_one_nonterminal_forced_break_f64() {
    assert_one_paragraph::<f64>(false);
}

#[test]
fn native_one_paragraph_continues_after_forced_break_across_float_f32() {
    assert_one_paragraph::<f32>(true);
}

#[test]
fn native_one_paragraph_continues_after_forced_break_across_float_f64() {
    assert_one_paragraph::<f64>(true);
}

fn assert_two_explicit_paragraphs<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let first = PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(1, Source::new("A\n"), style.clone())],
        style.clone(),
        ParagraphBaseDirection::LeftToRight,
        surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
    );
    let second = PreparedParagraph::plain(2, Source::new("X"), style);
    assert_ne!(first.prepared, second.prepared);
    let tree = tree::<S>(&first, &second, true);
    let capture =
        TextLayoutInput::try_new(&tree, 0, vec![first.binding(0), second.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(40.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 2);
    let first_layout = batch.placed_text()[0].layout();
    let second_layout = batch.placed_text()[1].layout();
    assert_eq!(first_layout.source().text().code_units(), &[65, 10]);
    assert_eq!(second_layout.source().text().code_units(), &[88]);
    assert_eq!(first_layout.lines().len(), 2);
    assert_eq!(
        first_layout.lines()[0].range(),
        surgeist_text::Range::new(0, 2)
    );
    assert_eq!(
        first_layout.lines()[1].range(),
        surgeist_text::Range::new(2, 2)
    );
    assert_eq!(first_layout.lines()[0].baseline(), 8.0);
    assert_eq!(first_layout.lines()[1].baseline(), 18.0);
    assert_eq!(second_layout.lines().len(), 1);
    assert_eq!(
        second_layout.lines()[0].range(),
        surgeist_text::Range::new(0, 1)
    );
    assert_eq!(second_layout.lines()[0].baseline(), 28.0);
    assert_eq!(root_height(&batch), S::from_f64(30.0));
    // This is valid two-paragraph custody, not a request to erase the first
    // preparation's genuine terminal empty ParagraphEnd line.
}

#[test]
fn native_two_explicit_paragraphs_retain_the_terminal_empty_line_f32() {
    assert_two_explicit_paragraphs::<f32>();
}

#[test]
fn native_two_explicit_paragraphs_retain_the_terminal_empty_line_f64() {
    assert_two_explicit_paragraphs::<f64>();
}
