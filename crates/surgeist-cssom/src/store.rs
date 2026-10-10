use crate::{
    css_adapter::{check_declarations, parse, projection},
    model::State,
    *,
};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
};
use surgeist_css::*;

/// Exclusive live writer. Snapshots/batches own captures and never borrow this store.
pub struct CssomStore {
    pub(crate) state: Arc<State>,
    pub(crate) limits: CssomLimits,
    pub(crate) history: VecDeque<CssomChangeSummary>,
    pub(crate) pending_replacements:
        HashMap<CssomSheetId, crate::sheet_operations::PendingReplacement>,
    pub(crate) effect_quota: Arc<crate::owner_effect::EffectQuota>,
    pub(crate) next_effect: u64,
    pub(crate) updating: Option<CssomBlockId>,
}
impl CssomStore {
    pub fn new(limits: CssomLimits, context: CssomContext) -> Result<Self, CssomError> {
        if limits.max_identity == 0 {
            return Err(CssomError::IdentityExhausted);
        }
        let owner = CssomOwnerId::new();
        let state = State {
            revision: CssomRevision {
                owner: owner.clone(),
                value: 0,
            },
            sheet_list: CssomSheetListId {
                owner: owner.clone(),
                serial: 1,
            },
            owner,
            next_identity: 2,
            context,
            sheet_order: Vec::new(),
            sheets: HashMap::new(),
            rules: HashMap::new(),
            blocks: HashMap::new(),
            lists: HashMap::new(),
            media: HashMap::new(),
            maps: HashMap::new(),
        };
        state.check_limits(&limits)?;
        Ok(Self {
            state: Arc::new(state),
            limits,
            history: VecDeque::new(),
            pending_replacements: HashMap::new(),
            effect_quota: Arc::new(crate::owner_effect::EffectQuota::default()),
            next_effect: 1,
            updating: None,
        })
    }
    pub fn snapshot(&self) -> CssomSnapshot {
        CssomSnapshot {
            state: self.state.clone(),
        }
    }
    pub fn limits(&self) -> &CssomLimits {
        &self.limits
    }
    pub fn batch(&self, guard: CssomEditGuard) -> Result<CssomBatch, CssomError> {
        self.check_guard(&guard)?;
        Ok(CssomBatch {
            token: Arc::new(()),
            base: self.state.clone(),
            staged: (*self.state).clone(),
            limits: self.limits.clone(),
            poisoned: false,
            categories: BTreeSet::new(),
            affected: Vec::new(),
            created: Vec::new(),
            full_recompute: false,
            terminal_replacement: None,
            declaration_changes: Vec::new(),
            owner_updates: Vec::new(),
            pending_preparations: Arc::new(AtomicUsize::new(0)),
        })
    }
    pub(crate) fn check_guard(&self, guard: &CssomEditGuard) -> Result<(), CssomError> {
        if guard.revision.owner != self.state.owner {
            return Err(CssomError::ForeignOwner);
        }
        if guard.revision != self.state.revision {
            return Err(CssomError::StaleRevision);
        }
        if guard.context != self.state.context {
            return Err(CssomError::StaleContext);
        }
        Ok(())
    }
    /// An operation error poisons the whole batch. Stale commits publish nothing.
    pub fn commit(&mut self, batch: CssomBatch) -> Result<CssomCommit, CssomError> {
        self.check_guard(&CssomEditGuard {
            revision: batch.base.revision.clone(),
            context: batch.base.context.clone(),
        })?;
        if batch.pending_preparations.load(Ordering::Relaxed) != 0 {
            return Err(CssomError::UnresolvedDeclarationPreparation);
        }
        if batch.poisoned {
            return Err(CssomError::BatchAborted);
        }
        let reserved = self.replacement_reservations(batch.terminal_replacement.as_ref())?;
        batch
            .staged
            .check_limits(&self.replacement_limits(reserved)?)?;
        let changed = batch.changed();
        if !changed {
            let (effects, next_effect) =
                self.reserve_owner_effects(&batch, &self.state.revision)?;
            self.next_effect = next_effect;
            return Ok(CssomCommit {
                effects,
                token: batch.token,
                created: batch.created,
                publication: CssomPublication::Unchanged {
                    revision: self.state.revision.clone(),
                },
            });
        }
        let revision = self
            .state
            .revision
            .value
            .checked_add(1)
            .filter(|v| {
                u64::try_from(reserved.0)
                    .ok()
                    .and_then(|count| v.checked_add(count))
                    .is_some_and(|end| end <= self.limits.max_revision)
            })
            .ok_or(CssomError::RevisionExhausted)?;
        let effect_revision = CssomRevision {
            owner: self.state.owner.clone(),
            value: revision,
        };
        let (effects, next_effect) = self.reserve_owner_effects(&batch, &effect_revision)?;
        let before = self.snapshot();
        let mut next = batch.staged;
        next.revision = CssomRevision {
            owner: next.owner.clone(),
            value: revision,
        };
        let next = Arc::new(next);
        let after = CssomSnapshot {
            state: next.clone(),
        };
        let summary = CssomChangeSummary {
            owner: self.state.owner.clone(),
            from: self.state.revision.clone(),
            to: next.revision.clone(),
            categories: batch.categories,
            affected: batch.affected,
            before_inputs: before.inputs(),
            after_inputs: after.inputs(),
            full_recompute: batch.full_recompute,
        };
        // Allocate history before the only live publication. Allocator failure is typed.
        if self.limits.summary_history > 0 {
            let slots = reserved.0.checked_add(1).ok_or(CssomError::Limit {
                resource: "summary capacity",
                maximum: self.limits.summary_history,
            })?;
            self.history
                .try_reserve(slots)
                .map_err(|_| CssomError::Limit {
                    resource: "summary capacity",
                    maximum: self.limits.summary_history,
                })?;
            self.history.push_back(summary.clone());
            while self.history.len() > self.limits.summary_history {
                self.history.pop_front();
            }
        }
        self.state = next;
        if let Some(token) = batch.terminal_replacement {
            self.pending_replacements.remove(token.sheet());
        }
        self.next_effect = next_effect;
        Ok(CssomCommit {
            effects,
            token: batch.token,
            created: batch.created,
            publication: CssomPublication::Changed(Box::new(summary)),
        })
    }
    /// Coalesces complete retained history only. Gaps and input drift demand recompute.
    pub fn changes_since(&self, before: &CssomSnapshot) -> CssomChanges {
        if before.owner() != &self.state.owner {
            return CssomChanges::FullRecompute(CssomRecomputeReason::ForeignOwner);
        }
        if before.revision().value > self.state.revision.value {
            return CssomChanges::FullRecompute(CssomRecomputeReason::FutureRevision);
        }
        if before.revision() == &self.state.revision {
            return if before.inputs() == self.snapshot().inputs() {
                CssomChanges::Unchanged
            } else {
                CssomChanges::FullRecompute(CssomRecomputeReason::ContextMismatch)
            };
        }
        let mut result: Option<CssomChangeSummary> = None;
        let mut cursor = before.revision().clone();
        for summary in &self.history {
            if summary.to.value <= cursor.value {
                continue;
            }
            if summary.from != cursor {
                return CssomChanges::FullRecompute(CssomRecomputeReason::CoverageGap);
            }
            if result
                .as_ref()
                .map_or_else(|| before.inputs(), |v| v.after_inputs.clone())
                != summary.before_inputs
            {
                return CssomChanges::FullRecompute(CssomRecomputeReason::ContextMismatch);
            }
            if let Some(total) = &mut result {
                total.to = summary.to.clone();
                total.after_inputs = summary.after_inputs.clone();
                total.categories.extend(summary.categories.iter().copied());
                for id in &summary.affected {
                    if !total.affected.contains(id) {
                        total.affected.push(id.clone());
                    }
                }
                total.full_recompute |= summary.full_recompute;
            } else {
                result = Some(summary.clone());
            }
            cursor = summary.to.clone();
        }
        if cursor != self.state.revision {
            return CssomChanges::FullRecompute(CssomRecomputeReason::CoverageGap);
        }
        match result {
            Some(v) if v.full_recompute => {
                CssomChanges::FullRecompute(CssomRecomputeReason::ConservativeChange)
            }
            Some(v) => CssomChanges::Incremental(Box::new(v)),
            None => CssomChanges::FullRecompute(CssomRecomputeReason::CoverageGap),
        }
    }
}

