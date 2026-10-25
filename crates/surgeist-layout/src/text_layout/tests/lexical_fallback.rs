//! Missing Thai/Lao lexical capability supplies ordinary TLU-pair fallback.
//! The adopted determination preserves this fallback under Normal and KeepAll.
//!
//! Ahem has no Thai/Lao cmap coverage. These tests deliberately qualify actual
//! .notdef glyph 0 geometry, not script glyph coverage: its hmtx advance is
//! 1000 design units at 1000 UPEM, hence 10px in the actual 10px instance.
//! Unicode source/TLU semantics remain independent of the missing glyph display.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const SCRIPTS: [(&str, &str); 2] = [("กก", "th"), ("ກກ", "lo")];

#[derive(Clone, Debug, PartialEq)]
enum Termination {
    Ordinary(text::PreparedBreakKind),
    Conditional,
}

type LineFact = (text::Range, f64, f64, text::LineEnd, Termination);

fn line(start: usize, end: usize, advance: f64, kind: text::PreparedBreakKind) -> LineFact {
    (
        text::Range::new(start, end),
        advance,
        advance,
        match kind {
            text::PreparedBreakKind::Allowed => text::LineEnd::Soft,
            text::PreparedBreakKind::Forced => text::LineEnd::Forced,
            text::PreparedBreakKind::ParagraphEnd => text::LineEnd::Paragraph,
            text::PreparedBreakKind::Discretionary => unreachable!(),
        },
        Termination::Ordinary(kind),
    )
}

fn paragraph(
    source: &str,
    locale: &str,
    word_break: text::WordBreak,
    ancestor_wrap: Wrap,
    rtl: bool,
) -> PreparedParagraph {
    let child = text::Style {
        locale: Some(locale.to_owned()),
        word_break,
        line_break: text::LineBreak::Auto,
        overflow_wrap: text::OverflowWrap::Normal,
        white_space: text::WhiteSpace::Preserve,
        hyphens: text::Hyphens::None,
        wrap: Wrap::Word,
        ..ahem_style(10.0, 10.0)
    };
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::text(
            1,
            Source::identified(source, text::Id::from_u64(65402), 19),
            child.clone(),
        )],
        text::Style {
            wrap: ancestor_wrap,
            ..child
        },
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    );
    let analyzed = paragraph.prepared.analyzed_source();
    assert_eq!(analyzed.style().locale.as_deref(), Some(locale));
    assert_eq!(analyzed.style().word_break, word_break);
    assert!(
        analyzed
            .wrapping_relations()
            .all(|relation| { relation.ancestor_wrap() == Some(ancestor_wrap) })
    );
    paragraph
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: AvailableOf<S>,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    compute_layout_with_text(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn facts<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    source: &str,
) -> Vec<LineFact> {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    let units = source.encode_utf16().collect::<Vec<_>>();
    assert_eq!(placed.source().text().code_units(), units);
    let origin = paragraph.origin(1);
    assert_eq!(origin.view().source().id(), Some(text::Id::from_u64(65402)));
    assert_eq!(origin.view().source().revision(), 19);
    assert_eq!(origin.view().source().text().code_units(), units);
    let endpoint = |offset, affinity| {
        placed
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
            .unwrap()
    };
    let copied = placed
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(units.len(), text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(copied.source_text(), &text::Text::from(source));
    assert_eq!(copied.plaintext(), &text::Text::from(source));
    let mut finalized = vec![None; placed.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        finalized[fragment.line_index()] = Some(line);
    }
    let mut glyph_count = 0;
    let result = placed
        .lines()
        .iter()
        .zip(finalized)
        .enumerate()
        .map(|(index, (placed_line, line))| {
            let line = line.expect("nonempty source line retains its real origin fragment");
            let selected = line.selected_line();
            let request = selected.request();
            assert!(request.alternative.is_none());
            assert!(request.incoming.is_none());
            assert!(line.hyphenation_continuation().is_none());
            let mut raw = 0.0;
            for run in line.run_views() {
                let face = run.face();
                assert_eq!(face.bytes(), AHEM);
                assert_eq!(face.index(), 0);
                let instance = run.effective_instance().unwrap();
                assert_eq!(instance.font_size(), 10.0);
                assert_eq!(instance.axis_count(), 0);
                let metrics = instance.metrics(face.catalogue_identity()).unwrap();
                assert_eq!(metrics.units_per_em(), 1000);
                assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
                raw += run.advance();
                for glyph in run.glyphs() {
                    assert_eq!(glyph.id(), 0, "this fixture has missing script glyphs");
                    assert_eq!(glyph.advance(), 10.0);
                    glyph_count += 1;
                }
            }
            assert_eq!(line.feedback().advance, raw);
            let placed_raw: f64 = placed
                .inline_metric_facts()
                .runs()
                .iter()
                .filter(|run| run.line() == index)
                .map(|run| run.advance())
                .sum();
            assert_eq!(placed_raw, raw);
            (
                placed_line.range(),
                raw,
                line.measurements().fit().advance(),
                request.context.end,
                match &request.termination {
                    text::PreparedTermination::Ordinary(offer) => {
                        Termination::Ordinary(offer.view().kind())
                    }
                    text::PreparedTermination::Conditional(_) => Termination::Conditional,
                    text::PreparedTermination::Measurement(_)
                    | text::PreparedTermination::PhraseMeasurement(_) => {
                        panic!("a measurement-only trial cannot be placed")
                    }
                },
            )
        })
        .collect();
    assert_eq!(glyph_count, source.chars().filter(|ch| *ch != '\n').count());
    result
}

fn allowed(paragraph: &PreparedParagraph) -> Vec<usize> {
    paragraph
        .prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
        .map(|offer| {
            assert_eq!(offer.alternatives().count(), 0);
            let boundary = offer.boundary();
            assert_eq!(
                boundary.view().source_before(),
                boundary.view().source_after()
            );
            boundary.view().source_after().get()
        })
        .collect()
}

#[test]
fn missing_lexical_capability_uses_ordinary_letter_pair_cuts_in_native_layout() {
    fn lane<S: LayoutScalar>() -> Vec<(Vec<usize>, Vec<LineFact>)> {
        let mut observed = Vec::new();
        for (source, locale) in SCRIPTS {
            for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
                for rtl in [false, true] {
                    let p = paragraph(source, locale, word, Wrap::Word, rtl);
                    let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(10.0)), rtl);
                    observed.push((allowed(&p), facts(&actual, &p, source)));
                }
            }
        }
        observed
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let expected = vec![
        (
            vec![1],
            vec![
                line(0, 1, 10.0, text::PreparedBreakKind::Allowed),
                line(1, 2, 10.0, text::PreparedBreakKind::ParagraphEnd),
            ],
        );
        8
    ];
    assert_eq!(actual, (expected.clone(), expected));
}

