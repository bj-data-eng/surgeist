//! Reached A/Area URL and wholly inactive Img updates in the original owner.
use super::*;
use surgeist_html::{
    HtmlHyperlinkSetUrlAction as Action, HtmlHyperlinkSetUrlCursor,
    HtmlHyperlinkSetUrlReply as Reply, HtmlImageAttributeChange, HtmlImageAttributeEffect,
    HtmlImageDocumentActivity, HtmlImageMutationTargets, HtmlImageTreeChange, HtmlImageUpdateFlags,
    HtmlImageUpdateStartDecision, HtmlImgFreshState, KnownHtmlAttribute, KnownHtmlElement,
};
use surgeist_html_semantics::{
    HtmlBaseMissing, HtmlDocumentFirstBase, HtmlEncodingUrlFactsError, HtmlEncodingUrlLookupError,
    HtmlEncodingUrlParseFacts, HtmlHyperlinkUrlSnapshot,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct AssociatedStorageUsage {
    bytes: u64,
    units: u64,
    entries: u64,
}
impl AssociatedStorageUsage {
    pub(super) fn checked_sum(
        &mut self,
        other: Self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.bytes = self.bytes.checked_add(other.bytes).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                .unwrap_err()
        })?;
        self.units = self.units.checked_add(other.units).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::ManagedUtf16Units)
                .unwrap_err()
        })?;
        self.entries = self.entries.checked_add(other.entries).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::AuxiliaryEntries)
                .unwrap_err()
        })?;
        Ok(())
    }
    pub(super) fn resources(self) -> [(HtmlParseResource, u64); 4] {
        [
            (HtmlParseResource::ManagedCapacityBytes, self.bytes),
            (HtmlParseResource::PeakTransientBytes, self.bytes),
            (HtmlParseResource::ManagedUtf16Units, self.units),
            (HtmlParseResource::AuxiliaryEntries, self.entries),
        ]
    }
    pub(super) fn add(
        &mut self,
        bytes: usize,
        units: usize,
        entries: usize,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.bytes = self.bytes.checked_add(bytes as u64).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                .unwrap_err()
        })?;
        self.units = self.units.checked_add(units as u64).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::ManagedUtf16Units)
                .unwrap_err()
        })?;
        self.entries = self.entries.checked_add(entries as u64).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::AuxiliaryEntries)
                .unwrap_err()
        })?;
        Ok(())
    }
    pub(super) fn acquire(self, c: &mut ParseControl) -> Result<(), SessionError> {
        c.charge(WorkCategory::ReserveByte, self.bytes)?;
        c.acquire_retained_provider(self.bytes, self.units, self.entries)?;
        Ok(())
    }
    pub(super) fn release(self, c: &mut ParseControl, owner: &ControlIdentity) {
        c.release_retained_provider(owner, self.bytes, self.units, self.entries);
    }
}

