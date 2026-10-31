//! A real composition uses its text-containing Inline or derived root-inline.
//! Root/declaration fonts remain distinct available resources, not metric donors.
use super::*;

#[path = "combined_cell_sideways.rs"]
mod sideways;

#[derive(Clone, Copy)]
struct ContainerContext {
    typography: AnalysisTypography,
    direction: ParagraphBaseDirection,
    writing_axis: SourceWritingAxis,
    root_reference: ScopeFontReference,
    container_reference: Option<ScopeFontReference>,
    combination: TextCombineUpright,
    alignment: Option<ScopeAlignmentBoundary>,
}

impl ContainerContext {
    fn vertical() -> Self {
        Self {
            typography: AnalysisTypography::Mixed,
            direction: ParagraphBaseDirection::LeftToRight,
            writing_axis: SourceWritingAxis::Vertical,
            root_reference: font_reference(),
            container_reference: Some(font_reference()),
            combination: TextCombineUpright::All,
            alignment: None,
        }
    }
}

#[derive(Clone, Copy)]
enum ContainerShape {
    Inline,
    RootInline,
    NestedComputedAll,
}

struct ContainerFixture {
    system: System,
    processed: Arc<ProcessedSource>,
    analyzed: AnalyzedSource,
    body: Vec<u8>,
    container: usize,
    lexical_parent: Option<usize>,
}

fn fixture(shape: ContainerShape, text: &str, context: ContainerContext) -> ContainerFixture {
    assert_eq!(
        text.encode_utf16().count(),
        1,
        "each authored fixture has one real source unit"
    );
    let whitespace = if text == " " {
        WhiteSpace::Collapse
    } else {
        WhiteSpace::Preserve
    };
    let root_style = Style {
        font: Font::new().family("Ahem"),
        size: if matches!(shape, ContainerShape::NestedComputedAll) {
            30.0
        } else {
            20.0
        },
        line_height: LineHeight::Absolute(if matches!(shape, ContainerShape::NestedComputedAll) {
            36.0
        } else {
            24.0
        }),
        white_space: whitespace,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let body_style = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: context.combination,
        ..root_style.clone()
    };
    let root_style = if matches!(shape, ContainerShape::RootInline) {
        body_style.clone()
    } else {
        root_style
    };
    let mut scopes = vec![SourceProcessingScope::Block {
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: root_style,
        used_direction: context.direction,
        writing_axis: Some(context.writing_axis),
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
        font_reference: Some(context.root_reference),
    }];
    let inline = |style: Style, reference, alignment_boundary| SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(style),
        boundaries: None,
        alignment_boundary,
        font_reference: reference,
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
    };
    if matches!(shape, ContainerShape::NestedComputedAll) {
        // Captures take complete computed styles. This is the supplied result
        // of inheritance, not a new declaration/inheritance resolver: outer
        // All/font20 and nearer computed All/font10 have distinct actual owners.
        scopes.push(inline(
            Style {
                font: Font::new().family("Ahem"),
                size: 20.0,
                line_height: LineHeight::Absolute(24.0),
                text_combine_upright: context.combination,
                ..body_style.clone()
            },
            Some(context.root_reference),
            None,
        ));
    }
    let container = if matches!(shape, ContainerShape::RootInline) {
        0
    } else {
        scopes.len()
    };
    if container != 0 {
        scopes.push(inline(
            body_style.clone(),
            context.container_reference,
            context.alignment,
        ));
    }
    let lexical_parent = container.checked_sub(1);
    let mut source = Source::identified(text, Id::from_u64(784_927), 1);
    // The real original's complete computed style agrees with its actual body
    // container. An enclosing Inline.Style is not allowed to restyle it later.
    source.span(Range::new(0, 1), body_style);
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for scope in 1..scopes.len() {
        events.push(SourceProcessingEvent::EnterScope(scope));
    }
    events.push(SourceProcessingEvent::Origin {
        origin: 0,
        affinity: Affinity::After,
    });
    for scope in (1..scopes.len()).rev() {
        events.push(SourceProcessingEvent::ExitScope(scope));
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
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
            joint_flows: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes,
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: context.direction,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    );
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text));
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        context.typography,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let body = composition_font();
    glyph_frame_font_premises(&body);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    // Both real resources are available, including the Root's Ahem at its own
    // size. No fallback or font-substitution excuse can supply a missing owner.
    system.register_fonts(parent_font()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    ContainerFixture {
        system,
        processed,
        analyzed,
        body,
        container,
        lexical_parent,
    }
}

