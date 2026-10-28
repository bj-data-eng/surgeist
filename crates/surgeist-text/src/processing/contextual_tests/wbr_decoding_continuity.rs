//! Wbr cuts original decoding and preserves every other connected context.
use super::*;

fn capture(value: impl Into<Text>, at: usize, mode: WhiteSpace) -> ContextualSourceInput {
    let mut p = input(value, mode);
    let end = p.source.text().len();
    p.origins = [Range::new(0, at), Range::new(at, end)]
        .into_iter()
        .map(|range| origin(&p.source, range))
        .collect();
    p.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    p.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    p.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    p
}

fn base(p: &mut ContextualSourceInput) -> &mut Style {
    let SourceProcessingScope::Block { base_style, .. } = &mut p.scopes[0] else {
        unreachable!()
    };
    base_style
}

fn add_control(p: &mut ContextualSourceInput, kind: ParagraphControlKind) {
    let control = p.controls.len();
    p.controls.push(ParagraphControl {
        at: p.origins[0].aggregate.end,
        kind,
    });
    let right = p
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::Origin { origin: 1, .. }))
        .unwrap();
    p.events.insert(
        right,
        SourceProcessingEvent::Control {
            control,
            affinity: Affinity::After,
        },
    );
}

fn add_wbr(p: &mut ContextualSourceInput, operation: WordSpaceTransform) {
    let mut own = style(WhiteSpace::Preserve, 10.0);
    own.word_space_transform = operation;
    add_control(
        p,
        ParagraphControlKind::Wbr {
            style: Box::new(own),
        },
    );
}

fn analyze(p: ContextualSourceInput) -> AnalyzedSource {
    let owner = Arc::new(process_contextual_source(p).unwrap());
    AnalyzedSource::from_processed_paragraph(
        owner.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap()
}

fn prepare_with(analyzed: &AnalyzedSource, font: Vec<u8>) -> PreparedText {
    let mut fonts = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(font).unwrap();
    fonts
        .prepare(
            &crate::test_wrapping::bind_missing(analyzed, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

fn prepare_ahem(analyzed: &AnalyzedSource) -> PreparedText {
    prepare_with(
        analyzed,
        include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
    )
}

fn whole(p: &PreparedText) -> FinalizedLine {
    let termination = p
        .opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let participants: Vec<_> = p.participants().collect();
    let controls = participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(control) => Some({
                let occurrence = control.clone();
                ControlAllocation {
                    boundary: matches!(
                        occurrence.view().kind(),
                        crate::ParagraphControlKind::InlineBoundaryStart
                            | crate::ParagraphControlKind::InlineBoundaryEnd
                    )
                    .then_some(crate::InlineBoundaryAllocation::ZERO),
                    occurrence,
                    strut: None,
                }
            }),
            _ => None,
        })
        .collect();
    p.select(LineSelectionRequest {
        boundary_context: Vec::new(),
        incoming: None,
        start: p.boundary_views().next().unwrap().handle(),
        end: termination.view().boundary(),
        termination: PreparedTermination::Ordinary(termination),
        alternative: None,
        participants,
        atomics: vec![],
        controls,
        line_box_struts: Vec::new(),
        line_strut: None,
        context: LineContext {
            alignment_insets: crate::LineAlignmentInsets::ZERO,
            formatted_line: crate::FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: crate::LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Horizontal,
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing: SpacingPolicy::Spaces,
        },
    })
    .unwrap()
    .finalize(SpacingAllocations::None)
    .unwrap()
}

fn direct(p: ContextualSourceInput) -> ParagraphInput {
    let SourceProcessingScope::Block { base_style, .. } = p.scopes[0].clone() else {
        unreachable!()
    };
    let at = p.origins[0].aggregate.end;
    ParagraphInput {
        source: p.source,
        base_style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: p.segment_breaks,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: p.origins,
        connections: vec![ParagraphConnection {
            at,
            kind: ParagraphConnectionKind::Continuous,
        }],
        controls: p.controls,
        replacements: vec![],
        boundary_policies: p.boundary_policies,
        anchor_order: p
            .events
            .iter()
            .filter_map(|event| match *event {
                SourceProcessingEvent::Control { control, affinity } => Some(ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(control),
                    affinity,
                }),
                _ => None,
            })
            .collect(),
    }
}

#[test]
fn positive_separators_and_generated_full_width_space_keep_original_units_opaque() {
    for (operation, full_width, scalar) in [
        (WordSpaceTransform::Space, false, 0x20),
        (WordSpaceTransform::IdeographicSpace, false, 0x3000),
        (WordSpaceTransform::Space, true, 0x3000),
    ] {
        let mut p = capture(
            Text::from_code_units(vec![0xd840, 0xdc00]),
            1,
            WhiteSpace::Preserve,
        );
        add_wbr(&mut p, operation);
        if full_width {
            let ParagraphControlKind::Wbr { style } = &mut p.controls[0].kind else {
                unreachable!()
            };
            style.text_transform =
                TextTransform::Set(TextTransformSet::none().with_full_width(true));
        }
        let a = analyze(p);
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            [0xd840, 0xdc00]
        );
        assert_eq!(
            a.inner
                .host_projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [0xd840, 0xdc00]
        );
        assert_eq!(
            a.inner
                .projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [0xd840, scalar, 0xdc00]
        );
        let prepared = prepare_ahem(&a);
        let line = whole(&prepared);
        assert_eq!(
            line.inner
                .selected
                .presentation
                .projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [0xd840, scalar, 0xdc00]
        );
        let generated: Vec<_> = line
            .mapping_views()
            .filter(|mapping| mapping.kind() == PresentationMappingKind::GeneratedSeparator)
            .collect();
        assert_eq!(generated.len(), 1);
        assert_eq!(
            generated[0].separator_control(),
            Some(prepared.control_views().next().unwrap().handle())
        );
        assert_eq!(a.source().text().code_units(), [0xd840, 0xdc00]);
    }
}

