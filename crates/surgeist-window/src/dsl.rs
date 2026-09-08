use super::{
    Command, Context, Controls, Cursor, CursorGrab, EventKind, Fullscreen, Handle, Id, ImeRequest,
    InputEvent, Level, Metrics, Modality, Point, Proxy, Rect, Ref, Result, Role, Size, Theme,
    WindowRequest, WindowRequestBuilder, WindowSnapshot, command::Action,
};
use std::time::Instant;

#[must_use]
/// Creates a logical point from horizontal and vertical coordinates.
pub fn point(x: impl Into<f64>, y: impl Into<f64>) -> Point {
    Point {
        x: x.into(),
        y: y.into(),
    }
}

#[must_use]
/// Creates a logical size from width and height.
pub fn size(width: impl Into<f64>, height: impl Into<f64>) -> Size {
    Size {
        width: width.into(),
        height: height.into(),
    }
}

#[must_use]
/// Creates a logical rectangle from its origin and size components.
pub fn rect(
    x: impl Into<f64>,
    y: impl Into<f64>,
    width: impl Into<f64>,
    height: impl Into<f64>,
) -> Rect {
    Rect {
        origin: point(x, y),
        size: size(width, height),
    }
}

#[must_use]
/// Starts an authored window-control builder.
pub fn controls() -> ControlsBuilder {
    ControlsBuilder::default()
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Builder for the authored native control affordances of a window request.
pub struct ControlsBuilder {
    controls: Controls,
}

impl ControlsBuilder {
    #[must_use]
    /// Sets whether the close control is requested.
    pub fn close(mut self, enabled: bool) -> Self {
        self.controls.close = enabled;
        self
    }

    #[must_use]
    /// Sets whether the minimize control is requested.
    pub fn minimize(mut self, enabled: bool) -> Self {
        self.controls.minimize = enabled;
        self
    }

    #[must_use]
    /// Sets whether the maximize control is requested.
    pub fn maximize(mut self, enabled: bool) -> Self {
        self.controls.maximize = enabled;
        self
    }

    #[must_use]
    /// Sets every control request to the same value.
    pub fn all(mut self, enabled: bool) -> Self {
        self.controls = Controls {
            close: enabled,
            minimize: enabled,
            maximize: enabled,
        };
        self
    }

    #[must_use]
    /// Finishes the authored control request.
    pub fn build(self) -> Controls {
        self.controls
    }
}

impl From<ControlsBuilder> for Controls {
    fn from(builder: ControlsBuilder) -> Self {
        builder.build()
    }
}

#[must_use]
/// Starts an authored window request with a caller-supplied identity name.
///
/// The name is application identity intent, not a live [`Id`]. Duplicate-name,
/// role, capability, and native validation occur only when the resulting open
/// command is normalized, planned, and applied.
pub fn open(name: impl Into<String>) -> Open {
    Open::unnamed().name(name)
}

#[derive(Clone, Debug, PartialEq)]
/// Fluent builder for an authored [`WindowRequest`].
///
/// Every method changes only authored request intent and returns the builder for
/// further chaining. It neither creates a window nor observes a native window.
/// When converted into [`Command::Open`] and dispatched, the loop later
/// normalizes dimensions, validates identity, role, and capability requirements,
/// plans the request, and asks the backend to apply it. Those later stages can
/// reject an otherwise buildable request.
pub struct Open {
    builder: WindowRequestBuilder,
}

impl Open {
    #[must_use]
    /// Starts an unnamed request with the model defaults.
    ///
    /// An unnamed request has no authored name for later name-based lookup; it
    /// still receives a runtime [`Id`] if opening succeeds.
    pub fn unnamed() -> Self {
        Self {
            builder: WindowRequestBuilder {
                request: WindowRequest::default(),
            },
        }
    }

    #[must_use]
    /// Starts a named request with the model defaults.
    ///
    /// The name is retained as authored identity intent until dispatch, where
    /// duplicate identity validation takes place.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            builder: WindowRequest::builder(name),
        }
    }

    #[must_use]
    /// Replaces this request's optional authored identity name.
    ///
    /// This does not rename an existing live window and does not validate the
    /// name until the open command is dispatched.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.builder = self.builder.name(name);
        self
    }

    #[must_use]
    /// Sets the title requested when this window is opened.
    ///
    /// The string is authored intent; support for the initial native title is
    /// checked during later planning and application.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.builder = self.builder.title(title);
        self
    }

    #[must_use]
    /// Sets the requested logical outer position.
    ///
    /// Coordinates are logical units. Their validity and the host's ability to
    /// set an outer position are checked when the request is dispatched.
    pub fn position(mut self, point: impl Into<Point>) -> Self {
        self.builder = self.builder.position(point);
        self
    }

    #[must_use]
    /// Sets the requested logical outer position.
    ///
    /// This chaining alias for [`Open::position`] preserves the same authored
    /// intent and deferred validation.
    pub fn at(self, point: impl Into<Point>) -> Self {
        self.position(point)
    }

    #[must_use]
    /// Sets the requested logical inner size.
    ///
    /// Size is expressed in logical units and is normalized and capability
    /// checked only when the request is dispatched.
    pub fn inner_size(mut self, size: impl Into<Size>) -> Self {
        self.builder = self.builder.inner_size(size);
        self
    }

    #[must_use]
    /// Sets the requested logical inner size.
    ///
    /// This chaining alias for [`Open::inner_size`] retains deferred
    /// normalization and host validation.
    pub fn size(self, size: impl Into<Size>) -> Self {
        self.inner_size(size)
    }

    #[must_use]
    /// Sets the requested logical minimum inner size.
    ///
    /// The bound uses logical units. Unlike [`Open::min`], this method always
    /// authors a bound rather than clearing one.
    pub fn min_inner_size(mut self, size: impl Into<Size>) -> Self {
        self.builder = self.builder.min_inner_size(size);
        self
    }

    #[must_use]
    /// Sets or clears the requested logical minimum inner-size bound.
    ///
    /// `Some` uses logical units; `None` removes an earlier explicit minimum.
    /// Bound consistency and platform support are deferred until dispatch.
    pub fn min(mut self, size: impl Into<Option<Size>>) -> Self {
        self.builder.request.set_min_inner_size(size.into());
        self
    }

    #[must_use]
    /// Sets the requested logical maximum inner size.
    ///
    /// The bound uses logical units. Unlike [`Open::max`], this method always
    /// authors a bound rather than clearing one.
    pub fn max_inner_size(mut self, size: impl Into<Size>) -> Self {
        self.builder = self.builder.max_inner_size(size);
        self
    }

    #[must_use]
    /// Sets or clears the requested logical maximum inner-size bound.
    ///
    /// `Some` uses logical units; `None` removes an earlier explicit maximum.
    /// Bound consistency and platform support are deferred until dispatch.
    pub fn max(mut self, size: impl Into<Option<Size>>) -> Self {
        self.builder.request.set_max_inner_size(size.into());
        self
    }

    #[must_use]
    /// Sets whether the request asks the host to permit user resizing.
    ///
    /// This is initial intent, not observed resizability, and host support is
    /// checked during later planning.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.builder = self.builder.resizable(resizable);
        self
    }

    #[must_use]
    /// Sets the initial user-resizability request to `false`.
    pub fn fixed(mut self) -> Self {
        self.builder = self.builder.fixed();
        self
    }

    #[must_use]
    /// Sets the requested native close, minimize, and maximize controls.
    ///
    /// These are initial affordance requests; unsupported controls are rejected
    /// during later capability validation rather than by this builder.
    pub fn controls(mut self, controls: impl Into<Controls>) -> Self {
        self.builder = self.builder.controls(controls);
        self
    }

    #[must_use]
    /// Sets whether the request asks for native decorations.
    ///
    /// Decoration support is capability-sensitive and checked when dispatched.
    pub fn decorations(mut self, enabled: bool) -> Self {
        self.builder = self.builder.decorations(enabled);
        self
    }

    #[must_use]
    /// Sets whether the request asks for an initially transparent native window.
    ///
    /// Transparency is initial intent and may be rejected by later capability or
    /// backend application checks.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.builder = self.builder.transparent(transparent);
        self
    }

    #[must_use]
    /// Sets whether the request asks to be initially visible.
    ///
    /// This does not change an existing window and does not report observed
    /// visibility; the created snapshot reports the state after opening.
    pub fn visible(mut self, visible: bool) -> Self {
        self.builder = self.builder.visible(visible);
        self
    }

    #[must_use]
    /// Sets the initial visibility request to `false`.
    pub fn hidden(mut self) -> Self {
        self.builder = self.builder.hidden();
        self
    }

    #[must_use]
    /// Sets the requested initial fullscreen mode.
    ///
    /// Fullscreen mode support is host-sensitive and is checked when the open is
    /// planned and applied.
    pub fn fullscreen(mut self, fullscreen: impl Into<Fullscreen>) -> Self {
        self.builder = self.builder.fullscreen(fullscreen);
        self
    }

    #[must_use]
    /// Sets the requested initial fullscreen mode to borderless.
    pub fn borderless(mut self) -> Self {
        self.builder = self.builder.borderless();
        self
    }

    #[must_use]
    /// Sets the requested native window level.
    ///
    /// Level support is host-sensitive and is checked when the open is planned.
    pub fn level(mut self, level: Level) -> Self {
        self.builder = self.builder.level(level);
        self
    }

    #[must_use]
    /// Sets or clears the requested explicit theme.
    ///
    /// `Some` requests that theme; `None` returns theme selection to the host.
    /// Explicit-theme and reset support are evaluated during dispatch.
    pub fn theme(mut self, theme: impl Into<Option<Theme>>) -> Self {
        self.builder = self.builder.theme(theme);
        self
    }

    #[must_use]
    /// Sets the requested window role and any role-specific relationship data.
    ///
    /// Parent existence, role support, and role invariants are validated later
    /// when the open is normalized and planned.
    pub fn role(mut self, role: Role) -> Self {
        self.builder = self.builder.role(role);
        self
    }

    #[must_use]
    /// Sets the requested role to a top-level root window.
    pub fn root(mut self) -> Self {
        self.builder = self.builder.root();
        self
    }

    #[must_use]
    /// Sets the requested role to a dialog with `parent` as its parent identity.
    ///
    /// The parent is resolved and validated only during dispatch.
    pub fn dialog(mut self, parent: Id) -> Self {
        self.builder = self.builder.dialog(parent);
        self
    }

    /// Sets the requested role to a dialog with `parent` and `modality`.
    ///
    /// Parent liveness, modal semantics, and host support are validated during
    /// later normalization and planning.
    ///
    /// # Migration
    ///
    /// Replace `.dialog(parent).modal(modality)` with `.modal(parent, modality)`.
    #[must_use]
    pub fn modal(mut self, parent: Id, modality: Modality) -> Self {
        self.builder = self.builder.modal(parent, modality);
        self
    }

    #[must_use]
    /// Sets the requested role to a tool with an optional parent identity.
    ///
    /// `None` authors an unparented tool; a supplied parent is resolved later.
    pub fn tool(mut self, parent: Option<Id>) -> Self {
        self.builder = self.builder.tool(parent);
        self
    }

    #[must_use]
    /// Sets the requested role to a popup with `parent` as its parent identity.
    ///
    /// The required parent is not checked until the open is dispatched.
    pub fn popup(mut self, parent: Id) -> Self {
        self.builder = self.builder.popup(parent);
        self
    }

    #[must_use]
    /// Borrows the request exactly as authored so far.
    ///
    /// The returned request is not normalized and is not an observed native
    /// snapshot.
    pub fn request(&self) -> &WindowRequest {
        &self.builder.request
    }

    #[must_use]
    /// Finishes this builder as an authored request without normalization.
    ///
    /// Dispatch still performs all state, capability, and backend validation.
    pub fn into_request(self) -> WindowRequest {
        self.builder.build()
    }

    #[must_use]
    /// Finishes this builder as an authored request without normalization.
    pub fn build(self) -> WindowRequest {
        self.into_request()
    }

    #[must_use]
    /// Converts this authored request into [`Command::Open`].
    ///
    /// Conversion does not dispatch the command or validate it against a live
    /// registry or capability report.
    pub fn into_command(self) -> Command {
        Command::Open {
            request: self.into_request(),
        }
    }
}

