//! The early stop of a line search leaves the layout output unchanged.
//!
//! Each document lays out twice through `compute_layout_with_flow`, once as the layout runs and
//! once with every line search forced to visit its whole opportunity window, and the two
//! completed batches must be identical. The comparison does not depend on how a document wraps.

use super::*;
use crate::accounting::{LayoutWork, TestHost};
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
    rtl: bool,
    white_space: text::style::WhiteSpace,
    wrap_style: TextWrapStyle,
    text_align: TextAlign,
    text_align_last: TextAlignLast,
    justification: TextJustification,
    hanging: text::style::HangingPunctuation,
}

impl Setup {
    const fn new(width: f64) -> Self {
        Self {
            width,
            rtl: false,
            white_space: text::style::WhiteSpace::Collapse,
            wrap_style: TextWrapStyle::Auto,
            text_align: TextAlign::Start,
            text_align_last: TextAlignLast::Auto,
            justification: TextJustification::None,
            hanging: text::style::HangingPunctuation::new(false, None, false),
        }
    }

    /// Justified text, each line stretched to the block width through the inter-word spaces.
    const fn justified(self) -> Self {
        Self {
            text_align: TextAlign::Justify,
            text_align_last: TextAlignLast::Justify,
            justification: TextJustification::InterWord,
            ..self
        }
    }

    const fn right_to_left(self) -> Self {
        Self { rtl: true, ..self }
    }

    const fn white_space(self, white_space: text::style::WhiteSpace) -> Self {
        Self {
            white_space,
            ..self
        }
    }
}

/// The debug rendering of the completed batch, or of the refusal, of one block container of
/// `spans`, reporting the call's work to `host`.
fn lay_out<S: LayoutScalar>(spans: &[Span], setup: Setup, host: &TestHost) -> String {
    let style = |span: &Span| text::style::Style {
        letter_spacing: span.letter_spacing,
        word_spacing: span.word_spacing,
        hanging_punctuation: setup.hanging,
        white_space: setup.white_space,
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
        text::style::Style {
            hanging_punctuation: setup.hanging,
            white_space: setup.white_space,
            wrap: Wrap::Word,
            ..ahem_style(10.0, 40.0)
        },
        if setup.rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    );
    let binding = paragraph.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if setup.rtl {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                },
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(setup.width)),
                    PreferredSizeOf::AUTO,
                ),
                text_wrap_style: setup.wrap_style,
                text_align: setup.text_align,
                text_align_last: setup.text_align_last,
                text_justification: setup.justification,
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
        crate::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = crate::engine::compute_layout_with_flow_invalidated_accounted(
        &capture,
        0,
        request(setup.width, {
            let root_input = crate::tree::LayoutTree::node_input((capture).tree(), 0);
            crate::geometry::FlowAxes::new(root_input.writing_mode, root_input.direction)
        }),
        &[],
        Some(&host.scope()),
    );
    // A refusal is part of the result: both walks must refuse the same way.
    match batch {
        Ok(batch) => format!("{batch:?}"),
        Err(error) => format!("refused: {error:?}"),
    }
}

/// Lays out one document with the early stop and with the full walk and returns how many
/// candidates the early stop saved, so a test can show that the comparison exercised the stop.
/// Both walks make the same line searches, and the early stop never evaluates more candidates.
fn assert_same_with_full_walk<S: LayoutScalar>(label: &str, spans: &[Span], setup: Setup) -> u64 {
    let early_host = TestHost::new();
    let early = lay_out::<S>(spans, setup, &early_host);
    let full_host = TestHost::new();
    let full = fit::with_full_walk(|| lay_out::<S>(spans, setup, &full_host));
    assert!(
        early == full,
        "{label}: the early stop changed the layout at width {}: early {:.300} full {:.300}",
        setup.width,
        early,
        full
    );
    assert_eq!(
        early_host.work(LayoutWork::LineSearch),
        full_host.work(LayoutWork::LineSearch),
        "{label}: the early stop changed the line searches at width {}",
        setup.width
    );
    full_host
        .work(LayoutWork::Candidate)
        .checked_sub(early_host.work(LayoutWork::Candidate))
        .unwrap_or_else(|| {
            panic!(
                "{label}: the early stop evaluated more candidates than the full walk at width {}",
                setup.width
            )
        })
}

