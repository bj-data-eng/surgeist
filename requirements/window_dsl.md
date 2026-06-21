# surgeist::window Lifecycle DSL Requirements

`surgeist::window` exposes a lifecycle-first authoring surface over native
windows. The DSL should make app code describe window intent while the
window layer owns native timing, draw scheduling, first-frame readiness,
occlusion recovery, resize churn, command buffering, and callback rollback.

The public API is designed for the `surgeist::window` namespace. Names stay short
because the module path supplies the layer: `app`, `open`, `Ready`, `Resize`,
`Frame`, `Input`, `Close`, `Closed`, `Context`, `Proxy`, `Id`, `Handle`,
`State`, `Metrics`, `Role`, `InputEvent`, `Code`, and `Handler`.

## Design Target

Application startup remains declarative:

```rust
use surgeist::window;

fn main() -> window::Result<()> {
    window::app(Studio::new())
        .open(
            window::open("main")
                .title("Surgeist Studio")
                .size(window::size(1200, 760))
                .min(window::size(720, 460))
                .theme(window::Theme::Dark),
        )
        .run()
}
```

Window handling is lifecycle-shaped:

```rust
impl window::Handler for Studio {
    fn ready(&mut self, win: &mut window::Ready<'_>) -> window::Result<()> {
        self.surface = Some(self.renderer.attach(win.handle()?, win.metrics())?);
        Ok(())
    }

    fn resize(&mut self, win: &mut window::Resize<'_>) -> window::Result<()> {
        self.surface.resize(win.metrics())?;
        Ok(())
    }

    fn input(&mut self, input: &mut window::Input<'_>) -> window::Result<()> {
        if input.key_pressed(window::Code::Escape) {
            input.close();
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut window::Frame<'_>) -> window::Result<()> {
        self.renderer.render(&mut self.surface, self.scene())?;
        Ok(())
    }
}
```

The app does not match native occlusion, expose, focus, or platform readiness
events to make the first frame appear. The app handles lifecycle intent; the
window layer handles native event-loop facts.

## Principles

- Builders are inert values until passed to `App`, `Context`, `Proxy`, or the
  test host.
- Builder output is inspectable. `Open` lowers to `Descriptor` and
  `Command::Open`; draw helpers queue draw requests through lifecycle scopes,
  `Context`, `Target`, or `Proxy`; target helpers lower to window commands.
- Fluent methods are short and chainable. Variants are expressed through typed
  values instead of verbose method families.
- Logical coordinates are the default. Physical units are explicit in type
  names.
- Startup windows are queued and opened when the native event loop reaches the
  first valid creation point.
- A lifecycle callback receives a scoped object with the current window id,
  state queries, command helpers, draw helpers, and access to the callback
  context.
- Native state facts are exposed as queries such as `is_occluded()`,
  `is_focused()`, `is_visible()`, `is_resizing()`, `size()`, `scale()`, and
  `metrics()`.
- Low-level native event payloads remain an internal adapter, diagnostic, and
  testing contract. The lifecycle DSL is the app-facing front door.

## Front Door

The DSL may live in `dsl.rs` internally and is exported from `lib.rs`:

```rust
pub use dsl::{
    app, controls, open, point, rect, size, App, Close, Closed, Code,
    ControlsBuilder, Frame, Input, Open, Ready, Resize, Scope, Selector, Target,
};
```

The module name is internal. Public users write through `surgeist::window::*`.
`Code` is re-exported from the stable keyboard representation used by
`KeyEvent`, so common key predicates stay concise without introducing a second
key model.

`lib.rs` also re-exports stable contracts from other window modules, including
`Proxy`, `Id`, `Handle`, `Descriptor`, `Command`, `State`, `Metrics`, `Role`,
`InputEvent`, `Error`, and `Result`.

## App Builder

`app(handler)` creates an `App<H>` wrapper around the native `Loop<H>`.

```rust
pub fn app<H>(handler: H) -> App<H>;

pub struct App<H> {
    // private
}
```

Required methods:

```rust
impl<H> App<H> {
    pub fn new(handler: H) -> Self;
    pub fn open(mut self, open: Open) -> Self;
    pub fn with_clipboard(mut self, clipboard: Box<dyn Clipboard>) -> Self;
    pub fn handler(&self) -> &H;
    pub fn handler_mut(&mut self) -> &mut H;
    pub fn into_loop(self) -> Loop<H>;
}

impl<H: Handler + 'static> App<H> {
    pub fn run(self) -> Result<()>;
}
```

Startup semantics:

