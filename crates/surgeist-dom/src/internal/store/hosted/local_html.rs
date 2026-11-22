//! Atomic canonical application of the bounded local HTML construction profile.

use super::*;
use surgeist_html_parser::live::{HtmlLiveAttributes, HtmlLiveElement, HtmlLiveTree};

/// Original cumulative native work admission for one hosted local parser.
pub struct DomHostedLocalHtmlEdits {
    document: DocumentId,
    global: DomHostedGlobalId,
    work: Arc<Work>,
}

impl DomHostedSource {
    /// Bind the native edit budget to the current loading Document and Global.
    pub fn begin_local_html_edits(
        &self,
        dom: &Dom,
        document: DocumentId,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedLocalHtmlEdits, DomHostedError> {
        self.qualify_hosted_local_html_parser(dom, document, global, cancel)?;
        Ok(DomHostedLocalHtmlEdits {
            document,
            global,
            work: self.work(cancel)?,
        })
    }

    /// Borrow the qualified canonical tree while the original parser advances.
    /// Every atomic edit shares the original native work admission, including
    /// copies of existing canonical data. Borrowing again cannot reset it.
    pub fn local_html_tree<'a>(
        &self,
        dom: &'a mut Dom,
        edits: &'a DomHostedLocalHtmlEdits,
        cancel: &'a AtomicBool,
    ) -> Result<impl HtmlLiveTree<Node = NodeId, Error = DomError> + 'a, DomHostedError> {
        self.qualify_hosted_local_html_parser(dom, edits.document, edits.global, cancel)?;
        let limits = crate::internal::source::DomSourceLimits::new(
            self.remaining_bytes()?.min(self.limits().scratch_bytes()),
            dom.limits.units,
            self.limits().work(),
            0,
            0,
            0,
            1,
        );
        Ok(LocalHtmlTree {
            dom,
            edits,
            limits,
            cancel,
        })
    }
}

/// One advance borrows the canonical DOM. The parser retains node identities
/// across advances and the next advance reads the current revision.
struct LocalHtmlTree<'a> {
    dom: &'a mut Dom,
    edits: &'a DomHostedLocalHtmlEdits,
    limits: crate::internal::source::DomSourceLimits,
    cancel: &'a AtomicBool,
}

impl LocalHtmlTree<'_> {
    fn apply(&mut self, batch: DomBatch) -> Result<DomCommit, DomError> {
        let result = (|| {
            let (mut draft, _) = Draft::new_source_root_with_work(
                self.dom,
                batch.owner,
                source::initial::CommandStorage {
                    commands: &batch.commands,
                    capacity: batch.commands.capacity(),
                },
                self.limits,
                source::initial::Replay {
                    work: self.edits.work.spent.clone(),
                    identity: self.edits.work.spent_id,
                    names: None,
                },
                self.cancel,
            )?;
            // This is an atomic batch. A reached asynchronous hook is refused
            // by the ordinary edit path before any publication.
            draft.phase_map = None;
            for command in batch.commands {
                draft.source_step(self.cancel)?;
                draft.apply(&command, self.dom.limits, self.cancel)?;
            }
            let prepared = draft.finish(self.dom, self.cancel)?;
            self.dom.publish(prepared, self.cancel)
        })();
        self.dom.custody_registry.sweep_blocks();
        result
    }

    fn qualify_parent(&self, parent: NodeId) -> Result<(), DomError> {
        let snapshot = self.dom.snapshot();
        let node = snapshot.node(parent)?;
        if (parent == self.edits.document.node() && matches!(node.kind(), DomNodeKind::Document(_)))
            || (node.owner_document() == Some(self.edits.document)
                && matches!(node.kind(), DomNodeKind::Element(_)))
        {
            Ok(())
        } else {
            Err(DomError::WrongDocument)
        }
    }

    fn adjacent_text(&self, parent: NodeId) -> Result<Option<(NodeId, usize)>, DomError> {
        let snapshot = self.dom.snapshot();
        let parent = snapshot.node(parent)?;
        let Some(&last) = parent.children().last() else {
            return Ok(None);
        };
        Ok(match snapshot.node(last)?.kind() {
            DomNodeKind::Text(data) => Some((last, data.code_units().len())),
            _ => None,
        })
    }
}

impl HtmlLiveTree for LocalHtmlTree<'_> {
    type Node = NodeId;
    type Error = DomError;

    fn insert_doctype(&mut self, document: NodeId) -> Result<(), DomError> {
        if document != self.edits.document.node() {
            return Err(DomError::WrongDocument);
        }
        self.qualify_parent(document)?;
        let mut batch = DomBatch::new();
        let doctype = batch.create_document_type(document, "html", "", "");
        batch.insert_before(&doctype, document, None);
        self.apply(batch)?;
        Ok(())
    }

    fn insert_element(
        &mut self,
        parent: NodeId,
        name: HtmlLiveElement,
        attributes: HtmlLiveAttributes<'_>,
    ) -> Result<NodeId, DomError> {
        self.qualify_parent(parent)?;
        let mut batch = DomBatch::new();
        let element = batch.create_element(self.edits.document.node(), name.as_str());
        for (name, value) in attributes.iter() {
            batch.set_attribute(&element, name.as_str(), value.clone());
        }
        batch.insert_before(&element, parent, None);
        let commit = self.apply(batch)?;
        // The original batch creates this exact output and cannot retire it.
        Ok(commit
            .created(&element)
            .expect("the creation ticket belongs to this batch")
            .expect("the created element survives this batch"))
    }

    fn insert_comment(&mut self, parent: NodeId, text: &DocumentString) -> Result<(), DomError> {
        self.qualify_parent(parent)?;
        let mut batch = DomBatch::new();
        let node = batch.create_comment(self.edits.document.node(), text.clone());
        batch.insert_before(&node, parent, None);
        self.apply(batch)?;
        Ok(())
    }

    fn insert_text(&mut self, parent: NodeId, text: &DocumentString) -> Result<(), DomError> {
        self.qualify_parent(parent)?;
        if parent == self.edits.document.node() || text.code_units().is_empty() {
            return Ok(());
        }
        let mut batch = DomBatch::new();
        if let Some((node, end)) = self.adjacent_text(parent)? {
            batch.replace_data(node, end, 0, text.clone());
        } else {
            let node = batch.create_text(self.edits.document.node(), text.clone());
            batch.insert_before(&node, parent, None);
        }
        self.apply(batch)?;
        Ok(())
    }
}
