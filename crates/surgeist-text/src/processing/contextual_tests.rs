//! CSS Text 4 §4.2 / Appendix B and exact original-piece domain oracles.
use crate::*;
use std::sync::Arc;

mod combination_facts;
mod joint_flow;
mod math;
mod phase_two;
mod source_connections;
mod wbr_connections;
mod wbr_decoding;
mod wbr_decoding_continuity;

#[test]
fn contextual_space_preservation_uses_emitted_token_facts_and_real_trim() {
    for (mode, expected, preserved) in [
        (WhiteSpace::Collapse, " A B ", vec![]),
        (
            WhiteSpace::Preserve,
            " A\tB\n ",
            vec![Range::new(0, 1), Range::new(5, 6)],
        ),
        (
            WhiteSpace::PreserveSpaces,
            " A B  ",
            vec![Range::new(0, 1), Range::new(2, 3), Range::new(4, 6)],
        ),
        (WhiteSpace::Discard, "AB", vec![]),
        (WhiteSpace::PreserveBreaks, " A B\n", vec![]),
        (
            WhiteSpace::BreakSpaces,
            " A\tB\n ",
            vec![Range::new(0, 1), Range::new(5, 6)],
        ),
    ] {
        let processed = process_contextual_source(input(" A\tB\n ", mode)).unwrap();
        assert_eq!(processed.processed_text(), &Text::from(expected));
        assert_eq!(processed.preserved_spaces, preserved);
        assert!(!processed.preserves_space_at(processed.processed_text().len()));
    }
    let mut capture = input("  A  ", WhiteSpace::Preserve);
    capture.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim {
            discard_inner: true,
            ..Default::default()
        },
    });
    capture
        .events
        .insert(2, SourceProcessingEvent::EnterScope(1));
    capture.events.insert(
        capture.events.len() - 2,
        SourceProcessingEvent::ExitScope(1),
    );
    let trimmed = process_contextual_source(capture).unwrap();
    assert_eq!(trimmed.processed_text(), &Text::from("A"));
    assert!(trimmed.preserved_spaces.is_empty());
}

#[test]
fn full_width_parent_projection_keeps_real_child_holes_and_each_domain_style() {
    for kind in [InlineBoxKind::OutOfFlow, InlineBoxKind::InFlow] {
        let mut capture = hole(kind);
        let SourceProcessingScope::Block { base_style, .. } = &mut capture.scopes[0] else {
            unreachable!()
        };
        base_style.text_transform =
            TextTransform::Set(crate::TextTransformSet::none().with_full_width(true));
        let result = Arc::new(process_contextual_source(capture).unwrap());
        let views: Vec<_> = result.paragraphs().collect();
        assert_eq!(
            views[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(6, 9)]
        );
        assert_eq!(
            views[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(3, 6)]
        );
        let parent = AnalyzedSource::from_processed_paragraph(
            views[0].clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let child = AnalyzedSource::from_processed_paragraph(
            views[1].clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            parent.inner.projection.text,
            if kind == InlineBoxKind::OutOfFlow {
                "Ａ Ｂ"
            } else {
                "Ａ  Ｂ"
            }
        );
        assert_eq!(child.inner.projection.text, " I ");
        let pieces = |analyzed: &AnalyzedSource| {
            (0..analyzed.inner.transform.rows.len())
                .flat_map(|row| {
                    analyzed
                        .inner
                        .transform
                        .original_pieces(row)
                        .iter()
                        .map(|piece| {
                            (
                                analyzed
                                    .inner
                                    .paragraph
                                    .origin_map
                                    .view_origin(piece.origin)
                                    .expect("original contribution belongs to this view")
                                    .get(),
                                piece.aggregate,
                                piece.local,
                            )
                        })
                })
                .collect::<Vec<_>>()
        };
        let mut expected = vec![
            (0, Range::new(0, 1), Range::new(0, 1)),
            (0, Range::new(1, 2), Range::new(1, 2)),
        ];
        if kind == InlineBoxKind::InFlow {
            expected.push((1, Range::new(6, 7), Range::new(6, 7)));
        }
        expected.push((1, Range::new(8, 9), Range::new(8, 9)));
        assert_eq!(pieces(&parent), expected);
        assert_eq!(
            pieces(&child),
            [
                (0, Range::new(3, 4), Range::new(3, 4)),
                (0, Range::new(4, 5), Range::new(4, 5)),
                (0, Range::new(5, 6), Range::new(5, 6))
            ]
        );
        assert_eq!(
            text(views[0].plaintext_items(Affinity::Before)),
            if kind == InlineBoxKind::OutOfFlow {
                "A B"
            } else {
                "A  B"
            }
            .encode_utf16()
            .collect::<Vec<_>>()
        );
        assert_eq!(
            text(views[1].plaintext_items(Affinity::Before)),
            " I ".encode_utf16().collect::<Vec<_>>()
        );
    }
}

fn style(mode: WhiteSpace, size: f32) -> Style {
    Style {
        white_space: mode,
        font: Font::new().family("Ahem"),
        size,
        line_height: LineHeight::Absolute(f64::from(size)),
        wrap: Wrap::Word,
        ..Style::default()
    }
}
fn facts(direction: ParagraphBaseDirection) -> SourceProcessingParagraph {
    SourceProcessingParagraph {
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
    }
}
fn origin(source: &Source, range: Range) -> ParagraphOrigin {
    ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: range,
        aggregate: range,
        kind: ParagraphOriginKind::Authored,
    }
}
fn input(text: impl Into<Text>, mode: WhiteSpace) -> ContextualSourceInput {
    let source = Source::new(text);
    let len = source.text().len();
    ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: vec![origin(&source, Range::new(0, len))],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            used_direction: crate::ParagraphBaseDirection::LeftToRight,
            trim: WhiteSpaceTrim::default(),
            base_style: style(mode, 10.0),
        }],
        paragraphs: vec![facts(ParagraphBaseDirection::LeftToRight)],
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    }
}
fn text<'a>(items: impl Iterator<Item = PlaintextItem<'a>>) -> Vec<u16> {
    items
        .flat_map(|item| match item {
            PlaintextItem::Text(text) => text.code_units().to_vec(),
            PlaintextItem::InlineBox(_) => vec![],
        })
        .collect()
}
fn policy(definition: ParagraphBoundaryDefinition) -> ParagraphBoundaryPolicy {
    ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }
}
fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    system
}
fn prepare(system: &mut System, view: ProcessedParagraph) -> PreparedText {
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_processed_paragraph(
                    view,
                    crate::AnalysisTypography::HorizontalComposition,
                )
                .unwrap(),
                Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn line(prepared: &PreparedText) -> FinalizedLine {
    let termination = prepared
        .opportunity_views()
        .find(|row| row.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let atomics = prepared
        .atomic_views()
        .map(|atomic| AtomicAllocation {
            occurrence: atomic.handle(),
            allocation: match atomic.original_box().kind() {
                InlineBoxKind::OutOfFlow => AtomicLineAllocation::OutOfFlow,
                InlineBoxKind::InFlow => AtomicLineAllocation::InFlow {
                    flow: AtomicFlowAllocation {
                        advance: 5.0,
                        before_baseline: 8.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 5.0,
                    },
                    box_geometry: AtomicBoxAllocation {
                        border: Rect::new(0.0, -8.0, 5.0, 10.0),
                        content: Rect::new(0.0, -8.0, 5.0, 10.0),
                        baseline: 0.0,
                    },
                    alignment: LineBoxAlignment::Baseline,
                },
            },
        })
        .collect();
    let controls = prepared
        .control_views()
        .map(|row| {
            let occurrence = row.handle();
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
        })
        .collect();
    let direction = match prepared.analyzed_source().direction() {
        Direction::LeftToRight => ParagraphBaseDirection::LeftToRight,
        Direction::RightToLeft => ParagraphBaseDirection::RightToLeft,
        Direction::Auto => panic!("analysis resolves direction"),
    };
    let selected = prepared
        .select(LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics,
            controls,
            line_box_struts: Vec::new(),
            line_strut: None,
            context: LineContext {
                alignment_insets: crate::LineAlignmentInsets::ZERO,
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: direction,
                line_box_direction: match direction {
                    crate::ParagraphBaseDirection::LeftToRight => {
                        crate::LineBoxDirection::LeftToRight
                    }
                    crate::ParagraphBaseDirection::RightToLeft => {
                        crate::LineBoxDirection::RightToLeft
                    }
                },
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
        .unwrap();
    selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn placed(prepared: PreparedText) -> Layout {
    let line = line(&prepared);
    let atomics = line
        .atomic_slots()
        .map(|atomic| {
            let translate = |rect: Rect| {
                Rect::new(
                    atomic.origin.x + rect.origin.x,
                    20.0 + atomic.origin.y - line.feedback().baseline + rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                )
            };
            match atomic.allocation {
                AtomicLineAllocation::OutOfFlow => AtomicPlacement {
                    occurrence: atomic.occurrence,
                    border: Rect::new(0.0, 0.0, 5.0, 10.0),
                    content: Rect::new(0.0, 0.0, 5.0, 10.0),
                    baseline: 8.0,
                },
                AtomicLineAllocation::InFlow { box_geometry, .. } => AtomicPlacement {
                    occurrence: atomic.occurrence,
                    border: translate(box_geometry.border),
                    content: translate(box_geometry.content),
                    baseline: 20.0 + atomic.origin.y - line.feedback().baseline
                        + box_geometry.baseline,
                },
            }
        })
        .collect();
    Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 20.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics,
        })],
    })
    .unwrap()
}

