//! Original OptionValue capture/query under the current synchronous envelope.
use super::*;
use source::checkpoint::Charge;
use source::storage::Payload;
use surgeist_html_semantics::*;

/// The actual returned occurrence drops before its accounting permission.
pub(super) struct Value {
    pub(super) value: DocumentString,
    pub(super) payload: Option<Charge>,
}

/// The actual producer graph drops before its cached admitted graph charge.
struct Producer {
    owner: HtmlControlledOptionValueCapture,
    charge: Option<Charge>,
    usage: HtmlControlledOptionValueUsage,
}
impl Producer {
    fn new(
        draft: &Draft,
        limits: HtmlCaptureLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        // Do not retain Formation's snapshot across mapping growth. Only its
        // empty movable charge survives; each allowance is derived anew.
        let charge = source::formation::Formation::new(draft, cancel)?
            .map(source::formation::Formation::into_charge);
        let envelope = draft.option_value_envelope(Default::default(), cancel)?;
        let result = HtmlControlledOptionValueCapture::new(
            limits,
            HtmlHostFactsRevision::fresh(),
            envelope,
            cancel,
        );
        let owner = match result {
            Ok(owner) => owner,
            Err(failure) => {
                let usage = failure.usage();
                draft.option_value_usage(Default::default(), usage)?;
                return Err(draft.option_value_error(failure.into_error()));
            }
        };
        let mut producer = Self {
            owner,
            charge,
            usage: Default::default(),
        };
        producer.sync(draft)?;
        Ok(producer)
    }
    fn sync(&mut self, draft: &Draft) -> Result<(), DomError> {
        let usage = self.owner.usage();
        if let Some(charge) = &mut self.charge {
            charge.replace_admitted_payload(Payload {
                bytes: usage.storage().retained_bytes(),
                units: usage.storage().utf16_extent(),
            });
        }
        draft.option_value_usage(self.usage, usage)?;
        self.usage = usage;
        Ok(())
    }
    fn call<T>(
        &mut self,
        draft: &Draft,
        cancel: &AtomicBool,
        call: impl FnOnce(
            &mut HtmlControlledOptionValueCapture,
            HtmlOptionValueEnvelope,
        ) -> Result<T, HtmlControlledOptionValueError>,
    ) -> Result<T, DomError> {
        let envelope = draft.option_value_envelope(self.usage.storage(), cancel)?;
        // The callback moves only the original typed error. The surviving
        // owner still supplies complete all-outcome usage before propagation.
        let result = call(&mut self.owner, envelope);
        // No intervening allocation, work or cancellation can separate the
        // producer's admitted actual graph from its cached shared-ledger debit.
        self.sync(draft)?;
        let value = result.map_err(|error| draft.option_value_error(error))?;
        #[cfg(test)]
        observation::joined(draft, self.usage.storage(), cancel)?;
        Ok(value)
    }
}

