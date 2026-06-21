use surgeist::app::{
    App, AppCommand, AppDescriptor, AppEffect, AppEvent, AppId, AppLoop, AppManifest, AppScope,
    AppSnapshot, CommandDescriptor, Diagnostic, DiagnosticCode, DiagnosticEffect, DiagnosticLog,
    DiagnosticSeverity, EffectKindId, EffectPayload, EventDescriptor, InputProvenance,
    InputSourceId, PersistEffect, QueueDiagnostic, RedrawTarget, RequestRedrawEffect,
    ResourceDescriptor, ResourceId, RootDescriptor, RootId, Runtime, SnapshotBinding,
    StartupWindow, TaskDescriptor, TaskName, UiSurface, WindowDescriptor, WindowRoot,
};

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
    let _ = std::mem::size_of::<EffectPayload>();
    let _ = std::mem::size_of::<RequestRedrawEffect>();
    let _ = std::mem::size_of::<PersistEffect>();
    let _ = std::mem::size_of::<DiagnosticEffect>();
    let _ = std::mem::size_of::<RedrawTarget>();
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
    let binding = SnapshotBinding::new("photos", "PhotoGridSnapshot");
    let root = RootDescriptor::new(RootId::new("gallery"))
        .requires_command(command.clone())
        .emits_event(event.clone())
        .binds_snapshot(binding.clone());
    let window = WindowDescriptor::new("main", "Photo Lab").allows_root(RootId::new("gallery"));
    let startup = StartupWindow::new("main", RootId::new("gallery"), AppScope::app());
    let secondary_startup = StartupWindow::new("main", RootId::new("gallery"), AppScope::app());
    let manifest: AppManifest = AppManifest::new(app.clone())
        .command(command)
        .event(event)
        .task(task)
        .resource(resource)
        .window(window)
        .root(root)
        .startup_window(startup)
        .startup_window(secondary_startup);

    assert_eq!(manifest.app(), &app);
    assert_eq!(manifest.commands().len(), 1);
    assert_eq!(manifest.events().len(), 1);
    assert_eq!(manifest.tasks().len(), 1);
    assert_eq!(manifest.resources().len(), 1);
    assert_eq!(manifest.windows().len(), 1);
    assert_eq!(manifest.roots().len(), 1);
    assert_eq!(manifest.startup().len(), 2);
    assert_eq!(manifest.roots()[0].snapshot_bindings(), &[binding]);
}
