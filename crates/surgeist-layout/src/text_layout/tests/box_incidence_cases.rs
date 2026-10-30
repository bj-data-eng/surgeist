//! Original box order, real control custody and shape feedback share one incidence.
use super::*;
use std::sync::Arc;
use surgeist_text as text;
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Copy)]
enum Shape {
    Single,
    Siblings,
    Nested,
    Paired,
}
struct Case {
    binding: TextParagraphBinding<u32>,
    scopes: Vec<text::CapturedScope>,
    direction: Direction,
    shape: Shape,
}
fn baseline(baseline: f64, extent: f64) -> text::LineBoxStrut {
    text::LineBoxStrut {
        scope: None,
        metrics: text::LineStrut {
            baseline,
            line_extent: extent,
        },
        alignment: text::LineBoxAlignment::Baseline,
    }
}
fn expected_owner(scope: &text::CapturedScope, raw: &text::LineBoxStrut) -> text::LineBoxStrut {
    text::LineBoxStrut {
        scope: Some(scope.clone()),
        ..raw.clone()
    }
}
fn metric(row: &text::LineBoxStrut) -> Option<InlineBoxMetricContribution> {
    Some(InlineBoxMetricContribution::Strut(row.clone()))
}
fn before_avoid() -> WrapEdges {
    WrapEdges {
        before: WrapEdge::Avoid,
        after: WrapEdge::Auto,
    }
}
fn after_avoid() -> WrapEdges {
    WrapEdges {
        before: WrapEdge::Auto,
        after: WrapEdge::Avoid,
    }
}
impl Case {
    fn new(shape: Shape, direction: Direction) -> Self {
        use text::SourceProcessingEvent as E;
        let base = if direction == Direction::Ltr {
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
        let mut source = text::Source::identified("AB", text::Id::from_u64(652000), 7);
        source.span(text::Range::new(0, 2), style.clone());
        let paired = matches!(shape, Shape::Paired);
        let count = if paired || matches!(shape, Shape::Single) {
            1
        } else {
            2
        };
        let mut scopes = vec![text::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: text::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style,
            used_direction: base,
        }];
        for _ in 0..count {
            scopes.push(text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: None,
                boundaries: paired
                    .then_some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
            });
        }
        let mut events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::Connection(0),
        ];
        match shape {
            Shape::Single => events.extend([E::EnterScope(1), E::ExitScope(1)]),
            Shape::Siblings => events.extend([
                E::EnterScope(1),
                E::ExitScope(1),
                E::EnterScope(2),
                E::ExitScope(2),
            ]),
            Shape::Nested => events.extend([
                E::EnterScope(1),
                E::EnterScope(2),
                E::ExitScope(2),
                E::ExitScope(1),
            ]),
            Shape::Paired => events.extend([
                E::EnterScope(1),
                E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(1),
            ]),
        }
        events.extend([
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
            E::ExitParagraph(0),
            E::ExitScope(0),
        ]);
        let processed = Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
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
                            text::Id::from_u64(652001 + i as u64),
                            7,
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
                controls: if paired {
                    vec![
                        text::ParagraphControl {
                            at: 1,
                            kind: ParagraphControlKind::InlineBoundaryStart,
                        },
                        text::ParagraphControl {
                            at: 1,
                            kind: ParagraphControlKind::InlineBoundaryEnd,
                        },
                    ]
                } else {
                    vec![]
                },
                replacements: vec![],
                computed_line_boundaries: vec![],
                boundary_policies: vec![text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                }],
                scopes,
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: base,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let scopes = (1..=count)
            .map(|i| processed.captured_scope(i).unwrap())
            .collect::<Vec<_>>();
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
        let origins = prepared
            .origin_views()
            .enumerate()
            .map(|(i, o)| {
                assert_eq!(o.source().id(), Some(text::Id::from_u64(652001 + i as u64)));
                assert_eq!(o.source().revision(), 7);
                TextOriginBinding {
                    origin: o.handle(),
                    node: i as u32 + 1,
                }
            })
            .collect();
        let controls = prepared
            .control_views()
            .map(|c| TextControlBinding {
                occurrence: c.handle(),
                node: match c.kind() {
                    ParagraphControlKind::InlineBoundaryStart => 20,
                    ParagraphControlKind::InlineBoundaryEnd => 21,
                    _ => unreachable!(),
                },
            })
            .collect();
        let nodes = if paired {
            vec![1, 20, 21, 2]
        } else {
            vec![1, 2]
        };
        Self {
            binding: TextParagraphBinding {
                prepared,
                container: 0,
                nodes,
                origins,
                controls,
                atomics: vec![],
                line_strut: None,
            },
            scopes,
            direction,
            shape,
        }
    }
    fn inline(&self, i: usize) -> GeneratedBoxRef<u32> {
        GeneratedBoxRef::Inline(InlineBoxRef::Scope(self.scopes[i].clone()))
    }
    fn inventory(
        &self,
        rows: &[(WrapEdges, Option<InlineBoxMetricContribution>)],
        reverse: bool,
        float: bool,
    ) -> WrappingInput<u32> {
        assert_eq!(rows.len(), self.scopes.len());
        let mut scopes = self
            .scopes
            .iter()
            .zip(rows)
            .map(
                |(scope, (edges, contribution))| WrappingScopeInput::Captured {
                    scope: scope.clone(),
                    wrap_inside: WrapInside::Auto,
                    edges: *edges,
                    metric_contribution: contribution.clone(),
                },
            )
            .collect::<Vec<_>>();
        if reverse {
            scopes.reverse();
        }
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
        let mut root = vec![];
        if float {
            root.push(GeneratedChildBoxInput {
                box_: GeneratedBoxRef::Node(90),
                participation: GeneratedBoxParticipation::OutOfFlow,
            });
        }
        root.push(child(GeneratedBoxRef::Node(1)));
        root.push(child(self.inline(0)));
        if matches!(self.shape, Shape::Siblings) {
            root.push(child(self.inline(1)));
        }
        root.push(child(GeneratedBoxRef::Node(2)));
        let mut child_order = vec![row(GeneratedBoxRef::Node(0), root)];
        child_order.push(row(
            self.inline(0),
            if matches!(self.shape, Shape::Nested) {
                vec![child(self.inline(1))]
            } else {
                vec![]
            },
        ));
        if self.scopes.len() == 2 {
            child_order.push(row(self.inline(1), vec![]));
        }
        WrappingInput {
            scopes,
            child_order,
        }
    }
    fn tree<S: LayoutScalar>(&self, width: f64, float: bool) -> PublicLayoutTreeOf<S> {
        let mut children = self.binding.nodes.clone();
        if float {
            children.insert(0, 90);
        }
        let mut tree = PublicLayoutTreeOf::<S>::new().children(0, children).style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: self.direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
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
        for control in &self.binding.controls {
            tree = tree.children(control.node, []).input(
                control.node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::new(
                        if control.node == 20 {
                            InlineBoundaryKind::Start
                        } else {
                            InlineBoundaryKind::End
                        },
                        InlineMetricsOf::try_new(
                            crate::InlineBaselineContext::Physical,
                            S::from_f64(16.0),
                            S::from_f64(20.0),
                        )
                        .unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .with_direction(self.direction),
                ),
            );
        }
        if float {
            tree = tree.children(90, []).style(
                90,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(15.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
        tree
    }
    fn run<S: LayoutScalar>(
        &self,
        rows: &[(WrapEdges, Option<InlineBoxMetricContribution>)],
        reverse: bool,
        float: bool,
        alias: Option<Option<InlineBoxMetricContribution>>,
    ) -> Result<CompletedLayoutBatchOf<u32, S>, LayoutErrorOf<u32, S>> {
        let width = if float { 20.0 } else { 15.0 };
        let tree = self.tree::<S>(width, float);
        let mut inventory = self.inventory(rows, reverse, float);
        if let Some(contribution) = alias {
            inventory.scopes.push(WrappingScopeInput::NativePair {
                start: 20,
                end: 21,
                wrap_inside: WrapInside::Auto,
                edges: rows[0].0,
                metric_contribution: contribution,
            });
        }
        let flow =
            LayoutFlowInput::try_new(&tree, 0, vec![self.binding.clone()], inventory, Vec::new())?;
        compute_layout_with_flow(
            &flow,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(width)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
    }
}
fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut out = vec![None; 2];
    for entry in batch.final_inline_fragments() {
        let f = entry.fragment();
        out[f.line_index()] = Some(f.finalized_fragment().finalized_line());
    }
    out.into_iter().map(Option::unwrap).collect()
}
// Numerical contribution bags have no public canonical scope-list order.
// Match every occurrence, preserving count, values and alignment; a set would
// conceal duplicated or missing same-valued actual box contributions.
fn same_box_struts(actual: &[text::LineBoxStrut], expected: &[text::LineBoxStrut]) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    let mut remaining = actual.to_vec();
    for contribution in expected {
        let Some(index) = remaining.iter().position(|row| row == contribution) else {
            return false;
        };
        remaining.swap_remove(index);
    }
    remaining.is_empty()
}
fn assert_box_struts(actual: &[text::LineBoxStrut], expected: &[text::LineBoxStrut]) {
    assert!(
        same_box_struts(actual, expected),
        "actual box contributions {actual:?}, expected the same occurrences as {expected:?}"
    );
}

fn assert_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    f: &Case,
    heights: [f64; 2],
    baselines: [f64; 2],
    allocations: [Vec<text::LineBoxStrut>; 2],
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
        baselines
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        heights
    );
    assert_eq!(placed.lines()[1].bounds().origin.y, heights[0]);
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size.height, S::from_f64(heights[0] + heights[1]));
    for (index, line) in lines(batch).iter().enumerate() {
        assert_eq!(line.prepared_text(), f.binding.prepared);
        assert_eq!(line.measurements().fit().advance(), 10.0);
        assert_box_struts(
            &line.selected_line().request().line_box_struts,
            &allocations[index],
        );
        assert_eq!(
            line.selected_line().request().context.end,
            if index == 0 {
                text::LineEnd::Soft
            } else {
                text::LineEnd::Paragraph
            }
        );
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
fn scope_error<S: LayoutScalar>(
    result: Result<CompletedLayoutBatchOf<u32, S>, LayoutErrorOf<u32, S>>,
    expected: WrappingScopeError,
) {
    let error = result.expect_err("the real requested contribution custody must refuse precisely");
    assert!(
        matches!(error.kind(),LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(actual)) if *actual==expected)
    );
}
fn sibling_case<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Siblings, direction);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let expected_t = expected_owner(&f.scopes[1], &t);
        let rows = [(before_avoid(), metric(&s)), (after_avoid(), metric(&t))];
        // S then T admits before-both / between / after-both. Only between
        // avoids both declarations. T-outgoing/S-receiving is not an incidence.
        let a = f.run::<S>(&rows, false, false, None).unwrap();
        let b = f.run::<S>(&rows, true, false, None).unwrap();
        assert_geometry(
            &a,
            &f,
            [20.0, 30.0],
            [16.0, 32.0],
            [vec![expected_s.clone()], vec![expected_t.clone()]],
        );
        assert_geometry(
            &b,
            &f,
            [20.0, 30.0],
            [16.0, 32.0],
            [vec![expected_s.clone()], vec![expected_t.clone()]],
        );
        assert_eq!(a.final_entries(), b.final_entries());
    }
}
#[test]
fn sibling_empty_boxes_share_one_original_cut_and_cannot_swap_metric_ownership() {
    sibling_case::<f32>();
    sibling_case::<f64>();
}
fn nested_case<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Nested, direction);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let expected_t = expected_owner(&f.scopes[1], &t);
        let outgoing = f
            .run::<S>(
                &[(before_avoid(), metric(&s)), (WrapEdges::AUTO, metric(&t))],
                false,
                false,
                None,
            )
            .unwrap();
        assert_geometry(
            &outgoing,
            &f,
            [34.0, 10.0],
            [16.0, 42.0],
            [vec![expected_s.clone(), expected_t.clone()], vec![]],
        );
        let receiving = f
            .run::<S>(
                &[(after_avoid(), metric(&s)), (WrapEdges::AUTO, metric(&t))],
                true,
                false,
                None,
            )
            .unwrap();
        assert_geometry(
            &receiving,
            &f,
            [10.0, 34.0],
            [8.0, 26.0],
            [vec![], vec![expected_s.clone(), expected_t.clone()]],
        );
    }
}
#[test]
fn nested_empty_parent_and_child_remain_whole_on_one_side_of_the_selected_cut() {
    nested_case::<f32>();
    nested_case::<f64>();
}
fn custody_case<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Siblings, direction);
        scope_error(
            f.run::<S>(
                &[(before_avoid(), None), (after_avoid(), metric(&t))],
                false,
                false,
                None,
            ),
            WrappingScopeError::MissingBoxMetrics,
        );
        for invalid in [
            baseline(-1.0, 20.0),
            baseline(16.0, 15.0),
            baseline(f64::NAN, 20.0),
            baseline(16.0, f64::INFINITY),
        ] {
            scope_error(
                f.run::<S>(
                    &[
                        (before_avoid(), metric(&invalid)),
                        (after_avoid(), metric(&t)),
                    ],
                    false,
                    false,
                    None,
                ),
                WrappingScopeError::InvalidBoxMetrics,
            );
        }
        let f = Case::new(Shape::Paired, direction);
        let offers = f
            .binding
            .prepared
            .opportunity_views()
            .map(|offer| offer.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            offers
                .iter()
                .map(|offer| {
                    let view = offer.view();
                    let boundary = view.boundary();
                    (
                        view.kind(),
                        boundary.view().participant_offset(),
                        boundary.view().source_before().get(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (text::PreparedBreakKind::Allowed, 1, 1),
                (text::PreparedBreakKind::Allowed, 3, 1),
                (text::PreparedBreakKind::ParagraphEnd, 4, 2),
            ]
        );
        // Both genuine mixed prefixes exist at source1. Each endpoint alias
        // has its own fixed original incidence; source equality cannot merge
        // [A] with [A, StartS, EndS] or select a different control prefix.
        for (offer, relation) in offers.iter().take(2).zip([
            text::CapturedScopeIncidence::AtBefore,
            text::CapturedScopeIncidence::AtAfter,
        ]) {
            let boundary = offer.view().boundary();
            let mut source_segments = f.binding.prepared.source_segments();
            let source_segment = source_segments.next().expect("fixture source segment");
            assert_eq!(source_segments.count(), 0, "one structural fixture segment");
            let incidences = f
                .binding
                .prepared
                .boundary_incidences(&source_segment, &boundary)
                .unwrap();
            assert_eq!(incidences.len(), 1);
            assert_eq!(incidences[0].boundary(), boundary);
            assert_eq!(
                f.binding
                    .prepared
                    .incidence_scope_relation(&incidences[0], &f.scopes[0])
                    .unwrap(),
                relation
            );
        }
        let before = offers[0].view().boundary();
        let a = f
            .run::<S>(
                &[(after_avoid(), metric(&s))],
                false,
                false,
                Some(metric(&s)),
            )
            .unwrap();
        assert_geometry(&a, &f, [10.0, 20.0], [8.0, 26.0], [vec![], vec![]]);
        // One authentic witness can supply the same normalized box through
        // either alias; missing alias evidence is not contradictory geometry.
        for (captured, native) in [(None, metric(&s)), (metric(&s), None)] {
            let one = f
                .run::<S>(&[(after_avoid(), captured)], false, false, Some(native))
                .unwrap();
            assert_geometry(&one, &f, [10.0, 20.0], [8.0, 26.0], [vec![], vec![]]);
            assert_eq!(one.final_entries(), a.final_entries());
        }
        let finalized = lines(&a);
        assert_eq!(finalized[0].selected_line().request().end, before);
        assert_eq!(finalized[1].selected_line().request().start, before);
        assert_eq!(
            finalized[0].selected_line().request().termination,
            text::PreparedTermination::Ordinary(offers[0].clone())
        );
        let participants = f.binding.prepared.participants().collect::<Vec<_>>();
        assert_eq!(participants.len(), 4);
        assert_eq!(
            finalized[0].selected_line().request().participants,
            participants[..1]
        );
        assert_eq!(
            finalized[1].selected_line().request().participants,
            participants[1..]
        );
        assert!(finalized[0].selected_line().request().controls.is_empty());
        let selected = finalized[1].selected_line();
        let controls = &selected.request().controls;
        assert_eq!(controls.len(), 2);
        for binding in &f.binding.controls {
            let actual = controls
                .iter()
                .find(|c| c.occurrence == binding.occurrence)
                .unwrap();
            assert_eq!(
                actual.strut,
                Some(text::ControlStrut {
                    baseline: 16.0,
                    line_extent: 20.0,
                    alignment: text::LineBoxAlignment::Baseline
                })
            );
        }
        // Idempotent metric maxima are insufficient: no extra S allocation is
        // permitted even though adding an equal one would hide geometrically.
        scope_error(
            f.run::<S>(
                &[(WrapEdges::AUTO, metric(&s))],
                false,
                false,
                Some(metric(&baseline(14.0, 22.0))),
            ),
            WrappingScopeError::ConflictingBoxMetrics,
        );
        scope_error(
            f.run::<S>(
                &[(WrapEdges::AUTO, metric(&baseline(14.0, 22.0)))],
                false,
                false,
                None,
            ),
            WrappingScopeError::ConflictingBoxMetrics,
        );
    }
}
#[test]
fn missing_invalid_alias_and_real_control_metric_custody_refuse_or_contribute_once() {
    custody_case::<f32>();
    custody_case::<f64>();
}
fn float_case<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    // LTR physical left float is qualified directly. RTL physical exclusion is
    // deliberately not guessed from mirrored text axes in this first witness.
    let f = Case::new(Shape::Single, Direction::Ltr);
    let expected_s = expected_owner(&f.scopes[0], &s);
    for (edge, heights, baselines, allocations, origins) in [
        (
            before_avoid(),
            [20.0, 10.0],
            [16.0, 28.0],
            [vec![expected_s.clone()], vec![]],
            [(10.0, 0.0), (0.0, 20.0)],
        ),
        (
            after_avoid(),
            [10.0, 20.0],
            [8.0, 26.0],
            [vec![], vec![expected_s.clone()]],
            [(10.0, 0.0), (10.0, 10.0)],
        ),
    ] {
        let rows = [(edge, metric(&s))];
        let a = f.run::<S>(&rows, false, true, None).unwrap();
        assert_geometry(&a, &f, heights, baselines, allocations.clone());
        let b = f.run::<S>(&rows, true, true, None).unwrap();
        assert_geometry(&b, &f, heights, baselines, allocations);
        assert_eq!(a.final_entries(), b.final_entries());
        let placed = a.placed_text()[0].layout();
        assert_eq!(
            placed
                .lines()
                .iter()
                .map(|l| (l.bounds().origin.x, l.bounds().origin.y))
                .collect::<Vec<_>>(),
            origins
        );
        let actual = a
            .final_entries()
            .iter()
            .find(|e| e.node() == 90)
            .unwrap()
            .output();
        assert_eq!(actual.location, Point::ZERO);
        assert_eq!(actual.size, Size::new(S::from_f64(10.0), S::from_f64(15.0)));
        assert_eq!(actual.margin, Edges::ZERO);
    }
}
#[test]
fn selected_empty_box_strut_moves_the_real_receiving_band_past_or_across_a_float() {
    float_case::<f32>();
    float_case::<f64>();
}

