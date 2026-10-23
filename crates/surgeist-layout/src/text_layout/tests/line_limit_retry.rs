//! Actual float trials, cache invalidation and transactional publication.
use super::*;
fn root_limit<S: LayoutScalar>(width: f64, limit: HyphenateLimitLines) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        hyphenate_limit_lines: limit,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}
fn ranges<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<(text::Range, f64)> {
    batch.placed_text()[0]
        .layout()
        .lines()
        .iter()
        .map(|line| (line.range(), line.baseline()))
        .collect()
}
fn check<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = text::Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.hyphenate_character = text::HyphenateCharacter::String(text::Text::from("A"));
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new(""), style.clone()),
            ParagraphPart::text(11, Source::new("cooperate"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
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
    let tree = float_tree::<S>(&binding, 30.0, Float::Left, 1)
        .style(2, root_limit(30.0, HyphenateLimitLines::Count(1)))
        .style(30, float.clone());
    let mut host = CachedHost::new(tree);
    let first = host.compute(&binding, &[]).unwrap();
    assert_eq!(output(&first, 30).location, Point::ZERO);
    assert_eq!(
        ranges(&first),
        [
            (text::Range::new(0, 2), 18.0),
            (text::Range::new(2, 9), 28.0)
        ]
    );
    assert_eq!(output(&first, 2).size.height.to_f64(), 30.0);
    assert_eq!(
        first.placed_text()[0].layout().source().text().code_units(),
        "cooperate".encode_utf16().collect::<Vec<_>>()
    );
    let first_line = first
        .final_inline_fragments()
        .iter()
        .find(|row| row.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(first_line.feedback().advance, 30.0);
    assert!(matches!(
        first_line
            .selected_line()
            .request()
            .alternative
            .as_ref()
            .unwrap()
            .view()
            .definition(),
        text::PreparedAlternativeDefinition::Automatic { .. }
    ));
    // Several rejected prefixes and the shortened band must not consume capacity.
    first.apply_to(&mut host).unwrap();
    let warm = host.compute(&binding, &[]).unwrap();
    assert_eq!(ranges(&warm), ranges(&first));
    warm.apply_to(&mut host).unwrap();
    for (limit, ends) in [
        (HyphenateLimitLines::NoLimit, vec![2, 4, 6, 9]),
        (HyphenateLimitLines::Count(0), vec![9]),
        (HyphenateLimitLines::Count(1), vec![2, 9]),
    ] {
        host.tree
            .insert_input(2, LayoutInputOf::box_input(root_limit(30.0, limit)));
        let batch = host.compute(&binding, &[2]).unwrap();
        assert_eq!(
            ranges(&batch)
                .iter()
                .map(|(range, _)| range.end)
                .collect::<Vec<_>>(),
            ends
        );
        let fresh = CachedHost::new(host.tree.clone())
            .compute(&binding, &[])
            .unwrap();
        assert_eq!(batch.final_entries(), fresh.final_entries());
        assert_eq!(ranges(&batch), ranges(&fresh));
        batch.apply_to(&mut host).unwrap();
    }
    host.tree.insert_input(
        2,
        LayoutInputOf::box_input(root_limit(60.0, HyphenateLimitLines::Count(1))),
    );
    let changed = host.compute(&binding, &[2]).unwrap();
    assert_eq!(
        ranges(&changed),
        [
            (text::Range::new(0, 4), 8.0),
            (text::Range::new(4, 9), 18.0)
        ]
    );
    changed.apply_to(&mut host).unwrap();
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
    assert_eq!(ranges(host.published.as_ref().unwrap()), ranges(&prior));
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
fn successive_line_limit_float_trials_cache_changes_and_late_refusals_are_atomic() {
    check::<f32>();
    check::<f64>();
}
