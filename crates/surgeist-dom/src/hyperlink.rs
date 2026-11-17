//! Canonical stored A/Area URL state. Readback does not reinitialize an IDL URL.
use crate::{DocumentId, DomNode, NodeId};
use surgeist_html_semantics::{
    HtmlBaseMissing, HtmlBaseUrlRecord, HtmlEncodingUrlParseFacts, HtmlHyperlinkUrlSnapshot,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomEncodingUrlFactsError {
    BasisRevisionMismatch,
    EnvironmentMismatch,
    Base(HtmlBaseMissing),
    MissingFacts,
    Producer(surgeist_html_semantics::HtmlEncodingUrlFactsError),
}
impl core::fmt::Display for DomEncodingUrlFactsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BasisRevisionMismatch => {
                f.write_str("encoding and URL facts were captured at a different store revision")
            }
            Self::EnvironmentMismatch => {
                f.write_str("encoding and URL facts do not describe this document's environment")
            }
            Self::Base(missing) => write!(
                f,
                "the base URL needed for encoding and URL facts is unavailable: {}",
                crate::base::missing_clause(*missing)
            ),
            Self::MissingFacts => f.write_str("the document has no encoding and URL facts"),
            Self::Producer(_) => {
                f.write_str("the HTML semantics producer refused the encoding and URL facts")
            }
        }
    }
}
impl core::error::Error for DomEncodingUrlFactsError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Producer(error) => Some(error),
            Self::BasisRevisionMismatch
            | Self::EnvironmentMismatch
            | Self::Base(_)
            | Self::MissingFacts => None,
        }
    }
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
    #[cfg(test)]
    pub(crate) fn units(&self) -> usize {
        self.parsed
            .as_ref()
            .map_or(0, |p| p.value.retained_utf16_extent())
    }
}
#[derive(Clone, Debug)]
pub(crate) struct EncodingUrlDocument {
    pub(crate) facts: Option<HtmlEncodingUrlParseFacts>,
}
impl PartialEq for EncodingUrlDocument {
    fn eq(&self, other: &Self) -> bool {
        match (&self.facts, &other.facts) {
            (None, None) => true,
            (Some(a), Some(b)) => a.same_owner(b),
            _ => false,
        }
    }
}
impl Eq for EncodingUrlDocument {}
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
            .hyperlink()?
            .parsed
            .as_ref()
            .map(|p| &p.value)
    }
    /// The actual historical parse Document, separately from current ownerDocument.
    pub fn parse_document(self) -> Option<DocumentId> {
        self.node
            .node
            .hyperlink()?
            .parsed
            .as_ref()
            .map(|p| DocumentId(p.document))
    }
}
impl<'a> DomNode<'a> {
    pub fn as_hyperlink(self) -> Option<DomHyperlink<'a>> {
        self.node.hyperlink()?;
        Some(DomHyperlink { node: self })
    }
}
impl<'a> crate::DomDocument<'a> {
    pub fn encoding_url_facts(self) -> Option<&'a HtmlEncodingUrlParseFacts> {
        self.snapshot
            .node(self.id.node())
            .expect("Document")
            .node
            .encoding_url_document()?
            .facts
    }
}
