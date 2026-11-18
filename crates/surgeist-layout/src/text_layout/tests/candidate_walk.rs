//! One line search evaluates the candidates that decide that line, through real native layout.
//!
//! CSS Text 3 section 5 breaks a line at the last soft wrap opportunity whose content still
//! fits the line box, and CSS 2.1 section 9.5 shortens or moves a line that meets a float. The
//! documents use Ahem at 10px, whose glyphs and spaces advance 10px, so every width below is
//! the glyph count times ten and every expectation is derived from those widths.

use super::*;
use crate::accounting::{LayoutWork, TestHost};
use crate::test_support::readback::{self, Phase};

const WORD: &str = "XXX ";

/// One block container of prepared Ahem text with an optional left float.
struct Document {
    source: String,
    width: f64,
    float: Option<(f64, f64)>,
    wrap_style: TextWrapStyle,
    letter_spacing: f64,
    word_spacing: f64,
    size: f32,
    /// Justify every line, including the last, with this method.
    justification: Option<TextJustification>,
}

impl Document {
    fn new(source: impl Into<String>, width: f64) -> Self {
        Self {
            source: source.into(),
            width,
            float: None,
            wrap_style: TextWrapStyle::Auto,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            size: 10.0,
            justification: None,
        }
    }

    fn words(count: usize, width: f64) -> Self {
        Self::new(WORD.repeat(count), width)
    }

    /// The placed lines as trimmed source text and physical bounds.
    fn lay_out<S: LayoutScalar>(&self) -> Vec<(String, surgeist_text::geometry::Rect)> {
        self.lay_out_reporting::<S>(None)
    }

    /// The same layout, reporting the call's work to `host`.
    fn lay_out_reporting<S: LayoutScalar>(
        &self,
        host: Option<&TestHost>,
    ) -> Vec<(String, surgeist_text::geometry::Rect)> {
        let mut style = ahem_style(self.size, 10.0);
        style.letter_spacing = self.letter_spacing;
        style.word_spacing = self.word_spacing;
        let paragraph = PreparedParagraph::plain(1, Source::new(self.source.as_str()), style);
        let children = if self.float.is_some() {
            vec![2, 1]
        } else {
            vec![1]
        };
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, children)
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(self.width)),
                        PreferredSizeOf::AUTO,
                    ),
                    text_wrap_style: self.wrap_style,
                    text_align: self
                        .justification
                        .map_or(TextAlign::Start, |_| TextAlign::Justify),
                    text_align_last: self
                        .justification
                        .map_or(TextAlignLast::Auto, |_| TextAlignLast::Justify),
                    text_justification: self.justification.unwrap_or(TextJustification::None),
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1));
        if let Some((width, height)) = self.float {
            tree = tree.children(2, []).style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(width)),
                        PreferredSizeOf::px(S::from_f64(height)),
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
            request(self.width, {
                let root_input = crate::LayoutTree::node_input((capture).tree(), 0);
                crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
            }),
            &[],
            host.map(TestHost::scope).as_ref(),
        )
        .unwrap();
        if self.float.is_some() {
            let float = readback::single_box_output(&batch, 2, Phase::Final);
            assert_eq!(float.location, Point::ZERO);
        }
        batch.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| {
                let range = line.range();
                (
                    self.source[range.start..range.end].trim_end().to_owned(),
                    line.bounds(),
                )
            })
            .collect()
    }
}

fn texts(lines: &[(String, surgeist_text::geometry::Rect)]) -> Vec<&str> {
    lines.iter().map(|(text, _)| text.as_str()).collect()
}

fn words_on_widest_line(lines: &[(String, surgeist_text::geometry::Rect)]) -> usize {
    lines
        .iter()
        .map(|(text, _)| text.split(' ').count())
        .max()
        .unwrap()
}

/// The line searches and candidates one layout reports to its host scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CandidateUsage {
    line_searches: u64,
    candidates: u64,
}

