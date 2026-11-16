//! The early stop of a line search leaves the layout output unchanged.
//!
//! Each document lays out twice through `compute_layout_with_flow`, once as the layout runs and
//! once with every line search forced to visit its whole opportunity window, and the two
//! completed batches must be identical. The comparison does not depend on how a document wraps.

use super::*;
use surgeist_text as text;

/// One authored run of text with its own size and spacing.
#[derive(Clone, Copy)]
struct Span<'a> {
    source: &'a str,
    size: f32,
    letter_spacing: f64,
    word_spacing: f64,
}

const fn span(source: &str, size: f32, letter_spacing: f64, word_spacing: f64) -> Span<'_> {
    Span {
        source,
        size,
        letter_spacing,
        word_spacing,
    }
}

/// Block-level and paragraph-level settings of one document.
#[derive(Clone, Copy)]
struct Setup {
    width: f64,
    wrap_style: TextWrapStyle,
    text_align: TextAlign,
    text_align_last: TextAlignLast,
    hanging: text::HangingPunctuation,
}

impl Setup {
    const fn new(width: f64) -> Self {
        Self {
            width,
            wrap_style: TextWrapStyle::Auto,
            text_align: TextAlign::Start,
            text_align_last: TextAlignLast::Auto,
            hanging: text::HangingPunctuation::new(false, None, false),
        }
    }
}

/// The debug rendering of the completed batch of one block container of `spans`.
fn lay_out<S: LayoutScalar>(spans: &[Span], setup: Setup) -> String {
    let style = |span: &Span| text::Style {
        letter_spacing: span.letter_spacing,
        word_spacing: span.word_spacing,
        hanging_punctuation: setup.hanging,
        wrap: Wrap::Word,
        ..ahem_style(span.size, 40.0)
    };
    let parts = spans
        .iter()
        .zip(1..)
        .map(|(span, node)| ParagraphPart::text(node, Source::new(span.source), style(span)))
        .collect();
    let paragraph = PreparedParagraph::new(
        parts,
        text::Style {
            hanging_punctuation: setup.hanging,
            wrap: Wrap::Word,
            ..ahem_style(10.0, 40.0)
        },
        ParagraphBaseDirection::LeftToRight,
    );
    let binding = paragraph.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(setup.width)),
                    PreferredSizeOf::AUTO,
                ),
                text_wrap_style: setup.wrap_style,
                text_align: setup.text_align,
                text_align_last: setup.text_align_last,
                ..NodeInputOf::default()
            },
        );
    for node in &binding.nodes {
        tree = tree
            .children(*node, [])
            .input(*node, paragraph.text_input(*node));
    }
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
    let batch = compute_layout_with_flow(
        &capture,
        0,
        request(setup.width, {
            let root_input = crate::LayoutTree::node_input((capture).tree(), 0);
            crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
        }),
    )
    .unwrap();
    format!("{batch:?}")
}

fn assert_same_with_full_walk<S: LayoutScalar>(label: &str, spans: &[Span], setup: Setup) {
    let early = lay_out::<S>(spans, setup);
    let full = fit::with_full_walk(|| lay_out::<S>(spans, setup));
    assert!(
        early == full,
        "{label}: the early stop changed the layout at width {}",
        setup.width
    );
}

fn assert_same_in_both_scalars(label: &str, spans: &[Span], widths: &[f64]) {
    for &width in widths {
        assert_same_with_full_walk::<f32>(label, spans, Setup::new(width));
        assert_same_with_full_walk::<f64>(label, spans, Setup::new(width));
    }
}

const WIDTHS: [f64; 5] = [30.0, 45.0, 60.0, 90.0, 140.0];

#[test]
fn mixed_size_spans_with_negative_word_spacing_lay_out_the_same_with_the_early_stop() {
    for word_spacing in [0.0, -5.0, -15.0, -25.0] {
        let spans = [
            span("XX ", 10.0, 0.0, word_spacing),
            span("XXXX ", 20.0, 0.0, word_spacing),
            span("X ", 10.0, 0.0, word_spacing),
            span("XXX XX ", 20.0, 0.0, word_spacing),
            span("X XXXXXX X ", 10.0, 0.0, word_spacing),
            span("XX ", 20.0, 0.0, word_spacing),
            span("X", 10.0, 0.0, word_spacing),
        ];
        assert_same_in_both_scalars("mixed sizes, word spacing", &spans, &WIDTHS);
    }
}

