use super::context::NativeSpan;
use super::*;
use crate::{Display, LayoutScalar, LayoutTree, Position, engine::validation::ValidationGraph};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EdgeLocation<Node, Scope = surgeist_text::CapturedScope> {
    pub(crate) flow: Node,
    pub(crate) declared_box: GeneratedBoxRef<Node, Scope>,
    pub(crate) box_: GeneratedBoxRef<Node, Scope>,
    pub(crate) native: Option<NativeSpan>,
    pub(crate) side: surgeist_text::SourceBoxSide,
    pub(crate) effect: EdgeEffect,
}
#[derive(Clone, Debug)]
pub(crate) struct EdgePlan<Node> {
    pub(crate) locations: Vec<EdgeLocation<Node>>,
    pub(crate) dependencies: Vec<Node>,
    pub(crate) subtrees: Vec<SubtreeRealization>,
}
/// This is generated participation proof, independent of strut dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SubtreeRealization {
    Empty,
    Participating,
    Unavailable,
}

#[derive(Clone)]
pub(super) struct BoxFact<Node, Scope = surgeist_text::CapturedScope> {
    pub(super) box_: GeneratedBoxRef<Node, Scope>,
    pub(super) flow: Node,
    pub(super) native: Option<NativeSpan>,
    pub(super) edges: WrapEdges,
    pub(super) unique: bool,
    pub(super) participation: GeneratedBoxParticipation,
}
pub(super) struct ChildOrderProof {
    parent: usize,
    children: Vec<(usize, GeneratedBoxParticipation)>,
    prefix: ChildOrderCompleteness,
    suffix: ChildOrderCompleteness,
}

pub(super) struct NodeWrapping {
    pub(super) participation: GeneratedBoxParticipation,
    pub(super) inline_edges: WrapEdges,
    mode: Option<WrappingMode>,
}

