use super::command::Action;
use super::testing::{Effect, Event as HostEvent};
#[cfg(feature = "accessibility")]
use super::winit_adapter::{
    AccessibilitySetupError, accessibility_event_from_winit, native_open_request,
    prepare_accessible_open,
};
use super::winit_adapter::{
    NativeTransitionRoute, PointerPositionKey, WinitRunner, code_from_winit, cursor_grab_failed,
    ime_event_from_winit, key_from_winit, location_from_winit, native_transition_route,
};
use super::*;
use crate::descriptor::WindowSnapshotSeed;
use crate::loop_::PreparedLoop;
use crate::planning::{ResolvedImePurpose, take_planned_command, take_planned_ime_request};
use crate::registry::ProxyQueue;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc, time::Instant};

fn conservative_capabilities() -> HostCapabilities {
    testing::Host::new().capabilities().clone()
}

fn fully_supported_capabilities_with_accessibility(
    accessibility: CapabilitySupport,
) -> HostCapabilities {
    HostCapabilities::builder()
        .role(RoleKind::Root, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::Borderless, CapabilitySupport::Supported)
        .cursor(CursorCapability::Icon, CapabilitySupport::Supported)
        .cursor(CursorCapability::Hidden, CapabilitySupport::Supported)
        .window(WindowOperation::SetTitle, CapabilitySupport::Supported)
        .window(
            WindowOperation::SetOuterPosition,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::SetVisible, CapabilitySupport::Supported)
        .window(WindowOperation::SetResizable, CapabilitySupport::Supported)
        .window(WindowOperation::SetControls, CapabilitySupport::Supported)
        .window(
            WindowOperation::SetDecorations,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::InitialTransparency,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::SetTransparent,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::RequestInnerSize,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::SetInnerSizeBounds,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::SetLevel, CapabilitySupport::Supported)
        .window(
            WindowOperation::SetExplicitTheme,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::ResetTheme, CapabilitySupport::Supported)
        .window(
            WindowOperation::RequestUserAttention,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::RequestDraw, CapabilitySupport::Supported)
        .window(WindowOperation::Destroy, CapabilitySupport::Supported)
        .cursor_grab(CursorGrab::None, CapabilitySupport::Supported)
        .cursor_grab(CursorGrab::Confined, CapabilitySupport::Supported)
        .cursor_grab(CursorGrab::Locked, CapabilitySupport::Supported)
        .ime(ImeCapability::Enablement, CapabilitySupport::Supported)
        .ime(ImeCapability::CursorArea, CapabilitySupport::Supported)
        .ime(
            ImeCapability::Purpose(ImePurpose::Normal),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Purpose(ImePurpose::Password),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Purpose(ImePurpose::Number),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Purpose(ImePurpose::Email),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Purpose(ImePurpose::Url),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Purpose(ImePurpose::Terminal),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Hint(ImeHint::None),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Hint(ImeHint::Spellcheck),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Hint(ImeHint::NoSpellcheck),
            CapabilitySupport::Supported,
        )
        .ime(ImeCapability::SurroundingText, CapabilitySupport::Supported)
        .accessibility(accessibility)
        .build()
}

fn fully_supported_capabilities() -> HostCapabilities {
    fully_supported_capabilities_with_accessibility(CapabilitySupport::Supported)
}

fn state(id: Id) -> WindowSnapshot {
    WindowSnapshot::from_seed(WindowSnapshotSeed {
        title: String::from("Test"),
        name: Some(format!("surgeist-test-{}", id.as_u64())),
        metrics: Metrics::from_physical_size(
            id,
            PhysicalSize {
                width: 200,
                height: 100,
            },
            2.0,
        )
        .and_then(|metrics| metrics.with_outer_geometry(Some(Point { x: 10.0, y: 20.0 }), None))
        .expect("test metrics are valid"),
        focused: false,
        visible: Some(true),
        minimized: Some(false),
        maximized: false,
        occluded: Some(false),
        fullscreen: false,
        theme: Some(Theme::Dark),
        role: Role::Root,
    })
}

#[test]
fn window_public_front_door_uses_modeled_phase_names() {
    let _ = std::mem::size_of::<WindowRequest>();
    let _ = std::mem::size_of::<WindowRequestBuilder>();
    let _ = std::mem::size_of::<WindowSnapshot>();
    let _ = std::mem::size_of::<HostCapabilities>();
    let _ = std::mem::size_of::<HostCommandPlan>();
    let _ = std::mem::size_of::<WindowStatePatch>();
    let _ = std::mem::size_of::<NativeEventTransition>();
}

#[test]
fn capabilities_support_has_exact_three_states() {
    fn name(support: CapabilitySupport) -> &'static str {
        match support {
            CapabilitySupport::Supported => "supported",
            CapabilitySupport::Unsupported => "unsupported",
            CapabilitySupport::RuntimeDependent => "runtime-dependent",
        }
    }

    let support = [
        CapabilitySupport::Supported,
        CapabilitySupport::Unsupported,
        CapabilitySupport::RuntimeDependent,
    ];

    assert_eq!(
        support.map(name),
        ["supported", "unsupported", "runtime-dependent"]
    );

    let runtime_dependent = HostCapabilities::builder()
        .role(RoleKind::Root, CapabilitySupport::RuntimeDependent)
        .fullscreen(FullscreenMode::None, CapabilitySupport::RuntimeDependent)
        .cursor(CursorCapability::Icon, CapabilitySupport::RuntimeDependent)
        .build();
    assert!(runtime_dependent.require_role(RoleKind::Root).is_ok());
    assert!(
        runtime_dependent
            .require_fullscreen(FullscreenMode::None)
            .is_ok()
    );
    assert!(
        runtime_dependent
            .require_cursor(CursorCapability::Icon)
            .is_ok()
    );
    assert!(
        HostCommandPlan::from_command(
            Command::Open {
                request: WindowRequest::default(),
            },
            &runtime_dependent,
        )
        .is_ok()
    );

    let unsupported = HostCapabilities::builder().build();
    assert_eq!(
        unsupported
            .require_role(RoleKind::Root)
            .expect_err("unsupported capabilities must be rejected predictably")
            .code,
        ErrorCode::UnsupportedFeature
    );
}

#[test]
fn capability_report_covers_every_fixed_operation_kind() {
    let window_operations = [
        (WindowOperation::SetTitle, CapabilitySupport::Supported),
        (
            WindowOperation::SetOuterPosition,
            CapabilitySupport::RuntimeDependent,
        ),
        (WindowOperation::SetVisible, CapabilitySupport::Supported),
        (WindowOperation::SetResizable, CapabilitySupport::Supported),
        (WindowOperation::SetControls, CapabilitySupport::Supported),
        (
            WindowOperation::SetDecorations,
            CapabilitySupport::Supported,
        ),
        (
            WindowOperation::InitialTransparency,
            CapabilitySupport::Supported,
        ),
        (
            WindowOperation::SetTransparent,
            CapabilitySupport::Supported,
        ),
        (
            WindowOperation::RequestInnerSize,
            CapabilitySupport::Supported,
        ),
        (
            WindowOperation::SetInnerSizeBounds,
            CapabilitySupport::Supported,
        ),
        (WindowOperation::SetLevel, CapabilitySupport::Supported),
        (
            WindowOperation::SetExplicitTheme,
            CapabilitySupport::Supported,
        ),
        (WindowOperation::ResetTheme, CapabilitySupport::Supported),
        (
            WindowOperation::RequestUserAttention,
            CapabilitySupport::Supported,
        ),
        (
            WindowOperation::RequestDraw,
            CapabilitySupport::RuntimeDependent,
        ),
        (WindowOperation::Destroy, CapabilitySupport::Supported),
    ];
    let cursor_grabs = [
        (CursorGrab::None, CapabilitySupport::Supported),
        (CursorGrab::Confined, CapabilitySupport::Supported),
        (CursorGrab::Locked, CapabilitySupport::RuntimeDependent),
    ];
    let ime_capabilities = [
        (ImeCapability::Enablement, CapabilitySupport::Supported),
        (ImeCapability::CursorArea, CapabilitySupport::Supported),
        (
            ImeCapability::Purpose(ImePurpose::Normal),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Purpose(ImePurpose::Password),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Purpose(ImePurpose::Number),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Purpose(ImePurpose::Email),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Purpose(ImePurpose::Url),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Purpose(ImePurpose::Terminal),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Hint(ImeHint::None),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Hint(ImeHint::Spellcheck),
            CapabilitySupport::Supported,
        ),
        (
            ImeCapability::Hint(ImeHint::NoSpellcheck),
            CapabilitySupport::RuntimeDependent,
        ),
        (ImeCapability::SurroundingText, CapabilitySupport::Supported),
    ];
    let capabilities = window_operations
        .iter()
        .copied()
        .fold(
            HostCapabilities::builder(),
            |builder, (operation, support)| builder.window(operation, support),
        )
        .accessibility(CapabilitySupport::RuntimeDependent);
    let capabilities = cursor_grabs
        .iter()
        .copied()
        .fold(capabilities, |builder, (grab, support)| {
            builder.cursor_grab(grab, support)
        });
    let capabilities = ime_capabilities
        .iter()
        .copied()
        .fold(capabilities, |builder, (capability, support)| {
            builder.ime(capability, support)
        })
        .build();

    for (operation, support) in window_operations {
        assert_eq!(capabilities.window(operation), support);
        let decision = CapabilityDecision::new(CapabilityKind::Window(operation), support);
        assert_eq!(decision.kind(), CapabilityKind::Window(operation));
        assert_eq!(decision.support(), support);
        assert!(capabilities.require_window(operation).is_ok());
    }
    for (grab, support) in cursor_grabs {
        assert_eq!(capabilities.cursor_grab(grab), support);
        let decision = CapabilityDecision::new(CapabilityKind::CursorGrab(grab), support);
        assert_eq!(decision.kind(), CapabilityKind::CursorGrab(grab));
        assert_eq!(decision.support(), support);
        assert!(capabilities.require_cursor_grab(grab).is_ok());
    }
    for (capability, support) in ime_capabilities {
        assert_eq!(capabilities.ime(capability), support);
        let decision = CapabilityDecision::new(CapabilityKind::Ime(capability), support);
        assert_eq!(decision.kind(), CapabilityKind::Ime(capability));
        assert_eq!(decision.support(), support);
        assert!(capabilities.require_ime(capability).is_ok());
    }
    let accessibility = CapabilityDecision::new(
        CapabilityKind::Accessibility,
        CapabilitySupport::RuntimeDependent,
    );
    assert_eq!(capabilities.accessibility(), accessibility.support());
    assert_eq!(accessibility.kind(), CapabilityKind::Accessibility);
    assert!(capabilities.require_accessibility().is_ok());

    let unsupported = HostCapabilities::builder().build();
    for (operation, _) in window_operations {
        assert_eq!(
            unsupported.window(operation),
            CapabilitySupport::Unsupported
        );
    }
    for (grab, _) in cursor_grabs {
        assert_eq!(
            unsupported.cursor_grab(grab),
            CapabilitySupport::Unsupported
        );
    }
    for (capability, _) in ime_capabilities {
        assert_eq!(unsupported.ime(capability), CapabilitySupport::Unsupported);
    }
    assert_eq!(unsupported.accessibility(), CapabilitySupport::Unsupported);
}

#[test]
fn testing_host_accepts_an_explicit_immutable_report() {
    let report = HostCapabilities::builder()
        .window(
            WindowOperation::RequestDraw,
            CapabilitySupport::RuntimeDependent,
        )
        .build();
    let host = testing::Host::with_capabilities(report.clone());

    assert_eq!(host.capabilities(), &report);
    assert_eq!(
        host.capabilities().window(WindowOperation::RequestDraw),
        CapabilitySupport::RuntimeDependent
    );
}

#[test]
fn capabilities_context_exposes_current_target_report() {
    struct ReportObserver {
        observed: Option<HostCapabilities>,
    }

    impl Handler for ReportObserver {
        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.observed = Some(ready.context_mut().capabilities().clone());
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("fixture root window opens");
    let expected = host.capabilities().clone();
    let mut observer = ReportObserver { observed: None };

    host.dispatch_ready(&mut observer, Id::from_u64(1))
        .expect("ready callback succeeds");

    assert_eq!(observer.observed.as_ref(), Some(&expected));
}

#[test]
fn host_command_plan_exposes_only_planned_diagnostics() {
    let id = Id::from_u64(7);
    let plan = HostCommandPlan::from_command(
        Command::SetTitle {
            id,
            title: String::from("Renamed"),
        },
        &conservative_capabilities(),
    )
    .expect("supported command should produce a host plan");

    assert_eq!(plan.kind(), CommandKind::SetTitle);
    assert_eq!(plan.target(), Some(id));
    assert_eq!(
        plan.capability_decisions(),
        [CapabilityDecision::new(
            CapabilityKind::Window(WindowOperation::SetTitle),
            CapabilitySupport::Supported,
        )]
    );

    let kinds = [
        CommandKind::Open,
        CommandKind::SetTitle,
        CommandKind::SetPosition,
        CommandKind::SetVisible,
        CommandKind::SetResizable,
        CommandKind::SetControls,
        CommandKind::SetDecorations,
        CommandKind::SetTransparent,
        CommandKind::SetInnerSize,
        CommandKind::SetMinInnerSize,
        CommandKind::SetMaxInnerSize,
        CommandKind::SetFullscreen,
        CommandKind::SetLevel,
        CommandKind::SetTheme,
        CommandKind::SetCursor,
        CommandKind::SetCursorGrab,
        CommandKind::SetIme,
        CommandKind::RequestUserAttention,
        CommandKind::RequestDraw,
        CommandKind::Destroy,
    ];

    assert_eq!(kinds.len(), 20);
    assert_eq!(
        Command::Open {
            request: WindowRequest::default(),
        }
        .kind(),
        CommandKind::Open
    );
    assert_eq!(
        Command::Open {
            request: WindowRequest::default(),
        }
        .target(),
        None
    );

    let targeted_commands = [
        (
            Command::SetTitle {
                id,
                title: String::from("title"),
            },
            CommandKind::SetTitle,
        ),
        (
            Command::SetPosition {
                id,
                position: Point { x: 1.0, y: 2.0 },
            },
            CommandKind::SetPosition,
        ),
        (
            Command::SetVisible { id, visible: true },
            CommandKind::SetVisible,
        ),
        (
            Command::SetResizable {
                id,
                resizable: true,
            },
            CommandKind::SetResizable,
        ),
        (
            Command::SetControls {
                id,
                controls: Controls::default(),
            },
            CommandKind::SetControls,
        ),
        (
            Command::SetDecorations {
                id,
                decorations: true,
            },
            CommandKind::SetDecorations,
        ),
        (
            Command::SetTransparent {
                id,
                transparent: false,
            },
            CommandKind::SetTransparent,
        ),
        (
            Command::SetInnerSize {
                id,
                size: Size {
                    width: 1.0,
                    height: 2.0,
                },
            },
            CommandKind::SetInnerSize,
        ),
        (
            Command::SetMinInnerSize { id, size: None },
            CommandKind::SetMinInnerSize,
        ),
        (
            Command::SetMaxInnerSize { id, size: None },
            CommandKind::SetMaxInnerSize,
        ),
        (
            Command::SetFullscreen {
                id,
                fullscreen: Fullscreen::None,
            },
            CommandKind::SetFullscreen,
        ),
        (
            Command::SetLevel {
                id,
                level: Level::Normal,
            },
            CommandKind::SetLevel,
        ),
        (
            Command::SetTheme {
                id,
                theme: Some(Theme::Dark),
            },
            CommandKind::SetTheme,
        ),
        (
            Command::SetCursor {
                id,
                cursor: Cursor::Hidden,
            },
            CommandKind::SetCursor,
        ),
        (
            Command::SetCursorGrab {
                id,
                grab: CursorGrab::None,
            },
            CommandKind::SetCursorGrab,
        ),
        (
            Command::SetIme {
                id,
                request: ImeRequest::Disable,
            },
            CommandKind::SetIme,
        ),
        (
            Command::RequestUserAttention { id },
            CommandKind::RequestUserAttention,
        ),
        (Command::RequestDraw { id }, CommandKind::RequestDraw),
        (Command::Destroy { id }, CommandKind::Destroy),
    ];

    for (command, kind) in targeted_commands {
        assert_eq!(command.kind(), kind);
        assert_eq!(command.target(), Some(id));
    }
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_feature_exposes_typed_tree_command() {
    let id = Id::from_u64(7);
    let update = accesskit::TreeUpdate {
        nodes: Vec::new(),
        tree: None,
        tree_id: accesskit::TreeId::ROOT,
        focus: accesskit::NodeId(42),
    };
    let capabilities = HostCapabilities::builder()
        .accessibility(CapabilitySupport::Supported)
        .build();
    let mut registry = Registry::default();
    let mut commands = Vec::new();
    let mut actions = Vec::new();
    let mut clipboard = MemoryClipboard::new();
    Context::new(
        &mut registry,
        &mut commands,
        &mut actions,
        &mut clipboard,
        &capabilities,
        None,
    )
    .update_accessibility(id, update.clone());
    let command = commands
        .pop()
        .expect("accessibility update should queue a typed command");
    let plan = HostCommandPlan::from_command(command, &capabilities)
        .expect("supported accessibility update should produce a host plan");

    assert_eq!(plan.kind(), CommandKind::UpdateAccessibility);
    assert_eq!(plan.target(), Some(id));
    assert_eq!(
        plan.capability_decisions(),
        [CapabilityDecision::new(
            CapabilityKind::Accessibility,
            CapabilitySupport::Supported,
        )]
    );
    assert_eq!(
        take_planned_command(plan).into_command(),
        Command::UpdateAccessibility { id, update }
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_action_surface_wraps_accesskit_request() {
    fn assert_clone_and_partial_eq<T: Clone + PartialEq>() {}

    assert_clone_and_partial_eq::<AccessibilityActionRequest>();

    let id = Id::from_u64(7);
    let request = accesskit::ActionRequest {
        action: accesskit::Action::SetValue,
        target_tree: accesskit::TreeId::ROOT,
        target_node: accesskit::NodeId(42),
        data: Some(accesskit::ActionData::NumericValue(1.5)),
    };
    let event = AccessibilityEvent::ActionRequested(AccessibilityActionRequest {
        id,
        request: request.clone(),
    });

    assert_eq!(event.id(), id);
    let AccessibilityEvent::ActionRequested(action) = event else {
        panic!("action request must retain its typed event variant");
    };
    assert_eq!(action.id, id);
    assert_eq!(action.request, request);
}

#[test]
fn host_command_plan_preserves_clone_equality_and_opaque_diagnostics() {
    fn assert_clone_and_partial_eq<T: Clone + PartialEq>() {}

    assert_clone_and_partial_eq::<HostCommandPlan>();

    let plan = HostCommandPlan::from_command(
        Command::SetTitle {
            id: Id::from_u64(7),
            title: String::from("opaque title"),
        },
        &conservative_capabilities(),
    )
    .expect("supported command should produce a host plan");
    let diagnostic = format!("{plan:?}");
    let cloned = plan.clone();

    assert_eq!(cloned, plan);
    assert_eq!(format!("{cloned:?}"), diagnostic);
    assert_eq!(
        diagnostic,
        "HostCommandPlan { kind: SetTitle, target: Some(Id(7)), capability_decisions: [CapabilityDecision { kind: Window(SetTitle), support: Supported }] }"
    );
}

#[test]
fn host_command_plan_records_all_relevant_capability_decisions() {
    let id = Id::from_u64(8);
    let capabilities = HostCapabilities::builder()
        .role(RoleKind::Root, CapabilitySupport::RuntimeDependent)
        .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::Borderless, CapabilitySupport::Supported)
        .fullscreen(
            FullscreenMode::Exclusive,
            CapabilitySupport::RuntimeDependent,
        )
        .cursor(
            CursorCapability::Custom,
            CapabilitySupport::RuntimeDependent,
        )
        .window(
            WindowOperation::SetTitle,
            CapabilitySupport::RuntimeDependent,
        )
        .build();

    let decisions = |plan: HostCommandPlan| {
        plan.capability_decisions()
            .iter()
            .map(|decision| (decision.kind(), decision.support()))
            .collect::<Vec<_>>()
    };

    let open = HostCommandPlan::from_command(
        Command::Open {
            request: WindowRequest::builder("planned")
                .fullscreen(Fullscreen::Borderless)
                .build(),
        },
        &capabilities,
    )
    .expect("supported and runtime-dependent open capabilities should plan");
    assert_eq!(
        decisions(open),
        vec![
            (
                CapabilityKind::Role(RoleKind::Root),
                CapabilitySupport::RuntimeDependent,
            ),
            (
                CapabilityKind::Fullscreen(FullscreenMode::Borderless),
                CapabilitySupport::Supported,
            ),
        ]
    );

    let fullscreen = HostCommandPlan::from_command(
        Command::SetFullscreen {
            id,
            fullscreen: Fullscreen::Exclusive,
        },
        &capabilities,
    )
    .expect("runtime-dependent fullscreen should plan");
    assert_eq!(
        decisions(fullscreen),
        vec![(
            CapabilityKind::Fullscreen(FullscreenMode::Exclusive),
            CapabilitySupport::RuntimeDependent,
        )]
    );

    let cursor = HostCommandPlan::from_command(
        Command::SetCursor {
            id,
            cursor: Cursor::Custom(CustomCursorId::from_u64(12)),
        },
        &capabilities,
    )
    .expect("runtime-dependent cursor should plan");
    assert_eq!(
        decisions(cursor),
        vec![(
            CapabilityKind::Cursor(CursorCapability::Custom),
            CapabilitySupport::RuntimeDependent,
        ),]
    );

    let title = HostCommandPlan::from_command(
        Command::SetTitle {
            id,
            title: String::from("Runtime-dependent title"),
        },
        &capabilities,
    )
    .expect("runtime-dependent title should plan");
    assert_eq!(
        decisions(title),
        vec![(
            CapabilityKind::Window(WindowOperation::SetTitle),
            CapabilitySupport::RuntimeDependent,
        )]
    );

    let error = HostCommandPlan::from_command(
        Command::SetCursor {
            id,
            cursor: Cursor::Custom(CustomCursorId::from_u64(12)),
        },
        &HostCapabilities::builder().build(),
    )
    .expect_err("unsupported capability must be rejected before a plan exists");
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
}

#[test]
fn host_command_plan_backend_applicators_accept_only_plans() {
    let fake = include_str!("testing.rs");
    const FAKE_APPLICATOR_SIGNATURE: &str =
        "    pub(crate) fn apply_plan(&mut self, plan: HostCommandPlan) -> Result<()> {";
    let fake_start = fake
        .find(FAKE_APPLICATOR_SIGNATURE)
        .expect("fake host should retain one private plan applicator");
    let fake_end = fake[fake_start..]
        .find("fn apply_open_request")
        .map(|offset| fake_start + offset)
        .expect("fake host applicator should end before open application");
    let fake_applicator = &fake[fake_start..fake_end];
    assert!(fake_applicator.contains("plan: HostCommandPlan"));
    assert!(!fake_applicator.contains("NormalizedCommand"));
    assert!(!fake.contains("pub fn apply_plan"));
    assert!(fake.contains("pub(crate) fn apply_plan"));

    let native = include_str!("winit_adapter.rs");
    const NATIVE_APPLICATOR_SIGNATURE: &str = "    fn apply_host_command_with_follow_ups(\n        &mut self,\n        event_loop: &winit::event_loop::ActiveEventLoop,\n        plan: HostCommandPlan,\n        callbacks: &mut Vec<BackendCallback>,\n    ) -> Result<()> {";
    let native_start = native
        .find(NATIVE_APPLICATOR_SIGNATURE)
        .expect("native backend should retain one private plan applicator");
    let native_end = native[native_start..]
        .find("fn handle")
        .map(|offset| native_start + offset)
        .expect("native host applicator should end before handle lookup");
    let native_applicator = &native[native_start..native_end];
    assert!(native_applicator.contains("plan: HostCommandPlan"));
    assert!(!native_applicator.contains("NormalizedCommand"));
    assert!(!native.contains("pub fn apply_host_command"));
    assert!(!native.contains("pub(crate) fn apply_host_command"));
}

#[test]
fn ingress_proxy_rejects_malformed_command_before_queueing() {
    let queue = Arc::new(ProxyQueue::new());
    let proxy = Proxy::with_queue(queue.clone());
    let error = proxy
        .send(Command::Open {
            request: WindowRequest::builder("").build(),
        })
        .expect_err("malformed proxy commands must fail before queue submission");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(error.command_kind, Some(CommandKind::Open));
    assert_eq!(error.id, None);
    assert_eq!(error.completed_prefix, 0);
    assert!(queue.pop().is_none());
}

#[test]
fn ingress_callback_proxy_direct_and_test_entries_share_planner() {
    fn diagnostics(error: &Error) -> (ErrorCode, Option<CommandKind>, Option<Id>, usize, String) {
        (
            error.code,
            error.command_kind,
            error.id,
            error.completed_prefix,
            error.message.clone(),
        )
    }

    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct CommandCallback(Command);

    impl Handler for CommandCallback {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.send(self.0.clone());
            Ok(())
        }
    }

    struct Noop;

    impl Handler for Noop {}

    fn direct_result(command: Command) -> Result<HostCommandPlan> {
        let mut runner = WinitRunner::from_loop(Loop::new(Noop));
        runner.resolve_capabilities_for_test(conservative_capabilities());
        runner.plan_command_for_test(command)
    }

    fn callback_result(command: Command) -> Result<()> {
        let mut runner = WinitRunner::from_loop(Loop::new(CommandCallback(command)));
        let mut capabilities = CapabilityProvider(conservative_capabilities());
        let _ = runner.resume_with_capability_provider_for_test(&mut capabilities);
        runner.into_terminal_result()
    }

    fn proxy_result(command: Command) -> Result<()> {
        let queue = Arc::new(ProxyQueue::new());
        let proxy = Proxy::with_queue(queue.clone());
        proxy.send(command)?;
        let event = queue
            .pop()
            .expect("accepted proxy command should enqueue one user event");
        let UserEvent::Command(command) = event else {
            panic!("accepted proxy command should enqueue a normalized command event");
        };

        let mut runner = WinitRunner::from_loop(Loop::new(Noop));
        runner.resolve_capabilities_for_test(conservative_capabilities());
        runner.apply_proxy_command_for_test(command);
        runner.into_terminal_result()
    }

    let valid = Command::Open {
        request: WindowRequest::builder("ingress-valid").build(),
    };
    let malformed = Command::Open {
        request: WindowRequest::builder("").build(),
    };
    let unsupported = Command::Open {
        request: WindowRequest::builder("ingress-dialog")
            .dialog(Id::from_u64(1))
            .build(),
    };
    let unknown = Command::SetTitle {
        id: Id::from_u64(91),
        title: String::from("missing"),
    };

    let direct_plan = direct_result(valid.clone()).expect("valid direct command should plan");
    assert_eq!(direct_plan.kind(), CommandKind::Open);
    assert_eq!(direct_plan.target(), None);
    assert_eq!(
        direct_plan
            .capability_decisions()
            .iter()
            .map(|decision| (decision.kind(), decision.support()))
            .collect::<Vec<_>>(),
        vec![
            (
                CapabilityKind::Role(RoleKind::Root),
                CapabilitySupport::Supported,
            ),
            (
                CapabilityKind::Fullscreen(FullscreenMode::None),
                CapabilitySupport::Supported,
            )
        ]
    );

    let mut host = testing::Host::new();
    host.apply(valid.clone())
        .expect("valid synchronous host command should plan and apply");
    assert_eq!(host.commands(), std::slice::from_ref(&valid));
    callback_result(valid.clone()).expect("valid callback command should plan and apply");
    proxy_result(valid).expect("valid proxy command should plan and apply after queue acceptance");

    let direct_malformed = direct_result(malformed.clone())
        .expect_err("malformed direct command should fail intrinsic normalization");
    let mut malformed_host = testing::Host::new();
    let host_malformed = malformed_host
        .apply(malformed.clone())
        .expect_err("malformed synchronous host command should fail intrinsic normalization");
    let callback_malformed = callback_result(malformed.clone())
        .expect_err("malformed callback command should fail intrinsic normalization");
    let malformed_queue = Arc::new(ProxyQueue::new());
    let malformed_proxy = Proxy::with_queue(malformed_queue.clone());
    let proxy_malformed = malformed_proxy
        .send(malformed)
        .expect_err("malformed proxy command should fail intrinsic normalization before queueing");
    assert_eq!(diagnostics(&direct_malformed), diagnostics(&host_malformed));
    assert_eq!(
        diagnostics(&direct_malformed),
        diagnostics(&callback_malformed)
    );
    assert_eq!(
        diagnostics(&direct_malformed),
        diagnostics(&proxy_malformed)
    );
    assert!(malformed_queue.pop().is_none());

    let direct_unsupported = direct_result(unsupported.clone())
        .expect_err("unsupported direct command should fail capability planning");
    let mut unsupported_host = testing::Host::new();
    let host_unsupported = unsupported_host
        .apply(unsupported.clone())
        .expect_err("unsupported synchronous host command should fail capability planning");
    let callback_unsupported = callback_result(unsupported.clone())
        .expect_err("unsupported callback command should fail capability planning");
    assert_eq!(
        diagnostics(&direct_unsupported),
        diagnostics(&host_unsupported)
    );
    assert_eq!(
        diagnostics(&direct_unsupported),
        diagnostics(&callback_unsupported)
    );
    let proxy_unsupported = proxy_result(unsupported)
        .expect_err("proxy capability failure must be reported by event-loop planning");
    assert_eq!(
        diagnostics(&direct_unsupported),
        diagnostics(&proxy_unsupported)
    );

    let direct_unknown = direct_result(unknown.clone())
        .expect_err("unknown direct command target should fail runtime planning");
    let mut unknown_host = testing::Host::new();
    let host_unknown = unknown_host
        .apply(unknown.clone())
        .expect_err("unknown synchronous host target should fail runtime planning");
    let callback_unknown = callback_result(unknown.clone())
        .expect_err("unknown callback target should fail runtime planning");
    assert_eq!(diagnostics(&direct_unknown), diagnostics(&host_unknown));
    assert_eq!(diagnostics(&direct_unknown), diagnostics(&callback_unknown));
    proxy_result(unknown)
        .expect("unknown proxy targets should become no-ops during event-loop ingress");
}

#[test]
fn ingress_low_level_applicators_cannot_apply_normalized_payloads() {
    let fake = include_str!("testing.rs");
    const FAKE_APPLICATOR_SIGNATURE: &str =
        "    pub(crate) fn apply_plan(&mut self, plan: HostCommandPlan) -> Result<()> {";
    let fake_start = fake
        .find(FAKE_APPLICATOR_SIGNATURE)
        .expect("fake host should retain one private plan applicator");
    let fake_end = fake[fake_start..]
        .find("fn apply_open_request")
        .map(|offset| fake_start + offset)
        .expect("fake host applicator should end before open application");
    let fake_applicator = &fake[fake_start..fake_end];
    assert!(fake_applicator.contains("plan: HostCommandPlan"));
    assert!(!fake_applicator.contains("Command)"));
    assert!(!fake_applicator.contains("NormalizedCommand"));

    let native = include_str!("winit_adapter.rs");
    const NATIVE_APPLICATOR_SIGNATURE: &str = "    fn apply_host_command_with_follow_ups(\n        &mut self,\n        event_loop: &winit::event_loop::ActiveEventLoop,\n        plan: HostCommandPlan,\n        callbacks: &mut Vec<BackendCallback>,\n    ) -> Result<()> {";
    let native_start = native
        .find(NATIVE_APPLICATOR_SIGNATURE)
        .expect("native backend should retain one private plan applicator");
    let native_end = native[native_start..]
        .find("fn handle")
        .map(|offset| native_start + offset)
        .expect("native host applicator should end before handle lookup");
    let native_applicator = &native[native_start..native_end];
    assert!(native_applicator.contains("plan: HostCommandPlan"));
    assert!(!native_applicator.contains("Command)"));
    assert!(!native_applicator.contains("NormalizedCommand"));

    let registry = include_str!("registry.rs");
    assert!(registry.contains("Command(NormalizedCommand)"));
    assert!(!registry.contains("Command(Command)"));
}

#[test]
fn pump_handler_failure_is_returned_from_loop_terminal_state() {
    struct FailingHandler;

    impl Handler for FailingHandler {
        fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
            Err(Error::new(ErrorCode::CommandFailed, "handler failure"))
        }
    }

    let mut runner = WinitRunner::from_loop(Loop::new(FailingHandler));
    runner.resolve_capabilities_for_test(conservative_capabilities());
    runner.deliver_event_to_pump_for_test(EventKind::Focused {
        id: Id::from_u64(1),
        focused: true,
    });
    runner.drain_pump_for_test();

    let error = runner
        .into_terminal_result()
        .expect_err("the runner returns its retained first handler failure");
    assert_eq!(error.message, "handler failure");
}

#[test]
fn pump_startup_and_resume_apply_in_order_after_joint_preflight() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    #[derive(Default)]
    struct Record {
        startup_was_live_during_resume: bool,
        created: Vec<String>,
        ready: Vec<String>,
        startup_ready_state: Option<(String, Size)>,
    }

    struct Recorder(Rc<RefCell<Record>>);

    impl Handler for Recorder {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            self.0.borrow_mut().startup_was_live_during_resume =
                context.state("startup-min").is_some();
            context
                .window(Id::from_u64(1))
                .title("Startup after resume")
                .min(Some(size(500, 400)))
                .size(size(900, 200));
            context.window(Id::from_u64(2)).max(Some(size(500, 450)));
            context.open(open("resume"));
            Ok(())
        }

        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if let EventKind::Created(state) = event.event() {
                self.0
                    .borrow_mut()
                    .created
                    .push(state.name().expect("test windows are named").to_owned());
            }
            Ok(())
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            let state = ready.state();
            let name = state.name().expect("test windows are named").to_owned();
            let mut record = self.0.borrow_mut();
            if name == "startup-min" {
                record.startup_ready_state =
                    Some((state.title().to_owned(), state.metrics().logical_size()));
            }
            record.ready.push(name);
            Ok(())
        }
    }

    let record = Rc::new(RefCell::new(Record::default()));
    let mut window_loop = Loop::new(Recorder(record.clone()));
    window_loop.startup.push(
        open("startup-min")
            .size(size(600, 300))
            .min(Some(size(400, 300)))
            .max(Some(size(800, 500)))
            .into(),
    );
    window_loop.startup.push(
        open("startup-max")
            .size(size(700, 600))
            .min(Some(size(400, 350)))
            .max(Some(size(800, 700)))
            .into(),
    );
    let mut runner = WinitRunner::from_loop(window_loop);
    let mut provider = CapabilityProvider(conservative_capabilities());

    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("first resume should complete");

    let record = record.borrow();
    assert!(!record.startup_was_live_during_resume);
    assert_eq!(record.created, ["startup-min", "startup-max", "resume"]);
    assert_eq!(record.ready, ["startup-min", "startup-max", "resume"]);
    assert_eq!(
        record.startup_ready_state,
        Some((
            String::from("Startup after resume"),
            Size {
                width: 800.0,
                height: 400.0,
            },
        ))
    );
    assert!(runner.registry_contains_for_test(Id::from_u64(1)));
    assert!(runner.registry_contains_for_test(Id::from_u64(2)));
    assert!(runner.registry_contains_for_test(Id::from_u64(3)));
    let startup_id = Id::from_u64(1);
    let startup = runner
        .registry_snapshot_for_test(startup_id)
        .expect("startup window remains live after resume");
    assert_eq!(startup.title(), "Startup after resume");
    assert_eq!(
        startup.metrics().logical_size(),
        Size {
            width: 800.0,
            height: 400.0,
        }
    );
    let bounds = runner
        .inner_size_bounds_for_test(startup_id)
        .expect("startup bounds remain available");
    assert_eq!(
        bounds.minimum,
        Some(Size {
            width: 500.0,
            height: 400.0,
        })
    );
    assert_eq!(
        bounds.maximum,
        Some(Size {
            width: 800.0,
            height: 500.0,
        })
    );
    let startup_max_id = Id::from_u64(2);
    let startup_max = runner
        .registry_snapshot_for_test(startup_max_id)
        .expect("startup max window remains live after resume");
    assert_eq!(
        startup_max.metrics().logical_size(),
        Size {
            width: 500.0,
            height: 450.0,
        }
    );
    let bounds = runner
        .inner_size_bounds_for_test(startup_max_id)
        .expect("startup max bounds remain available");
    assert_eq!(
        bounds.minimum,
        Some(Size {
            width: 400.0,
            height: 350.0,
        })
    );
    assert_eq!(
        bounds.maximum,
        Some(Size {
            width: 500.0,
            height: 450.0,
        })
    );
    let applied = runner.applied_commands_for_test();
    assert_eq!(
        applied.iter().map(Command::kind).collect::<Vec<_>>(),
        [
            CommandKind::Open,
            CommandKind::Open,
            CommandKind::SetTitle,
            CommandKind::SetMinInnerSize,
            CommandKind::SetInnerSize,
            CommandKind::SetMaxInnerSize,
            CommandKind::Open,
        ]
    );
    assert_eq!(
        applied.iter().map(Command::target).collect::<Vec<_>>(),
        [
            None,
            None,
            Some(startup_id),
            Some(startup_id),
            Some(startup_id),
            Some(startup_max_id),
            None,
        ]
    );
    assert!(matches!(
        applied[4],
        Command::SetInnerSize { id, size }
            if id == startup_id
                && size
                    == Size {
                        width: 800.0,
                        height: 400.0,
                    }
    ));
}

