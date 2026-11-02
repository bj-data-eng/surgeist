use super::fixtures::{
    Fri06C02StatefulTextTree, Fri06C05ShapeProvider, PublicFlowTree, RootSessionTree,
    computed_overflow, fri05_c03_block_root_state, fri05_c04_assert_initial_local_auto_state,
    fri05_c04_local_auto_state, fri05_c04_nested_flex_auto_tree, fri06_c02_final_node,
    fri06_c02_stateful_request, fri06_mr02_geometry_error_largest_finite,
    origin_fragment_associations, public_flow_output, public_layout_tree, scalar,
};
use super::*;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use surgeist_text::{Id, ParagraphBaseDirection, Source, WhiteSpace};

#[test]
fn fri08_c07_t02_scroll_source_root_preserves_error_identity_without_batch() {
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f32>();
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f64>();
}

#[test]
fn fri06_c04_float_lifecycle_cold_warm_dirty_failures_baseline_and_content_are_atomic_both_scalars()
{
    fn assert_lane<S: LayoutScalar>() {
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(40.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let mut tree = Fri06C02StatefulTextTree::new_float_mixed();
        let cold = tree
            .compute(request)
            .expect("cold float/line layout succeeds");
        let cold_unrounded = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_fragments = cold.unrounded_inline_fragments().to_vec();
        assert_eq!(
            public_flow_output(&cold_unrounded, 0).size.height,
            S::from_f64(12.5)
        );
        assert_eq!(
            public_flow_output(&cold_unrounded, 0).content_size,
            Size::new(S::from_f64(40.0), S::from_f64(12.5)),
        );
        assert_eq!(cold_fragments[0].fragment().baseline().y, S::from_f64(8.0));
        assert!(
            cold.cache_store_entries()
                .iter()
                .any(|entry| entry.node() == 1)
        );
        cold.apply_to(&mut tree).expect("cold float batch commits");

        let warm = tree
            .compute(request)
            .expect("warm float/line layout succeeds");
        assert_eq!(warm.unrounded_entries(), cold_unrounded);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(
            origin_fragment_associations(warm.unrounded_inline_fragments()),
            origin_fragment_associations(&cold_fragments)
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 1)
        );
        warm.apply_to(&mut tree)
            .expect("warm float batch recommits");

        let stale_float = tree.retained.unrounded_nodes[&1];
        let stale_text = tree.retained.unrounded_nodes[&2];
        tree.replace_float_inline_extent(25.75);
        tree.retained.dirty = vec![1, 1];
        let replacement = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect("dirty float replacement stages");
        assert_eq!(replacement.invalidated_nodes(), &[0, 1, 2]);
        replacement
            .apply_to(&mut tree)
            .expect("dirty float replacement commits");
        assert_ne!(tree.retained.unrounded_nodes[&1], stale_float);
        assert_eq!(
            tree.retained.unrounded_nodes[&1].size.width,
            S::from_f64(25.75)
        );
        assert_ne!(
            tree.retained.unrounded_nodes[&2].location,
            stale_text.location
        );
        assert!(tree.retained.dirty.is_empty());

        tree.replace_float_inline_extent(26.25);
        tree.retained.dirty = vec![1];
        tree.add_failing_float_path_control();
        let before_layout_failure = tree.retained.clone();
        let error = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect_err("invalid float-path control pairing rejects the whole layout batch");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(9));
        assert_eq!(tree.retained, before_layout_failure);

        tree.remove_failing_float_path_control();
        let rejected = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect("replacement after layout failure stages");
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            rejected.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_fragment_publication_preserves_actual_original_nodes_ranges_and_issued_anchors_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let base = ahem_style(10.0, 10.0);
        let mut collapsed = base.clone();
        collapsed.white_space = WhiteSpace::Collapse;
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(
                    1,
                    Source::identified("X\nX", Id::from_u64(11), 0),
                    base.clone(),
                ),
                ParagraphPart::text(2, Source::identified(" ", Id::from_u64(21), 0), collapsed),
            ],
            base,
            ParagraphBaseDirection::LeftToRight,
        );
        let inputs = HashMap::from([
            (
                0,
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                }),
            ),
            (1, paragraph.text_input(1)),
            (2, paragraph.text_input(2)),
        ]);
        let tree = public_layout_tree(
            inputs,
            HashMap::from([(0, vec![1, 2]), (1, Vec::new()), (2, Vec::new())]),
        );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![paragraph.binding(0)],
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(30.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
        let rows = origin_fragment_associations(batch.final_inline_fragments());
        let first: Vec<_> = rows.iter().filter(|row| row.0 == 1).collect();
        assert_eq!(
            first.iter().map(|row| row.2).collect::<Vec<_>>(),
            [
                surgeist_text::Range::new(0, 1),
                surgeist_text::Range::new(1, 2),
                surgeist_text::Range::new(2, 3)
            ]
        );
        assert!(first.iter().all(|row| row.1 == paragraph.origin(1)));
        let visible = first
            .iter()
            .filter(|row| row.2 != surgeist_text::Range::new(1, 2))
            .collect::<Vec<_>>();
        assert_eq!(visible.iter().map(|row| row.5).collect::<Vec<_>>(), [0, 1]);
        for (row, y) in visible.iter().zip([0.0, 10.0]) {
            let rect = row.3.unwrap();
            assert_eq!(rect.origin(), Point::new(S::ZERO, S::from_f64(y)));
            assert_eq!(rect.size(), Size::new(S::from_f64(10.0), S::from_f64(10.0)));
        }
        assert_eq!(
            first[1].3.unwrap().size().width,
            S::ZERO,
            "the original newline retains its genuine zero-advance occurrence"
        );
        let absent: Vec<_> = rows.iter().filter(|row| row.0 == 2).collect();
        assert_eq!(absent.len(), 1);
        assert_eq!(absent[0].1, paragraph.origin(2));
        assert_eq!(absent[0].2, surgeist_text::Range::new(0, 1));
        assert_eq!(
            absent[0].3.unwrap().size(),
            Size::new(S::ZERO, S::from_f64(10.0)),
            "the finalized zero-advance source retains Text's actual line-backed anchor"
        );
        let first_node = fri06_c02_final_node(&batch, 1);
        assert_eq!(first_node.location, Point::ZERO);
        assert_eq!(
            first_node.size,
            Size::new(S::from_f64(10.0), S::from_f64(20.0))
        );
        assert_eq!(first_node.content_size, first_node.size);
        assert_eq!(first_node.border, Edges::ZERO);
        assert_eq!(first_node.padding, Edges::ZERO);
        assert_eq!(first_node.margin, Edges::ZERO);
        assert!(first_node.scroll_geometry.is_none());
        assert_eq!(batch.placed_text().len(), 1);
        assert_eq!(batch.placed_text()[0].container(), 0);
        drop(batch);
        drop(capture);
        let mut hidden_inputs = HashMap::from([
            (
                0,
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                }),
            ),
            (1, paragraph.text_input(1)),
            (2, paragraph.text_input(2)),
        ]);
        let hidden = public_layout_tree(
            std::mem::take(&mut hidden_inputs),
            HashMap::from([(0, vec![1, 2]), (1, Vec::new()), (2, Vec::new())]),
        );
        let capture = LayoutFlowInput::try_new(
            &hidden,
            0,
            vec![paragraph.binding(0)],
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
        assert!(batch.final_inline_fragments().is_empty());
        assert!(batch.placed_text().is_empty());
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_atomicity_layout_and_preparation_failures_publish_nothing_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree =
            Fri06C02StatefulTextTree::new_shape_provider(Fri06C05ShapeProvider::Interval {
                minimum: S::ZERO,
                maximum: S::from_f64(8.0),
            });
        let cold = tree.compute(request).expect("provider layout succeeds");
        cold.apply_to(&mut tree).expect("provider batch commits");

        tree.shape_provider = Fri06C05ShapeProvider::Failure;
        tree.retained.dirty = vec![1];
        let before_layout_failure = tree.retained.clone();
        let error = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect_err("provider failure returns no batch");
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1,
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert!(matches!(error.kind(), LayoutErrorKindOf::Measurement(())));
        assert_eq!(tree.retained, before_layout_failure);
        assert_eq!(tree.retained.dirty, [1]);

        tree.shape_provider = Fri06C05ShapeProvider::Empty;
        let replacement = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect("provider recovery stages a complete batch");
        assert_eq!(replacement.invalidated_nodes(), &[0, 1, 2]);
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            replacement.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_lifecycle_mixed_cold_warm_rounding_dirty_replacement_scroll_and_failure_are_atomic_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new_mixed();
        let cold = tree.compute(request).expect("cold mixed layout succeeds");
        assert_eq!(cold.invalidated_nodes(), [0, 1, 3, 4]);
        let cold_unrounded_identity =
            origin_fragment_associations(cold.unrounded_inline_fragments());
        let cold_final_identity = origin_fragment_associations(cold.final_inline_fragments());
        assert_eq!(cold_unrounded_identity.len(), 1);
        let original = &cold_unrounded_identity[0];
        assert_eq!(
            (original.0, original.1.clone(), original.2, original.5),
            (
                1,
                tree.text.origins[0].origin.clone(),
                surgeist_text::Range::new(0, 1),
                0
            )
        );
        assert_eq!(
            cold_unrounded_identity
                .iter()
                .map(|row| (&row.1, row.2, row.5))
                .collect::<Vec<_>>(),
            cold_final_identity
                .iter()
                .map(|row| (&row.1, row.2, row.5))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            cold_unrounded_identity
                .iter()
                .map(|entry| entry.4)
                .collect::<Vec<_>>(),
            vec![Point::new(S::ZERO, S::from_f64(8.0))],
            "the mixed source must retain its exact unrounded baseline"
        );
        assert_eq!(
            cold_final_identity
                .iter()
                .map(|entry| entry.4)
                .collect::<Vec<_>>(),
            vec![Point::new(S::ZERO, S::from_f64(8.0))],
            "cumulative-origin rounding must publish the exact final baseline"
        );
        let cold_unrounded = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_unrounded_fragments = cold.unrounded_inline_fragments().to_vec();
        let cold_final_fragments = cold.final_inline_fragments().to_vec();
        let unrounded_root = cold_unrounded
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        let unrounded_range = unrounded_root.scroll_geometry.unwrap().physical_range();
        let placed = cold.placed_text()[0].layout();
        let lines = placed.lines();
        assert_eq!(lines.len(), 2);
        for (line, (top, baseline, height)) in
            lines.iter().zip([(0.0, 8.0, 10.0), (10.0, 20.0, 14.0)])
        {
            assert_eq!(
                line.bounds(),
                surgeist_text::Rect::new(0.0, top, 20.0, height)
            );
            assert_eq!(line.baseline(), baseline);
        }
        assert_eq!(lines[0].range(), surgeist_text::Range::new(0, 1));
        assert!(
            lines[1..]
                .iter()
                .all(|line| line.range() == surgeist_text::Range::new(1, 1))
        );
        // The legal Forced offer consumes the overwide atomic together with
        // the neutral boundary and the Forced control. Their canonical struts
        // are6/10 and8/10; atomic-before10 plus control-after4 makes14px.
        let control_line = cold
            .unrounded_inline_fragments()
            .iter()
            .find(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(3))
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        let feedback = control_line.feedback();
        assert_eq!(
            (
                feedback.ascent,
                feedback.descent,
                feedback.baseline,
                feedback.line_height
            ),
            (10.0, 4.0, 10.0, 14.0)
        );
        assert_eq!(
            control_line
                .selected_line()
                .request()
                .controls
                .iter()
                .map(|allocation| {
                    let strut = allocation.strut.unwrap();
                    (strut.baseline, strut.line_extent)
                })
                .collect::<Vec<_>>(),
            [(6.0, 10.0), (8.0, 10.0)]
        );
        // The actual native atomic shares baseline20. Its original10.5px
        // baseline gives raw top9.5, then canonical edges10..20 and0..25.
        let atomic_boxes = placed.inline_boxes();
        assert_eq!(atomic_boxes.len(), 1);
        assert_eq!(atomic_boxes[0].id(), surgeist_text::Id::from_u64(2));
        assert_eq!(atomic_boxes[0].line(), 1);
        assert_eq!(
            atomic_boxes[0].rect(),
            surgeist_text::Rect::new(0.0, 10.0, 25.0, 10.0)
        );
        for entries in [&cold_unrounded, &cold_final] {
            let atomic = entries
                .iter()
                .find(|entry| entry.node() == 2)
                .unwrap()
                .output();
            assert_eq!(atomic.location, Point::new(S::ZERO, S::from_f64(10.0)));
            assert_eq!(atomic.size, Size::new(S::from_f64(25.0), S::from_f64(10.0)));
        }
        // The real 10px text line plus the 14px atomic/Forced line end at 24, so
        // the 20px scrollport has 4px overflow. The final strut-only Paragraph
        // receiver retains its metric receipt but adds no used line extent.
        assert_eq!(
            unrounded_root.size,
            Size::new(S::from_f64(20.0), S::from_f64(20.0))
        );
        assert_eq!(unrounded_range.x().maximum(), S::from_f64(5.25));
        assert_eq!(unrounded_range.y().maximum(), S::from_f64(4.0));
        let final_root = cold_final
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            final_root.size,
            Size::new(S::from_f64(20.0), S::from_f64(20.0))
        );
        assert_eq!(
            final_root
                .scroll_geometry
                .unwrap()
                .physical_range()
                .y()
                .maximum(),
            S::from_f64(4.0)
        );
        let completions = placed.nonformatted_lines().cloned().collect::<Vec<_>>();
        assert_eq!(completions.len(), 1);
        let completion = completions[0].line.finalized_line();
        assert_eq!(completion.prepared_text(), tree.text.prepared);
        assert_eq!(
            (
                completion
                    .selected_line()
                    .request()
                    .start
                    .view()
                    .source_before()
                    .get(),
                completion
                    .selected_line()
                    .request()
                    .end
                    .view()
                    .source_after()
                    .get()
            ),
            (1, 1)
        );
        assert!(!completion.has_in_flow_content());
        assert_eq!(completion.feedback().baseline, 8.0);
        assert_eq!(completion.feedback().line_height, 10.0);
        assert_eq!(completions[0].origin.y, 24.0);
        let selected = completion.selected_line();
        let completion_request = selected.request();
        assert_eq!(
            completion_request.context.end,
            surgeist_text::LineEnd::Paragraph
        );
        assert!(matches!(&completion_request.termination,
            surgeist_text::PreparedTermination::Ordinary(offer)
                if offer.view().kind() == surgeist_text::PreparedBreakKind::ParagraphEnd));
        assert_eq!(completion_request.start, completion_request.end);
        assert_eq!(
            completion_request.start,
            control_line.selected_line().request().end
        );
        let source_indices = |entries: &[LayoutOutputEntryOf<u32, S>]| {
            entries
                .iter()
                .filter(|entry| (1..=4).contains(&entry.node()))
                .map(|entry| (entry.node(), entry.output().source_index))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            source_indices(&cold_unrounded),
            vec![
                (1, SourceIndex::new(0)),
                (2, SourceIndex::new(1)),
                (3, SourceIndex::new(2)),
                (4, SourceIndex::new(3)),
            ]
        );
        assert_eq!(source_indices(&cold_final), source_indices(&cold_unrounded));
        cold.apply_to(&mut tree).expect("cold mixed batch commits");
        assert!(tree.retained.unrounded_nodes.contains_key(&1));
        assert!(tree.retained.final_nodes.contains_key(&1));
        assert_eq!(tree.retained.unrounded_fragments[&1].len(), 1);
        assert_eq!(
            tree.retained.unrounded_fragments[&1],
            cold_unrounded_fragments
                .iter()
                .filter(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(1))
                .map(|entry| entry.fragment().clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            tree.retained.final_fragments[&1],
            cold_final_fragments
                .iter()
                .filter(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(1))
                .map(|entry| entry.fragment().clone())
                .collect::<Vec<_>>()
        );

        let warm = tree.compute(request).expect("warm mixed layout succeeds");
        assert_eq!(warm.invalidated_nodes(), [0, 1, 3, 4]);
        assert!(
            tree.cache_queries
                .borrow()
                .iter()
                .all(|(node, _)| ![0, 1, 3, 4].contains(node))
        );
        assert_eq!(warm.unrounded_entries(), cold_unrounded);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(
            warm.placed_text()[0]
                .layout()
                .nonformatted_lines()
                .cloned()
                .collect::<Vec<_>>(),
            completions
        );
        assert_eq!(
            origin_fragment_associations(warm.unrounded_inline_fragments()),
            origin_fragment_associations(&cold_unrounded_fragments)
        );
        assert_eq!(
            origin_fragment_associations(warm.final_inline_fragments()),
            origin_fragment_associations(&cold_final_fragments)
        );
        warm.apply_to(&mut tree)
            .expect("warm mixed batch recommits");

        let stale_atomic = tree.retained.unrounded_nodes[&2];
        tree.replace_atomic_inline_extent(27.75);
        assert_eq!(
            tree.node_input(2).size.width,
            PreferredSizeOf::px(S::from_f64(27.75)),
            "the authored replacement remains fractional"
        );
        tree.retained.dirty = vec![2, 2];
        let replacement = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect("dirty atomic replacement stages");
        assert_eq!(replacement.invalidated_nodes(), &[0, 1, 2, 3, 4]);
        assert_eq!(
            replacement.placed_text()[0]
                .layout()
                .nonformatted_lines()
                .cloned()
                .collect::<Vec<_>>(),
            completions
        );
        assert_eq!(
            origin_fragment_associations(replacement.unrounded_inline_fragments()),
            origin_fragment_associations(&cold_unrounded_fragments),
            "same actual preparation retains source associations during atomic replacement"
        );
        replacement
            .apply_to(&mut tree)
            .expect("dirty atomic replacement commits");
        assert_ne!(tree.retained.unrounded_nodes[&2], stale_atomic);
        // Native Text receives the accepted physical allocation. At x0 the
        // actual border edges round from0/27.75 to0/28 before publication,
        // including the batch's unrounded-node phase.
        assert_eq!(
            tree.retained.unrounded_nodes[&2].size.width,
            S::from_f64(28.0)
        );
        assert_eq!(tree.retained.final_nodes[&2].size.width, S::from_f64(28.0));

        tree.replace_atomic_inline_extent(29.25);
        tree.retained.dirty = vec![2];
        let rejected = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect("second dirty atomic replacement stages");
        tree.reject_preparation = true;
        let retained_before_rejection = tree.retained.clone();
        assert_eq!(
            rejected.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, retained_before_rejection);
        assert_eq!(tree.retained.dirty, [2]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_text_capture_bypasses_stale_geometry_cache_and_retains_actual_basis_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(
            Source::identified("X", Id::from_u64(61), 0),
            ahem_style(10.0, 10.0),
        );
        let original = tree.text.origins[0].origin.clone();
        let cold = tree.compute(request).expect("real prepared Text computes");
        assert_eq!(cold.invalidated_nodes(), [0, 1]);
        let cold_nodes = cold.unrounded_entries().to_vec();
        let cold_final = cold.final_entries().to_vec();
        let cold_fragments = origin_fragment_associations(cold.unrounded_inline_fragments());
        assert_eq!(cold_fragments.len(), 1);
        assert_eq!(cold_fragments[0].1, original);
        assert_eq!(cold_fragments[0].2, surgeist_text::Range::new(0, 1));
        assert_eq!(cold_fragments[0].3.unwrap().size().width, S::from_f64(10.0));
        cold.apply_to(&mut tree).unwrap();
        assert!(tree.retained.dirty.is_empty());
        tree.cache_queries.borrow_mut().clear();
        let warm = tree
            .compute(request)
            .expect("same actual preparation remains valid");
        assert_eq!(warm.unrounded_entries(), cold_nodes);
        assert_eq!(warm.final_entries(), cold_final);
        assert_eq!(
            origin_fragment_associations(warm.unrounded_inline_fragments()),
            cold_fragments
        );
        assert!(
            tree.cache_queries
                .borrow()
                .iter()
                .all(|(node, _)| ![0, 1].contains(node)),
            "actual captured Text and its ancestor bypass stale geometry cache lookup"
        );
        warm.apply_to(&mut tree).unwrap();
        let stale = tree.retained.clone();
        tree.replace_text(
            Source::identified("XX", Id::from_u64(62), 1),
            ahem_style(10.0, 10.0),
        );
        assert_ne!(tree.text.origins[0].origin, original);
        tree.retained.dirty = vec![1, 1];
        let dirty = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .unwrap();
        assert_eq!(dirty.invalidated_nodes(), [0, 1]);
        assert_eq!(tree.retained.dirty, [1, 1]);
        let sources = origin_fragment_associations(dirty.unrounded_inline_fragments());
        assert!(
            sources
                .iter()
                .all(|row| row.0 == 1 && row.1 == tree.text.origins[0].origin)
        );
        assert!(
            sources
                .iter()
                .all(|row| row.1.view().source().id() == Some(Id::from_u64(62)))
        );
        assert_eq!(sources.first().unwrap().2.start, 0);
        assert_eq!(sources.last().unwrap().2.end, 2);
        assert!(
            sources
                .windows(2)
                .all(|pair| pair[0].2.end == pair[1].2.start)
        );
        dirty.apply_to(&mut tree).unwrap();
        assert!(tree.retained.dirty.is_empty());
        assert_ne!(tree.retained.unrounded_nodes[&1], stale.unrounded_nodes[&1]);
        assert_ne!(
            tree.retained.unrounded_fragments[&1],
            stale.unrounded_fragments[&1]
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_text_zero_advance_source_fragment_survives_committed_state_and_fresh_capture_both_scalars()
 {
    fn assert_lane<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.white_space = WhiteSpace::Collapse;
        let mut tree =
            Fri06C02StatefulTextTree::new(Source::identified(" ", Id::from_u64(63), 0), style);
        let request = fri06_c02_stateful_request::<S>();
        // This stimulus is a collapsible SPACE removed at the line edge, not
        // actual retained text whose advance happens to be zero.
        let origin = tree.text.origins[0].origin.clone();
        let snapshot = |batch: &CompletedLayoutBatchOf<u32, S>| {
            assert!(batch.unrounded_inline_fragments().is_empty());
            assert!(batch.final_inline_fragments().is_empty());
            let placed = batch.placed_text()[0].layout();
            assert!(placed.lines().is_empty());
            let receipts = placed.nonformatted_lines().cloned().collect::<Vec<_>>();
            assert_eq!(receipts.len(), 1);
            let line = receipts[0].line.finalized_line();
            assert_eq!(line.prepared_text(), origin.prepared_text());
            assert!(!line.has_in_flow_content());
            let source = line
                .fragment_views()
                .map(|fragment| {
                    assert_eq!(fragment.origin(), Some(origin.clone()));
                    assert_eq!(
                        fragment.local_range(),
                        Some(surgeist_text::Range::new(0, 1))
                    );
                    assert_eq!(
                        fragment.aggregate_range(),
                        Some(surgeist_text::Range::new(0, 1))
                    );
                    let bounds = fragment
                        .bounds()
                        .expect("real nominal source anchor geometry");
                    assert_eq!(
                        (bounds.size.width, bounds.size.height),
                        (0.0, 10.0),
                        "nominal font-backed evidence remains without a rendered source rectangle"
                    );
                    (
                        fragment.owner(),
                        fragment.local_range(),
                        fragment.aggregate_range(),
                        fragment.bounds(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                source.len(),
                1,
                "the nonvisual occurrence retains its actual original association"
            );
            assert!(
                line.run_views()
                    .all(|run| run.glyphs().all(|glyph| glyph.advance() == 0.0))
            );
            (receipts, source)
        };
        let cold = tree.compute(request).unwrap();
        let source = snapshot(&cold);
        cold.apply_to(&mut tree).unwrap();
        assert!(tree.retained.unrounded_fragments[&1].is_empty());
        let warm = tree.compute(request).unwrap();
        assert_eq!(snapshot(&warm), source);
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_transaction_layout_and_preparation_failures_preserve_retained_state_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let request = fri06_c02_stateful_request::<S>();
        let mut tree = Fri06C02StatefulTextTree::new(
            Source::identified("X", Id::from_u64(81), 0),
            ahem_style(10.0, 10.0),
        );
        let cold = tree.compute(request).expect("cold text layout succeeds");
        cold.apply_to(&mut tree).expect("cold batch commits");
        assert!(
            tree.retained.unrounded_fragments.contains_key(&1),
            "transaction proof requires committed actual Text fragment state"
        );

        tree.replace_text(
            Source::identified("XX", Id::from_u64(82), 1),
            ahem_style(10.0, 10.0),
        );
        tree.retained.dirty = vec![1];
        tree.add_failing_noncanonical_control();
        let before_layout_failure = tree.retained.clone();

        let error = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect_err("noncanonical control pairing fails before retained-state mutation");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::NonBoxNodeRole {
                reason: NonBoxNodeRoleError::NonCanonicalNodeInput,
            }),
        );
        assert_eq!(tree.retained, before_layout_failure);

        tree.remove_failing_noncanonical_control();
        let replacement = tree
            .compute_invalidated(request, &tree.retained.dirty)
            .expect("dirty text replacement stages successfully");
        tree.reject_preparation = true;
        let before_preparation_failure = tree.retained.clone();
        assert_eq!(
            replacement.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before_preparation_failure);
        assert_eq!(tree.retained.dirty, [1]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn flex_item_root_uses_explicit_parent_axes_for_percentage_and_cache_in_both_scalar_lanes() {
    fn assert_lane<S: LayoutScalar>() {
        let parent_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let item_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let viewport = Size::new(
            AvailableOf::definite(scalar::<S>(200.0)),
            AvailableOf::definite(scalar::<S>(80.0)),
        );
        let available = Size::new(
            AvailableOf::definite(scalar::<S>(140.0)),
            AvailableOf::definite(scalar::<S>(300.0)),
        );
        let root_style = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::VerticalRl,
            direction: Direction::Rtl,
            padding: Edges::new(
                LengthOf::percent(scalar::<S>(0.0625)),
                LengthOf::percent(scalar::<S>(0.125)),
                LengthOf::percent(scalar::<S>(0.25)),
                LengthOf::percent(scalar::<S>(0.5)),
            ),
            ..NodeInputOf::<S>::default()
        };
        let child_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(scalar::<S>(10.0)),
                PreferredSizeOf::px(scalar::<S>(20.0)),
            ),
            ..NodeInputOf::<S>::default()
        };
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, root_style.clone());
        let request = LayoutRootRequestOf::flex_item_under_viewport(
            available,
            FlexItemRootContextOf::under_viewport(viewport, parent_axes)
                .expect("finite flex-item root context"),
        )
        .expect("finite flex-item root request");

        let cold = compute_layout(&tree, 0, request).expect("cold flex-item root layout");
        let root_entry = cold
            .cache_store_entries()
            .iter()
            .find(|entry| {
                entry.node() == 0 && entry.input().run_mode() == RunMode::PerformRootLayout
            })
            .expect("cold root compute is cached");
        let root_input = *root_entry.input();
        let root_output = root_entry.output();
        let descendant_tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, root_style)
            .with_style(1, child_style);
        let descendant_batch =
            compute_layout(&descendant_tree, 0, request).expect("root with descendant layout");
        let child_input = descendant_batch
            .cache_store_entries()
            .iter()
            .find(|entry| entry.node() == 1 && entry.input().run_mode() == RunMode::PerformLayout)
            .expect("child layout compute is cached")
            .input();

        assert_eq!(
            root_input.containing_layout_context(),
            ContainingLayoutContext::new(parent_axes, ParentFormattingContext::Flex)
        );
        assert_eq!(
            root_input.known(),
            Size::new(Some(scalar::<S>(140.0)), None)
        );
        assert_eq!(
            child_input.containing_layout_context(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::BlockFlow)
        );

        let root = public_flow_output(cold.unrounded_entries(), 0);
        let expected_padding = Edges::new(
            scalar::<S>(12.5),
            scalar::<S>(25.0),
            scalar::<S>(50.0),
            scalar::<S>(100.0),
        );
        assert_eq!(root.padding, expected_padding);
        let logical_padding = parent_axes.logical_edges(root.padding);
        assert_eq!(logical_padding.inline_start, scalar::<S>(100.0));
        assert_eq!(logical_padding.inline_end, scalar::<S>(25.0));
        assert_eq!(logical_padding.block_start, scalar::<S>(12.5));
        assert_eq!(logical_padding.block_end, scalar::<S>(50.0));

        let cache_context = CacheKeyContext::new();
        let mut cache = CacheOf::<S>::new();
        cache.store_with_context(&root_input, cache_context, root_output);
        assert_eq!(
            cache.get_with_context(&root_input, cache_context),
            Some(root_output)
        );
        let role_only = ComputeInputOf::flex_item_root(
            root_input.known(),
            root_input.parent(),
            ContainingLayoutContext::new(parent_axes, ParentFormattingContext::NoParent),
            root_input.available(),
        );
        assert_eq!(cache.get_with_context(&role_only, cache_context), None);
        let axes_only = ComputeInputOf::flex_item_root(
            root_input.known(),
            root_input.parent(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::Flex),
            root_input.available(),
        );
        assert_eq!(cache.get_with_context(&axes_only, cache_context), None);

        tree.apply_cache_entries(cold.cache_store_entries());
        tree.clear_cache_inputs();
        let warm = compute_layout(&tree, 0, request).expect("warm flex-item root layout");
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), 0),
            public_flow_output(cold.unrounded_entries(), 0)
        );
        assert_eq!(
            public_flow_output(warm.final_entries(), 0),
            public_flow_output(cold.final_entries(), 0)
        );
        assert!(
            warm.cache_store_entries()
                .iter()
                .all(|entry| entry.node() != 0),
            "the identical root context should hit the applied cold cache"
        );

        let viewport_tree = PublicFlowTree::default().with_children(0, []).with_style(
            0,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Rtl,
                ..NodeInputOf::<S>::default()
            },
        );
        let viewport_batch = compute_layout(
            &viewport_tree,
            0,
            LayoutRootRequestOf::viewport(available).expect("finite viewport request"),
        )
        .expect("viewport layout");
        let viewport_input = viewport_batch
            .cache_store_entries()
            .iter()
            .find(|entry| {
                entry.node() == 0 && entry.input().run_mode() == RunMode::PerformRootLayout
            })
            .expect("viewport root compute is cached")
            .input();
        assert_eq!(
            viewport_input.containing_layout_context(),
            ContainingLayoutContext::new(item_axes, ParentFormattingContext::NoParent)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn root_cache_input(available: Size<Available>) -> ComputeInput {
    ComputeInput::for_child(
        RunMode::PerformRootLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        available.map(Available::into_option),
        crate::ContainingLayoutContext::new(
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::NoParent,
        ),
        available,
    )
}

fn assert_fri06_mr02_geometry_error_public_root_has_no_batch<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    for display in [
        Display::Block,
        Display::Flex,
        Display::Grid,
        Display::GridLanes,
    ] {
        let style = NodeInputOf {
            display,
            size: Size::new(PreferredSizeOf::px(largest), PreferredSizeOf::px(S::ONE)),
            padding: Edges {
                left: LengthOf::px(largest),
                ..Edges::all(LengthOf::ZERO)
            },
            border: Edges {
                left: LengthOf::px(largest),
                ..Edges::all(LengthOf::ZERO)
            },
            grid_template_columns: vec![TrackComponentOf::AUTO],
            ..NodeInputOf::default()
        };
        let tree = public_layout_tree(
            HashMap::from([(0, LayoutInputOf::box_input(style))]),
            HashMap::from([(0, Vec::new())]),
        );
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(largest),
            AvailableOf::definite(S::ONE),
        ))
        .unwrap();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compute_layout(&tree, 0, request)
        }));
        let error = match outcome {
            Ok(Err(error)) => error,
            Ok(Ok(_)) => panic!("{display:?} overflow must publish no completed batch"),
            Err(_) => panic!("{display:?} overflow must not unwind"),
        };

        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidRootScrollGeometry
            )
        ));
    }
}

