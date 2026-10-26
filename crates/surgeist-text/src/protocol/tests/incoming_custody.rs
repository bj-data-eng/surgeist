//! Receiver event custody is independent of visibility and local bidi parity.
use super::*;
use crate::ParagraphControlKind as K;
use crate::protocol::prepared::{ParticipantData, PresentationUnitKind as U};

fn fixture(
    source: Source,
    direction: ParagraphBaseDirection,
    kinds: Vec<(usize, K)>,
) -> PreparedText {
    let len = source.text().len();
    let mut style = own_style("A", HyphenationPlacement::LogicalAfter);
    style.hyphenate_limit_chars = crate::HyphenateLimitChars {
        total: crate::HyphenationMinimum::Count(0),
        before: crate::HyphenationMinimum::Count(0),
        after: crate::HyphenationMinimum::Count(0),
    };
    let mut anchors: Vec<_> = (0..kinds.len())
        .map(|control| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(control),
            affinity: crate::Affinity::Before,
        })
        .collect();
    anchors.extend(
        (0..source.boxes().len()).map(|atomic| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Atomic(atomic),
            affinity: crate::Affinity::Before,
        }),
    );
    capture_input(
        ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: direction,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![paragraph::ParagraphOrigin {
                source: source.clone(),
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: paragraph::ParagraphOriginKind::Authored,
                text_node: None,
            }],
            connections: vec![],
            boundary_policies: (0..source.boxes().len())
                .flat_map(|atomic| {
                    [
                        paragraph::ParagraphBoundaryDefinition::AtomicBefore(atomic),
                        paragraph::ParagraphBoundaryDefinition::AtomicAfter(atomic),
                    ]
                })
                .map(|definition| paragraph::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            controls: kinds
                .into_iter()
                .map(|(at, kind)| paragraph::ParagraphControl { at, kind })
                .collect(),
            anchor_order: anchors,
        },
        AHEM,
    )
}

// These fixtures affirmatively allocate each actual Start five pixels and all
// other endpoints zero; demanded facts are answered from that same fixture fact.
fn finish_at_edges(
    p: &PreparedText,
    mut request: LineSelectionRequest,
    direction: ParagraphBaseDirection,
) -> crate::Result<FinalizedLine> {
    request.context.base_direction = direction;
    request.context.line_box_direction = match direction {
        crate::ParagraphBaseDirection::LeftToRight => crate::LineBoxDirection::LeftToRight,
        crate::ParagraphBaseDirection::RightToLeft => crate::LineBoxDirection::RightToLeft,
    };
    let edge = |control: &PreparedControl| {
        crate::InlineBoundaryAllocation::try_new(
            if *control.view().kind() == K::InlineBoundaryStart {
                5.0
            } else {
                0.0
            },
        )
        .unwrap()
    };
    for control in &mut request.controls {
        if control.boundary.is_some() {
            control.boundary = Some(edge(&control.occurrence));
        }
    }
    while let Some(control) = p.boundary_context_demand(&request)? {
        request
            .boundary_context
            .push(crate::InlineBoundaryContextAllocation {
                boundary: edge(&control),
                occurrence: control,
            });
    }
    finish(p, request)
}

