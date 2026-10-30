//! Native preparation and one-em measurement from actual accepted source cells.
//! Font-backed normal input and source-proved empty input retain the same
//! fixed outer advance without fabricated glyph or parent-metric receipts.

use crate::*;
use crate::{baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical};
use std::sync::Arc;

fn font_reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::VerticalX,
        baseline: FontBaseline::Central,
    }
}

fn native_metric_font(horizontal: &[u8]) -> Vec<u8> {
    // Actual native vertical tables and Latn VerticalX BASE0..1200 at
    // UPEM1000. Central is600; this does not change horizontal font operands.
    let count = sfnt::u16_at(sfnt::table(horizontal, *b"maxp"), 4);
    let native = vertical::with_native(horizontal, 1200, 100, count);
    sfnt::replace_tables(
        &native,
        vec![(
            *b"BASE",
            base::base(
                0,
                None,
                Some(base::Axis {
                    tags: vec![*b"ideo", *b"idtp"],
                    scripts: vec![base::Script {
                        tag: *b"latn",
                        default_index: 0,
                        coordinates: vec![base::Coord::Design(0), base::Coord::Design(1200)],
                    }],
                }),
                None,
            ),
        )],
    )
}

fn parent_font() -> Vec<u8> {
    native_metric_font(&vertical::horizontal(800, -200, 0, 1000))
}

fn composition_font() -> Vec<u8> {
    let original = crate::shaping_test_fonts::combined_width_ahem();
    let native = native_metric_font(&original);
    // The composition's actual Inline font needs the named vertical reference.
    // Adding those real tables must preserve C's independently qualified family,
    // source mapping, GSUB actions, advances and licensed outline identities.
    for tag in [
        *b"name", *b"cmap", *b"GSUB", *b"hhea", *b"hmtx", *b"maxp", *b"glyf", *b"loca", *b"OS/2",
    ] {
        assert_eq!(sfnt::table(&native, tag), sfnt::table(&original, tag));
    }
    assert_eq!(sfnt::u16_at(sfnt::table(&native, *b"hmtx"), 35 * 4), 1000);
    native
}

fn setup(
    text: &str,
    whitespace: WhiteSpace,
) -> (System, Arc<ProcessedSource>, AnalyzedSource, Vec<u8>) {
    let parent_style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: whitespace,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let child_style = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        ..parent_style.clone()
    };
    let mut source = Source::identified(text, Id::from_u64(784_904), 74);
    // Actual body text must carry its own computed style; Inline.Style alone
    // supplies context and cannot restyle the authored source.
    source.span(Range::new(0, 1), child_style.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            // Authentic scope font-reference facts require immutable capture caps,
            // even with ZERO classified alignment targets. None is not unlimited.
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: parent_style,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(font_reference()),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(child_style),
                    boundaries: None,
                    alignment_boundary: None,
                    font_reference: Some(font_reference()),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                // The parent is a genuine font context, not an extra donated strut.
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            joint_flows: vec![],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
    let raw = processed.paragraphs().next().unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [Range::new(0, 1)]);
    let domain = AnalyzedDomain::from_processed_paragraph(raw, AnalysisTypography::Mixed).unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(domain.outputs().len(), 1);
    let output = domain.outputs().next().unwrap();
    assert_eq!(output.source(), &source);
    assert!(std::ptr::eq(output.processed_source(), processed.as_ref()));
    assert_eq!(
        output.paragraph_strut(),
        ParagraphStrut::AllocatedContributions
    );
    let owner = output.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(
        owner.facts_owner(),
        processed.captured_paragraph(0).unwrap()
    );
    assert!(owner.joint_flow().is_none());
    assert_eq!(output.combined_cells().len(), 1);
    let cell = output.combined_cells().next().unwrap();
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    // This is lexical parent metadata. The composition's metric owner is
    // the actual text-containing Inline1, with its own effective C-family font.
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    );
    assert_eq!(
        processed.captured_scope(1).unwrap().font_reference(),
        Some(font_reference())
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(cell.selection_unit_count(), 1);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].source(), &source);
    assert_eq!(pieces[0].local(), Range::new(0, 1));
    assert_eq!(pieces[0].aggregate(), Range::new(0, 1));
    assert_eq!(
        pieces[0].origin_position(),
        processed.captured_origin(0).unwrap()
    );
    assert_eq!(
        pieces[0].paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(pieces[0].kind(), ParagraphOriginKind::Authored);
    let analyzed = crate::test_wrapping::bind_missing(output, Wrap::Word);
    let body = composition_font();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent_font()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    (system, processed, analyzed, body)
}

