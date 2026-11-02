//! Existing native overflow-wrap fields through real preparation and Layout.
//!
//! Ahem at10px supplies a10px advance,8px ascent and2px descent. The positive
//! cases require real Soft EGC cuts; no authored separator or forced control
//! substitutes for conditional permission. Float priority is the explicitly
//! adopted Surgeist ordering, not a universal browser-coordinate oracle.
use super::*;
use surgeist_text::{FinalizedLine, LineEnd, OverflowWrap, Range, Wrap};

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Debug, PartialEq)]
struct LineFact {
    source: Range,
    end: LineEnd,
    advance: f64,
    occupied: (f64, f64),
    allocation: surgeist_text::Rect,
    baseline: f64,
}

fn line(
    start: usize,
    end: usize,
    advance: f64,
    width: f64,
    x: f64,
    top: f64,
    soft: bool,
) -> LineFact {
    LineFact {
        source: Range::new(start, end),
        end: if soft {
            LineEnd::Soft
        } else {
            LineEnd::Paragraph
        },
        advance,
        occupied: (0.0, advance),
        allocation: surgeist_text::Rect::new(x, top, width, 10.0),
        baseline: top + 8.0,
    }
}

fn facts<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    prepared: &surgeist_text::PreparedText,
    source: &str,
) -> Vec<LineFact> {
    assert_eq!(batch.placed_text().len(), 1);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    let mut owners: Vec<Option<FinalizedLine>> = vec![None; layout.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let actual = fragment.finalized_fragment().finalized_line();
        assert_eq!(actual.prepared_text(), *prepared);
        if let Some(previous) = &owners[fragment.line_index()] {
            assert_eq!(*previous, actual);
        }
        owners[fragment.line_index()] = Some(actual);
    }
    // This fixture deliberately has only one-glyph ASCII A/X/SPACE inputs,
    // with no combining sequence, ligature feature or discarded whitespace.
    assert!(
        source
            .bytes()
            .all(|value| matches!(value, b'A' | b'X' | b' '))
    );
    let mut glyph_count = 0;
    for run in layout.run_views() {
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.face().bytes(), AHEM);
        // Every displayed A/X is an actual one-em provider glyph. SPACE has
        // the same real advance, without requiring nonempty paint ink.
        for glyph in run.glyphs() {
            glyph_count += 1;
            assert_eq!(glyph.advance(), 10.0);
        }
    }
    assert!(
        glyph_count > 0,
        "the registered provider must emit actual authored glyphs"
    );
    assert_eq!(
        glyph_count,
        source.len(),
        "one real Ahem glyph per authored ASCII unit, including preserved SPACE"
    );
    layout
        .lines()
        .iter()
        .zip(owners)
        .map(|(placed, owner)| {
            let owner =
                owner.expect("each authored nonempty Ahem line has an authentic origin fragment");
            let feedback = owner.feedback();
            assert_eq!((feedback.baseline, feedback.line_height), (8.0, 10.0));
            LineFact {
                source: placed.range(),
                end: owner.selected_line().request().context.end,
                advance: feedback.advance,
                occupied: (feedback.occupied_start, feedback.occupied_end),
                allocation: placed.bounds(),
                baseline: placed.baseline(),
            }
        })
        .collect()
}

