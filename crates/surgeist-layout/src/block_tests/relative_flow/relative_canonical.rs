use super::*;

fn fractional_atomic_paragraph(
    nominal: surgeist_text::Size,
    trailing: bool,
) -> crate::test_support::text::PreparedParagraph {
    use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
    let style = ahem_style(10.0, 10.0);
    let mut parts = vec![
        ParagraphPart::text(1, surgeist_text::Source::new("X"), style.clone()),
        ParagraphPart::Atomic {
            node: 2,
            size: nominal,
        },
    ];
    if trailing {
        parts.push(ParagraphPart::text(
            3,
            surgeist_text::Source::new("X"),
            style.clone(),
        ));
    }
    PreparedParagraph::new(
        parts,
        style,
        surgeist_text::ParagraphBaseDirection::LeftToRight,
    )
}

fn fractional_atomic_layout<S: LayoutScalar>(
    paragraph: &crate::test_support::text::PreparedParagraph,
    nominal: surgeist_text::Size,
    inset: Edges<LengthAutoOf<S>>,
    trailing: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let atomic = NodeInputOf {
        display: Display::InlineBlock,
        position: Position::Relative,
        vertical_align: VerticalAlignOf::Baseline,
        size: Size::new(
            PreferredSizeOf::px(px::<S>(nominal.width)),
            PreferredSizeOf::px(px::<S>(nominal.height)),
        ),
        inset,
        ..NodeInputOf::default()
    };
    assert!(atomic.atomic_inline_participation.is_none());
    let binding = paragraph.binding(0);
    let issued = paragraph.prepared.atomic_views().collect::<Vec<_>>();
    assert_eq!(
        binding.nodes,
        if trailing { vec![1, 2, 3] } else { vec![1, 2] }
    );
    assert_eq!(
        binding
            .origins
            .iter()
            .map(|row| row.node)
            .collect::<Vec<_>>(),
        if trailing { vec![1, 3] } else { vec![1] }
    );
    assert_eq!(issued.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(binding.atomics[0].occurrence, issued[0].handle());
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, if trailing { vec![1, 2, 3] } else { vec![1, 2] })
        .children(1, [])
        .children(2, [])
        .style(0, root_style(Direction::Ltr))
        .input(1, paragraph.text_input(1))
        .style(2, atomic);
    if trailing {
        tree = tree.children(3, []).input(3, paragraph.text_input(3));
    }
    // Nominal declared geometry is distinct from canonical emitted corners.
    assert_eq!(
        crate::LayoutTree::node_input(&tree, 2).size,
        Size::new(
            PreferredSizeOf::px(px::<S>(nominal.width)),
            PreferredSizeOf::px(px::<S>(nominal.height))
        )
    );
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(px::<S>(50.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap();
    let actual_atomic = output(&batch, 2);
    assert_eq!(actual_atomic.margin, Edges::all(S::ZERO));
    assert_eq!(actual_atomic.border, Edges::all(S::ZERO));
    assert_eq!(actual_atomic.padding, Edges::all(S::ZERO));
    batch
}

fn fractional_atomic_source_and_font<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    trailing: bool,
) {
    let placed = batch.placed_text();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].container(), 0);
    assert_eq!(
        placed[0].nodes(),
        if trailing {
            &[1, 2, 3][..]
        } else {
            &[1, 2][..]
        }
    );
    let layout = placed[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        if trailing {
            &[0x58, 0x58][..]
        } else {
            &[0x58][..]
        }
    );
    let lines = layout.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].range(),
        surgeist_text::Range::new(0, if trailing { 2 } else { 1 })
    );
    let runs = layout.glyph_runs();
    assert_eq!(
        runs.iter().map(|run| run.glyphs().len()).sum::<usize>(),
        if trailing { 2 } else { 1 }
    );
    for run in &runs {
        assert_eq!(run.font().index(), 0);
        assert_eq!(
            run.font().bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .as_slice()
        );
        assert_eq!(run.font_size(), 10.0);
    }
    if !trailing {
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].range(), surgeist_text::Range::new(0, 1));
        assert_eq!(runs[0].advance(), 10.0);
    }
    assert_eq!(layout.inline_boxes().len(), 1);
}

