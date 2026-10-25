//! Conditional selections use the same accepted child and publication boundary.
use super::*;

pub(super) fn assert_same<S: LayoutScalar>(
    a: &CompletedLayoutBatchOf<u32, S>,
    b: &CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(a.final_entries(), b.final_entries());
    let fragments = |batch: &CompletedLayoutBatchOf<u32, S>| {
        batch
            .final_inline_fragments()
            .iter()
            .map(|row| {
                let f = row.fragment();
                (row.owner().clone(), f.rect(), f.baseline(), f.line_index())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(fragments(a), fragments(b));
    let positions = |batch: &CompletedLayoutBatchOf<u32, S>| {
        let layout = batch.placed_text()[0].layout();
        layout
            .positions()
            .map(|position| {
                (
                    position.source_cursor(),
                    layout.position_geometry(&position).unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(positions(a), positions(b));
    let lines = |batch: &CompletedLayoutBatchOf<u32, S>| {
        batch.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| (line.range(), line.bounds(), line.baseline()))
            .collect::<Vec<_>>()
    };
    assert_eq!(lines(a), lines(b));
}
fn policy_paragraph(overflow: text::OverflowWrap) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.overflow_wrap = overflow;
    PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    )
}
fn root<S: LayoutScalar>(width: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}
fn cached<S: LayoutScalar>() {
    let paragraph = policy_paragraph(text::OverflowWrap::Anywhere);
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<S>(&binding, 20.0, Float::Left, 1);
    let mut host = CachedHost::new(tree);
    let cold = host.compute(&binding, &[]).unwrap();
    assert_eq!(
        cold.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [text::Range::new(0, 1), text::Range::new(1, 2)]
    );
    assert_eq!(output(&cold, 30).location, Point::ZERO);
    cold.apply_to(&mut host).unwrap();
    host.hits.borrow_mut().clear();
    let warm = host.compute(&binding, &[]).unwrap();
    assert!(host.hits.borrow().contains(&30));
    assert_same(&cold, &warm);
    warm.apply_to(&mut host).unwrap();

    host.tree
        .insert_input(2, LayoutInputOf::box_input(root(30.0)));
    let changed = host.compute(&binding, &[2]).unwrap();
    assert_eq!(changed.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(output(&changed, 30).location, Point::ZERO);
    let fresh = CachedHost::new(host.tree.clone())
        .compute(&binding, &[])
        .unwrap();
    assert_same(&changed, &fresh);
    changed.apply_to(&mut host).unwrap();

    let ordinary = policy_paragraph(text::OverflowWrap::Normal);
    let mut ordinary_binding = ordinary.binding(2);
    ordinary_binding.nodes = vec![10, 30, 11];
    host.tree
        .insert_input(2, LayoutInputOf::box_input(root(20.0)));
    for node in [10, 11] {
        host.tree.insert_input(node, ordinary.text_input(node));
    }
    let policy = host.compute(&ordinary_binding, &[2, 10, 11]).unwrap();
    assert_eq!(policy.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(
        output(&policy, 30).location,
        Point::new(S::ZERO, S::from_f64(10.0))
    );
    let fresh = CachedHost::new(host.tree.clone())
        .compute(&ordinary_binding, &[])
        .unwrap();
    assert_same(&policy, &fresh);
    policy.apply_to(&mut host).unwrap();

    let prior = host.published.clone().unwrap();
    let prior_cache = host.caches.clone();
    for node in [10, 11] {
        host.tree.insert_input(node, paragraph.text_input(node));
    }
    host.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(8.0)),
                PreferredSizeOf::px(S::from_f64(4.0)),
            ),
            ..NodeInputOf::default()
        }),
    );
    host.fail_shape = true;
    let error = host.compute(&binding, &[2, 10, 11, 30]).unwrap_err();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement("actual float shape refused")
    ));
    assert_eq!(host.caches, prior_cache);
    assert_same(host.published.as_ref().unwrap(), &prior);
    host.fail_shape = false;
    // This recorder returns no shape exclusion on success. Restore the real
    // margin box before checking the original shortened-band replacement.
    host.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(8.0)),
                PreferredSizeOf::px(S::from_f64(4.0)),
            ),
            ..NodeInputOf::default()
        }),
    );
    let replacement = host.compute(&binding, &[2, 10, 11, 30]).unwrap();
    assert_eq!(replacement.placed_text()[0].layout().lines().len(), 2);
    host.reject_batch = true;
    assert_eq!(
        replacement.apply_to(&mut host),
        Err("reject complete replacement")
    );
    assert_eq!(host.caches, prior_cache);
    assert_same(host.published.as_ref().unwrap(), &prior);
    host.reject_batch = false;
    replacement.apply_to(&mut host).unwrap();
    assert_eq!(
        host.published.as_ref().unwrap().placed_text()[0]
            .layout()
            .lines()
            .len(),
        2
    );
}
#[test]
fn conditional_float_replay_cold_warm_changed_frame_policy_and_late_failure_are_atomic() {
    cached::<f32>();
    cached::<f64>();
}

