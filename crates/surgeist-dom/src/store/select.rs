//! Source-ordered Select-family adaptation over the canonical staged tree.
use super::*;
mod value;
use surgeist_html::bounded::*;
use surgeist_html::{KnownHtmlAttribute as Attr, KnownHtmlElement as Tag, *};

/// A DOM-owned temporary bank debits one shared live-capacity ledger. Its
/// lifetime can cross nested synchronous effects; dropping it releases only
/// its own admitted capacity. Producer-owned capture/session banks stay separate.
pub(super) struct ScratchVec<T> {
    values: Vec<T>,
    live: Arc<AtomicUsize>,
    admit: bool,
}
impl<T> std::ops::Deref for ScratchVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl<T> std::ops::DerefMut for ScratchVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
impl<T> ScratchVec<T> {
    /// Transfer the bank directly into a canonical Draft field. No fallible
    /// work may intervene between releasing temporary custody and installing it.
    pub(super) fn into_values(mut self) -> Vec<T> {
        self.live.store(
            self.live.load(Ordering::Relaxed) - self.values.capacity() * size_of::<T>(),
            Ordering::Relaxed,
        );
        std::mem::take(&mut self.values)
    }
}
impl<T> Drop for ScratchVec<T> {
    fn drop(&mut self) {
        self.live.store(
            self.live.load(Ordering::Relaxed) - self.values.capacity() * size_of::<T>(),
            Ordering::Relaxed,
        );
    }
}

