//! Qualified actual href parsing through HTML's ordered set-URL actions.
use super::*;
use crate::hyperlink::{DomEncodingUrlFactsError as Failure, Parsed};
use surgeist_html::{
    HtmlHyperlinkAttributeEffect, HtmlHyperlinkSetUrlAction as Action, HtmlHyperlinkSetUrlCursor,
    HtmlHyperlinkSetUrlReply as Reply, HtmlHyperlinkTreeMutation, KnownHtmlAttribute,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseUrlRecord,
    HtmlDocumentFirstBase, HtmlEncodingUrlLookupError, HtmlEncodingUrlParseFacts,
};

struct RetainedParse {
    value: surgeist_html_semantics::HtmlHyperlinkUrlSnapshot,
    payload: Option<source::checkpoint::Charge>,
}

fn failure(error: Failure) -> DomError {
    DomError::EncodingUrlFacts(error)
}
impl Draft {
    pub(super) fn encoding_url_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.effect_step(cancel)?;
        let next = self
            .encoding_url_work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.encoding_url_work_limit {
            return Err(DomError::EncodingUrlWorkLimit {
                limit: self.encoding_url_work_limit,
            });
        }
        self.encoding_url_work.set(next);
        Ok(())
    }
    pub(super) fn encoding_url_environment(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<
        (
            &HtmlBaseUrlRecord,
            &HtmlBaseEncoding,
            &HtmlBaseFactsRevision,
        ),
        DomError,
    > {
        self.encoding_url_environment_with(document, &mut || self.encoding_url_step(cancel))
    }
    /// The original canonical selection with admission supplied by its actual
    /// caller. Parsed installation uses its own cumulative work/first-stop owner.
    pub(super) fn encoding_url_environment_with(
        &self,
        document: Key,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<
        (
            &HtmlBaseUrlRecord,
            &HtmlBaseEncoding,
            &HtmlBaseFactsRevision,
        ),
        DomError,
    > {
        step()?;
        let node = self.node(document)?;
        let transaction::KindRead::Document(metadata) = &node.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        if metadata.document_environment().has_ever_had_navigable() {
            return Err(failure(Failure::EnvironmentMismatch));
        }
        let native = matches!(
            metadata.creation_cause(),
            crate::DomDocumentCreationCause::Native
        );
        if native
            && (metadata.encoding().code_units() != [85, 84, 70, 45, 56]
                || metadata.url().code_units()
                    != [97, 98, 111, 117, 116, 58, 98, 108, 97, 110, 107])
        {
            return Err(failure(Failure::EnvironmentMismatch));
        }
        let base = node.base_document().ok_or(DomError::WrongNodeKind)?;
        let facts = base
            .facts
            .ok_or_else(|| failure(Failure::Base(HtmlBaseMissing::DocumentFacts)))?;
        let encoding = facts
            .document()
            .encoding()
            .ok_or_else(|| failure(Failure::Base(HtmlBaseMissing::CharacterEncoding)))?;
        if !native {
            step()?;
            let url = facts
                .document()
                .document_url()
                .ok_or_else(|| failure(Failure::Base(HtmlBaseMissing::DocumentUrl)))?;
            if !super::base::parsed_document_inputs_match(
                metadata,
                url,
                encoding,
                Some(facts.document()),
            )
            .map_err(|missing| failure(Failure::Base(missing)))?
            {
                return Err(failure(Failure::EnvironmentMismatch));
            }
        }
        let first = if let Some(key) = base.first {
            step()?;
            HtmlDocumentFirstBase::Present(
                self.node(key)?
                    .base()
                    .and_then(|b| b.frozen)
                    .map(|f| f.value),
            )
        } else {
            HtmlDocumentFirstBase::Absent
        };
        step()?;
        let value = surgeist_html_semantics::select_document_base(Some(facts.document()), first)
            .map_err(|missing| failure(Failure::Base(missing)))?;
        Ok((value, encoding, facts.document().revision()))
    }
    /// Resolve an actual encoding-parse operand without fabricating an href or
    /// hyperlink-cache history. The canonical bank and its cumulative work are
    /// shared with reached A/Area parsing in this same source operation.
    pub(super) fn resolve_encoding_url_units(
        &self,
        document: Key,
        raw: &[u16],
        cancel: &AtomicBool,
    ) -> Result<&surgeist_html_semantics::HtmlEncodingUrlParseOutcome, DomError> {
        self.encoding_url_step(cancel)?;
        let (base, encoding, revision) = self.encoding_url_environment(document, cancel)?;
        let facts = self
            .node(document)?
            .encoding_url_document()
            .as_ref()
            .and_then(|document| document.facts)
            .ok_or_else(|| failure(Failure::MissingFacts))?;
        facts
            .lookup_units_with(raw, base, encoding, revision, || {
                self.encoding_url_step(cancel)
            })
            .map_err(|error| match error {
                HtmlEncodingUrlLookupError::Facts(error) => failure(Failure::Producer(error)),
                HtmlEncodingUrlLookupError::Interrupted(error) => error,
            })
    }
    pub(super) fn bind_encoding_url_facts(
        &mut self,
        basis: crate::DomDocumentStamp,
        facts: &HtmlEncodingUrlParseFacts,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if basis.store().owner() != self.store {
            return Err(DomError::ForeignOwner);
        }
        if basis.store().revision() != self.basis.revision() {
            return Err(failure(Failure::BasisRevisionMismatch));
        }
        let key = self.resolve(basis.document().into())?;
        let (base, encoding, revision) = self.encoding_url_environment(key, cancel)?;
        self.encoding_url_step(cancel)?;
        facts
            .validate_inputs(base, encoding, revision)
            .map_err(|e| failure(Failure::Producer(e)))?;
        let previous_records = self
            .node(key)?
            .encoding_url_document()
            .map_or(0, |value| value.records());
        let records = self
            .counts
            .encoding_observations
            .checked_sub(previous_records)
            .and_then(|records| records.checked_add(facts.formation_usage().record_capacity()))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            records,
            limits.encoding_url_observations,
            DomResource::EncodingUrlObservations,
        )?;
        let previous = self
            .node(key)?
            .encoding_url_document()
            .as_ref()
            .expect("native Document")
            .units();
        let units = (self.units - previous)
            .checked_add(facts.formation_usage().utf16_extent())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.encoding_url_step(cancel)?;
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::EncodingFacts,
            cancel,
        )?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut payload = source::storage::Payload::default();
        payload.producer(facts.retained_storage_usage())?;
        self.replace_encoding_url_document_formed_from(
            key,
            payload,
            formation,
            |_| {
                let facts = facts.clone();
                #[cfg(test)]
                source::formation::mutation_observation::grew_payload(
                    growth,
                    facts.retained_storage_usage().retained_bytes(),
                    facts.retained_storage_usage().utf16_extent(),
                );
                Ok(Some(crate::hyperlink::EncodingUrlDocument {
                    facts: Some(facts),
                }))
            },
            cancel,
        )?;
        self.units = units;
        Ok(())
    }
    fn store_hyperlink_parse(
        &mut self,
        key: Key,
        value: RetainedParse,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self
            .node(key)?
            .hyperlink()
            .as_ref()
            .expect("hyperlink")
            .units();
        let units = (self.units - old)
            .checked_add(value.value.retained_utf16_extent())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let document = self.node(key)?.document();
        let RetainedParse { value, payload } = value;
        {
            let mut fields = self.hyperlink_fields_mut(key, cancel)?;
            (*fields).as_mut().expect("hyperlink").parsed = Some(Parsed { document, value });
            fields.commit_charged(payload, cancel)
        }?;
        self.units = units;
        Ok(())
    }
    pub(super) fn set_hyperlink_url(
        &mut self,
        key: Key,
        href: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut cursor = HtmlHyperlinkSetUrlCursor::new();
        let mut parsed = None;
        loop {
            // Creation/actual removal uses the admitted attribute owner's known
            // absent value and returns before qualified parse-work admission.
            self.effect_step(cancel)?;
            let reply = match cursor.action() {
                Action::ClearUrl => {
                    let previous = self
                        .node(key)?
                        .hyperlink()
                        .as_ref()
                        .expect("actual A/Area")
                        .units();
                    {
                        let mut fields = self.hyperlink_fields_mut(key, cancel)?;
                        (*fields).as_mut().expect("hyperlink").parsed = None;
                        fields.commit(cancel)
                    }?;
                    self.units -= previous;
                    Reply::Applied
                }
                Action::ReadHref => Reply::HrefPresent(href.is_some()),
                Action::ParseHref => {
                    self.encoding_url_step(cancel)?;
                    let document = self.node(key)?.document();
                    let (base, encoding, revision) =
                        self.encoding_url_environment(document, cancel)?;
                    let facts = self
                        .node(document)?
                        .encoding_url_document()
                        .as_ref()
                        .and_then(|d| d.facts)
                        .ok_or_else(|| failure(Failure::MissingFacts))?;
                    #[cfg(test)]
                    let snapshot_growth = std::cell::Cell::new(None);
                    let mut retained = None;
                    let value = facts
                        .resolve_hyperlink_with(
                            href.expect("actual present href"),
                            base,
                            encoding,
                            revision,
                            || self.encoding_url_step(cancel),
                            |usage| {
                                #[cfg(test)]
                                snapshot_growth
                                    .set(source::formation::mutation_observation::before(
                                    self,
                                    source::formation::mutation_observation::Site::HyperlinkSnapshot,
                                    cancel,
                                )?);
                                let formation = source::formation::Formation::new(self, cancel)?;
                                if let Some(budget) = formation.as_ref() {
                                    let mut payload = source::storage::Payload::default();
                                    payload.producer(usage)?;
                                    budget.retain_payload(payload, cancel)?;
                                }
                                retained = formation.map(source::formation::Formation::into_charge);
                                Ok(())
                            },
                        )
                        .map_err(|e| match e {
                            HtmlEncodingUrlLookupError::Facts(e) => failure(Failure::Producer(e)),
                            HtmlEncodingUrlLookupError::Interrupted(e) => e,
                        })?;
                    #[cfg(test)]
                    if snapshot_growth.get().is_some() {
                        source::formation::mutation_observation::grew_payload(
                            snapshot_growth.get(),
                            value.retained_storage_usage().retained_bytes(),
                            value.retained_storage_usage().utf16_extent(),
                        );
                    }
                    let success = value.url().is_some();
                    let value = RetainedParse {
                        value,
                        payload: retained,
                    };
                    if success {
                        parsed = Some(value);
                    } else {
                        // Observed failure retains meaningful input qualification,
                        // while the associated URL remains the already-cleared null.
                        self.store_hyperlink_parse(key, value, limits, cancel)?;
                    }
                    Reply::Parsed(success)
                }
                Action::StoreParsedUrl => {
                    self.store_hyperlink_parse(
                        key,
                        parsed.take().expect("actual successful parse"),
                        limits,
                        cancel,
                    )?;
                    Reply::Applied
                }
                Action::Complete => return Ok(()),
            };
            cursor
                .reply(reply)
                .expect("completed actual shared set-URL action");
        }
    }
    pub(super) fn hyperlink_attribute_change(
        &mut self,
        key: Key,
        name: &crate::DomQualifiedName,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(key)?.hyperlink().is_none() || name.namespace_uri().is_some() {
            return Ok(());
        }
        let Some(attribute) = KnownHtmlAttribute::from_name(name) else {
            return Ok(());
        };
        let equal = match (old, new) {
            (None, None) => true,
            (Some(a), Some(b)) if a.len() == b.len() => {
                let mut equal = true;
                for (a, b) in a.code_units().iter().zip(b.code_units()) {
                    self.effect_step(cancel)?;
                    if a != b {
                        equal = false;
                        break;
                    }
                }
                equal
            }
            _ => false,
        };
        match surgeist_html::hyperlink_attribute_effect(
            &name
                .namespace_uri()
                .cloned()
                .map_or_else(crate::Namespace::none, crate::Namespace::from_uri),
            attribute,
            equal,
        ) {
            HtmlHyperlinkAttributeEffect::None => Ok(()),
            HtmlHyperlinkAttributeEffect::SetUrlThenConsiderSpeculativeLoads => {
                self.set_hyperlink_url(key, new, limits, cancel)?;
                self.hyperlink_speculation(self.node(key)?.document(), cancel)
            }
            HtmlHyperlinkAttributeEffect::ConsiderSpeculativeLoads => {
                self.hyperlink_speculation(self.node(key)?.document(), cancel)
            }
        }
    }
    fn hyperlink_speculation(&self, document: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        self.effect_step(cancel)?;
        let transaction::KindRead::Document(metadata) = &self.node(document)?.kind() else {
            return Err(DomError::WrongNodeKind);
        };
        // HTML877d76574–76588: never-active-in-a-navigable establishes null node
        // navigable, so the not-top-level guard returns before flags/tasks.
        if metadata.document_environment().has_ever_had_navigable() {
            return Err(DomError::HyperlinkProcessingUnavailable);
        }
        Ok(())
    }
    pub(super) fn hyperlink_tree_change(
        &self,
        key: Key,
        old_ancestor: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(key)?.hyperlink().is_none() {
            return Ok(());
        }
        let (mutation, document) = if let Some(old) = old_ancestor {
            (
                HtmlHyperlinkTreeMutation::Removal {
                    old_ancestor_connected: self.shadow_connected(old, cancel)?,
                },
                self.node(old)?.document(),
            )
        } else {
            (
                HtmlHyperlinkTreeMutation::Insertion {
                    connected: self.shadow_connected(key, cancel)?,
                },
                self.node(key)?.document(),
            )
        };
        if surgeist_html::hyperlink_tree_considers_speculative_loads(mutation) {
            self.hyperlink_speculation(document, cancel)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Namespace;
    use surgeist_html_semantics::*;
    fn policy(work: u64, units: usize) -> DomLimits {
        DomLimits::new(
            64,
            units,
            256,
            128,
            64,
            8,
            131072,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(64, 1_000_000)
        .unwrap()
        .with_encoding_url_processing(64, work)
        .unwrap()
    }
    fn fixture(
        capacity: usize,
    ) -> (
        Dom,
        DocumentId,
        NodeId,
        HtmlEncodingUrlParseFacts,
        HtmlEncodingUrlParseFacts,
    ) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(policy(1_000_000, 65536)).unwrap();
        let mut b = DomBatch::new();
        let doc = b.create_document();
        let a = b.create_element_ns(&doc, Namespace::html(), "a");
        let c = dom.apply(b, &cancel).unwrap();
        let doc = c.document(&doc).unwrap();
        let a = c.created(&a).unwrap().unwrap();
        let r = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let encoding = HtmlBaseEncoding::fresh();
        let base = HtmlBaseUrlRecord::observed(
            r.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
            None,
        )
        .unwrap();
        let env = HtmlBaseDocumentFacts::new(
            r.clone(),
            Some(base.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap();
        let env =
            HtmlParseBaseFacts::new(env, vec![], HtmlBaseFactsLimits::new(64, 65536, 1_000_000))
                .unwrap();
        let url = HtmlBaseUrlRecord::observed(
            r.clone(),
            "https://example.test/".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap();
        let bank = |capacity: usize| {
            let mut raw = Vec::with_capacity(capacity);
            raw.extend("target".encode_utf16());
            HtmlEncodingUrlParseFacts::new(
                base.clone(),
                encoding.clone(),
                r.clone(),
                vec![
                    HtmlEncodingUrlParseObservation::new(
                        DocumentString::from_utf16(raw),
                        HtmlEncodingUrlParseOutcome::Parsed(url.clone()),
                    ),
                    HtmlEncodingUrlParseObservation::new(
                        "next".into(),
                        HtmlEncodingUrlParseOutcome::Parsed(url.clone()),
                    ),
                ],
                HtmlBaseFactsLimits::new(64, 65536, 1_000_000),
            )
            .unwrap()
        };
        let original = bank(capacity);
        let alternative = bank(if capacity == 64 { 8 } else { 64 });
        let mut b = DomBatch::new();
        b.bind_base_facts(dom.snapshot().document(doc).unwrap().stamp(), &env);
        b.bind_encoding_url_facts(dom.snapshot().document(doc).unwrap().stamp(), &original);
        b.set_attribute(a, "href", "target");
        dom.apply(b, &cancel).unwrap();
        (dom, doc, a, original, alternative)
    }
    fn edit(a: NodeId, values: &[&str]) -> DomBatch {
        let mut b = DomBatch::new();
        for value in values {
            b.set_attribute(a, "href", *value);
        }
        b
    }
    fn capacity_edit(
        dom: &Dom,
        doc: DocumentId,
        a: NodeId,
        original: &HtmlEncodingUrlParseFacts,
        alternative: &HtmlEncodingUrlParseFacts,
        semantic: bool,
    ) -> DomBatch {
        let stamp = dom.snapshot().document(doc).unwrap().stamp();
        let mut b = DomBatch::new();
        b.bind_encoding_url_facts(stamp, alternative);
        b.set_attribute(a, "href", "next");
        b.set_attribute(a, "href", "target");
        b.bind_encoding_url_facts(stamp, original);
        if semantic {
            b.set_attribute(a, "title", "changed");
        }
        b
    }
    fn source_snapshot_alias(
        bytes: usize,
        bind: bool,
    ) -> Option<source::formation::mutation_observation::Growth> {
        source_snapshot_alias_limits(bytes, 10_000_000, bind)
    }
    fn source_snapshot_alias_limits(
        bytes: usize,
        units: usize,
        bind: bool,
    ) -> Option<source::formation::mutation_observation::Growth> {
        use surgeist_html_semantics::*;
        let cancel = AtomicBool::new(false);
        let domain = DomLimits::new(
            32,
            1_000_000,
            64,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(32, 1_000_000)
        .unwrap()
        .with_encoding_url_processing(32, 1_000_000)
        .unwrap();
        let mut dom = Dom::new(domain).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let anchor = setup.create_element_ns(&document, crate::Namespace::html(), "a");
        setup.insert_before(&anchor, &document, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let anchor = created.created(&anchor).unwrap().unwrap();
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let base = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
            None,
        )
        .unwrap();
        let encoding = HtmlBaseEncoding::fresh();
        let base_facts = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(base.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding.clone()),
            )
            .unwrap(),
            Vec::new(),
            HtmlBaseFactsLimits::new(32, 1_000_000, 1_000_000),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        setup.bind_base_facts(
            dom.snapshot().document(document).unwrap().stamp(),
            &base_facts,
        );
        dom.apply(setup, &cancel).unwrap();
        let mut padded = Vec::with_capacity(65_536);
        padded.push(u16::from(b'r'));
        let facts = HtmlEncodingUrlParseFacts::new(
            base,
            encoding,
            revision,
            vec![HtmlEncodingUrlParseObservation::new(
                DocumentString::from_utf16(padded),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlBaseFactsLimits::new(32, 1_000_000, 1_000_000),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        setup.bind_encoding_url_facts(dom.snapshot().document(document).unwrap().stamp(), &facts);
        dom.apply(setup, &cancel).unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        if bind {
            batch.bind_encoding_url_facts(before.document(document).unwrap().stamp(), &facts);
        } else {
            batch.set_attribute(anchor, "href", "r");
        }
        let mut lease = match dom.begin_source_commands(
            batch,
            crate::DomSourceLimits::new(bytes, units, 100_000_000, 64, 4096, 4096, 1),
            &cancel,
        ) {
            Ok(lease) => lease,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            }) => return None,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                ..
            }) if units < 10_000_000 => return None,
            Err(error) => panic!("actual Source setup admission: {error:?}"),
        };
        source::formation::mutation_observation::start(if bind {
            source::formation::mutation_observation::Site::EncodingFacts
        } else {
            source::formation::mutation_observation::Site::HyperlinkSnapshot
        });
        let result = lease.advance(&cancel);
        let actual = source::formation::mutation_observation::finish();
        match result {
            Ok(crate::DomSourceProgress::Complete(_)) => {
                if !bind {
                    let snapshot = lease.snapshot();
                    let observation = snapshot
                        .node(anchor)
                        .unwrap()
                        .as_hyperlink()
                        .unwrap()
                        .parse_observation()
                        .unwrap();
                    assert!(observation.url().is_none());
                    assert_eq!(observation.raw().code_units(), &[u16::from(b'r')]);
                    assert!(observation.raw().shared_code_units().capacity() >= 65_536);
                }
            }
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            }) => assert_eq!(lease.snapshot().stamp(), before.stamp()),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                ..
            }) if units < 10_000_000 => assert_eq!(lease.snapshot().stamp(), before.stamp()),
            other => panic!("actual Source hyperlink result: {other:?}"),
        }
        actual
    }
    #[test]
    fn source_hyperlink_snapshot_admits_selected_spare_backing_before_producer_clones() {
        let high =
            source_snapshot_alias(16_000_000, false).expect("actual selected provider snapshot");
        if let Some(actual) = source_snapshot_alias(high.required - 1, false) {
            assert!(
                actual.required <= actual.limit,
                "selected snapshot retained {} under {}",
                actual.required,
                actual.limit
            );
        }
    }
    #[test]
    fn source_encoding_facts_admits_canonical_retain_before_whole_bank_clone() {
        let high = source_snapshot_alias(16_000_000, true).expect("actual whole fact bank retain");
        if let Some(actual) = source_snapshot_alias(high.required - 1, true) {
            assert!(
                actual.required <= actual.limit,
                "fact bank retained {} under {}",
                actual.required,
                actual.limit
            );
        }
    }
    #[test]
    fn source_selected_and_fact_occurrences_have_exact_and_one_short_utf16_admission() {
        for bind in [false, true] {
            let high = source_snapshot_alias_limits(16_000_000, 10_000_000, bind).unwrap();
            let exact = source_snapshot_alias_limits(16_000_000, high.required_units, bind)
                .expect("exact selected occurrence is formed");
            assert_eq!(exact.required_units, exact.unit_limit);
            assert!(
                source_snapshot_alias_limits(16_000_000, high.required_units - 1, bind).is_none(),
                "one-short must refuse before the actual selected retain"
            );
        }
    }
    #[test]
    fn capacity_only_candidates_keep_old_backing_usage_and_revision_in_both_directions() {
        for capacity in [8, 64] {
            let (mut dom, doc, a, original, alternative) = fixture(capacity);
            dom.revision = DomRevision(u64::MAX);
            let old = dom.snapshot();
            let units = dom.usage().string_code_units();
            let old_extent = old
                .node(a)
                .unwrap()
                .as_hyperlink()
                .unwrap()
                .parse_observation()
                .unwrap()
                .retained_utf16_extent();
            let batch = capacity_edit(&dom, doc, a, &original, &alternative, false);
            let prepared = dom.prepare(batch, &AtomicBool::new(false)).unwrap();
            assert_ne!(
                prepared.state.units, units,
                "real selected cache backing differs"
            );
            let c = dom.publish(prepared, &AtomicBool::new(false)).unwrap();
            assert!(c.changes().changes().is_empty());
            assert_eq!(dom.snapshot().stamp(), old.stamp());
            assert_eq!(dom.usage().string_code_units(), units);
            assert_eq!(
                dom.snapshot()
                    .node(a)
                    .unwrap()
                    .as_hyperlink()
                    .unwrap()
                    .parse_observation()
                    .unwrap()
                    .retained_utf16_extent(),
                old_extent
            );
            assert_eq!(
                old.node(a)
                    .unwrap()
                    .as_hyperlink()
                    .unwrap()
                    .parse_observation()
                    .unwrap()
                    .retained_utf16_extent(),
                old_extent
            );
        }
    }
    #[test]
    fn capacity_only_candidate_still_admits_storage_and_cancellation_before_discard() {
        let (mut dom, doc, a, original, alternative) = fixture(8);
        let old = dom.snapshot();
        let units = dom.usage().string_code_units();
        let b = capacity_edit(&dom, doc, a, &original, &alternative, false);
        let prepared = dom.prepare(b, &AtomicBool::new(false)).unwrap();
        assert!(prepared.state.units > units);
        let exact = prepared.state.units;
        assert_eq!(
            dom.publish(prepared, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        dom.set_limits(policy(1_000_000, exact - 1)).unwrap();
        assert!(matches!(
            dom.apply(
                capacity_edit(&dom, doc, a, &original, &alternative, false),
                &AtomicBool::new(false)
            ),
            Err(DomError::ResourceLimit {
                resource: DomResource::StringCodeUnits,
                ..
            })
        ));
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        assert_eq!(dom.usage().string_code_units(), units);
    }
    #[test]
    fn real_semantic_change_publishes_actual_candidate_usage_and_exhaustion_is_atomic() {
        for capacity in [8, 64] {
            let (mut dom, doc, a, original, alternative) = fixture(capacity);
            let old = dom.snapshot();
            let b = capacity_edit(&dom, doc, a, &original, &alternative, true);
            let prepared = dom.prepare(b, &AtomicBool::new(false)).unwrap();
            let units = prepared.state.units;
            let c = dom.publish(prepared, &AtomicBool::new(false)).unwrap();
            assert_ne!(dom.snapshot().stamp(), old.stamp());
            assert_eq!(dom.usage().string_code_units(), units);
            assert!(
                c.changes()
                    .changes()
                    .iter()
                    .any(|c| matches!(c,DomChange::Attributes(id) if *id==a))
            );
            dom.revision = DomRevision(u64::MAX);
            let old = dom.snapshot();
            assert_eq!(
                dom.apply(edit(a, &["next"]), &AtomicBool::new(false))
                    .unwrap_err(),
                DomError::RevisionExhausted
            );
            assert_eq!(dom.snapshot().stamp(), old.stamp());
            assert_eq!(dom.usage().string_code_units(), units);
        }
    }
    #[test]
    fn repeated_actual_parses_share_work_and_clear_precedes_missing_observation() {
        let (mut dom, _, a, _, _) = fixture(8);
        let cancel = AtomicBool::new(false);
        let usage = |b: DomBatch| {
            let mut d = Draft::new(&dom, b.owner, &cancel).unwrap();
            for command in b.commands {
                d.apply(&command, dom.limits, &cancel).unwrap();
            }
            d.encoding_url_work.get()
        };
        let one = usage(edit(a, &["next"]));
        let total = usage(edit(a, &["next", "target"]));
        assert!(total > one);
        dom.set_limits(policy(one, 65536)).unwrap();
        let old = dom.snapshot();
        assert_eq!(
            dom.apply(edit(a, &["next", "target"]), &cancel)
                .unwrap_err(),
            DomError::EncodingUrlWorkLimit { limit: one }
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        dom.set_limits(policy(total, 65536)).unwrap();
        dom.apply(edit(a, &["next", "target"]), &cancel).unwrap();
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        let b = edit(a, &["unobserved"]);
        let mut d = Draft::new(&dom, b.owner, &cancel).unwrap();
        assert_eq!(
            d.apply(&b.commands.into_iter().next().unwrap(), dom.limits, &cancel)
                .unwrap_err(),
            failure(Failure::Producer(
                HtmlEncodingUrlFactsError::MissingObservation
            ))
        );
        assert!(
            d.node(d.resolve(a.into()).unwrap())
                .unwrap()
                .hyperlink()
                .unwrap()
                .parsed
                .is_none()
        );
        assert!(old.node(a).unwrap().as_hyperlink().unwrap().url().is_some());
        assert_eq!(dom.snapshot().stamp(), old.stamp());
    }
    #[test]
    fn actual_anchor_name_and_id_hooks_refuse_nonempty_render_blocking_state() {
        let (dom, doc, a, _, _) = fixture(8);
        let cancel = AtomicBool::new(false);
        for attribute in ["name", "id"] {
            let mut b = DomBatch::new();
            b.set_attribute(a, attribute, "target");
            let mut d = Draft::new(&dom, b.owner, &cancel).unwrap();
            let document = d.resolve(doc.into()).unwrap();
            let key = d.resolve(a.into()).unwrap();
            d.relations_insert(
                document,
                transaction::RelationField::RenderBlocking,
                0,
                &[key],
                &cancel,
            )
            .unwrap();
            assert_eq!(
                d.apply(&b.commands.into_iter().next().unwrap(), dom.limits, &cancel)
                    .unwrap_err(),
                DomError::LinkProcessingUnavailable
            );
        }
    }
}