impl From<Open> for WindowRequest {
    fn from(open: Open) -> Self {
        open.into_request()
    }
}

impl From<Open> for Command {
    fn from(open: Open) -> Self {
        open.into_command()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Identifies a window by stable runtime id or authored name for callback lookup.
pub enum Selector {
    /// Selects the current live window with this runtime identity.
    Id(Id),
    /// Selects the current live window with this authored name.
    Name(String),
}

impl From<Id> for Selector {
    fn from(id: Id) -> Self {
        Self::Id(id)
    }
}

impl From<&str> for Selector {
    fn from(name: &str) -> Self {
        Self::Name(name.to_owned())
    }
}

impl From<String> for Selector {
    fn from(name: String) -> Self {
        Self::Name(name)
    }
}

/// Callback-local command builder for one fixed window [`Id`].
///
/// Each method appends intent to the enclosing callback transaction and returns
/// this target for ordered chaining. It changes neither the scope's observed
/// snapshot nor native state while the callback is running. The queued commands
/// commit only if that callback returns `Ok`; normalization, live-target and
/// capability planning, and backend application happen afterward and can still
/// fail terminally. A callback error discards this queued work.
pub struct Target<'a> {
    commands: &'a mut Vec<Command>,
    actions: &'a mut Vec<Action>,
    action: &'a mut Action,
    id: Id,
}

impl<'a> Target<'a> {
    pub(crate) fn new(
        id: Id,
        commands: &'a mut Vec<Command>,
        actions: &'a mut Vec<Action>,
        action: &'a mut Action,
    ) -> Self {
        Self {
            commands,
            actions,
            action,
            id,
        }
    }

