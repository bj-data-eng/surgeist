//! Native paired endpoints refuse inconsistent parent operations and bases.
//! Horizontal B LTR/P RTL cases preserve formatter validation, accepted strip
//! geometry and prior outputs through refusal and a separate valid retry.
use super::*;

fn accepted_geometry<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    let frame = FRAMES[0];
    assert!(batch.placed_text().is_empty());
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = |node| {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        assert_eq!(
            output(0).size,
            Size::new(S::from_f64(20.0), S::from_f64(10.0))
        );
        assert_eq!(output(3).location, Point::new(S::from_f64(4.0), S::ZERO));
        assert_eq!(
            output(3).size,
            Size::new(S::from_f64(10.0), S::from_f64(10.0))
        );
        // Keep the accepted physical packet/strip values unchanged.
        check_strip(output(1), frame, 0.0, 1.0, Side::Left, 1.0, 0.0);
        check_strip(output(2), frame, 14.0, 2.0, Side::Right, 1.0, 1.0);
        check_strip(output(4), frame, 1.0, 3.0, Side::Left, 2.0, 1.0);
        check_strip(output(5), frame, 16.0, 4.0, Side::Right, 1.0, 3.0);
        for (rank, node) in [1, 2, 3, 4, 5].into_iter().enumerate() {
            assert_eq!(output(node).source_index.get(), rank);
        }
    }
}
fn formatter_facts_unchanged<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) {
    for node in [1, 2, 4, 5] {
        let endpoint = tree.layout_input(node).as_inline_boundary().unwrap();
        assert_eq!(
            (endpoint.writing_mode(), endpoint.direction()),
            (WritingMode::HorizontalTb, Direction::Ltr)
        );
    }
}
fn refusal<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
) -> LayoutErrorOf<u32, S, std::convert::Infallible> {
    LayoutFlowInput::try_new(tree, 0, Vec::new(), wrapping(true), Vec::new())
        .and_then(|input| {
            compute_layout_with_flow(
                &input,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
            )
        })
        .expect_err("the invalid native parent request must not publish a replacement batch")
}
fn paired_parent_disagreement<S: LayoutScalar>(contribution_end: bool) {
    let frame = FRAMES[0];
    let valid_tree = fixture::<S>(frame, true);
    let prior = run(&valid_tree, true);
    accepted_geometry(&prior);
    let mut invalid = fixture::<S>(frame, true);
    let expected = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    // Only C's End4 changes. Genuine Cstart2 still declares its immediate
    // parent's RTL basis; P's endpoints and ALL formatter inputs stay B LTR.
    let replacement = if contribution_end {
        let original = invalid.layout_input(4).as_inline_boundary().unwrap();
        InlineBoundaryInputOf::contribution(original.kind(), original.metrics(), original.edge())
            .unwrap()
            .with_writing_mode(original.writing_mode())
            .with_direction(original.direction())
            .with_vertical_align(original.vertical_align())
    } else {
        endpoint(
            frame,
            InlineBoundaryKind::End,
            2.0,
            1.0,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        )
    };
    let actual = if contribution_end {
        None
    } else {
        Some(FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr))
    };
    assert_eq!(replacement.parent_edge_axes(), actual);
    invalid.insert_input(4, LayoutInputOf::inline_boundary(replacement));
    formatter_facts_unchanged(&invalid);
    let error = refusal(&invalid);
    // One actual pair has one operation/parent basis. End4 cannot contradict
    // the unchanged explicit BoxEdge Start2, including by becoming Contribution.
    // Fields describe that native declaration contradiction, not bidi parity.
    assert_eq!(
        invalid
            .layout_input(2)
            .as_inline_boundary()
            .unwrap()
            .parent_edge_axes(),
        Some(expected)
    );
    // These are invalid NATIVE scope-family facts. The owning public category is
    // WrappingScopeError, not a captured-control lowering diagnostic. Neither
    // basis disagreement nor an operation mismatch is silently downgraded.
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::InconsistentIncidence
            ))
        ),
        "{error:?}"
    );
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::InlineWrapping);
    // A separate valid operation remains independently admitted. A rejected
    // replacement neither mutates the retained batch nor downgrades its proof.
    let retry = run(&valid_tree, true);
    accepted_geometry(&retry);
    accepted_geometry(&prior);
}
fn cross_axis_parent<S: LayoutScalar>() {
    let frame = FRAMES[0];
    let valid_tree = fixture::<S>(frame, true);
    let prior = run(&valid_tree, true);
    accepted_geometry(&prior);
    let mut invalid = fixture::<S>(frame, true);
    let parent = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let formatter = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    // BOTH endpoints agree: this is axis refusal, never a mismatching pair or
    // an asserted supported orthogonal non-atomic layout. Formatter remains H.
    for (node, kind, border, padding) in [
        (2, InlineBoundaryKind::Start, 1.0, 1.0),
        (4, InlineBoundaryKind::End, 2.0, 1.0),
    ] {
        invalid.insert_input(
            node,
            LayoutInputOf::inline_boundary(endpoint(frame, kind, border, padding, parent)),
        );
    }
    formatter_facts_unchanged(&invalid);
    let error = refusal(&invalid);
    assert!(
        matches!(error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineBoundaryParentAxis {
            parent:reported_parent,formatter:reported_formatter,
        }) if *reported_parent==parent && *reported_formatter==formatter),
        "{error:?}"
    );
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 0,
            subject: 2
        }
    );
    assert_eq!(error.operation(), LayoutOperation::ChildLayout);
    let retry = run(&valid_tree, true);
    accepted_geometry(&retry);
    accepted_geometry(&prior);
}

#[test]
fn actual_native_pair_refuses_disagreeing_parent_bases_f32() {
    paired_parent_disagreement::<f32>(false);
}
#[test]
fn actual_native_pair_refuses_disagreeing_parent_bases_f64() {
    paired_parent_disagreement::<f64>(false);
}
#[test]
fn actual_native_pair_refuses_box_edge_contribution_disagreement_f32() {
    paired_parent_disagreement::<f32>(true);
}
#[test]
fn actual_native_pair_refuses_box_edge_contribution_disagreement_f64() {
    paired_parent_disagreement::<f64>(true);
}
#[test]
fn numeric_parent_axis_refuses_without_reinterpreting_formatter_f32() {
    cross_axis_parent::<f32>();
}
#[test]
fn numeric_parent_axis_refuses_without_reinterpreting_formatter_f64() {
    cross_axis_parent::<f64>();
}
