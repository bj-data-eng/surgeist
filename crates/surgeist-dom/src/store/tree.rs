use super::*;

impl Draft {
    pub(super) fn adopt(
        &mut self,
        node: Key,
        document: Key,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        // Removal precedes the document test, including same-document adoption.
        // Only generic no-namespace Elements exist in this capability surface;
        // HTML family, registry-object and hosted-tree hooks are inapplicable.
        self.remove(node);
        if self.node(node)?.document == document {
            return Ok(());
        }
        let mut pending = Vec::new();
        reserve(&mut pending, 1)?;
        pending.push(node);
        while let Some(node) = pending.pop() {
            check_cancel(cancelled)?;
            let children = &self.node(node)?.children;
            reserve(&mut pending, children.len())?;
            pending.extend(children.iter().rev().copied());
            self.node_mut(node).document = document;
            // Attributes are value records owned by this Element, not Attr
            // nodes. Their node-document is consequently this same relation.
        }
        Ok(())
    }

    fn next_sibling(&self, node: Key) -> Option<Key> {
        let parent = self.node(node).expect("staged node").parent?;
        let children = &self.node(parent).expect("staged parent").children;
        let position = children
            .iter()
            .position(|child| *child == node)
            .expect("staged child");
        children.get(position + 1).copied()
    }

    fn ensure_pre_insert(
        &self,
        node: Key,
        parent: Key,
        child: Option<Key>,
        exclude: Option<Key>,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(
            self.node(parent)?.kind,
            DomNodeKind::Document(_) | DomNodeKind::DocumentFragment | DomNodeKind::Element(_)
        ) {
            return Err(DomError::HierarchyRequest);
        }
        let mut ancestor = Some(parent);
        while let Some(key) = ancestor {
            check_cancel(cancelled)?;
            if key == node {
                return Err(DomError::HierarchyRequest);
            }
            ancestor = self.node(key)?.parent;
        }
        if child
            .is_some_and(|child| self.node(child).expect("resolved child").parent != Some(parent))
        {
            return Err(DomError::NotFound);
        }
        let kind = &self.node(node)?.kind;
        if matches!(kind, DomNodeKind::Document(_)) {
            return Err(DomError::HierarchyRequest);
        }
        if !matches!(self.node(parent)?.kind, DomNodeKind::Document(_)) {
            return if matches!(kind, DomNodeKind::DocumentType(_)) {
                Err(DomError::HierarchyRequest)
            } else {
                Ok(())
            };
        }
        match kind {
            DomNodeKind::Text(_) => return Err(DomError::HierarchyRequest),
            DomNodeKind::Comment(_) | DomNodeKind::ProcessingInstruction(_) => return Ok(()),
            DomNodeKind::DocumentFragment => {
                let mut elements = 0;
                for child in &self.node(node)?.children {
                    check_cancel(cancelled)?;
                    match self.node(*child)?.kind {
                        DomNodeKind::Text(_) => return Err(DomError::HierarchyRequest),
                        DomNodeKind::Element(_) => elements += 1,
                        _ => {}
                    }
                    if elements > 1 {
                        return Err(DomError::HierarchyRequest);
                    }
                }
                if elements == 0 {
                    return Ok(());
                }
            }
            _ => {}
        }
        let children = &self.node(parent)?.children;
        let reference = child.map(|child| {
            children
                .iter()
                .position(|key| *key == child)
                .expect("validated reference")
        });
        for (position, candidate) in children.iter().copied().enumerate() {
            check_cancel(cancelled)?;
            let candidate_kind = &self.node(candidate)?.kind;
            let invalid = if matches!(kind, DomNodeKind::DocumentType(_)) {
                (matches!(candidate_kind, DomNodeKind::DocumentType(_))
                    && Some(candidate) != exclude)
                    || (matches!(candidate_kind, DomNodeKind::Element(_))
                        && reference
                            .map_or(Some(candidate) != exclude, |reference| position < reference))
            } else {
                (matches!(candidate_kind, DomNodeKind::Element(_)) && Some(candidate) != exclude)
                    || (matches!(candidate_kind, DomNodeKind::DocumentType(_))
                        && reference.is_some_and(|reference| {
                            position > reference
                                || position == reference && Some(candidate) != exclude
                        }))
            };
            if invalid {
                return Err(DomError::HierarchyRequest);
            }
        }
        Ok(())
    }

    pub(super) fn pre_insert(
        &mut self,
        node: Key,
        parent: Key,
        mut before: Option<Key>,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(node, parent, before, None, cancelled)?;
        if before == Some(node) {
            before = self.next_sibling(node);
        }
        self.insert_validated(node, parent, before, cancelled)
    }

    pub(super) fn replace_child(
        &mut self,
        node: Key,
        parent: Key,
        child: Key,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(node, parent, Some(child), Some(child), cancelled)?;
        let mut before = self.next_sibling(child);
        if before == Some(node) {
            before = self.next_sibling(node);
        }
        // Unlike insert's per-child adoption, replace explicitly adopts its
        // supplied node first, including a DocumentFragment and its subtree.
        self.adopt(node, self.node(parent)?.document, cancelled)?;
        self.remove(child);
        self.insert_validated(node, parent, before, cancelled)
    }

    fn insert_validated(
        &mut self,
        node: Key,
        parent: Key,
        before: Option<Key>,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let fragment = matches!(self.node(node)?.kind, DomNodeKind::DocumentFragment);
        let moving = if fragment {
            copy(&self.node(node)?.children)?
        } else {
            let mut moving = Vec::new();
            reserve(&mut moving, 1)?;
            moving.push(node);
            moving
        };
        if moving.is_empty() {
            return Ok(());
        }
        reserve(&mut self.node_mut(parent).children, moving.len())?;
        // Fragment removal completes before any insertion; the generic XML
        // surface has no applicable language children-changed/post-connection
        // effects. All relation changes still use the same ordered primitives.
        if fragment {
            for child in &moving {
                check_cancel(cancelled)?;
                self.remove(*child);
            }
        }
        let document = self.node(parent)?.document;
        for child in moving {
            check_cancel(cancelled)?;
            self.adopt(child, document, cancelled)?;
            let target = self.node_mut(parent);
            let index = before.map_or(target.children.len(), |reference| {
                target
                    .children
                    .iter()
                    .position(|child| *child == reference)
                    .expect("retained reference")
            });
            target.children.insert(index, child);
            self.node_mut(child).parent = Some(parent);
        }
        Ok(())
    }
}
