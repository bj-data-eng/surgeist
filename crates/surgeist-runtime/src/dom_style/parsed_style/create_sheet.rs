//! One actual original CreateSheet through the sole association/preference owners.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use std::rc::Rc;
use surgeist_dom::{
    DomParsedHtmlStyleRuntimeHold, DomParsedHtmlStyleSheetCompletion, DomParsedHtmlStyleSheetError,
    DomParsedHtmlStyleSheetInput, DomParsedHtmlStyleSheetPreparation,
};

/// Original sheet processing refused before the joined publication fence.
#[derive(Debug)]
pub enum DomStyleParsedSheetError {
    /// Session ownership, idle custody or finite Runtime admission refused.
    Runtime(DomStyleRuntimeError),
    /// Original phase/mapping or the actual qualified producer refused.
    Sheet(DomParsedHtmlStyleSheetError),
    /// Real CSS input refusal, preserving its actual original/Runtime poll cause.
    Css(surgeist_css::CssUtf16InputError<DomStyleRuntimeError>),
}
impl From<DomStyleRuntimeError> for DomStyleParsedSheetError {
    fn from(value: DomStyleRuntimeError) -> Self {
        Self::Runtime(value)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedSheetError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Sheet(value.into())
    }
}
impl From<DomParsedHtmlStyleSheetError> for DomStyleParsedSheetError {
    fn from(value: DomParsedHtmlStyleSheetError) -> Self {
        Self::Sheet(value)
    }
}
impl std::fmt::Display for DomStyleParsedSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(e) => e.fmt(f),
            Self::Sheet(e) => e.fmt(f),
            Self::Css(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedSheetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(e) => Some(e),
            Self::Sheet(e) => Some(e),
            Self::Css(e) => Some(e),
        }
    }
}
/// One genuine original sheet association and its sole add-sheet preference result.
/// This stops at ReadScriptBlocking; the whole parsed history stays Pending.
#[must_use]
pub struct DomStyleParsedSheetOutcome {
    completion: DomParsedHtmlStyleSheetCompletion,
    observation: DomStyleSheetSetObservation,
}
impl std::fmt::Debug for DomStyleParsedSheetOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleParsedSheetOutcome")
            .field("contribution", &self.completion.contribution())
            .field("sheet", self.completion.sheet().id())
            .field("observation", &self.observation)
            .finish()
    }
}
impl DomStyleParsedSheetOutcome {
    /// Genuine joint DOM/CSSOM association, still short of whole readiness.
    pub fn completion(&self) -> &DomParsedHtmlStyleSheetCompletion {
        &self.completion
    }
    /// The existing preference owner joined to the actual published revision.
    pub fn observation(&self) -> &DomStyleSheetSetObservation {
        &self.observation
    }
    /// Move both authentic outputs without another publication or lookup.
    pub fn into_parts(
        self,
    ) -> (
        DomParsedHtmlStyleSheetCompletion,
        DomStyleSheetSetObservation,
    ) {
        (self.completion, self.observation)
    }
}

