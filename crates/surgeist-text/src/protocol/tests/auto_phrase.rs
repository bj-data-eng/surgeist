//! Independent functional acceptance for the native AutoPhrase protocol.
//! The full row 日本へ▁ようこそ is pinned upstream BudouX 0.9.3 gh1389;
//! isolated 日本へ was separately qualified with the original Python reference.
//! See the frozen Phrase Ahem provenance. These are new-API functional tests,
//! not executable evidence from before the API existed.

use super::*;
use crate::*;

const PHRASE_FONT: &[u8] =
    include_bytes!("../../../tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");
const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const DIRECTIONS: [ParagraphBaseDirection; 2] = [
    ParagraphBaseDirection::LeftToRight,
    ParagraphBaseDirection::RightToLeft,
];

fn phrase_style(locale: Option<&str>) -> Style {
    Style {
        font: Font::new().family("Surgeist Phrase Ahem").family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        word_break: WordBreak::AutoPhrase,
        white_space: WhiteSpace::Preserve,
        hyphens: Hyphens::Manual,
        hyphenate_character: HyphenateCharacter::String(Text::from("A")),
        locale: locale.map(str::to_owned),
        ..Style::default()
    }
}

fn input(value: &str, base_style: Style, direction: ParagraphBaseDirection) -> ParagraphInput {
    let source = Source::identified(value, Id::from_u64(65661), 4);
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn catalogue() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system.register_fonts(PHRASE_FONT.to_vec()).unwrap();
    system
}

fn prepare(input: ParagraphInput) -> PreparedText {
    let ancestor_wrap = input.base_style.wrap;
    prepare_in(input, ancestor_wrap)
}

