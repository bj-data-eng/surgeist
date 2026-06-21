# surgeist::window Requirements

`surgeist::window` is the native window boundary for Surgeist. It wraps `winit` behind a lifecycle-first Rust API for event-loop control, native window creation, native window state, native input capture, native services, live native handle access, and draw scheduling.

The public API should be designed for use as `surgeist::window::*`. The module path supplies the layer name, so public type names should stay short and local.

App code uses the lifecycle surface: `app`, `open`, `Loop`, `Handler`, `Context`, `Ready`, `Resize`, `Input`, `Close`, `Closed`, `Frame`, `Proxy`, `Id`, `Handle`, `State`, `Metrics`, `Role`, `InputEvent`, `Code`, and `Error`.

Inspectable contract values such as `Descriptor` and `Command` remain available for builder lowering, tests, proxies, and enabled window capabilities.

Adapter and scheduler machinery may expose types inside the crate for tests and diagnostics, but those types are not a second app-authoring surface.

## Scope

This module owns:

- Native event-loop integration with `winit`.
- Native window creation, tracking, and lifecycle state.
- Native platform event capture as stable wrapper events.
- Native draw scheduling at the event-loop boundary.
- Native window commands: title, size, visibility, decorations, controls, fullscreen, attention, close, cursor, cursor grab, and IME requests.
- Native clipboard access and memory fallback clipboard.
- Native AccessKit adapter connection and action-event forwarding when enabled.
- Live native handle access for renderer/surface creation.
- Platform/window diagnostics with stable error codes.

This module describes native window facts and native window actions. It does not compute application meaning, own rendering resources, own UI semantics, or interpret input beyond preserving platform facts.

## Dependencies

Expected direct dependencies:

```text
surgeist-window
  -> winit
  -> raw-window-handle
  -> cursor-icon
  -> keyboard-types
  -> optional clipboard backend(s)
  -> optional accesskit-winit bridge types
```

`surgeist-window` has no direct dependency on other Surgeist crates. Keep local primitives small so the crate remains reusable and easy to test.

Potential clipboard dependencies:

- `arboard` for common desktop clipboard support.
- `smithay-clipboard` for Wayland-specific behavior if required.

## Naming

Names are authored for the `surgeist::window` namespace:

- `Loop` owns the native event loop runner.
- `Handler` is the lifecycle callback trait implemented by consumers.
- `Ready`, `Resize`, `Input`, `Close`, `Closed`, and `Frame` are lifecycle callback scopes.
- `Context` is the lifecycle callback command and query context.
- `Proxy` is a cloneable cross-thread event-loop wakeup handle.
- `Target` is a fluent command helper for an existing live window.
- `Selector` is a read-only lookup selector for `Id` or stable window name.
- `Handle` is a cloneable live native handle token for renderer integration.
- `Ref` is borrowed native window access.
- `Access` is a trait for borrowed native window capabilities.
- `Id` is an opaque native window identifier.
- `Descriptor` is requested window configuration.
- `State` is observed native window state.
- `Metrics` is observed native window geometry and scale.
- `Theme` is observed native light/dark appearance.
- `Role` describes native window role and parent relationship.
- `Modality` describes dialog blocking intent.
- `InputEvent` is native input captured at the window boundary.
- `Code` is the stable physical key code used by common input predicates.
- `Command` is an inspectable native window command consumed by this module.
- `Error` and `ErrorCode` are stable diagnostics.
- `Result<T>` is this module's result alias.

Internal and testing names:

- `Registry` stores live native windows by `Id`.
- `Instance` is an owned live native window entry stored by the registry.
- `EventKind` is the normalized native event payload used by the adapter, diagnostics, and tests.
- `Action` is the internal scheduler/action representation produced by lifecycle helpers and test hosts.
- `DrawScheduler` owns per-window draw coalescing and delayed draw deadlines.

Avoid repeating `Window` in type names when the module path already supplies it. Prefer `surgeist::window::Command` over `surgeist::window::WindowCommand`.

## Geometry

The module may define minimal geometry primitives:

```rust
pub struct Point { pub x: f64, pub y: f64 }
pub struct Size { pub width: f64, pub height: f64 }
pub struct PhysicalPoint { pub x: i32, pub y: i32 }
pub struct PhysicalSize { pub width: u32, pub height: u32 }
pub struct Insets { pub top: f64, pub right: f64, pub bottom: f64, pub left: f64 }
pub struct Rect { pub origin: Point, pub size: Size }
```

Use logical coordinates unless a type explicitly says `Physical`.

## Public Lifecycle API

