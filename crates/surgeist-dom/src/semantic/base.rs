//! Actual canonical Base facts with current capture binding and historical value.
use super::*;
use surgeist_html_semantics::{
    HtmlBaseFactsError, HtmlBaseFactsLimits, HtmlCapturedBaseDocument, HtmlCapturedBaseFacts,
    HtmlCapturedFrozenBase,
};

fn subject(
    entries: &[Entry],
    id: NodeId,
    control: &mut Control<'_>,
) -> Result<HtmlCaptureSubjectId, DomHtmlCaptureError> {
    for e in entries {
        control.charge(1)?;
        if e.node == id {
            return Ok(e.subject.clone());
        }
    }
    Err(DomHtmlCaptureError::UnmappedSubject)
}
pub(super) fn with_facts(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    entries: &[Entry],
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
    let mut documents = Vec::new();
    let mut frozen = Vec::new();
    let mut revision = None;
    for entry in entries {
        control.charge(1)?;
        let node = source.node(entry.node)?;
        if let Some(facts) = node
            .node
            .base_document
            .as_ref()
            .and_then(|d| d.facts.as_ref())
        {
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
        if let Some(value) = node.node.base.as_ref().and_then(|b| b.frozen.as_ref()) {
            let mut href = None;
            for attr in node.as_element().expect("Base").attributes().iter() {
                control.charge(1)?;
                if attr.name().namespace().uri().is_none()
                    && attr.name().local_name().code_units() == [104, 114, 101, 102]
                {
                    href = Some(attr.value().clone());
                    break;
                }
            }
            if let Some(href) = href {
                control.reserve(&mut frozen, 1)?;
                // Producer row.document qualifies CURRENT nodeDocument. The
                // canonical freezing Document remains separate historical state.
                frozen.push(HtmlCapturedFrozenBase::new(
                    entry.subject.clone(),
                    capture.document_id(&subject(entries, node.node.document.node(), control)?)?,
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
                    .base
                    .as_ref()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dom, DomBatch, DomLimits, DomSlotQueryLimits, DomSlotSemanticLimits, Namespace};
    use surgeist_html_semantics::*;
    fn fixture() -> (Dom, DocumentId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(64, 32768, 128, 128, 64, 8, 131072)
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
        )
        .unwrap();
        let parsed = HtmlBaseUrlRecord::observed(
            r.clone(),
            "https://example.test/".into(),
            HtmlBaseUrlScheme::Other,
            false,
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
        b.set_attribute(base, "href", "target");
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
            HtmlCaptureLimits::new(64, 256, 64, 32768, 64, 0, 0),
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