fn fractional_atomic_visual_agreement<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    let node = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 2)
        .unwrap()
        .output();
    let visual = batch.placed_text()[0].layout().inline_boxes()[0].rect();
    // Preserve the shared canonical physical box contract without prescribing
    // the corner quantization result of a relatively translated rectangle.
    assert_eq!(visual.origin.x, node.location.x.to_f64());
    assert_eq!(visual.origin.y, node.location.y.to_f64());
    assert_eq!(visual.size.width, node.size.width.to_f64());
    assert_eq!(visual.size.height, node.size.height.to_f64());
}

fn fractional_atomic_top_preserves_normal_line<S: LayoutScalar>() {
    let nominal = surgeist_text::Size::new(10.0, 10.4);
    let paragraph = fractional_atomic_paragraph(nominal, false);
    for top in [0.0, 0.72] {
        let batch = fractional_atomic_layout::<S>(
            &paragraph,
            nominal,
            Edges {
                top: LengthAutoOf::px(px::<S>(top)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            false,
        );
        fractional_atomic_source_and_font(&batch, false);
        // Ahem contributes before8/after2. The unshifted canonical empty atomic
        // contributes before10/after0: normal height12, reference10. Relative
        // translation changes neither normal contribution nor its reference.
        // This is deliberately the first new normal-geometry assertion: the
        // TOP0 control precedes TOP.72 under the same authentic preparation.
        assert_eq!(
            output(&batch, 0).size,
            Size::new(px::<S>(50.0), px::<S>(12.0)),
            "top={top}"
        );
        let final_parent = batch
            .final_entries()
            .iter()
            .find(|row| row.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            final_parent.size,
            Size::new(px::<S>(50.0), px::<S>(12.0)),
            "top={top}"
        );
        let lines = batch.placed_text()[0].layout().lines();
        assert_eq!(lines[0].baseline(), 10.0, "top={top}");
        let text = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|entry| *entry.owner() == crate::InlineFragmentOwner::Node(1))
            .unwrap()
            .fragment();
        assert_eq!(text.baseline().y, px::<S>(10.0));
        let rect = text.rect().unwrap();
        assert_eq!(rect.origin(), Point::new(S::ZERO, px::<S>(2.0)));
        assert_eq!(rect.size(), Size::splat(px::<S>(10.0)));
        fractional_atomic_visual_agreement(&batch);
    }
}

fn fractional_atomic_left_preserves_following_text<S: LayoutScalar>() {
    let nominal = surgeist_text::Size::new(10.4, 10.0);
    let paragraph = fractional_atomic_paragraph(nominal, true);
    for left in [0.0, 0.32] {
        let batch = fractional_atomic_layout::<S>(
            &paragraph,
            nominal,
            Edges {
                left: LengthAutoOf::px(px::<S>(left)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            true,
        );
        fractional_atomic_source_and_font(&batch, true);
        // X10 + unshifted canonical atomic width10 puts the actual following
        // original X at20. A wider snapped VISUAL corner hull cannot become a
        // new normal FIT/advance or move that following source occurrence.
        let following = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|entry| *entry.owner() == crate::InlineFragmentOwner::Node(3))
            .unwrap()
            .fragment();
        let rect = following.rect().unwrap();
        assert_eq!(rect.origin().x, px::<S>(20.0), "left={left}");
        assert_eq!(rect.size().width, px::<S>(10.0));
        fractional_atomic_visual_agreement(&batch);
    }
}

#[test]
fn captured_atomic_relative_top_keeps_normal_line_f32() {
    fractional_atomic_top_preserves_normal_line::<f32>();
}
#[test]
fn captured_atomic_relative_top_keeps_normal_line_f64() {
    fractional_atomic_top_preserves_normal_line::<f64>();
}
#[test]
fn captured_atomic_relative_left_keeps_following_text_f32() {
    fractional_atomic_left_preserves_following_text::<f32>();
}
#[test]
fn captured_atomic_relative_left_keeps_following_text_f64() {
    fractional_atomic_left_preserves_following_text::<f64>();
}
