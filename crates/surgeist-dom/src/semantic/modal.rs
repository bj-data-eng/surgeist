//! Qualified canonical Document modal facts, independent of flat-node coverage.
use super::*;
use crate::dialog::DocumentInteractiveState;
use surgeist_html_semantics::{
    HtmlDocumentModalBlocking, HtmlDocumentModalFact, HtmlFlatTreeProducerId, HtmlFlatTreeRevision,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact,
};

pub(super) fn with_facts(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    entries: &[Entry],
    nodes: Vec<HtmlInertNodeFact>,
    limits: HtmlInertFactLimits,
    control: &mut Control<'_>,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let mut documents = Vec::new();
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        if node.as_document().is_none() {
            continue;
        }
        // This is the complete canonical managed list, even when the semantic
        // capture contains only a supplemental portion of this Document's tree.
        // Neither open attributes nor missing captured members establish absence.
        let blocking = match node.node.interactive_document.as_ref() {
            Some(DocumentInteractiveState::Known { top_layer, .. }) if top_layer.is_empty() => {
                HtmlDocumentModalBlocking::Unblocked
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