The lifecycle API is the app-facing authoring front door. Consumers implement
`Handler`, open windows with `Open`, query native state through lifecycle scopes
and `Context`, and request native work through scope, context, target, or proxy
methods.

The lifecycle DSL requirements in [`window_dsl.md`](window_dsl.md) refine this
front door. Lower-level adapter events, scheduler actions, registry entries, and
native command-drain details are implementation and testing concerns.

```rust
pub struct Loop<H> {
    handler: H,
    registry: Registry,
    draw: DrawScheduler,
    clipboard: Box<dyn Clipboard>,
}

pub trait Handler {
    fn resume(&mut self, cx: &mut Context<'_>) -> Result<()>;
    fn suspend(&mut self, cx: &mut Context<'_>) -> Result<()>;
    fn ready(&mut self, win: &mut Ready<'_>) -> Result<()>;
    fn resize(&mut self, win: &mut Resize<'_>) -> Result<()>;
    fn input(&mut self, input: &mut Input<'_>) -> Result<()>;
    fn close(&mut self, close: &mut Close<'_>) -> Result<()>;
    fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()>;
    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()>;
    fn wants_idle(&self) -> bool;
    fn idle(&mut self, cx: &mut Context<'_>) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id(/* private */);

pub struct Instance {
    // private native window storage
}

pub struct Ref<'a> {
    // borrowed native window access
}

pub struct Handle {
    // cloneable owner-backed native handle token
}

pub trait Access {
    fn id(&self) -> Id;
    fn metrics(&self) -> Metrics;
    fn handle(&self) -> Result<Handle>;
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>>;
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>>;
}
```

`Loop<H>` owns native orchestration. `Handler` receives lifecycle callbacks and
requests native work through scoped callback objects. App-facing callbacks are
`resume`, `suspend`, `ready`, `resize`, `input`, `close`, `closed`, `draw`, and
optional idle callbacks.

`Handle` is the preferred bridge to renderer crates. It should hold the native owner needed to keep the window valid and implement `raw_window_handle::HasWindowHandle` and `raw_window_handle::HasDisplayHandle` so renderer integrations such as Vello/wgpu can create surfaces without receiving loose raw values. Raw handles are available only through the typed handle traits.

`Proxy` is available from live handler contexts and can be cloned into app systems or worker threads. It wakes the native event loop for commands, draw requests, delayed draw requests, and exit requests.

`Ref<'_>` and `Access` expose borrowed native capabilities for renderer
attachment and adapter integration. Borrowed native access does not schedule
draws directly; draw scheduling flows through lifecycle scopes, `Context`,
`Target`, `Proxy`, or command draining so callback rollback remains coherent.
`Instance`, `Registry`, `DrawScheduler`, and `EventKind` are adapter,
scheduler, diagnostic, or testing types.

## Window State

```rust
#[derive(Clone, Debug)]
pub struct Descriptor {
    pub title: String,
    pub name: Option<String>,
    pub position: Option<Point>,
    pub inner_size: Option<Size>,
    pub min_inner_size: Option<Size>,
    pub max_inner_size: Option<Size>,
    pub resizable: bool,
    pub controls: Controls,
    pub decorations: bool,
    pub transparent: bool,
    pub visible: bool,
    pub fullscreen: Fullscreen,
    pub level: Level,
    pub theme: Option<Theme>,
    pub role: Role,
}

#[derive(Clone, Debug)]
pub struct State {
    pub id: Id,
    pub title: String,
    pub name: Option<String>,
    pub metrics: Metrics,
    pub position: Option<Point>,
    pub focused: bool,
    pub visible: Option<bool>,
    pub minimized: Option<bool>,
    pub maximized: bool,
    pub occluded: Option<bool>,
    pub fullscreen: bool,
    pub theme: Option<Theme>,
    pub role: Role,
}

#[derive(Clone, Debug)]
pub struct Metrics {
    pub id: Id,
    pub logical_size: Size,
    pub physical_size: PhysicalSize,
    pub outer_position: Option<Point>,
    pub outer_size: Option<Size>,
    pub scale_factor: f64,
    pub safe_area: Insets,
}

pub struct Registry {
    // private storage keyed by Id
}

pub struct Controls {
    pub close: bool,
    pub minimize: bool,
    pub maximize: bool,
}

pub enum Theme {
    Light,
    Dark,
}
```

`Descriptor` is requested configuration. `State` and `Metrics` are observed native state.

`State` and `Metrics` must contain enough information for callers to persist and restore native window placement without inspecting `winit` types directly.