/// Shared phase-independent applicability, before native or captured identity
/// is attached. This is the sole generated-node property conversion.
pub(super) fn node_wrapping(
    display: Display,
    position: Position,
    float: crate::Float,
    edges: WrapEdges,
    parent: Option<(Display, crate::FlexWrap)>,
) -> NodeWrapping {
    let out = position == Position::Absolute || !float.is_none();
    let inline = display.is_inline_level()
        && !out
        && parent.is_some_and(|(display, _)| display.inner_display() == Display::Block);
    let mode = parent.and_then(|(parent_display, flex_wrap)| {
        if parent_display.inner_display() == Display::Flex {
            Some(WrappingMode::Flex {
                multiline: flex_wrap != crate::FlexWrap::NoWrap,
            })
        } else if display.is_inline_level() && parent_display.inner_display() == Display::Block {
            Some(WrappingMode::Inline)
        } else {
            None
        }
    });
    NodeWrapping {
        participation: if out {
            GeneratedBoxParticipation::OutOfFlow
        } else {
            GeneratedBoxParticipation::InFlow
        },
        inline_edges: if inline { edges } else { WrapEdges::AUTO },
        mode,
    }
}
impl<Node: Copy + Eq> EdgePlan<Node> {
    pub(crate) fn new<Tree: LayoutTree<Node = Node>>(
        tree: &Tree,
        graph: &ValidationGraph<Node>,
        rows: &[ScopeRow<Node>],
        orders: &[InlineChildOrderInput<Node>],
        proofs: &[WrappingScopeProof<Node>],
    ) -> Result<Self, WrappingScopeError> {
        let counts = graph.path_counts();
        let mut boxes = Vec::new();
        for row in rows {
            let box_ = GeneratedBoxRef::Inline(if let Some(scope) = &row.capture {
                InlineBoxRef::Scope(scope.clone())
            } else {
                let (start, end) = row.pair.ok_or(WrappingScopeError::CaptureAssociation)?;
                InlineBoxRef::NativePair { start, end }
            });
            boxes.push(BoxFact {
                box_,
                flow: row.flow,
                native: row.native.clone(),
                edges: row.edges,
                unique: true,
                participation: GeneratedBoxParticipation::InFlow,
            });
        }
        let mut dependencies = Vec::new();
        for (i, &node) in graph.nodes.iter().enumerate() {
            if let Some(input) = tree.layout_input(node).as_box() {
                let Some(&parent) = graph.parents[i].first() else {
                    continue;
                };
                if graph.parents[i].len() != 1 && !input.wrap_edges.is_auto() {
                    return Err(WrappingScopeError::AmbiguousNode);
                }
                let rank = graph.children[parent]
                    .iter()
                    .position(|&j| j == i)
                    .ok_or(WrappingScopeError::ChildOrder)?;
                let parent_input = tree.node_input(graph.nodes[parent]);
                let wrapping = node_wrapping(
                    input.display,
                    input.position,
                    input.float,
                    input.wrap_edges,
                    Some((parent_input.display, parent_input.flex_wrap)),
                );
                if input.display != Display::None
                    && input.position != Position::Absolute
                    && input.float.is_none()
                    && wrapping.mode.is_some_and(|mode| {
                        [input.wrap_edges.before, input.wrap_edges.after]
                            .iter()
                            .any(|d| {
                                let e = d.effect(mode);
                                e.forced || e.avoided
                            })
                    })
                {
                    dependencies.push(node);
                }
                if input.display != Display::None {
                    boxes.push(BoxFact {
                        participation: wrapping.participation,
                        unique: counts[i] == 1,
                        box_: GeneratedBoxRef::Node(node),
                        flow: graph.nodes[parent],
                        native: Some(NativeSpan {
                            first: rank,
                            last: rank,
                        }),
                        edges: wrapping.inline_edges,
                    });
                }
            }
        }
        for (i, &node) in graph.nodes.iter().enumerate() {
            if tree.layout_input(node).as_inline_text().is_some() {
                let parent = *graph.parents[i]
                    .first()
                    .ok_or(WrappingScopeError::DifferentFlow)?;
                let rank = graph.children[parent]
                    .iter()
                    .position(|&j| j == i)
                    .ok_or(WrappingScopeError::ChildOrder)?;
                boxes.push(BoxFact {
                    participation: GeneratedBoxParticipation::InFlow,
                    unique: counts[i] == 1,
                    box_: GeneratedBoxRef::Node(node),
                    flow: graph.nodes[parent],
                    native: Some(NativeSpan {
                        first: rank,
                        last: rank,
                    }),
                    edges: WrapEdges::AUTO,
                });
            }
        }
        // Visible controls are not generated child boxes. Their omission cannot
        // prove a first/last child when the requested side crosses them.
        let mut unmodeled = Vec::new();
        for (owner, children) in graph.children.iter().enumerate() {
            for (rank, &child) in children.iter().enumerate() {
                let node = graph.nodes[child];
                let input = tree.layout_input(node);
                let endpoint = input.as_inline_boundary().is_some()
                    && !rows
                        .iter()
                        .any(|r| r.pair.is_some_and(|(a, b)| a == node || b == node));
                let line_break = input
                    .as_line_break()
                    .is_some_and(|b| !b.display().is_none());
                if endpoint || line_break {
                    unmodeled.push((graph.nodes[owner], rank));
                }
            }
        }
        let aliases = rows
            .iter()
            .filter_map(|row| row.capture.clone().zip(row.pair))
            .collect::<Vec<_>>();
        let contexts = graph
            .nodes
            .iter()
            .copied()
            .zip(counts.iter().map(|&n| n == 1))
            .collect::<Vec<_>>();
        let normalized = normalize_orders(&mut boxes, &aliases, orders, &contexts)?;
        validate_canonical_orders(&boxes, &normalized, proofs)?;
        let mut realization = RealizationProof::new(tree, &boxes, rows, &normalized, &unmodeled);
        let subtrees = (0..rows.len())
            .map(|index| realization.resolve(index))
            .collect::<Result<Vec<_>, _>>()?;
        let Propagation {
            locations,
            dependencies,
        } = propagate(&boxes, &normalized, &unmodeled, &aliases, dependencies)?;
        Ok(Self {
            locations,
            dependencies,
            subtrees,
        })
    }
}

