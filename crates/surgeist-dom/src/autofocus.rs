//! Canonical per-Document autofocus history, independent of Dialog lists.
use crate::{DomDocument, DomDocumentStamp, DomError, NodeId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DocumentAutofocusState<K> {
    Known { candidates: Vec<K>, processed: bool },
    Unavailable,
}
impl<K> DocumentAutofocusState<K> {
    pub(crate) fn fresh() -> Self {
        Self::Known {
            candidates: Vec::new(),
            processed: false,
        }
    }
}

/// Actual retained autofocus history at a complete historical Document basis.
/// Candidate identities grant no current execution or focusing authority.
#[derive(Clone, Copy, Debug)]
pub struct DomDocumentAutofocus<'a> {
    stamp: DomDocumentStamp,
    candidates: &'a [NodeId],
    processed: bool,
}
impl<'a> DomDocumentAutofocus<'a> {
    pub fn stamp(self) -> DomDocumentStamp {
        self.stamp
    }
    pub fn candidates(self) -> &'a [NodeId] {
        self.candidates
    }
    pub fn processed(self) -> bool {
        self.processed
    }
}
impl<'a> DomDocument<'a> {
    /// Read complete canonical history, independently of parser interaction proof.
    pub fn autofocus(self) -> Result<DomDocumentAutofocus<'a>, DomError> {
        self.snapshot.require_complete_source()?;
        let node = self.snapshot.node(self.id.node())?;
        match node.node.autofocus_document.as_ref() {
            Some(crate::store::canonical::Autofocus::Known {
                candidates,
                processed,
            }) => Ok(DomDocumentAutofocus {
                stamp: self.stamp(),
                candidates,
                processed: *processed,
            }),
            _ => Err(DomError::AutofocusDocumentStateUnavailable),
        }
    }
}
