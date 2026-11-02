//! Native family demand refuses actual control-only Prepared execution.
//! Reuses its real empty captured source, Ahem preparation and endpoint bindings.
use super::*;

fn control_only_prepared_flow_rejects_native_family_demand<S: LayoutScalar>() {
    let f = prepare(empty_input(true, text::ParagraphStrut::DefaultFont));
    assert!(f.binding.origins.is_empty());
    assert!(f.binding.atomics.is_empty());
    assert_eq!(f.binding.nodes, [200, 201]);
    assert_eq!(f.binding.prepared.unit_views().count(), 0);
    assert_eq!(f.binding.prepared.control_views().count(), 2);
    assert_eq!(computed_group(&f, 1).view().source_position().get(), 0);
    let scope = f.processed.captured_scope(1).unwrap();
    assert_eq!(scope.role(), text::ScopeRole::Inline);
    assert_eq!(
        scope.entry_control(),
        f.binding.controls[0].occurrence.captured_control()
    );
    assert_eq!(
        scope.exit_control(),
        f.binding.controls[1].occurrence.captured_control()
    );
    assert_eq!(scope.parent(), Some(f.processed.captured_scope(0).unwrap()));
    let mut tree = tree::<S>(&f, false);
    let parent = crate::LayoutTree::node_input(&tree, 0);
    let axes = crate::FlowAxes::new(parent.writing_mode, parent.direction);
    assert_eq!(
        axes,
        crate::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
    );
    for node in [200, 201] {
        let original = match crate::LayoutTree::layout_input(&tree, node) {
            LayoutInputViewOf::InlineBoundary(input) => input,
            _ => panic!("the maintained fixture owns exactly these two native endpoints"),
        };
        assert_eq!(original.edge(), InlineBoundaryEdgeOf::ZERO);
        assert_eq!(
            (original.writing_mode(), original.direction()),
            (WritingMode::HorizontalTb, Direction::Ltr)
        );
        // Only the actual operation changes. Components, metrics, formatter
        // fields and vertical alignment retain the maintained native values.
        let input = InlineBoundaryInputOf::box_edge(
            original.kind(),
            original.metrics(),
            original.edge(),
            axes,
        )
        .with_writing_mode(original.writing_mode())
        .with_direction(original.direction())
        .with_vertical_align(original.vertical_align());
        assert_eq!(input.parent_edge_axes(), Some(axes));
        tree.insert_input(node, LayoutInputOf::inline_boundary(input));
    }
    let actual_box = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope.clone()));
    let wrapping = crate::WrappingInput {
        scopes: vec![crate::WrappingScopeInput::Captured {
            scope: scope.clone(),
            wrap_inside: crate::WrapInside::Auto,
            edges: crate::WrapEdges::AUTO,
            metric_contribution: None,
        }],
        // The complete captured tree really is Block0 -> Inline1 -> no children.
        // Endpoint operations and the computed force are not generated child boxes.
        child_order: vec![
            crate::InlineChildOrderInput {
                parent: crate::GeneratedBoxRef::Node(0),
                children: vec![crate::GeneratedChildBoxInput {
                    box_: actual_box.clone(),
                    participation: crate::GeneratedBoxParticipation::InFlow,
                }],
                prefix: crate::ChildOrderCompleteness::Complete,
                suffix: crate::ChildOrderCompleteness::Complete,
                formatting_context: 0,
            },
            crate::InlineChildOrderInput {
                parent: actual_box,
                children: Vec::new(),
                prefix: crate::ChildOrderCompleteness::Complete,
                suffix: crate::ChildOrderCompleteness::Complete,
                formatting_context: 0,
            },
        ],
        gaps: Vec::new(),
        native_inline_boxes: Vec::new(),
    };
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![f.binding.clone()],
        wrapping.clone(),
        Vec::new(),
    )
    .expect("the genuine Prepared flow admits these exact complete generated-child proofs without native family demand");
    let prior = compute_layout_with_flow(
        &flow,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    let [placed] = prior.placed_text() else {
        panic!("one authentic Prepared output")
    };
    assert_eq!(placed.nodes(), &[200, 201]);
    assert!(placed.layout().source().text().is_empty());
    assert!(
        placed
            .layout()
            .glyph_runs()
            .iter()
            .all(|run| run.glyphs().is_empty() && run.advance() == 0.0)
    );
    assert_eq!(placed.layout().lines().len(), 1);
    let completions = placed.layout().nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let completion = completions[0].line.finalized_line();
    assert_eq!(completion.prepared_text(), f.binding.prepared);
    assert!(!completion.has_in_flow_content());
    assert_eq!(
        (
            completion.feedback().baseline,
            completion.feedback().line_height
        ),
        (8.0, 10.0)
    );
    assert_eq!(
        completion
            .selected_line()
            .request()
            .controls
            .iter()
            .map(|row| row.occurrence.clone())
            .collect::<Vec<_>>(),
        f.binding
            .controls
            .iter()
            .map(|row| row.occurrence.clone())
            .collect::<Vec<_>>()
    );

    let mut requested = wrapping;
    assert!(requested.native_inline_boxes.is_empty());
    assert!(requested.gaps.is_empty());
    requested
        .native_inline_boxes
        .push(crate::NativeInlineBoxFamilyInput { formatter: 0 });
    assert!(
        requested.gaps.is_empty(),
        "family demand must be refused independently of native gap requests"
    );
    let Err(error) =
        LayoutFlowInput::try_new(&tree, 0, vec![f.binding.clone()], requested, Vec::new())
    else {
        panic!(
            "native family demand must not be admitted for actual Prepared execution, even when all native slots are Structural"
        )
    };
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                crate::WrappingScopeError::UnsupportedIncidence
            ))
        ),
        "{error:?}"
    );
    assert_eq!(completion.prepared_text(), f.binding.prepared);
}

#[test]
fn native_family_on_control_only_prepared_flow_refuses_f32() {
    control_only_prepared_flow_rejects_native_family_demand::<f32>();
}

#[test]
fn native_family_on_control_only_prepared_flow_refuses_f64() {
    control_only_prepared_flow_rejects_native_family_demand::<f64>();
}