fn signed<S: LayoutScalar>(tracking: f64) {
    let mut first = ahem_style(10.0, 10.0);
    first.letter_spacing = tracking;
    first.overflow_wrap = text::OverflowWrap::Anywhere;
    let mut second = ahem_style(10.0, 10.0);
    second.overflow_wrap = text::OverflowWrap::Anywhere;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), first.clone()),
            ParagraphPart::text(11, Source::new("XX"), second),
        ],
        first,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<S>(&binding, 15.0, Float::Left, 1);
    let capture = LayoutFlowInput::try_new(&tree, 2, vec![binding], Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(15.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [
            text::Range::new(0, 1),
            text::Range::new(1, 2),
            text::Range::new(2, 3)
        ]
    );
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.bounds().origin.y)
            .collect::<Vec<_>>(),
        [0.0, 10.0, 20.0]
    );
    assert_eq!(output(&batch, 30).location, Point::ZERO);
    let line = batch
        .final_inline_fragments()
        .iter()
        .find(|row| row.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let origin = paragraph.origin(10);
    let seam = text::TextSeam::Origin {
        local: text::SourcePosition::try_new(origin.view().source().text(), 1).unwrap(),
        origin,
    };
    let prefix = line.prefix_at(&seam).unwrap();
    assert!(prefix.has_retained_flow_participation());
    assert_eq!(prefix.advance(), 10.0 + tracking);
    assert_eq!(line.feedback().advance, 10.0 + tracking);
    assert!(line.feedback().occupied_end - line.feedback().occupied_start <= 7.0);
}
#[test]
fn conditional_retained_zero_and_signed_prefixes_advance_source_and_keep_the_real_line() {
    for tracking in [-10.0, -12.5] {
        signed::<f32>(tracking);
        signed::<f64>(tracking);
    }
}

fn future<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Collapse;
    style.overflow_wrap = text::OverflowWrap::Anywhere;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("AAA "), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<S>(&binding, 20.0, Float::Left, 1);
    let capture = LayoutFlowInput::try_new(&tree, 2, vec![binding], Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(20.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [
            text::Range::new(0, 2),
            text::Range::new(2, 4),
            text::Range::new(4, 5)
        ]
    );
    assert_eq!(
        output(&batch, 30).location,
        Point::new(S::ZERO, S::from_f64(10.0))
    );
    let mut lines = vec![None; 3];
    for row in batch.final_inline_fragments() {
        lines[row.fragment().line_index()] =
            Some(row.fragment().finalized_fragment().finalized_line());
    }
    let lines = lines.into_iter().map(Option::unwrap).collect::<Vec<_>>();
    assert_eq!(
        lines
            .iter()
            .map(|line| line.feedback().advance)
            .collect::<Vec<_>>(),
        [20.0, 10.0, 10.0]
    );
    assert_eq!(
        lines
            .iter()
            .map(|line| line.selected_line().request().context.end)
            .collect::<Vec<_>>(),
        [
            text::LineEnd::Soft,
            text::LineEnd::Soft,
            text::LineEnd::Paragraph
        ]
    );
    let origin = paragraph.origin(10);
    let seam = text::TextSeam::Origin {
        local: text::SourcePosition::try_new(origin.view().source().text(), 4).unwrap(),
        origin,
    };
    assert_eq!(lines[1].prefix_at(&seam).unwrap().advance(), 10.0);
    assert_eq!(layout.lines()[0].bounds().size.width, 20.0);
    assert_eq!(layout.lines()[1].bounds().size.width, 12.0);
}
#[test]
fn conditional_cut_defers_future_events_and_restores_the_selected_discarded_edge_receipt() {
    future::<f32>();
    future::<f64>();
}
