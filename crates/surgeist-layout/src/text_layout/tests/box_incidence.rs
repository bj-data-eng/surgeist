//! Actual empty-box incidence assigns its supplied metrics to one real line.
#[path = "scope_alignment/context_only.rs"]
mod context_only;
use super::*;
use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct EmptyBox {
    binding: TextParagraphBinding<u32>,
    scope: text::CapturedScope,
    direction: Direction,
}
impl EmptyBox {
    fn new(direction: Direction) -> Self {
        Self::new_with_context(direction, None, None)
    }
    fn new_with_context(
        direction: Direction,
        scope_alignment_limits: Option<text::ScopeAlignmentLimits>,
        domain_font_reference: Option<text::ScopeFontReference>,
    ) -> Self {
        use text::SourceProcessingEvent as E;
        let base_direction = if direction == Direction::Ltr {
            text::ParagraphBaseDirection::LeftToRight
        } else {
            text::ParagraphBaseDirection::RightToLeft
        };
        let style = text::Style {
            word_break: text::WordBreak::BreakAll,
            hyphens: text::Hyphens::None,
            wrap: text::Wrap::Word,
            ..ahem_style(10.0, 10.0)
        };
        let mut source = text::Source::identified("AB", text::Id::from_u64(651000), 3);
        source.span(text::Range::new(0, 2), style.clone());
        let processed = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits,
                source,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::SourceProcessingEnvelope::Complete,
                origins: ["A", "B"]
                    .into_iter()
                    .enumerate()
                    .map(|(i, raw)| text::ParagraphOrigin {
                        text_node: None,
                        source: text::Source::identified(
                            raw,
                            text::Id::from_u64(651001 + i as u64),
                            3,
                        ),
                        local: text::Range::new(0, 1),
                        aggregate: text::Range::new(i, i + 1),
                        kind: text::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: vec![text::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: text::ParagraphConnectionKind::Continuous,
                }],
                controls: vec![],
                replacements: vec![],
                computed_line_boundaries: vec![],
                boundary_policies: vec![text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                }],
                scopes: vec![
                    text::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: domain_font_reference,
                        topology: text::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: style,
                        used_direction: base_direction,
                    },
                    text::SourceProcessingScope::Inline {
                        horizontal_inner_direction: None,
                        alignment_boundary: None,
                        font_reference: None,
                        trim: Default::default(),
                        style: None,
                        boundaries: None,
                    },
                ],
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction,
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
                    E::ExitScope(1),
                    E::Origin {
                        origin: 1,
                        affinity: text::Affinity::After,
                    },
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let scope = processed.captured_scope(1).unwrap();
        let analyzed = text::AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analyzed =
            crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = system
            .prepare(&analyzed, text::PreparationOptions::default())
            .unwrap();
        assert!(prepared.scopes().all(|s| s.captured_scope() != scope));
        assert_eq!(prepared.control_views().count(), 0);
        assert_eq!(prepared.atomic_views().count(), 0);
        let inventory = prepared
            .opportunity_views()
            .map(|o| (o.kind(), o.boundary().view().source_before().get()))
            .collect::<Vec<_>>();
        assert_eq!(
            inventory,
            [
                (text::PreparedBreakKind::Allowed, 1),
                (text::PreparedBreakKind::ParagraphEnd, 2)
            ]
        );
        let soft = prepared
            .opportunity_views()
            .find(|o| o.kind() == text::PreparedBreakKind::Allowed)
            .unwrap()
            .boundary();
        let mut source_segments = prepared.source_segments();
        let source_segment = source_segments.next().expect("fixture source segment");
        assert_eq!(source_segments.count(), 0, "one structural fixture segment");
        let incidences = prepared
            .boundary_incidences(&source_segment, &soft)
            .unwrap();
        assert_eq!(incidences.len(), 2);
        assert_eq!(
            incidences
                .iter()
                .map(|i| prepared.incidence_scope_relation(i, &scope).unwrap())
                .collect::<Vec<_>>(),
            [
                text::CapturedScopeIncidence::AtBefore,
                text::CapturedScopeIncidence::AtAfter
            ]
        );
        assert!(incidences.iter().all(|i| i.boundary() == soft));
        let origins = prepared
            .origin_views()
            .enumerate()
            .map(|(i, origin)| {
                assert_eq!(
                    origin.source().id(),
                    Some(text::Id::from_u64(651001 + i as u64))
                );
                assert_eq!(origin.source().revision(), 3);
                TextOriginBinding {
                    origin: origin.handle(),
                    node: i as u32 + 1,
                }
            })
            .collect();
        Self {
            binding: TextParagraphBinding {
                prepared,
                container: 0,
                nodes: vec![1, 2],
                origins,
                controls: vec![],
                atomics: vec![],
                line_strut: None,
            },
            scope,
            direction,
        }
    }
    fn run<S: LayoutScalar>(
        &self,
        edges: WrapEdges,
        contribution: Option<InlineBoxMetricContribution>,
    ) -> CompletedLayoutBatchOf<u32, S> {
        let mut tree = PublicLayoutTreeOf::<S>::new().children(0, [1, 2]).style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: self.direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(15.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
        for origin in &self.binding.origins {
            tree = tree.children(origin.node, []).input(
                origin.node,
                LayoutInputOf::inline_text(TextParticipation::new(origin.origin.clone())),
            );
        }
        let inline = GeneratedBoxRef::Inline(InlineBoxRef::Scope(self.scope.clone()));
        let child = |box_| GeneratedChildBoxInput {
            box_,
            participation: GeneratedBoxParticipation::InFlow,
        };
        let row = |parent, children| InlineChildOrderInput {
            parent,
            children,
            prefix: ChildOrderCompleteness::Complete,
            suffix: ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        let flow = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![self.binding.clone()],
            WrappingInput {
                scopes: vec![WrappingScopeInput::Captured {
                    scope: self.scope.clone(),
                    wrap_inside: WrapInside::Auto,
                    edges,
                    metric_contribution: contribution,
                }],
                child_order: vec![
                    row(
                        GeneratedBoxRef::Node(0),
                        vec![
                            child(GeneratedBoxRef::Node(1)),
                            child(inline.clone()),
                            child(GeneratedBoxRef::Node(2)),
                        ],
                    ),
                    row(inline, vec![]),
                ],
            },
            Vec::new(),
        )
        .unwrap();
        compute_layout_with_flow(
            &flow,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(15.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap()
    }
}
fn expected_owner(scope: &text::CapturedScope, raw: &text::LineBoxStrut) -> text::LineBoxStrut {
    text::LineBoxStrut {
        scope: Some(scope.clone()),
        ..raw.clone()
    }
}
fn assert_result<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    f: &EmptyBox,
    expected_baselines: [f64; 2],
    expected_heights: [f64; 2],
    strut: text::LineBoxStrut,
    owning_line: usize,
) {
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), &text::Text::from("AB"));
    assert_eq!(
        placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [text::Range::new(0, 1), text::Range::new(1, 2)]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        expected_baselines
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        expected_heights
    );
    assert_eq!(placed.lines()[0].bounds().origin.y, 0.0);
    assert_eq!(placed.lines()[1].bounds().origin.y, expected_heights[0]);
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size, Size::new(S::from_f64(15.0), S::from_f64(30.0)));
    let mut lines = vec![None; 2];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    let x = if f.direction == Direction::Ltr {
        0.0
    } else {
        5.0
    };
    for (index, line) in lines.into_iter().map(Option::unwrap).enumerate() {
        assert_eq!(line.prepared_text(), f.binding.prepared);
        assert_eq!(line.measurements().fit().advance(), 10.0);
        let selected = line.selected_line();
        let request = selected.request();
        assert!(request.controls.is_empty());
        assert!(request.line_strut.is_none());
        assert_eq!(
            request.line_box_struts,
            if index == owning_line {
                vec![strut.clone()]
            } else {
                vec![]
            }
        );
        assert_eq!(
            request.context.end,
            if index == 0 {
                text::LineEnd::Soft
            } else {
                text::LineEnd::Paragraph
            }
        );
        // Line bounds retain the full available metric frame. Alignment is
        // observed on actual placed run/glyph origins, not on that frame.
        assert_eq!(placed.lines()[index].bounds().origin.x, 0.0);
        let physical_x = placed
            .run_views()
            .filter(|run| run.line() == index)
            .flat_map(|run| {
                let origin = run.origin();
                run.glyphs()
                    .map(|glyph| origin.x + glyph.pen() + f64::from(glyph.x()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(physical_x, [x]);
        let glyphs = line
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), AHEM);
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                run.glyphs()
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            glyphs,
            [(text::Range::new(index, index + 1), 35 + index as u32, 10.0)]
        );
    }
}
fn asymmetric<S: LayoutScalar>() {
    use text::LineBoxAlignment::{Baseline, Bottom, Top};
    // Literal geometry, not a test-side MetricGroups implementation. Font8/2
    // supplies the body group. Top20 extends below it; Bottom20 extends above.
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = EmptyBox::new(direction);
        for (alignment, outgoing, receiving) in [
            (Baseline, [16.0, 28.0], [8.0, 26.0]),
            (Top, [8.0, 28.0], [8.0, 18.0]),
            (Bottom, [18.0, 28.0], [8.0, 28.0]),
        ] {
            let strut = text::LineBoxStrut {
                scope: None,
                metrics: text::LineStrut {
                    baseline: 16.0,
                    line_extent: 20.0,
                },
                alignment,
            };
            let expected = expected_owner(&f.scope, &strut);
            let after = f.run::<S>(
                WrapEdges {
                    before: WrapEdge::Avoid,
                    after: WrapEdge::Auto,
                },
                Some(InlineBoxMetricContribution::Strut(strut.clone())),
            );
            assert_result(&after, &f, outgoing, [20.0, 10.0], expected.clone(), 0);
            let before = f.run::<S>(
                WrapEdges {
                    before: WrapEdge::Auto,
                    after: WrapEdge::Avoid,
                },
                Some(InlineBoxMetricContribution::Strut(strut.clone())),
            );
            assert_result(&before, &f, receiving, [10.0, 20.0], expected, 1);
        }
    }
}
#[test]
fn authentic_empty_box_incidence_assigns_its_actual_metric_to_outgoing_or_receiving_line() {
    asymmetric::<f32>();
    asymmetric::<f64>();
}