- `App::open` stores startup window descriptors.
- Startup opens are staged once, after the underlying native event loop is
  active.
- Startup open commands are inserted before the user's first app resume callback
  is delegated.
- Native creation still happens through the normal command drain after the
  callback returns.
- Each created startup window receives the normal `ready` lifecycle callback.
- Startup validation and runtime creation errors use the same typed `Error`
  path as direct `Command::Open`.

Startup identity rules:

- A non-empty `Descriptor::name` is unique among live windows.
- Startup name uniqueness is validated before native creation begins.
- Runtime name uniqueness is validated by `Command::Open`.
- `Ready::id()` and `Ready::state().name` are the canonical app correlation
  points for startup windows.
- Startup windows are root windows in the first implementation. Dialogs, tools,
  and popups open from a live `Context` after the parent `Id` exists.

## Handler

`Handler` is the lifecycle trait implemented by app code. Every method defaults
to no work, except close handling, which accepts the native close request.

```rust
pub trait Handler {
    fn resume(&mut self, cx: &mut Context<'_>) -> Result<()> {
        Ok(())
    }

    fn suspend(&mut self, cx: &mut Context<'_>) -> Result<()> {
        Ok(())
    }

    fn ready(&mut self, win: &mut Ready<'_>) -> Result<()> {
        Ok(())
    }

    fn resize(&mut self, win: &mut Resize<'_>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, input: &mut Input<'_>) -> Result<()> {
        Ok(())
    }

    fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
        close.close();
        Ok(())
    }

    fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        Ok(())
    }

    fn wants_idle(&self) -> bool {
        false
    }

    fn idle(&mut self, cx: &mut Context<'_>) -> Result<()> {
        Ok(())
    }
}
```

Lifecycle meaning:

- `resume` runs when the native app lifecycle reaches a point where windows may
  exist or be recreated.
- `suspend` runs when native surfaces should release volatile resources.
- `ready` runs once per native window after it has a stable `Id`, `State`,
  `Metrics`, and `Handle`.
- `resize` runs after logical size, physical size, or scale changes.
- `input` runs for keyboard, pointer, wheel, IME, modifier, and standard key
  binding input.
- `close` runs for a native close request. The default behavior closes that
  window.
- `closed` runs after a window has been destroyed and removed from the native
  registry.
- `draw` runs when the window layer has a drawable frame for a live window.
- `idle` is opt-in and runs before the event loop waits when `wants_idle()`
  returns `true`.

The native runner may keep internal adapter callbacks that operate on the
normalized native event payload; the public `Handler` trait remains
lifecycle-shaped.

## Lifecycle Scopes

Every live-window lifecycle scope exposes a common window surface. `Ready`,
`Resize`, `Input`, `Close`, and `Frame` implement `Scope`. `Closed` is a
lifecycle callback object, but it is not a live-window scope because native
access has already ended.

```rust
pub trait Scope<'a> {
    fn id(&self) -> Id;
    fn state(&self) -> &State;
    fn metrics(&self) -> &Metrics;
    fn size(&self) -> Size;
    fn scale(&self) -> f64;
    fn is_focused(&self) -> bool;
    fn is_visible(&self) -> bool;
    fn is_occluded(&self) -> bool;
    fn is_resizing(&self) -> bool;
    fn access(&self) -> Result<Ref<'_>>;
    fn handle(&self) -> Result<Handle>;
    fn context_mut(&mut self) -> &mut Context<'a>;
    fn target(&mut self) -> Target<'_>;
    fn draw(&mut self) -> &mut Self;
    fn again(&mut self) -> &mut Self;
    fn at(&mut self, time: std::time::Instant) -> &mut Self;
    fn close(&mut self) -> &mut Self;
    fn exit(&mut self) -> &mut Self;
}
```

Implementation may share this behavior through an internal base type. Public
scopes still have concrete names so method signatures read plainly.

State query rules:

- `is_visible()` returns `State::visible.unwrap_or(true)`.
- `is_occluded()` returns `State::occluded.unwrap_or(false)`.
- `is_focused()` returns `State::focused`.
- `is_resizing()` is an ephemeral native-adapter flag that is true while the
  window layer is coalescing an active resize sequence and false otherwise.
- Callers that need platform uncertainty can inspect `State` directly.

### Ready

`Ready<'_>` is delivered once per live native window.

```rust
pub struct Ready<'a> {
    // private
}
```

`ready` is the app's attachment point for renderer surfaces, accessibility root
state, per-window app state, and initial subscriptions. After `ready` returns,
the window layer automatically schedules the first draw for that window.

### Resize

