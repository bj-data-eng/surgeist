//! Issued, stage-qualified URL observations. The original Source owner admits
//! and validates their actual raw operands before retaining a response.
use super::*;
use crate::{DomError, DomHostedChildRequest, DomOpaqueOrigin};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseParseObservation, HtmlBaseParseOutcome,
    HtmlBaseUrlRecord,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceUrlStage {
    IframeDeclaredOrigin,
    FrameIframeSrc,
}
#[derive(Clone, Debug)]
pub enum DomSourceUrlOrigin {
    Opaque(DomOpaqueOrigin),
    UnequalToTarget,
}
#[derive(Debug)]
pub(crate) struct UrlBasis {
    pub(crate) child: DomHostedChildRequest,
    pub(crate) occurrence: u64,
    pub(crate) document: DomDocumentStamp,
    pub(crate) stage: DomSourceUrlStage,
    pub(crate) raw: DocumentString,
    pub(crate) base: HtmlBaseUrlRecord,
    pub(crate) encoding: HtmlBaseEncoding,
    pub(crate) revision: HtmlBaseFactsRevision,
    pub(crate) target: Option<DomOpaqueOrigin>,
}
#[derive(Clone, Debug)]
pub struct DomSourceUrlRequest {
    pub(crate) basis: Arc<UrlBasis>,
}
impl DomSourceUrlRequest {
    pub fn operation(&self) -> DomSourceOperationId {
        self.basis.child.operation()
    }
    pub fn occurrence(&self) -> u64 {
        self.basis.occurrence
    }
    pub fn stage(&self) -> DomSourceUrlStage {
        self.basis.stage
    }
    pub fn embedder(&self) -> NodeId {
        self.basis.child.container()
    }
    pub fn document(&self) -> DomDocumentStamp {
        self.basis.document
    }
    pub fn raw(&self) -> &DocumentString {
        &self.basis.raw
    }
    pub fn base_url(&self) -> &HtmlBaseUrlRecord {
        &self.basis.base
    }
    pub fn encoding(&self) -> &HtmlBaseEncoding {
        &self.basis.encoding
    }
    pub fn revision(&self) -> &HtmlBaseFactsRevision {
        &self.basis.revision
    }
    pub fn target_origin(&self) -> Option<&DomOpaqueOrigin> {
        self.basis.target.as_ref()
    }
    /// Wrap the producer's actual response for this exact issued stage. This
    /// checks the stage/outcome pairing; canonical acceptance separately admits
    /// and compares every influencing raw/current/opaque operand with work.
    pub fn observe(
        &self,
        observation: HtmlBaseParseObservation,
        origin: Option<DomSourceUrlOrigin>,
    ) -> Result<DomSourceUrlObservation, DomError> {
        let valid = match (self.stage(), observation.outcome(), origin.as_ref()) {
            (DomSourceUrlStage::IframeDeclaredOrigin, HtmlBaseParseOutcome::Failure, None) => true,
            (
                DomSourceUrlStage::IframeDeclaredOrigin,
                HtmlBaseParseOutcome::Parsed { .. },
                Some(_),
            ) => self.basis.target.is_some(),
            (DomSourceUrlStage::FrameIframeSrc, _, None) => self.basis.target.is_none(),
            _ => false,
        };
        if !valid {
            return Err(DomError::SourceUrlInputsMismatch);
        }
        Ok(DomSourceUrlObservation {
            request: self.clone(),
            observation,
            origin,
        })
    }
}
#[derive(Debug)]
pub struct DomSourceUrlObservation {
    pub(crate) request: DomSourceUrlRequest,
    pub(crate) observation: HtmlBaseParseObservation,
    pub(crate) origin: Option<DomSourceUrlOrigin>,
}
impl DomSourceUrlObservation {
    pub fn request(&self) -> &DomSourceUrlRequest {
        &self.request
    }
    pub fn observation(&self) -> &HtmlBaseParseObservation {
        &self.observation
    }
    pub fn origin(&self) -> Option<&DomSourceUrlOrigin> {
        self.origin.as_ref()
    }
}