#[test]
fn fri06_mr02_geometry_error_public_root_preserves_no_publication_both_scalars() {
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f32>();
    assert_fri06_mr02_geometry_error_public_root_has_no_batch::<f64>();
}

#[test]
fn compute_layout_success_returns_completed_batch_without_tree_mutation() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.25), PreferredSize::px(20.5)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("root layout succeeds");

    assert_eq!(batch.unrounded_entries().len(), 1);
    assert_eq!(batch.unrounded_entries()[0].node(), 0);
    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(10.25, 20.5)
    );
    assert_eq!(batch.final_entries().len(), 1);
    assert_eq!(batch.final_entries()[0].node(), 0);
    assert_eq!(
        batch.final_entries()[0].output().size,
        Size::new(10.0, 21.0)
    );
}

#[test]
fn compute_layout_stages_cache_store_with_the_cold_root_output() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request).expect("cold root layout succeeds");

    assert_eq!(batch.cache_store_entries().len(), 1);
    let entry = &batch.cache_store_entries()[0];
    assert_eq!(entry.node(), 0);
    assert_eq!(entry.output().size, Size::new(10.0, 20.0));
    let mut applied_cache = Cache::new();
    applied_cache.store_with_context(entry.input(), entry.context(), entry.output());
    assert_eq!(
        applied_cache.get_with_context(entry.input(), entry.context()),
        Some(entry.output())
    );
}