`Descriptor::name` is the app-facing stable window name used for lookup,
correlation, diagnostics, and testing. Non-empty names are unique among live
windows. It is not a user-facing title and should not be mutable after native
creation.

Native platform identity, such as Wayland app id or X11 class, is not encoded by
`Descriptor::name`. If per-app or per-window native identity becomes necessary,
it must use a separate explicitly named field or app-level configuration.

`Descriptor::theme` is host appearance override intent. `State::theme` is the observed native appearance. A `None` theme means the platform default is in effect or unavailable.

## Multiple Windows

Multiple native windows are part of the first API version.

```rust
pub enum Role {
    Root,
    Dialog { parent: Id, modality: Modality },
    Tool { parent: Option<Id> },
    Popup { parent: Id },
}

pub enum Modality {
    Window,
    App,
    Modeless,
}
```

Rules:

- Per-window lifecycle scopes and per-window native event payloads carry or are
  delivered with an `Id`. App-level lifecycle callbacks, such as `resume` and
  `suspend`, receive `Context` and do not imply one window.
- Every `Command` targets an `Id` or explicitly opens a new window.
- Each native window has independent metrics, focus state, cursor state, IME state, draw schedule, accessibility adapter, and live handle access.
- Clipboard is app/global.
- Closing one window is distinct from exiting the app.
- Dialog modality and parent relationships are explicit host intent.
- Parent/role relationships are retained even when a platform can only approximate them.
- Native creation must reject unsupported role wiring with `UnsupportedFeature` instead of silently dropping parent or modality intent.

## Native Event Payloads

```rust
pub(crate) enum EventKind {
    Created(State),
    Destroyed(Id),
    Suspended,
    Resumed,
    CloseRequested(Id),
    Focused { id: Id, focused: bool },
    Resized(Metrics),
    ScaleFactorChanged(Metrics),
    Moved { id: Id, position: Point },
    Occluded { id: Id, occluded: bool },
    ThemeChanged { id: Id, theme: Option<Theme> },
    FileDrag(FileDragEvent),
    Input(InputEvent),
    Accessibility(AccessibilityEvent),
}
```

`EventKind` is the normalized native event payload used by the adapter,
diagnostics, and tests. It is not a parallel public handler model. Native
adapter paths call `Handler` lifecycle callbacks directly for app-facing work.
Lifecycle scopes expose app-facing state queries such as `is_occluded()` and
`is_focused()` for window conditions.

## Input

```rust
pub enum InputEvent {
    Pointer(PointerEvent),
    Wheel(WheelEvent),
    Key(KeyEvent),
    Modifiers { id: Id, modifiers: ModifierState },
    Ime(ImeEvent),
    StandardKeyBinding(StandardKeyBindingEvent),
}
```

`InputEvent` captures native input facts. It should preserve logical and physical coordinates, pointer delta when available, pointer enter/leave, pointer kind/source, pointer identity for touch/stylus contacts, touch phase, button identity, button press/release, wheel delta and phase, logical and physical keyboard keys, keyboard location, key press/release, repeat/synthetic state, modifiers, IME preedit/commit/delete-surrounding text, and event timestamp.

Pointer events should preserve extended device data when the platform provides it: force, pressure, tangential pressure, tilt, twist, altitude, and azimuth. The module records multi-touch contacts; gesture recognition belongs above this crate.

Keyboard events should use `keyboard-types` or an equivalent stable representation so consumers receive consistent logical keys, physical codes, locations, and modifiers without depending on raw `winit` enums. On macOS, standard text-editing key bindings should be forwarded as native input facts instead of being collapsed into app commands.

The module records input facts; it does not compute hit targets, focus targets, gestures, text selection, commands, or application meaning.

File drag/drop should preserve entered, hovered, dropped, and cancelled phases plus paths or platform payload metadata when available. Hovered and dropped positions are optional because native backends do not always provide coordinates for drag payload events.

## IME

```rust
pub enum ImeRequest {
    Disable,
    Enable(ImeConfig),
    Update(ImeConfig),
    Restart(ImeConfig),
}

pub struct ImeConfig {
    pub purpose: ImePurpose,
    pub hint: ImeHint,
    pub cursor_area: Option<Rect>,
    pub surrounding_text: Option<ImeSurroundingText>,
}

pub enum ImePurpose {
    Normal,
    Password,
    Number,
    Email,
    Url,
    Terminal,
}
```

Rules:

- Callers decide when IME should be active.
- Apply `ImeRequest` to winit using `Window::request_ime_update` where available.
- Changing editable targets should restart IME by disabling then enabling with the new target data.
- Cursor area updates follow layout, scroll, and scale changes.
- IME failures produce diagnostics.

