//! Standard sheet and collection objects over the single live CSSOM owner.
//!
//! Captures have no mutation methods; live edits require an explicit owner:
//! ```compile_fail
//! use surgeist_cssom::{CssomCapture, CssStyleSheet};
//! fn invalid(sheet: &CssomCapture<CssStyleSheet>) { sheet.set_disabled(true); }
//! ```
//! Replacement work cannot be manufactured from a native token:
//! ```compile_fail
//! use surgeist_cssom::{CssomStylesheetReplacementWork, CssomReplaceToken};
//! fn invalid(token: CssomReplaceToken) { let _ = CssomStylesheetReplacementWork { token }; }
//! ```
use crate::object_operations::execute;
use crate::*;
use surgeist_css::{CssRecoveryDiagnostic, CssSpecifiedValueSerializationLimits};

/// Explicit constructor media input. Captured lists are copied through CSSOM text,
/// including across owners; the constructed sheet receives a fresh MediaList.
pub enum CssStyleSheetMedia<'a> {
    Text(&'a str),
    List(&'a CssomCapture<MediaList>),
}
/// Standard constructor defaults with explicitly supplied document facts.
pub struct CssStyleSheetInit<'a> {
    pub base_url: Option<String>,
    pub media: CssStyleSheetMedia<'a>,
    pub disabled: bool,
}
impl Default for CssStyleSheetInit<'_> {
    fn default() -> Self {
        Self {
            base_url: None,
            media: CssStyleSheetMedia::Text(""),
            disabled: false,
        }
    }
}

// Concrete inherited methods keep dispatch closed. Captured children share the
// same owning snapshot; live children carry identity without freezing membership.
macro_rules! sheet_reads {
    ($object:ident) => {
        impl $object {
            pub fn r#type(&self, store: &CssomStore) -> Result<&'static str, CssomError> {
                store.snapshot().sheet_type(self.id())
            }
            pub fn href<'a>(&self, store: &'a CssomStore) -> Result<Option<&'a str>, CssomError> {
                store.state.sheet_href(self.id())
            }
            pub fn owner_node<'a>(
                &self,
                store: &'a CssomStore,
            ) -> Result<Option<&'a str>, CssomError> {
                store.state.sheet_owner_node(self.id())
            }
            pub fn title<'a>(&self, store: &'a CssomStore) -> Result<Option<&'a str>, CssomError> {
                store.state.sheet_title(self.id())
            }
            pub fn parent_style_sheet(
                &self,
                store: &CssomStore,
            ) -> Result<Option<CssStyleSheet>, CssomError> {
                store
                    .snapshot()
                    .parent_sheet(self.id())?
                    .map(|id| CssStyleSheet::from_state(&store.state, id))
                    .transpose()
            }
            pub fn media(&self, store: &CssomStore) -> Result<MediaList, CssomError> {
                MediaList::from_state(&store.state, store.state.sheet(self.id())?.media())
            }
            pub fn disabled(&self, store: &CssomStore) -> Result<bool, CssomError> {
                store.snapshot().sheet_disabled(self.id())
            }
            pub fn set_disabled(
                &self,
                store: &mut CssomStore,
                disabled: bool,
            ) -> Result<CssomOperationOutcome<()>, CssomError> {
                execute(store, |batch| {
                    batch.set_disabled(self.id(), disabled)?;
                    Ok(CssomStagedOperation::new((), Vec::new()))
                })
            }
        }
        impl CssomCapture<$object> {
            pub fn r#type(&self) -> Result<&'static str, CssomError> {
                self.snapshot.sheet_type(self.handle.id())
            }
            pub fn href(&self) -> Result<Option<&str>, CssomError> {
                self.snapshot.sheet_href(self.handle.id())
            }
            pub fn owner_node(&self) -> Result<Option<&str>, CssomError> {
                self.snapshot.sheet_owner_node(self.handle.id())
            }
            pub fn title(&self) -> Result<Option<&str>, CssomError> {
                self.snapshot.sheet_title(self.handle.id())
            }
            pub fn parent_style_sheet(
                &self,
            ) -> Result<Option<CssomCapture<CssStyleSheet>>, CssomError> {
                self.snapshot
                    .parent_sheet(self.handle.id())?
                    .map(|id| {
                        CssStyleSheet::from_state(&self.snapshot.state, id)?.capture(&self.snapshot)
                    })
                    .transpose()
            }
            pub fn media(&self) -> Result<CssomCapture<MediaList>, CssomError> {
                MediaList::from_state(
                    &self.snapshot.state,
                    self.snapshot.sheet_media(self.handle.id())?,
                )?
                .capture(&self.snapshot)
            }
            pub fn disabled(&self) -> Result<bool, CssomError> {
                self.snapshot.sheet_disabled(self.handle.id())
            }
        }
    };
}
sheet_reads!(StyleSheet);
sheet_reads!(CssStyleSheet);