fn prepare_in(mut input: ParagraphInput, ancestor_wrap: Wrap) -> PreparedText {
    // These fixtures author one containing block; its actual wrapping policy
    // is captured for connections/atomics and bound to missing text relations.
    let wrap = input.base_style.wrap;
    let definitions = (0..input.connections.len())
        .map(ParagraphBoundaryDefinition::Connection)
        .chain(
            input
                .source
                .boxes()
                .iter()
                .enumerate()
                .filter(|(_, b)| b.kind() == InlineBoxKind::InFlow)
                .flat_map(|(i, _)| {
                    [
                        ParagraphBoundaryDefinition::AtomicBefore(i),
                        ParagraphBoundaryDefinition::AtomicAfter(i),
                    ]
                }),
        );
    input
        .boundary_policies
        .extend(definitions.map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: wrap,
            following_wrap: wrap,
            ancestor_wrap,
        }));
    let analyzed =
        AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, ancestor_wrap);
    catalogue()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn ordinary(p: &PreparedText) -> Vec<usize> {
    p.opportunity_views()
        .filter(|offer| offer.kind() == PreparedBreakKind::Allowed)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

fn discretionary(p: &PreparedText) -> Vec<usize> {
    p.opportunity_views()
        .filter(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect()
}

fn whole(p: &PreparedText) -> FinalizedLine {
    let mut r = request(p);
    r.context.base_direction = if p.analyzed_source().direction() == Direction::RightToLeft {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    r.context.line_box_direction = if p.analyzed_source().direction() == Direction::RightToLeft {
        crate::LineBoxDirection::RightToLeft
    } else {
        crate::LineBoxDirection::LeftToRight
    };
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn first_after(p: &PreparedText, start: &PreparedBoundary) -> PreparedOpportunity {
    p.opportunity_views()
        .find(|offer| {
            offer.boundary().view().participant_offset() > start.view().participant_offset()
        })
        .unwrap()
        .handle()
}

fn interval_request(
    p: &PreparedText,
    start: PreparedBoundary,
    end: PreparedBoundary,
    termination: PreparedTermination,
    incoming: Option<HyphenationContinuation>,
    width: Option<f64>,
) -> LineSelectionRequest {
    let mut r = request(p);
    r.start = start;
    r.end = end;
    r.incoming = incoming;
    r.alternative = match &termination {
        PreparedTermination::Conditional(cut) => cut.alternative(),
        _ => None,
    };
    r.context.end = match &termination {
        PreparedTermination::Ordinary(offer)
            if offer.view().kind() == PreparedBreakKind::ParagraphEnd =>
        {
            LineEnd::Paragraph
        }
        _ => LineEnd::Soft,
    };
    r.termination = termination;
    let from = r.start.view().participant_offset();
    let to = r.end.view().participant_offset();
    r.participants = p.participants().skip(from).take(to - from).collect();
    r.controls = r
        .participants
        .iter()
        .filter_map(|part| match part {
            PreparedParticipant::Control(occurrence) => Some(ControlAllocation {
                occurrence: occurrence.clone(),
                boundary: matches!(
                    occurrence.view().kind(),
                    ParagraphControlKind::InlineBoundaryStart
                        | ParagraphControlKind::InlineBoundaryEnd
                )
                .then_some(InlineBoundaryAllocation::ZERO),
                strut: None,
            }),
            PreparedParticipant::Atomic(_) => panic!("staged phrase fixtures contain no atomics"),
            _ => None,
        })
        .collect();
    r.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.spacing = SpacingPolicy::None;
    r.context.available_inline = width;
    r
}

fn ordinary_trial(
    p: &PreparedText,
    start: PreparedBoundary,
    incoming: Option<HyphenationContinuation>,
    width: f64,
) -> FinalizedLine {
    let ceiling = first_after(p, &start);
    p.select(interval_request(
        p,
        start,
        ceiling.view().boundary(),
        PreparedTermination::Ordinary(ceiling),
        incoming,
        Some(width),
    ))
    .unwrap()
    .finalize(SpacingAllocations::None)
    .unwrap()
}

fn phrase_trial(p: &PreparedText, portion: &PreparedPhrasePortion, width: f64) -> FinalizedLine {
    p.select(interval_request(
        p,
        portion.start(),
        portion.end(),
        PreparedTermination::PhraseMeasurement(portion.clone()),
        portion.incoming(),
        Some(width),
    ))
    .unwrap()
    .finalize(SpacingAllocations::None)
    .unwrap()
}

fn conditional_trial(
    p: &PreparedText,
    cut: &PreparedConditionalOpportunity,
    incoming: Option<HyphenationContinuation>,
    width: Option<f64>,
) -> FinalizedLine {
    p.select(interval_request(
        p,
        cut.start(),
        cut.boundary(),
        PreparedTermination::Conditional(cut.clone()),
        incoming,
        width,
    ))
    .unwrap()
    .finalize(SpacingAllocations::None)
    .unwrap()
}

fn endpoints(batch: &PreparedConditionalBatch) -> Vec<usize> {
    batch
        .opportunities()
        .iter()
        .map(|cut| cut.boundary().view().source_after().get())
        .collect()
}

#[test]
fn declared_phrase_locale_and_overflow_remain_authored_and_hash_distinct() {
    let mut keys = Vec::new();
    for locale in [
        Some("ja"),
        Some("ja-Jpan-JP-u-ca-japanese"),
        None,
        Some("und-Jpan"),
        Some("en-JP"),
        Some("ja-Latn"),
    ] {
        for overflow in [
            OverflowWrap::Normal,
            OverflowWrap::BreakWord,
            OverflowWrap::Anywhere,
        ] {
            let mut style = phrase_style(locale);
            style.overflow_wrap = overflow;
            let validated = ValidatedStyle::try_from(style.clone()).unwrap();
            assert_eq!(validated.authored(), &style);
            let mut value = input("日本へ", style.clone(), DIRECTIONS[0]);
            value.source.span(Range::new(0, 3), style.clone());
            let source = value.source.clone();
            let prepared = prepare(value);
            assert_eq!(prepared.analyzed_source().style(), &style);
            assert_eq!(
                prepared.analyzed_source().source().spans()[0].style(),
                &style
            );
            assert_eq!(prepared.analyzed_source().source(), &source);
            let source = ValidatedSource::try_from(source).unwrap();
            let options = ValidatedOptions::try_from(Options::default()).unwrap();
            let key = Key::from_validated(
                &source,
                &validated,
                options,
                prepared.font_catalogue_identity(),
            );
            let mut normal = style;
            normal.word_break = WordBreak::Normal;
            let normal = Key::from_validated(
                &source,
                &ValidatedStyle::try_from(normal).unwrap(),
                options,
                prepared.font_catalogue_identity(),
            );
            assert_ne!(key.styles(), normal.styles());
            assert!(keys.iter().all(|old: &Key| old.styles() != key.styles()));
            keys.push(key);
        }
    }
}

#[test]
fn published_phrases_and_supported_empty_inference_differ_from_unsupported_locale() {
    for direction in DIRECTIONS {
        for (text, normal, phrase) in [
            ("日本へ", vec![1, 2], vec![]),
            ("日本へようこそ", (1..7).collect(), vec![3]),
        ] {
            let mut base = phrase_style(Some("ja"));
            base.word_break = WordBreak::Normal;
            let p = prepare(input(text, base, direction));
            assert_eq!(ordinary(&p), normal);
            for locale in [
                Some("ja"),
                Some("ja-Jpan-JP-u-ca-japanese"),
                None,
                Some("und"),
                Some("und-Jpan"),
                Some("en-JP"),
                Some("ja-Latn"),
            ] {
                let p = prepare(input(text, phrase_style(locale), direction));
                assert_eq!(p.analyzed_source().style().locale.as_deref(), locale);
                assert_eq!(
                    p.analyzed_source().source().text().code_units(),
                    text.encode_utf16().collect::<Vec<_>>()
                );
                assert_eq!(
                    ordinary(&p),
                    if locale.is_some_and(|l| l.starts_with("ja") && l != "ja-Latn") {
                        phrase.clone()
                    } else {
                        normal.clone()
                    },
                    "{text}/{locale:?}/{direction:?}"
                );
                assert_eq!(p.control_views().count(), 0);
                assert_eq!(p.virtual_separators().count(), 0);
                let supported = matches!(locale, Some("ja" | "ja-Jpan-JP-u-ca-japanese"));
                let phrases = p.phrases().collect::<Vec<_>>();
                assert_eq!(
                    phrases.len(),
                    if supported {
                        if text == "日本へ" { 1 } else { 2 }
                    } else {
                        0
                    }
                );
                let spans = phrases
                    .iter()
                    .map(|phrase| {
                        assert_eq!(phrase.prepared_text(), p);
                        assert_eq!(
                            phrase.provenance(),
                            PreparedPhraseProvenance::BudouXJapanese093
                        );
                        let span = phrase.presentation_range();
                        let pieces = phrase.pieces();
                        let mut covered = Vec::new();
                        for piece in pieces {
                            let PreparedPhrasePieceOwner::Original {
                                origin,
                                aggregate,
                                local,
                            } = piece.owner
                            else {
                                panic!(
                                    "plain source must not fabricate control/virtual phrase pieces"
                                );
                            };
                            assert_eq!(origin.view().source().id(), Some(Id::from_u64(65661)));
                            assert_eq!(aggregate, local);
                            covered.extend(aggregate.start..aggregate.end);
                        }
                        assert_eq!(covered, (span.start()..span.end()).collect::<Vec<_>>());
                        (span.start(), span.end())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    spans,
                    if !supported {
                        vec![]
                    } else if text == "日本へ" {
                        vec![(0, 3)]
                    } else {
                        vec![(0, 3), (3, 7)]
                    }
                );
            }
        }
    }
}

#[test]
fn phrase_context_uses_final_full_size_kana_across_style_font_and_origin_splits() {
    for direction in DIRECTIONS {
        let mut base = phrase_style(Some("ja"));
        base.text_transform =
            TextTransform::Set(TextTransformSet::none().with_full_size_kana(true));
        let source_text = "日本へょぅこそ";
        let mut value = input(source_text, base.clone(), direction);
        value.source.span(
            Range::new(1, 3),
            Style {
                brush: Brush::color(0.3, 0.4, 0.5, 1.0),
                size: 11.0,
                ..base.clone()
            },
        );
        value.source.span(
            Range::new(5, 7),
            Style {
                font: Font::new().family("Ahem"),
                ..base
            },
        );
        // The final two letters deliberately select a different real face.
        // Only opportunity/provenance assertions depend on that fragment; no
        // Japanese glyph coverage or geometry is claimed for original Ahem.
        value.origins = [Range::new(0, 1), Range::new(1, 3), Range::new(3, 7)]
            .into_iter()
            .enumerate()
            .map(|(i, aggregate)| {
                let text = Text::from_code_units(
                    value.source.text().code_units()[aggregate.start..aggregate.end].to_vec(),
                );
                ParagraphOrigin {
                    text_node: None,
                    source: Source::identified(text, Id::from_u64(65670 + i as u64), 9),
                    local: Range::new(0, aggregate.len()),
                    aggregate,
                    kind: ParagraphOriginKind::Authored,
                }
            })
            .collect();
        value.connections = [1, 3]
            .into_iter()
            .map(|at| ParagraphConnection {
                at,
                kind: ParagraphConnectionKind::Continuous,
            })
            .collect();
        let p = prepare(value);
        assert_eq!(ordinary(&p), [3]);
        assert_eq!(p.origin_views().count(), 3);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            source_text.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(p.control_views().count(), 0);
        assert_eq!(p.virtual_separators().count(), 0);
        for phrase in p.phrases() {
            for piece in phrase.pieces() {
                let PreparedPhrasePieceOwner::Original {
                    origin,
                    aggregate,
                    local,
                } = piece.owner
                else {
                    panic!("transform/font/origin fragmentation adds no source controls");
                };
                let (id, offset) = if aggregate.end <= 1 {
                    (65670, 0)
                } else if aggregate.end <= 3 {
                    (65671, 1)
                } else {
                    (65672, 3)
                };
                assert_eq!(origin.view().source().id(), Some(Id::from_u64(id)));
                assert_eq!(
                    local,
                    Range::new(aggregate.start - offset, aggregate.end - offset)
                );
            }
        }

        // With one selected qualified face, actual glyph35 for the transformed
        // small kana proves execution: neither original small kana has cmap
        // coverage, whereas the full-size published row does.
        let mut base = phrase_style(Some("ja"));
        base.text_transform =
            TextTransform::Set(TextTransformSet::none().with_full_size_kana(true));
        let p = prepare(input(source_text, base, direction));
        let line = whole(&p);
        assert_eq!(ordinary(&p), [3]);
        assert_eq!(line.feedback().advance, 70.0);
        let mut glyphs = line
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), PHRASE_FONT);
                run.glyphs()
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        glyphs.sort_by_key(|g| g.0.start);
        assert_eq!(
            glyphs,
            (0..7)
                .map(|i| (Range::new(i, i + 1), 35, 10.0))
                .collect::<Vec<_>>()
        );
        assert!(
            line.mapping_views()
                .all(|piece| piece.kind() == PresentationMappingKind::Source)
        );
    }
}

#[test]
fn following_logical_owner_controls_suppression_in_both_paragraph_directions() {
    for direction in DIRECTIONS {
        for following_phrase in [false, true] {
            let mut base = phrase_style(Some("ja"));
            if following_phrase {
                base.word_break = WordBreak::Normal;
            }
            let mut value = input("日本へ", base, direction);
            let mut following = phrase_style(Some("ja"));
            if !following_phrase {
                following.word_break = WordBreak::Normal;
            }
            value.source.span(Range::new(1, 3), following.clone());
            let p = prepare(value);
            assert_eq!(p.analyzed_source().source().spans()[0].style(), &following);
            assert_eq!(
                ordinary(&p),
                if following_phrase { vec![] } else { vec![1, 2] }
            );
        }
    }
}

#[test]
fn initial_hyphen_exclusion_uses_actual_owner_even_without_japanese_support() {
    for locale in [
        None,
        Some("ja"),
        Some("en-US"),
        Some("und-Jpan"),
        Some("ja-Latn"),
    ] {
        for indication in ["", "A"] {
            let mut base = phrase_style(locale);
            base.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
            let p = prepare(input("AX\u{ad}XXX", base.clone(), DIRECTIONS[0]));
            assert!(discretionary(&p).is_empty());
            assert_eq!(p.alternative_views().count(), 0);
            let mut value = input("AX\u{ad}XXX", base.clone(), DIRECTIONS[0]);
            let mut normal = base.clone();
            normal.word_break = WordBreak::Normal;
            value.source.span(Range::new(2, 3), normal.clone());
            assert_eq!(
                discretionary(&prepare(value)),
                [3],
                "the actual SHY owner is Normal"
            );
            let mut value = input("AX\u{ad}XXX", base, DIRECTIONS[0]);
            value.source.span(Range::new(3, 6), normal);
            assert!(
                discretionary(&prepare(value)).is_empty(),
                "a neighbor cannot donate hyphen eligibility"
            );
        }
    }
    let mut base = phrase_style(Some("en-US"));
    base.hyphens = Hyphens::Auto;
    let p = prepare(input("associate", base.clone(), DIRECTIONS[0]));
    assert!(discretionary(&p).is_empty());
    assert_eq!(p.alternative_views().count(), 0);
    base.word_break = WordBreak::Normal;
    assert_eq!(
        discretionary(&prepare(input("associate", base, DIRECTIONS[0]))),
        [2, 4]
    );
}

#[test]
fn explicit_wbr_source_zwsp_and_required_line_break_causes_survive_phrase_preference() {
    for direction in DIRECTIONS {
        for control_wrap in [Wrap::Word, Wrap::None] {
            let mut value = input("日本へ", phrase_style(Some("ja")), direction);
            let mut owner = phrase_style(Some("ja"));
            owner.wrap = control_wrap;
            value.controls.push(ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::Wbr {
                    style: Box::new(owner),
                },
            });
            value.anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: Affinity::After,
            });
            let p = prepare(value);
            let control = p.control_views().next().unwrap().handle();
            assert!(
                matches!(control.view().kind(), ParagraphControlKind::Wbr { style }
                if style.word_break == WordBreak::AutoPhrase && style.wrap == control_wrap)
            );
            let slot = p
                .participants()
                .position(|part| {
                    matches!(part,
                PreparedParticipant::Control(ref actual) if *actual == control)
                })
                .unwrap();
            let boundary = p
                .boundary_views()
                .find(|b| b.participant_offset() == slot + 1)
                .unwrap()
                .handle();
            let offers = p
                .opportunity_views()
                .filter(|offer| offer.boundary() == boundary)
                .collect::<Vec<_>>();
            assert_eq!(offers.len(), usize::from(control_wrap == Wrap::Word));
            for offer in offers {
                assert_eq!(offer.kind(), PreparedBreakKind::Allowed);
                assert_eq!(offer.alternatives().count(), 0);
            }
        }
        let p = prepare(input("日本\u{200b}へ", phrase_style(Some("ja")), direction));
        let zw = p
            .opportunity_views()
            .find(|offer| {
                offer.kind() == PreparedBreakKind::Allowed
                    && offer.boundary().view().source_after().get() == 3
            })
            .expect("the actual source ZWSP offer survives");
        assert_eq!(zw.alternatives().count(), 0);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [0x65e5, 0x672c, 0x200b, 0x3078]
        );

        let mut anywhere = phrase_style(Some("ja"));
        anywhere.line_break = LineBreak::Anywhere;
        assert_eq!(
            ordinary(&prepare(input("日本へ", anywhere, direction))),
            [1, 2]
        );
        let mut loose = phrase_style(Some("ja"));
        loose.line_break = LineBreak::Loose;
        // CSS Text's required CJ distinction is independent of phrase input.
        assert_eq!(ordinary(&prepare(input("日ぁ", loose, direction))), [1]);
    }
}

