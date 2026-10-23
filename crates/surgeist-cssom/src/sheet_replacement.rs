//! Bounded detached stylesheet replacement and terminal cleanup.
use crate::{css_adapter::parse, *};
use std::sync::Arc;
use surgeist_css::*;

/// Exact owner-qualified operation identity; never the publication revision alone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomReplaceToken {
    owner: CssomOwnerId,
    sheet: CssomSheetId,
    serial: u64,
}
impl CssomReplaceToken {
    pub fn owner(&self) -> &CssomOwnerId {
        &self.owner
    }
    pub fn sheet(&self) -> &CssomSheetId {
        &self.sheet
    }
}

struct ReplacementInput {
    text: String,
    parser: CssParserContext,
    parser_input: Option<CssomInput>,
    sheet_version: CssomInputVersion,
    base_url: Option<String>,
    location: Option<String>,
    constructor_document: Option<String>,
}
pub(crate) struct PendingReplacement {
    token: CssomReplaceToken,
    input: Arc<ReplacementInput>,
    bytes: usize,
}

/// Detached bounded parse work. Dropping it leaves an explicitly cancellable owner record.
#[derive(Clone)]
pub struct CssomReplaceJob {
    token: CssomReplaceToken,
    input: Arc<ReplacementInput>,
    limits: CssomLimits,
    begin: CssomPublication,
}
impl CssomReplaceJob {
    pub fn token(&self) -> &CssomReplaceToken {
        &self.token
    }
    pub fn begin_publication(&self) -> &CssomPublication {
        &self.begin
    }
    /// CSS-owned recovery, independently executable without borrowing the live store.
    pub fn parse(self) -> CssomReplaceWork {
        let result = parse(&self.input.text, &self.limits, self.input.parser);
        CssomReplaceWork {
            token: self.token,
            result,
        }
    }
}
pub struct CssomReplaceWork {
    token: CssomReplaceToken,
    result: Result<(CssSheet, Vec<CssRecoveryDiagnostic>), CssomError>,
}
impl CssomReplaceWork {
    /// Recovery from completed detached parsing; a parse failure has no successful report.
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        match &self.result {
            Ok((_, diagnostics)) => diagnostics,
            Err(_) => &[],
        }
    }
    pub fn token(&self) -> &CssomReplaceToken {
        &self.token
    }
}
#[derive(Debug)]
pub enum CssomReplaceOutcome {
    Replaced,
    Cancelled,
    Failed(CssomError),
}
pub struct CssomReplaceCompletion {
    sheet: CssomSheetId,
    publication: CssomPublication,
    outcome: CssomReplaceOutcome,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssomReplaceCompletion {
    /// Preserve the actual terminal publication even when adoption failed.
    pub fn into_parts(
        self,
    ) -> (
        CssomSheetId,
        CssomPublication,
        CssomReplaceOutcome,
        Vec<CssRecoveryDiagnostic>,
    ) {
        (self.sheet, self.publication, self.outcome, self.diagnostics)
    }
    /// Original completed parse report, including a report whose adoption failed.
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
    pub fn sheet(&self) -> &CssomSheetId {
        &self.sheet
    }
    pub fn publication(&self) -> &CssomPublication {
        &self.publication
    }
    pub fn outcome(&self) -> &CssomReplaceOutcome {
        &self.outcome
    }
}

impl CssomStore {
    /// Starts one exclusive replacement for this sheet. The owner retains a
    /// cancellable record and reserves quotas for its terminal publication.
    pub fn begin_replace_sheet(
        &mut self,
        guard: CssomEditGuard,
        sheet: &CssomSheetId,
        source: &str,
    ) -> Result<CssomReplaceJob, CssomError> {
        self.check_guard(&guard)?;
        let facts = self.state.sheet(sheet)?.inputs();
        if !facts.constructed || facts.disallow_modification {
            return Err(CssomError::Source(CssomException::NotAllowed));
        }
        if source.len() > self.limits.max_input_bytes {
            return Err(CssomError::Limit {
                resource: "parse input bytes",
                maximum: self.limits.max_input_bytes,
            });
        }
        let parser_input = parser_input(&self.state.context).cloned();
        let bytes = [
            source.len(),
            facts.version.identity.len(),
            facts.base_url.as_ref().map_or(0, String::len),
            facts.location.as_ref().map_or(0, String::len),
            facts.constructor_document.as_ref().map_or(0, String::len),
            parser_input
                .as_ref()
                .map_or(0, |input| input.version.identity.len()),
        ]
        .into_iter()
        .try_fold(0usize, |total, value| total.checked_add(value))
        .ok_or(CssomError::Limit {
            resource: "pending replacement bytes",
            maximum: self.limits.max_string_bytes,
        })?;
        let (count, retained) = self.replacement_reservations(None)?;
        let reserved = (
            count.checked_add(1).ok_or(CssomError::Limit {
                resource: "pending replacements",
                maximum: self.limits.max_objects,
            })?,
            retained.checked_add(bytes).ok_or(CssomError::Limit {
                resource: "pending replacement bytes",
                maximum: self.limits.max_string_bytes,
            })?,
        );
        self.state
            .check_limits(&self.replacement_limits(reserved)?)?;
        let terminal_revisions = u64::try_from(reserved.0)
            .ok()
            .and_then(|count| count.checked_add(1))
            .and_then(|count| self.state.revision.value.checked_add(count));
        if !terminal_revisions.is_some_and(|end| end <= self.limits.max_revision) {
            return Err(CssomError::RevisionExhausted);
        }
        let mut text = String::new();
        text.try_reserve_exact(source.len())
            .map_err(|_| CssomError::Limit {
                resource: "replacement input capacity",
                maximum: self.limits.max_input_bytes,
            })?;
        text.push_str(source);
        let input = Arc::new(ReplacementInput {
            text,
            parser: self.state.context.parser_context(),
            parser_input,
            sheet_version: facts.version.clone(),
            base_url: facts.base_url.clone(),
            location: facts.location.clone(),
            constructor_document: facts.constructor_document.clone(),
        });
        self.pending_replacements
            .try_reserve(1)
            .map_err(|_| CssomError::Limit {
                resource: "pending replacement capacity",
                maximum: self.limits.max_objects,
            })?;
        let mut edit = self.batch(guard)?;
        let token = edit.apply(|this| -> Result<_, CssomError> {
            let serial = this.staged.allocate(&this.limits)?;
            this.staged
                .sheets
                .get_mut(sheet)
                .expect("checked sheet")
                .inputs
                .disallow_modification = true;
            this.affect(
                CssomObjectId::Sheet(sheet.clone()),
                &[CssomChange::Modification],
            );
            Ok(CssomReplaceToken {
                owner: this.staged.owner.clone(),
                sheet: sheet.clone(),
                serial,
            })
        })?;
        self.pending_replacements.insert(
            sheet.clone(),
            PendingReplacement {
                token: token.clone(),
                input: input.clone(),
                bytes,
            },
        );
        let committed = match self.commit(edit) {
            Ok(committed) => committed,
            Err(error) => {
                self.pending_replacements.remove(sheet);
                return Err(error);
            }
        };
        Ok(CssomReplaceJob {
            token,
            input,
            limits: self.limits.clone(),
            begin: committed.publication().clone(),
        })
    }

