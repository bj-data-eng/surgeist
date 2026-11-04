//! Qualified canonical Document modal facts, independent of flat-node coverage.
use super::*;
use crate::store::canonical::InteractiveView as DocumentInteractiveState;
use surgeist_html_semantics::{
    HtmlDocumentModalBlocking, HtmlDocumentModalFact, HtmlFlatTreeProducerId, HtmlFlatTreeRevision,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact,
};

/// The modal facts of every Document among the entries of `nodes`. Its node
/// table, if the caller has none, is built on the first top-layer Dialog found,
/// charged then, and left there for the caller. It serves every blocker lookup
/// in constant steps; a capture without such a Dialog builds none.
pub(super) fn with_facts(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    entries: &mut super::nodes::LazyNodes<'_>,
    nodes: Vec<HtmlInertNodeFact>,
    limits: HtmlInertFactLimits,
    control: &mut Control<'_>,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let mut documents = Vec::new();
    for entry in entries.entries() {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        if node.as_document().is_none() {
            continue;
        }
        // This is the complete canonical managed list, even when the semantic
        // capture contains only a supplemental portion of this Document's tree.
        // Neither open/modal flags, pending removal nor missing captured members
        // replace the source's last actual HTML Dialog in raw order.
        let blocking = match node.node.interactive_document() {
            Some(DocumentInteractiveState::Known { top_layer, .. }) => {
                let mut blocking = HtmlDocumentModalBlocking::Unblocked;
                for member in top_layer.iter().rev() {
                    control.charge(1)?;
                    if source.node(*member)?.as_dialog().is_none() {
                        continue;
                    }
                    blocking = HtmlDocumentModalBlocking::Unavailable;
                    if let Some(mapped) = entries.entry(control, *member)? {
                        blocking = HtmlDocumentModalBlocking::BlockedBy(mapped.subject.clone());
                    }
                    break;
                }
                blocking
            }
            _ => HtmlDocumentModalBlocking::Unavailable,
        };
        control.reserve(&mut documents, 1)?;
        documents.push(HtmlDocumentModalFact::new(entry.subject.clone(), blocking));
    }
    control.charge(1)?;
    // The opaque formation/configuration and exact canonical revision qualify
    // only supplied facts. An initial empty node vector certifies no flat edges;
    // Slot reconstruction supplies its independently computed actual relations.
    let revision =
        HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 0, source.revision().get());
    let facts = HtmlInertFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        nodes,
        documents,
        limits,
    )
    .map_err(DomHtmlCaptureError::Inert)?;
    control.charge(1)?;
    Ok(input.with_inert(revision, Some(facts)))
}

#[cfg(test)]
mod tests;
