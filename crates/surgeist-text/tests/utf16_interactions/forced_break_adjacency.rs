//! Public hard-break precedence over atomic soft opportunities.
//!
//! The pinned August 2026 CSS Text 3/4 editions cite UAX #14 revision 55
//! (Unicode 17): LB4/LB5 require the following break, and LB6 prohibits a
//! preceding soft break before the later CB/LB20 rule is considered.
//! https://www.unicode.org/reports/tr14/tr14-55.html#LB6
//! The CSS atomic-compat-wrap exception does not override that precedence.

use super::*;

const SOURCE_ID: Id = Id::from_u64(108040);
const SOURCE_REVISION: u64 = 3;

fn prepare_paragraph(
    text: &str,
    kinds: &[ParagraphControlKind],
    transparent_chain: bool,
    white_space: WhiteSpace,
) -> PreparedText {
    prepare_paragraph_at(text, kinds, transparent_chain, white_space, 0)
}

fn prepare_paragraph_at(
    text: &str,
    kinds: &[ParagraphControlKind],
    transparent_chain: bool,
    white_space: WhiteSpace,
    control_at: usize,
) -> PreparedText {
    let mut source = Source::identified(text, SOURCE_ID, SOURCE_REVISION);
    source.inline_box(InlineBox::new(
        Id::from_u64(40),
        InlineBoxKind::InFlow,
        0,
        Size::new(40.0, 20.0),
    ));
    let mut anchor_order = vec![ParagraphAnchor {
        item: ParagraphAnchorItem::Atomic(0),
        affinity: Affinity::Before,
    }];
    if transparent_chain {
        source.inline_box(InlineBox::new(
            Id::from_u64(41),
            InlineBoxKind::OutOfFlow,
            0,
            Size::new(4.0, 5.0),
        ));
        anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(1),
            affinity: Affinity::Before,
        });
    }
    anchor_order.extend((0..kinds.len()).map(|index| ParagraphAnchor {
        item: ParagraphAnchorItem::Control(index),
        affinity: Affinity::After,
    }));
    let length = source.text().len();
    let analysis = AnalyzedSource::from_paragraph(ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            white_space,
            wrap: Wrap::Word,
            ..style()
        },
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            source: source.clone(),
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: Vec::new(),
        controls: kinds
            .iter()
            .cloned()
            .map(|kind| ParagraphControl {
                at: control_at,
                kind,
            })
            .collect(),
        boundary_policies: [
            ParagraphBoundaryDefinition::AtomicBefore(0),
            ParagraphBoundaryDefinition::AtomicAfter(0),
        ]
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .to_vec(),
        replacements: kinds
            .iter()
            .enumerate()
            .filter_map(|(control, kind)| {
                matches!(kind, ParagraphControlKind::DiscretionaryBreak { .. }).then_some(
                    ParagraphReplacement::resolved_native_logical_before(
                        ParagraphReplacementDefinition::Control {
                            control,
                            origin: None,
                        },
                        control_at,
                        vec![Text::from("-")],
                    )
                    .unwrap(),
                )
            })
            .collect(),
        anchor_order,
    })
    .expect("the actual paragraph capture is valid");
    let identity = analysis.identity();
    let mut system = registered_ahem();
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .expect("registered Ahem prepares the real mixed paragraph");
    drop(system);

    let retained = prepared.analyzed_source();
    assert_eq!(retained.identity(), identity);
    assert_eq!(retained.source(), &source);
    assert_eq!(retained.source().id(), Some(SOURCE_ID));
    assert_eq!(retained.source().revision(), SOURCE_REVISION);
    assert_eq!(
        retained.source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        prepared
            .atomic_views()
            .map(|atomic| *atomic.original_box())
            .collect::<Vec<_>>(),
        source.boxes()
    );
    assert_eq!(
        prepared
            .control_views()
            .map(|control| control.kind().clone())
            .collect::<Vec<_>>(),
        kinds
    );
    let controls: Vec<_> = prepared
        .control_views()
        .map(|control| {
            assert_eq!(control.source_position().get(), control_at);
            let handle = control.handle();
            assert_eq!(handle.prepared_text(), prepared);
            handle
        })
        .collect();
    for (index, control) in controls.iter().enumerate() {
        assert!(!controls[..index].contains(control));
    }
    // Ahem's 1000-unit X advance at UPEM 1000 and size 20 qualifies that
    // these offers came from actual registered-font preparation.
    assert_eq!(
        prepared
            .unit_views()
            .find(|unit| unit.source_range() == Range::new(length - 1, length))
            .expect("the actual authored X remains a prepared text unit")
            .advance(),
        20.0
    );
    prepared
}