/// The same comparison in both scalar types over `widths`; the candidates the early stop saved.
fn assert_same_in_both_scalars(label: &str, spans: &[Span], widths: &[f64], setup: Setup) -> u64 {
    widths
        .iter()
        .map(|&width| {
            let setup = Setup { width, ..setup };
            assert_same_with_full_walk::<f32>(label, spans, setup)
                + assert_same_with_full_walk::<f64>(label, spans, setup)
        })
        .sum()
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
        let saved = assert_same_in_both_scalars(
            "mixed sizes, word spacing",
            &spans,
            &WIDTHS,
            Setup::new(0.0),
        );
        // Negative spacing vetoes the stop, so only the document without it must stop.
        assert!(
            word_spacing < 0.0 || saved > 0,
            "no line search stopped early"
        );
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
        let saved = assert_same_in_both_scalars(
            "mixed sizes, letter spacing",
            &spans,
            &WIDTHS,
            Setup::new(0.0),
        );
        assert!(
            letter_spacing < 0.0 || saved > 0,
            "no line search stopped early"
        );
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
        assert_same_in_both_scalars(
            "small last word",
            &spans,
            &[41.0, 42.0, 43.0, 44.0, 46.0],
            Setup::new(0.0),
        );
    }
    for spacing in [-10.0, -16.0, -18.0, -20.0, -25.0] {
        let spans = [
            span("X ", 10.0, 0.0, spacing),
            span("X", 40.0, 0.0, spacing),
            span(" ", 10.0, 0.0, spacing),
            span("X", 5.0, 0.0, spacing),
        ];
        assert_same_in_both_scalars(
            "small last word",
            &spans,
            &[38.0, 39.0, 40.0, 41.0, 44.0],
            Setup::new(0.0),
        );
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
        assert_same_in_both_scalars(label, &spans, &[100.0], Setup::new(0.0));
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
        (
            true,
            Some(text::style::HangingPunctuationEnd::ForceEnd),
            false,
        ),
        (
            false,
            Some(text::style::HangingPunctuationEnd::AllowEnd),
            true,
        ),
    ];
    let aligns = [
        (TextAlign::Start, TextAlignLast::Auto),
        (TextAlign::Start, TextAlignLast::Center),
        (TextAlign::Justify, TextAlignLast::Auto),
        (TextAlign::Justify, TextAlignLast::Justify),
        (TextAlign::Center, TextAlignLast::End),
    ];
    let mut saved = 0;
    for (first, end, last) in hanging {
        for (text_align, text_align_last) in aligns {
            for width in [70.0, 130.0] {
                let setup = Setup {
                    text_align,
                    text_align_last,
                    hanging: text::style::HangingPunctuation::new(first, end, last),
                    ..Setup::new(width)
                };
                saved += assert_same_with_full_walk::<f32>("roles", &spans, setup);
                saved += assert_same_with_full_walk::<f64>("roles", &spans, setup);
            }
        }
    }
    assert!(saved > 0, "no line search stopped early");
}

/// The block settings of a justified or ragged document of one grid cell.
fn grid_setups(width: f64) -> Vec<Setup> {
    let mut setups = Vec::new();
    for rtl in [false, true] {
        for white_space in [
            text::style::WhiteSpace::Collapse,
            text::style::WhiteSpace::Preserve,
            text::style::WhiteSpace::BreakSpaces,
        ] {
            let base = Setup::new(width).white_space(white_space);
            let base = if rtl { base.right_to_left() } else { base };
            setups.push(base);
            setups.push(base.justified());
            setups.push(Setup {
                text_align: TextAlign::Justify,
                justification: TextJustification::Spaces,
                ..base
            });
        }
    }
    setups
}

#[test]
fn justified_text_that_fits_naturally_is_not_treated_as_overflow_by_the_early_stop() {
    // CSS Text 3 section 7.1 stretches a justified line to the block width, so the stretched line
    // measures the block width to within rounding. Whether a candidate overflows is decided on
    // its natural width: the early stop must not follow a stretched line that rounds above the
    // width while its natural width fits.
    const REPRODUCER: &str = "XXXXXXXXXXXXXXX X XXX  XXXXXXXXXXXXXXXXXXX  XX     XXXX";
    let spans = [span(REPRODUCER, 5.0, 1.0, 0.0)];
    let base = Setup::new(90.0)
        .justified()
        .right_to_left()
        .white_space(text::style::WhiteSpace::BreakSpaces);
    for wrap_style in [TextWrapStyle::Stable, TextWrapStyle::Auto] {
        for last in [TextAlignLast::Justify, TextAlignLast::Auto] {
            let setup = Setup {
                wrap_style,
                text_align_last: last,
                ..base
            };
            let saved = assert_same_with_full_walk::<f32>("reproducer", &spans, setup)
                + assert_same_with_full_walk::<f64>("reproducer", &spans, setup);
            assert!(saved > 0, "no line search stopped early");
        }
    }
}

