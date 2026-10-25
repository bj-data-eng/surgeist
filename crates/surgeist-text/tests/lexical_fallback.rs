#![forbid(unsafe_code)]

//! Missing Thai/Lao lexical capability requires ordinary TLU-pair fallback.
//! Independent authority: pinned CSS Text4 sections1.4,6.1.1.1,6.1.1.3 and
//! AppendixH; KeepAll/locale composition follows the adopted determination in
//! references/text-lexical-fallback--determination-20261025.md.
//!
//! The current non-complex ICU constructor has no Thai/Lao model. This is the
//! unavailable-capability case, not a qualified analyzer returning no words.
//! Ahem admits preparation but has no Thai/Lao coverage: no glyph/width claim.

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory as Category, GraphemeClusterBreak as Gcb, LineBreak as Class, Script},
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

fn style(locale: Option<&str>) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Preserve,
        word_break: t::WordBreak::Normal,
        line_break: t::LineBreak::Auto,
        overflow_wrap: t::OverflowWrap::Normal,
        wrap: t::Wrap::Word,
        hyphens: t::Hyphens::None,
        locale: locale.map(str::to_owned),
        ..t::Style::default()
    }
}

fn input(value: &str, base: t::Style, direction: t::ParagraphBaseDirection) -> t::ParagraphInput {
    let source = t::Source::identified(value, t::Id::from_u64(65401), 19);
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
    // Each fixture explicitly describes one real containing block with this Wrap.
    let ancestor = request.base_style.wrap;
    let source = request.source.clone();
    let base = request.base_style.clone();
    let analyzed = t::AnalyzedSource::from_paragraph(request).unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: ancestor,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.analyzed_source().style(), &base);
    assert_eq!(
        prepared.analyzed_source().processed_source().source(),
        &source
    );
    prepared
}

fn allowed(p: &t::PreparedText) -> Vec<t::PreparedOpportunity> {
    p.opportunity_views()
        .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
        .map(|offer| offer.handle())
        .collect()
}

fn cuts(p: &t::PreparedText) -> Vec<usize> {
    allowed(p)
        .iter()
        .map(|offer| {
            assert_eq!(offer.view().alternatives().count(), 0);
            let boundary = offer.view().boundary();
            assert_eq!(
                boundary.view().source_before(),
                boundary.view().source_after()
            );
            boundary.view().source_after().get()
        })
        .collect()
}

fn connect(request: &mut t::ParagraphInput, kind: t::ParagraphConnectionKind) {
    request.origins = [t::Range::new(0, 1), t::Range::new(1, 2)]
        .into_iter()
        .map(|range| t::ParagraphOrigin {
            source: request.source.clone(),
            local: range,
            aggregate: range,
            kind: t::ParagraphOriginKind::Authored,
            text_node: None,
        })
        .collect();
    request
        .connections
        .push(t::ParagraphConnection { at: 1, kind });
    request.boundary_policies.push(t::ParagraphBoundaryPolicy {
        definition: t::ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: t::Wrap::Word,
        following_wrap: t::Wrap::Word,
        ancestor_wrap: request.base_style.wrap,
    });
}

#[test]
fn thai_lao_fixtures_are_sa_letter_units_with_independently_qualified_graphemes() {
    for (letter, script) in [('ก', Script::Thai), ('ກ', Script::Lao)] {
        let cp = u32::from(letter);
        assert_eq!(
            CodePointMapData::<Category>::new().get32(cp),
            Category::OtherLetter
        );
        assert_eq!(
            CodePointMapData::<Class>::new().get32(cp),
            Class::ComplexContext
        );
        assert_eq!(CodePointMapData::<Gcb>::new().get32(cp), Gcb::Other);
        assert_eq!(CodePointMapData::<Script>::new().get32(cp), script);
    }
    assert_eq!(
        CodePointMapData::<Category>::new().get32(0x0e48),
        Category::NonspacingMark
    );
    assert_eq!(CodePointMapData::<Gcb>::new().get32(0x0e48), Gcb::Extend);
    for (value, locale) in [("กก", "th"), ("ກກ", "lo")] {
        let p = prepare(input(value, style(Some(locale)), DIRECTIONS[0]));
        assert_eq!(
            p.analyzed_source().graphemes().collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)]
        );
        assert_eq!(
            p.unit_views()
                .map(|unit| unit.source_range())
                .collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)]
        );
    }
}