// A genuine host-defined static shape, using the same public contract as the
// existing auto_phrase::ShapeTree. Answers depend only on actual geometry, not
// candidate identity, requested result, traversal count or trial acceptance.
struct ShapedFloat<S: LayoutScalar> {
    inner: PublicLayoutTreeOf<S>,
    queries: std::cell::RefCell<Vec<(f64, f64)>>,
}
impl<S: LayoutScalar> Traverse for ShapedFloat<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        S: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.inner, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.inner.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.inner.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for ShapedFloat<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.inner.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.inner.layout_input(node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 90);
        let lo = query.band_minimum().to_f64();
        let hi = query.band_maximum().to_f64();
        self.queries.borrow_mut().push((lo, hi));
        // The real10x20 float has a prescribed shape x[0,10], y(10,20).
        // Its upper10px is transparent; its border box still occupies20px.
        Some(Ok(if lo < 20.0 && hi > 10.0 {
            FloatExclusionIntervalOf::try_new(query, S::ZERO, S::from_f64(10.0)).unwrap()
        } else {
            None
        }))
    }
}
fn rejected_tall_trial<S: LayoutScalar>() {
    let f = Case::new(Shape::Single, Direction::Ltr);
    let s = baseline(16.0, 20.0);
    let expected_s = expected_owner(&f.scopes[0], &s);
    let tree = f.tree::<S>(15.0, true).style(
        90,
        NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            ..NodeInputOf::default()
        },
    );
    let host = ShapedFloat {
        inner: tree,
        queries: std::cell::RefCell::new(vec![]),
    };
    let flow = LayoutFlowInput::try_new(
        &host,
        0,
        vec![f.binding.clone()],
        f.inventory(&[(before_avoid(), metric(&s))], false, true),
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &flow,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(15.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap();
    // At y0 the real first line without S has height10 and available15, so A10
    // fits. Preferred outgoing S makes height20, intersects the lower shape,
    // and has available5: A10 cannot fit. Before-S remains legal despite Avoid.
    // The genuine receiving S line starting at10 also intersects that shape;
    // native float progress moves it below the real float bottom20, then B10
    // fits available15. The gap is layout, not a fabricated metric strut.
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
            .map(|l| (
                l.bounds().origin.x,
                l.bounds().origin.y,
                l.bounds().size.height,
                l.baseline()
            ))
            .collect::<Vec<_>>(),
        [(0.0, 0.0, 10.0, 8.0), (0.0, 20.0, 20.0, 36.0)]
    );
    let finalized = lines(&batch);
    assert!(
        finalized[0]
            .selected_line()
            .request()
            .line_box_struts
            .is_empty()
    );
    assert_eq!(
        finalized[1].selected_line().request().line_box_struts,
        [expected_s]
    );
    for (i, line) in finalized.iter().enumerate() {
        assert_eq!(line.measurements().fit().advance(), 10.0);
        assert!(line.selected_line().request().controls.is_empty());
        let glyphs = line
            .run_views()
            .flat_map(|r| {
                assert_eq!(r.face().bytes(), AHEM);
                r.glyphs()
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [(text::Range::new(i, i + 1), 35 + i as u32, 10.0)]);
    }
    let output = |node| {
        batch
            .final_entries()
            .iter()
            .find(|e| e.node() == node)
            .unwrap()
            .output()
    };
    assert_eq!(
        output(0).size,
        Size::new(S::from_f64(15.0), S::from_f64(40.0))
    );
    assert_eq!(output(90).location, Point::ZERO);
    assert_eq!(
        output(90).size,
        Size::new(S::from_f64(10.0), S::from_f64(20.0))
    );
    // Actual shape queries substantiate distinct supplied geometric contexts;
    // no incidental query count/order is required by this acceptance.
    let queries = host.queries.borrow();
    assert!(queries.contains(&(0.0, 10.0)));
    assert!(queries.contains(&(0.0, 20.0)));
    // At the float's exact bottom20, band20..40 is already disjoint from its
    // real margin box. The broad phase need not invoke the shape provider;
    // final placement/metrics above establish that receiving band instead.
}
#[test]
fn preferred_tall_incidence_that_hits_a_real_shape_does_not_corrupt_the_fitting_prefix_or_receiver()
{
    rejected_tall_trial::<f32>();
    rejected_tall_trial::<f64>();
}

fn run_inventory<S: LayoutScalar>(
    f: &Case,
    inventory: WrappingInput<u32>,
) -> Result<CompletedLayoutBatchOf<u32, S>, LayoutErrorOf<u32, S>> {
    let tree = f.tree::<S>(15.0, false);
    let flow = LayoutFlowInput::try_new(&tree, 0, vec![f.binding.clone()], inventory, Vec::new())?;
    compute_layout_with_flow(
        &flow,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(15.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
}
fn omit_s(inventory: &mut WrappingInput<u32>, f: &Case) {
    inventory.scopes.retain(
        |row| !matches!(row,WrappingScopeInput::Captured {scope,..} if *scope==f.scopes[0]),
    );
    // The real complete generated inventory still names S. Omission from the
    // declaration/metric rows does not erase its authenticated actual box.
}
fn omitted_actual_box<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Siblings, direction);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let mut omitted = f.inventory(
            &[
                (WrapEdges::AUTO, None),
                (
                    after_avoid(),
                    Some(InlineBoxMetricContribution::NoContribution),
                ),
            ],
            false,
            false,
        );
        omit_s(&mut omitted, &f);
        // T's neighboring real declaration activates incidence selection and
        // supplies its actual affirmative absence. S is not listed at all.
        scope_error(
            run_inventory::<S>(&f, omitted),
            WrappingScopeError::MissingBoxMetrics,
        );
        let absent = f
            .run::<S>(
                &[
                    (
                        WrapEdges::AUTO,
                        Some(InlineBoxMetricContribution::NoContribution),
                    ),
                    (
                        after_avoid(),
                        Some(InlineBoxMetricContribution::NoContribution),
                    ),
                ],
                false,
                false,
                None,
            )
            .unwrap();
        assert_geometry(&absent, &f, [10.0, 10.0], [8.0, 18.0], [vec![], vec![]]);
        let supplied = f
            .run::<S>(
                &[
                    (WrapEdges::AUTO, metric(&s)),
                    (
                        after_avoid(),
                        Some(InlineBoxMetricContribution::NoContribution),
                    ),
                ],
                false,
                false,
                None,
            )
            .unwrap();
        let finalized = lines(&supplied);
        let owners = finalized
            .iter()
            .enumerate()
            .filter(|(_, l)| l.selected_line().request().line_box_struts == [expected_s.clone()])
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(owners.len(), 1);
        let owner = owners[0];
        assert_geometry(
            &supplied,
            &f,
            if owner == 0 {
                [20.0, 10.0]
            } else {
                [10.0, 20.0]
            },
            if owner == 0 {
                [16.0, 28.0]
            } else {
                [8.0, 26.0]
            },
            if owner == 0 {
                [vec![expected_s.clone()], vec![]]
            } else {
                [vec![], vec![expected_s.clone()]]
            },
        );
    }
}
#[test]
fn actual_captured_box_omitted_from_metric_rows_is_not_affirmative_no_contribution() {
    omitted_actual_box::<f32>();
    omitted_actual_box::<f64>();
}

