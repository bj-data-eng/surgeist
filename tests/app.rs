use surgeist::app::{
    App, AppCommand, AppDescriptor, AppEffect, AppEvent, AppId, AppLoop, AppManifest, AppScope,
    AppSnapshot, CommandDescriptor, Diagnostic, DiagnosticCode, DiagnosticLog, DiagnosticSeverity,
    EffectKindId, EventDescriptor, ExpressionId, InputOrigin, InputProvenance, InputSourceId,
    QueueDiagnostic, ResourceDescriptor, ResourceId, RootDescriptor, RootId, Runtime,
    ServiceProvenance, SnapshotBinding, SnapshotBindingId, SnapshotSourceType, StartupWindow,
    SurfaceProvenance, TaskDescriptor, TaskName, TaskProvenance, UiSurface, WindowDescriptor,
    WindowDescriptorId, WindowRoot, testing::HeadlessApp,
};

#[test]
fn headless_app_runs_without_winit_or_tokio() {
    let mut app = HeadlessApp::counter();

    app.open_surface("main");
    app.input_increment();
    app.drain();

    assert_eq!(app.counter(), 1);
    assert_eq!(app.fake_window().redraws(), &[app.surface_id("main")]);
    assert_eq!(app.fake_executor().spawned().len(), 0);
}

#[test]
fn app_front_door_exports_expected_names() {
    let _scope = AppScope::app();
    let _ = std::mem::size_of::<App>();
    let _ = std::mem::size_of::<AppLoop>();
    let _ = std::mem::size_of::<Runtime<()>>();
    let _ = std::mem::size_of::<AppCommand>();
    let _ = std::mem::size_of::<AppEvent>();
    let _ = std::mem::size_of::<AppEffect>();
    let _ = std::mem::size_of::<EffectKindId>();
    let _ = std::mem::size_of::<ExpressionId>();
    let _ = std::mem::size_of::<AppSnapshot>();
    let _ = std::mem::size_of::<UiSurface>();
    let _ = std::mem::size_of::<WindowRoot>();
    let _ = std::mem::size_of::<AppDescriptor>();
    let _ = std::mem::size_of::<WindowDescriptor>();
    let _ = std::mem::size_of::<RootDescriptor>();
    let _ = std::mem::size_of::<StartupWindow>();
    let _ = std::mem::size_of::<AppManifest>();
    let _ = std::mem::size_of::<CommandDescriptor>();
    let _ = std::mem::size_of::<EventDescriptor>();
    let _ = std::mem::size_of::<InputSourceId>();
    let _ = std::mem::size_of::<InputProvenance>();
    let _ = std::mem::size_of::<InputOrigin>();
    let _ = std::mem::size_of::<SurfaceProvenance>();
    let _ = std::mem::size_of::<TaskProvenance>();
    let _ = std::mem::size_of::<ServiceProvenance>();
    let _ = std::mem::size_of::<DiagnosticSeverity>();
    let _ = std::mem::size_of::<DiagnosticCode>();
    let _ = std::mem::size_of::<QueueDiagnostic>();
    let _ = std::mem::size_of::<Diagnostic>();
    let _ = std::mem::size_of::<DiagnosticLog>();
}

#[test]
fn app_manifest_registers_identity_windows_roots_commands_events_and_bindings() {
    let app = AppDescriptor::new(AppId::new("photo.lab"), "0.1.0");
    let command = CommandDescriptor::new("photos.import", "ImportPhotos");
    let event = EventDescriptor::new("photos.imported", "ImportFinished");
    let task = TaskDescriptor::new(TaskName::new("photos.import"), "ImportPhotos");
    let resource = ResourceDescriptor::new(ResourceId::new("photos"), "PhotoResource");
    let binding = SnapshotBinding::new(
        SnapshotBindingId::new("photos"),
        SnapshotSourceType::new("PhotoGridSnapshot"),
    );
    let root = RootDescriptor::new(RootId::new("gallery"))
        .requires_command(command.clone())
        .emits_event(event.clone())
        .binds_snapshot(binding.clone());
    let window_id = WindowDescriptorId::new("main");
    let window =
        WindowDescriptor::new(window_id.clone(), "Photo Lab").allows_root(RootId::new("gallery"));
    let startup = StartupWindow::new(window_id, RootId::new("gallery"), AppScope::app());
    let manifest = AppManifest::new(app)
        .command(command)
        .event(event)
        .task(task)
        .resource(resource)
        .window(window)
        .root(root)
        .startup_window(startup);

    assert_eq!(manifest.commands().len(), 1);
    assert_eq!(manifest.events().len(), 1);
    assert_eq!(manifest.tasks().len(), 1);
    assert_eq!(manifest.resources().len(), 1);
    assert_eq!(manifest.windows().len(), 1);
    assert_eq!(manifest.roots().len(), 1);
    assert_eq!(manifest.startup().len(), 1);
    assert_eq!(manifest.roots()[0].snapshot_bindings(), &[binding]);
}

#[test]
fn app_manifest_can_register_multiple_roots_without_startup() {
    let app = AppDescriptor::new(AppId::new("photo.lab"), "0.1.0");
    let manifest = AppManifest::new(app)
        .root(RootDescriptor::new(RootId::new("gallery")))
        .root(RootDescriptor::new(RootId::new("editor")));

    assert_eq!(manifest.roots().len(), 2);
    assert_eq!(manifest.roots()[0].id().as_str(), "gallery");
    assert_eq!(manifest.roots()[1].id().as_str(), "editor");
    assert!(manifest.startup().is_empty());
}