#[test]
fn independent_anywhere_and_atomic_offers_keep_real_tcus_nca_and_hard_guards() {
    for direction in DIRECTIONS {
        let mut base = phrase_style(Some("ja"));
        base.line_break = LineBreak::Anywhere;
        let p = prepare(input("日\u{301}本へ", base.clone(), direction));
        assert_eq!(
            p.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 3), Range::new(3, 4)]
        );
        assert_eq!(ordinary(&p), [2, 3]);
        let p = prepare_in(input("日\u{301}本へ", base, direction), Wrap::None);
        assert!(ordinary(&p).is_empty());

        for ancestor in [Wrap::Word, Wrap::None] {
            let mut value = input("日本へ", phrase_style(Some("ja")), direction);
            value.source.inline_box(InlineBox::new(
                Id::from_u64(65680),
                InlineBoxKind::InFlow,
                1,
                Size::new(10.0, 10.0),
            ));
            value.anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: Affinity::After,
            });
            let p = prepare_in(value, ancestor);
            let atomic = p.atomic_views().next().unwrap().handle();
            assert_eq!(atomic.view().original_box().id(), Id::from_u64(65680));
            let slot = p
                .participants()
                .position(|part| {
                    matches!(part,
                PreparedParticipant::Atomic(ref actual) if *actual == atomic)
                })
                .unwrap();
            let actual = p
                .boundary_views()
                .filter(|b| [slot, slot + 1].contains(&b.participant_offset()))
                .map(|b| b.handle())
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), 2);
            for boundary in actual {
                assert_eq!(
                    p.opportunity_views()
                        .any(|offer| offer.kind() == PreparedBreakKind::Allowed
                            && offer.boundary() == boundary),
                    ancestor == Wrap::Word
                );
            }
        }
        let p = prepare(input("日\n本へ", phrase_style(Some("ja")), direction));
        assert_eq!(
            p.opportunity_views()
                .filter(|offer| offer.kind() == PreparedBreakKind::Forced)
                .map(|offer| offer.boundary().view().source_after().get())
                .collect::<Vec<_>>(),
            [2]
        );
        assert!(
            !ordinary(&p).contains(&1),
            "no soft offer immediately before a hard break"
        );
    }
}

