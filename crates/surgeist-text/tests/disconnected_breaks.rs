#![forbid(unsafe_code)]

//! A disconnected origin starts Unicode context, not an ordinary opportunity.
//!
//! ParagraphConnectionKind captures text contiguity. Its discontinuity
//! supplies no source character or control, and wrapping permission alone does
//! not create an opportunity (docs/text-layout-contract.md). ICU's component
//! start marker is not a meaningful line-break opportunity. Explicit controls,
//! in-flow atomics and actual BreakSpaces separators keep their separate owners.

use surgeist_text as t;

const DIRECTIONS: [t::paragraph::ParagraphBaseDirection; 2] = [
    t::paragraph::ParagraphBaseDirection::LeftToRight,
    t::paragraph::ParagraphBaseDirection::RightToLeft,
];
const MODES: [t::style::WordBreak; 3] = [
    t::style::WordBreak::Normal,
    t::style::WordBreak::KeepAll,
    t::style::WordBreak::BreakAll,
];

fn style(mode: t::style::WordBreak) -> t::style::Style {
    t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 10.0,
        word_break: mode,
        ..t::style::Style::default()
    }
}

fn paragraph(
    value: &str,
    at: usize,
    connection: t::paragraph::ParagraphConnectionKind,
    mode: t::style::WordBreak,
    direction: t::paragraph::ParagraphBaseDirection,
) -> t::paragraph::ParagraphInput {
    let base_style = style(mode);
    let source = t::source::Source::identified(value, t::source::Id::from_u64(65001), 1);
    let units = value.encode_utf16().collect::<Vec<_>>();
    let origins = [(0, at), (at, units.len())]
        .into_iter()
        .enumerate()
        .map(|(index, (start, end))| {
            let text = t::source::Text::from(String::from_utf16(&units[start..end]).unwrap());
            let local = t::source::Range::new(0, end - start);
            let mut original = t::source::Source::identified(
                text.clone(),
                t::source::Id::from_u64(65002 + u64::try_from(index).unwrap()),
                1,
            );
            original.span(local, base_style.clone());
            t::paragraph::ParagraphOrigin {
                source: original,
                local,
                aggregate: t::source::Range::new(start, end),
                kind: t::paragraph::ParagraphOriginKind::Authored,
                text_node: Some(
                    t::paragraph::CompleteTextNodeContribution::new(text, local).unwrap(),
                ),
            }
        })
        .collect();
    t::paragraph::ParagraphInput {
        source,
        base_style,
        segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
        terminal_edge: t::paragraph::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins,
        connections: vec![t::paragraph::ParagraphConnection {
            at,
            kind: connection,
        }],
        controls: vec![],
        // These are actual wrapping children of a wrapping ancestor. Supplying
        // the same permission for Disconnected cannot invent text adjacency.
        boundary_policies: vec![policy(
            t::paragraph::ParagraphBoundaryDefinition::Connection(0),
        )],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn policy(
    definition: t::paragraph::ParagraphBoundaryDefinition,
) -> t::paragraph::ParagraphBoundaryPolicy {
    t::paragraph::ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: t::style::Wrap::Word,
        following_wrap: t::style::Wrap::Word,
        ancestor_wrap: t::style::Wrap::Word,
    }
}

fn prepare(input: t::paragraph::ParagraphInput) -> t::prepared::PreparedText {
    let source = input.source.clone();
    let analyzed = t::analyzed::AnalyzedSource::from_paragraph(
        input,
        t::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: t::style::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::default();
    system
        .register_fonts(
            include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            crate::common::FONT_LIMITS,
        )
        .unwrap();
    let prepared = system
        .prepare(&analyzed, t::prepared::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert!(prepared.opportunity_views().any(|offer| {
        offer.kind() == t::prepared::PreparedBreakKind::ParagraphEnd
            && offer.boundary().view().source_after().get() == source.text().len()
    }));
    prepared
}

fn cuts(prepared: &t::prepared::PreparedText, kind: t::prepared::PreparedBreakKind) -> Vec<usize> {
    prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == kind)
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
fn disconnected_latin_seams_do_not_create_ordinary_opportunities() {
    let mut failures = Vec::new();
    for direction in DIRECTIONS {
        for mode in MODES {
            let prepared = prepare(paragraph(
                "ab",
                1,
                t::paragraph::ParagraphConnectionKind::Disconnected,
                mode,
                direction,
            ));
            let allowed = cuts(&prepared, t::prepared::PreparedBreakKind::Allowed);
            if !allowed.is_empty() {
                failures.push((direction, mode, allowed));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "discontinuity and Word permission cannot create a text/text opportunity: {failures:?}"
    );
}

#[test]
fn continuous_latin_seams_retain_only_real_word_break_opportunities() {
    for direction in DIRECTIONS {
        for mode in MODES {
            let prepared = prepare(paragraph(
                "ab",
                1,
                t::paragraph::ParagraphConnectionKind::Continuous,
                mode,
                direction,
            ));
            let expected = if mode == t::style::WordBreak::BreakAll {
                vec![1]
            } else {
                vec![]
            };
            assert_eq!(
                cuts(&prepared, t::prepared::PreparedBreakKind::Allowed),
                expected,
                "{direction:?} {mode:?}"
            );
        }
    }
}

#[test]
fn disconnected_seams_keep_explicit_optional_wbr_and_forced_owners() {
    for direction in DIRECTIONS {
        for control in [
            t::paragraph::ParagraphControlKind::OptionalBreak {
                wrap: t::style::Wrap::Word,
            },
            t::paragraph::ParagraphControlKind::Wbr {
                style: Box::new(style(t::style::WordBreak::Normal)),
            },
            t::paragraph::ParagraphControlKind::ForcedBreak,
        ] {
            let mut input = paragraph(
                "ab",
                1,
                t::paragraph::ParagraphConnectionKind::Disconnected,
                t::style::WordBreak::Normal,
                direction,
            );
            let kind = if control == t::paragraph::ParagraphControlKind::ForcedBreak {
                t::prepared::PreparedBreakKind::Forced
            } else {
                t::prepared::PreparedBreakKind::Allowed
            };
            input.controls.push(t::paragraph::ParagraphControl {
                at: 1,
                kind: control.clone(),
            });
            input.anchor_order.push(t::paragraph::ParagraphAnchor {
                item: t::paragraph::ParagraphAnchorItem::Control(0),
                affinity: t::layout::Affinity::After,
            });
            let prepared = prepare(input);
            assert_eq!(prepared.control_views().count(), 1);
            assert_eq!(prepared.control_views().next().unwrap().kind(), &control);
            assert!(
                cuts(&prepared, kind).contains(&1),
                "{direction:?} {control:?}"
            );
        }
    }
}

#[test]
fn disconnected_seams_keep_both_qualified_in_flow_atomic_offers() {
    for direction in DIRECTIONS {
        let mut input = paragraph(
            "ab",
            1,
            t::paragraph::ParagraphConnectionKind::Disconnected,
            t::style::WordBreak::Normal,
            direction,
        );
        input.source.inline_box(t::source::InlineBox::new(
            t::source::Id::from_u64(65004),
            t::source::InlineBoxKind::InFlow,
            1,
            t::geometry::Size::new(10.0, 10.0),
        ));
        input.anchor_order.push(t::paragraph::ParagraphAnchor {
            item: t::paragraph::ParagraphAnchorItem::Atomic(0),
            affinity: t::layout::Affinity::Before,
        });
        input.boundary_policies.extend([
            policy(t::paragraph::ParagraphBoundaryDefinition::AtomicBefore(0)),
            policy(t::paragraph::ParagraphBoundaryDefinition::AtomicAfter(0)),
        ]);
        let prepared = prepare(input);
        let atomic = prepared.atomic_views().next().unwrap().handle();
        let offset = prepared
            .participants()
            .position(|item| item == t::prepared::PreparedParticipant::Atomic(atomic.clone()))
            .unwrap();
        let offers = prepared
            .opportunity_views()
            .filter(|offer| offer.kind() == t::prepared::PreparedBreakKind::Allowed)
            .map(|offer| offer.boundary().view().participant_offset())
            .collect::<Vec<_>>();
        assert!(
            offers.contains(&offset),
            "before atomic: {direction:?} {offers:?}"
        );
        assert!(
            offers.contains(&(offset + 1)),
            "after atomic: {direction:?} {offers:?}"
        );
    }
}

#[test]
fn actual_break_spaces_separator_keeps_its_offer_at_a_disconnected_seam() {
    for direction in DIRECTIONS {
        let mut input = paragraph(
            "a b",
            2,
            t::paragraph::ParagraphConnectionKind::Disconnected,
            t::style::WordBreak::Normal,
            direction,
        );
        input.base_style.white_space = t::style::WhiteSpace::BreakSpaces;
        for origin in &mut input.origins {
            origin.source.span(origin.local, input.base_style.clone());
        }
        let prepared = prepare(input);
        assert!(
            cuts(&prepared, t::prepared::PreparedBreakKind::Allowed).contains(&2),
            "{direction:?}"
        );
    }
}

#[test]
fn actual_preserved_space_keeps_its_offer_at_a_disconnected_seam() {
    // Pinned CSS Text 4 whitespace processing supplies a soft wrap at the end
    // of each maximal preserved SPACE/TAB sequence. This plain Latin example
    // has no opening punctuation or other Unicode rule suppressing that offer.
    // https://www.w3.org/TR/2026/WD-css-text-4-20260814/#white-space-processing
    for direction in DIRECTIONS {
        let prepared = prepare(paragraph(
            "a b",
            2,
            t::paragraph::ParagraphConnectionKind::Disconnected,
            t::style::WordBreak::Normal,
            direction,
        ));
        assert!(
            cuts(&prepared, t::prepared::PreparedBreakKind::Allowed).contains(&2),
            "{direction:?}"
        );
    }
}

#[test]
fn source_zero_width_space_keeps_its_lb8_offer_at_a_disconnected_seam() {
    // UAX #14 revision55 LB8 supplies ZW SP* opportunities before the later
    // LB11 word-joiner and LB13 closing-punctuation prohibitions are considered.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB8
    let mut failures = Vec::new();
    for direction in DIRECTIONS {
        for mode in MODES {
            for spaces in ["", " ", "  "] {
                for following in ["b", "\u{2060}b", ")b"] {
                    let value = format!("a\u{200b}{spaces}{following}");
                    let at = 2 + spaces.len();
                    let prepared = prepare(paragraph(
                        &value,
                        at,
                        t::paragraph::ParagraphConnectionKind::Disconnected,
                        mode,
                        direction,
                    ));
                    let allowed = cuts(&prepared, t::prepared::PreparedBreakKind::Allowed);
                    if !allowed.contains(&at) {
                        failures.push((direction, mode, value, at, allowed));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "actual source ZW SP* retains its independent LB8 permission: {failures:?}"
    );
}

#[test]
fn source_zero_width_space_does_not_override_earlier_lb6_hard_break_protection() {
    // LB6 precedes LB8: ZW SP* cannot permit a soft break before LF or BK.
    // The actual hard separator still owns its following mandatory break.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB6
    for direction in DIRECTIONS {
        for mode in MODES {
            for spaces in ["", "  "] {
                for hard in ["\n", "\u{2028}"] {
                    let value = format!("a\u{200b}{spaces}{hard}b");
                    let at = 2 + spaces.len();
                    let prepared = prepare(paragraph(
                        &value,
                        at,
                        t::paragraph::ParagraphConnectionKind::Disconnected,
                        mode,
                        direction,
                    ));
                    assert!(
                        !cuts(&prepared, t::prepared::PreparedBreakKind::Allowed).contains(&at),
                        "LB6 before hard: {direction:?} {mode:?} {value:?}"
                    );
                    assert!(
                        cuts(&prepared, t::prepared::PreparedBreakKind::Forced).contains(&(at + 1)),
                        "mandatory after hard: {direction:?} {mode:?} {value:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn older_disconnected_zero_width_space_does_not_lend_permission_to_a_later_cut() {
    // The component beginning with SPACE has no ZW of its own. Its interior
    // cut before WJ is protected by LB11; an older component cannot donate LB8.
    // https://www.unicode.org/reports/tr14/tr14-55.html#LB11
    for direction in DIRECTIONS {
        for mode in MODES {
            let prepared = prepare(paragraph(
                "a\u{200b} \u{2060}b",
                2,
                t::paragraph::ParagraphConnectionKind::Disconnected,
                mode,
                direction,
            ));
            assert!(
                !cuts(&prepared, t::prepared::PreparedBreakKind::Allowed).contains(&3),
                "no LB8 donor across disconnection: {direction:?} {mode:?}"
            );
        }
    }
}

mod common;
