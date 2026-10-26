//! Source-qualified phrase preference through the real staged Layout fitter.
use super::*;
use surgeist_text as text;

const PHRASE_FONT: &[u8] =
    include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ending {
    Ordinary,
    Restored(text::ConditionalStage),
    Paragraph,
}
type Fact = (text::Range, f64, f64, Ending);

fn japanese(overflow_wrap: text::OverflowWrap) -> text::Style {
    text::Style {
        font: text::Font::new().family("Surgeist Phrase Ahem"),
        locale: Some("ja".into()),
        word_break: text::WordBreak::AutoPhrase,
        overflow_wrap,
        hyphens: text::Hyphens::None,
        ..ahem_style(10.0, 10.0)
    }
}

fn paragraph(source: &str, style: text::Style, rtl: bool, font: &[u8]) -> PreparedParagraph {
    PreparedParagraph::with_registered_font(
        vec![ParagraphPart::text(
            1,
            Source::identified(source, text::Id::from_u64(65067), 11),
            style.clone(),
        )],
        style,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
        font,
    )
}

fn compute<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: AvailableOf<S>,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let binding = p.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                ..NodeInputOf::default()
            },
        );
    for &node in &binding.nodes {
        tree = tree.children(node, []).input(node, p.text_input(node));
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn finalized<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
) -> Vec<text::FinalizedLine> {
    let mut result = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), p.prepared);
        result[fragment.line_index()] = Some(line);
    }
    result
        .into_iter()
        .map(|line| line.expect("each line retains its authentic source fragment"))
        .collect()
}

