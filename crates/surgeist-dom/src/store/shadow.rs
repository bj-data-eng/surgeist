use super::*;

impl Draft {
    pub(super) fn attach_shadow(
        &mut self,
        host: Key,
        init: crate::DomShadowRootInit,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancelled)?;
        let node = self.node(host)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Err(DomError::WrongNodeKind);
        };
        if !element.name().namespace().uri().is_some_and(|uri| {
            uri.code_units()
                .iter()
                .copied()
                .eq("http://www.w3.org/1999/xhtml".encode_utf16())
        }) {
            return Err(DomError::NotSupported);
        }
        let name = element.name().local_name();
        // DOM #valid-shadow-host-name owns this intentionally short list. HTML
        // owns custom-name grammar and actual native-family initialization.
        let mut valid = false;
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
            check_cancel(cancelled)?;
            valid |= name.code_units().iter().copied().eq(builtin.encode_utf16());
        }
        let custom = surgeist_html::bounded::validate_custom_element_name_with(name, |_| {
            check_cancel(cancelled)
        })?
        .is_ok();
        if !valid && !custom {
            return Err(DomError::NotSupported);
        }
        // The real admitted registry and definition bindings are known null.
        // There is no registry object/definition lookup or custom execution
        // capability to guess from a saved-is attribute or authored spelling.
        debug_assert_eq!(
            element.custom_element_registry(),
            crate::DomRegistryBinding::Null
        );
        debug_assert_eq!(
            element.custom_element_definition(),
            crate::DomElementDefinition::Null
        );
        debug_assert!(matches!(
            element.custom_element_state(),
            crate::HtmlCustomElementState::Uncustomized | crate::HtmlCustomElementState::Undefined
        ));
        if node.shadow_root.is_some() {
            // Every admitted root is non-declarative. Declarative reuse has no
            // native caller toggle and remains an installation capability.
            return Err(DomError::NotSupported);
        }
        let document = node.document;
        let root = self.allocate(
            DomNodeKind::ShadowRoot(crate::DomShadowRootState::native(init)),
            document,
            limits,
        )?;
        self.node_mut(root).host = Some(host);
        self.node_mut(host).shadow_root = Some(root);
        Ok(root)
    }

    /// Shadow roots are parentless ordinary roots. For cycle checks only, their
    /// fragment host continues the ancestry chain. Future template hosts must
    /// join this same source-defined relation rather than ordinary parent links.
    pub(super) fn host_including_parent(&self, key: Key) -> Result<Option<Key>, DomError> {
        let node = self.node(key)?;
        Ok(node.parent.or(node.host))
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
            crate::Namespace::from_uri("http://www.w3.org/1999/xhtml"),
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
        let mut dom = Dom::new(DomLimits::new(8, 256, 16, 8, 8192).unwrap()).unwrap();
        let host = host(&mut dom);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.attach_shadow(
            host,
            crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Closed),
        );
        let prepared = dom.prepare(batch, &AtomicBool::new(false)).unwrap();
        assert!(
            prepared
                .state
                .node(dom.owner, host)
                .unwrap()
                .shadow_root
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
        let mut dom = Dom::new(DomLimits::new(8, 256, 16, 8, 8192).unwrap()).unwrap();
        let host = host(&mut dom);
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.attach_shadow(
            host,
            crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Open),
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
