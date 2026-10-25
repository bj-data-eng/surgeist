#![forbid(unsafe_code)]

//! A disconnected origin starts Unicode context, not an ordinary opportunity.
//!
//! ParagraphConnectionKind captures genuine text contiguity. Its discontinuity
//! supplies no source character or control, and wrapping permission alone does
//! not create an opportunity (docs/text-layout-contract.md). ICU's component
//! start marker is not a meaningful line-break opportunity. Explicit controls,
//! in-flow atomics and actual BreakSpaces separators keep their separate owners.

use surgeist_text as t;

const DIRECTIONS: [t::ParagraphBaseDirection; 2] = [
    t::ParagraphBaseDirection::LeftToRight,
    t::ParagraphBaseDirection::RightToLeft,
];
const MODES: [t::WordBreak; 3] = [
    t::WordBreak::Normal,
    t::WordBreak::KeepAll,
    t::WordBreak::BreakAll,
];

fn style(mode: t::WordBreak) -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        word_break: mode,
        ..t::Style::default()
    }
}

fn paragraph(
    value: &str,
    at: usize,
    connection: t::ParagraphConnectionKind,
    mode: t::WordBreak,
    direction: t::ParagraphBaseDirection,
) -> t::ParagraphInput {
    let base_style = style(mode);
    let source = t::Source::identified(value, t::Id::from_u64(65001), 1);
    let units = value.encode_utf16().collect::<Vec<_>>();
    let origins = [(0, at), (at, units.len())]
        .into_iter()
        .enumerate()
        .map(|(index, (start, end))| {
            let text = t::Text::from(String::from_utf16(&units[start..end]).unwrap());
            let local = t::Range::new(0, end - start);
            let mut original = t::Source::identified(
                text.clone(),
                t::Id::from_u64(65002 + u64::try_from(index).unwrap()),
                1,
            );
            original.span(local, base_style.clone());
            t::ParagraphOrigin {
                source: original,
                local,
                aggregate: t::Range::new(start, end),
                kind: t::ParagraphOriginKind::Authored,
                text_node: Some(t::CompleteTextNodeContribution::new(text, local).unwrap()),
            }
        })
        .collect();
    t::ParagraphInput {
        source,
        base_style,
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: t::ParagraphStrut::DefaultFont,
        terminal_edge: t::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins,
        connections: vec![t::ParagraphConnection {
            at,
            kind: connection,
        }],
        controls: vec![],
        // These are actual wrapping children of a wrapping ancestor. Supplying
        // the same permission for Disconnected cannot invent text adjacency.
        boundary_policies: vec![policy(t::ParagraphBoundaryDefinition::Connection(0))],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn policy(definition: t::ParagraphBoundaryDefinition) -> t::ParagraphBoundaryPolicy {
    t::ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: t::Wrap::Word,
        following_wrap: t::Wrap::Word,
        ancestor_wrap: t::Wrap::Word,
    }
}

fn prepare(input: t::ParagraphInput) -> t::PreparedText {
    let source = input.source.clone();
    let analyzed = t::AnalyzedSource::from_paragraph(input).unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| t::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    let prepared = system
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert!(prepared.opportunity_views().any(|offer| {
        offer.kind() == t::PreparedBreakKind::ParagraphEnd
            && offer.boundary().view().source_after().get() == source.text().len()
    }));
    prepared
}

fn cuts(prepared: &t::PreparedText, kind: t::PreparedBreakKind) -> Vec<usize> {
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
                t::ParagraphConnectionKind::Disconnected,
                mode,
                direction,
            ));
            let allowed = cuts(&prepared, t::PreparedBreakKind::Allowed);
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
                t::ParagraphConnectionKind::Continuous,
                mode,
                direction,
            ));
            let expected = if mode == t::WordBreak::BreakAll {
                vec![1]
            } else {
                vec![]
            };
            assert_eq!(
                cuts(&prepared, t::PreparedBreakKind::Allowed),
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
            t::ParagraphControlKind::OptionalBreak {
                wrap: t::Wrap::Word,
            },
            t::ParagraphControlKind::Wbr {
                style: Box::new(style(t::WordBreak::Normal)),
            },
            t::ParagraphControlKind::ForcedBreak,
        ] {
            let mut input = paragraph(
                "ab",
                1,
                t::ParagraphConnectionKind::Disconnected,
                t::WordBreak::Normal,
                direction,
            );
            let kind = if control == t::ParagraphControlKind::ForcedBreak {
                t::PreparedBreakKind::Forced
            } else {
                t::PreparedBreakKind::Allowed
            };
            input.controls.push(t::ParagraphControl {
                at: 1,
                kind: control.clone(),
            });
            input.anchor_order.push(t::ParagraphAnchor {
                item: t::ParagraphAnchorItem::Control(0),
                affinity: t::Affinity::After,
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
            t::ParagraphConnectionKind::Disconnected,
            t::WordBreak::Normal,
            direction,
        );
        input.source.inline_box(t::InlineBox::new(
            t::Id::from_u64(65004),
            t::InlineBoxKind::InFlow,
            1,
            t::Size::new(10.0, 10.0),
        ));
        input.anchor_order.push(t::ParagraphAnchor {
            item: t::ParagraphAnchorItem::Atomic(0),
            affinity: t::Affinity::Before,
        });
        input.boundary_policies.extend([
            policy(t::ParagraphBoundaryDefinition::AtomicBefore(0)),
            policy(t::ParagraphBoundaryDefinition::AtomicAfter(0)),
        ]);
        let prepared = prepare(input);
        let atomic = prepared.atomic_views().next().unwrap().handle();
        let offset = prepared
            .participants()
            .position(|item| item == t::PreparedParticipant::Atomic(atomic.clone()))
            .unwrap();
        let offers = prepared
            .opportunity_views()
            .filter(|offer| offer.kind() == t::PreparedBreakKind::Allowed)
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
            t::ParagraphConnectionKind::Disconnected,
            t::WordBreak::Normal,
            direction,
        );
        input.base_style.white_space = t::WhiteSpace::BreakSpaces;
        for origin in &mut input.origins {
            origin.source.span(origin.local, input.base_style.clone());
        }
        let prepared = prepare(input);
        assert!(
            cuts(&prepared, t::PreparedBreakKind::Allowed).contains(&2),
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
            t::ParagraphConnectionKind::Disconnected,
            t::WordBreak::Normal,
            direction,
        ));
        assert!(
            cuts(&prepared, t::PreparedBreakKind::Allowed).contains(&2),
            "{direction:?}"
        );
    }
}