#[test]
fn pump_startup_capability_failure_skips_resume_and_native_effects() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct ResumeRecorder(Rc<RefCell<usize>>);

    impl Handler for ResumeRecorder {
        fn resume(&mut self, _context: &mut Context<'_>) -> Result<()> {
            *self.0.borrow_mut() += 1;
            Ok(())
        }
    }

    let resume_calls = Rc::new(RefCell::new(0));
    let mut window_loop = Loop::new(ResumeRecorder(resume_calls.clone()));
    window_loop.startup.push(open("startup").into());
    let mut runner = WinitRunner::from_loop(window_loop);
    let mut provider = CapabilityProvider(HostCapabilities::builder().build());
    let initial_high_water = runner.allocator_high_water_for_test();
    assert!(runner.applied_commands_for_test().is_empty());

    assert!(
        runner
            .resume_with_capability_provider_for_test(&mut provider)
            .is_err()
    );
    assert_eq!(*resume_calls.borrow(), 0);
    assert!(!runner.registry_contains_for_test(Id::from_u64(1)));
    assert!(runner.applied_commands_for_test().is_empty());
    assert_eq!(
        runner.allocator_high_water_for_test(),
        initial_high_water,
        "startup preflight rejection must not mutate the real allocator"
    );
    assert_eq!(
        runner
            .into_terminal_result()
            .expect_err("startup capability rejection is terminal")
            .code,
        ErrorCode::UnsupportedFeature
    );
}

#[test]
fn pump_resume_failure_applies_no_startup_or_callback_work() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct FailingResume;

    impl Handler for FailingResume {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.open(open("callback"));
            Err(Error::new(ErrorCode::CommandFailed, "resume failure"))
        }
    }

    let mut window_loop = Loop::new(FailingResume);
    window_loop.startup.push(open("startup").into());
    let mut runner = WinitRunner::from_loop(window_loop);
    let mut provider = CapabilityProvider(conservative_capabilities());

    assert!(
        runner
            .resume_with_capability_provider_for_test(&mut provider)
            .is_err()
    );
    assert!(!runner.registry_contains_for_test(Id::from_u64(1)));
    assert!(!runner.registry_contains_for_test(Id::from_u64(2)));
    assert_eq!(
        runner
            .into_terminal_result()
            .expect_err("resume error is retained")
            .message,
        "resume failure"
    );
}

#[test]
fn pump_created_failure_skips_ready() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct FailingCreated(Rc<RefCell<usize>>);

    impl Handler for FailingCreated {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                return Err(Error::new(ErrorCode::CommandFailed, "created failure"));
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            *self.0.borrow_mut() += 1;
            Ok(())
        }
    }

    let ready_calls = Rc::new(RefCell::new(0));
    let mut window_loop = Loop::new(FailingCreated(ready_calls.clone()));
    window_loop.startup.push(open("startup").into());
    let mut runner = WinitRunner::from_loop(window_loop);
    let mut provider = CapabilityProvider(conservative_capabilities());

    assert!(
        runner
            .resume_with_capability_provider_for_test(&mut provider)
            .is_err()
    );
    assert_eq!(*ready_calls.borrow(), 0);
    assert_eq!(
        runner
            .into_terminal_result()
            .expect_err("created error is retained")
            .message,
        "created failure"
    );
}

#[test]
fn pump_created_exit_skips_ready_and_returns_success_after_teardown() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct ExitingCreated(Rc<RefCell<usize>>);

    impl Handler for ExitingCreated {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                event.context_mut().open(open("before-exit"));
                event.exit();
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            *self.0.borrow_mut() += 1;
            Ok(())
        }
    }

    let ready_calls = Rc::new(RefCell::new(0));
    let mut window_loop = Loop::new(ExitingCreated(ready_calls.clone()));
    window_loop.startup.push(open("startup").into());
    let mut runner = WinitRunner::from_loop(window_loop);
    let mut provider = CapabilityProvider(conservative_capabilities());

    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("created exit is a successful terminal result");
    assert_eq!(*ready_calls.borrow(), 0);
    assert!(runner.registry_contains_for_test(Id::from_u64(2)));
    assert!(runner.into_terminal_result().is_ok());
}

#[test]
fn pump_later_resume_delivers_window_events_before_global_callback() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct Recorder(Rc<RefCell<Vec<String>>>);

    impl Handler for Recorder {
        fn resume(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.0.borrow_mut().push(String::from("resume"));
            Ok(())
        }

        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if let EventKind::Resumed(id) = event.event() {
                self.0.borrow_mut().push(format!("resumed:{}", id.as_u64()));
            }
            Ok(())
        }
    }

    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut window_loop = Loop::new(Recorder(trace.clone()));
    window_loop.startup.push(open("first").into());
    window_loop.startup.push(open("second").into());
    let mut runner = WinitRunner::from_loop(window_loop);
    let mut provider = CapabilityProvider(conservative_capabilities());

    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("first resume opens startup windows");
    trace.borrow_mut().clear();
    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("later resume succeeds");

    assert_eq!(
        trace.borrow().as_slice(),
        ["resumed:1", "resumed:2", "resume"]
    );
}

#[test]
fn capabilities_first_resume_resolves_before_handler() {
    struct CountingProvider {
        report: HostCapabilities,
        calls: Rc<RefCell<usize>>,
        trace: Rc<RefCell<Vec<&'static str>>>,
    }

    impl super::capability::CapabilityProvider for CountingProvider {
        fn resolve(&mut self) -> HostCapabilities {
            *self.calls.borrow_mut() += 1;
            self.trace.borrow_mut().push("resolve");
            self.report.clone()
        }
    }

    struct ResumeObserver {
        expected: HostCapabilities,
        trace: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Handler for ResumeObserver {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            self.trace.borrow_mut().push("resume");
            assert_eq!(context.capabilities(), &self.expected);
            Ok(())
        }
    }

    let report = conservative_capabilities();
    let calls = Rc::new(RefCell::new(0));
    let trace = Rc::new(RefCell::new(Vec::new()));
    let handler = ResumeObserver {
        expected: report.clone(),
        trace: trace.clone(),
    };
    let mut runner = WinitRunner::from_loop(Loop::new(handler));
    let mut provider = CountingProvider {
        report,
        calls: calls.clone(),
        trace: trace.clone(),
    };

    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("first resume callback succeeds");
    assert!(runner.startup_is_staged_for_test());
    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("later resume callback succeeds");

    assert_eq!(*calls.borrow(), 1);
    assert_eq!(trace.borrow().as_slice(), ["resolve", "resume", "resume"]);
}

#[test]
fn capabilities_pre_resume_suspend_skips_handler() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct SuspendObserver {
        reports: Rc<RefCell<Vec<HostCapabilities>>>,
    }

    impl Handler for SuspendObserver {
        fn suspend(&mut self, context: &mut Context<'_>) -> Result<()> {
            self.reports
                .borrow_mut()
                .push(context.capabilities().clone());
            Ok(())
        }
    }

    let report = conservative_capabilities();
    let reports = Rc::new(RefCell::new(Vec::new()));
    let handler = SuspendObserver {
        reports: reports.clone(),
    };
    let mut runner = WinitRunner::from_loop(Loop::new(handler));

    assert_eq!(
        runner
            .suspend_for_test()
            .expect("pre-resume suspend must not fail"),
        Action::Wait
    );
    assert!(reports.borrow().is_empty());

    let mut provider = CapabilityProvider(report.clone());
    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("resume resolves capabilities before later callbacks");

    assert_eq!(
        runner
            .suspend_for_test()
            .expect("post-resume suspend succeeds"),
        Action::Wait
    );
    assert_eq!(reports.borrow().as_slice(), &[report]);
}

#[test]
fn capabilities_pre_resume_idle_skips_handler() {
    struct CapabilityProvider(HostCapabilities);

    impl super::capability::CapabilityProvider for CapabilityProvider {
        fn resolve(&mut self) -> HostCapabilities {
            self.0.clone()
        }
    }

    struct IdleObserver {
        reports: Rc<RefCell<Vec<HostCapabilities>>>,
    }

    impl Handler for IdleObserver {
        fn wants_idle(&self) -> bool {
            true
        }

        fn idle(&mut self, context: &mut Context<'_>) -> Result<()> {
            self.reports
                .borrow_mut()
                .push(context.capabilities().clone());
            Ok(())
        }
    }

    let report = conservative_capabilities();
    let reports = Rc::new(RefCell::new(Vec::new()));
    let handler = IdleObserver {
        reports: reports.clone(),
    };
    let mut runner = WinitRunner::from_loop(Loop::new(handler));

    assert_eq!(
        runner
            .idle_for_test()
            .expect("pre-resume idle must not fail"),
        Some(Action::Wait)
    );
    assert!(reports.borrow().is_empty());

    let mut provider = CapabilityProvider(report.clone());
    runner
        .resume_with_capability_provider_for_test(&mut provider)
        .expect("resume resolves capabilities before later callbacks");

    assert_eq!(
        runner.idle_for_test().expect("post-resume idle succeeds"),
        Some(Action::Wait)
    );
    assert_eq!(reports.borrow().as_slice(), &[report]);
}

#[test]
fn capabilities_have_no_backend_default_constructor() {
    let capabilities = HostCapabilities::builder().build();

    assert_eq!(
        capabilities.role(RoleKind::Root),
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.fullscreen(FullscreenMode::Borderless),
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.cursor(CursorCapability::Icon),
        CapabilitySupport::Unsupported
    );
}

#[test]
fn proxy_public_front_door_exposes_typed_cross_thread_commands() {
    fn assert_proxy_methods(proxy: &Proxy, id: Id, time: Instant) {
        let command = Command::SetTitle {
            id,
            title: String::from("Renamed"),
        };

        let _type_checked = || -> Result<()> {
            proxy.send(command.clone())?;
            proxy.open(open("worker-window"))?;
            proxy.close(id)?;
            proxy.draw(id)?;
            proxy.again(id)?;
            proxy.at(id, time)?;
            proxy.exit()
        };
    }

    let _front_door: fn(&Proxy, Id, Instant) = assert_proxy_methods;
}

#[test]
fn metrics_convert_physical_to_logical() {
    let metrics = Metrics::from_physical_size(
        Id::from_u64(7),
        PhysicalSize {
            width: 300,
            height: 150,
        },
        1.5,
    )
    .expect("test metrics are valid");

    assert_eq!(
        metrics.logical_size(),
        Size {
            width: 200.0,
            height: 100.0
        }
    );
    assert_eq!(metrics.scale_factor(), 1.5);
}

#[test]
fn metrics_convert_points_between_logical_and_physical_space() {
    let metrics = Metrics::from_physical_size(
        Id::from_u64(7),
        PhysicalSize {
            width: 300,
            height: 150,
        },
        1.5,
    )
    .expect("test metrics are valid");

    let physical = metrics.logical_to_physical_point(Point { x: 20.0, y: 10.0 });

    assert_eq!(physical, PhysicalPoint { x: 30, y: 15 });
    assert_eq!(
        metrics.physical_to_logical_point(physical),
        Point { x: 20.0, y: 10.0 }
    );
}

#[test]
fn metrics_preserve_outer_geometry() {
    let metrics = Metrics::from_physical_size(
        Id::from_u64(7),
        PhysicalSize {
            width: 300,
            height: 150,
        },
        1.5,
    )
    .expect("test metrics are valid")
    .with_outer_geometry(
        Some(Point { x: 10.0, y: 20.0 }),
        Some(Size {
            width: 220.0,
            height: 120.0,
        }),
    )
    .expect("test outer geometry is valid");

    assert_eq!(metrics.outer_position(), Some(Point { x: 10.0, y: 20.0 }));
    assert_eq!(
        metrics.outer_size(),
        Some(Size {
            width: 220.0,
            height: 120.0
        })
    );
}

#[test]
fn metrics_reject_invalid_scale_and_overflowed_logical_size() {
    for scale_factor in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.0,
        -0.0,
        -1.0,
        f64::MIN_POSITIVE,
    ] {
        let error = Metrics::from_physical_size(
            Id::from_u64(8),
            PhysicalSize {
                width: 640,
                height: 480,
            },
            scale_factor,
        )
        .expect_err("invalid observed scale must be rejected without coercion");

        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }
}

#[test]
fn metrics_validate_observed_geometry_and_canonicalize_signed_zero() {
    let metrics = Metrics::from_physical_size(
        Id::from_u64(8),
        PhysicalSize {
            width: 640,
            height: 480,
        },
        1.0,
    )
    .expect("test metrics are valid");

    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for position in [Point { x: invalid, y: 0.0 }, Point { x: 0.0, y: invalid }] {
            let error = metrics
                .clone()
                .with_outer_geometry(Some(position), None)
                .expect_err("non-finite observed outer position must be rejected");
            assert_eq!(error.code, ErrorCode::InvalidRequest);
        }

        for size in [
            Size {
                width: invalid,
                height: 0.0,
            },
            Size {
                width: 0.0,
                height: invalid,
            },
        ] {
            let error = metrics
                .clone()
                .with_outer_geometry(None, Some(size))
                .expect_err("non-finite observed outer size must be rejected");
            assert_eq!(error.code, ErrorCode::InvalidRequest);
        }

        for safe_area in [
            Insets {
                top: invalid,
                ..Insets::default()
            },
            Insets {
                right: invalid,
                ..Insets::default()
            },
            Insets {
                bottom: invalid,
                ..Insets::default()
            },
            Insets {
                left: invalid,
                ..Insets::default()
            },
        ] {
            let error = metrics
                .clone()
                .with_safe_area(safe_area)
                .expect_err("non-finite observed safe-area inset must be rejected");
            assert_eq!(error.code, ErrorCode::InvalidRequest);
        }
    }

    for size in [
        Size {
            width: -1.0,
            height: 0.0,
        },
        Size {
            width: 0.0,
            height: -1.0,
        },
    ] {
        let error = metrics
            .clone()
            .with_outer_geometry(None, Some(size))
            .expect_err("negative observed outer size must be rejected");
        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }

    for safe_area in [
        Insets {
            top: -1.0,
            ..Insets::default()
        },
        Insets {
            right: -1.0,
            ..Insets::default()
        },
        Insets {
            bottom: -1.0,
            ..Insets::default()
        },
        Insets {
            left: -1.0,
            ..Insets::default()
        },
    ] {
        let error = metrics
            .clone()
            .with_safe_area(safe_area)
            .expect_err("negative observed safe-area inset must be rejected");
        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }

    let negative_outer_position = metrics
        .clone()
        .with_outer_geometry(Some(Point { x: -4.0, y: -5.0 }), None)
        .expect("finite negative outer positions are valid");
    assert_eq!(
        negative_outer_position.outer_position(),
        Some(Point { x: -4.0, y: -5.0 })
    );

    let metrics = Metrics::from_physical_size(
        Id::from_u64(9),
        PhysicalSize {
            width: 0,
            height: 0,
        },
        1.0,
    )
    .expect("zero physical metrics are valid")
    .with_outer_geometry(
        Some(Point { x: -0.0, y: -0.0 }),
        Some(Size {
            width: -0.0,
            height: -0.0,
        }),
    )
    .expect("finite negative outer positions and signed-zero sizes are valid")
    .with_safe_area(Insets {
        top: -0.0,
        right: -0.0,
        bottom: -0.0,
        left: -0.0,
    })
    .expect("signed-zero safe-area insets are valid");

    let logical_size = metrics.logical_size();
    assert_eq!(logical_size, Size::default());
    assert!(logical_size.width.is_sign_positive());
    assert!(logical_size.height.is_sign_positive());

    let outer_position = metrics
        .outer_position()
        .expect("outer position is retained");
    assert_eq!(outer_position.x, 0.0);
    assert_eq!(outer_position.y, 0.0);
    assert!(outer_position.x.is_sign_positive());
    assert!(outer_position.y.is_sign_positive());

    let outer_size = metrics.outer_size().expect("outer size is retained");
    assert_eq!(outer_size, Size::default());
    assert!(outer_size.width.is_sign_positive());
    assert!(outer_size.height.is_sign_positive());

    let safe_area = metrics.safe_area();
    assert_eq!(safe_area, Insets::default());
    assert!(safe_area.top.is_sign_positive());
    assert!(safe_area.right.is_sign_positive());
    assert!(safe_area.bottom.is_sign_positive());
    assert!(safe_area.left.is_sign_positive());
}

#[test]
fn registry_tracks_multiple_windows() {
    let mut registry = Registry::new();
    let first = registry.reserve_id().expect("first identity reserves");
    let second = registry.reserve_id().expect("second identity reserves");

    registry
        .insert(Instance::new(state(first)))
        .expect("first instance inserts");
    registry
        .insert(Instance::new(state(second)))
        .expect("second instance inserts");

    assert_eq!(registry.len(), 2);
    assert!(registry.contains(first));
    assert_eq!(registry.remove(first).map(|entry| entry.id()), Some(first));
    assert!(!registry.contains(first));
    assert!(registry.contains(second));
}

#[test]
fn registry_rejects_reserved_identity_after_removal_without_reuse() {
    let mut registry = Registry::new();
    let id = registry.reserve_id().expect("identity reserves");
    registry
        .insert(Instance::new(state(id)))
        .expect("reserved identity inserts once");
    assert_eq!(registry.remove(id).map(|instance| instance.id()), Some(id));

    let error = registry
        .insert(Instance::new(state(id)))
        .expect_err("a removed reserved identity must not be reinserted");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert!(!registry.contains(id));
    assert_eq!(
        registry
            .reserve_id()
            .expect("removed identity is not reused"),
        Id::from_u64(2)
    );
}

#[test]
fn registry_rejects_removed_explicit_fixture_identity() {
    let id = Id::from_u64(41);
    let mut registry = Registry::new();
    registry
        .insert(Instance::new(state(id)))
        .expect("fresh fixture identity inserts once");
    assert_eq!(registry.remove(id).map(|instance| instance.id()), Some(id));

    let error = registry
        .insert(Instance::new(state(id)))
        .expect_err("a removed explicit identity must not be reinserted");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert!(!registry.contains(id));
    assert_eq!(
        registry
            .reserve_id()
            .expect("fixture identity advances high water"),
        Id::from_u64(42)
    );
}

#[test]
fn registry_preserves_pending_reservation_after_duplicate_name_failure() {
    let mut registry = Registry::new();
    let existing_id = Id::from_u64(10);
    registry
        .insert(Instance::new(state(existing_id).named("shared")))
        .expect("existing name inserts");

    let reserved_id = registry.reserve_id().expect("identity reserves");
    let error = registry
        .insert(Instance::new(state(reserved_id).named("shared")))
        .expect_err("duplicate name must not consume a reservation");
    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert!(!registry.contains(reserved_id));

    registry
        .insert(Instance::new(state(reserved_id).named("corrected")))
        .expect("corrected reserved identity inserts once");
    assert!(registry.contains(reserved_id));
    assert_eq!(
        registry
            .reserve_id()
            .expect("reservation advances allocation once"),
        Id::from_u64(12)
    );
}

#[test]
fn registry_duplicate_name_failure_does_not_issue_fresh_fixture_or_advance_high_water() {
    let mut registry = Registry::new();
    let existing_id = Id::from_u64(20);
    let fresh_id = Id::from_u64(100);
    registry
        .insert(Instance::new(state(existing_id).named("shared")))
        .expect("existing name inserts");

    let error = registry
        .insert(Instance::new(state(fresh_id).named("shared")))
        .expect_err("duplicate name must reject a fresh fixture identity atomically");
    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert!(!registry.contains(fresh_id));
    assert_eq!(
        registry
            .reserve_id()
            .expect("failed insertion does not advance high water"),
        Id::from_u64(21)
    );

    registry
        .insert(Instance::new(state(fresh_id).named("corrected")))
        .expect("corrected fresh fixture identity remains insertable");
    assert_eq!(
        registry
            .reserve_id()
            .expect("successful fixture advances high water"),
        Id::from_u64(101)
    );
}

#[test]
fn registry_rejects_empty_snapshot_name() {
    let mut registry = Registry::new();
    let id = Id::from_u64(100);

    let error = registry
        .insert(Instance::new(state(id).named("")))
        .expect_err("empty snapshot names must be rejected");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(registry.len(), 0);
    assert!(!registry.contains(id));
    assert_eq!(registry.window_id(""), None);
    assert_eq!(
        registry
            .reserve_id()
            .expect("rejected insertion does not advance high water"),
        Id::from_u64(1)
    );

    registry
        .insert(Instance::new(state(id).named("corrected")))
        .expect("rejected identity remains insertable with a valid name");
    assert_eq!(
        registry
            .reserve_id()
            .expect("successful insertion advances high water"),
        Id::from_u64(101)
    );
}