impl<T: Clone> ScratchVec<T> {
    pub(super) fn checkpoint(
        &self,
        admission: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let bytes = self
            .len()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        admission.before(bytes)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(self.len())
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        admission.scratch_allocated(actual)?;
        self.live.store(
            self.live
                .load(Ordering::Relaxed)
                .checked_add(actual)
                .ok_or(DomError::RepresentationExhausted)?,
            Ordering::Relaxed,
        );
        let mut copy = Self {
            values,
            live: self.live.clone(),
            admit: self.admit,
        };
        for value in &self.values {
            check_cancel(cancel)?;
            admission.step()?;
            copy.values.push(value.clone());
        }
        Ok(copy)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Task<K> {
    Update { serial: u64, target: K },
    Slots,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Pending {
    Update { serial: Option<u64>, target: Key },
    Slots,
}
pub(super) fn draft_tasks_formed(
    tasks: &[Task<NodeId>],
    initial: Option<&source::initial::Initial>,
    cancel: &AtomicBool,
) -> Result<Vec<Pending>, DomError> {
    let mut out = Vec::new();
    source::initial::reserve(&mut out, tasks.len(), initial, cancel)?;
    for task in tasks {
        source::initial::step(initial, cancel)?;
        out.push(match *task {
            Task::Slots => Pending::Slots,
            Task::Update { serial, target } => Pending::Update {
                serial: Some(serial),
                target: Key::Existing(target.slot),
            },
        });
    }
    Ok(out)
}
pub(super) struct FinishedTasks {
    pub tasks: Vec<Task<NodeId>>,
    pub next: u64,
    pub cancellations: Vec<(u64, NodeId)>,
}
struct SelectContinuation {
    after: SelectAfter,
    acknowledge: bool,
}
struct SetterContinuation {
    edit: crate::select::Edit,
    acknowledge: bool,
    payload: Option<source::checkpoint::Charge>,
}
impl Draft {
    fn queue_select(
        &mut self,
        target: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.select_step(cancel)?;
        let mut count = 1usize;
        for task in &self.native_tasks {
            self.select_step(cancel)?;
            if matches!(task, Pending::Update { .. }) {
                count = count
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        admit(count, limits.select_tasks, DomResource::SelectTasks)?;
        let admitted = self.source_reserve_amount::<Pending>(
            self.native_tasks.len(),
            self.native_tasks.capacity(),
            1,
            cancel,
        )?;
        #[cfg(test)]
        let growth = if self.native_tasks.len() == self.native_tasks.capacity() {
            source::formation::mutation_observation::before(
                self,
                source::formation::mutation_observation::Site::SelectTasks,
                cancel,
            )?
        } else {
            None
        };
        source::reserve_bank(&mut self.native_tasks, 1, admitted)?;
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            self.native_tasks.capacity() * size_of::<Pending>(),
        );
        self.source_admit(0, cancel)?;
        self.headroom_select_issue(cancel)?;
        self.native_tasks.push(Pending::Update {
            serial: None,
            target,
        });
        Ok(())
    }
    pub(super) fn queue_slot_marker(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        if !self.notification_queued {
            if let Some(additional) = self.source_reserve_amount::<Pending>(
                self.native_tasks.len(),
                self.native_tasks.capacity(),
                1,
                cancel,
            )? {
                self.native_tasks
                    .try_reserve_exact(additional)
                    .map_err(|_| DomError::AllocationFailed)?;
            } else {
                reserve(&mut self.native_tasks, 1)?;
            }
            self.source_admit(0, cancel)?;
            self.native_tasks.push(Pending::Slots);
            self.notification_queued = true;
        }
        Ok(())
    }
    pub(super) fn cancel_select_tasks(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut kept = 0;
        for i in 0..self.native_tasks.len() {
            self.source_step(cancel)?;
            let task = self.native_tasks[i];
            if matches!(task, Pending::Update { .. }) {
                self.select_step(cancel)?;
            }
            if let Pending::Update { serial, target } = task
                && target == key
            {
                if let Some(id) = self.published_id(key) {
                    let admitted = self.source_reserve_amount::<(Option<u64>, NodeId)>(
                        self.cancelled_selects.len(),
                        self.cancelled_selects.capacity(),
                        1,
                        cancel,
                    )?;
                    source::reserve_bank(&mut self.cancelled_selects, 1, admitted)?;
                    self.source_admit(0, cancel)?;
                    self.cancelled_selects.push((serial, id));
                }
            } else {
                self.native_tasks[kept] = task;
                kept += 1;
            }
        }
        self.native_tasks.truncate(kept);
        Ok(())
    }
    pub(super) fn finish_select_tasks(
        &self,
        dom: &Dom,
        id: &impl Fn(Key) -> NodeId,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<FinishedTasks, DomError> {
        let mut next = dom.state.next_task;
        let mut out = Vec::new();
        let mut cancelled = Vec::new();
        source::formation::reserve(&mut out, self.native_tasks.len(), budget, cancel)?;
        source::formation::reserve(&mut cancelled, self.cancelled_selects.len(), budget, cancel)?;
        let mut serial = |old: Option<u64>| -> Result<u64, DomError> {
            if let Some(id) = old {
                Ok(id)
            } else {
                let id = next;
                next = next
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                Ok(id)
            }
        };
        for task in &self.native_tasks {
            source::formation::step(budget, cancel)?;
            if matches!(task, Pending::Update {target,..} if self.is_private(*target)) {
                continue;
            }
            out.push(match *task {
                Pending::Slots => Task::Slots,
                Pending::Update {
                    serial: old,
                    target,
                } => Task::Update {
                    serial: serial(old)?,
                    target: id(target),
                },
            });
        }
        for entry in &self.cancelled_selects {
            source::formation::step(budget, cancel)?;
            let (old, target) = *entry;
            cancelled.push((serial(old)?, target));
        }
        Ok(FinishedTasks {
            tasks: out,
            next,
            cancellations: cancelled,
        })
    }
}
impl Dom {
    pub fn pending_native_tasks(&self) -> usize {
        self.state.native_tasks.len()
    }
    pub fn native_checkpoint_step(
        &mut self,
        limits: crate::DomNativeCheckpointLimits,
        cancel: &AtomicBool,
    ) -> Result<crate::DomNativeCheckpoint, DomError> {
        if let Some(op) = &self.source_operation {
            match self.state.native_tasks.first() {
                Some(Task::Update { serial, target })
                    if op.allows_select_checkpoint(*serial, *target) => {}
                _ => return Err(DomError::SourceOperationPending),
            }
        }
        check_cancel(cancel)?;
        match self.state.native_tasks.first().copied() {
            None => Ok(crate::DomNativeCheckpoint::Idle),
            Some(Task::Update { serial, target }) => {
                if limits.work == 0 {
                    return Err(DomError::SelectWorkLimit { limit: 0 });
                }
                admit(
                    size_of::<crate::select::Authorization>()
                        + 2 * size_of::<usize>()
                        + size_of::<crate::DomSelectTask>(),
                    limits.scratch,
                    DomResource::SelectScratch,
                )?;
                let auth = if let Some(auth) = self.active_task.upgrade() {
                    if auth.id.serial != serial || auth.target != target || auth.status() != 0 {
                        return Err(DomError::InvalidNativeTask);
                    }
                    auth
                } else {
                    let auth = Arc::new(crate::select::Authorization::new(
                        crate::DomNativeTaskId {
                            owner: self.owner,
                            serial,
                        },
                        target,
                        self.snapshot(),
                    ));
                    self.active_task = Arc::downgrade(&auth);
                    auth
                };
                Ok(crate::DomNativeCheckpoint::Select(crate::DomSelectTask {
                    source: auth.source.clone(),
                    authorization: auth,
                }))
            }
            Some(Task::Slots) => {
                let (prepared, targets) = self.prepare_slot_checkpoint(limits.slots, cancel)?;
                let commit = self.publish(prepared, cancel)?;
                Ok(crate::DomNativeCheckpoint::Slots(
                    crate::DomSlotNotifications::new(self.snapshot(), targets, commit.changes),
                ))
            }
        }
    }
    pub fn complete_select_task(
        &mut self,
        task: &crate::DomSelectTask,
        limits: crate::DomNativeCheckpointLimits,
        cancel: &AtomicBool,
    ) -> Result<crate::DomSelectTaskCompletion, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        let auth = &task.authorization;
        if auth.id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if auth.status() == 2 {
            // Retirement resolves the canonical obligation. Reporting its old
            // intention still requires caller admission and exact-generation
            // revalidation, without touching any replacement slot occupant.
            if limits.work == 0 {
                return Err(DomError::SelectWorkLimit { limit: 0 });
            }
            admit(
                size_of::<crate::DomSelectTaskCompletion>(),
                limits.scratch,
                DomResource::SelectScratch,
            )?;
            if self.snapshot().node(auth.target).is_ok() {
                return Err(DomError::InvalidNativeTask);
            }
            check_cancel(cancel)?;
            auth.consume();
            return Ok(crate::DomSelectTaskCompletion {
                outcome: crate::DomSelectTaskOutcome::SkippedTargetDestroyed,
                changes: None,
            });
        }
        if auth.status() != 0
            || !self
                .active_task
                .upgrade()
                .is_some_and(|a| Arc::ptr_eq(&a, auth))
            || !matches!(self.state.native_tasks.first(),Some(Task::Update{serial,target}) if *serial==auth.id.serial && *target==auth.target)
        {
            return Err(DomError::InvalidNativeTask);
        }
        let mut draft = Draft::new(self, DomBatch::new().owner, cancel)?;
        draft.select_scratch_limit = limits.scratch;
        draft.select_work_limit = limits.work;
        let key = draft.resolve(auth.target.into())?;
        draft.schedule_selectedcontent(key, self.limits, cancel)?;
        // The head stays owned through every fallible effect and output allocation.
        draft.native_tasks.remove(0);
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        auth.consume();
        self.active_task = std::sync::Weak::new();
        Ok(crate::DomSelectTaskCompletion {
            outcome: crate::DomSelectTaskOutcome::Updated,
            changes: Some(commit.changes),
        })
    }
}
impl Draft {
    pub(super) fn element(&self, key: Key) -> Result<&DomElement, DomError> {
        match &self.node(key)?.kind {
            DomNodeKind::Element(e) => Ok(e),
            _ => Err(DomError::WrongNodeKind),
        }
    }
    /// The original family owner admits its actual shared immutable payload
    /// before copy-on-write. Custody transfers directly into the Draft family;
    /// subsequent mutable access is unique and performs no second allocation.
    pub(super) fn prepare_family_mutation(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.source_limits.is_none() {
            return Ok(());
        }
        self.source_step(cancel)?;
        let payload = self
            .element(key)?
            .shared_family_payload(|| self.source_step(cancel))?;
        let Some(payload) = payload else {
            return Ok(());
        };
        #[cfg(test)]
        let before = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::Family,
            cancel,
        )?;
        let formation = source::formation::Formation::new(self, cancel)?.expect("source context");
        formation.retain_payload(payload, cancel)?;
        self.element_mut(key).make_family_unique();
        #[cfg(test)]
        source::formation::mutation_observation::grew(before, payload.bytes);
        drop(formation);
        Ok(())
    }
    pub(super) fn element_mut(&mut self, key: Key) -> &mut DomElement {
        match &mut self.node_mut(key).kind {
            DomNodeKind::Element(e) => e,
            _ => unreachable!("checked Element"),
        }
    }
    fn tag(&self, key: Key) -> Result<Option<Tag>, DomError> {
        Ok(match &self.node(key)?.kind {
            DomNodeKind::Element(e) => Tag::from_name(e.name()),
            _ => None,
        })
    }
    fn select_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let next = self
            .select_work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.select_work_limit {
            return Err(DomError::SelectWorkLimit {
                limit: self.select_work_limit,
            });
        }
        // A source fast path may allocate nothing itself while earlier generic
        // effect banks remain live. Admit that entire current lifetime envelope
        // at the actual Select boundary too.
        self.select_scratch(0)?;
        self.select_work.set(next);
        Ok(())
    }
    pub(super) fn effect_step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if self.effect_scoped {
            self.select_step(cancel)
        } else {
            self.source_step(cancel)
        }
    }
    pub(super) fn select_scratch(&self, bytes: usize) -> Result<(), DomError> {
        let total = self
            .effect_bytes
            .checked_add(self.select_temporary_bytes.load(Ordering::Relaxed))
            .and_then(|n| n.checked_add(bytes))
            .ok_or(DomError::RepresentationExhausted)?;
        #[cfg(test)]
        super::tree::scratch_observation::observe(total, self.select_scratch_limit);
        admit(total, self.select_scratch_limit, DomResource::SelectScratch)
    }
    fn select_vec<T>(&self) -> ScratchVec<T> {
        ScratchVec {
            values: Vec::new(),
            live: self.select_temporary_bytes.clone(),
            admit: true,
        }
    }
    pub(super) fn effect_vec<T>(&self) -> ScratchVec<T> {
        ScratchVec {
            values: Vec::new(),
            live: self.select_temporary_bytes.clone(),
            admit: self.effect_scoped,
        }
    }
    pub(super) fn select_reserve<T>(
        &self,
        v: &mut ScratchVec<T>,
        n: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Register every actual effect bank for its full lifetime, including
        // an ordinary insertion whose later postconnection first reaches Select.
        // Admission is enforced only at a reached Select boundary.
        let enforce = v.admit || self.effect_scoped;
        let required = v
            .len()
            .checked_add(n)
            .ok_or(DomError::RepresentationExhausted)?;
        if required > v.capacity() {
            let old = v
                .capacity()
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?;
            let bytes = required
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?;
            // New allocation overlaps the old allocation until reserve returns.
            self.source_admit(bytes, cancel)?;
            if enforce {
                self.select_scratch(bytes)?;
            }
            v.values
                .try_reserve_exact(n)
                .map_err(|_| DomError::AllocationFailed)?;
            let actual = v
                .capacity()
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?;
            self.select_temporary_bytes.store(
                self.select_temporary_bytes
                    .load(Ordering::Relaxed)
                    .checked_add(actual - old)
                    .ok_or(DomError::RepresentationExhausted)?,
                Ordering::Relaxed,
            );
        }
        self.source_admit(0, cancel)?;
        if enforce {
            self.select_scratch(0)
        } else {
            Ok(())
        }
    }
    fn has(&self, key: Key, attr: Attr, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.select_step(cancel)?;
        Ok(self.control_attribute(key, attr, cancel)?.is_some())
    }
    fn selected(&self, key: Key) -> Result<bool, DomError> {
        Ok(self
            .element(key)?
            .option_state()
            .ok_or(DomError::OptionInapplicable)?
            .flags
            .selectedness())
    }
    fn write_selected(&mut self, key: Key, selected: bool) {
        self.element_mut(key)
            .option_state_mut()
            .expect("actual Option")
            .flags
            .set_selectedness_for_select(selected);
    }
    fn multiple(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.has(key, Attr::Multiple, cancel)
    }
    fn options(&self, key: Key, cancel: &AtomicBool) -> Result<ScratchVec<Key>, DomError> {
        let mut result = self.select_vec();
        let mut pending = self.select_vec();
        self.select_reserve(&mut pending, self.node(key)?.children.len(), cancel)?;
        for child in self.node(key)?.children.iter().rev() {
            self.source_step(cancel)?;
            pending.push((*child, false));
        }
        while !pending.is_empty() {
            self.source_step(cancel)?;
            let (key, group) = pending.pop().expect("admitted pending option");
            self.select_step(cancel)?;
            let tag = self.tag(key)?;
            match select_option_rule(tag, group) {
                HtmlSelectOptionRule::IncludeAndPrune => {
                    self.select_reserve(&mut result, 1, cancel)?;
                    result.push(key);
                }
                HtmlSelectOptionRule::Prune => (),
                HtmlSelectOptionRule::Descend => {
                    self.select_reserve(&mut pending, self.node(key)?.children.len(), cancel)?;
                    for child in self.node(key)?.children.iter().rev() {
                        self.source_step(cancel)?;
                        pending.push((*child, group || tag == Some(Tag::Optgroup)));
                    }
                }
            }
        }
        Ok(result)
    }
    fn option_disabled(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        if self.has(key, Attr::Disabled, cancel)? {
            return Ok(true);
        }
        let mut next = self.node(key)?.parent;
        while let Some(key) = next {
            self.select_step(cancel)?;
            match option_disabled_ancestor(self.tag(key)?) {
                HtmlOptionDisabledAncestor::Enabled => return Ok(false),
                HtmlOptionDisabledAncestor::ReadOptgroupDisabled => {
                    return self.has(key, Attr::Disabled, cancel);
                }
                HtmlOptionDisabledAncestor::Continue => (),
            }
            next = self.node(key)?.parent;
        }
        Ok(false)
    }
    fn display_size(&self, key: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.select_step(cancel)?;
        if self.element(key)?.select_state().is_none() {
            return Ok(false);
        }
        let raw = self.control_attribute(key, Attr::Size, cancel)?;
        let multiple = self.control_attribute(key, Attr::Multiple, cancel)?;
        // Preserve the existing independent Select raw-input tariff. The
        // enclosing Source owner additionally admits the original borrowed
        // grammar/projection reads; no owning scalar report is materialized.
        for value in [raw, multiple].into_iter().flatten() {
            for _ in value.code_units() {
                self.select_step(cancel)?;
            }
        }
        let recovered = if let Some(raw) = raw {
            let mut scanner = IntegerPrefixScanner::new(true);
            for unit in raw.code_units() {
                self.source_step(cancel)?;
                if !scanner.feed(*unit) {
                    break;
                }
            }
            self.source_step(cancel)?;
            if let Ok(prefix) = scanner.finish() {
                // The producer contract bounds this borrowed sign/digit check
                // by two visits per actual prefix unit plus fixed range work.
                let visits = prefix
                    .end()
                    .checked_sub(prefix.start())
                    .and_then(|n| n.checked_mul(2))
                    .and_then(|n| n.checked_add(2))
                    .ok_or(DomError::RepresentationExhausted)?;
                for _ in 0..visits {
                    self.source_step(cancel)?;
                }
                prefix.try_nonnegative_digits(raw.code_units()).ok()
            } else {
                None
            }
        } else {
            None
        };
        if let Some(digits) = recovered {
            let mut nonzero = false;
            let mut one = false;
            for unit in digits {
                self.source_step(cancel)?;
                if nonzero {
                    one = false;
                } else if *unit != b'0' as u16 {
                    nonzero = true;
                    one = *unit == b'1' as u16;
                }
            }
            Ok(one)
        } else {
            self.source_step(cancel)?;
            Ok(select_display_size_default(multiple.is_some()) == 1)
        }
    }
    pub(super) fn select_operation(
        &mut self,
        key: Key,
        operation: HtmlSelectOperation,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.push_effect(
            Effect::SelectStart {
                key,
                operation,
                after: SelectAfter::None,
            },
            true,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    fn select_cursor_step(
        &mut self,
        key: Key,
        options: ScratchVec<Key>,
        mut cursor: HtmlSelectCursor,
        continuation: SelectContinuation,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let SelectContinuation { after, acknowledge } = continuation;
        if acknowledge {
            cursor
                .reply(HtmlSelectReply::Applied)
                .map_err(|_| DomError::SelectProtocol)?;
        }
        let reply = match cursor.action() {
            HtmlSelectAction::ResetSelectFlags => {
                self.prepare_family_mutation(key, cancel)?;
                self.element_mut(key)
                    .select_state_mut()
                    .expect("Select")
                    .flags
                    .reset();
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::ResetOption { ordinal } => {
                let option = options[ordinal];
                let selected = self.has(option, Attr::Selected, cancel)?;
                self.element_mut(option)
                    .option_state_mut()
                    .expect("Option")
                    .flags
                    .reset(selected);
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::ReadSelectedness { ordinal } => {
                HtmlSelectReply::Selectedness(self.selected(options[ordinal])?)
            }
            HtmlSelectAction::WriteSelectedness { ordinal, selected } => {
                self.write_selected(options[ordinal], selected);
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::ReadDisabled { ordinal } => {
                HtmlSelectReply::Disabled(self.option_disabled(options[ordinal], cancel)?)
            }
            HtmlSelectAction::ReadDisplaySize => {
                HtmlSelectReply::DisplaySizeIsOne(self.display_size(key, cancel)?)
            }
            HtmlSelectAction::UpdateSelectedcontent => {
                self.push_effect(
                    Effect::SelectCursor {
                        key,
                        options,
                        cursor,
                        after,
                        acknowledge: true,
                    },
                    true,
                    cancel,
                )?;
                return self.schedule_selectedcontent(key, limits, cancel);
            }
            HtmlSelectAction::Complete {
                normalization_changed,
            } => {
                if let SelectAfter::OptionCache { option, removing } = after
                    && removing
                    && (self.selected(option)? || normalization_changed)
                {
                    self.queue_select(key, limits, cancel)?;
                }
                return Ok(());
            }
        };
        cursor.reply(reply).map_err(|_| DomError::SelectProtocol)?;
        self.push_effect(
            Effect::SelectCursor {
                key,
                options,
                cursor,
                after,
                acknowledge: false,
            },
            true,
            cancel,
        )
    }
    fn nearest_option_select(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        let mut cursor = HtmlOptionSelectAncestors::default();
        let mut next = self.node(key)?.parent;
        while let Some(key) = next {
            self.select_step(cancel)?;
            match cursor.observe(self.tag(key)?) {
                HtmlOptionSelectAncestorAction::NearestSelect => return Ok(Some(key)),
                HtmlOptionSelectAncestorAction::NoSelect => return Ok(None),
                HtmlOptionSelectAncestorAction::Continue => next = self.node(key)?.parent,
            }
        }
        cursor.finish_complete();
        Ok(None)
    }
    fn option_cache(
        &mut self,
        key: Key,
        removing: bool,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self.node(key)?.option_select;
        let new = self.nearest_option_select(key, cancel)?;
        self.node_mut(key).option_select = new;
        if old == new {
            return Ok(());
        }
        // LIFO continuations start the new Select only after the old Select's
        // complete refresh, including every nested clone/removal hook.
        for select in [new, old].into_iter().flatten() {
            self.push_effect(
                Effect::SelectStart {
                    key: select,
                    operation: HtmlSelectOperation::Normalize,
                    after: SelectAfter::OptionCache {
                        option: key,
                        removing,
                    },
                },
                true,
                cancel,
            )?;
        }
        self.run_effects(limits, cancel)?;
        Ok(())
    }
    fn selectedcontent_ancestors(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        let mut cursor = HtmlSelectedcontentAncestors::default();
        let mut nearest = None;
        let mut next = self.node(key)?.parent;
        while let Some(k) = next {
            self.select_step(cancel)?;
            match cursor.observe(self.tag(k)?) {
                HtmlSelectedcontentAncestorAction::RememberNearestSelect => nearest = Some(k),
                HtmlSelectedcontentAncestorAction::StopDisabled => break,
                HtmlSelectedcontentAncestorAction::Continue => (),
            }
            next = self.node(k)?.parent;
        }
        *self
            .element_mut(key)
            .selectedcontent_disabled_mut()
            .expect("Selectedcontent") = cursor.disabled();
        Ok(nearest)
    }
    pub(super) fn select_insertion(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        match self.tag(key)? {
            Some(Tag::Option) => self.option_cache(key, false, limits, cancel)?,
            Some(Tag::Selectedcontent) => {
                self.selectedcontent_ancestors(key, cancel)?;
            }
            _ => (),
        }
        Ok(())
    }
    pub(super) fn select_removal(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.tag(key)? == Some(Tag::Option) {
            self.option_cache(key, true, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn select_post_connection(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        match self.tag(key)? {
            Some(Tag::Option) if self.selected(key)? => {
                if let Some(select) = self.node(key)?.option_select {
                    self.schedule_selectedcontent(select, limits, cancel)?;
                }
            }
            Some(Tag::Selectedcontent) => {
                if let Some(select) = self.selectedcontent_ancestors(key, cancel)?
                    && !self.multiple(select, cancel)?
                    && !self.element(key)?.selectedcontent_disabled().expect("SC")
                {
                    self.push_effect(
                        Effect::Target {
                            select,
                            target: key,
                        },
                        true,
                        cancel,
                    )?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) enum SelectAfter {
    None,
    OptionCache { option: Key, removing: bool },
}
pub(super) enum Effect {
    AutofocusInsertion(super::autofocus::Frame),
    DialogHook(dialog::HookFrame),
    AttributeSuffix(attributes::AttributeSuffix),
    Style(style::Frame),
    XmlDocument(super::document::XmlDocumentFrame),
    ChildrenChanged {
        parent: Key,
        stage: u8,
    },
    Output(output::Frame),
    SelectStart {
        key: Key,
        operation: HtmlSelectOperation,
        after: SelectAfter,
    },
    SelectCursor {
        key: Key,
        options: ScratchVec<Key>,
        cursor: HtmlSelectCursor,
        after: SelectAfter,
        acknowledge: bool,
    },
    Setter {
        key: Key,
        options: ScratchVec<Key>,
        cursor: HtmlSelectSetterCursor,
        edit: crate::select::Edit,
        acknowledge: bool,
    },
    Clear {
        target: Key,
    },
    Tree(tree::Frame),
    Clone(import::Frame),
    Post {
        list: ScratchVec<Key>,
        index: usize,
        stage: u8,
    },
    Update {
        select: Key,
        targets: ScratchVec<Key>,
        index: usize,
    },
    Target {
        select: Key,
        target: Key,
    },
    Children {
        option: Key,
        fragment: Key,
        target: Key,
        index: usize,
    },
    Replace {
        fragment: Key,
        target: Key,
    },
    Cleanup(Key),
}
pub(super) struct EffectFrame {
    pub(super) effect: Effect,
    pub(super) scoped: bool,
    payload: Option<source::checkpoint::Charge>,
}
impl Effect {
    fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        step()?;
        match self {
            Self::AttributeSuffix(frame) => frame.retained_payload(step),
            Self::XmlDocument(frame) => frame.retained_payload(step),
            Self::Output(frame) => frame.retained_payload(step),
            Self::Setter {
                edit: crate::select::Edit::Value(value),
                ..
            } => {
                let mut payload = source::storage::Payload::default();
                payload.string(value)?;
                Ok(payload)
            }
            Self::Clone(frame) => frame.retained_payload(step),
            _ => Ok(source::storage::Payload::default()),
        }
    }
}
impl EffectFrame {
    pub(super) fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        check_cancel(cancel)?;
        a.step()?;
        let retained = self.effect.retained_payload(|| a.step())?;
        let payload = a.effect_payload(retained, cancel)?;
        let effect = match &self.effect {
            Effect::AutofocusInsertion(frame) => Effect::AutofocusInsertion(*frame),
            Effect::DialogHook(frame) => Effect::DialogHook(*frame),
            Effect::AttributeSuffix(frame) => Effect::AttributeSuffix(frame.clone()),
            Effect::Style(frame) => Effect::Style(*frame),
            Effect::XmlDocument(frame) => Effect::XmlDocument(frame.clone()),
            Effect::ChildrenChanged { parent, stage } => Effect::ChildrenChanged {
                parent: *parent,
                stage: *stage,
            },
            Effect::Output(frame) => Effect::Output(frame.clone()),
            Effect::SelectStart {
                key,
                operation,
                after,
            } => Effect::SelectStart {
                key: *key,
                operation: *operation,
                after: *after,
            },
            Effect::SelectCursor {
                key,
                options,
                cursor,
                after,
                acknowledge,
            } => Effect::SelectCursor {
                key: *key,
                options: options.checkpoint(a, cancel)?,
                cursor: *cursor,
                after: *after,
                acknowledge: *acknowledge,
            },
            Effect::Setter {
                key,
                options,
                cursor,
                edit,
                acknowledge,
            } => Effect::Setter {
                key: *key,
                options: options.checkpoint(a, cancel)?,
                cursor: *cursor,
                edit: edit.clone(),
                acknowledge: *acknowledge,
            },
            Effect::Clear { target } => Effect::Clear { target: *target },
            Effect::Tree(frame) => Effect::Tree(frame.checkpoint(a, cancel)?),
            Effect::Clone(frame) => Effect::Clone(frame.checkpoint()),
            Effect::Post { list, index, stage } => Effect::Post {
                list: list.checkpoint(a, cancel)?,
                index: *index,
                stage: *stage,
            },
            Effect::Update {
                select,
                targets,
                index,
            } => Effect::Update {
                select: *select,
                targets: targets.checkpoint(a, cancel)?,
                index: *index,
            },
            Effect::Target { select, target } => Effect::Target {
                select: *select,
                target: *target,
            },
            Effect::Children {
                option,
                fragment,
                target,
                index,
            } => Effect::Children {
                option: *option,
                fragment: *fragment,
                target: *target,
                index: *index,
            },
            Effect::Replace { fragment, target } => Effect::Replace {
                fragment: *fragment,
                target: *target,
            },
            Effect::Cleanup(fragment) => Effect::Cleanup(*fragment),
        };
        Ok(Self {
            effect,
            scoped: self.scoped,
            payload,
        })
    }
}

impl Draft {
    pub(super) fn push_effect(
        &mut self,
        effect: Effect,
        scoped: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Payload-bearing frames enter through push_effect_retained only: their
        // graph occurrence is admitted before it is materialized or cloned.
        debug_assert!(!matches!(
            &effect,
            Effect::Output(_)
                | Effect::Setter { .. }
                | Effect::Clone(_)
                | Effect::XmlDocument(_)
                | Effect::AttributeSuffix(_)
        ));
        self.push_effect_retained(effect, scoped, None, cancel)
    }
    pub(super) fn push_effect_retained(
        &mut self,
        effect: Effect,
        scoped: bool,
        payload: Option<source::checkpoint::Charge>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self.effects.capacity() * size_of::<EffectFrame>();
        let required = self
            .effects
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let target = if required > self.effects.capacity() {
            let maximum = isize::MAX as usize / size_of::<EffectFrame>();
            let doubled = self
                .effects
                .capacity()
                .checked_mul(2)
                .filter(|n| *n <= maximum);
            doubled.unwrap_or(required).max(required)
        } else {
            self.effects.capacity()
        };
        let new_allocation = if target > self.effects.capacity() {
            target
                .checked_mul(size_of::<EffectFrame>())
                .filter(|n| *n <= isize::MAX as usize)
                .ok_or(DomError::RepresentationExhausted)?
        } else {
            0
        };
        self.source_admit(new_allocation, cancel)?;
        if scoped {
            self.select_scratch(new_allocation)?;
        }
        #[cfg(test)]
        let before_growth = self.effect_bytes + self.select_temporary_bytes.load(Ordering::Relaxed);
        if target > self.effects.capacity() {
            // Own the geometric policy rather than asking amortized reserve to
            // choose a larger bank after preadmission. The full old bank stays
            // charged until this exact requested new bank replaces it.
            self.effects
                .try_reserve_exact(target - self.effects.len())
                .map_err(|_| DomError::AllocationFailed)?;
        }
        #[cfg(test)]
        if scoped && self.effects.capacity() * size_of::<EffectFrame>() > old {
            let actual_new = self.effects.capacity() * size_of::<EffectFrame>();
            super::tree::scratch_observation::observe_growth(
                before_growth,
                actual_new,
                before_growth + actual_new - old,
                self.select_scratch_limit,
            );
        }
        self.effect_bytes = self
            .effect_bytes
            .checked_add((self.effects.capacity() * size_of::<EffectFrame>()).saturating_sub(old))
            .ok_or(DomError::RepresentationExhausted)?;
        if scoped {
            self.select_scratch(0)?;
        }
        self.source_admit(0, cancel)?;
        self.effects.push(EffectFrame {
            effect,
            scoped,
            payload,
        });
        Ok(())
    }
    /// A constructor may append to the original effect stack while its caller
    /// is parked. Drain only that constructor's new frames; the suspended root's
    /// earlier frames remain in their original order for its own continuation.
    pub(in crate::store) fn with_effect_scope<T, E>(
        &mut self,
        action: impl FnOnce(&mut Draft) -> Result<T, E>,
    ) -> Result<T, E> {
        struct Scope<'a> {
            draft: &'a mut Draft,
            floor: usize,
            running: bool,
            scoped: bool,
        }
        impl Drop for Scope<'_> {
            fn drop(&mut self) {
                self.draft.effect_floor = self.floor;
                self.draft.effects_running = self.running;
                self.draft.effect_scoped = self.scoped;
            }
        }
        let floor = self.effect_floor;
        let running = self.effects_running;
        let scoped = self.effect_scoped;
        self.effect_floor = self.effects.len();
        let guard = Scope {
            draft: self,
            floor,
            running,
            scoped,
        };
        action(&mut *guard.draft)
    }
    pub(super) fn run_effects(
        &mut self,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.effects_running {
            return Ok(());
        }
        self.effects_running = true;
        let result = (|| {
            while self.effects.len() > self.effect_floor {
                // Cancellation must not consume an unfinished continuation.
                check_cancel(cancel)?;
                if self.effects.last().expect("nonempty effects").scoped {
                    self.select_step(cancel)?;
                } else {
                    self.source_step(cancel)?;
                }
                let EffectFrame {
                    effect,
                    scoped,
                    payload,
                } = self.effects.pop().expect("admitted nonempty effect");
                self.effect_scoped = scoped;
                match effect {
                    Effect::AutofocusInsertion(frame) => {
                        self.autofocus_effect(frame, limits, cancel)?
                    }
                    Effect::DialogHook(frame) => self.dialog_hook_effect(frame, cancel)?,
                    Effect::AttributeSuffix(frame) => {
                        self.attribute_suffix(frame, payload, limits, cancel)?
                    }
                    Effect::Style(frame) => self.style_effect(frame, cancel)?,
                    Effect::XmlDocument(frame) => {
                        self.xml_document_effect(frame, payload, limits, cancel)?
                    }
                    Effect::ChildrenChanged { parent, stage } => {
                        self.children_changed_effect(parent, stage, limits, cancel)?
                    }
                    Effect::Output(frame) => self.output_effect(frame, payload, limits, cancel)?,
                    Effect::SelectStart {
                        key,
                        operation,
                        after,
                    } => {
                        self.element(key)?
                            .select_state()
                            .ok_or(DomError::SelectInapplicable)?;
                        let multiple = self.multiple(key, cancel)?;
                        let options = if multiple && operation == HtmlSelectOperation::Normalize {
                            self.select_vec()
                        } else {
                            self.options(key, cancel)?
                        };
                        let cursor = HtmlSelectCursor::new(operation, multiple, options.len())
                            .map_err(|_| DomError::SelectProtocol)?;
                        self.push_effect(
                            Effect::SelectCursor {
                                key,
                                options,
                                cursor,
                                after,
                                acknowledge: false,
                            },
                            true,
                            cancel,
                        )?;
                    }
                    Effect::SelectCursor {
                        key,
                        options,
                        cursor,
                        after,
                        acknowledge,
                    } => {
                        self.select_cursor_step(
                            key,
                            options,
                            cursor,
                            SelectContinuation { after, acknowledge },
                            limits,
                            cancel,
                        )?;
                    }
                    Effect::Setter {
                        key,
                        options,
                        cursor,
                        edit,
                        acknowledge,
                    } => {
                        self.select_setter_step(
                            key,
                            options,
                            cursor,
                            SetterContinuation {
                                edit,
                                acknowledge,
                                payload,
                            },
                            limits,
                            cancel,
                        )?;
                    }
                    Effect::Clear { target } => {
                        if let Some(&child) = self.node(target)?.children.first() {
                            self.push_effect(Effect::Clear { target }, true, cancel)?;
                            self.remove(child, limits, cancel)?;
                        }
                    }
                    Effect::Tree(frame) => self.tree_effect(frame, limits, cancel)?,
                    Effect::Clone(frame) => {
                        self.clone_effect(frame, scoped, payload, limits, cancel)?
                    }
                    Effect::Post { list, index, stage } => {
                        if let Some(&key) = list.get(index) {
                            let connected = self.shadow_connected(key, cancel)?;
                            self.push_effect(
                                Effect::Post {
                                    list,
                                    index: if stage == 3 || !connected {
                                        index + 1
                                    } else {
                                        index
                                    },
                                    stage: if stage == 3 || !connected {
                                        0
                                    } else {
                                        stage + 1
                                    },
                                },
                                scoped,
                                cancel,
                            )?;
                            if connected {
                                match stage {
                                    0 => self.navigable_post_connection_node(key, cancel)?,
                                    1 => self.script_post_connection(key, cancel)?,
                                    2 => self.svg_script_post_connection(key, cancel)?,
                                    3 => self.select_post_connection(key, limits, cancel)?,
                                    _ => return Err(DomError::HostedSourceUnavailable),
                                }
                            }
                        }
                    }
                    Effect::Update {
                        select,
                        targets,
                        index,
                    } => {
                        if let Some(&target) = targets.get(index) {
                            self.push_effect(
                                Effect::Update {
                                    select,
                                    targets,
                                    index: index + 1,
                                },
                                true,
                                cancel,
                            )?;
                            self.push_effect(Effect::Target { select, target }, true, cancel)?;
                        }
                    }
                    Effect::Target { select, target } => {
                        // Each target observes the current selection after all preceding effects.
                        let disabled = self
                            .element(target)?
                            .selectedcontent_disabled()
                            .expect("actual selectedcontent");
                        if disabled {
                            continue;
                        }
                        let options = self.options(select, cancel)?;
                        let mut cursor = HtmlSelectFirstSelectedCursor::new(options.len());
                        let first = loop {
                            self.select_step(cancel)?;
                            match cursor.action() {
                                HtmlSelectFirstSelectedAction::ReadSelectedness { ordinal } => {
                                    cursor
                                        .reply(Some(self.selected(options[ordinal])?))
                                        .map_err(|_| DomError::SelectProtocol)?
                                }
                                HtmlSelectFirstSelectedAction::Complete(v) => break v,
                            }
                        };
                        match selectedcontent_update(false, disabled, first) {
                            HtmlSelectedcontentUpdate::Clear => {
                                self.push_effect(Effect::Clear { target }, true, cancel)?;
                            }
                            HtmlSelectedcontentUpdate::CloneOptionChildren { ordinal } => {
                                let option = options[ordinal];
                                let document = self.node(option)?.document;
                                let fragment = self.allocate(
                                    DomNodeKind::DocumentFragment,
                                    document,
                                    limits,
                                    cancel,
                                )?;
                                self.mark_source_private(fragment);
                                self.push_effect(
                                    Effect::Children {
                                        option,
                                        fragment,
                                        target,
                                        index: 0,
                                    },
                                    true,
                                    cancel,
                                )?;
                            }
                            HtmlSelectedcontentUpdate::Skip => (),
                            HtmlSelectedcontentUpdate::Unavailable => {
                                return Err(DomError::SelectProtocol);
                            }
                        }
                    }
                    Effect::Children {
                        option,
                        fragment,
                        target,
                        index,
                    } => {
                        if let Some(&source) = self.node(option)?.children.get(index) {
                            self.push_effect(
                                Effect::Children {
                                    option,
                                    fragment,
                                    target,
                                    index: index + 1,
                                },
                                true,
                                cancel,
                            )?;
                            self.begin_clone(
                                import::Source::Staged(source),
                                None,
                                import::CloneRequest {
                                    document: self.node(fragment)?.document,
                                    placement: import::Placement::AfterClone(fragment),
                                    subtree: true,
                                    scoped: true,
                                },
                                limits,
                                cancel,
                            )?;
                        } else {
                            self.push_effect(Effect::Replace { fragment, target }, true, cancel)?;
                        }
                    }
                    Effect::Replace { fragment, target } => {
                        if let Some(&child) = self.node(target)?.children.first() {
                            self.push_effect(Effect::Replace { fragment, target }, true, cancel)?;
                            self.remove(child, limits, cancel)?;
                        } else {
                            self.push_effect(Effect::Cleanup(fragment), true, cancel)?;
                            self.pre_insert(fragment, target, None, limits, cancel)?;
                        }
                    }
                    Effect::Cleanup(fragment) => {
                        let Key::Created(index) = fragment else {
                            unreachable!("private algorithm fragment")
                        };
                        let node = self.node(fragment)?;
                        assert!(
                            node.children.is_empty()
                                && node.parent.is_none()
                                && node.host.is_none()
                                && matches!(node.kind, DomNodeKind::DocumentFragment)
                        );
                        self.created[index] = None;
                        self.nodes -= 1;
                    }
                }
            }
            if self.effects.is_empty() {
                self.effect_bytes = 0;
                self.effects = Vec::new();
            }
            Ok(())
        })();
        self.effects_running = false;
        self.effect_scoped = false;
        result
    }
    pub(super) fn schedule_selectedcontent(
        &mut self,
        select: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.multiple(select, cancel)? {
            return Ok(());
        }
        let mut targets = self.select_vec();
        let mut next = Some(select);
        while let Some(key) = next {
            self.select_step(cancel)?;
            if self.tag(key)? == Some(Tag::Selectedcontent)
                && !self.element(key)?.selectedcontent_disabled().expect("SC")
            {
                self.select_reserve(&mut targets, 1, cancel)?;
                targets.push(key);
            }
            next = crate::slot::next_admitted(self, select, key, &mut || self.select_step(cancel))?;
        }
        self.push_effect(
            Effect::Update {
                select,
                targets,
                index: 0,
            },
            true,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn select_attribute(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        change: attributes::AttributeChange,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if name.namespace().uri().is_some() {
            return Ok(());
        }
        let is = |a: Attr| {
            name.local_name()
                .code_units()
                .iter()
                .copied()
                .eq(a.local_name().encode_utf16())
        };
        let changed_presence = !matches!(change, attributes::AttributeChange::Set);
        match self.tag(key)? {
            Some(Tag::Select) if is(Attr::Size) => {
                self.select_operation(
                    key,
                    HtmlSelectOperation::ResetSelectedness { has_changed: false },
                    limits,
                    cancel,
                )?;
            }
            Some(Tag::Select) if is(Attr::Multiple) && changed_presence => {
                self.select_operation(
                    key,
                    if matches!(change, attributes::AttributeChange::Removed) {
                        HtmlSelectOperation::MultipleRemoved
                    } else {
                        HtmlSelectOperation::MultipleAdded
                    },
                    limits,
                    cancel,
                )?;
            }
            Some(Tag::Option) if is(Attr::Selected) && changed_presence => {
                self.select_step(cancel)?;
                let changed = self
                    .element_mut(key)
                    .option_state_mut()
                    .expect("Option")
                    .flags
                    .selected_attribute_changed(!matches!(
                        change,
                        attributes::AttributeChange::Removed
                    ));
                if let Some(select) = self.node(key)?.option_select {
                    self.select_operation(
                        select,
                        HtmlSelectOperation::ResetSelectedness {
                            has_changed: changed,
                        },
                        limits,
                        cancel,
                    )?;
                }
            }
            Some(Tag::Option) if is(Attr::Disabled) && changed_presence => {
                if let Some(select) = self.node(key)?.option_select {
                    self.select_operation(
                        select,
                        HtmlSelectOperation::ResetSelectedness { has_changed: false },
                        limits,
                        cancel,
                    )?;
                }
            }
            Some(Tag::Optgroup) if is(Attr::Disabled) && changed_presence => {
                if let Some(select) = self.nearest_option_select(key, cancel)? {
                    self.select_operation(
                        select,
                        HtmlSelectOperation::ResetSelectedness { has_changed: false },
                        limits,
                        cancel,
                    )?;
                }
            }
            _ => (),
        }
        self.run_effects(limits, cancel)
    }
    pub(super) fn select_edit(
        &mut self,
        key: Key,
        edit: &crate::select::Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        match edit {
            crate::select::Edit::OptionSelected(selected) => {
                self.select_step(cancel)?;
                self.element_mut_checked(key)?
                    .option_state_mut()
                    .ok_or(DomError::OptionInapplicable)?
                    .flags
                    .selected_setter(*selected);
                if let Some(select) = self.node(key)?.option_select {
                    self.select_operation(
                        select,
                        HtmlSelectOperation::AskForReset,
                        limits,
                        cancel,
                    )?;
                }
            }
            crate::select::Edit::Reset => {
                self.select_operation(key, HtmlSelectOperation::ResetControl, limits, cancel)?;
            }
            edit => {
                self.element(key)?
                    .select_state()
                    .ok_or(DomError::SelectInapplicable)?;
                let options = self.options(key, cancel)?;
                let cursor = HtmlSelectSetterCursor::new(options.len());
                let formation = source::formation::Formation::new(self, cancel)?;
                if let (Some(formation), crate::select::Edit::Value(value)) = (&formation, edit) {
                    let mut payload = source::storage::Payload::default();
                    payload.string(value)?;
                    formation.retain_payload(payload, cancel)?;
                }
                let edit = edit.clone();
                self.push_effect_retained(
                    Effect::Setter {
                        key,
                        options,
                        cursor,
                        edit,
                        acknowledge: false,
                    },
                    true,
                    formation.map(source::formation::Formation::into_charge),
                    cancel,
                )?;
            }
        }
        self.run_effects(limits, cancel)
    }
    fn select_setter_step(
        &mut self,
        key: Key,
        options: ScratchVec<Key>,
        mut cursor: HtmlSelectSetterCursor,
        continuation: SetterContinuation,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let SetterContinuation {
            edit,
            acknowledge,
            payload,
        } = continuation;
        if acknowledge {
            cursor
                .reply(HtmlSelectSetterReply::Applied)
                .map_err(|_| DomError::SelectProtocol)?;
        }
        let reply = match cursor.action() {
            HtmlSelectSetterAction::Deselect { ordinal } => {
                self.write_selected(options[ordinal], false);
                HtmlSelectSetterReply::Applied
            }
            HtmlSelectSetterAction::ReadMatch { ordinal } => {
                HtmlSelectSetterReply::Matches(match &edit {
                    crate::select::Edit::Index(index) => {
                        usize::try_from(*index).ok() == Some(ordinal)
                    }
                    crate::select::Edit::Value(value) => {
                        let actual = self.option_value(options[ordinal], limits, cancel)?;
                        let matches = self.same_select_units(&actual.value, value, cancel)?;
                        drop(actual.value);
                        drop(actual.payload);
                        matches
                    }
                    _ => unreachable!("only value/index setter owns a setter cursor"),
                })
            }
            HtmlSelectSetterAction::SelectAndDirty { ordinal } => {
                self.element_mut(options[ordinal])
                    .option_state_mut()
                    .expect("Option")
                    .flags
                    .select_for_select_setter();
                HtmlSelectSetterReply::Applied
            }
            HtmlSelectSetterAction::UpdateSelectedcontent => {
                self.push_effect_retained(
                    Effect::Setter {
                        key,
                        options,
                        cursor,
                        edit,
                        acknowledge: true,
                    },
                    true,
                    payload,
                    cancel,
                )?;
                return self.schedule_selectedcontent(key, limits, cancel);
            }
            HtmlSelectSetterAction::Complete => return Ok(()),
        };
        cursor.reply(reply).map_err(|_| DomError::SelectProtocol)?;
        self.push_effect_retained(
            Effect::Setter {
                key,
                options,
                cursor,
                edit,
                acknowledge: false,
            },
            true,
            payload,
            cancel,
        )
    }
    fn element_mut_checked(&mut self, key: Key) -> Result<&mut DomElement, DomError> {
        match &mut self.node_mut(key).kind {
            DomNodeKind::Element(e) => Ok(e),
            _ => Err(DomError::WrongNodeKind),
        }
    }
    fn same_select_units(
        &self,
        a: &DocumentString,
        b: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.code_units().iter().zip(b.code_units()) {
            self.select_step(cancel)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn option_value(
        &self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<value::Value, DomError> {
        self.controlled_option_value(key, limits, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

    #[test]
    fn display_size_borrowed_projection_preserves_actual_select_size_recovery() {
        let cancel = AtomicBool::new(false);
        let cases: &[(Option<&str>, bool, bool)] = &[
            (None, false, true),
            (None, true, false),
            (Some(""), false, true),
            (Some("bogus"), true, false),
            (Some("0"), false, false),
            (Some("000"), false, false),
            (Some("1"), false, true),
            (Some("0001junk"), true, true),
            (Some("+1"), true, true),
            (Some("  +0001tail"), false, true),
            (Some("-0"), false, false),
            (Some("-000"), true, false),
            (Some("-1"), false, true),
            (Some("-1"), true, false),
            (Some("2"), false, false),
            (Some("10"), false, false),
        ];
        for &(raw, multiple, expected) in cases {
            let mut dom = Dom::new(
                DomLimits::new(16, 100_000, 64, 0, 0, 8, 100_000)
                    .unwrap()
                    .with_select_processing(64, 1_000_000, 1_000_000)
                    .unwrap(),
            )
            .unwrap();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let select = batch.create_element_ns(&document, crate::Namespace::html(), "select");
            if let Some(raw) = raw {
                batch.set_attribute(&select, "size", raw);
            }
            if multiple {
                batch.set_attribute(&select, "multiple", "");
            }
            let created = dom.apply(batch, &cancel).unwrap();
            let select = created.created(&select).unwrap().unwrap();
            let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
            assert_eq!(
                draft
                    .display_size(Key::Existing(select.slot), &cancel)
                    .unwrap(),
                expected,
                "raw={raw:?}, multiple={multiple}"
            );
        }
        for (raw, wrong_namespace, multiple, expected) in [
            (
                DocumentString::from("0".repeat(8192) + "1tail"),
                false,
                true,
                true,
            ),
            (DocumentString::from_utf16(vec![0xd800]), false, false, true),
            (DocumentString::from("1"), true, true, false),
        ] {
            let mut dom = Dom::new(
                DomLimits::new(16, 100_000, 64, 0, 0, 8, 100_000)
                    .unwrap()
                    .with_select_processing(64, 1_000_000, 1_000_000)
                    .unwrap(),
            )
            .unwrap();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let select = batch.create_element_ns(&document, crate::Namespace::html(), "select");
            if wrong_namespace {
                batch.set_attribute_ns(
                    &select,
                    crate::Namespace::from_uri("urn:other"),
                    "size",
                    raw,
                );
            } else {
                batch.set_attribute(&select, "size", raw);
            }
            if multiple {
                batch.set_attribute(&select, "multiple", "");
            }
            let created = dom.apply(batch, &cancel).unwrap();
            let select = created.created(&select).unwrap().unwrap();
            let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
            assert_eq!(
                draft
                    .display_size(Key::Existing(select.slot), &cancel)
                    .unwrap(),
                expected
            );
        }
    }
    fn policy(work: usize, bytes: usize) -> DomLimits {
        DomLimits::new(32, 4096, 64, 0, 0, 8, 65536)
            .unwrap()
            .with_select_processing(16, 100000, 100000)
            .unwrap()
            .with_select_value_queries(
                HtmlCaptureLimits::new(32, 64, 32, 4096, 32, 0, 0),
                HtmlSemanticLimits::new(work, 1, 128, 0, bytes, 128),
            )
    }
    fn fixture() -> (Dom, NodeId, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(policy(100000, 100000)).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let s = b.create_element_ns(&d, crate::Namespace::html(), "select");
        let o = b.create_element_ns(&d, crate::Namespace::html(), "option");
        b.set_attribute(&o, "value", "value");
        b.insert_before(&o, &s, None);
        let c = dom.apply(b, &cancel).unwrap();
        (
            dom,
            c.created(&s).unwrap().unwrap(),
            c.created(&o).unwrap().unwrap(),
        )
    }
    #[test]
    fn value_queries_admit_exact_actual_producer_usage_and_accumulate_across_commands() {
        let (mut dom, select, option) = fixture();
        let cancel = AtomicBool::new(false);
        let draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        let k = draft.resolve(option.into()).unwrap();
        assert_eq!(
            draft.option_value(k, dom.limits, &cancel).unwrap().value,
            DocumentString::from("value")
        );
        let work = draft.select_semantic_work.get();
        let bytes = draft.select_semantic_bytes.get();
        assert!(work > 0 && bytes > 0);
        drop(draft);
        // These are observed producer usage boundaries, not estimates of its
        // private layouts. A single real setter fits both exact allowances.
        dom.set_limits(policy(work, bytes)).unwrap();
        let mut b = DomBatch::new();
        b.set_select_value(select, "value");
        dom.apply(b, &cancel).unwrap();
        for limits in [policy(work - 1, bytes), policy(work, bytes - 1)] {
            dom.set_limits(limits).unwrap();
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            b.set_select_value(select, "different");
            assert!(matches!(
                dom.apply(b, &cancel),
                Err(DomError::SelectSemantic(_))
            ));
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
        for limits in [policy(work, 100000), policy(100000, bytes)] {
            dom.set_limits(limits).unwrap();
            let before = dom.snapshot();
            let mut b = DomBatch::new();
            b.set_select_value(select, "value");
            b.set_select_value(select, "value");
            assert!(matches!(
                dom.apply(b, &cancel),
                Err(DomError::SelectSemantic(_))
            ));
            assert_eq!(dom.snapshot().stamp(), before.stamp());
        }
    }
    #[test]
    fn unconfigured_query_and_short_capture_refuse_only_at_reached_value_query() {
        let (mut dom, select, _) = fixture();
        let cancel = AtomicBool::new(false);
        let base = DomLimits::new(32, 4096, 64, 0, 0, 8, 65536)
            .unwrap()
            .with_select_processing(16, 100000, 100000)
            .unwrap();
        dom.set_limits(base).unwrap();
        let mut b = DomBatch::new();
        b.set_select_selected_index(select, -1);
        dom.apply(b, &cancel).unwrap();
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.set_select_value(select, "value");
        assert_eq!(
            dom.apply(b, &cancel).unwrap_err(),
            DomError::SelectValueQueriesUnconfigured
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        dom.set_limits(base.with_select_value_queries(
            HtmlCaptureLimits::new(1, 64, 32, 4096, 32, 0, 0),
            HtmlSemanticLimits::new(100000, 1, 128, 0, 100000, 128),
        ))
        .unwrap();
        let mut b = DomBatch::new();
        b.set_select_value(select, "value");
        assert!(matches!(
            dom.apply(b, &cancel),
            Err(DomError::SelectCapture(_))
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    #[test]
    fn actual_dom_work_limit_is_exact_and_one_short_refuses_atomically() {
        let (mut dom, select, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        let key = draft.resolve(select.into()).unwrap();
        draft
            .select_edit(key, &crate::select::Edit::Index(-1), dom.limits, &cancel)
            .unwrap();
        let work = draft.select_work.get();
        assert!(work > 0);
        drop(draft);
        dom.set_limits(
            policy(100000, 100000)
                .with_select_processing(16, 100000, work - 1)
                .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.set_select_selected_index(select, -1);
        assert_eq!(
            dom.apply(b, &cancel).unwrap_err(),
            DomError::SelectWorkLimit { limit: work - 1 }
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        dom.set_limits(
            policy(100000, 100000)
                .with_select_processing(16, 100000, work)
                .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        b.set_select_selected_index(select, -1);
        dom.apply(b, &cancel).unwrap();
    }
    #[test]
    fn issued_task_revision_refusal_preserves_permission_and_serial_exhaustion_is_atomic() {
        let (mut dom, _, option) = fixture();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        b.detach(option);
        dom.apply(b, &cancel).unwrap();
        let checkpoint = crate::DomNativeCheckpointLimits::new(
            100000,
            100000,
            crate::DomSlotQueryLimits::new(0, 0).unwrap(),
        )
        .unwrap();
        let crate::DomNativeCheckpoint::Select(task) =
            dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
        else {
            panic!("Select")
        };
        let old_revision = dom.revision;
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        assert_eq!(
            dom.complete_select_task(&task, checkpoint, &cancel)
                .unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.pending_native_tasks(), 1);
        assert_eq!(task.authorization.status(), 0);
        dom.revision = old_revision;
        dom.complete_select_task(&task, checkpoint, &cancel)
            .unwrap();
        assert_eq!(task.authorization.status(), 1);
        let (mut dom, select, option) = fixture();
        Arc::get_mut(&mut dom.state)
            .expect("unshared fixture state")
            .next_task = u64::MAX;
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.detach(option);
        assert_eq!(
            dom.apply(b, &cancel).unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.pending_native_tasks(), 0);
        assert_eq!(dom.snapshot().node(option).unwrap().parent(), Some(select));
    }
    #[test]
    fn ordinary_effect_bank_is_registered_before_select_is_reached() {
        let (dom, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        draft.select_scratch_limit = 0;
        let mut ordinary = draft.effect_vec::<Key>();
        draft.select_reserve(&mut ordinary, 8, &cancel).unwrap();
        assert_eq!(
            draft.select_temporary_bytes.load(Ordering::Relaxed),
            ordinary.capacity() * size_of::<Key>()
        );
        draft.effect_scoped = true;
        assert_eq!(
            draft.select_step(&cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SelectScratch,
                limit: 0
            })
        );
        drop(ordinary);
        assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
        draft.select_step(&cancel).unwrap();
    }
    #[test]
    fn simultaneous_dom_banks_share_exact_capacity_and_release_only_their_own_charge() {
        let (dom, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        let width = size_of::<Key>();
        draft.select_scratch_limit = 2 * width;
        let mut first = draft.select_vec::<Key>();
        draft.select_reserve(&mut first, 1, &cancel).unwrap();
        let mut second = draft.select_vec::<Key>();
        draft.select_reserve(&mut second, 1, &cancel).unwrap();
        // The complete private owner and both live charges move together.
        // Exact/one-short admission and one-time release survive the handoff.
        std::thread::spawn(move || {
            assert_eq!(
                draft.select_temporary_bytes.load(Ordering::Relaxed),
                2 * width
            );
            drop(second);
            assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), width);
            draft.select_scratch_limit = 2 * width - 1;
            let mut refused = draft.select_vec::<Key>();
            assert!(matches!(
                draft.select_reserve(&mut refused, 1, &cancel),
                Err(DomError::ResourceLimit {
                    resource: DomResource::SelectScratch,
                    ..
                })
            ));
            assert_eq!(refused.capacity(), 0);
            assert_eq!(first.capacity(), 1);
            drop(first);
            assert_eq!(draft.select_temporary_bytes.load(Ordering::Relaxed), 0);
        })
        .join()
        .unwrap();
    }
}

#[cfg(test)]
mod effect_scope_tests {
    use super::*;
    fn fixture() -> (Dom, Draft, Key, Key, Key, Key) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 4096, 32, 0, 0, 8, 1_000_000).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element(&document, "root");
        let outer = batch.create_element(&document, "outer");
        let inner = batch.create_element(&document, "inner");
        let first = batch.create_text(&document, "A");
        let second = batch.create_text(&document, "B");
        batch.insert_before(&root, &document, None);
        batch.insert_before(&outer, &root, None);
        batch.insert_before(&inner, &root, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let key = |ticket: &crate::CreatedNode| {
            Key::Existing(commit.created(ticket).unwrap().unwrap().slot)
        };
        let (outer, inner, first, second) = (key(&outer), key(&inner), key(&first), key(&second));
        let draft = Draft::new(&dom, DomBatch::new().owner, &cancel).unwrap();
        (dom, draft, outer, inner, first, second)
    }
    #[test]
    fn nested_constructor_effects_preserve_the_parked_root_order() {
        let (dom, mut draft, outer, inner, first, second) = fixture();
        let cancel = AtomicBool::new(false);
        draft
            .push_effect(
                Effect::Tree(super::super::tree::Frame::ReplaceAll {
                    key: outer,
                    replacement: Some(first),
                    inserted: false,
                }),
                false,
                &cancel,
            )
            .unwrap();
        draft
            .with_effect_scope(|draft| draft.pre_insert(second, inner, None, dom.limits, &cancel))
            .unwrap();
        assert!(draft.node(outer).unwrap().children.is_empty());
        assert_eq!(draft.node(inner).unwrap().children.as_slice(), &[second]);
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert_eq!(draft.node(outer).unwrap().children.as_slice(), &[first]);
        assert_eq!(draft.node(inner).unwrap().children.as_slice(), &[second]);
        assert!(draft.effects.is_empty());
    }
    #[test]
    fn cancelled_constructor_scope_restores_the_original_root_continuation() {
        let (dom, mut draft, outer, inner, first, second) = fixture();
        let cancel = AtomicBool::new(false);
        draft
            .push_effect(
                Effect::Tree(super::super::tree::Frame::ReplaceAll {
                    key: outer,
                    replacement: Some(first),
                    inserted: false,
                }),
                false,
                &cancel,
            )
            .unwrap();
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            draft.with_effect_scope(
                |draft| draft.pre_insert(second, inner, None, dom.limits, &cancel)
            ),
            Err(DomError::Cancelled)
        );
        cancel.store(false, Ordering::Relaxed);
        draft.run_effects(dom.limits, &cancel).unwrap();
        assert_eq!(draft.node(outer).unwrap().children.as_slice(), &[first]);
        assert!(draft.node(inner).unwrap().children.is_empty());
    }
}
