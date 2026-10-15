//! Canonical Base history, native qualified observations and Link history.
use crate::{DocumentId, DomNode, NodeId};
use surgeist_html::{HtmlLinkFlags, HtmlLinkHistory};
use surgeist_html_semantics::{
    HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseUrlRecord, HtmlParseBaseFacts,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomBaseFactsError {
    EnvironmentMismatch,
    ForeignProducer,
    InputsMismatch,
    BasisRevisionMismatch,
    Missing(HtmlBaseMissing),
}
/// Actual native response publication, never a UI/style/task delivery claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomBaseResponse {
    Returned,
    NativeFactsPublished,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Response {
    Returned,
    Pending,
    NativeFactsPublished,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Frozen<K> {
    pub(crate) value: HtmlBaseUrlRecord,
    pub(crate) document: K,
    pub(crate) revision: HtmlBaseFactsRevision,
    pub(crate) response: Response,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BaseState<K> {
    pub(crate) frozen: Option<Frozen<K>>,
}
impl<K> BaseState<K> {
    pub(crate) fn fresh() -> Self {
        Self { frozen: None }
    }
    pub(crate) fn map<T>(&self, map: impl Fn(K) -> T) -> BaseState<T>
    where
        K: Copy,
    {
        BaseState {
            frozen: self.frozen.as_ref().map(|f| Frozen {
                value: f.value.clone(),
                document: map(f.document),
                revision: f.revision.clone(),
                response: f.response,
            }),
        }
    }
    pub(crate) fn units(&self) -> usize {
        self.frozen
            .as_ref()
            .map_or(0, |f| f.value.serialization().len())
    }
}
#[derive(Clone, Debug)]
pub(crate) struct BaseDocument<K> {
    pub(crate) facts: Option<HtmlParseBaseFacts>,
    pub(crate) first: Option<K>,
    pub(crate) render_blocking: Vec<K>,
}
impl<K: PartialEq> PartialEq for BaseDocument<K> {
    fn eq(&self, other: &Self) -> bool {
        self.render_blocking == other.render_blocking
            && self.first == other.first
            && match (&self.facts, &other.facts) {
                (None, None) => true,
                (Some(a), Some(b)) => a.same_owner(b),
                _ => false,
            }
    }
}
impl<K: Eq> Eq for BaseDocument<K> {}
impl<K> BaseDocument<K> {
    pub(crate) fn fresh() -> Self {
        Self {
            facts: None,
            first: None,
            render_blocking: Vec::new(),
        }
    }
    pub(crate) fn map<T>(&self, map: impl Fn(K) -> T) -> Result<BaseDocument<T>, crate::DomError>
    where
        K: Copy,
    {
        // Native Documents have the source-initial empty set; no admitted front
        // enters a render-blocking element. Retained resources need another phase.
        if self.render_blocking.capacity() != 0 {
            return Err(crate::DomError::LinkProcessingUnavailable);
        }
        Ok(BaseDocument {
            facts: self.facts.clone(),
            first: self.first.map(map),
            render_blocking: Vec::new(),
        })
    }
    pub(crate) fn units(&self) -> usize {
        self.facts
            .as_ref()
            .map_or(0, |f| f.formation_usage().utf16_extent())
    }
    pub(crate) fn records(&self) -> usize {
        self.facts
            .as_ref()
            .map_or(0, |f| f.formation_usage().record_capacity())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LinkState<K> {
    pub(crate) flags: HtmlLinkFlags,
    pub(crate) parser_document: Option<K>,
    pub(crate) sheet: Option<surgeist_cssom::CssomSheetId>,
}
impl<K> LinkState<K> {
    pub(crate) fn fresh() -> Self {
        Self {
            flags: HtmlLinkFlags::fresh(),
            parser_document: None,
            sheet: None,
        }
    }
    pub(crate) fn map<T>(&self, map: impl Fn(K) -> T) -> LinkState<T>
    where
        K: Copy,
    {
        LinkState {
            flags: self.flags,
            parser_document: self.parser_document.map(map),
            sheet: self.sheet.clone(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DomBase<'a> {
    node: DomNode<'a>,
}
#[derive(Clone, Copy, Debug)]
pub struct DomFrozenBase<'a> {
    value: &'a Frozen<NodeId>,
}
impl<'a> DomBase<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn frozen(self) -> Option<DomFrozenBase<'a>> {
        self.node
            .node
            .base
            .as_ref()?
            .frozen
            .as_ref()
            .map(|value| DomFrozenBase { value })
    }
}
impl<'a> DomFrozenBase<'a> {
    pub fn value(self) -> &'a HtmlBaseUrlRecord {
        &self.value.value
    }
    pub fn document(self) -> DocumentId {
        DocumentId(self.value.document)
    }
    pub fn revision(self) -> &'a HtmlBaseFactsRevision {
        &self.value.revision
    }
    pub fn response(self) -> DomBaseResponse {
        match self.value.response {
            Response::Returned => DomBaseResponse::Returned,
            Response::NativeFactsPublished => DomBaseResponse::NativeFactsPublished,
            Response::Pending => unreachable!("canonical publication completes Base response"),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DomLink<'a> {
    node: DomNode<'a>,
}
impl DomLink<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn flags(self) -> HtmlLinkFlags {
        self.node.node.link.as_ref().expect("checked Link").flags
    }
    pub fn parser_document(self) -> Option<DocumentId> {
        self.node
            .node
            .link
            .as_ref()
            .expect("checked Link")
            .parser_document
            .map(DocumentId)
    }
    pub fn history(self) -> HtmlLinkHistory {
        HtmlLinkHistory::new(
            self.flags(),
            self.parser_document() == self.node.owner_document(),
        )
    }
}
impl<'a> DomNode<'a> {
    pub fn as_base(self) -> Option<DomBase<'a>> {
        self.node.base.as_ref()?;
        Some(DomBase { node: self })
    }
    pub fn as_link(self) -> Option<DomLink<'a>> {
        self.node.link.as_ref()?;
        Some(DomLink { node: self })
    }
}
impl<'a> crate::DomDocument<'a> {
    pub fn base_facts(self) -> Option<&'a HtmlParseBaseFacts> {
        self.snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .base_document
            .as_ref()?
            .facts
            .as_ref()
    }
    pub fn first_base(self) -> Option<NodeId> {
        self.snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .base_document
            .as_ref()?
            .first
    }
}