#[test]
fn snapshot_derives_identity_from_metrics() {
    let metrics_id = Id::from_u64(7);
    let snapshot = WindowSnapshot::new(
        "Metric identity",
        Metrics::from_physical_size(
            metrics_id,
            PhysicalSize {
                width: 640,
                height: 480,
            },
            1.0,
        )
        .expect("test metrics are valid"),
    );

    assert_eq!(snapshot.id(), metrics_id);
}

#[test]
fn instance_derives_identity_from_snapshot() {
    let snapshot_id = Id::from_u64(11);
    let snapshot = WindowSnapshot::new(
        "Snapshot identity",
        Metrics::from_physical_size(
            snapshot_id,
            PhysicalSize {
                width: 640,
                height: 480,
            },
            1.0,
        )
        .expect("test metrics are valid"),
    );
    let instance = Instance::new(snapshot);

    assert_eq!(instance.id(), snapshot_id);
    assert_eq!(instance.state().id(), snapshot_id);
    assert_eq!(instance.state().metrics().id(), snapshot_id);
}

#[test]
fn registry_rejects_identity_collision_without_replacement() {
    let id = Id::from_u64(21);
    let mut registry = Registry::new();
    registry
        .insert(Instance::new(
            WindowSnapshot::new(
                "Original",
                Metrics::from_physical_size(
                    id,
                    PhysicalSize {
                        width: 640,
                        height: 480,
                    },
                    1.0,
                )
                .expect("test metrics are valid"),
            )
            .named("original"),
        ))
        .expect("original instance inserts");

    let error = registry
        .insert(Instance::new(
            WindowSnapshot::new(
                "Replacement",
                Metrics::from_physical_size(
                    id,
                    PhysicalSize {
                        width: 800,
                        height: 600,
                    },
                    1.0,
                )
                .expect("test metrics are valid"),
            )
            .named("replacement"),
        ))
        .expect_err("duplicate identity must be rejected");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry
            .remove(id)
            .expect("original instance remains live")
            .state()
            .title(),
        "Original"
    );
}

#[test]
fn registry_advances_allocator_past_inserted_fixture_id() {
    let fixture_id = Id::from_u64(41);
    let mut registry = Registry::new();
    registry
        .insert(Instance::new(WindowSnapshot::new(
            "Fixture",
            Metrics::from_physical_size(
                fixture_id,
                PhysicalSize {
                    width: 640,
                    height: 480,
                },
                1.0,
            )
            .expect("test metrics are valid"),
        )))
        .expect("fixture instance inserts");

    assert_eq!(
        registry
            .reserve_id()
            .expect("identity reserves after fixture"),
        Id::from_u64(42)
    );
}

#[test]
fn registry_rejects_duplicate_live_name() {
    let mut registry = Registry::new();
    let first_id = Id::from_u64(31);
    let second_id = Id::from_u64(32);
    registry
        .insert(Instance::new(
            WindowSnapshot::new(
                "First",
                Metrics::from_physical_size(
                    first_id,
                    PhysicalSize {
                        width: 640,
                        height: 480,
                    },
                    1.0,
                )
                .expect("test metrics are valid"),
            )
            .named("shared"),
        ))
        .expect("first name inserts");

    let error = registry
        .insert(Instance::new(
            WindowSnapshot::new(
                "Second",
                Metrics::from_physical_size(
                    second_id,
                    PhysicalSize {
                        width: 800,
                        height: 600,
                    },
                    1.0,
                )
                .expect("test metrics are valid"),
            )
            .named("shared"),
        ))
        .expect_err("duplicate live name must be rejected");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.window_id("shared"), Some(first_id));
    assert_eq!(
        registry
            .reserve_id()
            .expect("rejected name collision leaves allocator unchanged"),
        second_id
    );
}

#[test]
fn registry_reports_identity_exhausted_without_reuse() {
    let exhausted_id = Id::from_u64(u64::MAX);
    let mut registry = Registry::new();
    registry
        .insert(Instance::new(WindowSnapshot::new(
            "Exhausted",
            Metrics::from_physical_size(
                exhausted_id,
                PhysicalSize {
                    width: 640,
                    height: 480,
                },
                1.0,
            )
            .expect("test metrics are valid"),
        )))
        .expect("maximum fixture identity inserts");

    let error = registry
        .reserve_id()
        .expect_err("allocation must reject an exhausted identity space");

    assert_eq!(error.code, ErrorCode::IdentityExhausted);
    assert!(registry.contains(exhausted_id));
    assert_eq!(registry.len(), 1);
}

#[test]
fn metrics_reject_invalid_observed_scale_without_coercion() {
    let error = Metrics::from_physical_size(
        Id::from_u64(8),
        PhysicalSize {
            width: 640,
            height: 480,
        },
        f64::NAN,
    )
    .expect_err("non-finite observed scale must be rejected");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
}

#[test]
fn draw_scheduler_coalesces_and_chooses_earliest_deadline() {
    let id = Id::from_u64(1);
    let later = Instant::now() + std::time::Duration::from_secs(5);
    let earlier = Instant::now() + std::time::Duration::from_secs(1);
    let mut scheduler = DrawScheduler::new();

    scheduler.request(&Action::DrawAt { id, time: later });
    scheduler.request(&Action::DrawAt { id, time: earlier });

    assert_eq!(scheduler.next_deadline(), Some(earlier));
    assert_eq!(scheduler.take_ready(earlier), vec![id]);

    scheduler.request(&Action::DrawNext(id));
    scheduler.request(&Action::DrawNext(id));

    assert_eq!(scheduler.next_deadline(), None);
    assert_eq!(scheduler.take_ready(Instant::now()), vec![id]);
}

#[test]
fn delayed_draw_waits_until_deadline() {
    let id = Id::from_u64(1);
    let now = Instant::now();
    let deadline = now + std::time::Duration::from_millis(20);
    let mut scheduler = DrawScheduler::new();

    scheduler.request(&Action::DrawAt { id, time: deadline });

    assert_eq!(scheduler.take_ready(now), Vec::<Id>::new());
    assert_eq!(scheduler.next_deadline(), Some(deadline));
    assert_eq!(scheduler.take_ready(deadline), vec![id]);
}

#[test]
fn draw_scheduler_exposes_backend_neutral_deadline() {
    let id = Id::from_u64(1);
    let deadline = Instant::now() + std::time::Duration::from_millis(20);
    let mut scheduler = DrawScheduler::new();

    assert_eq!(scheduler.next_deadline(), None);
    assert_eq!(scheduler.take_ready(Instant::now()), Vec::<Id>::new());

    scheduler.request(&Action::DrawAt { id, time: deadline });

    assert_eq!(scheduler.next_deadline(), Some(deadline));
    assert_eq!(scheduler.take_ready(deadline), vec![id]);
}

#[test]
fn pointer_position_keys_separate_mouse_and_touch_contacts() {
    let window = Id::from_u64(1);
    let mouse = Point { x: 1.0, y: 2.0 };
    let first_touch = Point { x: 3.0, y: 4.0 };
    let second_touch = Point { x: 5.0, y: 6.0 };
    let mut positions = HashMap::new();

    positions.insert(PointerPositionKey::mouse(window), mouse);
    positions.insert(PointerPositionKey::touch(window, 10), first_touch);
    positions.insert(PointerPositionKey::touch(window, 11), second_touch);
    positions.remove(&PointerPositionKey::touch(window, 10));

    assert_eq!(
        positions.get(&PointerPositionKey::mouse(window)),
        Some(&mouse)
    );
    assert_eq!(
        positions.get(&PointerPositionKey::touch(window, 11)),
        Some(&second_touch)
    );
    assert!(!positions.contains_key(&PointerPositionKey::touch(window, 10)));
}

#[test]
fn file_drag_position_uses_last_mouse_position_when_available() {
    struct Noop;
    impl Handler for Noop {}

    let window = Id::from_u64(1);
    let mouse = Point { x: 8.0, y: 13.0 };
    let mut runner = WinitRunner::from_loop(Loop::new(Noop));

    assert_eq!(runner.last_mouse_position(window), None);

    runner
        .pointer_positions
        .insert(PointerPositionKey::mouse(window), mouse);

    assert_eq!(runner.last_mouse_position(window), Some(mouse));
}

#[test]
fn winit_runner_exposes_current_capabilities_for_command_planning() {
    struct Noop;
    impl Handler for Noop {}

    let mut runner = WinitRunner::from_loop(Loop::new(Noop));
    let capabilities = conservative_capabilities();
    runner.resolve_capabilities_for_test(capabilities.clone());

    assert_eq!(runner.capabilities(), &capabilities);
}

#[test]
fn winit_runner_rejects_unsupported_command_before_native_application_in_tests() {
    struct Noop;
    impl Handler for Noop {}

    let mut runner = WinitRunner::from_loop(Loop::new(Noop));
    runner.resolve_capabilities_for_test(conservative_capabilities());
    let command = Command::Open {
        request: WindowRequest::builder("dialog")
            .dialog(Id::from_u64(1))
            .build(),
    };

    let error = runner
        .plan_command_for_test(command)
        .expect_err("dialog role should be rejected before native create");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
}

#[test]
fn immediate_native_resize_commits_metrics_and_callback_once() {
    struct Recorder {
        resized: Rc<RefCell<Vec<Size>>>,
    }

    impl Handler for Recorder {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.borrow_mut().push(resize.size());
            Ok(())
        }
    }

    let id = Id::from_u64(801);
    let resized = Rc::new(RefCell::new(Vec::new()));
    let mut window_loop = Loop::new(Recorder {
        resized: resized.clone(),
    });
    window_loop
        .registry
        .insert(super::registry::Instance::new(state(id)))
        .expect("test identity inserts");
    let mut runner = WinitRunner::from_loop(window_loop);
    runner.resolve_capabilities_for_test(conservative_capabilities());

    runner
        .apply_size_request_result_for_test(
            id,
            Some(PhysicalSize {
                width: 960,
                height: 540,
            }),
        )
        .expect("immediate native size result applies");

    assert_eq!(
        runner
            .registry_snapshot_for_test(id)
            .expect("window remains live")
            .metrics()
            .physical_size(),
        PhysicalSize {
            width: 960,
            height: 540,
        }
    );
    assert_eq!(
        resized.borrow().as_slice(),
        vec![Size {
            width: 480.0,
            height: 270.0,
        }]
    );
}

#[test]
fn deferred_native_resize_waits_for_resized_event() {
    struct Recorder {
        resized: Rc<RefCell<Vec<Size>>>,
    }

    impl Handler for Recorder {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.borrow_mut().push(resize.size());
            Ok(())
        }
    }

    let id = Id::from_u64(802);
    let resized = Rc::new(RefCell::new(Vec::new()));
    let mut window_loop = Loop::new(Recorder {
        resized: resized.clone(),
    });
    window_loop
        .registry
        .insert(super::registry::Instance::new(state(id)))
        .expect("test identity inserts");
    let mut runner = WinitRunner::from_loop(window_loop);
    runner.resolve_capabilities_for_test(conservative_capabilities());
    let before = runner
        .registry_snapshot_for_test(id)
        .expect("window remains live")
        .metrics()
        .clone();

    runner
        .apply_size_request_result_for_test(id, None)
        .expect("deferred native size request records pending work");

    assert_eq!(
        runner
            .registry_snapshot_for_test(id)
            .expect("window remains live")
            .metrics(),
        &before
    );
    assert!(resized.borrow().is_empty());

    runner
        .observe_resized_for_test(
            id,
            PhysicalSize {
                width: 640,
                height: 360,
            },
        )
        .expect("later native resize observes the deferred request");

    assert_eq!(
        resized.borrow().as_slice(),
        vec![Size {
            width: 320.0,
            height: 180.0,
        }]
    );
}

#[test]
fn later_equal_resize_is_deduplicated_but_different_resize_delivers() {
    struct Recorder {
        resized: Rc<RefCell<Vec<Size>>>,
    }

    impl Handler for Recorder {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.borrow_mut().push(resize.size());
            Ok(())
        }
    }

    let id = Id::from_u64(803);
    let resized = Rc::new(RefCell::new(Vec::new()));
    let mut window_loop = Loop::new(Recorder {
        resized: resized.clone(),
    });
    window_loop
        .registry
        .insert(super::registry::Instance::new(state(id)))
        .expect("test identity inserts");
    let mut runner = WinitRunner::from_loop(window_loop);
    runner.resolve_capabilities_for_test(conservative_capabilities());
    let immediate = PhysicalSize {
        width: 960,
        height: 540,
    };

    runner
        .apply_size_request_result_for_test(id, Some(immediate))
        .expect("immediate native size result applies");
    runner
        .observe_resized_for_test(id, immediate)
        .expect("equal observation is accepted without redelivery");
    runner
        .observe_resized_for_test(
            id,
            PhysicalSize {
                width: 1024,
                height: 576,
            },
        )
        .expect("different observation is delivered");

    assert_eq!(
        resized.borrow().as_slice(),
        vec![
            Size {
                width: 480.0,
                height: 270.0,
            },
            Size {
                width: 512.0,
                height: 288.0,
            },
        ]
    );
}

#[test]
fn queued_immediate_resize_echoes_preserve_each_marker_after_unrelated_observation() {
    struct Recorder {
        resized: Rc<RefCell<Vec<Size>>>,
    }

    impl Handler for Recorder {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.borrow_mut().push(resize.size());
            Ok(())
        }
    }

    let id = Id::from_u64(804);
    let resized = Rc::new(RefCell::new(Vec::new()));
    let mut window_loop = Loop::new(Recorder {
        resized: resized.clone(),
    });
    window_loop
        .registry
        .insert(super::registry::Instance::new(state(id)))
        .expect("test identity inserts");
    let mut runner = WinitRunner::from_loop(window_loop);
    runner.resolve_capabilities_for_test(conservative_capabilities());
    let first = PhysicalSize {
        width: 960,
        height: 540,
    };
    let second = PhysicalSize {
        width: 1024,
        height: 576,
    };
    let unrelated = PhysicalSize {
        width: 1280,
        height: 720,
    };

    for size in [first, first, second] {
        runner
            .apply_size_request_result_for_test(id, Some(size))
            .expect("immediate native size result applies");
    }
    runner
        .observe_resized_for_test(id, unrelated)
        .expect("unrelated native resize is delivered");
    for size in [first, first, second] {
        runner
            .observe_resized_for_test(id, size)
            .expect("queued native echo is accepted without redelivery");
    }

    assert_eq!(
        resized.borrow().as_slice(),
        vec![
            Size {
                width: 480.0,
                height: 270.0,
            },
            Size {
                width: 480.0,
                height: 270.0,
            },
            Size {
                width: 512.0,
                height: 288.0,
            },
            Size {
                width: 640.0,
                height: 360.0,
            },
        ]
    );
}

#[test]
fn fake_constraints_resolve_size_like_native_contract() {
    #[derive(Default)]
    struct Recorder {
        resized: Vec<Size>,
    }

    impl Handler for Recorder {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.push(resize.size());
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").size(size(320, 180)).into()])
        .expect("startup is valid");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");

    runner.dispatch(Command::SetMinInnerSize {
        id,
        size: Some(Size {
            width: 640.0,
            height: 360.0,
        }),
    });

    assert_eq!(
        runner
            .host()
            .registry()
            .get(id)
            .expect("window remains live")
            .metrics()
            .logical_size(),
        Size {
            width: 640.0,
            height: 360.0,
        }
    );
    assert_eq!(
        runner.handler().resized,
        vec![Size {
            width: 640.0,
            height: 360.0,
        }]
    );
}

#[test]
fn memory_clipboard_round_trips_text_and_image() {
    let mut clipboard = MemoryClipboard::new();
    clipboard.write_text("hello").unwrap();
    assert_eq!(clipboard.read_text().unwrap(), Some(String::from("hello")));

    clipboard
        .write_image(ClipboardImageRef {
            width: 1,
            height: 1,
            rgba: &[255, 0, 0, 255],
        })
        .unwrap();

    assert_eq!(
        clipboard.read_image().unwrap(),
        Some(ClipboardImage {
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        })
    );
}

#[test]
fn missing_live_handle_reports_stable_error() {
    let id = Id::from_u64(9);
    let instance = Instance::new(state(id));
    let error = instance.as_ref().handle().unwrap_err();

    assert_eq!(error.code, ErrorCode::HandleUnavailable);
    assert_eq!(error.id, Some(id));
}

#[test]
fn window_request_builds_authored_creation_intent_without_public_field_mutation() {
    let request = WindowRequest::builder("main")
        .title("Main Window")
        .inner_size(size(320, 240))
        .hidden()
        .borderless()
        .build();

    assert_eq!(request.name(), Some("main"));
    assert_eq!(request.title(), "Main Window");
    assert_eq!(request.inner_size(), Some(size(320, 240)));
    assert!(!request.visible());
    assert_eq!(request.fullscreen(), Fullscreen::Borderless);
    assert_eq!(request.role().kind(), RoleKind::Root);
}

#[test]
fn window_snapshot_is_observed_runtime_state_built_through_constructor() {
    let metrics = Metrics::from_physical_size(
        Id::from_u64(7),
        PhysicalSize {
            width: 640,
            height: 480,
        },
        2.0,
    )
    .expect("test metrics are valid");
    let snapshot = WindowSnapshot::new("Main Window", metrics.clone())
        .named("main")
        .with_visible(true)
        .focused(true);

    assert_eq!(snapshot.id(), Id::from_u64(7));
    assert_eq!(snapshot.title(), "Main Window");
    assert_eq!(snapshot.name(), Some("main"));
    assert_eq!(snapshot.metrics(), &metrics);
    assert!(snapshot.is_visible());
    assert!(snapshot.is_focused());
}

#[test]
fn window_state_patch_updates_snapshot_and_reports_event() {
    let id = Id::from_u64(1);
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 600,
        },
        1.0,
    )
    .expect("test metrics are valid");
    let mut snapshot = WindowSnapshot::new("Main", metrics);

    let patch = WindowStatePatch::title(id, "Renamed");
    let event = patch.apply(&mut snapshot).expect("patch should apply");

    assert_eq!(snapshot.title(), "Renamed");
    assert!(event.is_none());

    let visible = WindowStatePatch::visible(id, false);
    let event = visible
        .apply(&mut snapshot)
        .expect("visible patch should apply");
    assert_eq!(snapshot.visible(), Some(false));
    assert!(event.is_none());
}

#[test]
fn metrics_state_patch_preserves_outer_geometry_and_reports_resize_event() {
    let id = Id::from_u64(3);
    let mut snapshot = WindowSnapshot::new(
        "Main",
        Metrics::from_physical_size(
            id,
            PhysicalSize {
                width: 200,
                height: 100,
            },
            1.0,
        )
        .expect("test metrics are valid"),
    );
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 600,
            height: 300,
        },
        2.0,
    )
    .expect("test metrics are valid")
    .with_outer_geometry(
        Some(Point { x: 10.0, y: 20.0 }),
        Some(Size {
            width: 340.0,
            height: 220.0,
        }),
    )
    .expect("test outer geometry is valid");

    let event = WindowStatePatch::metrics(metrics.clone(), MetricsEvent::Resized)
        .apply(&mut snapshot)
        .expect("metrics patch should apply")
        .expect("resize patch should emit an event");

    assert_eq!(snapshot.metrics(), &metrics);
    assert_eq!(event, EventKind::Resized(metrics));
}

#[test]
fn window_snapshot_rejects_mismatched_metrics_without_mutation() {
    let id = Id::from_u64(3);
    let mismatched_id = Id::from_u64(4);
    let mut snapshot = state(id);
    let before = snapshot.clone();
    let replacement = Metrics::from_physical_size(
        mismatched_id,
        PhysicalSize {
            width: 600,
            height: 300,
        },
        1.0,
    )
    .expect("test metrics are valid");

    let error = snapshot
        .set_metrics(replacement)
        .expect_err("mismatched metric identity must be rejected");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(error.id, Some(mismatched_id));
    assert_eq!(snapshot, before);
    assert_eq!(snapshot.id(), id);
}

#[test]
fn window_snapshot_replaces_metrics_with_the_same_identity() {
    let id = Id::from_u64(3);
    let mut snapshot = state(id);
    let replacement = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 600,
            height: 300,
        },
        1.0,
    )
    .expect("test metrics are valid");

    snapshot
        .set_metrics(replacement.clone())
        .expect("same identity metrics must replace the current metrics");

    assert_eq!(snapshot.id(), id);
    assert_eq!(snapshot.metrics(), &replacement);
}

#[test]
fn metrics_state_patch_rejects_mismatched_snapshot_identity_without_event() {
    let id = Id::from_u64(3);
    let mismatched_id = Id::from_u64(4);
    let mut snapshot = state(id);
    let before = snapshot.clone();
    let metrics = Metrics::from_physical_size(
        mismatched_id,
        PhysicalSize {
            width: 600,
            height: 300,
        },
        1.0,
    )
    .expect("test metrics are valid");

    let error = WindowStatePatch::metrics(metrics, MetricsEvent::Resized)
        .apply(&mut snapshot)
        .expect_err("mismatched metric transition must be rejected before an event");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(error.id, Some(mismatched_id));
    assert_eq!(snapshot, before);
    assert_eq!(snapshot.id(), id);
}

#[test]
fn window_state_patch_rejects_wrong_target() {
    let id = Id::from_u64(1);
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 600,
        },
        1.0,
    )
    .expect("test metrics are valid");
    let mut snapshot = WindowSnapshot::new("Main", metrics);
    let patch = WindowStatePatch::title(Id::from_u64(2), "Wrong");

    let error = patch
        .apply(&mut snapshot)
        .expect_err("wrong id should fail");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
}

#[test]
fn native_event_transition_updates_focus_state_and_emits_event() {
    let id = Id::from_u64(1);
    let mut snapshot = state(id);
    let event = NativeEventTransition::focused(id, true)
        .apply(&mut snapshot)
        .expect("focus transition should apply");

    assert!(snapshot.is_focused());
    assert_eq!(event, EventKind::Focused { id, focused: true });
}

#[test]
fn transition_event_derives_from_committed_patch() {
    #[derive(Default)]
    struct Recorder {
        event: Option<EventKind>,
        position: Option<Point>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            self.event = Some(event.event().clone());
            self.position = event.state().and_then(WindowSnapshot::position);
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    let committed_position = Point { x: 10.0, y: 20.0 };
    let mut handler = Recorder::default();

    host.dispatch_native_transition(
        &mut handler,
        NativeEventTransition::moved(id, committed_position),
    )
    .expect("native transition should dispatch");

    assert_eq!(handler.position, Some(committed_position));
    assert_eq!(
        handler.event,
        Some(EventKind::Moved {
            id,
            position: committed_position,
        })
    );
}

#[test]
fn transition_failure_emits_no_event() {
    #[derive(Default)]
    struct Recorder {
        calls: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
            self.calls += 1;
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    host.clear();
    let before = host
        .registry()
        .get(id)
        .expect("test window is live")
        .instance
        .state()
        .clone();
    let mut handler = Recorder::default();

    let error = host
        .dispatch_native_transition(
            &mut handler,
            NativeEventTransition::moved(
                id,
                Point {
                    x: f64::NAN,
                    y: 20.0,
                },
            ),
        )
        .expect_err("invalid transition payload must be rejected");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(handler.calls, 0);
    assert!(host.events().is_empty());
    assert_eq!(
        host.registry()
            .get(id)
            .expect("test window remains live")
            .instance
            .state(),
        &before
    );

    let mut snapshot = before.clone();
    let error = NativeEventTransition::moved(Id::from_u64(99), Point { x: 10.0, y: 20.0 })
        .apply(&mut snapshot)
        .expect_err("wrong transition target must be rejected");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(snapshot, before);
}

#[test]
fn native_event_transition_has_no_independent_patch_event_constructor() {
    let transition = include_str!("transition.rs");

    assert!(!transition.contains("pub const fn new(patch"));
    assert!(!transition.contains("pub fn new(patch"));
}

#[test]
fn native_event_transition_pairs_moved_theme_and_occlusion_patches_with_events() {
    let id = Id::from_u64(1);
    let mut snapshot = state(id);

    let moved = NativeEventTransition::moved(id, Point { x: 10.0, y: 20.0 })
        .apply(&mut snapshot)
        .expect("move transition should apply");
    assert_eq!(
        moved,
        EventKind::Moved {
            id,
            position: Point { x: 10.0, y: 20.0 },
        }
    );
    assert_eq!(snapshot.position(), Some(Point { x: 10.0, y: 20.0 }));

    let themed = NativeEventTransition::theme_changed(id, Some(Theme::Dark))
        .apply(&mut snapshot)
        .expect("theme transition should apply");
    assert_eq!(
        themed,
        EventKind::ThemeChanged {
            id,
            theme: Some(Theme::Dark),
        }
    );
    assert_eq!(snapshot.theme(), Some(Theme::Dark));

    let occluded = NativeEventTransition::occluded(id, true)
        .apply(&mut snapshot)
        .expect("occlusion transition should apply");
    assert_eq!(occluded, EventKind::Occluded { id, occluded: true });
    assert!(snapshot.is_occluded());
}

#[test]
fn native_pointer_transition_preserves_logical_and_physical_positions() {
    let id = Id::from_u64(1);
    let mut snapshot = state(id);
    let event = NativeEventTransition::mouse_moved(
        id,
        Point { x: 12.0, y: 24.0 },
        PhysicalPoint { x: 24, y: 48 },
        None,
        ModifierState::default(),
    )
    .apply(&mut snapshot)
    .expect("pointer transition should apply");

    let EventKind::Input(InputEvent::Pointer(pointer)) = event else {
        panic!("expected pointer input event");
    };

    assert_eq!(pointer.position, Some(Point { x: 12.0, y: 24.0 }));
    assert_eq!(
        pointer.physical_position,
        Some(PhysicalPoint { x: 24, y: 48 })
    );
}

#[test]
fn native_pointer_transition_routes_to_input_delivery() {
    let id = Id::from_u64(1);
    let mut snapshot = state(id);
    let event = NativeEventTransition::mouse_moved(
        id,
        Point { x: 12.0, y: 24.0 },
        PhysicalPoint { x: 24, y: 48 },
        Some(Point { x: 1.0, y: 2.0 }),
        ModifierState::default(),
    )
    .apply(&mut snapshot)
    .expect("pointer transition should apply");
    assert_eq!(
        native_transition_route(&event),
        NativeTransitionRoute::Input
    );
    let EventKind::Input(InputEvent::Pointer(pointer)) = event else {
        panic!("expected pointer input event");
    };

    assert_eq!(pointer.id, id);
    assert_eq!(pointer.phase, PointerPhase::Moved);
    assert_eq!(pointer.position, Some(Point { x: 12.0, y: 24.0 }));
    assert_eq!(
        pointer.physical_position,
        Some(PhysicalPoint { x: 24, y: 48 })
    );
}

#[test]
fn native_transition_route_sends_lifecycle_events_to_event_delivery() {
    let id = Id::from_u64(1);
    let mut snapshot = state(id);
    let event = NativeEventTransition::focused(id, true)
        .apply(&mut snapshot)
        .expect("focus transition should apply");

    assert_eq!(
        native_transition_route(&event),
        NativeTransitionRoute::Event
    );
}

#[test]
fn native_event_delivery_invokes_handler_event_callback_for_modeled_events() {
    #[derive(Default)]
    struct Record {
        events: Vec<EventKind>,
        state_observed: bool,
    }

    struct Recorder(Rc<RefCell<Record>>);

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            let mut record = self.0.borrow_mut();
            record.events.push(event.event().clone());
            record.state_observed = event.state().is_some();
            event.context_mut().open(open("child"));
            event.again();
            Ok(())
        }
    }

    let record = Rc::new(RefCell::new(Record::default()));
    let mut window_loop = Loop::new(Recorder(record.clone()));
    let id = window_loop
        .registry
        .reserve_id()
        .expect("test identity reserves");
    window_loop
        .registry
        .insert(Instance::new(state(id)))
        .expect("test instance inserts");
    let mut runner = WinitRunner::from_loop(window_loop);
    runner.resolve_capabilities_for_test(conservative_capabilities());
    let event = EventKind::Focused { id, focused: true };

    runner.deliver_event_to_pump_for_test(event.clone());
    runner.drain_pump_for_test();

    assert_eq!(record.borrow().events.first(), Some(&event));
    assert!(record.borrow().state_observed);
    assert!(runner.registry_contains_for_test(Id::from_u64(id.as_u64() + 1)));
}

