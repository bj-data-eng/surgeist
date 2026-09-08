use super::{
    Clipboard, Command, Error, ErrorCode, Handle, HostCapabilities, Id, Metrics, Open, Proxy, Ref,
    Registry, Selector, Target, command::Action,
};
use std::time::Instant;

/// Callback-local access to the live registry, host services, and queued work.
///
/// A context is borrowed only for the duration of one handler callback. Commands
/// and actions requested through it are committed after that callback returns
/// successfully; returning an error discards that queued work and makes the
/// first terminal failure observable from the loop. Clipboard I/O is immediate
/// and is therefore not rolled back with queued work.
pub struct Context<'a> {
    pub(crate) registry: &'a mut Registry,
    pub(crate) commands: &'a mut Vec<Command>,
    pub(crate) actions: &'a mut Vec<Action>,
    clipboard: &'a mut dyn Clipboard,
    capabilities: &'a HostCapabilities,
    action: Action,
    proxy: Option<Proxy>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        registry: &'a mut Registry,
        commands: &'a mut Vec<Command>,
        actions: &'a mut Vec<Action>,
        clipboard: &'a mut dyn Clipboard,
        capabilities: &'a HostCapabilities,
        proxy: Option<Proxy>,
    ) -> Self {
        let action = resolve_actions(actions);
        Self {
            registry,
            commands,
            actions,
            clipboard,
            capabilities,
            action,
            proxy,
        }
    }

    /// Returns the live registry as observed during this callback.
    ///
    /// The returned registry exposes current live entries but cannot be mutated
    /// directly; use this context's command methods to request work.
    pub fn registry(&self) -> &Registry {
        self.registry
    }

    /// Returns the immutable report resolved for the current host target.
    #[must_use]
    pub fn capabilities(&self) -> &HostCapabilities {
        self.capabilities
    }

    /// Returns the loop-owned clipboard borrowed for this callback.
    ///
    /// Clipboard operations occur immediately and are not part of the callback's
    /// queued command or action transaction.
    pub fn clipboard(&mut self) -> &mut dyn Clipboard {
        self.clipboard
    }

    pub(crate) fn request(&mut self, action: impl Into<Action>) -> &mut Self {
        self.actions.push(action.into());
        self.refresh_action();
        self
    }

    pub(crate) fn send(&mut self, command: impl Into<Command>) -> &mut Self {
        self.commands.push(command.into());
        self
    }

    /// Queues an authored open request for commit after a successful callback.
    ///
    /// The request is normalized and planned only after the callback returns;
    /// any resulting terminal failure stops later callback delivery.
    pub fn open(&mut self, open: Open) -> &mut Self {
        self.send(open)
    }

    /// Queues a typed AccessKit tree update for one live window.
    ///
    /// This method is available only with the `accessibility` feature. It defers
    /// submission until the enclosing callback returns `Ok`; the live target,
    /// accessibility capability, and tree phase are validated during later
    /// planning and backend application. Call it from the handler callback that
    /// received [`crate::AccessibilityEvent::InitialTreeRequested`] and pass
    /// that event's live target id. The first update after that request must be
    /// a complete AccessKit update: `TreeUpdate::tree` is present, its root node
    /// is included in `TreeUpdate::nodes`, `TreeUpdate::tree_id` identifies the
    /// tree, and `TreeUpdate::focus` names a node in it. Later active-phase
    /// updates may be incremental, but deactivation requires another complete
    /// initial update. A stale target, unsupported capability, invalid phase, or
    /// planning/application failure becomes the loop's committed terminal error
    /// and suppresses later callbacks.
    #[cfg(feature = "accessibility")]
    pub fn update_accessibility(&mut self, id: Id, update: accesskit::TreeUpdate) -> &mut Self {
        self.send(Command::UpdateAccessibility { id, update })
    }

    /// Queues a close-request action for a live window after callback success.
    ///
    /// The specialized close callback makes the final accept-or-cancel decision.
    pub fn close(&mut self, id: Id) -> &mut Self {
        self.request(Action::CloseRequested(id))
    }

    /// Queues a next-frame draw request after callback success.
    pub fn draw(&mut self, id: Id) -> &mut Self {
        self.request(Action::DrawNext(id))
    }

    /// Queues an immediate draw request after callback success.
    pub fn again(&mut self, id: Id) -> &mut Self {
        self.request(Action::DrawNow(id))
    }

    /// Queues a draw request for `time` after callback success.
    pub fn at(&mut self, id: Id, time: Instant) -> &mut Self {
        self.request(Action::DrawAt { id, time })
    }

    /// Queues a successful terminal exit after callback success.
    ///
    /// Once applied, exit suppresses later queued lifecycle callbacks and closes
    /// loop ingress.
    pub fn exit(&mut self) -> &mut Self {
        self.request(Action::Exit)
    }

    #[must_use]
    pub(crate) fn action(&self) -> &Action {
        &self.action
    }

    #[must_use]
    /// Looks up the current live window identity for an authored name.
    ///
    /// Returns `None` when no matching live window exists.
    pub fn window_id(&self, name: impl AsRef<str>) -> Option<Id> {
        self.registry.window_id(name)
    }

    #[must_use]
    /// Returns the observed snapshot for a live id or name selector.
    ///
    /// Lookup failure, including a closing or destroyed target no longer in the
    /// live registry, returns `None`.
    pub fn state(&self, target: impl Into<Selector>) -> Option<&super::WindowSnapshot> {
        match target.into() {
            Selector::Id(id) => self.registry.get(id).map(|window| window.instance.state()),
            Selector::Name(name) => self
                .registry
                .window_id(name)
                .and_then(|id| self.registry.get(id))
                .map(|window| window.instance.state()),
        }
    }

    /// Borrows registry access for a currently known live window.
    ///
    /// Unknown ids return [`ErrorCode::HandleUnavailable`](super::ErrorCode::HandleUnavailable).
    pub fn access(&self, id: Id) -> super::Result<Ref<'_>> {
        self.registry
            .get(id)
            .ok_or_else(|| Error::new(ErrorCode::HandleUnavailable, "unknown window").with_id(id))
    }

    /// Clones a native-handle lease for a currently known live window.
    ///
    /// Unknown ids return [`ErrorCode::HandleUnavailable`](super::ErrorCode::HandleUnavailable);
    /// a retained handle can remain usable through resource-owning closing.
    pub fn handle(&self, id: Id) -> super::Result<Handle> {
        use super::Access;

        self.access(id)?.handle()
    }

    /// Returns the current observed metrics for a currently known live window.
    ///
    /// Unknown ids return [`ErrorCode::HandleUnavailable`](super::ErrorCode::HandleUnavailable).
    pub fn metrics(&self, id: Id) -> super::Result<Metrics> {
        use super::Access;

        Ok(self.access(id)?.metrics())
    }

    /// Returns a callback-local command target for `id`.
    ///
    /// This accessor does not validate the id; its authored commands are checked
    /// when the successful callback's queue is planned and applied.
    pub fn window(&mut self, id: Id) -> Target<'_> {
        Target::new(id, self.commands, self.actions, &mut self.action)
    }

    pub(crate) fn resolved_action(&self) -> Action {
        self.action.clone()
    }

    fn refresh_action(&mut self) {
        self.action = resolve_actions(self.actions);
    }

    #[must_use]
    /// Returns a cloneable cross-thread proxy when this backend owns one.
    ///
    /// A proxy result reports queue acceptance, not eventual target success;
    /// queued stale work is ignored at event-loop ingress. Callback-free hosts
    /// have no proxy and return `None`.
    pub fn proxy(&self) -> Option<Proxy> {
        self.proxy.clone()
    }
}

pub(crate) fn resolve_actions(actions: &[Action]) -> Action {
    match actions {
        [] => Action::Wait,
        [action] => action.clone(),
        actions => Action::Batch(actions.to_vec()),
    }
}

pub(crate) fn resolve_actions_with(actions: &[Action], fallback: Action) -> Action {
    let mut batch = Vec::with_capacity(actions.len() + 1);
    push_unique_action(&mut batch, fallback);
    for action in actions
        .iter()
        .filter(|action| !matches!(action, Action::Wait))
        .cloned()
    {
        push_unique_action(&mut batch, action);
    }

    match batch.as_slice() {
        [] => Action::Wait,
        [action] => action.clone(),
        _ => Action::Batch(batch),
    }
}

fn push_unique_action(actions: &mut Vec<Action>, action: Action) {
    if !actions.iter().any(|stored| stored == &action) {
        actions.push(action);
    }
}
