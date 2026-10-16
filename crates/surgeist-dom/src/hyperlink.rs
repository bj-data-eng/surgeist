//! Canonical stored A/Area URL state. Readback does not reinitialize an IDL URL.
use crate::{DocumentId, DomNode, NodeId};
use surgeist_html_semantics::{
    HtmlBaseMissing, HtmlBaseUrlRecord, HtmlHyperlinkParseFacts, HtmlHyperlinkUrlSnapshot,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomHyperlinkFactsError {
    BasisRevisionMismatch,
    EnvironmentMismatch,
    Base(HtmlBaseMissing),
    MissingFacts,
    Producer(surgeist_html_semantics::HtmlHyperlinkFactsError),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Parsed<K> {
    pub(crate) document: K,
    pub(crate) value: HtmlHyperlinkUrlSnapshot,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HyperlinkState<K> {
    pub(crate) parsed: Option<Parsed<K>>,
}
impl<K: Copy> HyperlinkState<K> {
    pub(crate) fn fresh() -> Self {
        Self { parsed: None }
    }
    pub(crate) fn map<T>(&self, map: impl FnOnce(K) -> T) -> HyperlinkState<T> {
        HyperlinkState {
            parsed: self.parsed.as_ref().map(|p| Parsed {
                document: map(p.document),
                value: p.value.clone(),
            }),
        }
    }
    pub(crate) fn units(&self) -> usize {
        self.parsed
            .as_ref()
            .map_or(0, |p| p.value.retained_utf16_extent())
    }
}
#[derive(Clone, Debug)]
pub(crate) struct HyperlinkDocument {
    pub(crate) facts: Option<HtmlHyperlinkParseFacts>,
}
impl PartialEq for HyperlinkDocument {
    fn eq(&self, other: &Self) -> bool {
        match (&self.facts, &other.facts) {
            (None, None) => true,
            (Some(a), Some(b)) => a.same_owner(b),
            _ => false,
        }
    }
}
impl Eq for HyperlinkDocument {}
impl HyperlinkDocument {
    pub(crate) fn fresh() -> Self {
        Self { facts: None }
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
#[derive(Clone, Copy, Debug)]
pub struct DomHyperlink<'a> {
    node: DomNode<'a>,
}
impl<'a> DomHyperlink<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    /// Actual nullable stored URL, without a getter's reinitialization step.
    pub fn url(self) -> Option<&'a HtmlBaseUrlRecord> {
        self.parse_observation()
            .and_then(HtmlHyperlinkUrlSnapshot::url)
    }
    pub fn parse_observation(self) -> Option<&'a HtmlHyperlinkUrlSnapshot> {
        self.node
            .node
            .hyperlink
            .as_ref()?
            .parsed
            .as_ref()
            .map(|p| &p.value)
    }
    /// The actual historical parse Document, separately from current ownerDocument.
    pub fn parse_document(self) -> Option<DocumentId> {
        self.node
            .node
            .hyperlink
            .as_ref()?
            .parsed
            .as_ref()
            .map(|p| DocumentId(p.document))
    }
}
impl<'a> DomNode<'a> {
    pub fn as_hyperlink(self) -> Option<DomHyperlink<'a>> {
        self.node.hyperlink.as_ref()?;
        Some(DomHyperlink { node: self })
    }
}
impl<'a> crate::DomDocument<'a> {
    pub fn hyperlink_facts(self) -> Option<&'a HtmlHyperlinkParseFacts> {
        self.snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .hyperlink_document
            .as_ref()?
            .facts
            .as_ref()
    }
}