impl CandidateUsage {
    /// The most candidates `per_line` words per line allow over this many line searches.
    fn per_search_bound(&self, per_line: usize) -> u64 {
        (u64::try_from(per_line).unwrap() + 2) * self.line_searches
    }

    fn of(host: &TestHost) -> Self {
        Self {
            line_searches: host.work(LayoutWork::LineSearch),
            candidates: host.work(LayoutWork::Candidate),
        }
    }
}

/// Laying out one document and counting the line searches and candidates it evaluates.
fn candidate_usage(document: &Document) -> (CandidateUsage, usize, usize) {
    let host = TestHost::new();
    let lines = document.lay_out_reporting::<f64>(Some(&host));
    (
        CandidateUsage::of(&host),
        lines.len(),
        words_on_widest_line(&lines),
    )
}

/// The work of one document with every line search forced to visit its whole opportunity window.
fn full_walk_usage(document: &Document) -> CandidateUsage {
    let host = TestHost::new();
    fit::with_full_walk(|| document.lay_out_reporting::<f64>(Some(&host)));
    CandidateUsage::of(&host)
}

#[test]
fn auto_line_search_evaluates_only_the_words_that_fit_plus_the_first_that_does_not() {
    // A 100px line holds "XXX XXX" (70px) and not a third word (110px), so every line has two
    // words. The words that fit plus the first word that does not are 3 candidates per line
    // search; the paragraph end adds one more. Each line search may therefore evaluate at most
    // words-per-line + 2 candidates, however long the paragraph is.
    let mut per_search_bound = Vec::new();
    let mut candidates = Vec::new();
    for count in [50, 100] {
        let document = Document::words(count, 100.0);
        let (work, lines, per_line) = candidate_usage(&document);
        assert_eq!(lines, count / 2, "{count} words: two words per 100px line");
        assert_eq!(per_line, 2);
        assert!(
            usize::try_from(work.line_searches).unwrap() >= lines,
            "{work:?}"
        );
        per_search_bound.push((work, work.per_search_bound(per_line)));
        candidates.push(work.candidates);
    }
    for (work, bound) in &per_search_bound {
        assert!(
            work.candidates <= *bound,
            "{} candidates over {} line searches exceeds {bound}",
            work.candidates,
            work.line_searches
        );
    }
    // Doubling the paragraph at most doubles its candidates once per-line work is bounded.
    assert!(candidates[1] <= candidates[0] * 5 / 2, "{candidates:?}");
}

/// Negative spacing anywhere in the group keeps every candidate of a line search: the spacing
/// after the last glyph of a candidate is trimmed at its edge, so a longer candidate can be
/// narrower than a shorter one.
fn assert_negative_spacing_keeps_the_full_walk(document: &mut Document, words_per_line: usize) {
    let count = 50;
    document.source = WORD.repeat(count);
    let (work, lines, per_line) = candidate_usage(document);
    assert_eq!(per_line, words_per_line);
    assert_eq!(lines, count.div_ceil(words_per_line));
    assert!(
        work.candidates > work.per_search_bound(per_line),
        "{work:?} must keep the full walk"
    );
}

#[test]
fn negative_letter_spacing_keeps_the_full_candidate_walk() {
    // CSS Text 3 section 8.1: -2px leaves each 10px glyph 8px, so "XXX XXX XXX" is 88px and a
    // fourth word reaches 120px: three words per 100px line.
    let mut document = Document::new("", 100.0);
    document.letter_spacing = -2.0;
    assert_negative_spacing_keeps_the_full_walk(&mut document, 3);
}

#[test]
fn negative_word_spacing_keeps_the_full_candidate_walk() {
    // CSS Text 3 section 8.2: -15px makes each 10px space advance -5px, so three "XXX" words are
    // 90 - 10 = 80px and a fourth reaches 105px: three words per 100px line.
    let mut document = Document::new("", 100.0);
    document.word_spacing = -15.0;
    assert_negative_spacing_keeps_the_full_walk(&mut document, 3);
}

