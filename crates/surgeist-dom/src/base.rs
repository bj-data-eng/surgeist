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
    #[cfg(test)]
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
            .base()?
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
        self.node.node.link().expect("checked Link").flags
    }
    pub fn parser_document(self) -> Option<DocumentId> {
        self.node
            .node
            .link()
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
        self.node.base()?;
        Some(DomBase { node: self })
    }
    pub fn as_link(self) -> Option<DomLink<'a>> {
        self.node.link()?;
        Some(DomLink { node: self })
    }
}
impl<'a> crate::DomDocument<'a> {
    /// Constructor-established opener policy in the admitted hosted profile.
    /// This is neither a general parsed policy bank nor a currentness proof.
    pub fn opener_policy(self) -> Option<&'a surgeist_html::HtmlOpenerPolicy> {
        Some(&self.metadata().hosted_inputs.as_ref()?.0.opener)
    }
    /// Stored constructor URL, about-base and encoding inputs. These are not
    /// the selected effective base or a live source-authority observation.
    pub fn base_document_facts(self) -> Option<&'a surgeist_html_semantics::HtmlBaseDocumentFacts> {
        Some(&self.metadata().hosted_inputs.as_ref()?.0.facts)
    }
    fn selected_first_base(
        self,
    ) -> Result<surgeist_html_semantics::HtmlDocumentFirstBase<'a>, crate::DomError> {
        self.snapshot.require_complete_source()?;
        let node = self
            .snapshot
            .state
            .node(self.snapshot.owner, self.id.node())?;
        Ok(
            match node
                .base_document()
                .ok_or(crate::DomError::WrongDocument)?
                .first
            {
                None => surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
                Some(base) => surgeist_html_semantics::HtmlDocumentFirstBase::Present(
                    self.snapshot
                        .state
                        .node(self.snapshot.owner, base)?
                        .base()
                        .and_then(|state| state.frozen.as_ref())
                        .map(|frozen| &frozen.value),
                ),
            },
        )
    }
    /// Actual selected record using constructor facts and the maintained first
    /// frozen Base (O(1)). Missing facts or an unfrozen first Base stay missing.
    pub fn document_base_record(self) -> Result<&'a HtmlBaseUrlRecord, crate::DomError> {
        let first = self.selected_first_base()?;
        surgeist_html_semantics::select_document_base(self.base_document_facts(), first)
            .map_err(|missing| crate::DomError::BaseFacts(DomBaseFactsError::Missing(missing)))
    }
    /// Current base selection through the maintained first-Base index (O(1));
    /// this never scans descendants or substitutes URL parsing for an observation.
    pub fn document_base_url(self) -> Result<&'a crate::DocumentString, crate::DomError> {
        if self.base_document_facts().is_some() {
            return self
                .document_base_record()
                .map(HtmlBaseUrlRecord::serialization);
        }
        let first = self.selected_first_base()?;
        let metadata = self.metadata();
        if metadata.document_environment().has_ever_had_navigable()
            || metadata.url().code_units() != [97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107]
        {
            return Err(crate::DomError::NotSupported);
        }
        surgeist_html_semantics::select_document_base_serialization(metadata.url(), first)
            .map_err(|missing| crate::DomError::BaseFacts(DomBaseFactsError::Missing(missing)))
    }
    pub fn base_facts(self) -> Option<&'a HtmlParseBaseFacts> {
        self.snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .base_document()?
            .facts
    }
    pub fn first_base(self) -> Result<Option<NodeId>, crate::DomError> {
        self.snapshot.require_complete_source()?;
        Ok(self
            .snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .base_document()
            .and_then(|state| state.first))
    }
}
