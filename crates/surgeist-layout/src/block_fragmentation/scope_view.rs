//! Disposable source-window view over the captured scope partition.
use super::scopes::{ScopeId, Scopes};
use crate::*;
use std::ops::Range;

pub(super) struct ScopeView<'a, Tree: LayoutTree> {
    tree: &'a Tree,
    scopes: &'a Scopes<Tree::Node>,
    styles: Vec<NodeInputOf<Tree::Scalar>>,
    children: Vec<Vec<Tree::Node>>,
}
impl<'a, Tree: LayoutTree> ScopeView<'a, Tree> {
    pub(super) fn new(
        tree: &'a Tree,
        scopes: &'a Scopes<Tree::Node>,
        key: &ViewKey,
        axes: FlowAxes,
        trimmed: &[crate::block::fragment_margins::MarginOwner<Tree::Node>],
    ) -> Self {
        let scope = key.scope;
        let range = &key.range;
        let partial = key.partial;
        let resumed = key.resumed;
        let opening = key.opening;
        let own = &scopes.scopes[scope.0];
        let mut styles = Vec::with_capacity(scopes.source.nodes.len());
        let mut children = Vec::with_capacity(scopes.source.nodes.len());
        for (index, entry) in scopes.source.nodes.iter().enumerate() {
            let local = scopes.range(scope, index);
            let mut style = tree.node_input(entry.node).clone();
            if entry.atomic_owner.is_none()
                && let Some(local) = &local
            {
                if local.start < range.start
                    || resumed == Some(local.start)
                    || own.contents && own.owner == index && !opening
                {
                    super::source::clear_edge(&mut style.padding, axes.block_start());
                    super::source::clear_edge(&mut style.border, axes.block_start());
                }
                if local.end > range.end || partial.is_some_and(|unit| local.end == unit + 1) {
                    super::source::clear_edge(&mut style.padding, axes.block_end());
                    super::source::clear_edge(&mut style.border, axes.block_end());
                }
                if local.start < range.start && local.end > range.start {
                    super::source::clear_margin(&mut style.margin, axes.block_start());
                }
                if local.start < range.end && local.end > range.end {
                    super::source::clear_margin(&mut style.margin, axes.block_end());
                }
            }
            for owner in trimmed.iter().filter(|owner| owner.node == entry.node) {
                super::source::clear_margin(&mut style.margin, owner.side);
            }
            let foreign_owner =
                scopes.owned[index].is_some() && !(own.contents && own.owner == index);
            children.push(if foreign_owner {
                Vec::new()
            } else {
                entry
                    .children
                    .iter()
                    .copied()
                    .filter(|&child| {
                        scopes
                            .range(scope, child)
                            .is_some_and(|local| local.start < range.end && local.end > range.start)
                    })
                    .map(|child| scopes.source.nodes[child].node)
                    .collect()
            });
            styles.push(style);
        }
        let mut view = Self {
            tree,
            scopes,
            styles,
            children,
        };
        if let Some(closing) = key.closing {
            let root = scopes.source.nodes[own.owner].node;
            view.project_root_edges(root, axes, opening, closing);
        }
        view
    }
    /// Projection chooses whether this actual principal span owns the Slice
    /// edges. Sizing still receives the original node through content::prepare.
    fn project_root_edges(
        &mut self,
        node: Tree::Node,
        axes: FlowAxes,
        opening: bool,
        closing: bool,
    ) {
        let index = self.index(node);
        if !opening {
            super::source::clear_edge(&mut self.styles[index].padding, axes.block_start());
            super::source::clear_edge(&mut self.styles[index].border, axes.block_start());
            super::source::clear_margin(&mut self.styles[index].margin, axes.block_start());
        }
        if !closing {
            super::source::clear_edge(&mut self.styles[index].padding, axes.block_end());
            super::source::clear_edge(&mut self.styles[index].border, axes.block_end());
            super::source::clear_margin(&mut self.styles[index].margin, axes.block_end());
        }
    }
    fn index(&self, node: Tree::Node) -> usize {
        self.scopes
            .source
            .nodes
            .iter()
            .position(|entry| entry.node == node)
            .expect("scope view references captured nodes")
    }
}
impl<Tree: LayoutTree> Traverse for ScopeView<'_, Tree> {
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, Self::Node>>
    where
        Self: 'a;
    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children[self.index(node)].iter().copied()
    }
    fn child_count(&self, node: Self::Node) -> usize {
        self.children[self.index(node)].len()
    }
    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[self.index(node)][index]
    }
}
impl<Tree: LayoutTree> LayoutTree for ScopeView<'_, Tree> {
    type MeasureError = Tree::MeasureError;
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        &self.styles[self.index(node)]
    }
    fn layout_input(&self, node: Self::Node) -> LayoutInputViewOf<'_, Self::Scalar> {
        LayoutInputViewOf::Box(self.node_input(node))
    }
    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.tree.has_leaf_measurement(node)
    }
    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.tree.measure_leaf(node, input)
    }
}

/// Identity for the exact native source window and selected frame. Frame is
/// part of this binding because one unchanged window may project a different
/// principal span there; sizing and placement remain in Block's Pending.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct ViewKey {
    pub(super) scope: ScopeId,
    pub(super) frame: usize,
    pub(super) range: Range<usize>,
    pub(super) partial: Option<usize>,
    pub(super) resumed: Option<usize>,
    pub(super) opening: bool,
    pub(super) closing: Option<bool>,
}
pub(super) struct Views {
    entries: Vec<ViewKey>,
}
impl Views {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub(super) fn issue(
        &mut self,
        key: ViewKey,
    ) -> Result<crate::engine::block_scope::SourceViewId, ()> {
        if let Some(index) = self.entries.iter().position(|entry| *entry == key) {
            return Ok(crate::engine::block_scope::SourceViewId(index + 1));
        }
        let id = self.entries.len().checked_add(1).ok_or(())?;
        self.entries.try_reserve(1).map_err(|_| ())?;
        self.entries.push(key);
        Ok(crate::engine::block_scope::SourceViewId(id))
    }
    pub(super) fn get(&self, id: crate::engine::block_scope::SourceViewId) -> Option<&ViewKey> {
        self.entries.get(id.0.checked_sub(1)?)
    }
}
