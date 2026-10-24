#![forbid(unsafe_code)]

//! Each collapsed follower retains its own local gate and actual source custody.

use surgeist_text as text;

fn prepare(
    value: &str,
    pieces: &[(usize, usize, text::Wrap)],
    ancestor: text::Wrap,
    atomic_at: Option<usize>,
) -> text::PreparedText {
    let style = |wrap| text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Collapse,
        wrap,
        ..text::Style::default()
    };
    let mut source = text::Source::new(value);
    for &(start, end, wrap) in pieces {
        source.span(text::Range::new(start, end), style(wrap));
    }
    if let Some(at) = atomic_at {
        source.inline_box(text::InlineBox::new(
            text::Id::from_u64(6401),
            text::InlineBoxKind::InFlow,
            at,
            text::Size::new(10.0, 10.0),
        ));
    }
    let origins = pieces
        .iter()
        .map(|&(start, end, _)| text::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: text::Range::new(start, end),
            aggregate: text::Range::new(start, end),
            kind: text::ParagraphOriginKind::Authored,
        })
        .collect();
    let connections = pieces
        .windows(2)
        .map(|pair| text::ParagraphConnection {
            at: pair[0].1,
            kind: text::ParagraphConnectionKind::Continuous,
        })
        .collect();
    let mut boundary_policies: Vec<_> = pieces
        .windows(2)
        .enumerate()
        .map(|(index, pair)| text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: pair[0].2,
            following_wrap: pair[1].2,
            ancestor_wrap: ancestor,
        })
        .collect();
    let mut anchor_order = Vec::new();
    if atomic_at.is_some() {
        anchor_order.push(text::ParagraphAnchor {
            item: text::ParagraphAnchorItem::Atomic(0),
            affinity: text::Affinity::Before,
        });
        for definition in [
            text::ParagraphBoundaryDefinition::AtomicBefore(0),
            text::ParagraphBoundaryDefinition::AtomicAfter(0),
        ] {
            boundary_policies.push(text::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: pieces[pieces.len() - 1].2,
                following_wrap: pieces[pieces.len() - 1].2,
                ancestor_wrap: ancestor,
            });
        }
    }
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source,
        base_style: style(ancestor),
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins,
        connections,
        controls: vec![],
        boundary_policies,
        replacements: vec![],
        anchor_order,
    })
    .unwrap();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap()
}

#[test]
fn consecutive_collapsed_followers_keep_distinct_local_gates() {
    for ancestor in [text::Wrap::Word, text::Wrap::None] {
        for (wraps, expected) in [
            (
                [text::Wrap::None, text::Wrap::Word, text::Wrap::None],
                vec![3],
            ),
            (
                [text::Wrap::Word, text::Wrap::None, text::Wrap::Word],
                vec![2, 4],
            ),
        ] {
            let prepared = prepare(
                "A   B",
                &[(0, 2, wraps[0]), (2, 3, wraps[1]), (3, 5, wraps[2])],
                ancestor,
                None,
            );
            let cuts: Vec<_> = prepared
                .opportunity_views()
                .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                .map(|offer| offer.boundary().view().source_after().get())
                .collect();
            assert_eq!(cuts, expected, "ancestor={ancestor:?} wraps={wraps:?}");
        }
    }
}

#[test]
fn one_origin_collapsed_followers_keep_distinct_style_span_gates() {
    let mut observed = Vec::new();
    for ancestor in [text::Wrap::Word, text::Wrap::None] {
        for wraps in [
            [text::Wrap::None, text::Wrap::Word, text::Wrap::None],
            [text::Wrap::Word, text::Wrap::None, text::Wrap::Word],
        ] {
            let style = |wrap| text::Style {
                font: text::Font::new().family("Ahem"),
                size: 10.0,
                line_height: text::LineHeight::Absolute(10.0),
                white_space: text::WhiteSpace::Collapse,
                wrap,
                ..text::Style::default()
            };
            let mut source = text::Source::new("A   B");
            for (start, end, wrap) in [(0, 2, wraps[0]), (2, 3, wraps[1]), (3, 5, wraps[2])] {
                source.span(text::Range::new(start, end), style(wrap));
            }
            // Style spans are not native-origin seams. The one original owns
            // all five UTF-16 units and supplies no connection/NCA policy.
            let origin = text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 5),
                aggregate: text::Range::new(0, 5),
                kind: text::ParagraphOriginKind::Authored,
            };
            let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
                source,
                base_style: style(ancestor),
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
                terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![origin],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            })
            .unwrap();
            assert_eq!(analyzed.source().text(), &text::Text::from("A   B"));
            assert_eq!(
                analyzed.processed_source().processed_text(),
                &text::Text::from("A B")
            );
            let mut system = text::System::new(text::SystemOptions {
                font_catalogue: text::FontCatalogueMode::RegisteredOnly,
            })
            .unwrap();
            system
                .register_fonts(include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec())
                .unwrap();
            let prepared = system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap();
            let cuts: Vec<_> = prepared
                .opportunity_views()
                .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                .map(|offer| {
                    let boundary = offer.boundary();
                    assert_eq!(
                        boundary.view().source_before(),
                        boundary.view().source_after()
                    );
                    boundary.view().source_after().get()
                })
                .collect();
            observed.push(cuts);
        }
    }
    // SPACE originals [1,2), [2,3), [3,4) own cuts 2, 3, 4.
    // Only their local Word gates permit those cuts: the middle deleted SPACE
    // alone permits cut 3 in the first pattern; the outer two permit 2 and 4
    // in the second. The paragraph's base Wrap cannot donate local authority.
    assert_eq!(observed, [vec![3], vec![2, 4], vec![3], vec![2, 4]]);
}