fn after_in_flow(prepared: &PreparedText) -> usize {
    let atomic = prepared
        .atomic_views()
        .find(|atomic| atomic.original_box().kind() == InlineBoxKind::InFlow)
        .unwrap()
        .handle();
    assert_eq!(atomic.prepared_text(), prepared.clone());
    prepared
        .participants()
        .position(|participant| participant == PreparedParticipant::Atomic(atomic.clone()))
        .unwrap()
        + 1
}

fn native_hard_positions(prepared: &PreparedText) -> Vec<usize> {
    prepared
        .participants()
        .enumerate()
        .filter_map(|(index, participant)| match participant {
            PreparedParticipant::Control(control)
                if control.view().kind() == &ParagraphControlKind::ForcedBreak =>
            {
                Some(index)
            }
            _ => None,
        })
        .collect()
}

fn assert_hard_break_precedence(
    prepared: &PreparedText,
    hard_positions: &[usize],
    soft_forbidden_start: usize,
) {
    let offers: Vec<_> = prepared
        .opportunity_views()
        .map(|offer| {
            assert_eq!(offer.handle().prepared_text(), prepared.clone());
            let boundary = offer.boundary();
            assert_eq!(boundary.prepared_text(), prepared.clone());
            (boundary.view().participant_offset(), offer.kind())
        })
        .collect();
    let forced: Vec<_> = offers
        .iter()
        .filter_map(|&(offset, kind)| (kind == PreparedBreakKind::Forced).then_some(offset))
        .collect();
    assert_eq!(
        forced,
        hard_positions
            .iter()
            .map(|index| index + 1)
            .collect::<Vec<_>>(),
        "each actual hard occurrence retains its following mandatory offer"
    );
    assert_eq!(
        offers
            .iter()
            .filter_map(|&(offset, kind)| {
                (kind == PreparedBreakKind::ParagraphEnd).then_some(offset)
            })
            .collect::<Vec<_>>(),
        [prepared.participants().count()]
    );
    let before_hard = soft_forbidden_start..=*hard_positions.last().unwrap();
    let soft: Vec<_> = offers
        .iter()
        .filter_map(|&(offset, kind)| {
            (matches!(
                kind,
                PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
            ) && before_hard.contains(&offset))
            .then_some((offset, kind))
        })
        .collect();
    // Mandatory offers between consecutive hard controls are preserved above.
    // LB6 excludes additional soft offers; LB20 cannot reintroduce them.
    assert_eq!(
        soft,
        Vec::<(usize, PreparedBreakKind)>::new(),
        "all issued offers: {offers:?}"
    );
}

#[test]
fn native_atomic_before_forced_control_has_no_soft_opportunity() {
    let prepared = prepare_paragraph(
        "X",
        &[ParagraphControlKind::ForcedBreak],
        false,
        WhiteSpace::Preserve,
    );
    assert_hard_break_precedence(
        &prepared,
        &native_hard_positions(&prepared),
        after_in_flow(&prepared),
    );
}

#[test]
fn transparent_occurrences_do_not_hide_the_following_hard_control() {
    let prepared = prepare_paragraph(
        "X",
        &[
            ParagraphControlKind::InlineBoundaryStart,
            ParagraphControlKind::ForcedBreak,
        ],
        true,
        WhiteSpace::Preserve,
    );
    assert_hard_break_precedence(
        &prepared,
        &native_hard_positions(&prepared),
        after_in_flow(&prepared),
    );
}

#[test]
fn consecutive_hard_controls_keep_each_mandatory_offer_without_a_soft_offer() {
    let prepared = prepare_paragraph(
        "X",
        &[
            ParagraphControlKind::ForcedBreak,
            ParagraphControlKind::ForcedBreak,
        ],
        false,
        WhiteSpace::Preserve,
    );
    let hard = native_hard_positions(&prepared);
    assert_eq!(hard.len(), 2);
    assert_hard_break_precedence(&prepared, &hard, after_in_flow(&prepared));
}

#[test]
fn preserved_source_lf_is_hard_without_fabricating_a_native_control() {
    let prepared = prepare_paragraph("\nX", &[], false, WhiteSpace::Preserve);
    assert_eq!(prepared.control_views().len(), 0);
    assert_eq!(
        prepared
            .analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [0x0a, 0x58]
    );
    let lf = prepared
        .unit_views()
        .find(|unit| unit.source_range() == Range::new(0, 1))
        .unwrap()
        .handle();
    let position = prepared
        .participants()
        .position(|participant| participant == PreparedParticipant::Text(lf.clone()))
        .unwrap();
    assert_hard_break_precedence(&prepared, &[position], after_in_flow(&prepared));
}

