//! Settled scrollbar space changes the used relative-percentage content basis.
use super::*;
use crate::test_support::readback::{self, Phase};

fn percentage_gutter_binding(
    p: &PreparedParagraph,
    overflowing: bool,
) -> TextParagraphBinding<u32> {
    let mut binding = p.binding(0);
    if overflowing {
        // The complete native run includes the real Layout-owned absolute
        // participant, without inventing any Text atomic occurrence for it.
        binding.nodes.push(9);
    }
    binding
}

fn percentage_gutter_fixture<S: LayoutScalar>(
    p: &PreparedParagraph,
    top: LengthAutoOf<S>,
    overflowing: bool,
) -> PublicLayoutTreeOf<S> {
    let mut parent = custody_root();
    parent.box_sizing = BoxSizing::BorderBox;
    parent.size.height = PreferredSizeOf::px(custody_n::<S>(20.0));
    parent.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
    parent.scrollbar_gutter = ScrollbarGutter::Auto;
    parent.scrollbar_width = crate::ScrollbarWidthOf::try_new(custody_n::<S>(2.0)).unwrap();
    let mut atomic = custody_atom(false, 0.0);
    atomic.inset.top = top;
    let mut tree = custody_restorable_fixture(p, false, 0.0)
        .style(0, parent)
        .style(2, atomic);
    if overflowing {
        tree = tree.children(0, [1, 2, 9]).children(9, []).style(
            9,
            NodeInputOf {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(
                    PreferredSizeOf::px(custody_n::<S>(60.0)),
                    PreferredSizeOf::px(custody_n::<S>(1.0)),
                ),
                inset: Edges {
                    top: LengthAutoOf::px(S::ZERO),
                    left: LengthAutoOf::px(S::ZERO),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                ..NodeInputOf::default()
            },
        );
        assert_eq!(tree.child(0, 2), 9);
        assert_eq!(tree.node_input(9).position, Position::Absolute);
        assert!(tree.node_input(9).atomic_inline_participation.is_none());
        assert!(tree.layout_input(9).as_inline_text().is_none());
        assert!(tree.layout_input(9).as_text_control().is_none());
    }
    assert_eq!(p.binding(0).nodes, vec![1, 2]);
    assert_eq!(
        p.binding(0).atomics[0].occurrence,
        p.prepared.atomic_views().next().unwrap().handle()
    );
    assert!(tree.node_input(2).atomic_inline_participation.is_none());
    assert_eq!(tree.node_input(2).size, custody_atom::<S>(false, 0.0).size);
    tree
}

fn percentage_gutter_assert<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
    overflowing: bool,
) {
    // Real original X/Ahem10 and issued atomic: feedbackH12/reference10/FIT20
    // remain normal facts inside the fixed20px border box in both states.
    captured_percent_source(
        batch,
        p,
        false,
        50.0,
        20.0,
        &percentage_gutter_binding(p, overflowing).nodes,
    );
    let content_height = if overflowing { 18.0 } else { 20.0 };
    for finalized in [false, true] {
        let root = custody_entry(batch, 0, finalized).output();
        assert_eq!(root.location, Point::ZERO);
        assert_eq!(
            root.size,
            Size::new(custody_n::<S>(50.0), custody_n::<S>(20.0))
        );
        let geometry = root.scroll_geometry.unwrap();
        assert_eq!(geometry.used_overflow_x(), Overflow::Auto);
        assert_eq!(geometry.used_overflow_y(), Overflow::Auto);
        assert_eq!(geometry.border_box().origin(), Point::ZERO);
        assert_eq!(geometry.border_box().size(), root.size);
        assert_eq!(geometry.padding_box(), geometry.border_box());
        assert_eq!(geometry.content_box().origin(), Point::ZERO);
        assert_eq!(
            geometry.content_box().size(),
            Size::new(custody_n::<S>(50.0), custody_n::<S>(content_height))
        );
        assert_eq!(geometry.scrollport(), geometry.content_box());
        assert_eq!(
            geometry.scrollbar_size(),
            Size::new(S::ZERO, custody_n::<S>(if overflowing { 2.0 } else { 0.0 }))
        );
        assert_eq!(geometry.gutters().top(), None);
        assert_eq!(geometry.gutters().left(), None);
        assert_eq!(geometry.gutters().right(), None);
        if overflowing {
            let bottom = geometry
                .gutters()
                .bottom()
                .expect("actual horizontal scrollbar gutter");
            assert_eq!(bottom.origin(), Point::new(S::ZERO, custody_n::<S>(18.0)));
            assert_eq!(
                bottom.size(),
                Size::new(custody_n::<S>(50.0), custody_n::<S>(2.0))
            );
        } else {
            assert_eq!(geometry.gutters().bottom(), None);
        }
        // The actual absolute border reaches60; the scrollport stays50 wide.
        // Reserved gutter itself is not natural vertical overflow/range.
        let range = geometry.physical_range();
        assert_eq!(range.x().minimum(), S::ZERO);
        assert_eq!(
            range.x().maximum(),
            custody_n::<S>(if overflowing { 10.0 } else { 0.0 })
        );
        assert_eq!(range.y().minimum(), S::ZERO);
        assert_eq!(range.y().maximum(), S::ZERO);
        let bounds = geometry.scrollable_overflow();
        assert_eq!(bounds.origin(), Point::ZERO);
        assert_eq!(
            bounds.size(),
            Size::new(
                custody_n::<S>(if overflowing { 60.0 } else { 50.0 }),
                custody_n::<S>(20.0)
            )
        );
    }
    if overflowing {
        let absolute = custody_entry(batch, 9, true).output();
        assert_eq!(absolute.location, Point::ZERO);
        assert_eq!(
            absolute.size,
            Size::new(custody_n::<S>(60.0), custody_n::<S>(1.0))
        );
    } else {
        assert!(
            readback::continuous_custody(batch, Phase::Final)
                .as_slice()
                .iter()
                .all(|entry| entry.node() != 9)
        );
    }
    let atomic = custody_entry(batch, 2, true).output();
    let (top, height, content_top) = if overflowing {
        (1.0, 11.0, 1.0)
    } else {
        (2.0, 10.0, 0.0)
    };
    assert_eq!(
        atomic.location,
        Point::new(custody_n::<S>(10.0), custody_n::<S>(top))
    );
    assert_eq!(
        atomic.size,
        Size::new(custody_n::<S>(10.0), custody_n::<S>(height))
    );
    let local_content = atomic.scroll_geometry.unwrap().content_box();
    assert_eq!(
        local_content.origin(),
        Point::new(S::ZERO, custody_n::<S>(content_top))
    );
    assert_eq!(
        local_content.size(),
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
    );
}