impl Draft {
    fn option_value_envelope(
        &self,
        current: HtmlOptionValueStorageUsage,
        cancel: &AtomicBool,
    ) -> Result<HtmlOptionValueEnvelope, DomError> {
        let Some(limits) = self.source_limits else {
            return Ok(HtmlOptionValueEnvelope::new(
                usize::MAX,
                usize::MAX,
                u64::MAX,
            ));
        };
        let bytes = self
            .source_storage_bytes(cancel)?
            .checked_add(self.source_other_bytes)
            .and_then(|n| n.checked_sub(current.retained_bytes()))
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .source_storage_units(cancel)?
            .checked_add(self.source_other_units)
            .and_then(|n| n.checked_sub(current.utf16_extent()))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            bytes,
            limits.retained_bytes,
            DomResource::SourceRetainedBytes,
        )?;
        admit(
            units,
            limits.retained_units,
            DomResource::SourceRetainedUnits,
        )?;
        let work = limits
            .work
            .checked_sub(self.source_work.load(Ordering::Relaxed))
            .ok_or(DomError::SourceWorkLimit { limit: limits.work })?;
        Ok(HtmlOptionValueEnvelope::new(
            limits.retained_bytes - bytes,
            limits.retained_units - units,
            crate::source::event::remaining_ancestor_work(self.source_ancestors.as_ref(), work)?,
        ))
    }
    fn option_value_usage(
        &self,
        before: HtmlControlledOptionValueUsage,
        after: HtmlControlledOptionValueUsage,
    ) -> Result<(), DomError> {
        let work_delta = after
            .work()
            .checked_sub(before.work())
            .ok_or(DomError::RepresentationExhausted)?;
        let semantic_work = after
            .semantic()
            .work()
            .checked_sub(before.semantic().work())
            .and_then(|n| self.select_semantic_work.get().checked_add(n))
            .ok_or(DomError::RepresentationExhausted)?;
        let semantic_bytes = after
            .semantic()
            .allocated_bytes()
            .checked_sub(before.semantic().allocated_bytes())
            .and_then(|n| self.select_semantic_bytes.get().checked_add(n))
            .ok_or(DomError::RepresentationExhausted)?;
        if self.source_limits.is_some() {
            let work = self
                .source_work
                .load(Ordering::Relaxed)
                .checked_add(work_delta)
                .ok_or(DomError::RepresentationExhausted)?;
            self.source_work.store(work, Ordering::Relaxed);
        }
        self.select_semantic_work.set(semantic_work);
        self.select_semantic_bytes.set(semantic_bytes);
        if self.source_limits.is_some() {
            crate::source::event::retain_ancestor_work(self.source_ancestors.as_ref(), work_delta)?;
        }
        Ok(())
    }
    fn option_value_error(&self, error: HtmlControlledOptionValueError) -> DomError {
        match error {
            HtmlControlledOptionValueError::Control(error) => {
                self.option_value_control_error(error)
            }
            HtmlControlledOptionValueError::Capture(error) => DomError::SelectCapture(error),
            HtmlControlledOptionValueError::Semantic(error) => {
                if let HtmlSemanticFailureKind::ControlledOptionValue(parent) = error.kind() {
                    self.option_value_control_error(parent.clone())
                } else {
                    DomError::SelectSemantic(error)
                }
            }
            HtmlControlledOptionValueError::AllocationFailed => DomError::AllocationFailed,
            HtmlControlledOptionValueError::Context(_)
            | HtmlControlledOptionValueError::InvalidPhase => DomError::SelectProtocol,
        }
    }
    fn option_value_control_error(&self, error: HtmlOptionValueControlError) -> DomError {
        match error {
            HtmlOptionValueControlError::Cancelled => DomError::Cancelled,
            HtmlOptionValueControlError::RepresentationExhausted => {
                DomError::RepresentationExhausted
            }
            HtmlOptionValueControlError::LimitExceeded { resource, .. } => {
                let limits = self
                    .source_limits
                    .expect("only a bounded Source allowance can be exceeded");
                match resource {
                    HtmlOptionValueResource::LiveBytes => DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: limits.retained_bytes,
                    },
                    HtmlOptionValueResource::LiveUtf16 => DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedUnits,
                        limit: limits.retained_units,
                    },
                    HtmlOptionValueResource::Work => {
                        DomError::SourceWorkLimit { limit: limits.work }
                    }
                }
            }
        }
    }
    fn option_value_header<T>(&self, cancel: &AtomicBool) -> Result<Option<Charge>, DomError> {
        let formation = source::formation::Formation::new(self, cancel)?;
        if let Some(formation) = &formation {
            formation.retain_bytes(size_of::<ScratchVec<T>>(), cancel)?;
        }
        Ok(formation.map(source::formation::Formation::into_charge))
    }
    pub(super) fn controlled_option_value(
        &self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Value, DomError> {
        self.select_step(cancel)?;
        let (capture_limits, semantic_limits) = limits
            .select_value_queries
            .ok_or(DomError::SelectValueQueriesUnconfigured)?;
        let mode = match &self.node(self.node(key)?.document)?.kind {
            DomNodeKind::Document(document) => document.mode(),
            _ => return Err(DomError::WrongDocument),
        };
        let mut producer = Producer::new(self, capture_limits, cancel)?;
        let document = producer.call(self, cancel, |owner, envelope| {
            owner
                .add_document(mode, envelope, cancel)
                .map_err(HtmlControlledOptionValueFailure::into_error)
        })?;
        // Keep only the original ordinary subtree and its original source order.
        // Each actual mapping bank drops before its own fixed-header permission.
        let _entries_header =
            self.option_value_header::<(Key, HtmlCaptureBuilderSubject)>(cancel)?;
        let mut entries = self.select_vec();
        let mut next = Some(key);
        while let Some(current) = next {
            self.select_step(cancel)?;
            let subject = match &self.node(current)?.kind {
                DomNodeKind::Element(element) => producer.call(self, cancel, |owner, envelope| {
                    owner
                        .add_element(
                            &document,
                            element.name(),
                            element.attributes(),
                            element.saved_is(),
                            envelope,
                            cancel,
                        )
                        .map_err(HtmlControlledOptionValueFailure::into_error)
                }),
                DomNodeKind::Text(value) => producer.call(self, cancel, |owner, envelope| {
                    owner
                        .add_text(&document, value, envelope, cancel)
                        .map_err(HtmlControlledOptionValueFailure::into_error)
                }),
                DomNodeKind::Comment(value) => producer.call(self, cancel, |owner, envelope| {
                    owner
                        .add_comment(&document, value, envelope, cancel)
                        .map_err(HtmlControlledOptionValueFailure::into_error)
                }),
                DomNodeKind::ProcessingInstruction(pi) => {
                    producer.call(self, cancel, |owner, envelope| {
                        owner
                            .add_processing_instruction(
                                &document,
                                pi.target(),
                                pi.data(),
                                envelope,
                                cancel,
                            )
                            .map_err(HtmlControlledOptionValueFailure::into_error)
                    })
                }
                DomNodeKind::DocumentType(doctype) => {
                    producer.call(self, cancel, |owner, envelope| {
                        owner
                            .add_doctype(
                                &document,
                                doctype.name(),
                                Some(doctype.public_id()),
                                Some(doctype.system_id()),
                                envelope,
                                cancel,
                            )
                            .map_err(HtmlControlledOptionValueFailure::into_error)
                    })
                }
                _ => return Err(DomError::SelectProtocol),
            }?;
            self.select_reserve(&mut entries, 1, cancel)?;
            entries.push((current, subject));
            next =
                crate::slot::next_admitted(self, key, current, &mut || self.select_step(cancel))?;
        }
        for (current, subject) in entries.iter() {
            self.select_step(cancel)?;
            if !matches!(self.node(*current)?.kind, DomNodeKind::Element(_)) {
                continue;
            }
            let _children_header = self.option_value_header::<HtmlCaptureBuilderSubject>(cancel)?;
            let mut children = self.select_vec();
            for child in self.node(*current)?.children.iter() {
                let mut found = None;
                for (candidate, subject) in entries.iter() {
                    self.select_step(cancel)?;
                    if *candidate == child {
                        found = Some(subject.clone());
                        break;
                    }
                }
                self.select_reserve(&mut children, 1, cancel)?;
                children.push(found.ok_or(DomError::SelectProtocol)?);
            }
            producer.call(self, cancel, |owner, envelope| {
                owner
                    .establish_children(subject, &children, envelope, cancel)
                    .map_err(HtmlControlledOptionValueFailure::into_error)
            })?;
            self.select_step(cancel)?;
        }
        producer.call(self, cancel, |owner, envelope| {
            owner
                .finish(envelope, cancel)
                .map_err(HtmlControlledOptionValueFailure::into_error)
        })?;
        let remaining = HtmlSemanticLimits::new(
            semantic_limits
                .work()
                .saturating_sub(self.select_semantic_work.get()),
            semantic_limits.output_records(),
            semantic_limits.scratch_records(),
            semantic_limits.integer_digits(),
            semantic_limits
                .allocated_bytes()
                .saturating_sub(self.select_semantic_bytes.get()),
            semantic_limits.nested_output_records(),
        );
        producer.call(self, cancel, |owner, envelope| {
            owner
                .query_option_value(
                    &entries[0].1,
                    HtmlValueInterpreter::new(limits.units),
                    remaining,
                    envelope,
                    cancel,
                )
                .map(|_| ())
                .map_err(HtmlControlledOptionValueFailure::into_error)
        })?;
        self.select_step(cancel)?;
        let report = producer.owner.report().ok_or(DomError::SelectProtocol)?;
        match report.results()[0].fact() {
            HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(readback)) => {
                let (value, payload) =
                    source::formation::shared_string(self, readback.value(), cancel)?;
                Ok(Value { value, payload })
            }
            _ => Err(DomError::SelectProtocol),
        }
    }
}

