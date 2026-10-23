//! Zone references are pure trials of the existing cached/transactional owner.
use super::*;
fn root<S: LayoutScalar>(width: f64, zone: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        hyphenate_limit_zone: LengthPercentageOf::px(S::from_f64(zone)).unwrap(),
        ..NodeInputOf::default()
    }
}
fn ends<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<usize> {
    batch.placed_text()[0]
        .layout()
        .lines()
        .iter()
        .map(|line| line.range().end)
        .collect()
}
#[test]
fn zone_references_rebuild_for_policy_and_float_frames_and_never_publish_failed_trials() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.locale = Some("en-US".into());
        style.hyphens = text::Hyphens::Auto;
        style.hyphenate_character = text::HyphenateCharacter::String(text::Text::from("A"));
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(10, Source::new("X "), style.clone()),
                ParagraphPart::text(11, Source::new("associate"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let mut binding = paragraph.binding(2);
        binding.nodes = vec![10, 30, 11];
        let tree = float_tree::<S>(&binding, 60.0, Float::Left, 1).style(2, root(60.0, 40.0));
        let mut host = CachedHost::new(tree);
        let cold = host.compute(&binding, &[]).unwrap();
        // The actual prefix is10px; an8px float leaves52px, hence U42.
        assert_eq!(ends(&cold), [4, 6, 11]);
        assert_eq!(output(&cold, 30).location, Point::ZERO);
        cold.apply_to(&mut host).unwrap();
        host.hits.borrow_mut().clear();
        let warm = host.compute(&binding, &[]).unwrap();
        assert!(host.hits.borrow().contains(&30));
        super::conditional_cache::assert_same(&cold, &warm);
        warm.apply_to(&mut host).unwrap();
        for (width, zone, expected) in [
            (60.0, 42.0, vec![2, 6, 11]),
            (80.0, 42.0, vec![6, 11]),
            (60.0, 40.0, vec![4, 6, 11]),
        ] {
            host.tree
                .insert_input(2, LayoutInputOf::box_input(root(width, zone)));
            let changed = host.compute(&binding, &[2]).unwrap();
            assert_eq!(ends(&changed), expected);
            let fresh = CachedHost::new(host.tree.clone())
                .compute(&binding, &[])
                .unwrap();
            super::conditional_cache::assert_same(&changed, &fresh);
            changed.apply_to(&mut host).unwrap();
        }
        let prior = host.published.clone().unwrap();
        let caches = host.caches.clone();
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
        let error = host.compute(&binding, &[2, 30]).unwrap_err();
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement("actual float shape refused")
        ));
        assert_eq!(host.caches, caches);
        super::conditional_cache::assert_same(host.published.as_ref().unwrap(), &prior);
        host.fail_shape = false;
        let replacement = host.compute(&binding, &[2, 30]).unwrap();
        host.reject_batch = true;
        assert_eq!(
            replacement.apply_to(&mut host),
            Err("reject complete replacement")
        );
        assert_eq!(host.caches, caches);
        super::conditional_cache::assert_same(host.published.as_ref().unwrap(), &prior);
        host.reject_batch = false;
        replacement.apply_to(&mut host).unwrap();
        let fresh = CachedHost::new(host.tree.clone())
            .compute(&binding, &[])
            .unwrap();
        super::conditional_cache::assert_same(host.published.as_ref().unwrap(), &fresh);
    }
    check::<f32>();
    check::<f64>();
}