/// Reuses the same normalized generated-child inventory that owns propagation.
/// Strut geometry is deliberately absent from this participation proof.
struct RealizationProof<'a, Tree: LayoutTree> {
    tree: &'a Tree,
    boxes: &'a [BoxFact<Tree::Node>],
    rows: &'a [ScopeRow<Tree::Node>],
    orders: Vec<Option<&'a ChildOrderProof>>,
    unmodeled: &'a [(Tree::Node, usize)],
    memo: Vec<Option<SubtreeRealization>>,
    children: Vec<Vec<usize>>,
    visiting: Vec<bool>,
    text: std::collections::HashMap<
        surgeist_text::PreparedText,
        std::collections::HashSet<surgeist_text::PreparedOrigin>,
    >,
}
impl<'a, Tree: LayoutTree> RealizationProof<'a, Tree> {
    fn new(
        tree: &'a Tree,
        boxes: &'a [BoxFact<Tree::Node>],
        rows: &'a [ScopeRow<Tree::Node>],
        orders: &'a [ChildOrderProof],
        unmodeled: &'a [(Tree::Node, usize)],
    ) -> Self {
        let mut indexed = vec![None; boxes.len()];
        for order in orders {
            indexed[order.parent] = Some(order);
        }
        Self {
            tree,
            boxes,
            rows,
            orders: indexed,
            unmodeled,
            memo: vec![None; boxes.len()],
            children: vec![Vec::new(); boxes.len()],
            visiting: vec![false; boxes.len()],
            text: std::collections::HashMap::new(),
        }
    }
    fn leaf(&mut self, index: usize) -> Option<SubtreeRealization> {
        use SubtreeRealization::{Empty, Participating};
        let GeneratedBoxRef::Node(node) = self.boxes[index].box_ else {
            return None;
        };
        let input = self.tree.layout_input(node);
        let Some(participation) = input.as_inline_text() else {
            // Atomics/opaque generated nodes remain participants, including
            // zero/signed-width cases. No measured-width atomicity shortcut.
            return Some(Participating);
        };
        let origin = participation.origin();
        let prepared = origin.prepared_text();
        #[expect(
            clippy::mutable_key_type,
            reason = "Text preparation and origin hashing use immutable allocation identity and occurrence index; lazy cache contents never affect the keys"
        )]
        let participating = self.text.entry(prepared.clone()).or_insert_with(|| {
            prepared
                .unit_views()
                .filter(|unit| {
                    unit.empty_origin().is_none()
                        && unit.processed_range().start() != unit.processed_range().end()
                })
                .flat_map(|unit| {
                    unit.origins()
                        .map(|piece| piece.origin())
                        .collect::<Vec<_>>()
                })
                .collect()
        });
        Some(if participating.contains(origin) {
            Participating
        } else {
            Empty
        })
    }
    fn expand(&mut self, index: usize) -> Result<SubtreeRealization, WrappingScopeError> {
        use SubtreeRealization::{Empty, Participating, Unavailable};
        let decoration =
            self.rows
                .get(index)
                .and_then(|row| row.pair)
                .is_some_and(|(start, end)| {
                    [start, end].into_iter().any(|node| {
                        self.tree
                            .layout_input(node)
                            .as_inline_boundary()
                            .is_some_and(|boundary| {
                                boundary.edge().border() != Tree::Scalar::ZERO
                                    || boundary.edge().padding() != Tree::Scalar::ZERO
                            })
                    })
                });
        if decoration {
            return Ok(Participating);
        }
        let order = if let Some(order) = self.orders[index] {
            (order.prefix == ChildOrderCompleteness::Complete
                && order.suffix == ChildOrderCompleteness::Complete)
                .then(|| order.children.clone())
        } else if self.boxes[index].native.is_some() {
            Some(inferred_children(self.boxes, index)?)
        } else {
            None
        };
        let unrepresented = self.boxes[index].native.as_ref().is_some_and(|span| {
            self.unmodeled.iter().any(|(flow, rank)| {
                *flow == self.boxes[index].flow && span.first < *rank && *rank < span.last
            })
        });
        let state = if order.is_some() && !unrepresented {
            Empty
        } else {
            Unavailable
        };
        self.children[index] = order
            .into_iter()
            .flatten()
            .filter_map(|(child, participation)| {
                (participation == GeneratedBoxParticipation::InFlow).then_some(child)
            })
            .collect();
        Ok(state)
    }
    fn resolve(&mut self, root: usize) -> Result<SubtreeRealization, WrappingScopeError> {
        use SubtreeRealization::{Empty, Participating, Unavailable};
        let mut stack = vec![(root, false)];
        let mut initial = Vec::new();
        while let Some((index, finishing)) = stack.pop() {
            if self.memo[index].is_some() {
                continue;
            }
            if finishing {
                let mut state = initial.pop().ok_or(WrappingScopeError::ChildOrder)?;
                for &child in &self.children[index] {
                    match self.memo[child].ok_or(WrappingScopeError::ChildOrder)? {
                        Participating => {
                            state = Participating;
                            break;
                        }
                        Unavailable if state == Empty => state = Unavailable,
                        Empty | Unavailable => {}
                    }
                }
                self.visiting[index] = false;
                self.memo[index] = Some(state);
                continue;
            }
            if self.visiting[index] {
                return Err(WrappingScopeError::ChildOrder);
            }
            if let Some(value) = self.leaf(index) {
                self.memo[index] = Some(value);
                continue;
            }
            let state = self.expand(index)?;
            if state == Participating {
                self.memo[index] = Some(state);
                continue;
            }
            self.visiting[index] = true;
            initial.push(state);
            stack.push((index, true));
            for &child in self.children[index].iter().rev() {
                stack.push((child, false));
            }
        }
        self.memo[root].ok_or(WrappingScopeError::ChildOrder)
    }
}