    /// Owner-retained cancellation identity, including after detached job drop.
    pub fn pending_sheet_replacement(
        &self,
        sheet: &CssomSheetId,
    ) -> Result<Option<&CssomReplaceToken>, CssomError> {
        self.state.sheet(sheet)?;
        Ok(self
            .pending_replacements
            .get(sheet)
            .map(|pending| &pending.token))
    }

    /// Foreign/reused tokens leave every active operation untouched. Failure for
    /// the exact current token releases its lock and retains the original rules.
    pub fn finish_replace_sheet(
        &mut self,
        guard: CssomEditGuard,
        work: CssomReplaceWork,
    ) -> Result<CssomReplaceCompletion, CssomError> {
        self.checked_replacement(&work.token)?;
        let (parsed, diagnostics) = match work.result {
            Ok((authored, diagnostics)) => (Ok(authored), diagnostics),
            Err(error) => (Err(error), Vec::new()),
        };
        let attempt = (|| {
            self.check_guard(&guard)?;
            let pending = self.checked_replacement(&work.token)?;
            let current = self.state.sheet(&work.token.sheet)?.inputs();
            if current.version != pending.input.sheet_version
                || current.base_url != pending.input.base_url
                || current.location != pending.input.location
                || current.constructor_document != pending.input.constructor_document
                || parser_input(&self.state.context) != pending.input.parser_input.as_ref()
            {
                return Err(CssomError::StaleContext);
            }
            let authored = parsed?;
            // The terminal result owns the original report. Reserve its staged
            // sheet copy inside the native failure/cleanup boundary, before commit.
            let mut stored_diagnostics = Vec::new();
            stored_diagnostics
                .try_reserve_exact(diagnostics.len())
                .map_err(|_| CssomError::Limit {
                    resource: "replacement diagnostics capacity",
                    maximum: diagnostics.len(),
                })?;
            stored_diagnostics.extend_from_slice(&diagnostics);
            let mut edit = self.batch(guard)?;
            edit.apply(|this| -> Result<_, CssomError> {
                this.replace(&work.token.sheet, authored, stored_diagnostics)?;
                this.staged
                    .sheets
                    .get_mut(&work.token.sheet)
                    .expect("checked sheet")
                    .inputs
                    .disallow_modification = false;
                this.affect(
                    CssomObjectId::Sheet(work.token.sheet.clone()),
                    &[CssomChange::Modification],
                );
                Ok(())
            })?;
            edit.terminal_replacement = Some(work.token.clone());
            self.commit(edit)
        })();
        match attempt {
            Ok(committed) => Ok(CssomReplaceCompletion {
                sheet: work.token.sheet,
                publication: committed.publication().clone(),
                outcome: CssomReplaceOutcome::Replaced,
                diagnostics,
            }),
            Err(error) => self.end_replacement_with_diagnostics(
                &work.token,
                CssomReplaceOutcome::Failed(error),
                diagnostics,
            ),
        }
    }