/// Batch-local creation references cannot be used as live identities.
#[derive(Clone, Debug)]
pub struct CssomSheetTicket {
    pub(crate) token: Arc<()>,
    index: usize,
}
#[derive(Clone, Debug)]
pub struct CssomBlockTicket {
    pub(crate) token: Arc<()>,
    index: usize,
}
#[derive(Clone, Debug)]
pub struct CssomRuleTicket {
    pub(crate) token: Arc<()>,
    index: usize,
}
#[derive(Clone, Debug)]
enum Created {
    Sheet(CssomSheetId),
    Block(CssomBlockId),
    Rule(CssomRuleId),
}
pub struct CssomCommit {
    effects: Vec<CssomOwnerEffect>,
    pub(crate) token: Arc<()>,
    created: Vec<Created>,
    publication: CssomPublication,
}
impl CssomCommit {
    pub fn owner_effects(&self) -> &[CssomOwnerEffect] {
        &self.effects
    }
    pub fn take_owner_effects(&mut self) -> Vec<CssomOwnerEffect> {
        std::mem::take(&mut self.effects)
    }
    pub fn publication(&self) -> &CssomPublication {
        &self.publication
    }
    pub fn sheet(&self, ticket: &CssomSheetTicket) -> Result<&CssomSheetId, CssomError> {
        if !Arc::ptr_eq(&self.token, &ticket.token) {
            return Err(CssomError::ForeignTicket);
        }
        match self.created.get(ticket.index) {
            Some(Created::Sheet(v)) => Ok(v),
            _ => Err(CssomError::ForeignTicket),
        }
    }
    pub fn block(&self, ticket: &CssomBlockTicket) -> Result<&CssomBlockId, CssomError> {
        if !Arc::ptr_eq(&self.token, &ticket.token) {
            return Err(CssomError::ForeignTicket);
        }
        match self.created.get(ticket.index) {
            Some(Created::Block(v)) => Ok(v),
            _ => Err(CssomError::ForeignTicket),
        }
    }
    pub fn rule(&self, ticket: &CssomRuleTicket) -> Result<&CssomRuleId, CssomError> {
        if !Arc::ptr_eq(&self.token, &ticket.token) {
            return Err(CssomError::ForeignTicket);
        }
        match self.created.get(ticket.index) {
            Some(Created::Rule(v)) => Ok(v),
            _ => Err(CssomError::ForeignTicket),
        }
    }
}

