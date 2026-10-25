//! CSS Text4 §9.3: line-box geometry has a direction independent of body UBA.
use super::*;
use surgeist_text as text;

struct CachedTree<S: LayoutScalar> {
    inner: PublicLayoutTreeOf<S>,
    caches: std::collections::HashMap<u32, CacheOf<S>>,
    committed: std::collections::HashMap<u32, NodeOutputOf<S>>,
    hits: std::cell::Cell<usize>,
}

impl<S: LayoutScalar> Traverse for CachedTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::vec::IntoIter<u32>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.inner, node)
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn child_count(&self, node: u32) -> usize {
        self.inner.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.inner.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for CachedTree<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.inner.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.inner.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let result = self.caches.get(&node)?.get_with_context(input, context);
        if result.is_some() {
            self.hits.set(self.hits.get() + 1);
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.committed.get(&node).copied()
    }
}

impl<S: LayoutScalar> CachedTree<S> {
    fn new(inner: PublicLayoutTreeOf<S>) -> Self {
        Self {
            inner,
            caches: Default::default(),
            committed: Default::default(),
            hits: Default::default(),
        }
    }
    fn commit(&mut self, batch: &CompletedLayoutBatchOf<u32, S>) {
        for row in batch.cache_clear_entries() {
            self.caches.remove(&row.node());
        }
        for row in batch.cache_store_entries() {
            self.caches
                .entry(row.node())
                .or_default()
                .store_with_context(row.input(), row.context(), row.output());
        }
        self.committed = batch
            .unrounded_entries()
            .iter()
            .map(|row| (row.node(), row.output()))
            .collect();
    }
}

fn paragraph(
    node: u32,
    source: &str,
    style: text::Style,
    body: ParagraphBaseDirection,
) -> PreparedParagraph {
    PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(
            node,
            Source::new(source),
            style.clone(),
        )],
        style,
        body,
        text::ParagraphTerminalEdge::Unknown,
    )
}

fn block<S: LayoutScalar>(
    width: f64,
    direction: Direction,
    policy: LineBoxDirectionPolicy,
    indent: f64,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        direction,
        line_box_direction_policy: policy,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        text_indent: TextIndentOf {
            amount: LengthPercentageOf::px(S::from_f64(indent)).unwrap(),
            each_line: true,
            hanging: false,
        },
        ..NodeInputOf::default()
    }
}

fn compute<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: f64,
    direction: Direction,
    policy: LineBoxDirectionPolicy,
    indent: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, block(width, direction, policy, indent))
        .input(1, p.text_input(1));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![p.binding(0)]).unwrap();
    compute_layout_with_text(&capture, 0, request(width)).unwrap()
}

fn request<S: LayoutScalar>(width: f64) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(width)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}

