use crate::{
    Clipboard, Command, Context, Error, Event as EventScope, EventKind, Frame, Handler,
    HostCapabilities, Id, Input, InputEvent, Proxy, Ready, Registry, Result, WindowSnapshot,
    command::Action,
    context::resolve_actions_with,
    dsl::{Close, Closed},
    normalization::{NormalizedCommand, normalize_commands},
};
use std::collections::{HashSet, VecDeque};

#[derive(Debug)]
enum Terminal {
    Running,
    ExitRequested,
    Failed(Error),
}

/// Backend-neutral callback descriptor consumed by the shared lifecycle pump.
#[derive(Clone)]
pub(crate) enum Callback {
    Event(EventKind),
    Ready(Id),
    Close(Id),
    Closed(WindowSnapshot),
    Resize(Id),
    Input(InputEvent),
    Frame(Id),
    Resume,
    Suspend,
    Idle,
}

/// The one lifecycle transition a successful close callback may commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallbackCompletion {
    None,
    BeginClose(Id),
}

/// Selects lifecycle handling for work after intrinsic normalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkOrigin {
    /// Callback, direct, and synchronous host work keeps typed lifecycle failures.
    Strict,
    /// Accepted proxy work drops stale target operations at event-loop ingress.
    Queued,
}

/// Selects the same specialized handler route used for a native transition event.
pub(crate) fn callback_for_event(event: EventKind) -> Callback {
    match event {
        EventKind::Resized(metrics) | EventKind::ScaleFactorChanged(metrics) => {
            Callback::Resize(metrics.id())
        }
        EventKind::Input(input) => Callback::Input(input),
        event => Callback::Event(event),
    }
}

pub(crate) struct CallbackEnvironment<'a> {
    registry: &'a mut Registry,
    clipboard: &'a mut dyn Clipboard,
    capabilities: &'a HostCapabilities,
    proxy: Option<Proxy>,
}

impl<'a> CallbackEnvironment<'a> {
    pub(crate) fn new(
        registry: &'a mut Registry,
        clipboard: &'a mut dyn Clipboard,
        capabilities: &'a HostCapabilities,
        proxy: Option<Proxy>,
    ) -> Self {
        Self {
            registry,
            clipboard,
            capabilities,
            proxy,
        }
    }
}

