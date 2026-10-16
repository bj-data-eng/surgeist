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
    FactsPublished,
    ParsedCompleted,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Response {
    Returned,
    Pending,
    FactsPublished,
    ParsedCompleted,
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
        let mut render_blocking = Vec::new();
        render_blocking
            .try_reserve_exact(self.render_blocking.len())
            .map_err(|_| crate::DomError::AllocationFailed)?;
        render_blocking.extend(self.render_blocking.iter().copied().map(&map));
        Ok(BaseDocument {
            facts: self.facts.clone(),
            first: self.first.map(map),
            render_blocking,
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
            Response::FactsPublished => DomBaseResponse::FactsPublished,
            Response::ParsedCompleted => DomBaseResponse::ParsedCompleted,
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
    /// Current base selection through the maintained first-Base index (O(1));
    /// this never scans descendants or substitutes URL parsing for an observation.
    pub fn document_base_url(self) -> Result<&'a crate::DocumentString, crate::DomError> {
        self.snapshot.require_complete_source()?;
        let metadata = self.metadata();
        if metadata.document_environment().has_ever_had_navigable()
            && !matches!(
                metadata.creation_cause(),
                crate::DomDocumentCreationCause::HostedInitialAboutBlank { .. }
            )
        {
            return Err(crate::DomError::NotSupported);
        }
        if metadata.url().code_units() != [97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107] {
            return Err(crate::DomError::NotSupported);
        }
        let node = self
            .snapshot
            .state
            .node(self.snapshot.owner, self.id.node())?;
        let first = match node
            .base_document
            .as_ref()
            .ok_or(crate::DomError::WrongDocument)?
            .first
        {
            None => surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
            Some(base) => surgeist_html_semantics::HtmlDocumentFirstBase::Present(
                self.snapshot
                    .state
                    .node(self.snapshot.owner, base)?
                    .base
                    .as_ref()
                    .and_then(|state| state.frozen.as_ref())
                    .map(|frozen| &frozen.value),
            ),
        };
        surgeist_html_semantics::select_document_base_serialization(metadata.url(), first)
            .map_err(|missing| crate::DomError::BaseFacts(DomBaseFactsError::Missing(missing)))
    }
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
    pub fn first_base(self) -> Result<Option<NodeId>, crate::DomError> {
        self.snapshot.require_complete_source()?;
        Ok(self
            .snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .base_document
            .as_ref()
            .and_then(|state| state.first))
    }
}