## Clipboard

```rust
pub trait Clipboard {
    fn read_text(&mut self) -> Result<Option<String>>;
    fn write_text(&mut self, text: &str) -> Result<()>;
    fn read_image(&mut self) -> Result<Option<ClipboardImage>>;
    fn write_image(&mut self, image: ClipboardImageRef<'_>) -> Result<()>;
}
```

Rules:

- Provide a memory fallback clipboard when OS clipboard is unavailable.
- Clipboard operations are native reads and writes.
- Clipboard errors produce diagnostics unless a caller asks for fallible behavior.

## Cursor

```rust
pub enum Cursor {
    Icon(cursor_icon::CursorIcon),
    Hidden,
    Custom(CustomCursorId),
}

pub enum CursorGrab {
    None,
    Confined,
    Locked,
}
```

Rules:

- Deduplicate cursor updates.
- Cursor visibility and cursor grab are explicit native window requests.
- Failed cursor or grab requests produce diagnostics.

## Internal Actions

```rust
pub(crate) enum Action {
    Wait,
    DrawNow(Id),
    DrawNext(Id),
    DrawAt { id: Id, time: std::time::Instant },
    Close(Id),
    Exit,
    Batch(Vec<Action>),
}
```

`Action` is the internal scheduler representation used by lifecycle scopes,
`Context`, `Target`, `Proxy`, and the test host. Routine app code requests work
through fluent methods such as `draw()`, `again()`, `at(time)`, `close()`, and
`exit()` instead of constructing `Action` directly.

`Loop` owns draw coalescing, delayed draw, low-power waits,
invisible/minimized throttling, drawable-state recovery, and cross-thread
wakeups.

Consumers draw from `Handler::draw`. `surgeist::window` schedules the first draw
after `ready`, schedules draw after metrics changes, preserves pending draw
requests while native state is not drawable, and retries pending draws when the
window becomes drawable again.

## Commands

```rust
pub enum Command {
    Open { descriptor: Descriptor },
    SetTitle { id: Id, title: String },
    SetPosition { id: Id, position: Point },
    SetVisible { id: Id, visible: bool },
    SetResizable { id: Id, resizable: bool },
    SetControls { id: Id, controls: Controls },
    SetDecorations { id: Id, decorations: bool },
    SetTransparent { id: Id, transparent: bool },
    SetInnerSize { id: Id, size: Size },
    SetMinInnerSize { id: Id, size: Option<Size> },
    SetMaxInnerSize { id: Id, size: Option<Size> },
    SetFullscreen { id: Id, fullscreen: Fullscreen },
    SetLevel { id: Id, level: Level },
    SetTheme { id: Id, theme: Option<Theme> },
    SetCursor { id: Id, cursor: Cursor },
    SetCursorGrab { id: Id, grab: CursorGrab },
    SetIme { id: Id, request: ImeRequest },
    RequestUserAttention { id: Id },
    RequestDraw { id: Id },
    Destroy { id: Id },
}
```

`Command` is an inspectable native command contract for builder lowering,
proxies, tests, and enabled window capabilities. Lifecycle methods on `Open`,
scopes, `Context`, `Target`, and `Proxy` provide the same capabilities as named
window operations.

Rules:

- Commands are explicit and idempotent where possible.
- `Context::close(id)`, `Target::close()`, and `Proxy::close(id)` request the
  cancelable `close` lifecycle for the target window.
- `Command::RequestDraw { id }` is an inspectable command bridge that the
  command drain converts into the internal draw scheduler representation.
  Routine app code uses `draw()` methods instead of constructing draw commands.
- `Close::close()` accepts a pending close request and lowers to
  `Command::Destroy { id }`.
- `Close::cancel()` discards the pending close request and keeps the window
  live.
- `Command::Destroy` is the accepted native destruction command. It is distinct
  from requesting close.
- `SetControls`, `SetTheme`, and `RequestUserAttention` are advisory native host
  requests; support varies by platform and must be reported through diagnostics
  when unavailable.
- Fatal command failures produce diagnostics with target ids and stop the loop.
  Fatal failures include native window creation failure, unsupported role wiring
  during creation, duplicate live names, missing required handles, and destroy
  failures.
- Advisory command failures produce diagnostics with target ids and continue
  when the requested native feature is optional and the window remains valid.
- Command application must be testable through a fake native adapter.
- Platform-unsupported commands produce diagnostics rather than silently changing the contract.

## Native Handles

Native handle access lives on `Access`, `Ref<'_>`, and `Handle`.