#[test]
fn disappeared_local_space_gate_and_atomic_ancestor_permission_remain_independent() {
    let observed: Vec<_> = [
        (text::Wrap::None, text::Wrap::Word, text::Wrap::None),
        (text::Wrap::Word, text::Wrap::None, text::Wrap::None),
        (text::Wrap::None, text::Wrap::Word, text::Wrap::Word),
        (text::Wrap::Word, text::Wrap::None, text::Wrap::Word),
    ]
    .into_iter()
    .map(|(first, second, ancestor)| {
        let prepared = prepare("A  B", &[(0, 2, first), (2, 4, second)], ancestor, Some(3));
        let atomic = prepared.atomic_views().next().unwrap().handle();
        let cut = prepared
            .participants()
            .position(|participant| {
                participant == text::PreparedParticipant::Atomic(atomic.clone())
            })
            .unwrap();
        prepared.opportunity_views().any(|offer| {
            offer.kind() == text::PreparedBreakKind::Allowed
                && offer.boundary().view().participant_offset() == cut
        })
    })
    .collect();
    assert_eq!(observed, [true, false, true, true]);
}

fn finish(
    prepared: &text::PreparedText,
    start: text::PreparedBoundary,
    termination: text::PreparedOpportunity,
    continuation: bool,
) -> text::FinalizedLine {
    let end = termination.view().boundary();
    let participants = prepared
        .participants()
        .skip(start.view().participant_offset())
        .take(end.view().participant_offset() - start.view().participant_offset())
        .collect();
    let ending = if termination.view().kind() == text::PreparedBreakKind::ParagraphEnd {
        text::LineEnd::Paragraph
    } else {
        text::LineEnd::Soft
    };
    prepared
        .select(text::LineSelectionRequest {
            start,
            end,
            termination: text::PreparedTermination::Ordinary(termination),
            participants,
            incoming: None,
            alternative: None,
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: text::LineContext {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: if continuation {
                    text::LineStart::Continuation
                } else {
                    text::LineStart::Paragraph
                },
                end: ending,
                available_inline: Some(10.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::Spaces,
            },
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap()
}

#[test]
fn disappeared_follower_cut_finalizes_two_real_ten_pixel_lines_without_losing_source() {
    let prepared = prepare(
        "A  B",
        &[(0, 2, text::Wrap::None), (2, 4, text::Wrap::Word)],
        text::Wrap::None,
        None,
    );
    let first_end = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == text::PreparedBreakKind::Allowed
                && offer.boundary().view().source_after().get() == 3
        })
        .expect("the collapsed follower owns source cut3")
        .handle();
    let start = prepared.boundary_views().next().unwrap().handle();
    let continuation = first_end.view().boundary();
    let paragraph_end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let first = finish(&prepared, start, first_end, false);
    let second = finish(&prepared, continuation, paragraph_end, true);
    // Actual Ahem A/B each advance10px. Phase II removes the representative
    // trailing collapsed SPACE; its deleted follower owns no extra advance.
    for line in [&first, &second] {
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.measurements().fit().advance(), 10.0);
    }
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        lines: [first, second]
            .into_iter()
            .enumerate()
            .map(|(index, line)| text::LinePlacement {
                line,
                anchor: text::LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0 + index as f64 * 10.0,
                },
                orientation: text::LineOrientation::Horizontal,
                atomics: vec![],
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.lines().len(), 2);
    assert_eq!(layout.source().text(), &text::Text::from("A  B"));
    let before = layout
        .positions()
        .find(|position| position.source_cursor() == text::Cursor::new(0, text::Affinity::After))
        .unwrap();
    let after = layout
        .positions()
        .find(|position| position.source_cursor() == text::Cursor::new(4, text::Affinity::Before))
        .unwrap();
    let selected = layout.position_selection(&before, &after).unwrap();
    assert_eq!(selected.source_text(), &text::Text::from("A  B"));
    assert_eq!(selected.plaintext(), &text::Text::from("A B"));
}