#[test]
fn fri05_c05_grid_geometry_root_front_door_stages_only_canonical_ordinary_output() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
            scrollbar_width: ScrollbarWidth::try_new(10.0).unwrap(),
            grid_template_columns: vec![TrackComponent::px(120.0)],
            grid_template_rows: vec![TrackComponent::px(90.0)],
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("ordinary grid root layout succeeds");
    let unrounded = batch.unrounded_entries()[0].output();
    let geometry = unrounded
        .scroll_geometry
        .expect("ordinary grid root publishes canonical geometry");
    assert_eq!(geometry.used_overflow_x(), Overflow::Scroll);
    assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
    assert_eq!(geometry.target().border_box(), geometry.border_box());
    assert_eq!(unrounded.content_box_size(), geometry.content_box().size());
    assert_eq!(batch.cache_store_entries().len(), 1);
    assert_eq!(
        batch.cache_store_entries()[0]
            .output()
            .scroll_geometry
            .expect("stable root cache entry retains geometry"),
        geometry
    );
    assert!(batch.final_entries()[0].output().scroll_geometry.is_some());
}

#[test]
fn compute_layout_uses_a_matching_root_cache_hit_without_staging_a_store() {
    let style = NodeInput {
        size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
        ..NodeInput::default()
    };
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(0, style);
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let cached = ComputeOutput::from_outer_size(Size::new(33.0, 44.0));
    let mut cache = Cache::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cached);
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request).expect("cached root layout succeeds");

    assert_eq!(
        batch.unrounded_entries()[0].output().size,
        Size::new(33.0, 44.0)
    );
    assert!(batch.cache_store_entries().is_empty());
}

