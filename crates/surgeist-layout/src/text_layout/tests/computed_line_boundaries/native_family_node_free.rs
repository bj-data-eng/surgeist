//! Unscheduled captured empty Inline scopes retain authentic native family identity.
use super::*;

struct EmptyInlineSource<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    scopes: Vec<crate::SourceWrappingScopeInput<u32>>,
    associations: Vec<crate::SourceScopeAssociation<u32>>,
    orders: Vec<crate::InlineChildOrderInput<u32, usize>>,
}
impl<S: LayoutScalar> crate::Traverse for EmptyInlineSource<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as crate::Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        crate::Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        crate::Traverse::child_count(&self.tree, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        crate::Traverse::child(&self.tree, node, index)
    }
}
impl<S: LayoutScalar> crate::LayoutTree for EmptyInlineSource<S> {
    type MeasureError = <PublicLayoutTreeOf<S> as crate::LayoutTree>::MeasureError;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        crate::LayoutTree::node_input(&self.tree, node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        crate::LayoutTree::layout_input(&self.tree, node)
    }
}
impl<S: LayoutScalar> crate::WrapEdgeSource for EmptyInlineSource<S> {
    fn wrap_edge_source_node(&self, node: u32) -> crate::WrapEdgeSourceNodeView<'_, S> {
        match crate::LayoutTree::layout_input(&self.tree, node) {
            LayoutInputViewOf::Box(input) => crate::WrapEdgeSourceNodeView::Box(input),
            _ => panic!("this authentic empty native tree has only its Block root"),
        }
    }
    fn source_wrapping_scopes(&self) -> &[crate::SourceWrappingScopeInput<u32>] {
        &self.scopes
    }
    fn source_scope_associations(&self) -> &[crate::SourceScopeAssociation<u32>] {
        &self.associations
    }
    fn generated_child_order(&self) -> &[crate::InlineChildOrderInput<u32, usize>] {
        &self.orders
    }
}
fn complete_order<Scope>(
    parent: crate::GeneratedBoxRef<u32, Scope>,
    children: Vec<crate::GeneratedChildBoxInput<u32, Scope>>,
) -> crate::InlineChildOrderInput<u32, Scope> {
    crate::InlineChildOrderInput {
        parent,
        children,
        prefix: crate::ChildOrderCompleteness::Complete,
        suffix: crate::ChildOrderCompleteness::Complete,
        formatting_context: 0,
    }
}
fn child<Scope>(
    box_: crate::GeneratedBoxRef<u32, Scope>,
) -> crate::GeneratedChildBoxInput<u32, Scope> {
    crate::GeneratedChildBoxInput {
        box_,
        participation: crate::GeneratedBoxParticipation::InFlow,
    }
}
#[derive(Clone, Copy)]
enum Front {
    Direct,
    SourceReceipt,
}

#[derive(Clone, Copy)]
enum Attachment {
    ExplicitStart,
    EmptyParentRoot,
    UniqueEmptyRaw,
}

