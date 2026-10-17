//! The original NativeAgent's retained Window and close-watcher managers.
use super::*;
use surgeist_dom::{
    DomCloseWatcherId, DomCloseWatcherRequestPublication, DomDialogWatcherPublication,
    DomHostedRuntimeRetention, DomSnapshot, DomSourceCloseWatcherInvocationId,
    DomSourceCloseWatcherRequest, DomSourceCloseWatcherStep, DomSourceDialogRequest,
    DomSourceLease, NodeId,
};
use surgeist_html::{
    HtmlCloseWatcherManager, HtmlCloseWatcherPlacement, HtmlCloseWatcherRequestAction,
    HtmlCloseWatcherRequestReply, HtmlCloseWatcherState, HtmlDialogAction,
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
    // HTML's initial positive-infinity timestamp is represented by its exact
    // never-activated state, not a finite zero or a sampled clock. The current
    // native input domain admits no trusted activation-triggering HTML input;
    // any future such ingress must update/notify Windows before dispatch.
    activation: NativeWindowActivation,
    manager: HtmlCloseWatcherManager,
    groups: Vec<WatcherGroup>,
    groups_payload: Option<DomHostedRuntimeRetention>,
    custody: Vec<WatcherCustody>,
    custody_payload: Option<DomHostedRuntimeRetention>,
    last_history_action_activation: NativeWindowActivation,
}
#[derive(Clone, Copy)]
enum NativeWindowActivation {
    NeverActivated,
}
struct WatcherGroup {
    members: Vec<Rc<RefCell<Watcher>>>,
    payload: Option<DomHostedRuntimeRetention>,
}
enum WatcherCustody {
    HookRemoval(Rc<RefCell<Watcher>>),
    Invocation(InvocationCustody),
}
impl WatcherCustody {
    fn record(&self) -> &Rc<RefCell<Watcher>> {
        match self {
            Self::HookRemoval(record) => record,
            Self::Invocation(invocation) => &invocation.body.record,
        }
    }
}
struct InvocationCustody {
    // Deallocate the actual boxed body before releasing its relative debit.
    body: Box<WatcherInvocation>,
    _payload: DomHostedRuntimeRetention,
}
struct WatcherInvocation {
    // Only the immutable original method identity belongs in this binding.
    // The actual watcher identity is retained by the original shared record.
    binding: DomSourceCloseWatcherInvocationId,
    record: Rc<RefCell<Watcher>>,
    running_guard: bool,
    pending: Option<PendingWatcherAction>,
}
struct PendingWatcherAction {
    occurrence: u64,
    step: DomSourceCloseWatcherStep,
    publication: Option<DomCloseWatcherRequestPublication>,
}
fn invocation_body_bytes() -> usize {
    size_of::<WatcherInvocation>()
}
struct Watcher {
    id: DomCloseWatcherId,
    request: DomSourceDialogRequest,
    publication: Option<DomDialogWatcherPublication>,
    state: HtmlCloseWatcherState,
    actions: DialogActions,
    phase: WatcherPhase,
    // The actual Rc allocation (body, RefCell and two reference counts),
    // independently of all membership/invocation handle banks.
    _body: DomHostedRuntimeRetention,
}
// Actual closed source actions bound to the exact Dialog generation.
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
            activation: NativeWindowActivation::NeverActivated,
            manager: HtmlCloseWatcherManager::fresh(),
            groups: Vec::new(),
            groups_payload: None,
            custody: Vec::new(),
            custody_payload: None,
            last_history_action_activation: NativeWindowActivation::NeverActivated,
        }
    }
    pub(super) fn transient_activation(&self) -> bool {
        match self.activation {
            NativeWindowActivation::NeverActivated => false,
        }
    }
    fn history_action_activation(&self) -> bool {
        match (self.activation, self.last_history_action_activation) {
            (NativeWindowActivation::NeverActivated, NativeWindowActivation::NeverActivated) => {
                false
            }
        }
    }
    pub(super) fn needed(&self) -> bool {
        !self.groups.is_empty() || !self.custody.is_empty()
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
        bytes = bytes
            .checked_add(
                self.custody
                    .capacity()
                    .checked_mul(size_of::<WatcherCustody>())
                    .ok_or(DomHostedRuntimeError::Admission)?,
            )
            .ok_or(DomHostedRuntimeError::Admission)?;
        for group in &self.groups {
            spend(work, limit, cancel)?;
            admit()?;
            bytes = bytes
                .checked_add(
                    group
                        .members
                        .capacity()
                        .checked_mul(size_of::<Rc<RefCell<Watcher>>>())
                        .ok_or(DomHostedRuntimeError::Admission)?,
                )
                .ok_or(DomHostedRuntimeError::Admission)?;
            for _ in &group.members {
                spend(work, limit, cancel)?;
                admit()?;
                bytes = bytes
                    .checked_add(watcher_body_bytes())
                    .ok_or(DomHostedRuntimeError::Admission)?;
            }
        }
        for (index, custody) in self.custody.iter().enumerate() {
            spend(work, limit, cancel)?;
            admit()?;
            if matches!(custody, WatcherCustody::Invocation(_)) {
                spend(work, limit, cancel)?;
                admit()?;
                bytes = bytes
                    .checked_add(invocation_body_bytes())
                    .ok_or(DomHostedRuntimeError::Admission)?;
            }
            let mut represented = false;
            for group in &self.groups {
                spend(work, limit, cancel)?;
                admit()?;
                for member in &group.members {
                    spend(work, limit, cancel)?;
                    admit()?;
                    represented |= Rc::ptr_eq(member, custody.record());
                }
            }
            for previous in &self.custody[..index] {
                spend(work, limit, cancel)?;
                admit()?;
                represented |= Rc::ptr_eq(previous.record(), custody.record());
            }
            if !represented {
                bytes = bytes
                    .checked_add(watcher_body_bytes())
                    .ok_or(DomHostedRuntimeError::Admission)?;
            }
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
    pub(super) fn find_window(
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
fn watcher_body_bytes() -> usize {
    size_of::<RefCell<Watcher>>() + 2 * size_of::<usize>()
}
#[derive(Clone)]
struct WatcherLocation {
    record: Rc<RefCell<Watcher>>,
}
impl WatcherLocation {
    fn get<'a>(&'a self, _state: &mut HostedState) -> std::cell::RefMut<'a, Watcher> {
        self.record.borrow_mut()
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
        self.accept_watcher(lease, request, &location, cancel)?;
        Ok(true)
    }
    fn accept_watcher(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceDialogRequest,
        location: &WatcherLocation,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let mut watcher = location.get(&mut state);
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
        // A refused affine acceptance retains the exact effect in its actual
        // original record, whether membership still exists or was removed.
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
            for composition in &host.top_levels {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                if let Some(group) = &composition.group {
                    for cluster in &group.clusters {
                        spend(work, limits.work, cancel)?;
                        lease
                            .admit_hosted_runtime_work(1, cancel)
                            .map_err(canonical)?;
                        for window in &cluster.agent.windows.rows {
                            spend(work, limits.work, cancel)?;
                            lease
                                .admit_hosted_runtime_work(1, cancel)
                                .map_err(canonical)?;
                            for group in &window.groups {
                                spend(work, limits.work, cancel)?;
                                lease
                                    .admit_hosted_runtime_work(1, cancel)
                                    .map_err(canonical)?;
                                for record in &group.members {
                                    spend(work, limits.work, cancel)?;
                                    lease
                                        .admit_hosted_runtime_work(1, cancel)
                                        .map_err(canonical)?;
                                    let row = record.borrow();
                                    if row.request == *request
                                        && row.phase != WatcherPhase::Registered
                                    {
                                        return Ok(Some(WatcherLocation {
                                            record: record.clone(),
                                        }));
                                    }
                                }
                            }
                            for custody in &window.custody {
                                spend(work, limits.work, cancel)?;
                                lease
                                    .admit_hosted_runtime_work(1, cancel)
                                    .map_err(canonical)?;
                                if let WatcherCustody::HookRemoval(record) = custody
                                    && record.borrow().request == *request
                                {
                                    return Ok(Some(WatcherLocation {
                                        record: record.clone(),
                                    }));
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
        let record = if request.action() == HtmlDialogAction::EstablishCloseWatcher {
            let new_group = window.manager.establishment(window.groups.len())
                == HtmlCloseWatcherPlacement::NewGroup;
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
            let member_added = grow(
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
            if retained
                .checked_add(added)
                .and_then(|n| n.checked_add(member_added))
                .and_then(|n| n.checked_add(forming))
                .and_then(|n| n.checked_add(watcher_body_bytes()))
                .is_none_or(|n| n > limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            let body = lease
                .retain_hosted_runtime_payload(source, watcher_body_bytes(), cancel)
                .map_err(canonical)?;
            let publication = preparation.publish();
            let record = Rc::new(RefCell::new(Watcher {
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
                _body: body,
            }));
            group.members.push(record.clone());
            if !fresh.members.is_empty() {
                window.groups.push(fresh);
            }
            drop(candidate_payload);
            record
        } else {
            let mut found = None;
            for group in &window.groups {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                for record in &group.members {
                    spend(work, limits.work, cancel)?;
                    lease
                        .admit_hosted_runtime_work(1, cancel)
                        .map_err(canonical)?;
                    if record.borrow().id == id {
                        found = Some(record.clone());
                    }
                }
            }
            // Watcher Close destroys membership before the nested ordinary
            // Close Hook. That Hook still operates on this same original.
            for custody in &window.custody {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                if custody.record().borrow().id == id {
                    found = Some(custody.record().clone());
                }
            }
            let record = found.ok_or(DomHostedRuntimeError::Protocol)?;
            if !record.borrow().actions.matches(id.subject()) {
                return Err(DomHostedRuntimeError::Binding);
            }
            lease
                .admit_hosted_runtime_work(growth_steps(&window.custody)?, cancel)
                .map_err(canonical)?;
            grow(
                &mut window.custody,
                &mut window.custody_payload,
                retained,
                *limits,
                work,
                |bytes| lease.retain_hosted_runtime_payload(source, bytes, cancel),
                cancel,
            )?;
            let movements = membership_removal_steps(window, id)?;
            for _ in 0..movements {
                spend(work, limits.work, cancel)?;
            }
            lease
                .admit_hosted_runtime_work(movements, cancel)
                .map_err(canonical)?;
            // This original-record transition remains real work, separately
            // from the already prepaid membership predicates and moves.
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)?;
            let mut row = record.borrow_mut();
            row.request = request.clone();
            row.publication = Some(preparation.publish());
            row.phase = WatcherPhase::Removing;
            drop(row);
            window
                .custody
                .push(WatcherCustody::HookRemoval(record.clone()));
            remove_membership(window, id);
            record
        };
        Ok(Some(WatcherLocation { record }))
    }

    /// Fulfil one Source-selected watcher effect. The language cursors remain
    /// exclusively in Source; this bank holds only original native records and
    /// affine publications of effects that really happened.
    pub(crate) fn service_close_watcher_request(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: &DomSourceCloseWatcherRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let history = if request.step()
            == DomSourceCloseWatcherStep::Action(
                HtmlCloseWatcherRequestAction::ConsumeHistoryActionActivation,
            ) {
            Some(
                lease
                    .close_watcher_history_windows(request, &state.source, cancel)
                    .map_err(canonical)?,
            )
        } else {
            None
        };
        if state.dom != lease.snapshot().owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        let preparation = lease
            .prepare_close_watcher_request(request, &state.source, cancel)
            .map_err(canonical)?;
        let id = request.watcher();
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
        if let Some(capture) = &history {
            // The Source-owned capture is the actual W.navigable/top/active
            // Window closure. NeverActivated does not bypass this formation.
            if capture.window_navigable().is_none() && !capture.windows().is_empty() {
                return Err(DomHostedRuntimeError::Protocol);
            }
            // Resolve every real Window and prepay the second read-only bank
            // walk and write before any timestamp changes. Both walks see the
            // exact same callback-free native banks.
            for global in capture.windows() {
                let mut second_walk = 0usize;
                state.find_window(*global, cancel, || {
                    second_walk = second_walk
                        .checked_add(1)
                        .ok_or(DomHostedRuntimeError::Admission)?;
                    lease
                        .admit_hosted_runtime_work(1, cancel)
                        .map_err(canonical)
                })?;
                // find_window has charged the actual validation walk. The
                // callback-free assignment below traverses that same prefix
                // again, then performs one timestamp write. Admit both before
                // any Window timestamp changes.
                let assignment = second_walk
                    .checked_add(1)
                    .ok_or(DomHostedRuntimeError::Admission)?;
                for _ in 0..assignment {
                    state.step(cancel)?;
                }
                lease
                    .admit_hosted_runtime_work(assignment, cancel)
                    .map_err(canonical)?;
            }
        }
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
        let mut index = None;
        let mut original = None;
        for (position, custody) in window.custody.iter().enumerate() {
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)?;
            if let WatcherCustody::Invocation(invocation) = custody
                && invocation.body.binding == request.invocation()
            {
                if invocation.body.record.borrow().id != id
                    || invocation.body.binding.operation() != request.operation()
                {
                    return Err(DomHostedRuntimeError::Binding);
                }
                index = Some(position);
            }
            if custody.record().borrow().id == id {
                original = Some(custody.record().clone());
            }
        }
        for group in &window.groups {
            spend(work, limits.work, cancel)?;
            lease
                .admit_hosted_runtime_work(1, cancel)
                .map_err(canonical)?;
            for record in &group.members {
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                if record.borrow().id == id {
                    original = Some(record.clone());
                }
            }
        }
        let index = match index {
            Some(index) => index,
            None => {
                let record = original.ok_or(DomHostedRuntimeError::Binding)?;
                if !record.borrow().actions.matches(id.subject()) {
                    return Err(DomHostedRuntimeError::Binding);
                }
                lease
                    .admit_hosted_runtime_work(growth_steps(&window.custody)?, cancel)
                    .map_err(canonical)?;
                let added = grow(
                    &mut window.custody,
                    &mut window.custody_payload,
                    retained,
                    *limits,
                    work,
                    |bytes| lease.retain_hosted_runtime_payload(source, bytes, cancel),
                    cancel,
                )?;
                let body_bytes = invocation_body_bytes();
                if body_bytes > limits.scratch_bytes
                    || retained
                        .checked_add(added)
                        .and_then(|bytes| bytes.checked_add(body_bytes))
                        .is_none_or(|bytes| bytes > limits.storage_bytes)
                {
                    return Err(DomHostedRuntimeError::Admission);
                }
                spend(work, limits.work, cancel)?;
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(canonical)?;
                let payload = lease
                    .retain_hosted_runtime_payload(source, body_bytes, cancel)
                    .map_err(canonical)?;
                let body = Box::new(WatcherInvocation {
                    binding: request.invocation(),
                    record,
                    running_guard: false,
                    pending: None,
                });
                let index = window.custody.len();
                window
                    .custody
                    .push(WatcherCustody::Invocation(InvocationCustody {
                        body,
                        _payload: payload,
                    }));
                index
            }
        };
        if request.step() == DomSourceCloseWatcherStep::SettleInvocation {
            let steps = window
                .custody
                .len()
                .checked_sub(index)
                .ok_or(DomHostedRuntimeError::Protocol)?;
            for _ in 0..steps {
                spend(work, limits.work, cancel)?;
            }
            lease
                .admit_hosted_runtime_work(steps, cancel)
                .map_err(canonical)?;
        }
        let WatcherCustody::Invocation(custody) = &mut window.custody[index] else {
            return Err(DomHostedRuntimeError::Protocol);
        };
        let invocation = &mut custody.body;
        if let Some(pending) = &mut invocation.pending {
            if pending.occurrence != request.current_occurrence() || pending.step != request.step()
            {
                return Err(DomHostedRuntimeError::Protocol);
            }
            lease
                .accept_close_watcher_request(request, &mut pending.publication, cancel)
                .map_err(canonical)?;
            invocation.pending = None;
        } else {
            let record = invocation.record.clone();
            let owns_running_guard = invocation.running_guard;
            let publication = match request.step() {
                DomSourceCloseWatcherStep::SettleInvocation => {
                    if owns_running_guard {
                        return Err(DomHostedRuntimeError::Protocol);
                    }
                    preparation.publish_settled().map_err(canonical)?
                }
                DomSourceCloseWatcherStep::Action(action) => {
                    use HtmlCloseWatcherRequestAction as A;
                    use HtmlCloseWatcherRequestReply as R;
                    let reply = match action {
                        A::ReadActive => {
                            let mut active = false;
                            for group in &window.groups {
                                spend(work, limits.work, cancel)?;
                                lease
                                    .admit_hosted_runtime_work(1, cancel)
                                    .map_err(canonical)?;
                                for member in &group.members {
                                    spend(work, limits.work, cancel)?;
                                    lease
                                        .admit_hosted_runtime_work(1, cancel)
                                        .map_err(canonical)?;
                                    active |= Rc::ptr_eq(member, &record);
                                }
                            }
                            R::Active(active)
                        }
                        A::ReadEnabled => R::Enabled(
                            lease
                                .close_watcher_enabled(request, source, cancel)
                                .map_err(canonical)?,
                        ),
                        A::ReadRunningCancelAction => {
                            R::RunningCancelAction(record.borrow().state.running_cancel_action())
                        }
                        A::ReadWindowFullyActive => R::WindowFullyActive(
                            lease
                                .close_watcher_window_fully_active(request, source, cancel)
                                .map_err(canonical)?,
                        ),
                        A::ReadManagerGroups => R::ManagerGroups {
                            groups: window.groups.len(),
                            allowed_groups: std::num::NonZeroUsize::new(
                                window.manager.allowed_groups(),
                            )
                            .ok_or(DomHostedRuntimeError::Protocol)?,
                        },
                        A::ReadHistoryActionActivation => {
                            R::HistoryActionActivation(window.history_action_activation())
                        }
                        A::SetRunningCancelAction(running) => {
                            spend(work, limits.work, cancel)?;
                            lease
                                .admit_hosted_runtime_work(1, cancel)
                                .map_err(canonical)?;
                            if running == owns_running_guard {
                                return Err(DomHostedRuntimeError::Protocol);
                            }
                            let mut row = record.borrow_mut();
                            if running {
                                row.state.begin_cancel_action()
                            } else {
                                row.state.finish_cancel_action()
                            }
                            .map_err(|_| DomHostedRuntimeError::Protocol)?;
                            R::Applied
                        }
                        A::DestroyMembership => {
                            let steps = membership_removal_steps(window, id)?;
                            for _ in 0..steps {
                                spend(work, limits.work, cancel)?;
                            }
                            lease
                                .admit_hosted_runtime_work(steps, cancel)
                                .map_err(canonical)?;
                            remove_membership(window, id);
                            R::Applied
                        }
                        A::ConsumeHistoryActionActivation => R::Applied,
                        A::CaptureWindow
                        | A::RunCancelAction { .. }
                        | A::RunCloseAction
                        | A::Complete(_) => {
                            return Err(DomHostedRuntimeError::Protocol);
                        }
                    };
                    preparation.publish_reply(reply).map_err(canonical)?
                }
            };
            let WatcherCustody::Invocation(custody) = &mut window.custody[index] else {
                return Err(DomHostedRuntimeError::Protocol);
            };
            let invocation = &mut custody.body;
            if let DomSourceCloseWatcherStep::Action(
                HtmlCloseWatcherRequestAction::SetRunningCancelAction(running),
            ) = request.step()
            {
                invocation.running_guard = running;
            }
            invocation.pending = Some(PendingWatcherAction {
                occurrence: request.current_occurrence(),
                step: request.step(),
                publication: Some(publication),
            });
            if let Some(capture) = &history {
                for global in capture.windows() {
                    'find: for host in hosts.iter_mut() {
                        for composition in &mut host.top_levels {
                            if let Some(group) = &mut composition.group {
                                for cluster in &mut group.clusters {
                                    for target in &mut cluster.agent.windows.rows {
                                        if target.id == *global {
                                            target.last_history_action_activation =
                                                target.activation;
                                            break 'find;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let window = &mut hosts[h].top_levels[t]
                .group
                .as_mut()
                .expect("pinned original group")
                .clusters[c]
                .agent
                .windows
                .rows[w];
            let WatcherCustody::Invocation(custody) = &mut window.custody[index] else {
                return Err(DomHostedRuntimeError::Protocol);
            };
            let invocation = &mut custody.body;
            let publication = &mut invocation
                .pending
                .as_mut()
                .expect("retained actual effect")
                .publication;
            lease
                .accept_close_watcher_request(request, publication, cancel)
                .map_err(canonical)?;
            invocation.pending = None;
        }
        let window = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .expect("pinned original group")
            .clusters[c]
            .agent
            .windows
            .rows[w];
        if request.step() == DomSourceCloseWatcherStep::SettleInvocation {
            // The exact row splice was prepaid before affine acceptance.
            // No callback, allocation or fallible cleanup occurs here.
            window.custody.remove(index);
        }
        Ok(())
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
                            steps = steps
                                .checked_add(
                                    window
                                        .custody
                                        .len()
                                        .checked_mul(2)
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
fn published_slot(
    snapshot: &DomSnapshot,
    id: DomCloseWatcherId,
) -> Option<Option<DomCloseWatcherId>> {
    match snapshot.node(id.subject()) {
        Ok(node) => node.as_dialog().map(|dialog| dialog.close_watcher()),
        Err(surgeist_dom::DomError::StaleNode) => Some(None),
        Err(_) => None,
    }
}
fn membership_removal_steps(
    window: &NativeWindow,
    _id: DomCloseWatcherId,
) -> Result<usize, DomHostedRuntimeError> {
    let mut steps = window
        .groups
        .len()
        .checked_mul(2)
        .ok_or(DomHostedRuntimeError::Admission)?;
    for group in &window.groups {
        steps = steps
            .checked_add(
                group
                    .members
                    .len()
                    .checked_mul(2)
                    .ok_or(DomHostedRuntimeError::Admission)?,
            )
            .ok_or(DomHostedRuntimeError::Admission)?;
        // Every possible group splice is included above, independently of
        // whether membership had already been removed by watcher Close.
    }
    Ok(steps)
}
fn remove_membership(window: &mut NativeWindow, id: DomCloseWatcherId) {
    for group in &mut window.groups {
        group.members.retain(|record| record.borrow().id != id);
    }
    window.groups.retain(|group| !group.members.is_empty());
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
                                for record in &group.members {
                                    let mut watcher = record.borrow_mut();
                                    if watcher.phase == WatcherPhase::Registering
                                        && watcher.publication.is_none()
                                        && published_slot(snapshot, watcher.id)
                                            == Some(Some(watcher.id))
                                    {
                                        watcher.phase = WatcherPhase::Registered;
                                    }
                                }
                            }
                            window.custody.retain(|custody| match custody {
                                WatcherCustody::Invocation(_) => true,
                                WatcherCustody::HookRemoval(record) => {
                                    let watcher = record.borrow();
                                    watcher.publication.is_some()
                                        || !published_slot(snapshot, watcher.id)
                                            .is_some_and(|slot| slot != Some(watcher.id))
                                }
                            });
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