#[test]
fn created_event_close_skips_ready_delivery_when_window_is_destroyed() {
    #[derive(Default)]
    struct Record {
        created: usize,
        ready: usize,
    }

    struct Recorder(Rc<RefCell<Record>>);

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                self.0.borrow_mut().created += 1;
                event.close();
            }
            Ok(())
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.0.borrow_mut().ready += 1;
            let _ = ready.state();
            Ok(())
        }
    }

    let record = Rc::new(RefCell::new(Record::default()));
    let mut window_loop = Loop::new(Recorder(record.clone()));
    let id = window_loop
        .registry
        .reserve_id()
        .expect("test identity reserves");
    let window_state = state(id);
    window_loop
        .registry
        .insert(Instance::new(window_state.clone()))
        .expect("test instance inserts");
    let mut runner = WinitRunner::from_loop(window_loop);
    runner.resolve_capabilities_for_test(conservative_capabilities());

    runner
        .deliver_created_then_ready_for_test(window_state)
        .expect("created callback should be delivered without ready after close");

    assert_eq!(record.borrow().created, 1);
    assert_eq!(record.borrow().ready, 0);
    assert!(!runner.registry_contains_for_test(id));
}

#[test]
fn fake_host_native_transition_dispatch_matches_native_event_callback_contract() {
    #[derive(Default)]
    struct Recorder {
        event: Option<EventKind>,
        focused: Option<bool>,
        calls: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            self.calls += 1;
            self.event = Some(event.event().clone());
            self.focused = event.state().map(WindowSnapshot::is_focused);
            event.draw();
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    host.clear();
    let mut handler = Recorder::default();

    let effect = host
        .dispatch_native_transition(&mut handler, NativeEventTransition::focused(id, true))
        .expect("native transition should dispatch through generic event callback");

    assert_eq!(
        handler.event,
        Some(EventKind::Focused { id, focused: true })
    );
    assert_eq!(handler.focused, Some(true));
    assert_eq!(host.events(), &[HostEvent::Focused { id, focused: true }]);
    assert_eq!(effect, Effect::Draw(id));
}

#[test]
fn specialized_resize_input_close_callbacks_do_not_double_deliver_generic_event() {
    #[derive(Default)]
    struct Recorder {
        generic: usize,
        resize: usize,
        input: usize,
        close: usize,
        closed: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
            self.generic += 1;
            Ok(())
        }

        fn resize(&mut self, _win: &mut Resize<'_>) -> Result<()> {
            self.resize += 1;
            Ok(())
        }

        fn input(&mut self, _input: &mut Input<'_>) -> Result<()> {
            self.input += 1;
            Ok(())
        }

        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.close += 1;
            close.close();
            Ok(())
        }

        fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
            self.closed += 1;
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    runner.handler_mut().generic = 0;

    runner.transition(NativeEventTransition::resized(
        Metrics::from_physical_size(
            id,
            PhysicalSize {
                width: 400,
                height: 200,
            },
            1.0,
        )
        .expect("test metrics are valid"),
    ));
    runner.transition(NativeEventTransition::mouse_moved(
        id,
        Point { x: 12.0, y: 24.0 },
        PhysicalPoint { x: 12, y: 24 },
        None,
        ModifierState::default(),
    ));
    runner.request_close(id);

    assert_eq!(runner.handler().resize, 1);
    assert_eq!(runner.handler().input, 1);
    assert_eq!(runner.handler().close, 1);
    assert_eq!(runner.handler().closed, 1);
    assert_eq!(runner.handler().generic, 0);
}

#[test]
fn fake_host_generic_dispatch_rejects_specialized_transitions() {
    #[derive(Default)]
    struct Recorder {
        generic: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
            self.generic += 1;
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    let mut handler = Recorder::default();

    let mouse_moved_error = host
        .dispatch_native_transition(
            &mut handler,
            NativeEventTransition::mouse_moved(
                id,
                Point { x: 12.0, y: 24.0 },
                PhysicalPoint { x: 12, y: 24 },
                None,
                ModifierState::default(),
            ),
        )
        .expect_err("generic fake dispatch must reject specialized pointer input");
    assert_eq!(mouse_moved_error.code, ErrorCode::UnsupportedFeature);

    let resized_error = host
        .dispatch_native_transition(
            &mut handler,
            NativeEventTransition::resized(
                Metrics::from_physical_size(
                    id,
                    PhysicalSize {
                        width: 500,
                        height: 300,
                    },
                    1.0,
                )
                .expect("test metrics are valid"),
            ),
        )
        .expect_err("generic fake dispatch must reject specialized resize");
    assert_eq!(resized_error.code, ErrorCode::UnsupportedFeature);

    let scale_factor_changed_error = host
        .dispatch_native_transition(
            &mut handler,
            NativeEventTransition::scale_factor_changed(
                Metrics::from_physical_size(
                    id,
                    PhysicalSize {
                        width: 500,
                        height: 300,
                    },
                    2.0,
                )
                .expect("test metrics are valid"),
            ),
        )
        .expect_err("generic fake dispatch must reject specialized scale changes");
    assert_eq!(
        scale_factor_changed_error.code,
        ErrorCode::UnsupportedFeature
    );

    assert_eq!(handler.generic, 0);
}

#[test]
fn fake_host_escape_key_input_reports_pressed_and_requests_close_and_draw() {
    #[derive(Default)]
    struct Recorder {
        inputs: usize,
    }

    impl Handler for Recorder {
        fn input(&mut self, input: &mut Input<'_>) -> Result<()> {
            assert!(input.key_pressed(keyboard_types::Code::Escape));
            self.inputs += 1;
            input.close().draw();
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    let mut handler = Recorder::default();

    assert_eq!(
        host.dispatch_input(
            &mut handler,
            InputEvent::Key(KeyEvent {
                id,
                logical_key: keyboard_types::Key::Escape,
                physical_key: keyboard_types::Code::Escape,
                location: keyboard_types::Location::Standard,
                state: KeyState::Pressed,
                repeat: false,
                synthetic: false,
                modifiers: ModifierState::default(),
                timestamp: None,
            }),
        )
        .unwrap(),
        Effect::Batch(vec![Effect::CloseRequested(id), Effect::Draw(id)])
    );
    assert_eq!(handler.inputs, 1);
}

#[test]
fn winit_mapping_converts_window_request_to_native_attributes() {
    let request = WindowRequest::builder("surgeist-window")
        .title("Window")
        .position(Point { x: 12.0, y: 24.0 })
        .inner_size(Size {
            width: 800.0,
            height: 600.0,
        })
        .min_inner_size(Size {
            width: 320.0,
            height: 240.0,
        })
        .max_inner_size(Size {
            width: 1600.0,
            height: 1200.0,
        })
        .fixed()
        .controls(Controls {
            close: true,
            minimize: false,
            maximize: true,
        })
        .decorations(false)
        .transparent(true)
        .hidden()
        .borderless()
        .level(Level::AlwaysOnTop)
        .theme(Some(Theme::Dark))
        .root()
        .build();

    let attributes = super::winit_mapping::window_attributes_from_request(&request).unwrap();

    assert_eq!(attributes.title, "Window");
    assert_eq!(
        attributes.position,
        Some(winit::dpi::Position::Logical(
            winit::dpi::LogicalPosition::new(12.0, 24.0)
        ))
    );
    assert_eq!(
        attributes.inner_size,
        Some(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            800.0, 600.0
        )))
    );
    assert_eq!(
        attributes.min_inner_size,
        Some(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            320.0, 240.0
        )))
    );
    assert_eq!(
        attributes.max_inner_size,
        Some(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            1600.0, 1200.0
        )))
    );
    assert!(!attributes.resizable);
    assert_eq!(
        attributes.enabled_buttons,
        winit::window::WindowButtons::CLOSE | winit::window::WindowButtons::MAXIMIZE
    );
    assert!(!attributes.decorations);
    assert!(attributes.transparent);
    assert!(!attributes.visible);
    assert_eq!(
        attributes.window_level,
        winit::window::WindowLevel::AlwaysOnTop
    );
    assert_eq!(attributes.preferred_theme, Some(winit::window::Theme::Dark));
    assert!(matches!(
        attributes.fullscreen,
        Some(winit::window::Fullscreen::Borderless(None))
    ));
}

#[test]
fn exclusive_fullscreen_requires_native_video_mode() {
    let request = WindowRequest::builder("exclusive")
        .fullscreen(Fullscreen::Exclusive)
        .build();

    let error = super::winit_mapping::window_attributes_from_request(&request).unwrap_err();

    assert_eq!(error.code, ErrorCode::CommandFailed);
}

#[test]
fn native_window_request_rejects_unimplemented_roles() {
    let request = WindowRequest::builder("dialog")
        .dialog(Id::from_u64(1))
        .build();

    let error = super::winit_mapping::window_attributes_from_request(&request).unwrap_err();

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(error.message.contains("roles"));
}

#[test]
fn window_modeling_baseline_request_rejects_non_root_roles_for_winit_attributes() {
    let request = WindowRequest::builder("dialog")
        .dialog(Id::from_u64(1))
        .build();

    let error = super::winit_mapping::window_attributes_from_request(&request)
        .expect_err("role support is not modeled yet");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
}

#[test]
fn window_modeling_baseline_request_rejects_exclusive_fullscreen_for_winit_attributes() {
    let request = WindowRequest::builder("exclusive")
        .fullscreen(Fullscreen::Exclusive)
        .build();

    let error = super::winit_mapping::window_attributes_from_request(&request)
        .expect_err("exclusive fullscreen requires a native video mode");

    assert_eq!(error.code, ErrorCode::CommandFailed);
}

#[test]
fn host_capabilities_report_current_winit_support() {
    let capabilities = conservative_capabilities();

    assert_eq!(
        capabilities.role(RoleKind::Root),
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.role(RoleKind::Dialog),
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.role(RoleKind::Tool),
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.role(RoleKind::Popup),
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.fullscreen(FullscreenMode::None),
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.fullscreen(FullscreenMode::Borderless),
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.fullscreen(FullscreenMode::Exclusive),
        CapabilitySupport::Unsupported
    );
    assert_eq!(
        capabilities.cursor(CursorCapability::Icon),
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.cursor(CursorCapability::Hidden),
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.cursor(CursorCapability::Custom),
        CapabilitySupport::Unsupported
    );
}

#[test]
fn host_capabilities_explain_rejections_with_stable_codes() {
    let capabilities = conservative_capabilities();

    let role_error = capabilities
        .require_role(RoleKind::Dialog)
        .expect_err("dialog roles are not yet supported by current winit plan");
    let fullscreen_error = capabilities
        .require_fullscreen(FullscreenMode::Exclusive)
        .expect_err("exclusive fullscreen is unsupported without video mode selection");

    assert_eq!(role_error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(fullscreen_error.code, ErrorCode::UnsupportedFeature);
}

#[test]
fn normalization_rejects_empty_name_and_invalid_geometry() {
    let capabilities = conservative_capabilities();
    let commands = [
        Command::Open {
            request: WindowRequest::builder("").build(),
        },
        Command::SetPosition {
            id: Id::from_u64(1),
            position: Point {
                x: f64::NAN,
                y: 0.0,
            },
        },
        Command::SetInnerSize {
            id: Id::from_u64(1),
            size: Size {
                width: -1.0,
                height: 100.0,
            },
        },
    ];

    for command in commands {
        let error = HostCommandPlan::from_command(command, &capabilities)
            .expect_err("intrinsically invalid command should be rejected during normalization");

        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }
}

#[test]
fn normalization_rejects_invalid_ime_cursor_rectangles() {
    let capabilities = conservative_capabilities();
    let invalid_cursor_areas = [
        Rect {
            origin: Point {
                x: f64::NAN,
                y: 0.0,
            },
            size: Size {
                width: 1.0,
                height: 1.0,
            },
        },
        Rect {
            origin: Point {
                x: 0.0,
                y: f64::INFINITY,
            },
            size: Size {
                width: 1.0,
                height: 1.0,
            },
        },
        Rect {
            origin: Point { x: -1.0, y: 0.0 },
            size: Size {
                width: 1.0,
                height: 1.0,
            },
        },
        Rect {
            origin: Point { x: 0.0, y: -1.0 },
            size: Size {
                width: 1.0,
                height: 1.0,
            },
        },
        Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: f64::NAN,
                height: 1.0,
            },
        },
        Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 1.0,
                height: f64::INFINITY,
            },
        },
        Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: -1.0,
                height: 1.0,
            },
        },
        Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 1.0,
                height: -1.0,
            },
        },
    ];

    for cursor_area in invalid_cursor_areas {
        let error = HostCommandPlan::from_command(
            Command::SetIme {
                id: Id::from_u64(1),
                request: ImeRequest::Enable(ImeConfig {
                    purpose: ImePurpose::Normal,
                    hint: ImeHint::None,
                    cursor_area: Some(cursor_area),
                    surrounding_text: None,
                }),
            },
            &capabilities,
        )
        .expect_err("invalid IME cursor rectangles should be rejected during normalization");

        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }
}

#[test]
fn normalization_preserves_valid_ime_payloads() {
    let id = Id::from_u64(1);
    let requests = [
        (
            ImeRequest::Enable(ImeConfig {
                purpose: ImePurpose::Password,
                hint: ImeHint::None,
                cursor_area: Some(Rect {
                    origin: Point { x: -0.0, y: -0.0 },
                    size: Size {
                        width: -0.0,
                        height: -0.0,
                    },
                }),
                surrounding_text: None,
            }),
            ImeRequest::Enable(ImeConfig {
                purpose: ImePurpose::Password,
                hint: ImeHint::None,
                cursor_area: Some(Rect {
                    origin: Point { x: 0.0, y: 0.0 },
                    size: Size {
                        width: 0.0,
                        height: 0.0,
                    },
                }),
                surrounding_text: None,
            }),
        ),
        (
            ImeRequest::Update(ImeConfig {
                purpose: ImePurpose::Normal,
                hint: ImeHint::None,
                cursor_area: Some(Rect {
                    origin: Point { x: 12.5, y: 4.0 },
                    size: Size {
                        width: 8.0,
                        height: 16.0,
                    },
                }),
                surrounding_text: None,
            }),
            ImeRequest::Update(ImeConfig {
                purpose: ImePurpose::Normal,
                hint: ImeHint::None,
                cursor_area: Some(Rect {
                    origin: Point { x: 12.5, y: 4.0 },
                    size: Size {
                        width: 8.0,
                        height: 16.0,
                    },
                }),
                surrounding_text: None,
            }),
        ),
        (
            ImeRequest::Restart(ImeConfig {
                purpose: ImePurpose::Terminal,
                hint: ImeHint::None,
                cursor_area: Some(Rect {
                    origin: Point { x: 3.0, y: 7.0 },
                    size: Size {
                        width: 2.0,
                        height: 1.0,
                    },
                }),
                surrounding_text: None,
            }),
            ImeRequest::Restart(ImeConfig {
                purpose: ImePurpose::Terminal,
                hint: ImeHint::None,
                cursor_area: Some(Rect {
                    origin: Point { x: 3.0, y: 7.0 },
                    size: Size {
                        width: 2.0,
                        height: 1.0,
                    },
                }),
                surrounding_text: None,
            }),
        ),
    ];

    let mut host = testing::Host::with_capabilities(fully_supported_capabilities());
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("fixture window opens before IME normalization");

    for (request, expected_request) in requests {
        host.apply(Command::SetIme { id, request })
            .expect("valid IME payloads should normalize successfully");
        let command = host
            .commands()
            .last()
            .cloned()
            .expect("fake host records the normalized command");

        assert_eq!(
            command,
            Command::SetIme {
                id,
                request: expected_request,
            }
        );

        if let Command::SetIme {
            request: ImeRequest::Enable(config),
            ..
        } = command
        {
            let cursor_area = config
                .cursor_area
                .expect("enabled IME should retain its cursor area");

            assert_eq!(cursor_area.origin.x.to_bits(), 0.0f64.to_bits());
            assert_eq!(cursor_area.origin.y.to_bits(), 0.0f64.to_bits());
            assert_eq!(cursor_area.size.width.to_bits(), 0.0f64.to_bits());
            assert_eq!(cursor_area.size.height.to_bits(), 0.0f64.to_bits());
        }
    }
}

#[test]
fn intrinsic_normalization_retains_unsupported_ime_semantics_before_planning() {
    let id = Id::from_u64(1);
    let signed_zero_cursor_area = Some(Rect {
        origin: Point { x: -0.0, y: -0.0 },
        size: Size {
            width: -0.0,
            height: -0.0,
        },
    });
    let canonical_cursor_area = Some(Rect {
        origin: Point { x: 0.0, y: 0.0 },
        size: Size {
            width: 0.0,
            height: 0.0,
        },
    });
    let configurations = [
        ImeConfig {
            purpose: ImePurpose::Email,
            hint: ImeHint::None,
            cursor_area: signed_zero_cursor_area,
            surrounding_text: None,
        },
        ImeConfig {
            purpose: ImePurpose::Normal,
            hint: ImeHint::Spellcheck,
            cursor_area: signed_zero_cursor_area,
            surrounding_text: None,
        },
        ImeConfig {
            purpose: ImePurpose::Normal,
            hint: ImeHint::None,
            cursor_area: signed_zero_cursor_area,
            surrounding_text: Some(ImeSurroundingText {
                text: String::from("name@example.test"),
                cursor: 4,
                anchor: 1,
            }),
        },
    ];

    for configuration in configurations {
        let command = Command::SetIme {
            id,
            request: ImeRequest::Enable(configuration.clone()),
        };
        let normalized = crate::normalization::normalize_command(command.clone())
            .expect("intrinsically valid IME semantics should normalize")
            .into_command();

        assert_eq!(
            normalized,
            Command::SetIme {
                id,
                request: ImeRequest::Enable(ImeConfig {
                    cursor_area: canonical_cursor_area,
                    ..configuration
                }),
            }
        );

        let error = HostCommandPlan::from_command(command, &fully_supported_capabilities())
            .expect_err("unsupported IME semantics must fail during planning");
        assert_eq!(error.code, ErrorCode::ImeUnsupported);
    }
}

#[test]
fn normalization_allows_finite_negative_outer_position() {
    let command = Command::Open {
        request: WindowRequest::builder("negative-position")
            .position(Point {
                x: -1920.0,
                y: -1080.0,
            })
            .build(),
    };
    let expected = command.clone();

    let mut host = testing::Host::new();
    host.apply(command)
        .expect("finite negative outer positions should remain valid");

    assert_eq!(host.commands(), &[expected]);
}

#[test]
fn normalization_rejects_inverted_bounds_and_clamps_requested_size() {
    let capabilities = conservative_capabilities();
    let inverted = Command::Open {
        request: WindowRequest::builder("inverted-bounds")
            .min_inner_size(Size {
                width: 600.0,
                height: 200.0,
            })
            .max_inner_size(Size {
                width: 500.0,
                height: 400.0,
            })
            .build(),
    };

    let error = HostCommandPlan::from_command(inverted, &capabilities)
        .expect_err("inverted inner-size bounds should be rejected during normalization");
    assert_eq!(error.code, ErrorCode::InvalidRequest);

    let command = Command::Open {
        request: WindowRequest::builder("clamped-size")
            .inner_size(Size {
                width: 900.0,
                height: 50.0,
            })
            .min_inner_size(Size {
                width: 100.0,
                height: 200.0,
            })
            .max_inner_size(Size {
                width: 500.0,
                height: 600.0,
            })
            .build(),
    };

    let mut host = testing::Host::new();
    host.apply(command)
        .expect("well-formed bounds should normalize requested size");
    let Command::Open { request } = host
        .commands()
        .last()
        .expect("fake host records the normalized open command")
    else {
        panic!("open command should remain an open command after normalization");
    };

    assert_eq!(
        request.inner_size(),
        Some(Size {
            width: 500.0,
            height: 200.0,
        })
    );
}

#[test]
fn host_command_plan_keeps_canonical_payload_private() {
    let capabilities = conservative_capabilities();
    let command = Command::Open {
        request: WindowRequest::builder("canonical-accessors")
            .inner_size(Size {
                width: 700.0,
                height: 50.0,
            })
            .min_inner_size(Size {
                width: 100.0,
                height: 100.0,
            })
            .max_inner_size(Size {
                width: 500.0,
                height: 400.0,
            })
            .build(),
    };
    let plan = HostCommandPlan::from_command(command, &capabilities)
        .expect("well-formed command should normalize before host planning");

    assert_eq!(plan.kind(), CommandKind::Open);
    assert_eq!(plan.target(), None);
    assert_eq!(
        plan.capability_decisions()
            .iter()
            .map(|decision| (decision.kind(), decision.support()))
            .collect::<Vec<_>>(),
        vec![
            (
                CapabilityKind::Role(RoleKind::Root),
                CapabilitySupport::Supported,
            ),
            (
                CapabilityKind::Fullscreen(FullscreenMode::None),
                CapabilitySupport::Supported,
            ),
            (
                CapabilityKind::Window(WindowOperation::RequestInnerSize),
                CapabilitySupport::Supported,
            ),
            (
                CapabilityKind::Window(WindowOperation::SetInnerSizeBounds),
                CapabilitySupport::Supported,
            ),
        ]
    );
}

#[test]
fn host_command_plan_keeps_baseline_coarse_capability_outcomes() {
    let capabilities = conservative_capabilities();
    let commands = [
        Command::Open {
            request: WindowRequest::builder("dialog")
                .dialog(Id::from_u64(1))
                .build(),
        },
        Command::SetFullscreen {
            id: Id::from_u64(1),
            fullscreen: Fullscreen::Exclusive,
        },
        Command::SetCursor {
            id: Id::from_u64(1),
            cursor: Cursor::Custom(CustomCursorId::from_u64(9)),
        },
    ];

    for command in commands {
        let error = HostCommandPlan::from_command(command, &capabilities)
            .expect_err("baseline unsupported capability should still fail during host planning");

        assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    }
}

fn assert_prepared_startup<H>(prepared: PreparedLoop<H>, expected_names: &[&str]) {
    let names = prepared
        .startup()
        .iter()
        .map(|command| match command.command() {
            Command::Open { request } => request
                .name()
                .expect("prepared startup opens in this test are named"),
            command => panic!("prepared startup must retain open command order, got {command:?}"),
        })
        .collect::<Vec<_>>();

    assert_eq!(names, expected_names);
    assert_eq!(
        prepared.projection().live_window_count(),
        expected_names.len()
    );
    assert_eq!(
        prepared
            .projection()
            .names()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        expected_names
    );
}

#[test]
fn all_run_entry_points_share_startup_normalization() {
    struct Noop;
    impl Handler for Noop {}

    let mut direct_loop = Loop::new(Noop);
    direct_loop.startup = vec![open("main").into(), open("tools").into()];
    let mut direct_calls = 0;
    direct_loop
        .run_prepared_with(|prepared| {
            direct_calls += 1;
            assert_prepared_startup(prepared, &["main", "tools"]);
            Ok(())
        })
        .expect("loop preparation should invoke the injected runner once");
    assert_eq!(direct_calls, 1);

    let mut app_calls = 0;
    app(Noop)
        .open(open("main"))
        .open(open("tools"))
        .run_prepared_with(|prepared| {
            app_calls += 1;
            assert_prepared_startup(prepared, &["main", "tools"]);
            Ok(())
        })
        .expect("app preparation should invoke the injected runner once");
    assert_eq!(app_calls, 1);

    let mut into_loop_calls = 0;
    app(Noop)
        .open(open("main"))
        .into_loop()
        .run_prepared_with(|prepared| {
            into_loop_calls += 1;
            assert_prepared_startup(prepared, &["main"]);
            Ok(())
        })
        .expect("app into_loop preparation should invoke the injected runner once");
    assert_eq!(into_loop_calls, 1);
}

#[test]
fn startup_preflight_rejects_duplicate_names_before_native_effect() {
    struct Noop;
    impl Handler for Noop {}

    let mut calls = 0;
    let error = app(Noop)
        .open(open("main"))
        .open(open("main"))
        .run_prepared_with(|_| {
            calls += 1;
            Ok(())
        })
        .expect_err("duplicate startup names should fail before native construction");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert_eq!(calls, 0);
}

#[test]
fn startup_preflight_rejects_invalid_command_order_before_native_effect() {
    struct Noop;
    impl Handler for Noop {}

    let mut window_loop = Loop::new(Noop);
    window_loop.startup = vec![Command::SetTitle {
        id: Id::from_u64(1),
        title: String::from("too early"),
    }];

    let mut calls = 0;
    let error = window_loop
        .run_prepared_with(|_| {
            calls += 1;
            Ok(())
        })
        .expect_err("startup commands must begin with normalized opens");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(calls, 0);
}

#[test]
fn startup_preflight_rejects_invalid_virtual_lifecycle_before_native_effect() {
    struct Noop;
    impl Handler for Noop {}

    let mut calls = 0;
    let error = app(Noop)
        .open(open("dialog").dialog(Id::from_u64(1)))
        .run_prepared_with(|_| {
            calls += 1;
            Ok(())
        })
        .expect_err("startup lifecycle must not contain a child window without a live parent");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(calls, 0);
}

#[test]
fn fake_and_native_intrinsic_normalization_match() {
    struct Noop;
    impl Handler for Noop {}

    let command = Command::Open {
        request: WindowRequest::builder("main")
            .inner_size(Size {
                width: 40.0,
                height: 20.0,
            })
            .min_inner_size(Size {
                width: 80.0,
                height: 60.0,
            })
            .max_inner_size(Size {
                width: 120.0,
                height: 100.0,
            })
            .build(),
    };

    let mut fake = testing::Host::new();
    fake.apply(command.clone())
        .expect("fake host should accept and canonically record the request");

    let expected = Command::Open {
        request: WindowRequest::builder("main")
            .inner_size(Size {
                width: 80.0,
                height: 60.0,
            })
            .min_inner_size(Size {
                width: 80.0,
                height: 60.0,
            })
            .max_inner_size(Size {
                width: 120.0,
                height: 100.0,
            })
            .build(),
    };
    assert_eq!(fake.commands(), std::slice::from_ref(&expected));

    let mut native_runner = WinitRunner::from_loop(Loop::new(Noop));
    native_runner.resolve_capabilities_for_test(conservative_capabilities());
    let native = native_runner
        .plan_command_for_test(command)
        .expect("native runtime planning should normalize the same authored command");

    assert_eq!(native.kind(), CommandKind::Open);
    assert_eq!(native.target(), None);
    assert_eq!(take_planned_command(native).into_command(), expected);
}

#[test]
fn open_preflight_checks_only_nondefault_authored_attributes() {
    let capabilities = fully_supported_capabilities();
    let default = HostCommandPlan::from_command(
        Command::Open {
            request: WindowRequest::default(),
        },
        &capabilities,
    )
    .expect("the default open should use only the always-checked role and fullscreen facts");
    assert_eq!(
        default
            .capability_decisions()
            .iter()
            .map(CapabilityDecision::kind)
            .collect::<Vec<_>>(),
        vec![
            CapabilityKind::Role(RoleKind::Root),
            CapabilityKind::Fullscreen(FullscreenMode::None),
        ]
    );

    let authored = HostCommandPlan::from_command(
        Command::Open {
            request: WindowRequest::builder("authored")
                .title("Authored")
                .position(Point { x: 12.0, y: 34.0 })
                .hidden()
                .fixed()
                .controls(Controls {
                    close: false,
                    minimize: false,
                    maximize: false,
                })
                .decorations(false)
                .transparent(true)
                .inner_size(Size {
                    width: 640.0,
                    height: 480.0,
                })
                .min_inner_size(Size {
                    width: 320.0,
                    height: 240.0,
                })
                .max_inner_size(Size {
                    width: 1280.0,
                    height: 960.0,
                })
                .level(Level::AlwaysOnTop)
                .theme(Theme::Dark)
                .build(),
        },
        &capabilities,
    )
    .expect("non-default authored open attributes should use their exact report cells");
    assert_eq!(
        authored
            .capability_decisions()
            .iter()
            .map(CapabilityDecision::kind)
            .collect::<Vec<_>>(),
        vec![
            CapabilityKind::Role(RoleKind::Root),
            CapabilityKind::Fullscreen(FullscreenMode::None),
            CapabilityKind::Window(WindowOperation::SetTitle),
            CapabilityKind::Window(WindowOperation::SetOuterPosition),
            CapabilityKind::Window(WindowOperation::SetVisible),
            CapabilityKind::Window(WindowOperation::SetResizable),
            CapabilityKind::Window(WindowOperation::SetControls),
            CapabilityKind::Window(WindowOperation::SetDecorations),
            CapabilityKind::Window(WindowOperation::InitialTransparency),
            CapabilityKind::Window(WindowOperation::RequestInnerSize),
            CapabilityKind::Window(WindowOperation::SetInnerSizeBounds),
            CapabilityKind::Window(WindowOperation::SetLevel),
            CapabilityKind::Window(WindowOperation::SetExplicitTheme),
        ]
    );
}

