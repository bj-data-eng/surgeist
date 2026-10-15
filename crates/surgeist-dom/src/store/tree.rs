use super::*;

impl Draft {
    pub(super) fn adopt(
        &mut self,
        node: Key,
        document: Key,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        // Removal precedes the document test, including same-document adoption.
        // Supported common profiles retain nonce/common state during adoption;
        // registry objects, hosted trees and special families remain unexposed.
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
        // Fragment removal completes before any insertion. Supported common
        // profiles have no children-changed family effect; each inserted
        // subtree runs its applicable common insertion steps below.
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
            self.common_insertion_steps(child, &mut || check_cancel(cancelled))?;
        }
        Ok(())
    }

    fn common_insertion_steps(
        &self,
        root: Key,
        admit_step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        use surgeist_html::{
            HtmlBrowsingContextPresence, HtmlHeaderDeliveredCsp, HtmlNonceConnection,
            HtmlNonceInsertionEffect,
        };
        let mut pending = Vec::new();
        reserve(&mut pending, 1)?;
        pending.push(root);
        while let Some(key) = pending.pop() {
            admit_step()?;
            let node = self.node(key)?;
            reserve(&mut pending, node.children.len())?;
            pending.extend(node.children.iter().rev().copied());
            let DomNodeKind::Element(element) = &node.kind else {
                continue;
            };
            let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
                unreachable!("canonical node document")
            };
            // This is an established native environment fact, independent of
            // class or connectedness. No CSP observation is invented.
            let connection = match document.native_environment().browsing_context() {
                HtmlBrowsingContextPresence::Absent => {
                    HtmlNonceConnection::NotBrowsingContextConnected
                }
                HtmlBrowsingContextPresence::Unavailable | HtmlBrowsingContextPresence::Present => {
                    HtmlNonceConnection::Unavailable
                }
            };
            let mut content = None;
            for attribute in element.attributes().iter() {
                admit_step()?;
                if attribute.name().namespace().uri().is_none()
                    && attribute.name().local_name().code_units() == [0x6e, 0x6f, 0x6e, 0x63, 0x65]
                {
                    content = Some(attribute.value());
                    break;
                }
            }
            match surgeist_html::nonce_insertion_effect(
                element.name(),
                content,
                connection,
                HtmlHeaderDeliveredCsp::Unavailable,
            ) {
                HtmlNonceInsertionEffect::KeepContent => {}
                HtmlNonceInsertionEffect::Unavailable(missing) => {
                    return Err(DomError::NonceEnvironmentUnavailable(missing));
                }
                HtmlNonceInsertionEffect::HideContentPreservingSlot => {
                    unreachable!("native null browsing context cannot hide nonce content")
                }
            }
            // Closed HTML popovers are hidden, so their removal/attribute hide
            // steps return without changing state. Never-navigable documents
            // are not fully active, discharging the autofocus insertion guard.
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_during_common_descendant_insertion_keeps_publication_unchanged() {
        let mut dom = Dom::new(DomLimits::new(16, 2048, 32, 8, 8192).unwrap()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            "div",
        );
        let child = batch.create_element_ns(
            &document,
            crate::Namespace::from_uri("http://www.w3.org/2000/svg"),
            "g",
        );
        batch.set_attribute(&child, "nonce", "child");
        batch.insert_before(&child, &parent, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let parent = commit.created(&parent).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity)).unwrap();
        let key = draft.resolve(parent.into()).unwrap();
        draft
            .set_nonce(key, "unpublished".into(), dom.limits(), &cancel)
            .unwrap();
        let mut visited = 0;
        assert_eq!(
            draft.common_insertion_steps(key, &mut || {
                visited += 1;
                if visited == 2 {
                    cancel.store(true, Ordering::Release);
                }
                check_cancel(&cancel)
            }),
            Err(DomError::Cancelled)
        );
        assert_eq!(visited, 2);
        drop(draft);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
}
