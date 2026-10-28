#![forbid(unsafe_code)]

//! Public Manual semantics from pinned CSS Text4 section6.1: disable lexical
//! line breaking while retaining Normal orthographic/explicit/overflow owners.
//! Thai/Lao line TCUs follow EGCs. Balinese expectations use current UAX14rev55
//! LB28a/LB31 and the retained Unicode17 grapheme companion, not provider output.
//! Ahem admits semantic preparation; no Thai/Lao/Balinese glyph coverage or
//! geometry is assumed. ASCII cache checks use genuine Ahem glyph metrics.

use icu_properties::{
    CodePointMapData,
    props::{GraphemeClusterBreak as Gcb, LineBreak as Class},
};
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const DIRECTIONS: [t::ParagraphBaseDirection; 2] = [
    t::ParagraphBaseDirection::LeftToRight,
    t::ParagraphBaseDirection::RightToLeft,
];
const STRICTNESSES: [t::LineBreak; 4] = [
    t::LineBreak::Auto,
    t::LineBreak::Loose,
    t::LineBreak::Normal,
    t::LineBreak::Strict,
];

fn style(word: t::WordBreak, locale: &str) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        word_break: word,
        locale: Some(locale.into()),
        overflow_wrap: t::OverflowWrap::Normal,
        hyphens: t::Hyphens::None,
        hyphenate_character: t::HyphenateCharacter::String(t::Text::from("X")),
        hyphenate_limit_chars: t::HyphenateLimitChars {
            total: t::HyphenationMinimum::Count(0),
            before: t::HyphenationMinimum::Count(0),
            after: t::HyphenationMinimum::Count(0),
        },
        ..t::Style::default()
    }
}

fn system() -> t::System {
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
}

