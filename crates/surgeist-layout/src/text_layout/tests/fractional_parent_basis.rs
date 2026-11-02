//! Relative percentages consume the accepted measured normal content box.
use super::*;

fn fractional_parent_fixture<S: LayoutScalar>(
    p: &PreparedParagraph,
    offset: S,
) -> (PublicLayoutTreeOf<S>, TextParagraphBinding<u32>) {
    let mut binding = p.binding(0);
    assert_eq!(binding.nodes, vec![1, 2]);
    assert_eq!(binding.origins.len(), 1);
    assert_eq!(binding.origins[0].node, 1);
    assert_eq!(binding.atomics.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(
        binding.atomics[0].occurrence,
        p.prepared.atomic_views().next().unwrap().handle()
    );
    // Complete real encounter inventory, including the actual leading float.
    // It is not an original Text unit, synthetic control or atomic Source box.
    binding.nodes = vec![5, 1, 2];
    let mut parent = custody_root();
    parent.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
    parent.scrollbar_gutter = ScrollbarGutter::Auto;
    parent.scrollbar_width = crate::ScrollbarWidthOf::<S>::ZERO;
    let tree = custody_fixture(p, false, offset.to_f64())
        .children(0, [5, 1, 2])
        .children(5, [])
        .style(0, parent)
        .style(
            5,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(custody_n::<S>(1.0)),
                    PreferredSizeOf::px(custody_n::<S>(12.5)),
                ),
                ..NodeInputOf::default()
            },
        );
    assert_eq!(tree.node_input(0).size.height, PreferredSizeOf::AUTO);
    assert_eq!(
        tree.node_input(0).scrollbar_width,
        crate::ScrollbarWidthOf::<S>::ZERO
    );
    assert_eq!(tree.child_count(0), 3);
    assert_eq!(tree.child(0, 0), 5);
    assert_eq!(tree.child_count(2), 0);
    assert!(tree.node_input(2).atomic_inline_participation.is_none());
    assert_eq!(tree.node_input(2).size, custody_atom::<S>(false, 0.0).size);
    (tree, binding)
}

fn fractional_parent_premises<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    binding: &TextParagraphBinding<u32>,
) {
    use crate::output::{AtomicReadbackPhase, GeometryStateOf};
    let raw = custody_entry(batch, 0, false);
    let final_parent = custody_entry(batch, 0, true);
    assert_eq!(
        raw.output().size,
        Size::new(custody_n::<S>(50.0), custody_n::<S>(12.5))
    );
    assert_eq!(
        final_parent.output().size,
        Size::new(custody_n::<S>(50.0), custody_n::<S>(13.0))
    );
    // Inspect only actual pipeline-issued retained values, alongside public
    // source/geometry observations. No replacement normal receipt is created.
    let GeometryStateOf::OrdinaryUnrounded {
        measured_normal, ..
    } = raw.geometry_state()
    else {
        panic!("root must retain its actual measured normal output");
    };
    assert_eq!(measured_normal.location, Point::ZERO);
    assert_eq!(
        measured_normal.size,
        Size::new(custody_n::<S>(50.0), custody_n::<S>(12.5))
    );
    let measured_content = measured_normal.content_box.unwrap();
    assert_eq!(measured_content.origin(), Point::ZERO);
    assert_eq!(
        measured_content.size(),
        Size::new(custody_n::<S>(50.0), custody_n::<S>(12.5))
    );
    let GeometryStateOf::OrdinaryCanonical { normal, .. } = final_parent.geometry_state() else {
        panic!("final root must retain the actual canonical normal output");
    };
    assert_eq!(
        normal.size,
        Size::new(custody_n::<S>(50.0), custody_n::<S>(13.0))
    );
    assert_eq!(
        normal.content_box.unwrap().size(),
        Size::new(custody_n::<S>(50.0), custody_n::<S>(13.0))
    );
    let float_raw = custody_entry(batch, 5, false).output();
    assert_eq!(float_raw.location, Point::ZERO);
    assert_eq!(
        float_raw.size,
        Size::new(custody_n::<S>(1.0), custody_n::<S>(12.5))
    );
    let float_final = custody_entry(batch, 5, true).output();
    assert_eq!(float_final.location, Point::ZERO);
    assert_eq!(
        float_final.size,
        Size::new(custody_n::<S>(1.0), custody_n::<S>(13.0))
    );
    let atomic = custody_entry(batch, 2, false);
    let GeometryStateOf::CanonicalAtomic {
        measured_normal,
        normal,
        readback,
        ..
    } = atomic.geometry_state()
    else {
        panic!("actual captured atomic must retain its paired geometry");
    };
    assert_eq!(readback, AtomicReadbackPhase::Pending);
    assert_eq!(
        measured_normal.size,
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.4))
    );
    assert_eq!(
        normal.size,
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
    );

    let placed = batch.placed_text();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].container(), 0);
    assert_eq!(placed[0].nodes(), &[5, 1, 2]);
    let layout = placed[0].layout();
    assert_eq!(layout.source().text().code_units(), &[0x58]);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), text::Range::new(0, 1));
    assert_eq!(layout.lines()[0].baseline(), 10.0);
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
        assert_eq!(run.range(), text::Range::new(0, 1));
        for glyph in run.glyphs() {
            assert_eq!((glyph.id(), glyph.advance()), (58, 10.0));
            glyph_count += 1;
        }
    }
    assert_eq!(glyph_count, 1);
    let source = batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| *row.owner() == crate::InlineFragmentOwner::Node(1))
        .unwrap()
        .fragment();
    let issued = source.finalized_fragment();
    assert_eq!(
        issued.view().owner(),
        text::FinalizedFragmentOwner::Origin(binding.origins[0].origin.clone())
    );
    assert_eq!(issued.view().local_range(), Some(text::Range::new(0, 1)));
    assert_eq!(
        issued.view().aggregate_range(),
        Some(text::Range::new(0, 1))
    );
    // The real1px left float leaves49px. X10 + normal atomic10 fit beside it.
    assert_eq!(
        source.rect().unwrap().origin(),
        Point::new(custody_n::<S>(1.0), custody_n::<S>(2.0))
    );
    assert_eq!(
        source.rect().unwrap().size(),
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
    );
    assert_eq!(source.baseline().y, custody_n::<S>(10.0));
    let line = issued.finalized_line();
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (20.0, 10.0, 12.0)
    );
    assert_eq!(
        line.measurements().margin_edge_fit().required_inline_size(),
        20.0
    );
    custody_visual_agreement(batch);
}