#[test]
fn compute_layout_root_diagnostics_reject_invalid_cached_scroll_geometry_without_batch() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(f32::NAN, 20.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let error = compute_layout(&tree, 0, request)
        .expect_err("invalid cached root output must not complete a layout batch");

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::InternalInvariant(LayoutInternalInvariant::InvalidRootScrollGeometry),
    );
}

#[test]
fn compute_layout_ignores_cached_container_output_until_the_subtree_is_complete() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Ok(Size::new(12.0, 8.0)));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let cached = ComputeOutput::from_outer_size(Size::new(33.0, 44.0));
    let mut cache = Cache::new();
    cache.store_with_context(&input, CacheKeyContext::new(), cached);
    tree.caches.borrow_mut().insert(0, cache);
    let request = LayoutRootRequest::viewport(available).unwrap();

    let batch = compute_layout(&tree, 0, request)
        .expect("a cached container request must return a complete layout batch");

    for node in [0, 1] {
        assert!(
            batch
                .unrounded_entries()
                .iter()
                .any(|entry| entry.node() == node)
        );
        assert!(
            batch
                .final_entries()
                .iter()
                .any(|entry| entry.node() == node)
        );
    }
    assert_ne!(
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .expect("root output must be staged")
            .output()
            .size,
        cached.size
    );
    let measured_nodes = tree.measured_nodes();
    assert!(!measured_nodes.is_empty());
    assert!(measured_nodes.iter().all(|node| *node == 1));
}