#[test]
fn word_spacing_that_moves_the_pen_backwards_keeps_the_full_candidate_walk() {
    // CSS Text 3 section 8.2: -25px makes each 10px space advance -15px and splits that around the
    // space, so the pen steps back 2.5px at each space. A later word can then be narrower than the
    // content before it, so no candidate may be skipped after one overflows.
    let mut document = Document::new("XXXX ".repeat(50), 100.0);
    document.word_spacing = -25.0;
    let (work, lines, per_line) = candidate_usage(&document);
    assert!(lines > 1);
    assert!(
        work.candidates > work.per_search_bound(per_line),
        "{work:?} must keep the full walk"
    );
}

/// The candidates the early stop saves on one document: the forced full walk evaluates every
/// candidate of the same line searches, so the early stop evaluates the difference less.
fn candidates_saved_by_the_early_stop(document: &Document) -> u64 {
    let (early, ..) = candidate_usage(document);
    let full = full_walk_usage(document);
    assert_eq!(
        early.line_searches, full.line_searches,
        "the early stop changes no line search"
    );
    full.candidates
        .checked_sub(early.candidates)
        .expect("the early stop never evaluates more candidates than the full walk")
}

#[test]
fn justified_text_without_negative_spacing_still_stops_the_line_search_early() {
    for justification in [
        TextJustification::InterWord,
        TextJustification::InterCharacter,
    ] {
        let mut document = Document::words(50, 100.0);
        document.justification = Some(justification);
        assert!(
            candidates_saved_by_the_early_stop(&document) > 0,
            "{justification:?}"
        );
    }
}

#[test]
fn justified_inter_character_text_with_negative_letter_spacing_keeps_the_full_candidate_walk() {
    // CSS Text 3 section 8.1: letter-spacing -12px on a 5px glyph leaves -7px per glyph. Under
    // text-align Justify and text-align-last Justify each candidate's negative flow is stretched
    // to the 100px block, and a stretched line must never lead to a stop: it can lift the steps
    // that would show the negative spacing.
    let mut document = Document::new("XX ".repeat(6), 100.0);
    document.size = 5.0;
    document.letter_spacing = -12.0;
    document.justification = Some(TextJustification::InterCharacter);
    assert_eq!(candidates_saved_by_the_early_stop(&document), 0);
}

#[test]
fn justified_inter_word_text_with_negative_letter_spacing_keeps_the_full_candidate_walk() {
    // The same text under inter-word justification, which stretches only the spaces, so the
    // steps inside a word stay at -7px.
    let mut document = Document::new("XX ".repeat(6), 100.0);
    document.size = 5.0;
    document.letter_spacing = -12.0;
    document.justification = Some(TextJustification::InterWord);
    assert_eq!(candidates_saved_by_the_early_stop(&document), 0);
}

fn assert_trailing_space_hangs<S: LayoutScalar>() {
    // The ten-glyph word is exactly 100px. CSS Text 3 section 4.1.3 removes the collapsible space
    // at the end of a line, so the hanging space does not make the first line overflow.
    let lines = Document::new("XXXXXXXXXX XX ", 100.0).lay_out::<S>();
    assert_eq!(texts(&lines), ["XXXXXXXXXX", "XX"]);
    assert_eq!(
        lines[0].1,
        surgeist_text::geometry::Rect::new(0.0, 0.0, 100.0, 10.0)
    );
}

#[test]
fn collapsible_trailing_space_does_not_overflow_an_exactly_fitting_line() {
    assert_trailing_space_hangs::<f32>();
    assert_trailing_space_hangs::<f64>();
}

