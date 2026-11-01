//! Explicit family demand is an admission/custody capability, not geometry inference.
use super::*;
use crate::{InlineBoxFragmentationDemand as Demand, LayoutTree, Traverse};

fn wrapping() -> crate::WrappingInput<u32> {
    crate::WrappingInput {
        gaps: Vec::new(),
        scopes: Vec::new(),
        child_order: Vec::new(),
    }
}

/// An actual single native Block with no children, used in both early source
/// processing and final checked admission. No prepared/native role is invented.
struct EmptyTree(NodeInputOf<f64>);
impl Traverse for EmptyTree {
    type Node = u32;
    type Scalar = f64;
    type Children<'a>
        = std::iter::Empty<u32>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        assert_eq!(node, 0);
        std::iter::empty()
    }
    fn child_count(&self, node: u32) -> usize {
        assert_eq!(node, 0);
        0
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        panic!("empty Block {node} has no child {index}")
    }
}
impl LayoutTree for EmptyTree {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<f64> {
        assert_eq!(node, 0);
        &self.0
    }
    fn layout_input(&self, node: u32) -> crate::LayoutInputViewOf<'_, f64> {
        crate::LayoutInputViewOf::Box(self.node_input(node))
    }
}
impl crate::WrapEdgeSource for EmptyTree {
    fn wrap_edge_source_node(&self, node: u32) -> crate::WrapEdgeSourceNodeView<'_, f64> {
        crate::WrapEdgeSourceNodeView::Box(self.node_input(node))
    }
    fn source_wrapping_scopes(&self) -> &[crate::SourceWrappingScopeInput<u32>] {
        &[]
    }
    fn source_scope_associations(&self) -> &[crate::SourceScopeAssociation<u32>] {
        &[]
    }
    fn generated_child_order(&self) -> &[crate::InlineChildOrderInput<u32, usize>] {
        &[]
    }
}

#[test]
fn both_checked_constructors_refuse_complete_bodies_for_association_only() {
    let tree = EmptyTree(NodeInputOf {
        display: Display::Block,
        ..NodeInputOf::default()
    });
    let mut input = base_input(false, text::ParagraphStrut::DefaultFont);
    input.source = Source::identified("", text::Id::from_u64(650_759), 12);
    input.origins.clear();
    input.connections.clear();
    input.boundary_policies.clear();
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (processed, receipt) = crate::process_wrapping_source(
        &tree,
        0,
        input,
        crate::SourceOccurrenceAssociations::default(),
    )
    .unwrap()
    .into_parts();
    let fixture = prepare_captured(Arc::new(processed), Vec::new(), Vec::new());
    assert_eq!(
        crate::text_layout::capture::classify(&fixture.binding).unwrap(),
        crate::text_layout::capture::ParagraphClass::AssociationOnly
    );
    LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        wrapping(),
        Vec::new(),
    )
    .unwrap();
    LayoutFlowInput::try_new_with_edges(
        &tree,
        0,
        vec![fixture.binding.clone()],
        wrapping(),
        Vec::new(),
        &receipt,
    )
    .unwrap();
    let mut binding = fixture.binding;
    binding.inline_box_fragments = Demand::CompleteBodies;
    for with_receipt in [false, true] {
        let result = if with_receipt {
            LayoutFlowInput::try_new_with_edges(
                &tree,
                0,
                vec![binding.clone()],
                wrapping(),
                Vec::new(),
                &receipt,
            )
        } else {
            LayoutFlowInput::try_new(&tree, 0, vec![binding.clone()], wrapping(), Vec::new())
        };
        let Err(error) = result else {
            panic!("association-only demand must refuse")
        };
        assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), crate::LayoutOperation::TextCapture);
        assert!(
            matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
                    TextBindingError::InlineBoxFragmentsRequireFlow
                ))
            ),
            "{error:?}"
        );
    }
}