#[cfg(test)]
pub(super) mod observation {
    use super::*;
    use std::cell::Cell;
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Boundary {
        pub required: usize,
        pub limit: usize,
        pub producer: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Boundary>> = const { Cell::new(None) };
    }
    pub(super) fn start() {
        LAST.set(None);
        ACTIVE.set(true);
    }
    pub(super) fn finish() -> Option<Boundary> {
        ACTIVE.set(false);
        LAST.take()
    }
    pub(super) fn joined(
        draft: &Draft,
        producer: HtmlOptionValueStorageUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !ACTIVE.get() {
            return Ok(());
        }
        let Some(limits) = draft.source_limits else {
            return Ok(());
        };
        let required = draft
            .source_storage_bytes(cancel)?
            .checked_add(draft.source_other_bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let boundary = Boundary {
            required,
            limit: limits.retained_bytes,
            producer: producer.retained_bytes(),
        };
        if LAST.get().is_none_or(|old| old.required < required) {
            LAST.set(Some(boundary));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

    fn fixture() -> (Dom, NodeId, NodeId, usize) {
        let limits = DomLimits::new(64, 1_000_000, 64, 0, 0, 8, 1_000_000)
            .unwrap()
            .with_select_processing(64, 1_000_000, 1_000_000)
            .unwrap()
            .with_select_value_queries(
                HtmlCaptureLimits::new(64, 128, 64, 1_000_000, 64, 0, 0),
                HtmlSemanticLimits::new(1_000_000, 1, 128, 128, 1_000_000, 128),
            );
        let mut dom = Dom::new(limits).unwrap();
        let cancel = AtomicBool::new(false);
        let mut raw = Vec::with_capacity(65_536);
        raw.extend("chosen".encode_utf16());
        let capacity = raw.capacity();
        let mut setup = DomBatch::new();
        let doc = setup.create_document();
        let select = setup.create_element_ns(&doc, Namespace::html(), "select");
        let option = setup.create_element_ns(&doc, Namespace::html(), "option");
        setup.set_attribute(&option, "value", DocumentString::from_utf16(raw));
        setup.insert_before(&option, &select, None);
        let committed = dom.apply(setup, &cancel).unwrap();
        (
            dom,
            committed.created(&select).unwrap().unwrap(),
            committed.created(&option).unwrap().unwrap(),
            capacity,
        )
    }
    fn limits(bytes: usize) -> crate::DomSourceLimits {
        crate::DomSourceLimits::new(bytes, 10_000_000, 100_000_000, 64, 4096, 4096, 1)
    }
    fn source_value(bytes: usize) -> (Option<observation::Boundary>, Result<(), DomError>) {
        let (mut dom, select, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_select_value(select, "chosen");
        let mut lease = match dom.begin_source_commands(batch, limits(bytes), &cancel) {
            Ok(lease) => lease,
            Err(error) => return (None, Err(error)),
        };
        observation::start();
        let result = lease.advance(&cancel).map(|progress| {
            assert!(matches!(progress, crate::DomSourceProgress::Complete(_)));
        });
        let boundary = observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        (boundary, result)
    }

    #[test]
    fn source_option_value_actual_capture_sharing_obeys_the_live_envelope() {
        let (high, result) = source_value(16_000_000);
        result.unwrap();
        let high = high.expect("the actual Source setter forms its real Option capture");
        assert!(
            high.producer >= 2 * 65_536,
            "actual spare attribute backing is retained"
        );
        for bytes in [high.required, high.required - 1] {
            let (boundary, result) = source_value(bytes);
            if let Some(actual) = boundary {
                assert!(
                    actual.required <= actual.limit,
                    "actual original capture was retained at {} bytes under {}",
                    actual.required,
                    actual.limit
                );
            } else {
                assert!(
                    matches!(
                        result,
                        Err(DomError::ResourceLimit {
                            resource: DomResource::SourceRetainedBytes,
                            ..
                        })
                    ),
                    "a nonreached capture must be refused by earlier real admission"
                );
            }
        }
    }

    #[test]
    fn source_option_value_return_keeps_its_shared_occurrence_until_drop() {
        let (dom, _, option, capacity) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        draft.source_limits = Some(limits(16_000_000));
        let option = draft.resolve(option.into()).unwrap();
        let before_bytes = draft.source_formation_bytes.load(Ordering::Relaxed);
        let before_units = draft.source_temporary_units.load(Ordering::Relaxed);
        let actual = draft.option_value(option, dom.limits, &cancel).unwrap();
        assert_eq!(
            draft.source_temporary_units.load(Ordering::Relaxed),
            before_units + capacity,
            "the returned value occurrence remains live after producer temporaries drop"
        );
        assert!(
            draft.source_formation_bytes.load(Ordering::Relaxed) >= before_bytes + 2 * capacity
        );
        drop(actual);
        assert_eq!(
            draft.source_formation_bytes.load(Ordering::Relaxed),
            before_bytes
        );
        assert_eq!(
            draft.source_temporary_units.load(Ordering::Relaxed),
            before_units
        );
    }
    fn query_policy(dom: &Dom, work: usize, bytes: usize, scratch: usize) -> DomLimits {
        let (capture, original) = dom.limits.select_value_queries.unwrap();
        dom.limits.with_select_value_queries(
            capture,
            HtmlSemanticLimits::new(
                work,
                original.output_records(),
                scratch,
                original.integer_digits(),
                bytes,
                original.nested_output_records(),
            ),
        )
    }

    #[test]
    fn source_option_value_constructor_refusal_keeps_work_without_retained_graph() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(DomLimits::new(16, 100_000, 32, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        draft.source_limits = Some(limits(16_000_000));
        let baseline = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
        // The producer documents its fixed public owner header exactly once.
        let bytes = baseline + size_of::<HtmlControlledOptionValueCapture>() - 1;
        let before = draft.source_work.load(Ordering::Relaxed);
        draft.source_limits = Some(crate::DomSourceLimits::new(
            bytes,
            0,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
        let mut spent = before;
        for _ in 0..3 {
            let error = Producer::new(&draft, HtmlCaptureLimits::new(4, 0, 0, 0, 0, 0, 0), &cancel)
                .err()
                .expect("header refused");
            assert_eq!(
                error,
                DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: bytes
                }
            );
            let after = draft.source_work.load(Ordering::Relaxed);
            assert!(
                after > spent,
                "every refused actual constructor keeps its spent work"
            );
            spent = after;
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        }
        draft.source_limits.as_mut().unwrap().work = spent;
        assert_eq!(
            Producer::new(&draft, HtmlCaptureLimits::new(4, 0, 0, 0, 0, 0, 0), &cancel).err(),
            Some(DomError::SourceWorkLimit { limit: spent })
        );
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
        assert_eq!(draft.select_semantic_work.get(), 0);
        assert_eq!(draft.select_semantic_bytes.get(), 0);
    }

    #[test]
    fn source_option_value_plain_parity_and_cancelled_comparison_keep_output_custody() {
        for source in [false, true] {
            let (dom, _, option, capacity) = fixture();
            let cancel = AtomicBool::new(false);
            let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
            if source {
                draft.source_limits = Some(limits(16_000_000));
            }
            let key = draft.resolve(option.into()).unwrap();
            let actual = draft.option_value(key, dom.limits, &cancel).unwrap();
            assert_eq!(actual.value.code_units(), [99, 104, 111, 115, 101, 110]);
            assert_eq!(actual.payload.is_some(), source);
            assert!(draft.select_semantic_work.get() > 0);
            assert!(draft.select_semantic_bytes.get() > 0);
            let units = if source { capacity } else { 0 };
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), units);
            let spent = draft.source_work.load(Ordering::Relaxed);
            cancel.store(true, Ordering::Relaxed);
            assert_eq!(
                draft.same_select_units(&actual.value, &"chosen".into(), &cancel),
                Err(DomError::Cancelled)
            );
            assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
            assert_eq!(
                draft.source_temporary_units.load(Ordering::Relaxed),
                units,
                "a fallible Setter comparison cannot release the still-live result occurrence"
            );
            drop(actual);
            assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        }
    }

    fn fallback_fixture() -> (Dom, NodeId) {
        let (mut dom, _, option, _) = fixture();
        let document = dom
            .snapshot()
            .node(option)
            .unwrap()
            .owner_document()
            .unwrap();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        batch.remove_attribute(option, "value");
        let first = batch.create_text(document, " \ta\n");
        batch.insert_before(&first, option, None);
        let script = batch.create_element_ns(document, Namespace::html(), "script");
        let mut spare = Vec::with_capacity(32_768);
        spare.extend("excluded".encode_utf16());
        let hidden = batch.create_text(document, DocumentString::from_utf16(spare));
        batch.insert_before(&hidden, &script, None);
        batch.insert_before(&script, option, None);
        let span = batch.create_element_ns(document, Namespace::html(), "span");
        let last = batch.create_text(document, "\rb\u{a0}c \n");
        batch.insert_before(&last, &span, None);
        batch.insert_before(&span, option, None);
        dom.apply(batch, &cancel).unwrap();
        (dom, option)
    }

    #[test]
    fn source_option_value_real_mapping_and_original_text_frames_preserve_literal_rules() {
        let (dom, option) = fallback_fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        draft.source_limits = Some(limits(16_000_000));
        let actual = draft
            .option_value(draft.resolve(option.into()).unwrap(), dom.limits, &cancel)
            .unwrap();
        // HTML-aware text excludes Script, collapses only ASCII whitespace,
        // and preserves NBSP. The real nested tree exercises both mapper banks.
        assert_eq!(actual.value.code_units(), [97, 32, 98, 160, 99]);
        assert_eq!(
            draft.source_temporary_units.load(Ordering::Relaxed),
            actual.value.shared_code_units().capacity()
        );
        assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
        drop(actual);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn source_option_value_original_frame_stop_keeps_child_usage_after_all_cleanup() {
        let (mut dom, option) = fallback_fixture();
        dom.set_limits(query_policy(&dom, 1_000_000, 1_000_000, 1))
            .unwrap();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        draft.source_limits = Some(limits(16_000_000));
        let error = draft
            .option_value(draft.resolve(option.into()).unwrap(), dom.limits, &cancel)
            .err()
            .expect("nested frame2 refused");
        let DomError::SelectSemantic(error) = error else {
            panic!("original qualified semantic stop")
        };
        assert_eq!(
            error.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::ScratchRecords,
                limit: 1,
                required: 2,
            }
        );
        assert!(error.qualification().is_some());
        // The original failure freezes usage before unconditional frame cleanup.
        // Preserve that qualified snapshot; live adapter custody is zero below.
        assert_eq!(error.usage().scratch_records(), 1);
        assert_eq!(error.usage().peak_scratch_records(), 1);
        assert!(error.usage().work() > 1);
        assert!(error.usage().allocated_bytes() > size_of::<HtmlSemanticResult>());
        assert_eq!(draft.select_semantic_work.get(), error.usage().work());
        assert_eq!(
            draft.select_semantic_bytes.get(),
            error.usage().allocated_bytes()
        );
        assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn source_option_value_real_style_park_preserves_work_allocations_and_cancellation() {
        let (mut dom, select, option, _) = fixture();
        let cancel = AtomicBool::new(false);
        let document = dom
            .snapshot()
            .node(option)
            .unwrap()
            .owner_document()
            .unwrap();
        let record = size_of::<HtmlSemanticResult>();
        dom.set_limits(query_policy(&dom, 1, 2 * record, 128))
            .unwrap();
        let mut setup = DomBatch::new();
        let style = setup.create_element_ns(document, Namespace::html(), "style");
        let commit = dom.apply(setup, &cancel).unwrap();
        let style = commit.created(&style).unwrap().unwrap();
        let mut commands = DomBatch::new();
        commands.insert_before(style, document.node(), None);
        commands.set_select_value(select, "chosen");
        let mut lease = dom
            .begin_source_commands(commands, limits(16_000_000), &cancel)
            .unwrap();
        let operation = lease.operation();
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual inline policy")
        };
        lease
            .accept_inline_csp(
                &policy,
                &crate::DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual sheet creation")
        };
        let root = crate::store::source::operation::accounting_tests::commit_actual_empty_sheet(
            &mut lease, &create, &cancel,
        );
        let cssom_revision = root.snapshot().unwrap().revision().clone();
        let DomError::SelectSemantic(first) = lease.advance(&cancel).unwrap_err() else {
            panic!("actual original child work stop after publication")
        };
        assert_eq!(
            first.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: 1,
                required: 2
            }
        );
        assert_eq!(first.usage().work(), 1);
        assert_eq!(first.usage().allocated_bytes(), record);
        assert!(first.qualification().is_some());
        let first_status = lease.status();
        assert!(first_status.has_published());
        let parked = lease.snapshot().stamp();
        drop(lease);
        assert_eq!(
            dom.pending_source_edit().unwrap().spent_work(),
            first_status.spent_work()
        );
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            dom.resume_source_edit(operation, limits(16_000_000), &cancel)
                .err(),
            Some(DomError::Cancelled)
        );
        assert_eq!(
            dom.pending_source_edit().unwrap().spent_work(),
            first_status.spent_work()
        );
        cancel.store(false, Ordering::Relaxed);
        let mut lease = dom
            .resume_source_edit(operation, limits(16_000_000), &cancel)
            .unwrap();
        let DomError::SelectSemantic(second) = lease.advance(&cancel).unwrap_err() else {
            panic!("spent work must remain spent")
        };
        assert_eq!(
            second.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: 0,
                required: 1
            }
        );
        assert_eq!(second.usage().work(), 0);
        assert_eq!(second.usage().allocated_bytes(), 0);
        assert!(
            lease.status().spent_work() > first_status.spent_work(),
            "rebuilding the actual failed capture spends outer work again"
        );
        let second_work = lease.status().spent_work();
        assert_eq!(lease.snapshot().stamp(), parked);
        drop(lease);
        // Keep exactly the already-spent result-bank allocation allowance, while
        // raising work. The next original bank precheck sees zero remaining.
        dom.set_limits(query_policy(&dom, 1_000_000, record, 128))
            .unwrap();
        let mut lease = dom
            .resume_source_edit(operation, limits(16_000_000), &cancel)
            .unwrap();
        let DomError::SelectSemantic(third) = lease.advance(&cancel).unwrap_err() else {
            panic!("spent allocation must remain spent")
        };
        assert_eq!(
            third.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::AllocatedBytes,
                limit: 0,
                required: record
            }
        );
        assert_eq!(third.usage().work(), 0);
        assert_eq!(third.usage().allocated_bytes(), 0);
        assert!(lease.status().spent_work() > second_work);
        assert_eq!(lease.snapshot().stamp(), parked);
        drop(lease);
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        dom.set_limits(query_policy(&dom, 1_000_000, 2 * record, 128))
            .unwrap();
        let mut lease = dom
            .resume_source_edit(operation, limits(16_000_000), &cancel)
            .unwrap();
        // Current-command headroom runs this same original query once. Its
        // successful scan spends record2; actual execution sees no remaining
        // allocation. Private tree restoration does not refund child usage.
        let DomError::SelectSemantic(fourth) = lease.advance(&cancel).unwrap_err() else {
            panic!("successful scan must preserve its spent allocation")
        };
        assert_eq!(
            fourth.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::AllocatedBytes,
                limit: 0,
                required: record
            }
        );
        assert_eq!(fourth.usage().work(), 0);
        assert_eq!(fourth.usage().allocated_bytes(), 0);
        assert_eq!(
            fourth.qualification().unwrap().limits().allocated_bytes(),
            0
        );
        assert!(lease.status().spent_work() > second_work);
        assert_eq!(lease.snapshot().stamp(), parked);
        drop(lease);
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        // The admitted command does not repeat its scan on recovery. Record3
        // permits its actual invocation while records1/2 remain spent.
        dom.set_limits(query_policy(&dom, 1_000_000, 3 * record, 128))
            .unwrap();
        let mut lease = dom
            .resume_source_edit(operation, limits(16_000_000), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            crate::DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        assert!(
            dom.snapshot()
                .node(option)
                .unwrap()
                .as_element()
                .unwrap()
                .option_state()
                .unwrap()
                .flags()
                .selectedness()
        );
    }
    fn published_semantic_lowering(work: bool) {
        let (mut dom, select, option, _) = fixture();
        let cancel = AtomicBool::new(false);
        let document = dom
            .snapshot()
            .node(option)
            .unwrap()
            .owner_document()
            .unwrap();
        let record = size_of::<HtmlSemanticResult>();
        dom.set_limits(query_policy(&dom, 1, 2 * record, 128))
            .unwrap();
        let mut setup = DomBatch::new();
        let style = setup.create_element_ns(document, Namespace::html(), "style");
        let commit = dom.apply(setup, &cancel).unwrap();
        let style = commit.created(&style).unwrap().unwrap();
        let mut commands = DomBatch::new();
        commands.insert_before(style, document.node(), None);
        commands.set_select_value(select, "chosen");
        let mut lease = dom
            .begin_source_commands(commands, limits(16_000_000), &cancel)
            .unwrap();
        let operation = lease.operation();
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual inline policy")
        };
        lease
            .accept_inline_csp(
                &policy,
                &crate::DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual sheet creation")
        };
        let root = crate::store::source::operation::accounting_tests::commit_actual_empty_sheet(
            &mut lease, &create, &cancel,
        );
        let cssom_revision = root.snapshot().unwrap().revision().clone();
        let DomError::SelectSemantic(first) = lease.advance(&cancel).unwrap_err() else {
            panic!("actual original child work stop after publication")
        };
        assert_eq!(
            first.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: 1,
                required: 2
            }
        );
        assert_eq!(first.usage().work(), 1);
        assert_eq!(first.usage().allocated_bytes(), record);
        assert!(first.qualification().is_some());
        let first_status = lease.status();
        assert!(first_status.has_published());
        let parked = lease.snapshot().stamp();
        drop(lease);
        let usage = |dom: &Dom| {
            let (work, bytes) = dom
                .source_operation
                .as_ref()
                .unwrap()
                .semantic_usage_for_test();
            (work, bytes, dom.pending_source_edit().unwrap().spent_work())
        };
        let custody = |dom: &Dom| dom.pending_source_edit().unwrap();
        let original_usage = usage(&dom);
        let original_custody = custody(&dom);
        assert_eq!(original_usage.0, 1);
        assert_eq!(original_usage.1, record);
        assert!(dom.pending_source_edit().unwrap().has_published());
        let previous = dom.limits();
        let lowered = if work {
            query_policy(&dom, 0, 2 * record, 128)
        } else {
            query_policy(&dom, 1, record - 1, 128)
        };
        assert!(
            dom.set_limits(lowered).is_err(),
            "a published operation's spent semantic budget cannot be lowered"
        );
        let (resource, limit, used) = if work {
            (HtmlSemanticResource::Work, 0, original_usage.0)
        } else {
            (
                HtmlSemanticResource::AllocatedBytes,
                record - 1,
                original_usage.1,
            )
        };
        assert_eq!(
            dom.set_limits(lowered),
            Err(DomError::InvalidSelectSemanticLimit {
                resource,
                limit,
                used
            })
        );
        assert_eq!(dom.limits(), previous);
        assert_eq!(usage(&dom), original_usage);
        assert_eq!(custody(&dom), original_custody);
        assert_eq!(dom.snapshot().stamp(), parked);
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        // Equality is a valid absolute cap even though another query cannot run.
        let equal = query_policy(&dom, original_usage.0, original_usage.1, 128);
        dom.set_limits(equal).unwrap();
        assert_eq!(dom.limits(), equal);
        assert_eq!(usage(&dom), original_usage);
        assert_eq!(custody(&dom), original_custody);
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            dom.resume_source_edit(operation, limits(16_000_000), &cancel),
            Err(DomError::Cancelled)
        ));
        assert_eq!(usage(&dom), original_usage);
        assert_eq!(custody(&dom), original_custody);
        cancel.store(false, Ordering::Relaxed);
        dom.set_limits(query_policy(&dom, 1_000_000, 3 * record, 128))
            .unwrap();
        let mut lease = dom
            .resume_source_edit(operation, limits(16_000_000), &cancel)
            .unwrap();
        assert_eq!(lease.operation(), operation);
        assert!(lease.status().spent_work() >= first_status.spent_work());
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            crate::DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(root.snapshot().unwrap().revision(), &cssom_revision);
        assert_eq!(root.snapshot().unwrap().sheets().len(), 1);
        assert!(
            dom.snapshot()
                .node(option)
                .unwrap()
                .as_element()
                .unwrap()
                .option_state()
                .unwrap()
                .flags()
                .selectedness()
        );
    }
    #[test]
    fn source_option_value_published_work_limit_cannot_drop_below_actual_spent_usage() {
        published_semantic_lowering(true);
    }
    #[test]
    fn source_option_value_published_allocation_limit_cannot_drop_below_actual_spent_usage() {
        published_semantic_lowering(false);
    }
}