fn normalize_id<Node: Copy + Eq, Scope: Eq>(
    boxes: &[BoxFact<Node, Scope>],
    aliases: &[(Scope, (Node, Node))],
    id: &GeneratedBoxRef<Node, Scope>,
) -> Result<usize, WrappingScopeError> {
    boxes
        .iter()
        .position(|b| match (&b.box_, id) {
            (a, b) if a == b => true,
            (
                GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)),
                GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }),
            ) => aliases
                .iter()
                .any(|(actual, pair)| actual == scope && *pair == (*start, *end)),
            _ => false,
        })
        .ok_or(WrappingScopeError::ChildOrder)
}

fn inferred_children<Node: Copy + Eq, Scope>(
    boxes: &[BoxFact<Node, Scope>],
    parent: usize,
) -> Result<Vec<(usize, GeneratedBoxParticipation)>, WrappingScopeError> {
    let span = boxes[parent]
        .native
        .as_ref()
        .ok_or(WrappingScopeError::MissingChildOrder)?;
    let mut children = boxes
        .iter()
        .enumerate()
        .filter(|(i, b)| {
            *i != parent
                && b.flow == boxes[parent].flow
                && b.native
                    .as_ref()
                    .is_some_and(|s| span.first < s.first && s.last < span.last)
        })
        .filter(|(i, b)| {
            !boxes.iter().enumerate().any(|(j, m)| {
                j != parent
                    && j != *i
                    && m.flow == b.flow
                    && matches!(m.box_, GeneratedBoxRef::Inline(_))
                    && m.native
                        .as_ref()
                        .zip(b.native.as_ref())
                        .is_some_and(|(a, b)| {
                            span.first < a.first
                                && a.first < b.first
                                && b.last < a.last
                                && a.last < span.last
                        })
            })
        })
        .map(|(i, b)| (i, b.participation))
        .collect::<Vec<_>>();
    if children.iter().any(|&(i, _)| !boxes[i].unique) {
        return Err(WrappingScopeError::AmbiguousNode);
    }
    children.sort_by_key(|&(i, _)| boxes[i].native.as_ref().map_or(0, |s| s.first));
    Ok(children)
}