    fn send(&mut self, command: Command) {
        self.commands.push(command);
    }

    fn request(&mut self, action: Action) {
        self.actions.push(action);
        *self.action = super::context::resolve_actions(self.actions);
    }

    /// Queues a title string for this fixed target.
    ///
    /// Title-operation support and target liveness are checked after successful
    /// callback completion.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.send(Command::SetTitle {
            id: self.id,
            title: title.into(),
        });
        self
    }

    /// Queues a logical outer-position update for this fixed target.
    ///
    /// `point` is in logical units; its normalization and outer-position
    /// capability check are deferred to planning.
    pub fn at(&mut self, point: impl Into<Point>) -> &mut Self {
        self.send(Command::SetPosition {
            id: self.id,
            position: point.into(),
        });
        self
    }

    /// Queues a visibility request for this fixed target.
    ///
    /// This is not an immediate or observed visibility change; support is
    /// checked after callback completion.
    pub fn visible(&mut self, visible: bool) -> &mut Self {
        self.send(Command::SetVisible {
            id: self.id,
            visible,
        });
        self
    }

    /// Queues a `visible(true)` request for this fixed target.
    pub fn show(&mut self) -> &mut Self {
        self.visible(true)
    }

    /// Queues a `visible(false)` request for this fixed target.
    pub fn hide(&mut self) -> &mut Self {
        self.visible(false)
    }

    /// Queues a user-resizability request for this fixed target.
    ///
    /// The host validates support and applies any resulting observed state later.
    pub fn resizable(&mut self, resizable: bool) -> &mut Self {
        self.send(Command::SetResizable {
            id: self.id,
            resizable,
        });
        self
    }

    /// Queues requested native close, minimize, and maximize controls.
    ///
    /// Individual control support is capability-sensitive and checked later.
    pub fn controls(&mut self, controls: impl Into<Controls>) -> &mut Self {
        self.send(Command::SetControls {
            id: self.id,
            controls: controls.into(),
        });
        self
    }

    /// Queues a native-decoration request for this fixed target.
    ///
    /// Decoration support is checked during post-callback planning.
    pub fn decorations(&mut self, enabled: bool) -> &mut Self {
        self.send(Command::SetDecorations {
            id: self.id,
            decorations: enabled,
        });
        self
    }

    /// Queues a transparency request for this fixed target.
    ///
    /// Transparency support is checked during post-callback planning.
    pub fn transparent(&mut self, transparent: bool) -> &mut Self {
        self.send(Command::SetTransparent {
            id: self.id,
            transparent,
        });
        self
    }

    /// Queues a logical inner-size request for this fixed target.
    ///
    /// Size is normalized, bounded against retained constraints, and capability
    /// checked after the callback succeeds.
    pub fn size(&mut self, size: impl Into<Size>) -> &mut Self {
        self.send(Command::SetInnerSize {
            id: self.id,
            size: size.into(),
        });
        self
    }

    /// Queues a logical minimum-inner-size bound update for this fixed target.
    ///
    /// `Some` sets a bound and `None` clears it. Bounds are normalized and
    /// checked against the maximum and host support only after callback success.
    pub fn min(&mut self, size: impl Into<Option<Size>>) -> &mut Self {
        self.send(Command::SetMinInnerSize {
            id: self.id,
            size: size.into(),
        });
        self
    }

    /// Queues a logical maximum-inner-size bound update for this fixed target.
    ///
    /// `Some` sets a bound and `None` clears it. Bounds are normalized and
    /// checked against the minimum and host support only after callback success.
    pub fn max(&mut self, size: impl Into<Option<Size>>) -> &mut Self {
        self.send(Command::SetMaxInnerSize {
            id: self.id,
            size: size.into(),
        });
        self
    }

    /// Queues a fullscreen-mode request for this fixed target.
    ///
    /// The selected mode is capability-sensitive and is validated during later
    /// planning and backend application.
    pub fn fullscreen(&mut self, fullscreen: Fullscreen) -> &mut Self {
        self.send(Command::SetFullscreen {
            id: self.id,
            fullscreen,
        });
        self
    }

    /// Queues a native window-level request for this fixed target.
    ///
    /// Window-level support is capability-sensitive and checked later.
    pub fn level(&mut self, level: Level) -> &mut Self {
        self.send(Command::SetLevel { id: self.id, level });
        self
    }

    /// Queues an explicit-theme request or host-theme reset for this target.
    ///
    /// `Some` requests that theme; `None` requests reset to host selection. The
    /// two operations have separate capability checks during planning.
    pub fn theme(&mut self, theme: impl Into<Option<Theme>>) -> &mut Self {
        self.send(Command::SetTheme {
            id: self.id,
            theme: theme.into(),
        });
        self
    }

    /// Queues a cursor update for this fixed target.
    ///
    /// Custom and standard cursor support is capability-sensitive and checked
    /// after the callback succeeds.
    pub fn cursor(&mut self, cursor: Cursor) -> &mut Self {
        self.send(Command::SetCursor {
            id: self.id,
            cursor,
        });
        self
    }

    /// Queues a cursor-grab request for this fixed target.
    ///
    /// The requested grab mode is capability-sensitive and is validated later.
    pub fn cursor_grab(&mut self, grab: CursorGrab) -> &mut Self {
        self.send(Command::SetCursorGrab { id: self.id, grab });
        self
    }

    /// Queues an IME request for this fixed target.
    ///
    /// IME fields are normalized and their individual capability requirements
    /// are checked after callback success.
    pub fn ime(&mut self, request: ImeRequest) -> &mut Self {
        self.send(Command::SetIme {
            id: self.id,
            request,
        });
        self
    }

    /// Queues a native user-attention request for this fixed target.
    ///
    /// User-attention support and target liveness are checked during planning.
    pub fn attention(&mut self) -> &mut Self {
        self.send(Command::RequestUserAttention { id: self.id });
        self
    }

    /// Queues an `Action::DrawNext` request for this fixed target.
    ///
    /// Callback success commits the request. The lifecycle pump then checks the
    /// target lifecycle and schedules one later native redraw; draw requests do
    /// not have capability planning.
    pub fn draw(&mut self) -> &mut Self {
        self.request(Action::DrawNext(self.id));
        self
    }

    /// Queues the specialized close-request action for this fixed target.
    ///
    /// It does not close immediately. The later [`crate::Handler::close`]
    /// callback makes the final accept-or-cancel decision.
    pub fn close(&mut self) -> &mut Self {
        self.request(Action::CloseRequested(self.id));
        self
    }
}

