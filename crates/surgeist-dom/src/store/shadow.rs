use super::transaction::KindRead;
use super::tree::AfterRemovingChildren;
use super::*;

impl Draft {
    /// Transfers a qualified retained attachment rather than replaying native
    /// creation or descendant insertion. The populated tree's slot relation is
    /// recomputed against the actual host's current children at this join.
    pub(super) fn attach_parsed_shadow(
        &mut self,
        host: Key,
        root: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancelled)?;
        let host_node = self.node(host)?;
        if !matches!(host_node.kind(), KindRead::Element(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if host_node.shadow_root().is_some() {
            return Err(DomError::NotSupported);
        }
        let root_node = self.node(root)?;
        if !matches!(root_node.kind(), KindRead::ShadowRoot(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if root_node.parent().is_some() || root_node.host().is_some() {
            return Err(DomError::HierarchyRequest);
        }
        let mut current = Some(host);
        while let Some(key) = current {
            self.source_step(cancelled)?;
            if key == root {
                return Err(DomError::HierarchyRequest);
            }
            current = self.host_including_parent(key)?;
        }
        let mut relation = self.host_mut(root, cancelled)?;
        *relation = Some(host);
        relation.commit(cancelled)?;
        let mut relation = self.shadow_root_mut(host, cancelled)?;
        *relation = Some(root);
        relation.commit(cancelled)?;
        // Attachment is not an ordinary insertion: no Style/Details/control
        // insertion hooks or adoption run for the retained descendant state.
        self.assign_tree(root, limits, cancelled)
    }
    pub(super) fn attach_shadow(
        &mut self,
        host: Key,
        init: crate::DomShadowRootInit,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        let (document, existing) = self.qualify_shadow_host(host, cancelled)?;
        if existing.is_some() {
            return Err(DomError::NotSupported);
        }
        self.attach_new_shadow(host, document, init, limits, cancelled)
    }

    pub(super) fn attach_shadow_command(
        &mut self,
        host: Key,
        init: crate::DomShadowRootInit,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let (document, existing) = self.qualify_shadow_host(host, cancelled)?;
        let Some(root) = existing else {
            let root = self.attach_new_shadow(host, document, init, limits, cancelled)?;
            return self.register_ticket(root, cancelled);
        };
        let node = self.node(root)?;
        let KindRead::ShadowRoot(state) = node.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        if !state.declarative() || state.mode() != init.properties.mode() {
            return Err(DomError::NotSupported);
        }
        self.replace_all_children(
            root,
            AfterRemovingChildren::ReuseShadowRoot,
            limits,
            cancelled,
        )
    }

    fn qualify_shadow_host(
        &self,
        host: Key,
        cancelled: &AtomicBool,
    ) -> Result<(Key, Option<Key>), DomError> {
        self.source_step(cancelled)?;
        let node = self.node(host)?;
        let KindRead::Element(element) = node.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        let Some(uri) = element.name().namespace_uri() else {
            return Err(DomError::NotSupported);
        };
        if !self.shadow_name_is(uri, b"http://www.w3.org/1999/xhtml", cancelled)? {
            return Err(DomError::NotSupported);
        }
        let name = element.name().local_name();
        // DOM #valid-shadow-host-name owns this intentionally short list. HTML
        // owns custom-name grammar and actual native-family initialization.
        let mut builtin_host = false;
        for builtin in [
            "article",
            "aside",
            "blockquote",
            "body",
            "div",
            "footer",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "main",
            "nav",
            "p",
            "section",
            "span",
        ] {
            if self.shadow_name_is(name, builtin.as_bytes(), cancelled)? {
                builtin_host = true;
                break;
            }
        }
        if !builtin_host
            && surgeist_html::bounded::lexical::validate_custom_element_name_with(name, |_| {
                self.source_step(cancelled)
            })?
            .is_err()
        {
            return Err(DomError::NotSupported);
        }
        // The controlled complete-empty Global and Null domains both retain a
        // no-definition result; binding identity remains independent.
        debug_assert_eq!(
            element.custom_element_definition(),
            crate::DomElementDefinition::Null
        );
        debug_assert!(matches!(
            element.custom_element_state(),
            surgeist_html::elements::native::HtmlCustomElementState::Uncustomized
                | surgeist_html::elements::native::HtmlCustomElementState::Undefined
        ));
        Ok((node.document(), node.shadow_root()))
    }

    fn attach_new_shadow(
        &mut self,
        host: Key,
        document: Key,
        init: crate::DomShadowRootInit,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        let registry = self.document_metadata(document)?.custom_element_registry();
        let mut state = crate::DomShadowRootState::native(init);
        if !init.null_registry {
            state.select_global_registry(registry);
        }
        let root = self.allocate(NewNode::ShadowRoot(state), document, limits, cancelled)?;
        let mut relation = self.host_mut(root, cancelled)?;
        *relation = Some(host);
        relation.commit(cancelled)?;
        let mut relation = self.shadow_root_mut(host, cancelled)?;
        *relation = Some(root);
        relation.commit(cancelled)?;
        Ok(root)
    }

    fn shadow_name_is(
        &self,
        actual: &DocumentString,
        expected: &[u8],
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        if actual.len() != expected.len() {
            return Ok(false);
        }
        for (actual, expected) in actual.code_units().iter().zip(expected) {
            self.source_step(cancel)?;
            if *actual != u16::from(*expected) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Shadow roots are parentless ordinary roots. For cycle checks only, their
    /// fragment host continues the ancestry chain. Actual Template contents use
    /// this same host concept, separately from shadow connectedness.
    pub(super) fn host_including_parent(&self, key: Key) -> Result<Option<Key>, DomError> {
        let node = self.node(key)?;
        Ok(node.parent().or(node.host()))
    }
    pub(super) fn shadow_including_parent(&self, key: Key) -> Result<Option<Key>, DomError> {
        let node = self.node(key)?;
        Ok(node.parent().or_else(|| {
            matches!(node.kind(), KindRead::ShadowRoot(_))
                .then_some(node.host())
                .flatten()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(dom: &mut Dom) -> NodeId {
        let mut batch = DomBatch::new();
        let doc = batch.create_document();
        let host = batch.create_element_ns(
            &doc,
            surgeist_html::values::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            "div",
        );
        dom.apply(batch, &AtomicBool::new(false))
            .unwrap()
            .created(&host)
            .unwrap()
            .unwrap()
    }
    #[test]
    fn cancellation_after_preparing_shadow_links_and_ids_preserves_published_state() {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                256,
                16,
                0,
                0,
                8,
                8192,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let host = host(&mut dom);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.attach_shadow(
            host,
            crate::DomShadowRootInit::new(surgeist_html::construction::HtmlShadowRootMode::Closed),
        );
        let prepared = dom.prepare(batch, &AtomicBool::new(false)).unwrap();
        assert!(
            prepared
                .state
                .node(dom.owner, host)
                .unwrap()
                .shadow_root()
                .is_some()
        );
        assert_eq!(
            dom.publish(prepared, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    #[test]
    fn exhausted_revision_rejects_shadow_publication_without_consuming_an_identity() {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                256,
                16,
                0,
                0,
                8,
                8192,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let host = host(&mut dom);
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.attach_shadow(
            host,
            crate::DomShadowRootInit::new(surgeist_html::construction::HtmlShadowRootMode::Open),
        );
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
}