`Resize<'_>` is delivered when metrics change.

```rust
pub struct Resize<'a> {
    // private
}
```

`resize` is the app's attachment point for resizing renderer surfaces and
updating per-window size-dependent state. After `resize` returns, the window
layer automatically schedules a draw for that window.

The native adapter coalesces repeated resize churn at the request queue
boundary. The app may observe every delivered metrics change, but draw requests
remain per-window coalesced.

### Input

`Input<'_>` wraps one `InputEvent` and the current window scope.

```rust
pub struct Input<'a> {
    // private
}

impl Input<'_> {
    pub fn event(&self) -> &InputEvent;
    pub fn key_pressed(&self, code: Code) -> bool;
    pub fn pointer_pressed(&self, button: PointerButton) -> bool;
    pub fn position(&self) -> Option<Point>;
    pub fn modifiers(&self) -> ModifierState;
}
```

Input helpers cover common predicates. Exact input payload access remains
available through `event()`.

Input callbacks do not draw automatically. App code calls `input.draw()` when
input changes visible state.

### Close

`Close<'_>` is delivered for native close requests.

```rust
pub struct Close<'a> {
    // private
}

impl Close<'_> {
    pub fn cancel(&mut self) -> &mut Self;
    pub fn close(&mut self) -> &mut Self;
}
```

The close scope starts in a pending state. The default `Handler::close`
implementation accepts the close by calling `close.close()`. Apps cancel close
for unsaved-work prompts, background/tray mode, or custom shutdown policy.

`Context::close(id)`, `Target::close()`, and `Proxy::close(id)` request the
cancelable `close` lifecycle for the target window. `Close::close()` accepts the
pending close request and lowers to native destruction. Accepted destruction
produces `closed` after the window leaves the registry.

Closing one window is distinct from exiting the app. The native loop exits when
there are no live windows unless a future background policy explicitly keeps the
app alive.

### Closed

`Closed<'_>` is delivered after native destruction and registry removal.

```rust
pub struct Closed<'a> {
    // private
}

impl Closed<'_> {
    pub fn id(&self) -> Id;
    pub fn state(&self) -> &State;
    pub fn metrics(&self) -> &Metrics;
    pub fn context_mut(&mut self) -> &mut Context<'_>;
    pub fn exit(&mut self) -> &mut Self;
}
```

`closed` is the app's cleanup point for per-window surfaces, caches, and app
state keyed by `Id`. `Closed` exposes final state facts captured before removal,
the closed `Id`, and the current app context. It does not expose live native
access because the window has already been removed from the registry.

### Frame

`Frame<'_>` is delivered when a window should draw.

```rust
pub struct Frame<'a> {
    // private
}

impl Frame<'_> {
    pub fn again(&mut self) -> &mut Self;
    pub fn at(&mut self, time: Instant) -> &mut Self;
}
```

`Frame` gets size, scale, occlusion, handle, and metrics through `Scope`.
`again()` requests another draw on the next loop turn. `at()` requests a delayed
draw.

The window layer delivers `draw` only for live windows that are drawable
according to native lifecycle state. If the native backend reports an occluded or
temporarily unavailable frame, the draw remains pending and is retried when the
window becomes drawable again.

## Draw Scheduling

The app-facing term is `draw`; the native implementation owns the internal
schedule representation.

Scheduling rules:

- Draw requests are coalesced per window.
- `ready` schedules the first draw after the callback succeeds.
- `resize` and scale changes schedule a draw after the callback succeeds.
- A transition from occluded to drawable flushes any pending draw for that
  window.
- Native resume/expose events flush pending draws for affected windows.
- `Frame::again()` schedules the next draw after the current draw callback
  returns.
- Delayed draws wake the event loop no earlier than their requested time.
- Draw requests for destroyed windows are ignored.
- Callback errors discard commands and draw requests created by that callback.

## Context And Target Commands

`Context<'_>` is scoped to one lifecycle callback. It buffers commands, draw
requests, close requests, exit requests, and diagnostics for that callback.

```rust
impl Context<'_> {
    pub fn open(&mut self, open: Open) -> &mut Self;
    pub fn close(&mut self, id: Id) -> &mut Self;
    pub fn draw(&mut self, id: Id) -> &mut Self;
    pub fn again(&mut self, id: Id) -> &mut Self;
    pub fn at(&mut self, id: Id, time: std::time::Instant) -> &mut Self;
    pub fn target(&mut self, id: Id) -> Target<'_>;
    pub fn window_id(&self, name: impl AsRef<str>) -> Option<Id>;
    pub fn state(&self, target: impl Into<Selector>) -> Option<&State>;
    pub fn metrics(&self, id: Id) -> Result<Metrics>;
    pub fn handle(&self, id: Id) -> Result<Handle>;
    pub fn proxy(&self) -> Option<Proxy>;
}
```