/// Callback scope for one scheduled draw of a live window.
///
/// Its snapshot and live registry access describe state observed when the draw
/// was dispatched. Commands and actions requested here are deferred until a
/// successful draw callback completes.
pub struct Frame<'a> {
    id: Id,
    context: Context<'a>,
}

/// Callback scope delivered after a live window's successful `Created` event.
///
/// Its observed state is committed before delivery. A successful ready callback
/// also receives default next-frame scheduling from the lifecycle pump.
pub struct Ready<'a> {
    id: Id,
    context: Context<'a>,
}

/// Callback scope for observed resize or scale-factor changes.
///
/// Its snapshot already contains the committed metrics transition. A successful
/// callback also receives default next-frame scheduling from the lifecycle pump.
pub struct Resize<'a> {
    id: Id,
    context: Context<'a>,
}

/// Callback scope for normalized input delivered to a live window.
///
/// The payload is routed from [`EventKind::Input`] rather than
/// [`crate::Handler::event`]. State access observes the target at dispatch time;
/// commands and actions remain callback-local until successful completion.
pub struct Input<'a> {
    event: InputEvent,
    context: Context<'a>,
}

/// Callback scope for a normalized non-specialized host event.
///
/// This is the route for retained [`EventKind`] values other than input and
/// metric changes. State-changing producer transitions are committed before the
/// callback; [`crate::Handler::close`] and [`crate::Handler::closed`] remain
/// specialized lifecycle routes.
pub struct Event<'a> {
    event: EventKind,
    context: Context<'a>,
}

/// Callback scope that records the final decision for a native close request.
///
/// A new close scope starts cancelled; [`Close::close`] accepts and
/// [`Close::cancel`] rejects the request.
pub struct Close<'a> {
    id: Id,
    context: Context<'a>,
    decision: CloseDecision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloseDecision {
    Cancel,
    Accept,
}

/// Callback scope delivered once after closing has completed.
///
/// It retains the final snapshot but has no live-window target.
pub struct Closed<'a> {
    state: WindowSnapshot,
    context: Context<'a>,
}

/// Common live-window access and deferred actions for callback scopes.
///
/// `Ready`, `Resize`, `Input`, `Close`, and `Frame` implement this trait; each
/// exposes the committed snapshot available for its own lifecycle phase. Read
/// accessors return observed state, not authored intent. `access` and `handle`
/// can fail with [`crate::ErrorCode::HandleUnavailable`] if the target ceases to
/// be live before the lookup. `context_mut` and `target` expose callback-local
/// work: their commands and actions commit only when the enclosing callback
/// returns `Ok`, and subsequent normalization, planning, capability checks, or
/// backend application may still fail terminally.
pub trait Scope<'a> {
    /// Returns this scope's fixed live-window identity.
    fn id(&self) -> Id;
    /// Returns the committed snapshot available in this lifecycle phase.
    fn state(&self) -> &WindowSnapshot;

    #[must_use]
    /// Returns metrics from this scope's committed snapshot.
    fn metrics(&self) -> &Metrics {
        self.state().metrics()
    }

    #[must_use]
    /// Returns the observed logical inner size from [`Scope::metrics`].
    fn size(&self) -> Size {
        self.metrics().logical_size()
    }

    #[must_use]
    /// Returns observed physical pixels per logical unit from [`Scope::metrics`].
    fn scale(&self) -> f64 {
        self.metrics().scale_factor()
    }

    #[must_use]
    /// Returns whether the committed snapshot records focus.
    fn is_focused(&self) -> bool {
        self.state().is_focused()
    }

    #[must_use]
    /// Returns whether the committed snapshot records visibility.
    fn is_visible(&self) -> bool {
        self.state().is_visible()
    }

    #[must_use]
    /// Returns whether the committed snapshot records occlusion.
    fn is_occluded(&self) -> bool {
        self.state().is_occluded()
    }

    /// Borrows registry access for this scope's live target.
    ///
    /// Returns [`crate::ErrorCode::HandleUnavailable`] if it is no longer live.
    fn access(&self) -> Result<Ref<'_>>;
    /// Clones a native-handle lease for this scope's live target.
    ///
    /// Returns [`crate::ErrorCode::HandleUnavailable`] when no live access is
    /// available; an acquired lease may outlive live-registry access while the
    /// native resource remains owned during closing.
    fn handle(&self) -> Result<Handle>;
    /// Returns callback-local context for global queued work and immediate services.
    ///
    /// Clipboard I/O through this context is immediate and is not rolled back;
    /// its commands and actions otherwise share this callback's transaction.
    fn context_mut(&mut self) -> &mut Context<'a>;
    /// Returns a callback-local target builder for this scope's fixed identity.
    ///
    /// It queues commands; it does not mutate the observed snapshot immediately.
    fn target(&mut self) -> Target<'_>;
    /// Queues a next-frame draw for this scope's target.
    ///
    /// This is `Action::DrawNext`: the scheduler requests a later redraw only
    /// after successful callback completion.
    fn draw(&mut self) -> &mut Self;
    /// Queues an immediate draw for this scope's target.
    ///
    /// This is `Action::DrawNow`, distinct from next-frame scheduling.
    fn again(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        let id = self.id();
        self.context_mut().again(id);
        self
    }
    /// Queues a draw at absolute `time` for this scope's target.
    ///
    /// This is `Action::DrawAt`; the deadline is consumed only after callback
    /// success.
    fn at(&mut self, time: Instant) -> &mut Self
    where
        Self: Sized,
    {
        let id = self.id();
        self.context_mut().at(id, time);
        self
    }
    /// Queues a specialized close request for this scope's target.
    ///
    /// It is not an immediate destroy; the later close callback makes the final
    /// acceptance decision.
    fn close(&mut self) -> &mut Self;
    /// Queues successful terminal exit after this callback succeeds.
    ///
    /// Once consumed, exit suppresses later callback delivery and closes ingress.
    fn exit(&mut self) -> &mut Self;
}

