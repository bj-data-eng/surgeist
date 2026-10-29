//! Resolved post-baseline offsets through real registered Text and child boxes.
//! Ahem's retained UPEM1000/ascent800/descent-200/advance1000 are the oracle.
//! The numeric-only child6/4 example is deliberately not claimed for this font.
use super::*;
use std::cell::RefCell;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn table(tag: [u8; 4]) -> &'static [u8] {
    let record = (0..usize::from(word(AHEM, 4)))
        .map(|i| 12 + 16 * i)
        .find(|&at| AHEM[at..at + 4] == tag)
        .unwrap();
    let at = usize::try_from(dword(AHEM, record + 8)).unwrap();
    let len = usize::try_from(dword(AHEM, record + 12)).unwrap();
    &AHEM[at..at + len]
}
fn premises() {
    assert_eq!(word(table(*b"head"), 18), 1000);
    assert_eq!(word(table(*b"hmtx"), 58 * 4), 1000);
    let hhea = table(*b"hhea");
    let signed = |at| i16::from_be_bytes(hhea[at..at + 2].try_into().unwrap());
    assert_eq!((signed(4), signed(6), signed(8)), (800, -200, 0));
    assert_eq!(word(table(*b"OS/2"), 62) & 0x80, 0);
    // These actual font offsets are distinct from the selected native parent
    // size fallback. A parent20 would give2.86/9.06 if OS/2 were used instead.
    assert_eq!(
        (word(table(*b"OS/2"), 16), word(table(*b"OS/2"), 24)),
        (143, 453)
    );
}
fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn px<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::px(n(value))
}
fn vertical(mode: WritingMode) -> bool {
    mode != WritingMode::HorizontalTb
}
fn name(mode: WritingMode) -> BaselineName {
    if matches!(mode, WritingMode::VerticalRl | WritingMode::VerticalLr) {
        BaselineName::Central
    } else {
        BaselineName::Alphabetic
    }
}
fn size<S: LayoutScalar>(mode: WritingMode, inline: f64, cross: f64) -> Size<S> {
    if vertical(mode) {
        Size::new(n(cross), n(inline))
    } else {
        Size::new(n(inline), n(cross))
    }
}

struct Native {
    prepared: text::PreparedText,
    original: text::Source,
}
impl Native {
    fn binding(&self, container: u32, text_node: u32) -> TextParagraphBinding<u32> {
        let mut nodes = vec![text_node];
        let atomics = self
            .prepared
            .atomic_views()
            .map(|view| {
                let node = u32::try_from(view.original_box().id().as_u64()).unwrap();
                nodes.push(node);
                TextAtomicBinding {
                    occurrence: view.handle(),
                    node,
                }
            })
            .collect();
        TextParagraphBinding {
            prepared: self.prepared.clone(),
            container,
            nodes,
            origins: vec![TextOriginBinding {
                origin: self.prepared.origin_views().next().unwrap().handle(),
                node: text_node,
            }],
            atomics,
            controls: vec![],
            line_strut: None,
        }
    }
    fn input<S: LayoutScalar>(&self) -> LayoutInputOf<S> {
        LayoutInputOf::inline_text(TextParticipation::new(
            self.prepared.origin_views().next().unwrap().handle(),
        ))
    }
}

fn native(
    raw: &str,
    font_size: f32,
    line_height: f64,
    mode: WritingMode,
    boxes: &[(u32, f64, f64)],
) -> Native {
    let original = Source::new(raw);
    let mut source = original.clone();
    for &(node, inline, cross) in boxes {
        let physical = size::<f64>(mode, inline, cross);
        source.inline_box(text::InlineBox::new(
            text::Id::from_u64(u64::from(node)),
            text::InlineBoxKind::InFlow,
            original.text().len(),
            text::Size::new(physical.width, physical.height),
        ));
    }
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::None,
        ..ahem_style(font_size, line_height)
    };
    let extent = text::Range::new(0, original.text().len());
    let typography = if name(mode) == BaselineName::Central {
        text::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::LeftToRight,
        }
    } else {
        text::AnalysisTypography::HorizontalComposition
    };
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source,
            base_style: style,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: original.clone(),
                local: extent,
                aggregate: extent,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: (0..boxes.len())
                .flat_map(|index| {
                    [
                        text::ParagraphBoundaryDefinition::AtomicBefore(index),
                        text::ParagraphBoundaryDefinition::AtomicAfter(index),
                    ]
                })
                .map(|definition| text::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: text::Wrap::None,
                    following_wrap: text::Wrap::None,
                    ancestor_wrap: text::Wrap::None,
                })
                .collect(),
            anchor_order: (0..boxes.len())
                .map(|index| text::ParagraphAnchor {
                    item: text::ParagraphAnchorItem::Atomic(index),
                    affinity: text::Affinity::Before,
                })
                .collect(),
        },
        typography,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::None);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let orientation = match mode {
        WritingMode::HorizontalTb => text::LineOrientation::Horizontal,
        WritingMode::SidewaysRl => text::LineOrientation::SidewaysClockwise,
        WritingMode::VerticalRl | WritingMode::VerticalLr => text::LineOrientation::Vertical,
        WritingMode::SidewaysLr => text::LineOrientation::SidewaysCounterclockwise,
    };
    let mut options = text::PreparationOptions::default().with_orientation(orientation);
    if name(mode) == BaselineName::Alphabetic {
        options = options.with_baseline_reference(text::FontBaseline::Alphabetic);
    }
    Native {
        prepared: system.prepare(&analyzed, options).unwrap(),
        original,
    }
}