#[test]
fn block_inner_trim_keeps_indentation_while_inline_inner_trim_removes_it() {
    for (block, expected) in [(true, "  X"), (false, "X")] {
        let mut capture = input("\n  X\n\n  ", WhiteSpace::Preserve);
        if block {
            capture.scopes[0] = SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: SourceProcessingTopology::Unknown,
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim {
                    discard_inner: true,
                    ..Default::default()
                },
                base_style: style(WhiteSpace::Preserve, 10.0),
            };
        } else {
            capture.scopes.push(SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                style: None,
                boundaries: None,
                trim: WhiteSpaceTrim {
                    discard_inner: true,
                    ..Default::default()
                },
            });
            capture
                .events
                .insert(2, SourceProcessingEvent::EnterScope(1));
            capture
                .events
                .insert(4, SourceProcessingEvent::ExitScope(1));
        }
        let result = Arc::new(process_contextual_source(capture).unwrap());
        assert_eq!(
            result.processed_text().code_units(),
            expected.encode_utf16().collect::<Vec<_>>()
        );
        let view = result.paragraphs().next().unwrap();
        assert_eq!(
            text(view.plaintext_items(Affinity::Before)),
            expected.encode_utf16().collect::<Vec<_>>()
        );
        let prepared = prepare(&mut system(), view);
        let layout = placed(prepared);
        assert_eq!(
            layout.source().text().code_units(),
            "\n  X\n\n  ".encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn exterior_inline_trim_removes_parent_origin_spaces_with_original_affinities() {
    for (value, mode) in [
        ("A  B  C", WhiteSpace::Collapse),
        ("A \nB\n C", WhiteSpace::PreserveBreaks),
    ] {
        check_exterior_inline_trim(value, mode);
    }
}

fn check_exterior_inline_trim(value: &str, mode: WhiteSpace) {
    let mut capture = input(value, mode);
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 3)),
        origin(&capture.source, Range::new(3, 4)),
        origin(&capture.source, Range::new(4, 7)),
    ];
    capture.connections = (0..2)
        .map(|i| SourceProcessingConnection {
            before_origin: i,
            after_origin: i + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    capture.boundary_policies = (0..2)
        .map(|i| policy(ParagraphBoundaryDefinition::Connection(i)))
        .collect();
    capture.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim {
            discard_before: true,
            discard_after: true,
            ..Default::default()
        },
    });
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Connection(0),
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
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    assert_eq!(result.processed_text().code_units(), [65, 66, 67]);
    for at in [1, 2, 3] {
        for affinity in [Affinity::Before, Affinity::After] {
            assert_eq!(
                result
                    .source_to_processed(
                        SourcePosition::try_new(result.source().text(), at).unwrap(),
                        affinity
                    )
                    .unwrap()
                    .get(),
                1
            );
        }
    }
    assert_eq!(
        result
            .processed_to_source(result.position(1).unwrap(), Affinity::Before)
            .unwrap()
            .get(),
        1
    );
    assert_eq!(
        result
            .processed_to_source(result.position(1).unwrap(), Affinity::After)
            .unwrap()
            .get(),
        3
    );
    let layout = placed(prepare(&mut system(), result.paragraphs().next().unwrap()));
    assert_eq!(
        layout
            .glyph_runs()
            .iter()
            .map(|run| run.glyphs().len())
            .sum::<usize>(),
        3
    );
}

fn hole(kind: InlineBoxKind) -> ContextualSourceInput {
    let mut capture = input("A   I   B", WhiteSpace::Collapse);
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(7),
        kind,
        3,
        crate::Size::new(5.0, 10.0),
    ));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 3)),
        origin(&capture.source, Range::new(3, 6)),
        origin(&capture.source, Range::new(6, 9)),
    ];
    capture.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 2,
        kind: ParagraphConnectionKind::Continuous,
    }];
    capture.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    if kind == InlineBoxKind::InFlow {
        capture.boundary_policies.extend([
            policy(ParagraphBoundaryDefinition::AtomicBefore(0)),
            policy(ParagraphBoundaryDefinition::AtomicAfter(0)),
        ]);
    }
    capture.scopes.push(if kind == InlineBoxKind::InFlow {
        SourceProcessingScope::AtomicBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            used_direction: crate::ParagraphBaseDirection::LeftToRight,
            trim: WhiteSpaceTrim::default(),
            base_style: style(WhiteSpace::Preserve, 20.0),
            atomic: 0,
        }
    } else {
        SourceProcessingScope::OutOfFlowBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            used_direction: crate::ParagraphBaseDirection::LeftToRight,
            trim: WhiteSpaceTrim::default(),
            base_style: style(WhiteSpace::Preserve, 20.0),
            atomic: 0,
        }
    });
    capture
        .paragraphs
        .push(facts(ParagraphBaseDirection::RightToLeft));
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::Before,
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
    ];
    capture
}