fn source_custody(fixture: &ContainerFixture, empty: bool) {
    let cells = fixture.analyzed.combined_cells().collect::<Vec<_>>();
    assert_eq!(cells.len(), 1);
    let cell = &cells[0];
    assert_eq!(
        cell.scope(),
        fixture.processed.captured_scope(fixture.container).unwrap()
    );
    assert_eq!(
        cell.parent_scope(),
        fixture
            .lexical_parent
            .map(|scope| fixture.processed.captured_scope(scope).unwrap())
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.selection_unit_count(), 1);
    assert_eq!(cell.used_unit_count(), usize::from(!empty));
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        if empty { vec![] } else { vec![0x41] }
    );
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        if empty { vec![] } else { vec![0x41] }
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    let piece = &pieces[0];
    assert_eq!(piece.source(), fixture.processed.source());
    assert_eq!(piece.local(), Range::new(0, 1));
    assert_eq!(piece.aggregate(), Range::new(0, 1));
    assert_eq!(
        piece.origin_position(),
        fixture.processed.captured_origin(0).unwrap()
    );
    assert_eq!(
        piece.paragraph(),
        fixture.processed.captured_paragraph(0).unwrap()
    );
    assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
    assert_eq!(piece.is_used(), !empty);
    let owner = fixture.analyzed.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [fixture.processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(
        owner.facts_owner(),
        fixture.processed.captured_paragraph(0).unwrap()
    );
    assert!(owner.joint_flow().is_none());
}

fn actual_container_cell(mut fixture: ContainerFixture, empty: bool) {
    source_custody(&fixture, empty);
    let prepared = fixture
        .system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(
        prepared.analyzed_source().identity(),
        fixture.analyzed.identity()
    );
    let container = fixture.processed.captured_scope(fixture.container).unwrap();
    let metrics = prepared.scope_metrics(&container).unwrap();
    assert_eq!(metrics.captured_scope(), container);
    assert_eq!(metrics.font_reference(), font_reference());
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(metrics.line_height(), 10.0);
    assert_eq!(metrics.orientation(), LineOrientation::Vertical);
    assert_eq!(
        metrics.common_baseline_reference(),
        Some(FontBaseline::Central)
    );
    // This real BASE has ideo0/idtp1200 at UPEM1000. At size10 its native
    // Central is6 and the signed sides are O6/U-6 (public under distance6).
    // m=(6+(-6))/2=0, so E10 has before5/after5, regardless of Root20/30.
    assert_eq!(metrics.native_reference_coordinate(), 6.0);
    assert_eq!((metrics.over(), metrics.under()), (6.0, 6.0));
    let instance = metrics.effective_instance().unwrap();
    assert_eq!(instance.font_size(), 10.0);
    assert_eq!(instance.face().bytes(), fixture.body.as_slice());
    assert_eq!(instance.axis_count(), 0);
    let line = finalize(&prepared);
    one_em(&line);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), fixture.processed.source());
    assert_eq!(
        layout.analyzed_source().identity(),
        fixture.analyzed.identity()
    );
    assert_eq!(layout.lines().len(), 1);
    // The independently supplied common baseline100 and inline origin20 give
    // x95..105/y20..30. Horizontal hhea800/-200 gives baseline8 within E10.
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    if empty {
        assert!(layout.run_views().all(|run| run.glyphs().next().is_none()));
        assert!(
            layout
                .glyph_runs()
                .into_iter()
                .all(|run| run.glyphs().is_empty())
        );
        assert_eq!(layout.source().text(), &Text::from(" "));
    } else {
        let runs = layout
            .run_views()
            .filter(|run| run.glyphs().next().is_some())
            .collect::<Vec<_>>();
        assert_eq!(runs.len(), 1);
        let run = &runs[0];
        assert_eq!(run.face().bytes(), fixture.body.as_slice());
        assert_eq!(run.range(), Range::new(0, 1));
        assert_eq!(run.orientation(), LineOrientation::Vertical);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        assert_eq!(run.glyph_frame().origin(), Point::new(95.0, 28.0));
        assert_eq!(run.glyph_frame().pen_direction(), Point::new(1.0, 0.0));
        assert_eq!(run.glyph_frame().outline_x(), Point::new(1.0, 0.0));
        assert_eq!(run.glyph_frame().outline_y(), Point::new(0.0, 1.0));
        assert_eq!(
            run.glyphs()
                .map(|glyph| (
                    glyph.id(),
                    glyph.pen(),
                    glyph.x(),
                    glyph.y(),
                    glyph.advance()
                ))
                .collect::<Vec<_>>(),
            [(35, 0.0, 0.0, 0.0, 10.0)]
        );
    }
    source_custody(&fixture, empty);
}