/// Private staging state; dropped/failed batches expose no tentative live IDs.
pub struct CssomBatch {
    pub(crate) token: Arc<()>,
    pub(crate) base: Arc<State>,
    pub(crate) staged: State,
    pub(crate) limits: CssomLimits,
    pub(crate) poisoned: bool,
    pub(crate) categories: BTreeSet<CssomChange>,
    pub(crate) affected: Vec<CssomObjectId>,
    created: Vec<Created>,
    pub(crate) full_recompute: bool,
    pub(crate) declaration_changes: Vec<CssomBlockId>,
    pub(crate) owner_updates: Vec<(CssomBlockId, crate::declaration_support::DeclarationBudget)>,
    pub(crate) pending_preparations: Arc<AtomicUsize>,
    pub(crate) terminal_replacement: Option<CssomReplaceToken>,
}
impl CssomBatch {
    pub(crate) fn apply<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, CssomError>,
    ) -> Result<T, CssomError> {
        if self.poisoned {
            return Err(CssomError::BatchAborted);
        }
        let result = operation(self)
            .and_then(|value| self.staged.check_limits(&self.limits).map(|()| value));
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub(crate) fn affect(&mut self, id: CssomObjectId, changes: &[CssomChange]) {
        self.categories.extend(changes.iter().copied());
        if !self.affected.contains(&id) {
            self.affected.push(id);
        }
    }
    pub(crate) fn retain_rule_ticket(&mut self, id: CssomRuleId) -> CssomRuleTicket {
        let index = self.created.len();
        self.created.push(Created::Rule(id));
        CssomRuleTicket {
            token: self.token.clone(),
            index,
        }
    }
    pub fn create_parsed_sheet(
        &mut self,
        source: &str,
        inputs: CssomSheetInputs,
    ) -> Result<CssomSheetTicket, CssomError> {
        self.apply(|this| {
            let (sheet, diagnostics) =
                parse(source, &this.limits, this.staged.context.parser_context())?;
            this.create_sheet(sheet, diagnostics, inputs, true)
        })
    }
    pub fn create_typed_sheet(
        &mut self,
        sheet: CssSheet,
        inputs: CssomSheetInputs,
    ) -> Result<CssomSheetTicket, CssomError> {
        self.apply(|this| this.create_sheet(sheet, Vec::new(), inputs, true))
    }
    pub(crate) fn create_sheet(
        &mut self,
        authored: CssSheet,
        diagnostics: Vec<CssRecoveryDiagnostic>,
        inputs: CssomSheetInputs,
        register: bool,
    ) -> Result<CssomSheetTicket, CssomError> {
        let id = CssomSheetId {
            owner: self.staged.owner.clone(),
            serial: self.staged.allocate(&self.limits)?,
        };
        let rules = self.staged.list(&self.limits)?;
        let media = self.staged.media_list(inputs.media.clone(), &self.limits)?;
        for rule in authored.rules() {
            if inputs.constructed && matches!(rule, CssRule::Import(_)) {
                continue;
            }
            let child = self.staged.adopt(
                CssomAuthoredRule::Ordinary(rule.clone()),
                CssomParent::Sheet(id.clone()),
                1,
                &self.limits,
            )?;
            self.staged
                .lists
                .get_mut(&rules)
                .expect("new list")
                .push(child);
        }
        let namespaces = CssNamespaceContext::from_sheet(&authored);
        self.staged.sheets.insert(
            id.clone(),
            CssomSheet {
                inputs,
                rules,
                media,
                owner_rule: None,
                namespaces,
                authored,
                diagnostics,
            },
        );
        if register {
            self.staged.sheet_order.push(id.clone());
        }
        let index = self.created.len();
        self.created.push(Created::Sheet(id.clone()));
        self.affect(
            CssomObjectId::Sheet(id),
            &[
                CssomChange::SheetMembership,
                CssomChange::RuleMembership,
                CssomChange::Order,
                CssomChange::Ancestry,
                CssomChange::Namespaces,
                CssomChange::Provenance,
                CssomChange::Disabled,
                CssomChange::Media,
                CssomChange::Selectors,
                CssomChange::Declarations,
                CssomChange::Descriptors,
                CssomChange::FeatureMaps,
                CssomChange::ImportLinks,
                CssomChange::Origin,
                CssomChange::Layer,
                CssomChange::Base,
                CssomChange::Location,
                CssomChange::Parser,
                CssomChange::Support,
                CssomChange::Owner,
            ],
        );
        self.full_recompute = true;
        Ok(CssomSheetTicket {
            token: self.token.clone(),
            index,
        })
    }
    /// Supplied checked declaration data, including independent computed/readonly
    /// flags and nullable owner facts. No host attribute write is performed.
    pub fn create_declarations(
        &mut self,
        declarations: CssDeclarationList,
        computed: bool,
        readonly: bool,
        owner: Option<CssomInputVersion>,
    ) -> Result<CssomBlockTicket, CssomError> {
        self.apply(|this| {
            check_declarations(declarations.iter(), &this.limits)?;
            let selected = projection(
                CssSpecifiedDeclarationBlock::try_from_declarations_with_limits(
                    &declarations,
                    this.limits.css,
                ),
            )?;
            let id = this.staged.block_record(
                CssomBlockData::Properties {
                    domain: CssomPropertyDomain::Ordinary,
                    authored: CssomPropertyOccurrences::Ordinary(declarations),
                    selected,
                },
                None,
                owner,
                CssomBlockFlags {
                    computed,
                    readonly,
                    updating: false,
                },
                &this.limits,
            )?;
            let index = this.created.len();
            this.created.push(Created::Block(id.clone()));
            this.affect(
                CssomObjectId::Block(id),
                &[CssomChange::Declarations, CssomChange::Provenance],
            );
            Ok(CssomBlockTicket {
                token: this.token.clone(),
                index,
            })
        })
    }
    /// Removes a selected terminal directly; this is a product operation, not the
    /// broader source removeProperty algorithm (which also handles shorthands).
    pub fn remove_selected_terminal(
        &mut self,
        id: &CssomBlockId,
        name: CssPropertyNameRef<'_>,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.block(id)?;
            let block = this.staged.blocks.get_mut(id).expect("checked block");
            if block.flags.readonly {
                return Err(CssomError::Source(CssomException::NoModificationAllowed));
            }
            let CssomBlockData::Properties {
                domain, selected, ..
            } = &mut block.data
            else {
                return Err(CssomError::WrongKind);
            };
            let old = match selected {
                CssomProjection::Available(v) => v,
                CssomProjection::Unavailable(e) => return Err(CssomError::Declaration(e.clone())),
            };
            let entries = old
                .entries()
                .iter()
                .filter(|v| v.property_name() != name)
                .cloned()
                .collect::<Vec<_>>();
            if entries.len() == old.entries().len() {
                return Ok(false);
            }
            let value = match domain {
                CssomPropertyDomain::Ordinary => {
                    CssSpecifiedDeclarationBlock::try_from_entries_with_limits(
                        &entries,
                        this.limits.css,
                    )
                }
                CssomPropertyDomain::Keyframe => {
                    CssSpecifiedDeclarationBlock::try_from_keyframe_entries_with_limits(
                        &entries,
                        this.limits.css,
                    )
                }
                CssomPropertyDomain::Margin => {
                    CssSpecifiedDeclarationBlock::try_from_margin_entries_with_limits(
                        &entries,
                        this.limits.css,
                    )
                }
            }
            .map_err(CssomError::Declaration)?;
            *selected = CssomProjection::Available(value);
            this.affect(
                CssomObjectId::Block(id.clone()),
                &[CssomChange::Declarations],
            );
            Ok(true)
        })
    }
    pub fn set_disabled(&mut self, id: &CssomSheetId, disabled: bool) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.sheet(id)?;
            let sheet = this.staged.sheets.get_mut(id).expect("checked sheet");
            if sheet.inputs.disabled == disabled {
                return Ok(false);
            }
            sheet.inputs.disabled = disabled;
            this.affect(CssomObjectId::Sheet(id.clone()), &[CssomChange::Disabled]);
            Ok(true)
        })
    }
    /// Product typed media replacement uses the existing checked Media Queries carrier.
    pub fn set_media(
        &mut self,
        id: &CssomMediaListId,
        value: CssMediaQueryList,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.media(id)?;
            if this.staged.media[id] == value {
                return Ok(false);
            }
            for sheet in this.staged.sheets.values_mut().filter(|v| &v.media == id) {
                sheet.inputs.media = value.clone();
            }
            this.staged.media.insert(id.clone(), value);
            this.affect(CssomObjectId::MediaList(id.clone()), &[CssomChange::Media]);
            Ok(true)
        })
    }
    /// Recovered replacement follows constructed/modifiable guards before parsing.
    pub fn replace_sheet_text(
        &mut self,
        id: &CssomSheetId,
        source: &str,
    ) -> Result<(), CssomError> {
        self.apply(|this| {
            let snapshot = &this.staged;
            let sheet = snapshot.sheet(id)?;
            if !sheet.inputs.constructed || sheet.inputs.disallow_modification {
                return Err(CssomError::Source(CssomException::NotAllowed));
            }
            let (authored, diagnostics) =
                parse(source, &this.limits, this.staged.context.parser_context())?;
            this.replace(id, authored, diagnostics)
        })
    }
    pub(crate) fn replace(
        &mut self,
        id: &CssomSheetId,
        authored: CssSheet,
        diagnostics: Vec<CssRecoveryDiagnostic>,
    ) -> Result<(), CssomError> {
        let list = self
            .staged
            .sheets
            .get(id)
            .expect("checked sheet")
            .rules
            .clone();
        let old = self.staged.lists.get(&list).expect("sheet list").clone();
        for id in old {
            self.staged
                .rules
                .get_mut(&id)
                .expect("retained root")
                .parent = CssomParent::Detached;
            self.affect(CssomObjectId::Rule(id), &[CssomChange::Ancestry]);
        }
        let mut children = Vec::new();
        for rule in authored.rules() {
            if matches!(rule, CssRule::Import(_)) {
                continue;
            }
            children.push(self.staged.adopt(
                CssomAuthoredRule::Ordinary(rule.clone()),
                CssomParent::Sheet(id.clone()),
                1,
                &self.limits,
            )?);
        }
        self.staged.lists.insert(list, children);
        let sheet = self.staged.sheets.get_mut(id).expect("checked sheet");
        // Imports do not affect namespace bindings; source occurrences are retained.
        sheet.namespaces = CssNamespaceContext::from_sheet(&authored);
        sheet.authored = authored;
        sheet.diagnostics = diagnostics;
        self.affect(
            CssomObjectId::Sheet(id.clone()),
            &[
                CssomChange::Replacement,
                CssomChange::RuleMembership,
                CssomChange::Order,
                CssomChange::Ancestry,
                CssomChange::Namespaces,
                CssomChange::Provenance,
            ],
        );
        self.full_recompute = true;
        Ok(())
    }
    /// Sheet security/modification guards precede the shared index/namespace guards.
    pub fn delete_sheet_rule(
        &mut self,
        id: &CssomSheetId,
        index: usize,
    ) -> Result<CssomRuleId, CssomError> {
        self.apply(|this| {
            let snapshot = &this.staged;
            let sheet = snapshot.sheet(id)?;
            if !sheet.inputs.origin_clean {
                return Err(CssomError::Source(CssomException::Security));
            }
            if sheet.inputs.disallow_modification {
                return Err(CssomError::Source(CssomException::NotAllowed));
            }
            {
                let list = sheet.rules.clone();
                this.delete(&list, index)
            }
        })
    }
    /// Operates on a staged creation without leaking its tentative live identity.
    pub fn delete_created_sheet_rule(
        &mut self,
        ticket: &CssomSheetTicket,
        index: usize,
    ) -> Result<CssomRuleTicket, CssomError> {
        self.apply(|this| {
            if !Arc::ptr_eq(&this.token, &ticket.token) {
                return Err(CssomError::ForeignTicket);
            }
            let Some(Created::Sheet(id)) = this.created.get(ticket.index) else {
                return Err(CssomError::ForeignTicket);
            };
            let id = id.clone();
            let sheet = &this.staged.sheets[&id];
            if !sheet.inputs.origin_clean {
                return Err(CssomError::Source(CssomException::Security));
            }
            if sheet.inputs.disallow_modification {
                return Err(CssomError::Source(CssomException::NotAllowed));
            }
            let list = sheet.rules.clone();
            let root = this.delete(&list, index)?;
            let index = this.created.len();
            this.created.push(Created::Rule(root));
            Ok(CssomRuleTicket {
                token: this.token.clone(),
                index,
            })
        })
    }
    /// CSSGroupingRule removal is index-first after kind admission; descendants retain ancestry.
    pub fn delete_group_rule(
        &mut self,
        id: &CssomRuleId,
        index: usize,
    ) -> Result<CssomRuleId, CssomError> {
        self.apply(|this| {
            let list = match &this.staged.rule(id)?.data {
                CssomRuleData::Style { children, .. }
                | CssomRuleData::Group { children, .. }
                | CssomRuleData::Page { children, .. } => children.clone(),
                _ => return Err(CssomError::WrongKind),
            };
            this.delete(&list, index)
        })
    }
    pub(crate) fn delete(
        &mut self,
        list: &CssomRuleListId,
        index: usize,
    ) -> Result<CssomRuleId, CssomError> {
        let members = self.staged.lists.get(list).expect("owned list");
        let root = members
            .get(index)
            .ok_or(CssomError::Source(CssomException::IndexSize))?
            .clone();
        if matches!(
            &self.staged.rules[&root].authored,
            CssomAuthoredRule::Ordinary(CssRule::Namespace(_))
        ) && members.iter().any(|id| {
            !matches!(
                &self.staged.rules[id].authored,
                CssomAuthoredRule::Ordinary(CssRule::Namespace(_) | CssRule::Import(_))
            )
        }) {
            return Err(CssomError::Source(CssomException::InvalidState));
        }
        self.staged
            .lists
            .get_mut(list)
            .expect("owned list")
            .remove(index);
        let parent = self.staged.rules[&root].parent.clone();
        self.staged
            .rules
            .get_mut(&root)
            .expect("retained root")
            .parent = CssomParent::Detached;
        if let CssomParent::Sheet(sheet) = parent {
            self.refresh_namespaces(&sheet);
        }
        self.affect(
            CssomObjectId::RuleList(list.clone()),
            &[CssomChange::RuleMembership, CssomChange::Order],
        );
        self.affect(
            CssomObjectId::Rule(root.clone()),
            &[CssomChange::Ancestry, CssomChange::Namespaces],
        );
        Ok(root)
    }
    pub(crate) fn refresh_namespaces(&mut self, id: &CssomSheetId) {
        let list = &self.staged.sheets[id].rules;
        let bindings = self.staged.lists[list]
            .iter()
            .filter_map(|id| match &self.staged.rules[id].authored {
                CssomAuthoredRule::Ordinary(CssRule::Namespace(v)) => {
                    Some((v.prefix().cloned(), v.name().clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        self.staged.sheets.get_mut(id).expect("sheet").namespaces =
            CssNamespaceContext::from_bindings(bindings);
    }
    pub fn update_context(&mut self, context: CssomContext) -> Result<bool, CssomError> {
        self.apply(|this| {
            if this.staged.context == context {
                return Ok(false);
            }
            this.staged.context = context;
            this.categories.extend([
                CssomChange::Origin,
                CssomChange::Layer,
                CssomChange::Base,
                CssomChange::Parser,
                CssomChange::Support,
                CssomChange::Owner,
                CssomChange::ImportLinks,
            ]);
            this.full_recompute = true;
            Ok(true)
        })
    }
    /// Source CSSRule.cssText setter is a standards-defined no-op.
    pub fn set_rule_css_text(&mut self, id: &CssomRuleId, _text: &str) -> Result<(), CssomError> {
        self.apply(|this| this.staged.rule(id).map(|_| ()))
    }
    /// CSSOMString setter policy; deliberately accepts empty/raw custom names.
    pub fn set_counter_name(&mut self, id: &CssomRuleId, name: &str) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.rule(id)?;
            let rule = this.staged.rules.get_mut(id).expect("rule");
            let CssomRuleData::CounterStyle { name: old, .. } = &mut rule.data else {
                return Err(CssomError::WrongKind);
            };
            if [
                "none",
                "decimal",
                "disc",
                "circle",
                "square",
                "disclosure-open",
                "disclosure-closed",
            ]
            .iter()
            .any(|v| name.eq_ignore_ascii_case(v))
            {
                return Ok(false);
            }
            // Existing CSS owner normalizes predefined names. Invalid authored
            // identifiers are still legitimate raw live CSSOMString contents.
            let name = CssCounterStyleName::try_new(name)
                .map_or_else(|| name.to_owned(), |v| v.as_str().to_owned());
            if old == &name {
                return Ok(false);
            }
            *old = name;
            this.affect(CssomObjectId::Rule(id.clone()), &[CssomChange::Descriptors]);
            Ok(true)
        })
    }
    pub fn set_feature(
        &mut self,
        id: &CssomFeatureMapId,
        key: &str,
        value: &[u32],
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.feature_map(id)?;
            let map = this.staged.maps.get_mut(id).expect("map");
            let max = match map.kind {
                CssFontFeatureValueKind::HistoricalForms | CssFontFeatureValueKind::Styleset => {
                    usize::MAX
                }
                CssFontFeatureValueKind::CharacterVariant => 2,
                _ => 1,
            };
            if value.len() > max {
                return Err(CssomError::Source(CssomException::InvalidAccess));
            }
            if let Some(old) = map.entries.iter_mut().find(|v| v.key == key) {
                if matches!(&old.value,CssomFeatureValue::Unsigned(v) if v==value) {
                    return Ok(false);
                }
                old.value = CssomFeatureValue::Unsigned(value.to_vec());
            } else {
                map.entries.push(CssomFeatureEntry {
                    key: key.to_owned(),
                    value: CssomFeatureValue::Unsigned(value.to_vec()),
                });
            }
            this.affect(
                CssomObjectId::FeatureMap(id.clone()),
                &[CssomChange::FeatureMaps],
            );
            Ok(true)
        })
    }
    pub fn delete_feature(
        &mut self,
        id: &CssomFeatureMapId,
        key: &str,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            this.staged.feature_map(id)?;
            let map = this.staged.maps.get_mut(id).expect("map");
            let Some(index) = map.entries.iter().position(|v| v.key == key) else {
                return Ok(false);
            };
            map.entries.remove(index);
            this.affect(
                CssomObjectId::FeatureMap(id.clone()),
                &[CssomChange::FeatureMaps],
            );
            Ok(true)
        })
    }
    /// Explicit host association; importing rule media survives a null child.
    pub fn associate_import(
        &mut self,
        rule: &CssomRuleId,
        sheet: &CssomSheetId,
        input: CssomInputVersion,
        resolved_location: Option<String>,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            let snapshot = &this.staged;
            snapshot.sheet(sheet)?;
            let CssomRuleData::Import { child_sheet, .. } = snapshot.rule(rule)?.data() else {
                return Err(CssomError::WrongKind);
            };
            if child_sheet.as_ref().is_some_and(|id| id != sheet)
                || snapshot
                    .sheet(sheet)?
                    .owner_rule
                    .as_ref()
                    .is_some_and(|id| id != rule)
            {
                return Err(CssomError::Source(CssomException::InvalidState));
            }
            // Reject direct or transitive sheet import cycles; no live recursive read.
            let mut cursor = snapshot.parent_style_sheet(rule)?.cloned();
            while let Some(id) = cursor {
                if &id == sheet {
                    return Err(CssomError::Source(CssomException::HierarchyRequest));
                }
                cursor = match snapshot.sheet(&id)?.owner_rule() {
                    Some(id) => snapshot.parent_style_sheet(id)?.cloned(),
                    None => None,
                };
            }
            let CssomRuleData::Import {
                child_sheet,
                resolved_location: old,
                input: old_input,
                ..
            } = &mut this.staged.rules.get_mut(rule).expect("rule").data
            else {
                unreachable!()
            };
            if child_sheet.as_ref() == Some(sheet)
                && old == &resolved_location
                && old_input.as_ref() == Some(&input)
            {
                return Ok(false);
            }
            *child_sheet = Some(sheet.clone());
            *old = resolved_location;
            *old_input = Some(input);
            this.staged.sheets.get_mut(sheet).expect("sheet").owner_rule = Some(rule.clone());
            this.affect(
                CssomObjectId::Rule(rule.clone()),
                &[
                    CssomChange::ImportLinks,
                    CssomChange::Ancestry,
                    CssomChange::Location,
                ],
            );
            this.affect(
                CssomObjectId::Sheet(sheet.clone()),
                &[CssomChange::ImportLinks],
            );
            this.full_recompute = true;
            Ok(true)
        })
    }
    fn changed(&self) -> bool {
        if !self.declaration_changes.is_empty()
            || self.staged.next_identity != self.base.next_identity
            || self.staged.context != self.base.context
        {
            return true;
        }
        self.affected.iter().any(|id| match id {
            CssomObjectId::Sheet(id) => {
                self.staged.sheets[id].inputs != self.base.sheets[id].inputs
                    || self.staged.sheets[id].owner_rule != self.base.sheets[id].owner_rule
                    || self.staged.sheets[id].authored != self.base.sheets[id].authored
                    || self.staged.sheets[id].diagnostics != self.base.sheets[id].diagnostics
            }
            CssomObjectId::RuleList(id) => self.staged.lists.get(id) != self.base.lists.get(id),
            CssomObjectId::Rule(id) => {
                let a = &self.staged.rules[id];
                let b = &self.base.rules[id];
                a.parent != b.parent
                    || match (&a.data, &b.data) {
                        (
                            CssomRuleData::CounterStyle { name: a, .. },
                            CssomRuleData::CounterStyle { name: b, .. },
                        ) => a != b,
                        (
                            CssomRuleData::Import {
                                child_sheet: a,
                                resolved_location: al,
                                input: ai,
                                ..
                            },
                            CssomRuleData::Import {
                                child_sheet: b,
                                resolved_location: bl,
                                input: bi,
                                ..
                            },
                        ) => a != b || al != bl || ai != bi,
                        _ => false,
                    }
            }
            CssomObjectId::Block(id) => {
                let a = &self.staged.blocks[id];
                let b = &self.base.blocks[id];
                a.flags != b.flags
                    || match (&a.data, &b.data) {
                        (
                            CssomBlockData::Properties {
                                selected: CssomProjection::Available(a),
                                ..
                            },
                            CssomBlockData::Properties {
                                selected: CssomProjection::Available(b),
                                ..
                            },
                        ) => a.entries().len() != b.entries().len(),
                        _ => false,
                    }
            }
            CssomObjectId::FeatureMap(id) => {
                self.staged.maps[id].entries != self.base.maps[id].entries
            }
            CssomObjectId::MediaList(id) => self.staged.media.get(id) != self.base.media.get(id),
            CssomObjectId::SheetList(_) => self.staged.sheet_order != self.base.sheet_order,
        })
    }
}