#[derive(Debug, Default)]
pub(crate) struct Inputs {
    pub(crate) template: Option<crate::HtmlParseBaseFacts>,
    pub(crate) parser_document: Option<crate::HtmlParseBaseFacts>,
    pub(crate) captured: Vec<surgeist_html_semantics::HtmlDocumentEncodingUrlContext>,
    pub(crate) encoding_urls: Vec<HtmlEncodingUrlParseFacts>,
}
impl Inputs {
    fn usage(&self, c: &mut ParseControl) -> Result<AssociatedStorageUsage, SessionError> {
        let mut usage = AssociatedStorageUsage::default();
        // Like the original controller's shared payload tickets, this counts
        // the actual payload and Vec capacity, not opaque std Arc headers.
        let header = self
            .encoding_urls
            .capacity()
            .checked_mul(mem::size_of::<HtmlEncodingUrlParseFacts>())
            .and_then(|v| {
                self.captured
                    .capacity()
                    .checked_mul(mem::size_of::<
                        surgeist_html_semantics::HtmlDocumentEncodingUrlContext,
                    >())
                    .and_then(|n| v.checked_add(n))
            })
            .and_then(|v| mem::size_of::<Self>().checked_add(v))
            .ok_or_else(|| {
                c.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                    .unwrap_err()
            })?;
        usage.add(
            header,
            0,
            self.encoding_urls
                .capacity()
                .checked_add(self.captured.capacity())
                .ok_or(SessionError::AssociatedFacts)?,
            c,
        )?;
        for facts in [&self.template, &self.parser_document]
            .into_iter()
            .flatten()
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let retained = facts.retained_storage_usage();
            usage.add(
                retained.retained_bytes(),
                retained.utf16_extent(),
                facts.formation_usage().record_capacity(),
                c,
            )?;
        }
        for context in &self.captured {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let retained = context.retained_storage_usage();
            let backing = retained
                .retained_bytes()
                .checked_sub(mem::size_of::<
                    surgeist_html_semantics::HtmlDocumentEncodingUrlContext,
                >())
                .ok_or(SessionError::AssociatedFacts)?;
            usage.add(backing, retained.utf16_extent(), 0, c)?;
        }
        for (i, facts) in self.encoding_urls.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            for previous in &self.encoding_urls[..i] {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if previous.base() == facts.base()
                    && previous.encoding() == facts.encoding()
                    && previous.revision() == facts.revision()
                {
                    return Err(c.invalid_associated_facts::<()>().unwrap_err().into());
                }
            }
            let retained = facts.retained_storage_usage();
            usage.add(
                retained.retained_bytes(),
                retained.utf16_extent(),
                facts.formation_usage().record_capacity(),
                c,
            )?;
        }
        Ok(usage)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HyperlinkUnavailable {
    Base(HtmlBaseMissing),
    Observation(HtmlEncodingUrlFactsError),
    DocumentEnvironment,
    /// A reached CSP operation left the current Document policy unqualified.
    PolicyHistory,
}
pub(super) enum HyperlinkValue {
    Absent,
    Observed(HtmlHyperlinkUrlSnapshot),
    Unavailable(HyperlinkUnavailable),
}
pub(super) struct HyperlinkMetadata {
    pub(super) document: DocumentRef,
    pub(super) raw: Option<usize>,
    pub(super) value: HyperlinkValue,
    pub(super) cause: Option<provenance::ConstructionCauseRef>,
    pub(super) usage: AssociatedStorageUsage,
}
impl HyperlinkMetadata {
    pub(super) fn diagnostic_usage(&self) -> AssociatedStorageUsage {
        let mut usage = self.usage;
        if matches!(self.value, HyperlinkValue::Observed(_)) {
            usage.bytes = usage
                .bytes
                .checked_add(mem::size_of::<HtmlHyperlinkUrlSnapshot>() as u64)
                .expect("owner-computed complete snapshot footprint");
        }
        usage
    }
}
pub(super) struct ImageMetadata {
    pub(super) fresh: HtmlImgFreshState,
    pub(super) dimension_source: usize,
    pub(super) latest: u64,
    pub(super) waiting: Option<HtmlImageUpdateFlags>,
    pub(super) cause: Option<provenance::ConstructionCauseRef>,
    pub(super) usage: AssociatedStorageUsage,
}

