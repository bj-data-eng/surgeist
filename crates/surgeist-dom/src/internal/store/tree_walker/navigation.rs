//! Suspended ordinary-tree algorithms from the pinned DOM TreeWalker clauses.
//!
//! A filter result is deliberately an unrestricted `u16`: each algorithm tests
//! the particular constants named by its clause, rather than normalizing all
//! other results to skip or reject. Tree relations are read after resumption,
//! never speculatively before the filter runs. Removal adjustment belongs to
//! NodeIterator and does not participate in these algorithms.

use crate::internal::identity::NodeId;
use crate::internal::tree_walker::DomTreeWalkerDirection;

// DOM NodeFilter constants, not a closed set of callback return values.
pub(super) const ACCEPT: u16 = 1;
const REJECT: u16 = 2;
pub(super) const SKIP: u16 = 3;

/// Read-only access to live ordinary relations; shadow/Template host edges do
/// not participate. Implementations meter their actual storage accesses and
/// preserve typed identity, cancellation and admission failures. `step` admits
/// each algorithm transition independently of the relation's storage work.
pub(super) trait OrdinaryTree {
    type Error;

    fn step(&self) -> Result<(), Self::Error>;
    fn parent(&self, node: NodeId) -> Result<Option<NodeId>, Self::Error>;
    fn first_child(&self, node: NodeId) -> Result<Option<NodeId>, Self::Error>;
    fn last_child(&self, node: NodeId) -> Result<Option<NodeId>, Self::Error>;
    fn previous_sibling(&self, node: NodeId) -> Result<Option<NodeId>, Self::Error>;
    fn next_sibling(&self, node: NodeId) -> Result<Option<NodeId>, Self::Error>;
}

#[derive(Debug)]
pub(super) enum Progress {
    Filter(Filter),
    Match(NodeId),
    None,
}

/// Affine continuation at the exact filtering clause reached by the algorithm.
/// The owner performs active-filter checking before mask testing and supplies
/// the resulting callback (or mask/default) value when resuming.
#[derive(Debug)]
pub(super) struct Filter {
    root: NodeId,
    node: NodeId,
    clause: Clause,
}

#[derive(Clone, Copy, Debug)]
enum Order {
    Forward,
    Backward,
}

impl Order {
    fn child<T: OrdinaryTree>(self, tree: &T, node: NodeId) -> Result<Option<NodeId>, T::Error> {
        match self {
            Self::Forward => tree.first_child(node),
            Self::Backward => tree.last_child(node),
        }
    }

    fn sibling<T: OrdinaryTree>(self, tree: &T, node: NodeId) -> Result<Option<NodeId>, T::Error> {
        match self {
            Self::Forward => tree.next_sibling(node),
            Self::Backward => tree.previous_sibling(node),
        }
    }
}

#[derive(Debug)]
enum Clause {
    Parent,
    Child(Order),
    SiblingCandidate(Order),
    SiblingParent(Order),
    PreviousDescendant,
    PreviousParent,
    Next,
}

impl Filter {
    pub(super) fn node(&self) -> NodeId {
        self.node
    }

    pub(super) fn resume<T: OrdinaryTree>(
        self,
        tree: &T,
        live_current: NodeId,
        result: u16,
    ) -> Result<Progress, T::Error> {
        tree.step()?;
        let Self { root, node, clause } = self;
        match clause {
            Clause::Parent => {
                if result == ACCEPT {
                    Ok(Progress::Match(node))
                } else {
                    parent(tree, root, node)
                }
            }
            Clause::Child(order) => {
                if result == ACCEPT {
                    return Ok(Progress::Match(node));
                }
                if result == SKIP
                    && let Some(child) = order.child(tree, node)?
                {
                    return Ok(filter(root, child, Clause::Child(order)));
                }
                child_sibling(tree, root, live_current, node, order)
            }
            Clause::SiblingCandidate(order) => {
                if result == ACCEPT {
                    return Ok(Progress::Match(node));
                }
                // DOM explicitly obtains the child before testing REJECT.
                let child = order.child(tree, node)?;
                let sibling = if result == REJECT || child.is_none() {
                    order.sibling(tree, node)?
                } else {
                    child
                };
                sibling_candidate(tree, root, node, sibling, order)
            }
            Clause::SiblingParent(order) => {
                if result == ACCEPT {
                    Ok(Progress::None)
                } else {
                    sibling(tree, root, node, order)
                }
            }
            Clause::PreviousDescendant => {
                if result != REJECT
                    && let Some(child) = tree.last_child(node)?
                {
                    return Ok(filter(root, child, Clause::PreviousDescendant));
                }
                if result == ACCEPT {
                    return Ok(Progress::Match(node));
                }
                previous_sibling(tree, root, node)
            }
            Clause::PreviousParent => {
                if result == ACCEPT {
                    Ok(Progress::Match(node))
                } else {
                    previous(tree, root, node)
                }
            }
            Clause::Next => {
                if result == ACCEPT {
                    Ok(Progress::Match(node))
                } else {
                    next(tree, root, node, result)
                }
            }
        }
    }
}