impl CssStyleSheet {
    /// Construct and validate the returned handle before the single publication.
    /// Construction does not add the sheet to the host's selected sheet list.
    pub fn new(
        store: &mut CssomStore,
        document: CssomConstructorDocument,
        init: CssStyleSheetInit<'_>,
    ) -> Result<CssomOperationOutcome<Self>, CssomError> {
        execute(store, |batch| {
            let media = match init.media {
                CssStyleSheetMedia::Text(text) => CssomInitialMedia::Text(text),
                CssStyleSheetMedia::List(capture) => CssomInitialMedia::List {
                    snapshot: &capture.snapshot,
                    id: capture.handle.id(),
                },
            };
            let (ticket, diagnostics) = batch
                .construct_sheet(
                    document,
                    CssomSheetInit {
                        base_url: init.base_url,
                        media,
                        disabled: init.disabled,
                    },
                )?
                .into_parts();
            let value = Self::from_state(&batch.staged, batch.created_sheet_id(&ticket)?)?;
            Ok(CssomStagedOperation::new(value, diagnostics))
        })
    }
    pub fn owner_rule(&self, store: &CssomStore) -> Result<Option<CssRule>, CssomError> {
        store
            .state
            .sheet(self.id())?
            .owner_rule()
            .map(|id| CssRule::from_state(&store.state, id))
            .transpose()
    }
    /// Access is origin-clean guarded; subsequent list reads use its identity.
    pub fn css_rules(&self, store: &CssomStore) -> Result<CssRuleList, CssomError> {
        CssRuleList::from_state(&store.state, store.snapshot().sheet_css_rules(self.id())?)
    }
    /// The CSS-defined deprecated alias shares the same guarded list identity.
    pub fn rules(&self, store: &CssomStore) -> Result<CssRuleList, CssomError> {
        self.css_rules(store)
    }
    pub fn insert_rule(
        &self,
        store: &mut CssomStore,
        rule: &str,
        index: Option<usize>,
    ) -> Result<CssomOperationOutcome<usize>, CssomError> {
        execute(store, |batch| {
            let (index, _) = batch.insert_sheet_rule(self.id(), rule, index.unwrap_or(0))?;
            let diagnostics = insertion_diagnostics(batch, self.id(), index)?;
            Ok(CssomStagedOperation::new(index, diagnostics))
        })
    }
    pub fn delete_rule(
        &self,
        store: &mut CssomStore,
        index: usize,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            batch.delete_sheet_rule(self.id(), index)?;
            Ok(CssomStagedOperation::new((), Vec::new()))
        })
    }
    /// CSS-defined deprecated insertion with missing strings equal to "undefined".
    pub fn add_rule(
        &self,
        store: &mut CssomStore,
        selector: Option<&str>,
        style: Option<&str>,
        index: Option<usize>,
    ) -> Result<CssomOperationOutcome<i32>, CssomError> {
        execute(store, |batch| {
            let value = batch.add_sheet_rule(self.id(), selector, style, index)?;
            let list = batch.staged.sheet(self.id())?.rules();
            let index = index.unwrap_or(batch.staged.lists[list].len() - 1);
            Ok(CssomStagedOperation::new(
                value,
                insertion_diagnostics(batch, self.id(), index)?,
            ))
        })
    }
    pub fn remove_rule(
        &self,
        store: &mut CssomStore,
        index: Option<usize>,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        self.delete_rule(store, index.unwrap_or(0))
    }
    pub fn replace_sync(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            batch.replace_sheet_sync(self.id(), text)?;
            Ok(CssomStagedOperation::new(
                (),
                copy_diagnostics(batch.staged.sheet(self.id())?.diagnostics())?,
            ))
        })
    }
    /// Begin the executor-neutral native lifecycle. Parse work is detached and
    /// explicitly consumed; dropping it leaves the native cancellable token.
    pub fn replace(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomStylesheetReplacement, CssomError> {
        let job = store.begin_replace_sheet(
            CssomEditGuard::from_snapshot(&store.snapshot()),
            self.id(),
            text,
        )?;
        Ok(CssomStylesheetReplacement {
            sheet: self.clone(),
            job,
        })
    }
}
impl CssomCapture<CssStyleSheet> {
    pub fn owner_rule(&self) -> Result<Option<CssomCapture<CssRule>>, CssomError> {
        self.snapshot
            .sheet_owner_rule(self.handle.id())?
            .map(|id| CssRule::from_state(&self.snapshot.state, id)?.capture(&self.snapshot))
            .transpose()
    }
    pub fn css_rules(&self) -> Result<CssomCapture<CssRuleList>, CssomError> {
        CssRuleList::from_state(
            &self.snapshot.state,
            self.snapshot.sheet_css_rules(self.handle.id())?,
        )?
        .capture(&self.snapshot)
    }
    pub fn rules(&self) -> Result<CssomCapture<CssRuleList>, CssomError> {
        self.css_rules()
    }
}
fn copy_diagnostics(
    source: &[CssRecoveryDiagnostic],
) -> Result<Vec<CssRecoveryDiagnostic>, CssomError> {
    let mut diagnostics = Vec::new();
    diagnostics
        .try_reserve_exact(source.len())
        .map_err(|_| CssomError::Limit {
            resource: "object operation diagnostics capacity",
            maximum: source.len(),
        })?;
    diagnostics.extend_from_slice(source);
    Ok(diagnostics)
}
fn insertion_diagnostics(
    batch: &CssomBatch,
    sheet: &CssomSheetId,
    index: usize,
) -> Result<Vec<CssRecoveryDiagnostic>, CssomError> {
    let list = batch.staged.sheet(sheet)?.rules();
    let rule = &batch.staged.lists[list][index];
    copy_diagnostics(batch.staged.rule(rule)?.admission_diagnostics())
}

