//! Source-owned Template relations in the canonical DOM identity phase.
use crate::{DocumentId, DomDocument, DomNode, NodeId};

/// Whether HTML's template-owner algorithm actually created this Document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomTemplateDocumentRole {
    Ordinary,
    InertTemplate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TemplateDocument<K> {
    Ordinary { inert: Option<K> },
    InertTemplate,
}
impl<K: Copy> TemplateDocument<K> {
    pub(crate) fn map<T>(self, map: impl FnOnce(K) -> T) -> TemplateDocument<T> {
        match self {
            Self::Ordinary { inert } => TemplateDocument::Ordinary {
                inert: inert.map(map),
            },
            Self::InertTemplate => TemplateDocument::InertTemplate,
        }
    }
    pub(crate) fn role(self) -> DomTemplateDocumentRole {
        match self {
            Self::Ordinary { .. } => DomTemplateDocumentRole::Ordinary,
            Self::InertTemplate => DomTemplateDocumentRole::InertTemplate,
        }
    }
    pub(crate) fn inert(self) -> Option<K> {
        match self {
            Self::Ordinary { inert } => inert,
            Self::InertTemplate => None,
        }
    }
}

/// Checked native Template view, with real separately owned contents.
///
/// ```
/// use std::sync::atomic::AtomicBool;
/// use surgeist_dom::{Dom, DomBatch, DomLimits, Namespace};
/// let cancel = AtomicBool::new(false);
/// let mut dom = Dom::new(DomLimits::new(16, 4096, 32, 0, 0, 8, 16384)?)?;
/// let mut batch = DomBatch::new();
/// let document = batch.create_document();
/// let template = batch.create_element_ns(&document, Namespace::html(), "template");
/// let commit = dom.apply(batch, &cancel)?;
/// let old = dom.snapshot();
/// let template = old.node(commit.created(&template)?.unwrap())?.as_template().unwrap();
/// let contents = template.contents();
/// let owner = old.node(contents)?.owner_document().unwrap();
/// let mut batch = DomBatch::new();
/// let text = batch.create_text(owner, "body");
/// batch.insert_before(&text, contents, None);
/// dom.apply(batch, &cancel)?;
/// assert!(old.node(contents)?.children().is_empty());
/// assert_eq!(dom.snapshot().node(contents)?.children().len(), 1);
/// # Ok::<(), surgeist_dom::DomError>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct DomTemplate<'a> {
    node: DomNode<'a>,
}
impl DomTemplate<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn contents(self) -> NodeId {
        self.node
            .node
            .template_contents
            .expect("initialized native Template")
    }
    /// Known-null native references. Parser patching/installation is a different
    /// creation cause, which this native surface does not expose.
    pub fn insertion_state(self) -> surgeist_html::HtmlTemplateFreshInsertionState {
        self.node
            .as_element()
            .expect("Template element")
            .template_insertion_state()
            .expect("actual native Template state")
    }
}
impl<'a> DomNode<'a> {
    pub fn as_template(self) -> Option<DomTemplate<'a>> {
        self.as_element()?.template_insertion_state()?;
        Some(DomTemplate { node: self })
    }
    /// Actual DocumentFragment host, including Template contents and ShadowRoots.
    /// This ownership relation does not imply shadow connectedness.
    pub fn fragment_host(self) -> Option<NodeId> {
        self.node.host
    }
}
impl DomDocument<'_> {
    pub fn template_role(self) -> DomTemplateDocumentRole {
        self.template_document().role()
    }
    pub fn associated_inert_template_document(self) -> Option<DocumentId> {
        self.template_document().inert().map(DocumentId)
    }
    fn template_document(self) -> TemplateDocument<NodeId> {
        self.snapshot
            .node(self.id.node())
            .expect("captured Document")
            .node
            .template_document
            .expect("actual Document role")
    }
}
