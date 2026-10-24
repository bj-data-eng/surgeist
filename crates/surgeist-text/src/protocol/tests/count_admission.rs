//! Native counting uses the actual processed word and mixed-sequence custody.
use super::*;
use crate::*;

fn input(text: &str) -> ParagraphInput {
    let source = Source::new(text);
    let end = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: style(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, end),
            aggregate: Range::new(0, end),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn prepare(mut input: ParagraphInput) -> PreparedText {
    // This fixture authors a single genuine wrapping container. Connection
    // and in-flow atomic capture precedes analyzed missing-relation binding.
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
    input.boundary_policies = definitions
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn discretionary(p: &PreparedText) -> Vec<usize> {
    p.opportunity_views()
        .filter(|v| v.kind() == PreparedBreakKind::Discretionary)
        .map(|v| v.boundary().view().source_before().get())
        .collect()
}

#[test]
fn connected_origins_and_effective_instances_share_one_word_but_disconnections_do_not() {
    for connection in [
        ParagraphConnectionKind::Continuous,
        ParagraphConnectionKind::Disconnected,
    ] {
        let mut value = input("AX\u{ad}XXX");
        value.source.span(
            Range::new(3, 6),
            Style {
                size: 10.0,
                ..style()
            },
        );
        value.origins = vec![
            ParagraphOrigin {
                text_node: None,
                source: value.source.clone(),
                local: Range::new(0, 2),
                aggregate: Range::new(0, 2),
                kind: ParagraphOriginKind::Authored,
            },
            ParagraphOrigin {
                text_node: None,
                source: value.source.clone(),
                local: Range::new(2, 6),
                aggregate: Range::new(2, 6),
                kind: ParagraphOriginKind::Authored,
            },
        ];
        value.connections.push(ParagraphConnection {
            at: 2,
            kind: connection,
        });
        let p = prepare(value);
        assert_eq!(
            discretionary(&p),
            if connection == ParagraphConnectionKind::Disconnected {
                vec![]
            } else {
                vec![3]
            }
        );
        let whole = p
            .select(request(&p))
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(whole.feedback().advance, 70.0);
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            [65, 88, 173, 88, 88, 88]
        );
        assert!(whole.run_views().any(|r| r.font_size() == 10.0));
        assert!(whole.run_views().any(|r| r.font_size() == 20.0));
    }
}

#[test]
fn genuine_atomic_and_forced_barriers_cannot_supply_letters_to_a_discretionary_control() {
    for kind in [InlineBoxKind::OutOfFlow, InlineBoxKind::InFlow] {
        for control_first in [false, true] {
            let mut value = input("AXXXX");
            value.source.inline_box(InlineBox::new(
                Id::from_u64(6422),
                kind,
                2,
                Size::new(5.0, 10.0),
            ));
            value.controls.push(ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(style()),
                },
            });
            let mut items = vec![
                ParagraphAnchorItem::Atomic(0),
                ParagraphAnchorItem::Control(0),
            ];
            if control_first {
                items.reverse();
            }
            value.anchor_order = items
                .into_iter()
                .map(|item| ParagraphAnchor {
                    item,
                    affinity: Affinity::After,
                })
                .collect();
            let p = prepare(value);
            assert_eq!(
                discretionary(&p),
                if kind == InlineBoxKind::OutOfFlow {
                    vec![2]
                } else {
                    vec![]
                }
            );
            assert_eq!(p.atomic_views().count(), 1);
            assert_eq!(p.control_views().count(), 1);
            if kind == InlineBoxKind::OutOfFlow {
                let alt = p.alternative_views().next().unwrap();
                assert_eq!(
                    alt.definition(),
                    PreparedAlternativeDefinition::Control {
                        control: p.control_views().next().unwrap().handle(),
                        origin: None
                    }
                );
                let end = alt.opportunity().view().boundary();
                let actual: Vec<_> = p
                    .participants()
                    .take(end.view().participant_offset())
                    .collect();
                assert_eq!(
                    actual.last(),
                    Some(&PreparedParticipant::Control(
                        p.control_views().next().unwrap().handle()
                    ))
                );
            }
        }
    }
    for text in ["AX\u{ad}\nXXX", "AX\n\u{ad}XXX"] {
        let mut value = input(text);
        value.base_style.white_space = WhiteSpace::Preserve;
        let p = prepare(value);
        assert!(discretionary(&p).is_empty());
        assert!(
            p.opportunity_views()
                .any(|v| v.kind() == PreparedBreakKind::Forced)
        );
    }
}

