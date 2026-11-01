//! Explicit native-family demand requires complete actual proof without changing ordinary refusals.
use super::*;

fn demanded(wrapping: &WrappingInput<u32>) -> WrappingInput<u32> {
    let mut input = wrapping.clone();
    assert!(input.native_inline_boxes.is_empty());
    input
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    input
}

fn refused<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    input: WrappingInput<u32>,
    expected: WrappingScopeError,
) {
    let error = LayoutFlowInput::try_new(tree, 0, vec![], input, Vec::new())
        .err()
        .expect("an invalid actual family/proof cannot be downgraded to no demand");
    assert_eq!(error.operation(), crate::LayoutOperation::InlineWrapping);
    assert!(
        matches!(error.kind(),
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(reason))
            if *reason==expected),
        "expected {expected:?}, got {error:?}"
    );
}

fn refusal_preserves<S: LayoutScalar>(
    f: &Fixture<S>,
    expected: WrappingScopeError,
    ordinary_error: Option<WrappingScopeError>,
    change: impl FnOnce(&mut PublicLayoutTreeOf<S>, &mut WrappingInput<u32>),
) {
    // Both controls use the real unchanged supplied tree, six actual ten-pixel
    // atomics, exact NativePair20/21 and canonical generated proof helpers.
    let ordinary = f.flow();
    let old = compute_layout_with_flow(&ordinary, 0, request(n(100.0))).unwrap();
    let old_positions = atom_positions(&old);
    assert_eq!(old.native_inline_box_families().count(), 0);
    assert_eq!(old.native_inline_box_pieces().count(), 0);
    let valid_input = demanded(&f.wrapping);
    let valid =
        LayoutFlowInput::try_new(&f.tree, 0, vec![], valid_input.clone(), Vec::new()).unwrap();
    assert!(
        valid.paragraphs().is_empty(),
        "Native family admission fabricates no PreparedText"
    );
    let prior = compute_layout_with_flow(&valid, 0, request(n(100.0))).unwrap();
    let prior_positions = atom_positions(&prior);
    assert_eq!(prior.native_inline_box_families().count(), 1);
    let c = box_piece(&prior, 12).fragment();
    let a = box_piece(&prior, 13).fragment();
    assert_ne!(c, a);
    assert_eq!(c.parent(), a.parent());
    assert_eq!(c.parent().unwrap().owner(), GeneratedBoxRef::Node(0));

    // Change normalized public inputs only. The earlier admitted input/tree
    // remains intact; a rejection cannot mutate its retained proof or results.
    let mut other_tree = f.tree.clone();
    let mut invalid = valid_input;
    change(&mut other_tree, &mut invalid);
    let mut no_demand = invalid.clone();
    no_demand.native_inline_boxes.clear();
    if let Some(reason) = ordinary_error {
        refused(&other_tree, no_demand, reason);
    } else {
        // The family-specific stimulus must independently pass ordinary local
        // admission, rather than accidentally reproduce a malformed-order error.
        let allowed =
            LayoutFlowInput::try_new(&other_tree, 0, vec![], no_demand, Vec::new()).unwrap();
        let control = compute_layout_with_flow(&allowed, 0, request(n(100.0))).unwrap();
        assert_eq!(atom_positions(&control), old_positions);
        assert_eq!(control.native_inline_box_families().count(), 0);
        assert_eq!(control.native_inline_box_pieces().count(), 0);
    }
    refused(&other_tree, invalid, expected);
    assert_eq!(atom_positions(&old), old_positions);
    assert_eq!(atom_positions(&prior), prior_positions);
    assert_eq!(box_piece(&prior, 12).fragment(), c);
    assert_eq!(box_piece(&prior, 13).fragment(), a);
    let retry = compute_layout_with_flow(&valid, 0, request(n(100.0))).unwrap();
    assert_eq!(atom_positions(&retry), prior_positions);
    assert_eq!(box_piece(&retry, 12).fragment(), c);
    assert_eq!(box_piece(&retry, 13).fragment(), a);
    assert_eq!(
        retry.native_inline_box_families().collect::<Vec<_>>(),
        prior.native_inline_box_families().collect::<Vec<_>>()
    );
}

fn request_identity<S: LayoutScalar>() {
    let f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    refusal_preserves(&f, WrappingScopeError::DuplicateFamily, None, |_, input| {
        input
            .native_inline_boxes
            .push(NativeInlineBoxFamilyInput { formatter: 0 });
    });
    refusal_preserves(
        &f,
        WrappingScopeError::NodeNotReachable,
        None,
        |_, input| {
            // Node99 is genuinely absent from this supplied graph. A merely
            // different reachable formatter would not necessarily be foreign.
            input.native_inline_boxes[0].formatter = 99;
        },
    );
}