fn finalize(prepared: &PreparedText) -> FinalizedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let selected = prepared
        .select(LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                alignment_insets: LineAlignmentInsets::ZERO,
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Vertical,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap();
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn one_em(line: &FinalizedLine) {
    assert!(line.has_in_flow_content());
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.measurements().fit().advance(), 10.0);
    assert_eq!(line.measurements().alignment().advance(), 10.0);
    assert_eq!(
        line.measurements().min_content_contribution().advance(),
        10.0
    );
    assert_eq!(
        line.measurements().max_content_contribution().advance(),
        10.0
    );
    assert_eq!(line.metric_bounds().size.width, 10.0);
}

#[test]
fn normal_fitting_combined_a_prepares_as_one_em_without_width_substitution() {
    let (mut system, processed, analyzed, body) = setup("A", WhiteSpace::Preserve);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.used_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    assert_eq!(cell.presentation_code_points().collect::<Vec<_>>(), [0x41]);
    assert!(cell.source_pieces().next().unwrap().is_used());
    assert_eq!(
        processed
            .plaintext_items(Affinity::After)
            .flat_map(|item| match item {
                PlaintextItem::Text(text) => text.code_units().to_vec(),
                PlaintextItem::InlineBox(_) => vec![],
            })
            .collect::<Vec<_>>(),
        [0x41]
    );
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), processed.source());
    let line = finalize(&prepared);
    one_em(&line);
    // Qualified font has UPEM1000/A35 advance1000. Size10 fits E10 normally:
    // no hwid/twid/qwid substitution (glyph36/58/16) or enlargement is needed.
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                let glyphs = run
                    .glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>();
                if !glyphs.is_empty() {
                    assert_eq!(run.face().bytes(), body.as_slice());
                }
                glyphs
            })
            .collect::<Vec<_>>(),
        [(35, 10.0)]
    );
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(processed.source().text(), &Text::from("A"));
}

#[test]
fn source_empty_combined_payload_still_measures_one_em_without_a_body_glyph() {
    let (mut system, processed, analyzed, _) = setup(" ", WhiteSpace::Collapse);
    let cell = analyzed.combined_cells().next().unwrap();
    assert_eq!(cell.used_unit_count(), 0);
    assert_eq!(cell.used_code_points().len(), 0);
    assert_eq!(cell.presentation_code_points().len(), 0);
    assert!(!cell.source_pieces().next().unwrap().is_used());
    assert_eq!(analyzed.inner.projection.atoms.len(), 0);
    assert_eq!(processed.source().text(), &Text::from(" "));
    assert_eq!(processed.processed_text(), &Text::from(" "));
    assert_eq!(
        processed
            .plaintext_items(Affinity::After)
            .flat_map(|item| match item {
                PlaintextItem::Text(text) => text.code_units().to_vec(),
                PlaintextItem::InlineBox(_) => vec![],
            })
            .collect::<Vec<_>>(),
        []
    );
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert_eq!(prepared.analyzed_source().source(), processed.source());
    let line = finalize(&prepared);
    one_em(&line);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().len())
            .sum::<usize>(),
        0
    );
    assert_eq!(line.prepared_text(), prepared);
    assert_eq!(
        analyzed
            .combined_cells()
            .next()
            .unwrap()
            .source_pieces()
            .next()
            .unwrap()
            .source(),
        processed.source()
    );
}