#[test]
fn compute_layout_cached_container_failure_returns_no_batch() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(33.0, 44.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(available).unwrap();

    let error = compute_layout(&tree, 0, request)
        .expect_err("a cached container must not hide a descendant provider failure");

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::Measurement("measure failed"),
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_stages_hidden_root_cache_clear_without_a_store() {
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::None,
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("hidden root layout succeeds");

    assert!(batch.cache_store_entries().is_empty());
    assert_eq!(batch.cache_clear_entries().len(), 1);
    assert_eq!(batch.cache_clear_entries()[0].node(), 0);
}

#[test]
fn compute_layout_failure_drops_staged_cache_effects_without_mutating_tree_cache() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .measure(1, Err("measure failed"));
    let available = Size::new(Available::definite(100.0), Available::definite(80.0));
    let input = root_cache_input(available);
    let mut cache = Cache::new();
    cache.store_with_context(
        &input,
        CacheKeyContext::new(),
        ComputeOutput::from_outer_size(Size::new(7.0, 9.0)),
    );
    tree.caches.borrow_mut().insert(0, cache);
    let before = tree.caches.borrow().clone();
    let request = LayoutRootRequest::viewport(available).unwrap();

    let result = compute_layout(&tree, 0, request);

    assert!(result.is_err());
    assert_eq!(*tree.caches.borrow(), before);
}