#[test]
fn issued_parent_views_exclude_internal_domains_and_keep_actual_parent_atomics() {
    for kind in [InlineBoxKind::OutOfFlow, InlineBoxKind::InFlow] {
        let result = Arc::new(process_contextual_source(hole(kind)).unwrap());
        let views: Vec<_> = result.paragraphs().collect();
        assert_eq!(
            views[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(6, 9)]
        );
        assert_eq!(
            text(views[0].plaintext_items(Affinity::Before)),
            if kind == InlineBoxKind::OutOfFlow {
                "A B"
            } else {
                "A  B"
            }
            .encode_utf16()
            .collect::<Vec<_>>()
        );
        assert_eq!(
            text(views[1].plaintext_items(Affinity::Before)),
            " I ".encode_utf16().collect::<Vec<_>>()
        );
        let parent = AnalyzedSource::from_processed_paragraph(
            views[0].clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let child = AnalyzedSource::from_processed_paragraph(
            views[1].clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(parent.style().size, 10.0);
        assert_eq!(child.style().size, 20.0);
        assert_eq!(child.direction(), Direction::RightToLeft);
        assert_eq!(
            parent.graphemes().collect::<Vec<_>>(),
            [
                Range::new(0, 1),
                Range::new(1, 2),
                Range::new(2, 3),
                Range::new(6, 7),
                Range::new(7, 8),
                Range::new(8, 9)
            ]
        );
        let mut fonts = system();
        let parent = crate::test_wrapping::bind_missing(&parent, Wrap::Word);
        let child = crate::test_wrapping::bind_missing(&child, Wrap::Word);
        let pp = fonts
            .prepare(&parent, PreparationOptions::default())
            .unwrap();
        let cp = fonts
            .prepare(&child, PreparationOptions::default())
            .unwrap();
        assert_eq!(pp.atomic_views().count(), 1);
        assert_eq!(cp.atomic_views().count(), 0);
        assert!(
            pp.unit_views()
                .all(|row| row.source_range().end <= 3 || row.source_range().start >= 6)
        );
        let parent_layout = placed(pp);
        let child_layout = placed(cp);
        assert!(
            parent_layout
                .cursor(Cursor::new(4, Affinity::After))
                .is_err()
        );
        assert!(
            parent_layout
                .selection(Selection::new(
                    Cursor::new(0, Affinity::After),
                    Cursor::new(9, Affinity::Before)
                ))
                .is_err()
        );
        assert!(
            parent_layout
                .apply(Edit::Delete {
                    range: Range::new(2, 7)
                })
                .is_err()
        );
        assert!(
            child_layout
                .cursor(Cursor::new(0, Affinity::After))
                .is_err()
        );
        assert_eq!(child_layout.lines()[0].range(), Range::new(3, 6));
        let other = Arc::new(process_contextual_source(hole(kind)).unwrap());
        assert!(other.admit_paragraph(&views[0]).is_err());
        drop(other);
        drop(result);
        assert_eq!(
            text(views[0].plaintext_items(Affinity::Before)).first(),
            Some(&65)
        );
    }
}

#[test]
fn plaintext_uses_lexical_eligibility_and_only_real_line_context_resets() {
    for (source, mode, expected) in [
        ("\tX", WhiteSpace::PreserveSpaces, " X"),
        (" \n X", WhiteSpace::Collapse, "X"),
        (" \n X", WhiteSpace::Preserve, " \n X"),
        (" \n \n X", WhiteSpace::PreserveBreaks, "\n\nX"),
        (" \n \n X", WhiteSpace::BreakSpaces, " \n \n X"),
    ] {
        let result = process_contextual_source(input(source, mode)).unwrap();
        assert_eq!(
            text(result.plaintext_items(Affinity::Before)),
            expected.encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn trim_does_not_fuse_removed_original_surrogate_neighbors() {
    let mut capture = input(
        Text::from_code_units(vec![0xd800, 32, 0xdc00]),
        WhiteSpace::Preserve,
    );
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
        origin(&capture.source, Range::new(2, 3)),
    ];
    capture.connections = (0..2)
        .map(|i| SourceProcessingConnection {
            before_origin: i,
            after_origin: i + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    capture.boundary_policies = (0..2)
        .map(|i| policy(ParagraphBoundaryDefinition::Connection(i)))
        .collect();
    capture.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim {
            discard_inner: true,
            ..Default::default()
        },
    });
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Connection(0),
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
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    assert_eq!(result.processed_text().code_units(), [0xd800, 0xdc00]);
    let analyzed = AnalyzedSource::from_processed_paragraph(
        result.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd800, 0xdc00]
    );
    let pair = Arc::new(
        process_contextual_source(input(
            Text::from_code_units(vec![0xd800, 0xdc00]),
            WhiteSpace::Preserve,
        ))
        .unwrap(),
    );
    let analyzed = AnalyzedSource::from_processed_paragraph(
        pair.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.atoms[0].source, Range::new(0, 2));
    assert_eq!(analyzed.inner.projection.atoms[0].code_point, 0x10000);
}

#[test]
fn invalid_capture_is_rejected_without_issuing_partial_views() {
    let mut capture = input("X", WhiteSpace::Collapse);
    capture.envelope = SourceProcessingEnvelope::IncompleteExterior;
    assert!(process_contextual_source(capture).is_err());
    let mut capture = input("X", WhiteSpace::Collapse);
    capture.events.insert(
        3,
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    );
    assert!(process_contextual_source(capture).is_err());
    let mut capture = input("X", WhiteSpace::Collapse);
    capture.origins[0].source = Source::new("Y");
    assert!(process_contextual_source(capture).is_err());
    let mut capture = input("X", WhiteSpace::Collapse);
    capture.events.swap(3, 4);
    assert!(process_contextual_source(capture).is_err());
}

#[test]
fn forced_control_removes_only_plaintext_leading_space_without_adding_a_separator() {
    let mut capture = input("A  B", WhiteSpace::Collapse);
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 4)),
    ];
    capture.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    capture.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    capture.controls = vec![ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::ForcedBreak,
    }];
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    assert_eq!(result.processed_text().code_units(), [65, 32, 66]);
    assert_eq!(text(result.plaintext_items(Affinity::Before)), [65, 66]);
    let collapsed = process_contextual_source(input("A\n B", WhiteSpace::Collapse)).unwrap();
    assert_eq!(
        text(collapsed.plaintext_items(Affinity::Before)),
        [65, 32, 66]
    );
    let preserved = process_contextual_source(input("A\n B", WhiteSpace::Preserve)).unwrap();
    assert_eq!(
        text(preserved.plaintext_items(Affinity::Before)),
        [65, 10, 32, 66]
    );
}

#[test]
fn same_offset_nested_empty_scopes_and_consecutive_controls_retain_actual_order_and_strut() {
    let mut capture = input("", WhiteSpace::Preserve);
    capture.paragraphs[0] = facts(ParagraphBaseDirection::RightToLeft);
    capture.scopes.extend([
        SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            style: None,
            boundaries: None,
            trim: WhiteSpaceTrim {
                discard_inner: true,
                ..Default::default()
            },
        },
        SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            style: None,
            boundaries: None,
            trim: WhiteSpaceTrim {
                discard_before: true,
                discard_after: true,
                ..Default::default()
            },
        },
    ]);
    capture.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::LeftToRightIsolate,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::PopDirectionalIsolate,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
        },
    ];
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterScope(2),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(2),
        SourceProcessingEvent::Control {
            control: 1,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Control {
            control: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let result = Arc::new(process_contextual_source(capture.clone()).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        result.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.direction(), Direction::RightToLeft);
    let prepared = system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        prepared
            .control_views()
            .map(|row| row.kind().clone())
            .collect::<Vec<_>>(),
        capture
            .controls
            .iter()
            .map(|row| row.kind.clone())
            .collect::<Vec<_>>()
    );
    let finalized = line(&prepared);
    assert_eq!(finalized.feedback().line_height, 10.0);
    assert_eq!(finalized.feedback().advance, 0.0);
    let layout = placed(prepared);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 0));
    assert_eq!(text(result.plaintext_items(Affinity::Before)), []);
    capture.events.insert(
        9,
        SourceProcessingEvent::Control {
            control: 1,
            affinity: Affinity::After,
        },
    );
    assert!(matches!(
        process_contextual_source(capture).unwrap_err().detail(),
        Some(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::Occurrence
        })
    ));
}

#[test]
fn empty_internal_domain_keeps_its_captured_font_and_direction() {
    let mut capture = input("", WhiteSpace::Collapse);
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(4),
        InlineBoxKind::OutOfFlow,
        0,
        Size::new(5.0, 10.0),
    ));
    capture
        .origins
        .push(origin(&capture.source, Range::new(0, 0)));
    capture.scopes.push(SourceProcessingScope::OutOfFlowBlock {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: SourceProcessingTopology::Unknown,
        used_direction: crate::ParagraphBaseDirection::LeftToRight,
        trim: WhiteSpaceTrim::default(),
        base_style: style(WhiteSpace::Preserve, 20.0),
        atomic: 0,
    });
    capture
        .paragraphs
        .push(facts(ParagraphBaseDirection::RightToLeft));
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::Before,
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
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let views: Vec<_> = result.paragraphs().collect();
    let mut fonts = system();
    let parent = prepare(&mut fonts, views[0].clone());
    let child = prepare(&mut fonts, views[1].clone());
    assert_eq!(line(&parent).feedback().line_height, 10.0);
    assert_eq!(line(&child).feedback().line_height, 20.0);
    assert_eq!(child.analyzed_source().direction(), Direction::RightToLeft);
    assert_eq!(parent.atomic_views().count(), 1);
    assert_eq!(child.atomic_views().count(), 0);
    assert_eq!(text(result.plaintext_items(Affinity::Before)), []);
}

