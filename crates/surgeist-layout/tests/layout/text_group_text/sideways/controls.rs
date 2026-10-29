//! Real native controls retain semantic receipts and source-free physical slots.
use super::geometry::{available, break_all, fixture, paragraph, run};
use super::*;

fn forced_completion<S: l::LayoutScalar>() {
    for twice in [false, true] {
        let mut host = fixture::<S>(l::Direction::Ltr, Some(40.0), Some(15.0));
        let parts = if twice {
            vec![Part::Text("A"), Part::Forced, Part::Forced]
        } else {
            vec![Part::Text("A"), Part::Forced]
        };
        let p = paragraph(
            &mut host,
            0,
            &parts,
            style(),
            t::ParagraphBaseDirection::LeftToRight,
        );
        let batch = run(&host, &[&p], available(40.0, 15.0), &[]);
        let layout = placed(&batch, &p);
        assert_eq!(layout.source().text().code_units(), [65]);
        let count = if twice { 2 } else { 1 };
        assert_eq!(layout.lines().len(), count);
        assert_eq!(layout.nonformatted_lines().count(), 1);
        // The second real Forced line has no origin, so recover its issued
        // receipt from the genuine control fragment rather than inventing text.
        let mut selected = vec![None; count];
        for row in batch.final_inline_fragments() {
            let fragment = row.fragment();
            let line = fragment.finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), p.prepared);
            selected[fragment.line_index()] = Some(line);
        }
        for (i, line) in selected.into_iter().enumerate() {
            let line = line.expect("actual Forced control has a retained fragment");
            assert_eq!(
                line.selected_line().request().context.end,
                t::LineEnd::Forced
            );
            assert_eq!(
                line.selected_line().request().controls[0].occurrence,
                p.binding.controls[i].occurrence
            );
            assert_eq!(line.feedback().line_height, 10.0);
            assert_eq!(line.feedback().advance, if i == 0 { 10.0 } else { 0.0 });
            assert_eq!(
                layout.lines()[i].baseline(),
                if i == 0 { 32.0 } else { 22.0 }
            );
            assert_eq!(
                layout.lines()[i].bounds(),
                t::Rect::new(if i == 0 { 30.0 } else { 20.0 }, 0.0, 10.0, 15.0)
            );
        }
        let completion = layout.nonformatted_lines().next().unwrap();
        let line = completion.line.finalized_line();
        assert_eq!(line.prepared_text(), p.prepared);
        assert_eq!(
            line.selected_line().request().context.end,
            t::LineEnd::Paragraph
        );
        assert_eq!(
            line.selected_line().request().start,
            line.selected_line().request().end
        );
        assert!(!line.has_in_flow_content());
        assert_eq!(line.feedback().advance, 0.0);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (8.0, 10.0)
        );
        // Nominal8/10 is not an allocated baseline. The retained zero-height
        // used origin is exactly the next block edge (right40 minus10/20).
        assert_eq!(
            completion.origin,
            t::Point::new(if twice { 20.0 } else { 30.0 }, 0.0)
        );
        assert_eq!(
            completion.orientation,
            t::LineOrientation::SidewaysClockwise
        );
        assert_eq!(
            layout.metrics().size(),
            t::Size::new(if twice { 20.0 } else { 10.0 }, 10.0)
        );
        assert_eq!(
            layout
                .run_views()
                .flat_map(|r| r.glyphs().map(|g| (g.id(), r.range())).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [(35, t::Range::new(0, 1))]
        );
    }
}
#[test]
fn actual_forced_and_nonformatted_receipts_keep_distinct_geometry_f32() {
    forced_completion::<f32>();
}
#[test]
fn actual_forced_and_nonformatted_receipts_keep_distinct_geometry_f64() {
    forced_completion::<f64>();
}