Rules:

- Prefer passing `Handle` or a borrowed `Access` object to renderer integrations instead of passing detached raw handle values.
- `Handle` must keep the native window alive for as long as a renderer may create a native surface from it.
- Borrowed `WindowHandle<'_>` and `DisplayHandle<'_>` access is tied to a live native window and must not outlive the borrow.
- `Handle` and borrowed access types must document thread and platform constraints. Do not make native handle access `Send` or `Sync` unless the wrapper owns the lifetime and synchronization guarantees required by the underlying platform.
- Handle lookup failures produce diagnostics with the target `Id`.
- Resize, scale-factor, suspend, resume, and destroyed notifications are
  normalized by the internal adapter event payload before lifecycle dispatch.
- This module does not cache or own graphics resources, swapchains, render targets, or renderer-created objects.

## Accessibility

```rust
pub enum AccessibilityEvent {
    InitialTreeRequested(Id),
    ActionRequested(AccessibilityActionRequest),
    Deactivated(Id),
}
```

Rules:

- The module owns the native AccessKit adapter connection when enabled.
- AccessKit action requests are reported as `AccessibilityEvent`.
- Accessibility updates are accepted through adapter-facing methods or traits.
- Accessibility actions can request draw.
- Initial tree request and accessibility deactivation are native lifecycle events.
- A visible window may be delayed until its AccessKit adapter is initialized if required by the platform backend.

## Errors

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub id: Option<Id>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub enum ErrorCode {
    EventLoopCreateFailed,
    WindowCreateFailed,
    HandleUnavailable,
    ImeUnsupported,
    ImeRequestFailed,
    ClipboardUnavailable,
    ClipboardReadFailed,
    ClipboardWriteFailed,
    CursorRequestFailed,
    CommandFailed,
    UnsupportedFeature,
    AccessibilityAdapterFailed,
    UnknownNativeError,
}
```

Error codes must remain stable. Display messages may improve over time.

## Tests

Required contract tests:

- `Descriptor` converts to native attributes without losing size, visibility, decorations, transparency, fullscreen, or level intent.
- `Descriptor` reports unsupported native role wiring instead of silently creating a root window.
- `Descriptor` preserves app-facing name, controls, and theme override intent.
- `State` and `Metrics` expose enough position, size, scale, fullscreen, maximized, minimized, and occlusion data for window setting persistence.
- `State` reports observed native theme changes without requiring consumers to read raw `winit` types.
- `Metrics` converts physical/logical sizes, outer geometry, safe area, and scale factor correctly.
- `Registry` tracks multiple ids and removes closed windows without disturbing surviving windows.
- `Role::Dialog`, `Role::Tool`, and `Role::Popup` preserve parent relationships.
- Draw scheduling coalesces repeated draw requests and chooses earliest delayed draw per id.
- Invisible/minimized windows are throttled.
- Event-loop proxy wakeups can trigger delayed draw from another thread.
- Pointer, touch, keyboard, modifier, wheel, IME, standard key binding, and
  file-drag/drop phases are captured as `InputEvent` or the internal normalized
  native event payload.
- Extended pointer data is preserved when available and gracefully absent when unsupported.
- Clipboard fallback works when OS clipboard is unavailable.
- Cursor updates are deduplicated.
- IME enable/update/disable/restart requests are applied in correct order.
- `Command` failures produce stable `ErrorCode` values.
- AccessKit action requests become `AccessibilityEvent`.
- Startup `ready` schedules the first draw.
- Resize and scale changes update `Metrics` before `resize` and schedule draw after the callback.
- Pending draws retry when native state moves from occluded to drawable.
- Default close accepts close requests while `Close::cancel()` keeps the window live.
- `window::testing` can inspect recorded lifecycle order, recorded commands,
  recorded draw requests, diagnostics, and rollback results without exposing
  `Action` or `EventKind` as app-facing APIs.

Required smoke tests:

- Open one native window.
- Deliver one `ready` callback and one `draw` callback.
- Resize the window and observe `Metrics`.
- Route pointer and keyboard input into a fake `Handler`.
- Request delayed draw and verify wakeup behavior.
- Request cross-thread draw and verify event-loop wakeup behavior.
- Open and close a second native window.

## First Milestone

Create a `hello-window` example that:

1. Creates a native window through `surgeist::window::app`.
2. Attaches per-window state in `ready`.
3. Draws from `draw`.
4. Supports draw scheduling.
5. Supports resize metrics.
6. Applies cursor and title commands.
7. Includes registry tests for multiple ids.
8. Includes command conversion and draw scheduling tests.