#[test]
fn capability_preflight_records_every_runtime_operation() {
    let capabilities = fully_supported_capabilities();
    let id = Id::from_u64(41);
    let config = ImeConfig {
        purpose: ImePurpose::Normal,
        hint: ImeHint::None,
        cursor_area: Some(Rect {
            origin: Point { x: 1.0, y: 2.0 },
            size: Size {
                width: 3.0,
                height: 4.0,
            },
        }),
        surrounding_text: None,
    };
    let window_operations = [
        (
            Command::SetTitle {
                id,
                title: String::from("title"),
            },
            WindowOperation::SetTitle,
        ),
        (
            Command::SetPosition {
                id,
                position: Point { x: 1.0, y: 2.0 },
            },
            WindowOperation::SetOuterPosition,
        ),
        (
            Command::SetVisible { id, visible: false },
            WindowOperation::SetVisible,
        ),
        (
            Command::SetResizable {
                id,
                resizable: false,
            },
            WindowOperation::SetResizable,
        ),
        (
            Command::SetControls {
                id,
                controls: Controls {
                    close: false,
                    minimize: false,
                    maximize: false,
                },
            },
            WindowOperation::SetControls,
        ),
        (
            Command::SetDecorations {
                id,
                decorations: false,
            },
            WindowOperation::SetDecorations,
        ),
        (
            Command::SetTransparent {
                id,
                transparent: true,
            },
            WindowOperation::SetTransparent,
        ),
        (
            Command::SetInnerSize {
                id,
                size: Size {
                    width: 10.0,
                    height: 20.0,
                },
            },
            WindowOperation::RequestInnerSize,
        ),
        (
            Command::SetMinInnerSize {
                id,
                size: Some(Size {
                    width: 10.0,
                    height: 20.0,
                }),
            },
            WindowOperation::SetInnerSizeBounds,
        ),
        (
            Command::SetMaxInnerSize {
                id,
                size: Some(Size {
                    width: 10.0,
                    height: 20.0,
                }),
            },
            WindowOperation::SetInnerSizeBounds,
        ),
        (
            Command::SetLevel {
                id,
                level: Level::AlwaysOnTop,
            },
            WindowOperation::SetLevel,
        ),
        (
            Command::SetTheme {
                id,
                theme: Some(Theme::Dark),
            },
            WindowOperation::SetExplicitTheme,
        ),
        (
            Command::SetTheme { id, theme: None },
            WindowOperation::ResetTheme,
        ),
        (
            Command::RequestUserAttention { id },
            WindowOperation::RequestUserAttention,
        ),
        (Command::RequestDraw { id }, WindowOperation::RequestDraw),
        (Command::Destroy { id }, WindowOperation::Destroy),
    ];

    for (command, operation) in window_operations {
        let plan = HostCommandPlan::from_command(command, &capabilities)
            .expect("fully supported runtime operation should plan");
        assert_eq!(
            plan.capability_decisions(),
            [CapabilityDecision::new(
                CapabilityKind::Window(operation),
                CapabilitySupport::Supported,
            )]
        );
    }

    for grab in [CursorGrab::None, CursorGrab::Confined, CursorGrab::Locked] {
        let plan =
            HostCommandPlan::from_command(Command::SetCursorGrab { id, grab }, &capabilities)
                .expect("fully supported cursor grab should plan");
        assert_eq!(
            plan.capability_decisions(),
            [CapabilityDecision::new(
                CapabilityKind::CursorGrab(grab),
                CapabilitySupport::Supported,
            )]
        );
    }

    for (request, expected) in [
        (ImeRequest::Disable, vec![ImeCapability::Enablement]),
        (
            ImeRequest::Enable(config.clone()),
            vec![
                ImeCapability::Enablement,
                ImeCapability::Purpose(ImePurpose::Normal),
                ImeCapability::Hint(ImeHint::None),
                ImeCapability::CursorArea,
            ],
        ),
        (
            ImeRequest::Update(config.clone()),
            vec![
                ImeCapability::Purpose(ImePurpose::Normal),
                ImeCapability::Hint(ImeHint::None),
                ImeCapability::CursorArea,
            ],
        ),
        (
            ImeRequest::Restart(config),
            vec![
                ImeCapability::Enablement,
                ImeCapability::Purpose(ImePurpose::Normal),
                ImeCapability::Hint(ImeHint::None),
                ImeCapability::CursorArea,
            ],
        ),
    ] {
        let plan = HostCommandPlan::from_command(Command::SetIme { id, request }, &capabilities)
            .expect("fully supported IME request should plan");
        assert_eq!(
            plan.capability_decisions()
                .iter()
                .map(CapabilityDecision::kind)
                .collect::<Vec<_>>(),
            expected
                .into_iter()
                .map(CapabilityKind::Ime)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn fake_rejects_every_command_native_capabilities_reject() {
    let capabilities = HostCapabilities::builder()
        .role(RoleKind::Root, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
        .build();
    let mut host = testing::Host::with_capabilities(capabilities);
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("the selected report supports a default root open");
    let id = match host.events() {
        [HostEvent::Created(state)] => state.id(),
        events => panic!("expected one created event, got {events:?}"),
    };
    host.clear();

    let config = ImeConfig {
        purpose: ImePurpose::Normal,
        hint: ImeHint::None,
        cursor_area: None,
        surrounding_text: None,
    };
    let unsupported = vec![
        Command::SetTitle {
            id,
            title: String::from("title"),
        },
        Command::SetPosition {
            id,
            position: Point { x: 1.0, y: 2.0 },
        },
        Command::SetVisible { id, visible: false },
        Command::SetResizable {
            id,
            resizable: false,
        },
        Command::SetControls {
            id,
            controls: Controls::default(),
        },
        Command::SetDecorations {
            id,
            decorations: false,
        },
        Command::SetTransparent {
            id,
            transparent: true,
        },
        Command::SetInnerSize {
            id,
            size: Size {
                width: 1.0,
                height: 2.0,
            },
        },
        Command::SetMinInnerSize { id, size: None },
        Command::SetMaxInnerSize { id, size: None },
        Command::SetFullscreen {
            id,
            fullscreen: Fullscreen::Borderless,
        },
        Command::SetLevel {
            id,
            level: Level::AlwaysOnTop,
        },
        Command::SetTheme {
            id,
            theme: Some(Theme::Dark),
        },
        Command::SetTheme { id, theme: None },
        Command::SetCursor {
            id,
            cursor: Cursor::Icon(CursorIcon::Pointer),
        },
        Command::SetCursorGrab {
            id,
            grab: CursorGrab::Confined,
        },
        Command::SetIme {
            id,
            request: ImeRequest::Disable,
        },
        Command::SetIme {
            id,
            request: ImeRequest::Enable(config),
        },
        Command::RequestUserAttention { id },
    ];

    for command in unsupported {
        let expected_code = if matches!(&command, Command::SetIme { .. }) {
            ErrorCode::ImeUnsupported
        } else {
            ErrorCode::UnsupportedFeature
        };
        let error = host.apply(command).expect_err(
            "a fake host must reject every capability the selected native report rejects",
        );
        assert_eq!(error.code, expected_code);
        assert!(host.commands().is_empty());
        assert!(host.events().is_empty());
        assert!(host.take_ready_draws(Instant::now()).is_empty());
    }
}

#[test]
fn runtime_dependent_confined_grab_preserves_typed_backend_failure() {
    let capabilities = HostCapabilities::builder()
        .cursor_grab(CursorGrab::Confined, CapabilitySupport::RuntimeDependent)
        .build();
    let plan = HostCommandPlan::from_command(
        Command::SetCursorGrab {
            id: Id::from_u64(77),
            grab: CursorGrab::Confined,
        },
        &capabilities,
    )
    .expect("runtime-dependent confined grabs must reach native application");

    assert_eq!(
        plan.capability_decisions(),
        [CapabilityDecision::new(
            CapabilityKind::CursorGrab(CursorGrab::Confined),
            CapabilitySupport::RuntimeDependent,
        )]
    );

    let error = cursor_grab_failed(
        Id::from_u64(77),
        std::io::Error::other("Wayland rejected the confined cursor grab"),
    );
    assert_eq!(error.code, ErrorCode::CursorRequestFailed);
    assert_eq!(error.id, Some(Id::from_u64(77)));
    assert_eq!(
        error
            .source
            .expect("the native backend failure must be retained")
            .to_string(),
        "Wayland rejected the confined cursor grab"
    );
}

#[test]
fn backend_applicators_cannot_apply_authored_commands() {
    struct Noop;
    impl Handler for Noop {}

    let invalid = Command::Open {
        request: WindowRequest::builder("").build(),
    };

    let mut fake = testing::Host::new();
    let fake_error = fake
        .apply(invalid.clone())
        .expect_err("fake host must reject authored commands before recording or application");
    assert_eq!(fake_error.code, ErrorCode::InvalidRequest);
    assert!(fake.commands().is_empty());
    assert!(fake.events().is_empty());
    assert!(fake.registry().is_empty());

    let mut native_runner = WinitRunner::from_loop(Loop::new(Noop));
    native_runner.resolve_capabilities_for_test(conservative_capabilities());
    let native_error = native_runner
        .plan_command_for_test(invalid)
        .expect_err("native runtime must reject authored commands before application");
    assert_eq!(native_error.code, ErrorCode::InvalidRequest);
}

#[test]
fn host_command_plan_reports_validated_command_without_duplicate_host_enum() {
    let capabilities = conservative_capabilities();
    let command = Command::SetTitle {
        id: Id::from_u64(1),
        title: String::from("Renamed"),
    };
    let plan = HostCommandPlan::from_command(command, &capabilities).unwrap();

    assert_eq!(plan.kind(), CommandKind::SetTitle);
    assert_eq!(plan.target(), Some(Id::from_u64(1)));
    assert_eq!(
        plan.capability_decisions(),
        [CapabilityDecision::new(
            CapabilityKind::Window(WindowOperation::SetTitle),
            CapabilitySupport::Supported,
        )]
    );
}

#[test]
fn host_command_plan_keeps_backend_capability_rejections_before_application() {
    let capabilities = conservative_capabilities();
    let dialog = Command::Open {
        request: WindowRequest::builder("dialog")
            .title("Dialog")
            .dialog(Id::from_u64(1))
            .build(),
    };
    let exclusive_open = Command::Open {
        request: WindowRequest::builder("exclusive")
            .fullscreen(Fullscreen::Exclusive)
            .build(),
    };
    let exclusive_command = Command::SetFullscreen {
        id: Id::from_u64(1),
        fullscreen: Fullscreen::Exclusive,
    };
    let custom_cursor = Command::SetCursor {
        id: Id::from_u64(1),
        cursor: Cursor::Custom(CustomCursorId::from_u64(9)),
    };

    for command in [dialog, exclusive_open, exclusive_command, custom_cursor] {
        let error = HostCommandPlan::from_command(command, &capabilities)
            .expect_err("unsupported backend capability should be rejected during planning");

        assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    }
}

#[test]
fn host_command_enum_is_not_part_of_public_front_door() {
    let lib = include_str!("lib.rs");
    let duplicate_name = ["Host", "Command"].concat();
    let planning_export = "pub use planning::HostCommandPlan;";

    assert!(lib.contains(planning_export));
    assert!(!lib.contains("pub use transition::{HostCommandPlan"));
    assert!(!lib.contains(&format!("pub use command::{{{duplicate_name}")));
}

#[test]
fn modifier_state_converts_from_winit() {
    let modifiers = winit::keyboard::ModifiersState::SHIFT
        | winit::keyboard::ModifiersState::CONTROL
        | winit::keyboard::ModifiersState::SUPER;

    assert_eq!(
        ModifierState::from(modifiers),
        ModifierState {
            shift: true,
            control: true,
            alt: false,
            super_key: true,
        }
    );
}

#[test]
fn pointer_button_converts_from_winit() {
    assert_eq!(
        PointerButton::from(winit::event::MouseButton::Left),
        PointerButton::Primary
    );
    assert_eq!(
        PointerButton::from(winit::event::MouseButton::Other(42)),
        PointerButton::Other(42)
    );
}

#[test]
fn wheel_delta_converts_from_winit() {
    assert_eq!(
        WheelDelta::from(winit::event::MouseScrollDelta::LineDelta(1.5, -2.0)),
        WheelDelta::Lines { x: 1.5, y: -2.0 }
    );
    assert_eq!(
        WheelDelta::from(winit::event::MouseScrollDelta::PixelDelta(
            winit::dpi::PhysicalPosition::new(12.0, 24.0)
        )),
        WheelDelta::Pixels { x: 12.0, y: 24.0 }
    );
}

#[test]
fn keyboard_identity_converts_from_winit() {
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Character("x".into())),
        keyboard_types::Key::Character(String::from("x"))
    );
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Named(
            winit::keyboard::NamedKey::Enter
        )),
        keyboard_types::Key::Enter
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Code(
            winit::keyboard::KeyCode::KeyA
        )),
        keyboard_types::Code::KeyA
    );
    assert_eq!(
        location_from_winit(winit::keyboard::KeyLocation::Numpad),
        keyboard_types::Location::Numpad
    );
}

#[test]
fn dead_space_and_super_keys_preserve_semantic_identity() {
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Dead(Some('^'))),
        keyboard_types::Key::Dead
    );
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Named(
            winit::keyboard::NamedKey::Space
        )),
        keyboard_types::Key::Character(String::from(" "))
    );
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Named(
            winit::keyboard::NamedKey::Super
        )),
        keyboard_types::Key::Meta
    );
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Named(
            winit::keyboard::NamedKey::Meta
        )),
        keyboard_types::Key::Super
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Code(
            winit::keyboard::KeyCode::SuperLeft
        )),
        keyboard_types::Code::MetaLeft
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Code(
            winit::keyboard::KeyCode::SuperRight
        )),
        keyboard_types::Code::MetaRight
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Code(
            winit::keyboard::KeyCode::Meta
        )),
        keyboard_types::Code::Super
    );
}

#[test]
fn identical_named_keys_use_checked_mapping() {
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::F1)),
        keyboard_types::Key::F1
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Code(
            winit::keyboard::KeyCode::KeyA
        )),
        keyboard_types::Code::KeyA
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Code(
            winit::keyboard::KeyCode::NumpadMemorySubtract
        )),
        keyboard_types::Code::NumpadMemorySubtract
    );
}

#[test]
fn unknown_keys_remain_unidentified() {
    assert_eq!(
        key_from_winit(&winit::keyboard::Key::Unidentified(
            winit::keyboard::NativeKey::Unidentified
        )),
        keyboard_types::Key::Unidentified
    );
    assert_eq!(
        code_from_winit(&winit::keyboard::PhysicalKey::Unidentified(
            winit::keyboard::NativeKeyCode::Unidentified
        )),
        keyboard_types::Code::Unidentified
    );
}

#[test]
fn ime_event_converts_from_winit() {
    assert_eq!(
        ime_event_from_winit(
            Id::from_u64(1),
            winit::event::Ime::Preedit(String::from("draft"), Some((1, 2))),
        ),
        ImeEvent::Preedit {
            id: Id::from_u64(1),
            text: String::from("draft"),
            cursor: Some((1, 2)),
        }
    );
    assert_eq!(
        ime_event_from_winit(
            Id::from_u64(1),
            winit::event::Ime::Commit(String::from("done"))
        ),
        ImeEvent::Commit {
            id: Id::from_u64(1),
            text: String::from("done"),
        }
    );
}

#[test]
fn fake_host_applies_open_and_state_commands() {
    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::builder("fake")
            .title("Fake")
            .position(Point { x: 11.0, y: 22.0 })
            .inner_size(Size {
                width: 320.0,
                height: 240.0,
            })
            .theme(Some(Theme::Light))
            .build(),
    })
    .unwrap();

    let id = match &host.events()[0] {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected Created event, got {event:?}"),
    };

    host.apply(Command::SetTitle {
        id,
        title: String::from("Renamed"),
    })
    .unwrap();
    host.apply(Command::SetVisible { id, visible: false })
        .unwrap();
    host.apply(Command::SetInnerSize {
        id,
        size: Size {
            width: 640.0,
            height: 480.0,
        },
    })
    .unwrap();

    let state = host.registry().get(id).unwrap();
    assert_eq!(state.instance.state.title(), "Renamed");
    assert_eq!(state.instance.state.visible(), Some(false));
    assert_eq!(
        state.instance.state.metrics().logical_size(),
        Size {
            width: 640.0,
            height: 480.0,
        }
    );
    assert_eq!(
        state.instance.state.metrics().outer_position(),
        Some(Point { x: 11.0, y: 22.0 })
    );
    assert!(
        host.events()
            .iter()
            .any(|event| matches!(event, HostEvent::Resized(metrics) if metrics.id() == id))
    );
}

#[test]
fn fake_host_exercises_draw_and_close_contract() {
    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .unwrap();
    let id = match &host.events()[0] {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected Created event, got {event:?}"),
    };

    host.apply(Command::RequestDraw { id }).unwrap();
    assert_eq!(host.take_ready_draws(Instant::now()), vec![id]);

    host.apply(Command::Destroy { id }).unwrap();
    assert!(!host.registry().contains(id));
    assert!(
        host.events()
            .iter()
            .any(|event| matches!(event, HostEvent::Destroyed(destroyed) if *destroyed == id))
    );
}

#[test]
fn host_clear_preserves_completed_lifecycle_state() {
    let mut host = testing::Host::new();
    host.apply(open("main"))
        .expect("the low-level host opens the fixture window");
    let id = host.window_id("main").expect("fixture window is live");
    host.apply(Command::Destroy { id })
        .expect("the low-level host completes destroy synchronously");

    host.clear();

    assert!(!host.registry().contains(id));
    assert!(host.events().is_empty());
    assert!(host.commands().is_empty());
    let error = host
        .apply(Command::RequestDraw { id })
        .expect_err("clearing recordings must not reopen a closed generation");
    assert_eq!(error.code, ErrorCode::StaleWindow);
    assert!(host.events().is_empty());
    assert!(host.commands().is_empty());
}

#[test]
fn fake_host_reports_unknown_command_target() {
    let mut host = testing::Host::new();
    let id = Id::from_u64(404);

    let error = host
        .apply(Command::RequestDraw { id })
        .expect_err("unknown window should fail");

    assert_eq!(error.code, ErrorCode::StaleWindow);
    assert_eq!(error.id, Some(id));
}

#[test]
fn predictable_batch_failure_applies_no_commands() {
    let mut host = testing::Host::new();
    host.apply(open("main").title("Original")).unwrap();
    let id = host.window_id("main").expect("main window is live");
    let before = host
        .registry()
        .get(id)
        .expect("main window remains live")
        .instance
        .state()
        .clone();
    host.clear();

    let error = host
        .apply_batch_for_test(vec![
            Command::SetTitle {
                id,
                title: String::from("Updated"),
            },
            open("main").into(),
        ])
        .expect_err("the duplicate open must reject the complete batch before effects");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert_eq!(error.command_kind, Some(CommandKind::Open));
    assert_eq!(error.id, None);
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert_eq!(
        host.registry()
            .get(id)
            .expect("main window remains live")
            .instance
            .state(),
        &before
    );
}

#[test]
fn batch_preflight_rejects_duplicate_names_against_virtual_registry() {
    let mut host = testing::Host::new();

    let error = host
        .apply_batch_for_test(vec![open("main").into(), open("main").into()])
        .expect_err("a later open must see the virtual name created by the earlier open");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert_eq!(error.command_kind, Some(CommandKind::Open));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.registry().is_empty());

    host.apply(open("main"))
        .expect("preflight rejection must not advance the real allocator");
    assert_eq!(host.window_id("main"), Some(Id::from_u64(1)));
}

#[test]
fn batch_preflight_unknown_and_post_destroy_report_lifecycle_errors() {
    let mut host = testing::Host::new();
    let unknown = Id::from_u64(404);

    let unknown_error = host
        .apply_batch_for_test(vec![Command::RequestDraw { id: unknown }])
        .expect_err("an unknown target must be rejected during preflight");
    assert_eq!(unknown_error.code, ErrorCode::StaleWindow);
    assert_eq!(unknown_error.command_kind, Some(CommandKind::RequestDraw));
    assert_eq!(unknown_error.id, Some(unknown));
    assert!(host.commands().is_empty());

    host.apply(open("main")).unwrap();
    let id = host.window_id("main").expect("main window is live");
    host.clear();

    let destroyed_error = host
        .apply_batch_for_test(vec![
            Command::Destroy { id },
            Command::SetTitle {
                id,
                title: String::from("too late"),
            },
        ])
        .expect_err("a target destroyed earlier in the batch must be absent in the projection");
    assert_eq!(destroyed_error.code, ErrorCode::WindowClosing);
    assert_eq!(destroyed_error.command_kind, Some(CommandKind::SetTitle));
    assert_eq!(destroyed_error.id, Some(id));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.registry().contains(id));
}

#[test]
fn lifecycle_planning_distinguishes_closing_and_stale_targets() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let closing = host.window_id("main").expect("main window is live");
    host.clear();
    host.begin_close_for_test(closing)
        .expect("test fixture enters closing");

    let closing_error = host
        .apply(Command::RequestDraw { id: closing })
        .expect_err("a known closing target must be rejected during planning");
    assert_eq!(closing_error.code, ErrorCode::WindowClosing);
    assert_eq!(closing_error.id, Some(closing));
    assert_eq!(closing_error.command_kind, Some(CommandKind::RequestDraw));

    let stale = Id::from_u64(404);
    let stale_error = host
        .apply(Command::RequestDraw { id: stale })
        .expect_err("an unknown target must be rejected during planning");
    assert_eq!(stale_error.code, ErrorCode::StaleWindow);
    assert_eq!(stale_error.id, Some(stale));
    assert_eq!(stale_error.command_kind, Some(CommandKind::RequestDraw));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
}

#[test]
fn lifecycle_synchronous_host_unknown_and_closed_targets_return_stale_window() {
    let mut host = testing::Host::new();
    let unknown = Id::from_u64(404);

    let unknown_error = host
        .apply(Command::RequestDraw { id: unknown })
        .expect_err("an unknown strict target must be stale");
    assert_eq!(unknown_error.code, ErrorCode::StaleWindow);
    assert_eq!(unknown_error.id, Some(unknown));
    assert_eq!(unknown_error.command_kind, Some(CommandKind::RequestDraw));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.take_ready_draws(Instant::now()).is_empty());

    host.apply(open("main")).unwrap();
    let closed = host.window_id("main").expect("main window is live");
    assert_eq!(closed, Id::from_u64(1));
    host.apply(Command::Destroy { id: closed }).unwrap();
    host.clear();

    let closed_error = host
        .apply(Command::SetTitle {
            id: closed,
            title: String::from("too late"),
        })
        .expect_err("a closed strict target must be stale");
    assert_eq!(closed_error.code, ErrorCode::StaleWindow);
    assert_eq!(closed_error.id, Some(closed));
    assert_eq!(closed_error.command_kind, Some(CommandKind::SetTitle));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.take_ready_draws(Instant::now()).is_empty());

    host.apply(open("after"))
        .expect("stale rejection must not advance allocation");
    assert_eq!(host.window_id("after"), Some(Id::from_u64(2)));
}

#[test]
fn lifecycle_synchronous_host_closing_target_returns_window_closing() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").expect("main window is live");
    host.clear();
    host.begin_close_for_test(id)
        .expect("test fixture enters closing");

    let error = host
        .apply(Command::SetTitle {
            id,
            title: String::from("too late"),
        })
        .expect_err("a closing strict target must report its lifecycle state");
    assert_eq!(error.code, ErrorCode::WindowClosing);
    assert_eq!(error.id, Some(id));
    assert_eq!(error.command_kind, Some(CommandKind::SetTitle));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());

    host.apply(open("main"))
        .expect("a closing window name is available to a new live window");
    assert_eq!(host.window_id("main"), Some(Id::from_u64(2)));
}

#[test]
fn lifecycle_destroy_then_target_batch_rejects_before_effects() {
    let mut host = testing::Host::new();
    host.apply(open("main").title("Original")).unwrap();
    let id = host.window_id("main").expect("main window is live");
    let original = host
        .registry()
        .get(id)
        .expect("main window remains live")
        .instance
        .state()
        .clone();

    for (commands, rejected_kind) in [
        (
            vec![
                Command::Destroy { id },
                Command::SetTitle {
                    id,
                    title: String::from("too late"),
                },
            ],
            CommandKind::SetTitle,
        ),
        (
            vec![Command::Destroy { id }, Command::Destroy { id }],
            CommandKind::Destroy,
        ),
    ] {
        host.clear();
        let error = host
            .apply_batch_for_test(commands)
            .expect_err("a target after virtual destroy must reject the full batch");

        assert_eq!(error.code, ErrorCode::WindowClosing);
        assert_eq!(error.id, Some(id));
        assert_eq!(error.command_kind, Some(rejected_kind));
        assert!(host.commands().is_empty());
        assert!(host.events().is_empty());
        assert_eq!(
            host.registry()
                .get(id)
                .expect("rejected preflight leaves the window live")
                .instance
                .state(),
            &original
        );
    }
}

#[test]
fn backend_batch_failure_reports_applied_prefix_and_rejects_tail() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").expect("main window is live");
    host.clear();
    host.fail_command_at_for_test(1);

    let error = host
        .apply_batch_for_test(vec![
            Command::SetTitle {
                id,
                title: String::from("Applied"),
            },
            Command::SetVisible { id, visible: false },
            Command::SetTheme {
                id,
                theme: Some(Theme::Dark),
            },
        ])
        .expect_err("the injected backend failure must terminate the batch");

    assert_eq!(error.code, ErrorCode::CommandBatchFailed);
    assert_eq!(error.command_kind, Some(CommandKind::SetVisible));
    assert_eq!(error.id, Some(id));
    assert_eq!(error.completed_prefix, 1);
    assert_eq!(
        error
            .source
            .as_deref()
            .and_then(|source| source.downcast_ref::<Error>())
            .map(|source| source.code),
        Some(ErrorCode::CommandFailed)
    );
    assert_eq!(
        host.commands(),
        &[Command::SetTitle {
            id,
            title: String::from("Applied"),
        }]
    );
    let state = host.registry().get(id).expect("prefix remains committed");
    assert_eq!(state.instance.state().title(), "Applied");
    assert_eq!(state.instance.state().visible(), Some(true));
    assert_eq!(state.instance.state().theme(), None);
}

#[test]
fn command_batch_error_records_kind_target_and_completed_prefix() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").expect("main window is live");
    host.clear();
    host.fail_command_at_for_test(0);

    let error = host
        .apply_batch_for_test(vec![Command::SetVisible { id, visible: false }])
        .expect_err("the first backend failure must have an empty completed prefix");

    assert_eq!(error.code, ErrorCode::CommandBatchFailed);
    assert_eq!(error.command_kind, Some(CommandKind::SetVisible));
    assert_eq!(error.id, Some(id));
    assert_eq!(error.completed_prefix, 0);
    assert!(host.commands().is_empty());
    assert_eq!(
        host.registry()
            .get(id)
            .expect("failing command has no committed state change")
            .instance
            .state()
            .visible(),
        Some(true)
    );
}

#[test]
fn fake_host_records_failed_direct_command_without_state_or_event_side_effects() {
    let mut host = testing::Host::new();
    let id = Id::from_u64(404);

    let error = host
        .apply(Command::RequestDraw { id })
        .expect_err("unknown draw target should fail");

    assert_eq!(error.code, ErrorCode::StaleWindow);
    assert_eq!(error.id, Some(id));
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.take_ready_draws(Instant::now()).is_empty());
}

#[test]
fn fake_host_duplicate_open_rejects_without_recording_or_allocating() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let live_id = host.window_id("main").expect("main window is live");
    let before = host
        .registry()
        .get(live_id)
        .expect("main window remains available")
        .instance
        .state()
        .clone();
    host.clear();

    let error = host
        .apply(open("main"))
        .expect_err("duplicate name should fail");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert_eq!(host.registry().len(), 1);
    assert_eq!(host.window_id("main"), Some(live_id));
    assert_eq!(
        host.registry()
            .get(live_id)
            .expect("main window remains available")
            .instance
            .state(),
        &before
    );

    host.apply(open("tools"))
        .expect("duplicate rejection must not advance the allocator");
    let created_id = match host.events() {
        [HostEvent::Created(state)] => state.id(),
        events => panic!("expected one tools creation after rejection, got {events:?}"),
    };
    assert_eq!(created_id, Id::from_u64(2));
}

#[test]
fn window_modeling_baseline_fake_host_rejects_duplicate_without_recording_command() {
    let mut host = testing::Host::new();

    host.apply(open("main")).unwrap();
    let error = host
        .apply(open("main"))
        .expect_err("duplicate name should fail");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
    assert_eq!(host.commands().len(), 1);
    assert_eq!(host.registry().len(), 1);
}

#[test]
fn window_modeling_baseline_fake_host_destroy_removes_window_cursor_and_records_closed_state() {
    let mut host = testing::Host::new();

    host.apply(open("main")).unwrap();
    let id = host.window_id("main").expect("main window should exist");
    host.apply(Command::SetCursor {
        id,
        cursor: Cursor::Hidden,
    })
    .unwrap();
    host.apply(Command::Destroy { id }).unwrap();

    assert!(host.registry().get(id).is_none());
    assert!(
        host.events()
            .iter()
            .any(|event| matches!(event, testing::Event::Destroyed(destroyed) if *destroyed == id))
    );
}