#[test]
fn compute_layout_preserves_nested_subgrid_resolution_failure() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::from(
                    LengthPercentageOf::px(20.0).expect("finite track"),
                )],
                grid_template_rows: vec![TrackComponent::from(
                    LengthPercentageOf::px(20.0).expect("finite track"),
                )],
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
                grid_template_rows: vec![TrackComponent::from(overflowing)],
                size: Size::new(PreferredSize::AUTO, PreferredSize::px(f32::MAX)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(20.0),
        Available::definite(20.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
}

fn fri05_c04_flex_auto_public_tree(nested: bool, child_size: Size<f32>) -> PublicFlowTree<f32> {
    let container = NodeInput {
        display: Display::Flex,
        overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
        scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        align_items: Some(AlignItems::FlexStart),
        ..NodeInput::default()
    };
    let absolute = NodeInput {
        display: Display::Block,
        position: Position::Absolute,
        size: child_size.map(PreferredSize::px),
        inset: Edges::new(
            LengthAuto::px(0.0),
            LengthAuto::AUTO,
            LengthAuto::AUTO,
            LengthAuto::px(0.0),
        ),
        ..NodeInput::default()
    };

    if nested {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [2])
            .with_children(2, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(1, container)
            .with_style(2, absolute)
    } else {
        PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(0, container)
            .with_style(1, absolute)
    }
}

#[test]
fn fri05_c04_flex_auto_nested_nonoverflow_keeps_local_initial_and_outer_cache_passes() {
    let tree = fri05_c04_nested_flex_auto_tree(false);
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let cold = compute_layout(&tree, 0, request).expect("cold nested flex layout succeeds");
    let outer = public_flow_output(cold.unrounded_entries(), 0);
    let inner = public_flow_output(cold.unrounded_entries(), 1);
    assert_eq!(
        outer.scroll_geometry.unwrap().scrollbar_size(),
        Size::new(0.0, 15.0)
    );
    assert_eq!(inner.scroll_geometry.unwrap().scrollbar_size(), Size::ZERO);

    let grandchild_requests = tree.cache_inputs(2);
    fri05_c04_assert_initial_local_auto_state(&grandchild_requests);
    assert!(
        grandchild_requests
            .iter()
            .all(|input| fri05_c03_block_root_state(input) == (false, false))
    );
    for node in [0, 1, 2, 3] {
        assert_eq!(
            cold.unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable cold output is published for node {node}"
        );
    }

    tree.apply_cache_entries(cold.cache_store_entries());
    tree.clear_cache_inputs();
    let warm = compute_layout(&tree, 0, request).expect("warm nested flex layout succeeds");
    for node in [0, 1, 2, 3] {
        assert_eq!(
            public_flow_output(warm.unrounded_entries(), node),
            public_flow_output(cold.unrounded_entries(), node),
            "warm output remains the stable cold output for node {node}"
        );
        assert_eq!(
            warm.unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable warm output is published for node {node}"
        );
    }
}

#[test]
fn fri05_c04_flex_auto_root_and_nested_publish_stable_output_with_exact_pass_cache_bits() {
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    for nested in [false, true] {
        for (child_size, expected_states, expected_scrollbars) in [
            (Size::new(80.0, 80.0), vec![(false, false)], Size::ZERO),
            (
                Size::new(120.0, 80.0),
                vec![(false, false), (true, false)],
                Size::new(0.0, 15.0),
            ),
            (
                Size::new(80.0, 120.0),
                vec![(false, false), (false, true)],
                Size::new(15.0, 0.0),
            ),
            (
                Size::new(120.0, 100.0),
                vec![(false, false), (true, false), (true, true)],
                Size::splat(15.0),
            ),
            (
                Size::new(100.0, 120.0),
                vec![(false, false), (false, true), (true, true)],
                Size::splat(15.0),
            ),
        ] {
            let tree = fri05_c04_flex_auto_public_tree(nested, child_size);
            let container = u32::from(nested);
            let absolute = container + 1;
            let cold = compute_layout(&tree, 0, request).expect("cold flex auto layout succeeds");
            let output = public_flow_output(cold.unrounded_entries(), container);
            assert_eq!(
                output.scroll_geometry.unwrap().scrollbar_size(),
                expected_scrollbars,
                "nested={nested}, child={child_size:?}"
            );
            for node in [container, absolute] {
                assert_eq!(
                    cold.unrounded_entries()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .count(),
                    1,
                    "only stable node {node} output is published for nested={nested}"
                );
            }

            let cache_inputs = tree.cache_inputs(absolute);
            assert!(
                cache_inputs
                    .iter()
                    .all(|input| fri05_c04_local_auto_state(input) == (false, false)),
                "nested={nested}, child={child_size:?}: child-local state must start at INITIAL"
            );
            assert_eq!(
                cache_inputs
                    .iter()
                    .map(fri05_c03_block_root_state)
                    .collect::<Vec<_>>(),
                expected_states,
                "nested={nested}, child={child_size:?}: {cache_inputs:#?}"
            );
            assert_eq!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| {
                        entry.node() == absolute
                            && entry.input().run_mode() == RunMode::PerformLayout
                    })
                    .map(|entry| fri05_c03_block_root_state(entry.input()))
                    .collect::<Vec<_>>(),
                expected_states,
                "nested={nested}, child={child_size:?}"
            );
            assert!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| entry.node() == absolute)
                    .all(|entry| fri05_c04_local_auto_state(entry.input()) == (false, false)),
                "nested={nested}, child={child_size:?}: cached child-local state stays INITIAL"
            );
            let child_cache_inputs = cold
                .cache_store_entries()
                .iter()
                .filter(|entry| {
                    entry.node() == absolute && entry.input().run_mode() == RunMode::PerformLayout
                })
                .map(LayoutCacheStoreEntryOf::input)
                .collect::<Vec<_>>();
            assert!(
                child_cache_inputs
                    .iter()
                    .all(|input| input.known() == child_cache_inputs[0].known()),
                "distinct containing passes stage separate entries for identical known child geometry"
            );
            assert!(
                cold.cache_store_entries()
                    .iter()
                    .filter(|entry| entry.node() == container)
                    .all(|entry| fri05_c03_block_root_state(entry.input()) == (false, false)),
                "no speculative container pass is cached under an ordinary request"
            );

            tree.apply_cache_entries(cold.cache_store_entries());
            tree.clear_cache_inputs();
            let warm = compute_layout(&tree, 0, request).expect("warm flex auto layout succeeds");
            assert_eq!(
                public_flow_output(warm.unrounded_entries(), container),
                public_flow_output(cold.unrounded_entries(), container),
                "nested={nested}, child={child_size:?}"
            );
            assert_eq!(
                public_flow_output(warm.final_entries(), container),
                public_flow_output(cold.final_entries(), container),
                "nested={nested}, child={child_size:?}"
            );
            for node in [container, absolute] {
                assert_eq!(
                    warm.unrounded_entries()
                        .iter()
                        .filter(|entry| entry.node() == node)
                        .count(),
                    1,
                    "only stable warm output is published for node {node}"
                );
            }
        }
    }
}