fn validate_canonical_orders<Node: Copy + Eq>(
    boxes: &[BoxFact<Node>],
    orders: &[ChildOrderProof],
    proofs: &[WrappingScopeProof<Node>],
) -> Result<(), WrappingScopeError> {
    use super::canonical_order::CanonicalOrderError;
    let Some(bindings) = proofs.iter().find_map(|proof| match proof {
        WrappingScopeProof::Captured(proof) => proof.order.as_ref(),
        _ => None,
    }) else {
        return Ok(());
    };
    let fact = |index: usize| match &boxes[index].box_ {
        GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)) => {
            Some(CanonicalBox::Scope(scope.clone()))
        }
        GeneratedBoxRef::Node(node) => bindings
            .nodes
            .iter()
            .find(|(actual, _)| actual == node)
            .map(|(_, fact)| fact.clone()),
        GeneratedBoxRef::Inline(InlineBoxRef::NativePair { .. }) => None,
    };
    for order in orders {
        let children = order
            .children
            .iter()
            .filter_map(|&(i, _)| fact(i))
            .collect::<Vec<_>>();
        let parent = match &boxes[order.parent].box_ {
            GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)) => Some(scope.clone()),
            GeneratedBoxRef::Node(node) => {
                let mut domains = bindings
                    .flows
                    .iter()
                    .filter(|(actual, _)| actual == node)
                    .map(|(_, domain)| domain);
                let first = domains.next().cloned();
                if domains.any(|domain| Some(domain) != first.as_ref()) {
                    return Err(WrappingScopeError::UnsupportedIncidence);
                }
                first
            }
            GeneratedBoxRef::Inline(InlineBoxRef::NativePair { .. }) => None,
        };
        let Some(parent) = parent else {
            if order
                .children
                .iter()
                .any(|&(i, _)| boxes[i].native.is_none())
            {
                return Err(WrappingScopeError::UnsupportedIncidence);
            }
            continue;
        };
        bindings
            .inventory
            .validate(&parent, &children, order.prefix, order.suffix)
            .map_err(|error| match error {
                CanonicalOrderError::Unavailable => WrappingScopeError::UnsupportedIncidence,
                CanonicalOrderError::Contradiction => WrappingScopeError::ChildOrder,
            })?;
    }
    Ok(())
}