#[test]
fn incoming_uses_the_full_nested_receiver_level_and_preserves_prefix_controls() {
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let p = fixture(
            Source::new("AB\u{ad}CDE"),
            direction,
            vec![
                (0, K::RightToLeftEmbedding),
                (0, K::LeftToRightEmbedding),
                (3, K::InlineBoundaryStart),
                (3, K::InlineBoundaryStart),
            ],
        );
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish_at_edges(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
            direction,
        )
        .unwrap();
        let line = finish_at_edges(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                end(&p),
                first.hyphenation_continuation(),
                false,
            ),
            direction,
        )
        .unwrap();
        assert_eq!(line.feedback().advance, 50.0);
        let selected = &line.inner.selected;
        let units = &selected.presentation.units;
        let domain = &selected.presentation.domains[0];
        // LTR: RLE opens1 then LRE opens2; RTL: RLE opens3 then LRE opens4.
        // Latin C has even implicit level2 or4, while the local A stays L0.
        assert_eq!(
            domain.outer,
            if direction == ParagraphBaseDirection::LeftToRight {
                2
            } else {
                4
            }
        );
        assert_eq!((domain.base, units[domain.units.start].level), (0, 0));
        assert_eq!(domain.units, 2..3);
        let super::super::super::prepared::presentation::Attachment::After {
            participant,
            atom: Some(atom),
        } = domain.attachment
        else {
            panic!("actual nested original receiver required")
        };
        assert!(
            matches!(selected.prepared.tables.participants[participant], ParticipantData::Text(unit) if selected.prepared.tables.units[unit].atoms.contains(&atom))
        );
        assert_eq!(
            units
                .iter()
                .filter_map(|u| match u.kind {
                    U::Control { occurrence } => Some(occurrence),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [2, 3]
        );
        assert!(line.run_views().all(|run| run.face().bytes() == AHEM));
    }
}

#[test]
fn incoming_is_inserted_at_real_formatting_and_deleted_receivers() {
    for deleted in [false, true] {
        let p = fixture(
            Source::new(if deleted {
                "AB\u{ad}\u{ad}CDE"
            } else {
                "AB\u{ad}\u{200f}CDE"
            }),
            ParagraphBaseDirection::LeftToRight,
            vec![(3, K::InlineBoundaryStart)],
        );
        let offer = cut(&p, 3, counts(2, 3));
        let first = finish_at_edges(
            &p,
            selected_request(&p, start(&p), offer.clone(), None, true),
            ParagraphBaseDirection::LeftToRight,
        )
        .unwrap();
        let next = if deleted {
            cut(&p, 4, counts(2, 3))
        } else {
            end(&p)
        };
        let line = finish_at_edges(
            &p,
            selected_request(
                &p,
                offer.view().boundary(),
                next,
                first.hyphenation_continuation(),
                deleted,
            ),
            ParagraphBaseDirection::LeftToRight,
        )
        .unwrap();
        let selected = &line.inner.selected;
        let units = &selected.presentation.units;
        assert!(matches!(units[0].kind, U::Control { occurrence: 0 }));
        assert!(matches!(units[1].kind, U::Replacement { .. }));
        assert_eq!(units[1].source, Range::new(3, 3));
        assert_eq!(units[2].source, Range::new(3, 4));
        assert_eq!(units[2].kind, if deleted { U::Deleted } else { U::Text });
        assert_eq!(
            selected.presentation.domains[0].outer,
            if deleted { 0 } else { 1 }
        );
        assert_eq!(line.feedback().advance, if deleted { 15.0 } else { 45.0 });
        assert_eq!(
            p.analyzed_source().source().text().code_units()[3],
            if deleted { 0xad } else { 0x200f }
        );
    }
}