fn root<S: LayoutScalar>(mode: WritingMode) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode: mode,
        inline_baseline_context: InlineBaselineContext::Named(name(mode)),
        size: if vertical(mode) {
            Size::new(PreferredSizeOf::AUTO, px(100.0))
        } else {
            Size::new(px(100.0), PreferredSizeOf::AUTO)
        },
        ..NodeInputOf::default()
    }
}
fn child<S: LayoutScalar>(
    mode: WritingMode,
    cross: f64,
    source: BaselineSource,
    alignment: VerticalAlignOf<S>,
) -> NodeInputOf<S> {
    let physical = size::<f64>(mode, 20.0, cross);
    NodeInputOf {
        display: Display::InlineBlock,
        writing_mode: mode,
        inline_baseline_context: InlineBaselineContext::Named(name(mode)),
        baseline_source: source,
        vertical_align: alignment,
        size: Size::new(px(physical.width), px(physical.height)),
        ..NodeInputOf::default()
    }
}
fn tree<S: LayoutScalar>(
    mode: WritingMode,
    parent: &Native,
    inner: &Native,
    cross: f64,
    source: BaselineSource,
    alignment: VerticalAlignOf<S>,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [11])
        .children(11, [])
        .style(0, root(mode))
        .input(1, parent.input())
        .style(2, child(mode, cross, source, alignment))
        .input(11, inner.input())
}
fn solve<Tree: LayoutTree<Node = u32>>(
    tree: &Tree,
    mode: WritingMode,
    bindings: Vec<TextParagraphBinding<u32>>,
) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
{
    let flow = LayoutFlowInput::try_new(tree, 0, bindings, crate::WrappingInput::default())?;
    let available = if vertical(mode) {
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::definite(n(100.0)))
    } else {
        Size::new(AvailableOf::definite(n(100.0)), AvailableOf::MAX_CONTENT)
    };
    compute_layout_with_flow(&flow, 0, LayoutRootRequestOf::viewport(available).unwrap())
}
fn placed<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> &text::Layout {
    batch
        .placed_text()
        .iter()
        .find(|row| row.container() == container)
        .unwrap()
        .layout()
}
fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|row| row.node() == node)
        .unwrap()
        .output()
}
fn finalized<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> text::FinalizedLine {
    let layout = placed(batch, container);
    batch
        .final_inline_fragments()
        .iter()
        .map(|row| row.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.prepared_text().analyzed_source().identity() == layout.analyzed_source().identity()
        })
        .unwrap()
}
fn fact<S: LayoutScalar>(value: BaselineFactOf<S>, axis: PhysicalAxis, offset: f64) {
    assert_eq!(
        value,
        BaselineFactOf::Available(BaselineCoordinateOf::try_new(axis, n(offset)).unwrap())
    );
}
fn raw(layout: &text::Layout, font_size: f32, advance: f32) {
    let mut glyphs = 0;
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.font_size(), font_size);
        assert!(
            run.effective_instance()
                .unwrap()
                .normalized_coords()
                .next()
                .is_none()
        );
        for glyph in run.glyphs() {
            assert_eq!((glyph.id(), glyph.advance()), (58, advance));
            glyphs += 1;
        }
    }
    assert!(glyphs > 0);
}
fn custody(layout: &text::Layout, original: &Source) {
    assert_eq!(layout.source().text(), original.text());
    let endpoint = |index, affinity| {
        layout
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(index, affinity))
            .unwrap()
    };
    let selection = layout
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(original.text().len(), text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selection.source_text(), original.text());
    assert_eq!(selection.plaintext(), original.text());
}
fn same_geometry<S: LayoutScalar>(
    a: &CompletedLayoutBatchOf<u32, S>,
    b: &CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(a.final_entries(), b.final_entries());
    let fragments = |batch: &CompletedLayoutBatchOf<u32, S>| {
        batch
            .final_inline_fragments()
            .iter()
            .map(|row| {
                let fragment = row.fragment();
                let issued = fragment.finalized_fragment();
                (
                    row.owner().clone(),
                    issued.view().local_range(),
                    issued.view().aggregate_range(),
                    fragment.rect(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(fragments(a), fragments(b));
    for entry in a.placed_text() {
        let other = placed(b, entry.container());
        assert_eq!(entry.layout().source(), other.source());
        assert_eq!(entry.layout().lines(), other.lines());
        assert_eq!(entry.layout().clusters(), other.clusters());
        assert_eq!(entry.layout().inline_boxes(), other.inline_boxes());
    }
}

fn alpha<S: LayoutScalar>() {
    premises();
    for mode in [WritingMode::HorizontalTb, WritingMode::SidewaysRl] {
        let parent = native("X", 10.0, 10.0, mode, &[(2, 20.0, 20.0)]);
        let inner = native("X\nX", 10.0, 10.0, mode, &[]);
        for (source, before, after, local_first, local_last) in [
            (BaselineSource::First, 8.0, 12.0, 8.0, 18.0),
            (BaselineSource::Last, 18.0, 2.0, 8.0, 18.0),
            (BaselineSource::Auto, 18.0, 2.0, 8.0, 18.0),
        ] {
            for delta in [-3.0, 0.0, 3.0, 30.0] {
                let offset = BaselineOffsetOf::try_new(n(delta)).unwrap();
                let tree = tree::<S>(
                    mode,
                    &parent,
                    &inner,
                    20.0,
                    source,
                    VerticalAlignOf::Shift(offset),
                );
                let batch = solve(
                    &tree,
                    mode,
                    vec![parent.binding(0, 1), inner.binding(2, 11)],
                )
                .unwrap();
                // The selected child Alpha is8 or18, not the parent baseline.
                // Add Delta only after this choice, retaining signed after.
                let baseline = 8.0_f64.max(before + delta);
                let height = baseline + 2.0_f64.max(after - delta);
                let parent_text = placed(&batch, 0);
                let feedback = finalized(&batch, 0).feedback();
                assert_eq!(
                    (feedback.baseline, feedback.line_height),
                    (baseline, height)
                );
                assert_eq!(output(&batch, 0).size, size(mode, 100.0, height));
                let child_box = output(&batch, 2);
                assert_eq!(child_box.size, size(mode, 20.0, 20.0));
                let axis = if vertical(mode) {
                    PhysicalAxis::Horizontal
                } else {
                    PhysicalAxis::Vertical
                };
                let pair = child_box.named_baselines.get(BaselineName::Alphabetic);
                if vertical(mode) {
                    fact(pair.first(), axis, 20.0 - local_first);
                    fact(pair.last(), axis, 20.0 - local_last);
                    let selected = if source == BaselineSource::First {
                        20.0 - local_first
                    } else {
                        20.0 - local_last
                    };
                    assert_eq!(
                        child_box.location.x.to_f64() + selected,
                        parent_text.lines()[0].baseline() + delta
                    );
                } else {
                    fact(pair.first(), axis, local_first);
                    fact(pair.last(), axis, local_last);
                    let selected = if source == BaselineSource::First {
                        local_first
                    } else {
                        local_last
                    };
                    assert_eq!(
                        child_box.location.y.to_f64() + selected,
                        parent_text.lines()[0].baseline() - delta
                    );
                }
                let allocation =
                    finalized(&batch, 0).selected_line().request().atomics[0].allocation;
                let text::AtomicLineAllocation::InFlow { alignment, .. } = allocation else {
                    panic!("the bound child is an actual in-flow box")
                };
                assert_eq!(
                    alignment,
                    text::LineBoxAlignment::Shift(text::BaselineShift::try_new(delta).unwrap())
                );
                let natural = finalized(&batch, 0);
                let replay = natural
                    .with_spacing(text::SpacingAllocations::None)
                    .unwrap();
                assert_eq!(replay.selected_line(), natural.selected_line());
                assert_eq!(replay.feedback(), natural.feedback());
                raw(parent_text, 10.0, 10.0);
                raw(placed(&batch, 2), 10.0, 10.0);
                custody(placed(&batch, 2), &inner.original);
                if delta == 0.0 {
                    let ordinary_tree = tree
                        .clone()
                        .style(2, child(mode, 20.0, source, VerticalAlignOf::Baseline));
                    let ordinary = solve(
                        &ordinary_tree,
                        mode,
                        vec![parent.binding(0, 1), inner.binding(2, 11)],
                    )
                    .unwrap();
                    same_geometry(&ordinary, &batch);
                    assert_ne!(
                        finalized(&ordinary, 0).selected_line().request().atomics,
                        finalized(&batch, 0).selected_line().request().atomics
                    );
                }
                let warm = solve(
                    &tree,
                    mode,
                    vec![parent.binding(0, 1), inner.binding(2, 11)],
                )
                .unwrap();
                same_geometry(&warm, &batch);
            }
        }
    }
}

#[test]
fn real_alphabetic_first_last_and_auto_are_matched_before_signed_offset_in_both_scalar_lanes() {
    alpha::<f32>();
    alpha::<f64>();
}

fn central<S: LayoutScalar>() {
    premises();
    for mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let parent = native("X", 10.0, 10.0, mode, &[(2, 20.0, 20.0)]);
        let inner = native("X", 10.0, 10.0, mode, &[]);
        for delta in [-3.0, 0.0, 3.0] {
            let tree = tree::<S>(
                mode,
                &parent,
                &inner,
                20.0,
                BaselineSource::First,
                VerticalAlignOf::Shift(BaselineOffsetOf::try_new(n(delta)).unwrap()),
            );
            let batch = solve(
                &tree,
                mode,
                vec![parent.binding(0, 1), inner.binding(2, 11)],
            )
            .unwrap();
            // Ahem's absence-synthesized upright em is5/5. A20px child has
            // actual Central15 in VerticalRl and5 in VerticalLr, not center10.
            let child_c = if mode == WritingMode::VerticalRl {
                15.0
            } else {
                5.0
            };
            let over = 20.0 - child_c;
            let under = child_c;
            let before = 5.0_f64.max(over + delta);
            let after = 5.0_f64.max(under - delta);
            let line = finalized(&batch, 0);
            assert_eq!(
                (line.feedback().baseline, line.feedback().line_height),
                (before, before + after)
            );
            let child_box = output(&batch, 2);
            assert_eq!(child_box.size, size(mode, 20.0, 20.0));
            let pair = child_box.named_baselines.get(BaselineName::Central);
            fact(pair.first(), PhysicalAxis::Horizontal, child_c);
            fact(pair.last(), PhysicalAxis::Horizontal, child_c);
            // Positive line-over is physical right in both vertical flows.
            assert_eq!(
                child_box.location.x.to_f64() + child_c,
                placed(&batch, 0).lines()[0].baseline() + delta
            );
            raw(placed(&batch, 0), 10.0, 10.0);
            raw(placed(&batch, 2), 10.0, 10.0);
            custody(placed(&batch, 2), &inner.original);
        }
    }
}
#[test]
fn real_central_child_is_shifted_toward_physical_over_in_both_vertical_block_directions() {
    central::<f32>();
    central::<f64>();
}

fn normalized<S: LayoutScalar>() {
    premises();
    let mode = WritingMode::HorizontalTb;
    for (parent_size, delta, baseline, height, top) in [
        (20.0, -4.0, 16.0, 22.0, 12.0),
        (30.0, 10.0, 24.0, 30.0, 6.0),
    ] {
        let parent = native(
            "X",
            parent_size as f32,
            parent_size,
            mode,
            &[(2, 20.0, 10.0)],
        );
        let inner = native("X", 10.0, 10.0, mode, &[]);
        let used_parent = NonNegativeFiniteOf::new(n(parent_size)).unwrap();
        let offset = if delta < 0.0 {
            BaselineOffsetOf::subscript(used_parent).unwrap()
        } else {
            BaselineOffsetOf::superscript(used_parent).unwrap()
        };
        assert_eq!(offset.get(), n(delta));
        let tree = tree::<S>(
            mode,
            &parent,
            &inner,
            10.0,
            BaselineSource::Auto,
            VerticalAlignOf::Shift(offset),
        );
        let batch = solve(
            &tree,
            mode,
            vec![parent.binding(0, 1), inner.binding(2, 11)],
        )
        .unwrap();
        assert_eq!(
            (
                finalized(&batch, 0).feedback().baseline,
                finalized(&batch, 0).feedback().line_height
            ),
            (baseline, height)
        );
        assert_eq!(output(&batch, 2).location.y, n(top));
        assert_eq!(output(&batch, 2).size, Size::new(n(20.0), n(10.0)));
        fact(
            output(&batch, 2)
                .named_baselines
                .get(BaselineName::Alphabetic)
                .first(),
            PhysicalAxis::Vertical,
            8.0,
        );
        raw(placed(&batch, 0), parent_size as f32, parent_size as f32);
        raw(placed(&batch, 2), 10.0, 10.0);
    }
    let small = native("X", 10.0, 18.0, mode, &[]);
    let tall = native("X", 10.0, 60.0, mode, &[]);
    let parent = native("X", 10.0, 10.0, mode, &[(2, 20.0, 18.0), (3, 20.0, 60.0)]);
    let offset = BaselineOffsetOf::from_length_percentage(
        LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap(),
        PercentageBasisOf::definite(n(18.0)).unwrap(),
    )
    .unwrap();
    assert_eq!(offset.get(), n(9.0));
    let tree = tree::<S>(
        mode,
        &parent,
        &small,
        18.0,
        BaselineSource::Auto,
        VerticalAlignOf::Shift(offset),
    )
    .children(0, [1, 2, 3])
    .children(3, [12])
    .children(12, [])
    .style(
        3,
        child(mode, 60.0, BaselineSource::Auto, VerticalAlignOf::Baseline),
    )
    .input(12, tall.input());
    let batch = solve(
        &tree,
        mode,
        vec![
            parent.binding(0, 1),
            small.binding(2, 11),
            tall.binding(3, 12),
        ],
    )
    .unwrap();
    // Child own H18 gives baseline8+4=12 and shift9. The sibling H60 has
    // baseline8+25=33 and after27, setting the actual common60px envelope.
    assert_eq!(
        (
            finalized(&batch, 0).feedback().baseline,
            finalized(&batch, 0).feedback().line_height
        ),
        (33.0, 60.0)
    );
    assert_eq!(output(&batch, 2).location.y, n(12.0));
    assert_eq!(output(&batch, 2).size.height, n(18.0));
    assert_eq!(output(&batch, 3).location.y, S::ZERO);
    assert_eq!(placed(&batch, 2).lines()[0].baseline(), 24.0);
    fact(
        output(&batch, 2)
            .named_baselines
            .get(BaselineName::Alphabetic)
            .first(),
        PhysicalAxis::Vertical,
        12.0,
    );
    raw(placed(&batch, 2), 10.0, 10.0);
    custody(placed(&batch, 2), &small.original);
}
#[test]
fn authentic_parent_size_fallback_and_child_own_height_resolution_preserve_font_and_raw_baselines()
{
    normalized::<f32>();
    normalized::<f64>();
}

struct ShapeTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    bands: RefCell<Vec<(S, S)>>,
}
impl<S: LayoutScalar> Traverse for ShapeTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for ShapeTree<S> {
    type MeasureError = &'static str;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 9);
        assert_eq!(query.margin_box().size(), Size::new(n(30.0), n(20.0)));
        self.bands
            .borrow_mut()
            .push((query.band_minimum(), query.band_maximum()));
        // A fixed rectangle x0..30/y10..20 inside this real margin box. It is
        // absent from a0..10 line band and intersects a0..11 shifted envelope.
        let origin = query.margin_box().origin();
        if query.band_maximum() <= origin.y + n(10.0) || query.band_minimum() >= origin.y + n(20.0)
        {
            Some(Ok(None))
        } else {
            Some(Ok(FloatExclusionIntervalOf::try_new(
                query,
                origin.x,
                origin.x + n(30.0),
            )
            .unwrap()))
        }
    }
}
struct Publication<S: LayoutScalar> {
    batch: Option<CompletedLayoutBatchOf<u32, S>>,
    reject: bool,
}
impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Publication<S> {
    type Error = &'static str;
    type Prepared = CompletedLayoutBatchOf<u32, S>;
    fn prepare_layout_batch(&self, batch: &Self::Prepared) -> Result<Self::Prepared, Self::Error> {
        if self.reject {
            Err("reject baseline-offset packet")
        } else {
            Ok(batch.clone())
        }
    }
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.batch = Some(prepared);
    }
}
fn float_and_publication<S: LayoutScalar>() {
    premises();
    let mode = WritingMode::HorizontalTb;
    let parent = native("X", 10.0, 10.0, mode, &[(2, 20.0, 10.0)]);
    let inner = native("X", 10.0, 10.0, mode, &[]);
    let base = tree::<S>(
        mode,
        &parent,
        &inner,
        10.0,
        BaselineSource::Auto,
        VerticalAlignOf::Baseline,
    )
    .children(0, [9, 1, 2])
    .children(9, [])
    .style(
        9,
        NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(px(30.0), px(20.0)),
            ..NodeInputOf::default()
        },
    );
    let host = ShapeTree {
        tree: base.clone(),
        bands: RefCell::new(vec![]),
    };
    let ordinary = solve(
        &host,
        mode,
        vec![parent.binding(0, 1), inner.binding(2, 11)],
    )
    .unwrap();
    assert_eq!(finalized(&ordinary, 0).feedback().line_height, 10.0);
    assert_eq!(
        placed(&ordinary, 0).run_views().next().unwrap().origin().x,
        0.0
    );
    let shifted_tree = base.clone().style(
        2,
        child(
            mode,
            10.0,
            BaselineSource::Auto,
            VerticalAlignOf::Shift(BaselineOffsetOf::try_new(n(1.0)).unwrap()),
        ),
    );
    let shifted_host = ShapeTree {
        tree: shifted_tree.clone(),
        bands: RefCell::new(vec![]),
    };
    let shifted = solve(
        &shifted_host,
        mode,
        vec![parent.binding(0, 1), inner.binding(2, 11)],
    )
    .unwrap();
    assert_eq!(
        (
            finalized(&shifted, 0).feedback().baseline,
            finalized(&shifted, 0).feedback().line_height
        ),
        (9.0, 11.0)
    );
    assert_eq!(
        placed(&shifted, 0).run_views().next().unwrap().origin().x,
        30.0
    );
    assert_eq!(output(&shifted, 2).location, Point::new(n(40.0), S::ZERO));
    assert_eq!(
        finalized(&shifted, 0)
            .selected_line()
            .request()
            .context
            .available_inline,
        Some(70.0)
    );
    assert!(
        shifted_host
            .bands
            .borrow()
            .iter()
            .any(|&(start, end)| start == S::ZERO && end == n(11.0))
    );
    raw(placed(&shifted, 2), 10.0, 10.0);
    custody(placed(&shifted, 2), &inner.original);
    let mut sink = Publication {
        batch: None,
        reject: false,
    };
    ordinary.apply_to(&mut sink).unwrap();
    let prior = sink.batch.as_ref().unwrap().clone();
    sink.reject = true;
    assert_eq!(
        shifted.apply_to(&mut sink),
        Err("reject baseline-offset packet")
    );
    assert_eq!(
        sink.batch.as_ref().unwrap().final_entries(),
        prior.final_entries()
    );
    assert_eq!(
        sink.batch.as_ref().unwrap().final_inline_fragments(),
        prior.final_inline_fragments()
    );
    assert_eq!(
        placed(sink.batch.as_ref().unwrap(), 0).lines(),
        placed(&prior, 0).lines()
    );
    sink.reject = false;
    shifted.apply_to(&mut sink).unwrap();
    assert_eq!(
        sink.batch.as_ref().unwrap().final_entries(),
        shifted.final_entries()
    );
    let repeat = solve(
        &shifted_host,
        mode,
        vec![parent.binding(0, 1), inner.binding(2, 11)],
    )
    .unwrap();
    same_geometry(&repeat, &shifted);
    // This finite displacement is exactly representable in either scalar.
    // At 2^60, f64 itself cannot retain the nonzero 8/2 metric contributions.
    // Refuse at the owning numeric boundary before a replacement can publish.
    let prior = sink.batch.as_ref().unwrap().clone();
    let huge = 1_152_921_504_606_846_976.0;
    let extreme = ShapeTree {
        tree: base.style(
            2,
            child(
                mode,
                10.0,
                BaselineSource::Auto,
                VerticalAlignOf::Shift(BaselineOffsetOf::try_new(n(huge)).unwrap()),
            ),
        ),
        bands: RefCell::new(vec![]),
    };
    let error = match solve(
        &extreme,
        mode,
        vec![parent.binding(0, 1), inner.binding(2, 11)],
    ) {
        Ok(_) => panic!("unrepresentable baseline displacement must refuse before publication"),
        Err(error) => error,
    };
    match error.kind() {
        LayoutErrorKindOf::Text(error) => assert!(
            matches!(
                error.detail(),
                Some(text::ErrorDetail::InvalidTextNumeric {
                    category: text::TextNumericError::NonZeroLost,
                    ..
                })
            ),
            "{error:?}"
        ),
        LayoutErrorKindOf::TextNumeric {
            reason: LayoutTextNumericError::NonZeroLost,
            ..
        } => {}
        other => {
            panic!("actual nonzero contribution loss must retain its numeric cause: {other:?}")
        }
    }
    same_geometry(sink.batch.as_ref().unwrap(), &prior);
}
#[test]
fn shifted_real_text_envelope_changes_float_band_and_failed_sink_preserves_prior_packet() {
    float_and_publication::<f32>();
    float_and_publication::<f64>();
}