impl MediaList {
    pub fn media_text(
        &self,
        store: &CssomStore,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        store.snapshot().media_text(self.id(), limits)
    }
    pub fn length(&self, store: &CssomStore) -> Result<usize, CssomError> {
        store.snapshot().media_length(self.id())
    }
    pub fn item(
        &self,
        store: &CssomStore,
        index: usize,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        store.snapshot().media_item(self.id(), index, limits)
    }
    pub fn set_media_text(
        &self,
        store: &mut CssomStore,
        text: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Ok(CssomStagedOperation::new(
                (),
                batch.set_media_text(self.id(), text)?.into_diagnostics(),
            ))
        })
    }
    pub fn append_medium(
        &self,
        store: &mut CssomStore,
        medium: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Ok(CssomStagedOperation::new(
                (),
                batch.append_medium(self.id(), medium)?.into_diagnostics(),
            ))
        })
    }
    pub fn delete_medium(
        &self,
        store: &mut CssomStore,
        medium: &str,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            Ok(CssomStagedOperation::new(
                (),
                batch.delete_medium(self.id(), medium)?.into_diagnostics(),
            ))
        })
    }
}
impl CssomCapture<MediaList> {
    pub fn media_text(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        self.snapshot.media_text(self.handle.id(), limits)
    }
    pub fn length(&self) -> Result<usize, CssomError> {
        self.snapshot.media_length(self.handle.id())
    }
    pub fn item(
        &self,
        index: usize,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        self.snapshot.media_item(self.handle.id(), index, limits)
    }
}
impl CssRuleList {
    pub fn length(&self, store: &CssomStore) -> Result<usize, CssomError> {
        store.snapshot().rule_list_length(self.id())
    }
    pub fn item(&self, store: &CssomStore, index: usize) -> Result<Option<CssRule>, CssomError> {
        store
            .snapshot()
            .rule_list_item(self.id(), index)?
            .map(|id| CssRule::from_state(&store.state, id))
            .transpose()
    }
}
impl CssomCapture<CssRuleList> {
    pub fn length(&self) -> Result<usize, CssomError> {
        self.snapshot.rule_list_length(self.handle.id())
    }
    pub fn item(&self, index: usize) -> Result<Option<CssomCapture<CssRule>>, CssomError> {
        self.snapshot
            .rule_list_item(self.handle.id(), index)?
            .map(|id| CssRule::from_state(&self.snapshot.state, id)?.capture(&self.snapshot))
            .transpose()
    }
}
impl StyleSheetList {
    pub fn length(&self, store: &CssomStore) -> Result<usize, CssomError> {
        store.snapshot().sheet_list_length(self.id())
    }
    pub fn item(
        &self,
        store: &CssomStore,
        index: usize,
    ) -> Result<Option<CssStyleSheet>, CssomError> {
        store
            .snapshot()
            .sheet_list_item(self.id(), index)?
            .map(|id| CssStyleSheet::from_state(&store.state, id))
            .transpose()
    }
}
impl CssomCapture<StyleSheetList> {
    pub fn length(&self) -> Result<usize, CssomError> {
        self.snapshot.sheet_list_length(self.handle.id())
    }
    pub fn item(&self, index: usize) -> Result<Option<CssomCapture<CssStyleSheet>>, CssomError> {
        self.snapshot
            .sheet_list_item(self.handle.id(), index)?
            .map(|id| CssStyleSheet::from_state(&self.snapshot.state, id)?.capture(&self.snapshot))
            .transpose()
    }
}

