use super::super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Pointer<N> {
    pub node: N,
    pub before: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Direction {
    Next,
    Previous,
}

pub(super) trait OrdinaryTree<N: Copy + Eq> {
    fn position(&self, node: N) -> Result<Option<(N, usize)>, DomError>;
    fn child(&self, node: N, ordinal: usize) -> Result<Option<N>, DomError>;
    fn last_child(&self, node: N) -> Result<Option<N>, DomError>;
}

pub(super) fn following<N: Copy + Eq>(
    tree: &impl OrdinaryTree<N>,
    root: N,
    mut node: N,
) -> Result<Option<N>, DomError> {
    while node != root {
        let (parent, ordinal) = tree.position(node)?.ok_or(DomError::HierarchyRequest)?;
        let next = ordinal
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if let Some(sibling) = tree.child(parent, next)? {
            return Ok(Some(sibling));
        }
        node = parent;
    }
    Ok(None)
}

pub(super) fn preceding<N: Copy + Eq>(
    tree: &impl OrdinaryTree<N>,
    root: N,
    node: N,
) -> Result<Option<N>, DomError> {
    if node == root {
        return Ok(None);
    }
    let (parent, ordinal) = tree.position(node)?.ok_or(DomError::HierarchyRequest)?;
    let Some(previous) = ordinal.checked_sub(1) else {
        return Ok(Some(parent));
    };
    let mut node = tree
        .child(parent, previous)?
        .ok_or(DomError::HierarchyRequest)?;
    while let Some(child) = tree.last_child(node)? {
        node = child;
    }
    Ok(Some(node))
}

pub(super) fn advance<N: Copy + Eq>(
    tree: &impl OrdinaryTree<N>,
    root: N,
    pointer: Pointer<N>,
    direction: Direction,
) -> Result<Option<Pointer<N>>, DomError> {
    let node = match (direction, pointer.before) {
        (Direction::Next, true) | (Direction::Previous, false) => Some(pointer.node),
        (Direction::Next, false) => match tree.child(pointer.node, 0)? {
            Some(child) => Some(child),
            None => following(tree, root, pointer.node)?,
        },
        (Direction::Previous, true) => preceding(tree, root, pointer.node)?,
    };
    Ok(node.map(|node| Pointer {
        node,
        before: matches!(direction, Direction::Previous),
    }))
}

/// The caller has already selected roots that are strict ordinary ancestors of
/// the removed node. Neither shadow nor Template host edges enter this walk.
pub(super) fn adjust<N: Copy + Eq>(
    tree: &impl OrdinaryTree<N>,
    root: N,
    removed: N,
    pointer: Pointer<N>,
) -> Result<Pointer<N>, DomError> {
    let mut ancestor = pointer.node;
    loop {
        if ancestor == removed {
            break;
        }
        if ancestor == root {
            return Ok(pointer);
        }
        ancestor = tree
            .position(ancestor)?
            .ok_or(DomError::HierarchyRequest)?
            .0;
    }
    if pointer.before
        && let Some(node) = following(tree, root, removed)?
    {
        return Ok(Pointer { node, before: true });
    }
    let node = preceding(tree, root, removed)?.ok_or(DomError::HierarchyRequest)?;
    Ok(Pointer {
        node,
        before: false,
    })
}