fn pinned_after_s(direction: Direction) -> Case {
    use text::SourceProcessingEvent as E;
    let base = if direction == Direction::Ltr {
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
    let mut source = text::Source::identified("AB", text::Id::from_u64(653000), 9);
    source.span(text::Range::new(0, 2), style.clone());
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: ["A", "B"]
                .into_iter()
                .enumerate()
                .map(|(i, raw)| text::ParagraphOrigin {
                    text_node: None,
                    source: text::Source::identified(raw, text::Id::from_u64(653001 + i as u64), 9),
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
            computed_line_boundaries: vec![text::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::SourceBoxSide::After,
            }],
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
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style,
                    used_direction: base,
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
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
                base_direction: base,
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
                E::EnterScope(2),
                E::ExitScope(2),
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
    let scopes = vec![
        processed.captured_scope(1).unwrap(),
        processed.captured_scope(2).unwrap(),
    ];
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert!(
        prepared
            .scopes()
            .all(|s| !scopes.contains(&s.captured_scope()))
    );
    assert_eq!(prepared.control_views().count(), 0);
    let inventory = prepared
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().source_before().get()))
        .collect::<Vec<_>>();
    assert_eq!(
        inventory,
        [
            (text::PreparedBreakKind::Forced, 1),
            (text::PreparedBreakKind::ParagraphEnd, 2)
        ]
    );
    let force = prepared
        .opportunity_views()
        .find(|o| o.kind() == text::PreparedBreakKind::Forced)
        .unwrap();
    assert!(force.computed_line_boundary().is_some());
    let boundary = force.boundary();
    let mut source_segments = prepared.source_segments();
    let source_segment = source_segments.next().expect("fixture source segment");
    assert_eq!(source_segments.count(), 0, "one structural fixture segment");
    let incidences = prepared
        .boundary_incidences(&source_segment, &boundary)
        .unwrap();
    assert_eq!(incidences.len(), 1);
    assert_eq!(
        prepared
            .incidence_scope_relation(&incidences[0], &scopes[0])
            .unwrap(),
        text::CapturedScopeIncidence::AtAfter
    );
    assert_eq!(
        prepared
            .incidence_scope_relation(&incidences[0], &scopes[1])
            .unwrap(),
        text::CapturedScopeIncidence::AtBefore
    );
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(i, o)| {
            assert_eq!(o.source().id(), Some(text::Id::from_u64(653001 + i as u64)));
            assert_eq!(o.source().revision(), 9);
            TextOriginBinding {
                origin: o.handle(),
                node: i as u32 + 1,
            }
        })
        .collect();
    Case {
        binding: TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1, 2],
            origins,
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        },
        scopes,
        direction,
        shape: Shape::Siblings,
    }
}
fn fixed_original_custody<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = pinned_after_s(direction);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let rows = [
            (WrapEdges::AUTO, None),
            (
                after_avoid(),
                Some(InlineBoxMetricContribution::NoContribution),
            ),
        ];
        // The authentic computed pin leaves exactly one global placement.
        // Uniqueness supplies placement, not S's unprovided actual metrics.
        scope_error(
            run_inventory::<S>(&f, f.inventory(&rows, false, false)),
            WrappingScopeError::MissingBoxMetrics,
        );
        let mut omitted = f.inventory(&rows, false, false);
        omit_s(&mut omitted, &f);
        scope_error(
            run_inventory::<S>(&f, omitted),
            WrappingScopeError::MissingBoxMetrics,
        );
        let rows = [
            (WrapEdges::AUTO, metric(&s)),
            (
                after_avoid(),
                Some(InlineBoxMetricContribution::NoContribution),
            ),
        ];
        let supplied = run_inventory::<S>(&f, f.inventory(&rows, false, false)).unwrap();
        // Fixed AtAfterS places the real S contribution only on the outgoing
        // Forced line; B receives the ordinary body metrics, not another S.
        let placed = supplied.placed_text()[0].layout();
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
            [16.0, 28.0]
        );
        assert_eq!(
            placed
                .lines()
                .iter()
                .map(|l| l.bounds().size.height)
                .collect::<Vec<_>>(),
            [20.0, 10.0]
        );
        let finalized = lines(&supplied);
        assert_eq!(
            finalized[0].selected_line().request().context.end,
            text::LineEnd::Forced
        );
        assert_eq!(
            finalized[1].selected_line().request().context.end,
            text::LineEnd::Paragraph
        );
        assert_eq!(
            finalized[0]
                .selected_line()
                .request()
                .line_box_struts
                .as_slice(),
            std::slice::from_ref(&expected_s)
        );
        assert!(
            finalized[1]
                .selected_line()
                .request()
                .line_box_struts
                .is_empty()
        );
        assert!(
            finalized
                .iter()
                .all(|l| l.selected_line().request().controls.is_empty())
        );
        let root = supplied
            .final_entries()
            .iter()
            .find(|e| e.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, Size::new(S::from_f64(15.0), S::from_f64(30.0)));
    }
}
#[test]
fn computed_pin_with_one_original_placement_does_not_supply_missing_box_metrics() {
    fixed_original_custody::<f32>();
    fixed_original_custody::<f64>();
}