fn input(value: &str, base: t::Style, direction: t::ParagraphBaseDirection) -> t::ParagraphInput {
    let source = t::Source::identified(value, t::Id::from_u64(65501), 31);
    let end = source.text().len();
    t::ParagraphInput {
        source: source.clone(),
        base_style: base,
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![t::ParagraphOrigin {
            source,
            local: t::Range::new(0, end),
            aggregate: t::Range::new(0, end),
            kind: t::ParagraphOriginKind::Authored,
            text_node: None,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare(request: t::ParagraphInput) -> t::PreparedText {
    let ancestor = request.base_style.wrap;
    let source = request.source.clone();
    let base = request.base_style.clone();
    let analyzed =
        t::AnalyzedSource::from_paragraph(request, t::AnalysisTypography::HorizontalComposition)
            .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: ancestor,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let p = system()
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(p.analyzed_source().source(), &source);
    assert_eq!(p.analyzed_source().style(), &base);
    p
}

fn cuts(p: &t::PreparedText) -> Vec<usize> {
    p.opportunity_views()
        .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
        .map(|offer| {
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
fn manual_disables_thai_lao_lexical_fallback_across_ordinary_strictnesses() {
    for letter in ['ก', 'ກ'] {
        assert_eq!(
            CodePointMapData::<Class>::new().get32(u32::from(letter)),
            Class::ComplexContext
        );
        assert_eq!(
            CodePointMapData::<Gcb>::new().get32(u32::from(letter)),
            Gcb::Other
        );
    }
    for direction in DIRECTIONS {
        for (value, locale) in [("กก", "th"), ("ກກ", "lo")] {
            for word in [
                t::WordBreak::Normal,
                t::WordBreak::KeepAll,
                t::WordBreak::Manual,
            ] {
                for line_break in STRICTNESSES {
                    let mut base = style(word, locale);
                    base.line_break = line_break;
                    let p = prepare(input(value, base, direction));
                    assert_eq!(
                        p.analyzed_source().graphemes().collect::<Vec<_>>(),
                        [t::Range::new(0, 1), t::Range::new(1, 2)]
                    );
                    assert_eq!(
                        cuts(&p),
                        if word == t::WordBreak::Manual {
                            vec![]
                        } else {
                            vec![1]
                        },
                        "{word:?} {line_break:?} {locale} {direction:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn manual_sa_as_al_does_not_keep_a_provider_run_end_before_latin_letters() {
    assert_eq!(
        CodePointMapData::<Class>::new().get32(0x0e01),
        Class::ComplexContext
    );
    assert_eq!(
        CodePointMapData::<Class>::new().get32(u32::from('A')),
        Class::Alphabetic
    );
    for direction in DIRECTIONS {
        for line_break in STRICTNESSES {
            let mut base = style(t::WordBreak::Manual, "th");
            base.line_break = line_break;
            let p = prepare(input("กกAA", base, direction));
            assert_eq!(
                p.analyzed_source().graphemes().collect::<Vec<_>>(),
                (0..4)
                    .map(|at| t::Range::new(at, at + 1))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                cuts(&p),
                [],
                "Manual treats the two SA units as AL; four adjacent alphabetic units have no ordinary interior cut: {line_break:?} {direction:?}"
            );
        }
        let normal = prepare(input("กกAA", style(t::WordBreak::Normal, "th"), direction));
        assert!(
            cuts(&normal).contains(&1),
            "Normal retains the required no-model Thai fallback"
        );
    }
}

#[test]
fn following_manual_owner_and_real_nca_keep_whole_thai_tone_units() {
    for direction in DIRECTIONS {
        for following_manual in [false, true] {
            for origins in [false, true] {
                let before = if following_manual {
                    t::WordBreak::Normal
                } else {
                    t::WordBreak::Manual
                };
                let after = if following_manual {
                    t::WordBreak::Manual
                } else {
                    t::WordBreak::Normal
                };
                let mut request = input("ก่ก", style(before, "th"), direction);
                request.source.span(t::Range::new(1, 3), style(after, "th"));
                if origins {
                    request.origins = [t::Range::new(0, 2), t::Range::new(2, 3)]
                        .into_iter()
                        .map(|range| t::ParagraphOrigin {
                            source: request.source.clone(),
                            local: range,
                            aggregate: range,
                            kind: t::ParagraphOriginKind::Authored,
                            text_node: None,
                        })
                        .collect();
                    request.connections.push(t::ParagraphConnection {
                        at: 2,
                        kind: t::ParagraphConnectionKind::Continuous,
                    });
                    request.boundary_policies.push(t::ParagraphBoundaryPolicy {
                        definition: t::ParagraphBoundaryDefinition::Connection(0),
                        preceding_wrap: t::Wrap::Word,
                        following_wrap: t::Wrap::Word,
                        ancestor_wrap: t::Wrap::Word,
                    });
                }
                let p = prepare(request);
                assert_eq!(
                    p.analyzed_source().graphemes().collect::<Vec<_>>(),
                    [t::Range::new(0, 2), t::Range::new(2, 3)]
                );
                assert_eq!(
                    cuts(&p),
                    if following_manual { vec![] } else { vec![2] },
                    "actual following logical owner, origins={origins} {direction:?}"
                );
            }
        }
        let mut block = style(t::WordBreak::Manual, "th");
        block.wrap = t::Wrap::None;
        let mut request = input("กก", block, direction);
        let mut anywhere = style(t::WordBreak::Manual, "th");
        anywhere.line_break = t::LineBreak::Anywhere;
        request.source.span(t::Range::new(0, 2), anywhere);
        assert_eq!(
            cuts(&prepare(request)),
            [],
            "actual NCA nowrap still gates the independent Anywhere cause"
        );
    }
}

#[test]
fn manual_preserves_authentic_wbr_zwsp_and_forced_owners() {
    for direction in DIRECTIONS {
        for ancestor in [t::Wrap::Word, t::Wrap::None] {
            for control_wrap in [t::Wrap::Word, t::Wrap::None] {
                let mut block = style(t::WordBreak::Manual, "th");
                block.wrap = ancestor;
                let mut owner = style(t::WordBreak::Manual, "th");
                owner.wrap = control_wrap;
                let mut request = input("กก", block, direction);
                request.controls.push(t::ParagraphControl {
                    at: 1,
                    kind: t::ParagraphControlKind::Wbr {
                        style: Box::new(owner),
                    },
                });
                request.anchor_order.push(t::ParagraphAnchor {
                    item: t::ParagraphAnchorItem::Control(0),
                    affinity: t::Affinity::After,
                });
                let p = prepare(request);
                let issued = p.control_views().next().unwrap().handle();
                assert!(
                    matches!(issued.view().kind(), t::ParagraphControlKind::Wbr { style } if style.word_break == t::WordBreak::Manual && style.wrap == control_wrap)
                );
                let index=p.participants().position(|part|matches!(part,t::PreparedParticipant::Control(ref actual) if *actual==issued)).unwrap();
                let actual = p
                    .boundary_views()
                    .find(|boundary| boundary.participant_offset() == index + 1)
                    .unwrap()
                    .handle();
                let offers = p
                    .opportunity_views()
                    .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
                    .collect::<Vec<_>>();
                assert_eq!(offers.len(), usize::from(control_wrap == t::Wrap::Word));
                for offer in offers {
                    assert_eq!(offer.boundary(), actual);
                    assert_eq!(offer.alternatives().count(), 0);
                }
                assert_eq!(actual.view().source_after().get(), 1);
                assert_eq!(p.virtual_separators().count(), 0);
            }
            let mut base = style(t::WordBreak::Manual, "th");
            base.wrap = ancestor;
            let p = prepare(input("ก\u{200b}ก", base.clone(), direction));
            assert_eq!(
                cuts(&p),
                if ancestor == t::Wrap::Word {
                    vec![2]
                } else {
                    vec![]
                },
                "source ZWSP retains its actual containing wrap"
            );
            let p = prepare(input("ก\nก", base, direction));
            assert_eq!(cuts(&p), []);
            assert_eq!(
                p.opportunity_views()
                    .filter(|offer| offer.kind() == t::PreparedBreakKind::Forced)
                    .map(|offer| offer.boundary().view().source_after().get())
                    .collect::<Vec<_>>(),
                [2]
            );
        }
    }
}

#[test]
fn manual_retains_independent_anywhere_and_required_cjk_opportunities() {
    for direction in DIRECTIONS {
        for (value, locale, line_break, expected) in [
            ("กก", "th", t::LineBreak::Anywhere, vec![1]),
            ("ก่ก", "th", t::LineBreak::Anywhere, vec![2]),
            ("漢漢", "ja", t::LineBreak::Auto, vec![1]),
            ("漢ぁ", "ja", t::LineBreak::Loose, vec![1]),
            ("漢ぁ", "ja", t::LineBreak::Strict, vec![]),
        ] {
            let mut base = style(t::WordBreak::Manual, locale);
            base.line_break = line_break;
            assert_eq!(
                cuts(&prepare(input(value, base, direction))),
                expected,
                "{value} {line_break:?} {direction:?}"
            );
        }
    }
}

#[test]
fn manual_retains_declared_overflow_conditional_eligibility_and_readback() {
    for direction in DIRECTIONS {
        for overflow in [
            t::OverflowWrap::Normal,
            t::OverflowWrap::Anywhere,
            t::OverflowWrap::BreakWord,
        ] {
            let mut base = style(t::WordBreak::Manual, "en-US");
            base.overflow_wrap = overflow;
            assert_eq!(
                t::ValidatedStyle::try_from(base.clone())
                    .unwrap()
                    .authored(),
                &base
            );
            let mut request = input("AAAA", base.clone(), direction);
            request.source.span(t::Range::new(0, 4), base.clone());
            let p = prepare(request);
            assert_eq!(p.analyzed_source().style(), &base);
            assert_eq!(p.analyzed_source().source().spans()[0].style(), &base);
            assert_eq!(
                p.opportunity_views()
                    .map(|offer| offer.kind())
                    .collect::<Vec<_>>(),
                [t::PreparedBreakKind::ParagraphEnd]
            );
            let start = p.boundary_views().next().unwrap().handle();
            let ceiling = p.opportunity_views().next().unwrap().handle();
            for purpose in [
                t::ConditionalPurpose::Overflow,
                t::ConditionalPurpose::MinContent,
            ] {
                let admitted = overflow == t::OverflowWrap::Anywhere
                    || (overflow == t::OverflowWrap::BreakWord
                        && purpose == t::ConditionalPurpose::Overflow);
                // The only ordinary endpoint is the ceiling; no earlier variant
                // needs a disposition. Enumeration asserts eligibility, not fit.
                let actual = p
                    .conditional_search(&start, None, &ceiling, purpose, &[], None)
                    .and_then(|search| search.begin(&[]))
                    .map(|batch| batch.opportunities().to_vec())
                    .unwrap()
                    .iter()
                    .map(|offer| offer.boundary().view().source_after().get())
                    .collect::<Vec<_>>();
                assert_eq!(
                    actual,
                    if admitted { vec![1, 2, 3] } else { vec![] },
                    "Manual retains declared {overflow:?} for {purpose:?}"
                );
            }
        }
    }
}

#[test]
fn manual_keeps_normal_owned_source_shy_and_qualified_automatic_hyphenation() {
    for direction in DIRECTIONS {
        for word in [t::WordBreak::Normal, t::WordBreak::Manual] {
            let mut base = style(word, "en-US");
            base.hyphens = t::Hyphens::Manual;
            let p = prepare(input("a\u{ad}b", base, direction));
            let alternatives = p.alternative_views().collect::<Vec<_>>();
            assert_eq!(alternatives.len(), 1);
            assert_eq!(alternatives[0].source_range(), Some(t::Range::new(1, 2)));
            assert_eq!(alternatives[0].text(), &t::Text::from("X"));
            assert_eq!(
                alternatives[0].hyphenation_cut().view().style().word_break,
                word
            );
            let mut base = style(word, "en-US");
            base.hyphens = t::Hyphens::Auto;
            let p = prepare(input("associate", base, direction));
            assert_eq!(p.word_views().next().unwrap().query(), Some("associate"));
            assert_eq!(
                p.alternative_views()
                    .filter(|alternative| matches!(
                        alternative.definition(),
                        t::PreparedAlternativeDefinition::Automatic { .. }
                    ))
                    .map(|alternative| alternative.insertion_anchor().get())
                    .collect::<Vec<_>>(),
                [2, 4],
                "independent retained as-so-ciate exception remains eligible under {word:?}"
            );
        }
    }
}

#[test]
fn manual_preserves_balinese_orthographic_rules_and_whole_conjuncts() {
    // UAX14rev55 LB28a forbids AK×VI and VI-conjunct interiors; with no
    // conflicting rule, LB31 permits the separate AK/AK syllable boundary.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB28a
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB31
    for cp in [0x1b05, 0x1b26, 0x1b17, 0x1b13] {
        assert_eq!(CodePointMapData::<Class>::new().get32(cp), Class::Aksara);
        assert_eq!(CodePointMapData::<Gcb>::new().get32(cp), Gcb::Other);
    }
    assert_eq!(
        CodePointMapData::<Class>::new().get32(0x1b44),
        Class::Virama
    );
    assert_eq!(CodePointMapData::<Gcb>::new().get32(0x1b44), Gcb::Extend);
    for direction in DIRECTIONS {
        for word in [t::WordBreak::Normal, t::WordBreak::Manual] {
            for (value, expected, graphemes) in [
                (
                    "\u{1b05}\u{1b05}",
                    vec![1],
                    vec![t::Range::new(0, 1), t::Range::new(1, 2)],
                ),
                ("\u{1b05}\u{1b44}", vec![], vec![t::Range::new(0, 2)]),
            ] {
                let p = prepare(input(value, style(word, "ban"), direction));
                assert_eq!(
                    p.analyzed_source().graphemes().collect::<Vec<_>>(),
                    graphemes
                );
                assert_eq!(
                    cuts(&p),
                    expected,
                    "Balinese orthographic {value:?} {word:?} {direction:?}"
                );
            }
            let p = prepare(input(
                "\u{1b26}\u{1b17}\u{1b44}\u{1b13}",
                style(word, "ban"),
                direction,
            ));
            assert_eq!(
                p.analyzed_source().graphemes().collect::<Vec<_>>(),
                [t::Range::new(0, 1), t::Range::new(1, 4)],
                "retained Unicode17 GB9/GB9c sample"
            );
            assert!(
                cuts(&p).iter().all(|cut| *cut == 1),
                "no claim that every EGC edge is a syllable break; conjunct interiors2/3 are forbidden"
            );
        }
    }
}

#[test]
fn declared_manual_and_overflow_values_have_real_separate_cache_entries() {
    let mut system = system();
    let source = t::Source::identified("AA", t::Id::from_u64(65502), 37);
    let mut previous = Vec::new();
    for word in [t::WordBreak::Normal, t::WordBreak::Manual] {
        for overflow in [
            t::OverflowWrap::Normal,
            t::OverflowWrap::Anywhere,
            t::OverflowWrap::BreakWord,
        ] {
            let mut base = style(word, "en-US");
            base.overflow_wrap = overflow;
            let options = t::Options {
                quantize: false,
                ..t::Options::default()
            };
            let before = system.stats();
            let layout = system
                .layout(source.clone(), base.clone(), options)
                .unwrap();
            assert_eq!(layout.analyzed_source().style(), &base);
            assert_eq!(layout.source(), &source);
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [t::Range::new(0, 2)]
            );
            assert_eq!(layout.metrics().full_width(), 20.0);
            assert_eq!(system.stats().layout_misses(), before.layout_misses() + 1);
            assert!(
                previous.iter().all(|(key, _, _)| *key != layout.key()),
                "declared values remain in cache identity even when Latin behavior agrees"
            );
            let hit = system
                .layout(source.clone(), base.clone(), options)
                .unwrap();
            assert_eq!(hit.key(), layout.key());
            assert_eq!(system.stats().layout_hits(), before.layout_hits() + 1);
            previous.push((layout.key(), layout, base));
        }
    }
    for (key, layout, base) in previous {
        assert_eq!(layout.key(), key);
        assert_eq!(layout.analyzed_source().style(), &base);
        assert_eq!(layout.metrics().full_width(), 20.0);
    }
}
