//! Original HTML constructor state travels in the retained arena, not parser metadata.
use crate::parser::support::limits;
use surgeist_html::*;
use surgeist_html_parser::*;

#[test]
fn original_document_and_reused_inert_owner_keep_null_beside_pending_style_history() {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(
        "<!doctype html><input autofocus><script>document.body.focus()</script><dialog open></dialog><style>p{}</style><template><input autofocus></template><template><dialog open></dialog></template>"), limits()).parse().unwrap();
    let root = report.document().unwrap().root().id();
    let mut cursor = report.metadata();
    let mut documents = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) if row.document_facts().is_some() => {
                assert_eq!(
                    row.node().document_focused_area_designation(),
                    Some(HtmlFocusedAreaDesignation::KnownNull)
                );
                assert!(!documents.contains(&row.node().id()));
                documents.push(row.node().id());
            }
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            _ => (),
        }
    }
    assert_eq!(
        documents.len(),
        2,
        "ordinary owner and one genuinely reused inert owner"
    );
    assert!(documents.contains(&root));
    let mut pending = report.pending_associations();
    let mut influencing_style = false;
    loop {
        match pending.advance() {
            HtmlParsePendingAssociationStep::Occurrence(row) => {
                influencing_style |= row.style().is_some() && row.requires_processing();
            }
            HtmlParsePendingAssociationStep::Complete => break,
            _ => (),
        }
    }
    assert!(
        influencing_style,
        "null is not a receipt for later Style/focus-related work"
    );
    let retained = report.document().unwrap().clone();
    let alias = retained.clone();
    drop(report);
    assert_eq!(retained.root().id(), root);
    assert_eq!(
        alias.root().document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    drop(retained);
    assert_eq!(alias.root().id(), root);
}

#[test]
fn captured_target_stays_external_while_auxiliary_and_new_inert_owners_qualify() {
    let context = super::dialog::fragment_context(
        HtmlDocumentMode::NoQuirks,
        HtmlBrowsingContextPresence::Unavailable,
    );
    let target = context.target_document().clone();
    let report = HtmlFragmentParseRequest::new(HtmlSource::utf8(
        "<dialog open></dialog><template><input autofocus></template><template><dialog open></dialog></template>"), context, limits()).unwrap().parse().unwrap();
    let auxiliary = report
        .fragment_parser_document()
        .unwrap()
        .unwrap()
        .document();
    assert_eq!(
        auxiliary.document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    let mut cursor = report.metadata();
    let mut own_documents = Vec::new();
    let mut missing_captured = false;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if row.document_facts().is_some() {
                    assert_eq!(
                        row.node().document_focused_area_designation(),
                        Some(HtmlFocusedAreaDesignation::KnownNull)
                    );
                    own_documents.push(row.node().id());
                }
                if let Some(dialog) = row.dialog()
                    && let HtmlParseDialogDisposition::Unavailable(missing) = dialog.disposition()
                {
                    let HtmlParseMetadataDocument::Captured(document) = missing.document() else {
                        panic!("captured target must not become the auxiliary owner")
                    };
                    assert_eq!(document.id(), *target.subject());
                    assert_eq!(missing.action(), HtmlDialogAction::ReadFullyActive);
                    missing_captured = true;
                }
            }
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            _ => (),
        }
    }
    assert!(
        missing_captured,
        "fresh auxiliary null does not discharge captured activity/focusing"
    );
    assert_eq!(
        own_documents.len(),
        3,
        "auxiliary plus distinct auxiliary/target inert caches"
    );
    let effects = report.effects().unwrap();
    assert_eq!(
        effects.len(),
        1,
        "two templates reuse the same new captured-target inert cache"
    );
    assert!(matches!(
        effects.effect(0),
        Ok(Some(HtmlParseEffect::TemplateCache { .. }))
    ));
}