#[test]
fn genuinely_connected_original_pair_crosses_origin_boundary_but_not_an_internal_hole() {
    let mut capture = input(
        Text::from_code_units(vec![0xd800, 0xdc00]),
        WhiteSpace::Preserve,
    );
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
    ];
    capture.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    capture.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    capture.events = vec![
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
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        result.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.atoms.len(), 1);
    assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), [Range::new(0, 2)]);
    let mut capture = hole(InlineBoxKind::OutOfFlow);
    capture.source = Source::new(Text::from_code_units(vec![0xd800, 73, 0xdc00]));
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
        origin(&capture.source, Range::new(2, 3)),
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        result.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .any(|atom| atom.code_point == 0xd800)
    );
    assert!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .any(|atom| atom.code_point == 0xdc00)
    );
}

#[test]
fn contextual_source_defined_replacement_remains_presentation_only() {
    let mut capture = input("AX\u{ad}BBB", WhiteSpace::Preserve);
    capture.replacements = vec![
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(2, 3),
            },
            3,
            vec![Text::from("-")],
        )
        .unwrap(),
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let prepared = prepare(&mut system(), result.paragraphs().next().unwrap());
    let alternative = prepared.alternative_views().next().unwrap().handle();
    let termination = alternative.view().opportunity().unwrap();
    let selected = prepared
        .select(LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: Some(alternative),
            participants: prepared.participants().take(3).collect(),
            atomics: vec![],
            controls: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            context: LineContext {
                alignment_insets: crate::LineAlignmentInsets::ZERO,
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: crate::LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Soft,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,

                spacing: SpacingPolicy::Spaces,
            },
        })
        .unwrap();
    let finalized = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        finalized
            .run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        3
    );
    assert_eq!(
        text(result.plaintext_items(Affinity::Before)),
        [65, 88, 0xad, 66, 66, 66]
    );
}

#[test]
fn delimiter_crossing_scope_or_domain_is_rejected_atomically() {
    let mut capture = input("\r\n", WhiteSpace::Preserve);
    capture.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(0, 2)]);
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
    ];
    capture.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    capture.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    capture.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim::default(),
    });
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    assert!(matches!(
        process_contextual_source(capture).unwrap_err().detail(),
        Some(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::DelimiterCrossing
        })
    ));
}

#[cfg(feature = "text-accessibility")]
#[test]
fn source_and_geometric_accessibility_share_issued_piece_coverage() {
    let result = Arc::new(process_contextual_source(hole(InlineBoxKind::OutOfFlow)).unwrap());
    let views: Vec<_> = result.paragraphs().collect();
    let parent = AnalyzedSource::from_processed_paragraph(
        views[0].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let child = AnalyzedSource::from_processed_paragraph(
        views[1].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let parent = crate::test_wrapping::bind_missing(&parent, Wrap::Word);
    let child = crate::test_wrapping::bind_missing(&child, Wrap::Word);
    let id = accesskit::NodeId(100);
    let mut state = Accessibility::new(200);
    let update = parent.accessibility_update(&mut state, id, id).unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>(),
        "A    B"
    );
    assert!(
        parent
            .access_position(&state, Cursor::new(4, Affinity::After))
            .is_err()
    );
    assert!(
        parent
            .access_selection(
                &state,
                Selection::new(
                    Cursor::new(0, Affinity::After),
                    Cursor::new(9, Affinity::Before)
                )
            )
            .is_err()
    );
    let mut fonts = system();
    let pp = fonts
        .prepare(&parent, PreparationOptions::default())
        .unwrap();
    let cp = fonts
        .prepare(&child, PreparationOptions::default())
        .unwrap();
    let parent_layout = placed(pp);
    let child_layout = placed(cp);
    let update = parent_layout
        .accessibility_update(&mut state, id, id, Point::new(0.0, 0.0))
        .unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>(),
        "A    B"
    );
    assert!(
        parent_layout
            .access_position(&state, Cursor::new(4, Affinity::After))
            .is_err()
    );
    assert_eq!(
        child_layout
            .move_cursor(Cursor::new(4, Affinity::After), Movement::DocumentStart)
            .unwrap()
            .index(),
        3
    );
    assert_eq!(
        child_layout
            .move_cursor(Cursor::new(4, Affinity::After), Movement::DocumentEnd)
            .unwrap()
            .index(),
        6
    );
    let update = child_layout
        .accessibility_update(&mut state, id, id, Point::new(0.0, 0.0))
        .unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>(),
        " I "
    );
}

#[test]
fn actual_inline_trim_scope_survives_block_in_inline_paragraph_cuts_once() {
    for (first, second, expected_first, expected_second) in
        [("  X  ", "  B  ", "AX ", "B C"), ("  ", "  ", "A", "C")]
    {
        let mut capture = input(format!("A{first}  I  {second}  C"), WhiteSpace::Collapse);
        let cuts = [
            0,
            1,
            1 + first.len(),
            6 + first.len(),
            6 + first.len() + second.len(),
            9 + first.len() + second.len(),
        ];
        capture.origins = cuts
            .windows(2)
            .map(|pair| origin(&capture.source, Range::new(pair[0], pair[1])))
            .collect();
        capture.connections = vec![
            SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: ParagraphConnectionKind::Continuous,
            },
            SourceProcessingConnection {
                before_origin: 3,
                after_origin: 4,
                kind: ParagraphConnectionKind::Continuous,
            },
        ];
        capture.boundary_policies = vec![
            policy(ParagraphBoundaryDefinition::Connection(0)),
            policy(ParagraphBoundaryDefinition::Connection(1)),
        ];
        capture.scopes.extend([
            SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                style: None,
                boundaries: None,
                trim: WhiteSpaceTrim {
                    discard_inner: true,
                    ..Default::default()
                },
            },
            SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: SourceProcessingTopology::Unknown,
                used_direction: crate::ParagraphBaseDirection::LeftToRight,
                trim: WhiteSpaceTrim::default(),
                base_style: style(WhiteSpace::Preserve, 20.0),
            },
        ]);
        capture.paragraphs.extend([
            facts(ParagraphBaseDirection::RightToLeft),
            facts(ParagraphBaseDirection::LeftToRight),
        ]);
        capture.events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::EnterScope(2),
            SourceProcessingEvent::EnterParagraph(1),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(2),
            SourceProcessingEvent::EnterParagraph(2),
            SourceProcessingEvent::Origin {
                origin: 3,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::Connection(1),
            SourceProcessingEvent::Origin {
                origin: 4,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(2),
            SourceProcessingEvent::ExitScope(0),
        ];
        let result = Arc::new(process_contextual_source(capture.clone()).unwrap());
        let views: Vec<_> = result.paragraphs().collect();
        assert_eq!(
            text(views[0].plaintext_items(Affinity::Before)),
            expected_first.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            text(views[1].plaintext_items(Affinity::Before)),
            "  I  ".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            text(views[2].plaintext_items(Affinity::Before)),
            expected_second.encode_utf16().collect::<Vec<_>>()
        );
        let mut fonts = system();
        for (i, view) in views.into_iter().enumerate() {
            let prepared = prepare(&mut fonts, view);
            assert_eq!(
                line(&prepared).feedback().line_height,
                if i == 1 { 20.0 } else { 10.0 }
            );
            let layout = placed(prepared);
            assert_eq!(
                layout.lines()[0].range(),
                Range::new(
                    cuts[if i == 0 {
                        0
                    } else if i == 1 {
                        2
                    } else {
                        3
                    }],
                    cuts[if i == 0 {
                        2
                    } else if i == 1 {
                        3
                    } else {
                        5
                    }]
                )
            );
        }
        capture.events.swap(10, 14);
        assert!(process_contextual_source(capture).is_err());
    }
}