fn actual_control_alignment<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Paired, direction);
        let same = baseline(16.0, 20.0);
        let offers = f
            .binding
            .prepared
            .opportunity_views()
            .map(|offer| offer.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            offers
                .iter()
                .map(|offer| {
                    let view = offer.view();
                    let boundary = view.boundary();
                    (
                        view.kind(),
                        boundary.view().participant_offset(),
                        boundary.view().source_before().get(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (text::PreparedBreakKind::Allowed, 1, 1),
                (text::PreparedBreakKind::Allowed, 3, 1),
                (text::PreparedBreakKind::ParagraphEnd, 4, 2),
            ]
        );
        for (offer, relation) in offers.iter().take(2).zip([
            text::CapturedScopeIncidence::AtBefore,
            text::CapturedScopeIncidence::AtAfter,
        ]) {
            let boundary = offer.view().boundary();
            let mut source_segments = f.binding.prepared.source_segments();
            let source_segment = source_segments.next().expect("fixture source segment");
            assert_eq!(source_segments.count(), 0, "one structural fixture segment");
            let incidences = f
                .binding
                .prepared
                .boundary_incidences(&source_segment, &boundary)
                .unwrap();
            assert_eq!(incidences.len(), 1);
            assert_eq!(incidences[0].boundary(), boundary);
            assert_eq!(
                f.binding
                    .prepared
                    .incidence_scope_relation(&incidences[0], &f.scopes[0])
                    .unwrap(),
                relation
            );
        }
        let before = offers[0].view().boundary();
        let control = f
            .run::<S>(
                &[(after_avoid(), metric(&same))],
                false,
                false,
                Some(metric(&same)),
            )
            .unwrap();
        assert_geometry(&control, &f, [10.0, 20.0], [8.0, 26.0], [vec![], vec![]]);
        let finalized = lines(&control);
        assert_eq!(finalized[0].selected_line().request().end, before);
        assert_eq!(finalized[1].selected_line().request().start, before);
        assert!(finalized[0].selected_line().request().controls.is_empty());
        let selected = finalized[1].selected_line();
        assert_eq!(selected.request().controls.len(), 2);
        for owner in &f.binding.controls {
            let allocation = selected
                .request()
                .controls
                .iter()
                .find(|allocation| allocation.occurrence == owner.occurrence)
                .unwrap();
            assert_eq!(
                allocation.strut,
                Some(text::ControlStrut {
                    baseline: 16.0,
                    line_extent: 20.0,
                    alignment: text::LineBoxAlignment::Baseline,
                })
            );
        }
        for alignment in [text::LineBoxAlignment::Top, text::LineBoxAlignment::Bottom] {
            let contradictory = text::LineBoxStrut {
                scope: None,
                metrics: same.metrics,
                alignment,
            };
            // Equal numbers do not erase the actual owning control's Baseline
            // tag. Its canonical same-box identity is established by the real
            // captured endpoint pair, not an arbitrary descendant control.
            scope_error(
                f.run::<S>(
                    &[(WrapEdges::AUTO, metric(&contradictory))],
                    false,
                    false,
                    None,
                ),
                WrappingScopeError::ConflictingBoxMetrics,
            );
            scope_error(
                f.run::<S>(
                    &[(WrapEdges::AUTO, metric(&same))],
                    false,
                    false,
                    Some(metric(&contradictory)),
                ),
                WrappingScopeError::ConflictingBoxMetrics,
            );
        }
    }
}
#[test]
fn same_box_alignment_conflict_is_not_deduped_by_equal_metric_numbers() {
    actual_control_alignment::<f32>();
    actual_control_alignment::<f64>();
}

fn unique_outer_preference<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Nested, direction);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let expected_t = expected_owner(&f.scopes[1], &t);
        for (edge, heights, baselines, allocations) in [
            (
                before_avoid(),
                [34.0, 10.0],
                [16.0, 42.0],
                [vec![expected_s.clone(), expected_t.clone()], vec![]],
            ),
            (
                after_avoid(),
                [10.0, 34.0],
                [8.0, 26.0],
                [vec![], vec![expected_s.clone(), expected_t.clone()]],
            ),
        ] {
            let mut inventory = f.inventory(
                &[(edge, metric(&s)), (WrapEdges::AUTO, metric(&t))],
                false,
                false,
            );
            for row in &mut inventory.scopes {
                if let WrappingScopeInput::Captured {
                    scope, wrap_inside, ..
                } = row
                    && *scope == f.scopes[0]
                {
                    // An explicit additional preference, never inherited by T
                    // or smuggled into the original still-unresolved fixture.
                    *wrap_inside = WrapInside::Avoid;
                }
            }
            // The outside non-avoided edge violates no constraint. Any genuine
            // inside-S realization violates InsideAvoid; the opposite exterior
            // violates the authored edge. The unique outside class dominates.
            let actual = run_inventory::<S>(&f, inventory).unwrap();
            assert_geometry(&actual, &f, heights, baselines, allocations);
        }
    }
}
#[test]
fn explicit_outer_inside_avoid_plus_edge_avoid_uniquely_selects_the_other_outer_edge() {
    unique_outer_preference::<f32>();
    unique_outer_preference::<f64>();
}

