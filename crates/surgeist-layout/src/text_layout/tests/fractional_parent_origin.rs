//! Genuine normal border placement changes publication corners, not used inset basis.
use super::*;

fn fractional_origin_premises<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    binding: &TextParagraphBinding<u32>,
    border: S,
) {
    use crate::output::{AtomicReadbackPhase, GeometryStateOf};
    let inset = if border == S::ZERO { 0.0 } else { 1.0 };
    let canonical_content = if border == S::ZERO { 13.0 } else { 12.0 };
    let raw_parent = custody_entry(batch, 0, false);
    let final_parent = custody_entry(batch, 0, true);
    assert_eq!(
        raw_parent.output().size,
        Size::new(custody_n::<S>(50.0), custody_n::<S>(12.5) + border)
    );
    assert_eq!(
        final_parent.output().size,
        Size::new(custody_n::<S>(50.0), custody_n::<S>(13.0))
    );
    let GeometryStateOf::OrdinaryUnrounded {
        measured_normal, ..
    } = raw_parent.geometry_state()
    else {
        panic!("actual root retains measured normal geometry");
    };
    let measured_content = measured_normal.content_box.unwrap();
    assert_eq!(measured_normal.location, Point::ZERO);
    assert_eq!(measured_content.origin(), Point::new(S::ZERO, border));
    assert_eq!(
        measured_content.size(),
        Size::new(custody_n::<S>(50.0), custody_n::<S>(12.5))
    );
    let GeometryStateOf::OrdinaryCanonical { normal, .. } = final_parent.geometry_state() else {
        panic!("actual root retains canonical normal geometry");
    };
    assert_eq!(
        normal.content_box.unwrap().origin(),
        Point::new(S::ZERO, custody_n::<S>(inset))
    );
    assert_eq!(
        normal.content_box.unwrap().size(),
        Size::new(custody_n::<S>(50.0), custody_n::<S>(canonical_content))
    );
    // The same actual float12.5 controls both measured content extents; its
    //0..12.5 and.72..13.22 corners publish0..13 and1..13 respectively.
    let raw_float = custody_entry(batch, 5, false).output();
    assert_eq!(raw_float.location, Point::new(S::ZERO, border));
    assert_eq!(
        raw_float.size,
        Size::new(custody_n::<S>(1.0), custody_n::<S>(12.5))
    );
    let final_float = custody_entry(batch, 5, true).output();
    assert_eq!(
        final_float.location,
        Point::new(S::ZERO, custody_n::<S>(inset))
    );
    assert_eq!(
        final_float.size,
        Size::new(custody_n::<S>(1.0), custody_n::<S>(canonical_content))
    );
    let GeometryStateOf::CanonicalAtomic {
        measured_normal,
        normal,
        readback,
        ..
    } = custody_entry(batch, 2, false).geometry_state()
    else {
        panic!("actual atomic retains paired measured/canonical allocation");
    };
    assert_eq!(readback, AtomicReadbackPhase::Pending);
    assert_eq!(
        measured_normal.size,
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.4))
    );
    assert_eq!(
        normal.location,
        Point::new(custody_n::<S>(11.0), custody_n::<S>(inset))
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
    assert_eq!(layout.lines()[0].baseline(), 10.0 + inset);
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
    assert_eq!(
        source.rect().unwrap().origin(),
        Point::new(custody_n::<S>(1.0), custody_n::<S>(2.0 + inset))
    );
    assert_eq!(
        source.rect().unwrap().size(),
        Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
    );
    assert_eq!(source.baseline().y, custody_n::<S>(10.0 + inset));
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

fn measured_parent_origin_changes_corners_not_percentage_basis<S: LayoutScalar>() {
    let paragraph = custody_paragraph(false);
    let coefficient = custody_n::<S>(0.04);
    let resolved = coefficient * custody_n::<S>(12.5);
    assert_eq!(resolved, custody_n::<S>(0.5));
    let wrong_shifted = coefficient * custody_n::<S>(12.0);
    assert!(wrong_shifted > custody_n::<S>(0.47) && wrong_shifted < custody_n::<S>(0.49));
    let mut first_publication = None;
    let mut first_fragments = None;
    let mut first_binding = None;
    for border in [S::ZERO, custody_n::<S>(0.72)] {
        let mut observations = Vec::new();
        // Qualify both literal constant frames before the percentage demand.
        for case in [0, 2, 1] {
            if border == S::ZERO && case == 2 {
                continue;
            }
            let offset = if case == 2 { wrong_shifted } else { resolved };
            let (mut tree, binding) = fractional_parent_fixture(&paragraph, offset);
            let mut parent = tree.node_input(0).clone();
            // This is a real used normal border, never relative translation.
            assert_eq!(parent.inset, Edges::all(LengthAutoOf::AUTO));
            parent.border.top = LengthOf::px(border);
            tree.insert_input(0, LayoutInputOf::box_input(parent));
            assert_eq!(tree.node_input(0).border.top, LengthOf::px(border));
            if case == 1 {
                let mut atomic = custody_atom(false, 0.0);
                atomic.inset.top = LengthAutoOf::value(
                    LengthPercentageOf::from_percent_fraction(coefficient).unwrap(),
                );
                tree.insert_input(2, LayoutInputOf::box_input(atomic));
            }
            let host = CustodyHost::<S>::new(tree);
            let batch = custody_compute(&host, vec![binding.clone()], &[]);
            // Authenticate actual measured12.5 and canonical13/12 before
            // comparing percentages. These are setup premises, not substitutes
            // for Source/capture/font or the real public placement observations.
            fractional_origin_premises(&batch, &binding, border);
            let atomic = custody_entry(&batch, 2, true).output();
            let inset = if border == S::ZERO { 0.0 } else { 1.0 };
            assert_eq!(
                atomic.location,
                Point::new(custody_n::<S>(11.0), custody_n::<S>(inset))
            );
            let height = if case == 2 { 10.0 } else { 11.0 };
            assert_eq!(
                atomic.size,
                Size::new(custody_n::<S>(10.0), custody_n::<S>(height))
            );
            let content = atomic.scroll_geometry.unwrap().content_box();
            assert_eq!(content.origin(), Point::ZERO);
            assert_eq!(
                content.size(),
                Size::new(custody_n::<S>(10.0), custody_n::<S>(10.0))
            );
            // Origin0: rawtop-.4+.5 =>.1..10.5 =>0..11.
            // Normal content-origin1: rawtop.6+.5 =>1.1..11.5 =>1..12,
            // content1.35..11.25=>1..11. Substituting canonicalH12 gives
            // .48:1.08..11.48=>1..11 instead, although measuredH stays12.5.
            if first_publication.is_none() {
                first_fragments = Some(
                    batch
                        .final_inline_fragments()
                        .iter()
                        .map(|row| row.fragment().finalized_fragment().clone())
                        .collect::<Vec<_>>(),
                );
                first_publication = Some(batch.clone());
                first_binding = Some(binding.clone());
            }
            observations.push(batch);
        }
        let percent_index = if border == S::ZERO { 1 } else { 2 };
        custody_same_scene(&observations[percent_index], &observations[0]);
        if border != S::ZERO {
            assert_ne!(
                custody_entry(&observations[percent_index], 2, true)
                    .output()
                    .size,
                custody_entry(&observations[1], 2, true).output().size
            );
        }
    }
    let first_publication = first_publication.unwrap();
    fractional_origin_premises(&first_publication, first_binding.as_ref().unwrap(), S::ZERO);
    assert_eq!(
        custody_entry(&first_publication, 2, true).output().size,
        Size::new(custody_n::<S>(10.0), custody_n::<S>(11.0))
    );
    assert_eq!(
        first_publication
            .final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        first_fragments.unwrap()
    );
}

#[test]
fn captured_measured_basis_is_independent_of_normal_content_origin_f32() {
    measured_parent_origin_changes_corners_not_percentage_basis::<f32>();
}
#[test]
fn captured_measured_basis_is_independent_of_normal_content_origin_f64() {
    measured_parent_origin_changes_corners_not_percentage_basis::<f64>();
}