#[test]
fn phrase_measurements_are_complete_nonplaceable_and_semantically_repeatable() {
    use std::hash::{Hash, Hasher};
    let p = prepare(input("日本へ", phrase_style(Some("ja")), DIRECTIONS[0]));
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = first_after(&p, &start);
    let frame = ordinary_trial(&p, start.clone(), None, 20.0);
    let again_frame = ordinary_trial(&p, start.clone(), None, 20.0);
    assert_eq!(frame.feedback().advance, 30.0);
    let search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&frame),
        )
        .unwrap();
    let again = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&again_frame),
        )
        .unwrap();
    assert_eq!(
        search, again,
        "fresh trial allocations do not change semantic search identity"
    );
    let mut a = std::collections::hash_map::DefaultHasher::new();
    let mut b = std::collections::hash_map::DefaultHasher::new();
    search.hash(&mut a);
    again.hash(&mut b);
    assert_eq!(a.finish(), b.finish());
    let portions = search.phrase_portions();
    assert_eq!(portions.len(), 1);
    let portion = &portions[0];
    assert_eq!(portion.start(), start);
    assert_eq!(portion.end(), ceiling.view().boundary());
    assert_eq!(portion.incoming(), None);
    assert_eq!(
        (
            portion.phrase().presentation_range().start(),
            portion.phrase().presentation_range().end()
        ),
        (0, 3)
    );
    let trial = phrase_trial(&p, portion, 20.0);
    assert_eq!(
        trial.feedback().advance,
        30.0,
        "measure all three TCUs, never only a fitting prefix"
    );
    assert!(place(p.clone(), trial.clone(), 0.0, 8.0).is_err());
    assert!(
        search.begin(&[]).is_err(),
        "an unmeasured affected piece cannot authorize relaxation"
    );
    let batch = search
        .begin(&[PhraseOverflowDecision::Overflows(trial)])
        .unwrap();
    let again_trial = phrase_trial(&p, &again.phrase_portions()[0], 20.0);
    assert_eq!(
        batch,
        again
            .begin(&[PhraseOverflowDecision::Overflows(again_trial)])
            .unwrap()
    );
    assert_eq!(batch.stage(), ConditionalStage::Normal);
    assert_eq!(endpoints(&batch), [1, 2]);
    assert!(
        batch
            .opportunities()
            .iter()
            .all(|cut| cut.alternative().is_none())
    );
    let finite = &batch.opportunities()[1];
    let intrinsic = interval_request(
        &p,
        finite.start(),
        finite.boundary(),
        PreparedTermination::Conditional(finite.clone()),
        None,
        None,
    );
    assert!(
        p.select(intrinsic).is_err(),
        "a finite restored endpoint carries no intrinsic authority"
    );
    let changed_start = interval_request(
        &p,
        batch.opportunities()[0].boundary(),
        finite.boundary(),
        PreparedTermination::Conditional(finite.clone()),
        None,
        Some(20.0),
    );
    assert!(
        p.select(changed_start).is_err(),
        "a finite capability cannot authorize a different current start"
    );

    let frame = ordinary_trial(&p, start.clone(), None, 40.0);
    let search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&frame),
        )
        .unwrap();
    let trial = phrase_trial(&p, &search.phrase_portions()[0], 40.0);
    assert_eq!(trial.feedback().advance, 30.0);
    assert!(
        search
            .begin(&[PhraseOverflowDecision::Fits(trial)])
            .unwrap()
            .opportunities()
            .is_empty()
    );
}

