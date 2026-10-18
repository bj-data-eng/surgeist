//! Canonical top-layer membership and its actual generated UA declaration input.
use crate::{DomDocument, DomDocumentStamp, DomError, NodeId};

/// Generated declaration input at the UA important origin for `overlay`.
/// Style owns collection, cascade and evaluation of these actual producer rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomTopLayerUaRule {
    /// Universal selector with `overlay: none !important` at UA origin.
    UniversalOverlayNone,
    /// Exact Element target with `overlay: auto !important` at UA origin.
    ElementOverlayAuto(NodeId),
}

/// Borrowed canonical producer input at one complete historical Document basis.
/// Retaining this view grants no current execution or rendering permission.
#[derive(Clone, Copy, Debug)]
pub struct DomDocumentTopLayer<'a> {
    stamp: DomDocumentStamp,
    raw: &'a [NodeId],
    pending: &'a [NodeId],
}
impl<'a> DomDocumentTopLayer<'a> {
    pub fn stamp(self) -> DomDocumentStamp {
        self.stamp
    }
    pub fn raw_members(self) -> &'a [NodeId] {
        self.raw
    }
    pub fn pending_removals(self) -> &'a [NodeId] {
        self.pending
    }
    /// Baseline first, then exact dynamic rules in raw order excluding pending.
    /// No computed overlay or downstream installation result is inferred.
    pub fn ua_rules(self) -> impl Iterator<Item = DomTopLayerUaRule> + 'a {
        std::iter::once(DomTopLayerUaRule::UniversalOverlayNone).chain(
            self.raw
                .iter()
                .copied()
                .filter(move |member| !self.pending.contains(member))
                .map(DomTopLayerUaRule::ElementOverlayAuto),
        )
    }
}
impl<'a> DomDocument<'a> {
    /// Actual raw/pending membership and generated UA input. Incomplete Source
    /// state and unqualified Document feature history cannot supply a view.
    pub fn top_layer(self) -> Result<DomDocumentTopLayer<'a>, DomError> {
        self.snapshot.require_complete_source()?;
        let node = self.snapshot.node(self.id.node())?;
        match node.node.interactive_document.as_ref() {
            Some(crate::store::canonical::Interactive::Known {
                top_layer,
                pending_top_layer_removals,
                ..
            }) => Ok(DomDocumentTopLayer {
                stamp: self.stamp(),
                raw: top_layer,
                pending: pending_top_layer_removals,
            }),
            _ => Err(DomError::DialogDocumentStateUnavailable),
        }
    }
}
