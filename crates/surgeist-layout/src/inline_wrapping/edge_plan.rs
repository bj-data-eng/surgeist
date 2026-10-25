use super::context::NativeSpan;
use super::*;
use crate::{Display, LayoutTree, Position, engine::validation::ValidationGraph};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EdgeLocation<Node> {
    pub(crate) flow: Node,
    pub(crate) box_: GeneratedBoxRef<Node>,
    pub(crate) native: Option<NativeSpan>,
    pub(crate) side: surgeist_text::SourceBoxSide,
    pub(crate) effect: EdgeEffect,
}
#[derive(Clone, Debug)]
pub(crate) struct EdgePlan<Node> {
    pub(crate) locations: Vec<EdgeLocation<Node>>,
    pub(crate) dependencies: Vec<Node>,
}
#[derive(Clone)]
struct BoxFact<Node> {
    box_: GeneratedBoxRef<Node>,
    flow: Node,
    native: Option<NativeSpan>,
    edges: WrapEdges,
    unique: bool,
}
struct ChildOrderProof {
    parent: usize,
    children: Vec<(usize, GeneratedBoxParticipation)>,
    prefix: ChildOrderCompleteness,
    suffix: ChildOrderCompleteness,
}
impl<Node: Copy + Eq> EdgePlan<Node> {
    pub(crate) fn new<Tree: LayoutTree<Node = Node>>(
        tree: &Tree,
        graph: &ValidationGraph<Node>,
        rows: &[ScopeRow<Node>],
        orders: &[InlineChildOrderInput<Node>],
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
                let mode = if parent_input.display.inner_display() == Display::Flex {
                    Some(WrappingMode::Flex {
                        multiline: parent_input.flex_wrap != crate::FlexWrap::NoWrap,
                    })
                } else if input.display.is_inline_level()
                    && parent_input.display.inner_display() == Display::Block
                {
                    Some(WrappingMode::Inline)
                } else {
                    None
                };
                if input.display != Display::None
                    && input.position != Position::Absolute
                    && input.float.is_none()
                    && mode.is_some_and(|mode| {
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
                let inline = input.display.is_inline_level()
                    && input.position != Position::Absolute
                    && input.float.is_none()
                    && tree.node_input(graph.nodes[parent]).display.inner_display()
                        == Display::Block;
                if input.display != Display::None {
                    boxes.push(BoxFact {
                        unique: counts[i] == 1,
                        box_: GeneratedBoxRef::Node(node),
                        flow: graph.nodes[parent],
                        native: Some(NativeSpan {
                            first: rank,
                            last: rank,
                        }),
                        edges: if inline {
                            input.wrap_edges
                        } else {
                            WrapEdges::AUTO
                        },
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
        let mut normalized: Vec<ChildOrderProof> = Vec::new();
        for order in orders {
            let parent = if let GeneratedBoxRef::Node(node) = order.parent {
                if node == order.formatting_context && !boxes.iter().any(|b| b.box_ == order.parent)
                {
                    boxes.push(BoxFact {
                        box_: order.parent.clone(),
                        flow: node,
                        native: None,
                        edges: WrapEdges::AUTO,
                        unique: graph.index(node).is_some_and(|i| counts[i] == 1),
                    });
                }
                normalize_id(&boxes, rows, &order.parent)?
            } else {
                normalize_id(&boxes, rows, &order.parent)?
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
                let index = normalize_id(&boxes, rows, &child.box_)?;
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
                if let GeneratedBoxRef::Node(node) = &boxes[index].box_ {
                    let input = tree.node_input(*node);
                    let actual = if input.position == Position::Absolute || !input.float.is_none() {
                        GeneratedBoxParticipation::OutOfFlow
                    } else {
                        GeneratedBoxParticipation::InFlow
                    };
                    if actual != child.participation {
                        return Err(WrappingScopeError::ChildOrder);
                    }
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
                let actual = inferred_children(&boxes, tree, parent)?;
                for (i, _) in &children {
                    if !actual.iter().any(|(j, _)| i == j) {
                        return Err(WrappingScopeError::ChildOrder);
                    }
                }
                if let (Some(&(first, _)), Some(&(last, _))) = (children.first(), children.last()) {
                    let a = actual
                        .iter()
                        .position(|(i, _)| *i == first)
                        .ok_or(WrappingScopeError::ChildOrder)?;
                    let b = actual
                        .iter()
                        .position(|(i, _)| *i == last)
                        .ok_or(WrappingScopeError::ChildOrder)?;
                    if (order.prefix == ChildOrderCompleteness::Complete && a != 0)
                        || (order.suffix == ChildOrderCompleteness::Complete
                            && b + 1 != actual.len())
                    {
                        return Err(WrappingScopeError::ChildOrder);
                    }
                    if actual[a..=b] != children {
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
                        let (children, prefix, suffix) = if let Some(order) =
                            normalized.iter().find(|row| row.parent == parent)
                        {
                            (order.children.as_slice(), order.prefix, order.suffix)
                        } else {
                            owned = inferred_children(&boxes, tree, parent)?;
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
                        let parent_span = boxes[parent]
                            .native
                            .as_ref()
                            .ok_or(WrappingScopeError::MissingChildOrder)?;
                        let target_span = boxes[target]
                            .native
                            .as_ref()
                            .ok_or(WrappingScopeError::MissingChildOrder)?;
                        if unmodeled.iter().any(|&(flow, rank)| {
                            flow == boxes[target].flow
                                && parent_span.first < rank
                                && rank < parent_span.last
                                && if before {
                                    rank < target_span.first
                                } else {
                                    rank > target_span.last
                                }
                        }) {
                            return Err(WrappingScopeError::MissingChildOrder);
                        }
                        if (if before { prefix } else { suffix })
                            != ChildOrderCompleteness::Complete
                        {
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
                if let Some(row) = rows
                    .iter()
                    .find(|r| r.flow == target.flow && r.native == target.native)
                    && let Some((a, b)) = row.pair
                {
                    dependencies.extend([a, b]);
                }
                locations.push(EdgeLocation {
                    flow: target.flow,
                    box_: target.box_.clone(),
                    native: target.native.clone(),
                    side,
                    effect,
                });
            }
        }
        Ok(Self {
            locations,
            dependencies,
        })
    }
}

fn normalize_id<Node: Copy + Eq>(
    boxes: &[BoxFact<Node>],
    rows: &[ScopeRow<Node>],
    id: &GeneratedBoxRef<Node>,
) -> Result<usize, WrappingScopeError> {
    boxes
        .iter()
        .position(|b| match (&b.box_, id) {
            (a, b) if a == b => true,
            (
                GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope)),
                GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end }),
            ) => rows
                .iter()
                .any(|r| r.capture.as_ref() == Some(scope) && r.pair == Some((*start, *end))),
            _ => false,
        })
        .ok_or(WrappingScopeError::ChildOrder)
}

fn inferred_children<Node: Copy + Eq, Tree: LayoutTree<Node = Node>>(
    boxes: &[BoxFact<Node>],
    tree: &Tree,
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
        .map(|(i, b)| {
            let out = if let GeneratedBoxRef::Node(node) = b.box_ {
                let input = tree.node_input(node);
                input.position == Position::Absolute || !input.float.is_none()
            } else {
                false
            };
            (
                i,
                if out {
                    GeneratedBoxParticipation::OutOfFlow
                } else {
                    GeneratedBoxParticipation::InFlow
                },
            )
        })
        .collect::<Vec<_>>();
    if children.iter().any(|&(i, _)| !boxes[i].unique) {
        return Err(WrappingScopeError::AmbiguousNode);
    }
    children.sort_by_key(|&(i, _)| boxes[i].native.as_ref().map_or(0, |s| s.first));
    Ok(children)
}