#[test]
fn forced_suppression_keeps_the_same_decode_cut_on_both_actual_sides() {
    for operation in [
        WordSpaceTransform::None,
        WordSpaceTransform::Space,
        WordSpaceTransform::IdeographicSpace,
    ] {
        for forced_first in [false, true] {
            let mut p = capture(
                Text::from_code_units(vec![0xd840, 0xdc00]),
                1,
                WhiteSpace::Preserve,
            );
            if forced_first {
                add_control(&mut p, ParagraphControlKind::ForcedBreak);
            }
            add_wbr(&mut p, operation);
            if !forced_first {
                add_control(&mut p, ParagraphControlKind::ForcedBreak);
            }
            let a = analyze(p);
            assert_eq!(
                a.inner
                    .host_projection
                    .atoms
                    .iter()
                    .map(|atom| atom.code_point)
                    .collect::<Vec<_>>(),
                [0xd840, 0xdc00]
            );
            assert_eq!(
                a.inner
                    .projection
                    .atoms
                    .iter()
                    .map(|atom| atom.code_point)
                    .collect::<Vec<_>>(),
                [0xd840, 0xdc00]
            );
            assert_eq!(
                a.graphemes().collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(1, 2)]
            );
            let prepared = prepare_ahem(&a);
            assert_eq!(prepared.control_views().count(), 2);
            assert_eq!(
                prepared
                    .unit_views()
                    .map(|unit| unit.source_range())
                    .collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(1, 2)]
            );
        }
    }
}

#[test]
fn equal_offset_wbrs_keep_distinct_generated_owners_and_opaque_original_units() {
    let mut p = capture(
        Text::from_code_units(vec![0xd840, 0xdc00]),
        1,
        WhiteSpace::Preserve,
    );
    add_wbr(&mut p, WordSpaceTransform::Space);
    add_wbr(&mut p, WordSpaceTransform::IdeographicSpace);
    let a = analyze(p);
    assert_eq!(
        a.inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd840, 0x20, 0x3000, 0xdc00]
    );
    let prepared = prepare_ahem(&a);
    let controls: Vec<_> = prepared
        .control_views()
        .map(|control| control.handle())
        .collect();
    assert_ne!(controls[0], controls[1]);
    let line = whole(&prepared);
    assert_eq!(
        line.mapping_views()
            .filter_map(|mapping| mapping.separator_control())
            .collect::<Vec<_>>(),
        controls
    );
    assert_eq!(a.source().text().code_units(), [0xd840, 0xdc00]);
}

#[test]
fn distinct_original_source_custody_survives_the_decode_cut_and_placement() {
    let high = Source::identified(Text::from_code_units(vec![0xd840]), Id::from_u64(630021), 7);
    let low = Source::identified(
        Text::from_code_units(vec![0xdc00]),
        Id::from_u64(630022),
        11,
    );
    let mut p = capture(
        Text::from_code_units(vec![0xd840, 0xdc00]),
        1,
        WhiteSpace::Preserve,
    );
    p.origins[0].source = high.clone();
    p.origins[0].local = Range::new(0, 1);
    p.origins[1].source = low.clone();
    p.origins[1].local = Range::new(0, 1);
    add_wbr(&mut p, WordSpaceTransform::None);
    let prepared = prepare_ahem(&analyze(p));
    let origins: Vec<_> = prepared
        .origin_views()
        .map(|origin| origin.handle())
        .collect();
    assert_eq!(origins[0].view().source(), &high);
    assert_eq!(origins[1].view().source(), &low);
    let output = placed(prepared);
    let left = output
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let right = output
        .source_range_fragments(&origins[1], Range::new(0, 1))
        .unwrap();
    assert!(
        !left
            .units()
            .next()
            .unwrap()
            .same_unit(&right.units().next().unwrap())
    );
    let positions: Vec<_> = output.positions().collect();
    let selected = output
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selected.source_text().code_units(), [0xd840, 0xdc00]);
    assert_eq!(
        selected
            .source_pieces()
            .iter()
            .map(|piece| piece.origin().view().source().id())
            .collect::<Vec<_>>(),
        [high.id(), low.id()]
    );
}

