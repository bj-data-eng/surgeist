//! Actual canonical Base facts with current capture binding and historical value.
use super::*;
use surgeist_html_semantics::{
    HtmlBaseFactsError, HtmlBaseFactsLimits, HtmlCapturedBaseDocument, HtmlCapturedBaseFacts,
    HtmlCapturedFrozenBase,
};

pub(super) fn with_facts(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    subjects: &super::nodes::Subjects<'_>,
    control: &mut Control<'_>,
    budget: Option<(BaseCapturePolicy, usize)>,
) -> Result<(HtmlSemanticContextInput, usize), DomHtmlCaptureError> {
    let Some((policy, spent)) = budget else {
        return Ok((input, 0));
    };
    let remaining = policy
        .work
        .checked_sub(spent)
        .ok_or(DomHtmlCaptureError::BaseFacts(
            HtmlBaseFactsError::WorkLimit,
        ))?;
    let entries = subjects.entries();
    let mut documents = Vec::new();
    let mut frozen = Vec::new();
    let mut revision = None;
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        if let Some(facts) = node.node.base_document().and_then(|d| d.facts) {
            let observed = facts.document().revision();
            if revision.as_ref().is_some_and(|r| r != observed) {
                return Err(DomHtmlCaptureError::BaseFacts(
                    HtmlBaseFactsError::RevisionMismatch,
                ));
            }
            revision = Some(observed.clone());
            control.reserve(&mut documents, 1)?;
            documents.push(HtmlCapturedBaseDocument::new(
                capture.document_id(&entry.subject)?,
                facts.clone(),
            ));
        }
        if let Some(value) = node.node.base().and_then(|b| b.frozen.as_ref()) {
            let mut href = None;
            for attr in node.as_element().expect("Base").attributes().iter() {
                control.charge(1)?;
                if attr.name().namespace_uri().is_none()
                    && attr.name().local_name().code_units() == [104, 114, 101, 102]
                {
                    href = Some(owned_href(attr.value(), control)?);
                    break;
                }
            }
            if let Some(href) = href {
                control.reserve(&mut frozen, 1)?;
                // Producer row.document qualifies CURRENT nodeDocument. The
                // canonical freezing Document remains separate historical state.
                frozen.push(HtmlCapturedFrozenBase::new(
                    entry.subject.clone(),
                    capture
                        .document_id(&subjects.subject(control, node.node.document().node())?)?,
                    href,
                    Some(value.value.clone()),
                    value.revision.clone(),
                ));
            }
        }
    }
    let revision = match revision {
        Some(r) => r,
        None => {
            // Frozen history can remain meaningful after adoption into an unbound
            // native Document. The producer needs a revision identity even without
            // document rows; use the actual first historical revision, never fresh.
            let mut found = None;
            for entry in entries {
                control.charge(1)?;
                if let Some(f) = source
                    .node(entry.node)?
                    .node
                    .base()
                    .and_then(|b| b.frozen.as_ref())
                {
                    found = Some(f.revision.clone());
                    break;
                }
            }
            let Some(r) = found else {
                return Ok((input, spent));
            };
            r
        }
    };
    control.charge(1)?;
    let facts = HtmlCapturedBaseFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        documents,
        frozen,
        HtmlBaseFactsLimits::new(policy.records, policy.units, remaining),
    )
    .map_err(DomHtmlCaptureError::BaseFacts)?;
    let used = spent
        .checked_add(facts.formation_usage().validation_work())
        .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
    control.charge(1)?;
    Ok((input.with_base_facts(revision, Some(facts)), used))
}

