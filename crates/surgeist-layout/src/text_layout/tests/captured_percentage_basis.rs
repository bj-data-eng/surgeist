//! Captured relative percentages use final normal content dimensions.
//! Source/font/normal allocation and visual-corner domains remain separate.
use super::*;

fn captured_fraction<S: LayoutScalar>(value: f64) -> LengthAutoOf<S> {
    LengthAutoOf::value(LengthPercentageOf::from_percent_fraction(custody_n::<S>(value)).unwrap())
}

fn captured_percent_atom<S: LayoutScalar>(left: bool) -> NodeInputOf<S> {
    let mut atom = custody_atom(left, 0.0);
    if left {
        atom.inset.left = captured_fraction(0.0064);
    } else {
        atom.inset.top = captured_fraction(0.06);
    }
    atom
}

fn captured_percent_source<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    left: bool,
    width: f64,
    height: f64,
) {
    for finalized in [false, true] {
        assert_eq!(
            custody_entry(batch, 0, finalized).output().size,
            Size::new(custody_n::<S>(width), custody_n::<S>(height))
        );
    }
    let placed = batch.placed_text();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].container(), 0);
    assert_eq!(
        placed[0].nodes(),
        if left { &[1, 2, 3][..] } else { &[1, 2][..] }
    );
    let layout = placed[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        if left { &[0x58, 0x58][..] } else { &[0x58][..] }
    );
    let lines = layout.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].range(),
        text::Range::new(0, if left { 2 } else { 1 })
    );
    assert_eq!(lines[0].baseline(), 10.0);
    let mut glyph_count = 0;
    for run in layout.run_views() {
        assert_eq!(
            run.face().bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .as_slice()
        );
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.id(), glyph.advance()), (58, 10.0));
            glyph_count += 1;
        }
    }
    assert_eq!(glyph_count, if left { 2 } else { 1 });
    let binding = paragraph.binding(0);
    for association in &binding.origins {
        let row = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|row| *row.owner() == crate::InlineFragmentOwner::Node(association.node))
            .unwrap()
            .fragment();
        let issued = row.finalized_fragment();
        let view = issued.view();
        assert_eq!(
            view.owner(),
            text::FinalizedFragmentOwner::Origin(association.origin.clone())
        );
        assert_eq!(view.local_range(), Some(text::Range::new(0, 1)));
        assert_eq!(row.baseline().y, custody_n::<S>(10.0));
        assert_eq!(row.rect().unwrap().origin().y, custody_n::<S>(2.0));
        let line = issued.finalized_line();
        let feedback = line.feedback();
        assert_eq!(
            (feedback.advance, feedback.baseline, feedback.line_height),
            (if left { 30.0 } else { 20.0 }, 10.0, 12.0)
        );
        assert_eq!(
            line.measurements().margin_edge_fit().required_inline_size(),
            if left { 30.0 } else { 20.0 }
        );
        if association.node == 3 {
            assert_eq!(row.rect().unwrap().origin().x, custody_n::<S>(20.0));
        }
    }
    custody_visual_agreement(batch);
    custody_contained_float(batch);
}

