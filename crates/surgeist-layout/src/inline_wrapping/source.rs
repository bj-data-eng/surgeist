//! Early source-to-native wrapping association. Text owns the sole processing pass.

use super::{
    canonical_order::{CanonicalBox, CanonicalCoverage, CanonicalOrderInventory},
    context::native_span_with,
    edge_plan::{BoxFact, EdgeLocation, Propagation, node_wrapping, normalize_orders, propagate},
    *,
};
use crate::{
    Display, Float, InlineBoundaryInputOf, InlineBoundaryKind, LayoutResultOf, LayoutTree,
    LineBreakInputOf, NodeInputOf, Position, Traverse,
    engine::validation::{ValidationGraph, discover_graph},
};
use surgeist_text::{
    CapturedComputedLineBoundary, CapturedComputedLineBoundaryTarget, CapturedControl,
    CapturedEventPosition, CapturedScope, ComputedLineBoundaryInput, ContextualSourceInput,
    ParagraphControlKind, ProcessedSource, SourceProcessingScope,
};

/// A borrow of the actual host inventory before Text preparation exists.
pub enum WrapEdgeSourceNodeView<'a, S: crate::LayoutScalar> {
    Box(&'a NodeInputOf<S>),
    Boundary(InlineBoundaryInputOf<S>),
    LineBreak(LineBreakInputOf<S>),
    SourceOrigin { origin: usize },
    SourceControl { control: usize },
}

/// Input-local scope identities belong only to the consumed contextual input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceWrappingScopeInput<Node> {
    pub box_: InlineBoxRef<Node, usize>,
    pub wrap_inside: WrapInside,
    pub edges: WrapEdges,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceScopeAssociation<Node> {
    pub scope: usize,
    pub flow: Node,
    pub pair: Option<(Node, Node)>,
}

/// Actual occurrence-to-host bindings within one input, before issued handles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceOccurrenceAssociations<Node> {
    pub origins: Vec<(usize, Node)>,
    pub controls: Vec<(usize, Node)>,
    pub atomics: Vec<(usize, Node)>,
}
impl<Node> Default for SourceOccurrenceAssociations<Node> {
    fn default() -> Self {
        Self {
            origins: Vec::new(),
            controls: Vec::new(),
            atomics: Vec::new(),
        }
    }
}

/// A fresh view of the same real traversal, independent of prepared roles.
pub trait WrapEdgeSource: Traverse {
    fn wrap_edge_source_node(&self, node: Self::Node) -> WrapEdgeSourceNodeView<'_, Self::Scalar>;
    fn source_wrapping_scopes(&self) -> &[SourceWrappingScopeInput<Self::Node>];
    fn source_scope_associations(&self) -> &[SourceScopeAssociation<Self::Node>];
    fn generated_child_order(&self) -> &[InlineChildOrderInput<Self::Node, usize>];
}

/// A processed source and the resolved native semantic receipt issued for it.
pub struct ProcessedWrappingSource<Node> {
    processed: ProcessedSource,
    receipt: ResolvedWrapEdges<Node>,
}
impl<Node> ProcessedWrappingSource<Node> {
    #[must_use]
    pub fn into_parts(self) -> (ProcessedSource, ResolvedWrapEdges<Node>) {
        (self.processed, self.receipt)
    }
}

