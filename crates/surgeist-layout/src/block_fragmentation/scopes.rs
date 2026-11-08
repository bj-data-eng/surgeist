//! Immutable consuming inventories over one captured source graph.
use super::source::{Inventory, SourceKey, unsupported};
use crate::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ScopeId(pub(super) usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Unit {
    Atomic(usize),
    Owner { node: usize, contents: ScopeId },
    End(usize),
    PrincipalEnd(usize),
}
impl Unit {
    pub(super) fn node(self) -> usize {
        match self {
            Self::Atomic(node)
            | Self::End(node)
            | Self::PrincipalEnd(node)
            | Self::Owner { node, .. } => node,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Location {
    pub(super) scope: ScopeId,
    pub(super) first: usize,
    pub(super) end: usize,
}
pub(super) struct Scope {
    pub(super) owner: usize,
    pub(super) parent: Option<ScopeId>,
    pub(super) contents: bool,
    pub(super) units: Vec<Unit>,
}
pub(super) struct Scopes<Node> {
    pub(super) source: Arc<Inventory<Node>>,
    pub(super) scopes: Vec<Scope>,
    pub(super) locations: Vec<Location>,
    pub(super) owned: Vec<Option<ScopeId>>,
}

enum Work {
    Enter {
        node: usize,
        scope: ScopeId,
    },
    Close {
        node: usize,
        scope: ScopeId,
        end: bool,
        root: bool,
    },
}

pub(super) fn capped<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axes: FlowAxes,
) -> Result<bool, crate::sizing::resolve::SizingResolutionError<S>> {
    let own = FlowAxes::new(style.writing_mode, style.direction);
    Ok(own.block_axis() == axes.block_axis()
        && !super::source::unsupported_continuation(
            style,
            axes,
            super::source::ContinuationProfile::Maximum,
        )?)
}

impl<Node: Copy + Eq> Scopes<Node> {
    pub(super) fn new<Tree: LayoutTree<Node = Node>>(
        source: Inventory<Node>,
        tree: &Tree,
        axes: FlowAxes,
    ) -> LayoutResultOf<Node, Self, Tree::Scalar, Tree::MeasureError> {
        let mut result = Self {
            locations: vec![
                Location {
                    scope: ScopeId(0),
                    first: 0,
                    end: 0
                };
                source.nodes.len()
            ],
            owned: vec![None; source.nodes.len()],
            source: Arc::new(source),
            scopes: vec![Scope {
                owner: 0,
                parent: None,
                contents: false,
                units: Vec::new(),
            }],
        };
        let mut work = vec![Work::Enter {
            node: 0,
            scope: ScopeId(0),
        }];
        while let Some(step) = work.pop() {
            match step {
                Work::Enter { node, scope } => {
                    let entry = &result.source.nodes[node];
                    let first = result.scopes[scope.0].units.len();
                    result.locations[node] = Location {
                        scope,
                        first,
                        end: first,
                    };
                    if entry.atomic_owner == Some(node) {
                        result.scopes[scope.0].units.push(Unit::Atomic(node));
                        let location = Location {
                            scope,
                            first,
                            end: first + 1,
                        };
                        let mut atomic = vec![node];
                        while let Some(child) = atomic.pop() {
                            result.locations[child] = location;
                            atomic.extend(result.source.nodes[child].children.iter().copied());
                        }
                    } else if capped(tree.node_input(entry.node), axes)
                        .map_err(|error| crate::error::sizing_resolution_error(entry.node, error))?
                    {
                        let child = ScopeId(result.scopes.len());
                        result.scopes.push(Scope {
                            owner: node,
                            parent: Some(scope),
                            contents: true,
                            units: Vec::new(),
                        });
                        result.owned[node] = Some(child);
                        result.scopes[scope.0].units.push(Unit::Owner {
                            node,
                            contents: child,
                        });
                        result.scopes[scope.0].units.push(Unit::PrincipalEnd(node));
                        result.locations[node].end = first + 2;
                        work.push(Work::Close {
                            node,
                            scope: child,
                            end: true,
                            root: true,
                        });
                        work.extend(
                            entry
                                .children
                                .iter()
                                .rev()
                                .map(|&node| Work::Enter { node, scope: child }),
                        );
                    } else {
                        let minimum = axes
                            .logical_size(tree.node_input(entry.node).min_size.clone())
                            .block;
                        let end = !super::source::unsupported_continuation(
                            tree.node_input(entry.node),
                            axes,
                            super::source::ContinuationProfile::Minimum,
                        )
                        .map_err(|error| {
                            crate::error::sizing_resolution_error(entry.node, error)
                        })? && !minimum.is_auto()
                            && crate::block::fragment_minimum::absolute(
                                &minimum,
                                axes.block_axis(),
                            )
                            .map_err(|error| {
                                crate::error::sizing_resolution_error(entry.node, error)
                            })?;
                        work.push(Work::Close {
                            node,
                            scope,
                            end,
                            root: false,
                        });
                        work.extend(
                            entry
                                .children
                                .iter()
                                .rev()
                                .map(|&node| Work::Enter { node, scope }),
                        );
                    }
                }
                Work::Close {
                    node,
                    scope,
                    end,
                    root,
                } => {
                    if end {
                        result.scopes[scope.0].units.push(Unit::End(node));
                    }
                    if !root {
                        result.locations[node].end = result.scopes[scope.0].units.len();
                    }
                }
            }
        }
        Ok(result)
    }
    pub(super) fn range(&self, scope: ScopeId, node: usize) -> Option<std::ops::Range<usize>> {
        let own = &self.scopes[scope.0];
        if own.contents && own.owner == node {
            return Some(0..own.units.len());
        }
        let value = self.locations[node];
        (value.scope == scope).then_some(value.first..value.end)
    }
    pub(super) fn occurrences(
        &self,
        scope: ScopeId,
    ) -> Vec<FragmentationOccurrence<SourceKey<Node>>> {
        self.scopes[scope.0]
            .units
            .iter()
            .map(|&unit| FragmentationOccurrence {
                key: match unit {
                    Unit::Atomic(node) => SourceKey::Atomic(self.source.nodes[node].node),
                    Unit::End(node) => SourceKey::BoxEnd(self.source.nodes[node].node),
                    Unit::PrincipalEnd(node) => {
                        SourceKey::PrincipalEnd(self.source.nodes[node].node)
                    }
                    Unit::Owner { node, .. } => SourceKey::Owner(self.source.nodes[node].node),
                },
                source_extent: usize::from(matches!(unit, Unit::Atomic(_))),
                monolithic: matches!(unit, Unit::Atomic(_)),
            })
            .collect()
    }
    pub(super) fn controls<Tree: LayoutTree<Node = Node>>(
        &self,
        tree: &Tree,
        scope: ScopeId,
        boundary: usize,
    ) -> (
        Vec<FragmentationBreakControl>,
        Vec<FragmentationBreakControl>,
        Vec<FragmentationBreakInside>,
    ) {
        let mut before = Vec::new();
        let mut after = Vec::new();
        let mut inside = Vec::new();
        for (index, entry) in self.source.nodes.iter().enumerate() {
            if entry.hidden || entry.atomic_owner.is_some_and(|owner| owner != index) {
                continue;
            }
            let style = tree.node_input(entry.node);
            let propagated = |start: bool| {
                let mut owner = index;
                while let Some(parent) = self.source.nodes[owner].parent {
                    if self.source.nodes[parent].atomic_owner.is_some() {
                        break;
                    }
                    let mut children = self.source.nodes[parent]
                        .children
                        .iter()
                        .copied()
                        .filter(|&child| !self.source.nodes[child].hidden);
                    let edge = if start {
                        children.next()
                    } else {
                        children.next_back()
                    };
                    if edge != Some(owner) {
                        break;
                    }
                    owner = parent;
                }
                let location = self.locations[owner];
                (
                    location.scope,
                    if start { location.first } else { location.end },
                )
            };
            let control = |value| FragmentationBreakControl {
                value,
                role: FragmentationRole::Normal,
                source_order: index,
            };
            if propagated(true) == (scope, boundary) {
                before.push(control(style.break_before));
            }
            if propagated(false) == (scope, boundary) {
                after.push(control(style.break_after));
            }
            if self.locations[index].scope == scope
                && self.locations[index].first < boundary
                && self.locations[index].end > boundary
            {
                inside.push(style.break_inside);
            }
        }
        let own = &self.scopes[scope.0];
        if own.contents && boundary > 0 && boundary < own.units.len() {
            let mut node = Some(own.owner);
            while let Some(index) = node {
                inside.push(tree.node_input(self.source.nodes[index].node).break_inside);
                node = self.source.nodes[index].parent;
            }
        }
        (before, after, inside)
    }
    pub(super) fn validate(&self) -> Result<(), LayoutErrorOf<Node>> {
        for (index, scope) in self.scopes.iter().enumerate() {
            if scope.parent.is_some_and(|parent| parent.0 >= index) {
                return Err(unsupported(self.source.nodes[scope.owner].node));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "scopes_tests.rs"]
mod tests;
