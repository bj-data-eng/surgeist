//! One original CreateSheet action, privately assembled before either owner publishes.
use super::*;
use crate::parsed_style::DomParsedHtmlStyleSheetError as Error;
use crate::{DomCommit, DomStamp, DomStyleAssociationId, DomStyleCssomRoot, DomStyleSheet};
use surgeist_css::{CssRecoveryDiagnostic, CssSourceRetention, CssSourceRetentionAction};
use surgeist_cssom::{
    CssomBatch, CssomCommit, CssomCommitRetirement, CssomContext, CssomEditGuard, CssomInputRole,
    CssomSheetConstruction, CssomSheetId, CssomSheetInputs, CssomSnapshot, CssomStore,
};

// Ownership order is intentional on success AND every refusal: actual returned
// diagnostic/source graphs drop before their original controlling receipt.
struct Media {
    construction: CssomSheetConstruction,
    meter: Meter,
}
/// A completed creation cannot be confused with whole-contribution readiness.
pub struct DomParsedHtmlStyleSheetCompletion {
    contribution: DomParsedHtmlAssociationId,
    dom: DomCommit,
    sheet: DomStyleSheet,
    cssom: CssomCommit,
    media: Media,
    // Real output backing drops after every represented returned graph.
    _formation: Option<source::formation::Formation>,
}
impl DomParsedHtmlStyleSheetCompletion {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.contribution
    }
    pub fn stamp(&self) -> DomStamp {
        self.dom.changes().to()
    }
    pub fn dom(&self) -> &DomCommit {
        &self.dom
    }
    pub fn sheet(&self) -> &DomStyleSheet {
        &self.sheet
    }
    pub fn cssom(&self) -> &CssomCommit {
        &self.cssom
    }
    pub fn phase(&self) -> DomParsedHtmlStylePhase {
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    }
    pub fn media_diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        self.media.construction.media_diagnostics()
    }
}
/// Disposal only, after the complete CSSOM/DOM/Runtime name-installation fence.
/// Actual old State AND its frontier custody remain paired through that fence.
pub struct DomParsedHtmlStyleSheetRetirement {
    _state: Arc<State>,
    _state_custody: transaction::StateCustody,
    _binding: style_custody::Bank,
    _history: history::Retirement,
    _cssom: CssomCommitRetirement,
    _request: DomParsedHtmlStyleCreateSheetRequest,
    _snapshot: CssomSnapshot,
    _admission: control::Admission,
    _meter: Meter,
}
pub struct DomParsedHtmlStyleSheetPublished<'caps> {
    _guard: std::sync::MutexGuard<'caps, CssomStore>,
    completion: DomParsedHtmlStyleSheetCompletion,
    retirement: DomParsedHtmlStyleSheetRetirement,
}
impl DomParsedHtmlStyleSheetPublished<'_> {
    pub fn completion(&self) -> &DomParsedHtmlStyleSheetCompletion {
        &self.completion
    }
    pub fn into_parts(
        self,
    ) -> (
        DomParsedHtmlStyleSheetCompletion,
        DomParsedHtmlStyleSheetRetirement,
    ) {
        (self.completion, self.retirement)
    }
}
/// A closed editor for the authentic root. No raw staged state or Store loan
/// escapes. Drop publishes nothing; forgotten custody cannot reopen the cursor.
/// A prepared editor excludes overlapping use of its command hold:
/// ```compile_fail,E0502
/// use surgeist_dom::{DomParsedHtmlStyleRuntimeHold, DomStyleCssomRoot};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(hold: &mut DomParsedHtmlStyleRuntimeHold<'_>,
///            root: &DomStyleCssomRoot, cancel: &AtomicBool) {
///     // Caller supplies the reached original CreateSheet hold and actual root.
///     let prepared = hold.prepare_create_sheet(root, cancel).unwrap();
///     hold.step(cancel).unwrap();
///     let _ = prepared.request();
/// }
/// ```
/// End borrowed request inspection before consuming the prepared publication:
/// ```compile_fail,E0505
/// use surgeist_dom::{DomParsedHtmlStyleRuntimeHold, DomStyleCssomRoot};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(hold: &mut DomParsedHtmlStyleRuntimeHold<'_>,
///            root: &DomStyleCssomRoot, cancel: &AtomicBool) {
///     let prepared = hold.prepare_create_sheet(root, cancel).unwrap();
///     let request = prepared.request();
///     let _ = prepared.commit(cancel);
///     let _ = request.element();
/// }
/// ```
pub struct DomParsedHtmlStyleSheetPreparation<'hold, 'dom, 'caps> {
    graph: MutexGuard<'hold, Stored>,
    dom: &'hold mut &'dom mut Dom,
    runtime: &'hold Meter,
    cap: &'caps DomStyleCssomRoot,
    store: std::sync::MutexGuard<'caps, CssomStore>,
    snapshot: CssomSnapshot,
    batch: Option<CssomBatch>,
    media: Option<Media>,
    request: DomParsedHtmlStyleCreateSheetRequest,
    meter: Meter,
}
fn wrong_phase(graph: Option<&Graph>) -> DomParsedHtmlStyleError {
    DomParsedHtmlStyleError::WrongPhase {
        phase: graph.map_or(DomParsedHtmlStylePhase::Unstarted, Graph::phase),
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    pub fn create_sheet_document(
        &self,
        cancel: &AtomicBool,
    ) -> Result<DocumentId, DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        let Some(Graph::Inline(request)) = graph.as_ref().map(|h| h.value().as_ref()) else {
            return Err(wrong_phase(graph.as_ref().map(|h| h.value().as_ref())));
        };
        if request.action() != Action::CreateSheet {
            return Err(wrong_phase(graph.as_ref().map(|h| h.value().as_ref())));
        }
        Ok(request.qualification.document())
    }
}
impl<'dom> DomParsedHtmlStyleRuntimeHold<'dom> {
    #[expect(
        clippy::result_large_err,
        reason = "return the actual root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn prepare_create_sheet<'hold, 'caps>(
        &'hold mut self,
        cap: &'caps DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSheetPreparation<'hold, 'dom, 'caps>, Error> {
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        let completed = completed_lock(self.meter.entry())?;
        let value = graph.as_ref().map(|h| h.value().as_ref());
        let Some(Graph::Inline(original)) = value else {
            return Err(wrong_phase(value).into());
        };
        if original.action() != Action::CreateSheet {
            return Err(wrong_phase(value).into());
        }
        if let Some(previous) = completed.as_ref() {
            super::successor::qualify_creation(&self.meter, original, previous, cancel)?;
        }
        drop(completed);
        let document = original.qualification.document();
        self.dom.snapshot().document(document)?;
        if self.dom.pending_associations.document(document) != Some(self.meter.entry().id) {
            return Err(DomParsedHtmlStyleError::WrongRequest.into());
        }
        if cap.root() != document.node()
            || self
                .dom
                .style_cssom_roots
                .get(document.node())
                .is_some_and(|old| !old.same_owner(cap))
        {
            return Err(Error::MismatchedOwner {
                root: document.node(),
            });
        }
        let meter = self.meter.fork();
        meter.retain(
            size_of::<DomParsedHtmlStyleSheetPreparation<'_, '_, '_>>(),
            0,
            0,
            cancel,
        )?;
        let request = form_request(self.dom, &meter, original, cancel)?;
        meter.step(cancel)?;
        let store = cap.acquire().map_err(|e| match e {
            crate::style::CssomRootAccessError::Busy => Error::OwnerBusy {
                root: document.node(),
            },
            crate::style::CssomRootAccessError::Poisoned => Error::OwnerPoisoned {
                root: document.node(),
            },
        })?;
        let snapshot = store.snapshot();
        // This initial invocation or its authenticated same-Style successor has
        // no currently registered predecessor. No unexplained external
        // sheet may be discarded merely to force that original membership.
        if !snapshot.sheets().is_empty() {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput.into());
        }
        let batch = store
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .map_err(|error| Error::Cssom {
                root: document.node(),
                error,
            })?;
        Ok(DomParsedHtmlStyleSheetPreparation {
            graph,
            dom: &mut self.dom,
            runtime: &self.meter,
            cap,
            store,
            snapshot,
            batch: Some(batch),
            media: None,
            request,
            meter,
        })
    }
}
fn form_request(
    dom: &Dom,
    meter: &Meter,
    original: &DomParsedHtmlStyleInlinePolicyRequest,
    cancel: &AtomicBool,
) -> Result<DomParsedHtmlStyleCreateSheetRequest, DomParsedHtmlStyleError> {
    let entry = meter.entry();
    let mut element = None;
    for row in entry.bindings.iter() {
        meter.step(cancel)?;
        if row.source == *original.qualification.subject() {
            element = Some(row.node);
            break;
        }
    }
    let element = element.ok_or(DomParsedHtmlStyleError::WrongRequest)?;
    let snapshot = dom.snapshot();
    let node = snapshot.node(element)?;
    if node.node.document() != original.qualification.document() || node.as_style().is_none() {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    }
    let state = node.as_style().expect("qualified Style").state();
    if state.sheet.is_some()
        || state.availability != crate::DomStyleSheetAvailability::PendingParsedProcessing
    {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    }
    let report = &entry.report.value().value;
    let mode = match report
        .document()
        .ok_or(DomParsedHtmlStyleError::WrongRequest)?
        .root()
        .kind()
    {
        surgeist_html::HtmlNodeKind::Document(mode) => *mode,
        _ => return Err(DomParsedHtmlStyleError::WrongRequest),
    };
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Occurrence(row) if row.id() == *original.qualification.occurrence() => {
                let inputs = row.style().ok_or(DomParsedHtmlStyleError::WrongRequest)?;
                // The actual original list may name earlier inert Styles. Their
                // producer-proved no-effect prefix and actual completed predecessor
                // establish this supported membership; the Style occurs once.
                let mut selected = 0usize;
                for i in 0..inputs.sheet_order_len() {
                    meter.step(cancel)?;
                    let subject = inputs
                        .sheet_order(i)
                        .expect("qualified original order")
                        .id();
                    if subject == *original.qualification.subject() {
                        selected += 1;
                    }
                }
                if selected != 1 {
                    return Err(DomParsedHtmlStyleError::InvalidSheetInput);
                }
                let parser_document = match inputs.creating_parser_document() {
                    None => None,
                    Some(HtmlParseMetadataDocument::Owned(value)) => {
                        let source =
                            crate::DomParsedHtmlHistoricalDocument::Constructed(value.id());
                        let mut mapped = None;
                        for row in &entry.documents {
                            meter.step(cancel)?;
                            if row.source == source {
                                mapped = Some(row.document);
                                break;
                            }
                        }
                        Some(mapped.ok_or(DomParsedHtmlStyleError::WrongRequest)?)
                    }
                    Some(HtmlParseMetadataDocument::Captured(_)) => {
                        return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                            contribution: entry.id,
                        });
                    }
                };
                let attribute = |name| -> Result<Option<DocumentString>, DomParsedHtmlStyleError> {
                    let Some(group) = inputs.attribute(name) else {
                        return Ok(None);
                    };
                    let mut units = Vec::new();
                    append_group(report, group, &mut units, meter, cancel)?;
                    meter.retain(size_of::<Vec<u16>>(), 0, 0, cancel)?;
                    Ok(Some(DocumentString::from_utf16(units)))
                };
                let media = attribute(surgeist_html::KnownHtmlAttribute::Media)?;
                let title = attribute(surgeist_html::KnownHtmlAttribute::Title)?;
                return Ok(DomParsedHtmlStyleCreateSheetRequest {
                    qualification: original.qualification.clone(),
                    element,
                    text: original.text.clone(),
                    media,
                    title,
                    parser_document,
                    mode,
                });
            }
            Step::Complete => return Err(DomParsedHtmlStyleError::WrongRequest),
            _ => (),
        }
    }
}
impl DomParsedHtmlStyleSheetPreparation<'_, '_, '_> {
    pub fn request(&self) -> &DomParsedHtmlStyleCreateSheetRequest {
        &self.request
    }
    pub fn root_snapshot(&self) -> &CssomSnapshot {
        &self.snapshot
    }
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.meter.step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: crate::DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.runtime.runtime(usage, cancel)
    }
    #[expect(
        clippy::result_large_err,
        reason = "return the actual root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn update_context(
        &mut self,
        context: CssomContext,
        cancel: &AtomicBool,
    ) -> Result<(), Error> {
        self.step(cancel)?;
        self.batch
            .as_mut()
            .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?
            .update_context(context)
            .map_err(|error| Error::Cssom {
                root: self.request.document().node(),
                error,
            })?;
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "return the actual root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn set_disabled(
        &mut self,
        sheet: &CssomSheetId,
        disabled: bool,
        cancel: &AtomicBool,
    ) -> Result<(), Error> {
        let mut found = false;
        for id in self.snapshot.sheets() {
            self.step(cancel)?;
            if id == sheet {
                found = true;
                break;
            }
        }
        if !found {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput.into());
        }
        self.batch
            .as_mut()
            .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?
            .set_disabled(sheet, disabled)
            .map(|_| ())
            .map_err(|error| Error::Cssom {
                root: self.request.document().node(),
                error,
            })
    }
    #[expect(
        clippy::result_large_err,
        reason = "return the actual root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn stage_sheet(
        &mut self,
        input: &mut Option<DomParsedHtmlStyleSheetInput>,
        cancel: &AtomicBool,
    ) -> Result<(), Error> {
        self.step(cancel)?;
        if self.media.is_some() || self.batch.is_none() {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput.into());
        }
        let supplied = input
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?;
        if supplied.version.role != CssomInputRole::Owner || supplied.owner.is_empty() {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput.into());
        }
        validate_source(
            supplied.sheet.syntax().parse_source(),
            self.request.text.code_units(),
            self.request.mode,
            &self.meter,
            cancel,
        )?;
        validate_source(
            supplied.media.syntax().parse_source(),
            self.request
                .media
                .as_ref()
                .map_or(&[], DocumentString::code_units),
            self.request.mode,
            &self.meter,
            cancel,
        )?;
        self.step(cancel)?;
        let supplied = input.take().expect("validated owned input");
        let mut inputs = CssomSheetInputs::external(supplied.version, true);
        inputs.owner = Some(supplied.owner);
        inputs.disabled = supplied.disabled;
        if let Some(title) = &self.request.title {
            inputs.title = title.shared_code_units().clone();
        }
        let mut batch = self.batch.take().expect("one unattempted construction");
        let construction = batch
            .create_parsed_sheet_with_media_reports(supplied.sheet, supplied.media, inputs)
            .map_err(|error| Error::Cssom {
                root: self.request.document().node(),
                error,
            })?;
        let media = Media {
            construction,
            meter: self.meter.fork(),
        };
        admit_construction(&media.construction, &media.meter, cancel)?;
        self.batch = Some(batch);
        self.media = Some(media);
        Ok(())
    }
}
fn validate_source(
    source: Option<&surgeist_css::CssParseSource>,
    expected: &[u16],
    mode: surgeist_html::HtmlDocumentMode,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<(), DomParsedHtmlStyleError> {
    meter.step(cancel)?;
    let source = source.ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?;
    let selected = match mode {
        surgeist_html::HtmlDocumentMode::Quirks => surgeist_css::CssParserMode::Quirks,
        _ => surgeist_css::CssParserMode::Standards,
    };
    if source.context().mode() != selected {
        return Err(DomParsedHtmlStyleError::InvalidSheetInput);
    }
    let actual = source
        .source()
        .original_utf16()
        .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?;
    if actual.len() != expected.len() {
        return Err(DomParsedHtmlStyleError::InvalidSheetInput);
    }
    for (a, b) in actual.iter().zip(expected) {
        meter.step(cancel)?;
        if a != b {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput);
        }
    }
    Ok(())
}
fn admit_construction(
    value: &CssomSheetConstruction,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let bytes = value
        .retained_backing_bytes_with(|| meter.step(cancel))
        .map_err(|e| match e {
            surgeist_css::CssRetainedPayloadError::Admission(e) => e,
            surgeist_css::CssRetainedPayloadError::CapacityOverflow => {
                DomError::RepresentationExhausted
            }
        })?;
    meter.retain(bytes, 0, 0, cancel)?;
    let scratch = meter.fork();
    let mut admitted = 0usize;
    let mut refusal = None;
    let collected = {
        let mut admit = |action| {
            let result = match action {
                CssSourceRetentionAction::Step => scratch.step(cancel),
                CssSourceRetentionAction::Scratch { live_bytes } => {
                    let growth = live_bytes.saturating_sub(admitted);
                    let r = scratch.retain(growth, 0, 0, cancel);
                    if r.is_ok() {
                        admitted = admitted.max(live_bytes);
                    }
                    r
                }
            };
            match result {
                Ok(()) => true,
                Err(e) => {
                    refusal = Some(e);
                    false
                }
            }
        };
        match CssSourceRetention::with_admission(
            meter.entry().control.limits().retained_bytes(),
            usize::MAX,
            usize::MAX,
            &mut admit,
        ) {
            Err(error) => Err(error),
            Ok(mut sources) => (|| {
                for diagnostic in value.media_diagnostics() {
                    sources.charge_step()?;
                    if let Some(source) = diagnostic.source() {
                        sources.source(source)?;
                    }
                }
                sources
                    .complete_payload_bytes_admitted()
                    .map_err(|_| surgeist_css::CssSourceRetentionError::CallerAdmission)
            })(),
        }
    };
    let payload = match collected {
        Ok(bytes) => bytes,
        Err(_) => return Err(refusal.unwrap_or(DomError::RepresentationExhausted)),
    };
    // The producer's source collector owns byte union accounting. Count actual
    // retained original UTF-16 allocations once, without a parallel registry.
    let mut units = 0usize;
    for (index, diagnostic) in value.media_diagnostics().iter().enumerate() {
        meter.step(cancel)?;
        let Some(original) = diagnostic
            .source()
            .and_then(|source| source.original_utf16())
        else {
            continue;
        };
        let mut seen = false;
        for earlier in &value.media_diagnostics()[..index] {
            meter.step(cancel)?;
            if earlier
                .source()
                .and_then(|source| source.original_utf16())
                .is_some_and(|other| Arc::ptr_eq(other, original))
            {
                seen = true;
                break;
            }
        }
        if !seen {
            units = units
                .checked_add(original.capacity())
                .ok_or(DomError::RepresentationExhausted)?;
        }
    }
    meter.retain(payload, units, 0, cancel)
}