fn glyphs<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    placed: usize,
) -> Vec<(usize, u32, f64)> {
    let mut result = Vec::new();
    for run in batch.placed_text()[placed].layout().run_views() {
        assert_eq!(
            run.face().bytes(),
            include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
        );
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
        for glyph in run.glyphs().filter(|glyph| glyph.advance() > 0.0) {
            assert_eq!(glyph.advance(), 10.0);
            result.push((
                run.line(),
                glyph.id(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
            ));
        }
    }
    result
}

fn frames<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<(text::Range, text::LineContext, bool)> {
    let mut result = Vec::new();
    for entry in batch.final_inline_fragments() {
        let line = entry.fragment().finalized_fragment().finalized_line();
        let selected = line.selected_line();
        let request = selected.request();
        let range = text::Range::new(
            request.start.view().source_after().get(),
            request.end.view().source_before().get(),
        );
        let row = (
            range,
            request.context.clone(),
            matches!(
                &request.termination,
                text::PreparedTermination::Conditional(_)
            ),
        );
        if !result.contains(&row) {
            result.push(row);
        }
    }
    result
}

fn opposed<S: LayoutScalar>() {
    let p = paragraph(
        1,
        "XX",
        ahem_style(10.0, 10.0),
        ParagraphBaseDirection::RightToLeft,
    );
    for (container, policy, expected, direction) in [
        (
            Direction::Ltr,
            LineBoxDirectionPolicy::Normal,
            vec![5.0, 15.0],
            text::LineBoxDirection::LeftToRight,
        ),
        (
            Direction::Rtl,
            LineBoxDirectionPolicy::Normal,
            vec![15.0, 25.0],
            text::LineBoxDirection::RightToLeft,
        ),
        (
            Direction::Rtl,
            LineBoxDirectionPolicy::Plaintext,
            vec![5.0, 15.0],
            text::LineBoxDirection::LeftToRight,
        ),
    ] {
        let batch = compute::<S>(&p, 40.0, container, policy, 5.0);
        assert_eq!(
            glyphs(&batch, 0)
                .into_iter()
                .map(|(_, _, x)| x)
                .collect::<Vec<_>>(),
            expected
        );
        let facts = frames(&batch);
        assert_eq!(facts.len(), 1);
        assert_eq!(
            facts[0].1.base_direction,
            ParagraphBaseDirection::RightToLeft
        );
        assert_eq!(facts[0].1.line_box_direction, direction);
        assert_eq!(
            batch.placed_text()[0].layout().source(),
            p.prepared.analyzed_source().source()
        );
    }
}

#[test]
fn normalized_policy_changes_geometry_without_changing_body_uba() {
    opposed::<f32>();
    opposed::<f64>();
}

fn conditional<S: LayoutScalar>() {
    let style = text::Style {
        wrap: Wrap::Word,
        overflow_wrap: text::OverflowWrap::Anywhere,
        line_break: text::LineBreak::Auto,
        ..ahem_style(10.0, 10.0)
    };
    let p = paragraph(1, "123A", style, ParagraphBaseDirection::LeftToRight);
    let batch = compute::<S>(
        &p,
        25.0,
        Direction::Rtl,
        LineBoxDirectionPolicy::Plaintext,
        0.0,
    );
    assert_eq!(
        glyphs(&batch, 0)
            .into_iter()
            .map(|(line, _, x)| (line, x))
            .collect::<Vec<_>>(),
        [(0, 5.0), (0, 15.0), (1, 0.0), (1, 10.0)]
    );
    let facts = frames(&batch);
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].0, text::Range::new(0, 2));
    assert!(facts[0].2);
    assert_eq!(
        facts[0].1.line_box_direction,
        text::LineBoxDirection::RightToLeft
    );
    assert_eq!(facts[1].0, text::Range::new(2, 4));
    assert_eq!(
        facts[1].1.line_box_direction,
        text::LineBoxDirection::LeftToRight
    );
}

#[test]
fn weak_conditional_prefix_resolves_again_after_a_strong_failed_ordinary_trial() {
    conditional::<f32>();
    conditional::<f64>();
}

fn chronology<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let first = paragraph(1, "A", style.clone(), ParagraphBaseDirection::LeftToRight);
    let child = paragraph(6, "12", style.clone(), ParagraphBaseDirection::LeftToRight);
    let last = paragraph(5, "12", style, ParagraphBaseDirection::LeftToRight);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3, 4, 5])
        .children(1, [])
        .children(2, [6])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(
            0,
            block(40.0, Direction::Rtl, LineBoxDirectionPolicy::Plaintext, 0.0),
        )
        .style(
            2,
            block(40.0, Direction::Rtl, LineBoxDirectionPolicy::Plaintext, 0.0),
        )
        .style(
            4,
            block(40.0, Direction::Rtl, LineBoxDirectionPolicy::Plaintext, 0.0),
        )
        .style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    text::BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                ..NodeInputOf::default()
            },
        )
        .input(1, first.text_input(1))
        .input(5, last.text_input(5))
        .input(6, child.text_input(6));
    let mut tree = CachedTree::new(tree);
    let run = |tree: &CachedTree<S>| {
        let capture = TextLayoutInput::try_new(
            tree,
            0,
            vec![first.binding(0), child.binding(2), last.binding(0)],
        )
        .unwrap();
        compute_layout_with_text(&capture, 0, request(40.0)).unwrap()
    };
    let batch = run(&tree);
    let find = |node| {
        batch
            .placed_text()
            .iter()
            .position(|p| p.nodes() == [node])
            .unwrap()
    };
    assert_eq!(
        glyphs(&batch, find(1))
            .into_iter()
            .map(|(_, _, x)| x)
            .collect::<Vec<_>>(),
        [0.0]
    );
    // A new container resets the incoming history to RTL; the parent resumes
    // its own accepted LTR history after both intervening block children.
    assert_eq!(
        glyphs(&batch, find(6))
            .into_iter()
            .map(|(_, _, x)| x)
            .collect::<Vec<_>>(),
        [20.0, 30.0]
    );
    assert_eq!(
        glyphs(&batch, find(5))
            .into_iter()
            .map(|(_, _, x)| x)
            .collect::<Vec<_>>(),
        [0.0, 10.0]
    );
    let atomic = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 3)
        .unwrap()
        .output();
    assert_eq!(atomic.location.x.to_f64(), 0.0);
    tree.commit(&batch);
    let warm = run(&tree);
    assert!(
        tree.hits.get() > 0,
        "independent atomic/block cache entries must actually be reused"
    );
    assert_eq!(warm.final_entries(), batch.final_entries());
    for i in 0..batch.placed_text().len() {
        assert_eq!(glyphs(&warm, i), glyphs(&batch, i));
    }
    assert_eq!(frames(&warm), frames(&batch));
}