#[test]
fn missing_lexical_fallback_reduces_min_content_without_changing_max_content() {
    fn lane<S: LayoutScalar>() -> Vec<(f64, Vec<LineFact>)> {
        let mut observed = Vec::new();
        for (source, locale) in SCRIPTS {
            for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
                for rtl in [false, true] {
                    let p = paragraph(source, locale, word, Wrap::Word, rtl);
                    for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
                        let actual = compute::<S>(&p, available, rtl);
                        let width = actual
                            .unrounded_entries()
                            .iter()
                            .find(|entry| entry.node() == 0)
                            .unwrap()
                            .output()
                            .size
                            .width
                            .to_f64();
                        observed.push((width, facts(&actual, &p, source)));
                    }
                }
            }
        }
        observed
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let pair = [
        (
            10.0,
            vec![
                line(0, 1, 10.0, text::PreparedBreakKind::Allowed),
                line(1, 2, 10.0, text::PreparedBreakKind::ParagraphEnd),
            ],
        ),
        (
            20.0,
            vec![line(0, 2, 20.0, text::PreparedBreakKind::ParagraphEnd)],
        ),
    ];
    let expected = (0..8).flat_map(|_| pair.clone()).collect::<Vec<_>>();
    assert_eq!(actual, (expected.clone(), expected));
}

#[test]
fn real_notdef_metrics_remain_usable_with_ancestor_nowrap_and_independent_forced_breaks() {
    fn lane<S: LayoutScalar>() {
        for (source, locale) in SCRIPTS {
            for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
                for rtl in [false, true] {
                    let p = paragraph(source, locale, word, Wrap::None, rtl);
                    let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(10.0)), rtl);
                    assert!(allowed(&p).is_empty());
                    assert_eq!(
                        facts(&actual, &p, source),
                        [line(0, 2, 20.0, text::PreparedBreakKind::ParagraphEnd)]
                    );
                    let first = source.chars().next().unwrap();
                    let source = format!("{first}\n{first}");
                    let p = paragraph(&source, locale, word, Wrap::None, rtl);
                    let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(10.0)), rtl);
                    assert!(allowed(&p).is_empty());
                    assert_eq!(
                        facts(&actual, &p, &source),
                        [
                            line(0, 2, 10.0, text::PreparedBreakKind::Forced),
                            line(2, 3, 10.0, text::PreparedBreakKind::ParagraphEnd),
                        ]
                    );
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