#[test]
fn native_line_policy_warm_width_policy_changes_and_late_sink_failure_are_atomic() {
    fn lane<S: LayoutScalar>(justification: TextJustification) {
        let mut style = ahem_style(10.0, 10.0);
        style.white_space = WhiteSpace::Collapse;
        let mut tree = Fri06C02StatefulTextTree::<S>::new(Source::new("X X X"), style);
        let origin = tree.text.origins[0].origin.clone();
        let spacing = match justification {
            TextJustification::Spaces => surgeist_text::SpacingPolicy::Spaces,
            TextJustification::InterWord => surgeist_text::SpacingPolicy::InterWord,
            TextJustification::None => panic!("this fixture qualifies actual allocation methods"),
            TextJustification::InterCharacter => {
                panic!("this fixture qualifies the existing separator allocation methods")
            }
        };
        let set = |tree: &mut Fri06C02StatefulTextTree<S>, alignment, last| {
            tree.inputs.insert(
                0,
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::Block,
                    text_align: alignment,
                    text_align_last: last,
                    text_justification: justification,
                    ..NodeInputOf::default()
                }),
            );
        };
        let request = |width| {
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(width)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap()
        };
        let advances = |batch: &CompletedLayoutBatchOf<u32, S>| {
            assert_eq!(
                batch.placed_text()[0].layout().source().text(),
                &surgeist_text::Text::from("X X X")
            );
            let mut lines = Vec::new();
            for entry in batch.final_inline_fragments() {
                let line = entry.fragment().finalized_fragment().finalized_line();
                assert_eq!(line.selected_line().request().context.spacing, spacing);
                if !lines.contains(&line) {
                    lines.push(line);
                }
            }
            lines
                .iter()
                .map(|line| line.feedback().advance)
                .collect::<Vec<_>>()
        };
        let inter_word_receipts = |batch: &CompletedLayoutBatchOf<u32, S>, extra| {
            if justification != TextJustification::InterWord {
                return;
            }
            let first = batch.final_inline_fragments()[0]
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(
                first
                    .selected_line()
                    .request()
                    .end
                    .view()
                    .source_after()
                    .get(),
                4
            );
            let selected = first.selected_line();
            let rows = first.spacing_slots().collect::<Vec<_>>();
            assert_eq!(rows.len(), 2);
            assert_eq!(
                selected.spacing_slots().collect::<Vec<_>>(),
                rows.iter().map(|row| row.slot.clone()).collect::<Vec<_>>()
            );
            assert_eq!(
                rows[0]
                    .slot
                    .pieces()
                    .map(|piece| piece.owner())
                    .collect::<Vec<_>>(),
                [surgeist_text::SpacingSlotOwner::Source {
                    origin: origin.clone(),
                    local: surgeist_text::Range::new(1, 2),
                    aggregate: surgeist_text::Range::new(1, 2),
                }]
            );
            assert_eq!(
                rows[0].geometry,
                surgeist_text::SpacingSlotGeometry::Independent {
                    intrinsic_advance: 10.0
                }
            );
            assert_eq!(
                rows[0].stretchability,
                surgeist_text::SpacingStretchability::Direct
            );
            assert!(rows[0].can_expand());
            assert_eq!(rows[0].authored_word_spacing, 0.0);
            assert_eq!((rows[0].allocation, rows[0].advance), (extra, 10.0 + extra));
            assert_eq!(
                rows[1].disposition,
                surgeist_text::SpacingSlotDisposition::DiscardedAtLineEdge
            );
            assert_eq!((rows[1].allocation, rows[1].advance), (0.0, 0.0));
            assert_eq!(first.measurements().alignment().advance(), 30.0 + extra);
            let separator = first
                .run_views()
                .flat_map(|run| {
                    run.glyphs()
                        .filter(|glyph| glyph.range() == surgeist_text::Range::new(1, 2))
                        .map(|glyph| (glyph.id(), glyph.advance(), run.origin().x + glyph.pen()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            // Actual raw10 SPACE, with half of the logical extra before paint.
            assert_eq!(separator, [(3, 10.0, 10.0 + extra / 2.0)]);
        };
        set(&mut tree, TextAlign::Justify, TextAlignLast::Auto);
        let cold = tree.compute(request(35.0)).unwrap();
        assert_eq!(advances(&cold), [35.0, 10.0]);
        inter_word_receipts(&cold, 5.0);
        let cold_entries = cold.final_entries().to_vec();
        cold.apply_to(&mut tree).unwrap();
        let warm = tree.compute(request(35.0)).unwrap();
        assert_eq!(advances(&warm), [35.0, 10.0]);
        inter_word_receipts(&warm, 5.0);
        assert_eq!(warm.final_entries(), cold_entries);
        warm.apply_to(&mut tree).unwrap();
        let wider = tree.compute(request(45.0)).unwrap();
        assert_eq!(advances(&wider), [45.0, 10.0]);
        inter_word_receipts(&wider, 15.0);
        wider.apply_to(&mut tree).unwrap();
        set(&mut tree, TextAlign::Start, TextAlignLast::Center);
        let changed = tree.compute_invalidated(request(35.0), &[0]).unwrap();
        assert_eq!(advances(&changed), [30.0, 10.0]);
        inter_word_receipts(&changed, 0.0);
        changed.apply_to(&mut tree).unwrap();
        set(&mut tree, TextAlign::Justify, TextAlignLast::Justify);
        let staged = tree.compute(request(35.0)).unwrap();
        assert_eq!(advances(&staged), [35.0, 10.0]);
        inter_word_receipts(&staged, 5.0);
        tree.reject_preparation = true;
        let before = tree.retained.clone();
        assert_eq!(
            staged.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(tree.retained, before);
    }
    lane::<f32>(TextJustification::Spaces);
    lane::<f64>(TextJustification::Spaces);
    lane::<f32>(TextJustification::InterWord);
    lane::<f64>(TextJustification::InterWord);
}

#[test]
fn native_fractional_spacing_budget_is_stable_across_real_warm_float_cache() {
    let mut style = ahem_style(10.0, 20.0);
    style.white_space = WhiteSpace::Collapse;
    let mut source = Source::new("X X X XXXX");
    for (index, size) in [4.640625_f32, 4.09375, 0.28125, 14.046875, 0.21875]
        .into_iter()
        .enumerate()
    {
        let mut span = style.clone();
        span.size = size;
        source.span(surgeist_text::Range::new(index, index + 1), span);
    }
    let mut tree = Fri06C02StatefulTextTree::<f64>::new(source, style);
    tree.inputs.insert(
        0,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            text_align: TextAlign::Justify,
            text_justification: TextJustification::Spaces,
            size: Size::new(PreferredSizeOf::px(31.0), PreferredSizeOf::AUTO),
            ..NodeInputOf::default()
        }),
    );
    tree.inputs.insert(
        9,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(PreferredSizeOf::px(1.0), PreferredSizeOf::px(100.0)),
            ..NodeInputOf::default()
        }),
    );
    tree.children.insert(0, vec![9, 1]);
    tree.children.insert(9, vec![]);
    tree.shape_provider = Fri06C05ShapeProvider::Interval {
        minimum: 0.0,
        maximum: 0.34875,
    };
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(31.0),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let facts = |batch: &CompletedLayoutBatchOf<u32, f64>| {
        assert_eq!(
            batch.placed_text()[0].layout().lines()[0].range(),
            surgeist_text::Range::new(0, 6)
        );
        let line = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(
            line.selected_line().request().context.available_inline,
            Some(30.65125)
        );
        assert!(line.feedback().advance <= 30.65125);
        let rows = line.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            line.selected_line().spacing_slots().collect::<Vec<_>>(),
            rows.iter().map(|row| row.slot.clone()).collect::<Vec<_>>()
        );
        assert!(rows[0].advance > 4.09375 && rows[0].advance.is_finite());
        assert!(rows[1].advance > 14.046875 && rows[1].advance.is_finite());
        assert_eq!(
            rows[2].disposition,
            surgeist_text::SpacingSlotDisposition::DiscardedAtLineEdge
        );
        assert_eq!(rows[2].advance, 0.0);
        (
            line.feedback(),
            rows.into_iter()
                .map(|row| {
                    (
                        row.slot.pieces().collect::<Vec<_>>(),
                        row.slot.participants().cloned().collect::<Vec<_>>(),
                        row.disposition,
                        row.advance,
                    )
                })
                .collect::<Vec<_>>(),
        )
    };
    let cold = tree.compute(request).unwrap();
    let cold_facts = facts(&cold);
    let cold_nodes = cold.final_entries().to_vec();
    cold.apply_to(&mut tree).unwrap();
    tree.cache_queries.borrow_mut().clear();
    let warm = tree.compute(request).unwrap();
    assert_eq!(facts(&warm), cold_facts);
    assert_eq!(warm.final_entries(), cold_nodes);
    assert!(
        tree.cache_queries
            .borrow()
            .iter()
            .any(|(node, hit)| *node == 9 && *hit),
        "genuine independent float cache is warm"
    );
    warm.apply_to(&mut tree).unwrap();
}

