//! One original Meta occurrence through the existing stylesheet-set owner.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use surgeist_dom::{
    DomParsedHtmlMetaOccurrenceCompletion, DomParsedHtmlMetaOccurrenceError,
    DomParsedHtmlMetaOccurrencePreparation,
};
use surgeist_html_parser::HtmlParseAssociationOccurrenceId;

/// Actual Runtime or original per-occurrence preparation refusal.
#[derive(Debug)]
pub enum DomStyleParsedMetaOccurrenceError {
    /// Actual Session, phase, or finite Runtime admission refused.
    Runtime(DomStyleRuntimeError),
    /// Exact original history or its actual CSSOM producer refused.
    Meta(DomParsedHtmlMetaOccurrenceError),
}
impl From<DomStyleRuntimeError> for DomStyleParsedMetaOccurrenceError {
    fn from(e: DomStyleRuntimeError) -> Self {
        Self::Runtime(e)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedMetaOccurrenceError {
    fn from(e: DomParsedHtmlStyleError) -> Self {
        Self::Meta(e.into())
    }
}
impl From<DomParsedHtmlMetaOccurrenceError> for DomStyleParsedMetaOccurrenceError {
    fn from(e: DomParsedHtmlMetaOccurrenceError) -> Self {
        Self::Meta(e)
    }
}
impl std::fmt::Display for DomStyleParsedMetaOccurrenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(e) => e.fmt(f),
            Self::Meta(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedMetaOccurrenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(e) => Some(e),
            Self::Meta(e) => Some(e),
        }
    }
}