#[test]
fn fake_host_rejects_unsupported_commands_through_host_planning() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();

    let cursor_error = host
        .apply(Command::SetCursor {
            id,
            cursor: Cursor::Custom(CustomCursorId::from_u64(1)),
        })
        .expect_err("custom cursor should be rejected by capabilities");
    let fullscreen_error = host
        .apply(Command::SetFullscreen {
            id,
            fullscreen: Fullscreen::Exclusive,
        })
        .expect_err("exclusive fullscreen should be rejected by capabilities");

    assert_eq!(cursor_error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(fullscreen_error.code, ErrorCode::UnsupportedFeature);
}

#[test]
fn fake_host_uses_shared_state_patch_for_visible_and_theme_commands() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();

    host.apply(Command::SetVisible { id, visible: false })
        .unwrap();
    host.apply(Command::SetTheme {
        id,
        theme: Some(Theme::Dark),
    })
    .unwrap();

    let state = host.registry().get(id).unwrap();
    let state = state.instance.state();
    assert_eq!(state.visible(), Some(false));
    assert_eq!(state.theme(), Some(Theme::Dark));
    assert!(host.events().iter().any(|event| {
        matches!(event, testing::Event::ThemeChanged { id: event_id, theme: Some(Theme::Dark) } if *event_id == id)
    }));
}

#[test]
fn fake_host_deduplicates_cursor_updates() {
    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .unwrap();
    let id = match &host.events()[0] {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected Created event, got {event:?}"),
    };

    host.apply(Command::SetCursor {
        id,
        cursor: Cursor::Icon(CursorIcon::Pointer),
    })
    .unwrap();
    host.apply(Command::SetCursor {
        id,
        cursor: Cursor::Icon(CursorIcon::Pointer),
    })
    .unwrap();
    host.apply(Command::SetCursor {
        id,
        cursor: Cursor::Hidden,
    })
    .unwrap();

    assert_eq!(
        host.cursor_updates(),
        &[
            (id, Cursor::Icon(CursorIcon::Pointer)),
            (id, Cursor::Hidden)
        ]
    );
}

#[test]
fn fake_host_records_ime_request_order() {
    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .unwrap();
    let id = match &host.events()[0] {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected Created event, got {event:?}"),
    };
    let config = ImeConfig {
        purpose: ImePurpose::Normal,
        hint: ImeHint::None,
        cursor_area: Some(Rect {
            origin: Point { x: 1.0, y: 2.0 },
            size: Size {
                width: 3.0,
                height: 4.0,
            },
        }),
        surrounding_text: None,
    };

    host.apply(Command::SetIme {
        id,
        request: ImeRequest::Disable,
    })
    .unwrap();
    host.apply(Command::SetIme {
        id,
        request: ImeRequest::Enable(config.clone()),
    })
    .unwrap();
    host.apply(Command::SetIme {
        id,
        request: ImeRequest::Restart(config.clone()),
    })
    .unwrap();

    assert_eq!(
        host.ime_requests(),
        &[
            (id, ImeRequest::Disable),
            (id, ImeRequest::Enable(config.clone())),
            (id, ImeRequest::Restart(config)),
        ]
    );
}

fn supported_ime_config(purpose: ImePurpose) -> ImeConfig {
    ImeConfig {
        purpose,
        hint: ImeHint::None,
        cursor_area: Some(Rect {
            origin: Point { x: 1.0, y: 2.0 },
            size: Size {
                width: 3.0,
                height: 4.0,
            },
        }),
        surrounding_text: None,
    }
}

fn planned_ime_operations(request: ImeRequest) -> Vec<winit_mapping::NativeImeOperation> {
    let plan = HostCommandPlan::from_command(
        Command::SetIme {
            id: Id::from_u64(1),
            request,
        },
        &fully_supported_capabilities(),
    )
    .expect("supported IME request should produce a backend plan");
    let request = take_planned_ime_request(plan).expect("IME plan carries its resolved request");

    winit_mapping::native_ime_operations(&request)
}

#[test]
fn ime_disable_clears_active_state_without_reapplying_configuration() {
    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("fixture window opens before IME configuration");
    let id = host.events()[0].id();
    host.apply(Command::SetIme {
        id,
        request: ImeRequest::Enable(supported_ime_config(ImePurpose::Normal)),
    })
    .expect("supported configuration applies");

    host.apply(Command::SetIme {
        id,
        request: ImeRequest::Disable,
    })
    .expect("disable applies");

    assert!(!host.ime_active_for_test(id));
    assert!(host.ime_configuration_for_test(id).is_none());
    assert!(matches!(
        host.resolved_ime_requests_for_test().last(),
        Some((recorded_id, crate::planning::ResolvedImeRequest::Disable)) if *recorded_id == id
    ));
    assert_eq!(
        planned_ime_operations(ImeRequest::Disable),
        [winit_mapping::NativeImeOperation::Allowed(false)]
    );
}

#[test]
fn ime_enable_update_and_restart_apply_supported_cursor_area() {
    let config = supported_ime_config(ImePurpose::Normal);
    let expected = [
        winit_mapping::NativeImeOperation::Purpose(ResolvedImePurpose::Normal),
        winit_mapping::NativeImeOperation::CursorArea(
            config.cursor_area.expect("fixture includes cursor area"),
        ),
    ];

    for request in [
        ImeRequest::Enable(config.clone()),
        ImeRequest::Update(config.clone()),
        ImeRequest::Restart(config),
    ] {
        let operations = planned_ime_operations(request);
        assert!(operations.ends_with(&expected));
    }
}

#[test]
fn ime_update_does_not_toggle_enablement() {
    let operations =
        planned_ime_operations(ImeRequest::Update(supported_ime_config(ImePurpose::Normal)));

    assert!(
        !operations
            .iter()
            .any(|operation| matches!(operation, winit_mapping::NativeImeOperation::Allowed(_))),
        "updates must not toggle IME enablement: {operations:?}"
    );
}

#[test]
fn ime_restart_reapplies_purpose_and_cursor_area() {
    let config = supported_ime_config(ImePurpose::Normal);
    assert_eq!(
        planned_ime_operations(ImeRequest::Restart(config.clone())),
        [
            winit_mapping::NativeImeOperation::Allowed(false),
            winit_mapping::NativeImeOperation::Allowed(true),
            winit_mapping::NativeImeOperation::Purpose(ResolvedImePurpose::Normal),
            winit_mapping::NativeImeOperation::CursorArea(
                config.cursor_area.expect("fixture includes cursor area"),
            ),
        ]
    );
}

#[test]
fn ime_none_hint_is_preserved_without_native_operation() {
    let plan = HostCommandPlan::from_command(
        Command::SetIme {
            id: Id::from_u64(1),
            request: ImeRequest::Enable(supported_ime_config(ImePurpose::Normal)),
        },
        &fully_supported_capabilities(),
    )
    .expect("none hint is supported");
    let request = take_planned_ime_request(plan).expect("IME plan carries its resolved request");
    let operations = winit_mapping::native_ime_operations(&request);

    assert!(request.has_none_hint());
    assert_eq!(operations.len(), 3, "none hint adds no native operation");
}

#[test]
fn ime_rejects_unrepresentable_purpose_hint_and_surrounding_text() {
    let id = Id::from_u64(1);
    let unsupported = [
        ImeRequest::Enable(supported_ime_config(ImePurpose::Number)),
        ImeRequest::Enable(supported_ime_config(ImePurpose::Email)),
        ImeRequest::Enable(supported_ime_config(ImePurpose::Url)),
        ImeRequest::Enable(ImeConfig {
            hint: ImeHint::Spellcheck,
            ..supported_ime_config(ImePurpose::Normal)
        }),
        ImeRequest::Enable(ImeConfig {
            hint: ImeHint::NoSpellcheck,
            ..supported_ime_config(ImePurpose::Normal)
        }),
        ImeRequest::Enable(ImeConfig {
            surrounding_text: Some(ImeSurroundingText {
                text: String::from("complete contract unavailable"),
                cursor: 3,
                anchor: 0,
            }),
            ..supported_ime_config(ImePurpose::Normal)
        }),
    ];

    for request in unsupported {
        let error = HostCommandPlan::from_command(
            Command::SetIme { id, request },
            &fully_supported_capabilities(),
        )
        .expect_err("unrepresentable IME semantics must fail during planning");
        assert_eq!(error.code, ErrorCode::ImeUnsupported);
    }
}

#[test]
fn fake_host_rejects_unsupported_ime_cursor_area_without_effects() {
    let capabilities = HostCapabilities::builder()
        .role(RoleKind::Root, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
        .ime(ImeCapability::Enablement, CapabilitySupport::Supported)
        .ime(
            ImeCapability::Purpose(ImePurpose::Normal),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Hint(ImeHint::None),
            CapabilitySupport::Supported,
        )
        .build();
    let mut host = testing::Host::with_capabilities(capabilities);
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("the selected report supports a default root open");
    let id = host.events()[0].id();
    let before_state = host
        .registry()
        .get(id)
        .expect("opened window remains live")
        .instance
        .state()
        .clone();
    host.clear();

    let error = host
        .apply(Command::SetIme {
            id,
            request: ImeRequest::Enable(supported_ime_config(ImePurpose::Normal)),
        })
        .expect_err("unsupported IME cursor area must fail before fake application");

    assert_eq!(error.code, ErrorCode::ImeUnsupported);
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.cursor_updates().is_empty());
    assert!(host.ime_requests().is_empty());
    assert!(host.resolved_ime_requests_for_test().is_empty());
    assert!(!host.ime_active_for_test(id));
    assert!(host.ime_configuration_for_test(id).is_none());
    assert!(host.take_ready_draws(Instant::now()).is_empty());
    assert_eq!(
        host.registry()
            .get(id)
            .expect("rejected request keeps the window live")
            .instance
            .state(),
        &before_state
    );
}

#[test]
fn fake_host_emits_lifecycle_events() {
    let mut host = testing::Host::new();
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .unwrap();
    let id = match &host.events()[0] {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected Created event, got {event:?}"),
    };
    host.clear();

    host.suspend(id).unwrap();
    host.resume(id).unwrap();

    assert_eq!(
        host.events(),
        &[HostEvent::Suspended(id), HostEvent::Resumed(id)]
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn fake_host_forwards_accessibility_events() {
    let mut host = testing::Host::with_capabilities(fully_supported_capabilities());
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .unwrap();
    let id = match &host.events()[0] {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected Created event, got {event:?}"),
    };
    host.clear();

    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .unwrap();
    host.accessibility(AccessibilityEvent::ActionRequested(
        AccessibilityActionRequest {
            id,
            request: accesskit::ActionRequest {
                action: accesskit::Action::Click,
                target_tree: accesskit::TreeId::ROOT,
                target_node: accesskit::NodeId(42),
                data: None,
            },
        },
    ))
    .unwrap();
    host.accessibility(AccessibilityEvent::Deactivated(id))
        .unwrap();

    assert_eq!(
        host.events(),
        &[
            HostEvent::Accessibility(AccessibilityEvent::InitialTreeRequested(id)),
            HostEvent::Accessibility(AccessibilityEvent::ActionRequested(
                AccessibilityActionRequest {
                    id,
                    request: accesskit::ActionRequest {
                        action: accesskit::Action::Click,
                        target_tree: accesskit::TreeId::ROOT,
                        target_node: accesskit::NodeId(42),
                        data: None,
                    },
                }
            )),
            HostEvent::Accessibility(AccessibilityEvent::Deactivated(id)),
        ]
    );
}

#[cfg(feature = "accessibility")]
fn initial_accessibility_tree() -> accesskit::TreeUpdate {
    let root_id = accesskit::NodeId(42);
    let root = accesskit::Node::new(accesskit::Role::Window);

    accesskit::TreeUpdate {
        nodes: vec![(root_id, root)],
        tree: Some(accesskit::Tree::new(root_id)),
        tree_id: accesskit::TreeId::ROOT,
        focus: root_id,
    }
}

#[cfg(feature = "accessibility")]
fn incremental_accessibility_tree() -> accesskit::TreeUpdate {
    accesskit::TreeUpdate {
        nodes: Vec::new(),
        tree: None,
        tree_id: accesskit::TreeId::ROOT,
        focus: accesskit::NodeId(42),
    }
}

#[cfg(feature = "accessibility")]
fn fake_accessibility_host() -> (testing::Host, Id) {
    let mut host = testing::Host::with_capabilities(fully_supported_capabilities());
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("supported fake window opens");
    let id = host.events()[0].id();
    host.clear();
    (host, id)
}

#[cfg(feature = "accessibility")]
#[test]
fn test_runner_accessibility_pre_resume_is_terminal_without_effects() {
    #[derive(Default)]
    struct Recorder {
        callbacks: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
            self.callbacks += 1;
            Ok(())
        }
    }

    let id = Id::from_u64(1);
    let mut runner = testing::Runner::new(Recorder::default());

    runner.accessibility(AccessibilityEvent::InitialTreeRequested(id));

    let error = runner
        .result()
        .expect("pre-resume accessibility is terminal")
        .expect_err("pre-resume accessibility is invalid");
    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert!(
        runner.host().events().is_empty(),
        "no host fact is recorded"
    );
    assert!(runner.host().commands().is_empty(), "no command is applied");
    assert_eq!(
        runner.host().accessibility_phase_for_test(id),
        None,
        "accessibility phase does not change"
    );
    assert_eq!(runner.handler().callbacks, 0, "no handler callback runs");
    assert!(runner.host().accessibility_updates_for_test().is_empty());

    runner.accessibility(AccessibilityEvent::InitialTreeRequested(id));

    assert!(
        runner.host().events().is_empty(),
        "terminal ingress is a no-op"
    );
    assert!(
        runner.host().commands().is_empty(),
        "terminal ingress applies nothing"
    );
    assert_eq!(
        runner.handler().callbacks,
        0,
        "terminal ingress invokes nobody"
    );
    assert!(runner.host().accessibility_updates_for_test().is_empty());
}

#[cfg(feature = "accessibility")]
#[test]
fn test_runner_accessibility_initial_tree_reaches_active_phase() {
    struct InitialTreeHandler {
        update: accesskit::TreeUpdate,
        observed: Option<Id>,
    }

    impl Handler for InitialTreeHandler {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            let EventKind::Accessibility(AccessibilityEvent::InitialTreeRequested(id)) =
                event.event()
            else {
                return Ok(());
            };
            let id = *id;
            event
                .context_mut()
                .update_accessibility(id, self.update.clone());
            self.observed = Some(id);
            Ok(())
        }
    }

    let update = initial_accessibility_tree();
    let mut runner = testing::Runner::with_capabilities(
        InitialTreeHandler {
            update: update.clone(),
            observed: None,
        },
        fully_supported_capabilities(),
    );
    runner
        .startup(vec![open("accessible").into()])
        .expect("startup is configured before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("accessible")
        .expect("resume opens the live accessibility target");

    runner.accessibility(AccessibilityEvent::InitialTreeRequested(id));

    assert_eq!(runner.handler().observed, Some(id));
    assert_eq!(
        runner.host().commands().last(),
        Some(&Command::UpdateAccessibility {
            id,
            update: update.clone(),
        })
    );
    assert_eq!(
        runner.host().accessibility_updates_for_test(),
        &[(id, update)]
    );
    assert_eq!(
        runner.host().accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Active)
    );
    assert!(runner.result().is_none());
}

#[cfg(feature = "accessibility")]
#[test]
fn test_runner_accessibility_unsupported_is_terminal_without_callback() {
    #[derive(Default)]
    struct Recorder {
        callbacks: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
            self.callbacks += 1;
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("unsupported").into()])
        .expect("startup is configured before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("unsupported")
        .expect("resume opens the live target");
    let events = runner.host().events().to_vec();
    let commands = runner.host().commands().to_vec();
    let phase = runner.host().accessibility_phase_for_test(id);
    let callbacks = runner.handler().callbacks;

    runner.accessibility(AccessibilityEvent::InitialTreeRequested(id));

    let error = runner
        .result()
        .expect("unsupported accessibility is terminal")
        .expect_err("unsupported accessibility is rejected");
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(runner.host().events(), events, "no host fact is recorded");
    assert_eq!(runner.host().commands(), commands, "no update is applied");
    assert_eq!(
        runner.host().accessibility_phase_for_test(id),
        phase,
        "accessibility phase does not change"
    );
    assert_eq!(
        runner.handler().callbacks,
        callbacks,
        "no accessibility callback is delivered"
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_supported_open_hides_before_adapter_and_restores_visibility() {
    let request = WindowRequest::default();
    let native_request = native_open_request(&request, true);
    let operations = RefCell::new(Vec::new());

    let adapter = prepare_accessible_open(
        Id::from_u64(1),
        true,
        Some(()),
        request.visible(),
        |_| {
            operations.borrow_mut().push("adapter");
            "adapter"
        },
        || operations.borrow_mut().push("visible"),
    )
    .expect("a supported open prepares its adapter");

    assert!(!native_request.visible());
    assert_eq!(adapter, Some("adapter"));
    assert_eq!(operations.into_inner(), ["adapter", "visible"]);
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_hidden_open_never_shows() {
    let request = WindowRequest::builder("hidden").visible(false).build();
    let native_request = native_open_request(&request, true);
    let operations = RefCell::new(Vec::new());

    let adapter = prepare_accessible_open(
        Id::from_u64(1),
        true,
        Some(()),
        request.visible(),
        |_| {
            operations.borrow_mut().push("adapter");
        },
        || operations.borrow_mut().push("visible"),
    )
    .expect("a hidden supported open prepares its adapter");

    assert!(!native_request.visible());
    assert_eq!(adapter, Some(()));
    assert_eq!(operations.into_inner(), ["adapter"]);
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_unsupported_open_skips_adapter_without_visibility_rewrite() {
    let request = WindowRequest::default();
    let native_request = native_open_request(&request, false);

    let adapter = prepare_accessible_open(
        Id::from_u64(1),
        false,
        None::<()>,
        request.visible(),
        |_| panic!("unsupported opens must not create an adapter"),
        || panic!("unsupported opens must not restore visibility"),
    )
    .expect("unsupported opens need no accessibility setup");

    assert!(native_request.visible());
    assert_eq!(adapter, None);
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_missing_event_proxy_reports_nested_batch_error_without_open_effects() {
    struct Recorder(Rc<RefCell<Vec<&'static str>>>);

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            match event.event() {
                EventKind::Created(_) => self.0.borrow_mut().push("created"),
                EventKind::Focused { .. } => self.0.borrow_mut().push("later"),
                _ => {}
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            self.0.borrow_mut().push("ready");
            Ok(())
        }
    }

    let callbacks = Rc::new(RefCell::new(Vec::new()));
    let mut runner = WinitRunner::from_loop(Loop::new(Recorder(callbacks.clone())));
    runner.resolve_capabilities_for_test(fully_supported_capabilities());
    let local_window_drops = runner.simulate_missing_accessibility_proxy_for_test(Id::from_u64(2));
    runner.commands = vec![
        open("prefix").into(),
        open("failing").into(),
        Command::RequestDraw {
            id: Id::from_u64(1),
        },
    ];
    runner
        .enqueue_open_commands_for_test()
        .expect("the native-equivalent batch normalizes before pumping");
    runner.deliver_event_to_pump_for_test(EventKind::Focused {
        id: Id::from_u64(1),
        focused: true,
    });
    runner.enqueue_action_to_pump_for_test(Action::DrawNext(Id::from_u64(1)));
    runner.drain_pump_for_test();

    let error = runner
        .terminal_result_for_test()
        .expect("the native runner retains its first terminal result")
        .expect_err("a supported native open without an event proxy is terminal");
    assert_eq!(error.code, ErrorCode::CommandBatchFailed);
    assert_eq!(error.command_kind, Some(CommandKind::Open));
    assert_eq!(error.id, Some(Id::from_u64(2)));
    assert_eq!(error.completed_prefix, 1);
    let inner = error
        .source
        .as_deref()
        .and_then(|source| source.downcast_ref::<Error>())
        .expect("batch failures retain the crate accessibility error as their source");
    assert_eq!(inner.code, ErrorCode::AccessibilityAdapterFailed);
    assert_eq!(inner.id, Some(Id::from_u64(2)));
    assert!(matches!(
        inner
            .source
            .as_deref()
            .and_then(|source| source.downcast_ref::<AccessibilitySetupError>()),
        Some(AccessibilitySetupError::EventProxyUnavailable)
    ));
    assert_eq!(
        runner.applied_commands_for_test(),
        &[open("prefix").into()],
        "only the prefix command commits before the terminal failure"
    );
    assert_eq!(
        runner.lifecycle_state_for_test(Id::from_u64(2)),
        Some(crate::registry::LifecycleState::Closed)
    );
    assert_eq!(
        runner.accessibility_open_state_for_test(Id::from_u64(2)),
        (false, false, false, None),
        "the failed open leaves no live instance, route, adapter, or phase"
    );
    assert_eq!(local_window_drops.get(), 1, "the local native window drops");
    assert!(
        runner.take_ready_draws_for_test(Instant::now()).is_empty(),
        "the queued callback action is suppressed"
    );
    assert!(
        callbacks.borrow().is_empty(),
        "Created, Ready, and later callback work are suppressed"
    );
    assert!(
        runner.into_terminal_result().is_err(),
        "the retained terminal error is returned from the runner"
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn fake_accessibility_initial_tree_reaches_active_phase() {
    let (mut host, id) = fake_accessibility_host();
    let update = initial_accessibility_tree();

    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Absent)
    );
    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("initial tree request reaches the live fake window");
    host.apply(Command::UpdateAccessibility {
        id,
        update: update.clone(),
    })
    .expect("a full initial tree activates accessibility");

    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Active)
    );
    assert_eq!(host.accessibility_updates_for_test(), &[(id, update)]);
}

#[cfg(feature = "accessibility")]
#[test]
fn fake_runtime_dependent_accessibility_initial_tree_reaches_active_phase() {
    let mut host = testing::Host::with_capabilities(
        fully_supported_capabilities_with_accessibility(CapabilitySupport::RuntimeDependent),
    );
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("runtime-dependent fake window opens");
    let id = host.events()[0].id();
    host.clear();
    let update = initial_accessibility_tree();

    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Absent)
    );
    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("runtime-dependent accessibility lifecycle enters backend application");
    host.apply(Command::UpdateAccessibility {
        id,
        update: update.clone(),
    })
    .expect("a full runtime-dependent initial tree activates accessibility");

    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Active)
    );
    assert_eq!(host.accessibility_updates_for_test(), &[(id, update)]);
}

