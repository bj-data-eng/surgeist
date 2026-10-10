//! Source stylesheet operations and bounded replacement lifecycle.
use crate::{css_adapter::parse, *};
use std::sync::Arc;
use surgeist_css::*;

/// Supplied current-global Document facts; no document or URL resolution occurs.
#[derive(Clone, Debug)]
pub struct CssomConstructorDocument {
    pub version: CssomInputVersion,
    pub identity: String,
    pub base_location: String,
}

/// Actual source constructor media input. A list is copied through its CSSOM text.
pub enum CssomInitialMedia<'a> {
    Text(&'a str),
    List {
        snapshot: &'a CssomSnapshot,
        id: &'a CssomMediaListId,
    },
}
pub struct CssomSheetInit<'a> {
    pub base_url: Option<String>,
    pub media: CssomInitialMedia<'a>,
    pub disabled: bool,
}
impl Default for CssomSheetInit<'_> {
    fn default() -> Self {
        Self {
            base_url: None,
            media: CssomInitialMedia::Text(""),
            disabled: false,
        }
    }
}
/// Construction remains staged until commit resolves the creation ticket.
pub struct CssomSheetConstruction {
    ticket: CssomSheetTicket,
    media_diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssomSheetConstruction {
    pub fn ticket(&self) -> &CssomSheetTicket {
        &self.ticket
    }
    pub fn media_diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.media_diagnostics
    }
}