#[test]
fn casing_and_capitalization_keep_the_connected_word_across_none_wbr() {
    for (text, at, case, expected) in [
        ("ΟΣΑ", 2, CaseTransform::Lowercase, "οσα"),
        ("ΟΣ", 1, CaseTransform::Lowercase, "ος"),
        ("abcd", 2, CaseTransform::Capitalize, "Abcd"),
    ] {
        let mut p = capture(text, at, WhiteSpace::Preserve);
        base(&mut p).text_transform = TextTransform::Set(TextTransformSet::none().with_case(case));
        add_wbr(&mut p, WordSpaceTransform::None);
        assert_eq!(analyze(p).inner.projection.text, expected);
    }
}

#[test]
fn collapse_and_han_segment_break_context_cross_none_wbr() {
    for (text, at, expected) in [("A  B", 2, "A B"), ("漢\n漢", 1, "漢漢")] {
        let mut p = capture(text, at, WhiteSpace::Collapse);
        add_wbr(&mut p, WordSpaceTransform::None);
        let a = analyze(p);
        assert_eq!(a.processed_source().processed_text(), &Text::from(expected));
        assert_eq!(a.inner.projection.text, expected);
        assert_eq!(a.source().text(), &Text::from(text));
    }
}

#[test]
fn published_japanese_phrase_detection_crosses_a_genuine_none_wbr() {
    let mut p = capture("今日は天気です。", 3, WhiteSpace::Preserve);
    base(&mut p).locale = Some("ja".into());
    base(&mut p).word_space_transform = WordSpaceTransform::SpaceAutoPhrase;
    let SourceProcessingScope::Block { topology, .. } = &mut p.scopes[0] else {
        unreachable!()
    };
    *topology = SourceProcessingTopology::Complete;
    add_wbr(&mut p, WordSpaceTransform::None);
    let a = analyze(p);
    // Published upstream expectation: 今日は▁天気です。.
    assert_eq!(a.inner.projection.text, "今日は 天気です。");
    let prepared = prepare_ahem(&a);
    assert_eq!(prepared.virtual_separators().count(), 1);
    assert_eq!(
        prepared
            .virtual_separators()
            .next()
            .unwrap()
            .view()
            .source_position()
            .get(),
        3
    );
    assert_eq!(prepared.control_views().count(), 1);
}

#[test]
fn none_wbr_preserves_mixed_bidi_levels_and_real_ligature_shaping_context() {
    let mut bidi = capture("AאבB", 2, WhiteSpace::Preserve);
    add_wbr(&mut bidi, WordSpaceTransform::None);
    let prepared = prepare_ahem(&analyze(bidi));
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.level().get())
            .collect::<Vec<_>>(),
        [0, 1, 1, 0]
    );
    let mut ligature = capture("AA", 1, WhiteSpace::Preserve);
    add_wbr(&mut ligature, WordSpaceTransform::None);
    let prepared = prepare_with(
        &analyze(ligature),
        crate::shaping_test_fonts::ligature_ahem(),
    );
    let line = whole(&prepared);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|glyph| (glyph.id(), glyph.range(), glyph.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(58, Range::new(0, 2), 10.0)]
    );
}