fn reserve_processing(
    state: &mut StyleState,
    hold: &mut DomParsedHtmlStyleRuntimeHold<'_>,
    custody: &mut Custody,
    cancel: &AtomicBool,
) -> Result<(), DomStyleRuntimeError> {
    let steps = Steps {
        work: &state.work,
        maximum: state.limits.work,
        cancel,
    };
    steps.prefix(hold)?;
    if state.completions.len() >= state.limits.completions {
        return Err(DomStyleRuntimeError::Admission);
    }
    let (old, requested) = persistent_bank_growth(
        state.completions.len(),
        state.completions.capacity(),
        size_of::<Completion>(),
    )?;
    if requested == 0 {
        return Ok(());
    }
    // current already contains the old bank. Admit its full requested
    // replacement before allocation, then the actual old/new capacity union.
    custody.add(requested)?;
    hold.admit_runtime(custody.usage()?, cancel)?;
    state
        .completions
        .try_reserve_exact(1)
        .map_err(|_| DomStyleRuntimeError::Admission)?;
    steps.prefix(hold)?;
    let actual = state
        .completions
        .capacity()
        .checked_mul(size_of::<Completion>())
        .ok_or(DomStyleRuntimeError::Admission)?;
    custody.bytes = custody
        .bytes
        .checked_sub(requested)
        .and_then(|bytes| bytes.checked_add(actual))
        .ok_or(DomStyleRuntimeError::Admission)?;
    custody.check()?;
    hold.admit_runtime(custody.usage()?, cancel)?;
    // The reserve has disposed the old bank. Future observations price the
    // actual new bank, while the original meter keeps the admitted high-water.
    custody.current = custody
        .current
        .checked_sub(old)
        .and_then(|bytes| bytes.checked_add(actual))
        .ok_or(DomStyleRuntimeError::Admission)?;
    custody.bytes = custody
        .bytes
        .checked_sub(actual)
        .ok_or(DomStyleRuntimeError::Admission)?;
    Ok(())
}
#[expect(
    clippy::result_large_err,
    reason = "retain the actual root and original producer error without refusal allocation"
)]
fn parser_context(
    snapshot: &CssomSnapshot,
    mode: HtmlDocumentMode,
    label: &str,
    serial: u64,
    prep: &mut DomParsedHtmlStyleSheetPreparation<'_, '_, '_>,
    custody: &mut Custody,
    steps: &Steps<'_>,
) -> Result<CssParserContext, DomStyleParsedSheetError> {
    let mut existing = None;
    for (index, input) in snapshot.context().inputs.iter().enumerate() {
        steps.joined(prep)?;
        if let CssomInputData::ParserMode(context) = input.data {
            existing = Some((index, context));
            break;
        }
    }
    let current = existing.map_or_else(CssParserContext::default, |(_, context)| context);
    let selected = current.with_mode(match mode {
        HtmlDocumentMode::Quirks => CssParserMode::Quirks,
        HtmlDocumentMode::LimitedQuirks | HtmlDocumentMode::NoQuirks => CssParserMode::Standards,
    });
    if selected == current {
        return Ok(current);
    }
    visit_context_clone(snapshot.context(), || steps.joined(prep))?;
    let mut context = snapshot.context().clone();
    let mut identity = custody.string(
        label
            .len()
            .checked_add(22)
            .ok_or(DomStyleRuntimeError::Admission)?,
        prep,
        steps,
    )?;
    for ch in label.chars() {
        steps.joined(prep)?;
        identity.push(ch);
    }
    identity.push_str(":original-mode");
    let input = CssomInput {
        version: CssomInputVersion {
            role: CssomInputRole::ParserMode,
            identity,
            revision: serial,
        },
        data: CssomInputData::ParserMode(selected),
    };
    if let Some((index, _)) = existing {
        context.inputs[index] = input;
    } else {
        steps.joined(prep)?;
        context
            .inputs
            .try_reserve_exact(1)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        context.inputs.push(input);
    }
    prep.update_context(context, steps.cancel)?;
    Ok(selected)
}
impl DomStyleRuntime {
    /// Execute the reached original CreateSheet action with actual CSSOM/DOM
    /// publication and the existing add-sheet preference owner. CSS's private
    /// grammar work/transient allocation and fine-grained interruption are not
    /// bounded by this consumer: original work/cancellation is checked at every
    /// public CSS preparation/pre/post grammar poll, before publication. Failed
    /// attempts retain their spent work; retry performs and charges parsing again.
    #[expect(
        clippy::result_large_err,
        reason = "retain the actual root and original producer error without refusal allocation"
    )]
    pub fn create_parsed_style_sheet(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleParsedSheetOutcome, DomStyleParsedSheetError> {
        let owner = self.original()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        let view = state.require_sheet_set_idle(dom)?;
        let stamp = view.stamp();
        drop(view);
        let mut hold = dom
            .raw_mut()
            .begin_parsed_html_style(contribution, limits, cancel)?;
        let steps = Steps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        let mut custody = Custody {
            bytes: size_of::<DomStyleParsedSheetOutcome>()
                + size_of::<ParsedProcessing>()
                + size_of::<Rc<DomStyleCssomRoot>>()
                + size_of::<sheet_sets::State>() * 2
                + size_of::<CssomSnapshot>()
                + size_of::<Option<DomParsedHtmlStyleSheetInput>>()
                + size_of::<String>() * 2,
            shared: 0,
            units: 0,
            current: 0,
            maximum: state.limits.storage_bytes,
            outputs: 2, // Returned creation output and durable processing receipt.
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        custody.current = current_storage(&state, &mut hold, &custody, &steps)?;
        custody.check()?;
        reserve_processing(&mut state, &mut hold, &mut custody, cancel)?;
        let steps = Steps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        steps.prefix(&hold)?;
        let document = hold.create_sheet_document(cancel)?;
        let mut index = None;
        for (i, root) in state.roots.iter().enumerate() {
            steps.prefix(&hold)?;
            if root.node == document.node() {
                index = Some(i);
                break;
            }
        }
        let index = index.ok_or(DomStyleRuntimeError::SheetSetRootUnavailable {
            root: document.node(),
        })?;
        steps.prefix(&hold)?;
        let cap = state.roots[index].store.clone();
        let serial = state.roots[index]
            .serial
            .checked_add(1)
            .ok_or(DomStyleRuntimeError::Admission)?;
        let mut prep = hold.prepare_create_sheet(cap.as_ref(), cancel)?;
        steps.joined(&prep)?;
        let snapshot = prep.root_snapshot().clone();
        let mut label = custody.string(
            state.roots[index]
                .label
                .len()
                .checked_add(21)
                .ok_or(DomStyleRuntimeError::Admission)?,
            &mut prep,
            &steps,
        )?;
        for ch in state.roots[index].label.chars() {
            steps.joined(&prep)?;
            label.push(ch);
        }
        label.push(':');
        use std::fmt::Write;
        write!(&mut label, "{serial}").map_err(|_| DomStyleRuntimeError::Admission)?;
        let mut identity = custody.string(label.len(), &mut prep, &steps)?;
        for ch in label.chars() {
            steps.joined(&prep)?;
            identity.push(ch);
        }
        steps.joined(&prep)?;
        let mode = prep.request().document_mode();
        let context = parser_context(
            &snapshot,
            mode,
            &state.roots[index].label,
            serial,
            &mut prep,
            &mut custody,
            &steps,
        )?;
        steps.joined(&prep)?;
        let text = prep.request().text().shared_code_units().clone();
        let report = context
            .parse_sheet_utf16_with_limits(text, state.limits.css_input, || steps.joined(&prep))
            .map_err(DomStyleParsedSheetError::Css)?;
        steps.joined(&prep)?;
        let mut owner_label = custody.string(label.len(), &mut prep, &steps)?;
        for ch in label.chars() {
            steps.joined(&prep)?;
            owner_label.push(ch);
        }
        steps.joined(&prep)?;
        let media = prep
            .request()
            .media()
            .map(|s| s.shared_code_units().clone())
            .unwrap_or_default();
        let media = context
            .parse_media_query_list_utf16_with_limits(media, state.limits.css_input, || {
                steps.joined(&prep)
            })
            .map_err(DomStyleParsedSheetError::Css)?;
        steps.joined(&prep)?;
        let title = prep
            .request()
            .title()
            .map(|s| s.shared_code_units().clone());
        let current = state.roots[index].sheet_sets.state().ok_or(
            DomStyleRuntimeError::SheetSetDocumentRequired {
                root: document.node(),
            },
        )?;
        let mut disabled = false;
        let next = sheet_sets::prepare_added_sheet_sets_with(
            Some(current),
            &snapshot,
            sheet_sets::AddedSheetInputs {
                title: title.as_ref(),
                alternate: false,
                disabled: &mut disabled,
            },
            |operation| -> Result<(), DomStyleParsedSheetError> {
                match operation {
                    sheet_sets::AddedSheetOperation::Step => steps.joined(&prep)?,
                    sheet_sets::AddedSheetOperation::SetDisabled { sheet, disabled } => {
                        prep.set_disabled(sheet, disabled, cancel)?
                    }
                    sheet_sets::AddedSheetOperation::Candidate { next, current } => {
                        let bytes = next.additional_backing(Some(current))?;
                        // This candidate retains only the actual original title:
                        // Runtime's bank prices it, while DOM already owns the same
                        // backing through the original captured graph.
                        custody.add(bytes)?;
                        custody.shared = custody
                            .shared
                            .checked_add(bytes)
                            .ok_or(DomStyleRuntimeError::Admission)?;
                        custody.admit(&mut prep, &steps)?;
                    }
                }
                Ok(())
            },
        )?;
        custody.admit(&mut prep, &steps)?;
        let mut input = Some(DomParsedHtmlStyleSheetInput::new(
            report,
            media,
            CssomInputVersion {
                role: CssomInputRole::Owner,
                identity,
                revision: serial,
            },
            label,
            disabled,
        ));
        prep.stage_sheet(&mut input, cancel)?;
        let dependencies = match prep
            .resources()
            .expect("actual staged report classification")
        {
            surgeist_dom::DomParsedHtmlStyleResources::NoImports => ImportDependencies::NoImports,
            surgeist_dom::DomParsedHtmlStyleResources::ImportsUnresolved => {
                ImportDependencies::Imports
            }
        };
        let processing = ParsedProcessingCandidate {
            qualification: prep.request().qualification().clone(),
            owner: cap.clone(),
            // This closed original front uses the actual Session no-engine
            // profile; media, headlessness and CSS text do not establish it.
            profile: HtmlStyleExecutionProfile::NoExecutableJavaScript,
            dependencies,
            owner_label,
        };

        steps.joined(&prep)?;
        let mut observation = DomStyleSheetSetObservation::from_state(
            stamp,
            document.node(),
            next.as_ref().unwrap_or(current),
            snapshot.revision().clone(),
        );
        custody.admit(&mut prep, &steps)?;
        // Qualify actual destinations before the only publication call. The
        // capability is externally retained, not borrowed out of mutable Root.
        let StyleState {
            roots, completions, ..
        } = &mut *state;
        let root = &mut roots[index];
        let destination = root.sheet_sets.state_mut().expect("qualified Document");
        let published = prep.commit(cancel)?;
        // Infallible whole-fence suffix, retaining old names through lock release.
        let retired = next.map(|next| std::mem::replace(destination, next));
        root.serial = serial;
        let processing = processing.publish(published.completion().sheet().clone());
        completions.push(Completion::ParsedProcessing(processing));
        let (completion, retirement) = published.into_parts();
        let revision = match completion.cssom().publication() {
            surgeist_cssom::CssomPublication::Changed(summary) => summary.to(),
            surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
        };
        observation.finish_publication(completion.stamp(), revision.clone());
        let outcome = DomStyleParsedSheetOutcome {
            completion,
            observation,
        };
        drop(retired);
        drop(retirement);
        drop(snapshot);
        drop(cap);
        drop(hold);
        Ok(outcome)
    }
}
