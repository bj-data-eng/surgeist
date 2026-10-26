//! Actual native Text fitting against a real shortened float band.
//!
//! CSS2 visuren, retained capture line 628, requires moving a shortened line
//! downward until some content fits or no floats remain. Ahem's retained
//! UPEM 1000/advance 1000/ascent 800/descent -200 gives literal 10px X advances
//! and an 8px baseline at size 10/absolute line height 10.

use super::*;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn native_batch<S: LayoutScalar>(
    source: &str,
    width: f64,
    with_float: bool,
) -> (PreparedParagraph, CompletedLayoutBatchOf<u32, S>) {
    let paragraph = PreparedParagraph::plain(
        1,
        Source::identified(source, surgeist_text::Id::from_u64(42), 7),
        ahem_style(10.0, 10.0),
    );
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, if with_float { vec![2, 1] } else { vec![1] })
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    if with_float {
        tree = tree.children(2, []).style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(80.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(width)).unwrap();
    if with_float {
        let float = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(float.location, Point::ZERO);
        assert_eq!(float.size, Size::new(S::from_f64(80.0), S::from_f64(20.0)));
        assert_eq!(float.margin, Edges::ZERO);
    }
    (paragraph, batch)
}

fn assert_actual_text<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    source: &str,
    expected_advance: f64,
) {
    let placed = batch.placed_text()[0].layout();
    assert_eq!(batch.placed_text()[0].nodes(), &[1]);
    assert_eq!(
        placed.source().text().code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(placed.lines().len(), 1);
    assert_eq!(
        placed.lines()[0].range(),
        surgeist_text::Range::new(0, source.len())
    );
    let origin = paragraph.origin(1);
    assert_eq!(
        origin.view().source().id(),
        Some(surgeist_text::Id::from_u64(42))
    );
    assert_eq!(origin.view().source().revision(), 7);
    let mut lines = Vec::new();
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            surgeist_text::FinalizedFragmentOwner::Origin(origin.clone())
        );
        let line = fragment.finalized_line();
        let feedback = line.feedback();
        assert_eq!(
            (feedback.advance, feedback.baseline, feedback.line_height),
            (expected_advance, 8.0, 10.0)
        );
        assert!(line.selected_line().request().atomics.is_empty());
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    assert_eq!(lines.len(), 1);
    let mut runs = placed.run_views();
    let run = runs.next().unwrap();
    assert!(runs.next().is_none());
    assert_eq!(run.font_size(), 10.0);
    assert_eq!(run.face().bytes(), AHEM);
    let glyphs = run.glyphs().collect::<Vec<_>>();
    assert_eq!(glyphs.len(), source.len());
    for (index, glyph) in glyphs.into_iter().enumerate() {
        assert_eq!(glyph.id(), 58);
        assert_eq!(glyph.advance(), 10.0);
        assert_eq!(glyph.x(), 0.0);
        assert_eq!(glyph.y(), 0.0);
        assert_eq!(glyph.pen(), index as f64 * 10.0);
    }
}

fn assert_moves_below_float<S: LayoutScalar>(source: &str, expected_advance: f64) {
    let (paragraph, batch) = native_batch::<S>(source, 100.0, true);
    assert_actual_text(&batch, &paragraph, source, expected_advance);
    let placed = batch.placed_text()[0].layout();
    let line = &placed.lines()[0];
    // The actual 80px float leaves only 20px. No part of this indivisible
    // word fits there, so the real float end 20 determines the next line.
    // Once no float remains, even the 120px word may legitimately overflow 100.
    assert_eq!(
        line.bounds(),
        surgeist_text::Rect::new(0.0, 20.0, 100.0, 10.0)
    );
    assert_eq!(line.baseline(), 28.0);
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size.height, S::from_f64(30.0));
}

#[test]
fn indivisible_word_moves_below_nonzero_shortened_float_band_f32() {
    assert_moves_below_float::<f32>("XXXX", 40.0);
}

#[test]
fn indivisible_word_moves_below_nonzero_shortened_float_band_f64() {
    assert_moves_below_float::<f64>("XXXX", 40.0);
}

#[test]
fn overwide_word_clears_actual_floats_before_float_free_overflow_f32() {
    assert_moves_below_float::<f32>("XXXXXXXXXXXX", 120.0);
}

