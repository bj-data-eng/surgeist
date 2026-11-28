//! Current canonical Document state, joined by the shared inheritance owner.
use super::*;
use crate::internal::host::DomHostObservations;
use surgeist_html_semantics::{
    HtmlDesignMode, HtmlDocumentDesignModeFact, HtmlGlobalInheritanceFacts,
    HtmlGlobalInheritanceLimits, HtmlGlobalInheritanceProducerId, HtmlGlobalInheritanceRevision,
    HtmlSpellcheckDefaultFact,
};

pub(super) fn with_facts(
    input: HtmlSemanticContextInput,
    source: &DomSnapshot,
    capture: &HtmlCapture,
    entries: &[Entry],
    host: Option<&DomHostCapture>,
    control: &mut Control<'_>,
) -> Result<HtmlSemanticContextInput, DomHtmlCaptureError> {
    let mut documents = Vec::new();
    let mut defaults = Vec::new();
    let policy = host.and_then(|host| match host.observations() {
        DomHostObservations::SpellcheckDefaults(policy) => Some(policy),
        _ => None,
    });
    // The exact Document's capture contains every Element named by its hosted
    // policy, including detached nodes.
    if let Some(policy) = policy {
        control.reserve_extent(&mut defaults, policy.len(), control.limits.subjects)?;
    }
    let document_capacity = control
        .limits
        .subjects
        .checked_sub(defaults.capacity())
        .ok_or(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: control.limits.subjects,
        })?;
    for entry in entries {
        control.charge(1)?;
        if let Some(document) = source
            .node_with(entry.node, || control.charge(1))?
            .as_document()
        {
            control.reserve_extent(&mut documents, 1, document_capacity)?;
            documents.push(HtmlDocumentDesignModeFact::new(
                entry.subject.clone(),
                if document.metadata().design_mode_enabled() {
                    HtmlDesignMode::Enabled
                } else {
                    HtmlDesignMode::Disabled
                },
            ));
        }
        if let Some(defaults_policy) = policy
            && let Some(default) =
                defaults_policy.default_for_with(entry.node, || control.charge(1))?
        {
            control.reserve_extent(&mut defaults, 1, defaults_policy.len())?;
            defaults.push(HtmlSpellcheckDefaultFact::new(
                entry.subject.clone(),
                Some(default),
            ));
        }
    }
    control.charge(1)?;
    let revision = HtmlGlobalInheritanceRevision::new(
        HtmlGlobalInheritanceProducerId::fresh(),
        0,
        source.revision().get(),
    );
    // Same explicit finite work scalar, independently applied to this producer
    // formation. Only the supplied source contributes spellcheck defaults.
    let facts = HtmlGlobalInheritanceFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        documents,
        defaults,
        HtmlGlobalInheritanceLimits::new(control.limits.subjects, control.limits.work),
    )
    .map_err(DomHtmlCaptureError::GlobalInheritance)?;
    control.charge(1)?;
    Ok(input.with_global_inheritance(revision, Some(facts)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_document_rows_obey_adapter_work_and_record_bounds_with_a_measured_minimum() {
        let cancel = AtomicBool::new(false);
        let mut dom = crate::internal::store::Dom::new(
            crate::internal::store::DomLimits::new(
                32,
                4096,
                32,
                0,
                0,
                4,
                65536,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = crate::internal::batch::DomBatch::new();
        let tickets: Vec<_> = (0..16).map(|_| batch.create_document()).collect();
        let commit = dom.apply(batch, &cancel).unwrap();
        let source = dom.snapshot();
        let stamp = source.stamp();
        // Each captured Document comes from its actual canonical view and is
        // mapped back to that real NodeId; no surrogate identifiers participate.
        let revision = HtmlHostFactsRevision::fresh();
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(32, 32, 0, 4096, 32, 0, 0, 16_777_216),
            revision,
        );
        let mut staged = Vec::new();
        for ticket in &tickets {
            let document = source.document(commit.document(ticket).unwrap()).unwrap();
            staged.push((
                document.id().node(),
                builder
                    .add_document(
                        document.metadata().mode(),
                        HtmlScriptingStatus::Disabled,
                        HtmlCaptureRegistryInput::Null,
                    )
                    .unwrap(),
            ));
        }
        let capture = builder.finish().unwrap();
        let entries: Vec<_> = staged
            .iter()
            .map(|(node, subject)| Entry {
                node: *node,
                subject: capture.subject_id(subject).unwrap(),
            })
            .collect();
        let run = |subjects, work| {
            with_facts(
                HtmlSemanticContextInput::new(Vec::new()),
                &source,
                &capture,
                &entries,
                None,
                &mut Control {
                    limits: DomHtmlCaptureLimits::new(subjects, work),
                    work: 0,
                    cancellation: &cancel,
                },
            )
        };
        // The producer validates the 16 rows in fewer steps than the adapter
        // takes to stage them, so the shared work bound is reached by the
        // adapter first; the producer's own exact boundary is pinned by the
        // semantics crate's tests. Measure the least bound that finishes.
        let minimum = (0..1000)
            .find(|work| run(16, *work).is_ok())
            .expect("a finite bound admits the rows");
        assert!(minimum > 16, "every row is charged: {minimum}");
        for work in 0..minimum {
            assert!(
                matches!(
                    run(16, work),
                    Err(DomHtmlCaptureError::ResourceLimit {
                        resource: DomHtmlCaptureResource::Work,
                        limit
                    }) if limit == work
                ),
                "bound {work} below the minimum {minimum}"
            );
        }
        assert!(run(16, minimum + 50).is_ok());
        assert!(matches!(
            run(15, 1000),
            Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: 15
            })
        ));
        let input = run(16, 1000).unwrap();
        let context =
            HtmlSemanticContext::new(&capture, capture.host_facts_revision().clone(), input, 32)
                .unwrap();
        let facts = context.global_inheritance().unwrap();
        assert_eq!(facts.design_modes().len(), 16);
        assert!(
            facts
                .design_modes()
                .iter()
                .all(|fact| fact.state() == HtmlDesignMode::Disabled)
        );
        assert!(facts.spellcheck_defaults().is_empty());
        assert_eq!(dom.snapshot().stamp(), stamp);
    }
}