impl State {
    pub(crate) fn check_limits(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        if self
            .context
            .inputs
            .iter()
            .filter(|input| matches!(input.data, CssomInputData::ParserMode(_)))
            .count()
            > 1
        {
            return Err(CssomError::InvalidInput("ambiguous parser mode input"));
        }
        if self
            .context
            .inputs
            .iter()
            .filter(|input| matches!(input.data, CssomInputData::Support(_)))
            .count()
            > 1
        {
            return Err(CssomError::InvalidInput("ambiguous declaration support"));
        }
        for (i, input) in self.context.inputs.iter().enumerate() {
            if input.version.role != input.data.role() {
                return Err(CssomError::InvalidInput("supplied input role mismatch"));
            }
            if self.context.inputs[..i].iter().any(|v| {
                v.version.role == input.version.role && v.version.identity == input.version.identity
            }) {
                return Err(CssomError::InvalidInput(
                    "duplicate supplied input identity",
                ));
            }
        }
        let mut linked = self
            .context
            .linked
            .iter()
            .map(|v| (&v.snapshot, 1usize))
            .collect::<Vec<_>>();
        let mut visited = 0usize;
        while let Some((snapshot, depth)) = linked.pop() {
            if depth > limits.max_depth {
                return Err(CssomError::Limit {
                    resource: "linked input depth",
                    maximum: limits.max_depth,
                });
            }
            visited = visited.checked_add(1).ok_or(CssomError::Limit {
                resource: "linked input visits",
                maximum: limits.max_entries,
            })?;
            if visited > limits.max_entries {
                return Err(CssomError::Limit {
                    resource: "linked input visits",
                    maximum: limits.max_entries,
                });
            }
            linked.extend(
                snapshot
                    .context()
                    .linked
                    .iter()
                    .map(|v| (&v.snapshot, depth + 1)),
            );
        }
        let objects = 1usize
            .checked_add(self.sheets.len())
            .and_then(|v| v.checked_add(self.rules.len()))
            .and_then(|v| v.checked_add(self.blocks.len()))
            .and_then(|v| v.checked_add(self.lists.len()))
            .and_then(|v| v.checked_add(self.media.len()))
            .and_then(|v| v.checked_add(self.maps.len()));
        limit(objects, limits.max_objects, "objects")?;
        let mut entries = Some(self.sheet_order.len());
        let mut strings = Some(0usize);
        fn add(total: &mut Option<usize>, count: usize) {
            *total = total.and_then(|v| v.checked_add(count));
        }
        fn version(total: &mut Option<usize>, value: &CssomInputVersion) {
            add(total, value.identity.len());
        }
        for input in &self.context.inputs {
            version(&mut strings, &input.version);
            add(&mut entries, 1);
            match &input.data {
                CssomInputData::Document { identity } => add(&mut strings, identity.len()),
                CssomInputData::Origin { origin, profile } => {
                    add(&mut strings, origin.len());
                    add(&mut strings, profile.len());
                }
                CssomInputData::Layer { path, .. } => {
                    add(&mut entries, path.len());
                    for part in path {
                        add(&mut strings, part.len());
                    }
                }
                CssomInputData::Base { url, location } => {
                    for value in [url, location].into_iter().flatten() {
                        add(&mut strings, value.len());
                    }
                }
                CssomInputData::Owner {
                    identity,
                    attribute,
                } => {
                    add(&mut strings, identity.len());
                    if let Some(v) = attribute {
                        add(&mut strings, v.len());
                    }
                }
                CssomInputData::Import { resolved_location } => {
                    if let Some(v) = resolved_location {
                        add(&mut strings, v.len());
                    }
                }
                CssomInputData::ParserMode(_) => {}
                CssomInputData::Support(support) => add(
                    &mut entries,
                    support.properties().len()
                        + support.page_descriptors().len()
                        + support.font_face_descriptors().len()
                        + usize::from(support.svg_glyph_orientation_vertical().is_some()),
                ),
            }
        }
        for linked in &self.context.linked {
            version(&mut strings, &linked.version);
            linked.snapshot.sheet(&linked.sheet)?;
            add(&mut entries, 1);
        }
        for sheet in self.sheets.values() {
            // Retained raw input includes imports filtered from constructed sheets.
            add(&mut entries, sheet.authored.rules().len());
            version(&mut strings, &sheet.inputs.version);
            add(&mut strings, sheet.inputs.title.len());
            for text in [
                &sheet.inputs.base_url,
                &sheet.inputs.location,
                &sheet.inputs.constructor_document,
                &sheet.inputs.owner,
            ]
            .into_iter()
            .flatten()
            {
                add(&mut strings, text.len());
            }
            add(&mut entries, sheet.diagnostics.len());
        }
        for list in self.lists.values() {
            add(&mut entries, list.len());
        }
        for media in self.media.values() {
            add(&mut entries, media.queries().len());
        }
        for block in self.blocks.values() {
            if let Some(CssomProjection::Available(selected)) = &block.selected_font_face {
                add(&mut entries, selected.entries().len());
            }
            if let Some(input) = &block.replacement_input {
                add(&mut strings, input.source.len());
                add(&mut entries, input.diagnostics.len());
            }
            add(&mut entries, block.admission_inputs.len());
            for input in &block.admission_inputs {
                version(&mut strings, input);
            }
            if let Some(owner) = &block.owner {
                version(&mut strings, owner);
            }
            match &block.data {
                CssomBlockData::Properties {
                    authored, selected, ..
                } => {
                    add(
                        &mut entries,
                        match authored {
                            CssomPropertyOccurrences::Ordinary(v)
                            | CssomPropertyOccurrences::RawContents(v) => v.len(),
                            CssomPropertyOccurrences::Keyframe(v) => v.len(),
                            CssomPropertyOccurrences::Margin(v) => v.properties().len(),
                        },
                    );
                    if let CssomProjection::Available(v) = selected {
                        add(&mut entries, v.entries().len());
                    }
                }
                CssomBlockData::Page { authored, selected } => {
                    add(&mut entries, authored.occurrences().len());
                    if let CssomProjection::Available(v) = selected {
                        add(&mut entries, v.entries().len());
                    }
                }
                CssomBlockData::CounterStyle(v) => add(&mut entries, v.occurrences().len()),
                CssomBlockData::FontFace(v) => add(&mut entries, v.occurrences().len()),
            }
        }
        for map in self.maps.values() {
            add(&mut entries, map.authored.len());
            add(&mut entries, map.entries.len());
            for entry in &map.entries {
                add(&mut strings, entry.key.len());
                if let CssomFeatureValue::Unsigned(v) = &entry.value {
                    add(&mut entries, v.len());
                }
            }
        }
        for rule in self.rules.values() {
            match &rule.data {
                CssomRuleData::CounterStyle { name, .. } => add(&mut strings, name.len()),
                CssomRuleData::FontFeatureValues { families, .. } => {
                    add(&mut entries, families.len());
                    for family in families {
                        add(&mut strings, family.as_str().len());
                    }
                }
                CssomRuleData::Import {
                    resolved_location,
                    input,
                    ..
                } => {
                    if let Some(v) = resolved_location {
                        add(&mut strings, v.len());
                    }
                    if let Some(v) = input {
                        version(&mut strings, v);
                    }
                }
                _ => {}
            }
        }
        limit(entries, limits.max_entries, "entries")?;
        limit(
            strings,
            limits.max_string_bytes,
            "host and live string bytes",
        )
    }
}
fn limit(actual: Option<usize>, maximum: usize, resource: &'static str) -> Result<(), CssomError> {
    if actual.is_none_or(|v| v > maximum) {
        Err(CssomError::Limit { resource, maximum })
    } else {
        Ok(())
    }
}