/// An owned receipt, not authority to reuse a changed native projection.
/// Final admission revalidates its semantics against the actual current tree.
pub struct ResolvedWrapEdges<Node> {
    snapshot: SourceSnapshot<Node>,
    inventory: SourceInventory,
    occurrences: SourceOccurrenceAssociations<Node>,
    scopes: Vec<CapturedScope>,
    origins: Vec<CapturedEventPosition>,
    controls: Vec<CapturedControl>,
    atomics: Vec<CapturedEventPosition>,
    causes: Vec<CapturedComputedLineBoundary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeSemantics {
    Box {
        display: Display,
        position: Position,
        float: Float,
        edges: WrapEdges,
        flex_wrap: crate::FlexWrap,
        order: crate::ItemOrder,
        direction: crate::Direction,
        writing_mode: crate::WritingMode,
    },
    Boundary(InlineBoundaryKind, crate::Direction, crate::WritingMode),
    LineBreak(
        crate::LineBreakDisplay,
        crate::Direction,
        crate::WritingMode,
    ),
    Origin(usize),
    Control(usize),
}
#[derive(Clone, Debug, PartialEq)]
struct SourceSnapshot<Node> {
    nodes: Vec<(Node, NodeSemantics)>,
    flows: Vec<(Node, Vec<Node>)>,
    parent_paths: Vec<(Node, Vec<Node>)>,
    declarations: Vec<SourceWrappingScopeInput<Node>>,
    associations: Vec<SourceScopeAssociation<Node>>,
    orders: Vec<InlineChildOrderInput<Node, usize>>,
    locations: Vec<EdgeLocation<Node, usize>>,
}
struct SourceInventory {
    scopes: Vec<Option<surgeist_text::SourceProcessingScopeBoundaries>>,
    inline: Vec<bool>,
    controls: Vec<ParagraphControlKind>,
    origins: usize,
    atomics: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CapturedBoxIndex {
    Scope(usize),
    Origin(usize),
    Atomic(usize),
}

fn error<Node, S: crate::LayoutScalar, M>(
    node: Node,
    reason: WrappingScopeError,
) -> crate::LayoutErrorOf<Node, S, M> {
    super::wrapping_error(node, reason)
}

/// Resolves native wrapping before the one Text processing operation.
/// Used box metrics are supplied later through the final `WrappingInput`.
pub fn process_wrapping_source<Source: WrapEdgeSource>(
    source: &Source,
    root: Source::Node,
    mut input: ContextualSourceInput,
    occurrences: SourceOccurrenceAssociations<Source::Node>,
) -> LayoutResultOf<Source::Node, ProcessedWrappingSource<Source::Node>, Source::Scalar> {
    let inventory = SourceInventory {
        scopes: input
            .scopes
            .iter()
            .map(|scope| match scope {
                SourceProcessingScope::Inline { boundaries, .. } => *boundaries,
                _ => None,
            })
            .collect(),
        inline: input
            .scopes
            .iter()
            .map(|scope| matches!(scope, SourceProcessingScope::Inline { .. }))
            .collect(),
        controls: input
            .controls
            .iter()
            .map(|control| control.kind.clone())
            .collect(),
        origins: input.origins.len(),
        atomics: input.source.boxes().len(),
    };
    let graph = discover_graph(source, root)?;
    let snapshot =
        project(source, &graph, &inventory, &occurrences).map_err(|reason| error(root, reason))?;
    for location in &snapshot.locations {
        if !location.effect.forced {
            continue;
        }
        let request = match &location.box_ {
            GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)) => {
                ComputedLineBoundaryInput::Scope {
                    scope: *scope,
                    side: location.side,
                }
            }
            GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }) => {
                let node = if location.side == surgeist_text::SourceBoxSide::Before {
                    start
                } else {
                    end
                };
                let control = occurrences
                    .controls
                    .iter()
                    .find(|(_, actual)| actual == node)
                    .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?
                    .0;
                ComputedLineBoundaryInput::Endpoint { control }
            }
            GeneratedBoxRef::Node(node) => {
                let atomic = occurrences
                    .atomics
                    .iter()
                    .find(|(_, actual)| actual == node)
                    .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?
                    .0;
                ComputedLineBoundaryInput::Atomic {
                    atomic,
                    side: location.side,
                }
            }
        };
        input.computed_line_boundaries.push(request);
    }
    // Keep caller causes, duplicates, and their actual request lineage. Text
    // alone normalizes identical targets into retained cause identities.
    let requests = input.computed_line_boundaries.clone();
    let processed = surgeist_text::process_contextual_source(input).map_err(|cause| {
        crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(root),
            crate::LayoutOperation::TextCapture,
            crate::LayoutErrorKindOf::Text(cause),
        )
    })?;
    let scopes = (0..inventory.scopes.len())
        .map(|i| processed.captured_scope(i))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?;
    let origins = (0..inventory.origins)
        .map(|i| processed.captured_origin(i))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?;
    let controls = (0..inventory.controls.len())
        .map(|i| processed.captured_control(i))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?;
    let atomics = (0..inventory.atomics)
        .map(|i| processed.captured_atomic(i))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?;
    let mut causes = Vec::new();
    for (i, request) in requests.into_iter().enumerate() {
        let cause = processed
            .captured_computed_line_boundary(i)
            .ok_or_else(|| error(root, WrappingScopeError::SourceAssociation))?;
        let matches = match (request, cause.target()) {
            (
                ComputedLineBoundaryInput::Scope { scope, side },
                CapturedComputedLineBoundaryTarget::Scope {
                    scope: actual,
                    side: actual_side,
                },
            ) => scopes.get(scope) == Some(&actual) && side == actual_side,
            (
                ComputedLineBoundaryInput::Endpoint { control },
                CapturedComputedLineBoundaryTarget::Endpoint(actual),
            ) => controls.get(control) == Some(&actual),
            (
                ComputedLineBoundaryInput::Atomic { atomic, side },
                CapturedComputedLineBoundaryTarget::Atomic {
                    position,
                    side: actual_side,
                },
            ) => atomics.get(atomic) == Some(&position) && side == actual_side,
            _ => false,
        };
        if !matches {
            return Err(error(root, WrappingScopeError::SourceAssociation));
        }
        causes.push(cause);
    }
    let receipt = ResolvedWrapEdges {
        snapshot,
        inventory,
        occurrences,
        scopes,
        origins,
        controls,
        atomics,
        causes,
    };
    receipt
        .validate_capture()
        .map_err(|reason| error(root, reason))?;
    Ok(ProcessedWrappingSource { processed, receipt })
}

fn semantics<S: crate::LayoutScalar>(view: WrapEdgeSourceNodeView<'_, S>) -> NodeSemantics {
    match view {
        WrapEdgeSourceNodeView::Box(input) => NodeSemantics::Box {
            display: input.display,
            position: input.position,
            float: input.float,
            edges: input.wrap_edges,
            flex_wrap: input.flex_wrap,
            order: input.item_order,
            direction: input.direction,
            writing_mode: input.writing_mode,
        },
        WrapEdgeSourceNodeView::Boundary(input) => {
            NodeSemantics::Boundary(input.kind(), input.direction(), input.writing_mode())
        }
        WrapEdgeSourceNodeView::LineBreak(input) => {
            NodeSemantics::LineBreak(input.display(), input.direction(), input.writing_mode())
        }
        WrapEdgeSourceNodeView::SourceOrigin { origin } => NodeSemantics::Origin(origin),
        WrapEdgeSourceNodeView::SourceControl { control } => NodeSemantics::Control(control),
    }
}

