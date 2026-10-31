//! InterCharacter allocation composes with real group and float replay.
use super::*;

fn block<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        text_align: TextAlign::Justify,
        text_align_last: TextAlignLast::Justify,
        text_justification: TextJustification::InterCharacter,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(80.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}

fn tree<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    input: NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(0, input)
        .input(1, paragraph.text_input(1))
}

fn compute<Tree: LayoutTree<Node = u32>>(
    tree: &Tree,
    paragraph: &PreparedParagraph,
    dirty: &[u32],
) -> CompletedLayoutBatchOf<u32, Tree::Scalar>
where
    Tree::MeasureError: std::fmt::Debug,
{
    let capture = LayoutFlowInput::try_new(
        tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow_invalidated(&capture, 0, request(80.0), dirty).unwrap()
}

fn insets<S: LayoutScalar>() {
    let paragraph = paragraph(
        AHEM,
        "ABC\nAAAAA",
        own(0.0, Wrap::None, text::OverflowWrap::Normal, false),
    );
    let mut input = block::<S>();
    input.border = Edges::new(
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(3.0)),
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(2.0)),
    );
    input.padding = Edges::new(
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(4.0)),
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(3.0)),
    );
    input.text_indent = TextIndentOf {
        amount: LengthPercentageOf::px(S::from_f64(8.0)).unwrap(),
        each_line: true,
        hanging: false,
    };
    input.text_group_align = TextGroupAlign::Center;
    let actual = compute(&tree(&paragraph, input), &paragraph, &[]);
    let selected = lines(&actual);
    assert_eq!(selected.len(), 2);
    // Border+padding leave68; indent8 and actual group insets5+5 leave50.
    for line in &selected {
        let selected = line.selected_line();
        assert_eq!(selected.request().context.available_inline, Some(68.0));
        assert_eq!(selected.request().context.indent, 8.0);
        assert_eq!(
            selected.request().context.alignment_insets,
            text::LineAlignmentInsets::try_new(5.0, 5.0).unwrap()
        );
        assert_eq!(
            selected.request().context.spacing,
            text::SpacingPolicy::InterCharacter
        );
        assert_eq!(line.measurements().alignment().advance(), 50.0);
    }
    amounts(&selected[0], &[10.0, 10.0]);
    amounts(&selected[1], &[0.0; 4]);
    for (index, row) in selected[0].spacing_slots().enumerate() {
        boundary(&row.slot, &paragraph, index, 0.0);
    }
    assert_eq!(
        paints(&actual)
            .iter()
            .map(|fact| fact.3)
            .collect::<Vec<_>>(),
        [18.0, 38.0, 58.0, 18.0, 28.0, 38.0, 48.0, 58.0]
    );
    copy(&actual, "ABC\nAAAAA");
}

#[test]
fn character_fill_replays_group_capacity_with_real_content_edges_indent_and_insets() {
    insets::<f32>();
    insets::<f64>();
}

fn float_input<S: LayoutScalar>(height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        float: Float::Left,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(30.0)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..NodeInputOf::default()
    }
}

fn selected_boundary(
    slot: &text::SpacingSlot,
    paragraph: &PreparedParagraph,
    line_start: usize,
    at: usize,
) {
    let text::SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slot.target()
    else {
        panic!("InterCharacter retains actual adjacent unit targets")
    };
    for (unit, local) in [(visual_before, at), (visual_after, at + 1)] {
        assert_eq!(unit.letter_spacing_before(), 4.0);
        assert_eq!(unit.letter_spacing_after(), 4.0);
        let pieces = unit.pieces().collect::<Vec<_>>();
        assert_eq!(pieces.len(), 1);
        let source = text::Range::new(line_start + local, line_start + local + 1);
        assert_eq!(
            pieces[0].owner(),
            text::SpacingSlotOwner::Source {
                origin: paragraph.origin(1),
                local: source,
                aggregate: source,
            }
        );
        // Selected presentation coordinates restart at the actual line cut;
        // original source custody and prepared participants retain their offsets.
        assert_eq!(
            (
                pieces[0].presentation_range().start(),
                pieces[0].presentation_range().end(),
            ),
            (local, local + 1)
        );
        let participant = paragraph
            .prepared
            .unit_views()
            .find(|unit| unit.source_range() == source)
            .unwrap()
            .handle();
        assert_eq!(
            unit.participants().cloned().collect::<Vec<_>>(),
            [text::PreparedParticipant::Text(participant)]
        );
    }
}

