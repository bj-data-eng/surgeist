use super::{
    AppId, AppScope, CommandDescriptor, EventDescriptor, ResourceId, RootId, SnapshotBinding,
    TaskName,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct App {
    descriptor: AppDescriptor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppDescriptor {
    id: AppId,
    version: String,
    diagnostics_namespace: String,
}

impl AppDescriptor {
    #[must_use]
    pub fn new(id: AppId, version: impl Into<String>) -> Self {
        let diagnostics_namespace = id.as_str().to_owned();
        Self {
            id,
            version: version.into(),
            diagnostics_namespace,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowDescriptor {
    id: String,
    title: String,
    allowed_roots: Vec<RootId>,
}

impl WindowDescriptor {
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            allowed_roots: Vec::new(),
        }
    }

    #[must_use]
    pub fn allows_root(mut self, id: RootId) -> Self {
        self.allowed_roots.push(id);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootDescriptor {
    id: RootId,
    required_commands: Vec<CommandDescriptor>,
    required_events: Vec<EventDescriptor>,
    snapshot_bindings: Vec<SnapshotBinding>,
}

impl RootDescriptor {
    #[must_use]
    pub fn new(id: RootId) -> Self {
        Self {
            id,
            required_commands: Vec::new(),
            required_events: Vec::new(),
            snapshot_bindings: Vec::new(),
        }
    }

    #[must_use]
    pub fn requires_command(mut self, command: CommandDescriptor) -> Self {
        self.required_commands.push(command);
        self
    }

    #[must_use]
    pub fn emits_event(mut self, event: EventDescriptor) -> Self {
        self.required_events.push(event);
        self
    }

    #[must_use]
    pub fn binds_snapshot(mut self, binding: SnapshotBinding) -> Self {
        self.snapshot_bindings.push(binding);
        self
    }

    #[must_use]
    pub fn snapshot_bindings(&self) -> &[SnapshotBinding] {
        &self.snapshot_bindings
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDescriptor {
    name: TaskName,
    input_type: &'static str,
}

impl TaskDescriptor {
    #[must_use]
    pub fn new(name: TaskName, input_type: &'static str) -> Self {
        Self { name, input_type }
    }

    #[must_use]
    pub fn name(&self) -> &TaskName {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceDescriptor {
    id: ResourceId,
    value_type: &'static str,
}

impl ResourceDescriptor {
    #[must_use]
    pub fn new(id: ResourceId, value_type: &'static str) -> Self {
        Self { id, value_type }
    }

    #[must_use]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupWindow {
    pub window_id: String,
    pub root_id: RootId,
    pub scope: AppScope,
}

impl StartupWindow {
    #[must_use]
    pub fn new(window_id: impl Into<String>, root_id: RootId, scope: AppScope) -> Self {
        Self {
            window_id: window_id.into(),
            root_id,
            scope,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppManifest {
    app: AppDescriptor,
    commands: Vec<CommandDescriptor>,
    events: Vec<EventDescriptor>,
    tasks: Vec<TaskDescriptor>,
    resources: Vec<ResourceDescriptor>,
    windows: Vec<WindowDescriptor>,
    roots: Vec<RootDescriptor>,
    startup: Vec<StartupWindow>,
}

impl AppManifest {
    #[must_use]
    pub fn new(app: AppDescriptor) -> Self {
        Self {
            app,
            commands: Vec::new(),
            events: Vec::new(),
            tasks: Vec::new(),
            resources: Vec::new(),
            windows: Vec::new(),
            roots: Vec::new(),
            startup: Vec::new(),
        }
    }

    #[must_use]
    pub fn command(mut self, command: CommandDescriptor) -> Self {
        self.commands.push(command);
        self
    }

    #[must_use]
    pub fn event(mut self, event: EventDescriptor) -> Self {
        self.events.push(event);
        self
    }

    #[must_use]
    pub fn task(mut self, task: TaskDescriptor) -> Self {
        self.tasks.push(task);
        self
    }

    #[must_use]
    pub fn resource(mut self, resource: ResourceDescriptor) -> Self {
        self.resources.push(resource);
        self
    }

    #[must_use]
    pub fn window(mut self, window: WindowDescriptor) -> Self {
        self.windows.push(window);
        self
    }

    #[must_use]
    pub fn root(mut self, root: RootDescriptor) -> Self {
        self.roots.push(root);
        self
    }

    #[must_use]
    pub fn startup_window(mut self, startup: StartupWindow) -> Self {
        self.startup.push(startup);
        self
    }

    #[must_use]
    pub fn build(self) -> Self {
        self
    }

    #[must_use]
    pub fn app(&self) -> &AppDescriptor {
        &self.app
    }

    #[must_use]
    pub fn commands(&self) -> &[CommandDescriptor] {
        &self.commands
    }

    #[must_use]
    pub fn events(&self) -> &[EventDescriptor] {
        &self.events
    }

    #[must_use]
    pub fn tasks(&self) -> &[TaskDescriptor] {
        &self.tasks
    }

    #[must_use]
    pub fn resources(&self) -> &[ResourceDescriptor] {
        &self.resources
    }

    #[must_use]
    pub fn windows(&self) -> &[WindowDescriptor] {
        &self.windows
    }

    #[must_use]
    pub fn roots(&self) -> &[RootDescriptor] {
        &self.roots
    }

    #[must_use]
    pub fn startup(&self) -> &[StartupWindow] {
        &self.startup
    }
}
