//! Descendant of tests/layout/node_free_completion.rs; reuse its authentic Fixture.
use super::*;

fn auto_intrinsics<S: l::LayoutScalar>(direction: l::Direction) {
    for force in [false, true] {
        let mut fixture = Fixture::<S>::new(direction, force);
        // Unlike the parent's explicitly30px case, both authored root axes
        // are genuinely auto, with the original zero edges/default constraints.
        fixture.tree.input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO);
        let pin = fixture.inventory_facts(force);
        for available in [l::AvailableOf::MIN_CONTENT, l::AvailableOf::MAX_CONTENT] {
            let batch = fixture.try_compute(fixture.inventory(), available).unwrap();
            let root = batch
                .final_entries()
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output();
            // No glyph, atomic or inline decoration contributes inline size.
            // The genuine Force keeps its20px block extent; metadata alone does not.
            assert_eq!(
                root.size,
                l::Size::new(S::ZERO, S::from_f64(if force { 20.0 } else { 0.0 }))
            );
            assert!(batch.final_inline_fragments().is_empty());
            if !force {
                assert!(batch.placed_text().is_empty());
                assert_eq!(fixture.inventory_facts(false), pin);
                continue;
            }
            assert_eq!(batch.placed_text().len(), 1);
            let entry = &batch.placed_text()[0];
            assert_eq!(entry.container(), 0);
            assert!(entry.nodes().is_empty());
            let layout = entry.layout();
            assert_eq!(layout.source(), fixture.prepared.analyzed_source().source());
            assert_eq!(
                layout.analyzed_source().identity(),
                fixture.prepared.analyzed_source().identity()
            );
            assert_eq!(layout.metrics().size(), t::Size::new(0.0, 20.0));
            assert_eq!(layout.metrics().first_baseline(), Some(16.0));
            assert_eq!(layout.metrics().last_baseline(), Some(16.0));
            let lines = layout.lines();
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].range(), t::Range::new(0, 0));
            assert_eq!(lines[0].bounds().size, t::Size::new(0.0, 20.0));
            assert_eq!(lines[0].baseline(), 16.0);
            assert_eq!(
                layout
                    .run_views()
                    .map(|run| run.glyphs().count())
                    .sum::<usize>(),
                0
            );
            let completions = layout.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(completions.len(), 1);
            assert_eq!(completions[0].origin.y, 20.0);
            let receipt = completions[0].line.finalized_line();
            assert_eq!(receipt.prepared_text(), fixture.prepared);
            assert!(!receipt.has_in_flow_content());
            let selected = receipt.selected_line();
            let request = selected.request();
            let offers = fixture
                .prepared
                .opportunity_views()
                .map(|offer| offer.handle())
                .collect::<Vec<_>>();
            assert_eq!(request.context.end, t::LineEnd::Paragraph);
            assert_eq!(
                request.termination,
                t::PreparedTermination::Ordinary(offers[1].clone())
            );
            assert_eq!(request.start, offers[0].view().boundary());
            assert_eq!(request.end, offers[1].view().boundary());
            assert_eq!(request.start.view().source_before().get(), 0);
            assert_eq!(request.end.view().source_after().get(), 0);
            assert!(request.participants.is_empty());
            assert!(request.controls.is_empty());
            assert!(request.atomics.is_empty());
            assert_struts(
                &request.line_box_struts,
                &[metric(16.0, 20.0), metric(12.0, 30.0)],
            );
            assert_eq!(receipt.feedback().advance, 0.0);
            assert_eq!(receipt.feedback().baseline, 16.0);
            assert_eq!(receipt.feedback().line_height, 34.0);
            assert_eq!(
                receipt.measurements().min_content_contribution().advance(),
                0.0
            );
            assert_eq!(
                receipt.measurements().max_content_contribution().advance(),
                0.0
            );
            assert_eq!(fixture.inventory_facts(true), pin);
        }
    }
}

#[test]
fn genuine_auto_root_intrinsics_keep_semantic_force_and_zero_inline_contribution() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        auto_intrinsics::<f32>(direction);
        auto_intrinsics::<f64>(direction);
    }
}