pub(super) fn start<T: OrdinaryTree>(
    tree: &T,
    root: NodeId,
    current: NodeId,
    direction: DomTreeWalkerDirection,
) -> Result<Progress, T::Error> {
    match direction {
        DomTreeWalkerDirection::Parent => parent(tree, root, current),
        DomTreeWalkerDirection::FirstChild | DomTreeWalkerDirection::LastChild => {
            tree.step()?;
            let order = if matches!(direction, DomTreeWalkerDirection::FirstChild) {
                Order::Forward
            } else {
                Order::Backward
            };
            Ok(match order.child(tree, current)? {
                Some(child) => filter(root, child, Clause::Child(order)),
                None => Progress::None,
            })
        }
        DomTreeWalkerDirection::NextSibling | DomTreeWalkerDirection::PreviousSibling => {
            let order = if matches!(direction, DomTreeWalkerDirection::NextSibling) {
                Order::Forward
            } else {
                Order::Backward
            };
            if current == root {
                tree.step()?;
                Ok(Progress::None)
            } else {
                sibling(tree, root, current, order)
            }
        }
        DomTreeWalkerDirection::PreviousNode => previous(tree, root, current),
        DomTreeWalkerDirection::NextNode => next(tree, root, current, ACCEPT),
    }
}

fn filter(root: NodeId, node: NodeId, clause: Clause) -> Progress {
    Progress::Filter(Filter { root, node, clause })
}

fn parent<T: OrdinaryTree>(tree: &T, root: NodeId, node: NodeId) -> Result<Progress, T::Error> {
    tree.step()?;
    if node == root {
        return Ok(Progress::None);
    }
    Ok(match tree.parent(node)? {
        Some(parent) => filter(root, parent, Clause::Parent),
        None => Progress::None,
    })
}

fn child_sibling<T: OrdinaryTree>(
    tree: &T,
    root: NodeId,
    live_current: NodeId,
    mut node: NodeId,
    order: Order,
) -> Result<Progress, T::Error> {
    loop {
        tree.step()?;
        if let Some(sibling) = order.sibling(tree, node)? {
            return Ok(filter(root, sibling, Clause::Child(order)));
        }
        let Some(parent) = tree.parent(node)? else {
            return Ok(Progress::None);
        };
        if parent == root || parent == live_current {
            return Ok(Progress::None);
        }
        node = parent;
    }
}

fn sibling<T: OrdinaryTree>(
    tree: &T,
    root: NodeId,
    node: NodeId,
    order: Order,
) -> Result<Progress, T::Error> {
    tree.step()?;
    let candidate = order.sibling(tree, node)?;
    sibling_candidate(tree, root, node, candidate, order)
}

fn sibling_candidate<T: OrdinaryTree>(
    tree: &T,
    root: NodeId,
    node: NodeId,
    candidate: Option<NodeId>,
    order: Order,
) -> Result<Progress, T::Error> {
    tree.step()?;
    if let Some(sibling) = candidate {
        return Ok(filter(root, sibling, Clause::SiblingCandidate(order)));
    }
    let Some(parent) = tree.parent(node)? else {
        return Ok(Progress::None);
    };
    if parent == root {
        return Ok(Progress::None);
    }
    Ok(filter(root, parent, Clause::SiblingParent(order)))
}

fn previous<T: OrdinaryTree>(tree: &T, root: NodeId, node: NodeId) -> Result<Progress, T::Error> {
    tree.step()?;
    if node == root {
        Ok(Progress::None)
    } else {
        previous_sibling(tree, root, node)
    }
}

fn previous_sibling<T: OrdinaryTree>(
    tree: &T,
    root: NodeId,
    node: NodeId,
) -> Result<Progress, T::Error> {
    tree.step()?;
    if let Some(sibling) = tree.previous_sibling(node)? {
        return Ok(filter(root, sibling, Clause::PreviousDescendant));
    }
    // A previous-descendant filter can move `node` to root. The root check is
    // reached after the sibling read, exactly as in the pinned inner loop.
    if node == root {
        return Ok(Progress::None);
    }
    Ok(match tree.parent(node)? {
        Some(parent) => filter(root, parent, Clause::PreviousParent),
        None => Progress::None,
    })
}

fn next<T: OrdinaryTree>(
    tree: &T,
    root: NodeId,
    node: NodeId,
    result: u16,
) -> Result<Progress, T::Error> {
    tree.step()?;
    if result != REJECT
        && let Some(child) = tree.first_child(node)?
    {
        return Ok(filter(root, child, Clause::Next));
    }
    let mut temporary = node;
    loop {
        tree.step()?;
        if temporary == root {
            return Ok(Progress::None);
        }
        if let Some(sibling) = tree.next_sibling(temporary)? {
            return Ok(filter(root, sibling, Clause::Next));
        }
        let Some(parent) = tree.parent(temporary)? else {
            return Ok(Progress::None);
        };
        temporary = parent;
    }
}

#[cfg(test)]
mod tests;