/// The href a captured frozen Base retains. A value that borrows a block of text is copied into
/// an exact owner of its units, so the capture does not hold the block at the price of its
/// capacity; the work of the copy is charged before it allocates. An owner is shared.
///
/// The units of the copy are admitted afterwards, by the captured Base facts against the
/// policy's unit limit, and no control path admits them earlier without reporting a unit refusal
/// ahead of an earlier refusal of the facts. The allocation that precedes that admission is
/// equivalent to an admitted one: it is bounded by the length of a value the source snapshot
/// already holds, and the copies of one capture are bounded together by the work charged before
/// each of them.
fn owned_href(
    value: &surgeist_html::values::DocumentString,
    control: &mut Control<'_>,
) -> Result<surgeist_html::values::DocumentString, DomHtmlCaptureError> {
    if value.borrowed_block().is_none() {
        return Ok(value.clone());
    }
    control.charge(value.len())?;
    let mut units = Vec::new();
    units
        .try_reserve_exact(value.len())
        .map_err(|_| DomHtmlCaptureError::AllocationFailed)?;
    units.extend_from_slice(value.code_units());
    Ok(surgeist_html::values::DocumentString::from_utf16(units))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dom, DomBatch, DomLimits, DomSlotQueryLimits, DomSlotSemanticLimits};
    use surgeist_html::values::Namespace;
    use surgeist_html_semantics::*;
    fn fixture() -> (Dom, DocumentId) {
        fixture_with_href("target".into())
    }
    fn fixture_with_href(href: surgeist_html::values::DocumentString) -> (Dom, DocumentId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                32768,
                128,
                128,
                64,
                8,
                131072,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_base_processing(64, 1_000_000)
            .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let base = b.create_element_ns(&d, Namespace::html(), "base");
        let c = dom.apply(b, &cancel).unwrap();
        let d = c.document(&d).unwrap();
        let base = c.created(&base).unwrap().unwrap();
        let r = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let encoding = HtmlBaseEncoding::fresh();
        let fallback = HtmlBaseUrlRecord::observed(
            r.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
            None,
        )
        .unwrap();
        let parsed = HtmlBaseUrlRecord::observed(
            r.clone(),
            "https://example.test/".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap();
        let env = HtmlBaseDocumentFacts::new(
            r.clone(),
            Some(fallback.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap();
        let obs = HtmlBaseParseObservation::new(
            "target".into(),
            fallback,
            encoding,
            r,
            HtmlBaseParseOutcome::Parsed {
                record: parsed,
                base_allowed: HtmlBaseAllowed::Allowed,
            },
        )
        .unwrap();
        let facts =
            HtmlParseBaseFacts::new(env, vec![obs], HtmlBaseFactsLimits::new(8, 8192, 10000))
                .unwrap();
        let mut b = DomBatch::new();
        b.bind_base_facts(dom.snapshot().document(d).unwrap().stamp(), &facts);
        b.set_attribute(base, "href", href);
        b.insert_before(base, d, None);
        dom.apply(b, &cancel).unwrap();
        (dom, d)
    }
    fn capture(
        dom: &Dom,
        doc: DocumentId,
        records: usize,
        units: usize,
        work: usize,
    ) -> Result<DomHtmlCapture, DomHtmlCaptureError> {
        dom.snapshot().html_capture(
            doc,
            None,
            HtmlCaptureLimits::new(64, 256, 64, 32768, 64, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(64, 1_000_000).with_base_facts(records, units, work),
            &AtomicBool::new(false),
        )
    }
    fn slots() -> DomSlotSemanticLimits {
        DomSlotSemanticLimits::new(
            DomSlotQueryLimits::new(64, 1024).unwrap(),
            DomHtmlCaptureLimits::new(64, 1_000_000),
            HtmlLanguageDirectionLimits::new(64, 128, 0, 1_000_000),
            HtmlInertFactLimits::new(64, 1_000_000),
        )
    }
    #[test]
    fn producer_actual_capacity_and_units_are_independent_exact_capture_limits() {
        let (dom, doc) = fixture();
        let c = capture(&dom, doc, 64, 32768, 1_000_000).unwrap();
        let usage = c.context.base_facts().unwrap().formation_usage();
        let records = usage.record_capacity();
        let units = usage.utf16_extent();
        assert!(records > 0 && units > 0);
        assert!(matches!(
            capture(&dom, doc, records - 1, units, 1_000_000),
            Err(DomHtmlCaptureError::BaseFacts(
                HtmlBaseFactsError::RecordLimit
            ))
        ));
        assert!(matches!(
            capture(&dom, doc, records, units - 1, 1_000_000),
            Err(DomHtmlCaptureError::BaseFacts(
                HtmlBaseFactsError::Utf16Limit
            ))
        ));
        capture(&dom, doc, records, units, usage.validation_work()).unwrap();
    }
    #[test]
    fn the_href_of_a_frozen_base_that_borrows_a_block_is_captured_by_its_length() {
        let mut units = Vec::with_capacity(512);
        units.extend("target".encode_utf16());
        let block = surgeist_html::values::TextBlock::from_utf16(units);
        assert!(block.capacity() >= 512);
        // The reference is an exact owner of the same six units.
        let mut exact = Vec::new();
        exact.try_reserve_exact(6).unwrap();
        exact.extend("target".encode_utf16());
        assert_eq!(exact.capacity(), 6);
        let (owned, owned_doc) =
            fixture_with_href(surgeist_html::values::DocumentString::from_utf16(exact));
        let (borrowed, borrowed_doc) = fixture_with_href(block.string(0..6).unwrap());
        let reference = capture(&owned, owned_doc, 64, 32768, 1_000_000).unwrap();
        let usage = reference.context.base_facts().unwrap().formation_usage();
        // The limits that admit the capture of the owner admit the borrower, which the capture
        // keeps as its own six units and not as the capacity of its block.
        let copy = capture(
            &borrowed,
            borrowed_doc,
            usage.record_capacity(),
            usage.utf16_extent(),
            usage.validation_work(),
        )
        .unwrap();
        assert_eq!(copy.context.base_facts().unwrap().formation_usage(), usage);
        assert_eq!(
            copy.context.base_facts().unwrap().retained_storage_usage(),
            reference
                .context
                .base_facts()
                .unwrap()
                .retained_storage_usage()
        );
    }
    #[test]
    fn producer_validation_work_is_monotone_across_slot_reconstruction() {
        let (dom, doc) = fixture();
        let c = capture(&dom, doc, 64, 32768, 1_000_000).unwrap();
        let used = c.base_work;
        assert!(used > 0);
        let c = capture(&dom, doc, 64, 32768, used).unwrap();
        assert!(matches!(
            c.with_slot_semantics(slots(), &AtomicBool::new(false)),
            Err(DomHtmlCaptureError::BaseFacts(
                HtmlBaseFactsError::WorkLimit
            ))
        ));
        let c = capture(&dom, doc, 64, 32768, used * 2)
            .unwrap()
            .with_slot_semantics(slots(), &AtomicBool::new(false))
            .unwrap();
        assert_eq!(c.base_work, used * 2);
        assert!(c.context.base_facts().is_some());
        assert!(matches!(
            c.with_slot_semantics(slots(), &AtomicBool::new(false)),
            Err(DomHtmlCaptureError::BaseFacts(
                HtmlBaseFactsError::WorkLimit
            ))
        ));
    }
}