#[test]
fn justified_repeated_words_lay_out_the_same_with_the_early_stop() {
    let spans = [span("XX XX XX XX XX XX XX XX", 5.0, 0.0, 0.0)];
    let setup = Setup::new(100.0)
        .justified()
        .right_to_left()
        .white_space(text::style::WhiteSpace::BreakSpaces);
    let saved = assert_same_with_full_walk::<f32>("repeated words", &spans, setup)
        + assert_same_with_full_walk::<f64>("repeated words", &spans, setup);
    assert!(saved > 0, "no line search stopped early");
}

#[test]
fn direction_white_space_and_justification_lay_out_the_same_with_the_early_stop() {
    let sources = [
        "XX XX XX XX XX XX XX XX",
        "XXXX XXXX XXXX XXXX XXXX",
        "XX  XX  XX  XX  XX  XX",
    ];
    let mut saved = 0;
    for source in sources {
        for letter_spacing in [0.0, 1.0] {
            let spans = [span(source, 5.0, letter_spacing, 0.0)];
            for width in [55.0, 90.0, 100.0] {
                for setup in grid_setups(width) {
                    saved += assert_same_with_full_walk::<f32>("grid", &spans, setup);
                }
            }
        }
    }
    assert!(saved > 0, "no line search stopped early");
}

#[test]
fn inter_character_justification_lays_out_the_same_with_the_early_stop() {
    let spans = [span("XX XX XX XX XX XX XX XX", 5.0, 0.0, 0.0)];
    let mut saved = 0;
    for rtl in [false, true] {
        for width in [55.0, 90.0, 100.0] {
            let setup = Setup {
                justification: TextJustification::InterCharacter,
                rtl,
                ..Setup::new(width).justified()
            };
            saved += assert_same_with_full_walk::<f32>("inter-character", &spans, setup);
            saved += assert_same_with_full_walk::<f64>("inter-character", &spans, setup);
        }
    }
    assert!(saved > 0, "no line search stopped early");
}

#[test]
fn hanging_punctuation_with_stretched_justification_lays_out_the_same_with_the_early_stop() {
    let text = "\u{201c}XX XX XX. XX XXX XX XXXX XX XX. XX XXX XX XX XXXX XX.\u{201d}";
    let spans = [span(text, 10.0, 0.0, 0.0)];
    let hanging = [
        (true, None, true),
        (
            true,
            Some(text::style::HangingPunctuationEnd::ForceEnd),
            false,
        ),
        (
            false,
            Some(text::style::HangingPunctuationEnd::ForceEnd),
            true,
        ),
        (
            false,
            Some(text::style::HangingPunctuationEnd::AllowEnd),
            true,
        ),
        (
            true,
            Some(text::style::HangingPunctuationEnd::AllowEnd),
            false,
        ),
    ];
    let mut saved = 0;
    for (first, end, last) in hanging {
        for justification in [
            TextJustification::InterWord,
            TextJustification::InterCharacter,
        ] {
            for width in [70.0, 130.0] {
                let setup = Setup {
                    justification,
                    hanging: text::style::HangingPunctuation::new(first, end, last),
                    ..Setup::new(width).justified()
                };
                saved += assert_same_with_full_walk::<f32>("hanging, justified", &spans, setup);
                saved += assert_same_with_full_walk::<f64>("hanging, justified", &spans, setup);
            }
        }
    }
    assert!(saved > 0, "no line search stopped early");
}

#[test]
fn negative_spacing_with_justification_lays_out_the_same_with_the_early_stop() {
    // Negative spacing vetoes the stop wherever a glyph's natural pen step goes backwards, so no
    // stop is required; the layouts must still agree, stretched or not.
    for (letter_spacing, word_spacing, size) in [
        (-2.0, 0.0, 10.0),
        (-6.0, 0.0, 5.0),
        (0.0, -5.0, 10.0),
        (0.0, -15.0, 10.0),
        (0.0, -25.0, 10.0),
    ] {
        let spans = [span(
            "XXX XX X XXXX XX X XXX XXXXX X XX XXXX XXX XX",
            size,
            letter_spacing,
            word_spacing,
        )];
        for justification in [
            TextJustification::InterWord,
            TextJustification::InterCharacter,
        ] {
            for width in [30.0, 60.0, 100.0] {
                let setup = Setup {
                    justification,
                    ..Setup::new(width).justified()
                };
                assert_same_with_full_walk::<f32>("negative, justified", &spans, setup);
                assert_same_with_full_walk::<f64>("negative, justified", &spans, setup);
            }
        }
    }
}