fn realization_case(direction: Direction, force: bool) -> Case {
    use text::SourceProcessingEvent as E;
    let base = if direction == Direction::Ltr {
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
    let mut source = text::Source::identified("AB", text::Id::from_u64(654000), 11);
    source.span(text::Range::new(0, 2), style.clone());
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if force {
        // Real A/B are outside S. Before-T is a genuine early force INSIDE S,
        // so the exact hard cause, not a fabricated soft offer, splits S.
        events.extend([
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(1),
            E::EnterScope(2),
            E::ExitScope(2),
            E::ExitScope(1),
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
        ]);
    } else {
        // S has genuine retained original content on BOTH sides of T. Its
        // ordinary AB seam is inside S, not an empty whole-subtree edge choice.
        events.extend([
            E::EnterScope(1),
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(2),
            E::ExitScope(2),
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
        ]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
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
                        text::Id::from_u64(654001 + i as u64),
                        11,
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
            computed_line_boundaries: if force {
                vec![text::ComputedLineBoundaryInput::Scope {
                    scope: 2,
                    side: text::SourceBoxSide::Before,
                }]
            } else {
                vec![]
            },
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
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style,
                    used_direction: base,
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
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
                base_direction: base,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let scopes = vec![
        processed.captured_scope(1).unwrap(),
        processed.captured_scope(2).unwrap(),
    ];
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let inventory = prepared
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().source_before().get()))
        .collect::<Vec<_>>();
    assert_eq!(
        inventory,
        [
            (
                if force {
                    text::PreparedBreakKind::Forced
                } else {
                    text::PreparedBreakKind::Allowed
                },
                1
            ),
            (text::PreparedBreakKind::ParagraphEnd, 2)
        ]
    );
    let offer = prepared.opportunity_views().next().unwrap();
    assert_eq!(offer.computed_line_boundary().is_some(), force);
    let boundary = offer.boundary();
    let mut source_segments = prepared.source_segments();
    let source_segment = source_segments.next().expect("fixture source segment");
    assert_eq!(source_segments.count(), 0, "one structural fixture segment");
    let incidences = prepared
        .boundary_incidences(&source_segment, &boundary)
        .unwrap();
    let relations = incidences
        .iter()
        .map(|i| {
            assert_eq!(i.boundary(), boundary);
            (
                prepared.incidence_scope_relation(i, &scopes[0]).unwrap(),
                prepared.incidence_scope_relation(i, &scopes[1]).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    use text::CapturedScopeIncidence as R;
    assert_eq!(
        relations,
        if force {
            vec![(R::Inside, R::AtBefore)]
        } else {
            vec![(R::Inside, R::AtBefore), (R::Inside, R::AtAfter)]
        }
    );
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(i, o)| {
            assert_eq!(o.source().id(), Some(text::Id::from_u64(654001 + i as u64)));
            assert_eq!(o.source().revision(), 11);
            TextOriginBinding {
                origin: o.handle(),
                node: i as u32 + 1,
            }
        })
        .collect();
    Case {
        binding: TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1, 2],
            origins,
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        },
        scopes,
        direction,
        shape: Shape::Nested,
    }
}
fn realization_inventory(f: &Case, force: bool, t_edge: WrapEdges) -> WrappingInput<u32> {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    let mut inventory = f.inventory(
        &[(WrapEdges::AUTO, metric(&s)), (t_edge, metric(&t))],
        false,
        false,
    );
    if !force {
        // Honest generated parenting changes with actual source ownership.
        // Flat native text traversal is not an assertion that A/B lie outside S.
        let child = |box_| GeneratedChildBoxInput {
            box_,
            participation: GeneratedBoxParticipation::InFlow,
        };
        for row in &mut inventory.child_order {
            if row.parent == GeneratedBoxRef::Node(0) {
                row.children = vec![child(f.inline(0))];
            } else if row.parent == f.inline(0) {
                row.children = vec![
                    child(GeneratedBoxRef::Node(1)),
                    child(f.inline(1)),
                    child(GeneratedBoxRef::Node(2)),
                ];
            }
        }
    }
    // The force is an actual computed participant/cause, NOT a generated child
    // box or fabricated host control node. Empty T remains a genuine child.
    inventory
}
fn realization_compute<S: LayoutScalar>(
    f: &Case,
    force: bool,
    t_edge: WrapEdges,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = f.tree::<S>(15.0, false);
    let input = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        realization_inventory(f, force, t_edge),
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &input,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(15.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}
fn realization_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    f: &Case,
    force: bool,
    heights: [f64; 2],
    baselines: [f64; 2],
    allocations: [Vec<text::LineBoxStrut>; 2],
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
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        heights
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        baselines
    );
    assert_eq!(placed.lines()[1].bounds().origin.y, heights[0]);
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size, Size::new(S::from_f64(15.0), S::from_f64(54.0)));
    for (i, line) in lines(batch).iter().enumerate() {
        assert_eq!(line.prepared_text(), f.binding.prepared);
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(
            request.context.end,
            if i == 0 {
                if force {
                    text::LineEnd::Forced
                } else {
                    text::LineEnd::Soft
                }
            } else {
                text::LineEnd::Paragraph
            }
        );
        assert_box_struts(&request.line_box_struts, &allocations[i]);
        assert!(request.controls.is_empty());
        assert_eq!(line.measurements().fit().advance(), 10.0);
        let glyphs = line
            .run_views()
            .flat_map(|r| {
                assert_eq!(r.face().bytes(), AHEM);
                assert_eq!(r.effective_instance().unwrap().font_size(), 10.0);
                r.glyphs()
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [(text::Range::new(i, i + 1), 35 + i as u32, 10.0)]);
        let physical_x = placed
            .run_views()
            .filter(|r| r.line() == i)
            .flat_map(|r| {
                let origin = r.origin();
                r.glyphs()
                    .map(|g| origin.x + g.pen() + f64::from(g.x()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            physical_x,
            [if f.direction == Direction::Ltr {
                0.0
            } else {
                5.0
            }]
        );
    }
}
fn retained_parent_content<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = realization_case(direction, false);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let expected_t = expected_owner(&f.scopes[1], &t);
        let after = realization_compute::<S>(&f, false, before_avoid());
        realization_geometry(
            &after,
            &f,
            false,
            [34.0, 20.0],
            [16.0, 50.0],
            [
                vec![expected_s.clone(), expected_t.clone()],
                vec![expected_s.clone()],
            ],
        );
        let before = realization_compute::<S>(&f, false, after_avoid());
        realization_geometry(
            &before,
            &f,
            false,
            [20.0, 34.0],
            [16.0, 36.0],
            [
                vec![expected_s.clone()],
                vec![expected_s.clone(), expected_t.clone()],
            ],
        );
    }
}
#[test]
fn real_parent_content_on_both_sides_keeps_genuine_inner_incidence_and_parent_fragments() {
    retained_parent_content::<f32>();
    retained_parent_content::<f64>();
}
fn genuine_inner_force<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = realization_case(direction, true);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let expected_t = expected_owner(&f.scopes[1], &t);
        for edge in [WrapEdges::AUTO, before_avoid(), after_avoid()] {
            let actual = realization_compute::<S>(&f, true, edge);
            // The real Before-T hard pin is Inside-S. Neither whole-empty
            // filtering nor avoidance may move/suppress that mandatory cause.
            realization_geometry(
                &actual,
                &f,
                true,
                [20.0, 34.0],
                [16.0, 36.0],
                [
                    vec![expected_s.clone()],
                    vec![expected_s.clone(), expected_t.clone()],
                ],
            );
        }
    }
}
#[test]
fn genuine_computed_force_inside_a_glyph_free_parent_keeps_its_pinned_split() {
    genuine_inner_force::<f32>();
    genuine_inner_force::<f64>();
}