#[test]
fn incoming_missing_and_atomic_receivers_keep_precise_refusals() {
    let p = fixture(
        Source::new("AB\u{ad}"),
        ParagraphBaseDirection::LeftToRight,
        vec![(3, K::InlineBoundaryStart), (3, K::InlineBoundaryEnd)],
    );
    let offer = cut(&p, 3, counts(2, 0));
    let first = finish_at_edges(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
        ParagraphBaseDirection::LeftToRight,
    )
    .unwrap();
    let error = finish_at_edges(
        &p,
        selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        ),
        ParagraphBaseDirection::LeftToRight,
    )
    .unwrap_err();
    hyphenation_reason(&error, HyphenationError::MissingReceivingBidiAuthority);

    let mut source = Source::new("AB\u{ad}CDE");
    source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(91224),
        crate::InlineBoxKind::InFlow,
        3,
        crate::Size::new(10.0, 10.0),
    ));
    let p = fixture(
        source,
        ParagraphBaseDirection::LeftToRight,
        vec![(3, K::InlineBoundaryStart)],
    );
    // The real in-flow atomic terminates the preceding lexical word.
    let cut = p
        .hyphenation_cut_views()
        .find(|cut| {
            cut.provenance() == crate::HyphenationProvenance::Source
                && cut.boundary().view().source_before().get() == 3
        })
        .unwrap();
    assert_eq!(cut.counts(), Some(counts(2, 0)));
    let offer = cut.opportunity().unwrap();
    assert_eq!(offer.view().kind(), PreparedBreakKind::Allowed);
    assert_eq!(offer.view().alternatives().count(), 1);
    assert!(
        matches!(offer.view().alternatives().next().unwrap().definition(),crate::PreparedAlternativeDefinition::Source { source, .. } if source==Range::new(2,3))
    );
    let first = finish_at_edges(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
        ParagraphBaseDirection::LeftToRight,
    )
    .unwrap();
    let mut request = selected_request(
        &p,
        offer.view().boundary(),
        end(&p),
        first.hyphenation_continuation(),
        false,
    );
    request.atomics = request
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Atomic(occurrence) => Some(AtomicAllocation {
                occurrence: occurrence.clone(),
                allocation: AtomicLineAllocation::InFlow {
                    flow: AtomicFlowAllocation {
                        advance: 10.0,
                        before_baseline: 8.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 10.0,
                    },
                    box_geometry: AtomicBoxAllocation {
                        border: crate::Rect::new(0.0, 0.0, 10.0, 10.0),
                        content: crate::Rect::new(0.0, 0.0, 10.0, 10.0),
                        baseline: 8.0,
                    },
                    alignment: LineBoxAlignment::Baseline,
                },
            }),
            _ => None,
        })
        .collect();
    let error = finish_at_edges(&p, request, ParagraphBaseDirection::LeftToRight).unwrap_err();
    hyphenation_reason(&error, HyphenationError::UnsupportedReceivingRole);
}

#[test]
fn a_source_free_formatting_receiver_precedes_the_first_visible_original() {
    let direction = ParagraphBaseDirection::LeftToRight;
    let p = fixture(
        Source::new("AB\u{ad}CDE"),
        direction,
        vec![
            (3, K::InlineBoundaryStart),
            (3, K::RightToLeftEmbedding),
            (3, K::InlineBoundaryStart),
        ],
    );
    let offer = cut(&p, 3, counts(2, 3));
    let first = finish_at_edges(
        &p,
        selected_request(&p, start(&p), offer.clone(), None, true),
        direction,
    )
    .unwrap();
    let line = finish_at_edges(
        &p,
        selected_request(
            &p,
            offer.view().boundary(),
            end(&p),
            first.hyphenation_continuation(),
            false,
        ),
        direction,
    )
    .unwrap();
    assert_eq!(line.feedback().advance, 50.0);
    let selected = &line.inner.selected;
    let units = &selected.presentation.units;
    assert!(matches!(units[0].kind, U::Control { occurrence: 0 }));
    assert!(matches!(units[1].kind, U::Replacement { .. }));
    assert!(matches!(units[2].kind, U::Control { occurrence: 1 }));
    assert!(matches!(units[3].kind, U::Control { occurrence: 2 }));
    assert_eq!(units[4].source, Range::new(3, 4));
    let domain = &selected.presentation.domains[0];
    assert_eq!(
        (domain.units.clone(), domain.outer, domain.base),
        (1..2, 0, 0)
    );
    let super::super::super::prepared::presentation::Attachment::After {
        participant,
        atom: None,
    } = domain.attachment
    else {
        panic!("source-free RLE receiver required")
    };
    assert!(
        matches!(selected.prepared.tables.participants[participant],ParticipantData::Control(control) if selected.prepared.tables.controls[control].occurrence==1)
    );
}