fn assigned_rows<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<Vec<text::LineBoxStrut>> {
    let mut rows = vec![None; 2];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        rows[fragment.line_index()] = Some(line.selected_line().request().line_box_struts.clone());
    }
    rows.into_iter().map(Option::unwrap).collect()
}
fn neutral_geometry<S: LayoutScalar>() {
    use text::LineBoxAlignment::{Baseline, Bottom, Top};
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = EmptyBox::new(direction);
        for contribution in [None, Some(InlineBoxMetricContribution::NoContribution)] {
            let batch = f.run::<S>(WrapEdges::AUTO, contribution);
            let placed = batch.placed_text()[0].layout();
            assert_eq!(placed.source().text(), &text::Text::from("AB"));
            assert_eq!(
                placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
                [text::Range::new(0, 1), text::Range::new(1, 2)]
            );
            assert_eq!(
                placed
                    .lines()
                    .iter()
                    .map(|l| l.baseline())
                    .collect::<Vec<_>>(),
                [8.0, 18.0]
            );
            assert_eq!(
                placed
                    .lines()
                    .iter()
                    .map(|l| l.bounds().size.height)
                    .collect::<Vec<_>>(),
                [10.0, 10.0]
            );
            assert_eq!(assigned_rows(&batch), [vec![], vec![]]);
            let root = batch
                .final_entries()
                .iter()
                .find(|e| e.node() == 0)
                .unwrap()
                .output();
            assert_eq!(root.size, Size::new(S::from_f64(15.0), S::from_f64(20.0)));
        }
        for (alignment, outgoing, receiving) in [
            (Baseline, [16.0, 28.0], [8.0, 26.0]),
            (Top, [8.0, 28.0], [8.0, 18.0]),
            (Bottom, [18.0, 28.0], [8.0, 28.0]),
        ] {
            let strut = text::LineBoxStrut {
                scope: None,
                metrics: text::LineStrut {
                    baseline: 16.0,
                    line_extent: 20.0,
                },
                alignment,
            };
            let expected = expected_owner(&f.scope, &strut);
            let first = f.run::<S>(
                WrapEdges::AUTO,
                Some(InlineBoxMetricContribution::Strut(strut.clone())),
            );
            let rows = assigned_rows(&first);
            let owners = rows
                .iter()
                .enumerate()
                .filter(|(_, row)| row.as_slice() == [expected.clone()])
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            assert_eq!(
                owners.len(),
                1,
                "one complete incidence must assign the actual strut once"
            );
            let owning_line = owners[0];
            assert!(rows[1 - owning_line].is_empty());
            // Side is deliberately not guessed: the complete chosen outcome
            // must be exactly one of the independently derived coupled rows.
            assert_result(
                &first,
                &f,
                if owning_line == 0 {
                    outgoing
                } else {
                    receiving
                },
                if owning_line == 0 {
                    [20.0, 10.0]
                } else {
                    [10.0, 20.0]
                },
                expected.clone(),
                owning_line,
            );
            let repeated = f.run::<S>(
                WrapEdges::AUTO,
                Some(InlineBoxMetricContribution::Strut(strut)),
            );
            assert_eq!(first.final_entries(), repeated.final_entries());
            assert_eq!(assigned_rows(&first), assigned_rows(&repeated));
        }
    }
}
#[test]
fn auto_edges_retain_actual_supplied_geometry_without_inventing_a_tied_side() {
    neutral_geometry::<f32>();
    neutral_geometry::<f64>();
}
