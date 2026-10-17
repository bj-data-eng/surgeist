//! The original NativeAgent's retained Window and close-watcher managers.
use super::*;
use surgeist_dom::{
    DomCloseWatcherId, DomDialogWatcherPublication, DomHostedRuntimeRetention, DomSnapshot,
    DomSourceDialogRequest, DomSourceLease, NodeId,
};
use surgeist_html::{
    HtmlCloseWatcherManager, HtmlCloseWatcherPlacement, HtmlCloseWatcherState, HtmlDialogAction,
};

pub(super) struct WindowBank {
    pub rows: Vec<NativeWindow>,
    payload: Option<DomHostedRuntimeRetention>,
    // One fixed inline owner header, independent of backing replacement.
    _header: DomHostedRuntimeRetention,
}
impl WindowBank {
    pub(super) fn new(header: DomHostedRuntimeRetention) -> Self {
        Self {
            rows: Vec::new(),
            payload: None,
            _header: header,
        }
    }
    pub(super) const fn header_bytes() -> usize {
        size_of::<Self>()
    }
}
pub(super) struct NativeWindow {
    pub id: DomHostedGlobalId,
    pub retired: bool,
    manager: HtmlCloseWatcherManager,
    groups: Vec<WatcherGroup>,
    groups_payload: Option<DomHostedRuntimeRetention>,
}
struct WatcherGroup {
    members: Vec<Watcher>,
    payload: Option<DomHostedRuntimeRetention>,
}
struct Watcher {
    id: DomCloseWatcherId,
    request: DomSourceDialogRequest,
    publication: Option<DomDialogWatcherPublication>,
    state: HtmlCloseWatcherState,
    actions: DialogActions,
    phase: WatcherPhase,
}
// Actual closed source actions bound to the exact Dialog generation. No
// invocation entry point or successful callback is installed by this unit.
struct DialogActions {
    cancel_event: DialogCancelAction,
    close: DialogCloseAction,
    enabled: DialogEnabledRead,
}
enum DialogCancelAction {
    DispatchCancel { subject: NodeId },
}
enum DialogCloseAction {
    Close { subject: NodeId },
}
// A later invocation reads current force/closedby/modal and consumes HTML's
// dialog_close_watcher_enabled; this descriptor caches no enabled decision.
enum DialogEnabledRead {
    Dialog { subject: NodeId },
}
impl DialogActions {
    fn matches(&self, target: NodeId) -> bool {
        matches!(self.cancel_event, DialogCancelAction::DispatchCancel { subject } if subject == target)
            && matches!(self.close, DialogCloseAction::Close { subject } if subject == target)
            && matches!(self.enabled, DialogEnabledRead::Dialog { subject } if subject == target)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WatcherPhase {
    Registering,
    Registered,
    Removing,
}
impl NativeWindow {
    pub(super) fn fresh(id: DomHostedGlobalId) -> Self {
        Self {
            id,
            retired: false,
            manager: HtmlCloseWatcherManager::fresh(),
            groups: Vec::new(),
            groups_payload: None,
        }
    }
    pub(super) fn needed(&self) -> bool {
        !self.groups.is_empty()
    }
    pub(super) fn retained_bytes(
        &self,
        work: &mut u64,
        limit: u64,
        cancel: &AtomicBool,
        admit: &mut impl FnMut() -> Result<(), DomHostedRuntimeError>,
    ) -> Result<usize, DomHostedRuntimeError> {
        let mut bytes = self
            .groups
            .capacity()
            .checked_mul(size_of::<WatcherGroup>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        for group in &self.groups {
            spend(work, limit, cancel)?;
            admit()?;
            bytes = bytes
                .checked_add(
                    group
                        .members
                        .capacity()
                        .checked_mul(size_of::<Watcher>())
                        .ok_or(DomHostedRuntimeError::Admission)?,
                )
                .ok_or(DomHostedRuntimeError::Admission)?;
        }
        Ok(bytes)
    }
}
fn canonical(error: surgeist_dom::DomError) -> DomHostedRuntimeError {
    DomHostedRuntimeError::Dom(DomHostedError::Dom(error))
}

// The entry visit always occurs; movement occurs only for a full old bank.
// Length/capacity are scalar reads, so this admission itself performs no scan.
pub(super) fn growth_steps<T>(bank: &Vec<T>) -> Result<usize, DomHostedRuntimeError> {
    1usize
        .checked_add(if bank.len() == bank.capacity() {
            bank.len()
        } else {
            0
        })
        .ok_or(DomHostedRuntimeError::Admission)
}

// Both old and full candidate banks remain charged until the physical move.
fn grow<T>(
    bank: &mut Vec<T>,
    payload: &mut Option<DomHostedRuntimeRetention>,
    retained: usize,
    limits: DomHostedRuntimeLimits,
    work: &mut u64,
    mut retain: impl FnMut(usize) -> Result<DomHostedRuntimeRetention, surgeist_dom::DomError>,
    cancel: &AtomicBool,
) -> Result<usize, DomHostedRuntimeError> {
    spend(work, limits.work, cancel)?;
    if bank.len() < bank.capacity() {
        return Ok(0);
    }
    let count = bank
        .len()
        .checked_add(1)
        .ok_or(DomHostedRuntimeError::Admission)?;
    let requested = count
        .checked_mul(size_of::<T>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    let check = |bytes| {
        if bytes > limits.scratch_bytes
            || retained
                .checked_add(bytes)
                .is_none_or(|n| n > limits.storage_bytes)
        {
            Err(DomHostedRuntimeError::Admission)
        } else {
            Ok(())
        }
    };
    check(requested)?;
    let mut charge = retain(requested).map_err(canonical)?;
    for _ in 0..bank.len() {
        spend(work, limits.work, cancel)?;
    }
    let mut next = Vec::new();
    next.try_reserve_exact(count)
        .map_err(|_| DomHostedRuntimeError::Admission)?;
    let actual = next
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    check(actual)?;
    if actual != requested {
        charge
            .absorb(retain(actual - requested).map_err(canonical)?)
            .map_err(canonical)?;
    }
    let old = bank
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    next.append(bank);
    *bank = next;
    *payload = Some(charge);
    actual
        .checked_sub(old)
        .ok_or(DomHostedRuntimeError::Admission)
}
pub(super) fn reserve_window(
    agent: &mut records::NativeAgent,
    retained: usize,
    limits: DomHostedRuntimeLimits,
    work: &mut u64,
    retain: impl FnMut(usize) -> Result<DomHostedRuntimeRetention, surgeist_dom::DomError>,
    cancel: &AtomicBool,
) -> Result<usize, DomHostedRuntimeError> {
    grow(
        &mut agent.windows.rows,
        &mut agent.windows.payload,
        retained,
        limits,
        work,
        retain,
        cancel,
    )
}
impl HostedState {
    fn find_window(
        &mut self,
        id: DomHostedGlobalId,
        cancel: &AtomicBool,
        mut admit: impl FnMut() -> Result<(), DomHostedRuntimeError>,
    ) -> Result<(usize, usize, usize, usize), DomHostedRuntimeError> {
        for (h, host) in self.hosts.iter().enumerate() {
            spend(&mut self.work, self.limits.work, cancel)?;
            admit()?;
            for (t, composition) in host.top_levels.iter().enumerate() {
                spend(&mut self.work, self.limits.work, cancel)?;
                admit()?;
                if let Some(group) = &composition.group {
                    for (c, cluster) in group.clusters.iter().enumerate() {
                        spend(&mut self.work, self.limits.work, cancel)?;
                        admit()?;
                        for (w, window) in cluster.agent.windows.rows.iter().enumerate() {
                            spend(&mut self.work, self.limits.work, cancel)?;
                            admit()?;
                            if window.id == id {
                                return Ok((h, t, c, w));
                            }
                        }
                    }
                }
            }
        }
        Err(DomHostedRuntimeError::Binding)
    }
}
#[derive(Clone, Copy)]
struct WatcherLocation {
    host: usize,
    composition: usize,
    cluster: usize,
    window: usize,
    group: usize,
    member: usize,
}
impl WatcherLocation {
    fn get(self, state: &mut HostedState) -> &mut Watcher {
        &mut state.hosts[self.host].top_levels[self.composition]
            .group
            .as_mut()
            .expect("pinned group")
            .clusters[self.cluster]
            .agent
            .windows
            .rows[self.window]
            .groups[self.group]
            .members[self.member]
    }
}
impl HostedRuntime {
    pub(crate) fn service_dialog_watcher(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedRuntimeError> {
        let Some(location) = self.install_dialog_watcher(lease, request, cancel)? else {
            return Ok(false);
        };
        self.accept_watcher(lease, request, location, cancel)?;
        Ok(true)
    }
    fn accept_watcher(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceDialogRequest,
        location: WatcherLocation,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let watcher = location.get(&mut state);
        lease
            .accept_dialog_watcher(request, &mut watcher.publication, cancel)
            .map_err(canonical)
    }
    fn install_dialog_watcher(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<Option<WatcherLocation>, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        let Some(preparation) = lease
            .prepare_dialog_watcher(request, &state.source, cancel)
            .map_err(canonical)?
        else {
            return Ok(None);
        };
        // Retry actual effects before preparing another registration/removal.
        let HostedState {
            hosts,
            work,
            limits,
            ..
        } = &mut *state;
        for (h, host) in hosts.iter_mut().enumerate() {
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)?;
            for (t, composition) in host.top_levels.iter_mut().enumerate() {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                if let Some(group) = &mut composition.group {
                    for (c, cluster) in group.clusters.iter_mut().enumerate() {
                        spend(work, limits.work, cancel)?;
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(canonical)?;
                        for (w, window) in cluster.agent.windows.rows.iter_mut().enumerate() {
                            spend(work, limits.work, cancel)?;
                            lease
                                .admit_hosted_runtime_work(1, cancel)
                                .map_err(canonical)?;
                            for (g, group) in window.groups.iter_mut().enumerate() {
                                spend(work, limits.work, cancel)?;
                                lease
                                    .admit_hosted_runtime_work(1, cancel)
                                    .map_err(canonical)?;
                                for (m, watcher) in group.members.iter_mut().enumerate() {
                                    spend(work, limits.work, cancel)?;
                                    lease
                                        .admit_hosted_runtime_work(1, cancel)
                                        .map_err(canonical)?;
                                    if watcher.request == *request
                                        && watcher.phase != WatcherPhase::Registered
                                    {
                                        return Ok(Some(WatcherLocation {
                                            host: h,
                                            composition: t,
                                            cluster: c,
                                            window: w,
                                            group: g,
                                            member: m,
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let id = preparation.watcher();
        let (h, t, c, w) = state.find_window(id.global(), cancel, || {
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)
        })?;
        let HostedState {
            hosts,
            children,
            destruction,
            work,
            limits,
            ..
        } = &mut *state;
        let retained = native_retained_bytes(
            hosts,
            children,
            destruction,
            work,
            limits.work,
            cancel,
            || {
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)
            },
        )?;
        let HostedState {
            hosts,
            source,
            work,
            limits,
            ..
        } = &mut *state;
        let window = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c]
            .agent
            .windows
            .rows[w];
        let (group_index, member_index) =
            if request.action() == HtmlDialogAction::EstablishCloseWatcher {
                let new_group = window.manager.establishment(window.groups.len())
                    == HtmlCloseWatcherPlacement::NewGroup;
                let group_index = if new_group {
                    window.groups.len()
                } else {
                    window.groups.len() - 1
                };
                let added = if new_group {
                    lease
                        .admit_hosted_runtime_work(growth_steps(&window.groups)?, cancel)
                        .map_err(canonical)?;
                    grow(
                        &mut window.groups,
                        &mut window.groups_payload,
                        retained,
                        *limits,
                        work,
                        |bytes| lease.retain_hosted_runtime_payload(source, bytes, cancel),
                        cancel,
                    )?
                } else {
                    0
                };
                let forming = size_of::<WatcherGroup>();
                if retained
                    .checked_add(added)
                    .and_then(|n| n.checked_add(forming))
                    .is_none_or(|n| n > limits.storage_bytes)
                {
                    return Err(DomHostedRuntimeError::Admission);
                }
                let candidate_payload = lease
                    .retain_hosted_runtime_payload(source, forming, cancel)
                    .map_err(canonical)?;
                let mut fresh = WatcherGroup {
                    members: Vec::new(),
                    payload: None,
                };
                let group = if new_group {
                    &mut fresh
                } else {
                    window
                        .groups
                        .last_mut()
                        .ok_or(DomHostedRuntimeError::Protocol)?
                };
                lease
                    .admit_hosted_runtime_work(growth_steps(&group.members)?, cancel)
                    .map_err(canonical)?;
                grow(
                    &mut group.members,
                    &mut group.payload,
                    retained
                        .checked_add(added)
                        .and_then(|n| n.checked_add(forming))
                        .ok_or(DomHostedRuntimeError::Admission)?,
                    *limits,
                    work,
                    |bytes| lease.retain_hosted_runtime_payload(source, bytes, cancel),
                    cancel,
                )?;
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                let member_index = group.members.len();
                let publication = preparation.publish();
                group.members.push(Watcher {
                    id,
                    request: request.clone(),
                    publication: Some(publication),
                    state: HtmlCloseWatcherState::fresh(),
                    actions: DialogActions {
                        cancel_event: DialogCancelAction::DispatchCancel {
                            subject: id.subject(),
                        },
                        close: DialogCloseAction::Close {
                            subject: id.subject(),
                        },
                        enabled: DialogEnabledRead::Dialog {
                            subject: id.subject(),
                        },
                    },
                    phase: WatcherPhase::Registering,
                });
                if !fresh.members.is_empty() {
                    window.groups.push(fresh);
                }
                drop(candidate_payload);
                (group_index, member_index)
            } else {
                let mut found = None;
                for (g, group) in window.groups.iter().enumerate() {
                    spend(work, limits.work, cancel)?;
                    lease
                        .admit_hosted_runtime_work(1, cancel)
                        .map_err(canonical)?;
                    for (m, watcher) in group.members.iter().enumerate() {
                        spend(work, limits.work, cancel)?;
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(canonical)?;
                        if watcher.id == id && watcher.phase == WatcherPhase::Registered {
                            found = Some((g, m));
                        }
                    }
                }
                let (g, m) = found.ok_or(DomHostedRuntimeError::Protocol)?;
                spend(work, limits.work, cancel)?;
                let watcher = &mut window.groups[g].members[m];
                if watcher.state.running_cancel_action() || !watcher.actions.matches(id.subject()) {
                    return Err(DomHostedRuntimeError::Binding);
                }
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                watcher.request = request.clone();
                watcher.publication = Some(preparation.publish());
                watcher.phase = WatcherPhase::Removing;
                (g, m)
            };
        Ok(Some(WatcherLocation {
            host: h,
            composition: t,
            cluster: c,
            window: w,
            group: group_index,
            member: member_index,
        }))
    }

    /// Pin and prepay real scans/readback/splices before advance can close child.
    pub(crate) fn prepare_watcher_settlement(
        &self,
        lease: &DomSourceLease<'_>,
        cancel: &AtomicBool,
    ) -> Result<WatcherSettlement, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        let mut steps = 0usize;
        // Upper bound every actual scan and removal movement using the current
        // immutable lengths. No callbacks can append records during advance.
        let HostedState {
            hosts,
            work,
            limits,
            ..
        } = &mut *state;
        for host in hosts.iter() {
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)?;
            steps = steps
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            for composition in &host.top_levels {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                steps = steps
                    .checked_add(1)
                    .ok_or(DomHostedRuntimeError::Admission)?;
                if let Some(group) = &composition.group {
                    for cluster in &group.clusters {
                        spend(work, limits.work, cancel)?;
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(canonical)?;
                        steps = steps
                            .checked_add(1)
                            .and_then(|n| {
                                cluster
                                    .agent
                                    .windows
                                    .rows
                                    .len()
                                    .checked_mul(3)
                                    .and_then(|rows| n.checked_add(rows))
                            })
                            .ok_or(DomHostedRuntimeError::Admission)?;
                        for window in &cluster.agent.windows.rows {
                            spend(work, limits.work, cancel)?;
                            lease
                                .admit_hosted_runtime_work(1, cancel)
                                .map_err(canonical)?;
                            steps = steps
                                .checked_add(
                                    window
                                        .groups
                                        .len()
                                        .checked_mul(3)
                                        .ok_or(DomHostedRuntimeError::Admission)?,
                                )
                                .ok_or(DomHostedRuntimeError::Admission)?;
                            for group in &window.groups {
                                spend(work, limits.work, cancel)?;
                                lease
                                    .admit_hosted_runtime_work(1, cancel)
                                    .map_err(canonical)?;
                                steps = steps
                                    .checked_add(
                                        group
                                            .members
                                            .len()
                                            .checked_mul(2)
                                            .ok_or(DomHostedRuntimeError::Admission)?,
                                    )
                                    .ok_or(DomHostedRuntimeError::Admission)?;
                            }
                        }
                    }
                }
            }
        }
        // The second pass predicates every composition and may scan its real
        // clusters and remove it. Tail-row moves use original positions: earlier
        // removals do not increase any later original tail extent.
        for host in hosts.iter() {
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)?;
            steps = steps
                .checked_add(1)
                .and_then(|n| n.checked_add(host.top_levels.len()))
                .ok_or(DomHostedRuntimeError::Admission)?;
            for (index, composition) in host.top_levels.iter().enumerate() {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                if composition.top.is_none() {
                    let clusters = composition
                        .group
                        .as_ref()
                        .ok_or(DomHostedRuntimeError::Protocol)?
                        .clusters
                        .len();
                    steps = steps
                        .checked_add(clusters)
                        .and_then(|n| n.checked_add(host.top_levels.len() - index - 1))
                        .ok_or(DomHostedRuntimeError::Admission)?;
                }
            }
        }
        for _ in 0..steps {
            state.step(cancel)?;
        }
        lease
            .admit_hosted_runtime_work(steps, cancel)
            .map_err(canonical)?;
        drop(state);
        Ok(WatcherSettlement { owner })
    }
}
pub(crate) struct WatcherSettlement {
    owner: Rc<RefCell<HostedState>>,
}
impl WatcherSettlement {
    pub(crate) fn finish(self, snapshot: &DomSnapshot) {
        let mut state = self.owner.borrow_mut();
        // Exclusive Source advance contains no user callback and cannot change
        // native bank locations. All bounded readback/release was preadmitted.
        for host in &mut state.hosts {
            for composition in &mut host.top_levels {
                if let Some(group) = &mut composition.group {
                    for cluster in &mut group.clusters {
                        let agent = &mut cluster.agent;
                        for window in &mut agent.windows.rows {
                            for group in &mut window.groups {
                                group.members.retain_mut(|watcher| {
                                    if watcher.phase == WatcherPhase::Registered
                                        || watcher.publication.is_some()
                                    {
                                        return true;
                                    }
                                    debug_assert!(!watcher.state.running_cancel_action());
                                    debug_assert!(watcher.actions.matches(watcher.id.subject()));
                                    let slot = match snapshot.node(watcher.id.subject()) {
                                        Ok(node) => {
                                            node.as_dialog().map(|dialog| dialog.close_watcher())
                                        }
                                        Err(surgeist_dom::DomError::StaleNode) => Some(None),
                                        Err(_) => None,
                                    };
                                    match watcher.phase {
                                        WatcherPhase::Registering
                                            if slot == Some(Some(watcher.id)) =>
                                        {
                                            watcher.phase = WatcherPhase::Registered;
                                            true
                                        }
                                        WatcherPhase::Removing
                                            if slot
                                                .is_some_and(|slot| slot != Some(watcher.id)) =>
                                        {
                                            false
                                        }
                                        _ => true,
                                    }
                                });
                            }
                            window.groups.retain(|group| !group.members.is_empty());
                        }
                        agent
                            .windows
                            .rows
                            .retain(|window| window.needed() || !window.retired);
                    }
                }
            }
        }
        for host in &mut state.hosts {
            let mut index = 0;
            while index < host.top_levels.len() {
                if !destruction::prune_closed_composition(host, index) {
                    index += 1;
                }
            }
        }
    }
}

#[cfg(all(
    test,
    any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )
))]
mod tests;