`Selector` is the read-only lookup selector used by context helpers:

```rust
pub enum Selector {
    Id(Id),
    Name(String),
}

impl From<Id> for Selector;
impl From<&str> for Selector;
impl From<String> for Selector;
```

`Target<'_>` is a fluent command builder for an existing window:

```rust
impl Target<'_> {
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self;
    pub fn at(&mut self, point: impl Into<Point>) -> &mut Self;
    pub fn size(&mut self, size: impl Into<Size>) -> &mut Self;
    pub fn min(&mut self, size: impl Into<Option<Size>>) -> &mut Self;
    pub fn max(&mut self, size: impl Into<Option<Size>>) -> &mut Self;
    pub fn show(&mut self) -> &mut Self;
    pub fn hide(&mut self) -> &mut Self;
    pub fn resizable(&mut self, resizable: bool) -> &mut Self;
    pub fn controls(&mut self, controls: impl Into<Controls>) -> &mut Self;
    pub fn decorations(&mut self, enabled: bool) -> &mut Self;
    pub fn transparent(&mut self, transparent: bool) -> &mut Self;
    pub fn fullscreen(&mut self, fullscreen: Fullscreen) -> &mut Self;
    pub fn level(&mut self, level: Level) -> &mut Self;
    pub fn theme(&mut self, theme: impl Into<Option<Theme>>) -> &mut Self;
    pub fn cursor(&mut self, cursor: Cursor) -> &mut Self;
    pub fn cursor_grab(&mut self, grab: CursorGrab) -> &mut Self;
    pub fn ime(&mut self, request: ImeRequest) -> &mut Self;
    pub fn attention(&mut self) -> &mut Self;
    pub fn draw(&mut self) -> &mut Self;
    pub fn close(&mut self) -> &mut Self;
}
```

## Window Opening DSL

`open(name)` creates an `Open` builder.

```rust
pub fn open(name: impl Into<String>) -> Open;

pub struct Open {
    // private Descriptor
}
```

Required methods:

```rust
impl Open {
    pub fn unnamed() -> Self;
    pub fn name(mut self, name: impl Into<String>) -> Self;
    pub fn title(mut self, title: impl Into<String>) -> Self;
    pub fn at(mut self, point: impl Into<Point>) -> Self;
    pub fn size(mut self, size: impl Into<Size>) -> Self;
    pub fn min(mut self, size: impl Into<Size>) -> Self;
    pub fn max(mut self, size: impl Into<Size>) -> Self;
    pub fn resizable(mut self, resizable: bool) -> Self;
    pub fn fixed(self) -> Self;
    pub fn controls(mut self, controls: impl Into<Controls>) -> Self;
    pub fn decorations(mut self, enabled: bool) -> Self;
    pub fn transparent(mut self, transparent: bool) -> Self;
    pub fn visible(mut self, visible: bool) -> Self;
    pub fn hidden(self) -> Self;
    pub fn fullscreen(mut self, fullscreen: Fullscreen) -> Self;
    pub fn borderless(self) -> Self;
    pub fn level(mut self, level: Level) -> Self;
    pub fn theme(mut self, theme: impl Into<Option<Theme>>) -> Self;
    pub fn root(self) -> Self;
    pub fn dialog(self, parent: Id) -> Self;
    pub fn modal(mut self, modality: Modality) -> Self;
    pub fn tool(self, parent: Option<Id>) -> Self;
    pub fn popup(self, parent: Id) -> Self;
    pub fn descriptor(&self) -> &Descriptor;
    pub fn into_descriptor(self) -> Descriptor;
    pub fn into_command(self) -> Command;
}
```

`Open` remains a pure builder. Validation happens at startup validation or
command application, so tests can inspect builder output without a native event
loop.

## Geometry And Controls Helpers

```rust
pub fn point(x: impl Into<f64>, y: impl Into<f64>) -> Point;
pub fn size(width: impl Into<f64>, height: impl Into<f64>) -> Size;
pub fn rect(
    x: impl Into<f64>,
    y: impl Into<f64>,
    width: impl Into<f64>,
    height: impl Into<f64>,
) -> Rect;

pub fn controls() -> ControlsBuilder;
```

`ControlsBuilder` methods:

