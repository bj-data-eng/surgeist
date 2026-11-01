//! A real nonempty continuation needs its box row when neither endpoint is selected.
use super::*;

fn continuation_case() -> Case {
    use text::SourceProcessingEvent as E;
    let style = authored(10.0);
    let runs = [(1_u32, "X"), (2, "X"), (3, "XXXXXX")];
    let mut source = text::Source::identified("XXXXXXXX", text::Id::from_u64(1578120), 17);
    source.span(text::Range::new(0, 8), style.clone());
    let mut cursor = 0;
    let origins = runs
        .iter()
        .map(|(node, raw)| {
            let start = cursor;
            cursor += raw.len();
            text::ParagraphOrigin {
                text_node: None,
                source: text::Source::identified(
                    *raw,
                    text::Id::from_u64(1578120 + u64::from(*node)),
                    17,
                ),
                local: text::Range::new(0, raw.len()),
                aggregate: text::Range::new(start, cursor),
                kind: text::ParagraphOriginKind::Authored,
            }
        })
        .collect();
    let controls = vec![
        text::ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: 8,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
        text::ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: 8,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: Some(text::ScopeAlignmentLimits::new(
                10000,
                100,
                1_000_000,
                16 << 20,
            )),
            source,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins,
            connections: (0..2)
                .map(|i| text::SourceProcessingConnection {
                    before_origin: i,
                    after_origin: i + 1,
                    kind: text::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls,
            replacements: vec![],
            computed_line_boundaries: vec![],
            boundary_policies: (0..2)
                .map(|i| text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(i),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect(),
            scopes: vec![
                text::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style.clone(),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    font_reference: Some(reference()),
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: Default::default(),
                    style: Some(style),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                    alignment_boundary: Some(text::ScopeAlignmentBoundary::NonInitial),
                    font_reference: Some(reference()),
                },
                // Actual represented child, with numeric endpoint/box geometry.
                // It has neither an invented Initial classification nor a font demand.
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 2, end: 3 }),
                    alignment_boundary: None,
                    font_reference: None,
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::Connection(0),
                E::EnterScope(1),
                E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                },
                E::Origin {
                    origin: 1,
                    affinity: text::Affinity::After,
                },
                E::Connection(1),
                E::EnterScope(2),
                E::Control {
                    control: 2,
                    affinity: text::Affinity::After,
                },
                E::Origin {
                    origin: 2,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 3,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(2),
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let prepared = prepare(&analyzed, 1.0);
    let origins = prepared
        .origin_views()
        .zip(runs)
        .map(|(origin, (node, _))| TextOriginBinding {
            origin: origin.handle(),
            node,
        })
        .collect();
    let controls = prepared
        .control_views()
        .map(|control| {
            let occurrence = control.handle();
            let actual = occurrence.captured_control().unwrap();
            let original = (0..4)
                .find(|&index| processed.captured_control(index).as_ref() == Some(&actual))
                .unwrap();
            TextControlBinding {
                occurrence,
                node: 20 + u32::try_from(original).unwrap(),
            }
        })
        .collect();
    Case {
        binding: TextParagraphBinding {
            inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes: vec![1, 20, 2, 22, 3, 23, 21],
            origins,
            controls,
            atomics: vec![],
            line_strut: None,
        },
        scopes: (0..3)
            .map(|i| processed.captured_scope(i).unwrap())
            .collect(),
        shape: Shape::NumericChild,
        child: None,
    }
}

fn continuation<S: LayoutScalar>() {
    font_premises();
    let case = continuation_case();
    let child = &case.scopes[2];
    let parent = &case.scopes[1];
    assert_eq!(child.parent(), Some(parent.clone()));
    assert_eq!(child.alignment_boundary(), None);
    assert!(child.font_reference().is_none());
    assert!(child.style().is_none());
    assert!(matches!(
        case.binding
            .prepared
            .scope_metrics(child)
            .unwrap_err()
            .detail(),
        Some(text::ErrorDetail::ScopeAlignment {
            reason: text::ScopeAlignmentError::MissingMetrics { scope: 2 }
        })
    ));
    for cut in [4, 6] {
        assert!(
            case.binding
                .prepared
                .opportunity_views()
                .any(|offer| offer.kind() == text::PreparedBreakKind::Allowed
                    && offer.boundary().view().source_before().get() == cut)
        );
    }
    let tree = case.tree::<S>(20.0, VerticalAlignOf::Baseline);
    let wrapping = || WrappingInput {
        native_inline_boxes: Vec::new(),
        gaps: Vec::new(),
        scopes: vec![
            // Parent20/21 are genuine zero-metric boundaries, not absence. Its
            // continuation also needs that truthful0/0 witness when neither is selected.
            WrappingScopeInput::Captured {
                scope: parent.clone(),
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: Some(InlineBoxMetricContribution::Strut(text::LineBoxStrut {
                    scope: None,
                    metrics: text::LineStrut {
                        baseline: 0.0,
                        line_extent: 0.0,
                    },
                    alignment: text::LineBoxAlignment::Baseline,
                })),
            },
            WrappingScopeInput::Captured {
                scope: child.clone(),
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: Some(InlineBoxMetricContribution::Strut(text::LineBoxStrut {
                    scope: None,
                    metrics: text::LineStrut {
                        baseline: 6.0,
                        line_extent: 14.0,
                    },
                    alignment: text::LineBoxAlignment::Baseline,
                })),
            },
        ],
        child_order: vec![],
    };
    let rows = || vec![case.row(1, shifted::<S>(3.0))];
    let input = LayoutFlowInput::try_new(&tree, 0, case.bindings(), wrapping(), rows()).unwrap();
    let batch = compute_layout_with_flow(&input, 0, request(20.0)).unwrap();
    let final_lines = lines(&batch);
    let layout = placed(&batch);
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [
            text::Range::new(0, 2),
            text::Range::new(2, 4),
            text::Range::new(4, 6),
            text::Range::new(6, 8)
        ]
    );
    let middle = &final_lines[2];
    let selected = middle.selected_line();
    let line_request = selected.request();
    // Neither source endpoint can discharge this actual middle continuation.
    for node in [22, 23] {
        let endpoint = &case
            .binding
            .controls
            .iter()
            .find(|row| row.node == node)
            .unwrap()
            .occurrence;
        assert_eq!(endpoint.captured_control().unwrap().scope(), *child);
        assert!(!line_request.participants.iter().any(
            |p| matches!(p, text::PreparedParticipant::Control(control) if control == endpoint)
        ));
        assert!(
            line_request
                .controls
                .iter()
                .all(|row| &row.occurrence != endpoint)
        );
        let actual = final_lines
            .iter()
            .flat_map(|line| line.selected_line().request().controls.clone())
            .filter(|row| &row.occurrence == endpoint)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].strut,
            Some(text::ControlStrut {
                baseline: 6.0,
                line_extent: 14.0,
                alignment: text::LineBoxAlignment::Baseline
            })
        );
    }
    for node in [20, 21] {
        let endpoint = &case
            .binding
            .controls
            .iter()
            .find(|row| row.node == node)
            .unwrap()
            .occurrence;
        assert_eq!(endpoint.captured_control().unwrap().scope(), *parent);
        let actual = final_lines
            .iter()
            .flat_map(|line| line.selected_line().request().controls.clone())
            .filter(|row| &row.occurrence == endpoint)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0].strut,
            Some(text::ControlStrut {
                baseline: 0.0,
                line_extent: 0.0,
                alignment: text::LineBoxAlignment::Baseline
            })
        );
    }
    assert_eq!(line_request.line_box_struts.len(), 2);
    for (scope, baseline, extent) in [(parent, 0.0, 0.0), (child, 6.0, 14.0)] {
        let matches = line_request
            .line_box_struts
            .iter()
            .enumerate()
            .filter(|(_, row)| row.scope.as_ref() == Some(scope))
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        let (index, row) = matches[0];
        assert_eq!(
            row,
            &text::LineBoxStrut {
                scope: Some(scope.clone()),
                metrics: text::LineStrut {
                    baseline,
                    line_extent: extent
                },
                alignment: text::LineBoxAlignment::Baseline
            }
        );
        assert_eq!(
            middle
                .line_box_strut_scope_displacement(index)
                .unwrap()
                .get(),
            3.0
        );
    }
    assert_eq!(
        selected
            .scopes()
            .map(|scope| scope.captured_scope())
            .collect::<Vec<_>>()
            .as_slice(),
        std::slice::from_ref(parent)
    );
    assert_eq!(own(middle, parent), (3.0, 3.0));
    assert_eq!(middle.scope_allocations().len(), 1);
    // Root8/2, shifted glyph11/-1, child9/5 and parent3/-3 give union11/5.
    assert_eq!(
        (
            middle.feedback().advance,
            middle.feedback().baseline,
            middle.feedback().line_height
        ),
        (20.0, 11.0, 16.0)
    );
    let baseline = layout.lines()[2].baseline();
    assert_eq!(layout.lines()[2].bounds().size, text::Size::new(20.0, 16.0));
    assert_eq!(baseline, layout.lines()[2].bounds().origin.y + 11.0);
    assert_eq!(
        glyph_references(layout)
            .into_iter()
            .filter(|(range, _)| range.start >= 4 && range.end <= 6)
            .collect::<Vec<_>>(),
        [
            (text::Range::new(4, 5), baseline - 3.0),
            (text::Range::new(5, 6), baseline - 3.0)
        ]
    );
    let start = layout
        .positions()
        .find(|p| p.source_cursor() == text::Cursor::new(4, text::Affinity::After))
        .unwrap();
    assert_eq!(
        layout.position_geometry(&start).unwrap().rect(),
        text::Rect::new(0.0, baseline - 11.0, 1.0, 16.0)
    );
    let fragments = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| entry.fragment().line_index() == 2)
        .collect::<Vec<_>>();
    assert_eq!(
        fragments
            .iter()
            .map(|entry| entry.owner())
            .collect::<Vec<_>>(),
        [&InlineFragmentOwner::Node(3), &InlineFragmentOwner::Node(3)]
    );
    assert_eq!(
        fragments
            .iter()
            .map(|entry| entry
                .fragment()
                .finalized_fragment()
                .view()
                .aggregate_range())
            .collect::<Vec<_>>(),
        [Some(text::Range::new(4, 5)), Some(text::Range::new(5, 6))]
    );
    raw(layout);
    source_custody(layout);
    let replay = compute_layout_with_flow(&input, 0, request(20.0)).unwrap();
    assert_eq!(batch.final_entries(), replay.final_entries());
    assert_eq!(
        lines(&replay)[2].selected_line().request().line_box_struts,
        line_request.line_box_struts
    );
    for index in 0..2 {
        assert_eq!(
            lines(&replay)[2]
                .line_box_strut_scope_displacement(index)
                .unwrap()
                .get(),
            3.0
        );
    }
    source_custody(placed(&replay));
}

#[test]
fn represented_child_continuation_inherits_parent_frame_in_an_extra_owned_box_row() {
    continuation::<f32>();
    continuation::<f64>();
}
