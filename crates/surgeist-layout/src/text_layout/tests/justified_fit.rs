//! Line breaking fits each candidate by its natural line, before justification, through real
//! native layout.
//!
//! CSS Text 3 section 5 chooses the breaks of a paragraph by what fits the line box, and section
//! 6.4.1 distributes the remaining space of a line inside lines that line breaking has already chosen;
//! section 8.2 measures a conditionally hanging glyph for fit "prior to justification". A
//! justified paragraph therefore breaks at the same offsets as the same paragraph aligned to the
//! start. The documents use Ahem at 5px, whose glyphs and spaces advance 5px, so every width below
//! is the character count times five and every expectation is derived from those widths.

use super::*;
use surgeist_text as text;

const SOURCE: &str = "XX XX XX XX XX XX XX XX";

struct Document {
    width: f64,
    rtl: bool,
    white_space: text::WhiteSpace,
    justify: Option<TextJustification>,
}

impl Document {
    fn new(width: f64, justify: Option<TextJustification>) -> Self {
        Self {
            width,
            rtl: true,
            white_space: text::WhiteSpace::Preserve,
            justify,
        }
    }

    /// The source ranges of the placed lines.
    fn breaks<S: LayoutScalar>(&self) -> Vec<(usize, usize)> {
        let style = text::Style {
            white_space: self.white_space,
            wrap: Wrap::Word,
            ..ahem_style(5.0, 10.0)
        };
        let base = if self.rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        };
        let paragraph = PreparedParagraph::new(
            vec![ParagraphPart::text(1, Source::new(SOURCE), style.clone())],
            style,
            base,
        );
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    direction: if self.rtl {
                        Direction::Rtl
                    } else {
                        Direction::Ltr
                    },
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(self.width)),
                        PreferredSizeOf::AUTO,
                    ),
                    text_wrap_style: TextWrapStyle::Auto,
                    text_align: self
                        .justify
                        .map_or(TextAlign::Start, |_| TextAlign::Justify),
                    text_justification: self.justify.unwrap_or(TextJustification::None),
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
        let batch = compute_layout_with_flow(
            &capture,
            0,
            request(self.width, {
                let root_input = crate::LayoutTree::node_input((capture).tree(), 0);
                crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
            }),
        )
        .unwrap();
        batch.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| (line.range().start, line.range().end))
            .collect()
    }
}

fn assert_naturally_fitting_justified_line_keeps_its_break<S: LayoutScalar>() {
    // Four words and the hanging space after them are 55px of ink in a 69px line, and a fifth word
    // would reach 70px, so line breaking chooses 0..12 and 12..23. Justification only widens the
    // chosen lines.
    let document = Document::new(69.0, Some(TextJustification::InterWord));
    assert_eq!(document.breaks::<S>(), [(0, 12), (12, 23)]);
}

#[test]
fn inter_word_justified_line_that_fits_naturally_is_not_rejected_for_its_stretched_width() {
    assert_naturally_fitting_justified_line_keeps_its_break::<f32>();
    assert_naturally_fitting_justified_line_keeps_its_break::<f64>();
}

fn assert_justified_and_start_aligned_paragraphs_break_alike<S: LayoutScalar>() {
    let mut differing = Vec::new();
    for rtl in [true, false] {
        for white_space in [
            text::WhiteSpace::Preserve,
            text::WhiteSpace::Collapse,
            text::WhiteSpace::BreakSpaces,
        ] {
            for width in [69.0, 89.0, 96.0, 101.0, 104.0, 113.0] {
                let mut start = Document::new(width, None);
                start.rtl = rtl;
                start.white_space = white_space;
                let expected = start.breaks::<S>();
                for justify in [TextJustification::InterWord, TextJustification::Spaces] {
                    let mut document = Document::new(width, Some(justify));
                    document.rtl = rtl;
                    document.white_space = white_space;
                    let actual = document.breaks::<S>();
                    if actual != expected {
                        differing.push(format!(
                            "rtl {rtl} {white_space:?} width {width} {justify:?}: {actual:?} instead of {expected:?}"
                        ));
                    }
                }
            }
        }
    }
    assert!(differing.is_empty(), "{differing:#?}");
}

#[test]
fn justified_paragraph_breaks_where_the_start_aligned_paragraph_breaks() {
    assert_justified_and_start_aligned_paragraphs_break_alike::<f32>();
    assert_justified_and_start_aligned_paragraphs_break_alike::<f64>();
}

fn assert_justified_line_that_overflows_naturally_still_breaks<S: LayoutScalar>() {
    // The four-word candidate is 55px of ink and overflows a 54px line, so the first line takes
    // three words, 0..9, and the eight words fill 9..18 and 18..23.
    for justify in [None, Some(TextJustification::InterWord)] {
        let document = Document::new(54.0, justify);
        assert_eq!(
            document.breaks::<S>(),
            [(0, 9), (9, 18), (18, 23)],
            "{justify:?}"
        );
    }
}

#[test]
fn justified_line_whose_natural_width_overflows_still_breaks_before_the_overflowing_word() {
    assert_justified_line_that_overflows_naturally_still_breaks::<f32>();
    assert_justified_line_that_overflows_naturally_still_breaks::<f64>();
}