impl<'a> Ready<'a> {
    pub(crate) fn new(id: Id, context: Context<'a>) -> Self {
        Self { id, context }
    }

    #[must_use]
    /// Returns the live window identity for this ready callback.
    pub fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    /// Returns the committed live snapshot observed at ready delivery.
    pub fn state(&self) -> &WindowSnapshot {
        self.context
            .state(self.id)
            .expect("ready scope always targets a live window")
    }

    /// Returns the ready callback's transaction-local context.
    ///
    /// Its queued work commits only if `Handler::ready` returns `Ok`.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Returns a deferred-command target for the ready window.
    pub fn window(&mut self) -> Target<'_> {
        self.context.window(self.id)
    }

    /// Borrows registry access for the ready window.
    ///
    /// Returns `HandleUnavailable` if its live entry cannot be found.
    pub fn access(&self) -> Result<Ref<'_>> {
        self.context.access(self.id)
    }

    /// Clones a native-handle lease for the ready window.
    ///
    /// Returns `HandleUnavailable` when live access has ended.
    pub fn handle(&self) -> Result<Handle> {
        self.context.handle(self.id)
    }

    /// Returns metrics from the ready callback's committed snapshot.
    pub fn metrics(&self) -> &Metrics {
        self.state().metrics()
    }

    /// Adds a next-frame draw request to ready's default successful scheduling.
    pub fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id);
        self
    }

    /// Queues an immediate `Action::DrawNow` request for the ready window.
    pub fn again(&mut self) -> &mut Self {
        self.context.again(self.id);
        self
    }

    /// Queues an `Action::DrawAt` deadline for the ready window.
    pub fn at(&mut self, time: Instant) -> &mut Self {
        self.context.at(self.id, time);
        self
    }

    /// Queues a specialized close request for the ready window.
    ///
    /// The later close callback decides whether teardown begins.
    pub fn close(&mut self) -> &mut Self {
        self.context.close(self.id);
        self
    }

    /// Queues terminal exit when this ready callback succeeds.
    pub fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }
}

impl<'a> Resize<'a> {
    pub(crate) fn new(id: Id, context: Context<'a>) -> Self {
        Self { id, context }
    }

    #[must_use]
    /// Returns the live window identity whose metrics changed.
    pub fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    /// Returns the snapshot after the committed metrics transition.
    pub fn state(&self) -> &WindowSnapshot {
        self.context
            .state(self.id)
            .expect("resize scope always targets a live window")
    }

    #[must_use]
    /// Returns metrics after the committed resize or scale transition.
    pub fn metrics(&self) -> &Metrics {
        self.state().metrics()
    }

    #[must_use]
    /// Returns the post-transition logical inner size.
    pub fn size(&self) -> Size {
        self.metrics().logical_size()
    }

    #[must_use]
    /// Returns the post-transition physical-pixels-per-logical-unit scale factor.
    pub fn scale(&self) -> f64 {
        self.metrics().scale_factor()
    }

    /// Returns the resize callback's transaction-local context.
    ///
    /// Its queued work commits only if `Handler::resize` returns `Ok`.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Borrows registry access for the resized window, or returns `HandleUnavailable`.
    pub fn access(&self) -> Result<Ref<'_>> {
        self.context.access(self.id)
    }

    /// Clones a native-handle lease for the resized window, or returns `HandleUnavailable`.
    pub fn handle(&self) -> Result<Handle> {
        self.context.handle(self.id)
    }

    /// Returns a deferred-command target for the resized window.
    pub fn window(&mut self) -> Target<'_> {
        self.context.window(self.id)
    }

    /// Adds a next-frame draw request to resize's default successful scheduling.
    pub fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id);
        self
    }

    /// Queues an immediate `Action::DrawNow` request for the resized window.
    pub fn again(&mut self) -> &mut Self {
        self.context.again(self.id);
        self
    }

    /// Queues an `Action::DrawAt` deadline for the resized window.
    pub fn at(&mut self, time: Instant) -> &mut Self {
        self.context.at(self.id, time);
        self
    }

    /// Queues a specialized close request for the resized window.
    pub fn close(&mut self) -> &mut Self {
        self.context.close(self.id);
        self
    }

    /// Queues terminal exit when this resize callback succeeds.
    pub fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }
}

impl<'a> Input<'a> {
    pub(crate) fn new(event: InputEvent, context: Context<'a>) -> Self {
        Self { event, context }
    }

    #[must_use]
    /// Returns the live window identity that received this input payload.
    pub fn id(&self) -> Id {
        self.event.id()
    }

    #[must_use]
    /// Returns the normalized input payload routed to this callback.
    pub fn event(&self) -> &InputEvent {
        &self.event
    }

    #[must_use]
    /// Returns the live snapshot observed when this input was dispatched.
    ///
    /// Input does not itself author a state transition; this is the then-current
    /// committed snapshot for the target.
    pub fn state(&self) -> &WindowSnapshot {
        self.context
            .state(self.id())
            .expect("input scope always targets a live window")
    }

    #[must_use]
    /// Returns metrics from the input target's observed snapshot.
    pub fn metrics(&self) -> &Metrics {
        self.state().metrics()
    }

    #[must_use]
    /// Returns the input target's observed logical inner size.
    pub fn size(&self) -> Size {
        self.metrics().logical_size()
    }

    #[must_use]
    /// Returns the input target's observed physical-pixels-per-logical-unit scale.
    pub fn scale(&self) -> f64 {
        self.metrics().scale_factor()
    }