    /// Explicit terminal cleanup uses current owner state, independently of stale
    /// caller guards or detached parse work. No host scheduler is implied.
    pub fn cancel_replace_sheet(
        &mut self,
        token: &CssomReplaceToken,
    ) -> Result<CssomReplaceCompletion, CssomError> {
        self.end_replacement(token, CssomReplaceOutcome::Cancelled)
    }

    fn end_replacement(
        &mut self,
        token: &CssomReplaceToken,
        outcome: CssomReplaceOutcome,
    ) -> Result<CssomReplaceCompletion, CssomError> {
        self.end_replacement_with_diagnostics(token, outcome, Vec::new())
    }
    fn end_replacement_with_diagnostics(
        &mut self,
        token: &CssomReplaceToken,
        outcome: CssomReplaceOutcome,
        diagnostics: Vec<CssRecoveryDiagnostic>,
    ) -> Result<CssomReplaceCompletion, CssomError> {
        self.checked_replacement(token)?;
        let mut edit = self.batch(CssomEditGuard::from_snapshot(&self.snapshot()))?;
        edit.apply(|this| -> Result<_, CssomError> {
            this.staged
                .sheets
                .get_mut(&token.sheet)
                .expect("checked sheet")
                .inputs
                .disallow_modification = false;
            this.affect(
                CssomObjectId::Sheet(token.sheet.clone()),
                &[CssomChange::Modification],
            );
            Ok(())
        })?;
        edit.terminal_replacement = Some(token.clone());
        let committed = self.commit(edit)?;
        Ok(CssomReplaceCompletion {
            sheet: token.sheet.clone(),
            publication: committed.publication().clone(),
            outcome,
            diagnostics,
        })
    }

    fn checked_replacement(
        &self,
        token: &CssomReplaceToken,
    ) -> Result<&PendingReplacement, CssomError> {
        if token.owner != self.state.owner {
            return Err(CssomError::ForeignOwner);
        }
        self.pending_replacements
            .get(&token.sheet)
            .filter(|pending| &pending.token == token)
            .ok_or(CssomError::StaleReplacement)
    }

    pub(crate) fn replacement_reservations(
        &self,
        terminal: Option<&CssomReplaceToken>,
    ) -> Result<(usize, usize), CssomError> {
        if let Some(token) = terminal {
            self.checked_replacement(token)?;
        }
        self.pending_replacements
            .values()
            .filter(|pending| terminal != Some(&pending.token))
            .try_fold((0usize, 0usize), |(count, bytes), pending| {
                Ok((
                    count.checked_add(1).ok_or(CssomError::Limit {
                        resource: "pending replacements",
                        maximum: self.limits.max_objects,
                    })?,
                    bytes.checked_add(pending.bytes).ok_or(CssomError::Limit {
                        resource: "pending replacement bytes",
                        maximum: self.limits.max_string_bytes,
                    })?,
                ))
            })
    }

    pub(crate) fn replacement_limits(
        &self,
        (count, bytes): (usize, usize),
    ) -> Result<CssomLimits, CssomError> {
        let mut limits = self.limits.clone();
        limits.max_objects = limits
            .max_objects
            .checked_sub(count)
            .ok_or(CssomError::Limit {
                resource: "pending replacements",
                maximum: self.limits.max_objects,
            })?;
        limits.max_string_bytes =
            limits
                .max_string_bytes
                .checked_sub(bytes)
                .ok_or(CssomError::Limit {
                    resource: "pending replacement bytes",
                    maximum: self.limits.max_string_bytes,
                })?;
        Ok(limits)
    }
}

fn parser_input(context: &CssomContext) -> Option<&CssomInput> {
    context
        .inputs
        .iter()
        .find(|input| matches!(input.data, CssomInputData::ParserMode(_)))
}
