//! DOM3071 createHTMLDocument: one canonical transaction, including skeleton.
use super::*;

impl Draft {
    pub(super) fn create_html_document(
        &mut self,
        associated_document: Key,
        title: Option<DocumentString>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancelled)?;
        let DomNodeKind::Document(source) = &self.node(associated_document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        // Only origin is inherited. Native defaults establish all other facts;
        // neither a browsing context nor source metadata is implicitly copied.
        let metadata = DomDocumentMetadata::native_html(source.origin().clone());
        let document = Key::Created(self.created.len());
        let document = self.allocate(DomNodeKind::Document(metadata), document, limits)?;
        let doctype = self.allocate(
            DomNodeKind::DocumentType(DomDocumentType::new("html".into(), "".into(), "".into())?),
            document,
            limits,
        )?;
        self.pre_insert(doctype, document, None, limits, cancelled)?;
        let html = self.html_skeleton_element(document, "html", limits, cancelled)?;
        self.pre_insert(html, document, None, limits, cancelled)?;
        let head = self.html_skeleton_element(document, "head", limits, cancelled)?;
        self.pre_insert(head, html, None, limits, cancelled)?;
        if let Some(title) = title {
            let element = self.html_skeleton_element(document, "title", limits, cancelled)?;
            self.pre_insert(element, head, None, limits, cancelled)?;
            let text = self.allocate(DomNodeKind::Text(title), document, limits)?;
            self.pre_insert(text, element, None, limits, cancelled)?;
        }
        let body = self.html_skeleton_element(document, "body", limits, cancelled)?;
        self.pre_insert(body, html, None, limits, cancelled)?;
        Ok(document)
    }
    fn html_skeleton_element(
        &mut self,
        document: Key,
        local_name: &str,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancelled)?;
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            unreachable!("canonical skeleton Document")
        };
        let element = DomElement::native(local_name.into(), metadata, cancelled)?;
        self.allocate(DomNodeKind::Element(element), document, limits)
    }
    pub(super) fn html_element_in_html_document(&self, key: Key) -> Result<bool, DomError> {
        let node = self.node(key)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
            unreachable!("canonical node document")
        };
        Ok(crate::names::in_html_namespace(element.name())
            && document.class() == crate::DomDocumentClass::Html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn owner() -> (Dom, DocumentId) {
        let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let source = batch.create_document();
        let source = dom
            .apply(batch, &AtomicBool::new(false))
            .unwrap()
            .document(&source)
            .unwrap();
        (dom, source)
    }
    #[test]
    fn cancellation_after_complete_html_preparation_publishes_no_skeleton_origin_or_history() {
        let (mut dom, source) = owner();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        let created = batch.create_html_document(source, Some("title".into()));
        let prepared = dom.prepare(batch, &AtomicBool::new(false)).unwrap();
        assert_eq!(prepared.state.nodes, 8);
        let document = prepared.created[created.index].unwrap();
        let node = prepared.state.node(dom.owner, document).unwrap();
        let DomNodeKind::Document(metadata) = &node.kind else {
            panic!("prepared Document")
        };
        assert_eq!(metadata.class(), crate::DomDocumentClass::Html);
        assert_eq!(
            metadata.origin(),
            before.document(source).unwrap().metadata().origin()
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
    fn exhausted_revision_rejects_complete_html_publication_without_consuming_ids() {
        let (mut dom, source) = owner();
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.create_html_document(source, Some("".into()));
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}