/// Names and their immutable backing are disposed before the actual original
/// output receipt. The result cannot detach its receipt from its preference.
#[derive(Debug)]
pub struct DomStyleParsedMetaOccurrenceOutcome {
    preference: DomStyleSheetSetObservation,
    completion: DomParsedHtmlMetaOccurrenceCompletion,
}
impl DomStyleParsedMetaOccurrenceOutcome {
    /// Actual preferred-name history and final producer publication.
    pub fn preference(&self) -> &DomStyleSheetSetObservation {
        &self.preference
    }
    /// Original occurrence and its inseparable admitted output custody.
    pub fn completion(&self) -> &DomParsedHtmlMetaOccurrenceCompletion {
        &self.completion
    }
}
fn step(
    prep: &DomParsedHtmlMetaOccurrencePreparation<'_, '_, '_>,
    steps: &Steps<'_>,
) -> Result<(), DomStyleRuntimeError> {
    prep.step(steps.cancel)?;
    steps.runtime()
}
fn same(
    prep: &DomParsedHtmlMetaOccurrencePreparation<'_, '_, '_>,
    steps: &Steps<'_>,
    left: &[u16],
    right: &[u16],
) -> Result<bool, DomStyleRuntimeError> {
    step(prep, steps)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (a, b) in left.iter().zip(right) {
        step(prep, steps)?;
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}
impl DomStyleRuntime {
    /// Execute only the exact next original ChangePreferred after the supported
    /// completed Style prefix. Flags, preference and occurrence progress publish
    /// together; the contribution remains Pending until explicit end proof.
    pub fn consume_parsed_meta_occurrence(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        occurrence: &HtmlParseAssociationOccurrenceId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleParsedMetaOccurrenceOutcome, DomStyleParsedMetaOccurrenceError> {
        let owner = self.original()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        drop(state.require_sheet_set_idle(dom)?);
        let stamp = dom.raw().snapshot().stamp();
        let mut hold = dom
            .raw_mut()
            .begin_parsed_html_style(contribution, limits, cancel)?;
        let steps = Steps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        let mut custody = Custody {
            bytes: size_of::<DomStyleParsedMetaOccurrenceOutcome>()
                + 2 * size_of::<sheet_sets::State>()
                + size_of::<std::rc::Rc<DomStyleCssomRoot>>(),
            shared: 0,
            units: 0,
            current: 0,
            maximum: state.limits.storage_bytes,
            outputs: 1,
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        custody.current = current_storage(&state, &mut hold, &custody, &steps)?;
        custody.check()?;
        let document = hold.meta_occurrence_document(occurrence, cancel)?;
        let mut selected = None;
        for (index, root) in state.roots.iter().enumerate() {
            steps.prefix(&hold)?;
            if root.node == document.node() {
                selected = Some(index);
                break;
            }
        }
        let index = selected.ok_or(DomStyleRuntimeError::SheetSetRootUnavailable {
            root: document.node(),
        })?;
        steps.prefix(&hold)?;
        let cap = state.roots[index].store.clone();
        let mut prep = hold.prepare_meta_occurrence(occurrence, &cap, cancel)?;
        let StyleState {
            roots,
            work,
            limits: runtime_limits,
            ..
        } = &mut *state;
        let steps = Steps {
            work,
            maximum: runtime_limits.work,
            cancel,
        };
        step(&prep, &steps)?;
        let current = roots[index].sheet_sets.state().ok_or(
            DomStyleRuntimeError::SheetSetDocumentRequired {
                root: document.node(),
            },
        )?;
        let (next, enable) = current.transition(
            prep.occurrence().content(),
            sheet_sets::Action::Preferred,
            |a, b| same(&prep, &steps, a, b),
        )?;
        let mut surviving_bytes = size_of::<DomStyleSheetSetObservation>();
        let mut surviving_units = 0usize;
        for (name_index, name) in next.names().enumerate() {
            step(&prep, &steps)?;
            let allocation = name.shared_code_units();
            let mut duplicate = false;
            for previous in next.names().take(name_index) {
                step(&prep, &steps)?;
                duplicate |= Arc::ptr_eq(allocation, previous.shared_code_units());
            }
            if duplicate {
                continue;
            }
            let bytes = allocation
                .capacity()
                .checked_mul(size_of::<u16>())
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .ok_or(DomStyleRuntimeError::Admission)?;
            // Runtime's live old/new graph union is priced independently. The
            // lower occurrence owns its actual original decoded name backing.
            let mut existing = false;
            for root in roots.iter() {
                step(&prep, &steps)?;
                if let Some(old) = root.sheet_sets.state() {
                    for old in old.names() {
                        step(&prep, &steps)?;
                        existing |= Arc::ptr_eq(allocation, old.shared_code_units());
                    }
                }
            }
            if !existing {
                custody.add(bytes)?;
            }
            if Arc::ptr_eq(allocation, prep.occurrence().content().shared_code_units()) {
                if !existing {
                    custody.shared = custody
                        .shared
                        .checked_add(bytes)
                        .ok_or(DomStyleRuntimeError::Admission)?;
                }
            } else {
                surviving_bytes = surviving_bytes
                    .checked_add(bytes)
                    .ok_or(DomStyleRuntimeError::Admission)?;
                surviving_units = surviving_units
                    .checked_add(allocation.capacity())
                    .ok_or(DomStyleRuntimeError::Admission)?;
                // The output can outlive every Runtime row. Its independent
                // closing custody includes each actual other name allocation.
                custody.add(bytes)?;
                custody.units = custody
                    .units
                    .checked_add(allocation.capacity())
                    .ok_or(DomStyleRuntimeError::Admission)?;
            }
            prep.admit_runtime(custody.usage()?, cancel)
                .map_err(DomStyleRuntimeError::from)?;
        }
        if enable {
            for sheet_index in 0..prep.root_snapshot().sheets().len() {
                step(&prep, &steps)?;
                let sheet = &prep.root_snapshot().sheets()[sheet_index];
                let title = &prep
                    .root_snapshot()
                    .sheet(sheet)
                    .map_err(DomStyleRuntimeError::from)?
                    .inputs()
                    .title;
                let disabled = sheet_sets::named_sheet_disabled(
                    title,
                    prep.occurrence().content().code_units(),
                    |a, b| same(&prep, &steps, a, b),
                )?;
                if let Some(disabled) = disabled {
                    step(&prep, &steps)?;
                    let sheet = sheet.clone();
                    prep.set_disabled(&sheet, disabled, cancel)?;
                }
            }
        }
        step(&prep, &steps)?;
        let mut preference = DomStyleSheetSetObservation::from_state(
            stamp,
            document.node(),
            &next,
            prep.root_snapshot().revision().clone(),
        );
        prep.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        let destination = roots[index]
            .sheet_sets
            .state_mut()
            .expect("qualified actual Document preference owner");
        let surviving = surgeist_dom::DomParsedHtmlAssociationRuntimeUsage::new(
            surviving_bytes,
            surviving_units,
            1,
        )
        .map_err(DomStyleRuntimeError::from)?;
        step(&prep, &steps)?;
        let published = prep.commit(surviving, cancel)?;
        // No fallible work, allocation, lookup or large destruction follows the
        // first publication while the actual Store guard remains exclusive.
        let old = std::mem::replace(destination, next);
        let actual = published.completion();
        let revision = match actual.cssom().publication() {
            surgeist_cssom::CssomPublication::Changed(s) => s.to(),
            surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
        };
        preference.finish_publication(actual.stamp(), revision.clone());
        let (completion, retirement) = published.into_parts();
        drop(old);
        drop(cap);
        drop(retirement);
        drop(hold);
        Ok(DomStyleParsedMetaOccurrenceOutcome {
            preference,
            completion,
        })
    }
}