fn cached_float<S: LayoutScalar>() {
    use super::super::float_encounters::CachedHost;
    let paragraph = paragraph(
        AHEM,
        "ABC\nABC",
        own(4.0, Wrap::None, text::OverflowWrap::Normal, false),
    );
    let tree = tree(&paragraph, block::<S>())
        .children(0, [9, 1])
        .children(9, [])
        .style(9, float_input(10.0));
    let mut host = CachedHost::new(tree);
    let cold = compute(&host, &paragraph, &[]);
    let cold_lines = lines(&cold);
    for (index, line) in cold_lines.iter().enumerate() {
        amounts(
            line,
            if index == 0 {
                &[6.0, 6.0]
            } else {
                &[21.0, 21.0]
            },
        );
        assert_eq!(
            line.selected_line().request().context.spacing,
            text::SpacingPolicy::InterCharacter
        );
        assert_eq!(
            line.measurements().alignment().advance(),
            if index == 0 { 50.0 } else { 80.0 }
        );
    }
    assert_eq!(
        paints(&cold).iter().map(|fact| fact.3).collect::<Vec<_>>(),
        [30.0, 50.0, 70.0, 0.0, 35.0, 70.0]
    );
    cold.apply_to(&mut host).unwrap();
    let stored = cold
        .cache_store_entries()
        .iter()
        .find(|row| row.node() == 9)
        .unwrap();
    assert!(
        host.cache_get(9, stored.input(), stored.context())
            .is_some()
    );
    let warm = compute(&host, &paragraph, &[]);
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_eq!(paints(&warm), paints(&cold));
    for (warm, cold) in lines(&warm).iter().zip(&cold_lines) {
        assert_eq!(
            warm.spacing_slots()
                .map(|row| row.allocation)
                .collect::<Vec<_>>(),
            cold.spacing_slots()
                .map(|row| row.allocation)
                .collect::<Vec<_>>()
        );
        assert_eq!(raw(warm, AHEM), raw(cold, AHEM));
    }
    warm.apply_to(&mut host).unwrap();
    host.tree
        .insert_input(9, LayoutInputOf::box_input(float_input(20.0)));
    let changed = compute(&host, &paragraph, &[9]);
    let fresh = compute(&CachedHost::new(host.tree.clone()), &paragraph, &[]);
    assert_eq!(changed.final_entries(), fresh.final_entries());
    assert_eq!(paints(&changed), paints(&fresh));
    for batch in [&changed, &fresh] {
        for (index, line) in lines(batch).iter().enumerate() {
            amounts(line, &[6.0, 6.0]);
            assert_eq!(
                line.selected_line().request().context.available_inline,
                Some(50.0)
            );
            assert_eq!(line.measurements().alignment().advance(), 50.0);
            for (at, row) in line.spacing_slots().enumerate() {
                selected_boundary(&row.slot, &paragraph, index * 4, at);
            }
            assert_eq!(
                raw(line, AHEM),
                [
                    (text::Range::new(index * 4, index * 4 + 1), 35, 10.0),
                    (text::Range::new(index * 4 + 1, index * 4 + 2), 36, 10.0),
                    (text::Range::new(index * 4 + 2, index * 4 + 3), 37, 10.0),
                ]
            );
        }
    }
    assert_eq!(
        paints(&changed)
            .iter()
            .map(|fact| fact.3)
            .collect::<Vec<_>>(),
        [30.0, 50.0, 70.0, 30.0, 50.0, 70.0]
    );
    changed.apply_to(&mut host).unwrap();
    // Previously issued complete packets retain their allocation and geometry.
    amounts(&cold_lines[1], &[21.0, 21.0]);
    assert_eq!(
        paints(&cold).iter().map(|fact| fact.3).collect::<Vec<_>>(),
        [30.0, 50.0, 70.0, 0.0, 35.0, 70.0]
    );
    copy(&changed, "ABC\nABC");
}

#[test]
fn character_allocations_survive_cached_float_replay_and_changed_band_recomputation() {
    cached_float::<f32>();
    cached_float::<f64>();
}