#[test]
fn inter_character_source_revision_and_late_sink_failure_preserve_completed_cache_and_geometry() {
    fn receipts<S: LayoutScalar>(
        batch: &CompletedLayoutBatchOf<u32, S>,
        source: &Source,
        extra: f64,
        ids: [u32; 3],
    ) {
        let layout = batch.placed_text()[0].layout();
        // Paragraph assembly owns the canonical text and explicit part style;
        // the original node's identity remains on its retained origin below.
        let composed = layout.source();
        assert_eq!(composed.id(), None);
        assert_eq!(composed.revision(), 0);
        assert_eq!(composed.text(), source.text());
        assert!(composed.boxes().is_empty());
        assert_eq!(composed.spans().len(), 1);
        assert_eq!(composed.spans()[0].range(), surgeist_text::Range::new(0, 3));
        let mut part_style = ahem_style(10.0, 10.0);
        part_style.white_space = WhiteSpace::Preserve;
        part_style.wrap = surgeist_text::Wrap::None;
        assert_eq!(composed.spans()[0].style(), &part_style);
        assert_eq!(layout.lines().len(), 1);
        assert_eq!(layout.lines()[0].range(), surgeist_text::Range::new(0, 3));
        // The real preceding 1px block contributes physical y=1. Ahem's
        // independently known 8px ascent then places the text baseline at9.
        assert_eq!(layout.lines()[0].bounds().origin.y, 1.0);
        let line = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        let selected = line.selected_line();
        assert_eq!(
            selected.request().context.spacing,
            surgeist_text::SpacingPolicy::InterCharacter
        );
        assert_eq!(
            selected.request().context.alignment,
            surgeist_text::Alignment::Justify
        );
        assert_eq!(
            selected.request().context.end,
            surgeist_text::LineEnd::Paragraph
        );
        let prepared = line.prepared_text();
        let origins = prepared.origin_views().collect::<Vec<_>>();
        assert_eq!(origins.len(), 1);
        let origin = origins[0].handle();
        assert_eq!(origin.view().source(), source);
        assert_eq!(origin.view().source().id(), Some(Id::from_u64(1566071)));
        assert_eq!(origin.view().source().revision(), source.revision());
        assert_eq!(origin.view().local_range(), surgeist_text::Range::new(0, 3));
        assert_eq!(
            origin.view().aggregate_range(),
            surgeist_text::Range::new(0, 3)
        );
        let rows = line.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            selected.spacing_slots().collect::<Vec<_>>(),
            rows.iter().map(|row| row.slot.clone()).collect::<Vec<_>>()
        );
        for (at, row) in rows.iter().enumerate() {
            let surgeist_text::SpacingSlotTarget::Boundary {
                visual_before,
                visual_after,
            } = row.slot.target()
            else {
                panic!("character allocation must preserve actual endpoint units")
            };
            for (unit, index) in [(visual_before, at), (visual_after, at + 1)] {
                assert_eq!(unit.letter_spacing_before(), 0.0);
                assert_eq!(unit.letter_spacing_after(), 0.0);
                assert_eq!(
                    unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
                    [surgeist_text::SpacingSlotOwner::Source {
                        origin: origin.clone(),
                        local: surgeist_text::Range::new(index, index + 1),
                        aggregate: surgeist_text::Range::new(index, index + 1),
                    }]
                );
            }
            assert_eq!(
                row.geometry,
                surgeist_text::SpacingSlotGeometry::Boundary(
                    surgeist_text::SpacingBoundaryGeometry::Independent
                )
            );
            assert_eq!(
                row.stretchability,
                surgeist_text::SpacingStretchability::Direct
            );
            assert!(row.can_expand());
            assert_eq!(row.authored_word_spacing, 0.0);
            assert_eq!(row.authored_letter_spacing, 0.0);
            assert_eq!((row.allocation, row.advance), (extra, extra));
        }
        // Registered Ahem has three raw10 glyphs. Two direct character gaps
        // share only the positive residual:34 gives2 each;38 gives4 each.
        assert_eq!(line.feedback().advance, 30.0 + 2.0 * extra);
        assert_eq!(
            line.measurements().alignment().advance(),
            30.0 + 2.0 * extra
        );
        assert_eq!(
            layout
                .clusters()
                .into_iter()
                .map(|cluster| cluster.range())
                .collect::<Vec<_>>(),
            [
                surgeist_text::Range::new(0, 1),
                surgeist_text::Range::new(1, 2),
                surgeist_text::Range::new(2, 3),
            ]
        );
        // Finalized glyph readback owns the source-to-glyph correspondence;
        // placed glyph readback below owns the physical pens and raw metrics.
        assert_eq!(
            line.run_views()
                .flat_map(|run| {
                    run.glyphs()
                        .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            [
                (surgeist_text::Range::new(0, 1), ids[0], 10.0_f32),
                (surgeist_text::Range::new(1, 2), ids[1], 10.0_f32),
                (surgeist_text::Range::new(2, 3), ids[2], 10.0_f32),
            ]
        );
        assert_eq!(
            layout
                .run_views()
                .flat_map(|run| {
                    assert_eq!(run.font_size(), 10.0);
                    assert_eq!(run.range(), surgeist_text::Range::new(0, 3));
                    assert_eq!(run.origin().y, 9.0);
                    run.glyphs()
                        .map(|glyph| {
                            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
                            (glyph.id(), glyph.advance(), run.origin().x + glyph.pen())
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            [
                (ids[0], 10.0_f32, 0.0),
                (ids[1], 10.0_f32, 10.0 + extra),
                (ids[2], 10.0_f32, 20.0 + 2.0 * extra),
            ]
        );
    }

    fn lane<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.white_space = WhiteSpace::Preserve;
        style.wrap = surgeist_text::Wrap::None;
        let original = Source::identified("ABC", Id::from_u64(1566071), 1);
        let mut tree = Fri06C02StatefulTextTree::<S>::new(original.clone(), style.clone());
        tree.inputs.insert(
            0,
            LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                text_align: TextAlign::Justify,
                text_align_last: TextAlignLast::Justify,
                text_justification: TextJustification::InterCharacter,
                ..NodeInputOf::default()
            }),
        );
        // Captured Text and its ancestor intentionally bypass geometry caches.
        // This real independent sibling proves that retained cache publication
        // participates in the same complete batch and survives a rejected sink.
        tree.inputs.insert(
            9,
            LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(S::ONE), PreferredSizeOf::px(S::ONE)),
                ..NodeInputOf::default()
            }),
        );
        // The producer captures a genuine BlockEnd terminal. Keep text last
        // in actual in-flow order rather than placing another block after it.
        tree.children.insert(0, vec![9, 1]);
        tree.children.insert(9, vec![]);
        assert_eq!(tree.children[&0], [9, 1]);
        let request = |width| {
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(width)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap()
        };
        let cold = tree.compute(request(34.0)).unwrap();
        receipts(&cold, &original, 2.0, [35, 36, 37]);
        let old_origin = tree.text.origins[0].origin.clone();
        let cold_entries = cold.final_entries().to_vec();
        cold.apply_to(&mut tree).unwrap();
        assert!(!tree.retained.caches.is_empty());
        tree.cache_queries.borrow_mut().clear();
        let warm = tree.compute(request(34.0)).unwrap();
        receipts(&warm, &original, 2.0, [35, 36, 37]);
        assert_eq!(warm.final_entries(), cold_entries);
        assert!(
            tree.cache_queries
                .borrow()
                .iter()
                .any(|(node, hit)| *node == 9 && *hit)
        );
        assert!(
            tree.cache_queries
                .borrow()
                .iter()
                .all(|(node, _)| ![0, 1].contains(node))
        );
        warm.apply_to(&mut tree).unwrap();
        let wider = tree.compute(request(38.0)).unwrap();
        receipts(&wider, &original, 4.0, [35, 36, 37]);
        wider.apply_to(&mut tree).unwrap();

        let revised = Source::identified("ABD", Id::from_u64(1566071), 2);
        tree.replace_text(revised.clone(), style);
        assert_ne!(tree.text.origins[0].origin, old_origin);
        tree.retained.dirty = vec![1];
        let before = tree.retained.clone();
        let staged = tree.compute_invalidated(request(34.0), &[1]).unwrap();
        receipts(&staged, &revised, 2.0, [35, 36, 38]);
        assert!(staged.invalidated_nodes().contains(&1));
        assert_eq!(
            tree.retained, before,
            "computing a complete replacement stages only"
        );
        tree.reject_preparation = true;
        assert_eq!(
            staged.apply_to(&mut tree),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(
            tree.retained, before,
            "rejection preserves nodes, fragments, caches and dirty state"
        );
        receipts(&wider, &original, 4.0, [35, 36, 37]);
        tree.reject_preparation = false;
        staged.apply_to(&mut tree).unwrap();
        assert!(tree.retained.dirty.is_empty());
        assert_ne!(tree.retained, before);
        assert_eq!(
            tree.retained.final_nodes[&1],
            fri06_c02_final_node(&staged, 1)
        );
        let replay = tree.compute(request(34.0)).unwrap();
        receipts(&replay, &revised, 2.0, [35, 36, 38]);
        assert_eq!(replay.final_entries(), staged.final_entries());
        replay.apply_to(&mut tree).unwrap();
    }

    lane::<f32>();
    lane::<f64>();
}