#[test]
fn accepted_history_crosses_declared_runs_and_numeric_lines_but_resets_in_a_child_container() {
    chronology::<f32>();
    chronology::<f64>();
}

fn preserved_space<S: LayoutScalar>() {
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: Wrap::Word,
        ..ahem_style(10.0, 10.0)
    };
    let p = paragraph(1, "A A", style, ParagraphBaseDirection::RightToLeft);
    // Width15 minus indent5 fits the nonhanging A. The second A cannot fit;
    // SPACE remains in the first actual soft line with body-owned hanging.
    let batch = compute::<S>(
        &p,
        15.0,
        Direction::Ltr,
        LineBoxDirectionPolicy::Normal,
        5.0,
    );
    let entry = batch
        .final_inline_fragments()
        .iter()
        .find(|e| e.fragment().line_index() == 0)
        .unwrap();
    let line = entry.fragment().finalized_fragment().finalized_line();
    assert_eq!(
        line.selected_line().request().context.end,
        text::LineEnd::Soft
    );
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(line.measurements().alignment().advance(), 10.0);
    let a = glyphs(&batch, 0)
        .into_iter()
        .find(|(index, id, _)| *index == 0 && *id == 35)
        .unwrap();
    assert_eq!(a.2, 5.0);
}

#[test]
fn preserved_space_keeps_body_edge_ownership_under_opposed_line_box_alignment() {
    preserved_space::<f32>();
    preserved_space::<f64>();
}