fn project<Source: WrapEdgeSource>(
    source: &Source,
    graph: &ValidationGraph<Source::Node>,
    inventory: &SourceInventory,
    occurrences: &SourceOccurrenceAssociations<Source::Node>,
) -> Result<SourceSnapshot<Source::Node>, WrappingScopeError> {
    let counts = graph.path_counts();
    let nodes = graph
        .nodes
        .iter()
        .map(|&node| (node, semantics(source.wrap_edge_source_node(node))))
        .collect::<Vec<_>>();
    let associations = source.source_scope_associations();
    for (i, association) in associations.iter().enumerate() {
        if !inventory
            .inline
            .get(association.scope)
            .copied()
            .unwrap_or(false)
        {
            return Err(WrappingScopeError::NonInlineScope);
        }
        if associations[..i].iter().any(|other| {
            other.scope == association.scope
                || association.pair.is_some() && other.pair == association.pair
        }) {
            return Err(WrappingScopeError::DuplicateScope);
        }
        let owner = graph
            .index(association.flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        if counts[owner] != 1 {
            return Err(WrappingScopeError::AmbiguousNode);
        }
        if !matches!(nodes[owner].1, NodeSemantics::Box { display, .. } if display.inner_display() == Display::Block)
        {
            return Err(WrappingScopeError::DifferentFlow);
        }
        if let Some((start, end)) = association.pair {
            let (flow, _) = native_span_with(graph, start, end, &counts, |node| {
                match source.wrap_edge_source_node(node) {
                    WrapEdgeSourceNodeView::Boundary(input) => Some(input.kind()),
                    _ => None,
                }
            })?;
            if flow != association.flow {
                return Err(WrappingScopeError::DifferentFlow);
            }
            let Some(pair) = inventory.scopes[association.scope] else {
                return Err(WrappingScopeError::SourceAssociation);
            };
            if !occurrences.controls.contains(&(pair.start, start))
                || !occurrences.controls.contains(&(pair.end, end))
            {
                return Err(WrappingScopeError::SourceAssociation);
            }
        }
    }
    for (bindings, bound) in [
        (&occurrences.origins, inventory.origins),
        (&occurrences.controls, inventory.controls.len()),
        (&occurrences.atomics, inventory.atomics),
    ] {
        if bindings.len() != bound {
            return Err(WrappingScopeError::SourceAssociation);
        }
        for (i, &(ordinal, node)) in bindings.iter().enumerate() {
            if ordinal >= bound
                || !graph.index(node).is_some_and(|j| counts[j] == 1)
                || bindings[..i]
                    .iter()
                    .any(|&(other, actual)| other == ordinal || actual == node)
            {
                return Err(WrappingScopeError::SourceAssociation);
            }
        }
    }
    for &(ordinal, node) in &occurrences.origins {
        if !nodes
            .iter()
            .any(|(actual, role)| *actual == node && *role == NodeSemantics::Origin(ordinal))
        {
            return Err(WrappingScopeError::SourceAssociation);
        }
    }
    for &(_, node) in &occurrences.atomics {
        if !nodes.iter().any(|(actual, role)| {
            *actual == node
                && matches!(role, NodeSemantics::Box { display, .. } if *display != Display::None)
        }) {
            return Err(WrappingScopeError::SourceAssociation);
        }
    }
    for &(node, ref role) in &nodes {
        match role {
            NodeSemantics::Origin(origin) if !occurrences.origins.contains(&(*origin, node)) => {
                return Err(WrappingScopeError::SourceAssociation);
            }
            NodeSemantics::Control(control)
                if !occurrences.controls.contains(&(*control, node)) =>
            {
                return Err(WrappingScopeError::SourceAssociation);
            }
            _ => {}
        }
    }
    for &(ordinal, node) in &occurrences.controls {
        let role = &nodes[graph
            .index(node)
            .ok_or(WrappingScopeError::NodeNotReachable)?]
        .1;
        let matches = match role {
            NodeSemantics::Control(actual) => *actual == ordinal,
            NodeSemantics::Boundary(InlineBoundaryKind::Start, ..) => {
                inventory.controls[ordinal] == ParagraphControlKind::InlineBoundaryStart
            }
            NodeSemantics::Boundary(InlineBoundaryKind::End, ..) => {
                inventory.controls[ordinal] == ParagraphControlKind::InlineBoundaryEnd
            }
            NodeSemantics::LineBreak(..) => {
                inventory.controls[ordinal] == ParagraphControlKind::ForcedBreak
            }
            _ => false,
        };
        if !matches {
            return Err(WrappingScopeError::SourceAssociation);
        }
    }
    let mut boxes = Vec::new();
    let mut aliases = Vec::new();
    for association in associations {
        let native = association
            .pair
            .map(|(start, end)| {
                native_span_with(graph, start, end, &counts, |node| {
                    match source.wrap_edge_source_node(node) {
                        WrapEdgeSourceNodeView::Boundary(input) => Some(input.kind()),
                        _ => None,
                    }
                })
                .map(|(_, span)| span)
            })
            .transpose()?;
        if let Some(pair) = association.pair {
            aliases.push((association.scope, pair));
        }
        boxes.push(BoxFact {
            box_: GeneratedBoxRef::Inline(InlineBoxRef::Scope(association.scope)),
            flow: association.flow,
            native,
            edges: WrapEdges::AUTO,
            unique: true,
            participation: GeneratedBoxParticipation::InFlow,
        });
    }
    let mut declared = Vec::new();
    for declaration in source.source_wrapping_scopes() {
        let index = match &declaration.box_ {
            InlineBoxRef::Scope(scope) => boxes
                .iter()
                .position(|b| b.box_ == GeneratedBoxRef::Inline(InlineBoxRef::Scope(*scope)))
                .ok_or(WrappingScopeError::SourceAssociation)?,
            InlineBoxRef::NativePair { start, end } => {
                if let Some((scope, _)) = aliases.iter().find(|(_, pair)| *pair == (*start, *end)) {
                    boxes
                        .iter()
                        .position(|b| {
                            b.box_ == GeneratedBoxRef::Inline(InlineBoxRef::Scope(*scope))
                        })
                        .ok_or(WrappingScopeError::SourceAssociation)?
                } else {
                    let (flow, native) = native_span_with(graph, *start, *end, &counts, |node| {
                        match source.wrap_edge_source_node(node) {
                            WrapEdgeSourceNodeView::Boundary(input) => Some(input.kind()),
                            _ => None,
                        }
                    })?;
                    boxes.push(BoxFact {
                        box_: GeneratedBoxRef::Inline(declaration.box_.clone()),
                        flow,
                        native: Some(native),
                        edges: WrapEdges::AUTO,
                        unique: true,
                        participation: GeneratedBoxParticipation::InFlow,
                    });
                    boxes.len() - 1
                }
            }
        };
        if source
            .source_wrapping_scopes()
            .iter()
            .filter(|other| other.box_ == declaration.box_)
            .count()
            != 1
        {
            return Err(WrappingScopeError::DuplicateScope);
        }
        if declared.iter().any(|&(actual, inside, edges)| {
            actual == index && (inside != declaration.wrap_inside || edges != declaration.edges)
        }) {
            return Err(WrappingScopeError::ConflictingScope);
        }
        declared.push((index, declaration.wrap_inside, declaration.edges));
        boxes[index].edges = declaration.edges;
    }
    for (i, &(node, ref role)) in nodes.iter().enumerate() {
        let Some(&parent) = graph.parents[i].first() else {
            continue;
        };
        let rank = graph.children[parent]
            .iter()
            .position(|&j| j == i)
            .ok_or(WrappingScopeError::ChildOrder)?;
        let (participation, edges, visible) = match role {
            NodeSemantics::Box {
                display,
                position,
                float,
                edges,
                ..
            } => {
                let parent = match nodes[parent].1 {
                    NodeSemantics::Box {
                        display, flex_wrap, ..
                    } => Some((display, flex_wrap)),
                    _ => None,
                };
                let wrapping = node_wrapping(*display, *position, *float, *edges, parent);
                (
                    wrapping.participation,
                    wrapping.inline_edges,
                    *display != Display::None,
                )
            }
            NodeSemantics::Origin(_) => (GeneratedBoxParticipation::InFlow, WrapEdges::AUTO, true),
            _ => continue,
        };
        if visible {
            boxes.push(BoxFact {
                box_: GeneratedBoxRef::Node(node),
                flow: graph.nodes[parent],
                native: Some(super::context::NativeSpan {
                    first: rank,
                    last: rank,
                }),
                edges,
                unique: counts[i] == 1,
                participation,
            });
        }
    }
    let mut unmodeled = Vec::new();
    for (owner, children) in graph.children.iter().enumerate() {
        for (rank, &child) in children.iter().enumerate() {
            if matches!(nodes[child].1, NodeSemantics::Boundary(..)) && !boxes.iter().any(|b| matches!(&b.box_, GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }) if *start == nodes[child].0 || *end == nodes[child].0) || aliases.iter().any(|(_, (start, end))| *start == nodes[child].0 || *end == nodes[child].0))
                || matches!(nodes[child].1, NodeSemantics::LineBreak(display, ..) if !display.is_none()) { unmodeled.push((graph.nodes[owner], rank)); }
        }
    }
    let contexts = graph
        .nodes
        .iter()
        .copied()
        .zip(counts.iter().map(|&n| n == 1))
        .collect::<Vec<_>>();
    let orders = normalize_orders(
        &mut boxes,
        &aliases,
        source.generated_child_order(),
        &contexts,
    )?;
    let Propagation { locations, .. } =
        propagate(&boxes, &orders, &unmodeled, &aliases, Vec::new())?;
    // Retain the consumed native-flow and ancestry witnesses, not a second host
    // graph. Opaque descendants without source/effect participation stay outside
    // this semantic receipt; their geometry remains ordinary final layout input.
    let mut flow_nodes = associations
        .iter()
        .map(|a| a.flow)
        .chain(
            source
                .generated_child_order()
                .iter()
                .map(|order| order.formatting_context),
        )
        .chain(locations.iter().map(|location| location.flow))
        .collect::<Vec<_>>();
    for &(_, node) in occurrences
        .origins
        .iter()
        .chain(&occurrences.controls)
        .chain(&occurrences.atomics)
    {
        let index = graph
            .index(node)
            .ok_or(WrappingScopeError::SourceAssociation)?;
        flow_nodes.extend(graph.parents[index].iter().map(|&i| graph.nodes[i]));
    }
    for (i, (_, role)) in nodes.iter().enumerate() {
        if matches!(role, NodeSemantics::Box { edges, .. } if !edges.is_auto()) {
            flow_nodes.extend(graph.parents[i].iter().map(|&i| graph.nodes[i]));
        }
    }
    let mut flows = Vec::new();
    let mut retained = vec![false; nodes.len()];
    retained[0] = true;
    for flow in flow_nodes {
        if flows.iter().any(|(actual, _)| *actual == flow) {
            continue;
        }
        let index = graph
            .index(flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        retained[index] = true;
        let children = graph.children[index]
            .iter()
            .map(|&i| {
                retained[i] = true;
                graph.nodes[i]
            })
            .collect();
        flows.push((flow, children));
    }
    let mut pending = retained
        .iter()
        .enumerate()
        .filter_map(|(i, &keep)| keep.then_some(i))
        .collect::<Vec<_>>();
    while let Some(index) = pending.pop() {
        for &parent in &graph.parents[index] {
            if !retained[parent] {
                retained[parent] = true;
                pending.push(parent);
            }
        }
    }
    let parent_paths = retained
        .iter()
        .enumerate()
        .filter(|(_, keep)| **keep)
        .map(|(i, _)| {
            (
                graph.nodes[i],
                graph.parents[i].iter().map(|&j| graph.nodes[j]).collect(),
            )
        })
        .collect();
    let nodes = nodes
        .into_iter()
        .zip(retained)
        .filter_map(|(fact, keep)| keep.then_some(fact))
        .collect();
    Ok(SourceSnapshot {
        nodes,
        flows,
        parent_paths,
        declarations: source.source_wrapping_scopes().to_vec(),
        associations: associations.to_vec(),
        orders: source.generated_child_order().to_vec(),
        locations,
    })
}

