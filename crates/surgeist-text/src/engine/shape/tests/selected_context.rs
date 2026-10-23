use crate::{
    Alignment, AnalyzedSource, ControlAllocation, DocumentSegmentBreaks, Font, FontCatalogueMode,
    LineContext, LineEnd, LineOrientation, LineSelectionRequest, LineStart, ParagraphBaseDirection,
    ParagraphInput, ParagraphOrigin, ParagraphOriginKind, ParagraphReplacement,
    ParagraphReplacementDefinition, ParagraphStrut, PreparationOptions, PreparedBreakKind,
    PreparedParticipant, PreparedTermination, Range, Source, SpacingAllocations, SpacingPolicy,
    Style, System, SystemOptions, TabContext, Wrap,
};

fn selected_computed_beh(separate_instance: bool) -> Vec<(Range, u32, f32)> {
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 20.0,
        ..Style::default()
    };
    let mut source = Source::new("\u{621}\u{628}\u{628}\u{ad}\u{628}\u{621}");
    if separate_instance {
        source.span(
            Range::new(0, 3),
            Style {
                size: 30.0,
                ..style.clone()
            },
        );
    }
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: style,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::RightToLeft,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            source,
            local: Range::new(0, 6),
            aggregate: Range::new(0, 6),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![
            ParagraphReplacement::resolved_native_logical_before(
                ParagraphReplacementDefinition::Source {
                    origin: 0,
                    source: Range::new(3, 4),
                },
                4,
                vec!["\u{628}".into()],
            )
            .unwrap(),
        ],
        anchor_order: vec![],
    })
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let fixture = crate::shaping_test_fonts::arabic_nonjoining_ahem();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture.clone()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let offer = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    let end = offer.boundary();
    let participants: Vec<_> = prepared
        .participants()
        .take(end.view().participant_offset())
        .collect();
    let controls = participants
        .iter()
        .filter_map(|part| match part {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    let request = LineSelectionRequest {
        start: prepared.boundary_views().next().unwrap().handle(),
        end,
        termination: PreparedTermination::Ordinary(offer.handle()),
        alternative: Some(offer.alternatives().next().unwrap().handle()),
        participants,
        controls,
        atomics: vec![],
        line_strut: None,
        context: LineContext {
            base_direction: ParagraphBaseDirection::RightToLeft,
            orientation: LineOrientation::Horizontal,
            start: LineStart::Paragraph,
            end: LineEnd::Soft,
            available_inline: Some(200.0),
            indent: 0.0,
            alignment: Alignment::Left,
            tabs: TabContext::NoTabs,

            spacing: SpacingPolicy::None,
        },
    };
    drop(system);
    let line = prepared
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), fixture);
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), run.font_size());
        glyphs.extend(
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), run.font_size())),
        );
    }
    glyphs.retain(|(range, _, _)| range.start != 0);
    glyphs.sort_by_key(|(range, _, _)| (range.start, range.end));
    // Original init/medi come from the unbroken source; the inserted final
    // comes from this selected line. The unselected following original Beh
    // must not turn the inserted Beh into medial277.
    glyphs
}

#[test]
fn computed_joining_uses_selected_line_without_unselected_following_source() {
    assert_eq!(
        selected_computed_beh(false),
        [
            (Range::new(1, 2), 100, 20.0),
            (Range::new(2, 3), 277, 20.0),
            (Range::new(4, 4), 82, 20.0),
        ]
    );
}

#[test]
fn computed_joining_crosses_actual_selected_line_instance_edge() {
    assert_eq!(
        selected_computed_beh(true),
        [
            (Range::new(1, 2), 100, 30.0),
            (Range::new(2, 3), 277, 30.0),
            (Range::new(4, 4), 82, 20.0),
        ]
    );
}