    #[must_use]
    /// Returns whether this payload is a pressed key with the given physical code.
    pub fn key_pressed(&self, code: keyboard_types::Code) -> bool {
        matches!(
            &self.event,
            InputEvent::Key(event)
                if event.physical_key == code && event.state == super::KeyState::Pressed
        )
    }

    #[must_use]
    /// Returns whether this payload is a press for the given pointer button.
    pub fn pointer_pressed(&self, button: super::PointerButton) -> bool {
        matches!(
            &self.event,
            InputEvent::Pointer(event)
                if event.button == Some(button) && event.phase == super::PointerPhase::Pressed
        )
    }

    #[must_use]
    /// Returns the pointer or wheel logical position carried by this input, if any.
    pub fn position(&self) -> Option<Point> {
        match &self.event {
            InputEvent::Pointer(event) => event.position,
            InputEvent::Wheel(event) => event.position,
            _ => None,
        }
    }

    #[must_use]
    /// Returns payload modifiers, or the empty state for event kinds without them.
    pub fn modifiers(&self) -> super::ModifierState {
        match &self.event {
            InputEvent::Pointer(event) => event.modifiers,
            InputEvent::Wheel(event) => event.modifiers,
            InputEvent::Key(event) => event.modifiers,
            InputEvent::Modifiers { modifiers, .. } => *modifiers,
            _ => super::ModifierState::default(),
        }
    }

    /// Returns the input callback's transaction-local context.
    ///
    /// Its queued work commits only if `Handler::input` returns `Ok`.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Returns a deferred-command target for the input window.
    pub fn window(&mut self) -> Target<'_> {
        self.context.window(self.id())
    }

    /// Queues an `Action::DrawNext` request for the input window.
    pub fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id());
        self
    }

    /// Queues an immediate `Action::DrawNow` request for the input window.
    pub fn again(&mut self) -> &mut Self {
        self.context.again(self.id());
        self
    }

    /// Queues an `Action::DrawAt` deadline for the input window.
    pub fn at(&mut self, time: Instant) -> &mut Self {
        self.context.at(self.id(), time);
        self
    }

    /// Queues a specialized close request for the input window.
    pub fn close(&mut self) -> &mut Self {
        self.context.request(Action::CloseRequested(self.id()));
        self
    }

    /// Queues terminal exit when this input callback succeeds.
    pub fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }
}

impl<'a> Event<'a> {
    pub(crate) fn new(event: EventKind, context: Context<'a>) -> Self {
        Self { event, context }
    }

    #[must_use]
    /// Returns the identity carried by this normalized event.
    pub fn id(&self) -> Id {
        self.event.id()
    }

    #[must_use]
    /// Returns the normalized non-specialized event payload.
    pub fn event(&self) -> &EventKind {
        &self.event
    }

    #[must_use]
    /// Returns the event target's current committed snapshot when it remains live.
    ///
    /// `None` means the target is no longer in the live registry, so no scope
    /// target can be built for it.
    pub fn state(&self) -> Option<&WindowSnapshot> {
        self.context.state(self.id())
    }

    /// Returns the event callback's transaction-local context.
    ///
    /// Its queued work commits only if `Handler::event` returns `Ok`.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Returns a deferred-command target only while this event target is live.
    ///
    /// `None` mirrors [`Event::state`] when lookup finds no live entry.
    pub fn window(&mut self) -> Option<Target<'_>> {
        self.state()?;
        Some(self.context.window(self.id()))
    }

    /// Queues an `Action::DrawNext` request for the event target.
    pub fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id());
        self
    }

    /// Queues an immediate `Action::DrawNow` request for the event target.
    pub fn again(&mut self) -> &mut Self {
        self.context.again(self.id());
        self
    }

    /// Queues an `Action::DrawAt` deadline for the event target.
    pub fn at(&mut self, time: Instant) -> &mut Self {
        self.context.at(self.id(), time);
        self
    }

    /// Queues a specialized close request for the event target.
    pub fn close(&mut self) -> &mut Self {
        self.context.request(Action::CloseRequested(self.id()));
        self
    }

    /// Queues terminal exit when this event callback succeeds.
    pub fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }
}

impl<'a> Close<'a> {
    pub(crate) fn new(id: Id, context: Context<'a>) -> Self {
        Self {
            id,
            context,
            decision: CloseDecision::Cancel,
        }
    }

    #[must_use]
    /// Returns the live identity whose close request is being decided.
    pub fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    /// Returns the still-live committed snapshot available while deciding close.
    pub fn state(&self) -> &WindowSnapshot {
        self.context
            .state(self.id)
            .expect("close scope always targets a live window")
    }

    #[must_use]
    /// Returns metrics from the close-decision snapshot.
    pub fn metrics(&self) -> &Metrics {
        self.state().metrics()
    }

    /// Returns the close callback's transaction-local context.
    ///
    /// Its queued work commits only if `Handler::close` returns `Ok`; the close
    /// decision is also consumed only after that successful return.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Queues an `Action::DrawNext` request while the window remains live.
    pub fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id);
        self
    }

    /// Queues an immediate `Action::DrawNow` request while the window remains live.
    pub fn again(&mut self) -> &mut Self {
        self.context.again(self.id);
        self
    }

    /// Queues an `Action::DrawAt` deadline while the window remains live.
    pub fn at(&mut self, time: Instant) -> &mut Self {
        self.context.at(self.id, time);
        self
    }

    /// Records acceptance of this close request.
    ///
    /// The scope begins cancelled. The last call to [`Close::close`] or
    /// [`Close::cancel`] before a successful callback return is the final
    /// decision; accepting then starts shared teardown and eventually one
    /// [`crate::Handler::closed`] callback.
    pub fn close(&mut self) -> &mut Self {
        self.decision = CloseDecision::Accept;
        self
    }

    /// Records cancellation of this close request.
    ///
    /// The scope begins cancelled. A later [`Close::close`] may replace this
    /// decision before callback completion; cancellation leaves the window live.
    pub fn cancel(&mut self) -> &mut Self {
        self.decision = CloseDecision::Cancel;
        self
    }

    #[must_use]
    /// Returns whether the currently recorded final decision is acceptance.
    pub fn is_accepted(&self) -> bool {
        self.decision == CloseDecision::Accept
    }
}

impl<'a> Closed<'a> {
    pub(crate) fn new(state: WindowSnapshot, context: Context<'a>) -> Self {
        Self { state, context }
    }

    #[must_use]
    /// Returns the identity retained by the terminal snapshot.
    pub fn id(&self) -> Id {
        self.state.id()
    }

