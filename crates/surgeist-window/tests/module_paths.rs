//! Every public item resolves at exactly the path its module documents.
//!
//! The imports are the check: a missing or misplaced export fails to compile.
//! The accessibility section covers the feature surface, which the generated
//! API audit does not build.

#![expect(
    unused_imports,
    reason = "each import is a path assertion; a missing export fails to compile"
)]

use surgeist_window::callback::{
    Close, Closed, Context, Event, Frame, Input, Ready, Resize, Scope, Selector, Target,
};
use surgeist_window::capability::{
    CapabilityDecision, CapabilityKind, CapabilitySupport, CursorCapability, FullscreenMode,
    HostCapabilities, HostCapabilitiesBuilder, ImeCapability, RoleKind, WindowOperation,
};
use surgeist_window::clipboard::{Clipboard, ClipboardImage, ClipboardImageRef, MemoryClipboard};
use surgeist_window::command::{Command, CommandKind, HostCommandPlan};
use surgeist_window::cursor::{Cursor, CursorGrab, CursorIcon, CustomCursorId};
use surgeist_window::descriptor::{
    Controls, ControlsBuilder, Fullscreen, Level, Metrics, Modality, Role, Theme, WindowRequest,
    WindowRequestBuilder, WindowSnapshot, controls,
};
use surgeist_window::environment::{
    ClientWindowMetrics, ContrastPreference, DisplayMode, EnvironmentBlending, HostEnvironment,
    InputCapabilities, NavigationControls, OutputDevice, OutputResolution, PointerAccuracy,
    PointerCapability, ReductionPreference, ScaleFactor, ScreenAreaSource, ScreenMetrics,
    UserPreferences, ViewportMetrics, ViewportSizes, VisualViewportMetrics,
};
use surgeist_window::event::{
    EventKind, FileDragEvent, ImeConfig, ImeEvent, ImeHint, ImePurpose, ImeRequest,
    ImeSurroundingText, InputEvent, KeyEvent, KeyState, MagnificationDelta, MagnificationEvent,
    MagnificationGesture, ModifierState, NativePointerCoordinates, PointerButton,
    PointerDeviceData, PointerEvent, PointerKind, PointerPhase, StandardKeyBindingEvent,
    TouchPhase, WheelDelta, WheelEvent, keyboard_types,
};
use surgeist_window::geometry::{
    Insets, PhysicalPoint, PhysicalSize, Point, Rect, Size, point, rect, size,
};
use surgeist_window::host::{Loop, MetricsEvent, NativeEventTransition, WindowStatePatch};
use surgeist_window::native_geometry::{
    NativeGeometryAuthorization, NativeGeometryBasis, NativeGeometryDisposition,
    NativeGeometryNoAction, NativeGeometryOutcome, NativeGeometryRequest, NativeOpenFeatures,
};
use surgeist_window::registry::{
    Access, Handle, Id, Instance, Proxy, Ref, Registry, raw_window_handle,
};
use surgeist_window::system_fonts::{
    AppKitFontMetrics, AppKitFontTraits, AppKitFontWeight, AppKitSystemFont, SystemFont,
    SystemFontEnvironment, SystemFontError, SystemFontRole, WindowsSystemFont,
};
use surgeist_window::testing::{Effect, Host, Runner};
use surgeist_window::{App, Error, ErrorCode, Handler, Instant, Open, Result, app, open};

#[test]
fn the_root_keeps_the_program_the_failure_type_and_the_clock() {
    struct Quiet;
    impl Handler for Quiet {}

    let _app: App<Quiet> = app(Quiet).open(open("main"));
    let _open: Open = open("second");
    let failure: Result<()> = Err(Error::new(ErrorCode::CommandFailed, "closed"));
    assert!(failure.is_err());
    assert!(Instant::now() <= Instant::now());
}

#[test]
fn constructor_functions_live_in_their_domain_modules() {
    assert_eq!(point(1, 2), Point { x: 1.0, y: 2.0 });
    assert_eq!(
        size(3, 4),
        Size {
            width: 3.0,
            height: 4.0
        }
    );
    assert_eq!(rect(1, 2, 3, 4).size, size(3, 4));
    let _controls: ControlsBuilder = controls();
}

#[test]
fn the_kept_foreign_crates_are_reached_through_their_modules() {
    assert_eq!(keyboard_types::Code::Escape, keyboard_types::Code::Escape);
    assert!(matches!(
        Cursor::Icon(CursorIcon::Pointer),
        Cursor::Icon(CursorIcon::Pointer)
    ));
    assert!(matches!(
        raw_window_handle::HandleError::Unavailable,
        raw_window_handle::HandleError::Unavailable
    ));
}

#[test]
fn window_identity_lives_in_the_registry_module() {
    assert_eq!(Id::from_u64(9).as_u64(), 9);
}

#[test]
fn the_testing_module_keeps_its_own_event_type() {
    let created: surgeist_window::testing::Event =
        surgeist_window::testing::Event::Destroyed(Id::from_u64(1));
    assert_eq!(created.id(), Id::from_u64(1));
}

#[cfg(feature = "accessibility")]
mod accessibility {
    use surgeist_window::accessibility::{
        AccessibilityActionRequest, AccessibilityEvent, accesskit,
    };

    #[expect(
        unused_imports,
        reason = "each import is a path assertion; a missing export fails to compile"
    )]
    use surgeist_window::accessibility::accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate};

    #[test]
    fn typed_events_and_the_accesskit_crate_live_in_the_accessibility_module() {
        let id = surgeist_window::registry::Id::from_u64(3);
        assert!(matches!(
            AccessibilityEvent::InitialTreeRequested(id),
            AccessibilityEvent::InitialTreeRequested(_)
        ));
        let _node = accesskit::Node::new(accesskit::Role::Window);
        let _request: Option<AccessibilityActionRequest> = None;
    }
}