impl<Node: Copy + Eq> ResolvedWrapEdges<Node> {
    fn captured_box(&self, box_: &GeneratedBoxRef<Node, usize>) -> Option<CapturedBoxIndex> {
        match box_ {
            GeneratedBoxRef::Inline(InlineBoxRef::Scope(i)) => Some(CapturedBoxIndex::Scope(*i)),
            GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }) => self
                .snapshot
                .associations
                .iter()
                .find(|a| a.pair == Some((*start, *end)))
                .map(|a| CapturedBoxIndex::Scope(a.scope))
                .or_else(|| {
                    let first = self
                        .occurrences
                        .controls
                        .iter()
                        .find(|(_, node)| node == start)
                        .and_then(|(i, _)| self.controls.get(*i))?;
                    let last = self
                        .occurrences
                        .controls
                        .iter()
                        .find(|(_, node)| node == end)
                        .and_then(|(i, _)| self.controls.get(*i))?;
                    self.scopes
                        .iter()
                        .position(|scope| {
                            scope.entry_control().as_ref() == Some(first)
                                && scope.exit_control().as_ref() == Some(last)
                        })
                        .map(CapturedBoxIndex::Scope)
                }),
            GeneratedBoxRef::Node(node) => self
                .occurrences
                .origins
                .iter()
                .find(|(_, actual)| actual == node)
                .map(|(i, _)| CapturedBoxIndex::Origin(*i))
                .or_else(|| {
                    self.occurrences
                        .atomics
                        .iter()
                        .find(|(_, actual)| actual == node)
                        .map(|(i, _)| CapturedBoxIndex::Atomic(*i))
                }),
        }
    }

    fn position_domain(&self, position: &CapturedEventPosition) -> Option<CapturedScope> {
        self.scopes
            .iter()
            .find(|scope| {
                scope.flow_events().is_some_and(|mut events| {
                    events.any(|event| {
                        use surgeist_text::CapturedFlowEvent as E;
                        match event {
                            E::Origin {
                                position: actual, ..
                            }
                            | E::Atomic {
                                position: actual, ..
                            } => actual == *position,
                            E::Control(control) => control.event_position() == *position,
                            _ => false,
                        }
                    })
                })
            })
            .cloned()
    }

    fn canonical_box(&self, box_: CapturedBoxIndex) -> Result<CanonicalBox, WrappingScopeError> {
        match box_ {
            CapturedBoxIndex::Scope(i) => Ok(CanonicalBox::Scope(self.scopes[i].clone())),
            CapturedBoxIndex::Origin(i) => {
                let position = self.origins[i].clone();
                let domain = self
                    .position_domain(&position)
                    .ok_or(WrappingScopeError::SourceAssociation)?;
                Ok(CanonicalBox::Occurrence { position, domain })
            }
            CapturedBoxIndex::Atomic(i) => {
                let position = self.atomics[i].clone();
                let domain = self
                    .position_domain(&position)
                    .ok_or(WrappingScopeError::SourceAssociation)?;
                Ok(CanonicalBox::Occurrence { position, domain })
            }
        }
    }

    fn canonical_ref(
        &self,
        box_: &GeneratedBoxRef<Node, usize>,
        inventory: &CanonicalOrderInventory,
    ) -> Result<CanonicalBox, WrappingScopeError> {
        if let GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }) = box_ {
            let start = self
                .occurrences
                .controls
                .iter()
                .find(|(_, node)| node == start)
                .and_then(|(i, _)| self.controls.get(*i))
                .ok_or(WrappingScopeError::SourceAssociation)?;
            let end = self
                .occurrences
                .controls
                .iter()
                .find(|(_, node)| node == end)
                .and_then(|(i, _)| self.controls.get(*i))
                .ok_or(WrappingScopeError::SourceAssociation)?;
            return CanonicalOrderInventory::native_pair(
                &inventory.scopes,
                start.clone(),
                end.clone(),
            )
            .map_err(|_| WrappingScopeError::SourceAssociation);
        }
        self.canonical_box(
            self.captured_box(box_)
                .ok_or(WrappingScopeError::SourceAssociation)?,
        )
    }

    fn canonical_fact(
        &self,
        box_: &GeneratedBoxRef<Node, usize>,
        inventory: &CanonicalOrderInventory,
    ) -> Result<Option<CanonicalBox>, WrappingScopeError> {
        if matches!(
            box_,
            GeneratedBoxRef::Inline(InlineBoxRef::NativePair { .. })
        ) {
            return self.canonical_ref(box_, inventory).map(Some);
        }
        self.captured_box(box_)
            .map(|key| self.canonical_box(key))
            .transpose()
    }

    fn content_domain(&self, node: Node) -> Result<Option<CapturedScope>, WrappingScopeError> {
        let Some((_, children)) = self.snapshot.flows.iter().find(|(flow, _)| *flow == node) else {
            return Ok(None);
        };
        let mut domains = self
            .snapshot
            .associations
            .iter()
            .filter(|a| a.flow == node)
            .map(|a| self.scopes[a.scope].domain())
            .collect::<Vec<_>>();
        for &(i, child) in &self.occurrences.origins {
            if children.contains(&child) {
                domains.push(
                    self.position_domain(&self.origins[i])
                        .ok_or(WrappingScopeError::SourceAssociation)?,
                );
            }
        }
        for &(i, child) in &self.occurrences.atomics {
            if children.contains(&child) {
                domains.push(
                    self.position_domain(&self.atomics[i])
                        .ok_or(WrappingScopeError::SourceAssociation)?,
                );
            }
        }
        for &(i, child) in &self.occurrences.controls {
            if children.contains(&child) {
                domains.push(self.controls[i].paragraph().domain());
            }
        }
        let first = domains.first().cloned();
        if domains.iter().any(|domain| Some(domain) != first.as_ref()) {
            return Err(WrappingScopeError::SourceAssociation);
        }
        Ok(first)
    }

    fn native_boxes(
        &self,
        inventory: &CanonicalOrderInventory,
    ) -> Result<Vec<CanonicalBox>, WrappingScopeError> {
        let mut native = Vec::new();
        for declaration in &self.snapshot.declarations {
            if matches!(declaration.box_, InlineBoxRef::NativePair { .. }) {
                let fact = self.canonical_ref(
                    &GeneratedBoxRef::Inline(declaration.box_.clone()),
                    inventory,
                )?;
                if matches!(fact, CanonicalBox::NativePair { .. }) && !native.contains(&fact) {
                    native.push(fact);
                }
            }
        }
        CanonicalOrderInventory::validate_native(&inventory.scopes, &native)
            .map_err(|_| WrappingScopeError::SourceAssociation)?;
        Ok(native)
    }

    fn validate_parent_paths(
        &self,
        inventory: &CanonicalOrderInventory,
        native: &[CanonicalBox],
    ) -> Result<(), WrappingScopeError> {
        use surgeist_text::SourceBoxSide;
        for location in &self.snapshot.locations {
            if !location.effect.forced {
                continue;
            }
            let mut current = self.canonical_ref(&location.declared_box, inventory)?;
            let mut stopped = false;
            for step in &location.parents {
                if stopped || self.canonical_ref(&step.child, inventory)? != current {
                    return Err(WrappingScopeError::SourceAssociation);
                }
                let parent = inventory
                    .parent(&current, native)
                    .map_err(|_| WrappingScopeError::SourceAssociation)?
                    .filter(CanonicalBox::is_inline)
                    .ok_or(WrappingScopeError::SourceAssociation)?;
                if self.canonical_ref(&step.parent, inventory)? != parent {
                    return Err(WrappingScopeError::SourceAssociation);
                }
                if let Some(sibling) = &step.stopped_by {
                    if let Some(sibling) = self.canonical_fact(sibling, inventory)? {
                        inventory
                            .validate_sibling(&parent, &current, &sibling, location.side, native)
                            .map_err(|_| WrappingScopeError::SourceAssociation)?;
                    } else if !step.physical {
                        // An actual native parent/window can own a numeric
                        // sibling without inventing a Text occurrence for it.
                        return Err(WrappingScopeError::MissingChildOrder);
                    }
                    stopped = true;
                } else {
                    let at = step
                        .children
                        .iter()
                        .position(|child| *child == step.child)
                        .ok_or(WrappingScopeError::SourceAssociation)?;
                    let needed = match location.side {
                        SourceBoxSide::Before => &step.children[..=at],
                        SourceBoxSide::After => &step.children[at..],
                    };
                    let children = needed
                        .iter()
                        .map(|child| self.canonical_fact(child, inventory))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                    let (prefix, suffix) = match location.side {
                        SourceBoxSide::Before => (
                            ChildOrderCompleteness::Complete,
                            ChildOrderCompleteness::Unknown,
                        ),
                        SourceBoxSide::After => (
                            ChildOrderCompleteness::Unknown,
                            ChildOrderCompleteness::Complete,
                        ),
                    };
                    inventory
                        .validate(&parent, &children, prefix, suffix, native)
                        .map_err(|_| WrappingScopeError::MissingChildOrder)?;
                    current = parent;
                }
            }
            if !stopped
                && inventory
                    .parent(&current, native)
                    .map_err(|_| WrappingScopeError::SourceAssociation)?
                    .is_some_and(|parent| parent.is_inline())
            {
                return Err(WrappingScopeError::MissingChildOrder);
            }
            if self.canonical_ref(&location.box_, inventory)? != current {
                return Err(WrappingScopeError::SourceAssociation);
            }
            if self.content_domain(location.flow)?.as_ref() != Some(&current.domain()) {
                return Err(WrappingScopeError::SourceAssociation);
            }
        }
        Ok(())
    }

    fn validate_capture(&self) -> Result<(), WrappingScopeError> {
        let inventory = CanonicalOrderInventory {
            scopes: self.scopes.clone(),
            occurrences: (0..self.origins.len())
                .map(CapturedBoxIndex::Origin)
                .chain((0..self.atomics.len()).map(CapturedBoxIndex::Atomic))
                .map(|key| self.canonical_box(key))
                .collect::<Result<Vec<_>, _>>()?,
            coverage: CanonicalCoverage::WholeInput,
        };
        let native = self.native_boxes(&inventory)?;
        // Canonical parent/domain/order comes from Text, never an event parser.
        for order in &self.snapshot.orders {
            let parent = match &order.parent {
                GeneratedBoxRef::Inline(_) => Some(self.canonical_ref(&order.parent, &inventory)?),
                GeneratedBoxRef::Node(node) => self.content_domain(*node)?.map(CanonicalBox::Scope),
            };
            let children = order
                .children
                .iter()
                .map(|child| self.canonical_fact(&child.box_, &inventory))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let Some(parent) = parent else {
                if !children.is_empty() {
                    return Err(WrappingScopeError::SourceAssociation);
                }
                continue;
            };
            inventory
                .validate(&parent, &children, order.prefix, order.suffix, &native)
                .map_err(|_| WrappingScopeError::SourceAssociation)?;
        }
        self.validate_parent_paths(&inventory, &native)?;
        for association in &self.snapshot.associations {
            let scope = &self.scopes[association.scope];
            if scope.role() != surgeist_text::ScopeRole::Inline {
                return Err(WrappingScopeError::NonInlineScope);
            }
            let positions = self
                .occurrences
                .origins
                .iter()
                .map(|&(i, node)| (self.origins[i].clone(), node))
                .chain(
                    self.occurrences
                        .atomics
                        .iter()
                        .map(|&(i, node)| (self.atomics[i].clone(), node)),
                )
                .chain(
                    self.occurrences
                        .controls
                        .iter()
                        .map(|&(i, node)| (self.controls[i].event_position(), node)),
                );
            for (position, node) in positions {
                if self.position_domain(&position).as_ref() == Some(&scope.domain()) {
                    let children = self
                        .snapshot
                        .flows
                        .iter()
                        .find(|(flow, _)| *flow == association.flow)
                        .map(|(_, children)| children)
                        .ok_or(WrappingScopeError::SourceAssociation)?;
                    if !children.contains(&node) {
                        return Err(WrappingScopeError::SourceAssociation);
                    }
                }
            }
            match (
                association.pair,
                scope.entry_control(),
                scope.exit_control(),
            ) {
                (Some((start, end)), Some(a), Some(b))
                    if self
                        .occurrences
                        .controls
                        .iter()
                        .any(|(i, node)| *node == start && self.controls.get(*i) == Some(&a))
                        && self
                            .occurrences
                            .controls
                            .iter()
                            .any(|(i, node)| *node == end && self.controls.get(*i) == Some(&b)) => {
                }
                (None, None, None) => {}
                _ => return Err(WrappingScopeError::SourceAssociation),
            }
        }
        Ok(())
    }

    pub(crate) fn validate_final<Tree: LayoutTree<Node = Node> + WrapEdgeSource>(
        &self,
        tree: &Tree,
        root: Node,
        paragraphs: &[crate::TextParagraphBinding<Node>],
        wrapping: &WrappingInput<Node>,
    ) -> LayoutResultOf<Node, ValidationGraph<Node>, Tree::Scalar, Tree::MeasureError> {
        let graph = discover_graph(tree, root)?;
        let actual = project(tree, &graph, &self.inventory, &self.occurrences)
            .map_err(|reason| error(root, reason))?;
        if actual != self.snapshot {
            return Err(error(root, WrappingScopeError::PlanMismatch));
        }
        for association in &self.snapshot.associations {
            let domain = self.scopes[association.scope].domain();
            let mut observed = false;
            for paragraph in paragraphs {
                if paragraph
                    .prepared
                    .captured_owner()
                    .is_some_and(|p| p.domain() == domain)
                {
                    observed = true;
                    if paragraph.container != association.flow {
                        return Err(error(root, WrappingScopeError::PlanMismatch));
                    }
                }
            }
            if !observed {
                return Err(error(root, WrappingScopeError::PlanMismatch));
            }
        }
        for &(node, ref expected) in &self.snapshot.nodes {
            let actual = tree.layout_input(node);
            let agrees = match expected {
                NodeSemantics::Box { .. } => actual.as_box().is_some_and(|input| {
                    semantics(WrapEdgeSourceNodeView::Box(input)) == *expected
                }),
                NodeSemantics::Boundary(..) => actual.as_inline_boundary().is_some_and(|input| {
                    semantics(WrapEdgeSourceNodeView::Boundary(input)) == *expected
                }),
                NodeSemantics::LineBreak(..) => actual.as_line_break().is_some_and(|input| {
                    semantics(WrapEdgeSourceNodeView::LineBreak(input)) == *expected
                }),
                NodeSemantics::Origin(i) => {
                    actual
                        .as_inline_text()
                        .and_then(|p| p.origin().view().capture_position())
                        .as_ref()
                        == self.origins.get(*i)
                }
                NodeSemantics::Control(i) => {
                    actual
                        .as_text_control()
                        .and_then(|p| p.occurrence().captured_control())
                        .as_ref()
                        == self.controls.get(*i)
                }
            };
            if !agrees {
                return Err(error(root, WrappingScopeError::PlanMismatch));
            }
        }
        for &(i, node) in &self.occurrences.origins {
            if !paragraphs.iter().flat_map(|p| &p.origins).any(|b| {
                b.node == node && b.origin.view().capture_position().as_ref() == self.origins.get(i)
            }) {
                return Err(error(root, WrappingScopeError::PlanMismatch));
            }
        }
        for &(i, node) in &self.occurrences.controls {
            if !paragraphs.iter().flat_map(|p| &p.controls).any(|b| {
                b.node == node && b.occurrence.captured_control().as_ref() == self.controls.get(i)
            }) {
                return Err(error(root, WrappingScopeError::PlanMismatch));
            }
        }
        for &(i, node) in &self.occurrences.atomics {
            if !paragraphs.iter().flat_map(|p| &p.atomics).any(|b| {
                b.node == node
                    && b.occurrence.view().capture_position().as_ref() == self.atomics.get(i)
            }) {
                return Err(error(root, WrappingScopeError::PlanMismatch));
            }
        }
        self.validate_wrapping(wrapping)
            .map_err(|reason| error(root, reason))?;
        let mut seen = vec![false; self.causes.len()];
        for paragraph in paragraphs {
            for offer in paragraph.prepared.opportunity_views() {
                if let Some(group) = offer.computed_line_boundary() {
                    for cause in group.view().causes() {
                        let mut expected = false;
                        for (known, reached) in self.causes.iter().zip(&mut seen) {
                            if *known == cause {
                                *reached = true;
                                expected = true;
                            }
                        }
                        if !expected {
                            return Err(error(root, WrappingScopeError::PlanMismatch));
                        }
                    }
                }
            }
        }
        if seen.contains(&false) {
            return Err(error(root, WrappingScopeError::PlanMismatch));
        }
        Ok(graph)
    }

    fn validate_wrapping(&self, wrapping: &WrappingInput<Node>) -> Result<(), WrappingScopeError> {
        let box_of = |row: &WrappingScopeInput<Node>| match row {
            WrappingScopeInput::Captured { scope, .. } => self
                .scopes
                .iter()
                .position(|actual| actual == scope)
                .map(InlineBoxRef::Scope),
            WrappingScopeInput::NativePair { start, end, .. } => self
                .snapshot
                .associations
                .iter()
                .find(|a| a.pair == Some((*start, *end)))
                .map_or_else(
                    || {
                        Some(InlineBoxRef::NativePair {
                            start: *start,
                            end: *end,
                        })
                    },
                    |a| Some(InlineBoxRef::Scope(a.scope)),
                ),
        };
        for row in &wrapping.scopes {
            let box_ = box_of(row).ok_or(WrappingScopeError::PlanMismatch)?;
            let (inside, edges) = match row {
                WrappingScopeInput::Captured {
                    wrap_inside, edges, ..
                }
                | WrappingScopeInput::NativePair {
                    wrap_inside, edges, ..
                } => (*wrap_inside, *edges),
            };
            let declaration = self.snapshot.declarations.iter().find(|d| {
                d.box_ == box_
                    || match (&d.box_, &box_) {
                        (InlineBoxRef::NativePair { start, end }, InlineBoxRef::Scope(i)) => self
                            .snapshot
                            .associations
                            .iter()
                            .any(|a| a.scope == *i && a.pair == Some((*start, *end))),
                        _ => false,
                    }
            });
            if declaration.map_or(inside != WrapInside::Auto || !edges.is_auto(), |d| {
                d.wrap_inside != inside || d.edges != edges
            }) {
                return Err(WrappingScopeError::PlanMismatch);
            }
        }
        for declaration in &self.snapshot.declarations {
            if !wrapping.scopes.iter().any(|row| {
                box_of(row).is_some_and(|box_| {
                    box_ == declaration.box_
                        || match (&declaration.box_, box_) {
                            (InlineBoxRef::NativePair { start, end }, InlineBoxRef::Scope(i)) => {
                                self.snapshot
                                    .associations
                                    .iter()
                                    .any(|a| a.scope == i && a.pair == Some((*start, *end)))
                            }
                            _ => false,
                        }
                })
            }) {
                return Err(WrappingScopeError::PlanMismatch);
            }
        }
        let convert = |box_: &GeneratedBoxRef<Node, usize>| -> Result<GeneratedBoxRef<Node>, WrappingScopeError> { Ok(match box_ {
            GeneratedBoxRef::Node(node) => GeneratedBoxRef::Node(*node),
            GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }) => GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start: *start, end: *end }),
            GeneratedBoxRef::Inline(InlineBoxRef::Scope(i)) => GeneratedBoxRef::Inline(InlineBoxRef::Scope(self.scopes.get(*i).ok_or(WrappingScopeError::SourceAssociation)?.clone())),
        }) };
        let expected = self
            .snapshot
            .orders
            .iter()
            .map(|order| {
                Ok(InlineChildOrderInput {
                    parent: convert(&order.parent)?,
                    children: order
                        .children
                        .iter()
                        .map(|child| {
                            Ok(GeneratedChildBoxInput {
                                box_: convert(&child.box_)?,
                                participation: child.participation,
                            })
                        })
                        .collect::<Result<Vec<_>, WrappingScopeError>>()?,
                    prefix: order.prefix,
                    suffix: order.suffix,
                    formatting_context: order.formatting_context,
                })
            })
            .collect::<Result<Vec<_>, WrappingScopeError>>()?;
        if expected != wrapping.child_order {
            return Err(WrappingScopeError::PlanMismatch);
        }
        Ok(())
    }
}
