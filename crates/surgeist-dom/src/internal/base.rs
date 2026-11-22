//! Canonical Base history, native qualified observations and Link history.
use crate::internal::{
    document::DomDocumentMetadata,
    identity::{DocumentId, DomOwnerId, NodeId},
    snapshot::{DomNode, DomNodeKind},
    store::{DomError, Node, State},
};
use surgeist_html::elements::link::{HtmlLinkFlags, HtmlLinkHistory};
use surgeist_html::values::DocumentString;
use surgeist_html_semantics::{
    HtmlBaseDocumentFacts, HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseUrlRecord,
    HtmlDocumentFirstBase, HtmlParseBaseFacts, select_document_base,
    select_document_base_serialization,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomBaseFactsError {
    EnvironmentMismatch,
    ForeignProducer,
    InputsMismatch,
    BasisRevisionMismatch,
    Missing(HtmlBaseMissing),
    PolicyMismatch,
}
/// The clause naming which input the base URL lacked, shared by every message
/// that reports an `HtmlBaseMissing`.
pub(crate) const fn missing_clause(missing: HtmlBaseMissing) -> &'static str {
    match missing {
        HtmlBaseMissing::DocumentFacts => "document facts are missing",
        HtmlBaseMissing::DocumentUrl => "the document URL is missing",
        HtmlBaseMissing::AboutBaseUrl => "the about:base URL is missing",
        HtmlBaseMissing::CharacterEncoding => "the character encoding is missing",
        HtmlBaseMissing::ParseObservation => "the parse observation is missing",
        HtmlBaseMissing::Csp => "the Content Security Policy is missing",
        HtmlBaseMissing::FrozenBase => "no frozen base exists",
        HtmlBaseMissing::ResponseDomain => "the response domain is missing",
    }
}
impl core::fmt::Display for DomBaseFactsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                f.write_str("base URL facts do not describe this document's environment")
            }
            Self::ForeignProducer => f.write_str(
                "base URL facts came from a different producer than the facts already bound to this document",
            ),
            Self::InputsMismatch => f.write_str(
                "base URL facts were computed from inconsistent inputs",
            ),
            Self::BasisRevisionMismatch => {
                f.write_str("base URL facts were captured at a different store revision")
            }
            Self::Missing(missing) => write!(
                f,
                "the base URL cannot be established: {}",
                missing_clause(*missing)
            ),
            Self::PolicyMismatch => {
                f.write_str("base URL facts were produced under a Content Security Policy that differs from the document's")
            }
        }
    }
}
impl core::error::Error for DomBaseFactsError {}
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
/// A trusted Base observation bank paired to the actual canonical policy.
/// Opaque identities retain no native policy graph and grant no CSP execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CspBasis {
    document: surgeist_html::security::csp::HtmlCspDocumentId,
    policy: surgeist_html::security::csp::HtmlCspPolicyId,
    revision: surgeist_html::security::csp::HtmlCspPolicyRevision,
}
impl CspBasis {
    pub(crate) fn new(value: &surgeist_html::security::csp::HtmlCspAssociation) -> Self {
        Self {
            document: value.document().clone(),
            policy: value.policy().id().clone(),
            revision: value.policy().revision(),
        }
    }
    pub(crate) fn matches(&self, value: &surgeist_html::security::csp::HtmlCspAssociation) -> bool {
        self.document == *value.document()
            && self.policy == *value.policy().id()
            && self.revision == value.policy().revision()
    }
}
#[derive(Clone, Debug)]
pub(crate) struct BaseDocument<K> {
    pub(crate) facts: Option<HtmlParseBaseFacts>,
    pub(crate) policy_basis: Option<CspBasis>,
    pub(crate) first: Option<K>,
    pub(crate) render_blocking: Vec<K>,
}
impl<K: PartialEq> PartialEq for BaseDocument<K> {
    fn eq(&self, other: &Self) -> bool {
        self.render_blocking == other.render_blocking
            && self.policy_basis == other.policy_basis
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

pub(crate) struct DocumentBaseRead<'a> {
    metadata: &'a DomDocumentMetadata,
    facts: Option<&'a HtmlBaseDocumentFacts>,
    first: HtmlDocumentFirstBase<'a>,
}

impl<'a> DocumentBaseRead<'a> {
    pub(crate) fn open<E: From<DomError>>(
        state: &'a State,
        owner: DomOwnerId,
        document: &'a Node,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        step()?;
        let DomNodeKind::Document(metadata) = document.kind() else {
            return Err(DomError::WrongDocument.into());
        };
        let fields = document.base_document().ok_or(DomError::WrongDocument)?;
        let facts = metadata
            .hosted_inputs
            .as_ref()
            .map(|inputs| &inputs.0.facts)
            .or_else(|| fields.facts.map(HtmlParseBaseFacts::document));
        let first = match fields.first {
            None => HtmlDocumentFirstBase::Absent,
            Some(id) => HtmlDocumentFirstBase::Present(
                state
                    .node_with(owner, id, step)?
                    .base()
                    .and_then(|state| state.frozen.as_ref())
                    .map(|frozen| &frozen.value),
            ),
        };
        Ok(Self {
            metadata,
            facts,
            first,
        })
    }

