use super::winit_adapter::WinitRunner;
use super::winit_mapping;
use super::{
    Clipboard, Command, DrawScheduler, Error, ErrorCode, Handler, MemoryClipboard, Proxy, Registry,
    Result, Role, UserEvent,
    normalization::{NormalizedCommand, normalize_command},
};
#[cfg(test)]
use super::{Context, HostCapabilities, command::Action};

/// Owner of one handler, its registry, startup requests, and native event-loop run.
///
/// Construction is display-free. Before creating the native event loop,
/// [`Loop::run`] intrinsically normalizes retained startup commands and checks
/// them against a virtual registry and lifecycle projection. It then transfers
/// event-loop ownership to `winit`. On the first resume, resolved target
/// capabilities drive startup-batch planning and application; failures in that
/// later phase become retained terminal errors. Native scheduling and return
/// timing are platform-owned.
pub struct Loop<H> {
    pub(crate) handler: H,
    pub(crate) registry: Registry,
    pub(crate) draw: DrawScheduler,
    pub(crate) clipboard: Box<dyn Clipboard>,
    #[cfg(test)]
    pub(crate) commands: Vec<Command>,
    #[cfg(test)]
    pub(crate) actions: Vec<Action>,
    pub(crate) startup: Vec<Command>,
}

pub(crate) struct PreparedLoop<H> {
    window_loop: Loop<H>,
    startup: Vec<NormalizedCommand>,
    projection: VirtualLifecycleProjection,
}

impl<H> PreparedLoop<H> {
    #[cfg(test)]
    #[must_use]
    pub(crate) fn startup(&self) -> &[NormalizedCommand] {
        &self.startup
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn projection(&self) -> &VirtualLifecycleProjection {
        &self.projection
    }

    pub(crate) fn into_loop_and_startup(self) -> (Loop<H>, Vec<NormalizedCommand>) {
        debug_assert!(self.projection.live_window_count() >= self.projection.names().len());
        (self.window_loop, self.startup)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct VirtualLifecycleProjection {
    names: Vec<String>,
    live_window_count: usize,
}

impl VirtualLifecycleProjection {
    #[must_use]
    pub(crate) const fn live_window_count(&self) -> usize {
        self.live_window_count
    }

    #[must_use]
    pub(crate) fn names(&self) -> &[String] {
        &self.names
    }

    fn apply(&mut self, command: &NormalizedCommand) -> Result<()> {
        let Command::Open { request } = command.command() else {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "startup commands must be ordered open requests",
            ));
        };

        if !matches!(request.role(), Role::Root) {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "startup lifecycle supports root windows only",
            ));
        }

        if let Some(name) = request.name() {
            if self.names.iter().any(|existing| existing == name) {
                return Err(Error::new(
                    ErrorCode::DuplicateIdentity,
                    format!("duplicate startup window name '{name}'"),
                ));
            }
            self.names.push(name.to_owned());
        }

        self.live_window_count += 1;
        Ok(())
    }
}

/// Intrinsically normalizes and validates one complete pre-resume startup batch.
pub(crate) fn prepare_startup(
    commands: Vec<Command>,
) -> Result<(Vec<NormalizedCommand>, VirtualLifecycleProjection)> {
    let mut projection = VirtualLifecycleProjection::default();
    let mut startup = Vec::with_capacity(commands.len());

    for command in commands {
        let command = normalize_command(command)?;
        projection.apply(&command)?;
        startup.push(command);
    }

    Ok((startup, projection))
}

impl<H> Loop<H> {
    #[must_use]
    /// Creates a loop with an empty registry, no startup requests, and an in-memory clipboard.
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            registry: Registry::new(),
            draw: DrawScheduler::new(),
            clipboard: Box::new(MemoryClipboard::new()),
            #[cfg(test)]
            commands: Vec::new(),
            #[cfg(test)]
            actions: Vec::new(),
            startup: Vec::new(),
        }
    }

    #[must_use]
    /// Replaces the loop-owned clipboard borrowed into every handler callback.
    ///
    /// Clipboard operations remain immediate external effects rather than queued
    /// callback work.
    pub fn with_clipboard(mut self, clipboard: Box<dyn Clipboard>) -> Self {
        self.clipboard = clipboard;
        self
    }

    #[must_use]
    /// Returns the owned handler before [`Loop::run`] consumes this loop.
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Returns mutable access to the owned handler before [`Loop::run`] consumes it.
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    #[must_use]
    /// Returns the loop registry as currently retained outside callback dispatch.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Returns mutable access to the loop-owned clipboard before [`Loop::run`] consumes it.
    pub fn clipboard_mut(&mut self) -> &mut dyn Clipboard {
        self.clipboard.as_mut()
    }

    pub(crate) fn prepare(mut self) -> Result<PreparedLoop<H>> {
        let authored_startup = std::mem::take(&mut self.startup);
        let (startup, projection) = prepare_startup(authored_startup)?;

        Ok(PreparedLoop {
            window_loop: self,
            startup,
            projection,
        })
    }

    pub(crate) fn run_prepared_with(
        self,
        run: impl FnOnce(PreparedLoop<H>) -> Result<()>,
    ) -> Result<()> {
        run(self.prepare()?)
    }

    #[cfg(test)]
    pub(crate) fn context<'a>(&'a mut self, capabilities: &'a HostCapabilities) -> Context<'a> {
        Context::new(
            &mut self.registry,
            &mut self.commands,
            &mut self.actions,
            self.clipboard.as_mut(),
            capabilities,
            None,
        )
    }
}

impl<H: Handler + 'static> Loop<H> {
    /// Starts the native event loop and runs the handler.
    ///
    /// Before native event-loop creation, intrinsically normalizes retained startup
    /// commands and checks them against a virtual registry and lifecycle projection;
    /// errors from that phase return immediately. It then creates the native event
    /// loop. At the first resume, resolved target capabilities drive startup-batch
    /// planning and application, and errors from that later phase are retained as
    /// terminal errors. This consumes the loop and returns only when `winit` exits;
    /// an explicit terminal exit is successful while callback or terminal command
    /// failures are returned as errors.
    pub fn run(self) -> Result<()> {
        self.run_prepared_with(PreparedLoop::run_native)
    }
}

impl<H: Handler + 'static> PreparedLoop<H> {
    pub(crate) fn run_native(self) -> Result<()> {
        let event_loop = winit::event_loop::EventLoop::<UserEvent>::with_user_event()
            .build()
            .map_err(|source| {
                Error::new(
                    ErrorCode::EventLoopCreateFailed,
                    "failed to create native event loop",
                )
                .with_source(source)
            })?;
        let proxy = Proxy::from_winit(event_loop.create_proxy());
        let mut runner = WinitRunner::from_prepared(self);
        runner.proxy = Some(proxy);
        event_loop.set_control_flow(winit_mapping::native_control_flow(
            winit::event_loop::ControlFlow::Wait,
        ));
        let native_result = event_loop.run_app(&mut runner).map_err(|source| {
            Error::new(ErrorCode::UnknownNativeError, "native event loop failed")
                .with_source(source)
        });
        runner.finish_terminal_result(native_result)
    }
}