fn measured_fractional_parent_basis<S: LayoutScalar>() {
    let p = custody_paragraph(false);
    let coefficient = custody_n::<S>(0.0385);
    let correct = coefficient * custody_n::<S>(12.5);
    let canonical_counterfactual = coefficient * custody_n::<S>(13.0);
    // These finite products straddle the real10.5 bottom-corner threshold;
    // neither assertion compares separately rounded f32 decimal-literal bits.
    assert!(correct > S::ZERO && correct < custody_n::<S>(0.5));
    assert!(
        canonical_counterfactual > custody_n::<S>(0.5)
            && canonical_counterfactual < custody_n::<S>(0.6)
    );
    let mut observations = Vec::new();
    let mut original_fragments = None;
    for case in 0..3 {
        let offset = if case == 1 {
            canonical_counterfactual
        } else {
            correct
        };
        let (mut tree, binding) = fractional_parent_fixture(&p, offset);
        if case == 2 {
            let mut atomic = custody_atom(false, 0.0);
            atomic.inset.top = LengthAutoOf::value(
                LengthPercentageOf::from_percent_fraction(coefficient).unwrap(),
            );
            tree.insert_input(2, LayoutInputOf::box_input(atomic));
        }
        let host = CustodyHost::<S>::new(tree);
        let batch = custody_compute(&host, vec![binding.clone()], &[]);
        // Qualify the actual measured/canonical basis and native packet first.
        fractional_parent_premises(&batch, &binding);
        let atomic = custody_entry(&batch, 2, true).output();
        assert_eq!(atomic.location, Point::new(custody_n::<S>(11.0), S::ZERO));
        let expected_height = if case == 1 { 11.0 } else { 10.0 };
        // Normal raw top-.4 +.48125 gives.08125..10.48125=>0..10.
        // Using canonical13 gives.1005..10.5005=>0..11 instead.
        assert_eq!(
            atomic.size,
            Size::new(custody_n::<S>(10.0), custody_n::<S>(expected_height))
        );
        let content = atomic.scroll_geometry.unwrap().content_box();
        assert_eq!(content.origin(), Point::ZERO);
        assert_eq!(
            content.size(),
            Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
        );
        if case == 0 {
            original_fragments = Some(
                batch
                    .final_inline_fragments()
                    .iter()
                    .map(|row| row.fragment().finalized_fragment().clone())
                    .collect::<Vec<_>>(),
            );
        }
        observations.push(batch);
    }
    // Compare semantic/source/geometry observations across independently issued
    // finalized owners. Old retained publication identities stay unchanged.
    let original_fragments = original_fragments.unwrap();
    custody_same_scene(&observations[2], &observations[0]);
    assert_ne!(
        custody_entry(&observations[2], 2, true).output().size,
        custody_entry(&observations[1], 2, true).output().size
    );
    assert_eq!(
        observations[0]
            .final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        original_fragments
    );
}

#[test]
fn captured_percentage_uses_measured_fractional_parent_not_canonical_height_f32() {
    measured_fractional_parent_basis::<f32>();
}
#[test]
fn captured_percentage_uses_measured_fractional_parent_not_canonical_height_f64() {
    measured_fractional_parent_basis::<f64>();
}

#[path = "fractional_parent_origin.rs"]
mod fractional_parent_origin;