    #[must_use]
    /// Returns the terminal snapshot captured before the window was removed.
    ///
    /// This is historical state: the window has no live registry entry, target,
    /// access, or handle during `Handler::closed`.
    pub fn state(&self) -> &WindowSnapshot {
        &self.state
    }

    #[must_use]
    /// Returns metrics from the terminal snapshot.
    pub fn metrics(&self) -> &Metrics {
        self.state.metrics()
    }

    /// Returns the closed callback's transaction-local global context.
    ///
    /// It can queue global work only; its commands and actions commit only when
    /// `Handler::closed` returns `Ok`.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Queues terminal exit when this final callback succeeds.
    pub fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }
}

impl<'a> Frame<'a> {
    pub(crate) fn new(id: Id, context: Context<'a>) -> Self {
        Self { id, context }
    }

    #[must_use]
    /// Returns the live identity scheduled for this draw callback.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns the draw callback's transaction-local context.
    ///
    /// Its queued work commits only if `Handler::draw` returns `Ok`.
    pub fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    /// Returns a deferred-command target for the draw window.
    pub fn window(&mut self) -> Target<'_> {
        self.context.window(self.id)
    }

    /// Borrows registry access for the draw window, or returns `HandleUnavailable`.
    pub fn access(&self) -> Result<Ref<'_>> {
        self.context.access(self.id)
    }

    /// Clones a native-handle lease for the draw window, or returns `HandleUnavailable`.
    pub fn handle(&self) -> Result<Handle> {
        self.context.handle(self.id)
    }

    #[must_use]
    /// Returns the live committed snapshot observed for this draw.
    pub fn state(&self) -> &WindowSnapshot {
        self.context
            .state(self.id)
            .expect("frame scope always targets a live window")
    }

    /// Returns metrics from the draw callback's observed snapshot.
    pub fn metrics(&self) -> &Metrics {
        self.state().metrics()
    }

    #[must_use]
    /// Returns the draw window's observed logical inner size.
    pub fn size(&self) -> Size {
        self.context
            .state(self.id)
            .expect("frame scope always targets a live window")
            .metrics()
            .logical_size()
    }

    #[must_use]
    /// Returns the draw window's observed physical-pixels-per-logical-unit scale.
    pub fn scale(&self) -> f64 {
        self.context
            .state(self.id)
            .expect("frame scope always targets a live window")
            .metrics()
            .scale_factor()
    }

    /// Queues an `Action::DrawNext` request for the same window.
    pub fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id);
        self
    }

    /// Queues an immediate `Action::DrawNow` request for the same window.
    pub fn again(&mut self) -> &mut Self {
        self.context.again(self.id);
        self
    }

    /// Queues an `Action::DrawAt` deadline for the same window.
    pub fn at(&mut self, time: Instant) -> &mut Self {
        self.context.at(self.id, time);
        self
    }

    /// Queues a specialized close request for the draw window.
    pub fn close(&mut self) -> &mut Self {
        self.context.close(self.id);
        self
    }

    /// Queues terminal exit when this draw callback succeeds.
    pub fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }

    #[must_use]
    pub(crate) fn action(&self) -> &Action {
        self.context.action()
    }
}

impl<'a> Scope<'a> for Ready<'a> {
    fn id(&self) -> Id {
        Ready::id(self)
    }

    fn state(&self) -> &WindowSnapshot {
        Ready::state(self)
    }

    fn access(&self) -> Result<Ref<'_>> {
        Ready::access(self)
    }

    fn handle(&self) -> Result<Handle> {
        Ready::handle(self)
    }

    fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    fn target(&mut self) -> Target<'_> {
        self.window()
    }

    fn draw(&mut self) -> &mut Self {
        Ready::draw(self)
    }

    fn close(&mut self) -> &mut Self {
        Ready::close(self)
    }

    fn exit(&mut self) -> &mut Self {
        Ready::exit(self)
    }
}

impl<'a> Scope<'a> for Resize<'a> {
    fn id(&self) -> Id {
        Resize::id(self)
    }

    fn state(&self) -> &WindowSnapshot {
        Resize::state(self)
    }

    fn access(&self) -> Result<Ref<'_>> {
        Resize::access(self)
    }

    fn handle(&self) -> Result<Handle> {
        Resize::handle(self)
    }

    fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    fn target(&mut self) -> Target<'_> {
        self.window()
    }

    fn draw(&mut self) -> &mut Self {
        Resize::draw(self)
    }

    fn close(&mut self) -> &mut Self {
        Resize::close(self)
    }

    fn exit(&mut self) -> &mut Self {
        Resize::exit(self)
    }
}

impl<'a> Scope<'a> for Input<'a> {
    fn id(&self) -> Id {
        Input::id(self)
    }

    fn state(&self) -> &WindowSnapshot {
        Input::state(self)
    }

    fn access(&self) -> Result<Ref<'_>> {
        self.context.access(self.id())
    }

    fn handle(&self) -> Result<Handle> {
        self.context.handle(self.id())
    }

    fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    fn target(&mut self) -> Target<'_> {
        self.window()
    }

    fn draw(&mut self) -> &mut Self {
        Input::draw(self)
    }

    fn close(&mut self) -> &mut Self {
        Input::close(self)
    }

    fn exit(&mut self) -> &mut Self {
        Input::exit(self)
    }
}

impl<'a> Scope<'a> for Close<'a> {
    fn id(&self) -> Id {
        Close::id(self)
    }

    fn state(&self) -> &WindowSnapshot {
        Close::state(self)
    }

    fn access(&self) -> Result<Ref<'_>> {
        self.context.access(self.id)
    }

    fn handle(&self) -> Result<Handle> {
        self.context.handle(self.id)
    }

    fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    fn target(&mut self) -> Target<'_> {
        self.context.window(self.id)
    }

    fn draw(&mut self) -> &mut Self {
        self.context.draw(self.id);
        self
    }

    fn close(&mut self) -> &mut Self {
        Close::close(self)
    }

    fn exit(&mut self) -> &mut Self {
        self.context.request(Action::Exit);
        self
    }
}