fn counterclockwise<S: LayoutScalar>() {
    premises();
    let supported_parent = native("X", 10.0, 10.0, WritingMode::SidewaysRl, &[(2, 20.0, 10.0)]);
    let supported_inner = native("X", 10.0, 10.0, WritingMode::SidewaysRl, &[]);
    let supported_tree = tree::<S>(
        WritingMode::SidewaysRl,
        &supported_parent,
        &supported_inner,
        10.0,
        BaselineSource::Auto,
        VerticalAlignOf::Baseline,
    );
    let prior = solve(
        &supported_tree,
        WritingMode::SidewaysRl,
        vec![
            supported_parent.binding(0, 1),
            supported_inner.binding(2, 11),
        ],
    )
    .unwrap();
    let mut sink = Publication {
        batch: None,
        reject: false,
    };
    prior.apply_to(&mut sink).unwrap();

    let parent = native("X", 10.0, 10.0, WritingMode::SidewaysLr, &[(2, 20.0, 10.0)]);
    let inner = native("X", 10.0, 10.0, WritingMode::SidewaysLr, &[]);
    for native in [&parent, &inner] {
        assert_eq!(
            native.prepared.orientation(),
            text::LineOrientation::SidewaysCounterclockwise
        );
        assert_eq!(
            native.prepared.baseline_reference(),
            Some(text::FontBaseline::Alphabetic)
        );
    }
    let tree = tree::<S>(
        WritingMode::SidewaysLr,
        &parent,
        &inner,
        10.0,
        BaselineSource::Auto,
        VerticalAlignOf::Shift(BaselineOffsetOf::try_new(n(3.0)).unwrap()),
    );
    let error = solve(
        &tree,
        WritingMode::SidewaysLr,
        vec![parent.binding(0, 1), inner.binding(2, 11)],
    )
    .unwrap_err();
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::TextWritingMode(
                WritingMode::SidewaysLr
            ))
        ),
        "{error:?}"
    );
    same_geometry(sink.batch.as_ref().unwrap(), &prior);
}
#[test]
fn resolved_shift_does_not_claim_counterclockwise_native_text_support() {
    counterclockwise::<f32>();
    counterclockwise::<f64>();
}