#[cfg(feature = "accessibility")]
#[test]
fn fake_accessibility_active_phase_accepts_incremental_update() {
    let (mut host, id) = fake_accessibility_host();
    let initial = initial_accessibility_tree();
    let incremental = incremental_accessibility_tree();

    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("initial tree request reaches the live fake window");
    host.apply(Command::UpdateAccessibility {
        id,
        update: initial.clone(),
    })
    .expect("a full initial tree activates accessibility");
    host.apply(Command::UpdateAccessibility {
        id,
        update: incremental.clone(),
    })
    .expect("an active tree accepts an incremental update");

    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Active)
    );
    assert_eq!(
        host.accessibility_updates_for_test(),
        &[(id, initial), (id, incremental)]
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn fake_accessibility_incremental_update_requires_initialized_tree() {
    let (mut host, id) = fake_accessibility_host();

    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("initial tree request reaches the live fake window");
    let error = host
        .apply(Command::UpdateAccessibility {
            id,
            update: incremental_accessibility_tree(),
        })
        .expect_err("an incremental update cannot initialize a requested tree");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::WaitingForInitialTree)
    );
    assert!(host.accessibility_updates_for_test().is_empty());
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_deactivation_returns_to_absent() {
    let (mut host, id) = fake_accessibility_host();

    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("first request reaches the fake window");
    host.apply(Command::UpdateAccessibility {
        id,
        update: initial_accessibility_tree(),
    })
    .expect("first full tree activates accessibility");
    host.accessibility(AccessibilityEvent::Deactivated(id))
        .expect("deactivation reaches the fake window");

    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::Absent)
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_reactivation_requires_full_tree() {
    let (mut host, id) = fake_accessibility_host();

    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("first request reaches the fake window");
    host.apply(Command::UpdateAccessibility {
        id,
        update: initial_accessibility_tree(),
    })
    .expect("first full tree activates accessibility");
    host.accessibility(AccessibilityEvent::Deactivated(id))
        .expect("deactivation reaches the fake window");
    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("repeated request returns the window to the waiting phase");

    let error = host
        .apply(Command::UpdateAccessibility {
            id,
            update: incremental_accessibility_tree(),
        })
        .expect_err("reactivation requires another full tree");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(
        host.accessibility_phase_for_test(id),
        Some(crate::planning::AccessibilityPhase::WaitingForInitialTree)
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_batch_preflight_rejects_invalid_phase_without_prefix() {
    let (mut host, id) = fake_accessibility_host();

    let error = host
        .apply_batch_for_test(vec![
            Command::SetTitle {
                id,
                title: String::from("must not apply"),
            },
            Command::UpdateAccessibility {
                id,
                update: incremental_accessibility_tree(),
            },
        ])
        .expect_err("an invalid accessibility tail rejects the whole authored batch");

    assert_eq!(error.code, ErrorCode::InvalidRequest);
    assert_eq!(
        host.registry()
            .get(id)
            .expect("window stays live")
            .instance
            .state()
            .title(),
        "Surgeist"
    );
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.accessibility_updates_for_test().is_empty());
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_unsupported_lifecycle_events_reject_without_effects() {
    let mut host = testing::Host::with_capabilities(conservative_capabilities());
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("ordinary fake window opens without accessibility support");
    let id = host.events()[0].id();
    host.clear();

    for event in [
        AccessibilityEvent::InitialTreeRequested(id),
        AccessibilityEvent::Deactivated(id),
    ] {
        let error = host
            .accessibility(event)
            .expect_err("unsupported lifecycle events must reject before fake effects");
        assert_eq!(error.code, ErrorCode::UnsupportedFeature);
        assert_eq!(host.accessibility_phase_for_test(id), None);
        assert!(host.events().is_empty());
    }
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_unsupported_target_rejects_update_without_effects() {
    let mut host = testing::Host::with_capabilities(conservative_capabilities());
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("ordinary fake window opens without accessibility support");
    let id = host.events()[0].id();
    host.clear();

    let error = host
        .apply(Command::UpdateAccessibility {
            id,
            update: initial_accessibility_tree(),
        })
        .expect_err("unsupported accessibility rejects before fake effects");

    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert!(host.accessibility_updates_for_test().is_empty());
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_stale_target_retains_lifecycle_error() {
    let mut host = testing::Host::with_capabilities(conservative_capabilities());
    host.apply(Command::Open {
        request: WindowRequest::default(),
    })
    .expect("ordinary fake window opens without accessibility support");
    let id = host.events()[0].id();
    host.apply(Command::Destroy { id })
        .expect("fixture window closes before the stale update");
    host.clear();

    let error = host
        .apply(Command::UpdateAccessibility {
            id,
            update: initial_accessibility_tree(),
        })
        .expect_err("a stale target keeps its lifecycle diagnostic");

    assert_eq!(error.code, ErrorCode::StaleWindow);
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_close_removes_phase_once() {
    let (mut host, id) = fake_accessibility_host();

    host.accessibility(AccessibilityEvent::InitialTreeRequested(id))
        .expect("initial tree request reaches the live fake window");
    host.apply(Command::Destroy { id })
        .expect("the live fake window closes");
    let error = host
        .apply(Command::Destroy { id })
        .expect_err("a strict repeated close retains its stale lifecycle diagnostic");

    assert_eq!(error.code, ErrorCode::StaleWindow);
    assert_eq!(host.accessibility_phase_for_test(id), None);
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_feature_maps_accesskit_window_events() {
    let id = Id::from_u64(12);

    assert_eq!(
        accessibility_event_from_winit(id, accesskit_winit::WindowEvent::InitialTreeRequested),
        AccessibilityEvent::InitialTreeRequested(id)
    );
    assert_eq!(
        accessibility_event_from_winit(id, accesskit_winit::WindowEvent::AccessibilityDeactivated),
        AccessibilityEvent::Deactivated(id)
    );
}

#[cfg(feature = "accessibility")]
#[test]
fn accessibility_event_exposes_target_id() {
    let id = Id::from_u64(8);

    assert_eq!(AccessibilityEvent::InitialTreeRequested(id).id(), id);
    assert_eq!(
        AccessibilityEvent::ActionRequested(AccessibilityActionRequest {
            id,
            request: accesskit::ActionRequest {
                action: accesskit::Action::Focus,
                target_tree: accesskit::TreeId::ROOT,
                target_node: accesskit::NodeId(42),
                data: None,
            },
        })
        .id(),
        id
    );
    assert_eq!(AccessibilityEvent::Deactivated(id).id(), id);
}

#[test]
fn dsl_open_builder_lowers_to_request_and_command() {
    let parent = Id::from_u64(11);
    let open = open("inspector")
        .title("Inspector")
        .at(point(12, 24))
        .size(size(420, 640))
        .min(size(320, 240))
        .max(size(1200, 900))
        .fixed()
        .controls(controls().minimize(false).maximize(false))
        .decorations(false)
        .transparent(true)
        .hidden()
        .borderless()
        .level(Level::AlwaysOnTop)
        .theme(Some(Theme::Dark))
        .tool(Some(parent));

    let request = open.request().clone();

    assert_eq!(request.name(), Some("inspector"));
    assert_eq!(request.title(), "Inspector");
    assert_eq!(request.position(), Some(Point { x: 12.0, y: 24.0 }));
    assert_eq!(
        request.inner_size(),
        Some(Size {
            width: 420.0,
            height: 640.0,
        })
    );
    assert_eq!(
        request.min_inner_size(),
        Some(Size {
            width: 320.0,
            height: 240.0,
        })
    );
    assert_eq!(
        request.max_inner_size(),
        Some(Size {
            width: 1200.0,
            height: 900.0,
        })
    );
    assert!(!request.resizable());
    assert_eq!(
        request.controls(),
        Controls {
            close: true,
            minimize: false,
            maximize: false,
        }
    );
    assert!(!request.decorations());
    assert!(request.transparent());
    assert!(!request.visible());
    assert_eq!(request.fullscreen(), Fullscreen::Borderless);
    assert_eq!(request.level(), Level::AlwaysOnTop);
    assert_eq!(request.theme(), Some(Theme::Dark));
    assert_eq!(
        request.role(),
        &Role::Tool {
            parent: Some(parent),
        }
    );

    assert_eq!(Command::from(open), Command::Open { request });
}

#[test]
fn modal_authors_dialog_without_call_order_dependency() {
    let parent = Id::from_u64(4);

    assert_eq!(
        open("dialog").modal(parent, Modality::App).request().role(),
        &Role::Dialog {
            parent,
            modality: Modality::App,
        }
    );
}

#[test]
fn dsl_open_builder_lowers_role_theme_and_control_variants() {
    assert_eq!(Open::unnamed().request().name(), None);
    assert_eq!(
        open("first").name("second").request().name(),
        Some("second")
    );
    assert_eq!(open("root").root().request().role(), &Role::Root);
    assert_eq!(
        open("dialog")
            .modal(Id::from_u64(4), Modality::App)
            .request()
            .role(),
        &Role::Dialog {
            parent: Id::from_u64(4),
            modality: Modality::App,
        }
    );
    assert_eq!(
        open("popup").popup(Id::from_u64(8)).request().role(),
        &Role::Popup {
            parent: Id::from_u64(8),
        }
    );
    assert_eq!(open("theme").theme(None).request().theme(), None);
    assert_eq!(
        Controls::from(controls().all(false).close(true)),
        Controls {
            close: true,
            minimize: false,
            maximize: false,
        }
    );
}

#[test]
fn dsl_context_resolves_named_window_state_then_targets_commands_and_actions() {
    let mut window_loop = Loop::new(NoopHandler);
    let capabilities = conservative_capabilities();
    let id = window_loop
        .registry
        .reserve_id()
        .expect("test identity reserves");
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 480,
            height: 240,
        },
        2.0,
    )
    .expect("test metrics are valid");
    window_loop
        .registry
        .insert(Instance::new(
            WindowSnapshot::new("Main", metrics)
                .named("main")
                .with_visible(true),
        ))
        .expect("test instance inserts");

    {
        let mut cx = window_loop.context(&capabilities);
        let target = cx.window_id("main").expect("named window should resolve");

        assert_eq!(cx.state("main").map(WindowSnapshot::id), Some(id));

        cx.window(target)
            .title("Ready Main")
            .size(size(320, 160))
            .draw()
            .close();
        cx.again(target);
    }

    assert_eq!(
        window_loop.commands,
        vec![
            Command::SetTitle {
                id,
                title: String::from("Ready Main"),
            },
            Command::SetInnerSize {
                id,
                size: Size {
                    width: 320.0,
                    height: 160.0,
                },
            }
        ]
    );
    assert_eq!(
        window_loop.context(&capabilities).action(),
        &Action::Batch(vec![
            Action::DrawNext(id),
            Action::CloseRequested(id),
            Action::DrawNow(id)
        ])
    );
}

#[test]
fn dsl_target_helpers_lower_to_exact_commands() {
    let mut window_loop = Loop::new(NoopHandler);
    let capabilities = conservative_capabilities();
    let id = window_loop
        .registry
        .reserve_id()
        .expect("test identity reserves");
    window_loop
        .registry
        .insert(Instance::new(state(id)))
        .expect("test instance inserts");
    let ime = ImeRequest::Enable(ImeConfig {
        purpose: ImePurpose::Email,
        hint: ImeHint::Spellcheck,
        cursor_area: Some(rect(1, 2, 3, 4)),
        surrounding_text: None,
    });

    {
        let mut cx = window_loop.context(&capabilities);
        cx.window(id)
            .at(point(10, 20))
            .hide()
            .show()
            .resizable(false)
            .controls(controls().all(false))
            .decorations(false)
            .transparent(true)
            .min(Some(size(100, 80)))
            .max(Option::<Size>::None)
            .fullscreen(Fullscreen::Borderless)
            .level(Level::AlwaysOnBottom)
            .theme(None)
            .cursor(Cursor::Hidden)
            .cursor_grab(CursorGrab::Locked)
            .ime(ime.clone())
            .attention();
    }

    assert_eq!(
        window_loop.commands,
        vec![
            Command::SetPosition {
                id,
                position: point(10, 20),
            },
            Command::SetVisible { id, visible: false },
            Command::SetVisible { id, visible: true },
            Command::SetResizable {
                id,
                resizable: false,
            },
            Command::SetControls {
                id,
                controls: Controls {
                    close: false,
                    minimize: false,
                    maximize: false,
                },
            },
            Command::SetDecorations {
                id,
                decorations: false,
            },
            Command::SetTransparent {
                id,
                transparent: true,
            },
            Command::SetMinInnerSize {
                id,
                size: Some(size(100, 80)),
            },
            Command::SetMaxInnerSize { id, size: None },
            Command::SetFullscreen {
                id,
                fullscreen: Fullscreen::Borderless,
            },
            Command::SetLevel {
                id,
                level: Level::AlwaysOnBottom,
            },
            Command::SetTheme { id, theme: None },
            Command::SetCursor {
                id,
                cursor: Cursor::Hidden,
            },
            Command::SetCursorGrab {
                id,
                grab: CursorGrab::Locked,
            },
            Command::SetIme { id, request: ime },
            Command::RequestUserAttention { id },
        ]
    );
}

#[test]
fn dsl_fake_host_accepts_builders_and_dispatches_scoped_events() {
    #[derive(Default)]
    struct Recorder {
        created: Option<Id>,
        resized: Option<Metrics>,
        draws: Vec<Id>,
    }

    impl Handler for Recorder {
        fn ready(&mut self, win: &mut Ready<'_>) -> Result<()> {
            self.created = Some(win.id());
            win.draw();
            Ok(())
        }

        fn resize(&mut self, win: &mut Resize<'_>) -> Result<()> {
            self.resized = Some(win.metrics().clone());
            win.target().title("Resized");
            win.again();
            Ok(())
        }

        fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
            self.draws.push(frame.id());
            frame.exit();
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main").title("Main").size(size(320, 200)))
        .unwrap();
    let id = host.window_id("main").expect("named window");
    let mut handler = Recorder::default();

    let action = host.dispatch_ready(&mut handler, id).unwrap();
    assert_eq!(handler.created, Some(id));
    assert_eq!(action, Effect::Draw(id));

    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 400,
        },
        2.0,
    )
    .expect("test metrics are valid");
    let action = host.dispatch_resize(&mut handler, metrics.clone()).unwrap();
    assert_eq!(handler.resized, Some(metrics));
    assert_eq!(
        action,
        Effect::Batch(vec![Effect::Draw(id), Effect::Again(id)])
    );
    assert!(host.commands().iter().any(|command| matches!(
        command,
        Command::SetTitle { id: command_id, title }
            if *command_id == id && title == "Resized"
    )));

    let action = host.dispatch_draw(&mut handler, id).unwrap();
    assert_eq!(handler.draws, vec![id]);
    assert_eq!(action, Effect::Exit);
}

#[test]
fn dsl_app_queues_startup_open_commands() {
    let window_loop = app(NoopHandler)
        .open(open("main").title("Main"))
        .open(open("tools").title("Tools"))
        .into_loop();

    assert!(window_loop.commands.is_empty());
    assert_eq!(window_loop.startup.len(), 2);
    assert!(matches!(
        &window_loop.startup[0],
        Command::Open { request } if request.name() == Some("main")
    ));
    assert!(matches!(
        &window_loop.startup[1],
        Command::Open { request } if request.name() == Some("tools")
    ));
}

#[test]
fn dsl_startup_commands_remain_normalized_until_first_resume() {
    let window_loop = app(NoopHandler)
        .open(open("main").title("Main"))
        .open(open("tools").title("Tools"))
        .into_loop();
    let runner = WinitRunner::from_loop(window_loop);

    assert_eq!(runner.startup.len(), 2);
    assert!(matches!(
        runner.startup[0].command(),
        Command::Open { request } if request.name() == Some("main")
    ));
    assert!(matches!(
        runner.startup[1].command(),
        Command::Open { request } if request.name() == Some("tools")
    ));
}

#[test]
fn dsl_fake_host_rejects_duplicate_runtime_window_names() {
    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();

    let error = host
        .apply(open("main"))
        .expect_err("duplicate runtime names should fail");

    assert_eq!(error.code, ErrorCode::DuplicateIdentity);
}

#[test]
fn dsl_lifecycle_dispatch_rolls_back_callback_commands_on_error() {
    struct FailingHandler;

    impl Handler for FailingHandler {
        fn resize(&mut self, win: &mut Resize<'_>) -> Result<()> {
            win.target().title("Should not apply");
            Err(Error::new(ErrorCode::CommandFailed, "intentional failure"))
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    host.clear();

    let error = host
        .dispatch_resize(
            &mut FailingHandler,
            Metrics::from_physical_size(
                id,
                PhysicalSize {
                    width: 320,
                    height: 200,
                },
                1.0,
            )
            .expect("test metrics are valid"),
        )
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::CommandFailed);
    assert!(host.commands().is_empty());
}

#[test]
fn fake_host_callback_failure_rolls_back_callback_commands_and_command_induced_state() {
    struct FailingReady;

    impl Handler for FailingReady {
        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            ready.target().title("Should not apply");
            Err(Error::new(ErrorCode::CommandFailed, "intentional failure"))
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    let original_title = host
        .registry()
        .get(id)
        .unwrap()
        .instance
        .state()
        .title()
        .to_owned();
    host.clear();

    let error = host.dispatch_ready(&mut FailingReady, id).unwrap_err();

    assert_eq!(error.code, ErrorCode::CommandFailed);
    assert!(host.commands().is_empty());
    assert!(host.events().is_empty());
    assert_eq!(
        host.registry().get(id).unwrap().instance.state().title(),
        original_title
    );
}

#[test]
fn dsl_lifecycle_dispatch_drains_commands_before_returning_action() {
    struct OpenAndExit;

    impl Handler for OpenAndExit {
        fn input(&mut self, input: &mut Input<'_>) -> Result<()> {
            input.context_mut().open(open("child"));
            input.exit();
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    host.clear();

    let action = host
        .dispatch_input(
            &mut OpenAndExit,
            InputEvent::Modifiers {
                id,
                modifiers: ModifierState::default(),
            },
        )
        .unwrap();

    assert_eq!(action, Effect::Exit);
    assert!(matches!(
        host.events().first(),
        Some(HostEvent::Created(state)) if state.name() == Some("child")
    ));
}

#[test]
fn lifecycle_ready_and_resize_preserve_automatic_draw_with_delayed_draws() {
    struct DelayedDraw {
        time: Instant,
    }

    impl Handler for DelayedDraw {
        fn ready(&mut self, win: &mut Ready<'_>) -> Result<()> {
            win.at(self.time);
            Ok(())
        }

        fn resize(&mut self, win: &mut Resize<'_>) -> Result<()> {
            win.at(self.time);
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main")).unwrap();
    let id = host.window_id("main").unwrap();
    let time = Instant::now() + std::time::Duration::from_millis(25);
    let mut handler = DelayedDraw { time };

    assert_eq!(
        host.dispatch_ready(&mut handler, id).unwrap(),
        Effect::Batch(vec![Effect::Draw(id), Effect::At { id, time }])
    );

    assert_eq!(
        host.dispatch_resize(
            &mut handler,
            Metrics::from_physical_size(
                id,
                PhysicalSize {
                    width: 320,
                    height: 200,
                },
                1.0,
            )
            .expect("test metrics are valid"),
        )
        .unwrap(),
        Effect::Batch(vec![Effect::Draw(id), Effect::At { id, time }])
    );
}

#[test]
fn dsl_frame_exposes_metrics_and_fake_handle_error() {
    struct RendererProbe {
        metrics: Option<Metrics>,
        handle_error: Option<ErrorCode>,
    }

    impl Handler for RendererProbe {
        fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
            self.metrics = Some(frame.metrics().clone());
            self.handle_error = Some(frame.handle().unwrap_err().code);
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("main").size(size(320, 180))).unwrap();
    let id = host.window_id("main").unwrap();
    let mut handler = RendererProbe {
        metrics: None,
        handle_error: None,
    };

    let action = host.dispatch_draw(&mut handler, id).unwrap();

    assert_eq!(action, Effect::Wait);
    assert_eq!(
        handler.metrics.unwrap().logical_size(),
        Size {
            width: 320.0,
            height: 180.0,
        }
    );
    assert_eq!(handler.handle_error, Some(ErrorCode::HandleUnavailable));
}

#[test]
fn dsl_idle_is_opt_in() {
    struct IdleHandler;

    impl Handler for IdleHandler {
        fn wants_idle(&self) -> bool {
            true
        }

        fn idle(&mut self, cx: &mut Context<'_>) -> Result<()> {
            cx.exit();
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    let mut noop = NoopHandler;
    let mut handler = IdleHandler;

    assert_eq!(host.idle(&mut noop).unwrap(), None);
    assert_eq!(host.idle(&mut handler).unwrap(), Some(Effect::Exit));
}

#[test]
fn lifecycle_startup_open_delivers_ready_and_draw() {
    #[derive(Default)]
    struct Studio {
        ready: Vec<Id>,
        draws: Vec<Id>,
        ready_size: Option<Size>,
    }

    impl Handler for Studio {
        fn ready(&mut self, win: &mut Ready<'_>) -> Result<()> {
            self.ready.push(win.id());
            self.ready_size = Some(win.metrics().logical_size());
            assert_eq!(win.state().name(), Some("main"));
            win.draw();
            Ok(())
        }

        fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
            self.draws.push(frame.id());
            assert_eq!(
                frame.size(),
                Size {
                    width: 640.0,
                    height: 360.0,
                }
            );
            Ok(())
        }
    }

    let app =
        app(Studio::default()).open(open("main").title("Surgeist Studio").size(size(640, 360)));
    let mut window_loop = app.into_loop();
    let mut host = testing::Host::new();

    assert_eq!(window_loop.startup.len(), 1);
    host.apply(window_loop.startup.remove(0)).unwrap();
    let id = host.window_id("main").unwrap();

    let ready = host.dispatch_ready(window_loop.handler_mut(), id).unwrap();
    assert_eq!(ready, Effect::Draw(id));

    let draw = host.dispatch_draw(window_loop.handler_mut(), id).unwrap();
    assert_eq!(draw, Effect::Wait);
    assert_eq!(window_loop.handler().ready, vec![id]);
    assert_eq!(window_loop.handler().draws, vec![id]);
    assert_eq!(
        window_loop.handler().ready_size,
        Some(Size {
            width: 640.0,
            height: 360.0,
        })
    );
}

#[test]
fn lifecycle_scopes_share_common_window_surface() {
    fn assert_scope<'a, T: Scope<'a>>() {}
    assert_scope::<Ready<'static>>();
    assert_scope::<Resize<'static>>();
    assert_scope::<Input<'static>>();
    assert_scope::<Close<'static>>();
    assert_scope::<Frame<'static>>();

    struct Probe {
        observed: Option<(Id, Size, f64, bool, bool, bool)>,
    }

    impl Handler for Probe {
        fn ready(&mut self, win: &mut Ready<'_>) -> Result<()> {
            self.observed = Some((
                win.id(),
                win.size(),
                win.scale(),
                win.is_focused(),
                win.is_visible(),
                win.is_occluded(),
            ));
            win.target().title("Scoped").draw();
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("scope").title("Scope")).unwrap();
    let id = match host.events().last().unwrap() {
        HostEvent::Created(state) => state.id(),
        event => panic!("expected created event, got {event:?}"),
    };
    let mut probe = Probe { observed: None };

    assert_eq!(
        host.dispatch_ready(&mut probe, id).unwrap(),
        Effect::Draw(id)
    );
    assert_eq!(
        probe.observed,
        Some((
            id,
            Size {
                width: 800.0,
                height: 600.0,
            },
            1.0,
            false,
            true,
            false,
        ))
    );
    assert_eq!(
        host.events()
            .iter()
            .find_map(|event| match event {
                HostEvent::Created(state) if state.id() == id => Some(state.name()),
                _ => None,
            })
            .flatten(),
        Some("scope")
    );
    assert_eq!(
        host.commands().last(),
        Some(&Command::SetTitle {
            id,
            title: String::from("Scoped")
        })
    );
}

#[test]
fn lifecycle_resize_input_close_and_closed_are_scoped() {
    #[derive(Default)]
    struct Studio {
        resized: Vec<Size>,
        inputs: usize,
        close_requested: Vec<Id>,
        closed: Vec<Id>,
    }

    impl Handler for Studio {
        fn resize(&mut self, win: &mut Resize<'_>) -> Result<()> {
            self.resized.push(win.size());
            Ok(())
        }

        fn input(&mut self, input: &mut Input<'_>) -> Result<()> {
            self.inputs += 1;
            input.close().draw();
            Ok(())
        }

        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.close_requested.push(close.id());
            close.close();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            self.closed.push(closed.id());
            assert_eq!(closed.state().name(), Some("main"));
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Studio::default());
    runner
        .startup(vec![open("main").size(size(320, 180)).into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    let resized_metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 400,
        },
        2.0,
    )
    .expect("test metrics are valid");

    runner.transition(NativeEventTransition::resized(resized_metrics));
    runner.transition(NativeEventTransition::mouse_moved(
        id,
        Point { x: 12.0, y: 24.0 },
        PhysicalPoint { x: 12, y: 24 },
        None,
        ModifierState::default(),
    ));

    assert_eq!(
        runner.handler().resized,
        vec![Size {
            width: 400.0,
            height: 200.0,
        }]
    );
    assert_eq!(runner.handler().inputs, 1);
    assert_eq!(runner.handler().close_requested, vec![id]);
    assert_eq!(runner.handler().closed, vec![id]);
    assert!(!runner.host().registry().contains(id));
}

#[test]
fn lifecycle_close_cancel_keeps_window_live() {
    #[derive(Default)]
    struct CancelClose {
        close_requested: Vec<Id>,
        closed: Vec<Id>,
    }

    impl Handler for CancelClose {
        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.close_requested.push(close.id());
            close.cancel();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            self.closed.push(closed.id());
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(CancelClose::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    runner.request_close(id);

    assert!(runner.host().registry().contains(id));
    assert_eq!(runner.handler().close_requested, vec![id]);
    assert!(runner.handler().closed.is_empty());
}

#[test]
fn test_runner_pre_resume_dispatch_is_terminal_and_has_no_effects() {
    #[derive(Default)]
    struct Recorder {
        created: usize,
        ready: usize,
        resumed: usize,
        suspended: usize,
        idle: usize,
        resized: usize,
        drawn: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                self.created += 1;
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            self.ready += 1;
            Ok(())
        }

        fn resume(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.resumed += 1;
            Ok(())
        }

        fn suspend(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.suspended += 1;
            Ok(())
        }

        fn wants_idle(&self) -> bool {
            true
        }

        fn idle(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.idle += 1;
            Ok(())
        }

        fn resize(&mut self, _resize: &mut Resize<'_>) -> Result<()> {
            self.resized += 1;
            Ok(())
        }

        fn draw(&mut self, _frame: &mut Frame<'_>) -> Result<()> {
            self.drawn += 1;
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner.dispatch(Command::Open {
        request: WindowRequest::builder("").build(),
    });

    let first_error = match runner.result() {
        Some(Err(error)) => (
            error.code,
            error.message.clone(),
            error.id,
            error.command_kind,
            error.completed_prefix,
        ),
        result => panic!("pre-resume dispatch must be terminal, got {result:?}"),
    };
    assert_eq!(
        first_error,
        (
            ErrorCode::InvalidRequest,
            String::from("dispatch requires first resume"),
            None,
            None,
            0,
        ),
        "the first-resume phase guard must run before malformed authored command normalization"
    );

    let ready_draws = runner.take_ready_draws_for_test(Instant::now());
    let host_snapshot = (
        runner.host().commands().to_vec(),
        runner.host().events().to_vec(),
        runner.host().registry().live_ids(),
        runner.host().cursor_updates().to_vec(),
        runner.host().ime_requests().to_vec(),
        runner.host().capabilities().clone(),
        ready_draws,
    );
    let callback_snapshot = (
        runner.handler().created,
        runner.handler().ready,
        runner.handler().resumed,
        runner.handler().suspended,
        runner.handler().idle,
        runner.handler().resized,
        runner.handler().drawn,
    );
    assert!(
        host_snapshot.0.is_empty()
            && host_snapshot.1.is_empty()
            && host_snapshot.2.is_empty()
            && host_snapshot.3.is_empty()
            && host_snapshot.4.is_empty()
            && host_snapshot.6.is_empty()
            && callback_snapshot == (0, 0, 0, 0, 0, 0, 0),
        "pre-resume dispatch recorded commands {:?}, events {:?}, windows {:?}, cursor updates {:?}, IME requests {:?}, ready draws {:?}, and callbacks {callback_snapshot:?}",
        host_snapshot.0,
        host_snapshot.1,
        host_snapshot.2,
        host_snapshot.3,
        host_snapshot.4,
        host_snapshot.6,
    );

    assert!(
        runner
            .startup(vec![Command::Open {
                request: WindowRequest::builder("").build(),
            }])
            .is_ok()
    );
    runner.dispatch(open("ignored-after-terminal").into());
    runner.resume();
    runner.suspend();
    runner.idle();
    runner.transition(NativeEventTransition::resized(
        Metrics::from_physical_size(
            Id::from_u64(1),
            PhysicalSize {
                width: 640,
                height: 480,
            },
            1.0,
        )
        .expect("test transition metrics are valid"),
    ));
    runner.draw(Id::from_u64(1));
    runner.request_close(Id::from_u64(1));
    runner.destroy(Id::from_u64(1));

    let ready_draws = runner.take_ready_draws_for_test(Instant::now());
    let after_host_snapshot = (
        runner.host().commands().to_vec(),
        runner.host().events().to_vec(),
        runner.host().registry().live_ids(),
        runner.host().cursor_updates().to_vec(),
        runner.host().ime_requests().to_vec(),
        runner.host().capabilities().clone(),
        ready_draws,
    );
    let after_callback_snapshot = (
        runner.handler().created,
        runner.handler().ready,
        runner.handler().resumed,
        runner.handler().suspended,
        runner.handler().idle,
        runner.handler().resized,
        runner.handler().drawn,
    );
    let after_error = match runner.result() {
        Some(Err(error)) => (
            error.code,
            error.message.clone(),
            error.id,
            error.command_kind,
            error.completed_prefix,
        ),
        result => panic!("terminal result changed after no-op calls: {result:?}"),
    };

    assert_eq!(after_host_snapshot, host_snapshot);
    assert_eq!(after_callback_snapshot, callback_snapshot);
    assert_eq!(after_error, first_error);
}

#[test]
fn test_runner_open_delivers_created_then_ready_like_native_pump() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<&'static str>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                self.callbacks.push("created");
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            self.callbacks.push("ready");
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before resume");
    assert!(runner.host().registry().is_empty());

    runner.resume();

    assert_eq!(runner.handler().callbacks, ["created", "ready"]);
    assert!(runner.host().registry().contains(Id::from_u64(1)));
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_created_close_skips_ready_like_native_pump() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<&'static str>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                self.callbacks.push("created");
                event.close();
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            self.callbacks.push("ready");
            Ok(())
        }

        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.callbacks.push("close");
            close.close();
            Ok(())
        }

        fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
            self.callbacks.push("closed");
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before resume");

    runner.resume();

    assert_eq!(runner.handler().callbacks, ["created", "close", "closed"]);
    assert!(!runner.host().registry().contains(Id::from_u64(1)));
    assert!(matches!(
        runner.host().events(),
        [HostEvent::Created(_), HostEvent::Destroyed(id)] if *id == Id::from_u64(1)
    ));
    assert!(matches!(runner.host().commands(), [Command::Open { .. }]));
    assert!(runner.result().is_none());
}

#[test]
fn close_and_destroy_use_specialized_callbacks_only() {
    #[derive(Default)]
    struct Recorder {
        generic_events: Vec<EventKind>,
        close_calls: usize,
        closed_calls: usize,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            self.generic_events.push(event.event().clone());
            Ok(())
        }

        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.close_calls += 1;
            close.close();
            Ok(())
        }

        fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
            self.closed_calls += 1;
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");

    runner.request_close(id);
    runner.request_close(id);

    assert!(matches!(
        runner.handler().generic_events.as_slice(),
        [EventKind::Created(_)]
    ));
    assert_eq!(runner.handler().close_calls, 1);
    assert_eq!(runner.handler().closed_calls, 1);
    assert!(!runner.host().registry().contains(id));
    assert_eq!(
        runner
            .host()
            .events()
            .iter()
            .filter(
                |event| matches!(event, HostEvent::CloseRequested(requested) if *requested == id)
            )
            .count(),
        1
    );
    assert_eq!(
        runner
            .host()
            .events()
            .iter()
            .filter(|event| matches!(event, HostEvent::Destroyed(destroyed) if *destroyed == id))
            .count(),
        1
    );
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_destroy_command_and_accepted_close_share_cleanup_and_closed_order() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<&'static str>,
    }

    impl Handler for Recorder {
        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.callbacks.push("close");
            close.close();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            self.callbacks.push("closed");
            let id = closed.id();
            assert!(closed.context_mut().state(id).is_none());
            Ok(())
        }
    }

    let mut authored_destroy = testing::Runner::new(Recorder::default());
    authored_destroy
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    authored_destroy.resume();
    let authored_id = authored_destroy
        .host()
        .window_id("main")
        .expect("startup window is live");
    authored_destroy.dispatch(Command::Destroy { id: authored_id });

    let mut accepted_close = testing::Runner::new(Recorder::default());
    accepted_close
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    accepted_close.resume();
    let accepted_id = accepted_close
        .host()
        .window_id("main")
        .expect("startup window is live");
    accepted_close.request_close(accepted_id);

    for (runner, id) in [
        (&authored_destroy, authored_id),
        (&accepted_close, accepted_id),
    ] {
        assert!(!runner.host().registry().contains(id));
        assert_eq!(
            runner
                .host()
                .events()
                .iter()
                .filter(|event| {
                    matches!(event, HostEvent::Destroyed(destroyed) if *destroyed == id)
                })
                .count(),
            1
        );
    }
    assert_eq!(authored_destroy.handler().callbacks, ["closed"]);
    assert_eq!(accepted_close.handler().callbacks, ["close", "closed"]);
    assert!(
        authored_destroy
            .host()
            .commands()
            .contains(&Command::Destroy { id: authored_id })
    );
    assert!(
        !accepted_close
            .host()
            .commands()
            .iter()
            .any(|command| matches!(command, Command::Destroy { .. }))
    );
    assert!(authored_destroy.result().is_none());
    assert!(accepted_close.result().is_none());
}

#[test]
fn test_runner_request_close_honors_final_decision() {
    #[derive(Clone, Copy)]
    enum Decision {
        Cancel,
        Accept,
    }

    struct Recorder {
        decision: Decision,
        close_calls: usize,
        closed_calls: usize,
    }

    impl Handler for Recorder {
        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            self.close_calls += 1;
            match self.decision {
                Decision::Cancel => {
                    close.close().cancel();
                }
                Decision::Accept => {
                    close.cancel().close();
                }
            }
            Ok(())
        }

        fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
            self.closed_calls += 1;
            Ok(())
        }
    }

    let mut cancelled = testing::Runner::new(Recorder {
        decision: Decision::Cancel,
        close_calls: 0,
        closed_calls: 0,
    });
    cancelled
        .startup(vec![open("cancelled").into()])
        .expect("startup is stored before first resume");
    cancelled.resume();
    let cancelled_id = cancelled
        .host()
        .window_id("cancelled")
        .expect("startup window is live");
    cancelled.request_close(cancelled_id);

    assert!(cancelled.host().registry().contains(cancelled_id));
    assert_eq!(cancelled.handler().close_calls, 1);
    assert_eq!(cancelled.handler().closed_calls, 0);
    assert!(matches!(
        cancelled.host().events(),
        [HostEvent::Created(_), HostEvent::CloseRequested(id)] if *id == cancelled_id
    ));

    let mut accepted = testing::Runner::new(Recorder {
        decision: Decision::Accept,
        close_calls: 0,
        closed_calls: 0,
    });
    accepted
        .startup(vec![open("accepted").into()])
        .expect("startup is stored before first resume");
    accepted.resume();
    let accepted_id = accepted
        .host()
        .window_id("accepted")
        .expect("startup window is live");
    accepted.request_close(accepted_id);
    accepted.request_close(accepted_id);

    assert!(!accepted.host().registry().contains(accepted_id));
    assert_eq!(accepted.handler().close_calls, 1);
    assert_eq!(accepted.handler().closed_calls, 1);
    assert!(matches!(
        accepted.host().events(),
        [HostEvent::Created(_), HostEvent::CloseRequested(id), HostEvent::Destroyed(destroyed)]
            if *id == accepted_id && *destroyed == accepted_id
    ));
    assert!(accepted.result().is_none());
}

#[test]
fn test_runner_close_and_closed_failures_discard_local_work() {
    struct CloseFailure;

    impl Handler for CloseFailure {
        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            let id = close.id();
            close.close();
            close.context_mut().send(Command::SetTitle {
                id,
                title: String::from("discarded"),
            });
            Err(Error::new(ErrorCode::CommandFailed, "close failed"))
        }
    }

    let mut close_failure = testing::Runner::new(CloseFailure);
    close_failure
        .startup(vec![open("close-failure").into()])
        .expect("startup is stored before first resume");
    close_failure.resume();
    let close_failure_id = close_failure
        .host()
        .window_id("close-failure")
        .expect("startup window is live");
    close_failure.request_close(close_failure_id);

    assert!(close_failure.host().registry().contains(close_failure_id));
    assert!(matches!(
        close_failure.result(),
        Some(Err(error)) if error.message == "close failed"
    ));
    assert!(matches!(
        close_failure.host().commands(),
        [Command::Open { request }] if request.name() == Some("close-failure")
    ));
    assert!(matches!(
        close_failure.host().events(),
        [HostEvent::Created(_), HostEvent::CloseRequested(id)] if *id == close_failure_id
    ));

    struct ClosedFailure;

    impl Handler for ClosedFailure {
        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            close.close();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            closed.context_mut().open(open("discarded"));
            Err(Error::new(ErrorCode::CommandFailed, "closed failed"))
        }
    }

    let mut closed_failure = testing::Runner::new(ClosedFailure);
    closed_failure
        .startup(vec![open("closed-failure").into()])
        .expect("startup is stored before first resume");
    closed_failure.resume();
    let closed_failure_id = closed_failure
        .host()
        .window_id("closed-failure")
        .expect("startup window is live");
    closed_failure.request_close(closed_failure_id);

    assert!(!closed_failure.host().registry().contains(closed_failure_id));
    assert_eq!(closed_failure.host().window_id("discarded"), None);
    assert!(matches!(
        closed_failure.result(),
        Some(Err(error)) if error.message == "closed failed"
    ));
    assert!(matches!(
        closed_failure.host().commands(),
        [Command::Open { request }] if request.name() == Some("closed-failure")
    ));
    assert!(matches!(
        closed_failure.host().events(),
        [HostEvent::Created(_), HostEvent::CloseRequested(id), HostEvent::Destroyed(destroyed)]
            if *id == closed_failure_id && *destroyed == closed_failure_id
    ));
}

#[test]
fn test_runner_stale_proxy_work_is_a_noop() {
    struct Noop;

    impl Handler for Noop {}

    let mut runner = testing::Runner::new(Noop);
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    let proxy = runner.proxy();
    runner.destroy(id);
    let commands = runner.host().commands().to_vec();
    let events = runner.host().events().to_vec();

    proxy
        .send(Command::SetTitle {
            id,
            title: String::from("too late"),
        })
        .expect("a proxy accepts normalized work before the loop processes it");
    proxy
        .draw(id)
        .expect("a proxy accepts stale draw work before processing");
    runner.flush_proxy();

    assert_eq!(runner.host().commands(), commands);
    assert_eq!(runner.host().events(), events);
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_closed_commands_run_after_coherent_close() {
    #[derive(Default)]
    struct Replacement {
        closed: usize,
        replacement: Option<Id>,
    }

    impl Handler for Replacement {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if let EventKind::Created(state) = event.event()
                && state.name() == Some("main")
                && state.id() != Id::from_u64(1)
            {
                self.replacement = Some(state.id());
            }
            Ok(())
        }

        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            close.close();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            self.closed += 1;
            let id = closed.id();
            assert!(closed.context_mut().state(id).is_none());
            closed.context_mut().open(open("main"));
            Ok(())
        }
    }

    let mut replacement = testing::Runner::new(Replacement::default());
    replacement
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    replacement.resume();
    let closed_id = replacement
        .host()
        .window_id("main")
        .expect("startup window is live");
    replacement.request_close(closed_id);

    assert_eq!(replacement.handler().closed, 1);
    assert_eq!(replacement.handler().replacement, Some(Id::from_u64(2)));
    assert_eq!(replacement.host().window_id("main"), Some(Id::from_u64(2)));
    assert!(replacement.result().is_none());

    struct StaleCommand;

    impl Handler for StaleCommand {
        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            close.close();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            let id = closed.id();
            assert!(closed.context_mut().state(id).is_none());
            closed.context_mut().send(Command::SetTitle {
                id,
                title: String::from("stale"),
            });
            Ok(())
        }
    }

    let mut stale = testing::Runner::new(StaleCommand);
    stale
        .startup(vec![open("stale").into()])
        .expect("startup is stored before first resume");
    stale.resume();
    let stale_id = stale
        .host()
        .window_id("stale")
        .expect("startup window is live");
    stale.request_close(stale_id);

    assert!(matches!(
        stale.result(),
        Some(Err(error))
            if error.code == ErrorCode::StaleWindow
                && error.id == Some(stale_id)
                && error.command_kind == Some(CommandKind::SetTitle)
    ));
}

#[test]
fn proxy_close_after_close_does_not_invoke_close_scope() {
    #[derive(Default)]
    struct Recorder {
        close_calls: usize,
    }

    impl Handler for Recorder {
        fn close(&mut self, _close: &mut Close<'_>) -> Result<()> {
            self.close_calls += 1;
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    runner.dispatch(Command::Destroy { id });

    runner
        .proxy()
        .close(id)
        .expect("a proxy accepts close work before the loop processes it");
    runner.flush_proxy();

    assert_eq!(runner.handler().close_calls, 0);
    assert!(runner.result().is_none());
}

#[test]
fn proxy_draw_after_close_does_not_schedule() {
    struct Noop;

    impl Handler for Noop {}

    let mut runner = testing::Runner::new(Noop);
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    assert_eq!(runner.take_ready_draws_for_test(Instant::now()), [id]);
    runner.dispatch(Command::Destroy { id });

    runner
        .proxy()
        .draw(id)
        .expect("a proxy accepts draw work before the loop processes it");
    runner.flush_proxy();

    assert!(runner.take_ready_draws_for_test(Instant::now()).is_empty());
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_proxy_flush_matches_native_fifo_policy() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<String>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if let EventKind::Created(state) = event.event() {
                let name = state.name().expect("test windows are named");
                self.callbacks.push(format!("created:{name}"));
                if name == "first" {
                    event
                        .context_mut()
                        .proxy()
                        .expect("runner callbacks receive the runner queue")
                        .open(open("third"))?;
                }
            }
            Ok(())
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.callbacks.push(format!(
                "ready:{}",
                ready.state().name().expect("test windows are named")
            ));
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    let proxy = runner.proxy();
    proxy
        .open(open("first"))
        .expect("normalized work enters the queue");
    proxy
        .open(open("second"))
        .expect("normalized work enters the queue");

    runner.flush_proxy();
    assert!(runner.host().events().is_empty());
    assert!(runner.handler().callbacks.is_empty());

    runner.resume();
    runner.flush_proxy();

    assert_eq!(
        runner.handler().callbacks,
        [
            "created:first",
            "ready:first",
            "created:second",
            "ready:second",
            "created:third",
            "ready:third",
        ]
    );
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_proxy_rejects_send_after_terminal_or_runner_drop() {
    struct Noop;

    impl Handler for Noop {}

    let mut terminal = testing::Runner::new(Noop);
    terminal.resume();
    let retained = terminal.proxy();
    retained.exit().expect("the running queue accepts exit");
    terminal.flush_proxy();
    assert!(matches!(terminal.result(), Some(Ok(()))));

    let error = retained
        .open(open("too late"))
        .expect_err("terminal runner queues are closed");
    assert_eq!(error.code, ErrorCode::CommandFailed);
    let error = retained
        .send(Command::Open {
            request: WindowRequest::builder("").build(),
        })
        .expect_err("queue closure outranks malformed post-terminal work");
    assert_eq!(error.code, ErrorCode::CommandFailed);
    terminal.flush_proxy();
    assert!(matches!(terminal.result(), Some(Ok(()))));

    let dropped = {
        let runner = testing::Runner::new(Noop);
        runner.proxy()
    };
    let error = dropped
        .open(open("after drop"))
        .expect_err("dropped runners close retained queues");
    assert_eq!(error.code, ErrorCode::CommandFailed);
}

#[test]
fn proxy_live_unsupported_work_remains_terminal() {
    struct Noop;

    impl Handler for Noop {}

    let mut runner = testing::Runner::new(Noop);
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup window is live");
    let proxy = runner.proxy();

    proxy
        .send(Command::SetFullscreen {
            id,
            fullscreen: Fullscreen::Exclusive,
        })
        .expect("intrinsically valid live work enters the queue");
    runner.flush_proxy();

    assert!(matches!(
        runner.result(),
        Some(Err(error)) if error.code == ErrorCode::UnsupportedFeature
    ));
    let error = proxy
        .open(open("after failure"))
        .expect_err("terminal planning failures close the queue");
    assert_eq!(error.code, ErrorCode::CommandFailed);
}

#[test]
fn test_runner_post_resume_dispatch_delivers_created_work_then_ready() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<String>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if let EventKind::Created(state) = event.event() {
                let name = state.name().expect("test windows are named");
                self.callbacks.push(format!("created:{name}"));
                if name == "main" {
                    event.context_mut().open(open("work"));
                }
            }
            Ok(())
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.callbacks.push(format!(
                "ready:{}",
                ready.state().name().expect("test windows are named")
            ));
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner.resume();
    runner.dispatch(open("main").into());

    assert_eq!(
        runner.handler().callbacks,
        ["created:main", "created:work", "ready:main", "ready:work"]
    );
    assert!(matches!(
        runner.host().events(),
        [HostEvent::Created(main), HostEvent::Created(work)]
            if main.name() == Some("main") && work.name() == Some("work")
    ));
    assert_eq!(
        runner
            .host()
            .commands()
            .iter()
            .filter(|command| matches!(command, Command::Open { .. }))
            .count(),
        2
    );
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_dispatch_remains_active_after_first_resume_then_suspend() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<String>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            match event.event() {
                EventKind::Created(state) => self.callbacks.push(format!(
                    "created:{}",
                    state.name().expect("test windows are named")
                )),
                EventKind::Suspended(_) => self.callbacks.push(String::from("suspended")),
                _ => {}
            }
            Ok(())
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.callbacks.push(format!(
                "ready:{}",
                ready.state().name().expect("test windows are named")
            ));
            Ok(())
        }

        fn suspend(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.callbacks.push(String::from("suspend"));
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("first").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    runner.handler_mut().callbacks.clear();

    runner.suspend();
    runner.dispatch(open("after-suspend").into());

    assert_eq!(
        runner.handler().callbacks,
        [
            "suspended",
            "suspend",
            "created:after-suspend",
            "ready:after-suspend"
        ]
    );
    assert!(matches!(
        runner.host().events(),
        [
            HostEvent::Created(first),
            HostEvent::Suspended(_),
            HostEvent::Created(after_suspend)
        ] if first.name() == Some("first") && after_suspend.name() == Some("after-suspend")
    ));
    assert!(matches!(
        runner.host().commands(),
        [Command::Open { request: first }, Command::Open { request: after_suspend }]
            if first.name() == Some("first") && after_suspend.name() == Some("after-suspend")
    ));
    assert_eq!(runner.host().window_id("first"), Some(Id::from_u64(1)));
    assert_eq!(
        runner.host().window_id("after-suspend"),
        Some(Id::from_u64(2))
    );
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_resume_suspend_order_matches_native_pump() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<&'static str>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            match event.event() {
                EventKind::Resumed(_) => self.callbacks.push("window-resumed"),
                EventKind::Suspended(_) => self.callbacks.push("window-suspended"),
                _ => {}
            }
            Ok(())
        }

        fn resume(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.callbacks.push("resume");
            Ok(())
        }

        fn suspend(&mut self, _context: &mut Context<'_>) -> Result<()> {
            self.callbacks.push("suspend");
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    runner.handler_mut().callbacks.clear();

    runner.resume();
    runner.suspend();

    assert_eq!(
        runner.handler().callbacks,
        ["window-resumed", "resume", "window-suspended", "suspend"]
    );
    assert!(matches!(
        runner.host().events(),
        [
            HostEvent::Created(_),
            HostEvent::Resumed(_),
            HostEvent::Suspended(_)
        ]
    ));
}

#[test]
fn test_runner_idle_uses_shared_pump_transaction() {
    #[derive(Default)]
    struct IdleHandler {
        id: Option<Id>,
        idle_calls: usize,
    }

    impl Handler for IdleHandler {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                self.id = Some(event.id());
            }
            Ok(())
        }

        fn wants_idle(&self) -> bool {
            true
        }

        fn idle(&mut self, context: &mut Context<'_>) -> Result<()> {
            self.idle_calls += 1;
            context
                .window(self.id.expect("created callback records the live ID"))
                .title("idle title");
            context.exit();
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(IdleHandler::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();

    runner.idle();
    runner.idle();

    assert_eq!(runner.handler().idle_calls, 1);
    assert!(matches!(
        runner.host().commands().last(),
        Some(Command::SetTitle { title, .. }) if title == "idle title"
    ));
    assert!(matches!(runner.result(), Some(Ok(()))));
}

#[test]
fn test_runner_idle_is_inert_before_resume_then_uses_shared_transaction() {
    #[derive(Default)]
    struct IdleHandler {
        idle_calls: usize,
    }

    impl Handler for IdleHandler {
        fn wants_idle(&self) -> bool {
            true
        }

        fn idle(&mut self, context: &mut Context<'_>) -> Result<()> {
            self.idle_calls += 1;
            context.open(open("idle"));
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(IdleHandler::default());
    let events = runner.host().events().to_vec();
    let commands = runner.host().commands().to_vec();

    runner.idle();

    assert_eq!(runner.handler().idle_calls, 0);
    assert_eq!(runner.host().events(), events);
    assert_eq!(runner.host().commands(), commands);
    assert!(runner.result().is_none());

    runner.resume();
    runner.idle();

    assert_eq!(runner.handler().idle_calls, 1);
    assert!(matches!(
        runner.host().commands(),
        [Command::Open { request }] if request.name() == Some("idle")
    ));
    assert!(matches!(
        runner.host().events(),
        [HostEvent::Created(state)] if state.name() == Some("idle")
    ));
    assert!(runner.result().is_none());
}

#[test]
fn test_runner_startup_is_no_op_after_terminal_result() {
    struct FailingResume;

    impl Handler for FailingResume {
        fn resume(&mut self, _context: &mut Context<'_>) -> Result<()> {
            Err(Error::new(ErrorCode::CommandFailed, "resume failed"))
        }
    }

    let mut failing = testing::Runner::new(FailingResume);
    failing
        .startup(vec![open("initial").into()])
        .expect("startup is stored before first resume");
    failing.resume();
    let failure = failing
        .result()
        .expect("failing resume reaches a terminal result")
        .expect_err("failing resume retains its error");
    let failure_code = failure.code;
    let failure_message = failure.message.clone();
    let failure_events = failing.host().events().to_vec();
    let failure_commands = failing.host().commands().to_vec();

    failing
        .startup(vec![open("ignored-after-failure").into()])
        .expect("startup after terminal failure is a no-op");

    assert!(matches!(
        failing.result(),
        Some(Err(error)) if error.code == failure_code && error.message == failure_message
    ));
    assert_eq!(failing.host().events(), failure_events);
    assert_eq!(failing.host().commands(), failure_commands);

    struct ExitingResume;

    impl Handler for ExitingResume {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.exit();
            Ok(())
        }
    }

    let mut exiting = testing::Runner::new(ExitingResume);
    exiting
        .startup(vec![open("initial").into()])
        .expect("startup is stored before first resume");
    exiting.resume();
    let success_events = exiting.host().events().to_vec();
    let success_commands = exiting.host().commands().to_vec();
    assert!(matches!(exiting.result(), Some(Ok(()))));

    exiting
        .startup(vec![open("ignored-after-exit").into()])
        .expect("startup after terminal success is a no-op");

    assert!(matches!(exiting.result(), Some(Ok(()))));
    assert_eq!(exiting.host().events(), success_events);
    assert_eq!(exiting.host().commands(), success_commands);
}

#[test]
fn test_runner_callback_failure_matches_native_terminal_result() {
    #[derive(Default)]
    struct FailingCreated {
        ready_calls: usize,
    }

    impl Handler for FailingCreated {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if matches!(event.event(), EventKind::Created(_)) {
                return Err(Error::new(
                    ErrorCode::CommandFailed,
                    "created callback failed",
                ));
            }
            Ok(())
        }

        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            self.ready_calls += 1;
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(FailingCreated::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let command_count = runner.host().commands().len();

    runner.dispatch(Command::SetTitle {
        id: Id::from_u64(1),
        title: String::from("ignored after failure"),
    });
    runner.resume();

    assert_eq!(runner.handler().ready_calls, 0);
    assert_eq!(runner.host().commands().len(), command_count);
    assert!(matches!(
        runner.result(),
        Some(Err(error)) if error.code == ErrorCode::CommandFailed
    ));
}

#[test]
fn test_runner_nested_commands_match_native_pump_order() {
    #[derive(Default)]
    struct Recorder {
        callbacks: Vec<String>,
    }

    impl Handler for Recorder {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            if let EventKind::Created(state) = event.event() {
                self.callbacks
                    .push(format!("created:{}", state.name().unwrap()));
                if state.name() == Some("first") {
                    event.context_mut().open(open("second"));
                }
            }
            Ok(())
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            self.callbacks
                .push(format!("ready:{}", ready.state().name().unwrap()));
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(Recorder::default());
    runner
        .startup(vec![open("first").into()])
        .expect("startup is stored before first resume");
    runner.resume();

    assert_eq!(
        runner.handler().callbacks,
        [
            "created:first",
            "created:second",
            "ready:first",
            "ready:second"
        ]
    );
    assert_eq!(
        runner
            .host()
            .commands()
            .iter()
            .filter(|command| matches!(command, Command::Open { .. }))
            .count(),
        2
    );
}

#[test]
fn test_runner_replaces_public_host_callback_dispatch_surface() {
    #[derive(Default)]
    struct ReadyCounter(usize);

    impl Handler for ReadyCounter {
        fn ready(&mut self, _ready: &mut Ready<'_>) -> Result<()> {
            self.0 += 1;
            Ok(())
        }
    }

    let mut host = testing::Host::new();
    host.apply(open("low-level"))
        .expect("low-level host applies plans");
    assert!(matches!(host.events(), [testing::Event::Created(_)]));

    let mut runner = testing::Runner::new(ReadyCounter::default());
    runner
        .startup(vec![open("startup").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    runner.dispatch(open("runner").into());

    assert_eq!(runner.handler().0, 2);
    assert!(matches!(
        runner.host().events(),
        [testing::Event::Created(_), testing::Event::Created(_)]
    ));
}

#[test]
fn test_runner_resized_transition_uses_resize_callback_and_schedules_draw() {
    #[derive(Default)]
    struct MetricHandler {
        resized: Vec<Metrics>,
        generic_events: Vec<EventKind>,
    }

    impl Handler for MetricHandler {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.push(resize.metrics().clone());
            Ok(())
        }

        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            self.generic_events.push(event.event().clone());
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(MetricHandler::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("open runner window has a stable ID");
    assert_eq!(runner.take_ready_draws_for_test(Instant::now()), [id]);
    runner.handler_mut().generic_events.clear();

    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 960,
            height: 540,
        },
        1.5,
    )
    .expect("test metrics are valid");
    runner.transition(NativeEventTransition::resized(metrics.clone()));

    assert_eq!(
        runner.handler().resized.as_slice(),
        std::slice::from_ref(&metrics)
    );
    assert!(runner.handler().generic_events.is_empty());
    assert_eq!(runner.take_ready_draws_for_test(Instant::now()), [id]);
    assert_eq!(
        runner
            .host()
            .registry()
            .get(id)
            .expect("transition keeps the window live")
            .metrics(),
        metrics
    );
}

#[test]
fn test_runner_scale_factor_transition_uses_resize_callback_and_commits_metrics() {
    #[derive(Default)]
    struct MetricHandler {
        resized: Vec<Metrics>,
        generic_events: Vec<EventKind>,
    }

    impl Handler for MetricHandler {
        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            self.resized.push(resize.metrics().clone());
            Ok(())
        }

        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            self.generic_events.push(event.event().clone());
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(MetricHandler::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("open runner window has a stable ID");
    assert_eq!(runner.take_ready_draws_for_test(Instant::now()), [id]);
    runner.handler_mut().generic_events.clear();

    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: 800,
            height: 600,
        },
        2.5,
    )
    .expect("test metrics are valid");
    runner.transition(NativeEventTransition::scale_factor_changed(metrics.clone()));

    assert_eq!(
        runner.handler().resized.as_slice(),
        std::slice::from_ref(&metrics)
    );
    assert!(runner.handler().generic_events.is_empty());
    assert_eq!(runner.take_ready_draws_for_test(Instant::now()), [id]);
    assert_eq!(
        runner
            .host()
            .registry()
            .get(id)
            .expect("transition keeps the window live")
            .metrics(),
        metrics
    );
}

#[test]
fn test_runner_transition_preserves_input_and_generic_callback_routes() {
    #[derive(Default)]
    struct RouteHandler {
        inputs: Vec<InputEvent>,
        generic_events: Vec<EventKind>,
    }

    impl Handler for RouteHandler {
        fn input(&mut self, input: &mut Input<'_>) -> Result<()> {
            self.inputs.push(input.event().clone());
            Ok(())
        }

        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            self.generic_events.push(event.event().clone());
            Ok(())
        }
    }

    let mut runner = testing::Runner::new(RouteHandler::default());
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("open runner window has a stable ID");
    runner.handler_mut().generic_events.clear();

    runner.transition(NativeEventTransition::mouse_moved(
        id,
        Point { x: 10.0, y: 20.0 },
        PhysicalPoint { x: 20, y: 40 },
        None,
        ModifierState::default(),
    ));
    runner.transition(NativeEventTransition::focused(id, true));

    assert!(matches!(
        runner.handler().inputs.as_slice(),
        [InputEvent::Pointer(PointerEvent {
            id: input_id,
            phase: PointerPhase::Moved,
            ..
        })] if *input_id == id
    ));
    assert!(matches!(
        runner.handler().generic_events.as_slice(),
        [EventKind::Focused {
            id: event_id,
            focused: true,
        }] if *event_id == id
    ));
}

struct CallbackClipboard {
    writes: Rc<RefCell<Vec<String>>>,
    fail_writes: bool,
}

impl Clipboard for CallbackClipboard {
    fn read_text(&mut self) -> Result<Option<String>> {
        Ok(None)
    }

    fn write_text(&mut self, text: &str) -> Result<()> {
        if self.fail_writes {
            return Err(Error::new(
                ErrorCode::ClipboardWriteFailed,
                "injected clipboard rejected text",
            ));
        }
        self.writes.borrow_mut().push(text.to_owned());
        Ok(())
    }

    fn read_image(&mut self) -> Result<Option<ClipboardImage>> {
        Ok(None)
    }

    fn write_image(&mut self, _image: ClipboardImageRef<'_>) -> Result<()> {
        Ok(())
    }
}

#[test]
fn handler_context_uses_injected_clipboard() {
    struct WritesFromResume;

    impl Handler for WritesFromResume {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.clipboard().write_text("from handler")
        }
    }

    let writes = Rc::new(RefCell::new(Vec::new()));
    let mut runner =
        testing::Runner::new(WritesFromResume).with_clipboard(Box::new(CallbackClipboard {
            writes: writes.clone(),
            fail_writes: false,
        }));

    runner.resume();

    assert_eq!(writes.borrow().as_slice(), ["from handler"]);
    assert!(runner.result().is_none());
}

#[test]
fn every_callback_receives_same_clipboard_instance() {
    struct WritesFromEveryCallback;

    impl Handler for WritesFromEveryCallback {
        fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
            event.context_mut().clipboard().write_text("event")
        }

        fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
            ready.context_mut().clipboard().write_text("ready")
        }

        fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
            close.context_mut().clipboard().write_text("close")?;
            close.close();
            Ok(())
        }

        fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
            closed.context_mut().clipboard().write_text("closed")
        }

        fn resize(&mut self, resize: &mut Resize<'_>) -> Result<()> {
            resize.context_mut().clipboard().write_text("resize")
        }

        fn input(&mut self, input: &mut Input<'_>) -> Result<()> {
            input.context_mut().clipboard().write_text("input")
        }

        fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
            frame.context_mut().clipboard().write_text("draw")
        }

        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.clipboard().write_text("resume")
        }

        fn suspend(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.clipboard().write_text("suspend")
        }

        fn wants_idle(&self) -> bool {
            true
        }

        fn idle(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.clipboard().write_text("idle")
        }
    }

    let writes = Rc::new(RefCell::new(Vec::new()));
    let mut runner =
        testing::Runner::new(WritesFromEveryCallback).with_clipboard(Box::new(CallbackClipboard {
            writes: writes.clone(),
            fail_writes: false,
        }));
    runner
        .startup(vec![open("main").into()])
        .expect("startup is stored before first resume");
    runner.resume();
    let id = runner
        .host()
        .window_id("main")
        .expect("startup opens the test window");

    runner.draw(id);
    runner.transition(NativeEventTransition::resized(
        Metrics::from_physical_size(
            id,
            PhysicalSize {
                width: 400,
                height: 300,
            },
            1.0,
        )
        .expect("test metrics are valid"),
    ));
    runner.transition(NativeEventTransition::mouse_moved(
        id,
        Point { x: 10.0, y: 20.0 },
        PhysicalPoint { x: 10, y: 20 },
        None,
        ModifierState::default(),
    ));
    runner.suspend();
    runner.resume();
    runner.idle();
    runner.request_close(id);

    let writes = writes.borrow();
    for callback in [
        "event", "ready", "close", "closed", "resize", "input", "draw", "resume", "suspend", "idle",
    ] {
        assert!(writes.iter().any(|write| write == callback));
    }
    assert!(runner.result().is_none());
}