#[test]
fn optional_offer_immediately_before_a_hard_control_is_prohibited() {
    let prepared = prepare_paragraph(
        "X",
        &[
            ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            ParagraphControlKind::ForcedBreak,
        ],
        false,
        WhiteSpace::Preserve,
    );
    let hard = native_hard_positions(&prepared);
    assert_eq!(hard.len(), 1);
    assert_hard_break_precedence(&prepared, &hard, hard[0]);
}

#[test]
fn discretionary_offer_immediately_before_a_hard_control_is_prohibited() {
    let prepared = prepare_paragraph(
        "X",
        &[
            ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(style()),
            },
            ParagraphControlKind::ForcedBreak,
        ],
        false,
        WhiteSpace::Preserve,
    );
    let hard = native_hard_positions(&prepared);
    assert_eq!(hard.len(), 1);
    assert_hard_break_precedence(&prepared, &hard, hard[0]);
}

#[test]
fn optional_and_discretionary_owners_keep_permission_without_a_hard_neighbor() {
    for (kind, expected) in [
        (
            ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            PreparedBreakKind::Allowed,
        ),
        (
            ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(style()),
            },
            PreparedBreakKind::Discretionary,
        ),
    ] {
        let short = prepare_paragraph(
            "X",
            std::slice::from_ref(&kind),
            false,
            WhiteSpace::Preserve,
        );
        if expected == PreparedBreakKind::Discretionary {
            assert!(
                !short
                    .opportunity_views()
                    .any(|v| v.kind() == PreparedBreakKind::Discretionary),
                "no before/after lexical counts at the original beginning control"
            );
        }
        if expected == PreparedBreakKind::Allowed {
            let control = short.control_views().next().unwrap().handle();
            let after = short
                .participants()
                .position(|p| p == PreparedParticipant::Control(control.clone()))
                .unwrap()
                + 1;
            assert!(
                short
                    .opportunity_views()
                    .any(|v| v.boundary().view().participant_offset() == after
                        && v.kind() == PreparedBreakKind::Allowed),
                "the original atomic-adjacent OptionalBreak at zero keeps its independent permission"
            );
        }
        let prepared = prepare_paragraph_at("AXXXX", &[kind], false, WhiteSpace::Preserve, 2);
        let control = prepared.control_views().next().unwrap().handle();
        let after_control = prepared
            .participants()
            .position(|participant| participant == PreparedParticipant::Control(control.clone()))
            .unwrap()
            + 1;
        let offer = prepared
            .opportunity_views()
            .find(|offer| offer.boundary().view().participant_offset() == after_control)
            .expect("the actual soft owner has its own opportunity");
        assert_eq!(offer.kind(), expected);
        if expected == PreparedBreakKind::Discretionary {
            assert_eq!(offer.alternatives().len(), 1);
            let alternative = offer.alternatives().next().unwrap();
            assert_eq!(alternative.text().code_units(), [0x2d]);
            match alternative.definition() {
                PreparedAlternativeDefinition::Control {
                    control: defining,
                    origin,
                } => {
                    assert_eq!(defining, control);
                    assert!(origin.is_none());
                }
                PreparedAlternativeDefinition::Automatic { .. }
                | PreparedAlternativeDefinition::Source { .. } => {
                    panic!("a real native control must not acquire a fabricated source definition");
                }
            }
        } else {
            assert_eq!(offer.alternatives().len(), 0);
        }
    }
}

#[test]
fn collapsed_source_lf_has_space_semantics_without_a_forced_offer() {
    let prepared = prepare_paragraph("\nX", &[], false, WhiteSpace::Collapse);
    assert_eq!(prepared.control_views().len(), 0);
    assert_eq!(
        prepared
            .analyzed_source()
            .processed_source()
            .processed_text()
            .code_units(),
        [0x20, 0x58]
    );
    let after = after_in_flow(&prepared);
    let offers: Vec<_> = prepared
        .opportunity_views()
        .map(|offer| (offer.boundary().view().participant_offset(), offer.kind()))
        .collect();
    assert!(offers.contains(&(after, PreparedBreakKind::Allowed)));
    assert!(
        !offers
            .iter()
            .any(|&(_, kind)| kind == PreparedBreakKind::Forced)
    );
}

#[test]
fn ordinary_atomic_permission_and_nbsp_exception_remain_distinct_from_hard_breaks() {
    for (text, allowed) in [("X", true), ("\u{a0}X", true), ("\u{202f}X", false)] {
        let prepared = prepare_paragraph(text, &[], false, WhiteSpace::Preserve);
        let after = after_in_flow(&prepared);
        assert_eq!(
            prepared.opportunity_views().any(|offer| {
                offer.kind() == PreparedBreakKind::Allowed
                    && offer.boundary().view().participant_offset() == after
            }),
            allowed,
            "actual source {text:?}"
        );
        assert!(
            !prepared
                .opportunity_views()
                .any(|offer| offer.kind() == PreparedBreakKind::Forced)
        );
    }
}