impl<'a> Scope<'a> for Frame<'a> {
    fn id(&self) -> Id {
        Frame::id(self)
    }

    fn state(&self) -> &WindowSnapshot {
        Frame::state(self)
    }

    fn access(&self) -> Result<Ref<'_>> {
        Frame::access(self)
    }

    fn handle(&self) -> Result<Handle> {
        Frame::handle(self)
    }

    fn context_mut(&mut self) -> &mut Context<'a> {
        &mut self.context
    }

    fn target(&mut self) -> Target<'_> {
        self.window()
    }

    fn draw(&mut self) -> &mut Self {
        Frame::draw(self)
    }

    fn close(&mut self) -> &mut Self {
        Frame::close(self)
    }

    fn exit(&mut self) -> &mut Self {
        Frame::exit(self)
    }
}

/// Display-free application builder that owns a handler and authored startup opens.
///
/// [`App::into_loop`] retains authored startup requests without validation.
/// [`App::run`] or [`super::Loop::run`] intrinsically validates the complete batch
/// before native event-loop creation.
pub struct App<H> {
    window_loop: super::Loop<H>,
    startup: Vec<Open>,
}

#[must_use]
/// Starts an application builder around `handler`.
pub fn app<H>(handler: H) -> App<H> {
    App::new(handler)
}

impl<H> App<H> {
    #[must_use]
    /// Creates an application builder with no startup windows.
    pub fn new(handler: H) -> Self {
        Self {
            window_loop: super::Loop::new(handler),
            startup: Vec::new(),
        }
    }

    #[must_use]
    /// Adds an authored startup open to the complete pre-resume batch.
    pub fn open(mut self, open: Open) -> Self {
        self.startup.push(open);
        self
    }

    #[must_use]
    /// Replaces the loop-owned clipboard used by every callback.
    pub fn with_clipboard(mut self, clipboard: Box<dyn super::Clipboard>) -> Self {
        self.window_loop = self.window_loop.with_clipboard(clipboard);
        self
    }

    #[must_use]
    /// Returns the owned handler before the application is run or converted.
    pub fn handler(&self) -> &H {
        self.window_loop.handler()
    }

    /// Returns mutable access to the owned handler before running the application.
    pub fn handler_mut(&mut self) -> &mut H {
        self.window_loop.handler_mut()
    }

    #[must_use]
    /// Converts startup opens into commands retained by a display-free [`super::Loop`].
    ///
    /// Validation is deferred until the loop's first run or deterministic runner path.
    pub fn into_loop(mut self) -> super::Loop<H> {
        for open in self.startup {
            self.window_loop.startup.push(open.into_command());
        }
        self.window_loop
    }

    pub(crate) fn run_prepared_with(
        self,
        run: impl FnOnce(super::loop_::PreparedLoop<H>) -> Result<()>,
    ) -> Result<()> {
        self.into_loop().run_prepared_with(run)
    }
}

impl<H: super::Handler + 'static> App<H> {
    /// Validates startup requests and transfers ownership to the native event loop.
    ///
    /// This consumes the application and returns only after native event-loop exit;
    /// it does not promise deterministic native scheduling or an early return.
    pub fn run(self) -> Result<()> {
        self.run_prepared_with(super::loop_::PreparedLoop::run_native)
    }
}

impl Proxy {
    /// Enqueues a typed [`Command`] on this proxy's owning event loop.
    ///
    /// Intrinsically malformed commands are rejected before ingress. `Ok(())`
    /// reports only that the normalized command entered the owning loop's queue,
    /// not that planning or backend application will succeed. The deterministic
    /// [`crate::testing::Runner`] consumes accepted work FIFO through
    /// [`crate::testing::Runner::flush_proxy`]; the production event loop consumes
    /// it at native ingress. A target that is stale or closing at consumption is a
    /// no-op. Closed ingress returns [`crate::ErrorCode::CommandFailed`].
    pub fn send(&self, command: impl Into<Command>) -> Result<()> {
        self.command(command.into())
    }

    /// Enqueues `open` as a [`Command::Open`] on the owning event loop.
    ///
    /// [`Proxy::send`] performs intrinsic normalization in the proxy command
    /// path before ingress, so an invalid authored open can return `Err`
    /// immediately.
    /// `Ok(())` means only that the normalized open entered the owning loop's
    /// queue; lifecycle planning, capability validation, and backend application
    /// remain deferred and may fail. The runner consumes accepted payloads FIFO
    /// when [`crate::testing::Runner::flush_proxy`] is called, while production
    /// consumes them at native ingress. Queued target work that is stale or
    /// closing at consumption is a no-op. Closed ingress returns
    /// [`crate::ErrorCode::CommandFailed`].
    pub fn open(&self, open: Open) -> Result<()> {
        self.send(open)
    }

    /// Enqueues `Action::CloseRequested` for `id` on the owning event loop.
    ///
    /// `Ok(())` is queue acceptance only. Runner flushing is FIFO and production
    /// consumes the action at native ingress; a stale or closing target is a
    /// no-op when consumed. Closed ingress returns
    /// [`crate::ErrorCode::CommandFailed`].
    pub fn close(&self, id: Id) -> Result<()> {
        self.request(Action::CloseRequested(id))
    }

    /// Enqueues `Action::DrawNext` for `id` on the owning event loop.
    ///
    /// `Ok(())` is queue acceptance only. Runner flushing is FIFO and production
    /// consumes the action at native ingress; a stale or closing target is a
    /// no-op when consumed. Closed ingress returns
    /// [`crate::ErrorCode::CommandFailed`].
    pub fn draw(&self, id: Id) -> Result<()> {
        self.request(Action::DrawNext(id))
    }

    /// Enqueues `Action::DrawNow` for `id` on the owning event loop.
    ///
    /// `Ok(())` is queue acceptance only. Runner flushing is FIFO and production
    /// consumes the action at native ingress; a stale or closing target is a
    /// no-op when consumed. Closed ingress returns
    /// [`crate::ErrorCode::CommandFailed`].
    pub fn again(&self, id: Id) -> Result<()> {
        self.request(Action::DrawNow(id))
    }

    /// Enqueues `Action::DrawAt` for `id` and absolute `time` on the owning event loop.
    ///
    /// `Ok(())` is queue acceptance only. Runner flushing is FIFO and production
    /// consumes the action at native ingress; a stale or closing target is a
    /// no-op when consumed. Closed ingress returns
    /// [`crate::ErrorCode::CommandFailed`].
    pub fn at(&self, id: Id, time: Instant) -> Result<()> {
        self.request(Action::DrawAt { id, time })
    }

    pub(crate) fn request(&self, action: impl Into<Action>) -> Result<()> {
        self.request_action(action.into())
    }
}