#[test]
fn sole_stream_orders_paragraphs_and_same_offset_atomics_independently_of_fact_table_order() {
    let mut capture = hole(InlineBoxKind::OutOfFlow);
    capture.paragraphs.swap(0, 1);
    for event in &mut capture.events {
        match event {
            SourceProcessingEvent::EnterParagraph(i) | SourceProcessingEvent::ExitParagraph(i) => {
                *i = 1 - *i
            }
            _ => {}
        }
    }
    let result = Arc::new(process_contextual_source(capture).unwrap());
    assert_eq!(
        result
            .paragraphs()
            .next()
            .unwrap()
            .source_pieces()
            .collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(6, 9)]
    );
    let mut capture = input("", WhiteSpace::Collapse);
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(10),
        InlineBoxKind::OutOfFlow,
        0,
        Size::new(1.0, 1.0),
    ));
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(11),
        InlineBoxKind::OutOfFlow,
        0,
        Size::new(1.0, 1.0),
    ));
    capture.events.insert(
        3,
        SourceProcessingEvent::Atomic {
            atomic: 1,
            affinity: Affinity::After,
        },
    );
    capture.events.insert(
        4,
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::Before,
        },
    );
    let result = process_contextual_source(capture).unwrap();
    assert_eq!(
        result
            .plaintext_items(Affinity::Before)
            .filter_map(|item| match item {
                PlaintextItem::InlineBox(box_) => Some(box_.original().id()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [Id::from_u64(11), Id::from_u64(10)]
    );
}

#[test]
fn all_whitespace_inline_trim_keeps_original_endpoints_and_real_empty_line_strut() {
    let mut capture = input(" \t ", WhiteSpace::Preserve);
    capture.scopes.push(SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        style: None,
        boundaries: None,
        trim: WhiteSpaceTrim {
            discard_inner: true,
            ..Default::default()
        },
    });
    capture
        .events
        .insert(2, SourceProcessingEvent::EnterScope(1));
    capture
        .events
        .insert(4, SourceProcessingEvent::ExitScope(1));
    let result = Arc::new(process_contextual_source(capture).unwrap());
    assert!(result.processed_text().is_empty());
    assert_eq!(
        result
            .processed_to_source(result.position(0).unwrap(), Affinity::Before)
            .unwrap()
            .get(),
        0
    );
    assert_eq!(
        result
            .processed_to_source(result.position(0).unwrap(), Affinity::After)
            .unwrap()
            .get(),
        3
    );
    let prepared = prepare(&mut system(), result.paragraphs().next().unwrap());
    assert_eq!(line(&prepared).feedback().line_height, 10.0);
    let layout = placed(prepared);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 3));
    assert_eq!(layout.source().text().code_units(), [32, 9, 32]);
}

#[test]
fn selected_soft_wrap_does_not_reset_plaintext_context_or_copy_line_edge_deletion() {
    let result = Arc::new(process_contextual_source(input("A B", WhiteSpace::Collapse)).unwrap());
    let prepared = prepare(&mut system(), result.paragraphs().next().unwrap());
    let termination = prepared
        .opportunity_views()
        .find(|row| row.kind() == PreparedBreakKind::Allowed)
        .unwrap()
        .handle();
    let boundary = termination.view().boundary();
    let count = boundary.view().participant_offset();
    let selected = prepared
        .select(LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: boundary,
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().take(count).collect(),
            atomics: vec![],
            controls: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            context: LineContext {
                alignment_insets: crate::LineAlignmentInsets::ZERO,
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: crate::LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Soft,
                available_inline: Some(10.0),
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,

                spacing: SpacingPolicy::Spaces,
            },
        })
        .unwrap();
    let finalized = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(finalized.feedback().advance, 10.0);
    assert_eq!(text(result.plaintext_items(Affinity::Before)), [65, 32, 66]);
    assert_eq!(result.processed_text().code_units(), [65, 32, 66]);
}

#[test]
fn block_exterior_trim_crosses_its_own_boundary_and_stops_at_a_genuine_forced_control() {
    for forced in [false, true] {
        let mut capture = input("A  X  B", WhiteSpace::Collapse);
        capture.origins = vec![
            origin(&capture.source, Range::new(0, 3)),
            origin(&capture.source, Range::new(3, 4)),
            origin(&capture.source, Range::new(4, 7)),
        ];
        capture.scopes.push(SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            used_direction: crate::ParagraphBaseDirection::LeftToRight,
            trim: WhiteSpaceTrim {
                discard_before: true,
                discard_after: true,
                ..Default::default()
            },
            base_style: style(WhiteSpace::Preserve, 20.0),
        });
        capture.paragraphs.extend([
            facts(ParagraphBaseDirection::LeftToRight),
            facts(ParagraphBaseDirection::LeftToRight),
        ]);
        capture.events = vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::EnterParagraph(1),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(1),
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::EnterParagraph(2),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(2),
            SourceProcessingEvent::ExitScope(0),
        ];
        if forced {
            capture.controls.push(ParagraphControl {
                at: 4,
                kind: ParagraphControlKind::ForcedBreak,
            });
            capture.events.insert(
                10,
                SourceProcessingEvent::Control {
                    control: 0,
                    affinity: Affinity::After,
                },
            );
        }
        let result = Arc::new(process_contextual_source(capture).unwrap());
        assert_eq!(
            result.processed_text().code_units(),
            if forced { "AX B" } else { "AXB" }
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        assert_eq!(text(result.plaintext_items(Affinity::Before)), [65, 88, 66]);
        for at in 1..=3 {
            assert_eq!(
                result
                    .source_to_processed(
                        SourcePosition::try_new(result.source().text(), at).unwrap(),
                        Affinity::After
                    )
                    .unwrap()
                    .get(),
                1
            );
        }
        if !forced {
            for at in 4..=6 {
                assert_eq!(
                    result
                        .source_to_processed(
                            SourcePosition::try_new(result.source().text(), at).unwrap(),
                            Affinity::After
                        )
                        .unwrap()
                        .get(),
                    2
                );
            }
            for view in result.paragraphs() {
                placed(prepare(&mut system(), view));
            }
        }
    }
}

#[test]
fn in_flow_atomic_ends_plaintext_leading_context_and_out_of_flow_atomic_is_transparent() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let mut capture = input(" B", WhiteSpace::Collapse);
        capture.source.inline_box(InlineBox::new(
            Id::from_u64(5),
            kind,
            0,
            Size::new(5.0, 10.0),
        ));
        capture.events.insert(
            2,
            SourceProcessingEvent::Atomic {
                atomic: 0,
                affinity: Affinity::Before,
            },
        );
        if kind == InlineBoxKind::InFlow {
            capture.boundary_policies = vec![
                policy(ParagraphBoundaryDefinition::AtomicBefore(0)),
                policy(ParagraphBoundaryDefinition::AtomicAfter(0)),
            ];
        }
        let result = Arc::new(process_contextual_source(capture).unwrap());
        assert_eq!(result.processed_text().code_units(), [32, 66]);
        assert_eq!(
            text(result.plaintext_items(Affinity::Before)),
            if kind == InlineBoxKind::InFlow {
                " B"
            } else {
                "B"
            }
            .encode_utf16()
            .collect::<Vec<_>>()
        );
        let view = result.paragraphs().next().unwrap();
        assert_eq!(
            text(view.plaintext_items(Affinity::Before)),
            if kind == InlineBoxKind::InFlow {
                " B"
            } else {
                "B"
            }
            .encode_utf16()
            .collect::<Vec<_>>()
        );
        let prepared = prepare(&mut system(), view);
        assert_eq!(prepared.atomic_views().count(), 1);
        placed(prepared);
    }
}

#[test]
fn original_domain_edges_allow_actual_span_boundaries_between_unconnected_surrogate_halves() {
    let mut capture = hole(InlineBoxKind::OutOfFlow);
    capture.source = Source::new(Text::from_code_units(vec![0xd800, 0xdc00]));
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    capture
        .source
        .span(Range::new(1, 2), style(WhiteSpace::Preserve, 20.0));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
        origin(&capture.source, Range::new(2, 2)),
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let views: Vec<_> = result.paragraphs().collect();
    let parent = AnalyzedSource::from_processed_paragraph(
        views[0].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let child = AnalyzedSource::from_processed_paragraph(
        views[1].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        parent
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd800]
    );
    assert_eq!(
        child
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xdc00]
    );
    assert_eq!(parent.graphemes().collect::<Vec<_>>(), [Range::new(0, 1)]);
    assert_eq!(child.graphemes().collect::<Vec<_>>(), [Range::new(1, 2)]);
}

