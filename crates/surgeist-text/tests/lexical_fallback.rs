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
            p.conditional_opportunities(
                &start,
                None,
                &ceiling,
                t::ConditionalPurpose::Overflow,
                &[]
            )
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
                line_strut: None,
                context: t::LineContext {
                    base_direction: direction,
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
