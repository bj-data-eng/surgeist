//! Original parent custody for captured node-free empty scopes.
use super::*;

#[derive(Clone, Copy)]
enum NestedNomination {
    DirectParent,
    SkippedParent,
}

fn nested_node_free_empty_parent<S: LayoutScalar>(front: Front, nomination: NestedNomination) {
    let mut input = empty_input(false, text::ParagraphStrut::DefaultFont);
    input.computed_line_boundaries.clear();
    assert!(input.source.text().is_empty());
    assert!(input.origins.is_empty());
    assert!(input.controls.is_empty());
    assert_eq!(input.paragraphs.len(), 1);
    assert_eq!(
        input
            .scopes
            .iter()
            .filter(|scope| matches!(scope, text::SourceProcessingScope::Block { .. }))
            .count(),
        1
    );
    assert_eq!(
        input
            .scopes
            .iter()
            .filter(|scope| matches!(scope, text::SourceProcessingScope::Inline { .. }))
            .count(),
        1
    );
    let root_scope = input
        .scopes
        .iter()
        .position(|scope| matches!(scope, text::SourceProcessingScope::Block { .. }))
        .unwrap();
    let parent_scope = input
        .scopes
        .iter()
        .position(|scope| matches!(scope, text::SourceProcessingScope::Inline { .. }))
        .unwrap();
    assert!(matches!(
        &input.scopes[parent_scope],
        text::SourceProcessingScope::Inline {
            boundaries: None,
            ..
        }
    ));
    let child_scope = input.scopes.len();
    input.scopes.push(text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: None,
    });
    // These are real nested source occurrences BEFORE processing, not synthetic
    // CapturedScopes or a post-capture change to their parent relationships.
    input.events = vec![
        E::EnterScope(root_scope),
        E::EnterParagraph(0),
        E::EnterScope(parent_scope),
        E::EnterScope(child_scope),
        E::ExitScope(child_scope),
        E::ExitScope(parent_scope),
        E::ExitParagraph(0),
        E::ExitScope(root_scope),
    ];
    let local_parent = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(parent_scope));
    let local_child = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(child_scope));
    let source = EmptyInlineSource::<S> {
        tree: PublicLayoutTreeOf::new().children(0, []).style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        ),
        scopes: [parent_scope, child_scope]
            .into_iter()
            .map(|scope| crate::SourceWrappingScopeInput {
                box_: crate::InlineBoxRef::Scope(scope),
                wrap_inside: crate::WrapInside::Auto,
                edges: crate::WrapEdges::AUTO,
            })
            .collect(),
        associations: [parent_scope, child_scope]
            .into_iter()
            .map(|scope| crate::SourceScopeAssociation {
                scope,
                flow: 0,
                pair: None,
            })
            .collect(),
        orders: vec![
            complete_order(
                crate::GeneratedBoxRef::Node(0),
                vec![child(local_parent.clone())],
            ),
            complete_order(local_parent, vec![child(local_child.clone())]),
            complete_order(local_child, Vec::new()),
        ],
    };
    let (processed, receipt) = crate::process_wrapping_source(
        &source,
        0,
        input,
        crate::SourceOccurrenceAssociations::default(),
    )
    .expect("the genuine nested source authenticates Root -> P -> C complete proofs")
    .into_parts();
    let f = prepare_captured(Arc::new(processed), Vec::new(), Vec::new());
    assert!(f.binding.nodes.is_empty());
    assert!(f.binding.origins.is_empty());
    assert!(f.binding.controls.is_empty());
    assert!(f.binding.atomics.is_empty());
    assert!(f.binding.line_strut.is_none());
    assert_eq!(f.binding.prepared.origin_views().count(), 0);
    assert_eq!(f.binding.prepared.control_views().count(), 0);
    assert_eq!(f.binding.prepared.atomic_views().count(), 0);
    assert_eq!(f.binding.prepared.participants().count(), 0);
    assert_eq!(f.binding.prepared.scopes().count(), 0);
    assert_eq!(f.binding.prepared.virtual_separators().count(), 0);
    assert_eq!(
        crate::text_layout::capture::classify(&f.binding),
        Ok(crate::text_layout::capture::ParagraphClass::AssociationOnly)
    );
    let owner = f.binding.prepared.captured_owner().unwrap();
    assert_eq!(owner.members().len(), 1);
    let member = owner.members().next().unwrap();
    let domain = owner.domain();
    let scopes = member
        .scope_intersections()
        .map(|intersection| intersection.scope())
        .filter(|scope| scope.role() == text::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 2);
    // Derive issued P/C by their real parents, not assumed captured ordinals.
    let parent = scopes
        .iter()
        .find(|scope| scope.parent().as_ref() == Some(&domain))
        .unwrap()
        .clone();
    let child_scope = scopes
        .iter()
        .find(|scope| scope.parent().as_ref() == Some(&parent))
        .unwrap()
        .clone();
    assert_ne!(parent, child_scope);
    assert_eq!(parent.role(), text::ScopeRole::Inline);
    assert_eq!(child_scope.role(), text::ScopeRole::Inline);
    assert_eq!(parent.parent(), Some(domain.clone()));
    assert_eq!(child_scope.parent(), Some(parent.clone()));
    for scope in [&parent, &child_scope] {
        assert!(scope.entry_control().is_none());
        assert!(scope.exit_control().is_none());
    }
    let parent_ref = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(parent.clone()));
    let child_ref = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(child_scope.clone()));
    let wrapping = crate::WrappingInput {
        scopes: [&parent, &child_scope]
            .into_iter()
            .map(|scope| crate::WrappingScopeInput::Captured {
                scope: scope.clone(),
                wrap_inside: crate::WrapInside::Auto,
                edges: crate::WrapEdges::AUTO,
                metric_contribution: None,
            })
            .collect(),
        child_order: vec![
            complete_order(
                crate::GeneratedBoxRef::Node(0),
                vec![child(parent_ref.clone())],
            ),
            complete_order(parent_ref.clone(), vec![child(child_ref.clone())]),
            complete_order(child_ref.clone(), Vec::new()),
        ],
        gaps: Vec::new(),
        native_inline_boxes: Vec::new(),
    };
    let construct = |wrapping| match front {
        Front::Direct => {
            LayoutFlowInput::try_new(&source, 0, vec![f.binding.clone()], wrapping, Vec::new())
        }
        Front::SourceReceipt => LayoutFlowInput::try_new_with_edges(
            &source,
            0,
            vec![f.binding.clone()],
            wrapping,
            Vec::new(),
            &receipt,
        ),
    };
    let baseline = construct(wrapping.clone())
        .expect("the exact genuine nested complete proofs admit without native family demand");
    assert!(!baseline.associations.has_prepared_execution(0));
    assert!(baseline.associations.runs().is_empty());
    assert!(baseline.wrapping.scope_pair(&parent).is_none());
    assert!(baseline.wrapping.scope_pair(&child_scope).is_none());
    let root_index = baseline.graph.index(0).unwrap();
    assert_eq!(baseline.graph.nodes.len(), 1);
    assert!(baseline.graph.children[root_index].is_empty());
    assert_eq!(crate::Traverse::child_count(&source, 0), 0);
    let prior = compute_layout_with_flow(
        &baseline,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert!(prior.placed_text().is_empty());
    assert_eq!(prior.final_entries().len(), 1);
    assert_eq!(prior.final_entries()[0].node(), 0);
    assert_eq!(prior.final_entries()[0].output().size.height, S::ZERO);
    assert_eq!(prior.native_inline_box_families().count(), 0);

    let mut demand = wrapping;
    demand
        .native_inline_boxes
        .push(crate::NativeInlineBoxFamilyInput { formatter: 0 });
    let anchor_parent = match nomination {
        NestedNomination::DirectParent => parent_ref.clone(),
        NestedNomination::SkippedParent => crate::GeneratedBoxRef::Node(0),
    };
    demand.gaps.push(crate::InlineGapGroupInput {
        anchor: crate::InlineGapAnchor::EmptyParent(anchor_parent),
        boxes: vec![crate::InlineBoxRef::Scope(child_scope.clone())],
    });
    match nomination {
        NestedNomination::DirectParent => {
            let demanded = construct(demand).expect(
                "the actual immediate ordering-empty captured parent P authenticates child C",
            );
            assert!(!demanded.associations.has_prepared_execution(0));
            assert!(demanded.associations.runs().is_empty());
            let batch = compute_layout_with_flow(
                &demanded,
                0,
                request_at(AvailableOf::definite(S::from_f64(40.0))),
            )
            .unwrap();
            assert_eq!(batch.final_entries(), prior.final_entries());
            assert!(batch.placed_text().is_empty());
            assert_eq!(batch.native_inline_box_pieces().count(), 0);
            let families = batch.native_inline_box_families().collect::<Vec<_>>();
            assert_eq!(families.len(), 1);
            let family = &families[0];
            assert_eq!(family.formatter(), 0);
            let parent_fragments = family
                .fragments()
                .filter(|fragment| fragment.owner() == parent_ref)
                .collect::<Vec<_>>();
            let child_fragments = family
                .fragments()
                .filter(|fragment| fragment.owner() == child_ref)
                .collect::<Vec<_>>();
            assert_eq!(parent_fragments.len(), 1);
            assert_eq!(child_fragments.len(), 1);
            let p = &parent_fragments[0];
            let c = &child_fragments[0];
            assert_eq!(p.family(), family);
            assert_eq!(c.family(), family);
            assert_eq!(p.parent().unwrap().owner(), crate::GeneratedBoxRef::Node(0));
            assert_eq!(c.parent(), Some(p.clone()));
            assert_eq!(c.nearest_common_ancestor(p).unwrap(), *p);
            assert_eq!(p.body_root(), c.body_root());
            assert_eq!(c.body_root().owner(), crate::GeneratedBoxRef::Node(0));
        }
        NestedNomination::SkippedParent => {
            let Err(error) = construct(demand) else {
                panic!(
                    "an ancestor at the same numeric gap must not replace the actual direct parent P"
                );
            };
            assert!(
                matches!(
                    error.kind(),
                    LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                        crate::WrappingScopeError::InvalidGapAnchor
                    ))
                ),
                "{error:?}"
            );
        }
    }
    // A refused demand cannot mutate the previous ordinary publication or its
    // authentic immutable admission. Reexecute that same baseline after it.
    let replay = compute_layout_with_flow(
        &baseline,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(replay.final_entries(), prior.final_entries());
    assert!(prior.placed_text().is_empty());
    assert!(replay.placed_text().is_empty());
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(replay.native_inline_box_families().count(), 0);
}