#[test]
fn inline10_container_does_not_borrow_root20_em_or_font_reference() {
    actual_container_cell(
        fixture(ContainerShape::Inline, "A", ContainerContext::vertical()),
        false,
    );
}

#[test]
fn direct_block_text_uses_its_real_root_inline_without_a_lexical_parent() {
    actual_container_cell(
        fixture(
            ContainerShape::RootInline,
            "A",
            ContainerContext::vertical(),
        ),
        false,
    );
}

#[test]
fn inherited_computed_all_uses_nearest_inline10_instead_of_outer_inline20() {
    actual_container_cell(
        fixture(
            ContainerShape::NestedComputedAll,
            "A",
            ContainerContext::vertical(),
        ),
        false,
    );
}

#[test]
fn missing_inline_container_reference_refuses_despite_real_root_reference() {
    let mut fixture = fixture(
        ContainerShape::Inline,
        "A",
        ContainerContext {
            container_reference: None,
            ..ContainerContext::vertical()
        },
    );
    source_custody(&fixture, false);
    assert_eq!(
        fixture
            .processed
            .captured_scope(0)
            .unwrap()
            .font_reference(),
        Some(font_reference())
    );
    assert_eq!(
        fixture
            .processed
            .captured_scope(1)
            .unwrap()
            .font_reference(),
        None
    );
    let error = match fixture.system.prepare(
        &fixture.analyzed,
        PreparationOptions::default().with_orientation(LineOrientation::Vertical),
    ) {
        Ok(_) => {
            panic!("an ancestor reference cannot supply the actual container's missing reference")
        }
        Err(error) => error,
    };
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    assert!(
        matches!(
            error.detail(),
            Some(ErrorDetail::ScopeAlignment {
                reason: ScopeAlignmentError::MissingFontReference { scope: 1 }
            })
        ),
        "{error:?}"
    );
    source_custody(&fixture, false);
}

#[test]
fn glyph_free_space_keeps_actual_inline10_metrics_and_original_custody_under_root20() {
    actual_container_cell(
        fixture(ContainerShape::Inline, " ", ContainerContext::vertical()),
        true,
    );
}