#[test]
fn suppressed_native_points_do_not_resolve_unused_indication_resources_or_placement() {
    for text in ["A\u{ad}X", "AX\u{ad}XX"] {
        let mut value = input(text);
        value.base_style.native_hyphenation_convention = Some(
            NativeHyphenationConvention::declared_script(
                "Latn",
                Text::from("~"),
                HyphenationPlacement::BothSides,
            )
            .unwrap(),
        );
        value.base_style.locale = Some("und-Latn".into());
        let p = prepare(value);
        assert!(discretionary(&p).is_empty());
        assert_eq!(p.analyzed_source().source().text(), &Text::from(text));
        let whole = p
            .select(request(&p))
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            whole.feedback().advance,
            if text.starts_with("AX") { 80.0 } else { 40.0 }
        );
    }
    let mut value = input("AX");
    value.controls.push(ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(Style {
                font: Font::new().family("Missing indication primary"),
                ..style()
            }),
        },
    });
    value.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::After,
    });
    assert!(discretionary(&prepare(value)).is_empty());
}

#[test]
fn child_domain_hole_cannot_supply_or_remove_the_parent_words_counting_units() {
    use std::sync::Arc;
    for text in ["AXI\u{ad}XX", "AXI\u{ad}XXX"] {
        let mut source = Source::new(text);
        let end = source.text().len();
        source.inline_box(InlineBox::new(
            Id::from_u64(6423),
            InlineBoxKind::OutOfFlow,
            2,
            Size::new(5.0, 10.0),
        ));
        let origins = [Range::new(0, 2), Range::new(2, 3), Range::new(3, end)]
            .into_iter()
            .map(|range| ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
            })
            .collect();
        let capture = ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins,
            connections: vec![SourceProcessingConnection {
                before_origin: 0,
                after_origin: 2,
                kind: ParagraphConnectionKind::Continuous,
            }],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }],
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: crate::SourceProcessingTopology::Unknown,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    trim: WhiteSpaceTrim::default(),
                    base_style: style(),
                },
                SourceProcessingScope::OutOfFlowBlock {
                    topology: crate::SourceProcessingTopology::Unknown,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    trim: WhiteSpaceTrim::default(),
                    base_style: style(),
                    atomic: 0,
                },
            ],
            paragraphs: vec![
                SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: ParagraphStrut::DefaultFont
                };
                2
            ],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Atomic {
                    atomic: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::EnterParagraph(1),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(1),
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        };
        let processed = Arc::new(process_contextual_source(capture).unwrap());
        let parent =
            AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
                .unwrap();
        let p = system()
            .prepare(&parent, PreparationOptions::default())
            .unwrap();
        assert_eq!(
            discretionary(&p),
            if end == 7 { vec![4] } else { vec![] },
            "the child I cannot supply a missing fifth parent count"
        );
        if end == 7 {
            let alternative = p.alternative_views().next().unwrap();
            assert!(
                matches!(alternative.definition(),PreparedAlternativeDefinition::Source {origin,source} if origin==p.origin_views().nth(1).unwrap().handle() && source==Range::new(3,4))
            );
        }
        let mut r = request(&p);
        r.atomics = p
            .atomic_views()
            .map(|a| AtomicAllocation {
                occurrence: a.handle(),
                allocation: AtomicLineAllocation::OutOfFlow,
            })
            .collect();
        let whole = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(
            whole.feedback().advance,
            if end == 7 { 100.0 } else { 80.0 }
        );
        assert!(
            whole
                .run_views()
                .flat_map(|r| r.glyphs().map(|g| g.range()).collect::<Vec<_>>())
                .all(|range| !(range.start < 3 && range.end > 2))
        );
        assert_eq!(p.analyzed_source().source(), &source);
    }
}

#[test]
fn one_lexical_word_survives_a_real_registered_face_change() {
    let bytes = crate::shaping_test_fonts::ligature_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let mut name = font
        .table_data(skrifa::Tag::new(b"name"))
        .unwrap()
        .as_bytes()
        .to_vec();
    for i in 0..name.len().saturating_sub(7) {
        if &name[i..i + 8] == b"\0A\0h\0e\0m" {
            name[i + 1] = b'B';
        }
    }
    let secondary = crate::test_fonts::replace_tables(&bytes, vec![(*b"name", name)]);
    let mut value = input("AX\u{ad}XXX");
    value.source.span(
        Range::new(3, 6),
        Style {
            font: Font::new().family("Bhem"),
            ..style()
        },
    );
    value.origins[0].source = value.source.clone();
    let analyzed = AnalyzedSource::from_paragraph(value).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut owner = system();
    owner.register_fonts(secondary.clone()).unwrap();
    let p = owner
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(discretionary(&p), [3]);
    let whole = p
        .select(request(&p))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(whole.feedback().advance, 100.0);
    let actual: Vec<_> = whole
        .run_views()
        .map(|r| {
            assert!(
                r.effective_instance()
                    .unwrap()
                    .face()
                    .is_same_face(&r.face())
            );
            r.face().bytes() == secondary
        })
        .collect();
    assert!(actual.contains(&true));
    assert!(actual.contains(&false));
}