/// Current applicable owner attributes supplied by a host, without DOM inspection.
pub struct CssomSheetOwnerAttributes<'a> {
    pub owner: &'a str,
    pub version: CssomInputVersion,
    pub title: Option<&'a str>,
    pub media: Option<&'a str>,
}
pub struct CssomSheetAttributeEdit {
    changed: bool,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl CssomSheetAttributeEdit {
    pub const fn changed(&self) -> bool {
        self.changed
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
}

impl CssomSnapshot {
    pub fn sheet_type(&self, id: &CssomSheetId) -> Result<&'static str, CssomError> {
        self.sheet(id)?;
        Ok("text/css")
    }
    pub fn sheet_href(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        Ok(self.sheet(id)?.inputs.location.as_deref())
    }
    pub fn sheet_owner_node(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        Ok(self.sheet(id)?.inputs.owner.as_deref())
    }
    pub fn sheet_title(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        let title = &self.sheet(id)?.inputs.title;
        Ok((!title.is_empty()).then_some(title.as_str()))
    }
    pub fn sheet_media(&self, id: &CssomSheetId) -> Result<&CssomMediaListId, CssomError> {
        Ok(self.sheet(id)?.media())
    }
    pub fn sheet_disabled(&self, id: &CssomSheetId) -> Result<bool, CssomError> {
        Ok(self.sheet(id)?.inputs.disabled)
    }
    pub fn sheet_owner_rule(&self, id: &CssomSheetId) -> Result<Option<&CssomRuleId>, CssomError> {
        Ok(self.sheet(id)?.owner_rule())
    }
    /// Same live list identity, guarded independently of trusted snapshot transport.
    pub fn sheet_css_rules(&self, id: &CssomSheetId) -> Result<&CssomRuleListId, CssomError> {
        self.sheet_rules(id)?;
        Ok(self.sheet(id)?.rules())
    }
    /// CSS-defined deprecated alias; returns the same guarded list identity.
    pub fn sheet_legacy_rules(&self, id: &CssomSheetId) -> Result<&CssomRuleListId, CssomError> {
        self.sheet_css_rules(id)
    }
    fn sheet_list_members(&self, id: &CssomSheetListId) -> Result<&[CssomSheetId], CssomError> {
        if id.owner() != self.owner() {
            return Err(CssomError::ForeignOwner);
        }
        if id != self.sheet_list() {
            return Err(CssomError::MissingObject);
        }
        Ok(self.sheets())
    }
    pub fn sheet_list_length(&self, id: &CssomSheetListId) -> Result<usize, CssomError> {
        Ok(self.sheet_list_members(id)?.len())
    }
    pub fn sheet_list_item(
        &self,
        id: &CssomSheetListId,
        index: usize,
    ) -> Result<Option<&CssomSheetId>, CssomError> {
        Ok(self.sheet_list_members(id)?.get(index))
    }
}

impl CssomBatch {
    /// Source construction starts empty and does not assert host list membership.
    pub fn construct_sheet(
        &mut self,
        document: CssomConstructorDocument,
        init: CssomSheetInit<'_>,
    ) -> Result<CssomSheetConstruction, CssomError> {
        self.apply(|this| {
            if document.version.role != CssomInputRole::Document {
                return Err(CssomError::InvalidInput(
                    "constructor document version role",
                ));
            }
            let (media, media_diagnostics) = match init.media {
                CssomInitialMedia::Text(source) => parse_media(source, &this.limits)?,
                CssomInitialMedia::List { snapshot, id } => {
                    let text = snapshot
                        .media(id)?
                        .serialize_cssom_with_limits(this.limits.css)
                        .map_err(CssomError::Media)?;
                    parse_media(text.as_css(), &this.limits)?
                }
            };
            let mut inputs =
                CssomSheetInputs::constructed(document.version, document.identity, init.base_url);
            inputs.location = Some(document.base_location);
            inputs.disabled = init.disabled;
            inputs.media = media;
            let ticket = this.create_sheet(CssSheet::new(), Vec::new(), inputs, false)?;
            Ok(CssomSheetConstruction {
                ticket,
                media_diagnostics,
            })
        })
    }
    /// Publishes the supplied selected host collection; CSSOM does not select a DOM list.
    pub fn set_sheet_list_membership(
        &mut self,
        id: &CssomSheetListId,
        members: &[CssomSheetId],
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            if id.owner() != &this.staged.owner {
                return Err(CssomError::ForeignOwner);
            }
            if id != &this.staged.sheet_list {
                return Err(CssomError::MissingObject);
            }
            if members.len() > this.limits.max_entries {
                return Err(CssomError::Limit {
                    resource: "sheet list entries",
                    maximum: this.limits.max_entries,
                });
            }
            for member in members {
                this.staged.sheet(member)?;
            }
            if this.staged.sheet_order == members {
                return Ok(false);
            }
            let mut selected = Vec::new();
            selected
                .try_reserve_exact(members.len())
                .map_err(|_| CssomError::Limit {
                    resource: "sheet list capacity",
                    maximum: this.limits.max_entries,
                })?;
            selected.extend_from_slice(members);
            this.staged.sheet_order = selected;
            this.affect(
                CssomObjectId::SheetList(id.clone()),
                &[CssomChange::SheetMembership, CssomChange::Order],
            );
            Ok(true)
        })
    }
    /// Reflects applicable host attribute facts without changing fixed location or owners.
    pub fn update_sheet_owner_attributes(
        &mut self,
        id: &CssomSheetId,
        attributes: CssomSheetOwnerAttributes<'_>,
    ) -> Result<CssomSheetAttributeEdit, CssomError> {
        self.apply(|this| {
            let sheet = this.staged.sheet(id)?;
            if sheet.inputs.owner.as_deref() != Some(attributes.owner) {
                return Err(CssomError::InvalidInput(
                    "owner attribute association mismatch",
                ));
            }
            let media_id = sheet.media.clone();
            let title = attributes.title.unwrap_or("");
            let (media, diagnostics) = parse_media(attributes.media.unwrap_or(""), &this.limits)?;
            let mut title_value = String::new();
            if title.len() > this.limits.max_string_bytes {
                return Err(CssomError::Limit {
                    resource: "owner title bytes",
                    maximum: this.limits.max_string_bytes,
                });
            }
            title_value
                .try_reserve_exact(title.len())
                .map_err(|_| CssomError::Limit {
                    resource: "owner title capacity",
                    maximum: this.limits.max_string_bytes,
                })?;
            title_value.push_str(title);
            let changed =
                sheet.inputs.title != title_value || sheet.inputs.version != attributes.version;
            let media_changed = this.set_media(&media_id, media)?;
            if changed {
                let sheet = this.staged.sheets.get_mut(id).expect("checked sheet");
                sheet.inputs.title = title_value;
                sheet.inputs.version = attributes.version;
                this.affect(
                    CssomObjectId::Sheet(id.clone()),
                    &[CssomChange::Owner, CssomChange::Provenance],
                );
            }
            Ok(CssomSheetAttributeEdit {
                changed: changed || media_changed,
                diagnostics,
            })
        })
    }
}