#[test]
fn finite_normal_hyphen_and_arbitrary_stages_require_exact_complete_trials() {
    let mut base = phrase_style(Some("ja"));
    base.overflow_wrap = OverflowWrap::Anywhere;
    let p = prepare(input("日本へ", base, DIRECTIONS[0]));
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = first_after(&p, &start);
    let frame = ordinary_trial(&p, start.clone(), None, 5.0);
    let search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&frame),
        )
        .unwrap();
    let trial = phrase_trial(&p, &search.phrase_portions()[0], 5.0);
    let normal = search
        .begin(&[PhraseOverflowDecision::Overflows(trial)])
        .unwrap();
    assert_eq!(normal.stage(), ConditionalStage::Normal);
    assert_eq!(endpoints(&normal), [1, 2]);
    assert!(normal.advance(&[]).is_err());
    let trials = normal
        .opportunities()
        .iter()
        .map(|cut| {
            let line = conditional_trial(&p, cut, None, Some(5.0));
            assert!(
                line.feedback().advance > 5.0,
                "these actual prefixes overflow the finite frame"
            );
            ConditionalDisposition::Exhausted(line)
        })
        .collect::<Vec<_>>();
    assert!(normal.advance(&trials[..1]).is_err());
    let reversed = trials.iter().rev().cloned().collect::<Vec<_>>();
    assert!(normal.advance(&reversed).is_err());
    let hyphens = normal.advance(&trials).unwrap().unwrap();
    assert_eq!(hyphens.stage(), ConditionalStage::Hyphens);
    assert!(hyphens.opportunities().is_empty());
    let arbitrary = hyphens.advance(&[]).unwrap().unwrap();
    assert_eq!(arbitrary.stage(), ConditionalStage::Arbitrary);
    assert_eq!(endpoints(&arbitrary), [1, 2]);
    assert_ne!(normal.opportunities()[0], arbitrary.opportunities()[0]);
    assert!(arbitrary.advance(&[]).unwrap().is_none());
}

#[test]
fn intrinsic_anywhere_has_its_own_authority_and_cannot_unlock_finite_relaxation() {
    for overflow in [
        OverflowWrap::Normal,
        OverflowWrap::BreakWord,
        OverflowWrap::Anywhere,
    ] {
        let mut base = phrase_style(Some("ja"));
        base.overflow_wrap = overflow;
        let p = prepare(input("日本へ", base, DIRECTIONS[0]));
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = first_after(&p, &start);
        let intrinsic = p
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::MinContent,
                &[],
                None,
            )
            .unwrap();
        assert!(intrinsic.phrase_portions().is_empty());
        let batch = intrinsic.begin(&[]).unwrap();
        assert_eq!(batch.stage(), ConditionalStage::Arbitrary);
        assert_eq!(
            endpoints(&batch),
            if overflow == OverflowWrap::Anywhere {
                vec![1, 2]
            } else {
                vec![]
            }
        );
        assert!(
            p.conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                None
            )
            .is_err()
        );
        for cut in batch.opportunities() {
            let line = conditional_trial(&p, cut, None, None);
            assert_eq!(
                line.feedback().advance,
                cut.boundary().view().source_after().get() as f64 * 10.0
            );
            let finite = interval_request(
                &p,
                cut.start(),
                cut.boundary(),
                PreparedTermination::Conditional(cut.clone()),
                None,
                Some(20.0),
            );
            assert!(p.select(finite).is_err());
        }
        assert!(
            ordinary(&p).is_empty(),
            "intrinsic eligibility leaves ordinary phrase inventory intact"
        );
    }
}

#[test]
fn a_restored_cut_reissues_only_the_actual_remaining_phrase_on_the_next_line() {
    let p = prepare(input("日本へ", phrase_style(Some("ja")), DIRECTIONS[0]));
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = first_after(&p, &start);
    let frame = ordinary_trial(&p, start.clone(), None, 20.0);
    let search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&frame),
        )
        .unwrap();
    let original = search.phrase_portions()[0].clone();
    let trial = phrase_trial(&p, &original, 20.0);
    let normal = search
        .begin(&[PhraseOverflowDecision::Overflows(trial.clone())])
        .unwrap();
    let cut = normal
        .opportunities()
        .iter()
        .find(|cut| cut.boundary().view().source_after().get() == 2)
        .unwrap();
    let first = conditional_trial(&p, cut, None, Some(20.0));
    assert_eq!(first.feedback().advance, 20.0);
    let remaining_start = cut.boundary();
    let remaining_ceiling = first_after(&p, &remaining_start);
    let second = ordinary_trial(&p, remaining_start.clone(), None, 20.0);
    assert_eq!(second.feedback().advance, 10.0);
    let remaining = p
        .conditional_search(
            &remaining_start,
            None,
            &remaining_ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&second),
        )
        .unwrap();
    let portions = remaining.phrase_portions();
    assert_eq!(
        portions.len(),
        1,
        "a residual retains full phrase identity even with no internal cut left"
    );
    let portion = &portions[0];
    assert_eq!(portion.phrase(), original.phrase());
    assert_eq!(portion.start(), remaining_start);
    assert_eq!(portion.end(), original.end());
    assert_eq!(
        (
            portion.phrase().presentation_range().start(),
            portion.phrase().presentation_range().end()
        ),
        (0, 3)
    );
    assert!(
        remaining
            .begin(&[PhraseOverflowDecision::Overflows(trial)])
            .is_err(),
        "the previous line's full phrase receipt is stale"
    );
    let trial = phrase_trial(&p, portion, 20.0);
    assert_eq!(
        trial.feedback().advance,
        10.0,
        "the consumed prefix is not measured again"
    );
    assert!(
        remaining
            .begin(&[PhraseOverflowDecision::Fits(trial)])
            .unwrap()
            .opportunities()
            .is_empty()
    );
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: [first, second]
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                crate::PlacementEntry::Formatted(LinePlacement {
                    line,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0 + i as f64 * 10.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source(), p.analyzed_source().source());
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
}