fn exterior_case(direction: Direction, trailing: bool) -> Case {
    use text::SourceProcessingEvent as E;
    let base = if direction == Direction::Ltr {
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
    let mut source = text::Source::identified("AB", text::Id::from_u64(655000), 13);
    source.span(text::Range::new(0, 2), style.clone());
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if !trailing {
        events.extend([E::EnterScope(1), E::ExitScope(1)]);
    }
    events.extend([
        E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        },
        E::Connection(0),
        E::Origin {
            origin: 1,
            affinity: text::Affinity::After,
        },
    ]);
    if trailing {
        events.extend([E::EnterScope(1), E::ExitScope(1)]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
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
                        text::Id::from_u64(655001 + i as u64),
                        13,
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
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style,
                    used_direction: base,
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
                base_direction: base,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let scope = processed.captured_scope(1).unwrap();
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert_eq!(
        prepared.participants().count(),
        2,
        "no fake zero-advance participant represents S"
    );
    assert_eq!(prepared.control_views().count(), 0);
    assert!(prepared.scopes().all(|s| s.captured_scope() != scope));
    let opportunities = prepared
        .opportunity_views()
        .map(|o| {
            assert!(o.computed_line_boundary().is_none());
            (
                o.kind(),
                o.boundary().view().participant_offset(),
                o.boundary().view().source_before().get(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        opportunities,
        [
            (text::PreparedBreakKind::Allowed, 1, 1),
            (text::PreparedBreakKind::ParagraphEnd, 2, 2)
        ]
    );
    // Original metadata can have two authentic placements at the paragraph
    // edge. That observation does not grant a preceding/following source line.
    let exterior = if trailing {
        prepared.boundary_views().last().unwrap().handle()
    } else {
        prepared.boundary_views().next().unwrap().handle()
    };
    assert_eq!(
        exterior.view().participant_offset(),
        if trailing { 2 } else { 0 }
    );
    let mut source_segments = prepared.source_segments();
    let source_segment = source_segments.next().expect("fixture source segment");
    assert_eq!(source_segments.count(), 0, "one structural fixture segment");
    let incidences = prepared
        .boundary_incidences(&source_segment, &exterior)
        .unwrap();
    assert_eq!(
        incidences
            .iter()
            .map(|i| {
                assert_eq!(i.boundary(), exterior);
                prepared.incidence_scope_relation(i, &scope).unwrap()
            })
            .collect::<Vec<_>>(),
        [
            text::CapturedScopeIncidence::AtBefore,
            text::CapturedScopeIncidence::AtAfter
        ]
    );
    let soft = prepared.opportunity_views().next().unwrap().boundary();
    let soft_incidences = prepared
        .boundary_incidences(&source_segment, &soft)
        .unwrap();
    assert_eq!(soft_incidences.len(), 1);
    assert_eq!(
        prepared
            .incidence_scope_relation(&soft_incidences[0], &scope)
            .unwrap(),
        if trailing {
            text::CapturedScopeIncidence::Before
        } else {
            text::CapturedScopeIncidence::After
        }
    );
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(i, o)| {
            assert_eq!(o.source().id(), Some(text::Id::from_u64(655001 + i as u64)));
            assert_eq!(o.source().revision(), 13);
            TextOriginBinding {
                origin: o.handle(),
                node: i as u32 + 1,
            }
        })
        .collect();
    Case {
        binding: TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1, 2],
            origins,
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        },
        scopes: vec![scope],
        direction,
        shape: Shape::Single,
    }
}
fn exterior_inventory(f: &Case, trailing: bool, edge: WrapEdges) -> WrappingInput<u32> {
    let s = baseline(16.0, 20.0);
    let mut inventory = f.inventory(&[(edge, metric(&s))], false, false);
    let child = |box_| GeneratedChildBoxInput {
        box_,
        participation: GeneratedBoxParticipation::InFlow,
    };
    let root = inventory
        .child_order
        .iter_mut()
        .find(|row| row.parent == GeneratedBoxRef::Node(0))
        .unwrap();
    root.children = if trailing {
        vec![
            child(GeneratedBoxRef::Node(1)),
            child(GeneratedBoxRef::Node(2)),
            child(f.inline(0)),
        ]
    } else {
        vec![
            child(f.inline(0)),
            child(GeneratedBoxRef::Node(1)),
            child(GeneratedBoxRef::Node(2)),
        ]
    };
    inventory
}
fn exterior_custody<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        for trailing in [false, true] {
            let f = exterior_case(direction, trailing);
            let expected_s = expected_owner(&f.scopes[0], &s);
            let exterior_avoid = if trailing {
                after_avoid()
            } else {
                before_avoid()
            };
            for edge in [WrapEdges::AUTO, exterior_avoid] {
                let tree = f.tree::<S>(15.0, false);
                let input = LayoutFlowInput::try_new(
                    &tree,
                    0,
                    vec![f.binding.clone()],
                    exterior_inventory(&f, trailing, edge),
                    Vec::new(),
                )
                .unwrap();
                let actual = compute_layout_with_flow(
                    &input,
                    0,
                    LayoutRootRequestOf::viewport(Size::new(
                        AvailableOf::definite(S::from_f64(15.0)),
                        AvailableOf::MAX_CONTENT,
                    ))
                    .unwrap(),
                )
                .unwrap();
                // Paragraph entry has no previous line to consume leading S.
                // Paragraph completion cannot leave trailing S to an invented
                // third line, even if its exterior edge discourages wrapping.
                assert_geometry(
                    &actual,
                    &f,
                    if trailing { [10.0, 20.0] } else { [20.0, 10.0] },
                    if trailing { [8.0, 26.0] } else { [16.0, 28.0] },
                    if trailing {
                        [vec![], vec![expected_s.clone()]]
                    } else {
                        [vec![expected_s.clone()], vec![]]
                    },
                );
                let root = actual
                    .final_entries()
                    .iter()
                    .find(|e| e.node() == 0)
                    .unwrap()
                    .output();
                assert_eq!(root.size, Size::new(S::from_f64(15.0), S::from_f64(30.0)));
                let placed = actual.placed_text()[0].layout();
                for i in 0..2 {
                    let physical_x = placed
                        .run_views()
                        .filter(|run| run.line() == i)
                        .flat_map(|run| {
                            let origin = run.origin();
                            run.glyphs()
                                .map(|g| origin.x + g.pen() + f64::from(g.x()))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        physical_x,
                        [if direction == Direction::Ltr {
                            0.0
                        } else {
                            5.0
                        }]
                    );
                }
            }
        }
    }
}
#[test]
fn leading_and_trailing_actual_empty_boxes_are_owned_by_the_existing_first_and_last_lines() {
    exterior_custody::<f32>();
    exterior_custody::<f64>();
}
fn nested_auto_coupling<S: LayoutScalar>() {
    let s = baseline(16.0, 20.0);
    let t = baseline(12.0, 30.0);
    for direction in [Direction::Ltr, Direction::Rtl] {
        let f = Case::new(Shape::Nested, direction);
        let expected_s = expected_owner(&f.scopes[0], &s);
        let expected_t = expected_owner(&f.scopes[1], &t);
        let soft = f
            .binding
            .prepared
            .opportunity_views()
            .find(|o| o.kind() == text::PreparedBreakKind::Allowed)
            .unwrap()
            .boundary();
        let mut source_segments = f.binding.prepared.source_segments();
        let source_segment = source_segments.next().expect("fixture source segment");
        assert_eq!(source_segments.count(), 0, "one structural fixture segment");
        let incidences = f
            .binding
            .prepared
            .boundary_incidences(&source_segment, &soft)
            .unwrap();
        use text::CapturedScopeIncidence as R;
        assert_eq!(
            incidences
                .iter()
                .map(|i| (
                    f.binding
                        .prepared
                        .incidence_scope_relation(i, &f.scopes[0])
                        .unwrap(),
                    f.binding
                        .prepared
                        .incidence_scope_relation(i, &f.scopes[1])
                        .unwrap(),
                ))
                .collect::<Vec<_>>(),
            [
                (R::AtBefore, R::Before),
                (R::Inside, R::AtBefore),
                (R::Inside, R::AtAfter),
                (R::AtAfter, R::After)
            ]
        );
        // Text exposes all four true original classes. The adopted complete,
        // zero-advance content-free subtree realization admits only the two
        // outer whole-box choices; neither side's Auto declaration deletes S/T.
        let mut chosen: Option<Vec<Vec<text::LineBoxStrut>>> = None;
        for reverse in [false, true] {
            let actual = f
                .run::<S>(
                    &[(WrapEdges::AUTO, metric(&s)), (WrapEdges::AUTO, metric(&t))],
                    reverse,
                    false,
                    None,
                )
                .unwrap();
            let finalized = lines(&actual);
            let assigned = finalized
                .iter()
                .map(|line| line.selected_line().request().line_box_struts.clone())
                .collect::<Vec<_>>();
            if let Some(prior) = &chosen {
                for (current, previous) in assigned.iter().zip(prior) {
                    assert_box_struts(current, previous);
                }
            } else {
                chosen = Some(assigned.clone());
            }
            let owner = if same_box_struts(&assigned[0], &[expected_s.clone(), expected_t.clone()])
                && assigned[1].is_empty()
            {
                0
            } else {
                assert!(
                    assigned[0].is_empty(),
                    "Auto must retain one whole coupled outcome, not a Cartesian or spanning pair"
                );
                assert_box_struts(&assigned[1], &[expected_s.clone(), expected_t.clone()]);
                1
            };
            assert_geometry(
                &actual,
                &f,
                if owner == 0 {
                    [34.0, 10.0]
                } else {
                    [10.0, 34.0]
                },
                if owner == 0 {
                    [16.0, 42.0]
                } else {
                    [8.0, 26.0]
                },
                if owner == 0 {
                    [vec![expected_s.clone(), expected_t.clone()], vec![]]
                } else {
                    [vec![], vec![expected_s.clone(), expected_t.clone()]]
                },
            );
            let root = actual
                .final_entries()
                .iter()
                .find(|e| e.node() == 0)
                .unwrap()
                .output();
            assert_eq!(root.size, Size::new(S::from_f64(15.0), S::from_f64(44.0)));
        }
    }
}
#[test]
fn nested_auto_edges_keep_exactly_one_of_the_two_whole_coupled_metric_outcomes() {
    nested_auto_coupling::<f32>();
    nested_auto_coupling::<f64>();
}