#[test]
fn complete_bodies_flow_retains_one_binding_family_and_existing_source_geometry() {
    let mut fixture = prepare(scope_input(false, Side::Before, true));
    fixture.binding.inline_box_fragments = Demand::CompleteBodies;
    let batch = compute::<f64>(&fixture, false, AvailableOf::definite(100.0));
    assert_eq!(batch.placed_text().len(), 1);
    let output = &batch.placed_text()[0];
    let fragments = output.inline_box_fragments().unwrap();
    assert_eq!(fragments.family().prepared_text(), fixture.binding.prepared);
    assert!(fragments.pieces().next().is_some());
    for piece in fragments.pieces() {
        assert_eq!(piece.family(), fragments.family());
    }
    assert_eq!(
        output
            .layout()
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [text::Range::new(0, 1), text::Range::new(1, 2)]
    );
    assert_originals(&batch, &fixture);
    let mut unrequested = prepare(scope_input(false, Side::Before, true));
    unrequested.binding.inline_box_fragments = Demand::NotRequested;
    let ordinary = compute::<f64>(&unrequested, false, AvailableOf::definite(100.0));
    assert!(ordinary.placed_text()[0].inline_box_fragments().is_none());
}

#[test]
fn complete_bodies_accepts_authentic_node_free_semantic_flow() {
    let mut fixture = prepare(empty_input(false, text::ParagraphStrut::DefaultFont));
    assert_eq!(
        crate::text_layout::capture::classify(&fixture.binding).unwrap(),
        crate::text_layout::capture::ParagraphClass::SemanticFlow
    );
    fixture.binding.inline_box_fragments = Demand::CompleteBodies;
    let tree = tree::<f64>(&fixture, false);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        wrapping(),
        Vec::new(),
    )
    .unwrap();
    let family = flow.associations.inline_boxes(0);
    assert_eq!(
        family.family().unwrap().prepared_text(),
        fixture.binding.prepared
    );
    assert!(flow.paragraphs()[0].nodes.is_empty());
}

#[test]
fn unknown_topology_refuses_precisely_without_replacing_prior_output_and_recovers() {
    let mut complete = prepare(scope_input(false, Side::Before, true));
    complete.binding.inline_box_fragments = Demand::CompleteBodies;
    let prior = compute::<f64>(&complete, false, AvailableOf::definite(100.0));
    let before = prior.placed_text()[0]
        .layout()
        .lines()
        .iter()
        .map(|line| (line.range(), line.bounds(), line.baseline()))
        .collect::<Vec<_>>();
    let mut input = scope_input(false, Side::Before, true);
    let text::SourceProcessingScope::Block { topology, .. } = &mut input.scopes[0] else {
        unreachable!("real domain")
    };
    *topology = text::SourceProcessingTopology::Unknown;
    let mut unknown = prepare(input);
    let ordinary_tree = tree::<f64>(&unknown, false);
    LayoutFlowInput::try_new(
        &ordinary_tree,
        0,
        vec![unknown.binding.clone()],
        wrapping(),
        Vec::new(),
    )
    .unwrap();
    unknown.binding.inline_box_fragments = Demand::CompleteBodies;
    let Err(error) = LayoutFlowInput::try_new(
        &ordinary_tree,
        0,
        vec![unknown.binding],
        wrapping(),
        Vec::new(),
    ) else {
        panic!("Unknown topology cannot establish complete fragments")
    };
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), crate::LayoutOperation::TextCapture);
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::Text(error) if matches!(
            error.detail(), Some(text::ErrorDetail::InvalidTextProtocol {
                reason: text::ProtocolError::InlineBoxFragmentation(
                    text::InlineBoxFragmentationError::IncompleteTopology { domain: 0 }
                )
            })
        )),
        "{error:?}"
    );
    assert_eq!(
        prior.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| (line.range(), line.bounds(), line.baseline()))
            .collect::<Vec<_>>(),
        before
    );
    let recovered = compute::<f64>(&complete, false, AvailableOf::definite(100.0));
    assert_eq!(
        recovered.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .map(|line| (line.range(), line.bounds(), line.baseline()))
            .collect::<Vec<_>>(),
        before
    );
    assert_originals(&prior, &complete);
    assert_originals(&recovered, &complete);
}