```rust
impl ControlsBuilder {
    pub fn close(mut self, enabled: bool) -> Self;
    pub fn minimize(mut self, enabled: bool) -> Self;
    pub fn maximize(mut self, enabled: bool) -> Self;
    pub fn all(mut self, enabled: bool) -> Self;
    pub fn build(self) -> Controls;
}
```

## Proxy

`Proxy` is the cross-thread wakeup handle. Its fluent methods lower to the same
command and draw-request queues as `Context`.

```rust
impl Proxy {
    pub fn open(&self, open: Open) -> Result<()>;
    pub fn close(&self, id: Id) -> Result<()>;
    pub fn draw(&self, id: Id) -> Result<()>;
    pub fn again(&self, id: Id) -> Result<()>;
    pub fn at(&self, id: Id, time: std::time::Instant) -> Result<()>;
    pub fn send(&self, command: impl Into<Command>) -> Result<()>;
}
```

`Proxy::send` accepts an inspectable `Command` for capabilities that are enabled
through the window contract. For every enabled command capability, `Proxy`
should also expose a named typed method when that capability is useful from
another thread. Delayed cross-thread draw support should be added as a typed
proxy method rather than by exposing the internal scheduler action type.

`Context::proxy()` returns `Some` when the native loop has created a wakeup
proxy for the active run. It may return `None` in pure builder tests, fake hosts,
or pre-run inspection contexts.

## Internal Scheduling

The public DSL does not export `Action` or action helper functions. Internally,
the lifecycle dispatcher may lower draw, close, delayed draw, wait, and exit
requests into a scheduler action representation.

`Command::RequestDraw { id }` is an inspectable bridge for tests, proxies, and
enabled draw capabilities. The command drain converts it into the internal draw
scheduler representation. The lifecycle surface exposes draw through `draw()`
on a scope, `Context`, `Target`, or `Proxy`.

## Error Handling

- Callback commands, draw requests, close requests, and exit requests are
  transactional within the callback that produced them.
- If a lifecycle callback returns `Err`, that callback's buffered commands and
  requests are discarded.
- Fatal command-drain errors stop the loop through the typed `Error` path.
- Advisory command failures produce diagnostics and continue when the requested
  native feature is optional and the target window remains valid.
- Startup validation errors are returned before native creation begins.
- Native creation errors identify the failing descriptor when possible.
- Errors preserve stable `ErrorCode` values for tests and diagnostics.

## Testing Contract

The `testing` module must exercise the lifecycle DSL without opening native
windows. Its public records should describe lifecycle order, commands, draw
requests, diagnostics, and rollback outcomes directly rather than exposing the
internal scheduler action or native event payload types.

Required host capabilities:

- Create named windows and deliver `ready`.
- Drive `resize` and verify automatic draw scheduling.
- Drive input and verify explicit draw scheduling.
- Drive close, cancel close, accepted close, and `closed` cleanup.
- Toggle occlusion and verify pending draws retry when `is_occluded()` becomes
  false.
- Drive suspend/resume and verify pending startup opens are applied once.
- Inspect recorded lifecycle callback order, buffered commands, recorded draw
  requests, diagnostics, and rollback results.
- Validate multi-window draw coalescing and close-one-window behavior.

Stable testing records:

```rust
pub enum Record {
    Resumed,
    Suspended,
    Ready(Id),
    Resized(Id),
    Input(Id),
    CloseRequested(Id),
    Closed(Id),
    Draw(Id),
    Command(Command),
    DrawRequested(Id),
    DelayedDrawRequested { id: Id, time: Instant },
    Diagnostic(ErrorCode),
    RolledBack,
}
```

`Record` is a testing vocabulary. It is not the runtime scheduler or adapter
representation.

Required regression tests:

- Startup `ready` schedules a first draw without app code calling draw.
- A draw requested while occluded is retried when the window becomes drawable.
- Resize updates metrics before `resize` and schedules a draw after `resize`.
- A callback error discards commands and draw requests from that callback.
- Closing one of multiple windows does not exit while other windows remain.
- Default close accepts native close requests.
- `Close::cancel()` keeps the window live.
- `Scope::is_occluded()` reflects current window state without matching raw
  event variants.

## Implementation Notes

- Keep `surgeist-window` independent from renderer crates.
- Keep render-specific recovery in render; keep native lifecycle recovery in
  window.
- The normalized native event payload may remain the internal adapter,
  diagnostic, and testing model.
- The lifecycle dispatcher should be implemented over the existing command,
  registry, scheduler, and context buffers where that remains simpler than a
  rewrite.
- Prefer small internal adapters over widening the public surface.