#[test]
fn phrase_receipts_reject_foreign_start_context_and_measurement_mismatches() {
    let p = prepare(input("日本へ", phrase_style(Some("ja")), DIRECTIONS[0]));
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = first_after(&p, &start);
    let frame = ordinary_trial(&p, start.clone(), None, 20.0);
    let search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&frame),
        )
        .unwrap();
    let portion = search.phrase_portions()[0].clone();
    let trial = phrase_trial(&p, &portion, 20.0);
    assert!(
        p.conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&trial)
        )
        .is_err(),
        "a nonplaceable measurement is not an ordinary search frame"
    );
    assert!(
        search
            .begin(&[PhraseOverflowDecision::Overflows(frame.clone())])
            .is_err(),
        "an ordinary trial is not a phrase measurement"
    );
    let other = prepare(input("日本へ", phrase_style(Some("ja")), DIRECTIONS[0]));
    let other_start = other.boundary_views().next().unwrap().handle();
    let other_ceiling = first_after(&other, &other_start);
    let other_frame = ordinary_trial(&other, other_start.clone(), None, 20.0);
    assert!(
        p.conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&other_frame)
        )
        .is_err()
    );
    assert!(
        other
            .conditional_search(
                &start,
                None,
                &other_ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&other_frame)
            )
            .is_err()
    );
    let other_search = other
        .conditional_search(
            &other_start,
            None,
            &other_ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&other_frame),
        )
        .unwrap();
    assert!(
        other_search
            .begin(&[PhraseOverflowDecision::Overflows(trial.clone())])
            .is_err()
    );

    let changed_frame = ordinary_trial(&p, start.clone(), None, 40.0);
    let changed_search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&changed_frame),
        )
        .unwrap();
    assert_ne!(search, changed_search);
    assert!(
        changed_search
            .begin(&[PhraseOverflowDecision::Overflows(trial)])
            .is_err()
    );
    // The prospective otherwise-empty line may have a different frame from
    // the failed ordinary line. Its real measurement context belongs to the
    // decision identity; it is not forcibly equal to the failed line's width.
    let fresh = phrase_trial(&p, &portion, 40.0);
    assert_eq!(fresh.feedback().advance, 30.0);
    let fits = search
        .begin(&[PhraseOverflowDecision::Fits(fresh)])
        .unwrap();
    assert!(fits.opportunities().is_empty());
    let wider = phrase_trial(&p, &portion, 50.0);
    let wider_fits = search
        .begin(&[PhraseOverflowDecision::Fits(wider)])
        .unwrap();
    assert!(wider_fits.opportunities().is_empty());
    assert_ne!(
        fits, wider_fits,
        "visible fresh measurement context participates in identity"
    );
    let shortened = p
        .boundary_views()
        .find(|b| b.source_after().get() == 2)
        .unwrap()
        .handle();
    let wrong_extent = interval_request(
        &p,
        portion.start(),
        shortened,
        PreparedTermination::PhraseMeasurement(portion.clone()),
        None,
        Some(20.0),
    );
    assert!(
        p.select(wrong_extent).is_err(),
        "the fitting prefix is not the issued whole portion"
    );
    let intrinsic = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::MinContent,
            &[],
            None,
        )
        .unwrap();
    assert!(
        intrinsic
            .begin(&[PhraseOverflowDecision::Overflows(phrase_trial(
                &p, &portion, 20.0
            ))])
            .is_err()
    );
}

#[test]
fn unsupported_phrase_language_restores_only_eligible_actual_hyphen_variants_before_arbitrary() {
    for (hyphens, indication, expected) in [
        (Hyphens::Manual, "", vec![3]),
        (Hyphens::Manual, "A", vec![3]),
        (Hyphens::None, "A", vec![]),
    ] {
        let mut base = phrase_style(Some("en-US"));
        base.hyphens = hyphens;
        base.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
        base.overflow_wrap = OverflowWrap::Anywhere;
        let p = prepare(input("AX\u{ad}XXX", base, DIRECTIONS[0]));
        assert_eq!(p.phrases().count(), 0);
        assert_eq!(
            p.alternative_views().count(),
            0,
            "latent variants do not leak through ordinary enumeration"
        );
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = first_after(&p, &start);
        let frame = ordinary_trial(&p, start.clone(), None, 5.0);
        let search = p
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&frame),
            )
            .unwrap();
        assert!(
            search.phrase_portions().is_empty(),
            "unsupported is not supported-empty Japanese"
        );
        let normal = search.begin(&[]).unwrap();
        assert_eq!(normal.stage(), ConditionalStage::Normal);
        assert!(normal.opportunities().is_empty());
        let hyphens = normal.advance(&[]).unwrap().unwrap();
        assert_eq!(hyphens.stage(), ConditionalStage::Hyphens);
        assert_eq!(endpoints(&hyphens), expected);
        let mut trials = Vec::new();
        for cut in hyphens.opportunities() {
            let alternative = cut
                .alternative()
                .expect("an exact restored variant, not an invented no-alternative offer");
            assert_eq!(
                alternative.view().text().code_units(),
                indication.encode_utf16().collect::<Vec<_>>()
            );
            let mut invalid = interval_request(
                &p,
                cut.start(),
                cut.boundary(),
                PreparedTermination::Conditional(cut.clone()),
                None,
                Some(5.0),
            );
            invalid.alternative = None;
            assert!(p.select(invalid).is_err());
            let trial = conditional_trial(&p, cut, None, Some(5.0));
            assert!(trial.feedback().advance > 5.0);
            trials.push(ConditionalDisposition::Exhausted(trial));
        }
        if !hyphens.opportunities().is_empty() {
            assert!(hyphens.advance(&[]).is_err());
        }
        let arbitrary = hyphens.advance(&trials).unwrap().unwrap();
        assert_eq!(arbitrary.stage(), ConditionalStage::Arbitrary);
        assert!(!arbitrary.opportunities().is_empty());
        assert!(
            arbitrary
                .opportunities()
                .iter()
                .all(|cut| cut.alternative().is_none())
        );
        assert_eq!(p.alternative_views().count(), 0);
    }
}