fn paragraph<S: LayoutScalar>(
    source: &str,
    overflow: OverflowWrap,
    wrap: Wrap,
    available: AvailableOf<S>,
) -> (f64, Vec<LineFact>) {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = overflow;
    style.wrap = wrap;
    let paragraph = PreparedParagraph::plain(
        1,
        Source::identified(source, surgeist_text::Id::from_u64(42), 7),
        style,
    );
    let width = match available {
        AvailableOf::Definite(value) => PreferredSizeOf::px(value),
        _ => PreferredSizeOf::AUTO,
    };
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(width, PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let request =
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
    let width = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output()
        .size
        .width
        .to_f64();
    (width, facts(&batch, &paragraph.prepared, source))
}
fn expected_twenty() -> (f64, Vec<LineFact>) {
    (
        20.0,
        vec![
            line(0, 2, 20.0, 20.0, 0.0, 0.0, true),
            line(2, 4, 20.0, 20.0, 0.0, 10.0, false),
        ],
    )
}
#[test]
fn native_anywhere_wraps_the_actual_aaaa_into_two_fitting_lines() {
    let actual = (
        paragraph::<f32>(
            "AAAA",
            OverflowWrap::Anywhere,
            Wrap::Word,
            AvailableOf::definite(20.0),
        ),
        paragraph::<f64>(
            "AAAA",
            OverflowWrap::Anywhere,
            Wrap::Word,
            AvailableOf::definite(20.0),
        ),
    );
    assert_eq!(actual, (expected_twenty(), expected_twenty()));
}
#[test]
fn native_break_word_wraps_the_actual_aaaa_into_two_fitting_lines() {
    let actual = (
        paragraph::<f32>(
            "AAAA",
            OverflowWrap::BreakWord,
            Wrap::Word,
            AvailableOf::definite(20.0),
        ),
        paragraph::<f64>(
            "AAAA",
            OverflowWrap::BreakWord,
            Wrap::Word,
            AvailableOf::definite(20.0),
        ),
    );
    assert_eq!(actual, (expected_twenty(), expected_twenty()));
}
#[test]
fn native_fitting_ordinary_endpoints_win_over_possible_conditional_cuts() {
    for overflow in [OverflowWrap::Anywhere, OverflowWrap::BreakWord] {
        let at40 = (
            40.0,
            vec![
                line(0, 3, 30.0, 40.0, 0.0, 0.0, true),
                line(3, 5, 20.0, 40.0, 0.0, 10.0, false),
            ],
        );
        let at50 = (50.0, vec![line(0, 5, 50.0, 50.0, 0.0, 0.0, false)]);
        let actual = [
            paragraph::<f32>("AA AA", overflow, Wrap::Word, AvailableOf::definite(40.0)),
            paragraph::<f64>("AA AA", overflow, Wrap::Word, AvailableOf::definite(40.0)),
            paragraph::<f32>("AA AA", overflow, Wrap::Word, AvailableOf::definite(50.0)),
            paragraph::<f64>("AA AA", overflow, Wrap::Word, AvailableOf::definite(50.0)),
        ];
        assert_eq!(actual, [at40.clone(), at40, at50.clone(), at50]);
    }
}
#[test]
fn native_anywhere_min_content_uses_actual_ten_pixel_pieces() {
    let expected = (
        10.0,
        (0..4)
            .map(|i| line(i, i + 1, 10.0, 10.0, 0.0, i as f64 * 10.0, i != 3))
            .collect::<Vec<_>>(),
    );
    let actual = (
        paragraph::<f32>(
            "AAAA",
            OverflowWrap::Anywhere,
            Wrap::Word,
            AvailableOf::MIN_CONTENT,
        ),
        paragraph::<f64>(
            "AAAA",
            OverflowWrap::Anywhere,
            Wrap::Word,
            AvailableOf::MIN_CONTENT,
        ),
    );
    assert_eq!(actual, (expected.clone(), expected));
}
#[test]
fn native_break_word_min_content_retains_the_ordinary_forty_pixel_word() {
    let expected = (40.0, vec![line(0, 4, 40.0, 40.0, 0.0, 0.0, false)]);
    let actual = (
        paragraph::<f32>(
            "AAAA",
            OverflowWrap::BreakWord,
            Wrap::Word,
            AvailableOf::MIN_CONTENT,
        ),
        paragraph::<f64>(
            "AAAA",
            OverflowWrap::BreakWord,
            Wrap::Word,
            AvailableOf::MIN_CONTENT,
        ),
    );
    assert_eq!(actual, (expected.clone(), expected));
}

#[test]
fn native_hyphen_policy_and_empty_indication_preserve_intrinsic_wrap_distinctions() {
    fn check<S: LayoutScalar>() {
        for (hyphens, ordinary_minimum) in [
            (surgeist_text::Hyphens::Manual, 30.0),
            (surgeist_text::Hyphens::None, 50.0),
        ] {
            for (overflow, minimum) in [
                (OverflowWrap::Normal, ordinary_minimum),
                (OverflowWrap::BreakWord, ordinary_minimum),
                (OverflowWrap::Anywhere, 10.0),
            ] {
                let mut style = ahem_style(10.0, 10.0);
                style.hyphens = hyphens;
                style.hyphenate_character =
                    surgeist_text::HyphenateCharacter::String(surgeist_text::Text::from(""));
                style.overflow_wrap = overflow;
                let short = PreparedParagraph::plain(1, Source::new("AA\u{ad}AA"), style.clone());
                assert_eq!(
                    short.prepared.alternative_views().count(),
                    0,
                    "short source remains suppressed"
                );
                let paragraph = PreparedParagraph::plain(1, Source::new("AX\u{ad}XXX"), style);
                let alternatives: Vec<_> = paragraph.prepared.alternative_views().collect();
                assert_eq!(
                    alternatives.len(),
                    usize::from(hyphens == surgeist_text::Hyphens::Manual)
                );
                assert!(
                    alternatives
                        .iter()
                        .all(|alternative| alternative.text().is_empty())
                );

                let tree = PublicLayoutTreeOf::<S>::new()
                    .children(0, [1])
                    .children(1, [])
                    .style(
                        0,
                        NodeInputOf {
                            display: Display::Block,
                            ..NodeInputOf::default()
                        },
                    )
                    .input(1, paragraph.text_input(1));
                let capture = LayoutFlowInput::try_new(
                    &tree,
                    0,
                    vec![paragraph.binding(0)],
                    crate::WrappingInput {
                        native_inline_boxes: Vec::new(),
                        gaps: Vec::new(),
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                )
                .unwrap();
                // Five actual Ahem glyphs are 50px unbroken; manual's empty
                // indication splits into 20px and 30px pieces. Only Anywhere admits
                // the 10px emergency pieces into min-content.
                for (available, expected) in [
                    (AvailableOf::MIN_CONTENT, minimum),
                    (AvailableOf::MAX_CONTENT, 50.0),
                ] {
                    let request = LayoutRootRequestOf::viewport(Size::new(
                        available,
                        AvailableOf::MAX_CONTENT,
                    ))
                    .unwrap();
                    let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
                    let width = batch
                        .final_entries()
                        .iter()
                        .find(|entry| entry.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64();
                    assert_eq!(width, expected, "{hyphens:?}/{overflow:?}/{available:?}");
                    assert_eq!(
                        batch.placed_text()[0].layout().source().text().code_units(),
                        [65, 88, 173, 88, 88, 88]
                    );
                }
            }
        }
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn native_max_content_keeps_both_overflow_wrap_modes_unbroken() {
    let expected = (40.0, vec![line(0, 4, 40.0, 40.0, 0.0, 0.0, false)]);
    for overflow in [OverflowWrap::Anywhere, OverflowWrap::BreakWord] {
        let actual = (
            paragraph::<f32>("AAAA", overflow, Wrap::Word, AvailableOf::MAX_CONTENT),
            paragraph::<f64>("AAAA", overflow, Wrap::Word, AvailableOf::MAX_CONTENT),
        );
        assert_eq!(actual, (expected.clone(), expected.clone()));
    }
}
#[test]
fn native_nowrap_does_not_acquire_conditional_permission() {
    let expected = (20.0, vec![line(0, 4, 40.0, 20.0, 0.0, 0.0, false)]);
    for overflow in [OverflowWrap::Anywhere, OverflowWrap::BreakWord] {
        let actual = (
            paragraph::<f32>("AAAA", overflow, Wrap::None, AvailableOf::definite(20.0)),
            paragraph::<f64>("AAAA", overflow, Wrap::None, AvailableOf::definite(20.0)),
        );
        assert_eq!(actual, (expected.clone(), expected.clone()));
    }
}

fn interleaved<S: LayoutScalar>(
    overflow: OverflowWrap,
    float_width: f64,
) -> (Vec<LineFact>, Point<f64>) {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = overflow;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("A"), style.clone()),
            ParagraphPart::text(3, Source::new("X"), style.clone()),
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(0);
    binding.nodes = vec![1, 2, 3];
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(float_width)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .input(3, paragraph.text_input(3));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(20.0)).unwrap();
    let location = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 2)
        .unwrap()
        .output()
        .location;
    (
        facts(&batch, &paragraph.prepared, "AX"),
        Point::new(location.x.to_f64(), location.y.to_f64()),
    )
}
#[test]
fn native_adopted_conditional_order_keeps_the_higher_legal_soft_tied_float() {
    // Adopted Surgeist priority: current highest legal assignment, ordinary
    // exhaustion, conditional fit, then optional whole-candidate deferral.
    let expected = (
        vec![
            line(0, 1, 10.0, 12.0, 8.0, 0.0, true),
            line(1, 2, 10.0, 20.0, 0.0, 10.0, false),
        ],
        Point::ZERO,
    );
    let actual = (
        interleaved::<f32>(OverflowWrap::Anywhere, 8.0),
        interleaved::<f64>(OverflowWrap::Anywhere, 8.0),
    );
    assert_eq!(actual, (expected.clone(), expected));
}
#[test]
fn native_mandatory_float_deferral_precedes_conditional_search() {
    // A10 plus float15 cannot share20 even at the seam; the actual float
    // must already be below the preceding line, where ordinary AX20 fits.
    let expected = (
        vec![line(0, 2, 20.0, 20.0, 0.0, 0.0, false)],
        Point::new(0.0, 10.0),
    );
    let actual = (
        interleaved::<f32>(OverflowWrap::Anywhere, 15.0),
        interleaved::<f64>(OverflowWrap::Anywhere, 15.0),
    );
    assert_eq!(actual, (expected.clone(), expected));
}
#[test]
fn native_no_conditional_permission_retains_the_existing_float_deferral() {
    let expected = (
        vec![line(0, 2, 20.0, 20.0, 0.0, 0.0, false)],
        Point::new(0.0, 10.0),
    );
    let actual = (
        interleaved::<f32>(OverflowWrap::Normal, 8.0),
        interleaved::<f64>(OverflowWrap::Normal, 8.0),
    );
    assert_eq!(actual, (expected.clone(), expected));
}

#[path = "conditional_progress.rs"]
mod progress;