// C is exactly representable in both scalars. Around C, adjacent f32 values
// are eight units apart: C+4 and C+3 both narrow to C, while C+8 and C+7 both
// narrow to C+8. All four coordinates already survive integer snapping.
const PRECISION_CURSOR: f64 = 100_000_000.0;

fn precision_native(control: bool) -> (Native, TextParagraphBinding<u32>) {
    if !control {
        let native = native(
            "X",
            10.0,
            10.0,
            WritingMode::HorizontalTb,
            &[(2, 20.0, 4.0)],
        );
        let binding = native.binding(10, 1);
        return (native, binding);
    }
    let original = Source::new("X");
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, original.clone(), style.clone()),
            ParagraphPart::Control {
                node: 2,
                kind: text::ParagraphControlKind::ForcedBreak,
            },
        ],
        style,
        ParagraphBaseDirection::LeftToRight,
    );
    // Reprepare this actual captured source with the same explicit named
    // reference as the atomic fixture; retain only the newly issued handles.
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(
            &paragraph.prepared.analyzed_source(),
            text::PreparationOptions::default()
                .with_baseline_reference(text::FontBaseline::Alphabetic),
        )
        .unwrap();
    let native = Native { prepared, original };
    let mut binding = native.binding(10, 1);
    binding.nodes.push(2);
    binding.controls.push(TextControlBinding {
        occurrence: native.prepared.control_views().next().unwrap().handle(),
        node: 2,
    });
    (native, binding)
}