fn node_free_captured_inline_family<S: LayoutScalar>(front: Front, attachment: Attachment) {
    // Author a real undecorated empty Inline, without a computed force. This is
    // a pre-processing input choice, not deletion of an issued participant.
    let mut input = empty_input(false, text::ParagraphStrut::DefaultFont);
    input.computed_line_boundaries.clear();
    assert!(input.source.text().is_empty());
    assert!(input.origins.is_empty());
    assert!(input.controls.is_empty());
    assert!(matches!(
        &input.scopes[1],
        text::SourceProcessingScope::Inline {
            boundaries: None,
            ..
        }
    ));
    let local_box = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(1usize));
    let source = EmptyInlineSource::<S> {
        tree: PublicLayoutTreeOf::new().children(0, []).style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        ),
        scopes: vec![crate::SourceWrappingScopeInput {
            box_: crate::InlineBoxRef::Scope(1),
            wrap_inside: crate::WrapInside::Auto,
            edges: crate::WrapEdges::AUTO,
        }],
        associations: vec![crate::SourceScopeAssociation {
            scope: 1,
            flow: 0,
            pair: None,
        }],
        orders: vec![
            complete_order(
                crate::GeneratedBoxRef::Node(0),
                vec![child(local_box.clone())],
            ),
            complete_order(local_box, Vec::new()),
        ],
    };
    let (processed, receipt) = crate::process_wrapping_source(
        &source,
        0,
        input,
        crate::SourceOccurrenceAssociations::default(),
    )
    .expect("real source projection authenticates the complete empty Inline child proof")
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
    let scope = f.processed.captured_scope(1).unwrap();
    assert_eq!(scope.role(), text::ScopeRole::Inline);
    assert_eq!(scope.parent(), Some(f.processed.captured_scope(0).unwrap()));
    assert!(scope.entry_control().is_none());
    assert!(scope.exit_control().is_none());
    assert_eq!(crate::Traverse::child_count(&source, 0), 0);
    let box_ = crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope.clone()));
    let wrapping = crate::WrappingInput {
        scopes: vec![crate::WrappingScopeInput::Captured {
            scope: scope.clone(),
            wrap_inside: crate::WrapInside::Auto,
            edges: crate::WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![
            complete_order(crate::GeneratedBoxRef::Node(0), vec![child(box_.clone())]),
            complete_order(box_, Vec::new()),
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
    let baseline = construct(wrapping.clone()).expect("the real AssociationOnly captured scope and these exact complete proofs admit without native family demand");
    assert!(!baseline.associations.has_prepared_execution(0));
    assert!(baseline.associations.runs().is_empty());
    assert!(baseline.wrapping.scope_pair(&scope).is_none());
    let root_index = baseline.graph.index(0).unwrap();
    assert_eq!(baseline.graph.nodes.len(), 1);
    assert!(
        baseline.graph.children[root_index].is_empty(),
        "unique raw frontier requires zero actual slots, not merely no supplied operands"
    );
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

    // This actual formatter has zero raw children/slots, hence one body and
    // one frontier0..0. Metadata or unclassified controls would invalidate the
    // zero-raw-slots premise even if they supply no ordering operand.
    let mut demand = wrapping;
    demand
        .native_inline_boxes
        .push(crate::NativeInlineBoxFamilyInput { formatter: 0 });
    let reason = match attachment {
        Attachment::ExplicitStart => {
            demand.gaps.push(crate::InlineGapGroupInput {
                anchor: crate::InlineGapAnchor::Formatter {
                    node: 0,
                    edge: crate::InlineFormatterEdge::Start,
                },
                boxes: vec![crate::InlineBoxRef::Scope(scope.clone())],
            });
            "an actual unscheduled node-free captured Inline with an explicit authenticated gap must retain native family identity"
        }
        Attachment::EmptyParentRoot => {
            demand.gaps.push(crate::InlineGapGroupInput {
                anchor: crate::InlineGapAnchor::EmptyParent(crate::GeneratedBoxRef::Node(0)),
                boxes: vec![crate::InlineBoxRef::Scope(scope.clone())],
            });
            "the actual ordering-empty formatter parent must authenticate its real node-free captured Inline attachment"
        }
        Attachment::UniqueEmptyRaw => {
            assert!(demand.gaps.is_empty());
            "an actual unscheduled node-free captured Inline at the unique empty raw frontier must retain native family identity without an explicit gap"
        }
    };
    let demanded = construct(demand).expect(reason);
    assert!(!demanded.associations.has_prepared_execution(0));
    assert!(demanded.associations.runs().is_empty());
    assert!(demanded.wrapping.scope_pair(&scope).is_none());
    let batch = compute_layout_with_flow(
        &demanded,
        0,
        request_at(AvailableOf::definite(S::from_f64(40.0))),
    )
    .unwrap();
    assert_eq!(batch.final_entries(), prior.final_entries());
    assert!(batch.placed_text().is_empty());
    assert_eq!(
        batch.native_inline_box_pieces().count(),
        0,
        "no line/run may be fabricated for association metadata"
    );
    let families = batch.native_inline_box_families().collect::<Vec<_>>();
    assert_eq!(families.len(), 1);
    let family = &families[0];
    assert_eq!(family.formatter(), 0);
    let owners = family
        .fragments()
        .filter(|fragment| {
            fragment.owner()
                == crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope.clone()))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        owners.len(),
        1,
        "retain the actual empty captured Inline, not merely the formatter root"
    );
    assert_eq!(owners[0].family(), family);
    assert_eq!(
        owners[0].parent().unwrap().owner(),
        crate::GeneratedBoxRef::Node(0)
    );
    assert_eq!(
        owners[0].body_root().owner(),
        crate::GeneratedBoxRef::Node(0)
    );
    assert_eq!(prior.native_inline_box_families().count(), 0);
}

#[test]
fn native_family_keeps_unscheduled_node_free_captured_inline_direct_f32() {
    node_free_captured_inline_family::<f32>(Front::Direct, Attachment::ExplicitStart);
}
#[test]
fn native_family_keeps_unscheduled_node_free_captured_inline_direct_f64() {
    node_free_captured_inline_family::<f64>(Front::Direct, Attachment::ExplicitStart);
}
#[test]
fn native_family_keeps_unscheduled_node_free_captured_inline_source_receipt_f32() {
    node_free_captured_inline_family::<f32>(Front::SourceReceipt, Attachment::ExplicitStart);
}
#[test]
fn native_family_keeps_unscheduled_node_free_captured_inline_source_receipt_f64() {
    node_free_captured_inline_family::<f64>(Front::SourceReceipt, Attachment::ExplicitStart);
}

#[test]
fn native_family_derives_unique_empty_raw_frontier_direct_f32() {
    node_free_captured_inline_family::<f32>(Front::Direct, Attachment::UniqueEmptyRaw);
}
#[test]
fn native_family_derives_unique_empty_raw_frontier_direct_f64() {
    node_free_captured_inline_family::<f64>(Front::Direct, Attachment::UniqueEmptyRaw);
}
#[test]
fn native_family_derives_unique_empty_raw_frontier_source_receipt_f32() {
    node_free_captured_inline_family::<f32>(Front::SourceReceipt, Attachment::UniqueEmptyRaw);
}
#[test]
fn native_family_derives_unique_empty_raw_frontier_source_receipt_f64() {
    node_free_captured_inline_family::<f64>(Front::SourceReceipt, Attachment::UniqueEmptyRaw);
}

#[test]
fn native_family_attaches_to_empty_formatter_parent_direct_f32() {
    node_free_captured_inline_family::<f32>(Front::Direct, Attachment::EmptyParentRoot);
}
#[test]
fn native_family_attaches_to_empty_formatter_parent_direct_f64() {
    node_free_captured_inline_family::<f64>(Front::Direct, Attachment::EmptyParentRoot);
}
#[test]
fn native_family_attaches_to_empty_formatter_parent_source_receipt_f32() {
    node_free_captured_inline_family::<f32>(Front::SourceReceipt, Attachment::EmptyParentRoot);
}
#[test]
fn native_family_attaches_to_empty_formatter_parent_source_receipt_f64() {
    node_free_captured_inline_family::<f64>(Front::SourceReceipt, Attachment::EmptyParentRoot);
}

#[path = "native_family_node_free/nested_empty_parent.rs"]
mod nested_empty_parent;