#[test]
fn clipboard_error_becomes_terminal_callback_error() {
    struct FailingClipboardHandler;

    impl Handler for FailingClipboardHandler {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.open(open("discarded"));
            context.clipboard().write_text("rejected")
        }
    }

    let writes = Rc::new(RefCell::new(Vec::new()));
    let mut runner =
        testing::Runner::new(FailingClipboardHandler).with_clipboard(Box::new(CallbackClipboard {
            writes,
            fail_writes: true,
        }));

    runner.resume();

    assert!(matches!(
        runner.result(),
        Some(Err(error)) if error.code == ErrorCode::ClipboardWriteFailed
    ));
    assert_eq!(runner.host().window_id("discarded"), None);
}

#[test]
fn successful_clipboard_write_is_not_rolled_back_on_handler_error() {
    struct FailingHandler;

    impl Handler for FailingHandler {
        fn resume(&mut self, context: &mut Context<'_>) -> Result<()> {
            context.clipboard().write_text("persistent")?;
            context.open(open("discarded"));
            Err(Error::new(
                ErrorCode::CommandFailed,
                "handler failed after write",
            ))
        }
    }

    let writes = Rc::new(RefCell::new(Vec::new()));
    let mut runner =
        testing::Runner::new(FailingHandler).with_clipboard(Box::new(CallbackClipboard {
            writes: writes.clone(),
            fail_writes: false,
        }));

    runner.resume();

    assert_eq!(writes.borrow().as_slice(), ["persistent"]);
    assert!(matches!(
        runner.result(),
        Some(Err(error)) if error.message == "handler failed after write"
    ));
    assert_eq!(runner.host().window_id("discarded"), None);
}

#[derive(Default)]
struct NoopHandler;

impl Handler for NoopHandler {}