fn precision_participant<S: LayoutScalar>(
    control: bool,
    alignment: VerticalAlignOf<S>,
) -> LayoutInputOf<S> {
    if control {
        // Actual normalized four-unit used control metrics, not supplied font
        // baseline facts. Before/after 4/0 stays inside Ahem's actual 8/2 line.
        let metrics = InlineMetricsOf::try_new(
            InlineBaselineContext::Named(BaselineName::Alphabetic),
            n(4.0),
            n(4.0),
        )
        .unwrap();
        LayoutInputOf::line_break(
            LineBreakInputOf::new()
                .with_metrics(metrics)
                .with_vertical_align(alignment),
        )
    } else {
        // No child source, measurement callback or supplied baseline facts.
        // Native layout must establish genuine absence before synthesizing
        // the empty inline-block's bottom Alphabetic baseline at four units.
        LayoutInputOf::box_input(child(
            WritingMode::HorizontalTb,
            4.0,
            BaselineSource::Auto,
            alignment,
        ))
    }
}

fn precision_tree<S: LayoutScalar>(
    native: &Native,
    control: bool,
    delta: f64,
    spacer: f64,
    translated_parent: bool,
) -> PublicLayoutTreeOf<S> {
    let alignment = if delta == 0.0 {
        VerticalAlignOf::Baseline
    } else {
        VerticalAlignOf::Shift(BaselineOffsetOf::try_new(n(delta)).unwrap())
    };
    let tree = PublicLayoutTreeOf::new()
        .children(1, [])
        .children(2, [])
        .children(9, [])
        .style(0, root(WritingMode::HorizontalTb))
        .style(10, root(WritingMode::HorizontalTb))
        .style(
            9,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(px(100.0), px(spacer)),
                ..NodeInputOf::default()
            },
        )
        .input(1, native.input())
        .input(2, precision_participant(control, alignment));
    if translated_parent {
        tree.children(0, [9, 10]).children(10, [1, 2])
    } else {
        tree.children(0, [10]).children(10, [9, 1, 2])
    }
}