/// Begun replacement and its exact native token. No scheduler is implied.
#[must_use = "parse and finish, or explicitly cancel, this replacement"]
pub struct CssomStylesheetReplacement {
    sheet: CssStyleSheet,
    job: CssomReplaceJob,
}
impl CssomStylesheetReplacement {
    pub fn sheet(&self) -> &CssStyleSheet {
        &self.sheet
    }
    pub fn token(&self) -> &CssomReplaceToken {
        self.job.token()
    }
    pub fn begin_publication(&self) -> &CssomPublication {
        self.job.begin_publication()
    }
    pub fn parse(self) -> CssomStylesheetReplacementWork {
        CssomStylesheetReplacementWork {
            sheet: self.sheet,
            work: self.job.parse(),
        }
    }
    pub fn cancel(
        self,
        store: &mut CssomStore,
    ) -> Result<CssomOperationOutcome<CssomStylesheetReplacementResult>, CssomError> {
        CssStyleSheet::from_state(&store.state, self.sheet.id())?;
        let completion = store.cancel_replace_sheet(self.job.token())?;
        Ok(replacement_outcome(self.sheet, completion))
    }
}
/// Detached, bounded, Send parse result; it carries no live owner capability.
#[must_use = "finish or explicitly cancel this replacement work"]
pub struct CssomStylesheetReplacementWork {
    sheet: CssStyleSheet,
    work: CssomReplaceWork,
}
impl CssomStylesheetReplacementWork {
    pub fn sheet(&self) -> &CssStyleSheet {
        &self.sheet
    }
    pub fn token(&self) -> &CssomReplaceToken {
        self.work.token()
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        self.work.diagnostics()
    }
    pub fn finish(
        self,
        store: &mut CssomStore,
    ) -> Result<CssomOperationOutcome<CssomStylesheetReplacementResult>, CssomError> {
        CssStyleSheet::from_state(&store.state, self.sheet.id())?;
        let guard = CssomEditGuard::from_snapshot(&store.snapshot());
        let completion = store.finish_replace_sheet(guard, self.work)?;
        Ok(replacement_outcome(self.sheet, completion))
    }
    pub fn cancel(
        self,
        store: &mut CssomStore,
    ) -> Result<CssomOperationOutcome<CssomStylesheetReplacementResult>, CssomError> {
        CssStyleSheet::from_state(&store.state, self.sheet.id())?;
        let completion = store.cancel_replace_sheet(self.work.token())?;
        Ok(replacement_outcome(self.sheet, completion))
    }
}
/// Terminal disposition and the typed sheet, including Failed with its real
/// cleanup publication in the enclosing shared outcome.
pub struct CssomStylesheetReplacementResult {
    sheet: CssStyleSheet,
    disposition: CssomReplaceOutcome,
}
impl CssomStylesheetReplacementResult {
    pub fn sheet(&self) -> &CssStyleSheet {
        &self.sheet
    }
    pub fn disposition(&self) -> &CssomReplaceOutcome {
        &self.disposition
    }
    pub fn into_parts(self) -> (CssStyleSheet, CssomReplaceOutcome) {
        (self.sheet, self.disposition)
    }
}
fn replacement_outcome(
    sheet: CssStyleSheet,
    completion: CssomReplaceCompletion,
) -> CssomOperationOutcome<CssomStylesheetReplacementResult> {
    let (_, publication, disposition, diagnostics) = completion.into_parts();
    CssomOperationOutcome::from_parts(
        CssomStylesheetReplacementResult { sheet, disposition },
        publication,
        diagnostics,
        Vec::new(),
    )
}