fn assert_negative_letter_spacing_tightens<S: LayoutScalar>() {
    // Without tracking "XXXXXX XXXX" is 110px and wraps in a 100px line. CSS Text 3 section 8.1
    // adds letter-spacing to every character; -2px leaves each of the 11 glyphs 8px, 88px in all.
    let wide = Document::new("XXXXXX XXXX", 100.0).lay_out::<S>();
    assert_eq!(texts(&wide), ["XXXXXX", "XXXX"]);
    let mut tight = Document::new("XXXXXX XXXX", 100.0);
    tight.letter_spacing = -2.0;
    assert_eq!(texts(&tight.lay_out::<S>()), ["XXXXXX XXXX"]);
}

#[test]
fn negative_letter_spacing_lets_more_words_fit_one_line() {
    assert_negative_letter_spacing_tightens::<f32>();
    assert_negative_letter_spacing_tightens::<f64>();
}

fn assert_lines_move_below_float_where_the_band_widens<S: LayoutScalar>() {
    // An 80x20 left float leaves a 20px band at y 0 and y 10. "XX" fits it; the indivisible 40px
    // and 80px words do not, so CSS 2.1 section 9.5 moves them below the float, where the band is
    // the full 100px. "XXXX XXXXXXXX" would be 130px there, so each word keeps its own line.
    let mut document = Document::new("XX XXXX XXXXXXXX ", 100.0);
    document.float = Some((80.0, 20.0));
    let lines = document.lay_out::<S>();
    assert_eq!(texts(&lines), ["XX", "XXXX", "XXXXXXXX"]);
    assert_eq!(
        lines[0].1,
        surgeist_text::geometry::Rect::new(80.0, 0.0, 20.0, 10.0)
    );
    assert_eq!(
        lines[1].1,
        surgeist_text::geometry::Rect::new(0.0, 20.0, 100.0, 10.0)
    );
    assert_eq!(
        lines[2].1,
        surgeist_text::geometry::Rect::new(0.0, 30.0, 100.0, 10.0)
    );
}

#[test]
fn words_too_wide_for_the_band_beside_a_float_fit_in_the_wider_band_below() {
    assert_lines_move_below_float_where_the_band_widens::<f32>();
    assert_lines_move_below_float_where_the_band_widens::<f64>();
}

fn assert_text_wrap_styles_keep_their_line_choices<S: LayoutScalar>() {
    // Five 40px words in a 190px line. CSS Text 4 text-wrap-style: auto fills each line, giving
    // four words (190px) and one; balance equalises the lines, three words (140px) and two (90px).
    let source = "XXXX XXXX XXXX XXXX XXXX ";
    let lines = Document::new(source, 190.0).lay_out::<S>();
    assert_eq!(texts(&lines), ["XXXX XXXX XXXX XXXX", "XXXX"]);
    let mut balanced = Document::new(source, 190.0);
    balanced.wrap_style = TextWrapStyle::Balance;
    assert_eq!(
        texts(&balanced.lay_out::<S>()),
        ["XXXX XXXX XXXX", "XXXX XXXX"]
    );
    // pretty and avoid-short-last-line choose among the same candidates: every line still fits
    // the 190px line box and the lines still cover the whole source in order.
    for style in [TextWrapStyle::Pretty, TextWrapStyle::AvoidShortLastLine] {
        let mut document = Document::new(source, 190.0);
        document.wrap_style = style;
        let lines = document.lay_out::<S>();
        assert_eq!(lines.len(), 2, "{style:?}");
        assert_eq!(
            lines
                .iter()
                .map(|(text, _)| text.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            source.trim_end(),
            "{style:?}"
        );
        for (text, bounds) in &lines {
            assert!(text.len() as f64 * 10.0 <= 190.0, "{style:?}: {text:?}");
            assert_eq!(bounds.size.width, 190.0);
        }
    }
}

#[test]
fn balance_pretty_and_avoid_short_last_line_choose_among_all_fitting_candidates() {
    assert_text_wrap_styles_keep_their_line_choices::<f32>();
    assert_text_wrap_styles_keep_their_line_choices::<f64>();
}