impl<'caps> DomParsedHtmlStyleSheetPreparation<'_, '_, 'caps> {
    /// Complete all fallible work before the first publication. The actual
    /// producer preparation remains local under its genuine exclusive guard;
    /// no owning guard/self-referential prepared-value pair is constructed.
    #[expect(
        clippy::result_large_err,
        reason = "return the actual root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn commit(
        mut self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSheetPublished<'caps>, Error> {
        let root = self.request.document().node();
        self.meter.step(cancel)?;
        let media = self
            .media
            .take()
            .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?;
        self.batch
            .as_mut()
            .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?
            .set_sheet_list_membership(
                self.snapshot.sheet_list(),
                &[surgeist_cssom::CssomSheetListTarget::Created(
                    media.construction.ticket(),
                )],
            )
            .map_err(|error| Error::Cssom { root, error })?;
        // All consumer-owned output/retirement headers exist before the native
        // registry baseline. Their represented graphs stay in these same owners.
        self.meter.retain(
            size_of::<DomParsedHtmlStyleSheetPublished<'_>>()
                + size_of::<DomParsedHtmlStyleSheetRetirement>(),
            0,
            2,
            cancel,
        )?;
        media
            .meter
            .retain(size_of::<DomParsedHtmlStyleSheetCompletion>(), 0, 0, cancel)?;
        self.dom.custody_registry.reconcile_releases();
        let admission = self
            .meter
            .admission(self.dom.custody_registry.total(), cancel)?;
        self.meter.step(cancel)?;
        let producer = self
            .store
            .prepare_commit(self.batch.take().expect("one unattempted producer batch"))
            .map_err(|error| Error::Cssom { root, error })?;
        #[cfg(test)]
        PREPARED_FRONTIER.with(|count| count.set(count.get() + 1));
        let metadata = producer
            .sheet(media.construction.ticket())
            .map_err(|error| Error::Cssom { root, error })?;
        let id = metadata.id().clone();
        let revision = metadata.revision().clone();
        let element = self.request.element;
        let mut draft = Draft::new_parsed(self.dom, Arc::new(BatchIdentity), &admission, cancel)?;
        let mut style = draft.style_state_mut(Key::Existing(element), cancel)?;
        if style.sheet.is_some()
            || style.availability != crate::DomStyleSheetAvailability::PendingParsedProcessing
        {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput.into());
        }
        let serial = style
            .next_association
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let sheet = DomStyleSheet {
            id,
            revision,
            association: DomStyleAssociationId { element, serial },
            root,
        };
        style.sheet = Some(sheet.clone());
        style.next_association = serial;
        style.commit(cancel)?;
        let prepared = draft.finish(self.dom, cancel)?;
        let mut publication = self.dom.prepare_publication(prepared, cancel)?;
        let bank = publication
            .style_custody
            .as_ref()
            .unwrap_or(&self.dom.style_cssom_roots);
        if let Some(current) = bank.get(root) {
            if !current.same_owner(self.cap) {
                return Err(Error::MismatchedOwner { root });
            }
        } else {
            publication.style_custody = Some(bank.prepare_insert(
                self.cap,
                self.request.document(),
                |total| admission.controlled(total, cancel),
                || admission.step(cancel),
            )?);
        }
        publication.close_ordinary_sharing(cancel)?;
        let history = self.dom.history.retain_evictions(
            &publication.history_plan,
            self.dom.custody_registry.ledger(),
            |total| admission.controlled(total, cancel),
            || admission.step(cancel),
        )?;
        // All discarded formation graphs are private until this point. Dispose
        // them before either owner publishes; the publication owns survivors.
        drop(draft);
        let mut successor = match self.graph.as_ref().map(|h| h.value().as_ref()) {
            Some(Graph::Inline(request)) => request.cursor,
            _ => unreachable!("borrowed original creation cursor"),
        };
        successor
            .reply(Reply::Applied)
            .map_err(DomParsedHtmlStyleError::from)?;
        if successor.action() != Action::ReadScriptBlocking {
            return Err(DomParsedHtmlStyleError::InvalidSheetInput.into());
        }
        let old_state = self.dom.state.clone();
        let old_state_custody = self.dom.state_custody.clone();
        let old_binding = self.dom.style_cssom_roots.clone();
        admission.step(cancel)?;
        let output_formation = publication._formation.take();
        // Sole composed publication fence: only prechecked moves and actual
        // infallible producer/DOM publication; dead backing is retained below.
        let (cssom, dead) = producer.publish();
        let dom = self.dom.publish_prepared(publication);
        let Graph::Inline(request) = self
            .graph
            .as_mut()
            .expect("original graph")
            .value_mut()
            .as_mut()
        else {
            unreachable!("qualified original creation")
        };
        request.cursor = successor;
        request.created_sheet = Some(sheet.clone());
        let contribution = self.meter.entry().id;
        let completion = DomParsedHtmlStyleSheetCompletion {
            contribution,
            dom,
            sheet,
            cssom,
            media,
            _formation: output_formation,
        };
        let retirement = DomParsedHtmlStyleSheetRetirement {
            _state: old_state,
            _state_custody: old_state_custody,
            _binding: old_binding,
            _history: history,
            _cssom: dead,
            _request: self.request,
            _snapshot: self.snapshot,
            _admission: admission,
            _meter: self.meter,
        };
        Ok(DomParsedHtmlStyleSheetPublished {
            _guard: self.store,
            completion,
            retirement,
        })
    }
}

#[cfg(test)]
thread_local! { pub(super) static PREPARED_FRONTIER: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

#[cfg(test)]
mod tests;