fn precision_success<S: LayoutScalar>(control: bool, spacer: f64, translated_parent: bool) {
    premises();
    let (native, binding) = precision_native(control);
    let local_cursor = if translated_parent { 0.0 } else { spacer };
    let global_origin = if translated_parent { spacer } else { 0.0 };
    let raw_reference = if control { 8.0 } else { 4.0 };
    for delta in [0.0, 1.0] {
        let tree = precision_tree::<S>(&native, control, delta, spacer, translated_parent);
        let batch = solve(&tree, WritingMode::HorizontalTb, vec![binding.clone()]).unwrap();
        let line = finalized(&batch, 10);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0, 10.0)
        );
        assert_eq!(output(&batch, 10).location.y, n(global_origin));
        assert_eq!(
            output(&batch, 2).location.y,
            n(local_cursor + raw_reference - delta)
        );
        assert_eq!(placed(&batch, 10).lines()[0].baseline(), spacer + 8.0);
        raw(placed(&batch, 10), 10.0, 10.0);
        custody(placed(&batch, 10), &native.original);
        if control {
            assert_eq!(line.control_slots().next().unwrap().slot().y, 8.0 - delta);
            let strut = line.selected_line().request().controls[0].strut.unwrap();
            assert_eq!((strut.baseline, strut.line_extent), (4.0, 4.0));
        } else {
            let pair = output(&batch, 2)
                .named_baselines
                .get(BaselineName::Alphabetic);
            assert_eq!(
                (pair.first(), pair.last()),
                (BaselineFactOf::Absent, BaselineFactOf::Absent)
            );
            let text::AtomicLineAllocation::InFlow {
                flow, box_geometry, ..
            } = line.selected_line().request().atomics[0].allocation
            else {
                panic!("real in-flow child")
            };
            assert_eq!((flow.before_baseline, flow.after_baseline), (4.0, 0.0));
            // Box geometry is relative to the actual flow slot, rather than
            // the border top. Its raw border-to-baseline distance stays four.
            assert_eq!(box_geometry.baseline - box_geometry.border.origin.y, 4.0);
            assert_eq!(
                placed(&batch, 10).inline_boxes()[0].rect().origin.y,
                spacer + 4.0 - delta
            );
            let unrounded = batch
                .unrounded_entries()
                .iter()
                .find(|row| row.node() == 2)
                .unwrap()
                .output();
            assert_eq!(unrounded.size, Size::new(n(20.0), n(4.0)));
        }
    }
}