#[test]
fn after_and_both_latent_controls_require_real_counts_and_exact_incoming_custody() {
    for placement in [
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        let mut base = phrase_style(Some("ja-Jpan-x-native"));
        base.native_hyphenation_convention = Some(
            NativeHyphenationConvention::language_script(
                "ja-Jpan-x-native",
                "Jpan",
                Text::from("A"),
                placement,
            )
            .unwrap(),
        );
        base.hyphenate_limit_chars = HyphenateLimitChars {
            total: HyphenationMinimum::Count(0),
            before: HyphenationMinimum::Count(0),
            after: HyphenationMinimum::Count(0),
        };
        let mut value = input("日本へ", base.clone(), DIRECTIONS[0]);
        value.controls.push(ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(base),
            },
        });
        value.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        });
        let p = prepare(value);
        // Zero minima do not manufacture missing native word/count authority.
        // The qualified Japanese phrase stays real, but this control cannot
        // serve as an admitted After/Both variant under the current resources.
        assert_eq!(p.phrases().count(), 1);
        let cut = p
            .hyphenation_cut_views()
            .find(|cut| cut.provenance() == HyphenationProvenance::Control)
            .unwrap();
        assert_eq!(cut.counts(), None);
        assert!(cut.opportunity().is_none());
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = first_after(&p, &start);
        let frame = ordinary_trial(&p, start.clone(), None, 5.0);
        let search = p
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&frame),
            )
            .unwrap();
        let trial = phrase_trial(&p, &search.phrase_portions()[0], 5.0);
        let normal = search
            .begin(&[PhraseOverflowDecision::Overflows(trial)])
            .unwrap();
        let exhausted = normal
            .opportunities()
            .iter()
            .map(|cut| {
                let trial = conditional_trial(&p, cut, None, Some(5.0));
                assert!(trial.feedback().advance > 5.0);
                ConditionalDisposition::Exhausted(trial)
            })
            .collect::<Vec<_>>();
        assert!(
            normal
                .advance(&exhausted)
                .unwrap()
                .unwrap()
                .opportunities()
                .is_empty()
        );

        // This genuine Latin word supplies independently countable five units;
        // unsupported phrase admission does not invent a Japanese phrase here.
        let mut base = phrase_style(Some("en-Latn-x-native"));
        base.native_hyphenation_convention = Some(
            NativeHyphenationConvention::language_script(
                "en-Latn-x-native",
                "Latn",
                Text::from("A"),
                placement,
            )
            .unwrap(),
        );
        base.hyphenate_limit_chars = HyphenateLimitChars {
            total: HyphenationMinimum::Count(0),
            before: HyphenationMinimum::Count(0),
            after: HyphenationMinimum::Count(0),
        };
        let mut value = input("AAAAA", base.clone(), DIRECTIONS[0]);
        value.controls.push(ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(base),
            },
        });
        value.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        });
        let p = prepare(value);
        assert_eq!(p.phrases().count(), 0);
        assert!(ordinary(&p).is_empty());
        assert!(discretionary(&p).is_empty());
        let cut = p
            .hyphenation_cut_views()
            .find(|cut| cut.provenance() == HyphenationProvenance::Control)
            .unwrap();
        assert_eq!(
            cut.counts(),
            Some(HyphenationCounts {
                total: 5,
                before: 1,
                after: 4
            })
        );
        let control = p.control_views().next().unwrap().handle();
        assert!(
            matches!(control.view().kind(), ParagraphControlKind::DiscretionaryBreak { style }
            if style.word_break == WordBreak::AutoPhrase)
        );
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = first_after(&p, &start);
        let frame = ordinary_trial(&p, start.clone(), None, 20.0);
        assert_eq!(frame.feedback().advance, 50.0);
        let search = p
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&frame),
            )
            .unwrap();
        assert!(search.phrase_portions().is_empty());
        let normal = search.begin(&[]).unwrap();
        assert!(normal.opportunities().is_empty());
        let hyphens = normal.advance(&[]).unwrap().unwrap();
        assert_eq!(hyphens.opportunities().len(), 1);
        let cut = &hyphens.opportunities()[0];
        let alternative = cut.alternative().unwrap();
        assert!(
            matches!(alternative.view().definition(), PreparedAlternativeDefinition::Control { control: actual, .. } if actual == control)
        );
        let first = conditional_trial(&p, cut, None, Some(20.0));
        assert_eq!(
            first.feedback().advance,
            if placement == HyphenationPlacement::LogicalAfter {
                10.0
            } else {
                20.0
            }
        );
        let incoming = first
            .hyphenation_continuation()
            .expect("the actual After emission is owed");
        assert_eq!(incoming.alternative(), alternative);
        assert_eq!(incoming.boundary(), cut.boundary());
        let residual_start = cut.boundary();
        let residual_ceiling = first_after(&p, &residual_start);
        let second = ordinary_trial(&p, residual_start.clone(), Some(incoming.clone()), 50.0);
        assert_eq!(
            second.feedback().advance,
            50.0,
            "receiving A plus the complete four-unit suffix"
        );
        let remaining = p
            .conditional_search(
                &residual_start,
                Some(&incoming),
                &residual_ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&second),
            )
            .unwrap();
        assert!(remaining.phrase_portions().is_empty());
        assert!(remaining.begin(&[]).unwrap().opportunities().is_empty());
        assert!(
            p.conditional_search(
                &residual_start,
                None,
                &residual_ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&second)
            )
            .is_err()
        );
        assert!(second.hyphenation_continuation().is_none());
        let place_lines = |lines: Vec<FinalizedLine>| {
            Layout::from_finalized_lines(PlacementRequest {
                prepared: p.clone(),
                coverage: PlacementCoverage::WholeParagraph,
                entries: lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, line)| {
                        crate::PlacementEntry::Formatted(LinePlacement {
                            line,
                            anchor: LineAnchor {
                                inline_origin: 0.0,
                                baseline: 8.0 + i as f64 * 10.0,
                            },
                            orientation: LineOrientation::Horizontal,
                            atomics: vec![],
                        })
                    })
                    .collect(),
            })
        };
        let omitted = ordinary_trial(&p, residual_start, None, 50.0);
        assert_eq!(omitted.feedback().advance, 40.0);
        assert!(
            place_lines(vec![first.clone(), omitted]).is_err(),
            "publication cannot silently drop the pending After emission"
        );
        let layout = place_lines(vec![first, second]).unwrap();
        assert_eq!(layout.source().text().code_units(), [65; 5]);
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 5)]
        );
    }
}