#[test]
fn overwide_word_clears_actual_floats_before_float_free_overflow_f64() {
    assert_moves_below_float::<f64>("XXXXXXXXXXXX", 120.0);
}

fn assert_fitting_and_float_free_controls<S: LayoutScalar>() {
    let (paragraph, batch) = native_batch::<S>("X", 100.0, true);
    assert_actual_text(&batch, &paragraph, "X", 10.0);
    let line = &batch.placed_text()[0].layout().lines()[0];
    assert_eq!(
        line.bounds(),
        surgeist_text::Rect::new(80.0, 0.0, 20.0, 10.0)
    );
    assert_eq!(line.baseline(), 8.0);

    let (paragraph, batch) = native_batch::<S>("XXXX", 20.0, false);
    assert_actual_text(&batch, &paragraph, "XXXX", 40.0);
    let line = &batch.placed_text()[0].layout().lines()[0];
    assert_eq!(
        line.bounds(),
        surgeist_text::Rect::new(0.0, 0.0, 20.0, 10.0)
    );
    assert_eq!(line.baseline(), 8.0);
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size.height, S::from_f64(10.0));
    assert_eq!(
        root.scroll_geometry
            .unwrap()
            .scrollable_overflow()
            .size()
            .width,
        S::from_f64(40.0)
    );
}

#[test]
fn fitting_word_stays_beside_float_and_float_free_overflow_never_fabricates_clearance() {
    assert_fitting_and_float_free_controls::<f32>();
    assert_fitting_and_float_free_controls::<f64>();
}

fn intrinsic_root_batch<S: LayoutScalar>(
    available: AvailableOf<S>,
) -> (PreparedParagraph, CompletedLayoutBatchOf<u32, S>) {
    let paragraph = PreparedParagraph::plain(
        1,
        Source::identified("XXXX", surgeist_text::Id::from_u64(42), 7),
        ahem_style(10.0, 10.0),
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [2, 1])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(80.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
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
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    // This is the actual root PerformLayout front with both preferred axes
    // AUTO. No separate ComputeSize or caller-provided known width is used.
    let request =
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
    let float = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .unwrap()
        .output();
    assert_eq!(float.location, Point::ZERO);
    assert_eq!(float.size, Size::new(S::from_f64(80.0), S::from_f64(20.0)));
    assert_eq!(float.margin, Edges::ZERO);
    (paragraph, batch)
}

fn assert_min_content_root<S: LayoutScalar>() {
    let (paragraph, batch) = intrinsic_root_batch::<S>(AvailableOf::MIN_CONTENT);
    assert_actual_text(&batch, &paragraph, "XXXX", 40.0);
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    // A min-content box admits the 80px float and indivisible 40px word
    // separately, rather than requiring their 120px side-by-side sum.
    assert_eq!(root.size, Size::new(S::from_f64(80.0), S::from_f64(30.0)));
    let line = &batch.placed_text()[0].layout().lines()[0];
    assert_eq!(line.bounds().origin, surgeist_text::Point::new(0.0, 20.0));
    assert_eq!(line.baseline(), 28.0);
    assert_eq!(line.bounds().size.height, 10.0);
}

fn assert_max_content_root<S: LayoutScalar>() {
    let (paragraph, batch) = intrinsic_root_batch::<S>(AvailableOf::MAX_CONTENT);
    assert_actual_text(&batch, &paragraph, "XXXX", 40.0);
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    // With no soft wrapping imposed, max-content retains the actual float's
    // 80px followed by the word's 40px. Its 20px float determines the height.
    assert_eq!(root.size, Size::new(S::from_f64(120.0), S::from_f64(20.0)));
    let line = &batch.placed_text()[0].layout().lines()[0];
    assert_eq!(line.bounds().origin, surgeist_text::Point::new(80.0, 0.0));
    assert_eq!(line.baseline(), 8.0);
    assert_eq!(line.bounds().size.height, 10.0);
}

#[test]
fn min_content_auto_root_moves_word_below_actual_float_f32() {
    assert_min_content_root::<f32>();
}

#[test]
fn min_content_auto_root_moves_word_below_actual_float_f64() {
    assert_min_content_root::<f64>();
}

#[test]
fn max_content_auto_root_keeps_word_beside_actual_float_f32() {
    assert_max_content_root::<f32>();
}

#[test]
fn max_content_auto_root_keeps_word_beside_actual_float_f64() {
    assert_max_content_root::<f64>();
}