fn complete_family_proof<S: LayoutScalar>() {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    contribution_edges(&mut f, 0.0, 0.0);
    for parent in [GeneratedBoxRef::Node(0), pair(20, 21)] {
        refusal_preserves(
            &f,
            WrappingScopeError::MissingChildOrder,
            None,
            |_, input| {
                let old = input.child_order.len();
                input.child_order.retain(|row| row.parent != parent);
                assert_eq!(input.child_order.len() + 1, old);
            },
        );
        for prefix in [true, false] {
            refusal_preserves(
                &f,
                WrappingScopeError::IncompleteFamily,
                None,
                |_, input| {
                    let row = input
                        .child_order
                        .iter_mut()
                        .find(|row| row.parent == parent)
                        .unwrap();
                    assert_eq!(row.prefix, ChildOrderCompleteness::Complete);
                    assert_eq!(row.suffix, ChildOrderCompleteness::Complete);
                    if prefix {
                        row.prefix = ChildOrderCompleteness::Unknown;
                    } else {
                        row.suffix = ChildOrderCompleteness::Unknown;
                    }
                },
            );
        }
    }
    refusal_preserves(
        &f,
        WrappingScopeError::IncompleteFamily,
        None,
        |tree, input| {
            assert!(crate::Traverse::children(tree, 0).any(|node| node == 10));
            let row = input
                .child_order
                .iter_mut()
                .find(|row| row.parent == GeneratedBoxRef::Node(0))
                .unwrap();
            assert_eq!(
                (row.prefix, row.suffix),
                (
                    ChildOrderCompleteness::Complete,
                    ChildOrderCompleteness::Complete
                )
            );
            let old = row.children.len();
            row.children
                .retain(|child| child.box_ != GeneratedBoxRef::Node(10));
            assert_eq!(row.children.len() + 1, old);
            // This still-consistent relative root order does not falsely remove
            // node10 from the actual tree. Complete alone cannot certify closure.
        },
    );
    refusal_preserves(
        &f,
        WrappingScopeError::ChildOrder,
        Some(WrappingScopeError::ChildOrder),
        |_, input| {
            let row = input
                .child_order
                .iter_mut()
                .find(|row| row.parent == GeneratedBoxRef::Node(0))
                .unwrap();
            row.children.push(row.children[0].clone());
            // Independent duplicate-child inconsistency retains its old cause;
            // it is not flattened into the new complete-family capability error.
        },
    );
}

fn pair_input(start: u32, end: u32) -> WrappingScopeInput<u32> {
    WrappingScopeInput::NativePair {
        start,
        end,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    }
}
fn neutral<S: LayoutScalar>(kind: InlineBoundaryKind) -> LayoutInputOf<S> {
    LayoutInputOf::inline_boundary(
        InlineBoundaryInputOf::contribution(
            kind,
            metrics(),
            InlineBoundaryEdgeOf::try_new(S::ZERO, S::ZERO, S::ZERO).unwrap(),
        )
        .unwrap()
        .with_direction(Direction::Ltr),
    )
}

fn malformed_and_stale_pairs<S: LayoutScalar>() {
    let f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    refusal_preserves(
        &f,
        WrappingScopeError::CrossingScopes,
        Some(WrappingScopeError::CrossingScopes),
        |tree, input| {
            tree.insert_input(30, neutral(InlineBoundaryKind::Start));
            tree.insert_input(31, neutral(InlineBoundaryKind::End));
            tree.insert_children(0, [10, 11, 20, 12, 30, 13, 21, 14, 31, 15]);
            input.scopes.push(pair_input(30, 31));
            input.child_order.clear();
            // Actual roles/order20S<30S<21E<31E prove crossing without a guessed
            // hierarchy. No valid complete generated tree exists for these pairs.
        },
    );
    refusal_preserves(
        &f,
        WrappingScopeError::CrossingScopes,
        Some(WrappingScopeError::CrossingScopes),
        |tree, input| {
            tree.insert_input(31, neutral(InlineBoundaryKind::End));
            tree.insert_children(0, [10, 11, 20, 12, 13, 21, 14, 31, 15]);
            input.scopes.push(pair_input(20, 31));
            input.child_order.clear();
            // Two distinct declarations cannot reuse the SAME actual Start20.
            // Strict disjointness/containment, not equal cut values, proves boxes.
        },
    );
    refusal_preserves(
        &f,
        WrappingScopeError::NodeNotReachable,
        Some(WrappingScopeError::NodeNotReachable),
        |_, input| {
            input.scopes[0] = pair_input(20, 99);
        },
    );
    refusal_preserves(
        &f,
        WrappingScopeError::EndpointRole,
        Some(WrappingScopeError::EndpointRole),
        |tree, _| {
            // An earlier End21 declaration is stale after a real normalized role
            // change. Its old identity/value cannot certify a current End event.
            tree.insert_input(21, neutral(InlineBoundaryKind::Start));
        },
    );
}

fn actual_capture_requires_an_owner<S: LayoutScalar>() {
    let f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    let text = crate::test_support::inline_endpoint::Fixture::new(
        "A",
        (0, 1),
        surgeist_text::WhiteSpace::Preserve,
        false,
        false,
    );
    let scope = text.binding.controls[0]
        .occurrence
        .captured_control()
        .unwrap()
        .scope();
    assert_eq!(scope.role(), surgeist_text::ScopeRole::Inline);
    assert_eq!(
        scope.domain_topology(),
        surgeist_text::SourceProcessingTopology::Complete
    );
    refusal_preserves(
        &f,
        WrappingScopeError::MissingCapture,
        Some(WrappingScopeError::MissingCapture),
        |_, input| {
            // This is a real captured Inline, but the Native formatter contains no
            // actual Prepared binding that owns it. That is MissingCapture, not a
            // counterfeit ForeignCapture comparison or a numeric-source adapter.
            input.scopes.push(WrappingScopeInput::Captured {
                scope,
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
            });
        },
    );
}

#[test]
fn duplicate_and_unreachable_native_formatter_demands_refuse_in_both_scalars() {
    request_identity::<f32>();
    request_identity::<f64>();
}
#[test]
fn native_family_requires_explicit_complete_root_and_nested_closure_in_both_scalars() {
    complete_family_proof::<f32>();
    complete_family_proof::<f64>();
}
#[test]
fn demanded_family_preserves_crossing_shared_endpoint_and_stale_pair_causes_in_both_scalars() {
    malformed_and_stale_pairs::<f32>();
    malformed_and_stale_pairs::<f64>();
}
#[test]
fn real_captured_scope_without_prepared_owner_remains_missing_capture_in_both_scalars() {
    actual_capture_requires_an_owner::<f32>();
    actual_capture_requires_an_owner::<f64>();
}