#[test]
fn native_family_nested_empty_parent_direct_f32() {
    nested_node_free_empty_parent::<f32>(Front::Direct, NestedNomination::DirectParent);
}
#[test]
fn native_family_nested_empty_parent_direct_f64() {
    nested_node_free_empty_parent::<f64>(Front::Direct, NestedNomination::DirectParent);
}
#[test]
fn native_family_nested_empty_parent_source_receipt_f32() {
    nested_node_free_empty_parent::<f32>(Front::SourceReceipt, NestedNomination::DirectParent);
}
#[test]
fn native_family_nested_empty_parent_source_receipt_f64() {
    nested_node_free_empty_parent::<f64>(Front::SourceReceipt, NestedNomination::DirectParent);
}
#[test]
fn native_family_skipped_empty_parent_refuses_direct_f32() {
    nested_node_free_empty_parent::<f32>(Front::Direct, NestedNomination::SkippedParent);
}
#[test]
fn native_family_skipped_empty_parent_refuses_direct_f64() {
    nested_node_free_empty_parent::<f64>(Front::Direct, NestedNomination::SkippedParent);
}
#[test]
fn native_family_skipped_empty_parent_refuses_source_receipt_f32() {
    nested_node_free_empty_parent::<f32>(Front::SourceReceipt, NestedNomination::SkippedParent);
}
#[test]
fn native_family_skipped_empty_parent_refuses_source_receipt_f64() {
    nested_node_free_empty_parent::<f64>(Front::SourceReceipt, NestedNomination::SkippedParent);
}