/// Invokes one callback with the same context and action collection rules for every backend.
pub(crate) fn invoke_handler_callback<H: Handler>(
    handler: &mut H,
    environment: CallbackEnvironment<'_>,
    callback: Callback,
    commands: &mut Vec<Command>,
    actions: &mut Vec<Action>,
) -> Result<CallbackCompletion> {
    let completion = match callback {
        Callback::Event(event) => {
            let action = {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut event = EventScope::new(event, context);
                handler.event(&mut event)?;
                event.context_mut().resolved_action()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
        Callback::Ready(id) => {
            if !environment.registry.contains(id) {
                return Ok(CallbackCompletion::None);
            }

            {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut ready = Ready::new(id, context);
                handler.ready(&mut ready)?;
            }
            replace_callback_actions(actions, resolve_actions_with(actions, Action::DrawNext(id)));
            CallbackCompletion::None
        }
        Callback::Close(id) => {
            if !environment.registry.contains(id) {
                return Ok(CallbackCompletion::None);
            }

            let (action, accepted) = {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut close = Close::new(id, context);
                handler.close(&mut close)?;
                (close.context_mut().resolved_action(), close.is_accepted())
            };
            replace_callback_actions(actions, action);
            if accepted {
                CallbackCompletion::BeginClose(id)
            } else {
                CallbackCompletion::None
            }
        }
        Callback::Closed(state) => {
            let action = {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut closed = Closed::new(state, context);
                handler.closed(&mut closed)?;
                closed.context_mut().resolved_action()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
        Callback::Resize(id) => {
            {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut resize = crate::Resize::new(id, context);
                handler.resize(&mut resize)?;
            }
            replace_callback_actions(actions, resolve_actions_with(actions, Action::DrawNext(id)));
            CallbackCompletion::None
        }
        Callback::Input(input) => {
            let action = {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut input = Input::new(input, context);
                handler.input(&mut input)?;
                input.context_mut().resolved_action()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
        Callback::Frame(id) => {
            let action = {
                let context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                let mut frame = Frame::new(id, context);
                handler.draw(&mut frame)?;
                frame.action().clone()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
        Callback::Resume => {
            let action = {
                let mut context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                handler.resume(&mut context)?;
                context.resolved_action()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
        Callback::Suspend => {
            let action = {
                let mut context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                handler.suspend(&mut context)?;
                context.resolved_action()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
        Callback::Idle => {
            let action = {
                let mut context = Context::new(
                    environment.registry,
                    commands,
                    actions,
                    environment.clipboard,
                    environment.capabilities,
                    environment.proxy,
                );
                handler.idle(&mut context)?;
                context.resolved_action()
            };
            replace_callback_actions(actions, action);
            CallbackCompletion::None
        }
    };
    Ok(completion)
}

/// Queues `Ready` only after a successful, still-live `Created` callback transaction.
pub(crate) fn enqueue_ready_after_created(
    callback: Callback,
    registry: &Registry,
    pump: &mut Pump<Callback>,
) {
    let Callback::Event(EventKind::Created(state)) = callback else {
        return;
    };
    if registry.contains(state.id()) {
        pump.enqueue_callback(Callback::Ready(state.id()));
    }
}

/// Queues per-window `Resumed` callbacks in deterministic ID order before the global callback.
pub(crate) fn enqueue_resume_callbacks(registry: &Registry, pump: &mut Pump<Callback>) {
    for id in registry.live_ids() {
        pump.enqueue_callback(Callback::Event(EventKind::Resumed(id)));
    }
    pump.enqueue_callback(Callback::Resume);
}

/// Queues per-window `Suspended` callbacks in deterministic ID order before the global callback.
pub(crate) fn enqueue_suspend_callbacks(registry: &Registry, pump: &mut Pump<Callback>) {
    for id in registry.live_ids() {
        pump.enqueue_callback(Callback::Event(EventKind::Suspended(id)));
    }
    pump.enqueue_callback(Callback::Suspend);
}

fn replace_callback_actions(actions: &mut Vec<Action>, action: Action) {
    actions.clear();
    actions.push(action);
}

enum Item<C> {
    Callback(C),
    BeginClose(Id),
    CompleteClose(Id),
    Work {
        origin: WorkOrigin,
        commands: Vec<NormalizedCommand>,
        actions: Vec<Action>,
    },
}

/// Private backend operations consumed by the lifecycle pump.
pub(crate) trait PumpBackend<C> {
    fn invoke_callback(
        &mut self,
        callback: C,
        commands: &mut Vec<Command>,
        actions: &mut Vec<Action>,
    ) -> Result<CallbackCompletion>;

    fn apply_commands(
        &mut self,
        commands: Vec<NormalizedCommand>,
        origin: WorkOrigin,
        pump: &mut Pump<C>,
    ) -> Result<()>;

    fn apply_action(
        &mut self,
        action: Action,
        origin: WorkOrigin,
        pump: &mut Pump<C>,
    ) -> Result<()>;

    fn begin_close(&mut self, id: Id, pump: &mut Pump<C>) -> Result<()>;

    /// Completes an already-started close after native ownership ends.
    fn complete_close(&mut self, _id: Id, _pump: &mut Pump<C>) -> Result<()> {
        Ok(())
    }

    fn after_callback(&mut self, _callback: C, _pump: &mut Pump<C>) {}

    /// Closes backend-owned ingress before the pump records a terminal result.
    fn close_ingress(&mut self) {}

    /// Invalidates backend-owned resources without delivering further callbacks.
    fn teardown(&mut self) {}
}

/// Backend-neutral non-reentrant owner of callback work and terminal state.
pub(crate) struct Pump<C> {
    items: VecDeque<Item<C>>,
    begin_closes: HashSet<Id>,
    started_closes: HashSet<Id>,
    completion_requests: HashSet<Id>,
    completion_items: HashSet<Id>,
    completed_closes: HashSet<Id>,
    terminal: Terminal,
    draining: bool,
}

impl<C> Pump<C> {
    pub(crate) fn new() -> Self {
        Self {
            items: VecDeque::new(),
            begin_closes: HashSet::new(),
            started_closes: HashSet::new(),
            completion_requests: HashSet::new(),
            completion_items: HashSet::new(),
            completed_closes: HashSet::new(),
            terminal: Terminal::Running,
            draining: false,
        }
    }

    pub(crate) fn enqueue_callback(&mut self, callback: C) {
        if !self.is_running() {
            return;
        }

        self.items.push_back(Item::Callback(callback));
    }

    pub(crate) fn enqueue_normalized_commands(&mut self, commands: Vec<NormalizedCommand>) {
        self.enqueue_work(WorkOrigin::Strict, commands, Vec::new());
    }

    /// Queues one accepted proxy command for lifecycle-aware event-loop processing.
    pub(crate) fn enqueue_queued_normalized_command(&mut self, command: NormalizedCommand) {
        self.enqueue_work(WorkOrigin::Queued, vec![command], Vec::new());
    }

    /// Queues one accepted proxy action for lifecycle-aware event-loop processing.
    pub(crate) fn enqueue_queued_action(&mut self, action: Action) {
        self.enqueue_work(WorkOrigin::Queued, Vec::new(), vec![action]);
    }

    /// Inserts an idempotent lifecycle item before later callback work.
    pub(crate) fn enqueue_begin_close(&mut self, id: Id) {
        if self.is_running() && self.begin_closes.insert(id) {
            self.items.push_front(Item::BeginClose(id));
        }
    }

    /// Applies the shared lifecycle item at the authored `Destroy` position.
    pub(crate) fn apply_begin_close(
        &mut self,
        backend: &mut impl PumpBackend<C>,
        id: Id,
    ) -> Result<()>
    where
        C: Clone,
    {
        if self.is_running() && self.begin_closes.insert(id) {
            self.start_close(backend, id)?;
            if self.completion_requests.contains(&id) {
                self.apply_close_completion(backend, id)?;
            }
        }
        Ok(())
    }

    /// Requests one serialized completion after an already-started close.
    ///
    /// A release may arrive while the corresponding begin-close item is still
    /// queued. Recording the request first lets that item schedule completion
    /// after it establishes coherent closing ownership.
    pub(crate) fn enqueue_close_completion(&mut self, id: Id) {
        if !self.is_running() || !self.completion_requests.insert(id) {
            return;
        }
        if self.started_closes.contains(&id) {
            self.enqueue_close_completion_item(id);
        }
    }

    /// Cancels not-yet-processed target actions while retaining queue order.
    pub(crate) fn cancel_target_actions(&mut self, id: Id) {
        for item in &mut self.items {
            let Item::Work { actions, .. } = item else {
                continue;
            };
            *actions = actions
                .drain(..)
                .filter_map(|action| action.without_target(id))
                .collect();
        }
    }

    pub(crate) fn drain(&mut self, backend: &mut impl PumpBackend<C>)
    where
        C: Clone,
    {
        if self.draining || !self.is_running() {
            return;
        }

        self.draining = true;
        while self.is_running() {
            let Some(item) = self.items.pop_front() else {
                break;
            };

            let result = self.apply_item(backend, item);
            if let Err(error) = result {
                backend.close_ingress();
                backend.teardown();
                self.fail(error);
            }
        }
        self.draining = false;
    }

    pub(crate) fn is_running(&self) -> bool {
        matches!(self.terminal, Terminal::Running)
    }

    pub(crate) fn result(&self) -> Option<std::result::Result<(), &Error>> {
        match &self.terminal {
            Terminal::Running => None,
            Terminal::ExitRequested => Some(Ok(())),
            Terminal::Failed(error) => Some(Err(error)),
        }
    }

    #[cfg(test)]
    pub(crate) fn has_failed(&self) -> bool {
        matches!(self.terminal, Terminal::Failed(_))
    }

    pub(crate) fn fail(&mut self, error: Error) {
        if self.is_running() {
            self.terminal = Terminal::Failed(error);
        }
    }

    pub(crate) fn into_result(self) -> Result<()> {
        match self.terminal {
            Terminal::Failed(error) => Err(error),
            Terminal::Running | Terminal::ExitRequested => Ok(()),
        }
    }

    fn enqueue_work(
        &mut self,
        origin: WorkOrigin,
        commands: Vec<NormalizedCommand>,
        actions: Vec<Action>,
    ) {
        if self.is_running() {
            self.items.push_back(Item::Work {
                origin,
                commands,
                actions,
            });
        }
    }

    fn apply_item(&mut self, backend: &mut impl PumpBackend<C>, item: Item<C>) -> Result<()>
    where
        C: Clone,
    {
        match item {
            Item::Callback(callback) => self.apply_callback(backend, callback),
            Item::BeginClose(id) => {
                self.start_close(backend, id)?;
                if self.completion_requests.contains(&id) {
                    self.enqueue_close_completion_item(id);
                }
                Ok(())
            }
            Item::CompleteClose(id) => self.apply_close_completion(backend, id),
            Item::Work {
                origin,
                commands,
                actions,
            } => self.apply_work(backend, origin, commands, actions, false),
        }
    }

    fn apply_callback(&mut self, backend: &mut impl PumpBackend<C>, callback: C) -> Result<()>
    where
        C: Clone,
    {
        let completed_callback = callback.clone();
        let mut commands = Vec::new();
        let mut actions = Vec::new();
        let completion = backend.invoke_callback(callback, &mut commands, &mut actions)?;
        self.apply_work(
            backend,
            WorkOrigin::Strict,
            normalize_commands(commands)?,
            actions,
            true,
        )?;
        if self.is_running() {
            if let CallbackCompletion::BeginClose(id) = completion {
                self.enqueue_begin_close(id);
            }
            backend.after_callback(completed_callback, self);
        }
        Ok(())
    }

    fn apply_work(
        &mut self,
        backend: &mut impl PumpBackend<C>,
        origin: WorkOrigin,
        commands: Vec<NormalizedCommand>,
        actions: Vec<Action>,
        apply_empty_batch: bool,
    ) -> Result<()> {
        if apply_empty_batch || !commands.is_empty() {
            backend.apply_commands(commands, origin, self)?;
        }
        self.apply_actions(backend, origin, actions)
    }

    fn apply_actions(
        &mut self,
        backend: &mut impl PumpBackend<C>,
        origin: WorkOrigin,
        actions: Vec<Action>,
    ) -> Result<()> {
        let mut pending = VecDeque::from(actions);
        while self.is_running() {
            let Some(action) = pending.pop_front() else {
                break;
            };
            match action {
                Action::Batch(actions) => {
                    for action in actions.into_iter().rev() {
                        pending.push_front(action);
                    }
                }
                Action::Exit => {
                    backend.apply_action(Action::Exit, origin, self)?;
                    backend.close_ingress();
                    backend.teardown();
                    self.terminal = Terminal::ExitRequested;
                }
                action => backend.apply_action(action, origin, self)?,
            }
        }
        Ok(())
    }

    fn enqueue_close_completion_item(&mut self, id: Id) {
        if !self.completed_closes.contains(&id) && self.completion_items.insert(id) {
            self.items.push_front(Item::CompleteClose(id));
        }
    }

    fn start_close(&mut self, backend: &mut impl PumpBackend<C>, id: Id) -> Result<()>
    where
        C: Clone,
    {
        if self.started_closes.insert(id) {
            self.cancel_target_actions(id);
            backend.begin_close(id, self)?;
        }
        Ok(())
    }

    fn apply_close_completion(&mut self, backend: &mut impl PumpBackend<C>, id: Id) -> Result<()> {
        if self.completed_closes.insert(id) {
            backend.complete_close(id, self)?;
        }
        Ok(())
    }
}

impl<C> Default for Pump<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{CallbackCompletion, Pump, PumpBackend};
    use crate::{Command, Error, ErrorCode, Result, command::Action};

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Callback {
        Failure,
        FailureWithWork,
        CommandAndAction,
        Initial,
        AfterCommand,
        Exit,
        Later,
    }

    #[derive(Default)]
    struct TestBackend {
        commands: Vec<Command>,
        actions: Vec<Action>,
        callbacks: Vec<&'static str>,
        callback_observed_coherent_state: bool,
        fail_commands: bool,
    }

    impl PumpBackend<Callback> for TestBackend {
        fn invoke_callback(
            &mut self,
            callback: Callback,
            commands: &mut Vec<Command>,
            actions: &mut Vec<Action>,
        ) -> Result<CallbackCompletion> {
            match callback {
                Callback::Failure => Err(Error::new(ErrorCode::CommandFailed, "handler failure")),
                Callback::FailureWithWork => {
                    commands.push(Command::RequestDraw {
                        id: crate::Id::from_u64(1),
                    });
                    actions.push(Action::DrawNow(crate::Id::from_u64(1)));
                    Err(Error::new(ErrorCode::CommandFailed, "handler failure"))
                }
                Callback::CommandAndAction => {
                    commands.push(Command::RequestDraw {
                        id: crate::Id::from_u64(1),
                    });
                    actions.push(Action::DrawNow(crate::Id::from_u64(1)));
                    Ok(CallbackCompletion::None)
                }
                Callback::Initial => {
                    commands.push(Command::RequestDraw {
                        id: crate::Id::from_u64(1),
                    });
                    Ok(CallbackCompletion::None)
                }
                Callback::AfterCommand => {
                    self.callbacks.push("after-command");
                    self.callback_observed_coherent_state = self.commands.len() == 1;
                    commands.push(Command::RequestDraw {
                        id: crate::Id::from_u64(2),
                    });
                    Ok(CallbackCompletion::None)
                }
                Callback::Exit => {
                    commands.push(Command::RequestDraw {
                        id: crate::Id::from_u64(1),
                    });
                    actions.push(Action::Exit);
                    Ok(CallbackCompletion::None)
                }
                Callback::Later => {
                    self.callbacks.push("later");
                    Ok(CallbackCompletion::None)
                }
            }
        }

        fn apply_commands(
            &mut self,
            commands: Vec<crate::normalization::NormalizedCommand>,
            _origin: super::WorkOrigin,
            pump: &mut Pump<Callback>,
        ) -> Result<()> {
            if self.fail_commands {
                return Err(Error::new(
                    ErrorCode::CommandFailed,
                    "backend command failure",
                ));
            }
            self.commands
                .extend(commands.into_iter().map(|command| command.into_command()));
            if self.commands.len() == 1 {
                pump.enqueue_callback(Callback::AfterCommand);
            }
            Ok(())
        }

        fn apply_action(
            &mut self,
            action: Action,
            _origin: super::WorkOrigin,
            _pump: &mut Pump<Callback>,
        ) -> Result<()> {
            self.actions.push(action);
            Ok(())
        }

        fn begin_close(&mut self, _id: crate::Id, _pump: &mut Pump<Callback>) -> Result<()> {
            Ok(())
        }
    }

    fn drain(pump: &mut Pump<Callback>, backend: &mut TestBackend) {
        pump.drain(backend);
    }

    #[test]
    fn pump_handler_failure_is_retained_as_terminal_state() {
        let mut pump = Pump::new();
        let mut backend = TestBackend::default();

        pump.enqueue_callback(Callback::Failure);
        drain(&mut pump, &mut backend);

        let error = pump
            .into_result()
            .expect_err("the first handler failure reaches the loop result");
        assert_eq!(error.message, "handler failure");
        assert!(backend.commands.is_empty());
        assert!(backend.actions.is_empty());
    }

    #[test]
    fn pump_callback_failure_discards_commands_and_actions() {
        let mut pump = Pump::new();
        let mut backend = TestBackend::default();

        pump.enqueue_callback(Callback::FailureWithWork);
        drain(&mut pump, &mut backend);

        assert!(backend.commands.is_empty());
        assert!(backend.actions.is_empty());
    }

    #[test]
    fn pump_command_failure_discards_actions() {
        let mut pump = Pump::new();
        let mut backend = TestBackend {
            fail_commands: true,
            ..TestBackend::default()
        };

        pump.enqueue_callback(Callback::CommandAndAction);
        drain(&mut pump, &mut backend);

        assert!(backend.commands.is_empty());
        assert!(backend.actions.is_empty());
        assert!(pump.into_result().is_err());
    }

    #[test]
    fn pump_nested_commands_run_non_reentrantly_after_coherent_state() {
        let mut pump = Pump::new();
        let mut backend = TestBackend::default();

        pump.enqueue_callback(Callback::Initial);
        drain(&mut pump, &mut backend);

        assert_eq!(backend.callbacks, ["after-command"]);
        assert_eq!(backend.commands.len(), 2);
        assert!(backend.callback_observed_coherent_state);
    }

    #[test]
    fn pump_exit_suppresses_later_callbacks() {
        let mut pump = Pump::new();
        let mut backend = TestBackend::default();

        pump.enqueue_callback(Callback::Exit);
        pump.enqueue_callback(Callback::Later);
        drain(&mut pump, &mut backend);

        assert_eq!(backend.actions, [Action::Exit]);
        assert_eq!(backend.commands.len(), 1);
        assert!(backend.callbacks.is_empty());
        assert!(pump.into_result().is_ok());
    }

    #[test]
    fn closing_cancels_queued_target_actions() {
        let closing = crate::Id::from_u64(1);
        let surviving = crate::Id::from_u64(2);
        let mut pump = Pump::new();
        let mut backend = TestBackend::default();

        pump.enqueue_queued_normalized_command(
            crate::normalization::normalize_command(crate::Command::Open {
                request: crate::WindowRequest::builder("surviving").build(),
            })
            .expect("open command normalizes"),
        );
        let queued_actions = Action::Batch(vec![
            Action::DrawNext(closing),
            Action::Batch(vec![Action::DrawNow(closing), Action::DrawNext(surviving)]),
            Action::CloseRequested(closing),
            Action::Exit,
        ]);
        assert_eq!(
            queued_actions.without_target(closing),
            Some(Action::Batch(vec![
                Action::Batch(vec![Action::DrawNext(surviving)]),
                Action::Exit,
            ]))
        );
        pump.enqueue_queued_action(queued_actions);
        pump.enqueue_begin_close(closing);

        drain(&mut pump, &mut backend);

        assert!(matches!(
            backend.commands.first(),
            Some(crate::Command::Open { request }) if request.name() == Some("surviving")
        ));
        assert_eq!(backend.actions, [Action::DrawNext(surviving), Action::Exit]);
    }
}