const WJ_RUN: &str = "กก\u{2060}AA";

/// Mixed missing Thai/real Latin glyphs. Any formatting controls have no
/// positive advance, but their glyph count and glyph ID are unspecified.
fn mixed_sa_latin_facts<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    source: &str,
    latin_start: usize,
) -> Vec<LineFact> {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    let source = text::Text::from(source);
    assert_eq!(placed.source().text(), &source);
    let origin = paragraph.origin(1);
    assert_eq!(origin.view().source().text(), &source);
    assert_eq!(origin.view().source().id(), Some(text::Id::from_u64(65402)));
    assert_eq!(origin.view().source().revision(), 19);
    let endpoint = |offset, affinity| {
        placed
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
            .unwrap()
    };
    let copied = placed
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(source.len(), text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(copied.source_text(), &source);
    assert_eq!(copied.plaintext(), &source);
    let mut finalized = vec![None; placed.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        finalized[fragment.line_index()] = Some(line);
    }
    let mut positive = Vec::new();
    let result = placed
        .lines()
        .iter()
        .zip(finalized)
        .enumerate()
        .map(|(index, (placed_line, finalized))| {
            let finalized = finalized.expect("mixed source line keeps its authentic origin");
            let mut raw = 0.0;
            for run in finalized.run_views() {
                let face = run.face();
                assert_eq!(face.bytes(), AHEM);
                assert_eq!(face.index(), 0);
                let instance = run.effective_instance().unwrap();
                assert_eq!(instance.font_size(), 10.0);
                assert_eq!(instance.axis_count(), 0);
                let metrics = instance.metrics(face.catalogue_identity()).unwrap();
                assert_eq!(metrics.units_per_em(), 1000);
                assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
                raw += run.advance();
                for glyph in run.glyphs() {
                    if glyph.advance() > 0.0 {
                        positive.push((glyph.range(), glyph.id(), glyph.advance()));
                    } else {
                        assert_eq!(glyph.advance(), 0.0);
                    }
                }
            }
            assert_eq!(finalized.feedback().advance, raw);
            let placed_raw: f64 = placed
                .inline_metric_facts()
                .runs()
                .iter()
                .filter(|run| run.line() == index)
                .map(|run| run.advance())
                .sum();
            assert_eq!(placed_raw, raw);
            let selected = finalized.selected_line();
            let request = selected.request();
            assert!(request.alternative.is_none());
            assert!(request.incoming.is_none());
            assert!(finalized.hyphenation_continuation().is_none());
            let text::PreparedTermination::Ordinary(offer) = &request.termination else {
                panic!("OverflowNormal cannot grant a conditional seam")
            };
            (
                placed_line.range(),
                raw,
                finalized.measurements().fit().advance(),
                request.context.end,
                Termination::Ordinary(offer.view().kind()),
            )
        })
        .collect();
    positive.sort_by_key(|glyph| glyph.0.start);
    // Actual Ahem metadata: Thai is missing glyph0; A maps to glyph35. Both
    // advances are1000 design units at1000 UPEM, hence10px in this instance.
    assert_eq!(
        positive,
        [
            (text::Range::new(0, 1), 0, 10.0),
            (text::Range::new(1, 2), 0, 10.0),
            (text::Range::new(latin_start, latin_start + 1), 35, 10.0),
            (text::Range::new(latin_start + 1, latin_start + 2), 35, 10.0),
        ]
    );
    result
}

#[test]
fn sa_provider_run_end_before_word_joiner_cannot_split_the_native_fitting_line() {
    // UAX14 revision55 LB11 prohibits breaks before/after WJ. No earlier
    // ZW SP* permission exists here. Genuine lexical fallback1 is unaffected.
    fn lane<S: LayoutScalar>() -> Vec<(Vec<usize>, Vec<LineFact>)> {
        let mut observed = Vec::new();
        for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
            for rtl in [false, true] {
                let p = paragraph(WJ_RUN, "th", word, Wrap::Word, rtl);
                let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(20.0)), rtl);
                observed.push((allowed(&p), mixed_sa_latin_facts(&actual, &p, WJ_RUN, 3)));
            }
        }
        observed
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let expected = vec![
        (
            vec![1],
            vec![
                line(0, 1, 10.0, text::PreparedBreakKind::Allowed),
                line(1, 5, 30.0, text::PreparedBreakKind::ParagraphEnd),
            ],
        );
        4
    ];
    assert_eq!(actual, (expected.clone(), expected));
}