#[test]
fn missing_thai_lexical_capability_normal_publishes_ordinary_letter_pair_cut() {
    let mut observations = Vec::new();
    for direction in DIRECTIONS {
        let p = prepare(input("กก", style(Some("th")), direction));
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            &[0x0e01, 0x0e01]
        );
        assert_eq!(
            p.analyzed_source().graphemes().collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)]
        );
        let participants = p.participants().collect::<Vec<_>>();
        for offer in allowed(&p) {
            let boundary = offer.view().boundary();
            let offset = boundary.view().participant_offset();
            assert!(matches!(
                participants[offset - 1],
                t::PreparedParticipant::Text(_)
            ));
            assert!(matches!(
                participants[offset],
                t::PreparedParticipant::Text(_)
            ));
            assert_eq!(boundary.view().source_before().get(), 1);
            assert_eq!(boundary.view().source_after().get(), 1);
        }
        // OverflowWrap Normal has no conditional capability to disguise the missing ordinary cut.
        let start = p.boundary_views().next().unwrap().handle();
        let ceiling = p.opportunity_views().next().unwrap().handle();
        assert!(
            p.conditional_search(
                &start,
                None,
                &ceiling,
                t::ConditionalPurpose::Overflow,
                &[],
                None
            )
            .and_then(|search| search.begin(&[]))
            .map(|batch| batch.opportunities().to_vec())
            .unwrap()
            .is_empty()
        );
        observations.push((direction, cuts(&p)));
    }
    assert_eq!(
        observations,
        [(DIRECTIONS[0], vec![1]), (DIRECTIONS[1], vec![1])],
        "CSS fallback is ordinary permission between the two whole Thai letter units; current no-model output is not a supported lexical result"
    );
}