#[test]
fn contextual_parent_preparation_does_not_scale_unowned_child_span_styles() {
    let mut capture = hole(InlineBoxKind::OutOfFlow);
    capture
        .source
        .span(Range::new(3, 6), style(WhiteSpace::Preserve, f32::MAX));
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let views: Vec<_> = result.paragraphs().collect();
    let parent = AnalyzedSource::from_processed_paragraph(
        views[0].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let child = AnalyzedSource::from_processed_paragraph(
        views[1].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut fonts = system();
    let parent = fonts
        .prepare(
            &parent,
            PreparationOptions::try_new(2.0, crate::TabPreparation::FontAdvance).unwrap(),
        )
        .unwrap();
    assert_eq!(line(&parent).feedback().line_height, 20.0);
    assert_eq!(
        fonts
            .prepare(
                &child,
                PreparationOptions::try_new(2.0, crate::TabPreparation::FontAdvance).unwrap()
            )
            .unwrap_err()
            .code,
        ErrorCode::InvalidTextNumeric
    );
}

#[cfg(feature = "text-accessibility")]
#[test]
fn empty_domain_at_nonzero_original_offset_has_source_and_placed_accessibility() {
    let mut capture = input("A", WhiteSpace::Preserve);
    capture
        .origins
        .push(origin(&capture.source, Range::new(1, 1)));
    capture.scopes.push(SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: SourceProcessingTopology::Unknown,
        used_direction: crate::ParagraphBaseDirection::LeftToRight,
        trim: WhiteSpaceTrim::default(),
        base_style: style(WhiteSpace::Preserve, 20.0),
    });
    capture
        .paragraphs
        .push(facts(ParagraphBaseDirection::RightToLeft));
    capture.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::ExitScope(0),
    ];
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let view = result.paragraphs().nth(1).unwrap();
    let analyzed = AnalyzedSource::from_processed_paragraph(
        view,
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let parent = accesskit::NodeId(100);
    let mut state = Accessibility::new(200);
    let update = analyzed
        .accessibility_update(&mut state, parent, parent)
        .unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<Vec<_>>(),
        [""]
    );
    assert!(
        analyzed
            .access_position(&state, Cursor::new(1, Affinity::After))
            .unwrap()
            .is_some()
    );
    assert!(
        analyzed
            .access_position(&state, Cursor::new(0, Affinity::After))
            .is_err()
    );
    let prepared = system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(line(&prepared).feedback().line_height, 20.0);
    let layout = placed(prepared);
    assert_eq!(layout.lines()[0].range(), Range::new(1, 1));
    let update = layout
        .accessibility_update(&mut state, parent, parent, Point::new(2.0, 3.0))
        .unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<Vec<_>>(),
        [""]
    );
    assert!(
        layout
            .access_position(&state, Cursor::new(1, Affinity::After))
            .unwrap()
            .is_some()
    );
    assert!(
        layout
            .access_position(&state, Cursor::new(0, Affinity::After))
            .is_err()
    );
}

fn right_line(prepared: &PreparedText) -> FinalizedLine {
    let termination = prepared
        .opportunity_views()
        .find(|row| row.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let direction = match prepared.analyzed_source().direction() {
        Direction::LeftToRight => ParagraphBaseDirection::LeftToRight,
        Direction::RightToLeft => ParagraphBaseDirection::RightToLeft,
        Direction::Auto => panic!("analysis resolves direction"),
    };
    let selected = prepared
        .select(LineSelectionRequest {
            boundary_context: Vec::new(),
            incoming: None,
            start: prepared.boundary_views().nth(2).unwrap().handle(),
            end: termination.view().boundary(),
            termination: crate::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().skip(2).collect(),
            atomics: vec![],
            controls: vec![],
            line_box_struts: Vec::new(),
            line_strut: None,
            context: LineContext {
                alignment_insets: crate::LineAlignmentInsets::ZERO,
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: direction,
                line_box_direction: match direction {
                    crate::ParagraphBaseDirection::LeftToRight => {
                        crate::LineBoxDirection::LeftToRight
                    }
                    crate::ParagraphBaseDirection::RightToLeft => {
                        crate::LineBoxDirection::RightToLeft
                    }
                },
                orientation: LineOrientation::Horizontal,
                start: LineStart::Continuation,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,

                spacing: SpacingPolicy::Spaces,
            },
        })
        .unwrap();
    selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn short_hole(kind: InlineBoxKind) -> ContextualSourceInput {
    let mut capture = hole(kind);
    capture.source = Source::new("AIB");
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(7),
        kind,
        1,
        Size::new(5.0, 10.0),
    ));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
        origin(&capture.source, Range::new(2, 3)),
    ];
    capture
}

#[test]
fn boundaries_retain_both_original_incident_endpoints_across_a_child_hole() {
    let result = Arc::new(process_contextual_source(short_hole(InlineBoxKind::OutOfFlow)).unwrap());
    let prepared = prepare(&mut system(), result.paragraphs().next().unwrap());
    assert_eq!(
        prepared
            .boundary_views()
            .map(|b| (b.source_before().get(), b.source_after().get()))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 1), (1, 2), (3, 3)]
    );
    let right = right_line(&prepared);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::Range {
            start: prepared.boundary_views().nth(2).unwrap().handle(),
            end: prepared.boundary_views().last().unwrap().handle(),
        },
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line: right,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 20.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.lines()[0].range(), Range::new(2, 3));
    assert!(layout.selection(selection(Range::new(1, 2))).is_err());
}

#[cfg(feature = "text-accessibility")]
#[test]
fn geometric_accessibility_keeps_both_owned_pieces_of_a_cross_hole_ligature() {
    let mut capture = short_hole(InlineBoxKind::OutOfFlow);
    capture.source = Source::new("AIA");
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
        origin(&capture.source, Range::new(2, 3)),
    ];
    if let SourceProcessingScope::Block { base_style, .. } = &mut capture.scopes[0] {
        base_style.locale = Some("en".into());
    }
    let mut child = style(WhiteSpace::Preserve, 20.0);
    child.locale = Some("fr".into());
    capture.source.span(Range::new(1, 2), child);
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        result.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let id = accesskit::NodeId(100);
    let mut state = Accessibility::new(200);
    assert_eq!(
        analyzed
            .accessibility_update(&mut state, id, id)
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, n)| n.value())
            .collect::<String>(),
        "AA"
    );
    let mut fonts = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts
        .register_fonts(crate::shaping_test_fonts::ligature_ahem())
        .unwrap();
    let prepared = fonts
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        line(&prepared)
            .run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        1
    );
    let layout = placed(prepared);
    let update = layout
        .accessibility_update(&mut state, id, id, Point::new(0.0, 0.0))
        .unwrap();
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, n)| n.value())
            .collect::<String>(),
        "AA"
    );
    assert_eq!(
        update
            .nodes
            .iter()
            .filter_map(|(_, n)| n.language())
            .collect::<Vec<_>>(),
        ["en", "en"]
    );
    assert!(layout.selection(selection(Range::new(0, 3))).is_err());
    for piece in [Range::new(0, 1), Range::new(2, 3)] {
        assert_eq!(layout.selection(selection(piece)).unwrap().rects().len(), 1);
    }
}

#[test]
fn trailing_child_domains_do_not_extend_parent_original_ownership() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        for atomic_only in [false, true] {
            let mut capture = short_hole(kind);
            capture.source = Source::new(if atomic_only { "I" } else { "AI" });
            let at = usize::from(!atomic_only);
            capture.source.inline_box(InlineBox::new(
                Id::from_u64(7),
                kind,
                at,
                Size::new(5.0, 10.0),
            ));
            capture.origins = vec![
                origin(&capture.source, Range::new(0, at)),
                origin(&capture.source, Range::new(at, at + 1)),
            ];
            capture.connections.clear();
            capture
                .boundary_policies
                .retain(|p| !matches!(p.definition, ParagraphBoundaryDefinition::Connection(_)));
            capture.events.retain(|event| {
                !matches!(
                    event,
                    SourceProcessingEvent::Connection(_)
                        | SourceProcessingEvent::Origin { origin: 2, .. }
                )
            });
            if atomic_only {
                capture.events.retain(|event| {
                    !matches!(event, SourceProcessingEvent::Origin { origin: 0, .. })
                });
                capture.origins.remove(0);
                for event in &mut capture.events {
                    if let SourceProcessingEvent::Origin { origin, .. } = event {
                        *origin = 0;
                    }
                }
            }
            let result = Arc::new(process_contextual_source(capture).unwrap());
            let view = result.paragraphs().next().unwrap();
            assert_eq!(view.source_range(), Range::new(0, at));
            let layout = placed(prepare(&mut system(), view));
            assert_eq!(layout.lines()[0].range(), Range::new(0, at));
            assert_eq!(
                layout
                    .move_cursor(Cursor::new(0, Affinity::After), Movement::DocumentEnd)
                    .unwrap()
                    .index(),
                at
            );
            assert!(layout.selection(selection(Range::new(at, at + 1))).is_err());
        }
    }
}