fn precision_refusal_preserves_publication(control: bool) {
    premises();
    let (native, binding) = precision_native(control);
    let valid = precision_tree::<f32>(&native, control, 0.0, PRECISION_CURSOR, false);
    let accepted = solve(&valid, WritingMode::HorizontalTb, vec![binding.clone()]).unwrap();
    assert_eq!(
        (
            finalized(&accepted, 10).feedback().baseline,
            finalized(&accepted, 10).feedback().line_height
        ),
        (8.0, 10.0)
    );
    raw(placed(&accepted, 10), 10.0, 10.0);
    let zero_tree = valid.clone().input(
        2,
        precision_participant(control, VerticalAlignOf::Shift(BaselineOffsetOf::ZERO)),
    );
    let zero = solve(&zero_tree, WritingMode::HorizontalTb, vec![binding.clone()]).unwrap();
    same_geometry(&zero, &accepted);
    let mut sink = Publication {
        batch: None,
        reject: false,
    };
    accepted.apply_to(&mut sink).unwrap();
    let prior = sink.batch.as_ref().unwrap().clone();
    let large_cursor = precision_tree::<f32>(&native, control, 1.0, PRECISION_CURSOR, false);
    let error = match solve(&large_cursor, WritingMode::HorizontalTb, vec![binding]) {
        Ok(_) => {
            panic!("lost parent-local f32 baseline displacement must refuse before publication")
        }
        Err(error) => error,
    };
    // f64 shaping, envelope construction and integer snapping retain Delta.
    // This loss belongs specifically to Layout's parent-local f32 boundary.
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::TextNumeric {
                reason: LayoutTextNumericError::NonZeroLost,
                ..
            }
        ),
        "{error:?}"
    );
    same_geometry(sink.batch.as_ref().unwrap(), &prior);
    assert_eq!(
        sink.batch.as_ref().unwrap().final_inline_fragments(),
        prior.final_inline_fragments()
    );
}

