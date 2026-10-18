//! Fixed-counter headroom from the original, checkpointed command interpreter.
//! No source service, publication or invocation issuance runs during this scan.
use super::super::*;
use super::{checkpoint, formation};

pub(in crate::store) enum Entry<'a> {
    Command(&'a Command),
    Native {
        wrapper: Key,
        parent: Key,
        before: Option<Key>,
    },
    Select {
        target: Key,
    },
}

#[derive(Clone, Copy)]
pub(in crate::store) enum Kind {
    Update,
    OwnerAttributes,
    SetDisabled,
}

// Canonical publication stops in the supported Dialog domain. The final
// publication also carries requestClose's final force-enabled reset.
pub(in crate::store) const FINAL_PUBLICATION: u64 = 1;
pub(in crate::store) fn dialog_close(watcher: bool) -> u64 {
    // Toggle tracker, optional watcher stop, Close queue/open removal, final.
    2 + u64::from(watcher) + FINAL_PUBLICATION
}
pub(in crate::store) fn dialog_show(watcher: bool, modal: bool) -> u64 {
    // Toggle tracker, open insertion, optional watcher/modal prefix, final.
    2 + u64::from(watcher) + u64::from(modal) + FINAL_PUBLICATION
}
pub(in crate::store) fn request_close() -> u64 {
    // Request slots are published before the original watcher's Close action.
    1 + dialog_close(true)
}

#[derive(Debug)]
pub(in crate::store) struct Reservation {
    pub remaining_phases: u64,
}

struct Row {
    key: Key,
    association_start: u64,
    creates: u64,
}