fn facts<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
    source: &str,
    font: &[u8],
) -> (f64, Vec<Fact>) {
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), &text::Text::from(source));
    let endpoint = |offset, affinity| {
        placed
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
            .unwrap()
    };
    let copied = placed
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(source.encode_utf16().count(), text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(copied.source_text(), &text::Text::from(source));
    assert_eq!(copied.plaintext(), &text::Text::from(source));
    let mut original = Vec::new();
    let lines = placed
        .lines()
        .iter()
        .zip(finalized(batch, p))
        .enumerate()
        .map(|(index, (placed_line, line))| {
            let mut raw = 0.0;
            for run in line.run_views() {
                let face = run.face();
                assert_eq!(face.bytes(), font);
                assert_eq!(face.index(), 0);
                let instance = run.effective_instance().unwrap();
                assert_eq!(instance.font_size(), 10.0);
                assert_eq!(
                    instance
                        .metrics(face.catalogue_identity())
                        .unwrap()
                        .units_per_em(),
                    1000
                );
                raw += run.advance();
                for glyph in run.glyphs() {
                    if glyph.advance() != 0.0 {
                        assert_eq!(glyph.advance(), 10.0);
                        if !glyph.range().is_empty() {
                            original.push(glyph.range());
                        }
                    }
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
            let selected = line.selected_line();
            let request = selected.request();
            let ending = match &request.termination {
                text::PreparedTermination::Ordinary(offer) => match offer.view().kind() {
                    text::PreparedBreakKind::Allowed => {
                        assert_eq!(request.context.end, text::LineEnd::Soft);
                        Ending::Ordinary
                    }
                    text::PreparedBreakKind::ParagraphEnd => {
                        assert_eq!(request.context.end, text::LineEnd::Paragraph);
                        Ending::Paragraph
                    }
                    _ => panic!("no ordinary hyphen or forced owner in this fixture"),
                },
                text::PreparedTermination::Conditional(offer) => {
                    assert_eq!(request.context.end, text::LineEnd::Soft);
                    assert_eq!(offer.alternative(), request.alternative);
                    if offer.stage() == text::ConditionalStage::Hyphens {
                        assert!(request.alternative.is_some());
                    } else {
                        assert!(request.alternative.is_none());
                    }
                    Ending::Restored(offer.stage())
                }
                _ => panic!("measurement-only selections cannot be published"),
            };
            (
                placed_line.range(),
                raw,
                line.measurements().fit().advance(),
                ending,
            )
        })
        .collect();
    original.sort_by_key(|range| range.start);
    assert_eq!(
        original,
        source
            .encode_utf16()
            .enumerate()
            .filter(|(_, cp)| *cp != 0xad)
            .map(|(index, _)| text::Range::new(index, index + 1))
            .collect::<Vec<_>>()
    );
    let width = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output()
        .size
        .width
        .to_f64();
    (width, lines)
}

fn line(start: usize, end: usize, raw: f64, ending: Ending) -> Fact {
    (text::Range::new(start, end), raw, raw, ending)
}

fn intrinsic<S: LayoutScalar>(
    p: &PreparedParagraph,
    available: AvailableOf<S>,
    rtl: bool,
) -> super::super::run::Output<u32, S> {
    let size = Size::new(S::from_f64(100.0), S::from_f64(10.0));
    super::super::run::layout(
        &p.binding(0),
        &Default::default(),
        &[],
        &[],
        super::super::run::Context {
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                if rtl { Direction::Rtl } else { Direction::Ltr },
            ),
            frame: super::super::run::Frame::Measurement,
            containing_size: size,
            inset: Point::ZERO,
            right_inset: S::ZERO,
            cursor_block: S::ZERO,
            axes: FlowAxes::new(
                WritingMode::HorizontalTb,
                if rtl { Direction::Rtl } else { Direction::Ltr },
            ),
            available,
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: Default::default(),
            hyphenation_state: Default::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: super::super::fit::TailContext::Unrestricted,
        },
        &[],
        super::super::run::LineEnvironment {
            checkpoint_group: || {},
            reset: |_| {},
            band: |_, _| {
                Ok(super::super::fit::Band {
                    start: 0.0,
                    end: 100.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &text::LineSelectionRequest| Ok(None),
            replay: |_: &text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
    .unwrap_or_else(|_| panic!("real intrinsic Text consumer must complete"))
}

#[test]
fn real_whole_phrase_overflow_restores_normal_before_arbitrary_and_requeries_remaining_pieces() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let p = paragraph(
                "日本へ",
                japanese(text::OverflowWrap::Anywhere),
                rtl,
                PHRASE_FONT,
            );
            assert_eq!(p.prepared.phrases().count(), 1);
            assert!(
                p.prepared
                    .opportunity_views()
                    .all(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
            );
            for (width, expected) in [
                (40.0, vec![line(0, 3, 30.0, Ending::Paragraph)]),
                (
                    20.0,
                    vec![
                        line(0, 2, 20.0, Ending::Restored(text::ConditionalStage::Normal)),
                        line(2, 3, 10.0, Ending::Paragraph),
                    ],
                ),
                (
                    10.0,
                    vec![
                        line(0, 1, 10.0, Ending::Restored(text::ConditionalStage::Normal)),
                        line(1, 2, 10.0, Ending::Restored(text::ConditionalStage::Normal)),
                        line(2, 3, 10.0, Ending::Paragraph),
                    ],
                ),
            ] {
                let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(width)), rtl);
                assert_eq!(facts(&actual, &p, "日本へ", PHRASE_FONT), (width, expected));
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn earlier_fitting_prefix_wraps_a_phrase_intact_instead_of_relaxing_into_the_suffix() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let ja = japanese(text::OverflowWrap::Anywhere);
            let prefix = text::Style {
                locale: Some("en-US".into()),
                word_break: text::WordBreak::Normal,
                ..ja.clone()
            };
            let p = PreparedParagraph::with_registered_font(
                vec![
                    ParagraphPart::text(
                        1,
                        Source::identified("AA ", text::Id::from_u64(65068), 1),
                        prefix,
                    ),
                    ParagraphPart::text(
                        2,
                        Source::identified("日本へ", text::Id::from_u64(65069), 1),
                        ja.clone(),
                    ),
                ],
                ja,
                if rtl {
                    ParagraphBaseDirection::RightToLeft
                } else {
                    ParagraphBaseDirection::LeftToRight
                },
                PHRASE_FONT,
            );
            let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(40.0)), rtl);
            assert_eq!(
                facts(&actual, &p, "AA 日本へ", PHRASE_FONT),
                (
                    40.0,
                    vec![
                        (text::Range::new(0, 3), 30.0, 20.0, Ending::Ordinary),
                        line(3, 6, 30.0, Ending::Paragraph)
                    ]
                )
            );
            assert_eq!(p.origin(1).view().source().text(), &text::Text::from("AA "));
            assert_eq!(
                p.origin(2).view().source().text(),
                &text::Text::from("日本へ")
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn intrinsic_anywhere_is_independent_of_finite_phrase_restoration_and_query_order() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            for (source, font, size) in [("日本へ", PHRASE_FONT, 30.0), ("ABCD", AHEM, 40.0)] {
                for overflow in [
                    text::OverflowWrap::Normal,
                    text::OverflowWrap::BreakWord,
                    text::OverflowWrap::Anywhere,
                ] {
                    let style = if source == "日本へ" {
                        japanese(overflow)
                    } else {
                        text::Style {
                            word_break: text::WordBreak::AutoPhrase,
                            overflow_wrap: overflow,
                            locale: Some("en-US".into()),
                            hyphens: text::Hyphens::None,
                            ..ahem_style(10.0, 10.0)
                        }
                    };
                    let p = paragraph(source, style, rtl, font);
                    assert_eq!(
                        p.prepared.phrases().count(),
                        usize::from(source == "日本へ")
                    );
                    let before = compute::<S>(&p, AvailableOf::definite(S::from_f64(20.0)), rtl);
                    let minimum = compute::<S>(&p, AvailableOf::MIN_CONTENT, rtl);
                    let minimum = facts(&minimum, &p, source, font);
                    assert_eq!(
                        minimum.0,
                        if overflow == text::OverflowWrap::Anywhere {
                            10.0
                        } else {
                            size
                        }
                    );
                    if overflow == text::OverflowWrap::Anywhere {
                        let measured = intrinsic::<S>(&p, AvailableOf::MIN_CONTENT, rtl);
                        assert_eq!(measured.report.inline_extent, S::from_f64(10.0));
                        for row in measured
                            .paragraph
                            .entries
                            .iter()
                            .take(measured.paragraph.entries.len() - 1)
                        {
                            let selected = row.finalized_line().selected_line();
                            let text::PreparedTermination::Conditional(offer) =
                                &selected.request().termination
                            else {
                                panic!("intrinsic TCU cut must retain its independent capability")
                            };
                            assert_eq!(offer.stage(), text::ConditionalStage::Arbitrary);
                            assert_eq!(offer.purpose(), text::ConditionalPurpose::MinContent);
                            assert_eq!(selected.request().context.available_inline, None);
                            assert!(selected.request().alternative.is_none());
                        }
                    } else {
                        assert_eq!(
                            minimum.1,
                            [line(
                                0,
                                source.encode_utf16().count(),
                                size,
                                Ending::Paragraph
                            )]
                        );
                    }
                    let maximum = compute::<S>(&p, AvailableOf::MAX_CONTENT, rtl);
                    assert_eq!(
                        facts(&maximum, &p, source, font),
                        (
                            size,
                            vec![line(
                                0,
                                source.encode_utf16().count(),
                                size,
                                Ending::Paragraph
                            )]
                        )
                    );
                    let after = compute::<S>(&p, AvailableOf::definite(S::from_f64(20.0)), rtl);
                    assert_eq!(
                        facts(&after, &p, source, font),
                        facts(&before, &p, source, font)
                    );
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn unsupported_phrase_language_restores_real_owner_automatic_hyphen_before_arbitrary() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let style = text::Style {
                word_break: text::WordBreak::AutoPhrase,
                overflow_wrap: text::OverflowWrap::Anywhere,
                locale: Some("en-US".into()),
                hyphens: text::Hyphens::Auto,
                hyphenate_character: text::HyphenateCharacter::String("A".into()),
                ..ahem_style(10.0, 10.0)
            };
            for (source, width, cut, first, residual, automatic) in [
                ("associate", 60.0, 4, 50.0, 50.0, true),
                ("as\u{ad}sociate", 80.0, 3, 30.0, 70.0, false),
                ("as\u{ad}sociate", 60.0, 5, 50.0, 50.0, true),
            ] {
                let p = paragraph(source, style.clone(), rtl, AHEM);
                assert_eq!(p.prepared.phrases().count(), 0);
                assert_eq!(p.prepared.alternative_views().count(), 0);
                let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(width)), rtl);
                assert_eq!(
                    facts(&actual, &p, source, AHEM),
                    (
                        width,
                        vec![
                            line(
                                0,
                                cut,
                                first,
                                Ending::Restored(text::ConditionalStage::Hyphens)
                            ),
                            line(
                                cut,
                                source.encode_utf16().count(),
                                residual,
                                Ending::Paragraph
                            )
                        ]
                    )
                );
                let lines = finalized(&actual, &p);
                let selected = lines[0].selected_line();
                let alternative = selected.request().alternative.as_ref().unwrap();
                assert_eq!(alternative.view().text(), &text::Text::from("A"));
                assert_eq!(
                    matches!(
                        alternative.view().definition(),
                        text::PreparedAlternativeDefinition::Automatic { .. }
                    ),
                    automatic
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn restored_after_and_both_indications_keep_receiving_authority_across_finite_frames() {
    fn lane<S: LayoutScalar>() {
        for placement in [
            text::HyphenationPlacement::LogicalAfter,
            text::HyphenationPlacement::BothSides,
        ] {
            let style = text::Style {
                word_break: text::WordBreak::AutoPhrase,
                overflow_wrap: text::OverflowWrap::Anywhere,
                hyphens: text::Hyphens::Manual,
                locale: Some("und-Latn".into()),
                hyphenate_character: text::HyphenateCharacter::Auto,
                native_hyphenation_convention: Some(
                    text::NativeHyphenationConvention::language_script(
                        "und-Latn",
                        "Latn",
                        "A".into(),
                        placement,
                    )
                    .unwrap(),
                ),
                ..ahem_style(10.0, 10.0)
            };
            let p = paragraph("AB\u{ad}CDE", style, false, AHEM);
            let first = if placement == text::HyphenationPlacement::LogicalAfter {
                20.0
            } else {
                30.0
            };
            for (width, expected) in [
                (
                    40.0,
                    vec![
                        line(
                            0,
                            3,
                            first,
                            Ending::Restored(text::ConditionalStage::Hyphens),
                        ),
                        line(3, 6, 40.0, Ending::Paragraph),
                    ],
                ),
                (
                    30.0,
                    vec![
                        line(
                            0,
                            3,
                            first,
                            Ending::Restored(text::ConditionalStage::Hyphens),
                        ),
                        line(
                            3,
                            5,
                            30.0,
                            Ending::Restored(text::ConditionalStage::Arbitrary),
                        ),
                        line(5, 6, 10.0, Ending::Paragraph),
                    ],
                ),
            ] {
                let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(width)), false);
                assert_eq!(facts(&actual, &p, "AB\u{ad}CDE", AHEM), (width, expected));
                let lines = finalized(&actual, &p);
                let continuation = lines[0]
                    .hyphenation_continuation()
                    .expect("selected After emission has authentic receiving custody");
                assert_eq!(
                    lines[1].selected_line().request().incoming,
                    Some(continuation)
                );
                assert!(
                    lines.iter().skip(2).all(|line| line
                        .selected_line()
                        .request()
                        .incoming
                        .is_none())
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

struct ShapeTree<S: LayoutScalar> {
    inner: PublicLayoutTreeOf<S>,
    queries: std::cell::RefCell<Vec<(f64, f64)>>,
    caches: std::collections::HashMap<u32, CacheOf<S>>,
    committed: std::collections::HashMap<u32, NodeOutputOf<S>>,
    hits: std::cell::Cell<usize>,
}
impl<S: LayoutScalar> Traverse for ShapeTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::vec::IntoIter<u32>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.inner, node)
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn child_count(&self, node: u32) -> usize {
        self.inner.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.inner.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for ShapeTree<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.inner.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.inner.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let result = self.caches.get(&node)?.get_with_context(input, context);
        if result.is_some() {
            self.hits.set(self.hits.get() + 1)
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.committed.get(&node).copied()
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 2);
        let lo = query.band_minimum().to_f64();
        let hi = query.band_maximum().to_f64();
        self.queries.borrow_mut().push((lo, hi));
        // A real static shape occupies x[0,20] only over y(10,20).
        Some(Ok(if lo < 20.0 && hi > 10.0 {
            FloatExclusionIntervalOf::try_new(query, S::ZERO, S::from_f64(20.0)).unwrap()
        } else {
            None
        }))
    }
}
impl<S: LayoutScalar> ShapeTree<S> {
    fn commit(&mut self, batch: &CompletedLayoutBatchOf<u32, S>) {
        for row in batch.cache_clear_entries() {
            self.caches.remove(&row.node());
        }
        for row in batch.cache_store_entries() {
            self.caches
                .entry(row.node())
                .or_default()
                .store_with_context(row.input(), row.context(), row.output());
        }
        self.committed = batch
            .unrounded_entries()
            .iter()
            .map(|row| (row.node(), row.output()))
            .collect();
    }
}

#[test]
fn phantom_prefix_shape_replay_allows_each_restored_candidate_its_own_height_band_and_warm_cache() {
    fn lane<S: LayoutScalar>() {
        let style = japanese(text::OverflowWrap::Anywhere);
        let mut source = Source::identified("日本へ", text::Id::from_u64(65070), 1);
        source.span(
            text::Range::new(1, 2),
            text::Style {
                size: 20.0,
                line_height: text::LineHeight::Absolute(20.0),
                ..style.clone()
            },
        );
        let p = PreparedParagraph::with_registered_font(
            vec![
                ParagraphPart::Control {
                    node: 1,
                    kind: text::ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
                },
                ParagraphPart::text(3, source, style.clone()),
            ],
            style,
            ParagraphBaseDirection::LeftToRight,
            PHRASE_FONT,
        );
        let mut binding = p.binding(0);
        binding.nodes = vec![1, 2, 3];
        let inner = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [])
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
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    float_exclusion: FloatExclusion::Shape,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(20.0)),
                        PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(
                1,
                LayoutInputOf::text_control(TextControlParticipation::new(
                    binding.controls[0].occurrence.clone(),
                )),
            )
            .input(3, p.text_input(3));
        let mut tree = ShapeTree {
            inner,
            queries: Default::default(),
            caches: Default::default(),
            committed: Default::default(),
            hits: Default::default(),
        };
        let run = |tree: &ShapeTree<S>| {
            let capture = LayoutFlowInput::try_new(
                tree,
                0,
                vec![binding.clone()],
                crate::WrappingInput {
                    scopes: Vec::new(),
                    child_order: Vec::new(),
                },
            )
            .unwrap();
            compute_layout_with_flow(
                &capture,
                0,
                LayoutRootRequestOf::viewport(Size::new(
                    AvailableOf::definite(S::from_f64(40.0)),
                    AvailableOf::MAX_CONTENT,
                ))
                .unwrap(),
            )
            .unwrap()
        };
        let cold = run(&tree);
        let observe = |batch: &CompletedLayoutBatchOf<u32, S>| {
            finalized(batch, &p)
                .iter()
                .zip(batch.placed_text()[0].layout().lines())
                .map(|(line, placed)| {
                    for view in line.run_views() {
                        assert_eq!(view.face().bytes(), PHRASE_FONT);
                        for glyph in view.glyphs() {
                            if glyph.advance() > 0.0 {
                                assert_eq!(glyph.id(), 35);
                                assert!(glyph.advance() == 10.0 || glyph.advance() == 20.0);
                            }
                        }
                    }
                    (
                        placed.range(),
                        line.feedback().advance,
                        line.feedback().line_height,
                        line.selected_line().request().context.available_inline,
                    )
                })
                .collect::<Vec<_>>()
        };
        let expected = vec![
            (text::Range::new(0, 1), 10.0, 10.0, Some(40.0)),
            (text::Range::new(1, 2), 20.0, 20.0, Some(20.0)),
            (text::Range::new(2, 3), 10.0, 10.0, Some(40.0)),
        ];
        assert_eq!(observe(&cold), expected);
        assert!(tree.queries.borrow().contains(&(0.0, 10.0)));
        assert!(tree.queries.borrow().contains(&(0.0, 20.0)));
        tree.commit(&cold);
        let warm = run(&tree);
        assert_eq!(observe(&warm), expected);
        assert_eq!(warm.final_entries(), cold.final_entries());
        assert!(
            tree.hits.get() > 0,
            "the independent float cache must actually be reused"
        );
    }
    lane::<f32>();
    lane::<f64>();
}