fn selection(range: Range) -> Selection {
    Selection::new(
        Cursor::new(range.start, Affinity::After),
        Cursor::new(range.end, Affinity::Before),
    )
}

#[cfg(feature = "text-accessibility")]
#[test]
fn genuine_empty_and_control_endpoints_after_a_child_remain_parent_occurrences() {
    for terminal in 0..3 {
        let mut capture = short_hole(InlineBoxKind::OutOfFlow);
        capture.source = Source::new("AI");
        capture.source.inline_box(InlineBox::new(
            Id::from_u64(7),
            InlineBoxKind::OutOfFlow,
            1,
            Size::new(5.0, 10.0),
        ));
        capture.origins = vec![
            origin(&capture.source, Range::new(0, 1)),
            origin(&capture.source, Range::new(1, 2)),
            origin(&capture.source, Range::new(2, 2)),
        ];
        let has_empty = terminal != 1;
        let has_control = terminal != 0;
        if !has_empty {
            capture.origins.pop();
            capture.connections.clear();
            capture.boundary_policies.clear();
            capture.events.retain(|event| {
                !matches!(
                    event,
                    SourceProcessingEvent::Connection(_)
                        | SourceProcessingEvent::Origin { origin: 2, .. }
                )
            });
        }
        if has_control {
            capture.controls.push(ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
            });
            let end = capture.events.len() - 2;
            capture.events.insert(
                end,
                SourceProcessingEvent::Control {
                    control: 0,
                    affinity: Affinity::After,
                },
            );
        }
        let result = Arc::new(process_contextual_source(capture).unwrap());
        let view = result.paragraphs().next().unwrap();
        assert_eq!(view.source_range(), Range::new(0, 2));
        let analyzed = AnalyzedSource::from_processed_paragraph(
            view,
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(analyzed.inner.host_projection.byte(2).0, 1);
        let id = accesskit::NodeId(100);
        let mut state = Accessibility::new(200);
        assert_eq!(
            analyzed
                .accessibility_update(&mut state, id, id)
                .unwrap()
                .nodes
                .iter()
                .filter_map(|(_, n)| n.value())
                .collect::<String>(),
            "A"
        );
        let prepared = system()
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        assert_eq!(prepared.control_views().count(), usize::from(has_control));
        let end = prepared.boundary_views().last().unwrap();
        assert_eq!(
            (end.source_before().get(), end.source_after().get()),
            (2, 2)
        );
        let layout = placed(prepared);
        assert_eq!(layout.lines()[0].range(), Range::new(0, 2));
        let start = layout.positions().next().unwrap();
        let end = layout.move_position(&start, Movement::DocumentEnd).unwrap();
        assert_eq!(end.source_cursor().index(), 2);
        layout.position_geometry(&end).unwrap();
        let numeric_end =
            layout.move_cursor(Cursor::new(0, Affinity::After), Movement::DocumentEnd);
        if has_empty && has_control {
            assert!(matches!(
                numeric_end.unwrap_err().detail(),
                Some(ErrorDetail::InvalidTextProtocol {
                    reason: ProtocolError::AmbiguousSourcePosition { index: 2, .. }
                })
            ));
        } else {
            assert_eq!(numeric_end.unwrap().index(), 2);
        }
        assert!(layout.selection(selection(Range::new(1, 2))).is_err());
        assert_eq!(
            layout
                .accessibility_update(&mut state, id, id, Point::new(0.0, 0.0))
                .unwrap()
                .nodes
                .iter()
                .filter_map(|(_, n)| n.value())
                .collect::<String>(),
            "A"
        );
    }
}

fn consecutive_child_holes(empty: bool, control: bool) -> ContextualSourceInput {
    let mut capture = input("AIJB", WhiteSpace::Preserve);
    for at in [1, 2] {
        capture.source.inline_box(InlineBox::new(
            Id::from_u64(at as u64),
            InlineBoxKind::OutOfFlow,
            at,
            Size::new(5.0, 10.0),
        ));
    }
    capture.origins = (0..4)
        .map(|i| origin(&capture.source, Range::new(i, i + 1)))
        .collect();
    if empty {
        capture
            .origins
            .push(origin(&capture.source, Range::new(2, 2)));
    }
    capture.connections = if empty {
        vec![
            SourceProcessingConnection {
                before_origin: 0,
                after_origin: 4,
                kind: ParagraphConnectionKind::Continuous,
            },
            SourceProcessingConnection {
                before_origin: 4,
                after_origin: 3,
                kind: ParagraphConnectionKind::Continuous,
            },
        ]
    } else {
        vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 3,
            kind: ParagraphConnectionKind::Continuous,
        }]
    };
    capture.boundary_policies = (0..capture.connections.len())
        .map(|i| policy(ParagraphBoundaryDefinition::Connection(i)))
        .collect();
    for atomic in [0, 1] {
        capture.scopes.push(SourceProcessingScope::OutOfFlowBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            used_direction: crate::ParagraphBaseDirection::LeftToRight,
            trim: WhiteSpaceTrim::default(),
            base_style: style(WhiteSpace::Preserve, 20.0),
            atomic,
        });
        capture
            .paragraphs
            .push(facts(ParagraphBaseDirection::LeftToRight));
    }
    capture.events = vec![
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
    ];
    if empty {
        capture.events.extend([
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 4,
                affinity: Affinity::After,
            },
        ]);
    }
    if control {
        capture.controls.push(ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
        });
        capture.events.push(SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::After,
        });
    }
    capture.events.extend([
        SourceProcessingEvent::Atomic {
            atomic: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::EnterScope(2),
        SourceProcessingEvent::EnterParagraph(2),
        SourceProcessingEvent::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(2),
        SourceProcessingEvent::ExitScope(2),
        SourceProcessingEvent::Connection(usize::from(empty)),
        SourceProcessingEvent::Origin {
            origin: 3,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    capture
}

fn middle_occurrence_layout(empty: bool, control: bool) -> Layout {
    let result =
        Arc::new(process_contextual_source(consecutive_child_holes(empty, control)).unwrap());
    let view = result.paragraphs().next().unwrap();
    assert_eq!(view.source_range(), Range::new(0, 4));
    assert_eq!(text(view.plaintext_items(Affinity::After)), [65, 66]);
    let layout = placed(prepare(&mut system(), view));
    for child in [Range::new(1, 2), Range::new(2, 3), Range::new(1, 3)] {
        assert!(layout.selection(selection(child)).is_err());
        assert!(layout.apply(Edit::Delete { range: child }).is_err());
    }
    layout
}

#[test]
fn authentic_middle_hole_control_cursor_uses_its_actual_placed_geometry() {
    let layout = middle_occurrence_layout(false, true);
    let actual: Vec<_> = layout
        .positions()
        .filter(|p| p.source_cursor() == Cursor::new(2, Affinity::After))
        .collect();
    assert_eq!(actual.len(), 1);
    let geometry = layout.position_geometry(&actual[0]).unwrap();
    assert_eq!(geometry.rect(), Rect::new(10.0, 12.0, 1.0, 10.0));
    assert_eq!(
        layout.cursor(actual[0].source_cursor()).unwrap().rect(),
        geometry.rect()
    );
    assert_eq!(
        layout
            .move_cursor(actual[0].source_cursor(), Movement::DocumentEnd)
            .unwrap()
            .index(),
        4
    );
    assert_eq!(
        layout
            .selection(Selection::collapsed(actual[0].source_cursor()))
            .unwrap()
            .rects()
            .len(),
        0
    );
    let edited = layout
        .apply(Edit::Insert {
            index: 2,
            text: Text::from("X"),
        })
        .unwrap();
    assert_eq!(edited.text().code_units(), [65, 73, 88, 74, 66]);
    #[cfg(feature = "text-accessibility")]
    check_middle_accessibility(&layout);
}

#[test]
fn authentic_middle_hole_atomic_point_is_owned_without_owning_child_text() {
    let layout = middle_occurrence_layout(false, false);
    layout.cursor(Cursor::new(2, Affinity::After)).unwrap();
    assert_eq!(
        layout
            .apply(Edit::Insert {
                index: 2,
                text: Text::from("X")
            })
            .unwrap()
            .text()
            .code_units(),
        [65, 73, 88, 74, 66]
    );
    let mut wide = consecutive_child_holes(false, false);
    wide.source = Source::new("AIIJJB");
    for at in [1, 3] {
        wide.source.inline_box(InlineBox::new(
            Id::from_u64(at as u64),
            InlineBoxKind::OutOfFlow,
            at,
            Size::new(5.0, 10.0),
        ));
    }
    wide.origins = [
        Range::new(0, 1),
        Range::new(1, 3),
        Range::new(3, 5),
        Range::new(5, 6),
    ]
    .into_iter()
    .map(|range| origin(&wide.source, range))
    .collect();
    let wide = Arc::new(process_contextual_source(wide).unwrap());
    let wide = placed(prepare(&mut system(), wide.paragraphs().next().unwrap()));
    wide.cursor(Cursor::new(3, Affinity::After)).unwrap();
    for unowned in [2, 4] {
        assert!(matches!(
            wide.cursor(Cursor::new(unowned, Affinity::After))
                .unwrap_err()
                .detail(),
            Some(ErrorDetail::SourceProcessing {
                reason: SourceProcessingError::OutsideParagraph
            })
        ));
        assert!(
            wide.apply(Edit::Insert {
                index: unowned,
                text: Text::from("X")
            })
            .is_err()
        );
    }
    #[cfg(feature = "text-accessibility")]
    check_middle_accessibility(&layout);
}

#[test]
fn authentic_middle_hole_empty_origin_is_owned_and_retains_real_ambiguity() {
    let layout = middle_occurrence_layout(true, false);
    let position = layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(2, Affinity::After))
        .unwrap();
    assert_eq!(
        layout.cursor(position.source_cursor()).unwrap().rect(),
        layout.position_geometry(&position).unwrap().rect()
    );
    #[cfg(feature = "text-accessibility")]
    check_middle_accessibility(&layout);
    let ambiguous = middle_occurrence_layout(true, true);
    for position in ambiguous
        .positions()
        .filter(|p| p.source_cursor().index() == 2)
    {
        ambiguous.position_geometry(&position).unwrap();
    }
    assert!(matches!(
        ambiguous
            .cursor(Cursor::new(2, Affinity::After))
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::AmbiguousSourcePosition { index: 2, .. }
        })
    ));
}