#[test]
fn supported_phrase_measurement_authenticates_after_and_both_incoming_on_the_residual() {
    // Actual primary ja with no explicit script admits the pinned model; it
    // does not infer a language from these Latin glyphs. The hash-verified
    // original Python Parser independently returns ["AAAAA"] for this complete
    // input. This is inference conformance, not a Japanese quality judgment.
    for placement in [
        HyphenationPlacement::LogicalAfter,
        HyphenationPlacement::BothSides,
    ] {
        let mut base = phrase_style(Some("ja"));
        base.native_hyphenation_convention = Some(
            NativeHyphenationConvention::language_script("ja", "Latn", Text::from("A"), placement)
                .unwrap(),
        );
        base.hyphenate_limit_chars = HyphenateLimitChars {
            total: HyphenationMinimum::Count(0),
            before: HyphenationMinimum::Count(0),
            after: HyphenationMinimum::Count(0),
        };
        let mut value = input("AAAAA", base.clone(), DIRECTIONS[0]);
        value.controls.push(ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(base),
            },
        });
        value.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        });
        let p = prepare(value);
        assert_eq!(p.analyzed_source().style().locale.as_deref(), Some("ja"));
        assert_eq!(p.analyzed_source().source().text().code_units(), [65; 5]);
        let phrase = p.phrases().next().unwrap();
        assert_eq!(p.phrases().count(), 1);
        assert_eq!(
            (
                phrase.presentation_range().start(),
                phrase.presentation_range().end()
            ),
            (0, 5)
        );
        let counts = p
            .hyphenation_cut_views()
            .find(|cut| cut.provenance() == HyphenationProvenance::Control)
            .unwrap()
            .counts();
        assert_eq!(
            counts,
            Some(HyphenationCounts {
                total: 5,
                before: 1,
                after: 4
            })
        );
        assert!(ordinary(&p).is_empty());
        assert!(discretionary(&p).is_empty());
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = first_after(&p, &start);
        let frame = ordinary_trial(&p, start.clone(), None, 20.0);
        assert_eq!(frame.feedback().advance, 50.0);
        let search = p
            .conditional_search(
                &start,
                None,
                &ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&frame),
            )
            .unwrap();
        let portions = search.phrase_portions();
        assert_eq!(portions.len(), 1);
        assert_eq!(portions[0].phrase(), phrase);
        let measured = phrase_trial(&p, &portions[0], 20.0);
        assert_eq!(measured.feedback().advance, 50.0);
        let normal = search
            .begin(&[PhraseOverflowDecision::Overflows(measured.clone())])
            .unwrap();
        assert!(normal.opportunities().is_empty());
        let hyphens = normal.advance(&[]).unwrap().unwrap();
        assert_eq!(hyphens.opportunities().len(), 1);
        let cut = &hyphens.opportunities()[0];
        let first = conditional_trial(&p, cut, None, Some(20.0));
        assert_eq!(
            first.feedback().advance,
            if placement == HyphenationPlacement::LogicalAfter {
                10.0
            } else {
                20.0
            }
        );
        let incoming = first.hyphenation_continuation().unwrap();
        assert_eq!(incoming.alternative(), cut.alternative().unwrap());
        assert_eq!(incoming.boundary(), cut.boundary());
        let remaining_start = cut.boundary();
        let remaining_ceiling = first_after(&p, &remaining_start);
        let second = ordinary_trial(&p, remaining_start.clone(), Some(incoming.clone()), 50.0);
        assert_eq!(second.feedback().advance, 50.0);
        let remaining = p
            .conditional_search(
                &remaining_start,
                Some(&incoming),
                &remaining_ceiling,
                ConditionalPurpose::Overflow,
                &[],
                Some(&second),
            )
            .unwrap();
        let portions = remaining.phrase_portions();
        assert_eq!(portions.len(), 1);
        let portion = &portions[0];
        assert_eq!(portion.phrase(), phrase);
        assert_eq!(portion.start(), remaining_start);
        assert_eq!(portion.start().view().source_after().get(), 1);
        assert_eq!(portion.end().view().source_after().get(), 5);
        assert_eq!(portion.incoming(), Some(incoming));
        assert_eq!(
            (
                portion.phrase().presentation_range().start(),
                portion.phrase().presentation_range().end()
            ),
            (0, 5)
        );
        let missing = interval_request(
            &p,
            portion.start(),
            portion.end(),
            PreparedTermination::PhraseMeasurement(portion.clone()),
            None,
            Some(50.0),
        );
        assert!(p.select(missing).is_err());
        assert!(
            remaining
                .begin(&[PhraseOverflowDecision::Overflows(measured)])
                .is_err(),
            "the previous full-phrase trial has no receiving authority"
        );
        let measured = phrase_trial(&p, portion, 50.0);
        assert_eq!(
            measured.feedback().advance,
            50.0,
            "actual incoming A plus four unconsumed A units"
        );
        assert!(place(p.clone(), measured.clone(), 0.0, 8.0).is_err());
        assert!(
            remaining
                .begin(&[PhraseOverflowDecision::Fits(measured)])
                .unwrap()
                .opportunities()
                .is_empty()
        );
        assert!(second.hyphenation_continuation().is_none());
    }
}
