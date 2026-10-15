//! Source-ordered Select-family adaptation over the canonical staged tree.
use super::*;
use surgeist_html::bounded::*;
use surgeist_html::{KnownHtmlAttribute as Attr, KnownHtmlElement as Tag, *};

/// A DOM-owned temporary bank debits one shared live-capacity ledger. Its
/// lifetime can cross nested synchronous effects; dropping it releases only
/// its own admitted capacity. Producer-owned capture/session banks stay separate.
pub(super) struct ScratchVec<T> {
    values: Vec<T>,
    live: std::rc::Rc<std::cell::Cell<usize>>,
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
impl<T> Drop for ScratchVec<T> {
    fn drop(&mut self) {
        self.live
            .set(self.live.get() - self.values.capacity() * size_of::<T>());
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
pub(super) fn draft_tasks(
    tasks: &[Task<NodeId>],
    cancel: &AtomicBool,
) -> Result<Vec<Pending>, DomError> {
    let mut out = Vec::new();
    reserve(&mut out, tasks.len())?;
    for task in tasks {
        check_cancel(cancel)?;
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
        reserve(&mut self.native_tasks, 1)?;
        self.native_tasks.push(Pending::Update {
            serial: None,
            target,
        });
        Ok(())
    }
    pub(super) fn queue_slot_marker(&mut self) -> Result<(), DomError> {
        if !self.notification_queued {
            reserve(&mut self.native_tasks, 1)?;
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
            check_cancel(cancel)?;
            let task = self.native_tasks[i];
            if matches!(task, Pending::Update { .. }) {
                self.select_step(cancel)?;
            }
            if let Pending::Update { serial, target } = task
                && target == key
            {
                if let Key::Existing(slot) = key {
                    reserve(&mut self.cancelled_selects, 1)?;
                    self.cancelled_selects.push((
                        serial,
                        NodeId {
                            owner: self.store,
                            slot,
                            generation: self.generations[slot],
                        },
                    ));
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
        cancel: &AtomicBool,
    ) -> Result<FinishedTasks, DomError> {
        let mut next = dom.state.next_task;
        let mut out = Vec::new();
        let mut cancelled = Vec::new();
        reserve(&mut out, self.native_tasks.len())?;
        reserve(&mut cancelled, self.cancelled_selects.len())?;
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
            check_cancel(cancel)?;
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
        for &(old, target) in &self.cancelled_selects {
            check_cancel(cancel)?;
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
    fn element(&self, key: Key) -> Result<&DomElement, DomError> {
        match &self.node(key)?.kind {
            DomNodeKind::Element(e) => Ok(e),
            _ => Err(DomError::WrongNodeKind),
        }
    }
    fn element_mut(&mut self, key: Key) -> &mut DomElement {
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
        check_cancel(cancel)?;
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
            check_cancel(cancel)
        }
    }
    fn select_scratch(&self, bytes: usize) -> Result<(), DomError> {
        let total = self
            .effect_bytes
            .checked_add(self.select_temporary_bytes.get())
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
            self.select_temporary_bytes.set(
                self.select_temporary_bytes
                    .get()
                    .checked_add(actual - old)
                    .ok_or(DomError::RepresentationExhausted)?,
            );
        }
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
        self.select_reserve(&mut pending, self.node(key)?.children.len())?;
        pending.extend(self.node(key)?.children.iter().rev().map(|&k| (k, false)));
        while let Some((key, group)) = pending.pop() {
            self.select_step(cancel)?;
            let tag = self.tag(key)?;
            match select_option_rule(tag, group) {
                HtmlSelectOptionRule::IncludeAndPrune => {
                    self.select_reserve(&mut result, 1)?;
                    result.push(key);
                }
                HtmlSelectOptionRule::Prune => (),
                HtmlSelectOptionRule::Descend => {
                    self.select_reserve(&mut pending, self.node(key)?.children.len())?;
                    pending.extend(
                        self.node(key)?
                            .children
                            .iter()
                            .rev()
                            .map(|&k| (k, group || tag == Some(Tag::Optgroup))),
                    );
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
        let element = self.element(key)?;
        let raw = element.attributes().iter().find(|a| {
            a.name().namespace().uri().is_none()
                && a.name()
                    .local_name()
                    .code_units()
                    .iter()
                    .copied()
                    .eq("size".encode_utf16())
        });
        let multiple = element.attributes().iter().find(|a| {
            a.name().namespace().uri().is_none()
                && a.name()
                    .local_name()
                    .code_units()
                    .iter()
                    .copied()
                    .eq("multiple".encode_utf16())
        });
        for a in [raw, multiple].into_iter().flatten() {
            for _ in a.value().code_units() {
                self.select_step(cancel)?;
            }
        }
        let report = HtmlValueInterpreter::new(
            raw.map_or(0, |a| a.value().len())
                .max(multiple.map_or(0, |a| a.value().len())),
        )
        .select_size(element.name(), raw, multiple)
        .map_err(|_| DomError::SelectProtocol)?;
        Ok(
            matches!(report.display_size(),HtmlSelectDisplaySize::Value{value,..} if value.decimal_digits()=="1"),
        )
    }
    pub(super) fn select_operation(
        &mut self,
        key: Key,
        op: HtmlSelectOperation,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.element(key)?
            .select_state()
            .ok_or(DomError::SelectInapplicable)?;
        let multiple = self.multiple(key, cancel)?;
        // Preserve the shared multiple fast path without walking an irrelevant list.
        let options = if multiple && op == HtmlSelectOperation::Normalize {
            self.select_vec()
        } else {
            self.options(key, cancel)?
        };
        let mut cursor = HtmlSelectCursor::new(op, multiple, options.len())
            .map_err(|_| DomError::SelectProtocol)?;
        loop {
            self.select_step(cancel)?;
            let reply = match cursor.action() {
                HtmlSelectAction::ResetSelectFlags => {
                    self.element_mut(key)
                        .select_state_mut()
                        .expect("Select")
                        .flags
                        .reset();
                    HtmlSelectReply::Applied
                }
                HtmlSelectAction::ResetOption { ordinal } => {
                    let k = options[ordinal];
                    let selected = self.has(k, Attr::Selected, cancel)?;
                    self.element_mut(k)
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
                    self.schedule_selectedcontent(key, limits, cancel)?;
                    HtmlSelectReply::Applied
                }
                HtmlSelectAction::Complete {
                    normalization_changed,
                } => return Ok(normalization_changed),
            };
            cursor.reply(reply).map_err(|_| DomError::SelectProtocol)?;
        }
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
        for select in [old, new].into_iter().flatten() {
            let changed =
                self.select_operation(select, HtmlSelectOperation::Normalize, limits, cancel)?;
            if removing && (self.selected(key)? || changed) {
                self.queue_select(select, limits, cancel)?;
            }
        }
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
                    )?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

pub(super) enum Effect {
    Clone(import::Frame),
    Post {
        list: ScratchVec<Key>,
        index: usize,
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
    effect: Effect,
    scoped: bool,
}
impl Draft {
    pub(super) fn push_effect(&mut self, effect: Effect, scoped: bool) -> Result<(), DomError> {
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
        if scoped {
            self.select_scratch(new_allocation)?;
        }
        #[cfg(test)]
        let before_growth = self.effect_bytes + self.select_temporary_bytes.get();
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
        self.effects.push(EffectFrame { effect, scoped });
        Ok(())
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
            while let Some(EffectFrame { effect, scoped }) = self.effects.pop() {
                check_cancel(cancel)?;
                if scoped {
                    self.select_step(cancel)?;
                }
                self.effect_scoped = scoped;
                match effect {
                    Effect::Clone(frame) => self.clone_effect(frame, scoped, limits, cancel)?,
                    Effect::Post { list, index } => {
                        if let Some(&key) = list.get(index) {
                            self.push_effect(
                                Effect::Post {
                                    list,
                                    index: index + 1,
                                },
                                scoped,
                            )?;
                            if self.shadow_connected(key, cancel)? {
                                self.navigable_post_connection_node(key, &mut || {
                                    check_cancel(cancel)
                                })?;
                                self.script_post_connection(key, cancel)?;
                                self.svg_script_post_connection(key, cancel)?;
                                self.select_post_connection(key, limits, cancel)?;
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
                            )?;
                            self.push_effect(Effect::Target { select, target }, true)?;
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
                                while let Some(&child) = self.node(target)?.children.first() {
                                    self.select_step(cancel)?;
                                    self.remove(child, limits, cancel)?;
                                }
                            }
                            HtmlSelectedcontentUpdate::CloneOptionChildren { ordinal } => {
                                let option = options[ordinal];
                                let document = self.node(option)?.document;
                                let fragment =
                                    self.allocate(DomNodeKind::DocumentFragment, document, limits)?;
                                self.push_effect(
                                    Effect::Children {
                                        option,
                                        fragment,
                                        target,
                                        index: 0,
                                    },
                                    true,
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
                            )?;
                            self.begin_clone(
                                import::Source::Staged(source),
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
                            self.push_effect(Effect::Replace { fragment, target }, true)?;
                        }
                    }
                    Effect::Replace { fragment, target } => {
                        while let Some(&child) = self.node(target)?.children.first() {
                            self.select_step(cancel)?;
                            self.remove(child, limits, cancel)?;
                        }
                        self.push_effect(Effect::Cleanup(fragment), true)?;
                        self.pre_insert(fragment, target, None, limits, cancel)?;
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
            self.effect_bytes = 0;
            self.effects = Vec::new();
            Ok(())
        })();
        self.effects_running = false;
        self.effect_scoped = false;
        result
    }
    fn schedule_selectedcontent(
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
                self.select_reserve(&mut targets, 1)?;
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
        edit: crate::select::Edit,
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
                    .selected_setter(selected);
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
                let mut cursor = HtmlSelectSetterCursor::new(options.len());
                loop {
                    self.select_step(cancel)?;
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
                                    let actual =
                                        self.option_value(options[ordinal], limits, cancel)?;
                                    self.same_select_units(&actual, value, cancel)?
                                }
                                _ => unreachable!(),
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
                            self.schedule_selectedcontent(key, limits, cancel)?;
                            HtmlSelectSetterReply::Applied
                        }
                        HtmlSelectSetterAction::Complete => break,
                    };
                    cursor.reply(reply).map_err(|_| DomError::SelectProtocol)?;
                }
            }
        }
        self.run_effects(limits, cancel)
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
    ) -> Result<DocumentString, DomError> {
        use surgeist_html_semantics::*;
        self.select_step(cancel)?;
        let revision = HtmlHostFactsRevision::fresh();
        let (capture_limits, semantic_limits) = limits
            .select_value_queries
            .ok_or(DomError::SelectValueQueriesUnconfigured)?;
        let mut builder = HtmlCaptureBuilder::new(capture_limits, revision.clone());
        let document = match &self.node(self.node(key)?.document)?.kind {
            DomNodeKind::Document(d) => d,
            _ => return Err(DomError::WrongDocument),
        };
        let doc = builder
            .add_document(
                document.mode(),
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .map_err(DomError::SelectCapture)?;
        check_cancel(cancel)?;
        // Only the ordinary Option subtree influences this query. Its omitted
        // ancestry remains unavailable; association-owned subtrees are not children.
        let mut entries = self.select_vec();
        let mut next = Some(key);
        while let Some(k) = next {
            self.select_step(cancel)?;
            let subject = match &self.node(k)?.kind {
                DomNodeKind::Element(e) => builder.add_element(
                    &doc,
                    e.name().clone(),
                    e.attributes().clone(),
                    match e.saved_is() {
                        None => HtmlCapturedIsValue::Absent,
                        Some(v) => HtmlCapturedIsValue::Value(v.clone()),
                    },
                    HtmlCaptureRegistryInput::Null,
                ),
                DomNodeKind::Text(s) => builder.add_text(&doc, s.clone()),
                DomNodeKind::Comment(s) => builder.add_comment(&doc, s.clone()),
                DomNodeKind::ProcessingInstruction(p) => {
                    builder.add_processing_instruction(&doc, p.target().clone(), p.data().clone())
                }
                DomNodeKind::DocumentType(d) => builder.add_doctype(
                    &doc,
                    HtmlDoctype::new(
                        d.name().clone(),
                        Some(d.public_id().clone()),
                        Some(d.system_id().clone()),
                    ),
                ),
                _ => return Err(DomError::SelectProtocol),
            }
            .map_err(DomError::SelectCapture)?;
            check_cancel(cancel)?;
            self.select_reserve(&mut entries, 1)?;
            entries.push((k, subject));
            next = crate::slot::next_admitted(self, key, k, &mut || self.select_step(cancel))?;
        }
        for (k, subject) in entries.iter() {
            self.select_step(cancel)?;
            if !matches!(self.node(*k)?.kind, DomNodeKind::Element(_)) {
                continue;
            }
            let mut children = self.select_vec();
            for child in &self.node(*k)?.children {
                let mut found = None;
                for (candidate, subject) in entries.iter() {
                    self.select_step(cancel)?;
                    if candidate == child {
                        found = Some(subject.clone());
                        break;
                    }
                }
                self.select_reserve(&mut children, 1)?;
                children.push(found.ok_or(DomError::SelectProtocol)?);
            }
            // Producer validation is bounded independently by captured subjects,
            // relationships and depth; poll both sides of this atomic builder call.
            builder
                .establish_children(subject, &children)
                .map_err(DomError::SelectCapture)?;
            self.select_step(cancel)?;
        }
        check_cancel(cancel)?;
        let capture = builder.finish().map_err(DomError::SelectCapture)?;
        check_cancel(cancel)?;
        let subject = capture
            .subject_id(&entries[0].1)
            .map_err(DomError::SelectCapture)?;
        check_cancel(cancel)?;
        let context = HtmlSemanticContext::new(
            &capture,
            revision,
            HtmlSemanticContextInput::new(Vec::new()),
            0,
        )
        .map_err(|_| DomError::SelectProtocol)?;
        check_cancel(cancel)?;
        // Producer internals have their own public retention/work contracts.
        // Only real DOM mapping banks belong to the Select scratch domain.
        let remaining = HtmlSemanticLimits::new(
            semantic_limits
                .work()
                .saturating_sub(self.select_semantic_work.get()),
            semantic_limits.output_records(),
            semantic_limits.scratch_records(),
            semantic_limits.integer_digits(),
            semantic_limits
                .allocated_bytes()
                .saturating_sub(self.select_semantic_bytes.get()),
            semantic_limits.nested_output_records(),
        );
        check_cancel(cancel)?;
        let mut session = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(limits.units),
            remaining,
            HtmlSemanticCancellation::new(),
        )
        .map_err(DomError::SelectSemantic)?;
        check_cancel(cancel)?;
        let report = session
            .query(&[HtmlSemanticRequest::new(
                subject,
                HtmlSemanticQuery::OptionValue,
            )])
            .map_err(DomError::SelectSemantic)?;
        check_cancel(cancel)?;
        self.select_semantic_work.set(
            self.select_semantic_work
                .get()
                .checked_add(report.usage().work())
                .ok_or(DomError::RepresentationExhausted)?,
        );
        self.select_semantic_bytes.set(
            self.select_semantic_bytes
                .get()
                .checked_add(report.usage().allocated_bytes())
                .ok_or(DomError::RepresentationExhausted)?,
        );
        self.select_step(cancel)?;
        match report.results()[0].fact() {
            HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(value)) => {
                Ok(value.value().clone())
            }
            _ => Err(DomError::SelectProtocol),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

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
            draft.option_value(k, dom.limits, &cancel).unwrap(),
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
            .select_edit(key, crate::select::Edit::Index(-1), dom.limits, &cancel)
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
        draft.select_reserve(&mut ordinary, 8).unwrap();
        assert_eq!(
            draft.select_temporary_bytes.get(),
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
        assert_eq!(draft.select_temporary_bytes.get(), 0);
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
        draft.select_reserve(&mut first, 1).unwrap();
        let mut second = draft.select_vec::<Key>();
        draft.select_reserve(&mut second, 1).unwrap();
        assert_eq!(draft.select_temporary_bytes.get(), 2 * width);
        drop(second);
        assert_eq!(draft.select_temporary_bytes.get(), width);
        draft.select_scratch_limit = 2 * width - 1;
        let mut refused = draft.select_vec::<Key>();
        assert!(matches!(
            draft.select_reserve(&mut refused, 1),
            Err(DomError::ResourceLimit {
                resource: DomResource::SelectScratch,
                ..
            })
        ));
        assert_eq!(refused.capacity(), 0);
        assert_eq!(first.capacity(), 1);
        drop(first);
        assert_eq!(draft.select_temporary_bytes.get(), 0);
    }
}