impl ParserArenaSession {
    pub(crate) fn install_associated_inputs(
        &mut self,
        c: &mut ParseControl,
        inputs: &Arc<Inputs>,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        for (index, context) in inputs.captured.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 3)?;
            let Some(fragment) = &storage.fragment else {
                return Err(c.invalid_associated_facts::<()>().unwrap_err().into());
            };
            let capture = fragment.context.capture();
            if context.host_revision() != capture.host_facts_revision() {
                return Err(c.invalid_associated_facts::<()>().unwrap_err().into());
            }
            c.charge(WorkCategory::ProviderCall, 1)?;
            if capture.document_id(context.document().subject()).is_err() {
                return Err(c.invalid_associated_facts::<()>().unwrap_err().into());
            }
            for previous in &inputs.captured[..index] {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if previous.document() == context.document() {
                    return Err(c.invalid_associated_facts::<()>().unwrap_err().into());
                }
            }
        }
        let usage = inputs.usage(c)?;
        storage.prepare_payload_slot_owned(c)?;
        payload_write::<Arc<Inputs>>(c, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        usage.acquire(c)?;
        storage
            .payloads
            .values
            .push(RetainedPayload::AssociatedInputs {
                inputs: inputs.clone(),
                usage,
            });
        Ok(())
    }
}
pub(super) struct EncodingUrlContext<'a> {
    pub(super) facts: &'a HtmlEncodingUrlParseFacts,
    pub(super) base: &'a crate::HtmlBaseUrlRecord,
    pub(super) encoding: &'a crate::HtmlBaseEncoding,
    pub(super) expected: &'a crate::HtmlBaseFactsRevision,
}
impl SessionStorage {
    fn associated_inputs(
        &self,
        c: &mut ParseControl,
    ) -> Result<Option<&Arc<Inputs>>, SessionError> {
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::AssociatedInputs { inputs, .. } = payload {
                return Ok(Some(inputs));
            }
        }
        Ok(None)
    }
    pub(super) fn bind_template_base(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        let facts = self.associated_inputs(c)?.and_then(|i| i.template.as_ref());
        if let Some(facts) = facts {
            c.charge(WorkCategory::ProviderCall, 1)?;
            payload_write::<crate::HtmlParseBaseFacts>(c, 1)?;
            let facts = facts.clone();
            self.prepare_payload_slot_owned(c)?;
            self.payloads
                .push_row(RetainedPayload::BaseDocument {
                    node,
                    facts,
                    first: None,
                })
                .map_err(|_| latch_registry_row(c))?;
        }
        Ok(())
    }
    pub(super) fn associated_cause(
        &self,
        c: &mut ParseControl,
    ) -> Result<Option<provenance::ConstructionCauseRef>, SessionError> {
        self.evidence
            .as_ref()
            .map(|e| e.current_cause(c))
            .transpose()
            .map(Option::flatten)
    }
    pub(super) fn associated_tag(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<KnownHtmlElement>, SessionError> {
        for (name, tag) in [
            ("a", KnownHtmlElement::A),
            ("area", KnownHtmlElement::Area),
            ("img", KnownHtmlElement::Img),
            ("source", KnownHtmlElement::Source),
            ("picture", KnownHtmlElement::Picture),
            ("audio", KnownHtmlElement::Audio),
            ("video", KnownHtmlElement::Video),
        ] {
            if self.html_local(c, node, name)? {
                return Ok(Some(tag));
            }
        }
        Ok(None)
    }
    pub(super) fn initial_associated_state(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        self.initialize_style(c, node)?;
        self.initialize_dialog(c, node)?;
        let tag = self.associated_tag(c, node)?;
        if matches!(tag, Some(KnownHtmlElement::A | KnownHtmlElement::Area)) {
            let document = self.node_document_ref(c, node)?;
            let cause = self.associated_cause(c)?;
            self.prepare_payload_slot_owned(c)?;
            self.payloads
                .push_row(RetainedPayload::Hyperlink {
                    node,
                    state: HyperlinkMetadata {
                        document,
                        raw: None,
                        value: HyperlinkValue::Absent,
                        cause,
                        usage: AssociatedStorageUsage::default(),
                    },
                })
                .map_err(|_| latch_registry_row(c))?;
            self.set_hyperlink(c, node, None)?;
        } else if tag == Some(KnownHtmlElement::Img) {
            self.prepare_payload_slot_owned(c)?;
            // The shared fresh tuple owns exactly one public Arc<Vec<u16>> URL.
            // Admit its known empty Vec payload before calling the constructor;
            // opaque allocator bookkeeping has the same limitation as strings.
            let mut usage = AssociatedStorageUsage::default();
            usage.add(mem::size_of::<Vec<u16>>(), 0, 0, c)?;
            payload_write::<ImageMetadata>(c, 1)?;
            usage.acquire(c)?;
            self.payloads
                .push_row(RetainedPayload::Image {
                    node,
                    state: ImageMetadata {
                        fresh: HtmlImgFreshState::fresh(),
                        dimension_source: node,
                        latest: 0,
                        waiting: None,
                        cause: None,
                        usage,
                    },
                })
                .map_err(|_| latch_registry_row(c))?;
            self.start_img_update(c, node, HtmlImageUpdateFlags::new(false, false))?;
        } else {
            return Ok(());
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let count = element.attributes().len();
        // The immutable backing may be fused: this closed prefix cannot observe
        // a later attribute. URL parses use only the reached unique href;
        // image starts stop at Document activity before reading image sources.
        for position in 0..count {
            self.initial_associated_attribute(c, node, position)?;
        }
        Ok(())
    }
    pub(super) fn initial_associated_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let attr = element
            .attributes()
            .as_slice()
            .get(position)
            .ok_or(SessionError::WrongToken)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if attr.name().namespace().uri().is_some() {
            return Ok(());
        }
        let mut name = None;
        for candidate in [
            KnownHtmlAttribute::Href,
            KnownHtmlAttribute::Rel,
            KnownHtmlAttribute::Referrerpolicy,
            KnownHtmlAttribute::Src,
            KnownHtmlAttribute::Srcset,
            KnownHtmlAttribute::Sizes,
            KnownHtmlAttribute::Crossorigin,
            KnownHtmlAttribute::Loading,
        ] {
            if exact_ascii(c, Some(attr.name().local_name()), candidate.local_name())? {
                name = Some(candidate);
                break;
            }
        }
        let Some(name) = name else {
            return Ok(());
        };
        let tag = self.associated_tag(c, node)?;
        if matches!(tag, Some(KnownHtmlElement::A | KnownHtmlElement::Area)) {
            if surgeist_html::hyperlink_attribute_effect(attr.name().namespace(), name, false)
                == surgeist_html::HtmlHyperlinkAttributeEffect::SetUrlThenConsiderSpeculativeLoads
            {
                let raw = self.attribute_value_slot(c, node, position)?;
                self.set_hyperlink(c, node, Some(raw))?;
            }
        } else if tag == Some(KnownHtmlElement::Img) {
            let effect = surgeist_html::image_attribute_effect(
                attr.name().namespace(),
                name,
                HtmlImageAttributeChange::Set { same_value: false },
            );
            let flags = match effect {
                HtmlImageAttributeEffect::Update(flags) => Some(flags),
                HtmlImageAttributeEffect::CompareCrossOrigin
                | HtmlImageAttributeEffect::CompareReferrerPolicy => {
                    let mut admit = |step| match step {
                        surgeist_html::bounded::HtmlAttributeKeywordRead::ValueHeader
                        | surgeist_html::bounded::HtmlAttributeKeywordRead::KeywordHeader => {
                            c.charge(WorkCategory::RecordVisit, 1)
                        }
                        surgeist_html::bounded::HtmlAttributeKeywordRead::KeywordPair => {
                            c.charge(WorkCategory::InspectByte, 3)
                        }
                    };
                    let changed = if effect == HtmlImageAttributeEffect::CompareCrossOrigin {
                        let old =
                            surgeist_html::bounded::crossorigin_processing_with(None, &mut admit)?;
                        let new = surgeist_html::bounded::crossorigin_processing_with(
                            Some(attr.value()),
                            &mut admit,
                        )?;
                        surgeist_html::image_crossorigin_changed(old, new)
                    } else {
                        let old = surgeist_html::bounded::referrerpolicy_processing_with(
                            None, &mut admit,
                        )?;
                        let new = surgeist_html::bounded::referrerpolicy_processing_with(
                            Some(attr.value()),
                            &mut admit,
                        )?;
                        surgeist_html::image_referrerpolicy_changed(old, new)
                    };
                    changed.then_some(HtmlImageUpdateFlags::new(false, false))
                }
                // Fresh resumption is actually null; eager cannot invoke one.
                HtmlImageAttributeEffect::CompareLoading | HtmlImageAttributeEffect::None => None,
            };
            if let Some(flags) = flags {
                self.start_img_update(c, node, flags)?;
            }
        }
        Ok(())
    }
    fn hyperlink_resolution(
        &self,
        c: &mut ParseControl,
        node: usize,
        raw: usize,
    ) -> Result<Result<HtmlHyperlinkUrlSnapshot, HyperlinkUnavailable>, SessionError> {
        let Some(RetainedPayload::String { value: raw, .. }) = self.payloads.values.get(raw) else {
            return Err(latch_registry_row(c));
        };
        let context = match self.encoding_url_context(c, node)? {
            Ok(value) => value,
            Err(missing) => return Ok(Err(missing)),
        };
        payload_write::<HtmlHyperlinkUrlSnapshot>(c, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        match context.facts.resolve_hyperlink_with(
            raw,
            context.base,
            context.encoding,
            context.expected,
            || encoding_url_step(c),
            |_| Ok(()),
        ) {
            Ok(value) => Ok(Ok(value)),
            Err(HtmlEncodingUrlLookupError::Facts(error)) => {
                Ok(Err(HyperlinkUnavailable::Observation(error)))
            }
            Err(HtmlEncodingUrlLookupError::Interrupted(stop)) => Err(stop.into()),
        }
    }
    pub(super) fn encoding_url_context<'a>(
        &'a self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Result<EncodingUrlContext<'a>, HyperlinkUnavailable>, SessionError> {
        let document = match self.node_document_ref(c, node)? {
            DocumentRef::Owned(d) => d,
            DocumentRef::Captured(document) => {
                return self.captured_encoding_url_context(c, &document);
            }
        };
        if !self.csp_base_current(c, document)? || self.meta_policy_missing(c, document)?.is_some()
        {
            return Ok(Err(HyperlinkUnavailable::PolicyHistory));
        }
        let mut environment = None;
        if let Some(slot) = self.payloads.joined(c, document, Join::BaseDocument)? {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::BaseDocument { node, facts, first }) =
                self.payloads.values.get(slot)
                && *node == document
            {
                environment = Some((facts, first));
            }
        }
        let Some((facts, first)) = environment else {
            return Ok(Err(HyperlinkUnavailable::Base(
                HtmlBaseMissing::DocumentFacts,
            )));
        };
        let Some(encoding) = facts.document().encoding() else {
            return Ok(Err(HyperlinkUnavailable::Base(
                HtmlBaseMissing::CharacterEncoding,
            )));
        };
        let mut frozen = None;
        if let Some(first) = first
            && let Some(slot) = self.payloads.joined(c, *first, Join::Base)?
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Base {
                node: Some(node),
                frozen: base::BaseValue::Frozen(value),
                ..
            }) = self.payloads.values.get(slot)
                && node == first
            {
                frozen = Some(&value.value);
            }
        }
        let selection = if first.is_some() {
            HtmlDocumentFirstBase::Present(frozen)
        } else {
            HtmlDocumentFirstBase::Absent
        };
        let base = match surgeist_html_semantics::select_document_base(
            Some(facts.document()),
            selection,
        ) {
            Ok(base) => base,
            Err(e) => return Ok(Err(HyperlinkUnavailable::Base(e))),
        };
        let expected = facts.document().revision();
        self.encoding_url_bank(c, base, encoding, expected)
    }
    fn captured_encoding_url_context<'a>(
        &'a self,
        c: &mut ParseControl,
        document: &crate::HtmlCaptureDocumentId,
    ) -> Result<Result<EncodingUrlContext<'a>, HyperlinkUnavailable>, SessionError> {
        // This source-created fragment has only these exact captured effects.
        // None changes captured Document policy or its ordinary Base tree. This
        // exhaustive inventory must be revised before admitting any such new
        // producer: it must retain the actual captured gap and refuse here.
        c.charge(WorkCategory::RecordVisit, 1)?;
        let fragment = self
            .fragment
            .as_ref()
            .ok_or(SessionError::ContextUnavailable)?;
        for effect in fragment.effects.as_slice() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            match effect {
                context::ContextEffect::TemplateCache { .. }
                | context::ContextEffect::AttachShadow { .. } => {}
            }
        }
        let Some(inputs) = self.associated_inputs(c)? else {
            return Ok(Err(HyperlinkUnavailable::DocumentEnvironment));
        };
        for value in &inputs.captured {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if value.document() == document {
                c.charge(WorkCategory::RecordVisit, 3)?;
                return self.encoding_url_bank(
                    c,
                    value.base_url().value(),
                    value.encoding(),
                    value.revision(),
                );
            }
        }
        Ok(Err(HyperlinkUnavailable::DocumentEnvironment))
    }
    fn encoding_url_bank<'a>(
        &'a self,
        c: &mut ParseControl,
        base: &'a crate::HtmlBaseUrlRecord,
        encoding: &'a crate::HtmlBaseEncoding,
        expected: &'a crate::HtmlBaseFactsRevision,
    ) -> Result<Result<EncodingUrlContext<'a>, HyperlinkUnavailable>, SessionError> {
        let Some(inputs) = self.associated_inputs(c)? else {
            return Ok(Err(HyperlinkUnavailable::Observation(
                HtmlEncodingUrlFactsError::MissingObservation,
            )));
        };
        let mut missing = HtmlEncodingUrlFactsError::MissingObservation;
        for candidate in &inputs.encoding_urls {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let Err(error) = candidate.validate_inputs(base, encoding, expected) {
                missing = error;
                continue;
            }
            return Ok(Ok(EncodingUrlContext {
                facts: candidate,
                base,
                encoding,
                expected,
            }));
        }
        Ok(Err(HyperlinkUnavailable::Observation(missing)))
    }
    fn set_hyperlink(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        raw: Option<usize>,
    ) -> Result<(), SessionError> {
        let slot = self
            .payloads
            .joined(c, node, Join::Hyperlink)?
            .ok_or(SessionError::WrongToken)?;
        let mut cursor = HtmlHyperlinkSetUrlCursor::new();
        let mut result = None;
        loop {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let reply = match cursor.action() {
                Action::ClearUrl => {
                    payload_write::<HyperlinkValue>(c, 1)?;
                    let Some(RetainedPayload::Hyperlink { state, .. }) =
                        self.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(c));
                    };
                    let old = mem::replace(&mut state.value, HyperlinkValue::Absent);
                    let old_usage = mem::take(&mut state.usage);
                    drop(old);
                    old_usage.release(c, &self.control);
                    Reply::Applied
                }
                Action::ReadHref => Reply::HrefPresent(raw.is_some()),
                Action::ParseHref => {
                    result = Some(self.hyperlink_resolution(
                        c,
                        node,
                        raw.ok_or(SessionError::WrongToken)?,
                    )?);
                    match &result {
                        Some(Ok(value)) => Reply::Parsed(value.url().is_some()),
                        Some(Err(_)) => break,
                        None => unreachable!(),
                    }
                }
                Action::StoreParsedUrl => Reply::Applied,
                Action::Complete => break,
            };
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        }
        let value = match result {
            None => HyperlinkValue::Absent,
            Some(Ok(value)) => HyperlinkValue::Observed(value),
            Some(Err(error)) => HyperlinkValue::Unavailable(error),
        };
        let mut usage = AssociatedStorageUsage::default();
        if let HyperlinkValue::Observed(value) = &value {
            let retained = value.retained_storage_usage();
            // The original payload Vec already owns this SAME inline snapshot.
            // The producer receipt expressly includes its public outer header;
            // admit only its reachable backing here, with no private layout copy.
            let backing = retained
                .retained_bytes()
                .checked_sub(mem::size_of::<HtmlHyperlinkUrlSnapshot>())
                .ok_or(SessionError::AssociatedFacts)?;
            usage.add(backing, retained.utf16_extent(), 0, c)?;
        }
        let document = self.node_document_ref(c, node)?;
        let cause = self.associated_cause(c)?;
        payload_write::<HyperlinkMetadata>(c, 1)?;
        usage.acquire(c)?;
        #[cfg(test)]
        if matches!(value, HyperlinkValue::Observed(_)) {
            CANCEL_AFTER_SELECTED.with(|hook| {
                if let Some(cancel) = hook.borrow_mut().take() {
                    cancel.cancel();
                }
            });
        }
        let Some(RetainedPayload::Hyperlink { state, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(c));
        };
        let old = mem::replace(
            state,
            HyperlinkMetadata {
                document,
                raw,
                value,
                cause,
                usage,
            },
        );
        let old_usage = old.usage;
        drop(old);
        old_usage.release(c, &self.control);
        Ok(())
    }
    fn image_document_activity(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<HtmlImageDocumentActivity, SessionError> {
        Ok(match self.document_fully_active(c, node)? {
            Some(true) => HtmlImageDocumentActivity::FullyActive,
            Some(false) => HtmlImageDocumentActivity::NotFullyActive,
            None => HtmlImageDocumentActivity::Unavailable,
        })
    }
    pub(super) fn start_img_update(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        flags: HtmlImageUpdateFlags,
    ) -> Result<(), SessionError> {
        let Some(slot) = self.payloads.joined(c, node, Join::Image)? else {
            return Ok(());
        };
        let activity = self.image_document_activity(c, node)?;
        let decision = surgeist_html::image_update_start(activity);
        let cause = self.associated_cause(c)?;
        let Some(RetainedPayload::Image { state, .. }) = self.payloads.values.get(slot) else {
            return Err(latch_registry_row(c));
        };
        let next = state.latest.checked_add(1).ok_or_else(|| {
            c.representation::<()>(HtmlParseResource::AuxiliaryEntries)
                .unwrap_err()
        })?;
        if state.waiting.is_some()
            && !surgeist_html::image_update_is_superseded(state.latest.cmp(&next))
                .map_err(|_| SessionError::AssociatedFacts)?
        {
            return Err(SessionError::AssociatedFacts);
        }
        payload_write::<(
            u64,
            Option<HtmlImageUpdateFlags>,
            Option<provenance::ConstructionCauseRef>,
        )>(c, 1)?;
        let Some(RetainedPayload::Image { state, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(c));
        };
        state.latest = next;
        state.waiting =
            (decision == HtmlImageUpdateStartDecision::WaitForFullyActive).then_some(flags);
        state.cause = cause;
        Ok(())
    }
    pub(super) fn image_tree_changed(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        change: HtmlImageTreeChange,
        root: usize,
        old_parent: Option<usize>,
    ) -> Result<(), SessionError> {
        let Some(tag @ (KnownHtmlElement::Img | KnownHtmlElement::Source)) =
            self.associated_tag(c, node)?
        else {
            return Ok(());
        };
        let parent = self.ordinary_parent(c, node, None)?;
        let parent_tag = parent
            .map(|p| self.associated_tag(c, p))
            .transpose()?
            .flatten();
        let old_tag = old_parent
            .map(|p| self.associated_tag(c, p))
            .transpose()?
            .flatten();
        let effect =
            surgeist_html::image_tree_effect(tag, change, parent_tag, old_tag, root == node)
                .map_err(|_| SessionError::WrongToken)?;
        // Media resource selection has no Img recipient and is outside this
        // retained standalone-Img prefix. No Audio/Video capability is issued.
        if effect.check_media_parent_selection() {
            return Ok(());
        }
        match effect.targets() {
            HtmlImageMutationTargets::None => {}
            HtmlImageMutationTargets::ThisImage => {
                self.start_img_update(c, node, HtmlImageUpdateFlags::new(false, false))?
            }
            HtmlImageMutationTargets::ParentPictureImages
            | HtmlImageMutationTargets::OldParentPictureImages => {
                let picture =
                    if effect.targets() == HtmlImageMutationTargets::ParentPictureImages {
                        parent
                    } else {
                        old_parent
                    }
                    .ok_or(SessionError::WrongToken)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                let count = self
                    .construction
                    .node_at(picture)
                    .ok_or(SessionError::ForeignNode)?
                    .child_count();
                for ordinal in 0..count {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let child = self
                        .construction
                        .node_at(picture)
                        .ok_or(SessionError::ForeignNode)?
                        .child_at(ordinal)
                        .ok_or(SessionError::WrongToken)?
                        .position();
                    if self.associated_tag(c, child)? == Some(KnownHtmlElement::Img) {
                        self.start_img_update(c, child, HtmlImageUpdateFlags::new(false, false))?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub(super) fn encoding_url_step(c: &mut ParseControl) -> Result<(), ParseStop> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    c.charge(WorkCategory::InspectByte, 4)
}

#[cfg(test)]
std::thread_local! {
    static CANCEL_AFTER_SELECTED: std::cell::RefCell<Option<crate::HtmlParseCancellation>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_after_selected_backing_admission_reclaims_original_owning_prefix() {
        use surgeist_html_semantics::*;
        let limits = crate::HtmlParseLimits::builder()
            .source_units(1000)
            .buffered_utf16_units(1000)
            .token_attributes(100)
            .auxiliary_entries(100_000)
            .managed_nodes(1000)
            .managed_edges(1000)
            .managed_attributes(1000)
            .managed_utf16_units(10000)
            .source_map_entries(10000)
            .source_map_payload_units(10000)
            .diagnostics(100)
            .diagnostic_payload_units(10000)
            .context_effects(100)
            .open_elements(100)
            .tree_depth(1000)
            .managed_capacity_bytes(10_000_000)
            .peak_transient_bytes(10_000_000)
            .atomic_layout_bytes(10_000_000)
            .work_credits(10_000_000)
            .finish()
            .unwrap();
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let base = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "https://example.test/".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap();
        let encoding = HtmlBaseEncoding::fresh();
        let environment = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(base.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding.clone()),
            )
            .unwrap(),
            vec![],
            HtmlBaseFactsLimits::new(10, 1000, 10000),
        )
        .unwrap();
        let fact = HtmlEncodingUrlParseFacts::new(
            base,
            encoding,
            revision,
            vec![HtmlEncodingUrlParseObservation::new(
                "x".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlBaseFactsLimits::new(10, 1000, 10000),
        )
        .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<!doctype html><a href=x>"),
            limits,
        )
        .with_base_facts(environment)
        .unwrap()
        .with_encoding_url_facts(vec![fact.clone()]);
        CANCEL_AFTER_SELECTED
            .with(|hook| *hook.borrow_mut() = Some(request.cancellation().clone()));
        let failure = request.parse().unwrap_err();
        assert_eq!(failure.first_stop(), Some(crate::HtmlParseStop::Cancelled));
        assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&fact));
        assert_eq!(failure.cleanup_error(), None);
        for resource in [
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::BufferedUtf16Units,
        ] {
            assert_eq!(
                failure
                    .after_cleanup()
                    .unwrap()
                    .resource(resource)
                    .current(),
                0,
                "{resource:?}"
            );
        }
        CANCEL_AFTER_SELECTED.with(|hook| assert!(hook.borrow().is_none()));
    }

    #[test]
    fn fragment_cancel_after_selected_admission_keeps_original_bank_and_reclaims_prefix() {
        use surgeist_html_semantics::*;
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(20, 100, 10, 1000, 10, 0, 0, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                surgeist_html::HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let div = builder
            .add_element(
                &document,
                surgeist_html::QualifiedName::new(surgeist_html::Namespace::html(), None, "div"),
                surgeist_html::OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_no_shadow_root(&div).unwrap();
        builder
            .establish_children(&document, std::slice::from_ref(&div))
            .unwrap();
        builder.establish_children(&div, &[]).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
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
        let environment = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(base.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding.clone()),
            )
            .unwrap(),
            vec![],
            HtmlBaseFactsLimits::new(10, 1000, 10000),
        )
        .unwrap();
        let fact = HtmlEncodingUrlParseFacts::new(
            base,
            encoding,
            revision,
            vec![HtmlEncodingUrlParseObservation::new(
                "x".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlBaseFactsLimits::new(10, 1000, 10000),
        )
        .unwrap();
        let mut facts = Vec::with_capacity(7);
        facts.push(fact.clone());
        let request = crate::HtmlFragmentParseRequest::new(
            crate::HtmlSource::utf8("<a href=x>"),
            context,
            crate::construction::tests::limits(),
        )
        .unwrap()
        .with_parser_document_base_facts(environment)
        .unwrap()
        .with_encoding_url_facts(facts);
        CANCEL_AFTER_SELECTED
            .with(|hook| *hook.borrow_mut() = Some(request.cancellation().clone()));
        let failure = request.parse().unwrap_err();
        assert_eq!(failure.first_stop(), Some(crate::HtmlParseStop::Cancelled));
        assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&fact));
        assert_eq!(failure.encoding_url_fact_capacity(), Some(7));
        assert_eq!(failure.cleanup_error(), None);
        for resource in [
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::BufferedUtf16Units,
        ] {
            assert_eq!(
                failure
                    .after_cleanup()
                    .unwrap()
                    .resource(resource)
                    .current(),
                0,
                "{resource:?}"
            );
        }
        CANCEL_AFTER_SELECTED.with(|hook| assert!(hook.borrow().is_none()));
    }
}