fn captured_percent_fixture<S: LayoutScalar>(
    p: &PreparedParagraph,
    left: bool,
) -> PublicLayoutTreeOf<S> {
    let binding = p.binding(0);
    let issued = p.prepared.atomic_views().next().unwrap();
    assert_eq!(binding.atomics.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(binding.atomics[0].occurrence, issued.handle());
    let tree = custody_restorable_fixture(p, left, 0.0).style(2, captured_percent_atom(left));
    assert!(tree.node_input(2).atomic_inline_participation.is_none());
    // Actual authored10.4 is retained independently of canonical10/visual11.
    assert_eq!(tree.node_input(2).size, custody_atom::<S>(left, 0.0).size);
    assert_eq!(tree.child_count(2), 1);
    assert_eq!(tree.child(2, 0), 4);
    assert_eq!(tree.node_input(4).float, Float::Left);
    assert_eq!(tree.node_input(4).float_exclusion, FloatExclusion::Shape);
    tree
}

fn captured_percent_matches_constant<S: LayoutScalar>(left: bool) {
    let p = custody_paragraph(left);
    // Match the documented S-lane coefficient/product, rather than compare
    // decimal f32 bits with a separately rounded .72 or .32 literal.
    let coefficient = custody_n::<S>(if left { 0.0064 } else { 0.06 });
    let basis = custody_n::<S>(if left { 50.0 } else { 12.0 });
    let resolved = coefficient * basis;
    let control_host =
        CustodyHost::<S>::new(custody_restorable_fixture(&p, left, resolved.to_f64()));
    let control = custody_compute(&control_host, vec![p.binding(0)], &[]);
    custody_assert_simple(&control, left, if left { 0.32 } else { 0.72 });
    captured_percent_source(&control, &p, left, 50.0, 12.0);
    let percent_host = CustodyHost::<S>::new(captured_percent_fixture(&p, left));
    let actual = custody_compute(&percent_host, vec![p.binding(0)], &[]);
    captured_percent_source(&actual, &p, left, 50.0, 12.0);
    //6% of final normal12 gives.72: measured top-.4+.72=.32,
    // end10.72 => visual0..11, with content1..10.
    // .64% of actual width50 gives.32: border10.32..20.72
    // =>10..21; content10.57..20.47 =>11..20.
    custody_assert_simple(&actual, left, if left { 0.32 } else { 0.72 });
    custody_same_scene(&actual, &control);
    custody_assert_simple(&control, left, if left { 0.32 } else { 0.72 });
}

fn captured_percent_changed_basis_preserves_publication<S: LayoutScalar>(left: bool) {
    let p = custody_paragraph(left);
    let mut host = CustodyHost::<S>::new(captured_percent_fixture(&p, left));
    let cold = custody_compute(&host, vec![p.binding(0)], &[]);
    captured_percent_source(&cold, &p, left, 50.0, 12.0);
    custody_assert_simple(&cold, left, if left { 0.32 } else { 0.72 });
    let old_fragments = cold
        .final_inline_fragments()
        .iter()
        .map(|row| row.fragment().finalized_fragment().clone())
        .collect::<Vec<_>>();
    cold.apply_to(&mut host).unwrap();
    let pending = custody_entry(&cold, 2, false);
    host.inner.hits.borrow_mut().clear();
    host.returned.borrow_mut().clear();
    let warm = custody_compute(&host, vec![p.binding(0)], &[]);
    assert!(host.inner.hits.borrow().contains(&2));
    assert!(host.returned.borrow().contains(&pending));
    captured_percent_source(&warm, &p, left, 50.0, 12.0);
    custody_same_scene(&warm, &cold);
    warm.apply_to(&mut host).unwrap();
    let prior = host.inner.published.as_ref().unwrap().clone();
    let prior_caches = host.inner.caches.clone();

    let (width, height, basis) = if left {
        let mut root = custody_root();
        root.size.width = PreferredSizeOf::px(custody_n::<S>(100.0));
        host.inner
            .tree
            .insert_input(0, LayoutInputOf::box_input(root));
        (100.0, 12.0, 100.0)
    } else {
        // A real following normal Block8 changes final content height
        //12->20. It is not visual overflow or a donated Text strut.
        host.inner.tree = host
            .inner
            .tree
            .clone()
            .children(0, [1, 2, 5])
            .children(5, [])
            .style(
                5,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(custody_n::<S>(50.0)),
                        PreferredSizeOf::px(custody_n::<S>(8.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        (50.0, 20.0, 20.0)
    };
    let dirty = if left { &[0][..] } else { &[0, 5][..] };
    let changed = custody_compute(&host, vec![p.binding(0)], dirty);
    captured_percent_source(&changed, &p, left, width, height);
    if !left {
        let block = custody_entry(&changed, 5, true).output();
        assert_eq!(block.location, Point::new(S::ZERO, custody_n::<S>(12.0)));
        assert_eq!(
            block.size,
            Size::new(custody_n::<S>(50.0), custody_n::<S>(8.0))
        );
    }
    let node = custody_entry(&changed, 2, true).output();
    let expected_location = if left { (11.0, 0.0) } else { (10.0, 1.0) };
    assert_eq!(
        node.location,
        Point::new(
            custody_n::<S>(expected_location.0),
            custody_n::<S>(expected_location.1)
        )
    );
    assert_eq!(
        node.size,
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
    );
    let geometry = node.scroll_geometry.unwrap();
    assert_eq!(geometry.content_box().origin(), Point::ZERO);
    assert_eq!(geometry.content_box().size(), node.size);
    assert_eq!(geometry.scrollable_overflow(), geometry.padding_box());
    // Top:6%*20=1.2, border.8..11.2=>1..11; content1.05..10.95=>1..11.
    // Left:.64%*100=.64, border10.64..21.04=>11..21;
    // content10.89..20.79=>11..21. Both keep normal10 and normal source/FIT.
    let resolved = custody_n::<S>(if left { 0.0064 } else { 0.06 }) * custody_n::<S>(basis);
    let mut control_tree = host.inner.tree.clone();
    control_tree.insert_input(
        2,
        LayoutInputOf::box_input(custody_atom(left, resolved.to_f64())),
    );
    let control_host = CustodyHost::<S>::new(control_tree);
    let constant = custody_compute(&control_host, vec![p.binding(0)], &[]);
    captured_percent_source(&constant, &p, left, width, height);
    custody_same_scene(&changed, &constant);
    assert_ne!(
        custody_entry(&changed, 2, true).output().size,
        custody_entry(&cold, 2, true).output().size
    );

    // This is the actual sink's late typed rejection, after successful
    // complete computation. No unobserved shape callback is claimed.
    host.inner.reject_batch = true;
    assert_eq!(
        changed.apply_to(&mut host),
        Err("reject complete replacement")
    );
    assert_eq!(host.inner.caches, prior_caches);
    custody_same_scene(host.inner.published.as_ref().unwrap(), &prior);
    captured_percent_source(&cold, &p, left, 50.0, 12.0);
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
    captured_percent_source(&cold, &p, left, 50.0, 12.0);
}

#[test]
fn captured_percent_top_uses_final_normal_height_f32() {
    captured_percent_matches_constant::<f32>(false);
}
#[test]
fn captured_percent_top_uses_final_normal_height_f64() {
    captured_percent_matches_constant::<f64>(false);
}
#[test]
fn captured_percent_left_keeps_normal_fit_and_following_source_f32() {
    captured_percent_matches_constant::<f32>(true);
}
#[test]
fn captured_percent_left_keeps_normal_fit_and_following_source_f64() {
    captured_percent_matches_constant::<f64>(true);
}
#[test]
fn captured_percent_changed_height_warm_publication_f32() {
    captured_percent_changed_basis_preserves_publication::<f32>(false);
}
#[test]
fn captured_percent_changed_height_warm_publication_f64() {
    captured_percent_changed_basis_preserves_publication::<f64>(false);
}
#[test]
fn captured_percent_changed_width_warm_publication_f32() {
    captured_percent_changed_basis_preserves_publication::<f32>(true);
}
#[test]
fn captured_percent_changed_width_warm_publication_f64() {
    captured_percent_changed_basis_preserves_publication::<f64>(true);
}