#[cfg(feature = "text-accessibility")]
fn check_middle_accessibility(layout: &Layout) {
    let mut state = Accessibility::new(200);
    let id = accesskit::NodeId(100);
    let analyzed = layout.analyzed_source();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(3, 4)]
    );
    assert_eq!(
        analyzed
            .accessibility_update(&mut state, id, id)
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, n)| n.value())
            .collect::<String>(),
        "AB"
    );
    analyzed
        .access_position(&state, Cursor::new(2, Affinity::After))
        .unwrap();
    assert!(
        analyzed
            .access_selection(&state, selection(Range::new(1, 3)))
            .is_err()
    );
    assert_eq!(
        layout
            .accessibility_update(&mut state, id, id, Point::new(0.0, 0.0))
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, n)| n.value())
            .collect::<String>(),
        "AB"
    );
    layout
        .access_position(&state, Cursor::new(2, Affinity::After))
        .unwrap();
    assert!(
        layout
            .access_selection(&state, selection(Range::new(1, 3)))
            .is_err()
    );
}

#[test]
fn authentic_middle_hole_points_do_not_claim_foreign_span_scalar_boundaries() {
    let mut capture = consecutive_child_holes(false, false);
    capture.source = Source::new(Text::from_code_units(vec![65, 0xd800, 0xdc00, 66]));
    for at in [1, 2] {
        capture.source.inline_box(InlineBox::new(
            Id::from_u64(at as u64),
            InlineBoxKind::OutOfFlow,
            at,
            Size::new(5.0, 10.0),
        ));
    }
    capture
        .source
        .span(Range::new(2, 3), style(WhiteSpace::Preserve, 20.0));
    capture.origins = (0..4)
        .map(|i| origin(&capture.source, Range::new(i, i + 1)))
        .collect();
    let result = Arc::new(process_contextual_source(capture).unwrap());
    let views: Vec<_> = result.paragraphs().collect();
    let parent = AnalyzedSource::from_processed_paragraph(
        views[0].clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        parent.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(3, 4)]
    );
    assert_eq!(
        placed(
            system()
                .prepare(&parent, PreparationOptions::default())
                .unwrap()
        )
        .glyph_runs()
        .iter()
        .map(|run| run.glyphs().len())
        .sum::<usize>(),
        2
    );
    for (view, unit) in views.into_iter().skip(1).zip([0xd800, 0xdc00]) {
        let child = AnalyzedSource::from_processed_paragraph(
            view,
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            child
                .inner
                .projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect::<Vec<_>>(),
            [unit]
        );
    }
}

#[test]
fn finalized_prefix_keeps_real_cross_hole_associations_without_claiming_child_source() {
    let mut capture = short_hole(InlineBoxKind::OutOfFlow);
    capture.source = Source::new("AIA");
    capture.source.inline_box(InlineBox::new(
        Id::from_u64(7),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(5.0, 10.0),
    ));
    capture.origins = vec![
        origin(&capture.source, Range::new(0, 1)),
        origin(&capture.source, Range::new(1, 2)),
        origin(&capture.source, Range::new(2, 3)),
    ];
    let owner = Arc::new(process_contextual_source(capture).unwrap());
    let paragraphs: Vec<_> = owner.paragraphs().collect();
    let mut fonts = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts
        .register_fonts(crate::shaping_test_fonts::ligature_ahem())
        .unwrap();
    let prepared = prepare(&mut fonts, paragraphs[0].clone());
    let actual = line(&prepared);
    let first = prepared.origin_views().next().unwrap().handle();
    let seam = TextSeam::Origin {
        local: SourcePosition::try_new(first.view().source().text(), 1).unwrap(),
        origin: first,
    };
    let observation = actual.prefix_at(&seam).unwrap();
    assert_eq!(observation.advance(), 10.0);
    assert!(observation.has_retained_flow_participation());
    let unit = observation.interior_unit().unwrap();
    assert_eq!(unit.source_range(), Range::new(0, 3));
    assert_eq!(
        unit.source_pieces().collect::<Vec<_>>(),
        vec![Range::new(0, 1), Range::new(2, 3)]
    );
    let child = prepare(&mut fonts, paragraphs[1].clone());
    let child_origin = child.origin_views().next().unwrap().handle();
    let foreign = TextSeam::Origin {
        local: SourcePosition::try_new(child_origin.view().source().text(), 1).unwrap(),
        origin: child_origin,
    };
    assert!(matches!(
        actual.prefix_at(&foreign).unwrap_err().detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::ForeignSelectedLine
        })
    ));
    let outflow = prepared.atomic_views().next().unwrap().handle();
    // The authentic zero-flow participant is retained at the same internal
    // seam; joint-unit accounting observes the whole actual AA glyph.
    for seam in [
        TextSeam::BeforeParticipant(PreparedParticipant::Atomic(outflow.clone())),
        TextSeam::AfterParticipant(PreparedParticipant::Atomic(outflow)),
    ] {
        let observation = actual.prefix_at(&seam).unwrap();
        assert_eq!(observation.advance(), 10.0);
        assert!(observation.has_retained_flow_participation());
    }
}