// The inline header belongs to Draft's already priced Self. Only this real Vec
// backing acquires a separate charge. Drop the backing before releasing charge.
pub(in crate::store) struct Scan {
    rows: Vec<Row>,
    charge: Option<checkpoint::Charge>,
    phases: u64,
    invocations: u64,
    selects: u64,
    details: u64,
    dialog_wait: bool,
}
impl Scan {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            charge: None,
            phases: 0,
            invocations: 0,
            selects: 0,
            details: 0,
            dialog_wait: false,
        }
    }
    fn find(
        &self,
        draft: &Draft,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<usize>, DomError> {
        draft.source_step(cancel)?;
        for index in 0..self.rows.len() {
            draft.source_step(cancel)?;
            if self.rows[index].key == key {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn append(
        &mut self,
        draft: &Draft,
        key: Key,
        association_start: u64,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        // A fresh envelope reads the CURRENT draft, including all existing live
        // charges. Its reserve admits old/new backing overlap and charges only
        // actual growth; merge that custody even if the final poll refused.
        let budget = formation::Formation::new(draft, cancel)?;
        let result = formation::reserve(&mut self.rows, 1, budget.as_ref(), cancel);
        formation::merge_charge(
            &mut self.charge,
            budget.map(formation::Formation::into_charge),
        );
        result?;
        let index = self.rows.len();
        self.rows.push(Row {
            key,
            association_start,
            creates: 0,
        });
        Ok(index)
    }
    fn qualify(
        &self,
        draft: &Draft,
        dom: &Dom,
        used_phases: u64,
        cancel: &AtomicBool,
    ) -> Result<Reservation, DomError> {
        draft.source_step(cancel)?;
        draft
            .source_invocation
            .checked_add(self.invocations)
            .ok_or(DomError::RepresentationExhausted)?;
        draft.source_step(cancel)?;
        dom.state
            .next_task
            .checked_add(self.selects)
            .ok_or(DomError::RepresentationExhausted)?;
        draft.source_step(cancel)?;
        dom.state
            .next_details_intention
            .checked_add(self.details)
            .ok_or(DomError::RepresentationExhausted)?;
        for index in 0..self.rows.len() {
            draft.source_step(cancel)?;
            let row = &self.rows[index];
            row.association_start
                .checked_add(row.creates)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let remaining_phases = self
            .phases
            .checked_add(FINAL_PUBLICATION)
            .ok_or(DomError::RepresentationExhausted)?;
        if self.phases != 0 {
            draft.source_step(cancel)?;
            let limits = draft.source_limits.ok_or(DomError::StyleProtocol)?;
            let required = used_phases
                .checked_add(remaining_phases)
                .ok_or(DomError::RepresentationExhausted)?;
            if required > limits.phases {
                return Err(DomError::SourcePublicationLimit);
            }
            let protected = dom.source_reserved_closing(cancel)?;
            dom.revision
                .0
                .checked_add(remaining_phases)
                .and_then(|revision| revision.checked_add(dom.hosted_revisions))
                .and_then(|revision| revision.checked_add(protected))
                .ok_or(DomError::RevisionExhausted)?;
        }
        Ok(Reservation { remaining_phases })
    }
}

// Restore the actual private checkpoint on both Result paths and unwind. There
// are no external callbacks or fallible cleanup operations in this guard.
struct Restore<'a> {
    draft: &'a mut Draft,
    checkpoint: Option<checkpoint::Checkpoint>,
}
impl Drop for Restore<'_> {
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take() {
            checkpoint.restore(self.draft);
        }
    }
}

impl Draft {
    pub(in crate::store) fn admit_source_command_headroom(
        &mut self,
        dom: &Dom,
        entry: Entry<'_>,
        phases: u64,
        cancel: &AtomicBool,
    ) -> Result<Reservation, DomError> {
        if self.source_headroom.is_some() || self.source_limits.is_none() {
            return Err(DomError::StyleProtocol);
        }
        let checkpoint = self.source_checkpoint(self.source_other_bytes, 0, cancel)?;
        let (result, scan) = {
            let restore = Restore {
                draft: self,
                checkpoint: Some(checkpoint),
            };
            restore.draft.source_headroom = Some(Scan::new());
            let mut result = (|| {
                restore.draft.seed_headroom(cancel)?;
                match entry {
                    Entry::Command(command) => restore.draft.apply(command, dom.limits, cancel),
                    Entry::Native {
                        wrapper,
                        parent,
                        before,
                    } => restore
                        .draft
                        .pre_insert(wrapper, parent, before, dom.limits, cancel),
                    Entry::Select { target } => restore
                        .draft
                        .schedule_selectedcontent(target, dom.limits, cancel),
                }
            })();
            while matches!(result, Err(DomError::FullscreenExitHandoffRequired)) {
                restore.draft.fullscreen_exit_prefix(dom.limits, cancel)?;
                restore.draft.range_publication_ready(cancel)?;
                let scan = restore
                    .draft
                    .source_headroom
                    .as_mut()
                    .expect("original scan");
                // Handoff publication plus the invocation's retained mandatory
                // credit. The enclosing command's close remains separate.
                scan.phases = scan
                    .phases
                    .checked_add(2)
                    .ok_or(DomError::RepresentationExhausted)?;
                scan.invocations = scan
                    .invocations
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                restore.draft.fullscreen_exit = None;
                result = restore.draft.run_effects(dom.limits, cancel);
            }
            let dialog_wait = matches!(&result, Err(DomError::DialogProcessingUnavailable { action })
                if restore.draft.pending_dialog_hook().is_some_and(|pending| pending.action == *action) || matches!(action, surgeist_html::HtmlDialogAction::RequestWatcherClose { .. }));
            let design_mode_wait = matches!(&result, Err(DomError::DesignModeProcessingUnavailable { step })
                if restore.draft.pending_design_mode().is_some_and(|pending| pending.step == *step));
            if dialog_wait || design_mode_wait {
                restore.draft.range_publication_ready(cancel)?;
            }
            if design_mode_wait && let Some(scan) = &mut restore.draft.source_headroom {
                // Only an actual unavailable selection/focusing action exposes
                // one prefix. Its local flag/reset writes coalesce there; the
                // same original operation still owes its final publication.
                scan.phases = scan
                    .phases
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                scan.invocations = scan
                    .invocations
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
            if dialog_wait && let Some(scan) = &mut restore.draft.source_headroom {
                let stopped = if matches!(
                    &result,
                    Err(DomError::DialogProcessingUnavailable {
                        action: surgeist_html::HtmlDialogAction::RequestWatcherClose { .. }
                    })
                ) {
                    request_close() - FINAL_PUBLICATION
                } else {
                    1
                };
                scan.phases = scan
                    .phases
                    .checked_add(stopped)
                    .ok_or(DomError::RepresentationExhausted)?;
                scan.dialog_wait = true;
            }
            let scan = restore.draft.source_headroom.take();
            // `restore` drops here, before qualifying the retained real counts.
            (result, scan)
        };
        let scan = scan.ok_or(DomError::StyleProtocol)?;
        match result {
            Err(DomError::DialogProcessingUnavailable { .. }) if scan.dialog_wait => {}
            Err(DomError::DesignModeProcessingUnavailable { .. }) => {}
            result => result?,
        }
        scan.qualify(self, dom, phases, cancel)
    }

    pub(in crate::store) fn admit_dialog_occurrences(
        &self,
        count: u64,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.source_invocation
            .checked_add(count)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(())
    }
    pub(in crate::store) fn headroom_external_hook(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        if self.source_headroom.is_some() {
            self.range_publication_ready(cancel)?;
        }
        let Some(scan) = &mut self.source_headroom else {
            return Ok(false);
        };
        scan.invocations = scan
            .invocations
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        scan.phases = scan
            .phases
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(true)
    }
    pub(in crate::store) fn headroom_child_hook(
        &mut self,
        removing: bool,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        if self.source_headroom.is_some() {
            self.range_publication_ready(cancel)?;
        }
        let Some(scan) = &mut self.source_headroom else {
            return Ok(false);
        };
        scan.phases = scan
            .phases
            .checked_add(if removing { 2 } else { 6 })
            .ok_or(DomError::RepresentationExhausted)?;
        scan.invocations = scan
            .invocations
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(true)
    }

    fn seed_headroom(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.source_step(cancel)?;
        for index in 0..self.native_tasks.len() {
            self.source_step(cancel)?;
            if matches!(
                self.native_tasks[index],
                select::Pending::Update { serial: None, .. }
            ) {
                self.headroom_select_issue(cancel)?;
            }
        }
        self.source_step(cancel)?;
        for index in 0..self.cancelled_selects.len() {
            self.source_step(cancel)?;
            if self.cancelled_selects[index].0.is_none() {
                self.headroom_select_issue(cancel)?;
            }
        }
        self.source_step(cancel)?;
        for index in 0..self.details_intentions.len() {
            self.source_step(cancel)?;
            if self.details_intentions[index].is_unissued() {
                self.headroom_details_issue(cancel)?;
            }
        }
        self.source_step(cancel)?;
        for index in 0..self.cancelled_details.len() {
            self.source_step(cancel)?;
            if self.cancelled_details[index].is_unissued() {
                self.headroom_details_issue(cancel)?;
            }
        }
        Ok(())
    }

    pub(in crate::store) fn headroom_style(
        &mut self,
        key: Key,
        kind: Kind,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let Some(mut scan) = self.source_headroom.take() else {
            return Ok(false);
        };
        let result = (|| {
            self.source_step(cancel)?;
            let state = self
                .element(key)?
                .style_state()
                .ok_or(DomError::WrongNodeKind)?;
            if state.availability == crate::DomStyleSheetAvailability::PendingParsedProcessing {
                return Err(DomError::StyleProcessingUnavailable);
            }
            let start = state.next_association;
            let actual_sheet = state.sheet.is_some();
            let row = scan.find(self, key, cancel)?;
            let count = match kind {
                Kind::Update => true,
                Kind::OwnerAttributes | Kind::SetDisabled => {
                    if actual_sheet {
                        true
                    } else if let Some(index) = row {
                        self.source_step(cancel)?;
                        scan.rows[index].creates != 0
                    } else {
                        false
                    }
                }
            };
            if !count {
                return Ok(());
            }
            self.range_publication_ready(cancel)?;
            let index = match row {
                Some(index) => index,
                None => scan.append(self, key, start, cancel)?,
            };
            self.source_step(cancel)?;
            scan.invocations = scan
                .invocations
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            match kind {
                Kind::Update => {
                    scan.phases = scan
                        .phases
                        .checked_add(6)
                        .ok_or(DomError::RepresentationExhausted)?;
                    scan.rows[index].creates = scan.rows[index]
                        .creates
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
                Kind::OwnerAttributes | Kind::SetDisabled => {
                    scan.phases = scan
                        .phases
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
            Ok(())
        })();
        self.source_headroom = Some(scan);
        result.map(|()| true)
    }

    pub(in crate::store) fn headroom_select_issue(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.source_headroom.is_some() {
            self.source_step(cancel)?;
            let scan = self
                .source_headroom
                .as_mut()
                .ok_or(DomError::StyleProtocol)?;
            scan.selects = scan
                .selects
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(())
    }
    pub(in crate::store) fn headroom_details_issue(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.source_headroom.is_some() {
            self.source_step(cancel)?;
            let scan = self
                .source_headroom
                .as_mut()
                .ok_or(DomError::StyleProtocol)?;
            scan.details = scan
                .details
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        dom: Dom,
        document: DocumentId,
        styles: [NodeId; 3],
        select: NodeId,
        details: [NodeId; 2],
    }
    fn limits(bytes: usize) -> crate::DomSourceLimits {
        crate::DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1)
    }
    fn fixture() -> Fixture {
        let mut dom = Dom::new(
            DomLimits::new(32, 16_384, 64, 0, 0, 16, 1_000_000)
                .unwrap()
                .with_select_processing(32, 1_000_000, 1_000_000)
                .unwrap()
                .with_details_toggle_intentions(32)
                .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let styles =
            [0; 3].map(|_| batch.create_element_ns(&document, crate::Namespace::html(), "style"));
        let select = batch.create_element_ns(&document, crate::Namespace::html(), "select");
        let details =
            [0; 2].map(|_| batch.create_element_ns(&document, crate::Namespace::html(), "details"));
        batch.set_attribute(&details[1], "open", "");
        let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        Fixture {
            document: done.document(&document).unwrap(),
            styles: styles.map(|ticket| done.created(&ticket).unwrap().unwrap()),
            select: done.created(&select).unwrap().unwrap(),
            details: details.map(|ticket| done.created(&ticket).unwrap().unwrap()),
            dom,
        }
    }
    fn draft(dom: &Dom, batch: &DomBatch) -> Draft {
        Draft::new_source(
            dom,
            batch.owner.clone(),
            &batch.commands,
            limits(16_000_000),
            &AtomicBool::new(false),
        )
        .unwrap()
        .0
    }
    fn set_association_start(draft: &mut Draft, key: Key, start: u64) {
        let mut state = draft.element(key).unwrap().style_state().unwrap().clone();
        state.next_association = start;
        let DomNodeKind::Element(element) = &mut draft.node_mut(key).kind else {
            unreachable!()
        };
        element.set_style_state(state);
    }

    #[test]
    fn kinds_count_potential_sheet_actions_without_creating_a_sheet_or_invocation() {
        let f = fixture();
        let batch = DomBatch::new();
        let mut draft = draft(&f.dom, &batch);
        let key = Key::Existing(f.styles[0].slot);
        let original_invocation = draft.source_invocation;
        draft.source_headroom = Some(Scan::new());
        assert!(
            draft
                .headroom_style(key, Kind::OwnerAttributes, &AtomicBool::new(false))
                .unwrap()
        );
        assert!(
            draft
                .headroom_style(key, Kind::SetDisabled, &AtomicBool::new(false))
                .unwrap()
        );
        let empty = draft.source_headroom.take().unwrap();
        assert_eq!(
            empty
                .qualify(&draft, &f.dom, 0, &AtomicBool::new(false))
                .unwrap()
                .remaining_phases,
            1
        );
        assert!(empty.rows.is_empty());
        drop(empty);
        draft.source_headroom = Some(Scan::new());
        for kind in [Kind::Update, Kind::OwnerAttributes, Kind::SetDisabled] {
            assert!(
                draft
                    .headroom_style(key, kind, &AtomicBool::new(false))
                    .unwrap()
            );
        }
        let scan = draft.source_headroom.take().unwrap();
        // Six acyclic external Update actions, two direct actions, one close.
        assert_eq!(
            scan.qualify(&draft, &f.dom, 0, &AtomicBool::new(false))
                .unwrap()
                .remaining_phases,
            9
        );
        assert_eq!(draft.source_invocation, original_invocation);
        assert!(
            draft
                .element(key)
                .unwrap()
                .style_state()
                .unwrap()
                .sheet
                .is_none()
        );
        drop(scan);
        assert!(
            !draft
                .headroom_style(key, Kind::Update, &AtomicBool::new(false))
                .unwrap()
        );
    }

    #[test]
    fn row_bank_growth_owns_only_actual_capacity_and_releases_on_drop_and_refusal() {
        let f = fixture();
        let batch = DomBatch::new();
        let mut draft = draft(&f.dom, &batch);
        let cancel = AtomicBool::new(false);
        let live = draft.source_formation_bytes.clone();
        let original = live.load(Ordering::Relaxed);
        draft.source_headroom = Some(Scan::new());
        for node in f.styles {
            draft
                .headroom_style(Key::Existing(node.slot), Kind::Update, &cancel)
                .unwrap();
            let scan = draft.source_headroom.as_ref().unwrap();
            assert_eq!(
                live.load(Ordering::Relaxed),
                original + scan.rows.capacity() * size_of::<Row>()
            );
        }
        drop(draft.source_headroom.take());
        assert_eq!(live.load(Ordering::Relaxed), original);
        let existing =
            draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes + original;
        let key = Key::Existing(f.styles[0].slot);
        draft.source_headroom = Some(Scan::new());
        draft.source_limits = Some(limits(existing + size_of::<Row>() - 1));
        assert!(matches!(
            draft.headroom_style(key, Kind::Update, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert_eq!(draft.source_headroom.as_ref().unwrap().rows.capacity(), 0);
        assert_eq!(live.load(Ordering::Relaxed), original);
        draft.source_limits = Some(limits(existing + size_of::<Row>()));
        draft.headroom_style(key, Kind::Update, &cancel).unwrap();
        assert_eq!(live.load(Ordering::Relaxed), original + size_of::<Row>());
        drop(draft.source_headroom.take());
        assert_eq!(live.load(Ordering::Relaxed), original);
    }

    #[test]
    fn actual_association_invocation_and_phase_counters_have_exact_checked_boundaries() {
        let mut f = fixture();
        f.dom.revision = DomRevision(u64::MAX - 7);
        let batch = DomBatch::new();
        let mut draft = draft(&f.dom, &batch);
        let cancel = AtomicBool::new(false);
        let key = Key::Existing(f.styles[0].slot);
        set_association_start(&mut draft, key, u64::MAX - 1);
        draft.source_invocation = u64::MAX - 1;
        draft.source_headroom = Some(Scan::new());
        draft.headroom_style(key, Kind::Update, &cancel).unwrap();
        let scan = draft.source_headroom.take().unwrap();
        assert_eq!(
            scan.qualify(&draft, &f.dom, 0, &cancel)
                .unwrap()
                .remaining_phases,
            7
        );
        f.dom.revision = DomRevision(u64::MAX - 6);
        assert!(matches!(
            scan.qualify(&draft, &f.dom, 0, &cancel),
            Err(DomError::RevisionExhausted)
        ));
        f.dom.revision = DomRevision(1);
        draft.source_invocation = u64::MAX;
        assert!(matches!(
            scan.qualify(&draft, &f.dom, 0, &cancel),
            Err(DomError::RepresentationExhausted)
        ));
        draft.source_invocation = 0;
        let policy = draft.source_limits.unwrap();
        draft.source_limits = Some(crate::DomSourceLimits::new(
            policy.retained_bytes,
            policy.retained_units,
            policy.work,
            6,
            policy.outputs,
            policy.identities,
            1,
        ));
        assert!(matches!(
            scan.qualify(&draft, &f.dom, 0, &cancel),
            Err(DomError::SourcePublicationLimit)
        ));
        draft.source_limits = Some(limits(16_000_000));
        draft.source_headroom = Some(scan);
        draft.headroom_style(key, Kind::Update, &cancel).unwrap();
        let scan = draft.source_headroom.take().unwrap();
        assert!(
            matches!(
                scan.qualify(&draft, &f.dom, 0, &cancel),
                Err(DomError::RepresentationExhausted)
            ),
            "two potential creations cannot fit the original MAX-1 association counter"
        );
    }

    #[test]
    fn seed_counts_unissued_pending_and_cancellations_once_and_fresh_rows_even_same_target() {
        let mut f = fixture();
        Arc::get_mut(&mut f.dom.state).unwrap().next_task = u64::MAX - 2;
        Arc::get_mut(&mut f.dom.state)
            .unwrap()
            .next_details_intention = u64::MAX - 2;
        let mut batch = DomBatch::new();
        batch.set_attribute(f.details[0], "open", "");
        batch.remove_attribute(f.details[1], "open");
        batch.destroy(f.details[1]);
        let mut draft = draft(&f.dom, &batch);
        let cancel = AtomicBool::new(false);
        for command in &batch.commands {
            draft.apply(command, f.dom.limits, &cancel).unwrap();
        }
        // Counter-component inputs use actual canonical Select identities. Some
        // rows already have an ordinal; only the two explicit None rows issue.
        let target = Key::Existing(f.select.slot);
        draft.native_tasks = vec![
            select::Pending::Update {
                serial: None,
                target,
            },
            select::Pending::Update {
                serial: Some(0),
                target,
            },
            select::Pending::Slots,
        ];
        draft.cancelled_selects = vec![(None, f.select), (Some(1), f.select)];
        draft.source_headroom = Some(Scan::new());
        draft.seed_headroom(&cancel).unwrap();
        let scan = draft.source_headroom.take().unwrap();
        assert_eq!(scan.selects, 2);
        assert_eq!(
            scan.details, 2,
            "actual new pending Details plus actual unissued cancellation"
        );
        assert!(scan.qualify(&draft, &f.dom, 0, &cancel).is_ok());
        draft.source_headroom = Some(scan);
        draft.headroom_select_issue(&cancel).unwrap();
        let mut scan = draft.source_headroom.take().unwrap();
        assert!(matches!(
            scan.qualify(&draft, &f.dom, 0, &cancel),
            Err(DomError::RepresentationExhausted)
        ));
        scan.selects = 2;
        draft.source_headroom = Some(scan);
        draft.headroom_details_issue(&cancel).unwrap();
        let scan = draft.source_headroom.take().unwrap();
        assert!(matches!(
            scan.qualify(&draft, &f.dom, 0, &cancel),
            Err(DomError::RepresentationExhausted)
        ));
    }

    #[test]
    fn real_command_scan_restores_tree_and_payload_but_preserves_spent_work_and_policy() {
        let f = fixture();
        let mut batch = DomBatch::new();
        batch.insert_before(f.styles[0], f.document, None);
        let mut draft = draft(&f.dom, &batch);
        let key = Key::Existing(f.styles[0].slot);
        let cancel = AtomicBool::new(false);
        let work = draft.source_work.load(Ordering::Relaxed);
        let policy = draft.source_limits.unwrap();
        let invocation = draft.source_invocation;
        let live = draft.source_formation_bytes.clone();
        let original_live = live.load(Ordering::Relaxed);
        let admitted = draft
            .admit_source_command_headroom(&f.dom, Entry::Command(&batch.commands[0]), 0, &cancel)
            .unwrap();
        assert_eq!(admitted.remaining_phases, 7);
        assert_eq!(draft.node(key).unwrap().parent, None);
        assert!(draft.effects.is_empty());
        assert!(draft.source_headroom.is_none());
        assert_eq!(draft.source_invocation, invocation);
        assert_eq!(draft.source_limits, Some(policy));
        assert!(draft.source_work.load(Ordering::Relaxed) > work);
        assert_eq!(live.load(Ordering::Relaxed), original_live);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            draft.admit_source_command_headroom(
                &f.dom,
                Entry::Command(&batch.commands[0]),
                0,
                &cancel
            ),
            Err(DomError::Cancelled)
        ));
        assert_eq!(draft.node(key).unwrap().parent, None);
        assert!(draft.source_headroom.is_none());
        assert_eq!(live.load(Ordering::Relaxed), original_live);
    }

    #[test]
    fn native_two_style_insertion_reserves_both_updates_before_revision_exhaustion() {
        use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
        let cancel = AtomicBool::new(false);
        for remaining in [13, 12] {
            let mut f = fixture();
            let mut setup = DomBatch::new();
            let parent = setup.create_element_ns(f.document, crate::Namespace::html(), "div");
            setup.insert_before(&parent, f.document, None);
            let done = f.dom.apply(setup, &cancel).unwrap();
            let parent = done.created(&parent).unwrap().unwrap();
            let mut builder =
                HtmlFragment::builder(HtmlConstructionLimits::new(8, 8, 8, 256, 8)).unwrap();
            for _ in 0..2 {
                builder
                    .append_element(
                        &builder.root(),
                        crate::QualifiedName::new(crate::Namespace::html(), None, "style"),
                        crate::OrderedAttributes::default(),
                    )
                    .unwrap();
            }
            let source = builder.finish();
            f.dom.revision = DomRevision(u64::MAX - remaining);
            let before = f.dom.snapshot();
            let mut staged = f
                .dom
                .stage_native_html_installation(
                    before.document(f.document).unwrap().stamp(),
                    parent,
                    None,
                    &source,
                    installation::Budgets {
                        native: crate::DomNativeHtmlInstallLimits::new(8, 1_000_000, 1_000_000),
                        source: Some(limits(16_000_000)),
                    },
                    &cancel,
                )
                .unwrap();
            let wrapper = staged.continuation.wrapper;
            let parent = staged.continuation.parent;
            let children = staged.draft.node(wrapper).unwrap().children.clone();
            assert_eq!(children.len(), 2);
            let created = staged.draft.created.len();
            let live = staged.draft.source_formation_bytes.load(Ordering::Relaxed);
            let work = staged.draft.source_work.load(Ordering::Relaxed);
            let invocation = staged.draft.source_invocation;
            let result = staged.draft.admit_source_command_headroom(
                &f.dom,
                Entry::Native {
                    wrapper,
                    parent,
                    before: staged.continuation.before,
                },
                0,
                &cancel,
            );
            // Each of the two real connected insertions can require six
            // external Style phases; the mandatory close needs one more.
            if remaining == 13 {
                assert_eq!(result.unwrap().remaining_phases, 13);
            } else {
                assert!(matches!(result, Err(DomError::RevisionExhausted)));
            }
            assert_eq!(staged.draft.node(wrapper).unwrap().children, children);
            assert_eq!(staged.draft.node(wrapper).unwrap().parent, None);
            assert!(staged.draft.node(parent).unwrap().children.is_empty());
            assert_eq!(staged.draft.created.len(), created);
            assert_eq!(staged.draft.source_invocation, invocation);
            assert!(staged.draft.source_headroom.is_none());
            assert!(staged.draft.effects.is_empty());
            assert!(staged.draft.source_work.load(Ordering::Relaxed) > work);
            assert_eq!(
                staged.draft.source_formation_bytes.load(Ordering::Relaxed),
                live
            );
            assert_eq!(f.dom.snapshot().stamp(), before.stamp());
        }
    }

    #[test]
    fn selectedcontent_style_clone_has_fresh_association_headroom_and_checked_revision() {
        let cancel = AtomicBool::new(false);
        for remaining in [7, 6] {
            let mut f = fixture();
            let mut setup = DomBatch::new();
            let option = setup.create_element_ns(f.document, crate::Namespace::html(), "option");
            let selectedcontent =
                setup.create_element_ns(f.document, crate::Namespace::html(), "selectedcontent");
            let text = setup.create_text(f.document, "a {}");
            let previous = setup.create_text(f.document, "previous");
            setup.insert_before(&previous, &selectedcontent, None);
            setup.set_attribute(&option, "selected", "");
            setup.insert_before(&text, f.styles[0], None);
            setup.insert_before(f.styles[0], &option, None);
            setup.insert_before(&option, f.select, None);
            setup.insert_before(&selectedcontent, f.select, None);
            let done = f.dom.apply(setup, &cancel).unwrap();
            let selectedcontent = done.created(&selectedcontent).unwrap().unwrap();
            f.dom.revision = DomRevision(u64::MAX - remaining);
            let batch = DomBatch::new();
            let mut draft = draft(&f.dom, &batch);
            let original = Key::Existing(f.styles[0].slot);
            set_association_start(&mut draft, original, u64::MAX);
            draft.source_invocation = u64::MAX - 1;
            let target = Key::Existing(f.select.slot);
            let selectedcontent = Key::Existing(selectedcontent.slot);
            let children = draft.node(selectedcontent).unwrap().children.clone();
            assert_eq!(
                children.len(),
                1,
                "existing Selectedcontent payload is replaced only in the scan"
            );
            let created = draft.created.len();
            let live = draft.source_formation_bytes.load(Ordering::Relaxed);
            let result =
                draft.admit_source_command_headroom(&f.dom, Entry::Select { target }, 0, &cancel);
            // Cloning the actual Text into a fresh Style triggers one update.
            // Its association occurrence is fresh, even though the source Style
            // has exhausted its own independent occurrence counter.
            if remaining == 7 {
                assert_eq!(result.unwrap().remaining_phases, 7);
            } else {
                assert!(matches!(result, Err(DomError::RevisionExhausted)));
            }
            assert_eq!(draft.node(selectedcontent).unwrap().children, children);
            assert_eq!(draft.created.len(), created);
            assert_eq!(draft.source_invocation, u64::MAX - 1);
            assert_eq!(
                draft
                    .element(original)
                    .unwrap()
                    .style_state()
                    .unwrap()
                    .next_association,
                u64::MAX
            );
            assert!(draft.source_headroom.is_none());
            assert!(draft.effects.is_empty());
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), live);
        }
    }

    #[test]
    fn retired_ranges_preserve_no_live_range_style_headroom() {
        let cancel = AtomicBool::new(false);
        let mut f = fixture();
        f.dom.limits = f.dom.limits.with_ranges(1).unwrap();
        let mut setup = DomBatch::new();
        let first = setup.create_text(f.document, "a {}");
        setup.insert_before(&first, f.styles[0], None);
        let range = setup.create_range(f.document);
        let done = f.dom.apply(setup, &cancel).unwrap();
        let first = done.created(&first).unwrap().unwrap();
        let range = done.created_range(&range).unwrap().unwrap();
        let mut retire = DomBatch::new();
        retire.retire_range(range);
        f.dom.apply(retire, &cancel).unwrap();
        assert_eq!(f.dom.state.ranges.len(), 1);
        assert!(f.dom.state.ranges[0].record.is_none());

        let mut batch = DomBatch::new();
        batch.insert_before(first, f.styles[0], None);
        let mut draft = draft(&f.dom, &batch);
        let result = draft
            .admit_source_command_headroom(&f.dom, Entry::Command(&batch.commands[0]), 0, &cancel)
            .unwrap();
        // Removal and reinsertion each reserve six possible Style phases;
        // the retained vacant slot has no endpoint requiring another policy.
        assert_eq!(result.remaining_phases, 13);
        assert_eq!(
            draft
                .node(Key::Existing(f.styles[0].slot))
                .unwrap()
                .children,
            [Key::Existing(first.slot)]
        );
    }

    #[test]
    fn same_parent_text_move_reserves_both_visits_of_the_original_style() {
        let cancel = AtomicBool::new(false);
        let mut f = fixture();
        let mut setup = DomBatch::new();
        let first = setup.create_text(f.document, "a {}");
        let second = setup.create_text(f.document, "b {}");
        setup.insert_before(&first, f.styles[0], None);
        setup.insert_before(&second, f.styles[0], None);
        let done = f.dom.apply(setup, &cancel).unwrap();
        let first = done.created(&first).unwrap().unwrap();
        let second = done.created(&second).unwrap().unwrap();
        let mut batch = DomBatch::new();
        batch.insert_before(first, f.styles[0], None);
        for available in [2, 1] {
            let mut draft = draft(&f.dom, &batch);
            let style = Key::Existing(f.styles[0].slot);
            set_association_start(&mut draft, style, u64::MAX - available);
            let live = draft.source_formation_bytes.load(Ordering::Relaxed);
            let result = draft.admit_source_command_headroom(
                &f.dom,
                Entry::Command(&batch.commands[0]),
                0,
                &cancel,
            );
            // Actual removal and reinsertion each run the Style children hook.
            // Both potential creations consume the SAME association namespace.
            if available == 2 {
                assert_eq!(result.unwrap().remaining_phases, 13);
            } else {
                assert!(matches!(result, Err(DomError::RepresentationExhausted)));
            }
            assert_eq!(
                draft.node(style).unwrap().children,
                [Key::Existing(first.slot), Key::Existing(second.slot)]
            );
            assert_eq!(
                draft
                    .element(style)
                    .unwrap()
                    .style_state()
                    .unwrap()
                    .next_association,
                u64::MAX - available
            );
            assert_eq!(draft.source_invocation, 0);
            assert!(draft.source_headroom.is_none());
            assert!(draft.effects.is_empty());
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), live);
        }
    }
}

#[cfg(test)]
mod dialog_occurrence_tests {
    use super::*;
    #[test]
    fn source_formed_draft_checks_hook_capacity_without_issuing_or_mutating_state() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(16, 4096, 64, 0, 0, 8, 1_000_000).unwrap()).unwrap();
        let mut create = DomBatch::new();
        let document = create.create_document();
        let dialog = create.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let done = dom.apply(create, &cancel).unwrap();
        let subject = done.created(&dialog).unwrap().unwrap();
        let mut input = DomBatch::new();
        input.set_attribute(subject, "data-counter-proof", "");
        let limits =
            crate::DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 16, 4096, 4096, 1);
        let (mut draft, _, _) =
            Draft::new_source(&dom, input.owner.clone(), &input.commands, limits, &cancel).unwrap();
        let stamp = dom.snapshot().stamp();
        for value in [u64::MAX - 1, u64::MAX] {
            draft.source_invocation = value;
            let result = draft.admit_dialog_occurrences(1, &cancel);
            if value == u64::MAX {
                assert_eq!(result, Err(DomError::RepresentationExhausted));
            } else {
                result.unwrap();
            }
            assert_eq!(draft.source_invocation, value);
            assert_eq!(dom.snapshot().stamp(), stamp);
        }
        draft.source_invocation = 0;
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            draft.admit_dialog_occurrences(1, &cancel),
            Err(DomError::Cancelled)
        );
        assert_eq!(draft.source_invocation, 0);
    }
}