fn parse_media(
    source: &str,
    limits: &CssomLimits,
) -> Result<(CssMediaQueryList, Vec<CssRecoveryDiagnostic>), CssomError> {
    if source.is_empty() {
        return Ok((CssMediaQueryList::new(Vec::new()), Vec::new()));
    }
    if source.len() > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    let report = parse_media_query_list(source);
    for diagnostic in report.diagnostics() {
        if matches!(diagnostic.error().kind(), ErrorKind::NestingLimit(_))
            || matches!(diagnostic.error().kind(), ErrorKind::InvalidComponentValue(error)
                if matches!(error.kind(), CssComponentValueErrorKind::NestingLimit
                    | CssComponentValueErrorKind::ComponentLimit
                    | CssComponentValueErrorKind::ByteLimit
                    | CssComponentValueErrorKind::CapacityOverflow))
        {
            return Err(CssomError::ParseResource(Box::new(diagnostic.clone())));
        }
    }
    Ok(report.into_parts())
}

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
}
impl CssomReplaceCompletion {
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
        let token = edit.apply(|this| {
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
            let (authored, diagnostics) = work.result?;
            let mut edit = self.batch(guard)?;
            edit.apply(|this| {
                this.replace(&work.token.sheet, authored, diagnostics)?;
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
            }),
            Err(error) => self.end_replacement(&work.token, CssomReplaceOutcome::Failed(error)),
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
        self.checked_replacement(token)?;
        let mut edit = self.batch(CssomEditGuard::from_snapshot(&self.snapshot()))?;
        edit.apply(|this| {
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

impl CssomBatch {
    /// Standards-defined `addRule()` constructs its actual rule input, delegates
    /// the sole stylesheet insertion algorithm and returns the legacy -1 result.
    pub fn add_sheet_rule(
        &mut self,
        sheet: &CssomSheetId,
        selector: Option<&str>,
        block: Option<&str>,
        index: Option<usize>,
    ) -> Result<i32, CssomError> {
        // Product handle validation precedes reading the list default. Source
        // Security/NotAllowed and syntax guards belong to insert_sheet_rule.
        let (source, index) = self.apply(|this| {
            let current = this.staged.sheet(sheet)?;
            let index = index.unwrap_or_else(|| this.staged.lists[&current.rules].len());
            let selector = selector.unwrap_or("undefined");
            let block = block.unwrap_or("undefined");
            let bytes = selector
                .len()
                .checked_add(3)
                .and_then(|v| v.checked_add(block.len()))
                .and_then(|v| v.checked_add(usize::from(!block.is_empty())))
                .and_then(|v| v.checked_add(1))
                .filter(|v| *v <= this.limits.max_input_bytes)
                .ok_or(CssomError::Limit {
                    resource: "legacy rule input bytes",
                    maximum: this.limits.max_input_bytes,
                })?;
            let mut source = String::new();
            source
                .try_reserve_exact(bytes)
                .map_err(|_| CssomError::Limit {
                    resource: "legacy rule input capacity",
                    maximum: this.limits.max_input_bytes,
                })?;
            source.push_str(selector);
            source.push_str(" { ");
            if !block.is_empty() {
                source.push_str(block);
                source.push(' ');
            }
            source.push('}');
            Ok((source, index))
        })?;
        self.insert_sheet_rule(sheet, &source, index)?;
        Ok(-1)
    }
    /// Standards-defined `removeRule()` defaults its index to zero.
    pub fn remove_sheet_rule(
        &mut self,
        sheet: &CssomSheetId,
        index: Option<usize>,
    ) -> Result<CssomRuleId, CssomError> {
        self.delete_sheet_rule(sheet, index.unwrap_or(0))
    }
    /// Synchronous CSSOM replacement uses the sole recovered replacement path.
    pub fn replace_sheet_sync(
        &mut self,
        sheet: &CssomSheetId,
        source: &str,
    ) -> Result<(), CssomError> {
        self.replace_sheet_text(sheet, source)
    }
}
