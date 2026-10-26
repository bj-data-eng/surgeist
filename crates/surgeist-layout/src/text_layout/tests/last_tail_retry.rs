//! Completed-tail branches stay inside the actual unpublished float/cache pass.
use super::*;
fn root_tail<S: LayoutScalar>(policy: HyphenateLimitLast) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        hyphenate_limit_last: policy,
        hyphenate_limit_lines: HyphenateLimitLines::Count(1),
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}
fn ranges<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::Range> {
    batch.placed_text()[0]
        .layout()
        .lines()
        .iter()
        .map(|line| line.range())
        .collect()
}
fn check<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Collapse;
    style.hyphens = text::Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.hyphenate_character = text::HyphenateCharacter::String(text::Text::from("A"));
    let p = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("XXXX "), style.clone()),
            ParagraphPart::text(11, Source::new("associate"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = p.binding(2);
    binding.nodes = vec![10, 30, 11];
    let float = NodeInputOf {
        display: Display::Block,
        float: Float::Left,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(10.0)),
            PreferredSizeOf::px(S::from_f64(10.0)),
        ),
        ..NodeInputOf::default()
    };
    let tree = float_tree::<S>(&binding, 100.0, Float::Left, 1)
        .style(2, root_tail(HyphenateLimitLast::Always))
        .style(30, float.clone());
    let mut host = CachedHost::new(tree);
    let first = host.compute(&binding, &[]).unwrap();
    assert_eq!(
        ranges(&first),
        [text::Range::new(0, 5), text::Range::new(5, 14)]
    );
    assert_eq!(
        first.placed_text()[0].layout().source().text().code_units(),
        "XXXX associate".encode_utf16().collect::<Vec<_>>()
    );
    for (index, advance) in [40.0, 90.0].into_iter().enumerate() {
        let line = first
            .final_inline_fragments()
            .iter()
            .find(|row| row.fragment().line_index() == index)
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(line.feedback().advance, advance);
        assert!(line.selected_line().request().alternative.is_none());
    }
    first.apply_to(&mut host).unwrap();
    let warm = host.compute(&binding, &[]).unwrap();
    assert_eq!(ranges(&warm), ranges(&first));
    warm.apply_to(&mut host).unwrap();
    for policy in [
        HyphenateLimitLast::None,
        HyphenateLimitLast::Always,
        HyphenateLimitLast::None,
        HyphenateLimitLast::Always,
    ] {
        host.tree
            .insert_input(2, LayoutInputOf::box_input(root_tail(policy)));
        let actual = host.compute(&binding, &[2]).unwrap();
        let fresh = CachedHost::new(host.tree.clone())
            .compute(&binding, &[])
            .unwrap();
        assert_eq!(actual.final_entries(), fresh.final_entries());
        assert_eq!(ranges(&actual), ranges(&fresh));
        assert_eq!(
            ranges(&actual),
            if policy == HyphenateLimitLast::Always {
                vec![text::Range::new(0, 5), text::Range::new(5, 14)]
            } else {
                vec![text::Range::new(0, 7), text::Range::new(7, 14)]
            }
        );
        actual.apply_to(&mut host).unwrap();
    }
    let prior = host.published.clone().unwrap();
    let caches = host.caches.clone();
    host.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            float_exclusion: FloatExclusion::Shape,
            ..float.clone()
        }),
    );
    host.fail_shape = true;
    let error = host.compute(&binding, &[2, 30]).unwrap_err();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement("actual float shape refused")
    ));
    assert_eq!(host.caches, caches);
    assert_eq!(
        host.published.as_ref().unwrap().final_entries(),
        prior.final_entries()
    );
    host.fail_shape = false;
    host.tree.insert_input(30, LayoutInputOf::box_input(float));
    let retry = host.compute(&binding, &[2, 30]).unwrap();
    assert_eq!(ranges(&retry), ranges(&prior));
    host.reject_batch = true;
    assert_eq!(
        retry.apply_to(&mut host),
        Err("reject complete replacement")
    );
    assert_eq!(host.caches, caches);
    host.reject_batch = false;
    retry.apply_to(&mut host).unwrap();
    assert_eq!(ranges(host.published.as_ref().unwrap()), ranges(&prior));
}
#[test]
fn completed_tail_float_replay_policy_changes_and_late_refusal_are_pure() {
    check::<f32>();
    check::<f64>();
}