#[test]
fn asymmetric_alphabetic_reference_matches_once_after_central_cell_centering() {
    // Actual VerticalX raw coordinates: under0,over1400,romn200 at UPEM1000.
    // Central is their midpoint700. Alphabetic sides are asymmetric, while
    // Central-relative cell sides remain symmetric: this test invents no cell m.
    let table = base::base(
        0,
        None,
        Some(base::Axis {
            tags: vec![*b"ideo", *b"idtp", *b"romn"],
            scripts: vec![base::Script {
                tag: *b"latn",
                default_index: 2,
                coordinates: vec![
                    base::Coord::Design(0),
                    base::Coord::Design(1400),
                    base::Coord::Design(200),
                ],
            }],
        }),
        None,
    );
    let original_parent = parent_font();
    let original_body = composition_font();
    let parent = sfnt::replace_tables(&original_parent, vec![(*b"BASE", table.clone())]);
    let body = sfnt::replace_tables(&original_body, vec![(*b"BASE", table)]);
    for (actual, original) in [(&parent, &original_parent), (&body, &original_body)] {
        for tag in [
            *b"name", *b"cmap", *b"glyf", *b"loca", *b"hhea", *b"hmtx", *b"vhea", *b"vmtx",
            *b"maxp",
        ] {
            assert_eq!(sfnt::table(actual, tag), sfnt::table(original, tag));
        }
    }
    assert_eq!(sfnt::u16_at(sfnt::table(&parent, *b"hmtx"), 58 * 4), 1000);
    assert_eq!(sfnt::u16_at(sfnt::table(&body, *b"hmtx"), 35 * 4), 1000);
    let reference = ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::VerticalX,
        baseline: FontBaseline::Alphabetic,
    };
    let outside = Style {
        font: Font::new().family("Ahem"),
        size: 20.0,
        line_height: LineHeight::Absolute(24.0),
        white_space: WhiteSpace::Preserve,
        text_combine_upright: TextCombineUpright::None,
        text_transform: TextTransform::none(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let inside = Style {
        font: Font::new().family(crate::shaping_test_fonts::COMBINED_WIDTH_FAMILY),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        text_combine_upright: TextCombineUpright::All,
        ..outside.clone()
    };
    let mut source = Source::identified("XAX", Id::from_u64(784_951), 1);
    source.span(Range::new(1, 2), inside.clone());
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: (0..3)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..2)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            boundary_policies: (0..2)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                4096, 4096, 1_000_000, 16_777_216,
            )),
            scopes: vec![
                SourceProcessingScope::Block {
                    topology: SourceProcessingTopology::Complete,
                    trim: WhiteSpaceTrim::default(),
                    base_style: outside,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(reference),
                },
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: Some(inside),
                    boundaries: None,
                    alignment_boundary: Some(ScopeAlignmentBoundary::NonInitial),
                    font_reference: Some(reference),
                    horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::Connection(1),
                SourceProcessingEvent::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::Mixed,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    let mut cells = analyzed.combined_cells();
    let source_cell = cells.next().unwrap();
    assert!(cells.next().is_none());
    assert_eq!(source_cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(source_cell.selection_unit_count(), 1);
    assert_eq!(source_cell.used_code_points().collect::<Vec<_>>(), [0x41]);
    let pieces = source_cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].source(), &source);
    assert_eq!(
        (pieces[0].local(), pieces[0].aggregate()),
        (Range::new(1, 2), Range::new(1, 2))
    );
    assert_eq!(
        pieces[0].origin_position(),
        processed.captured_origin(1).unwrap()
    );
    assert_eq!(
        pieces[0].paragraph(),
        processed.captured_paragraph(0).unwrap()
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent.clone()).unwrap();
    system.register_fonts(body.clone()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    // Qualify real glyph-free font observations before inspecting any placed
    // cell/payload. These values come from the declared tables at20 and10.
    for (scope, size, native, alpha, distance, matching, bytes, alpha_over, alpha_under) in [
        (
            0,
            20.0,
            14.0,
            4.0,
            14.0,
            -10.0,
            parent.as_slice(),
            24.0,
            -4.0,
        ),
        (1, 10.0, 7.0, 2.0, 7.0, -5.0, body.as_slice(), 12.0, -2.0),
    ] {
        let metrics = prepared
            .scope_metrics(&processed.captured_scope(scope).unwrap())
            .unwrap();
        assert_eq!(
            metrics.captured_scope(),
            processed.captured_scope(scope).unwrap()
        );
        assert_eq!(metrics.font_reference(), reference);
        assert_eq!(metrics.font_size(), size);
        assert_eq!(
            metrics.common_baseline_reference(),
            Some(FontBaseline::Central)
        );
        assert_eq!(metrics.native_reference_coordinate(), native);
        assert_eq!((metrics.over(), metrics.under()), (distance, distance));
        assert_eq!(
            metrics.matching_coordinate(FontBaseline::Alphabetic),
            Some(matching)
        );
        assert_eq!(
            metrics.matching_coordinate(FontBaseline::Central),
            Some(0.0)
        );
        for (name, coordinate, provenance) in [
            (
                FontBaseline::Alphabetic,
                alpha,
                FontBaselineProvenance::BaseCoordinate,
            ),
            (
                FontBaseline::Central,
                native,
                FontBaselineProvenance::BaseEdgesCssMidpoint,
            ),
        ] {
            let Some(FontBaselineOutcome::Available(value)) = metrics.baseline_outcome(name) else {
                panic!("actual authored BASE name must be available")
            };
            assert_eq!(value.coordinate(), coordinate);
            assert_eq!(value.provenance(), provenance);
        }
        let instance = metrics.effective_instance().unwrap();
        assert_eq!(instance.face().bytes(), bytes);
        assert_eq!(instance.font_size(), size);
        let upright = instance
            .upright_metrics(instance.face().catalogue_identity(), reference.script)
            .unwrap();
        assert_eq!(upright.reference(), UprightMetricReference::FontDesignX);
        assert_eq!(
            upright.provenance(),
            UprightMetricProvenance::RawBaseline(FontBaselineProvenance::BaseEdgesCssMidpoint)
        );
        assert_eq!(upright.base_script(), Some(*b"latn"));
        assert_eq!(upright.em_under() - alpha, alpha_under);
        assert_eq!(upright.em_over() - alpha, alpha_over);
    }
    let mut prepared_cells = prepared.cell_views();
    let cell = prepared_cells.next().unwrap();
    assert!(prepared_cells.next().is_none());
    // Relative to actual native Central, m=7/2-7/2=0. Cell E10 therefore
    // has before5/after5; asymmetric Alphabetic is handled by scope matching.
    assert_eq!(
        (cell.extent(), cell.before(), cell.after()),
        (10.0, 5.0, 5.0)
    );
    let segment = prepared.source_segments().next().unwrap();
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
    let mut scopes = selected.scopes();
    let scope = scopes.next().unwrap();
    assert!(scopes.next().is_none());
    assert_eq!(scope.captured_scope(), processed.captured_scope(1).unwrap());
    let own = scope.metrics();
    let enclosing = scope.parent_metrics().unwrap();
    assert_eq!(own.captured_scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        enclosing.captured_scope(),
        processed.captured_scope(0).unwrap()
    );
    assert_eq!(
        own.matching_coordinate(FontBaseline::Alphabetic),
        Some(-5.0)
    );
    assert_eq!(
        enclosing.matching_coordinate(FontBaseline::Alphabetic),
        Some(-10.0)
    );
    // Genuine consuming protocol client: resolve the actual selected receipts.
    // The independent table equation is(4-14)-(2-7)=-10-(-5)=-5.
    let matched = BaselineShift::try_new(
        enclosing
            .matching_coordinate(FontBaseline::Alphabetic)
            .unwrap()
            - own.matching_coordinate(FontBaseline::Alphabetic).unwrap(),
    )
    .unwrap();
    assert_eq!(matched, BaselineShift::try_new(-5.0).unwrap());
    let finalize_at = |displacement| {
        selected
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                    scope: scope.clone(),
                    displacement,
                }]),
            })
            .unwrap()
    };
    let zero = finalize_at(BaselineShift::try_new(0.0).unwrap());
    let shifted = finalize_at(matched);
    assert_eq!(shifted.selected_line(), selected);
    assert_eq!(shifted.scope_allocations().len(), 1);
    let frame = &shifted.scope_allocations()[0];
    assert_eq!(frame.scope, processed.captured_scope(1).unwrap());
    assert_eq!(frame.displacement, matched);
    assert_eq!(frame.cumulative_displacement, matched);
    // This is Text's allocation protocol consuming a resolved client delta;
    // it does not stand in for Layout's separate alignment-policy calculation.
    for line in [&zero, &shifted] {
        let cells = line.cell_views().collect::<Vec<_>>();
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].square().origin.x, 20.0);
        assert_eq!(cells[0].square().size, Size::new(10.0, 10.0));
        assert_eq!(cells[0].square().origin.y, cells[0].reference() - 5.0);
    }
    let shifted_cells = shifted.cell_views().collect::<Vec<_>>();
    assert_eq!(shifted_cells.len(), 1);
    assert_eq!(shifted_cells[0].scope_displacement(), matched);
    assert_eq!(
        shifted_cells[0].source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    let zero_layout = placed_vertical_cell(&prepared, zero);
    let shifted_layout = placed_vertical_cell(&prepared, shifted);
    for (layout, x) in [(&zero_layout, 95.0), (&shifted_layout, 90.0)] {
        assert_eq!(layout.source(), &source);
        assert_eq!(layout.analyzed_source().identity(), analyzed.identity());
        // Root X20 supplies the prefix20; fixed anchor20 gives square y40 and
        // horizontal hhea8/-2 gives glyph y48. Positive-over -5 maps to X-5.
        let runs = layout
            .run_views()
            .filter(|run| run.glyphs().next().is_some() && run.range() == Range::new(1, 2))
            .collect::<Vec<_>>();
        assert_eq!(runs.len(), 1);
        let run = &runs[0];
        assert_eq!(run.face().bytes(), body.as_slice());
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        assert_eq!(run.glyph_frame().origin(), Point::new(x, 48.0));
        assert_eq!(run.glyph_frame().pen_direction(), Point::new(1.0, 0.0));
        assert_eq!(run.glyph_frame().outline_x(), Point::new(1.0, 0.0));
        assert_eq!(run.glyph_frame().outline_y(), Point::new(0.0, 1.0));
        assert_eq!(
            run.glyphs()
                .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>(),
            [(35, 0.0, 0.0, 0.0, 10.0)]
        );
    }
    let siblings = |layout: &Layout| {
        layout
            .run_views()
            .filter(|run| run.glyphs().next().is_some() && run.range() != Range::new(1, 2))
            .map(|run| {
                assert_eq!(run.face().bytes(), parent.as_slice());
                let raw = run
                    .glyphs()
                    .map(|g| (g.id(), g.pen(), g.x(), g.y(), g.advance()))
                    .collect::<Vec<_>>();
                assert_eq!(raw, [(58, 0.0, 0.0, 0.0, 20.0)]);
                (run.range(), run.glyph_frame())
            })
            .collect::<Vec<_>>()
    };
    let unchanged = siblings(&zero_layout);
    assert_eq!(
        unchanged.iter().map(|row| row.0).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(2, 3)]
    );
    assert_eq!(siblings(&shifted_layout), unchanged);
}