pub(super) fn normalize_orders<Node: Copy + Eq, Scope: Clone + Eq>(
    boxes: &mut Vec<BoxFact<Node, Scope>>,
    aliases: &[(Scope, (Node, Node))],
    orders: &[InlineChildOrderInput<Node, Scope>],
    contexts: &[(Node, bool)],
) -> Result<Vec<ChildOrderProof>, WrappingScopeError> {
    let mut normalized: Vec<ChildOrderProof> = Vec::new();
    for order in orders {
        let parent = if let GeneratedBoxRef::Node(node) = order.parent {
            if node == order.formatting_context && !boxes.iter().any(|b| b.box_ == order.parent) {
                boxes.push(BoxFact {
                    box_: order.parent.clone(),
                    flow: node,
                    native: None,
                    edges: WrapEdges::AUTO,
                    unique: contexts
                        .iter()
                        .any(|&(actual, unique)| actual == node && unique),
                    participation: GeneratedBoxParticipation::InFlow,
                });
            }
            normalize_id(boxes, aliases, &order.parent)?
        } else {
            normalize_id(boxes, aliases, &order.parent)?
        };
        if boxes[parent].flow != order.formatting_context
            && boxes[parent].box_ != GeneratedBoxRef::Node(order.formatting_context)
        {
            return Err(WrappingScopeError::DifferentFlow);
        }
        if normalized.iter().any(|row| row.parent == parent) {
            return Err(WrappingScopeError::ChildOrder);
        }
        let mut children = Vec::new();
        for child in &order.children {
            let index = normalize_id(boxes, aliases, &child.box_)?;
            if !boxes[index].unique {
                return Err(WrappingScopeError::AmbiguousNode);
            }
            if index == parent
                || children.iter().any(|(i, _)| *i == index)
                || boxes[index].flow != order.formatting_context
            {
                return Err(WrappingScopeError::ChildOrder);
            }
            if matches!(boxes[parent].box_, GeneratedBoxRef::Inline(_))
                && let (Some(a), Some(b)) = (&boxes[parent].native, &boxes[index].native)
                && !(a.first < b.first && b.last < a.last)
            {
                return Err(WrappingScopeError::ChildOrder);
            }
            if boxes[index].participation != child.participation {
                return Err(WrappingScopeError::ChildOrder);
            }
            if normalized
                .iter()
                .any(|row| row.children.iter().any(|(i, _)| *i == index))
            {
                return Err(WrappingScopeError::ChildOrder);
            }
            children.push((index, child.participation));
        }
        for pair in children.windows(2) {
            if let (Some(a), Some(b)) = (&boxes[pair[0].0].native, &boxes[pair[1].0].native)
                && a.last >= b.first
            {
                return Err(WrappingScopeError::ChildOrder);
            }
        }
        if boxes[parent].native.is_some()
            && matches!(boxes[parent].box_, GeneratedBoxRef::Inline(_))
        {
            let actual = inferred_children(boxes, parent)?;
            // Only genuine scoped identities may lack a physical span. Source
            // claims are authenticated after Text; final captured facts are
            // checked by the same canonical-order owner before admission.
            if children.iter().any(|&(i, _)| {
                boxes[i].native.is_none()
                    && !matches!(
                        boxes[i].box_,
                        GeneratedBoxRef::Inline(InlineBoxRef::Scope(_))
                    )
            }) {
                return Err(WrappingScopeError::UnsupportedIncidence);
            }
            let physical = children
                .iter()
                .copied()
                .filter(|&(i, _)| boxes[i].native.is_some())
                .collect::<Vec<_>>();
            for (i, _) in &physical {
                if !actual.iter().any(|(j, _)| i == j) {
                    return Err(WrappingScopeError::ChildOrder);
                }
            }
            if let (Some(&(first, _)), Some(&(last, _))) = (physical.first(), physical.last()) {
                let a = actual
                    .iter()
                    .position(|(i, _)| *i == first)
                    .ok_or(WrappingScopeError::ChildOrder)?;
                let b = actual
                    .iter()
                    .position(|(i, _)| *i == last)
                    .ok_or(WrappingScopeError::ChildOrder)?;
                if (order.prefix == ChildOrderCompleteness::Complete && a != 0)
                    || (order.suffix == ChildOrderCompleteness::Complete && b + 1 != actual.len())
                {
                    return Err(WrappingScopeError::ChildOrder);
                }
                if actual[a..=b] != physical {
                    return Err(WrappingScopeError::ChildOrder);
                }
            } else if !actual.is_empty()
                && (order.prefix == ChildOrderCompleteness::Complete
                    || order.suffix == ChildOrderCompleteness::Complete)
            {
                return Err(WrappingScopeError::ChildOrder);
            }
        }
        normalized.push(ChildOrderProof {
            parent,
            children,
            prefix: order.prefix,
            suffix: order.suffix,
        });
    }
    Ok(normalized)
}

pub(super) struct Propagation<Node, Scope> {
    pub(super) locations: Vec<EdgeLocation<Node, Scope>>,
    pub(super) dependencies: Vec<Node>,
}