fn accepted_gutter_updates_percentage_basis_atomically<S: LayoutScalar>() {
    let p = custody_paragraph(false);
    let coefficient = custody_n::<S>(0.1);
    let percent =
        LengthAutoOf::value(LengthPercentageOf::from_percent_fraction(coefficient).unwrap());
    let mut host = CustodyHost::<S>::new(percentage_gutter_fixture(&p, percent, false));
    let cold = custody_compute(&host, vec![p.binding(0)], &[]);
    percentage_gutter_assert(&cold, &p, false);
    // Used content20 gives2px; rawtop-.4+2=1.6 and bottom12 =>2..12.
    // Padding corners1.85..11.75 =>local content0..10.
    let cold_offset = coefficient * custody_n::<S>(20.0);
    let cold_control_host = CustodyHost::<S>::new(percentage_gutter_fixture(
        &p,
        LengthAutoOf::px(cold_offset),
        false,
    ));
    let cold_control = custody_compute(&cold_control_host, vec![p.binding(0)], &[]);
    percentage_gutter_assert(&cold_control, &p, false);
    custody_same_scene(&cold, &cold_control);
    let old_fragments = cold
        .final_inline_fragments()
        .iter()
        .map(|row| row.fragment().finalized_fragment().clone())
        .collect::<Vec<_>>();
    cold.apply_to(&mut host).unwrap();
    let prior = host.inner.published.as_ref().unwrap().clone();
    let prior_caches = host.inner.caches.clone();

    // Only actual Layout-owned absolute geometry is added. The original
    // paragraph/BlockEnd and all original Text/atomic handles remain unchanged.
    host.inner.tree = percentage_gutter_fixture(&p, percent, true);
    let changed = custody_compute(&host, vec![percentage_gutter_binding(&p, true)], &[0, 9]);
    percentage_gutter_assert(&changed, &p, true);
    // The actual outer auto-scrollbar loop must settle content18.10% gives1.8:
    // border1.4..11.8=>1..12; padding1.65..11.55=>2..12, local content1..11.
    // Use the same S-lane product in the independent fresh constant control.
    let changed_offset = coefficient * custody_n::<S>(18.0);
    let control_host = CustodyHost::<S>::new(percentage_gutter_fixture(
        &p,
        LengthAutoOf::px(changed_offset),
        true,
    ));
    let control = custody_compute(
        &control_host,
        vec![percentage_gutter_binding(&p, true)],
        &[],
    );
    percentage_gutter_assert(&control, &p, true);
    custody_same_scene(&changed, &control);
    assert_ne!(
        custody_entry(&changed, 2, true).output().size,
        custody_entry(&cold, 2, true).output().size
    );
    // No hit is required: a changed gutter legitimately changes child inputs.
    assert_eq!(host.inner.caches, prior_caches);
    custody_same_scene(host.inner.published.as_ref().unwrap(), &prior);
    host.inner.reject_batch = true;
    assert_eq!(
        changed.apply_to(&mut host),
        Err(crate::LayoutBatchApplyError::Preparation(
            "reject complete replacement"
        ))
    );
    assert_eq!(host.inner.caches, prior_caches);
    custody_same_scene(host.inner.published.as_ref().unwrap(), &prior);
    percentage_gutter_assert(&cold, &p, false);
    assert_eq!(
        cold.final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        old_fragments
    );
    host.inner.reject_batch = false;
    changed.apply_to(&mut host).unwrap();
    custody_same_scene(host.inner.published.as_ref().unwrap(), &changed);

    // Removing the overflow child removes the actual bottom gutter and
    // recovers the old20px content basis through the same public operation.
    host.inner.tree = percentage_gutter_fixture(&p, percent, false);
    let recovered = custody_compute(&host, vec![p.binding(0)], &[0]);
    percentage_gutter_assert(&recovered, &p, false);
    custody_same_scene(&recovered, &cold_control);
    custody_same_scene(host.inner.published.as_ref().unwrap(), &changed);
    recovered.apply_to(&mut host).unwrap();
    custody_same_scene(host.inner.published.as_ref().unwrap(), &recovered);
    percentage_gutter_assert(&changed, &p, true);
    percentage_gutter_assert(&cold, &p, false);
    assert_eq!(
        cold.final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        old_fragments
    );
}

#[test]
fn captured_percentage_tracks_accepted_scrollbar_gutter_and_recovers_f32() {
    accepted_gutter_updates_percentage_basis_atomically::<f32>();
}
#[test]
fn captured_percentage_tracks_accepted_scrollbar_gutter_and_recovers_f64() {
    accepted_gutter_updates_percentage_basis_atomically::<f64>();
}