#[test]
fn mixed_size_spans_with_negative_letter_spacing_lay_out_the_same_with_the_early_stop() {
    for letter_spacing in [0.0, -2.0, -4.0, -9.0, -12.0] {
        let spans = [
            span("XXX ", 10.0, letter_spacing, 0.0),
            span("XX ", 20.0, letter_spacing, 0.0),
            span("XXXXX ", 10.0, letter_spacing, 0.0),
            span("X ", 20.0, letter_spacing, 0.0),
            span("XXX XX ", 10.0, letter_spacing, 0.0),
            span("XX", 20.0, letter_spacing, 0.0),
        ];
        assert_same_in_both_scalars("mixed sizes, letter spacing", &spans, &WIDTHS);
    }
}

#[test]
fn a_small_last_word_after_a_large_one_lays_out_the_same_with_the_early_stop() {
    // The last glyph of a group is its own edge in every candidate that contains it, so any
    // spacing trimmed at that edge is the case where a later candidate could be narrower.
    for spacing in [-1.0, -3.0, -5.0, -9.0] {
        let spans = [
            span("X", 10.0, spacing, 0.0),
            span(" ", 5.0, spacing, 0.0),
            span("X", 40.0, spacing, 0.0),
            span(" ", 5.0, spacing, 0.0),
            span("X", 2.0, spacing, 0.0),
        ];
        assert_same_in_both_scalars("small last word", &spans, &[41.0, 42.0, 43.0, 44.0, 46.0]);
    }
    for spacing in [-10.0, -16.0, -18.0, -20.0, -25.0] {
        let spans = [
            span("X ", 10.0, 0.0, spacing),
            span("X", 40.0, 0.0, spacing),
            span(" ", 10.0, 0.0, spacing),
            span("X", 5.0, 0.0, spacing),
        ];
        assert_same_in_both_scalars("small last word", &spans, &[38.0, 39.0, 40.0, 41.0, 44.0]);
    }
}

#[test]
fn the_documents_with_single_span_negative_spacing_lay_out_the_same_with_the_early_stop() {
    let three = "XXX ".repeat(50);
    let four = "XXXX ".repeat(50);
    let shrinking = format!("XX XXXXXXXXXX {}", "X ".repeat(40));
    for (label, spans) in [
        ("letter -2", [span(&three, 10.0, -2.0, 0.0)]),
        ("word -15", [span(&three, 10.0, 0.0, -15.0)]),
        ("word -25 three", [span(&three, 10.0, 0.0, -25.0)]),
        ("word -40", [span(&three, 10.0, 0.0, -40.0)]),
        ("word -25 four", [span(&four, 10.0, 0.0, -25.0)]),
        ("word -25 shrinking", [span(&shrinking, 10.0, 0.0, -25.0)]),
    ] {
        assert_same_in_both_scalars(label, &spans, &[100.0]);
    }
}

#[test]
fn first_and_last_line_roles_do_not_change_with_the_early_stop() {
    // Hanging punctuation on the first and last formatted lines and the alignment of the last
    // line make each line's measure depend on whether it is first or last, so the role rows
    // that skipped candidates no longer contribute must not matter.
    let text: &'static str =
        "\u{201c}XX XX XX. XX XXX XX XXXX XX XX. XX XXX XX XX XXXX XX.\u{201d}";
    let spans = [span(text, 10.0, 0.0, 0.0)];
    let hanging = [
        (false, None, false),
        (true, None, true),
        (true, Some(text::HangingPunctuationEnd::ForceEnd), false),
        (false, Some(text::HangingPunctuationEnd::AllowEnd), true),
    ];
    let aligns = [
        (TextAlign::Start, TextAlignLast::Auto),
        (TextAlign::Start, TextAlignLast::Center),
        (TextAlign::Justify, TextAlignLast::Auto),
        (TextAlign::Justify, TextAlignLast::Justify),
        (TextAlign::Center, TextAlignLast::End),
    ];
    for (first, end, last) in hanging {
        for (text_align, text_align_last) in aligns {
            for width in [70.0, 130.0] {
                let setup = Setup {
                    text_align,
                    text_align_last,
                    hanging: text::HangingPunctuation::new(first, end, last),
                    ..Setup::new(width)
                };
                assert_same_with_full_walk::<f32>("roles", &spans, setup);
                assert_same_with_full_walk::<f64>("roles", &spans, setup);
            }
        }
    }
}