#[test]
fn precision_f32_atomic_shift_absorption_refuses_before_replacing_prior_publication() {
    assert_eq!(
        (PRECISION_CURSOR + 4.0) as f32,
        (PRECISION_CURSOR + 3.0) as f32
    );
    precision_refusal_preserves_publication(false);
}
#[test]
fn precision_f32_control_shift_absorption_refuses_before_replacing_prior_publication() {
    assert_eq!(
        (PRECISION_CURSOR + 8.0) as f32,
        (PRECISION_CURSOR + 7.0) as f32
    );
    precision_refusal_preserves_publication(true);
}
#[test]
fn precision_f64_retains_atomic_and_control_shifts_after_real_large_local_spacer() {
    precision_success::<f64>(false, PRECISION_CURSOR, false);
    precision_success::<f64>(true, PRECISION_CURSOR, false);
}
#[test]
fn precision_large_parent_translation_keeps_local_shifts_valid_in_both_scalar_lanes() {
    for control in [false, true] {
        precision_success::<f32>(control, PRECISION_CURSOR, true);
        precision_success::<f64>(control, PRECISION_CURSOR, true);
    }
}

fn precision_fractional<S: LayoutScalar>(control: bool) {
    let (native, binding) = precision_native(control);
    let tree = precision_tree::<S>(&native, control, 0.25, 0.0, false);
    let batch = solve(&tree, WritingMode::HorizontalTb, vec![binding]).unwrap();
    // Atomic edges undergo canonical pixel snapping: 4-0.25 becomes4.
    // Control anchors reach the scalar consumer directly and retain8-0.25.
    // Qualification must follow each actual recipient's conversion policy.
    assert_eq!(
        output(&batch, 2).location.y,
        n(if control { 7.75 } else { 4.0 })
    );
    raw(placed(&batch, 10), 10.0, 10.0);
}
#[test]
fn precision_fractional_atomic_edge_snaps_while_control_anchor_retains_displacement() {
    for control in [false, true] {
        precision_fractional::<f32>(control);
        precision_fractional::<f64>(control);
    }
}

fn precision_tiny_atomic<S: LayoutScalar>() {
    let delta = 1.0 / 67_108_864.0; // 2^-26, exactly representable in both lanes.
    assert_ne!(4.0_f64 - delta, 4.0);
    assert_eq!((4.0_f64 - delta).round(), 4.0);
    assert_eq!((4.0_f64 - delta) as f32, 4.0_f32);
    let (native, binding) = precision_native(false);
    let tree = precision_tree::<S>(&native, false, delta, 0.0, false);
    let batch = solve(&tree, WritingMode::HorizontalTb, vec![binding]).unwrap();
    let line = finalized(&batch, 10);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (8.0, 10.0)
    );
    let text::AtomicLineAllocation::InFlow { alignment, .. } =
        line.selected_line().request().atomics[0].allocation
    else {
        panic!("real in-flow child")
    };
    assert_eq!(
        alignment,
        text::LineBoxAlignment::Shift(text::BaselineShift::try_new(delta).unwrap())
    );
    // This atomic's raw top is distinct in Text's f64 domain. Its canonical
    // pixel edge is4 before the actual Layout scalar conversion consumes it.
    // Projecting the earlier raw pair to f32 would invent a loss at a boundary
    // that this atomic never consumes and wrongly reject this valid operation.
    assert_eq!(placed(&batch, 10).inline_boxes()[0].rect().origin.y, 4.0);
    assert_eq!(output(&batch, 2).location.y, n(4.0));
    let pair = output(&batch, 2)
        .named_baselines
        .get(BaselineName::Alphabetic);
    assert_eq!(
        (pair.first(), pair.last()),
        (BaselineFactOf::Absent, BaselineFactOf::Absent)
    );
    raw(placed(&batch, 10), 10.0, 10.0);
    custody(placed(&batch, 10), &native.original);
}
#[test]
fn precision_atomic_tiny_shift_can_reach_its_canonical_pixel_edge_before_scalar_narrowing() {
    precision_tiny_atomic::<f32>();
    precision_tiny_atomic::<f64>();
}