    fn record(self) -> Result<&'a HtmlBaseUrlRecord, DomError> {
        select_document_base(self.facts, self.first)
            .map_err(|missing| DomError::BaseFacts(DomBaseFactsError::Missing(missing)))
    }

    pub(crate) fn serialization(self) -> Result<&'a DocumentString, DomError> {
        if self.facts.is_some() || matches!(self.first, HtmlDocumentFirstBase::Present(_)) {
            return self.record().map(HtmlBaseUrlRecord::serialization);
        }
        if self
            .metadata
            .document_environment()
            .has_ever_had_navigable()
            || self.metadata.url().code_units()
                != [97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107]
        {
            return Err(DomError::NotSupported);
        }
        select_document_base_serialization(self.metadata.url(), self.first)
            .map_err(|missing| DomError::BaseFacts(DomBaseFactsError::Missing(missing)))
    }
}

impl<'a> crate::internal::snapshot::DomDocument<'a> {
    /// Constructor-established opener policy in the admitted hosted profile.
    /// This is neither a general parsed policy bank nor a currentness proof.
    pub fn opener_policy(
        self,
    ) -> Option<&'a surgeist_html::security::opener_policy::HtmlOpenerPolicy> {
        Some(&self.metadata().hosted_inputs.as_ref()?.0.opener)
    }
    /// Stored constructor URL, about-base and encoding inputs. These are not
    /// the selected effective base or a live source-authority observation.
    pub fn base_document_facts(self) -> Option<&'a surgeist_html_semantics::HtmlBaseDocumentFacts> {
        Some(&self.metadata().hosted_inputs.as_ref()?.0.facts)
    }
    fn document_base_read(self) -> Result<DocumentBaseRead<'a>, DomError> {
        self.snapshot.require_complete_source()?;
        let state = &self.snapshot.state;
        let owner = self.snapshot.owner;
        let document = state.node(owner, self.id.node())?;
        DocumentBaseRead::open(state, owner, document, &mut || Ok(()))
    }
    /// Actual selected record using published facts and the maintained first
    /// frozen Base (O(1)). Missing facts or an unfrozen first Base stay missing.
    pub fn document_base_record(
        self,
    ) -> Result<&'a HtmlBaseUrlRecord, crate::internal::store::DomError> {
        self.document_base_read()?.record()
    }
    /// Current base selection through the maintained first-Base index (O(1));
    /// this never scans descendants or substitutes URL parsing for an observation.
    pub fn document_base_url(
        self,
    ) -> Result<&'a surgeist_html::values::DocumentString, crate::internal::store::DomError> {
        self.document_base_read()?.serialization()
    }
    pub fn base_facts(self) -> Option<&'a HtmlParseBaseFacts> {
        self.snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .base_document()?
            .facts
    }
    pub fn first_base(self) -> Result<Option<NodeId>, crate::internal::store::DomError> {
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

#[cfg(test)]
mod csp_basis_tests {
    use super::*;
    use surgeist_html::security::csp::{HtmlCspLayout, HtmlCspOwner};

    #[test]
    fn equal_revision_numbers_from_independent_issuers_are_not_policy_qualification() {
        let mut admit = |_: HtmlCspLayout| Ok::<_, ()>(());
        let mut first = HtmlCspOwner::new_with(&mut admit).unwrap();
        let mut other = HtmlCspOwner::new_with(&mut admit).unwrap();
        let first = first.initial_document_with(&mut admit).unwrap();
        let other = other.initial_document_with(&mut admit).unwrap();
        assert_eq!(first.policy().revision(), other.policy().revision());
        assert_ne!(first.policy().id(), other.policy().id());
        let basis = CspBasis::new(&first);
        assert!(basis.matches(&first));
        assert!(!basis.matches(&other));
    }
}