#[test]
fn qualified_thai_lao_fallback_retains_modes_strictness_and_declared_locale() {
    let mut missing = Vec::new();
    for direction in DIRECTIONS {
        for value in ["กก", "ກກ"] {
            for locale in [Some("th"), Some("lo"), None, Some("und"), Some("en")] {
                for word in [
                    t::WordBreak::Normal,
                    t::WordBreak::KeepAll,
                    t::WordBreak::BreakWord,
                    t::WordBreak::BreakAll,
                ] {
                    for strictness in STRICTNESSES {
                        let mut base = style(locale);
                        base.word_break = word;
                        base.line_break = strictness;
                        let p = prepare(input(value, base, direction));
                        assert_eq!(p.analyzed_source().style().locale.as_deref(), locale);
                        if cuts(&p) != [1] {
                            missing.push((direction, value, locale, word, strictness, cuts(&p)));
                        }
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "required unavailable-capability fallback (BreakAll is an independent effective-ID control): {missing:?}"
    );
}

#[test]
fn thai_tone_mark_stays_in_its_whole_letter_unit_before_fallback() {
    let mut missing = Vec::new();
    for direction in DIRECTIONS {
        for word in [
            t::WordBreak::Normal,
            t::WordBreak::KeepAll,
            t::WordBreak::BreakWord,
            t::WordBreak::BreakAll,
        ] {
            for strictness in STRICTNESSES {
                let mut base = style(Some("th"));
                base.word_break = word;
                base.line_break = strictness;
                let p = prepare(input("ก่ก", base, direction));
                assert_eq!(
                    p.analyzed_source().graphemes().collect::<Vec<_>>(),
                    [t::Range::new(0, 2), t::Range::new(2, 3)]
                );
                assert!(
                    !cuts(&p).contains(&1),
                    "tone mark must not be detached from its base"
                );
                if cuts(&p) != [2] {
                    missing.push((direction, word, strictness, cuts(&p)));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "ordinary fallback follows the complete Thai base+tone TCU: {missing:?}"
    );
}

#[test]
fn empty_provider_output_does_not_authorize_latin_formatting_punctuation_or_glue_fallback() {
    for direction in DIRECTIONS {
        for value in [
            "AA",
            "ก\u{200e}ก",
            "\u{200e}\u{200e}",
            "ก(ก",
            "ก)ก",
            "ก\u{a0}ก",
            "ก\u{2060}ก",
            "ก\u{200d}ก",
        ] {
            let p = prepare(input(value, style(Some("th")), direction));
            assert_eq!(
                cuts(&p),
                [],
                "no new TLU-pair permission around a nonletter or protected unit: {value:?}"
            );
        }
        let p = prepare(input("ก\u{1f600}ก", style(Some("th")), direction));
        assert!(
            !cuts(&p).contains(&2),
            "no cut inside a supplementary surrogate pair"
        );
    }
}

#[test]
fn continuous_source_and_paint_style_seams_preserve_required_fallback() {
    let mut missing = Vec::new();
    for direction in DIRECTIONS {
        for origins in [false, true] {
            let mut request = input("กก", style(Some("th")), direction);
            let mut following = style(Some("und"));
            following.brush = t::Brush::color(1.0, 0.0, 0.0, 1.0);
            request.source.span(t::Range::new(1, 2), following);
            if origins {
                connect(&mut request, t::ParagraphConnectionKind::Continuous);
            }
            let p = prepare(request);
            if cuts(&p) != [1] {
                missing.push((direction, origins, cuts(&p)));
            }
            if origins {
                assert_eq!(
                    p.origin_views()
                        .map(|origin| origin.aggregate_range())
                        .collect::<Vec<_>>(),
                    [t::Range::new(0, 1), t::Range::new(1, 2)]
                );
            }
        }
    }
    assert!(
        missing.is_empty(),
        "continuous style/origin edges do not terminate lexical fallback context: {missing:?}"
    );
}

#[test]
fn fallback_cannot_cross_nowrap_disconnected_or_real_hard_boundaries() {
    for direction in DIRECTIONS {
        let mut base = style(Some("th"));
        base.wrap = t::Wrap::None;
        let mut request = input("กก", base, direction);
        request.source.span(t::Range::new(0, 2), style(Some("th")));
        assert_eq!(
            cuts(&prepare(request)),
            [],
            "the real nearest common ancestor forbids soft wrapping"
        );
        let mut request = input("กก", style(Some("th")), direction);
        connect(&mut request, t::ParagraphConnectionKind::Disconnected);
        assert_eq!(
            cuts(&prepare(request)),
            [],
            "two disconnected letter units do not form a fallback pair"
        );
        for (value, delimiter, at) in [("ก\nก", None, 2), ("ก\r\nก", Some(t::Range::new(1, 3)), 3)]
        {
            let mut request = input(value, style(Some("th")), direction);
            if let Some(range) = delimiter {
                request.segment_breaks = t::DocumentSegmentBreaks::Defined(vec![range]);
            }
            let p = prepare(request);
            assert_eq!(cuts(&p), []);
            assert_eq!(
                p.opportunity_views()
                    .filter(|offer| offer.kind() == t::PreparedBreakKind::Forced)
                    .map(|offer| offer.boundary().view().source_after().get())
                    .collect::<Vec<_>>(),
                [at]
            );
        }
    }
}

#[test]
fn explicit_wbr_zwsp_and_in_flow_atomic_offers_keep_their_authentic_owners() {
    for direction in DIRECTIONS {
        let mut request = input("กก", style(Some("th")), direction);
        request.controls.push(t::ParagraphControl {
            at: 1,
            kind: t::ParagraphControlKind::Wbr {
                style: Box::new(style(Some("th"))),
            },
        });
        request.anchor_order.push(t::ParagraphAnchor {
            item: t::ParagraphAnchorItem::Control(0),
            affinity: t::Affinity::After,
        });
        let p = prepare(request);
        let control = p.control_views().next().unwrap().handle();
        let offset = p.participants().position(|part| matches!(part, t::PreparedParticipant::Control(ref actual) if *actual == control)).unwrap();
        let after = p
            .boundary_views()
            .find(|boundary| boundary.participant_offset() == offset + 1)
            .unwrap()
            .handle();
        assert!(
            allowed(&p)
                .iter()
                .any(|offer| offer.view().boundary() == after)
        );
        assert_eq!(after.view().source_after().get(), 1);
        assert_eq!(
            cuts(&prepare(input("ก\u{200b}ก", style(Some("th")), direction))),
            [2]
        );
        for ancestor in [t::Wrap::Word, t::Wrap::None] {
            let mut block = style(Some("th"));
            block.wrap = ancestor;
            let mut request = input("กก", block, direction);
            request.source.span(t::Range::new(0, 2), style(Some("th")));
            request.source.inline_box(t::InlineBox::new(
                t::Id::from_u64(65402),
                t::InlineBoxKind::InFlow,
                1,
                t::Size::new(10.0, 10.0),
            ));
            request.anchor_order.push(t::ParagraphAnchor {
                item: t::ParagraphAnchorItem::Atomic(0),
                affinity: t::Affinity::After,
            });
            for definition in [
                t::ParagraphBoundaryDefinition::AtomicBefore(0),
                t::ParagraphBoundaryDefinition::AtomicAfter(0),
            ] {
                request.boundary_policies.push(t::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: t::Wrap::Word,
                    following_wrap: t::Wrap::Word,
                    ancestor_wrap: ancestor,
                });
            }
            let p = prepare(request);
            let atomic = p.atomic_views().next().unwrap().handle();
            let index = p.participants().position(|part| matches!(part, t::PreparedParticipant::Atomic(ref actual) if *actual == atomic)).unwrap();
            let issued = allowed(&p)
                .iter()
                .map(|offer| offer.view().boundary())
                .collect::<Vec<_>>();
            for at in [index, index + 1] {
                let actual = p
                    .boundary_views()
                    .find(|boundary| boundary.participant_offset() == at)
                    .unwrap()
                    .handle();
                assert_eq!(
                    issued.contains(&actual),
                    ancestor == t::Wrap::Word,
                    "only atomic compatibility plus its real NCA gate supplies this exact cut"
                );
            }
        }
    }
}

#[test]
fn transparent_out_of_flow_occurrence_does_not_terminate_fallback_context() {
    let mut missing = Vec::new();
    for direction in DIRECTIONS {
        let mut request = input("กก", style(Some("th")), direction);
        request.source.inline_box(t::InlineBox::new(
            t::Id::from_u64(65403),
            t::InlineBoxKind::OutOfFlow,
            1,
            t::Size::new(10.0, 10.0),
        ));
        request.anchor_order.push(t::ParagraphAnchor {
            item: t::ParagraphAnchorItem::Atomic(0),
            affinity: t::Affinity::After,
        });
        let p = prepare(request);
        let atomic = p.atomic_views().next().unwrap().handle();
        let index = p.participants().position(|part| matches!(part, t::PreparedParticipant::Atomic(ref actual) if *actual == atomic)).unwrap();
        let offers = allowed(&p);
        if offers.is_empty() {
            missing.push(direction);
        }
        for offer in offers {
            let actual = offer.view().boundary();
            assert_eq!(actual.view().source_before().get(), 1);
            assert_eq!(actual.view().source_after().get(), 1);
            assert!(
                [index, index + 1].contains(&actual.view().participant_offset()),
                "retained out-of-flow occurrence has exact mixed-cut custody"
            );
        }
    }
    assert!(
        missing.is_empty(),
        "transparent out-of-flow metadata cannot hide the required text/text fallback: {missing:?}"
    );
}

#[test]
fn assumed_line_opportunities_do_not_become_detected_word_movement_stops() {
    // The adopted missing-capability policy explicitly keeps assumed line
    // opportunities out of editing/linguistic word facts. With no lexical
    // model and no separator, this run has no detected interior word stop.
    // This does not claim that these letters form one linguistically known
    // Thai word; it qualifies only the no-model operation's truthful boundary.
    // Three letters also detects a fabricated backward stop after traversal
    // skips the row adjacent to its starting cursor.
    for direction in DIRECTIONS {
        let p = prepare(input("กกก", style(Some("th")), direction));
        let start = p.boundary_views().next().unwrap().handle();
        let end = p
            .opportunity_views()
            .find(|offer| offer.kind() == t::PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let line = p
            .select(t::LineSelectionRequest {
                start,
                end: end.view().boundary(),
                participants: p.participants().collect(),
                termination: t::PreparedTermination::Ordinary(end),
                alternative: None,
                incoming: None,
                atomics: vec![],
                controls: vec![],
                boundary_context: vec![],
                line_box_struts: Vec::new(),
                line_strut: None,
                context: t::LineContext {
                    formatted_line: surgeist_text::FormattedLineContext::default(),
                    base_direction: direction,
                    line_box_direction: match direction {
                        surgeist_text::ParagraphBaseDirection::LeftToRight => {
                            surgeist_text::LineBoxDirection::LeftToRight
                        }
                        surgeist_text::ParagraphBaseDirection::RightToLeft => {
                            surgeist_text::LineBoxDirection::RightToLeft
                        }
                    },
                    orientation: t::LineOrientation::Horizontal,
                    start: t::LineStart::Paragraph,
                    end: t::LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: t::Alignment::Start,
                    tabs: t::TabContext::NoTabs,
                    spacing: t::SpacingPolicy::None,
                },
            })
            .unwrap()
            .finalize(t::SpacingAllocations::None)
            .unwrap();
        let layout = t::Layout::from_finalized_lines(t::PlacementRequest {
            prepared: p,
            coverage: t::PlacementCoverage::WholeParagraph,
            lines: vec![t::LinePlacement {
                line,
                anchor: t::LineAnchor {
                    inline_origin: 0.0,
                    baseline: 0.0,
                },
                orientation: t::LineOrientation::Horizontal,
                atomics: vec![],
            }],
        })
        .unwrap();
        assert_eq!(
            layout.source().text().code_units(),
            &[0x0e01, 0x0e01, 0x0e01]
        );
        for affinity in [t::Affinity::Before, t::Affinity::After] {
            assert_eq!(
                layout
                    .move_cursor(t::Cursor::new(0, affinity), t::Movement::NextWord)
                    .unwrap()
                    .index(),
                3,
                "assumed Thai line cuts do not assert detected word boundaries: {direction:?} {affinity:?}"
            );
            assert_eq!(
                layout
                    .move_cursor(t::Cursor::new(3, affinity), t::Movement::PreviousWord)
                    .unwrap()
                    .index(),
                0,
                "word traversal cannot use fallback as lexical evidence: {direction:?} {affinity:?}"
            );
        }
    }
}

#[test]
fn standalone_consumes_missing_lexical_fallback_as_ordinary_with_real_notdef_metrics() {
    // The font's unscaled hmtx gives an em-sized .notdef; exact source ranges
    // and actual shaped advances qualify the consuming line decisions below.
    use skrifa::{
        GlyphId, MetadataProvider,
        instance::{LocationRef, Size},
    };
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.charmap().map('ก'), None);
    assert_eq!(font.charmap().map('ກ'), None);
    assert_eq!(
        font.metrics(Size::unscaled(), LocationRef::default())
            .units_per_em,
        1000
    );
    assert_eq!(
        font.glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(0)),
        Some(1000.0)
    );
    // These are authentic missing-glyph advances, not Thai/Lao glyph coverage.
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    for (value, locale, word) in [
        ("กก", "th", t::WordBreak::Normal),
        ("ກກ", "lo", t::WordBreak::Normal),
        ("กก", "th", t::WordBreak::KeepAll),
        ("ກກ", "lo", t::WordBreak::KeepAll),
    ] {
        let source = t::Source::identified(value, t::Id::from_u64(65404), 23);
        let mut base = style(Some(locale));
        base.word_break = word;
        assert_eq!(base.overflow_wrap, t::OverflowWrap::Normal);
        let layout = system
            .layout(
                source.clone(),
                base,
                t::Options {
                    width: Some(10.0),
                    quantize: false,
                    ..t::Options::default()
                },
            )
            .unwrap();
        assert_eq!(layout.source(), &source);
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)],
            "ordinary missing-lexical fallback is consumed under OverflowNormal: {locale} {word:?}"
        );
        let mut glyphs = Vec::new();
        for run in layout.glyph_runs() {
            assert_eq!(run.font().bytes(), AHEM);
            for glyph in run.glyphs() {
                glyphs.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
        glyphs.sort_by_key(|glyph| glyph.0.start);
        assert_eq!(
            glyphs,
            [
                (t::Range::new(0, 1), 0, 10.0),
                (t::Range::new(1, 2), 0, 10.0)
            ]
        );
        assert_eq!(
            layout
                .move_cursor(t::Cursor::new(0, t::Affinity::After), t::Movement::NextWord)
                .unwrap()
                .index(),
            2,
            "soft line placement does not invent a detected word boundary"
        );
    }
    // A later fitting emergency candidate must not displace an earlier fitting
    // ordinary fallback. The enclosing paragraph remains OverflowWrap::Normal;
    // only the last Thai unit independently authorizes raw emergency cuts.
    // The middle Thai+ZWJ TCU has no ordinary cut at its protected trailing edge.
    let mut source = t::Source::identified("กก\u{200d}ก", t::Id::from_u64(65405), 29);
    let mut emergency_tail = style(Some("th"));
    emergency_tail.overflow_wrap = t::OverflowWrap::Anywhere;
    source.span(t::Range::new(3, 4), emergency_tail);
    let mut native = input("กก\u{200d}ก", style(Some("th")), DIRECTIONS[0]);
    native.source = source.clone();
    native.origins[0].source = source.clone();
    let qualified = prepare(native);
    assert_eq!(
        qualified.analyzed_source().graphemes().collect::<Vec<_>>(),
        [
            t::Range::new(0, 1),
            t::Range::new(1, 3),
            t::Range::new(3, 4)
        ]
    );
    assert_eq!(
        cuts(&qualified),
        [1],
        "the complete Thai+ZWJ TCU leaves only the lexical fallback as ordinary permission"
    );
    let raw_analysis = t::AnalyzedSource::new(source.clone(), style(Some("th"))).unwrap();
    let raw = system
        .prepare(&raw_analysis, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(
        cuts(&raw),
        [1, 3],
        "the distinct raw inventory actually offers the later emergency candidate"
    );
    let layout = system
        .layout(
            source.clone(),
            style(Some("th")),
            t::Options {
                width: Some(20.0),
                quantize: false,
                ..t::Options::default()
            },
        )
        .unwrap();
    assert_eq!(layout.source(), &source);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [t::Range::new(0, 1), t::Range::new(1, 4)],
        "ordinary fallback at1 outranks the later fitting raw emergency cut at3"
    );
    let mut positive_advances = Vec::new();
    for run in layout.glyph_runs() {
        assert_eq!(run.font().bytes(), AHEM);
        for glyph in run.glyphs() {
            if glyph.advance() != 0.0 {
                assert_eq!(glyph.id(), 0);
                positive_advances.push(glyph.advance());
            }
        }
    }
    // Do not prescribe a glyph count for the formatting ZWJ itself.
    assert_eq!(positive_advances, [10.0, 10.0, 10.0]);
}

#[test]
fn uniform_sa_run_end_cannot_offer_a_break_immediately_before_word_joiner() {
    // Unicode17 UAX14 revision55 LB11 prohibits WJ adjacency after earlier
    // rules have been considered. No earlier LB8 cause exists in this source.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB11
    assert_eq!(
        CodePointMapData::<Class>::new().get32(0x2060),
        Class::WordJoiner
    );
    let mut observed = Vec::new();
    for direction in DIRECTIONS {
        let p = prepare(input("กก\u{2060}", style(Some("th")), direction));
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            &[0x0e01, 0x0e01, 0x2060]
        );
        assert_eq!(
            p.analyzed_source().graphemes().collect::<Vec<_>>(),
            [
                t::Range::new(0, 1),
                t::Range::new(1, 2),
                t::Range::new(2, 3)
            ]
        );
        for offer in allowed(&p) {
            let boundary = offer.view().boundary();
            let at = boundary.view().participant_offset();
            let participants = p.participants().collect::<Vec<_>>();
            assert!(matches!(
                participants[at - 1],
                t::PreparedParticipant::Text(_)
            ));
            assert!(matches!(participants[at], t::PreparedParticipant::Text(_)));
        }
        observed.push((direction, cuts(&p)));
    }
    assert_eq!(
        observed,
        [(DIRECTIONS[0], vec![1]), (DIRECTIONS[1], vec![1])],
        "retain genuine fallback1; the provider SA-run end is not permission for forbidden before-WJ cut2"
    );
}

#[test]
fn before_wj_prohibition_does_not_depend_on_a_latin_style_or_overflow_seam() {
    let mut violations = Vec::new();
    for direction in DIRECTIONS {
        for latin_span in [false, true] {
            let mut request = input("กก\u{2060}AA", style(Some("th")), direction);
            if latin_span {
                let mut tail = style(Some("en"));
                tail.overflow_wrap = t::OverflowWrap::Anywhere;
                request.source.span(t::Range::new(3, 5), tail);
            }
            let p = prepare(request);
            if cuts(&p) != [1] {
                violations.push((direction, latin_span, cuts(&p)));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "LB11 applies to the real source WJ with or without the original unrelated Latin span: {violations:?}"
    );
}

#[test]
fn normal_keep_all_and_break_all_preserve_wj_adjacency_across_strictnesses() {
    let mut violations = Vec::new();
    for direction in DIRECTIONS {
        for joiner in ['\u{2060}', '\u{feff}'] {
            assert_eq!(
                CodePointMapData::<Class>::new().get32(u32::from(joiner)),
                Class::WordJoiner
            );
            for word in [
                t::WordBreak::Normal,
                t::WordBreak::KeepAll,
                t::WordBreak::BreakAll,
            ] {
                for strictness in STRICTNESSES {
                    for trailing in ["", "ก"] {
                        let value = format!("กก{joiner}{trailing}");
                        let mut base = style(Some("th"));
                        base.word_break = word;
                        base.line_break = strictness;
                        let p = prepare(input(&value, base, direction));
                        let end = value.encode_utf16().count();
                        assert_eq!(
                            p.analyzed_source().graphemes().collect::<Vec<_>>(),
                            (0..end)
                                .map(|at| t::Range::new(at, at + 1))
                                .collect::<Vec<_>>()
                        );
                        if cuts(&p) != [1] {
                            violations.push((
                                direction,
                                joiner,
                                word,
                                strictness,
                                trailing,
                                cuts(&p),
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "LB11 protects both sides of WJ; BreakAll's valid letter-pair cut1 remains independent: {violations:?}"
    );
}

#[test]
fn earlier_zero_width_space_rule_retains_its_break_before_wj_through_spaces() {
    // LB8 precedes LB11: its genuine ZW SP* cause must survive a WJ guard.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB8
    for direction in DIRECTIONS {
        for spaces in [0, 1, 2] {
            let value = format!("A\u{200b}{}\u{2060}A", " ".repeat(spaces));
            let before_wj = 2 + spaces;
            let p = prepare(input(&value, style(Some("th")), direction));
            assert!(
                allowed(&p).iter().any(|offer| {
                    let boundary = offer.view().boundary();
                    boundary.view().source_before().get() == before_wj
                        && boundary.view().source_after().get() == before_wj
                }),
                "the earlier LB8 cause authenticates the before-WJ cut: spaces={spaces} {direction:?}"
            );
            assert!(
                !cuts(&p).contains(&(before_wj + 1)),
                "LB8 does not lend permission to the separate after-WJ cut"
            );
        }
    }
}

#[test]
fn deliberate_line_break_anywhere_overrides_wj_only_at_whole_typographic_units() {
    for direction in DIRECTIONS {
        for (value, expected) in [("กก\u{2060}ก", vec![1, 2, 3]), ("ก่\u{2060}ก", vec![2, 3])]
        {
            for word in [
                t::WordBreak::Normal,
                t::WordBreak::KeepAll,
                t::WordBreak::BreakAll,
            ] {
                let mut base = style(Some("th"));
                base.word_break = word;
                base.line_break = t::LineBreak::Anywhere;
                let p = prepare(input(value, base, direction));
                assert_eq!(
                    cuts(&p),
                    expected,
                    "explicit Anywhere has independent ordinary authority while the base+tone unit stays intact: {value:?} {word:?} {direction:?}"
                );
            }
        }
    }
}

#[test]
fn sa_run_end_cannot_offer_a_break_before_a_whole_glue_unit() {
    // UAX14 revision55 LB12/LB12a protect GL here: the preceding Thai letter
    // is not one of the SP/BA/HY/HH exceptions, and no earlier LB8 applies.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB12a
    let mut violations = Vec::new();
    for glue in ['\u{a0}', '\u{202f}', '\u{2007}'] {
        assert_eq!(
            CodePointMapData::<Class>::new().get32(u32::from(glue)),
            Class::Glue
        );
        assert_eq!(
            CodePointMapData::<Gcb>::new().get32(u32::from(glue)),
            Gcb::Other
        );
        assert_eq!(
            CodePointMapData::<Category>::new().get32(u32::from(glue)),
            Category::SpaceSeparator
        );
        for direction in DIRECTIONS {
            for trailing in ["", "ก"] {
                let value = format!("กก{glue}{trailing}");
                let p = prepare(input(&value, style(Some("th")), direction));
                let end = value.encode_utf16().count();
                assert_eq!(
                    p.analyzed_source().graphemes().collect::<Vec<_>>(),
                    (0..end)
                        .map(|at| t::Range::new(at, at + 1))
                        .collect::<Vec<_>>(),
                    "GL is a genuine whole unit, not an interior combining-mark seam"
                );
                if cuts(&p) != [1] {
                    violations.push((glue, direction, trailing, cuts(&p)));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "retain fallback1 but no before-GL cut2 or after-GL cut3: {violations:?}"
    );
}

#[test]
fn lb12a_space_break_after_and_hyphen_exceptions_keep_before_gl_permission() {
    // These four concrete whole-unit prefixes qualify the adopted LB12a
    // exceptions; a generic GL veto would erase their genuine opportunities.
    for (prefix, class) in [
        (' ', Class::Space),
        ('\u{2003}', Class::BreakAfter),
        ('-', Class::Hyphen),
        ('\u{2010}', Class::UnambiguousHyphen),
    ] {
        assert_eq!(
            CodePointMapData::<Class>::new().get32(u32::from(prefix)),
            class
        );
        assert_eq!(
            CodePointMapData::<Gcb>::new().get32(u32::from(prefix)),
            Gcb::Other
        );
        for glue in ['\u{a0}', '\u{202f}'] {
            assert_eq!(
                CodePointMapData::<Class>::new().get32(u32::from(glue)),
                Class::Glue
            );
            for direction in DIRECTIONS {
                let value = format!("A{prefix}{glue}A");
                let p = prepare(input(&value, style(Some("th")), direction));
                assert_eq!(
                    p.analyzed_source().graphemes().collect::<Vec<_>>(),
                    [
                        t::Range::new(0, 1),
                        t::Range::new(1, 2),
                        t::Range::new(2, 3),
                        t::Range::new(3, 4)
                    ]
                );
                assert!(
                    allowed(&p).iter().any(|offer| {
                        let boundary = offer.view().boundary();
                        boundary.view().source_before().get() == 2
                            && boundary.view().source_after().get() == 2
                    }),
                    "LB12a {class:?} exception supplies the actual before-GL cut: {value:?} {direction:?}"
                );
                assert!(
                    !cuts(&p).contains(&3),
                    "LB12 still protects the separate after-GL edge"
                );
            }
        }
    }
}

#[test]
fn sa_run_end_preserves_hard_precedence_and_does_not_break_before_sp_or_zwsp() {
    let mut violations = Vec::new();
    for direction in DIRECTIONS {
        for separator in [' ', '\u{200b}'] {
            assert_eq!(
                CodePointMapData::<Class>::new().get32(u32::from(separator)),
                if separator == ' ' {
                    Class::Space
                } else {
                    Class::ZWSpace
                }
            );
            let value = format!("กก{separator}ก");
            let p = prepare(input(&value, style(Some("th")), direction));
            assert_eq!(
                p.analyzed_source().graphemes().collect::<Vec<_>>(),
                [
                    t::Range::new(0, 1),
                    t::Range::new(1, 2),
                    t::Range::new(2, 3),
                    t::Range::new(3, 4)
                ]
            );
            if cuts(&p) != [1, 3] {
                violations.push((separator, direction, cuts(&p)));
            }
        }
        // Native preparation already gives real hard separators precedence.
        assert_eq!(
            CodePointMapData::<Class>::new().get32(0x000a),
            Class::LineFeed
        );
        let hard = prepare(input("กก\nก", style(Some("th")), direction));
        assert_eq!(
            cuts(&hard),
            [1],
            "the real hard separator forbids a before-hard soft cut2"
        );
        assert_eq!(
            hard.opportunity_views()
                .filter(|offer| offer.kind() == t::PreparedBreakKind::Forced)
                .map(|offer| offer.boundary().view().source_after().get())
                .collect::<Vec<_>>(),
            [3]
        );
    }
    assert!(
        violations.is_empty(),
        "LB7 forbids provider slice-end cut2 before SP/ZWSP while fallback1 and the genuine after-separator3 remain: {violations:?}"
    );
}

#[test]
fn ordinary_sa_context_has_fallback_without_a_fabricated_latin_run_end() {
    // Text4 lexical-breaking resolves SA as AL plus actual analyzed word cuts.
    // No lexical model exists here: fallback supplies cut1, not the slice end2.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB28
    let mut violations = Vec::new();
    for direction in DIRECTIONS {
        for (value, locale, word) in [
            ("กกAA", "th", t::WordBreak::Normal),
            ("กกAA", "th", t::WordBreak::KeepAll),
            ("กกAA", "th", t::WordBreak::BreakWord),
            ("ກກAA", "lo", t::WordBreak::Normal),
        ] {
            let mut base = style(Some(locale));
            base.word_break = word;
            let p = prepare(input(value, base, direction));
            assert_eq!(
                p.analyzed_source().graphemes().collect::<Vec<_>>(),
                (0..4)
                    .map(|at| t::Range::new(at, at + 1))
                    .collect::<Vec<_>>()
            );
            assert_eq!(p.analyzed_source().style().word_break, word);
            assert_eq!(
                p.analyzed_source().style().overflow_wrap,
                t::OverflowWrap::Normal
            );
            if cuts(&p) != [1] {
                violations.push((direction, value, word, cuts(&p)));
            }
        }
        for (value, locale) in [("กกAA", "th"), ("ກກAA", "lo")] {
            let mut manual = style(Some(locale));
            manual.word_break = t::WordBreak::Manual;
            assert_eq!(
                cuts(&prepare(input(value, manual, direction))),
                [],
                "published Manual AL context remains distinct from lexical fallback"
            );
        }
    }
    assert!(
        violations.is_empty(),
        "ordinary resolved AL context keeps only genuine Thai/Lao fallback1: {violations:?}"
    );
}

#[test]
fn ordinary_sa_resolution_retains_full_letter_number_and_punctuation_context() {
    for (cp, class) in [
        (u32::from('A'), Class::Alphabetic),
        (0x10400, Class::Alphabetic),
        (u32::from('א'), Class::HebrewLetter),
        (u32::from('1'), Class::Numeric),
        (u32::from(','), Class::InfixNumeric),
        (u32::from('('), Class::OpenPunctuation),
    ] {
        assert_eq!(CodePointMapData::<Class>::new().get32(cp), class);
    }
    let mut violations = Vec::new();
    for direction in DIRECTIONS {
        for (value, expected) in [
            ("กกא", vec![1]),          // LB28: AL/HL attachment.
            ("กก1", vec![1]),          // LB23: AL/NU attachment.
            ("1กก", vec![2]),          // LB23: NU/AL attachment, fallback2.
            ("(  กกA", vec![4]),       // LB14 OP SP*; then fallback4, LB28.
            (",กกA", vec![2]),         // LB29 IS/AL; then fallback2, LB28.
            ("กก A", vec![1, 3]),      // Independent preserved-SPACE owner.
            ("\u{10400}กกA", vec![3]), // Unchanged four-byte AL context.
        ] {
            for ch in value.chars() {
                assert_eq!(
                    CodePointMapData::<Gcb>::new().get32(u32::from(ch)),
                    Gcb::Other
                );
            }
            let expected_units = value
                .chars()
                .scan(0, |at, ch| {
                    let start = *at;
                    *at += ch.len_utf16();
                    Some(t::Range::new(start, *at))
                })
                .collect::<Vec<_>>();
            let p = prepare(input(value, style(Some("th")), direction));
            assert_eq!(
                p.analyzed_source().graphemes().collect::<Vec<_>>(),
                expected_units
            );
            if cuts(&p) != expected {
                violations.push((direction, value, cuts(&p), expected));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "LB1-resolved SA context must pass through the complete ordinary rules, with exact source endpoints: {violations:?}"
    );
}

#[test]
fn ordinary_sa_marks_inherit_glue_before_a_loose_cjk_opportunity() {
    // LB1 resolves SA Mn/Mc to CM; LB9 retains GL through that mark, so LB12
    // still forbids the Loose candidate. The Mc fixture tests class resolution,
    // not Myanmar lexical or orthographic support.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB1
    assert_eq!(CodePointMapData::<Class>::new().get32(0x00a0), Class::Glue);
    let mut violations = Vec::new();
    for (mark, category, class, gcb) in [
        (
            '\u{0e48}',
            Category::NonspacingMark,
            Class::ComplexContext,
            Gcb::Extend,
        ),
        (
            '\u{103b}',
            Category::SpacingMark,
            Class::ComplexContext,
            Gcb::SpacingMark,
        ),
        (
            '\u{0301}',
            Category::NonspacingMark,
            Class::CombiningMark,
            Gcb::Extend,
        ),
    ] {
        let cp = u32::from(mark);
        assert_eq!(CodePointMapData::<Category>::new().get32(cp), category);
        assert_eq!(CodePointMapData::<Class>::new().get32(cp), class);
        assert_eq!(CodePointMapData::<Gcb>::new().get32(cp), gcb);
        for direction in DIRECTIONS {
            for word in [t::WordBreak::Normal, t::WordBreak::Manual] {
                let mut base = style(Some("ja"));
                base.word_break = word;
                base.line_break = t::LineBreak::Loose;
                let value = format!("\u{a0}{mark}\u{301c}");
                let p = prepare(input(&value, base, direction));
                assert_eq!(
                    p.analyzed_source().graphemes().collect::<Vec<_>>(),
                    [t::Range::new(0, 2), t::Range::new(2, 3)]
                );
                if !cuts(&p).is_empty() {
                    violations.push((mark, direction, word, cuts(&p)));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "resolved CM must not erase inherited GL or manufacture cut2: {violations:?}"
    );
}

#[test]
fn resolved_sa_context_preserves_required_loose_and_deliberate_anywhere_owners() {
    for direction in DIRECTIONS {
        for (word, expected) in [
            (t::WordBreak::Normal, vec![1, 2]),
            (t::WordBreak::Manual, vec![2]),
        ] {
            let mut base = style(Some("ja"));
            base.word_break = word;
            base.line_break = t::LineBreak::Loose;
            assert_eq!(
                cuts(&prepare(input("กก\u{301c}", base, direction))),
                expected,
                "the actual required CJK owner survives ordinary SA resolution"
            );
        }
        let mut base = style(Some("th"));
        base.line_break = t::LineBreak::Anywhere;
        assert_eq!(
            cuts(&prepare(input("กกAA", base, direction))),
            [1, 2, 3],
            "deliberate Anywhere retains its independent whole-TCU cuts"
        );
    }
}