pub(super) fn propagate<Node: Copy + Eq, Scope: Clone + Eq>(
    boxes: &[BoxFact<Node, Scope>],
    normalized: &[ChildOrderProof],
    unmodeled: &[(Node, usize)],
    aliases: &[(Scope, (Node, Node))],
    mut dependencies: Vec<Node>,
) -> Result<Propagation<Node, Scope>, WrappingScopeError> {
    let mut locations = Vec::new();
    for (index, b) in boxes.iter().enumerate() {
        for (side, decl) in [
            (surgeist_text::SourceBoxSide::Before, b.edges.before),
            (surgeist_text::SourceBoxSide::After, b.edges.after),
        ] {
            let effect = decl.effect(WrappingMode::Inline);
            if !effect.forced && !effect.avoided {
                continue;
            }
            if !b.unique {
                return Err(WrappingScopeError::AmbiguousNode);
            }
            let mut target = index;
            if effect.forced {
                let mut seen = Vec::new();
                loop {
                    if seen.contains(&target) {
                        return Err(WrappingScopeError::ChildOrder);
                    }
                    seen.push(target);
                    let mut candidates = boxes
                        .iter()
                        .enumerate()
                        .filter(|(i, p)| {
                            *i != target
                                && p.flow == boxes[target].flow
                                && matches!(p.box_, GeneratedBoxRef::Inline(_))
                        })
                        .filter(|(_, p)| {
                            p.native
                                .as_ref()
                                .zip(boxes[target].native.as_ref())
                                .is_some_and(|(a, b)| a.first < b.first && b.last < a.last)
                        })
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();
                    candidates.sort_by_key(|&i| {
                        boxes[i]
                            .native
                            .as_ref()
                            .map_or(usize::MAX, |s| s.last - s.first)
                    });
                    let explicit = normalized
                        .iter()
                        .find(|row| row.children.iter().any(|(i, _)| *i == target));
                    let Some(parent) = explicit
                        .map(|row| row.parent)
                        .or_else(|| candidates.first().copied())
                    else {
                        break;
                    };
                    if matches!(boxes[parent].box_, GeneratedBoxRef::Node(_)) {
                        break;
                    }
                    let owned;
                    let (children, prefix, suffix) =
                        if let Some(order) = normalized.iter().find(|row| row.parent == parent) {
                            (order.children.as_slice(), order.prefix, order.suffix)
                        } else {
                            owned = inferred_children(boxes, parent)?;
                            (
                                owned.as_slice(),
                                ChildOrderCompleteness::Complete,
                                ChildOrderCompleteness::Complete,
                            )
                        };
                    let at = children
                        .iter()
                        .position(|(i, _)| *i == target)
                        .ok_or(WrappingScopeError::ChildOrder)?;
                    let before = side == surgeist_text::SourceBoxSide::Before;
                    let sibling = if before {
                        &children[..at]
                    } else {
                        &children[at + 1..]
                    };
                    if sibling
                        .iter()
                        .any(|(_, p)| *p == GeneratedBoxParticipation::InFlow)
                    {
                        break;
                    }
                    // A canonical boundaries-none box uses its explicit
                    // generated order; physical spans are not fabricated.
                    if let (Some(parent_span), Some(target_span)) =
                        (&boxes[parent].native, &boxes[target].native)
                        && unmodeled.iter().any(|&(flow, rank)| {
                            flow == boxes[target].flow
                                && parent_span.first < rank
                                && rank < parent_span.last
                                && if before {
                                    rank < target_span.first
                                } else {
                                    rank > target_span.last
                                }
                        })
                    {
                        return Err(WrappingScopeError::MissingChildOrder);
                    }
                    if (if before { prefix } else { suffix }) != ChildOrderCompleteness::Complete {
                        return Err(WrappingScopeError::MissingChildOrder);
                    }
                    target = parent;
                }
            }
            let target = &boxes[target];
            if !dependencies.contains(&target.flow) {
                dependencies.push(target.flow);
            }
            if let GeneratedBoxRef::Node(node) = &target.box_
                && !dependencies.contains(node)
            {
                dependencies.push(*node);
            }
            let pair = match &target.box_ {
                GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }) => {
                    Some((*start, *end))
                }
                GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)) => aliases
                    .iter()
                    .find(|(actual, _)| actual == scope)
                    .map(|(_, pair)| *pair),
                GeneratedBoxRef::Node(_) => None,
            };
            if let Some((a, b)) = pair {
                dependencies.extend([a, b]);
            }
            locations.push(EdgeLocation {
                flow: target.flow,
                declared_box: b.box_.clone(),
                box_: target.box_.clone(),
                native: target.native.clone(),
                side,
                effect,
            });
        }
    }
    Ok(Propagation {
        locations,
        dependencies,
    })
}