#[test]
fn word_joiner_at_sa_provider_run_end_keeps_the_thirty_pixel_min_content_piece() {
    fn lane<S: LayoutScalar>() -> Vec<(f64, Vec<LineFact>)> {
        let mut observed = Vec::new();
        for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
            for rtl in [false, true] {
                let p = paragraph(WJ_RUN, "th", word, Wrap::Word, rtl);
                for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
                    let actual = compute::<S>(&p, available, rtl);
                    let width = actual
                        .unrounded_entries()
                        .iter()
                        .find(|entry| entry.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64();
                    observed.push((width, mixed_sa_latin_facts(&actual, &p, WJ_RUN, 3)));
                }
            }
        }
        observed
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let pair = [
        (
            30.0,
            vec![
                line(0, 1, 10.0, text::PreparedBreakKind::Allowed),
                line(1, 5, 30.0, text::PreparedBreakKind::ParagraphEnd),
            ],
        ),
        (
            40.0,
            vec![line(0, 5, 40.0, text::PreparedBreakKind::ParagraphEnd)],
        ),
    ];
    let expected = (0..4).flat_map(|_| pair.clone()).collect::<Vec<_>>();
    assert_eq!(actual, (expected.clone(), expected));
}

const SA_LATIN_RUN: &str = "กกAA";

#[test]
fn ordinary_sa_context_does_not_accept_a_missing_model_run_end_before_latin() {
    // Text4 6.1.1.1 resolves ordinary SA context as AL plus lexical causes.
    // Missing models authenticate no lexical endpoint at2. Fallback1 remains;
    // resolved AL x AL at2 cannot separate the remaining Thai/Latin sequence.
    fn lane<S: LayoutScalar>() -> Vec<(Vec<usize>, Vec<LineFact>)> {
        let mut observed = Vec::new();
        for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
            for rtl in [false, true] {
                let p = paragraph(SA_LATIN_RUN, "th", word, Wrap::Word, rtl);
                let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(20.0)), rtl);
                observed.push((
                    allowed(&p),
                    mixed_sa_latin_facts(&actual, &p, SA_LATIN_RUN, 2),
                ));
            }
        }
        observed
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let expected = vec![
        (
            vec![1],
            vec![
                line(0, 1, 10.0, text::PreparedBreakKind::Allowed),
                line(1, 4, 30.0, text::PreparedBreakKind::ParagraphEnd),
            ],
        );
        4
    ];
    assert_eq!(actual, (expected.clone(), expected));
}

#[test]
fn ordinary_sa_context_keeps_the_thirty_pixel_min_content_piece() {
    fn lane<S: LayoutScalar>() -> Vec<(f64, Vec<LineFact>)> {
        let mut observed = Vec::new();
        for word in [text::WordBreak::Normal, text::WordBreak::KeepAll] {
            for rtl in [false, true] {
                let p = paragraph(SA_LATIN_RUN, "th", word, Wrap::Word, rtl);
                for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
                    let actual = compute::<S>(&p, available, rtl);
                    let width = actual
                        .unrounded_entries()
                        .iter()
                        .find(|entry| entry.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64();
                    observed.push((width, mixed_sa_latin_facts(&actual, &p, SA_LATIN_RUN, 2)));
                }
            }
        }
        observed
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let pair = [
        (
            30.0,
            vec![
                line(0, 1, 10.0, text::PreparedBreakKind::Allowed),
                line(1, 4, 30.0, text::PreparedBreakKind::ParagraphEnd),
            ],
        ),
        (
            40.0,
            vec![line(0, 4, 40.0, text::PreparedBreakKind::ParagraphEnd)],
        ),
    ];
    let expected = (0..4).flat_map(|_| pair.clone()).collect::<Vec<_>>();
    assert_eq!(actual, (expected.clone(), expected));
}