fn intrinsic<S: LayoutScalar>(
    p: &PreparedParagraph,
    available: AvailableOf<S>,
) -> super::super::run::Output<u32, S> {
    let size = Size::new(S::from_f64(100.0), S::from_f64(10.0));
    super::super::run::layout(
        &p.binding(0),
        &Default::default(),
        &[],
        &[],
        super::super::run::Context {
            line_box_context: crate::line_box::LineBoxContext::new(
                LineBoxDirectionPolicy::Plaintext,
                Direction::Rtl,
            ),
            frame: super::super::run::Frame::Measurement,
            containing_size: size,
            inset: Point::ZERO,
            right_inset: S::ZERO,
            cursor_block: S::ZERO,
            axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
            available,
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: Default::default(),
            indent_state: Default::default(),
            hyphenation_state: Default::default(),
            hyphenate_limit_zone: LengthPercentageOf::ZERO,
            tail: super::super::fit::TailContext::Unrestricted,
        },
        &[],
        super::super::run::LineEnvironment {
            reset: |_| {},
            band: |_, _| {
                Ok(super::super::fit::Band {
                    start: 0.0,
                    end: 100.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &text::LineSelectionRequest| Ok(None),
            replay: |_: &text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
    .unwrap_or_else(|_| panic!("actual intrinsic Text run must finalize"))
}

fn independent_intrinsics<S: LayoutScalar>() {
    let style = text::Style {
        wrap: Wrap::Word,
        overflow_wrap: text::OverflowWrap::Anywhere,
        ..ahem_style(10.0, 10.0)
    };
    let p = paragraph(1, "123A", style, ParagraphBaseDirection::LeftToRight);
    let finite = || {
        compute::<S>(
            &p,
            25.0,
            Direction::Rtl,
            LineBoxDirectionPolicy::Plaintext,
            0.0,
        )
    };
    let before = finite();
    for order in [
        [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT],
        [AvailableOf::MAX_CONTENT, AvailableOf::MIN_CONTENT],
    ] {
        for available in order {
            let output = intrinsic::<S>(&p, available);
            assert!(output.paragraph.lines.iter().all(|line| {
                line.line
                    .selected_line()
                    .request()
                    .context
                    .available_inline
                    .is_none()
            }));
            if available == AvailableOf::MIN_CONTENT {
                assert_eq!(output.report.inline_extent, S::from_f64(10.0));
                let directions = output
                    .paragraph
                    .lines
                    .iter()
                    .map(|p| p.line.selected_line().request().context.line_box_direction)
                    .collect::<Vec<_>>();
                assert_eq!(
                    directions,
                    [
                        text::LineBoxDirection::RightToLeft,
                        text::LineBoxDirection::RightToLeft,
                        text::LineBoxDirection::RightToLeft,
                        text::LineBoxDirection::LeftToRight
                    ]
                );
            } else {
                assert_eq!(output.report.inline_extent, S::from_f64(40.0));
                assert_eq!(
                    output.paragraph.lines[0]
                        .line
                        .selected_line()
                        .request()
                        .context
                        .line_box_direction,
                    text::LineBoxDirection::LeftToRight
                );
            }
        }
        assert_eq!(glyphs(&finite(), 0), glyphs(&before, 0));
        assert_eq!(frames(&finite()), frames(&before));
    }
}

#[test]
fn intrinsic_chronology_is_local_and_query_order_does_not_change_finite_lines() {
    independent_intrinsics::<f32>();
    independent_intrinsics::<f64>();
}

fn forced_and_full_context<S: LayoutScalar>() {
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: Wrap::Word,
        ..ahem_style(10.0, 10.0)
    };
    let p = paragraph(
        1,
        "A\n\n12",
        style.clone(),
        ParagraphBaseDirection::LeftToRight,
    );
    let batch = compute::<S>(
        &p,
        40.0,
        Direction::Rtl,
        LineBoxDirectionPolicy::Plaintext,
        0.0,
    );
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 3);
    assert_eq!(
        glyphs(&batch, 0)
            .into_iter()
            .map(|(i, _, x)| (i, x))
            .collect::<Vec<_>>(),
        [(0, 0.0), (2, 0.0), (2, 10.0)]
    );
    assert!(
        frames(&batch).iter().all(
            |(_, context, _)| context.line_box_direction == text::LineBoxDirection::LeftToRight
        )
    );
    // RLM is actual strong semantic body text with no advance; the later A line
    // uses the full paragraph's R direction, not a fresh suffix first-strong scan.
    let p = paragraph(1, "\u{200f}A A", style, ParagraphBaseDirection::LeftToRight);
    let batch = compute::<S>(
        &p,
        15.0,
        Direction::Ltr,
        LineBoxDirectionPolicy::Plaintext,
        0.0,
    );
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 2);
    assert!(
        frames(&batch).iter().all(
            |(_, context, _)| context.line_box_direction == text::LineBoxDirection::RightToLeft
        )
    );
    assert_eq!(
        glyphs(&batch, 0)
            .into_iter()
            .filter(|(_, id, _)| *id == 35)
            .map(|(i, _, x)| (i, x))
            .collect::<Vec<_>>(),
        [(0, 5.0), (1, 5.0)]
    );
}

#[test]
fn forced_empty_lines_keep_accepted_fallback_and_later_strong_lines_use_full_context() {
    forced_and_full_context::<f32>();
    forced_and_full_context::<f64>();
}

fn numeric_overflow_cases<S: LayoutScalar>(
    containing: Direction,
    policy: LineBoxDirectionPolicy,
) -> Vec<String> {
    let prior_rtl = containing == Direction::Ltr;
    let resolved_rtl = if policy == LineBoxDirectionPolicy::Plaintext {
        prior_rtl
    } else {
        containing == Direction::Rtl
    };
    let mut failures = Vec::new();
    for indent in [-5.0, 0.0, 5.0] {
        for alignment in [
            TextAlign::Start,
            TextAlign::End,
            TextAlign::Left,
            TextAlign::Right,
            TextAlign::Center,
            TextAlign::Justify,
        ] {
            let p = paragraph(
                1,
                if prior_rtl { "\u{200f}A" } else { "A" },
                ahem_style(10.0, 10.0),
                ParagraphBaseDirection::LeftToRight,
            );
            let mut container = block::<S>(40.0, containing, policy, 0.0);
            container.text_align = alignment;
            // Genuine hanging indentation applies to the later numeric line,
            // after the accepted Text run has consumed the first line state.
            container.text_indent = TextIndentOf {
                amount: LengthPercentageOf::px(S::from_f64(indent)).unwrap(),
                each_line: false,
                hanging: true,
            };
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1, 2, 3])
                .children(1, [])
                .children(2, [])
                .children(3, [])
                .style(0, container)
                .style(2, block(40.0, containing, policy, 0.0))
                .style(
                    3,
                    NodeInputOf {
                        display: Display::InlineBlock,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(50.0)),
                            PreferredSizeOf::px(S::from_f64(10.0)),
                        ),
                        atomic_inline_participation: Some(AtomicInlineParticipation::new(
                            text::BidiLevel::try_new(0).unwrap(),
                            InlineBreakOpportunity::Allowed,
                        )),
                        ..NodeInputOf::default()
                    },
                )
                .input(1, p.text_input(1));
            let capture = TextLayoutInput::try_new(&tree, 0, vec![p.binding(0)]).unwrap();
            let batch = compute_layout_with_text(&capture, 0, request(40.0)).unwrap();
            assert_eq!(
                glyphs(&batch, 0)
                    .iter()
                    .filter(|(_, id, _)| *id == 35)
                    .count(),
                1
            );
            let prefix = frames(&batch);
            assert_eq!(prefix.len(), 1);
            assert_eq!(
                prefix[0].1.base_direction,
                ParagraphBaseDirection::LeftToRight
            );
            assert_eq!(
                prefix[0].1.line_box_direction,
                if resolved_rtl {
                    text::LineBoxDirection::RightToLeft
                } else {
                    text::LineBoxDirection::LeftToRight
                }
            );
            let atomic = batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == 3)
                .unwrap()
                .output();
            assert_eq!(atomic.size.width.to_f64(), 50.0);
            // Text4 §7.1: after any justification, overlong content is Start
            // aligned. Its physical Start is indent in LTR, or40-indent in RTL.
            // These edge equations use the actual50px box, independent of the
            // numeric fitter's free-space or body-order calculations.
            let expected = if resolved_rtl {
                40.0 - indent - 50.0
            } else {
                indent
            };
            let actual = atomic.location.x.to_f64();
            if actual != expected {
                failures.push(format!("{} {containing:?}/{policy:?} {alignment:?} indent{indent}: x{actual}, expected{expected}",std::any::type_name::<S>()));
            }
        }
    }
    failures
}

#[test]
fn numeric_overflow_after_rtl_history_uses_rtl_start_in_an_ltr_container() {
    let mut failures =
        numeric_overflow_cases::<f32>(Direction::Ltr, LineBoxDirectionPolicy::Plaintext);
    failures.extend(numeric_overflow_cases::<f64>(
        Direction::Ltr,
        LineBoxDirectionPolicy::Plaintext,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn numeric_overflow_after_ltr_history_uses_ltr_start_in_an_rtl_container() {
    let mut failures =
        numeric_overflow_cases::<f32>(Direction::Rtl, LineBoxDirectionPolicy::Plaintext);
    failures.extend(numeric_overflow_cases::<f64>(
        Direction::Rtl,
        LineBoxDirectionPolicy::Plaintext,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn numeric_overflow_normal_preserves_containing_start_for_all_alignments() {
    let mut failures = Vec::new();
    for containing in [Direction::Ltr, Direction::Rtl] {
        failures.extend(numeric_overflow_cases::<f32>(
            containing,
            LineBoxDirectionPolicy::Normal,
        ));
        failures.extend(numeric_overflow_cases::<f64>(
            containing,
            LineBoxDirectionPolicy::Normal,
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