#[test]
fn none_wbr_preserves_the_authored_grapheme_and_registered_mark_attachment() {
    let mut p = capture("A\u{301}", 1, WhiteSpace::Preserve);
    add_wbr(&mut p, WordSpaceTransform::None);
    let a = analyze(p);
    assert_eq!(a.graphemes().collect::<Vec<_>>(), [Range::new(0, 2)]);
    let prepared = prepare_with(&a, crate::shaping_test_fonts::mark_ahem());
    let line = whole(&prepared);
    let glyphs: Vec<_> = line
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.advance(),
                        glyph.pen() + f64::from(glyph.x()),
                        glyph.y(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(glyphs, [(35, 10.0, 0.0, 0.0), (100, 0.0, 3.0, -4.0)]);
}

#[test]
fn direct_paragraph_and_scalar_span_admission_use_only_genuine_wbr_decode_cuts() {
    for wbr in [false, true] {
        let mut p = capture(
            Text::from_code_units(vec![0xd840, 0xdc00, 0xa, 0x6f22]),
            1,
            WhiteSpace::Collapse,
        );
        if wbr {
            add_wbr(&mut p, WordSpaceTransform::None);
        }
        let mut p = direct(p);
        p.origins = vec![origin(&p.source, Range::new(0, 4))];
        p.connections.clear();
        p.boundary_policies.clear();
        let a = AnalyzedSource::from_paragraph(p, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
        let expected: &[u16] = if wbr {
            &[0xd840, 0xdc00, 0x20, 0x6f22]
        } else {
            &[0xd840, 0xdc00, 0x6f22]
        };
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            expected,
            "direct single-origin Wbr={wbr}"
        );
        assert_eq!(
            a.source().text().code_units(),
            [0xd840, 0xdc00, 0xa, 0x6f22]
        );
    }
    let mut unsplit = capture(
        Text::from_code_units(vec![0xd840, 0xdc00]),
        1,
        WhiteSpace::Preserve,
    );
    add_wbr(&mut unsplit, WordSpaceTransform::None);
    let mut unsplit = direct(unsplit);
    unsplit.origins = vec![origin(&unsplit.source, Range::new(0, 2))];
    unsplit.connections.clear();
    unsplit.boundary_policies.clear();
    let a =
        AnalyzedSource::from_paragraph(unsplit, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_eq!(
        a.inner
            .host_projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd840, 0xdc00]
    );
    let mut p = capture(
        Text::from_code_units(vec![0xd840, 0xdc00]),
        1,
        WhiteSpace::Preserve,
    );
    p.source
        .span(Range::new(1, 2), style(WhiteSpace::Preserve, 10.0));
    for origin in &mut p.origins {
        origin.source = p.source.clone();
    }
    let without_wbr = direct(p.clone());
    add_wbr(&mut p, WordSpaceTransform::None);
    let a =
        AnalyzedSource::from_paragraph(direct(p), crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_eq!(
        a.inner
            .host_projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd840, 0xdc00]
    );
    assert_eq!(prepare_ahem(&a).unit_views().count(), 2);
    let error = AnalyzedSource::from_paragraph(
        without_wbr,
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedShapingBoundary {
            index: 1,
            kind: ShapingBoundary::SpanStart
        })
    );
}

#[test]
fn an_empty_out_of_flow_child_wbr_cannot_split_parent_phase_one_decoding() {
    for (child_wbr, empty_origin) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut p = capture(
            Text::from_code_units(vec![0xd840, 0xdc00, 0xa, 0x6f22]),
            1,
            WhiteSpace::Collapse,
        );
        p.source.inline_box(InlineBox::new(
            Id::from_u64(630023),
            InlineBoxKind::OutOfFlow,
            1,
            Size::new(5.0, 10.0),
        ));
        for origin in &mut p.origins {
            origin.source = p.source.clone();
        }
        p.scopes.push(SourceProcessingScope::OutOfFlowBlock {
            topology: SourceProcessingTopology::Complete,
            trim: WhiteSpaceTrim::default(),
            base_style: style(WhiteSpace::Preserve, 10.0),
            used_direction: ParagraphBaseDirection::LeftToRight,
            atomic: 0,
        });
        p.paragraphs
            .push(facts(ParagraphBaseDirection::LeftToRight));
        p.events = vec![
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
        ];
        if empty_origin {
            p.origins.push(origin(&p.source, Range::new(1, 1)));
            p.events.push(SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            });
        }
        if child_wbr {
            p.controls.push(ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::Wbr {
                    style: Box::new(style(WhiteSpace::Preserve, 10.0)),
                },
            });
            p.events.push(SourceProcessingEvent::Control {
                control: 0,
                affinity: Affinity::After,
            });
        }
        p.events.extend([
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ]);
        let processed = Arc::new(process_contextual_source(p).unwrap());
        // The parent's uninterrupted U+20000/Han context deletes the LF. The
        // child's zero-source Wbr belongs to that child alone at the same index.
        assert_eq!(
            processed.processed_text().code_units(),
            [0xd840, 0xdc00, 0x6f22],
            "child Wbr={child_wbr}, genuine empty origin={empty_origin}"
        );
        assert_eq!(
            processed.source().text().code_units(),
            [0xd840, 0xdc00, 0xa, 0x6f22]
        );
        let paragraphs: Vec<_> = processed.paragraphs().collect();
        let parent = AnalyzedSource::from_processed_paragraph(
            paragraphs[0].clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            parent
                .inner
                .host_projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [0x20000, 0xa, 0x6f22]
        );
        assert_eq!(
            parent
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [0x20000, 0x6f22]
        );
        assert_eq!(prepare_ahem(&parent).control_views().count(), 0);
        assert!(paragraphs[1].source_range().is_empty());
    }
}