fn endpoints<S: l::LayoutScalar>() {
    let mut host = fixture::<S>(l::Direction::Ltr, Some(40.0), Some(15.0));
    let p = paragraph(
        &mut host,
        0,
        &[Part::Start(2.0), Part::Text("A"), Part::End(3.0)],
        style(),
        t::ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(p.binding.nodes.len(), 3);
    assert_eq!(p.binding.controls.len(), 2);
    assert_eq!(p.prepared.atomic_views().count(), 0);
    let batch = run(&host, &[&p], available(40.0, 15.0), &[]);
    let layout = placed(&batch, &p);
    assert_eq!(layout.source().text().code_units(), [65]);
    let rows = receipts(&batch, &p);
    assert_eq!(rows.len(), 1);
    let line = &rows[0];
    assert_eq!(line.feedback().advance, 15.0);
    assert_eq!(
        line.control_slots()
            .map(|slot| (slot.control(), slot.inline_bounds(), slot.level().get()))
            .collect::<Vec<_>>(),
        [
            (p.binding.controls[0].occurrence.clone(), (0.0, 2.0), 0),
            (p.binding.controls[1].occurrence.clone(), (12.0, 15.0), 0),
        ]
    );
    // Endpoint node footprints are at the actual baseline and have zero cross
    // extent. Their source-free Control fragments retain the full metric band.
    for ((binding, y), advance) in p.binding.controls.iter().zip([0.0, 12.0]).zip([2.0, 3.0]) {
        let output = node_output(&batch, binding.node);
        assert_eq!(
            output.location,
            l::Point::new(S::from_f64(32.0), S::from_f64(y))
        );
        assert_eq!(output.size, l::Size::new(S::ZERO, S::from_f64(advance)));
        assert_eq!(output.content_size, l::Size::ZERO);
        assert_eq!(
            output.source_index,
            l::SourceIndex::new(if y == 0.0 { 0 } else { 2 })
        );
        let fragment = batch.final_inline_fragments().iter().find(|row| matches!(row.owner(), l::InlineFragmentOwner::Node(node) if *node == binding.node)).unwrap().fragment();
        let view = fragment.finalized_fragment().view();
        assert_eq!(
            view.owner(),
            t::FinalizedFragmentOwner::Control(binding.occurrence.clone())
        );
        assert!(view.origin().is_none());
        assert!(view.aggregate_range().is_none());
        let rect = fragment.rect().unwrap();
        assert_eq!(
            rect.origin(),
            l::Point::new(S::from_f64(30.0), S::from_f64(y))
        );
        assert_eq!(
            rect.size(),
            l::Size::new(S::from_f64(10.0), S::from_f64(advance))
        );
    }
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        t::Point::new(32.0, 2.0)
    );
    let positions = layout.positions().collect::<Vec<_>>();
    let at = |index, y| {
        positions
            .iter()
            .find(|position| {
                position.source_cursor().index() == index
                    && layout.position_geometry(position).unwrap().rect()
                        == t::Rect::new(30.0, y, 10.0, 1.0)
            })
            .expect("actual minted endpoint side")
    };
    for (index, before, after) in [(0, 0.0, 2.0), (1, 12.0, 15.0)] {
        let selection = layout
            .position_selection(at(index, before), at(index, after))
            .unwrap();
        assert!(!selection.is_collapsed());
        assert!(selection.source_pieces().is_empty());
        assert!(selection.source_text().is_empty());
        assert!(selection.plaintext().is_empty());
        assert_eq!(
            selection.geometry().rects()[0].rect(),
            t::Rect::new(30.0, before, 10.0, after - before)
        );
    }
    let selection = layout.position_selection(at(0, 0.0), at(1, 15.0)).unwrap();
    assert_eq!(selection.source_text().code_units(), [65]);
    assert_eq!(selection.plaintext().code_units(), [65]);
}
#[test]
fn real_source_free_boundary_slots_and_exact_positions_f32() {
    endpoints::<f32>();
}
#[test]
fn real_source_free_boundary_slots_and_exact_positions_f64() {
    endpoints::<f64>();
}

fn refusal_transaction<S: l::LayoutScalar>() {
    let mut host = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::None);
    host.box_change(0, |root| {
        root.size = l::Size::new(super::geometry::px(40.0), super::geometry::px(15.0))
    });
    let p = Paragraph::new(
        &mut host,
        0,
        &[Part::Text("AB")],
        break_all(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let old = run(&host, &[&p], available(40.0, 15.0), &[]);
    old.apply_to(&mut host).unwrap();
    let old_bounds = placed(&old, &p).bounds();
    for wm in [l::WritingMode::SidewaysRl, l::WritingMode::VerticalRl] {
        host.box_change(0, |root| root.writing_mode = wm);
        let flow = l::LayoutFlowInput::try_new(
            &host,
            0,
            vec![p.binding.clone()],
            l::WrappingInput::default(),
        )
        .unwrap();
        let error = l::compute_layout_with_flow_invalidated(
            &flow,
            0,
            l::LayoutRootRequestOf::viewport(available(40.0, 15.0)).unwrap(),
            &[0],
        )
        .unwrap_err();
        let l::LayoutErrorKindOf::Text(error) = error.kind() else {
            panic!("actual Text owner must report orientation refusal: {error:?}");
        };
        assert_eq!(
            error.detail(),
            Some(&t::ErrorDetail::InvalidTextProtocol {
                reason: if wm == l::WritingMode::SidewaysRl {
                    t::ProtocolError::MismatchedContext {
                        field: t::ProtocolContextField::Orientation,
                    }
                } else {
                    t::ProtocolError::UnsupportedContext {
                        field: t::ProtocolContextField::Orientation,
                    }
                }
            })
        );
        assert_eq!(
            placed(host.published.batch.as_ref().unwrap(), &p).bounds(),
            old_bounds
        );
        assert_eq!(placed(&old, &p).bounds(), old_bounds);
    }
    // A separately issued preparation cannot replace the preparation while
    // retaining the old origin capability, even for identical original AB.
    let mut other = Host::<S>::new(l::Direction::Ltr, l::TextGroupAlign::None);
    let fresh = Paragraph::new(
        &mut other,
        0,
        &[Part::Text("AB")],
        break_all(),
        t::ParagraphBaseDirection::LeftToRight,
        false,
    );
    let mut stale = p.binding.clone();
    stale.prepared = fresh.prepared;
    host.box_change(0, |root| root.writing_mode = l::WritingMode::HorizontalTb);
    let error = l::LayoutFlowInput::try_new(&host, 0, vec![stale], l::WrappingInput::default())
        .err()
        .expect("foreign issued origins cannot satisfy preparation coverage");
    assert!(matches!(
        error.kind(),
        l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TextBinding(
            l::TextBindingError::OriginCoverage
        ))
    ));
    assert_eq!(
        placed(host.published.batch.as_ref().unwrap(), &p).bounds(),
        old_bounds
    );
    let restored = run(&host, &[&p], available(40.0, 15.0), &[0]);
    assert_eq!(placed(&restored, &p).bounds(), old_bounds);
}
#[test]
fn actual_orientation_and_foreign_origin_refusal_preserve_publication_f32() {
    refusal_transaction::<f32>();
}
#[test]
fn actual_orientation_and_foreign_origin_refusal_preserve_publication_f64() {
    refusal_transaction::<f64>();
}
