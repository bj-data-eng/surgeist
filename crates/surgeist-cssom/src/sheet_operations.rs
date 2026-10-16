//! Source stylesheet construction, attributes and collection operations.
use crate::parser_resources::check_resource_diagnostics;
use crate::*;
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
    pub(crate) fn into_parts(self) -> (CssomSheetTicket, Vec<CssRecoveryDiagnostic>) {
        (self.ticket, self.media_diagnostics)
    }
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
    pub(crate) fn sheet_type(&self, id: &CssomSheetId) -> Result<&'static str, CssomError> {
        self.sheet(id)?;
        Ok("text/css")
    }
    pub(crate) fn sheet_href(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        self.state.sheet_href(id)
    }
    pub(crate) fn sheet_owner_node(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        self.state.sheet_owner_node(id)
    }
    pub(crate) fn sheet_title(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        self.state.sheet_title(id)
    }
    pub(crate) fn sheet_media(&self, id: &CssomSheetId) -> Result<&CssomMediaListId, CssomError> {
        Ok(self.sheet(id)?.media())
    }
    pub(crate) fn sheet_disabled(&self, id: &CssomSheetId) -> Result<bool, CssomError> {
        Ok(self.sheet(id)?.inputs.disabled)
    }
    pub(crate) fn sheet_owner_rule(
        &self,
        id: &CssomSheetId,
    ) -> Result<Option<&CssomRuleId>, CssomError> {
        Ok(self.sheet(id)?.owner_rule())
    }
    /// Same live list identity, guarded independently of trusted snapshot transport.
    pub(crate) fn sheet_css_rules(
        &self,
        id: &CssomSheetId,
    ) -> Result<&CssomRuleListId, CssomError> {
        self.sheet_rules(id)?;
        Ok(self.sheet(id)?.rules())
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
    pub(crate) fn sheet_list_length(&self, id: &CssomSheetListId) -> Result<usize, CssomError> {
        Ok(self.sheet_list_members(id)?.len())
    }
    pub(crate) fn sheet_list_item(
        &self,
        id: &CssomSheetListId,
        index: usize,
    ) -> Result<Option<&CssomSheetId>, CssomError> {
        Ok(self.sheet_list_members(id)?.get(index))
    }
}

impl CssomBatch {
    /// Removes this sheet from the selected host list and clears its parent,
    /// owner node and owner rule. The sheet, rules and existing handles survive.
    ///
    /// An importing rule's forward `styleSheet` reference is retained. Pending
    /// replacement work and its modification lock are independent of removal.
    /// Returns whether selected membership or an owner link changed.
    pub fn remove_sheet(&mut self, id: &CssomSheetId) -> Result<bool, CssomError> {
        self.apply(|this| {
            let sheet = this.staged.sheet(id)?;
            let owned = sheet.inputs.owner.is_some();
            let imported = sheet.owner_rule.is_some();
            let retained = this
                .staged
                .sheet_order
                .iter()
                .filter(|sheet| *sheet != id)
                .count();
            let listed = retained != this.staged.sheet_order.len();
            if !listed && !owned && !imported {
                return Ok(false);
            }
            if listed {
                let mut members = Vec::new();
                members
                    .try_reserve_exact(retained)
                    .map_err(|_| CssomError::Limit {
                        resource: "sheet list capacity",
                        maximum: this.limits.max_entries,
                    })?;
                members.extend(
                    this.staged
                        .sheet_order
                        .iter()
                        .filter(|sheet| *sheet != id)
                        .cloned(),
                );
                this.staged.sheet_order = std::sync::Arc::new(members);
                this.affect(
                    CssomObjectId::SheetList(this.staged.sheet_list.clone()),
                    &[CssomChange::SheetMembership, CssomChange::Order],
                );
                this.affect(
                    CssomObjectId::Sheet(id.clone()),
                    &[CssomChange::SheetMembership],
                );
            }
            if owned || imported {
                let sheet = this.staged.sheets.get_mut(id).expect("checked sheet");
                sheet.inputs.owner = None;
                sheet.owner_rule = None;
                this.affect(CssomObjectId::Sheet(id.clone()), &[CssomChange::Owner]);
            }
            if imported {
                this.affect(
                    CssomObjectId::Sheet(id.clone()),
                    &[CssomChange::Ancestry, CssomChange::ImportLinks],
                );
                this.full_recompute = true;
            }
            Ok(true)
        })
    }
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
            if this.staged.sheet_order.as_slice() == members {
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
            this.staged.sheet_order = std::sync::Arc::new(selected);
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
    check_resource_diagnostics(report.diagnostics())?;
    Ok(report.into_parts())
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

impl crate::model::State {
    pub(crate) fn sheet_href(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        Ok(self.sheet(id)?.inputs.location.as_deref())
    }
    pub(crate) fn sheet_owner_node(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        Ok(self.sheet(id)?.inputs.owner.as_deref())
    }
    pub(crate) fn sheet_title(&self, id: &CssomSheetId) -> Result<Option<&str>, CssomError> {
        let title = &self.sheet(id)?.inputs.title;
        Ok((!title.is_empty()).then_some(title.as_str()))
    }
}
