//! Authenticated ancestry expands physical empty-gap constraints.
use super::*;

fn ancestry_expanded_empty_gap_cycle<S: LayoutScalar>(front: Front) {
    let mut input = empty_input(false, text::ParagraphStrut::DefaultFont);
    input.computed_line_boundaries.clear();
    assert!(input.source.text().is_empty());
    assert!(input.origins.is_empty());
    assert!(input.controls.is_empty());
    assert_eq!(input.paragraphs.len(), 1);
    let root_scope = input
        .scopes
        .iter()
        .position(|scope| matches!(scope, text::SourceProcessingScope::Block { .. }))
        .unwrap();
    let first_parent = input
        .scopes
        .iter()
        .position(|scope| matches!(scope, text::SourceProcessingScope::Inline { .. }))
        .unwrap();
    assert_eq!(input.scopes.len(), 2);
    assert!(matches!(
        &input.scopes[first_parent],
        text::SourceProcessingScope::Inline {
            boundaries: None,
            ..
        }
    ));
    let inline = || text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: None,
    };
    let mut raw_pairs = Vec::new();
    for pair in 0..3 {
        let parent = if pair == 0 {
            first_parent
        } else {
            let index = input.scopes.len();
            input.scopes.push(inline());
            index
        };
        let child = input.scopes.len();
        input.scopes.push(inline());
        raw_pairs.push((parent, child));
    }
    input.events = vec![E::EnterScope(root_scope), E::EnterParagraph(0)];
    for &(parent, child) in &raw_pairs {
        input.events.extend([
            E::EnterScope(parent),
            E::EnterScope(child),
            E::ExitScope(child),
            E::ExitScope(parent),
        ]);
    }
    input
        .events
        .extend([E::ExitParagraph(0), E::ExitScope(root_scope)]);
    let local = |scope| crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope));
    let mut orders = vec![complete_order(
        crate::GeneratedBoxRef::Node(0),
        raw_pairs
            .iter()
            .map(|&(parent, _)| child(local(parent)))
            .collect(),
    )];
    for &(parent, child_scope) in &raw_pairs {
        orders.push(complete_order(
            local(parent),
            vec![child(local(child_scope))],
        ));
        orders.push(complete_order(local(child_scope), Vec::new()));
    }
    let source = EmptyInlineSource::<S> {
        tree: PublicLayoutTreeOf::new().children(0, []).style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        ),
        scopes: raw_pairs
            .iter()
            .flat_map(|&(parent, child)| [parent, child])
            .map(|scope| crate::SourceWrappingScopeInput {
                box_: crate::InlineBoxRef::Scope(scope),
                wrap_inside: crate::WrapInside::Auto,
                edges: crate::WrapEdges::AUTO,
            })
            .collect(),
        associations: raw_pairs
            .iter()
            .flat_map(|&(parent, child)| [parent, child])
            .map(|scope| crate::SourceScopeAssociation {
                scope,
                flow: 0,
                pair: None,
            })
            .collect(),
        orders,
    };
    let (processed, receipt) = crate::process_wrapping_source(
        &source,
        0,
        input,
        crate::SourceOccurrenceAssociations::default(),
    )
    .expect("the real source authenticates Root -> [P -> C, Q -> D, R -> E]")
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
    let domain = owner.domain();
    let member = owner.members().next().unwrap();
    let scopes = member
        .scope_intersections()
        .map(|intersection| intersection.scope())
        .filter(|scope| scope.role() == text::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 6);
    let mut parents = scopes
        .iter()
        .filter(|scope| scope.parent().as_ref() == Some(&domain))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(parents.len(), 3);
    // Source event order names authored P/Q/R and authenticates child proof
    // order. It is NOT an oracle for physical order at the tied frontier.
    parents.sort_by(|left, right| {
        left.enter_position()
            .partial_cmp(&right.enter_position())
            .unwrap()
    });
    let mut pairs = Vec::new();
    for parent in &parents {
        let children = scopes
            .iter()
            .filter(|scope| scope.parent().as_ref() == Some(parent))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(children.len(), 1);
        let child = children[0].clone();
        assert_eq!(parent.role(), text::ScopeRole::Inline);
        assert_eq!(child.role(), text::ScopeRole::Inline);
        assert_eq!(parent.parent(), Some(domain.clone()));
        assert_eq!(child.parent(), Some(parent.clone()));
        for scope in [parent, &child] {
            assert!(scope.entry_control().is_none());
            assert!(scope.exit_control().is_none());
        }
        pairs.push((parent.clone(), child));
    }
    let (p, c) = &pairs[0];
    let (q, d) = &pairs[1];
    let (r, e) = &pairs[2];
    let actual = |scope: &text::CapturedScope| {
        crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope.clone()))
    };
    let mut orders = vec![complete_order(
        crate::GeneratedBoxRef::Node(0),
        parents.iter().map(|scope| child(actual(scope))).collect(),
    )];
    for (parent, child_scope) in &pairs {
        orders.push(complete_order(
            actual(parent),
            vec![child(actual(child_scope))],
        ));
        orders.push(complete_order(actual(child_scope), Vec::new()));
    }
    let wrapping = crate::WrappingInput {
        scopes: scopes
            .iter()
            .map(|scope| crate::WrappingScopeInput::Captured {
                scope: scope.clone(),
                wrap_inside: crate::WrapInside::Auto,
                edges: crate::WrapEdges::AUTO,
                metric_contribution: None,
            })
            .collect(),
        child_order: orders,
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
        .expect("real six-scope source and complete proofs admit ordinary association-only layout");
    assert!(!baseline.associations.has_prepared_execution(0));
    assert!(baseline.associations.runs().is_empty());
    assert_eq!(baseline.graph.nodes.len(), 1);
    assert!(baseline.graph.children[baseline.graph.index(0).unwrap()].is_empty());
    assert_eq!(crate::Traverse::child_count(&source, 0), 0);
    for scope in &scopes {
        assert!(baseline.wrapping.scope_pair(scope).is_none());
    }
    let prior = compute_layout_with_flow(
        &baseline,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(prior.final_entries().len(), 1);
    assert_eq!(prior.final_entries()[0].node(), 0);
    assert_eq!(prior.final_entries()[0].output().size.height, S::ZERO);
    assert!(prior.placed_text().is_empty());
    assert_eq!(prior.native_inline_box_families().count(), 0);
    let group =
        |left: &text::CapturedScope, right: &text::CapturedScope| crate::InlineGapGroupInput {
            anchor: crate::InlineGapAnchor::Formatter {
                node: 0,
                edge: crate::InlineFormatterEdge::Start,
            },
            boxes: vec![
                crate::InlineBoxRef::Scope(left.clone()),
                crate::InlineBoxRef::Scope(right.clone()),
            ],
        };
    // Direct group owners are all distinct. The cycle exists only after
    // original ancestry expands P/Q/R to their real descendant leaves C/D/E.
    let named = [p, q, d, r, e, c];
    for (index, scope) in named.iter().enumerate() {
        assert!(!named[..index].contains(scope));
    }
    let mut acyclic = wrapping;
    acyclic
        .native_inline_boxes
        .push(crate::NativeInlineBoxFamilyInput { formatter: 0 });
    acyclic.gaps = vec![group(p, q), group(d, r), group(c, e)];
    let valid = construct(acyclic.clone())
        .expect("[P,Q], [D,R], [C,E] gives consistent C<D<E constraints over genuine ancestry");
    assert!(!valid.associations.has_prepared_execution(0));
    assert!(valid.associations.runs().is_empty());
    let valid_batch = compute_layout_with_flow(
        &valid,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(valid_batch.final_entries(), prior.final_entries());
    assert!(valid_batch.placed_text().is_empty());
    assert_eq!(valid_batch.native_inline_box_pieces().count(), 0);
    let families = valid_batch.native_inline_box_families().collect::<Vec<_>>();
    assert_eq!(families.len(), 1);
    let family = &families[0];
    for (parent, child_scope) in &pairs {
        let parents = family
            .fragments()
            .filter(|fragment| fragment.owner() == actual(parent))
            .collect::<Vec<_>>();
        let children = family
            .fragments()
            .filter(|fragment| fragment.owner() == actual(child_scope))
            .collect::<Vec<_>>();
        assert_eq!(parents.len(), 1);
        assert_eq!(children.len(), 1);
        assert_eq!(
            parents[0].parent().unwrap().owner(),
            crate::GeneratedBoxRef::Node(0)
        );
        assert_eq!(children[0].parent(), Some(parents[0].clone()));
        assert_eq!(
            children[0].body_root().owner(),
            crate::GeneratedBoxRef::Node(0)
        );
    }
    let mut cyclic = acyclic;
    cyclic.gaps[2] = group(e, c);
    let Err(error) = construct(cyclic) else {
        panic!("C<D, D<E, E<C must refuse an ancestry-expanded gap cycle");
    };
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::WrappingScope(
                crate::WrappingScopeError::CyclicGap
            ))
        ),
        "{error:?}"
    );
    let replay = compute_layout_with_flow(
        &baseline,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(replay.final_entries(), prior.final_entries());
    assert!(replay.placed_text().is_empty());
    assert_eq!(replay.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(valid_batch.native_inline_box_families().count(), 1);
    for scope in &scopes {
        assert_eq!(
            family
                .fragments()
                .filter(|fragment| fragment.owner() == actual(scope))
                .count(),
            1
        );
    }
}

#[test]
fn native_family_ancestry_expanded_gap_cycle_direct_f32() {
    ancestry_expanded_empty_gap_cycle::<f32>(Front::Direct);
}
#[test]
fn native_family_ancestry_expanded_gap_cycle_direct_f64() {
    ancestry_expanded_empty_gap_cycle::<f64>(Front::Direct);
}
#[test]
fn native_family_ancestry_expanded_gap_cycle_source_receipt_f32() {
    ancestry_expanded_empty_gap_cycle::<f32>(Front::SourceReceipt);
}
#[test]
fn native_family_ancestry_expanded_gap_cycle_source_receipt_f64() {
    ancestry_expanded_empty_gap_cycle::<f64>(Front::SourceReceipt);
}
